//! [Generated by cargo-readme: `cargo readme --no-title > README.md`]::
//!  # flagd Provider for OpenFeature
//!
//! A Rust implementation of the OpenFeature provider for flagd, enabling dynamic
//! feature flag evaluation in your applications.
//!
//! This provider supports multiple evaluation modes, advanced targeting rules, caching strategies,
//! and connection management. It is designed to work seamlessly with the OpenFeature SDK and the flagd service.
//!
//! ## Core Features
//!
//! - **Multiple Evaluation Modes**
//!     - **RPC Resolver (Remote Evaluation):** Uses gRPC to perform flag evaluations remotely at a flagd instance. Supports bi-directional streaming, retry backoff, and custom name resolution (including Envoy support).
//!     - **REST Resolver:** Uses the OpenFeature Remote Evaluation Protocol (OFREP) over HTTP to evaluate flags.
//!     - **In-Process Resolver:** Performs evaluations locally using an embedded evaluation engine. Flag configurations can be retrieved via gRPC (sync mode).
//!     - **File Resolver:** Operates entirely from a flag definition file, updating on file changes in a best-effort manner.
//!
//! - **Advanced Targeting**
//!     - **Fractional Rollouts:** Uses consistent hashing (implemented via murmurhash3) to split traffic between flag variants in configurable proportions.
//!     - **Semantic Versioning:** Compare values using common operators such as '=', '!=', '<', '<=', '>', '>=', '^', and '~'.
//!     - **String Operations:** Custom operators for performing “starts_with” and “ends_with” comparisons.
//!     - **Complex Targeting Rules:** Leverages JSONLogic and custom operators to support nested conditions and dynamic evaluation.
//!
//! - **Caching Strategies**
//!     - Built-in support for LRU caching as well as an in-memory alternative. Flag evaluation results can be cached and later returned with a “CACHED” reason until the configuration updates.
//!
//! - **Connection Management**
//!     - Automatic connection establishment with configurable retries, timeout settings, and custom TLS or Unix-socket options.
//!     - Support for upstream name resolution including a custom resolver for Envoy proxy integration.
//!
//! ## Installation
//! Add the dependency in your `Cargo.toml`:
//! ```bash
//! cargo add open-feature-flagd
//! cargo add open-feature
//! ```
//! Then integrate it into your application:
//!
//! ```rust,no_run
//! use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Example using the REST resolver mode.
//!     let provider = FlagdProvider::new(FlagdOptions {
//!         host: "localhost".to_string(),
//!         port: 8016,
//!         resolver_type: ResolverType::Rest,
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     let context = EvaluationContext::default().with_targeting_key("user-123");
//!     let result = provider.resolve_bool_value("bool-flag", &context).await.unwrap();
//!     println!("Flag value: {}", result.value);
//! }
//! ```
//!
//! ## Evaluation Modes
//! ### Remote Resolver (RPC)
//! In RPC mode, the provider communicates with flagd via gRPC. It supports features like streaming updates, retry mechanisms, and name resolution (including Envoy).
//!
//! ```rust,no_run
//! use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//!
//! #[tokio::main]
//! async fn main() {
//!     let provider = FlagdProvider::new(FlagdOptions {
//!         host: "localhost".to_string(),
//!         port: 8013,
//!         resolver_type: ResolverType::Rpc,
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     let context = EvaluationContext::default().with_targeting_key("user-123");
//!     let bool_result = provider.resolve_bool_value("feature-enabled", &context).await.unwrap();
//!     println!("Feature enabled: {}", bool_result.value);
//! }
//! ```
//!
//! ### REST Resolver
//! In REST mode the provider uses the OpenFeature Remote Evaluation Protocol (OFREP) over HTTP.
//! It is useful when gRPC is not an option.
//! ```rust,no_run
//! use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//!
//! #[tokio::main]
//! async fn main() {
//!     let provider = FlagdProvider::new(FlagdOptions {
//!         host: "localhost".to_string(),
//!         port: 8016,
//!         resolver_type: ResolverType::Rest,
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     let context = EvaluationContext::default().with_targeting_key("user-456");
//!     let result = provider.resolve_string_value("feature-variant", &context).await.unwrap();
//!     println!("Variant: {}", result.value);
//! }
//! ```
//!
//! ### In-Process Resolver
//! In-process evaluation is performed locally. Flag configurations are sourced via gRPC sync stream.
//! This mode supports advanced targeting operators (fractional, semver, string comparisons)
//! using the built-in evaluation engine.
//! ```rust,no_run
//! use open_feature_flagd::{CacheSettings, FlagdOptions, FlagdProvider, ResolverType};
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//!
//! #[tokio::main]
//! async fn main() {
//!     let provider = FlagdProvider::new(FlagdOptions {
//!         host: "localhost".to_string(),
//!         port: 8015,
//!         resolver_type: ResolverType::InProcess,
//!         selector: Some("my-service".to_string()),
//!         cache_settings: Some(CacheSettings::default()),
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     let context = EvaluationContext::default()
//!         .with_targeting_key("user-abc")
//!         .with_custom_field("environment", "production")
//!         .with_custom_field("semver", "2.1.0");
//!
//!     let dark_mode = provider.resolve_bool_value("dark-mode", &context).await.unwrap();
//!     println!("Dark mode enabled: {}", dark_mode.value);
//! }
//! ```
//!
//! ### File Mode
//! File mode is an in-process variant where flag configurations are read from a file.
//! This is useful for development or environments without network access.
//! ```rust,no_run
//! use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
//! use open_feature::provider::FeatureProvider;
//! use open_feature::EvaluationContext;
//!
//! #[tokio::main]
//! async fn main() {
//!     let file_path = "./path/to/flagd-config.json".to_string();
//!     let provider = FlagdProvider::new(FlagdOptions {
//!         host: "localhost".to_string(),
//!         resolver_type: ResolverType::File,
//!         source_configuration: Some(file_path),
//!         ..Default::default()
//!     }).await.unwrap();
//!
//!     let context = EvaluationContext::default();
//!     let result = provider.resolve_int_value("rollout-percentage", &context).await.unwrap();
//!     println!("Rollout percentage: {}", result.value);
//! }
//! ```
//!
//! ## Configuration Options
//! Configurations can be provided as constructor options or via environment variables (with constructor options taking priority). The following options are supported:
//!
//! | Option                                  | Env Variable                            | Type / Supported Value            | Default                             | Compatible Resolver            |
//! |-----------------------------------------|-----------------------------------------|-----------------------------------|-------------------------------------|--------------------------------|
//! | Host                                    | FLAGD_HOST                              | string                            | "localhost"                         | RPC, REST, In-Process, File    |
//! | Port                                    | FLAGD_PORT                              | number                            | 8013 (RPC), 8016 (REST)             | RPC, REST, In-Process, File    |
//! | Target URI                              | FLAGD_TARGET_URI                        | string                            | ""                                  | RPC, In-Process                |
//! | TLS                                     | FLAGD_TLS                               | boolean                           | false                               | RPC, In-Process                |
//! | Socket Path                             | FLAGD_SOCKET_PATH                       | string                            | ""                                  | RPC                            |
//! | Certificate Path                        | FLAGD_SERVER_CERT_PATH                  | string                            | ""                                  | RPC, In-Process                |
//! | Cache Type (LRU / In-Memory / Disabled) | FLAGD_CACHE                             | string ("lru", "mem", "disabled") | lru                                 | RPC, In-Process, File          |
//! | Cache TTL (Seconds)                     | FLAGD_CACHE_TTL                         | number                            | 60                                  | RPC, In-Process, File          |
//! | Max Cache Size                          | FLAGD_MAX_CACHE_SIZE                    | number                            | 1000                                | RPC, In-Process, File          |
//! | Offline File Path                       | FLAGD_OFFLINE_FLAG_SOURCE_PATH          | string                            | ""                                  | File                           |
//! | Retry Backoff (ms)                      | FLAGD_RETRY_BACKOFF_MS                  | number                            | 1000                                | RPC, In-Process                |
//! | Retry Backoff Maximum (ms)              | FLAGD_RETRY_BACKOFF_MAX_MS              | number                            | 120000                              | RPC, In-Process                |
//! | Retry Grace Period                      | FLAGD_RETRY_GRACE_PERIOD                | number                            | 5                                   | RPC, In-Process                |
//! | Event Stream Deadline (ms)              | FLAGD_STREAM_DEADLINE_MS                | number                            | 600000                              | RPC                            |
//! | Offline Poll Interval (ms)              | FLAGD_OFFLINE_POLL_MS                   | number                            | 5000                                | File                           |
//! | Source Selector                         | FLAGD_SOURCE_SELECTOR                   | string                            | ""                                  | In-Process                     |
//!
//! ## License
//! Apache 2.0 - See [LICENSE](./../../LICENSE) for more information.
//!

