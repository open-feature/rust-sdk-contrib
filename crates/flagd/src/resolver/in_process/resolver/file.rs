use crate::error::FlagdError;
use crate::resolver::in_process::model::value_converter::ValueConverter;
use crate::resolver::in_process::storage::connector::file::FileConnector;
use crate::resolver::in_process::storage::{FlagStore, StorageState, StorageStateChange};
use crate::resolver::in_process::targeting::Operator;
use crate::{CacheService, CacheSettings};
use async_trait::async_trait;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, EvaluationErrorCode, StructValue, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

pub struct FileResolver {
    store: Arc<FlagStore>,
    operator: Operator,
    metadata: ProviderMetadata,
    cache: Option<Arc<CacheService<Value>>>,
    state_receiver: Arc<Mutex<tokio::sync::mpsc::Receiver<StorageStateChange>>>,
}
impl FileResolver {
    pub async fn new(
        source_path: String,
        cache_settings: Option<CacheSettings>,
    ) -> Result<Self, FlagdError> {
        let connector = FileConnector::new(source_path);
        let (store, mut state_receiver) = FlagStore::new(Arc::new(connector));
        let store = Arc::new(store);

        store.init().await?;

        // Wait for initial state update with timeout
        if let Ok(Some(state_change)) =
            tokio::time::timeout(std::time::Duration::from_secs(5), state_receiver.recv()).await
        {
            if state_change.storage_state != StorageState::Ok {
                return Err(FlagdError::Sync(
                    "Failed to initialize flag store".to_string(),
                ));
            }
        } else {
            return Err(FlagdError::Timeout(
                "Timeout waiting for initial flag state".to_string(),
            ));
        }

        let cache = cache_settings.map(|settings| Arc::new(CacheService::new(settings)));

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

    async fn resolve_value<T>(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
        value_converter: impl Fn(&serde_json::Value) -> Option<T>,
        type_name: &str,
    ) -> Result<ResolutionDetails<T>, EvaluationError> {
        // Check for flag updates and clear cache if needed
        self.check_for_updates().await;

        if let Some(cache) = &self.cache
            && let Some(cached_value) = cache.get(flag_key, context).await
        {
            debug!("Cache hit for key: {}", flag_key);
            let json_value = cached_value.to_serde_json();
            if let Some(value) = value_converter(&json_value) {
                return Ok(ResolutionDetails::new(value));
            }
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

        if let Some(cache) = &self.cache {
            let cache_value = flag
                .variants
                .get(&variant)
                .map(|v| Value::String(v.to_string()));

            if let Some(v) = cache_value {
                let _ = cache.add(flag_key, context, v).await;
            }
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
impl FeatureProvider for FileResolver {
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

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        self.resolve_value(flag_key, context, |v| v.as_i64(), "integer")
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
                                serde_json::Value::String(s) => Value::String(s.clone()),
                                serde_json::Value::Number(n) => {
                                    if n.is_i64() {
                                        Value::Int(n.as_i64().unwrap())
                                    } else {
                                        Value::Float(n.as_f64().unwrap())
                                    }
                                }
                                serde_json::Value::Bool(b) => Value::Bool(*b),
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
