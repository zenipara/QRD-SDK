# Stability

## Stability Guarantees

QRD stability is defined along two axes: format compatibility and implementation behavior.

- schema serialization is deterministic across all bindings
- row group and footer semantics are stable within a format major version
- readers ignore unknown optional metadata when safe to do so
- encoding and compression metadata remain explicit and self-describing
- public API changes in the Rust core are documented and versioned

## Stability Levels

- **Stable:** core format and Rust APIs are intended for production use.
- **Incubating:** SDK bindings and ecosystem tools with limited production guarantees.
- **Experimental:** new optional features and extension points that may change.

## Supported APIs

- Rust core public APIs are the reference surface.
- FFI and binding APIs are wrappers around the Rust engine.
- public format semantics are documented in `docs/FORMAT_SPEC.md`.

## Deprecation Policy

- deprecations are communicated at least one release before removal.
- deprecated features remain readable unless explicitly removed from the format.
- breaking format changes use a major version increment.
- public deprecations appear in `CHANGELOG.md` and release notes.

## Compatibility Policy

- backward compatibility is preserved for known stable format fields.
- forward compatibility is enabled through optional metadata and extension points.
- unknown optional fields are ignored rather than rejected when safe.
- unknown critical identifiers or version mismatches fail fast with a clear error.

## Release Readiness

A feature or API is considered stable when it has:

- been documented in `docs/FORMAT_SPEC.md` or `docs/SDKS.md`
- received regression test coverage
- been validated in at least one benchmark or compatibility test scenario
- been reviewed for security and parsing safety

## Change Control

- format-affecting changes require a spec update and release note.
- binding-affecting changes require corresponding SDK documentation updates.
- compatibility-impacting changes must be reviewed in `docs/COMPATIBILITY.md`.