pub mod cache;
pub mod error;
pub mod resolver;
use crate::error::FlagdError;
use crate::resolver::in_process::resolver::{FileResolver, InProcessResolver};
use async_trait::async_trait;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{
    EvaluationContext, EvaluationContextFieldValue, EvaluationError, StructValue, Value,
};
use resolver::rest::RestResolver;
use tracing::debug;
use tracing::instrument;

use std::collections::BTreeMap;
use std::sync::Arc;

pub use cache::{CacheService, CacheSettings, CacheType};
pub use resolver::rpc::RpcResolver;

// Include the generated protobuf code
pub mod flagd {
    pub mod evaluation {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/flagd.evaluation.v1.rs"));
        }
    }
    pub mod sync {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/flagd.sync.v1.rs"));
        }
    }
}

/// Configuration options for the flagd provider
#[derive(Debug, Clone)]
pub struct FlagdOptions {
    /// Host address for the service
    pub host: String,
    /// Port number for the service
    pub port: u16,
    /// Target URI for custom name resolution (e.g. "envoy://service/flagd")
    pub target_uri: Option<String>,
    /// Type of resolver to use
    pub resolver_type: ResolverType,
    /// Whether to use TLS
    pub tls: bool,
    /// Path to TLS certificate
    pub cert_path: Option<String>,
    /// Request timeout in milliseconds
    pub deadline_ms: u32,
    /// Cache configuration settings
    pub cache_settings: Option<CacheSettings>,
    /// Initial backoff duration in milliseconds for retry attempts (default: 1000ms)
    /// Not supported in OFREP (REST) evaluation
    pub retry_backoff_ms: u32,
    /// Maximum backoff duration in milliseconds for retry attempts, prevents exponential backoff from growing indefinitely (default: 120000ms)
    /// Not supported in OFREP (REST) evaluation
    pub retry_backoff_max_ms: u32,
    /// Maximum number of retry attempts before giving up (default: 5)
    /// Not supported in OFREP (REST) evaluation
    pub retry_grace_period: u32,
    /// Source selector for filtering flag configurations
    /// Used to scope flag sync requests in in-process evaluation
    pub selector: Option<String>,
    /// Unix domain socket path for connecting to flagd
    /// When provided, this takes precedence over host:port configuration
    /// Example: "/var/run/flagd.sock"
    /// Only works with GRPC resolver
    pub socket_path: Option<String>,
    /// Source configuration for file-based resolver
    pub source_configuration: Option<String>,
    /// The deadline in milliseconds for event streaming operations. Set to 0 to disable.
    /// Recommended to prevent infrastructure from killing idle connections.
    pub stream_deadline_ms: u32,
    /// Offline polling interval in milliseconds
    pub offline_poll_interval_ms: Option<u32>,
}
/// Type of resolver to use for flag evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum ResolverType {
    /// Remote evaluation using gRPC connection to flagd service
    Rpc,
    /// Remote evaluation using REST connection to flagd service
    Rest,
    /// Local evaluation with embedded flag engine using gRPC connection
    InProcess,
    /// Local evaluation with no external dependencies
    File,
}
impl Default for FlagdOptions {
    fn default() -> Self {
        let resolver_type = if let Ok(r) = std::env::var("FLAGD_RESOLVER") {
            match r.to_uppercase().as_str() {
                "RPC" => ResolverType::Rpc,
                "REST" => ResolverType::Rest,
                "IN-PROCESS" | "INPROCESS" => ResolverType::InProcess,
                "FILE" | "OFFLINE" => ResolverType::File,
                _ => ResolverType::Rpc,
            }
        } else {
            ResolverType::Rpc
        };

        let port = match resolver_type {
            ResolverType::Rpc => 8013,
            ResolverType::InProcess => 8015,
            _ => 8013,
        };

        let mut options = Self {
            host: std::env::var("FLAGD_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("FLAGD_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(port),
            target_uri: std::env::var("FLAGD_TARGET_URI").ok(),
            resolver_type,
            tls: std::env::var("FLAGD_TLS")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            cert_path: std::env::var("FLAGD_SERVER_CERT_PATH").ok(),
            deadline_ms: std::env::var("FLAGD_DEADLINE_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            retry_backoff_ms: std::env::var("FLAGD_RETRY_BACKOFF_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            retry_backoff_max_ms: std::env::var("FLAGD_RETRY_BACKOFF_MAX_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120000),
            retry_grace_period: std::env::var("FLAGD_RETRY_GRACE_PERIOD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            stream_deadline_ms: std::env::var("FLAGD_STREAM_DEADLINE_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600000),
            socket_path: std::env::var("FLAGD_SOCKET_PATH").ok(),
            selector: std::env::var("FLAGD_SOURCE_SELECTOR").ok(),
            cache_settings: Some(CacheSettings::default()),
            source_configuration: std::env::var("FLAGD_OFFLINE_FLAG_SOURCE_PATH").ok(),
            offline_poll_interval_ms: Some(
                std::env::var("FLAGD_OFFLINE_POLL_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5000),
            ),
        };

        if options.source_configuration.is_some() && options.resolver_type != ResolverType::Rpc {
            options.resolver_type = ResolverType::File;
        }

        options
    }
}

/// Main provider implementation for flagd
#[derive(Clone)]
pub struct FlagdProvider {
    /// The underlying feature flag resolver
    provider: Arc<dyn FeatureProvider + Send + Sync>,
    /// Optional caching layer
    cache: Option<Arc<CacheService<Value>>>,
}

impl FlagdProvider {
    #[instrument(skip(options))]
    pub async fn new(options: FlagdOptions) -> Result<Self, FlagdError> {
        debug!("Initializing FlagdProvider with options: {:?}", options);

        let provider: Arc<dyn FeatureProvider + Send + Sync> = match options.resolver_type {
            ResolverType::Rpc => {
                debug!("Using RPC resolver");
                Arc::new(RpcResolver::new(&options).await?)
            }
            ResolverType::Rest => {
                debug!("Using REST resolver");
                Arc::new(RestResolver::new(&options))
            }
            ResolverType::InProcess => {
                debug!("Using in-process resolver");
                Arc::new(InProcessResolver::new(&options).await?)
            }
            ResolverType::File => {
                debug!("Using file resolver");
                Arc::new(
                    FileResolver::new(
                        options.source_configuration.unwrap(),
                        options.cache_settings.clone(),
                    )
                    .await?,
                )
            }
        };

        Ok(Self {
            provider,
            cache: options
                .cache_settings
                .map(|settings| Arc::new(CacheService::new(settings))),
        })
    }
}

impl std::fmt::Debug for FlagdProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlagdProvider")
            .field("cache", &self.cache)
            .finish()
    }
}

fn convert_context(context: &EvaluationContext) -> Option<prost_types::Struct> {
    let mut fields = BTreeMap::new();

    if let Some(targeting_key) = &context.targeting_key {
        fields.insert(
            "targetingKey".to_string(),
            prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue(targeting_key.clone())),
            },
        );
    }

