mod error;
mod resolver;

use error::OfrepError;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, StructValue};
use reqwest::header::HeaderMap;
use resolver::Resolver;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
use tracing::instrument;
use url::Url;

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

impl fmt::Debug for OfrepProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OfrepProvider")
            .field("provider", &"<FeatureProvider>")
            .finish()
    }
}

impl OfrepProvider {
    #[instrument(skip(options))]
    pub async fn new(options: OfrepOptions) -> Result<Self, OfrepError> {
        debug!("Initializing OfrepProvider with options: {:?}", options);

        let url = Url::parse(&options.base_url).map_err(|e| {
            OfrepError::Config(format!("Invalid base url: '{}' ({})", options.base_url, e))
        })?;

        if !matches!(url.scheme(), "http" | "https") {
            return Err(OfrepError::Config(format!(
                "Invalid base url: '{}' (unsupported scheme)",
                url.scheme()
            )));
        }

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
        self.provider.resolve_bool_value(flag_key, context).await
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        self.provider.resolve_int_value(flag_key, context).await
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        self.provider.resolve_float_value(flag_key, context).await
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        self.provider.resolve_string_value(flag_key, context).await
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        self.provider.resolve_struct_value(flag_key, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_ofrep_options_validation() {
        let provider_with_empty_host = OfrepProvider::new(OfrepOptions {
            base_url: "http://".to_string(),
            ..Default::default()
        })
        .await;

        let provider_with_invalid_scheme = OfrepProvider::new(OfrepOptions {
            base_url: "invalid://".to_string(),
            ..Default::default()
        })
        .await;

        assert!(provider_with_empty_host.is_err());
        assert!(provider_with_invalid_scheme.is_err());

        assert_eq!(
            provider_with_empty_host.unwrap_err(),
            OfrepError::Config("Invalid base url: 'http://' (empty host)".to_string())
        );
        assert_eq!(
            provider_with_invalid_scheme.unwrap_err(),
            OfrepError::Config("Invalid base url: 'invalid' (unsupported scheme)".to_string())
        );
    }
}
