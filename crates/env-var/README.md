# Environment Variables Provider

This Rust library provides a feature flag provider that resolves feature flags from environment variables.

## Supported Types

The provider supports the following types:

- Int
- Float
- String
- Bool

> Please note that *Struct* type is not currently supported yet.

## Error Handling

The provider will return `EvaluationResult::Err(EvaluationError)` if the flag is not found or if the value is not of the expected type.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
open-feature-env-var = "0.1.0"
```

## Example

```rust

let mut api = OpenFeature::singleton_mut().await;
let provider  = EnvVarProvider::default();
api.set_provider(provider).await;
let client = api.create_named_client("env-var-client");

let mut message =  "Hello rustaceans!";
let is_feature_enabled = client.get_bool_value("env-flag-key", &EvaluationContext::default(), None).await.unwrap_or(false);

if is_feature_enabled {
    message = "Hello rustaceans from feature flag!";
}
```

The environment variable names can be customized by injecting a custom `Rename` implementation:

```rust
/// Transforms env-flag-key to ENV_FLAG_KEY
fn underscore(flag_key: &str) -> Cow<'_, str> {
    flag_key.replace("-", "_").to_uppercase().into()
}

let provider  = EnvVarProvider::new(underscore);
```

## Testing

Run `cargo test` to execute tests.

## Maintainers

- [Jose Bovet Derpich](https://github.com/jbovet)

## License

Apache 2.0 - See [LICENSE](./../../LICENSE) for more information.
