# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-env-var-v0.1.1...open-feature-env-var-v0.2.0) (2026-01-07)


### ⚠ BREAKING CHANGES

* Update dependencies and fix flaky retry mechanism tests ([#74](https://github.com/open-feature/rust-sdk-contrib/issues/74))

### ✨ New Features

* add renaming mechanism for EnvVarProvider ([#75](https://github.com/open-feature/rust-sdk-contrib/issues/75)) ([b1265ee](https://github.com/open-feature/rust-sdk-contrib/commit/b1265ee741652587a78b7387b6c6b4f9833840bb))
* **flagd:** Cargo features for evaluation modes ([#88](https://github.com/open-feature/rust-sdk-contrib/issues/88)) ([6ba9d48](https://github.com/open-feature/rust-sdk-contrib/commit/6ba9d48422313cba941fd77b99d2dfae06e95324))


### 🧹 Chore

* **deps:** update rust crate cucumber to v0.22.0 ([#85](https://github.com/open-feature/rust-sdk-contrib/issues/85)) ([47bf579](https://github.com/open-feature/rust-sdk-contrib/commit/47bf5799b2e2d98dce89e5872ff4cc3b8333b42e))
* Update dependencies and fix flaky retry mechanism tests ([#74](https://github.com/open-feature/rust-sdk-contrib/issues/74)) ([9b78024](https://github.com/open-feature/rust-sdk-contrib/commit/9b780249584eb1ddfe7ad7f1049c415ff8658234))

## [0.1.1](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-env-var-v0.1.0...open-feature-env-var-v0.1.1) (2025-08-20)


### ✨ New Features

* add rust github actions ([#16](https://github.com/open-feature/rust-sdk-contrib/issues/16)) ([f084cab](https://github.com/open-feature/rust-sdk-contrib/commit/f084cabaa2f8d99d5fdf0488a8da0acd8deac36e))


### 🧹 Chore

* update dependencies and manifest to publish crate. ([#68](https://github.com/open-feature/rust-sdk-contrib/issues/68)) ([f717203](https://github.com/open-feature/rust-sdk-contrib/commit/f717203350d810a3249ff1e1637b6963a03d8418))

## [0.1.0]

### Added

- Initial release of the library.
- Support for resolving feature flags from environment variables.
- Support for parsing environment variables into int, float, string, and bool types.
