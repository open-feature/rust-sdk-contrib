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

pub mod error;

use async_trait::async_trait;
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
pub use error::FlagsmithError;
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
        // Use spawn_blocking because Flagsmith::new() creates threads and can conflict with tokio runtime
        let client =
            tokio::task::spawn_blocking(move || Flagsmith::new(environment_key, sdk_options))
                .await
                .map_err(|e| {
                    error::FlagsmithError::Config(format!(
                        "Failed to initialize Flagsmith client: {}",
                        e
                    ))
                })?;

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

        // Since all Flagsmith values are stored as strings internally, we can always
        // return the value as a string regardless of the declared type
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

        // Flagsmith stores all values as strings, so we try to parse regardless of value_type
        // First check the declared type, then fall back to attempting string parsing
        let value = match flag.value.value_type {
            FlagsmithValueType::Integer | FlagsmithValueType::String => flag
                .value
                .value
                .parse::<i64>()
                .map_err(|e| EvaluationError {
                    code: open_feature::EvaluationErrorCode::TypeMismatch,
                    message: Some(format!(
                        "Failed to parse integer value '{}': {}",
                        flag.value.value, e
                    )),
                })?,
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

        // Flagsmith stores all values as strings, so we try to parse regardless of value_type
        // First check the declared type, then fall back to attempting string parsing
        let value = match flag.value.value_type {
            FlagsmithValueType::Float | FlagsmithValueType::String => flag
                .value
                .value
                .parse::<f64>()
                .map_err(|e| EvaluationError {
                    code: open_feature::EvaluationErrorCode::TypeMismatch,
                    message: Some(format!(
                        "Failed to parse float value '{}': {}",
                        flag.value.value, e
                    )),
                })?,
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

        if !matches!(flag.value.value_type, FlagsmithValueType::String) {
            return Err(EvaluationError {
                code: open_feature::EvaluationErrorCode::TypeMismatch,
                message: Some(format!(
                    "Expected string type for JSON, but flag '{}' has type {:?}",
                    flag_key, flag.value.value_type
                )),
            });
        }

        let json_value: JsonValue =
            serde_json::from_str(&flag.value.value).map_err(|e| EvaluationError {
                code: open_feature::EvaluationErrorCode::ParseError,
                message: Some(format!("Failed to parse JSON: {}", e)),
            })?;

        let struct_value = match json_value {
            JsonValue::Object(map) => {
                let mut struct_map = std::collections::HashMap::new();
                for (key, json_val) in map {
                    // Filter out null values - absent fields are more semantically correct than empty strings
                    if !json_val.is_null() {
                        let of_value = json_to_open_feature_value(json_val);
                        struct_map.insert(key, of_value);
                    }
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
pub fn json_to_open_feature_value(json_val: JsonValue) -> Value {
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
                // Filter out null values - absent fields are more semantically correct than empty strings
                if !v.is_null() {
                    fields.insert(k, json_to_open_feature_value(v));
                }
            }
            Value::Struct(StructValue { fields })
        }
    }
}

/// Validate that a flag key is not empty.
pub fn validate_flag_key(flag_key: &str) -> Result<(), EvaluationError> {
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
///
/// Note: Struct fields are not supported by Flagsmith traits and will be
/// filtered out with a warning logged.
pub fn context_to_traits(
    context: &EvaluationContext,
) -> Vec<flagsmith::flagsmith::models::SDKTrait> {
    context
        .custom_fields
        .iter()
        .filter_map(|(key, value)| {
            let flagsmith_value = match value {
                EvaluationContextFieldValue::Bool(b) => Some(FlagsmithValue {
                    value: b.to_string(),
                    value_type: FlagsmithValueType::Bool,
                }),
                EvaluationContextFieldValue::String(s) => Some(FlagsmithValue {
                    value: s.clone(),
                    value_type: FlagsmithValueType::String,
                }),
                EvaluationContextFieldValue::Int(i) => Some(FlagsmithValue {
                    value: i.to_string(),
                    value_type: FlagsmithValueType::Integer,
                }),
                EvaluationContextFieldValue::Float(f) => Some(FlagsmithValue {
                    value: f.to_string(),
                    value_type: FlagsmithValueType::Float,
                }),
                EvaluationContextFieldValue::DateTime(dt) => Some(FlagsmithValue {
                    value: dt.to_string(),
                    value_type: FlagsmithValueType::String,
                }),
                EvaluationContextFieldValue::Struct(_) => {
                    tracing::warn!(
                        "Trait '{}': Struct values in evaluation context are not supported as Flagsmith traits and will be ignored.",
                        key
                    );
                    None
                }
            };

            flagsmith_value.map(|fv| flagsmith::flagsmith::models::SDKTrait::new(key.clone(), fv))
        })
        .collect()
}

/// Determine the OpenFeature reason based on the evaluation context and flag state.
///
/// Maps Flagsmith evaluation scenarios to OpenFeature reason codes:
/// - Identity evaluation (has targeting_key) � TargetingMatch
/// - Environment evaluation (no targeting_key) � Static
/// - Flag disabled � Disabled
pub fn determine_reason(context: &EvaluationContext, enabled: bool) -> Reason {
    if !enabled {
        Reason::Disabled
    } else if context.targeting_key.is_some() {
        Reason::TargetingMatch
    } else {
        Reason::Static
    }
}
