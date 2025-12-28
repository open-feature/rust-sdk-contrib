use crate::resolver::common::upstream::UpstreamConfig;
use crate::{CacheService, FlagdOptions};
use anyhow::Result;
use async_trait::async_trait;
use flagd_evaluator::evaluation::{
    evaluate_bool_flag, evaluate_float_flag, evaluate_flag, evaluate_int_flag,
    evaluate_string_flag, ErrorCode as EvaluatorErrorCode, EvaluationResult,
    ResolutionReason as EvaluatorReason,
};
use flagd_evaluator::model::ParsingResult;
use flagd_evaluator::storage::{update_flag_state, ValidationMode};
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationReason, FlagMetadata,
    FlagMetadataValue, StructValue, Value,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::resolver::in_process::storage::connector::grpc::GrpcStreamConnector;
use crate::resolver::in_process::storage::connector::{Connector, QueuePayloadType};
use flagd_evaluator::model::FeatureFlag;

/// Helper to create an empty FeatureFlag for a given key when one doesn't exist
fn empty_flag(key: &str) -> FeatureFlag {
    FeatureFlag {
        key: Some(key.to_string()),
        state: "DISABLED".to_string(),
        default_variant: None,
        variants: Default::default(),
        targeting: None,
        metadata: Default::default(),
    }
}

/// In-process resolver using the native flagd-evaluator
pub struct InProcessResolver {
    /// Connector for syncing flag configuration from gRPC
    connector: Arc<GrpcStreamConnector>,
    metadata: ProviderMetadata,
    cache: Option<Arc<CacheService<Value>>>,
}

impl InProcessResolver {
    pub async fn new(options: &FlagdOptions) -> Result<Self> {
        // Set validation mode to permissive to match other providers
        flagd_evaluator::storage::set_validation_mode(ValidationMode::Permissive);

        let connector = match &options.socket_path {
            Some(_) => {
                return Err(anyhow::anyhow!(
                    "Unix socket support for in-process is not implemented"
                ));
            }
            None => Self::create_tcp_connector(options).await?,
        };

        let cache = options
            .cache_settings
            .clone()
            .map(|settings| Arc::new(CacheService::new(settings)));

        // Initialize the connector to start syncing
        connector.init().await?;

        // Get the stream and wait for the initial sync
        let stream = connector.get_stream();
        let mut receiver_opt = stream.lock().await;

        // Wait for initial sync with timeout
        if let Some(receiver) = receiver_opt.as_mut() {
            match tokio::time::timeout(std::time::Duration::from_secs(5), receiver.recv()).await {
                Ok(Some(payload)) => {
                    if payload.payload_type == QueuePayloadType::Data {
                        debug!("Received initial flag configuration");
                        match ParsingResult::parse(&payload.flag_data) {
                            Ok(_) => {
                                if let Err(e) = update_flag_state(&payload.flag_data) {
                                    return Err(anyhow::anyhow!("Failed to update flag state: {}", e));
                                }
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Failed to parse initial flag configuration: {}", e));
                            }
                        }
                    }
                }
                Ok(None) => {
                    return Err(anyhow::anyhow!("No initial sync message received"));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("Timeout waiting for initial flag state"));
                }
            }
        }
        drop(receiver_opt); // Release the lock before spawning

