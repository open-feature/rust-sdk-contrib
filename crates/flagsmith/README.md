# Flagsmith Provider for OpenFeature

A Rust implementation of the OpenFeature provider for Flagsmith, enabling dynamic feature flag evaluation using the Flagsmith platform.

This provider integrates the [Flagsmith Rust SDK](https://github.com/Flagsmith/flagsmith-rust-client) with [OpenFeature](https://openfeature.dev/), supporting both environment-level and identity-specific flag evaluation with trait-based targeting.

## Features

- **Environment-level evaluation**: Evaluate flags at the environment level without user context
- **Identity-specific evaluation**: Target users with personalized flag values based on traits
- **Type safety**: Full support for boolean, string, integer, float, and structured (JSON) flag types
- **Local evaluation**: Optional local evaluation mode for improved performance and offline support
- **Async support**: Built on Tokio with non-blocking flag evaluations

## Installation

Add the dependency in your `Cargo.toml`:
```bash
cargo add open-feature-flagsmith
cargo add open-feature
```

## Basic Usage

```rust
use open_feature::OpenFeature;
use open_feature::EvaluationContext;
use open_feature_flagsmith::{FlagsmithProvider, FlagsmithOptions};

#[tokio::main]
async fn main() {
    // Initialize the provider
    let provider = FlagsmithProvider::new(
        "your-environment-key".to_string(),
        FlagsmithOptions::default()
    ).await.unwrap();

    // Set up OpenFeature API
    let mut api = OpenFeature::singleton_mut().await;
    api.set_provider(provider).await;
    let client = api.create_client();

    // Evaluate a flag
    let context = EvaluationContext::default();
    let enabled = client
        .get_bool_value("my-feature", &context, None)
        .await
        .unwrap_or(false);

    println!("Feature enabled: {}", enabled);
}
```

## Identity-Specific Evaluation

```rust
use open_feature::EvaluationContext;

// Create context with targeting key and user traits
let context = EvaluationContext::default()
    .with_targeting_key("user-123")
    .with_custom_field("email", "user@example.com")
    .with_custom_field("plan", "premium")
    .with_custom_field("age", 25);

let enabled = client
    .get_bool_value("premium-feature", &context, None)
    .await
    .unwrap_or(false);
```

## Flag Types

```rust
// Assuming you have set up the client as shown in the Basic Usage section
let context = EvaluationContext::default();

// Boolean flags
let enabled = client.get_bool_value("feature-toggle", &context, None).await.unwrap();

// String flags
let theme = client.get_string_value("theme", &context, None).await.unwrap();

// Integer flags
let max_items = client.get_int_value("max-items", &context, None).await.unwrap();

// Float flags
let multiplier = client.get_float_value("price-multiplier", &context, None).await.unwrap();

// Structured flags (JSON objects)
let config = client.get_object_value("config", &context, None).await.unwrap();
```

## Local Evaluation

Local evaluation mode downloads the environment configuration and evaluates flags locally for better performance:

```rust
use open_feature_flagsmith::FlagsmithOptions;

// Requires a server-side environment key (starts with "ser.")
let provider = FlagsmithProvider::new(
    "ser.your-server-key".to_string(),
    FlagsmithOptions::default()
        .with_local_evaluation(true)
).await.unwrap();
```

**Benefits:**
- Lower latency (no API calls per evaluation)
- Works offline (uses cached environment)
- Reduced API load

**Requirements:**
- Server-side environment key (starts with `ser.`)
- Initial API call to fetch environment
- Periodic polling to refresh (default: 60s)

## Configuration Options

Configurations can be provided as constructor options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `api_url` | `Option<String>` | Flagsmith Edge API | Custom Flagsmith API endpoint |
| `request_timeout_seconds` | `Option<u64>` | 10 | Request timeout in seconds |
| `enable_local_evaluation` | `bool` | `false` | Enable local evaluation mode |
| `environment_refresh_interval_mills` | `Option<u64>` | 60000 | Polling interval for local mode (ms) |
| `enable_analytics` | `bool` | `false` | Enable analytics tracking |
| `custom_headers` | `Option<HeaderMap>` | None | Custom HTTP headers |

### Example Configuration

```rust
use open_feature_flagsmith::FlagsmithOptions;

let options = FlagsmithOptions::default()
    .with_local_evaluation(true)
    .with_analytics(true)
    .with_timeout(15);

let provider = FlagsmithProvider::new(
    "ser.your-key".to_string(),
    options
).await.unwrap();
```

## Evaluation Context Transformation

OpenFeature standardizes the evaluation context with a `targeting_key` and arbitrary custom fields. For Flagsmith:

- **`targeting_key`** → Flagsmith identity identifier
- **`custom_fields`** → Flagsmith traits for segmentation

When a `targeting_key` is present, the provider performs identity-specific evaluation. Otherwise, it evaluates at the environment level.

## License

Apache 2.0 - See [LICENSE](./../../LICENSE) for more information.
