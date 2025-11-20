# OpenFeature Rust SDK Contributions - Repository Context

**Repository**: https://github.com/open-feature/rust-sdk-contrib
**Purpose**: OpenFeature provider implementations for Rust
**Last Updated**: 2025-11-19

---

## 1. Repository Structure

### 1.1 Project Organization

This is the OpenFeature Rust SDK contributions repository. Each provider lives in its own crate under `crates/`:

```
rust-sdk-contrib/
├── crates/
│   ├── env-var/          # Environment variable provider
│   ├── flagd/            # flagd provider (comprehensive reference)
│   ├── flipt/            # Flipt provider (simpler reference)
│   └── ofrep/            # OFREP provider (HTTP-based reference)
├── Cargo.toml            # Workspace definition
├── CONTRIBUTING.md       # Contribution guidelines
├── README.md
├── release-please-config.json
└── renovate.json
```

### 1.2 Workspace Configuration

The root `Cargo.toml` defines a workspace with all provider crates as members:

```toml
[workspace]
members = [
    "crates/env-var",
    "crates/flagd",
    "crates/flipt",
    "crates/ofrep"
]
```

**Key settings**:
- **Edition**: Rust 2024
- **License**: Apache 2.0

---

## 2. Contributing Guidelines

From `CONTRIBUTING.md`:

### 2.1 Project Hierarchy

Each contrib must be placed under `crates/<provider-name>`:
- Create a new directory: `crates/<provider-name>/`
- Add to workspace in root `Cargo.toml`
- Follow standard Rust crate structure

### 2.2 Coding Style

