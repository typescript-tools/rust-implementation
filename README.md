# TypeScript Tools 🦀

[![Build Status]](https://github.com/typescript-tools/rust-implementation/actions/workflows/release.yml)

[build status]: https://github.com/typescript-tools/rust-implementation/actions/workflows/release.yml/badge.svg?event=push

The `typescript-tools` are an opinionated collection of utilities for working with
TypeScript monorepos. Read more in the [typescript-tools spec].

[typescript-tools spec]: https://github.com/typescript-tools/spec

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

The following target triples are supported:

- x86_64-unknown-linux-musl
- i686-unknown-linux-musl
- x86_64-apple-darwin
- aarch64-unknown-linux-musl
- aarch64-apple-darwin

The following package managers are supported

- [x] npm
- [ ] yarn
- [ ] pnpm
