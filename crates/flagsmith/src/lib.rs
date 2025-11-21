//! Flagsmith Provider for OpenFeature
//!
//! A Rust implementation of the OpenFeature provider for Flagsmith, enabling dynamic
//! feature flag evaluation using the Flagsmith platform.
//!
//! # Overview
//!
//! This provider integrates the Flagsmith Rust SDK with OpenFeature, supporting both
//! environment-level and identity-specific flag evaluation.
//!
//! # Installation
//!
//! Add the dependency in your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! open-feature-flagsmith = "0.1"
//! open-feature = "0.2"
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//! use open_feature_flagsmith::{FlagsmithProvider, FlagsmithOptions};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create provider with environment key
//!     let provider = FlagsmithProvider::new(
//!         "your-environment-key".to_string(),
//!         FlagsmithOptions::default()
//!     ).await.unwrap();
//!
//!     // Environment-level evaluation (no targeting)
//!     let context = EvaluationContext::default();
//!     let result = provider.resolve_bool_value("my-feature", &context).await;
//!     println!("Feature enabled: {}", result.unwrap().value);
//!
//!     // Identity-specific evaluation (with targeting)
//!     let context = EvaluationContext::default()
//!         .with_targeting_key("user-123")
//!         .with_custom_field("email", "user@example.com")
//!         .with_custom_field("plan", "premium");
//!
//!     let result = provider.resolve_bool_value("my-feature", &context).await;
//!     println!("Feature for user: {}", result.unwrap().value);
//! }
//! ```

mod error;

use async_trait::async_trait;
use error::FlagsmithError;
use flagsmith::{Flagsmith, FlagsmithOptions as FlagsmithSDKOptions};
use flagsmith_flag_engine::types::{FlagsmithValue, FlagsmithValueType};
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{
    EvaluationContext, EvaluationContextFieldValue, EvaluationError, EvaluationReason as Reason,
    StructValue, Value,
};
use serde_json::Value as JsonValue;
use std::fmt;
use std::sync::Arc;
use tracing::{debug, instrument};

// Re-export for convenience
pub use error::FlagsmithError as Error;

/// Trait for Flagsmith client operations, enabling mockability in tests.
pub trait FlagsmithClient: Send + Sync {
    fn get_environment_flags(
        &self,
    ) -> Result<flagsmith::flagsmith::models::Flags, flagsmith::error::Error>;
    fn get_identity_flags(
        &self,
        identifier: &str,
        traits: Option<Vec<flagsmith::flagsmith::models::SDKTrait>>,
        transient: Option<bool>,
    ) -> Result<flagsmith::flagsmith::models::Flags, flagsmith::error::Error>;
}

impl FlagsmithClient for Flagsmith {
    fn get_environment_flags(
        &self,
    ) -> Result<flagsmith::flagsmith::models::Flags, flagsmith::error::Error> {
        self.get_environment_flags()
    }

    fn get_identity_flags(
        &self,
        identifier: &str,
        traits: Option<Vec<flagsmith::flagsmith::models::SDKTrait>>,
        transient: Option<bool>,
    ) -> Result<flagsmith::flagsmith::models::Flags, flagsmith::error::Error> {
        self.get_identity_flags(identifier, traits, transient)
    }
}

/// Configuration options for the Flagsmith provider.
#[derive(Debug, Clone, Default)]
pub struct FlagsmithOptions {
    /// Custom API URL (defaults to Flagsmith Edge API)
    pub api_url: Option<String>,

    /// Custom HTTP headers
    pub custom_headers: Option<reqwest::header::HeaderMap>,

    /// Request timeout in seconds
    pub request_timeout_seconds: Option<u64>,

    /// Enable local evaluation mode (requires server-side key)
    pub enable_local_evaluation: bool,

    /// Environment refresh interval in milliseconds (for local evaluation)
    pub environment_refresh_interval_mills: Option<u64>,

    /// Enable analytics tracking
    pub enable_analytics: bool,
}

impl FlagsmithOptions {
    /// Create a new FlagsmithOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom API URL
    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.api_url = Some(api_url);
        self
    }

    /// Enable local evaluation mode
    pub fn with_local_evaluation(mut self, enable: bool) -> Self {
        self.enable_local_evaluation = enable;
        self
    }

    /// Enable analytics tracking
    pub fn with_analytics(mut self, enable: bool) -> Self {
        self.enable_analytics = enable;
        self
    }

    /// Set request timeout in seconds
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.request_timeout_seconds = Some(seconds);
        self
    }
}

