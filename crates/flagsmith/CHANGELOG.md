# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-19

### Added

- Initial release of the Flagsmith OpenFeature provider
- Support for all five OpenFeature flag types:
  - Boolean flags via `resolve_bool_value`
  - String flags via `resolve_string_value`
  - Integer flags via `resolve_int_value`
  - Float flags via `resolve_float_value`
  - Structured (JSON) flags via `resolve_struct_value`
- Environment-level flag evaluation (without targeting)
- Identity-specific flag evaluation (with targeting key and traits)
- Automatic conversion of OpenFeature context to Flagsmith traits
- Local evaluation mode support (requires server-side key)
- Comprehensive error handling and mapping:
  - `FlagNotFound` for missing flags
  - `ProviderNotReady` for API/network errors
  - `TypeMismatch` for type conversion errors
  - `ParseError` for JSON/value parsing errors
- OpenFeature reason code support:
  - `Static` for environment-level evaluation
  - `TargetingMatch` for identity-specific evaluation
  - `Disabled` for disabled flags
- Configuration options:
  - Custom API URL
  - Request timeout
  - Local evaluation mode
  - Analytics tracking
  - Custom HTTP headers
- Comprehensive unit tests
- Full documentation and examples

### Dependencies

- `open-feature` 0.2.x
- `flagsmith` (local path to Rust SDK)
- `tokio` 1.x
- `async-trait` 0.1.x
- `thiserror` 2.0.x
- `serde_json` 1.0.x
- `tracing` 0.1.x

[0.1.0]: https://github.com/open-feature/rust-sdk-contrib/releases/tag/open-feature-flagsmith-v0.1.0
