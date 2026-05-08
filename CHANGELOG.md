# Changelog

All notable changes to the QRD project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial project scaffolding
- Comprehensive specification (v1.0.0-draft)
- Repository structure
- Rust workspace setup
- Binary format definition
- Schema engine framework
- Writer/reader architecture
- Encoding pipeline
- Compression framework
- Test infrastructure
- Documentation templates

### In Development

- Core writer implementation
- Core reader implementation
- Encoding engines
- Compression engines
- Streaming support

## [Future]

### [1.0.0] - Expected Q3 2026

**First production-ready release**

- Core binary format stable
- Streaming writer complete
- Full and streaming readers
- All encoding types
- ZSTD and LZ4 compression
- CRC32 validation
- Golden test vector suite
- Comprehensive documentation
- Benchmark suite

### [1.1.0] - Expected Q4 2026

- Partial column reads
- Footer-based random access
- Column statistics
- Metadata indexing
- Query pushdown optimization

### [1.2.0] - Expected Q4 2026

- AES-256-GCM encryption
- Per-column encryption keys
- Reed-Solomon ECC
- Encryption test vectors

### [2.0.0] - Expected Q1 2027

- Python SDK (PyO3)
- TypeScript SDK (WASM)
- Go SDK (CGO)
- Java SDK (JNI)
- Determinism cross-SDK tests

---

## Contributor Notes

### Development Practices

1. **Determinism First**: All changes must maintain bit-perfect reproducibility
2. **Binary Correctness**: Format changes require RFC and test vectors
3. **Performance**: Benchmark any encoding/compression changes
4. **Testing**: New features require golden test vectors and fuzz tests
5. **Documentation**: Format changes must update SPECIFICATION.md

### Breaking Changes

Format breaking changes (post-1.0):
- Require major version bump
- 6-month deprecation period
- Documented migration path
- Backward compatibility layer provided