/// The Flagsmith OpenFeature provider.
///
/// This provider wraps the Flagsmith Rust SDK and implements the OpenFeature
/// `FeatureProvider` trait, enabling feature flag evaluation with OpenFeature.
pub struct FlagsmithProvider {
    metadata: ProviderMetadata,
    client: Arc<dyn FlagsmithClient>,
}

impl fmt::Debug for FlagsmithProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlagsmithProvider")
            .field("metadata", &self.metadata)
            .field("client", &"<Flagsmith>")
            .finish()
    }
}

impl FlagsmithProvider {
    /// Creates a new Flagsmith provider instance.
    ///
    /// # Arguments
    ///
    /// * `environment_key` - Your Flagsmith environment API key
    /// * `options` - Configuration options for the provider
    ///
    /// # Errors
    ///
    /// Returns `FlagsmithError::Config` if:
    /// - The environment key is empty
    /// - The API URL is invalid
    /// - Local evaluation is enabled without a server-side key
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use open_feature_flagsmith::{FlagsmithProvider, FlagsmithOptions};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let provider = FlagsmithProvider::new(
    ///         "your-environment-key".to_string(),
    ///         FlagsmithOptions::default()
    ///     ).await.unwrap();
    /// }
    /// ```
    #[instrument(skip(environment_key, options))]
    pub async fn new(
        environment_key: String,
        options: FlagsmithOptions,
    ) -> Result<Self, FlagsmithError> {
        debug!("Initializing FlagsmithProvider");

        // Validate environment key
        if environment_key.is_empty() {
            return Err(FlagsmithError::Config(
                "Environment key cannot be empty".to_string(),
            ));
        }

        // Validate local evaluation requirements
        if options.enable_local_evaluation && !environment_key.starts_with("ser.") {
            return Err(FlagsmithError::Config(
                "Local evaluation requires a server-side environment key (starts with 'ser.')"
                    .to_string(),
            ));
        }

        // Validate API URL if provided
        if let Some(ref url_str) = options.api_url {
            let parsed_url = url::Url::parse(url_str)?;
            if !matches!(parsed_url.scheme(), "http" | "https") {
                return Err(FlagsmithError::Config(format!(
                    "Invalid API URL scheme '{}'. Only http and https are supported",
                    parsed_url.scheme()
                )));
            }
        }

        // Build Flagsmith SDK options
        let mut sdk_options = FlagsmithSDKOptions::default();

        if let Some(api_url) = options.api_url {
            sdk_options.api_url = api_url;
        }

        if let Some(custom_headers) = options.custom_headers {
            sdk_options.custom_headers = custom_headers;
        }

        if let Some(timeout) = options.request_timeout_seconds {
            sdk_options.request_timeout_seconds = timeout;
        }

        sdk_options.enable_local_evaluation = options.enable_local_evaluation;
        sdk_options.enable_analytics = options.enable_analytics;

        if let Some(interval) = options.environment_refresh_interval_mills {
            sdk_options.environment_refresh_interval_mills = interval;
        }

        // Initialize Flagsmith client
        let client = Flagsmith::new(environment_key, sdk_options);

        Ok(Self::from_client(Arc::new(client)))
    }

    /// Creates a provider from an existing Flagsmith client.
    ///
    /// * `client` - An Arc-wrapped Flagsmith client instance
    pub fn from_client(client: Arc<dyn FlagsmithClient>) -> Self {
        Self {
            metadata: ProviderMetadata::new("flagsmith"),
            client,
        }
    }

    /// Fetches flags from the Flagsmith client.
    ///
    /// This helper function handles both environment-level and identity-specific flag fetching
    /// based on whether a targeting key is present in the evaluation context.
    ///
    /// # Arguments
    ///
    /// * `context` - The evaluation context containing targeting information
    ///
    /// # Returns
    ///
    /// Returns the flags object from Flagsmith, or an evaluation error if the operation fails.
    async fn get_flags(
        &self,
        context: &EvaluationContext,
    ) -> Result<flagsmith::flagsmith::models::Flags, EvaluationError> {
        let client = Arc::clone(&self.client);
        let targeting_key = context.targeting_key.clone();
        let traits = if targeting_key.is_some() {
            Some(context_to_traits(context))
        } else {
            None
        };

        Ok(tokio::task::spawn_blocking(move || {
            if let Some(key) = targeting_key {
                client.get_identity_flags(&key, traits, None)
            } else {
                client.get_environment_flags()
            }
        })
        .await
        .map_err(|e| EvaluationError {
            code: open_feature::EvaluationErrorCode::General("Task execution error".to_string()),
            message: Some(format!("Failed to execute blocking task: {}", e)),
        })?
        .map_err(FlagsmithError::from)?)
    }
}

#[async_trait]
impl FeatureProvider for FlagsmithProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    #[instrument(skip(self, context))]
    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        validate_flag_key(flag_key)?;
        debug!("Resolving boolean flag: {}", flag_key);