**Requirements**:
1. Add comments and tests for publicly exposed APIs
2. Follow Clippy rules from [rust-sdk/src/lib.rs](https://github.com/open-feature/rust-sdk/blob/main/src/lib.rs)

**Best practices** (from existing providers):
- Use `tracing` for logging (not `log`)
- Use `thiserror` for error types
- Use `async-trait` for async trait implementations
- Document public APIs with doc comments
- Include usage examples in doc comments

---

## 3. Development Setup

Based on `crates/flagd/docs/contributing.md`:

### 3.1 Building

```bash
# Navigate to your crate directory
cd crates/<provider-name>

# Build the crate
cargo build

# Build entire workspace (from root)
cargo build --workspace
```

### 3.2 Testing

**Unit tests** (no external dependencies):
```bash
# Run unit tests only
cargo test --lib

# Run with full logging
RUST_LOG_SPAN_EVENTS=full RUST_LOG=debug cargo test -- --nocapture
```

**Integration tests** (may require Docker):
```bash
# Run all tests (including E2E)
cargo test

# Note: E2E tests use testcontainers-rs
# Docker is required (podman not currently supported)
```

### 3.3 Development Dependencies

Common dev dependencies across providers:
- `test-log = "0.2"` - For tracing logs in tests
- `tracing-subscriber` - For test logging
- `testcontainers` - For E2E tests with Docker (optional)

---

## 4. OpenFeature Provider Interface

### 4.1 Required Trait Implementation

All providers must implement the `FeatureProvider` trait from `open-feature` crate:

```rust
use async_trait::async_trait;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError, StructValue};

#[async_trait]
pub trait FeatureProvider {
    // Required: Provider metadata
    fn metadata(&self) -> &ProviderMetadata;

    // Required: Five evaluation methods for different types
    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError>;

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<i64>, EvaluationError>;

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<f64>, EvaluationError>;

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<String>, EvaluationError>;

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<StructValue>, EvaluationError>;
}
```

### 4.2 Key Types

**EvaluationContext**:
```rust
pub struct EvaluationContext {
    pub targeting_key: Option<String>,  // User identifier
    pub custom_fields: HashMap<String, EvaluationContextFieldValue>,  // Additional attributes
}

// Usage
let context = EvaluationContext::default()
    .with_targeting_key("user-123")
    .with_custom_field("email", "user@example.com")
    .with_custom_field("plan", "premium");
```

**ResolutionDetails**:
```rust
pub struct ResolutionDetails<T> {
    pub value: T,                                           // The evaluated flag value
    pub reason: Option<Reason>,                             // Why this value was returned
    pub variant: Option<String>,                            // Variant identifier
    pub error_code: Option<EvaluationErrorCode>,            // Error code if applicable
    pub error_message: Option<String>,                      // Error message if applicable
    pub flag_metadata: Option<HashMap<String, Value>>,      // Additional metadata
}

// Simple construction
ResolutionDetails::new(true)  // Just the value

// With reason
ResolutionDetails {
    value: true,
    reason: Some(Reason::TargetingMatch),
    ..Default::default()
}
```

**EvaluationError**:
```rust
pub struct EvaluationError {
    pub code: EvaluationErrorCode,
    pub message: Option<String>,
}

pub enum EvaluationErrorCode {
    FlagNotFound,           // Flag doesn't exist
    ParseError,             // Failed to parse value
    TypeMismatch,           // Wrong type returned
    TargetingKeyMissing,    // Required targeting_key not provided
    InvalidContext,         // Invalid evaluation context
    ProviderNotReady,       // Provider not initialized or unavailable
    General(String),        // Other errors
}
```

**Reason** (from OpenFeature spec):
```rust
pub enum Reason {
    Static,           // Flag evaluated without targeting
    TargetingMatch,   // Flag evaluated with targeting rules
    Disabled,         // Flag is disabled
    Cached,           // Value returned from cache
    Default,          // Default value returned (error case)
    Error,            // Error occurred during evaluation
    Unknown,          // Reason unknown
}
```

---

## 5. Common Provider Patterns

### 5.1 Standard Crate Structure

```
crates/<provider-name>/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs           # Main provider + re-exports
│   ├── error.rs         # Custom error types (optional)
│   ├── resolver.rs      # Core evaluation logic (optional)
│   └── utils.rs         # Helper functions (optional)
├── tests/
│   ├── integration_test.rs
│   └── fixtures/
└── examples/
    └── basic_usage.rs
```

### 5.2 Provider Implementation Pattern

**Basic structure** (from OFREP and Flipt providers):

```rust
// lib.rs
mod error;  // Optional: Custom error types

use async_trait::async_trait;
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{EvaluationContext, EvaluationError};

// Configuration struct
#[derive(Debug, Clone)]
pub struct ProviderOptions {
    pub required_field: String,
    pub optional_field: Option<String>,
}

impl Default for ProviderOptions {
    fn default() -> Self {
        ProviderOptions {
            required_field: "default_value".to_string(),
            optional_field: None,
        }
    }
}

// Main provider struct
pub struct MyProvider {
    metadata: ProviderMetadata,
    client: SdkClient,  // Feature flag SDK client
}

impl MyProvider {
    /// Creates a new provider instance
    pub async fn new(options: ProviderOptions) -> Result<Self, MyError> {
        // 1. Validate configuration
        // 2. Initialize SDK client
        // 3. Return provider instance
        Ok(Self {
            metadata: ProviderMetadata::new("my-provider"),
            client: SdkClient::new(options)?,
        })
    }
}

#[async_trait]
impl FeatureProvider for MyProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
    ) -> Result<ResolutionDetails<bool>, EvaluationError> {
        // Implementation
    }

    // ... other resolve methods
}
```

### 5.3 Configuration Patterns

**Pattern 1 - Simple struct (OFREP)**:
```rust
#[derive(Debug, Clone)]
pub struct OfrepOptions {
    pub base_url: String,
    pub headers: HeaderMap,
    pub connect_timeout: Duration,
}

impl Default for OfrepOptions {
    fn default() -> Self {
        OfrepOptions {
            base_url: "http://localhost:8016".to_string(),
            headers: HeaderMap::new(),
            connect_timeout: Duration::from_secs(10),
        }
    }
}
```

**Pattern 2 - Generic config (Flipt)**:
```rust
pub struct Config<A>
where
    A: AuthenticationStrategy,
{
    pub url: String,
    pub authentication_strategy: A,
    pub timeout: u64,
}
```

**Pattern 3 - Builder pattern (flagd)**:
```rust
let options = FlagdOptions::builder()
    .host("localhost")
    .port(8013)
    .resolver_type(ResolverType::Rpc)
    .build()?;
```

### 5.4 Error Handling Pattern

**Define custom errors with thiserror**:
```rust
// error.rs
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MyProviderError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Flag not found: {0}")]
    FlagNotFound(String),
}
```

**Map to OpenFeature errors**:
```rust
fn map_error(err: SdkError) -> EvaluationError {
    match err {
        SdkError::NotFound(flag) => EvaluationError {
            code: EvaluationErrorCode::FlagNotFound,
            message: Some(format!("Flag '{}' not found", flag)),
        },
        SdkError::NetworkError(e) => EvaluationError {
            code: EvaluationErrorCode::ProviderNotReady,
            message: Some(format!("Network error: {}", e)),
        },
        SdkError::ParseError(e) => EvaluationError {
            code: EvaluationErrorCode::ParseError,
            message: Some(e),
        },
        _ => EvaluationError {
            code: EvaluationErrorCode::General("Unknown error".to_string()),
            message: Some(err.to_string()),
        },
    }
}
```

**Error handling principles**:
- Never panic from evaluation methods
- Always return `Result` with appropriate error code
- Include descriptive error messages
- Use `tracing` for debug logging
- Map SDK-specific errors to OpenFeature error codes

### 5.5 Context Mapping Pattern

Feature flag SDKs often have their own context format. Map OpenFeature context to SDK format:

```rust
// Example from Flipt provider
fn translate_context(ctx: &EvaluationContext) -> HashMap<String, String> {
    ctx.custom_fields
        .iter()
        .map(|(k, v)| {
            let value = match v {
                EvaluationContextFieldValue::Bool(b) => b.to_string(),
                EvaluationContextFieldValue::String(s) => s.clone(),
                EvaluationContextFieldValue::Int(i) => i.to_string(),
                EvaluationContextFieldValue::Float(f) => f.to_string(),
                // ... handle other types
            };
            (k.clone(), value)
        })
        .collect()
}
```

### 5.6 Type Conversion Patterns

**Boolean** (direct mapping):
```rust
async fn resolve_bool_value(
    &self,
    flag_key: &str,
    ctx: &EvaluationContext,
) -> Result<ResolutionDetails<bool>, EvaluationError> {
    self.client
        .evaluate_boolean(flag_key, ctx)
        .await
        .map_err(map_error)
        .map(|result| ResolutionDetails::new(result.enabled))
}
```

**Integer/Float** (with parsing):
```rust
async fn resolve_int_value(
    &self,
    flag_key: &str,
    ctx: &EvaluationContext,
) -> Result<ResolutionDetails<i64>, EvaluationError> {
    let result = self.client.get_value(flag_key, ctx).await?;

    result.value
        .parse::<i64>()
        .map(ResolutionDetails::new)
        .map_err(|e| EvaluationError {
            code: EvaluationErrorCode::TypeMismatch,
            message: Some(format!(
                "Expected i64, but got '{}': {}",
                result.value, e
            )),
        })
}
```

**String** (direct or conversion):
```rust
async fn resolve_string_value(
    &self,
    flag_key: &str,
    ctx: &EvaluationContext,
) -> Result<ResolutionDetails<String>, EvaluationError> {
    self.client
        .get_value(flag_key, ctx)
        .await
        .map_err(map_error)
        .map(|result| ResolutionDetails::new(result.value))
}
```

**Struct** (JSON parsing):
```rust
async fn resolve_struct_value(
    &self,
    flag_key: &str,
    ctx: &EvaluationContext,
) -> Result<ResolutionDetails<StructValue>, EvaluationError> {
    let result = self.client.get_value(flag_key, ctx).await?;

    // Parse JSON string to Value
    let value: Value = serde_json::from_str(&result.value)
        .map_err(|e| EvaluationError {
            code: EvaluationErrorCode::ParseError,
            message: Some(format!("Failed to parse JSON: {}", e)),
        })?;

    // Ensure it's a struct/object
    match value {
        Value::Struct(struct_value) => {
            Ok(ResolutionDetails::new(struct_value))
        }
        _ => Err(EvaluationError {
            code: EvaluationErrorCode::TypeMismatch,
            message: Some(format!(
                "Expected object, but got: {}",
                result.value
            )),
        }),
    }
}
```

---

## 6. Testing Patterns

### 6.1 Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;  // For tracing logs in tests

    #[test(tokio::test)]
    async fn test_bool_evaluation() {
        // Arrange
        let provider = MyProvider::new(options).await.unwrap();
        let context = EvaluationContext::default()
            .with_targeting_key("user-123");

        // Act
        let result = provider
            .resolve_bool_value("my-flag", &context)
            .await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, true);
    }

    #[test(tokio::test)]
    async fn test_flag_not_found() {
        let provider = MyProvider::new(options).await.unwrap();
        let context = EvaluationContext::default();

        let result = provider
            .resolve_bool_value("non-existent", &context)
            .await;

        assert!(result.is_err());
        match result.unwrap_err().code {
            EvaluationErrorCode::FlagNotFound => {},
            _ => panic!("Expected FlagNotFound error"),
        }
    }
}
```

### 6.2 Test Logging

Enable detailed logs during tests:

```bash
# Full tracing output
RUST_LOG_SPAN_EVENTS=full RUST_LOG=debug cargo test -- --nocapture

