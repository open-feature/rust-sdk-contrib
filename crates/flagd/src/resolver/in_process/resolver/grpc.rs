use crate::error::FlagdError;
use crate::resolver::common::upstream::UpstreamConfig;
use crate::resolver::in_process::targeting::Operator;
use crate::{CacheService, FlagdOptions};
use async_trait::async_trait;
use open_feature::Value as OpenFeatureValue;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, EvaluationErrorCode, StructValue, Value};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::resolver::in_process::storage::connector::grpc::GrpcStreamConnector;
use crate::resolver::in_process::storage::{FlagStore, StorageState, StorageStateChange};

pub struct InProcessResolver {
    store: Arc<FlagStore>,
    operator: Operator,
    metadata: ProviderMetadata,
    cache: Option<Arc<CacheService<Value>>>,
    state_receiver: Arc<Mutex<tokio::sync::mpsc::Receiver<StorageStateChange>>>,
}

impl InProcessResolver {
    /// Gracefully shutdown the resolver and release resources
    pub async fn shutdown(&self) -> Result<(), FlagdError> {
        debug!("Shutting down InProcessResolver");
        self.store.shutdown().await?;
        Ok(())
    }
}

impl InProcessResolver {
    pub async fn new(options: &FlagdOptions) -> Result<Self, FlagdError> {
        let (store, state_receiver) = match &options.socket_path {
            Some(_) => Self::create_unix_socket_store(options).await?,
            None => Self::create_tcp_store(options).await?,
        };

        let cache = options
            .cache_settings
            .clone()
            .map(|settings| Arc::new(CacheService::new(settings)));

        Ok(Self {
            store,
            operator: Operator::new(),
            metadata: ProviderMetadata::new("flagd"),
            cache,
            state_receiver: Arc::new(Mutex::new(state_receiver)),
        })
    }

    /// Check for flag updates and clear cache if needed (non-blocking)
    async fn check_for_updates(&self) {
        if self.cache.is_none() {
            return;
        }

        let mut receiver = self.state_receiver.lock().await;

        // Drain all pending state changes (non-blocking)
        let mut should_clear = false;
        while let Ok(state_change) = receiver.try_recv() {
            if state_change.storage_state == StorageState::Ok {
                should_clear = true;
            }
        }

        if should_clear {
            debug!("Flag store updated, clearing cache");
            if let Some(cache) = &self.cache {
                cache.purge().await;
            }
        }
    }

    async fn create_unix_socket_store(
        options: &FlagdOptions,
    ) -> Result<
        (
            Arc<FlagStore>,
            tokio::sync::mpsc::Receiver<crate::resolver::in_process::storage::StorageStateChange>,
        ),
        FlagdError,
    > {
        let socket_path = options
            .socket_path
            .as_ref()
            .ok_or_else(|| FlagdError::Config("Unix socket path not provided".to_string()))?;

        debug!("Creating Unix socket store with path: {}", socket_path);

        // For Unix sockets, we use a special URI format
        let target = format!("unix://{}", socket_path);
        let connector = GrpcStreamConnector::new_unix(
            target,
            socket_path.clone(),
            options.selector.clone(),
            options,
        );

        let (store, state_receiver) = FlagStore::new(Arc::new(connector));
        let store = Arc::new(store);
        store.init().await?;
        Ok((store, state_receiver))
    }

    async fn create_tcp_store(
        options: &FlagdOptions,
    ) -> Result<
        (
            Arc<FlagStore>,
            tokio::sync::mpsc::Receiver<crate::resolver::in_process::storage::StorageStateChange>,
        ),
        FlagdError,
    > {
        let target = options
            .target_uri
            .clone()
            .unwrap_or_else(|| format!("{}:{}", options.host, options.port));
        let upstream_config =
            UpstreamConfig::new(target, true, options.tls, options.cert_path.as_deref())?;
        let connector = GrpcStreamConnector::new(
            upstream_config.endpoint().uri().to_string(),
            options.selector.clone(),
            options,
            upstream_config.authority(),
        );

        let (store, state_receiver) = FlagStore::new(Arc::new(connector));
        let store = Arc::new(store);
        store.init().await?;
        Ok((store, state_receiver))
    }

