# Flagd Evaluation Engine

A JSONLogic-based evaluation engine for flagd, providing local flag evaluation with support for targeting rules, fractional rollouts, and semantic version comparisons.

## Features

- **JSONLogic-based Targeting**: Support for complex targeting rules using JSONLogic
- **Fractional Rollouts**: Consistent hashing (murmurhash3) for percentage-based rollouts
- **Semantic Versioning**: Compare values using semver operators (^, ~, =, !=, <, <=, >, >=)
- **String Operations**: Custom operators for "starts_with" and "ends_with" comparisons

## Installation

Add the dependency in your `Cargo.toml`:

```bash
cargo add flagd-evaluation-engine
```

## Usage

```rust
use flagd_evaluation_engine::{FlagdEvaluationError, FeatureFlag, FlagParser, Operator};
use open_feature::EvaluationContext;

fn main() {
    // Create the operator for evaluating targeting rules
    let operator = Operator::new();

    // Create evaluation context
    let context = EvaluationContext::default()
        .with_targeting_key("user-123")
        .with_custom_field("tier", "premium");

    // Apply a targeting rule (synchronous)
    let result = operator.apply(
        "my-flag",
        r#"{"if": [{"==": [{"var": "tier"}, "premium"]}, "gold", "silver"]}"#,
        &context,
    );

    println!("Result: {:?}", result);
}
```

## License

Apache 2.0 - See [LICENSE](./LICENSE) for more information.