## [6.0.2](https://github.com/typescript-tools/rust-implementation/compare/v6.0.1...v6.0.2) (2023-04-12)


### Bug Fixes

* **deps:** update rust crate serde to v1.0.160 ([e6880c2](https://github.com/typescript-tools/rust-implementation/commit/e6880c23ecf2d8b4ada7613e2f9e4d7cc1e2535c))

## [6.0.1](https://github.com/typescript-tools/rust-implementation/compare/v6.0.0...v6.0.1) (2023-04-06)


### Bug Fixes

* **deps:** update dependency axios to v1.3.5 ([88459a4](https://github.com/typescript-tools/rust-implementation/commit/88459a4c81db6bd0498a10e999e57e4ca7633327))

# [6.0.0](https://github.com/typescript-tools/rust-implementation/compare/v5.0.34...v6.0.0) (2023-04-02)


* refactor!: replace anyhow with thiserror ([6ebfdb4](https://github.com/typescript-tools/rust-implementation/commit/6ebfdb49b659a7736725148c96b30069f25c61b9))
* refactor!: use associated types in ConfigurationFile trait ([f26e2b5](https://github.com/typescript-tools/rust-implementation/commit/f26e2b5126360d464d56b091a76a28f233b19aae))


### BREAKING CHANGES

* Library functions no longer expose `anyhow::Result`,
instead exposing `std::error::Error` via thiserror.
* The `ConfigurationFile` trait changed the generic type
into an associated type.

## [5.0.34](https://github.com/typescript-tools/rust-implementation/compare/v5.0.33...v5.0.34) (2023-03-30)


### Bug Fixes

* **deps:** update rust crate clap to v4.2.1 ([013d235](https://github.com/typescript-tools/rust-implementation/commit/013d235f469f0b8ee9b01ea00b9f5fe3bb2fd4d3))

## [5.0.33](https://github.com/typescript-tools/rust-implementation/compare/v5.0.32...v5.0.33) (2023-03-29)


### Bug Fixes

* **deps:** update rust crate clap to v4.2.0 ([cd0b096](https://github.com/typescript-tools/rust-implementation/commit/cd0b0967dc7b5e3cf1a75d07722caef8525c3e1d))

## [5.0.32](https://github.com/typescript-tools/rust-implementation/compare/v5.0.31...v5.0.32) (2023-03-29)


### Bug Fixes

* **deps:** update rust crate serde_json to v1.0.95 ([f37a79a](https://github.com/typescript-tools/rust-implementation/commit/f37a79a722f5894599a0191e359d8e001dfd030e))

## [5.0.31](https://github.com/typescript-tools/rust-implementation/compare/v5.0.30...v5.0.31) (2023-03-29)


### Bug Fixes

* **deps:** update rust crate serde to v1.0.159 ([ef712df](https://github.com/typescript-tools/rust-implementation/commit/ef712dfe6911409d11b48528ea6ceed02cb69431))

## [5.0.30](https://github.com/typescript-tools/rust-implementation/compare/v5.0.29...v5.0.30) (2023-03-28)


### Bug Fixes

* **deps:** update rust crate indoc to v2.0.1 ([9dd74e4](https://github.com/typescript-tools/rust-implementation/commit/9dd74e418b9a196ac91c0f232cccd2aafe4b47dc))

## [5.0.29](https://github.com/typescript-tools/rust-implementation/compare/v5.0.28...v5.0.29) (2023-03-28)


### Bug Fixes

* **deps:** update rust crate clap to v4.1.14 ([58b14bb](https://github.com/typescript-tools/rust-implementation/commit/58b14bb3a169751800410ce616c5c997752a8ae1))

## [5.0.28](https://github.com/typescript-tools/rust-implementation/compare/v5.0.27...v5.0.28) (2023-03-27)


### Bug Fixes

* **deps:** update rust crate clap to v4.1.13 ([38d0c63](https://github.com/typescript-tools/rust-implementation/commit/38d0c63ee43a9a70717d7bab3d0e2d4a12805700))

## [5.0.27](https://github.com/typescript-tools/rust-implementation/compare/v5.0.26...v5.0.27) (2023-03-27)


### Bug Fixes

* **deps:** update rust crate anyhow to v1.0.70 ([83ff329](https://github.com/typescript-tools/rust-implementation/commit/83ff32951a08f55835532d97bff9bd5bd92cdbed))

## [5.0.26](https://github.com/typescript-tools/rust-implementation/compare/v5.0.25...v5.0.26) (2023-03-23)


### Bug Fixes

* **deps:** update dependency rimraf to v4.4.1 ([e8347ff](https://github.com/typescript-tools/rust-implementation/commit/e8347ffb6e6abc6985c22f65cf56a7b408ade11e))

## [5.0.25](https://github.com/typescript-tools/rust-implementation/compare/v5.0.24...v5.0.25) (2023-03-09)


### Bug Fixes

* **deps:** update dependency rimraf to v4.4.0 ([c43ee08](https://github.com/typescript-tools/rust-implementation/commit/c43ee08c30bcc2abe0cd211024c1fa1dc7e79973))

## [5.0.24](https://github.com/typescript-tools/rust-implementation/compare/v5.0.23...v5.0.24) (2023-03-07)


### Bug Fixes

* **deps:** update dependency rimraf to v4.3.1 ([c737d74](https://github.com/typescript-tools/rust-implementation/commit/c737d7437645f0183a2cce6d1095381eb8883bf5))

## [5.0.23](https://github.com/typescript-tools/rust-implementation/compare/v5.0.22...v5.0.23) (2023-03-07)


### Bug Fixes

* **deps:** update rust crate askama to 0.12.0 ([8e327ab](https://github.com/typescript-tools/rust-implementation/commit/8e327ab61136da65194d30b8772c2e9fec4d154a))

## [5.0.22](https://github.com/typescript-tools/rust-implementation/compare/v5.0.21...v5.0.22) (2023-03-04)


### Bug Fixes

* **deps:** update dependency rimraf to v4.3.0 ([dcb4e1f](https://github.com/typescript-tools/rust-implementation/commit/dcb4e1fe3d6cd47d15667e67545590e5922a40b1))

## [5.0.21](https://github.com/typescript-tools/rust-implementation/compare/v5.0.20...v5.0.21) (2023-03-03)


### Bug Fixes

* **deps:** update dependency rimraf to v4.2.0 ([d31f293](https://github.com/typescript-tools/rust-implementation/commit/d31f2936d9dc3ef23a3c78823ce257dfd5e4796b))

## [5.0.20](https://github.com/typescript-tools/rust-implementation/compare/v5.0.19...v5.0.20) (2023-02-23)


### Bug Fixes

* **deps:** update dependency axios to v1.3.4 ([ed1bfab](https://github.com/typescript-tools/rust-implementation/commit/ed1bfab01b2f6e52d3e6190039b360c966d8593e))

## [5.0.19](https://github.com/typescript-tools/rust-implementation/compare/v5.0.18...v5.0.19) (2023-02-14)


### Bug Fixes

* **deps:** update dependency axios to v1.3.3 ([711e78d](https://github.com/typescript-tools/rust-implementation/commit/711e78d99a6a8f30e1f7ab042c0a3667863f1714))

## [5.0.18](https://github.com/typescript-tools/rust-implementation/compare/v5.0.17...v5.0.18) (2023-02-04)


### Bug Fixes

* **deps:** update dependency axios to v1.3.2 ([87f2731](https://github.com/typescript-tools/rust-implementation/commit/87f273104bbe31cd44acf1936b57676093aa2415))

## [5.0.17](https://github.com/typescript-tools/rust-implementation/compare/v5.0.16...v5.0.17) (2023-02-02)


### Bug Fixes

* **deps:** update dependency axios to v1.3.1 ([5f1dd61](https://github.com/typescript-tools/rust-implementation/commit/5f1dd615f72e33573050b6285a2efef679ba9940))

## [5.0.16](https://github.com/typescript-tools/rust-implementation/compare/v5.0.15...v5.0.16) (2023-02-01)


### Bug Fixes

* **deps:** update dependency axios to v1.3.0 ([6f1f685](https://github.com/typescript-tools/rust-implementation/commit/6f1f6853fc1908716955c0891f65a5b7a41cf84d))

## [5.0.15](https://github.com/typescript-tools/rust-implementation/compare/v5.0.14...v5.0.15) (2023-01-31)


### Bug Fixes

* **deps:** update rust crate indoc to v2 ([4719263](https://github.com/typescript-tools/rust-implementation/commit/47192639c13970819e2305f25406d6200e5d7270))

## [5.0.14](https://github.com/typescript-tools/rust-implementation/compare/v5.0.13...v5.0.14) (2023-01-28)


### Bug Fixes

* **deps:** update dependency axios to v1.2.6 ([ad10d98](https://github.com/typescript-tools/rust-implementation/commit/ad10d989947787c990ae54f0190e1914a9bc092d))

## [5.0.13](https://github.com/typescript-tools/rust-implementation/compare/v5.0.12...v5.0.13) (2023-01-27)


### Bug Fixes

* **deps:** update dependency axios to v1.2.5 ([4dc2e07](https://github.com/typescript-tools/rust-implementation/commit/4dc2e07b8da066f1b52d32d8e4e0f07bcfd9cf08))

## [5.0.12](https://github.com/typescript-tools/rust-implementation/compare/v5.0.11...v5.0.12) (2023-01-25)


### Bug Fixes

* **deps:** update dependency axios to v1.2.4 ([58c8595](https://github.com/typescript-tools/rust-implementation/commit/58c85952b8c372df637cf0f1f692c388ad4da223))

## [5.0.11](https://github.com/typescript-tools/rust-implementation/compare/v5.0.10...v5.0.11) (2023-01-24)


### Bug Fixes

* **deps:** update dependency rimraf to v4.1.2 ([efb5e60](https://github.com/typescript-tools/rust-implementation/commit/efb5e6036484a2330e3af8bb44bc394d2be40434))

## [5.0.10](https://github.com/typescript-tools/rust-implementation/compare/v5.0.9...v5.0.10) (2023-01-18)


### Bug Fixes

* **deps:** update dependency rimraf to v4.1.1 ([8cc3c95](https://github.com/typescript-tools/rust-implementation/commit/8cc3c957568ff163ef1746f2bf7de9f1bb4d7556))

## [5.0.9](https://github.com/typescript-tools/rust-implementation/compare/v5.0.8...v5.0.9) (2023-01-18)


### Bug Fixes

* **deps:** update dependency axios to v1.2.3 ([4404f24](https://github.com/typescript-tools/rust-implementation/commit/4404f24e373e4357cc8d9e4f46f4cd3c167db77b))

## [5.0.8](https://github.com/typescript-tools/rust-implementation/compare/v5.0.7...v5.0.8) (2023-01-17)


### Bug Fixes

* **deps:** update dependency rimraf to v4.1.0 ([02ab3ce](https://github.com/typescript-tools/rust-implementation/commit/02ab3ce674c686ae0595f1232d58eaf9821cb477))

## [5.0.7](https://github.com/typescript-tools/rust-implementation/compare/v5.0.6...v5.0.7) (2023-01-16)


### Bug Fixes

* **deps:** update dependency rimraf to v4.0.7 ([d480c2e](https://github.com/typescript-tools/rust-implementation/commit/d480c2e79ec703f2f1e8bb36ce8a6858d4e1a5f5))

## [5.0.6](https://github.com/typescript-tools/rust-implementation/compare/v5.0.5...v5.0.6) (2023-01-15)


### Bug Fixes

* **deps:** update dependency rimraf to v4.0.6 ([3274b7f](https://github.com/typescript-tools/rust-implementation/commit/3274b7fb895f57451fa1cf92da76ddca7f054d26))

## [5.0.5](https://github.com/typescript-tools/rust-implementation/compare/v5.0.4...v5.0.5) (2023-01-15)


### Bug Fixes

* **deps:** update dependency rimraf to v4.0.5 ([08f4252](https://github.com/typescript-tools/rust-implementation/commit/08f42527bbafe2b5153b01e8b1be7fe7b8b72809))

## [5.0.4](https://github.com/typescript-tools/rust-implementation/compare/v5.0.3...v5.0.4) (2023-01-14)


### Bug Fixes

* **deps:** update dependency rimraf to v4 ([5650c4a](https://github.com/typescript-tools/rust-implementation/commit/5650c4a344b2185c73d17a77fb59c9742c2d916f))

## [5.0.3](https://github.com/typescript-tools/rust-implementation/compare/v5.0.2...v5.0.3) (2023-01-14)


### Bug Fixes

* **deps:** migrate to semantic-release-action/next-release-version ([ca788a1](https://github.com/typescript-tools/rust-implementation/commit/ca788a155886debd53aaf71e2edc066c09a2e41f))

## [5.0.2](https://github.com/typescript-tools/rust-implementation/compare/v5.0.1...v5.0.2) (2023-01-14)


### Bug Fixes

* **deps:** update rust crate clap to 4.1.1 ([226b308](https://github.com/typescript-tools/rust-implementation/commit/226b3085e95bce1fb36306e2e881710d946029c9))

## [5.0.1](https://github.com/typescript-tools/rust-implementation/compare/v5.0.0...v5.0.1) (2022-12-29)


### Bug Fixes

* **deps:** update dependency axios to v1.2.2 ([c707fa0](https://github.com/typescript-tools/rust-implementation/commit/c707fa06682c523f98c67bbd71fe868cd7260791))

# [5.0.0](https://github.com/typescript-tools/rust-implementation/compare/v4.3.5...v5.0.0) (2022-12-08)


* fix!: rename function with _exclusive suffix ([aeb54fe](https://github.com/typescript-tools/rust-implementation/commit/aeb54fe5cd802126417b6c0379ea5fa36e5e6076)), closes [#216](https://github.com/typescript-tools/rust-implementation/issues/216)


### BREAKING CHANGES

* rename `transitive_internal_dependency_package_names`
to `transitive_internal_dependency_package_names_exclusive`.

## [4.3.5](https://github.com/typescript-tools/rust-implementation/compare/v4.3.4...v4.3.5) (2022-12-08)


### Bug Fixes

* **deps:** update dependency tar to v6.1.13 ([4a31faa](https://github.com/typescript-tools/rust-implementation/commit/4a31faa2f7c8153e93e7a28ba187627f42b26b7d))

## [4.3.4](https://github.com/typescript-tools/rust-implementation/compare/v4.3.3...v4.3.4) (2022-12-06)


### Bug Fixes

* **deps:** update dependency axios to v1.2.1 ([07f8c91](https://github.com/typescript-tools/rust-implementation/commit/07f8c91925364d23f37ea80a92bbc3a01df28524))

## [4.3.3](https://github.com/typescript-tools/rust-implementation/compare/v4.3.2...v4.3.3) (2022-12-04)


### Bug Fixes

* **ci:** add changelog generation ([eba9f0c](https://github.com/typescript-tools/rust-implementation/commit/eba9f0c9d4e4518caabf719f49ea98dc63dcdacc)), closes [#113](https://github.com/typescript-tools/rust-implementation/issues/113)
