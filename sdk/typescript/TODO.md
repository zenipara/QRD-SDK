# QRD TypeScript SDK TODO

## Current progress
- Implemented binary QRD header, schema serialization, row group layout, footer metadata, and checksum.
- Added optional null value support for schema fields.
- Added regression test for optional null roundtrip.

## Next tasks
- [ ] Add support for multiple row groups and row group sizing.
- [ ] Implement schema metadata pairs in footer serialization.
- [ ] Add compression and alternative encoding IDs beyond plain format.
- [ ] Add partial read/inspect APIs to avoid requiring full row-group decode.
- [ ] Support repeated fields / nested array types.
- [ ] Add more tests for footer inspection, schema validation, and invalid files.
- [ ] Align TypeScript implementation with Rust `qrd-core` schema and footer parser semantics.
