# QRD Roadmap

**Last Updated:** 2026-05-08  
**Focus:** Stability first, features second

## Release Strategy

- **1.0.0**: Core binary format, streaming write/read, encoding, compression
- **1.1.0**: Partial reads, footer-based access
- **1.2.0**: Encryption and ECC
- **2.0.0**: SDK language bindings (Python, TypeScript, Go, Java)

## Phase 1: Core Engine (Current)

### Q2 2026: Foundation

- [x] Repository structure
- [x] Specification v1.0
- [ ] Rust workspace setup
- [ ] Binary format implementation
- [ ] Schema engine
- [ ] Row/column transposition
- [ ] Basic encoding (PLAIN, RLE, BIT_PACKED)

### Q2-Q3 2026: Writer & Reader

- [ ] Streaming writer
- [ ] Full reader
- [ ] Row group flushing
- [ ] Header/footer generation
- [ ] CRC32 validation
- [ ] Golden test vectors

### Q3 2026: Advanced Features

- [ ] All encodings (DELTA_BINARY, DELTA_BYTE_ARRAY, etc.)
- [ ] Compression (ZSTD, LZ4)
- [ ] Partial reads
- [ ] Compression-first footer access
- [ ] Statistics collection
- [ ] Metadata indexing

### Q3-Q4 2026: Polish

- [ ] Benchmarks
- [ ] SIMD optimization
- [ ] Memory profiling
- [ ] Fuzz testing
- [ ] Documentation
- [ ] Example suite

## Phase 2: Encryption & ECC (Q4 2026)

- [ ] AES-256-GCM encryption
- [ ] Per-column keys
- [ ] Reed-Solomon ECC
- [ ] Recovery procedures
- [ ] Encrypted test vectors

## Phase 3: Language Bindings (Q1 2027)

### Python

- [ ] Pure Rust implementation via PyO3
- [ ] Numpy integration
- [ ] Pandas I/O
- [ ] Determinism tests

### TypeScript

- [ ] WASM build
- [ ] Browser support
- [ ] Node.js binding
- [ ] Async I/O

### Go

- [ ] C FFI wrapper
- [ ] CGO bindings
- [ ] io.Reader/Writer support

### Java

- [ ] JNI bindings
- [ ] Maven integration
- [ ] Stream API support

## Phase 4: Ecosystem (Q2 2027+)

### CLI Tools

- [ ] `qrd-write`: Stream JSON → QRD
- [ ] `qrd-read`: QRD → JSON/CSV/Parquet
- [ ] `qrd-inspect`: Metadata viewer
- [ ] `qrd-convert`: Cross-format conversion

### Testing Infrastructure

- [ ] Fuzzer corpus
- [ ] Corruption test suite
- [ ] Determinism validators
- [ ] Cross-SDK compatibility suite

### Documentation

- [ ] Architecture deep-dives
- [ ] Format specification sections
- [ ] Performance tuning guide
- [ ] SDK integration guide

## Non-Goals (Explicitly NOT Building)

- ❌ SaaS multi-tenant system
- ❌ Cloud-first architecture
- ❌ Frontend dashboard
- ❌ User authentication
- ❌ Billing engine
- ❌ Complex microservices

## Success Metrics

### Correctness

- ✓ 100% test coverage for binary format
- ✓ Deterministic encoding verified cross-SDK
- ✓ Zero data loss in corruption tests

### Performance

- ✓ >500MB/s write throughput on typical hardware
- ✓ >1GB/s read throughput
- ✓ O(row-group-size) memory, not O(dataset-size)

### Stability

- ✓ Zero panic/crash bugs
- ✓ Graceful error handling
- ✓ Forward/backward compatible format

## Input Roadmap

Users can request:

1. **Priority shifts** - Move features between phases
2. **New encodings** - Add specialized encoding codecs
3. **Format extensions** - New data types or compression
4. **Binding urgency** - Accelerate specific language SDK

---

**Core Principle:** Right tool for the job. Rust for performance, quality-first before scaling.
