//! # REST Flag Resolver
//!
//! Evaluates feature flags using the OpenFeature Remote Evaluation Protocol (OFREP) over HTTP.
//! This module wraps the `open-feature-ofrep` crate.
//!
//! ## Example
//!
//! ```rust,no_run
//! use open_feature_flagd::resolver::rest::RestResolver;
//! use open_feature_flagd::FlagdOptions;
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//!
//! #[tokio::main]
//! async fn main() {
//!     let options = FlagdOptions {
//!         host: "localhost".to_string(),
//!         port: 8016,
//!         ..Default::default()
//!     };
//!     let resolver = RestResolver::new(&options).await.unwrap();
//!     let context = EvaluationContext::default();
//!
//!     let result = resolver.resolve_bool_value("my-flag", &context).await.unwrap();
//!     println!("Flag value: {}", result.value);
//! }
//! ```

use async_trait::async_trait;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, StructValue};
use open_feature_ofrep::{OfrepOptions, OfrepProvider};
use std::time::Duration;
use tracing::instrument;

use crate::FlagdOptions;
use crate::error::FlagdError;

/// REST-based resolver implementing the OpenFeature Remote Evaluation Protocol (OFREP).
/// This is a wrapper around the `open-feature-ofrep` crate.
#[derive(Debug)]
pub struct RestResolver {
    provider: OfrepProvider,
}

impl RestResolver {
    /// Creates a new REST resolver with the specified options.
    ///
    /// # Arguments
    ///
    /// * `options` - Configuration options including host, port, TLS settings, and certificate path
    ///
    /// # Returns
    ///
    /// A `Result` containing the new RestResolver instance or an error if initialization fails
    pub async fn new(options: &FlagdOptions) -> Result<Self, FlagdError> {
        let scheme = if options.tls { "https" } else { "http" };
        let base_url = if let Some(uri) = &options.target_uri {
            // Check if URI already has a scheme
            if uri.starts_with("http://") || uri.starts_with("https://") {
                uri.clone()
            } else {
                format!("{}://{}", scheme, uri)
            }
        } else {
            format!("{}://{}:{}", scheme, options.host, options.port)
        };

        let connect_timeout = Duration::from_millis(options.deadline_ms as u64);

        let ofrep_options = OfrepOptions {
            base_url,
            headers: Default::default(),
            connect_timeout,
            cert_path: options.cert_path.clone(),
        };

        OfrepProvider::new(ofrep_options)
            .await
            .map(|provider| Self { provider })
            .map_err(|e| FlagdError::Config(e.to_string()))
    }
}

#[async_trait]
impl FeatureProvider for RestResolver {
    fn metadata(&self) -> &ProviderMetadata {
        self.provider.metadata()
    }

    /// Resolves a boolean flag value
    #[instrument(skip(self, evaluation_context), fields(flag_key = %flag_key))]
    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        self.provider
            .resolve_bool_value(flag_key, evaluation_context)
            .await
    }

    /// Resolves a string flag value
    #[instrument(skip(self, evaluation_context), fields(flag_key = %flag_key))]
    async fn resolve_string_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        self.provider
            .resolve_string_value(flag_key, evaluation_context)
            .await
    }

    /// Resolves a float flag value
    #[instrument(skip(self, evaluation_context), fields(flag_key = %flag_key))]
    async fn resolve_float_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        self.provider
            .resolve_float_value(flag_key, evaluation_context)
            .await
    }

    /// Resolves an integer flag value
    #[instrument(skip(self, evaluation_context), fields(flag_key = %flag_key))]
    async fn resolve_int_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        self.provider
            .resolve_int_value(flag_key, evaluation_context)
            .await
    }

    /// Resolves a structured flag value
    #[instrument(skip(self, evaluation_context), fields(flag_key = %flag_key))]
    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        self.provider
            .resolve_struct_value(flag_key, evaluation_context)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_feature::EvaluationContext;
    use serde_json::json;
    use test_log::test;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_mock_server() -> (MockServer, RestResolver) {
        let mock_server = MockServer::start().await;
        let options = FlagdOptions {
            host: mock_server.address().ip().to_string(),
            port: mock_server.address().port(),
            target_uri: None,
            deadline_ms: 5000,
            ..Default::default()
        };
        let resolver = RestResolver::new(&options)
            .await
            .expect("Failed to create RestResolver");
        (mock_server, resolver)
    }

    #[test(tokio::test)]
    async fn test_resolve_bool_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": true,
                "variant": "on",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_bool_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, true);
        assert_eq!(result.variant, Some("on".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    async fn test_resolve_string_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": "test-value",
                "variant": "key1",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_string_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, "test-value");
        assert_eq!(result.variant, Some("key1".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    async fn test_resolve_float_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": 1.23,
                "variant": "one",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_float_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, 1.23);
        assert_eq!(result.variant, Some("one".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    async fn test_resolve_int_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": 42,
                "variant": "one",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_int_value("test-flag", &context)
            .await
            .unwrap();

        assert_eq!(result.value, 42);
        assert_eq!(result.variant, Some("one".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }

    #[test(tokio::test)]
    async fn test_resolve_struct_value() {
        let (mock_server, resolver) = setup_mock_server().await;

        Mock::given(method("POST"))
            .and(path("/ofrep/v1/evaluate/flags/test-flag"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": {
                    "key": "val",
                    "number": 42,
                    "boolean": true,
                    "nested": {
                        "inner": "value"
                    }
                },
                "variant": "object1",
                "reason": "STATIC"
            })))
            .mount(&mock_server)
            .await;

        let context = EvaluationContext::default().with_targeting_key("test-user");
        let result = resolver
            .resolve_struct_value("test-flag", &context)
            .await
            .unwrap();

        let value = &result.value;
        assert_eq!(value.fields.get("key").unwrap().as_str().unwrap(), "val");
        assert_eq!(value.fields.get("number").unwrap().as_i64().unwrap(), 42);
        assert_eq!(
            value.fields.get("boolean").unwrap().as_bool().unwrap(),
            true
        );

        let nested = value.fields.get("nested").unwrap().as_struct().unwrap();
        assert_eq!(
            nested.fields.get("inner").unwrap().as_str().unwrap(),
            "value"
        );

        assert_eq!(result.variant, Some("object1".to_string()));
        assert_eq!(result.reason, Some(open_feature::EvaluationReason::Static));
    }
}
