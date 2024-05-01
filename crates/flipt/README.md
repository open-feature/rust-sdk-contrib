# Flipt Provider

[Flipt](https://www.flipt.io/) is an open source developer friendly feature flagging solution, that allows for easy management and fast feature evaluation.

This provider is an implementation on top of the official [Flipt Rust Server Side SDK](https://github.com/flipt-io/flipt-server-sdks/tree/main/flipt-rust).

## Installation

Add the following line to Cargo.toml

```
open-feature-flipt="0.1.0"
```

## Note

> [!NOTE]  
> [Variant Evaluation](https://docs.flipt.io/reference/evaluation/batch-evaluation) is not implemented yet.

## Basic Usage and Examples

```rust
use std::collections::HashMap;

use openfeature_flipt::flipt::{Config, FliptProvider, NoneAuthentication};
use openfeature_flipt::open_feature::{provider::FeatureProvider, EvaluationContext};

let config = Config {
    url: "http://localhost:8080/".to_string(),
    authentication_strategy: NoneAuthentication::new(),
    timeout: 60,
};
let ctx = EvaluationContext {
    targeting_key: None,
    custom_fields: HashMap::new(),
};

let provider = FliptProvider::new("default".to_owned(), config).unwrap();
let details = provider
    .resolve_bool_value("flag_boolean", &ctx)
    .await
    .unwrap();
assert!(details.value);
```

## Evaluation Context Transformation

OpenFeature standardizes the evaluation context to include a `targeting_key`, and some other additional arbitrary properties that each provider can use fit for their use case.

For Flipt, we translate the `targeting_key` as the `entityId`, and `custom_fields` as the `context` in Flipt vernacular. You can find the meaning of those two words [here](https://www.flipt.io/docs/reference/evaluation/variant-evaluation) in our API docs.

## Testing

Run `cargo test` to excecute the integration tests.

## Maintainers

- [Raiki Tamura](https://github.com/tamaroning)
