# QRD Core - The Engine

The core Rust implementation of the QRD columnar binary format.

## What's Here

```
qrd-core/
├── src/
│   ├── lib.rs              # Main library root
│   ├── error.rs            # Error types
│   ├── schema/             # Schema definitions
│   ├── encoding/           # Encoding algorithms (PLAIN, RLE, etc.)
│   ├── compression/        # Compression (ZSTD, LZ4)
│   ├── encryption/         # AES-256-GCM (optional)
│   ├── ecc/                # Reed-Solomon ECC (optional)
│   ├── writer/             # Streaming writer
│   ├── reader/             # Multi-mode reader
│   ├── footer/             # Footer metadata
│   ├── metadata/           # Column/row group metadata
│   ├── validation/         # CRC32 validation
│   ├── io/                 # Buffered I/O
│   └── utils/              # Helper functions
├── examples/               # Example programs
└── benches/                # Performance benchmarks
```

## Building

```bash
cd core/qrd-core
cargo build --release
```

## Testing

```bash
cargo test --all
```

## Running Examples

```bash
cargo run --example basic_writer --release
cargo run --example basic_reader --release
cargo run --example streaming_read --release
```

## Benchmarking

```bash
cargo bench
```

## Features

Enable optional features:

```bash
cargo build --features "encryption,ecc,threading"
```

- `threading`: Multi-threaded encoding/compression
- `compression`: ZSTD + LZ4 (enabled by default)
- `encryption`: AES-256-GCM support
- `ecc`: Reed-Solomon error correction

## Key Modules

### Schema (`schema/`)
- Type definitions (INT64, FLOAT64, STRING, etc.)
- Nullability (REQUIRED, OPTIONAL, REPEATED)
- Deterministic schema hashing

### Encoding (`encoding/`)
Automatic encoding selection:
- **PLAIN**: Raw data
- **RLE**: Run-length encoding
- **BIT_PACKED**: Bit-level compression
- **DELTA_BINARY**: Sorted integer delta
- **DELTA_BYTE_ARRAY**: Sorted string delta
- **BYTE_STREAM_SPLIT**: Float optimization
- **DICTIONARY_RLE**: Low-cardinality combination
- **PASSTHROUGH**: Pre-encoded data

### Compression (`compression/`)
- **ZSTD**: Archive mode (high compression)
- **LZ4**: Real-time mode (low latency)
- **NONE**: Incompressible data

### Writer (`writer/`)
- Streaming row ingestion
- Configurable row group size
- Automatic column transposition
- Bounded memory usage

### Reader (`reader/`)
- Full file read
- Streaming iteration
- Partial column reads
- Footer-based metadata access

### Validation (`validation/`)
- CRC32 checksums
- Magic byte verification
- Version compatibility checking
- Schema validation

## Architecture

```
User Code
    ↓
Writer API / Reader API
    ↓
Row Buffering / Row Iteration
    ↓
Encoding Pipeline
    ↓
Compression Pipeline
    ↓
I/O (File/Network/Memory)
    ↓
Binary Format
```

## Development Guidelines

### Adding New Encodings

1. Create `encoding/new_encoding.rs`
2. Implement `Encoder` trait
3. Register in `encoding/mod.rs`
4. Add to `EncodingType` enum
5. Update automatic selection algorithm
6. Add test vectors

### Adding Language Bindings

1. Ensure Rust API is stable
2. Create `../qrd-ffi/` C interface
3. Build language wrapper
4. Verify determinism
5. Add cross-SDK tests

### Testing

- Unit tests in each module
- Integration tests in `tests/`
- Golden test vectors in `../../test-vectors/`
- Fuzz tests for encoding/compression
- Determinism tests across SDKs

## Performance Notes

Target throughput (on modern x86_64):
- **Write**: 1-5 GB/s per column
- **Read**: 2-10 GB/s  
- **Compression**: 500MB-2GB/s (ZSTD)

Memory usage:
- O(row\_group\_size) not O(dataset\_size)
- Example: 1M rows @ 1KB/row ≈ 1.3GB per group

Optimization opportunities:
- SIMD for bit-packing
- Parallel row group compression
- Zero-copy reads via mmap
- Streaming decompression

## Status

**Phase 1: Core (In Progress)**
- [x] Schema engine
- [x] Error handling
- [x] Binary format specification
- [x] Encoding framework
- [x] Compression framework
- [ ] Full writer implementation
- [ ] Full reader implementation
- [ ] Golden test vectors
- [ ] Benchmarks
- [ ] Documentation

---

**Built with quality-first principles. Binary correctness is non-negotiable.**
