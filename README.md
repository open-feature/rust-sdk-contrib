# OpenFeature Rust Contributions

This repository is intended for OpenFeature contributions which are not included in the [OpenFeature SDK](https://github.com/open-feature/go-sdk).

## License

Apache 2.0 - See [LICENSE](./LICENSE) for more information.

## Contribution Guideline

It is welcomed to open an issue to discuss about this guideline if you find potential improvements.

1. Project hierarchy.

   Each contrib should be placed under its own directory under `<repo>/crates`. For example, `flagd` contrib project should be created under `<repo>/crates/flagd` directory.

2. Coding style.

   Please add comments and tests at least for publicly exposed APIs.

   Please refer to [open-feature/rust-sdk/src/lib.rs](https://github.com/open-feature/rust-sdk/blob/main/src/lib.rs) for the Clippy rules.
