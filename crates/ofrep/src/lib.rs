mod error;
mod resolver;

use error::OfrepError;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, StructValue};
use reqwest::header::HeaderMap;
use resolver::Resolver;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
use tracing::instrument;

use async_trait::async_trait;

const DEFAULT_BASE_URL: &str = "http://localhost:8016";
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct OfrepOptions {
    pub base_url: String,
    pub headers: HeaderMap,
    pub connect_timeout: Duration,
}

impl Default for OfrepOptions {
    fn default() -> Self {
        OfrepOptions {
            base_url: DEFAULT_BASE_URL.to_string(),
            headers: HeaderMap::new(),
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
        }
    }
}

pub struct OfrepProvider {
    provider: Arc<dyn FeatureProvider + Send + Sync>,
}

impl OfrepProvider {
    #[instrument(skip(options))]
    pub async fn new(options: OfrepOptions) -> Result<Self, OfrepError> {
        debug!("Initializing OfrepProvider with options: {:?}", options);
        Ok(Self {
            provider: Arc::new(Resolver::new(&options)),
        })
    }
}

#[async_trait]
impl FeatureProvider for OfrepProvider {
    fn metadata(&self) -> &ProviderMetadata {
        self.provider.metadata()
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        let result = self.provider.resolve_bool_value(flag_key, context).await?;
        Ok(result)
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        let result = self.provider.resolve_int_value(flag_key, context).await?;
        Ok(result)
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        let result = self.provider.resolve_float_value(flag_key, context).await?;
        Ok(result)
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        let result = self
            .provider
            .resolve_string_value(flag_key, context)
            .await?;
        Ok(result)
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        let result = self
            .provider
            .resolve_struct_value(flag_key, context)
            .await?;
        Ok(result)
    }
}