# Specific module
RUST_LOG=my_provider=debug cargo test -- --nocapture
```

### 6.3 Integration Testing

Use `testcontainers-rs` for E2E tests with Docker:

```rust
#[cfg(test)]
mod integration_tests {
    use testcontainers::{clients, images};

    #[tokio::test]
    async fn test_against_real_service() {
        let docker = clients::Cli::default();
        let container = docker.run(images::generic::GenericImage::new(
            "my-service",
            "latest"
        ));

        let port = container.get_host_port_ipv4(8080);
        let provider = MyProvider::new(ProviderOptions {
            host: "localhost".to_string(),
            port,
            ..Default::default()
        }).await.unwrap();

        // Run tests against real service
    }
}
```

---

## 7. Documentation Standards

### 7.1 README Structure

Each provider should have a README with:

1. **Title and brief description**
2. **Installation instructions**
3. **Basic usage example**
4. **Configuration options table**
5. **Advanced usage examples** (optional)
6. **Testing instructions**
7. **License**

### 7.2 Inline Documentation

Use doc comments for public APIs:

```rust
/// A feature flag provider for [Service Name].
///
/// This provider enables dynamic feature flag evaluation using the
/// [Service Name] platform.
///
/// # Example
///
/// ```rust
/// use my_provider::{MyProvider, MyOptions};
///
/// #[tokio::main]
/// async fn main() {
///     let provider = MyProvider::new(MyOptions {
///         api_key: "your-key".to_string(),
///         ..Default::default()
///     }).await.unwrap();
/// }
/// ```
pub struct MyProvider {
    // ...
}
```

### 7.3 cargo-readme

Consider using `cargo-readme` to generate README from doc comments:

```bash
cargo install cargo-readme
cargo readme --no-title --no-license > README.md
```

Pattern seen in OFREP provider:
```rust
//! [Generated by cargo-readme: `cargo readme --no-title --no-license > README.md`]::
//!  # OFREP Provider for OpenFeature
//!
//! A Rust implementation of...
```

---

## 8. Common Dependencies

### 8.1 Runtime Dependencies

```toml
[dependencies]
# OpenFeature SDK
open-feature = "0.x.x"

# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Error handling
thiserror = "1.0"
anyhow = "1.0"  # Optional

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"

# HTTP client (if needed)
reqwest = { version = "0.11", features = ["json"] }

# URL parsing (if needed)
url = "2.0"
```

### 8.2 Development Dependencies

```toml
[dev-dependencies]
# Testing
test-log = "0.2"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Integration testing
testcontainers = "0.15"  # Optional, if using Docker

# Mocking
mockito = "1.0"  # Optional, for HTTP mocking
```

---

## 9. References

### 9.1 OpenFeature Resources

- **OpenFeature Specification**: https://openfeature.dev/specification/
- **Provider Specification**: https://openfeature.dev/specification/sections/providers
- **Rust SDK Repository**: https://github.com/open-feature/rust-sdk
- **Rust SDK Documentation**: https://docs.rs/open-feature/

### 9.2 Repository Resources

- **Contributing Guide**: `CONTRIBUTING.md`
- **flagd Provider** (comprehensive): `crates/flagd/`
- **OFREP Provider** (HTTP-based): `crates/ofrep/`
- **Flipt Provider** (simple): `crates/flipt/`
- **env-var Provider** (basic): `crates/env-var/`

### 9.3 Rust Resources

- **async-trait**: https://docs.rs/async-trait/
- **thiserror**: https://docs.rs/thiserror/
- **tracing**: https://docs.rs/tracing/
- **tokio**: https://docs.rs/tokio/

---

## 10. Quick Start Checklist

When creating a new provider:

- [ ] Create directory: `crates/<provider-name>/`
- [ ] Setup `Cargo.toml` with dependencies
- [ ] Add to workspace in root `Cargo.toml`
- [ ] Create basic structure: `src/lib.rs`, `src/error.rs`
- [ ] Define configuration struct with `Default` trait
- [ ] Implement provider constructor with validation
- [ ] Implement `FeatureProvider` trait (5 methods)
- [ ] Add error mapping function
- [ ] Write unit tests for each method
- [ ] Create usage example in `examples/`
- [ ] Write README with installation and examples
- [ ] Add CHANGELOG.md
- [ ] Test with `cargo test --lib`
- [ ] Test with full suite: `cargo test`
- [ ] Run clippy: `cargo clippy --all-targets`
- [ ] Format code: `cargo fmt`

---

**Last Updated**: 2025-11-19
**Maintainers**: OpenFeature Contributors
