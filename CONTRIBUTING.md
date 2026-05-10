# Contributing to QRD-SDK

## Goals

This repository aims to support infrastructure-grade contributions with clear expectations for correctness, performance, security, and documentation.

## Contribution Process

1. Open an issue describing the proposed change.
2. Reference the relevant docs in `docs/`.
3. Submit a pull request with a clear description and tests.
4. Run the test and benchmark suite for the impacted components.

## Code Style

- follow Rust idioms in `core/qrd-core/`
- keep FFI bindings thin and consistent
- document public APIs and format changes
- avoid marketing language in docs

## Testing Requirements

- add unit tests for new behavior
- update or add integration tests for feature changes
- preserve existing regression coverage
- run `cargo test --package qrd-core` for core changes

## Benchmark Rules

- benchmark changes must include the command and hardware details
- benchmark values should reflect realistic workloads
- avoid synthetic microbenchmarks without context
- when changing encoding/compression, include before/after comparisons

## Documentation

- update the relevant `docs/` file for format or architecture changes
- keep terminology consistent with repository standards
- avoid duplicate or stale documentation

## CI Expectations

- prs should pass the existing CI workflows
- include coverage or fuzz targets when relevant
- document if a change impacts packaging or SDK bindings

## Release Process

- update `CHANGELOG.md` for user-visible changes
- update `docs/ROADMAP.md` and `docs/STABILITY.md` for compatibility-impacting changes
- coordinate major or breaking changes with maintainers

## Security

- changes to encryption, ECC, parser logic, or FFI boundaries require security review
- add tests for security-sensitive code paths
- consult `SECURITY.md` for responsible reporting