    for (key, value) in &context.custom_fields {
        let prost_value = match value {
            EvaluationContextFieldValue::String(s) => prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue(s.clone())),
            },
            EvaluationContextFieldValue::Bool(b) => prost_types::Value {
                kind: Some(prost_types::value::Kind::BoolValue(*b)),
            },
            EvaluationContextFieldValue::Int(i) => prost_types::Value {
                kind: Some(prost_types::value::Kind::NumberValue(*i as f64)),
            },
            EvaluationContextFieldValue::Float(f) => prost_types::Value {
                kind: Some(prost_types::value::Kind::NumberValue(*f)),
            },
            EvaluationContextFieldValue::DateTime(dt) => prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue(dt.to_string())),
            },
            EvaluationContextFieldValue::Struct(s) => prost_types::Value {
                kind: Some(prost_types::value::Kind::StringValue(format!("{:?}", s))),
            },
        };
        fields.insert(key.clone(), prost_value);
    }

    Some(prost_types::Struct { fields })
}

fn convert_proto_struct_to_struct_value(proto_struct: prost_types::Struct) -> StructValue {
    let fields = proto_struct
        .fields
        .into_iter()
        .map(|(key, value)| {
            (
                key,
                match value.kind.unwrap() {
                    prost_types::value::Kind::NullValue(_) => Value::String(String::new()),
                    prost_types::value::Kind::NumberValue(n) => Value::Float(n),
                    prost_types::value::Kind::StringValue(s) => Value::String(s),
                    prost_types::value::Kind::BoolValue(b) => Value::Bool(b),
                    prost_types::value::Kind::StructValue(s) => Value::String(format!("{:?}", s)),
                    prost_types::value::Kind::ListValue(l) => Value::String(format!("{:?}", l)),
                },
            )
        })
        .collect();

    StructValue { fields }
}

