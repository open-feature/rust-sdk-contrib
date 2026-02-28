# Changelog

## [0.6.4](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.6.3...protobuf-v0.6.4) (2025-04-10)


### ✨ New Features

* add sync stream type and context ([#183](https://github.com/open-feature/flagd-schemas/issues/183)) ([9b0ee43](https://github.com/open-feature/flagd-schemas/commit/9b0ee43ecc477e277d41770034fa495ec78838fe))


### 📚 Documentation

* fix url for flags.json ([#182](https://github.com/open-feature/flagd-schemas/issues/182)) ([e840a03](https://github.com/open-feature/flagd-schemas/commit/e840a037dc49801185217f45cc404fc8ff2cd0d4))

## [0.6.3](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.6.2...protobuf-v0.6.3) (2025-01-27)


### ✨ New Features

* add bulk metadata ([#180](https://github.com/open-feature/flagd-schemas/issues/180)) ([9ca6172](https://github.com/open-feature/flagd-schemas/commit/9ca6172cfe54f4025cfa04d5f0c517f95a93f602))

## [0.6.2](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.6.1...protobuf-v0.6.2) (2024-12-05)


### ✨ New Features

* add metadata to the AnyFlag response used for bulk eval ([#176](https://github.com/open-feature/flagd-schemas/issues/176)) ([3acc9e4](https://github.com/open-feature/flagd-schemas/commit/3acc9e44fcd4686318c242248c1621cc3cdd4c41))

## [0.6.1](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.6.0...protobuf-v0.6.1) (2024-08-28)


### 🧹 Chore

* deprecate old protos ([#170](https://github.com/open-feature/flagd-schemas/issues/170)) ([53374a7](https://github.com/open-feature/flagd-schemas/commit/53374a7f79558e8453dc340a4f881b46dabb4eaf))

## [0.6.0](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.5.4...protobuf-v0.6.0) (2024-02-15)


### ⚠ BREAKING CHANGES

* struct for metadata for consistency and power ([#130](https://github.com/open-feature/flagd-schemas/issues/130))

### ✨ New Features

* struct for metadata for consistency and power ([#130](https://github.com/open-feature/flagd-schemas/issues/130)) ([673fc15](https://github.com/open-feature/flagd-schemas/commit/673fc1539bbde0b60f2a706e68714cfe8ff17913))

## [0.5.4](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.5.3...protobuf-v0.5.4) (2023-12-15)


### 🐛 Bug Fixes

* js+dts -&gt; ts in connect build ([#118](https://github.com/open-feature/flagd-schemas/issues/118)) ([ceb0bff](https://github.com/open-feature/flagd-schemas/commit/ceb0bff823a7b3e9b81e2720ac67cbafc324ff65))

## [0.5.3](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.5.2...protobuf-v0.5.3) (2023-10-31)


### ✨ New Features

* add metadata endpoint ([#113](https://github.com/open-feature/flagd-schemas/issues/113)) ([3c19735](https://github.com/open-feature/flagd-schemas/commit/3c1973514a1f087d0514700ed772f5502eb8adfe))

## [0.5.2](https://github.com/open-feature/flagd-schemas/compare/protobuf-v0.5.1...protobuf-v0.5.2) (2023-10-24)


### ✨ New Features

* add better-named evaluation/sync protos ([#109](https://github.com/open-feature/flagd-schemas/issues/109)) ([7b8e75d](https://github.com/open-feature/flagd-schemas/commit/7b8e75d7511ac310e8cb077d0b9a3d4732d24563))


### 🧹 Chore

* fixed typo in the flag evaluation api comment ([c6cc820](https://github.com/open-feature/flagd-schemas/commit/c6cc820975181ec5e0f8d287c34e2692949f8e9e))

## [0.5.1](https://github.com/open-feature/schemas/compare/protobuf-v0.5.0...protobuf-v0.5.1) (2023-10-02)


### 🧹 Chore

* add schema proto description ([#105](https://github.com/open-feature/schemas/issues/105)) ([f3e419c](https://github.com/open-feature/schemas/commit/f3e419c5ea676b6e0a4384b1ba3c9ad43b047eed))

## [0.5.0](https://github.com/open-feature/schemas/compare/protobuf-v0.4.0...protobuf-v0.5.0) (2023-08-22)


### ⚠ BREAKING CHANGES

* add language packaging options for sync proto ([#102](https://github.com/open-feature/schemas/issues/102))

### ✨ New Features

* add language packaging options for sync proto ([#102](https://github.com/open-feature/schemas/issues/102)) ([617c32a](https://github.com/open-feature/schemas/commit/617c32ade300720c3c90138fc697ccb339a4cac4))

## [0.4.0](https://github.com/open-feature/schemas/compare/protobuf-v0.3.1...protobuf-v0.4.0) (2023-07-25)


### ⚠ BREAKING CHANGES

* use ts-proto for node ([#99](https://github.com/open-feature/schemas/issues/99))

### ✨ New Features

* use ts-proto for node ([#99](https://github.com/open-feature/schemas/issues/99)) ([33e127c](https://github.com/open-feature/schemas/commit/33e127c1fc8da11c53d44e843b425065e0844b0b))

## [0.3.1](https://github.com/open-feature/schemas/compare/protobuf-v0.3.0...protobuf-v0.3.1) (2023-07-20)


### ✨ New Features

* introduce flagd evaluation metadata ([#97](https://github.com/open-feature/schemas/issues/97)) ([99db23a](https://github.com/open-feature/schemas/commit/99db23a3a6e4e68010801f45c4126080f7689d1d))

## [0.3.0](https://github.com/open-feature/schemas/compare/protobuf-v0.2.12...protobuf-v0.3.0) (2023-07-10)


### ⚠ BREAKING CHANGES

* migrate to buf managed plugins ([#95](https://github.com/open-feature/schemas/issues/95))

### ✨ New Features

* migrate to buf managed plugins ([#95](https://github.com/open-feature/schemas/issues/95)) ([b582e5d](https://github.com/open-feature/schemas/commit/b582e5d5318d26dad08351b6a40bfed296247add))

## [0.2.12](https://github.com/open-feature/schemas/compare/protobuf-v0.2.11...protobuf-v0.2.12) (2023-03-17)


### ✨ New Features

* introduce selector field to requests ([#89](https://github.com/open-feature/schemas/issues/89)) ([d7ded9d](https://github.com/open-feature/schemas/commit/d7ded9d83d6b66884998764236c452c29ddaa215))

## [0.2.11](https://github.com/open-feature/schemas/compare/protobuf-v0.2.10...protobuf-v0.2.11) (2023-03-13)


### Features

* introduce provider_id to sync service FetchAllFlags ([#87](https://github.com/open-feature/schemas/issues/87)) ([32a5ba7](https://github.com/open-feature/schemas/commit/32a5ba7b4681bff8c998e4010aa997187b7f7d2e))

## [0.2.10](https://github.com/open-feature/schemas/compare/protobuf-v0.2.9...protobuf-v0.2.10) (2023-02-22)


### Features

* fetch all flags rpc for sync server ([#82](https://github.com/open-feature/schemas/issues/82)) ([c42dbbb](https://github.com/open-feature/schemas/commit/c42dbbb2c6cfb91e8e534d17242ee941bdbcad98))

## [0.2.9](https://github.com/open-feature/schemas/compare/protobuf-v0.2.8...protobuf-v0.2.9) (2023-02-07)


### Features

* Introduce service definition for GRPC sync contract ([#78](https://github.com/open-feature/schemas/issues/78)) ([2ba81e3](https://github.com/open-feature/schemas/commit/2ba81e3580a3944b837fe63482993d31b02dd9c0))

## [0.2.8](https://github.com/open-feature/schemas/compare/v0.2.7...v0.2.8) (2022-12-26)


### Bug Fixes

* Fix ruby package to correct namespace ([#68](https://github.com/open-feature/schemas/issues/68)) ([0acbda3](https://github.com/open-feature/schemas/commit/0acbda3cf205915bde8dac4f7d3a6965fcb16c19))

## [0.2.7](https://github.com/open-feature/schemas/compare/v0.2.6...v0.2.7) (2022-12-22)


### Features

* Add ruby support ([#65](https://github.com/open-feature/schemas/issues/65)) ([31a02e4](https://github.com/open-feature/schemas/commit/31a02e447436d91bcbab7ecd057730cfcb560700))

## [0.2.6](https://github.com/open-feature/schemas/compare/v0.2.5...v0.2.6) (2022-12-20)


### Features

* resolve-all functionality ([#62](https://github.com/open-feature/schemas/issues/62)) ([9ca9ee3](https://github.com/open-feature/schemas/commit/9ca9ee3fa8b677c48ec6e859d0b78cc9f2042dfc))

## [0.2.5](https://github.com/open-feature/schemas/compare/v0.2.4...v0.2.5) (2022-12-05)


### Bug Fixes

* doc fix ([#58](https://github.com/open-feature/schemas/issues/58)) ([01f0934](https://github.com/open-feature/schemas/commit/01f09340a2a8e99d51cf875d8325c0174a6e6f91))

## [0.2.4](https://github.com/open-feature/schemas/compare/v0.2.3...v0.2.4) (2022-12-02)


### Miscellaneous Chores

* release ([f8ae2e8](https://github.com/open-feature/schemas/commit/f8ae2e8acdc0ac2db7055e347c35d9f070130a1b))

## [0.2.3](https://github.com/open-feature/schemas/compare/v0.2.2...v0.2.3) (2022-12-02)


### Miscellaneous Chores

* release ([387959e](https://github.com/open-feature/schemas/commit/387959e2d12c6c0707aadde1554282304c1bd5b4))

## [0.2.2](https://github.com/open-feature/schemas/compare/v0.2.2...v0.2.2) (2022-12-02)


### ⚠ BREAKING CHANGES

* Replaced unnamed empty message type ([#46](https://github.com/open-feature/schemas/issues/46))

### Features

* add java_package name specifier ([aac33f6](https://github.com/open-feature/schemas/commit/aac33f63378f2c8f712bbfbd918b61a860a3e865))
* custom PHP schema namespace ([de56d5f](https://github.com/open-feature/schemas/commit/de56d5f0ee18cf34ec13e79091c75e3ea80ca2f7))
* include protocolbuffers plugin for full types generation ([76c70a3](https://github.com/open-feature/schemas/commit/76c70a31a6aceb2a42994e1ae7662ce1ae092fd0))
* support for PHP gRPC client generation ([4ddccc8](https://github.com/open-feature/schemas/commit/4ddccc8f46ceab3177ed19fefa7ba887dbfdfdcc))


### Bug Fixes

* buf ci fix ([#50](https://github.com/open-feature/schemas/issues/50)) ([727c0d7](https://github.com/open-feature/schemas/commit/727c0d7b6735b2712bb0a671c1a83d0e390e189f))
* properly escape php namespace ([302d0fa](https://github.com/open-feature/schemas/commit/302d0fa1f813586d213468d631633611808b6ef1))
* Replaced unnamed empty message type ([#46](https://github.com/open-feature/schemas/issues/46)) ([610e6f5](https://github.com/open-feature/schemas/commit/610e6f5ce566e6a6458ec73bb631a5020989fa61))


### Release-As

* 0.2.2 ([d057b1f](https://github.com/open-feature/schemas/commit/d057b1f433d775fc2d01d1daf136b881ff4e15f1))

## [0.2.2](https://github.com/open-feature/schemas/compare/v0.2.1...v0.2.2) (2022-12-02)


### Release-As

* 0.2.2 ([d057b1f](https://github.com/open-feature/schemas/commit/d057b1f433d775fc2d01d1daf136b881ff4e15f1))

## [0.2.1](https://github.com/open-feature/schemas/compare/v0.2.0...v0.2.1) (2022-12-02)


### Bug Fixes

* buf ci fix ([#50](https://github.com/open-feature/schemas/issues/50)) ([727c0d7](https://github.com/open-feature/schemas/commit/727c0d7b6735b2712bb0a671c1a83d0e390e189f))

## [0.2.0](https://github.com/open-feature/schemas/compare/v0.1.0...v0.2.0) (2022-12-02)


### ⚠ BREAKING CHANGES

* Replaced unnamed empty message type ([#46](https://github.com/open-feature/schemas/issues/46))

### Features

* add java_package name specifier ([aac33f6](https://github.com/open-feature/schemas/commit/aac33f63378f2c8f712bbfbd918b61a860a3e865))
* custom PHP schema namespace ([de56d5f](https://github.com/open-feature/schemas/commit/de56d5f0ee18cf34ec13e79091c75e3ea80ca2f7))


### Bug Fixes

* properly escape php namespace ([302d0fa](https://github.com/open-feature/schemas/commit/302d0fa1f813586d213468d631633611808b6ef1))
* Replaced unnamed empty message type ([#46](https://github.com/open-feature/schemas/issues/46)) ([610e6f5](https://github.com/open-feature/schemas/commit/610e6f5ce566e6a6458ec73bb631a5020989fa61))