        let flags = self.get_flags(context).await?;

        let enabled = flags
            .is_feature_enabled(flag_key)
            .map_err(FlagsmithError::from)?;

        let reason = determine_reason(context, enabled);

        Ok(ResolutionDetails {
            value: enabled,
            reason: Some(reason),
            variant: None,
            flag_metadata: None,
        })
    }

    #[instrument(skip(self, context))]
    async fn resolve_string_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        validate_flag_key(flag_key)?;
        debug!("Resolving string flag: {}", flag_key);

        let flags = self.get_flags(context).await?;

        let flag = flags.get_flag(flag_key).map_err(FlagsmithError::from)?;

        if !matches!(flag.value.value_type, FlagsmithValueType::String) {
            return Err(EvaluationError {
                code: open_feature::EvaluationErrorCode::TypeMismatch,
                message: Some(format!(
                    "Expected string type, but flag '{}' has type {:?}",
                    flag_key, flag.value.value_type
                )),
            });
        }

        let value = flag.value.value.clone();
        let reason = determine_reason(context, flag.enabled);

        Ok(ResolutionDetails {
            value,
            reason: Some(reason),
            variant: None,
            flag_metadata: None,
        })
    }

    #[instrument(skip(self, context))]
    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        validate_flag_key(flag_key)?;
        debug!("Resolving integer flag: {}", flag_key);

        let flags = self.get_flags(context).await?;

        let flag = flags.get_flag(flag_key).map_err(FlagsmithError::from)?;

        let value = match flag.value.value_type {
            FlagsmithValueType::Integer => {
                flag.value
                    .value
                    .parse::<i64>()
                    .map_err(|e| EvaluationError {
                        code: open_feature::EvaluationErrorCode::TypeMismatch,
                        message: Some(format!(
                            "Failed to parse integer value '{}': {}",
                            flag.value.value, e
                        )),
                    })?
            }
            _ => {
                return Err(EvaluationError {
                    code: open_feature::EvaluationErrorCode::TypeMismatch,
                    message: Some(format!(
                        "Expected integer type, but got {:?}",
                        flag.value.value_type
                    )),
                });
            }
        };

        let reason = determine_reason(context, flag.enabled);

        Ok(ResolutionDetails {
            value,
            reason: Some(reason),
            variant: None,
            flag_metadata: None,
        })
    }

    #[instrument(skip(self, context))]
    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        validate_flag_key(flag_key)?;
        debug!("Resolving float flag: {}", flag_key);

        let flags = self.get_flags(context).await?;

        let flag = flags.get_flag(flag_key).map_err(FlagsmithError::from)?;

        let value = match flag.value.value_type {
            FlagsmithValueType::Float => {
                flag.value
                    .value
                    .parse::<f64>()
                    .map_err(|e| EvaluationError {
                        code: open_feature::EvaluationErrorCode::TypeMismatch,
                        message: Some(format!(
                            "Failed to parse float value '{}': {}",
                            flag.value.value, e
                        )),
                    })?
            }
            _ => {
                return Err(EvaluationError {
                    code: open_feature::EvaluationErrorCode::TypeMismatch,
                    message: Some(format!(
                        "Expected float type, but got {:?}",
                        flag.value.value_type
                    )),
                });
            }
        };

        let reason = determine_reason(context, flag.enabled);

        Ok(ResolutionDetails {
            value,
            reason: Some(reason),
            variant: None,
            flag_metadata: None,
        })
    }

    #[instrument(skip(self, context))]
    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        validate_flag_key(flag_key)?;
        debug!("Resolving struct flag: {}", flag_key);

        let flags = self.get_flags(context).await?;

        let flag = flags.get_flag(flag_key).map_err(FlagsmithError::from)?;

        let json_value: JsonValue =
            serde_json::from_str(&flag.value.value).map_err(|e| EvaluationError {
                code: open_feature::EvaluationErrorCode::ParseError,
                message: Some(format!("Failed to parse JSON: {}", e)),
            })?;

        let struct_value = match json_value {
            JsonValue::Object(map) => {
                let mut struct_map = std::collections::HashMap::new();
                for (key, json_val) in map {
                    let of_value = json_to_open_feature_value(json_val);
                    struct_map.insert(key, of_value);
                }
                StructValue { fields: struct_map }
            }
            _ => {
                return Err(EvaluationError {
                    code: open_feature::EvaluationErrorCode::TypeMismatch,
                    message: Some(format!(
                        "Expected JSON object, but got: {}",
                        flag.value.value
                    )),
                });
            }
        };

        let reason = determine_reason(context, flag.enabled);

        Ok(ResolutionDetails {
            value: struct_value,
            reason: Some(reason),
            variant: None,
            flag_metadata: None,
        })
    }
}