impl FlagdProvider {
    async fn get_cached_value<T>(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
        value_converter: impl Fn(Value) -> Option<T>,
    ) -> Option<T> {
        if let Some(cache) = &self.cache {
            if let Some(cached_value) = cache.get(flag_key, context).await {
                return value_converter(cached_value);
            }
        }
        None
    }
}

#[async_trait]
impl FeatureProvider for FlagdProvider {
    fn metadata(&self) -> &ProviderMetadata {
        self.provider.metadata()
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        if let Some(value) = self
            .get_cached_value(flag_key, context, |v| match v {
                Value::Bool(b) => Some(b),
                _ => None,
            })
            .await
        {
            return Ok(ResolutionDetails::new(value));
        }

        let result = self.provider.resolve_bool_value(flag_key, context).await?;

        if let Some(cache) = &self.cache {
            cache
                .add(flag_key, context, Value::Bool(result.value))
                .await;
        }

        Ok(result)
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError> {
        if let Some(value) = self
            .get_cached_value(flag_key, context, |v| match v {
                Value::Int(i) => Some(i),
                _ => None,
            })
            .await
        {
            return Ok(ResolutionDetails::new(value));
        }

        let result = self.provider.resolve_int_value(flag_key, context).await?;

        if let Some(cache) = &self.cache {
            cache.add(flag_key, context, Value::Int(result.value)).await;
        }

        Ok(result)
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError> {
        if let Some(value) = self
            .get_cached_value(flag_key, context, |v| match v {
                Value::Float(f) => Some(f),
                _ => None,
            })
            .await
        {
            return Ok(ResolutionDetails::new(value));
        }

        let result = self.provider.resolve_float_value(flag_key, context).await?;

        if let Some(cache) = &self.cache {
            cache
                .add(flag_key, context, Value::Float(result.value))
                .await;
        }

        Ok(result)
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError> {
        if let Some(value) = self
            .get_cached_value(flag_key, context, |v| match v {
                Value::String(s) => Some(s),
                _ => None,
            })
            .await
        {
            return Ok(ResolutionDetails::new(value));
        }

        let result = self
            .provider
            .resolve_string_value(flag_key, context)
            .await?;

        if let Some(cache) = &self.cache {
            cache
                .add(flag_key, context, Value::String(result.value.clone()))
                .await;
        }

        Ok(result)
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
        if let Some(value) = self
            .get_cached_value(flag_key, context, |v| match v {
                Value::Struct(s) => Some(s),
                _ => None,
            })
            .await
        {
            return Ok(ResolutionDetails::new(value));
        }

        let result = self
            .provider
            .resolve_struct_value(flag_key, context)
            .await?;

        if let Some(cache) = &self.cache {
            cache
                .add(flag_key, context, Value::Struct(result.value.clone()))
                .await;
        }

        Ok(result)
    }
}
