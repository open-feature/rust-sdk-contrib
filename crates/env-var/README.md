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
 async fn test_resolve_bool_value() {
        let flag_key = "TEST_BOOL_ENV_VAR";
        let provider = EnvVarProvider::default();
        for &flag_value in &["true", "false"] {
            std::env::set_var(flag_key, flag_value);

            let result = provider
                .resolve_bool_value(flag_key, &EvaluationContext::default())
                .await;
            assert!(result.is_ok());

            std::env::remove_var(flag_key);
        }
    }
```

## Testing

Run `cargo test` to execute tests.

## Maintainers

- [Jose Bovet Derpich](https://github.com/jbovet)
