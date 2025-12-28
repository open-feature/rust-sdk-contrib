use crate::resolver::in_process::resolver::common;
use crate::resolver::in_process::storage::connector::file::FileConnector;
use crate::resolver::in_process::storage::connector::{Connector, QueuePayloadType};
use crate::{CacheService, CacheSettings};
use anyhow::Result;
use async_trait::async_trait;
use flagd_evaluator::types::EvaluationResult;
use flagd_evaluator::model::ParsingResult;
use flagd_evaluator::ValidationMode;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, Value};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing::debug;

/// File-based resolver using the native flagd-evaluator
pub struct FileResolver {
    /// Connector for watching file changes
    connector: Arc<FileConnector>,
    /// Instance-based flag evaluator with its own state and validation mode
    evaluator: Arc<tokio::sync::RwLock<flagd_evaluator::FlagEvaluator>>,
    metadata: ProviderMetadata,
    cache: Option<Arc<CacheService<Value>>>,
}

impl FileResolver {
    pub async fn new(source_path: String, cache_settings: Option<CacheSettings>) -> Result<Self> {
        let connector = Arc::new(FileConnector::new(source_path));
        let cache = cache_settings.map(|settings| Arc::new(CacheService::new(settings)));

        // Create evaluator instance with permissive validation mode
        let evaluator = Arc::new(tokio::sync::RwLock::new(
            flagd_evaluator::FlagEvaluator::new(ValidationMode::Permissive)
        ));

        // Initialize the connector to start watching the file
        connector.init().await?;

        // Get the stream and wait for the initial sync
        let stream = connector.get_stream();
        let mut receiver_opt = stream.lock().await;

        // Wait for initial sync with timeout
        if let Some(receiver) = receiver_opt.as_mut() {
            match tokio::time::timeout(std::time::Duration::from_secs(5), receiver.recv()).await {
                Ok(Some(payload)) => {
                    if payload.payload_type == QueuePayloadType::Data {
                        debug!("Received initial flag configuration from file");
                        match ParsingResult::parse(&payload.flag_data) {
                            Ok(_) => {
                                let mut eval = evaluator.write().await;
                                if let Err(e) = eval.update_state(&payload.flag_data) {
                                    return Err(anyhow::anyhow!("Failed to update flag state: {}", e));
                                }
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Failed to parse initial flag configuration: {}", e));
                            }
                        }
                    } else {
                        return Err(anyhow::anyhow!("Error loading initial flag configuration"));
                    }
                }
                Ok(None) => {
                    return Err(anyhow::anyhow!("No initial flag configuration received"));
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
        let evaluator_clone = evaluator.clone();
        tokio::spawn(async move {
            let mut receiver_opt = stream_clone.lock().await;
            if let Some(receiver) = receiver_opt.as_mut() {
                while let Some(payload) = receiver.recv().await {
                    if payload.payload_type == QueuePayloadType::Data {
                        debug!("Received flag configuration update from file");

                        // Parse and update state in evaluator
                        match ParsingResult::parse(&payload.flag_data) {
                            Ok(_) => {
                                let mut eval = evaluator_clone.write().await;
                                if let Err(e) = eval.update_state(&payload.flag_data) {
                                    tracing::error!("Failed to update flag state: {}", e);
                                } else {
                                    // Clear cache when flags update
                                    if let Some(cache) = &cache_clone {
                                        cache.purge().await;
                                    }
                                }
                                drop(eval); // Explicitly drop lock before continuing
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
            evaluator,
            metadata: ProviderMetadata::new("flagd"),
            cache,
        })
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
        evaluator_fn: impl Fn(&flagd_evaluator::FlagEvaluator, &JsonValue) -> EvaluationResult,
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

        // Get read lock on evaluator and call evaluation function
        let eval = self.evaluator.read().await;
        let result = evaluator_fn(&*eval, &ctx_json);
        drop(eval); // Release lock before awaiting

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
impl FeatureProvider for FileResolver {
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
            |eval, ctx| eval.evaluate_bool(flag_key, ctx),
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
            |eval, ctx| eval.evaluate_string(flag_key, ctx),
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
            |eval, ctx| eval.evaluate_int(flag_key, ctx),
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
            |eval, ctx| eval.evaluate_float(flag_key, ctx),
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
            |eval, ctx| eval.evaluate_flag(flag_key, ctx),
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
