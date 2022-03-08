# TypeScript Tools 🦀

[![NPM Package][]](https://npmjs.org/package/@typescript-tools/rust-implementation)
[![Build Status]](https://github.com/typescript-tools/rust-implementation/actions/workflows/ci.yml)
[![semantic-release]](https://github.com/semantic-release/semantic-release)

[npm package]: https://img.shields.io/npm/v/@typescript-tools/rust-implementation.svg
[build status]: https://github.com/typescript-tools/typescript-tools/actions/workflows/ci.yml/badge.svg
[semantic-release]: https://img.shields.io/badge/%20%20%F0%9F%93%A6%F0%9F%9A%80-semantic--release-e10079.svg

The `typescript-tools` are an opinionated collection of utilities for working with
TypeScript monorepos.

## The Problem

Whereas [Lerna] was created for managing JavaScript monorepos, TypeScript monorepos have
additional requirements introduced by the compilation step.

The [original and reference implementation] of the `typescript-tools` is written in
TypeScript. The Rust implementation of the `typescript-tools` optimizes execution
speed.

[lerna]: https://github.com/lerna/lerna
[original and reference implementation]: https://github.com/typescript-tools/typescript-tools

## Goals

The goals of the typescript-tools are to give back the maximum amount of human time
possible; chiefly through stability and aggressive automation.

The Rust implementation aims to minimize the amount of latency added to your monorepo's
workflow.

## Supported Systems

The following operating systems/architectures are supported

- [x] GNU/Linux
  - [x] x86_64
- [x] Mac OS
  - [x] x86_64
  - [ ] aarch64
- [ ] Windows

The following package managers are supported

- [x] npm
- [ ] yarn
