use crate::resolver::common::upstream::UpstreamConfig;
use crate::resolver::in_process::resolver::common;
use crate::{CacheService, FlagdOptions};
use anyhow::Result;
use async_trait::async_trait;
use flagd_evaluator::evaluation::{
    evaluate_bool_flag, evaluate_float_flag, evaluate_flag, evaluate_int_flag,
    evaluate_string_flag, EvaluationResult,
};
use flagd_evaluator::model::ParsingResult;
use flagd_evaluator::storage::{update_flag_state, ValidationMode};
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, Value};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing::debug;

use crate::resolver::in_process::storage::connector::grpc::GrpcStreamConnector;
use crate::resolver::in_process::storage::connector::{Connector, QueuePayloadType};

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

    async fn resolve_value<T>(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
        evaluator_fn: impl Fn(&serde_json::Map<String, JsonValue>) -> EvaluationResult,
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
        let ctx_json = common::build_context_json(context);
        let ctx_map = ctx_json.as_object().unwrap_or_else(|| {
            panic!("build_context_json should always return an object")
        });

        // Call evaluator (no clone needed)
        let result = evaluator_fn(ctx_map);

        // Convert result to details
        let details = common::result_to_details(&result, value_extractor)?;

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
            |ctx| {
                let (flag, metadata) = common::get_flag_and_metadata(flag_key);
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
            |ctx| {
                let (flag, metadata) = common::get_flag_and_metadata(flag_key);
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
            |ctx| {
                let (flag, metadata) = common::get_flag_and_metadata(flag_key);
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
            |ctx| {
                let (flag, metadata) = common::get_flag_and_metadata(flag_key);
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
    ) -> Result<ResolutionDetails<open_feature::StructValue>, EvaluationError> {
        self.resolve_value(
            flag_key,
            context,
            |ctx| {
                let (flag, metadata) = common::get_flag_and_metadata(flag_key);
                evaluate_flag(&flag, &JsonValue::Object(ctx.clone()), &metadata)
            },
            |v| {
                v.as_object().map(|obj| {
                    let fields = obj
                        .iter()
                        .map(|(k, v)| (k.clone(), common::json_to_value(v)))
                        .collect();
                    open_feature::StructValue { fields }
                })
            },
            |s| Value::Struct(s),
        )
        .await
    }
}