        // Spawn task to handle subsequent config updates
        let stream_clone = stream.clone();
        let cache_clone = cache.clone();
        tokio::spawn(async move {
            let mut receiver_opt = stream_clone.lock().await;
            if let Some(receiver) = receiver_opt.as_mut() {
                while let Some(payload) = receiver.recv().await {
                    if payload.payload_type == QueuePayloadType::Data {
                        debug!("Received flag configuration update");

                        // Parse and update state in evaluator
                        match ParsingResult::parse(&payload.flag_data) {
                            Ok(_) => {
                                if let Err(e) = update_flag_state(&payload.flag_data) {
                                    tracing::error!("Failed to update flag state: {}", e);
                                } else {
                                    // Clear cache when flags update
                                    if let Some(cache) = &cache_clone {
                                        cache.purge().await;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse flag configuration: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            connector,
            metadata: ProviderMetadata::new("flagd"),
            cache,
        })
    }

    async fn create_tcp_connector(options: &FlagdOptions) -> Result<Arc<GrpcStreamConnector>> {
        let target = options
            .target_uri
            .clone()
            .unwrap_or_else(|| format!("{}:{}", options.host, options.port));
        let upstream_config = UpstreamConfig::new(target, true)?;
        let connector = GrpcStreamConnector::new(
            upstream_config.endpoint().uri().to_string(),
            options.selector.clone(),
            options,
            upstream_config.authority().to_string(),
        );

        Ok(Arc::new(connector))
    }

    async fn get_cached_value<T>(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
        value_converter: impl Fn(&Value) -> Option<T>,
    ) -> Option<T> {
        if let Some(cache) = &self.cache
            && let Some(cached_value) = cache.get(flag_key, context).await
        {
            return value_converter(&cached_value);
        }
        None
    }

    /// Build context JSON for evaluator from OpenFeature context
    fn build_context_json(context: &EvaluationContext) -> JsonValue {
        let mut root = serde_json::Map::new();

        // Add targeting key if present
        if let Some(targeting_key) = &context.targeting_key {
            root.insert("targetingKey".to_string(), JsonValue::String(targeting_key.clone()));
        }

        // Add custom fields
        for (key, value) in &context.custom_fields {
            use open_feature::EvaluationContextFieldValue;
            let json_value = match value {
                EvaluationContextFieldValue::String(s) => JsonValue::String(s.clone()),
                EvaluationContextFieldValue::Bool(b) => JsonValue::Bool(*b),
                EvaluationContextFieldValue::Int(i) => JsonValue::Number((*i).into()),
                EvaluationContextFieldValue::Float(f) => {
                    JsonValue::Number(serde_json::Number::from_f64(*f).unwrap())
                }
                EvaluationContextFieldValue::DateTime(dt) => {
                    JsonValue::String(dt.to_string())
                }
                EvaluationContextFieldValue::Struct(_) => {
                    // For now, convert struct to string
                    JsonValue::String(format!("{:?}", value))
                }
            };
            root.insert(key.clone(), json_value);
        }

        JsonValue::Object(root)
    }

    /// Map evaluator reason to OpenFeature reason
    fn map_reason(reason: &EvaluatorReason) -> Option<EvaluationReason> {
        match reason {
            EvaluatorReason::Static => Some(EvaluationReason::Static),
            EvaluatorReason::Default => Some(EvaluationReason::Default),
            EvaluatorReason::TargetingMatch => Some(EvaluationReason::TargetingMatch),
            EvaluatorReason::Disabled => Some(EvaluationReason::Disabled),
            EvaluatorReason::Error | EvaluatorReason::FlagNotFound | EvaluatorReason::Fallback => {
                Some(EvaluationReason::Error)
            }
        }
    }

    /// Map evaluator error code to OpenFeature error code
    fn map_error_code(code: &EvaluatorErrorCode) -> EvaluationErrorCode {
        match code {
            EvaluatorErrorCode::FlagNotFound => EvaluationErrorCode::FlagNotFound,
            EvaluatorErrorCode::ParseError => EvaluationErrorCode::ParseError,
            EvaluatorErrorCode::TypeMismatch => EvaluationErrorCode::TypeMismatch,
            EvaluatorErrorCode::General => {
                EvaluationErrorCode::General("Evaluation error".to_string())
            }
        }
    }

    /// Convert evaluation result to resolution details
    fn result_to_details<T>(
        result: &EvaluationResult,
        value_extractor: impl Fn(&JsonValue) -> Option<T>,
    ) -> Result<ResolutionDetails<T>, EvaluationError> {
        // Check for errors
        if let Some(error_code) = &result.error_code {
            return Err(EvaluationError::builder()
                .code(Self::map_error_code(error_code))
                .message(result.error_message.clone().unwrap_or_default())
                .build());
        }

        // Extract value
        let value = value_extractor(&result.value).ok_or_else(|| {
            EvaluationError::builder()
                .code(EvaluationErrorCode::TypeMismatch)
                .message("Value type mismatch".to_string())
                .build()
        })?;

        Ok(ResolutionDetails {
            value,
            variant: result.variant.clone(),
            reason: Self::map_reason(&result.reason),
            flag_metadata: result.flag_metadata.as_ref().map(|metadata| {
                let mut flag_metadata = FlagMetadata::default();
                for (key, value) in metadata {
                    if let Some(metadata_value) = json_to_metadata_value(value) {
                        flag_metadata = flag_metadata.with_value(key.clone(), metadata_value);
                    }
                }
                flag_metadata
            }),
        })
    }

    async fn resolve_value<T>(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
        evaluator_fn: impl Fn(&JsonValue, &serde_json::Map<String, JsonValue>) -> EvaluationResult,
        value_extractor: impl Fn(&JsonValue) -> Option<T>,
        cache_value_fn: impl Fn(T) -> Value,
    ) -> Result<ResolutionDetails<T>, EvaluationError>
    where
        T: Clone,
    {
        // Try cache first
        if let Some(cached_value) = self
            .get_cached_value(flag_key, context, |v| match v {
                Value::String(s) => value_extractor(&JsonValue::String(s.clone())),
                Value::Bool(b) => value_extractor(&JsonValue::Bool(*b)),
                Value::Int(i) => value_extractor(&JsonValue::Number((*i).into())),
                Value::Float(f) => {
                    value_extractor(&JsonValue::Number(serde_json::Number::from_f64(*f).unwrap()))
                }
                _ => None,
            })
            .await
        {
            return Ok(ResolutionDetails::new(cached_value));
        }

        // Build context for evaluator
        let ctx_json = Self::build_context_json(context);
        let ctx_map = ctx_json.as_object().cloned().unwrap_or_default();

        // Call evaluator
        let result = evaluator_fn(&ctx_json, &ctx_map);

        // Convert result to details
        let details = Self::result_to_details(&result, value_extractor)?;

        // Cache the result
        if let Some(cache) = &self.cache {
            cache
                .add(flag_key, context, cache_value_fn(details.value.clone()))
                .await;
        }

        Ok(details)
    }
}

#[async_trait]
impl FeatureProvider for InProcessResolver {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        self.resolve_value(
            flag_key,
            context,
            |_, ctx| {
                let state = flagd_evaluator::storage::get_flag_state();
                let flag = state
                    .as_ref()
                    .and_then(|s| s.flags.get(flag_key))
                    .cloned()
                    .unwrap_or_else(|| empty_flag(flag_key));
                let metadata = state
                    .as_ref()
                    .map(|s| &s.flag_set_metadata)
                    .cloned()
                    .unwrap_or_default();
                evaluate_bool_flag(&flag, &JsonValue::Object(ctx.clone()), &metadata)
            },
            |v| v.as_bool(),
            Value::Bool,
        )
        .await
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        self.resolve_value(
            flag_key,
            context,
            |_, ctx| {
                let state = flagd_evaluator::storage::get_flag_state();
                let flag = state
                    .as_ref()
                    .and_then(|s| s.flags.get(flag_key))
                    .cloned()
                    .unwrap_or_else(|| empty_flag(flag_key));
                let metadata = state
                    .as_ref()
                    .map(|s| &s.flag_set_metadata)
                    .cloned()
                    .unwrap_or_default();
                evaluate_string_flag(&flag, &JsonValue::Object(ctx.clone()), &metadata)
            },
            |v| v.as_str().map(String::from),
            Value::String,
        )
        .await
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        self.resolve_value(
            flag_key,
            context,
            |_, ctx| {
                let state = flagd_evaluator::storage::get_flag_state();
                let flag = state
                    .as_ref()
                    .and_then(|s| s.flags.get(flag_key))
                    .cloned()
                    .unwrap_or_else(|| empty_flag(flag_key));
                let metadata = state
                    .as_ref()
                    .map(|s| &s.flag_set_metadata)
                    .cloned()
                    .unwrap_or_default();
                evaluate_int_flag(&flag, &JsonValue::Object(ctx.clone()), &metadata)
            },
            |v| v.as_i64(),
            Value::Int,
        )
        .await
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        self.resolve_value(
            flag_key,
            context,
            |_, ctx| {
                let state = flagd_evaluator::storage::get_flag_state();
                let flag = state
                    .as_ref()
                    .and_then(|s| s.flags.get(flag_key))
                    .cloned()
                    .unwrap_or_else(|| empty_flag(flag_key));
                let metadata = state
                    .as_ref()
                    .map(|s| &s.flag_set_metadata)
                    .cloned()
                    .unwrap_or_default();
                evaluate_float_flag(&flag, &JsonValue::Object(ctx.clone()), &metadata)
            },
            |v| v.as_f64(),
            Value::Float,
        )
        .await
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        self.resolve_value(
            flag_key,
            context,
            |_, ctx| {
                let state = flagd_evaluator::storage::get_flag_state();
                let flag = state
                    .as_ref()
                    .and_then(|s| s.flags.get(flag_key))
                    .cloned()
                    .unwrap_or_else(|| empty_flag(flag_key));
                let metadata = state
                    .as_ref()
                    .map(|s| &s.flag_set_metadata)
                    .cloned()
                    .unwrap_or_default();
                evaluate_flag(&flag, &JsonValue::Object(ctx.clone()), &metadata)
            },
            |v| {
                v.as_object().map(|obj| {
                    let fields = obj
                        .iter()
                        .map(|(k, v)| (k.clone(), json_to_value(v)))
                        .collect();
                    StructValue { fields }
                })
            },
            |s| Value::Struct(s),
        )
        .await
    }
}

/// Convert JsonValue to OpenFeature Value
fn json_to_value(v: &JsonValue) -> Value {
    match v {
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap())
            }
        }
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Object(obj) => {
            let fields = obj.iter().map(|(k, v)| (k.clone(), json_to_value(v))).collect();
            Value::Struct(StructValue { fields })
        }
        JsonValue::Array(arr) => Value::Array(arr.iter().map(json_to_value).collect()),
        JsonValue::Null => Value::String(String::new()), // Default for null
    }
}

/// Convert JsonValue to FlagMetadataValue
fn json_to_metadata_value(v: &JsonValue) -> Option<FlagMetadataValue> {
    match v {
        JsonValue::String(s) => Some(FlagMetadataValue::String(s.clone())),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(FlagMetadataValue::Int(i))
            } else {
                n.as_f64().map(FlagMetadataValue::Float)
            }
        }
        JsonValue::Bool(b) => Some(FlagMetadataValue::Bool(*b)),
        _ => None, // FlagMetadata only supports primitives
    }
}