/// Convert serde_json::Value to open_feature::Value.
fn json_to_open_feature_value(json_val: JsonValue) -> Value {
    match json_val {
        JsonValue::Null => Value::String(String::new()),
        JsonValue::Bool(b) => Value::Bool(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        JsonValue::String(s) => Value::String(s),
        JsonValue::Array(arr) => {
            let values: Vec<Value> = arr.into_iter().map(json_to_open_feature_value).collect();
            Value::Array(values)
        }
        JsonValue::Object(map) => {
            let mut fields = std::collections::HashMap::new();
            for (k, v) in map {
                fields.insert(k, json_to_open_feature_value(v));
            }
            Value::Struct(StructValue { fields })
        }
    }
}

/// Validate that a flag key is not empty.
fn validate_flag_key(flag_key: &str) -> Result<(), EvaluationError> {
    if flag_key.is_empty() {
        return Err(EvaluationError {
            code: open_feature::EvaluationErrorCode::General("Invalid flag key".to_string()),
            message: Some("Flag key cannot be empty".to_string()),
        });
    }
    Ok(())
}

/// Convert OpenFeature EvaluationContext to Flagsmith traits.
///
/// Maps custom_fields from the context into Flagsmith trait format,
/// converting each field value to the appropriate Flagsmith type.
fn context_to_traits(context: &EvaluationContext) -> Vec<flagsmith::flagsmith::models::SDKTrait> {
    context
        .custom_fields
        .iter()
        .map(|(key, value)| {
            let flagsmith_value = match value {
                EvaluationContextFieldValue::Bool(b) => FlagsmithValue {
                    value: b.to_string(),
                    value_type: FlagsmithValueType::Bool,
                },
                EvaluationContextFieldValue::String(s) => FlagsmithValue {
                    value: s.clone(),
                    value_type: FlagsmithValueType::String,
                },
                EvaluationContextFieldValue::Int(i) => FlagsmithValue {
                    value: i.to_string(),
                    value_type: FlagsmithValueType::Integer,
                },
                EvaluationContextFieldValue::Float(f) => FlagsmithValue {
                    value: f.to_string(),
                    value_type: FlagsmithValueType::Float,
                },
                EvaluationContextFieldValue::DateTime(dt) => FlagsmithValue {
                    value: dt.to_string(),
                    value_type: FlagsmithValueType::String,
                },
                EvaluationContextFieldValue::Struct(_) => FlagsmithValue {
                    value: String::new(),
                    value_type: FlagsmithValueType::String,
                },
            };

            flagsmith::flagsmith::models::SDKTrait::new(key.clone(), flagsmith_value)
        })
        .collect()
}

/// Determine the OpenFeature reason based on the evaluation context and flag state.
///
/// Maps Flagsmith evaluation scenarios to OpenFeature reason codes:
/// - Identity evaluation (has targeting_key) � TargetingMatch
/// - Environment evaluation (no targeting_key) � Static
/// - Flag disabled � Disabled
fn determine_reason(context: &EvaluationContext, enabled: bool) -> Reason {
    if !enabled {
        Reason::Disabled
    } else if context.targeting_key.is_some() {
        Reason::TargetingMatch
    } else {
        Reason::Static
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_empty_environment_key_fails() {
        let result = FlagsmithProvider::new("".to_string(), FlagsmithOptions::default()).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            FlagsmithError::Config("Environment key cannot be empty".to_string())
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_local_evaluation_without_server_key_fails() {
        let result = FlagsmithProvider::new(
            "regular-key".to_string(),
            FlagsmithOptions::default().with_local_evaluation(true),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            FlagsmithError::Config(msg) => {
                assert!(msg.contains("server-side environment key"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_context_to_traits() {
        let context = EvaluationContext::default()
            .with_custom_field("email", "user@example.com")
            .with_custom_field("age", 25)
            .with_custom_field("premium", true)
            .with_custom_field("score", 98.5);

        let traits = context_to_traits(&context);

        assert_eq!(traits.len(), 4);

        // Check that all traits were created
        let trait_keys: Vec<String> = traits.iter().map(|t| t.trait_key.clone()).collect();
        assert!(trait_keys.contains(&"email".to_string()));
        assert!(trait_keys.contains(&"age".to_string()));
        assert!(trait_keys.contains(&"premium".to_string()));
        assert!(trait_keys.contains(&"score".to_string()));
    }

    #[test]
    fn test_determine_reason_disabled() {
        let context = EvaluationContext::default();
        let reason = determine_reason(&context, false);
        assert_eq!(reason, Reason::Disabled);
    }

    #[test]
    fn test_determine_reason_targeting_match() {
        let context = EvaluationContext::default().with_targeting_key("user-123");
        let reason = determine_reason(&context, true);
        assert_eq!(reason, Reason::TargetingMatch);
    }

    #[test]
    fn test_determine_reason_static() {
        let context = EvaluationContext::default();
        let reason = determine_reason(&context, true);
        assert_eq!(reason, Reason::Static);
    }

    #[test]
    fn test_metadata() {
        let provider = FlagsmithProvider {
            metadata: ProviderMetadata::new("flagsmith"),
            client: Arc::new(Flagsmith::new(
                "test-key".to_string(),
                FlagsmithSDKOptions::default(),
            )),
        };

        assert_eq!(provider.metadata().name, "flagsmith");
    }

    #[test]
    fn test_validate_flag_key_empty() {
        let result = validate_flag_key("");
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.message.unwrap().contains("empty"));
        }
    }

    #[test]
    fn test_validate_flag_key_valid() {
        let result = validate_flag_key("my-flag");
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_to_open_feature_value_primitives() {
        let json_null = serde_json::json!(null);
        let json_bool = serde_json::json!(true);
        let json_int = serde_json::json!(42);
        let json_float = serde_json::json!(3.14);
        let json_string = serde_json::json!("hello");

        assert!(matches!(
            json_to_open_feature_value(json_null),
            Value::String(_)
        ));
        assert!(matches!(
            json_to_open_feature_value(json_bool),
            Value::Bool(true)
        ));
        assert!(matches!(
            json_to_open_feature_value(json_int),
            Value::Int(42)
        ));

        if let Value::Float(f) = json_to_open_feature_value(json_float) {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected Float value");
        }

        if let Value::String(s) = json_to_open_feature_value(json_string) {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected String value");
        }
    }

    #[test]
    fn test_json_to_open_feature_value_array() {
        let json_array = serde_json::json!([1, 2, 3]);

        if let Value::Array(arr) = json_to_open_feature_value(json_array) {
            assert_eq!(arr.len(), 3);
            assert!(matches!(arr[0], Value::Int(1)));
            assert!(matches!(arr[1], Value::Int(2)));
            assert!(matches!(arr[2], Value::Int(3)));
        } else {
            panic!("Expected Array value");
        }
    }

    #[test]
    fn test_json_to_open_feature_value_object() {
        let json_object = serde_json::json!({
            "name": "test",
            "count": 10,
            "active": true
        });

        if let Value::Struct(s) = json_to_open_feature_value(json_object) {
            assert_eq!(s.fields.len(), 3);
            assert!(s.fields.contains_key("name"));
            assert!(s.fields.contains_key("count"));
            assert!(s.fields.contains_key("active"));
        } else {
            panic!("Expected Struct value");
        }
    }

    #[test]
    fn test_json_to_open_feature_value_nested() {
        let json_nested = serde_json::json!({
            "user": {
                "name": "Alice",
                "age": 30
            },
            "tags": ["admin", "user"]
        });

        if let Value::Struct(s) = json_to_open_feature_value(json_nested) {
            assert_eq!(s.fields.len(), 2);

            if let Some(Value::Struct(user)) = s.fields.get("user") {
                assert_eq!(user.fields.len(), 2);
            } else {
                panic!("Expected nested struct for user");
            }

            if let Some(Value::Array(tags)) = s.fields.get("tags") {
                assert_eq!(tags.len(), 2);
            } else {
                panic!("Expected array for tags");
            }
        } else {
            panic!("Expected Struct value");
        }
    }
}