    async fn get_cached_value<T>(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
        value_converter: impl Fn(&OpenFeatureValue) -> Option<T>,
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
        value_converter: impl Fn(&JsonValue) -> Option<T>,
        type_name: &str,
    ) -> Result<ResolutionDetails<T>, EvaluationError> {
        // Check for flag updates and clear cache if needed
        self.check_for_updates().await;

        // Try cache first
        if let Some(cached_value) = self
            .get_cached_value(flag_key, context, |v| match v {
                OpenFeatureValue::String(s) => value_converter(&JsonValue::String(s.clone())),
                OpenFeatureValue::Bool(b) => value_converter(&JsonValue::Bool(*b)),
                OpenFeatureValue::Float(f) => value_converter(&JsonValue::Number(
                    serde_json::Number::from_f64(*f).unwrap(),
                )),
                OpenFeatureValue::Int(i) => value_converter(&JsonValue::Number((*i).into())),
                OpenFeatureValue::Struct(s) => {
                    // Convert OpenFeature struct to JsonValue object
                    let obj = convert_struct_to_json(s);
                    value_converter(&obj)
                }
                OpenFeatureValue::Array(arr) => {
                    // Convert OpenFeature array to JsonValue array
                    let json_array =
                        JsonValue::Array(arr.iter().map(convert_to_json_value).collect());
                    value_converter(&json_array)
                }
            })
            .await
        {
            return Ok(ResolutionDetails::new(cached_value));
        }

        let query_result = self.store.get_flag(flag_key).await;

        let flag = match query_result.feature_flag {
            Some(flag) => flag,
            None => {
                return Err(EvaluationError::builder()
                    .code(EvaluationErrorCode::FlagNotFound)
                    .message(format!("Flag {} not found", flag_key))
                    .build());
            }
        };

        if flag.state == "DISABLED" {
            return Err(EvaluationError::builder()
                .code(EvaluationErrorCode::FlagNotFound)
                .message(format!("Flag {} is disabled", flag_key))
                .build());
        }

        let variant = if flag.get_targeting() == "{}" {
            flag.default_variant
        } else {
            match self
                .operator
                .apply(flag_key, &flag.get_targeting(), context)
                .map_err(|e| {
                    EvaluationError::builder()
                        .code(EvaluationErrorCode::General(e.to_string()))
                        .message(e.to_string())
                        .build()
                })? {
                Some(variant) => variant,
                None => flag.default_variant,
            }
        };

        let value = flag
            .variants
            .get(&variant)
            .and_then(value_converter)
            .ok_or_else(|| {
                EvaluationError::builder()
                    .code(EvaluationErrorCode::TypeMismatch)
                    .message(format!(
                        "Value for flag {} is not a {}",
                        flag_key, type_name
                    ))
                    .build()
            })?;

        // Cache the result based on the type
        if let Some(cache) = &self.cache {
            let cache_value = match flag.variants.get(&variant) {
                Some(JsonValue::Bool(b)) => OpenFeatureValue::Bool(*b),
                Some(JsonValue::String(s)) => OpenFeatureValue::String(s.clone()),
                Some(JsonValue::Number(n)) => {
                    if n.is_i64() {
                        OpenFeatureValue::Int(n.as_i64().unwrap())
                    } else {
                        OpenFeatureValue::Float(n.as_f64().unwrap())
                    }
                }
                _ => {
                    return Ok(ResolutionDetails {
                        value,
                        variant: Some(variant),
                        reason: Some(open_feature::EvaluationReason::TargetingMatch),
                        flag_metadata: None,
                    });
                }
            };
            cache.add(flag_key, context, cache_value).await;
        }

        Ok(ResolutionDetails {
            value,
            variant: Some(variant),
            reason: Some(open_feature::EvaluationReason::TargetingMatch),
            flag_metadata: None,
        })
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
        self.resolve_value(flag_key, context, |v| v.as_bool(), "boolean")
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
            |v| v.as_str().map(String::from),
            "string",
        )
        .await
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        self.resolve_value(flag_key, context, |v| v.as_f64(), "float")
            .await
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        self.resolve_value(flag_key, context, |v| v.as_i64(), "integer")
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
            |v| {
                v.as_object().map(|obj| {
                    let fields = obj
                        .iter()
                        .map(|(k, v)| {
                            let value = match v {
                                JsonValue::String(s) => Value::String(s.clone()),
                                JsonValue::Number(n) => {
                                    if n.is_i64() {
                                        Value::Int(n.as_i64().unwrap())
                                    } else {
                                        Value::Float(n.as_f64().unwrap())
                                    }
                                }
                                JsonValue::Bool(b) => Value::Bool(*b),
                                _ => Value::String(v.to_string()),
                            };
                            (k.clone(), value)
                        })
                        .collect();
                    StructValue { fields }
                })
            },
            "struct",
        )
        .await
    }
}

fn convert_struct_to_json(struct_value: &StructValue) -> JsonValue {
    let mut map = serde_json::Map::new();
    for (key, value) in &struct_value.fields {
        map.insert(key.clone(), convert_to_json_value(value));
    }
    JsonValue::Object(map)
}

fn convert_to_json_value(value: &OpenFeatureValue) -> JsonValue {
    match value {
        OpenFeatureValue::String(s) => JsonValue::String(s.clone()),
        OpenFeatureValue::Bool(b) => JsonValue::Bool(*b),
        OpenFeatureValue::Int(i) => JsonValue::Number((*i).into()),
        OpenFeatureValue::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap()),
        OpenFeatureValue::Struct(s) => convert_struct_to_json(s),
        OpenFeatureValue::Array(arr) => {
            JsonValue::Array(arr.iter().map(convert_to_json_value).collect())
        }
    }
}
