# Versioning

## Semantic Versioning Policy

QRD-SDK follows semantic versioning for releases.

- `MAJOR` version: incompatible format or API change.
- `MINOR` version: additive backwards-compatible feature.
- `PATCH` version: bug fixes and stability improvements.

## Format Versioning

The QRD binary format uses a separate file version encoded in the file header.

- `VERSION_MAJOR` changes indicate incompatible binary layout or semantics.
- `VERSION_MINOR` changes indicate additive optional metadata or backward-compatible extensions.

### Reader Behavior

- readers may accept files with the same major version and lower or equal minor versions.
- readers must reject higher major versions as unsupported.
- readers should handle unknown optional fields and metadata conservatively.

## Examples

- adding a new optional footer field: minor version increase.
- introducing a new compression ID that can be skipped safely: minor version increase.
- changing schema serialization layout: major version increase.
- fixing a checksum bug without changing the format: patch release.

## Release Requirements

- update `CHANGELOG.md` with the user-visible change.
- update `docs/ROADMAP.md`, `docs/STABILITY.md`, and `docs/COMPATIBILITY.md` when needed.
- include benchmark or compatibility data for format changes.
- include a migration plan for major or breaking changes.

## Format Compatibility Rules

- unknown optional fields must be ignored when safe.
- unknown critical identifiers must produce a clear unsupported feature error.
- readers must be robust to truncated metadata and invalid offsets.

## Binding Versioning

- SDK bindings should reflect the Rust core behavior and be versioned independently when published.
- binding stability statements should appear in `docs/sdk/SDKS.md`.
