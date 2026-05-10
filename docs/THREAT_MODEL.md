# Threat Model

## Scope

This threat model covers the QRD file format parser, reader, and associated runtime behavior for untrusted input.

It assumes the runtime may receive files from external sources, including network, removable media, or browser upload.

## Threat Actors

- **Malicious producer**: crafts a malformed or adversarial QRD file.
- **Corrupted storage**: disk or network errors modify file bytes.
- **Compromised dependency**: a vulnerable compression or crypto library is exploited.

## Trust Model

- file contents are untrusted input.
- the runtime executes in the local process context.
- security relies on parser validation, checksum integrity, and safe code boundaries.

## Threat Surfaces

- header and footer parsing
- row group metadata and offsets
- column chunk encoding/compression boundaries
- schema and metadata deserialization
- ECC recovery and integrity checks
- encryption metadata and authentication
- FFI boundary between Rust and external bindings

## Attack Classes

- malformed header or magic sequence
- footer truncation or incorrect length values
- offset arithmetic overflow or out-of-bounds seeks
- invalid compression payloads and decoder abuse
- CRC bypass through manipulated checksums
- ECC state corruption or inconsistent parity
- authenticated encryption misuse

## Mitigations

- validate magic and version before parsing.
- verify offsets and lengths against file size.
- parse metadata separately from payloads.
- verify CRC32 and footer checksum before decoding.
- reject unknown critical identifiers explicitly.
- use safe Rust code for parsing and avoid unchecked pointer arithmetic.
- audit unsafe blocks and FFI interfaces carefully.

## Security Properties

- separation of metadata parsing and payload decoding.
- explicit failure modes for unsupported features.
- defensive parsing for corner cases.
- documented assumptions about optional metadata.

## Residual Risks

- unsupported codec or encoding IDs remain a compatibility risk.
- bindings may add their own runtime risks beyond the core format.
- dependency vulnerabilities require supply-chain management.

## Recommendations

- keep the core parser minimal and focused.
- review unsafe code and FFI boundaries for each release.
- include threat model updates when format or parser assumptions change.
