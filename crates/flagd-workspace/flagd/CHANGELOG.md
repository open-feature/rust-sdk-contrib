# Changelog

## [0.1.0](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.8...open-feature-flagd-v0.1.0) (2025-12-23)


### ⚠ BREAKING CHANGES

* flagd, behavioral tests conformance (evaluation and config) ([#76](https://github.com/open-feature/rust-sdk-contrib/issues/76))
* Update dependencies and fix flaky retry mechanism tests ([#74](https://github.com/open-feature/rust-sdk-contrib/issues/74))

### 🐛 Bug Fixes

* extend flagd ci to ofrep, add msrv ([#65](https://github.com/open-feature/rust-sdk-contrib/issues/65)) ([692c588](https://github.com/open-feature/rust-sdk-contrib/commit/692c588bc411ef4f1b8ef1baaca89db9b9df02eb))
* flagd, behavioral tests conformance (evaluation and config) ([#76](https://github.com/open-feature/rust-sdk-contrib/issues/76)) ([d3c4ec4](https://github.com/open-feature/rust-sdk-contrib/commit/d3c4ec4aacfc375c96ca71eec34346d1a7e10eb9))


### ✨ New Features

* **flagd:** Cargo features for evaluation modes ([#88](https://github.com/open-feature/rust-sdk-contrib/issues/88)) ([6ba9d48](https://github.com/open-feature/rust-sdk-contrib/commit/6ba9d48422313cba941fd77b99d2dfae06e95324))
* **flagd:** improve in-process evaluation mode, optimize dependencies ([#91](https://github.com/open-feature/rust-sdk-contrib/issues/91)) ([c62885f](https://github.com/open-feature/rust-sdk-contrib/commit/c62885ff633f4fa01cacf6cd913e0d3f7f8f0de4))


### 🧹 Chore

* **deps:** update rust crate cucumber to v0.22.0 ([#85](https://github.com/open-feature/rust-sdk-contrib/issues/85)) ([47bf579](https://github.com/open-feature/rust-sdk-contrib/commit/47bf5799b2e2d98dce89e5872ff4cc3b8333b42e))
* **deps:** update rust crate testcontainers to 0.26.0 ([#83](https://github.com/open-feature/rust-sdk-contrib/issues/83)) ([b6b60f2](https://github.com/open-feature/rust-sdk-contrib/commit/b6b60f2eb65b2c0889ee8a16b8fbffecc17a056d))
* Update dependencies and fix flaky retry mechanism tests ([#74](https://github.com/open-feature/rust-sdk-contrib/issues/74)) ([9b78024](https://github.com/open-feature/rust-sdk-contrib/commit/9b780249584eb1ddfe7ad7f1049c415ff8658234))

## [0.0.8](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.7...open-feature-flagd-v0.0.8) (2025-07-11)


### 🐛 Bug Fixes

* **deps:** update rust crate lru to 0.16 ([#56](https://github.com/open-feature/rust-sdk-contrib/issues/56)) ([a57f8f9](https://github.com/open-feature/rust-sdk-contrib/commit/a57f8f908a6102a09a2dcb5f56c799c9744c6696))
* response code to open feature error mapping ([#55](https://github.com/open-feature/rust-sdk-contrib/issues/55)) ([02722c0](https://github.com/open-feature/rust-sdk-contrib/commit/02722c064dd90442bcf43308ccc4b4ccc8ce43a1))

## [0.0.7](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.6...open-feature-flagd-v0.0.7) (2025-06-01)


### ✨ New Features

* Add `Send + Sync` trait bounds to error types in flagd `RpcResolver` ([#46](https://github.com/open-feature/rust-sdk-contrib/issues/46)) ([7959cf3](https://github.com/open-feature/rust-sdk-contrib/commit/7959cf35e73722c0d53834729a0beab8a8d3d046))


### 🧹 Chore

* **deps:** update rust crate testcontainers to 0.24.0 ([#42](https://github.com/open-feature/rust-sdk-contrib/issues/42)) ([b24b54d](https://github.com/open-feature/rust-sdk-contrib/commit/b24b54d9c112205c2f41264b51836c2d428594b6))


### 🚀 Performance

* chore: flagd: Update dependencies, reuse reqwest client for better performance, add cargo audit to CI ([#47](https://github.com/open-feature/rust-sdk-contrib/issues/47)) ([b6425f4](https://github.com/open-feature/rust-sdk-contrib/commit/b6425f447bb8e91abaa1ab35cf16a89437d62f47))

## [0.0.6](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.5...open-feature-flagd-v0.0.6) (2025-04-12)


### 🐛 Bug Fixes

* **deps:** update datalogic_rs to 3.0.x latest version ([#39](https://github.com/open-feature/rust-sdk-contrib/issues/39)) ([8c9c747](https://github.com/open-feature/rust-sdk-contrib/commit/8c9c747cd1fcb5a64155433f0c653d7f1d19daa7))
* **deps:** update rust crate lru to 0.14 ([#41](https://github.com/open-feature/rust-sdk-contrib/issues/41)) ([91b9ecd](https://github.com/open-feature/rust-sdk-contrib/commit/91b9ecd9cbdf3bf04957882e217488f4427069e9))


### 🧹 Chore

* striped away the openssl dependency and use rustls instead ([#34](https://github.com/open-feature/rust-sdk-contrib/issues/34)) ([eacf2bd](https://github.com/open-feature/rust-sdk-contrib/commit/eacf2bdc3a3deaf43fb8f086288b527451d3e3c8))

## [0.0.5](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.4...open-feature-flagd-v0.0.5) (2025-03-24)


### 🧹 Chore

* update dependencies ([#33](https://github.com/open-feature/rust-sdk-contrib/issues/33)) ([32b154c](https://github.com/open-feature/rust-sdk-contrib/commit/32b154c5f6da711d850102baaac6686a18f623be))


### 📚 Documentation

* flagd update lib.rs and README to have version agnostic instructions ([#29](https://github.com/open-feature/rust-sdk-contrib/issues/29)) ([af1bd6e](https://github.com/open-feature/rust-sdk-contrib/commit/af1bd6eda1a0b70d85dcf64ba9d29003d4169235))

## [0.0.4](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.3...open-feature-flagd-v0.0.4) (2025-02-20)


### 🐛 Bug Fixes

* increase MSRV to correct version, delete unnecessary include ([#27](https://github.com/open-feature/rust-sdk-contrib/issues/27)) ([4217f8d](https://github.com/open-feature/rust-sdk-contrib/commit/4217f8d88a3208edc08cf04929fc362f627a97fe))

## [0.0.3](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.2...open-feature-flagd-v0.0.3) (2025-02-20)


### 🧹 Chore

* add metadata to cargo ([#24](https://github.com/open-feature/rust-sdk-contrib/issues/24)) ([ec89ef8](https://github.com/open-feature/rust-sdk-contrib/commit/ec89ef8471482bb8164beb343d0e4297127b17b3))
* cargo package include only source code ([#25](https://github.com/open-feature/rust-sdk-contrib/issues/25)) ([a0ec8b9](https://github.com/open-feature/rust-sdk-contrib/commit/a0ec8b93460d01a1a65acd452cb9518b261b3069))

## [0.0.2](https://github.com/open-feature/rust-sdk-contrib/compare/open-feature-flagd-v0.0.1...open-feature-flagd-v0.0.2) (2025-02-20)


### 🐛 Bug Fixes

* **deps:** update rust crate open-feature to 0.2 ([#10](https://github.com/open-feature/rust-sdk-contrib/issues/10)) ([a1c59e1](https://github.com/open-feature/rust-sdk-contrib/commit/a1c59e1de6c4d25b1f13b891acde9569c045b20d))


### ✨ New Features

* add flagd Provider ([#19](https://github.com/open-feature/rust-sdk-contrib/issues/19)) ([a1a8573](https://github.com/open-feature/rust-sdk-contrib/commit/a1a857302e3af47d508866b92976c12d6641ab8f))


### 🧹 Chore

* initialize project skeleton ([#7](https://github.com/open-feature/rust-sdk-contrib/issues/7)) ([6928d06](https://github.com/open-feature/rust-sdk-contrib/commit/6928d062c2b7c2c1f15d9de2fe0ff94e9bab83ec))
