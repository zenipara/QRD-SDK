# Compatibility

## Compatibility Model

QRD compatibility is built on explicit format versioning, safe schema evolution, and clear reader behavior.

- **Backward compatibility**: newer readers can read older files within the same major format version.
- **Forward compatibility**: readers should ignore unknown optional metadata fields when safe.
- **Schema compatibility**: schema changes are managed through deterministic schema IDs and explicit compatibility rules.

## Version Compatibility Rules

The file header contains a major and minor format version.

- readers may accept files with the same major version and lower or equal minor versions.
- readers must reject higher major versions as unsupported.
- unknown optional footer and metadata fields are ignored when they do not affect required parsing.
- unknown critical identifiers (encoding, compression, row group marker) must be reported as unsupported.

## Schema Compatibility

### Compatible evolution

- adding an optional column is compatible for readers that can ignore unknown fields.
- adding optional metadata fields is compatible.
- adding new compression or encoding IDs is compatible only if readers can safely reject unsupported values.

### Incompatible changes

- renaming a field or changing its type is a breaking change.
- changing nullability from `REQUIRED` to `OPTIONAL` or vice versa is breaking.
- altering the schema serialization layout is breaking.

### Schema IDs

- schema IDs identify exact field layouts.
- readers may use schema IDs for cross-file validation and compatibility checks.
- schema IDs are deterministic and derived from field name, type, and nullability.

## Reader Expectations

Readers should:

- parse the header and footer before processing row groups.
- validate offsets, lengths, and checksums before reading payloads.
- skip unknown optional metadata fields rather than failing when possible.
- fail fast on unsupported critical format values.

## Writer Obligations

Writers should:

- emit stable headers and footers according to `docs/FORMAT_SPEC.md`.
- preserve schema IDs for deterministic compatibility.
- document any optional metadata usage.
- avoid using experimental or unstable codec IDs in stable releases.

## Compatibility Practices

- prefer explicit metadata fields over implicit assumptions.
- keep identifiers stable and reserve ranges for future extensions.
- document compatibility-impacting changes in `docs/FORMAT_SPEC.md`.
- maintain a compatibility matrix alongside release notes when needed.
