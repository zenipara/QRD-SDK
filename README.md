# QRD - Columnar Binary Data Container SDK

QRD is a production-grade, SDK-first, high-performance binary columnar container format designed for streaming, offline-first, edge-native data processing.

**Core Philosophy:** Rust engine + FFI bindings = single source of truth for all SDKs.

## Project Status

🚀 **Phase 1: Core Engine Implementation**
- Binary format
- Schema engine
- Writer/Reader
- Encoding/Compression
- Integrity validation
- Streaming support

## Key Features

- **Columnar**: Efficient column-oriented storage with row groups
- **Binary**: Deterministic little-endian format with versioning
- **Streaming-First**: Incremental row ingestion with bounded memory
- **Offline-First**: Generate and read locally without cloud dependency
- **Compression-Optimized**: ZSTD + LZ4 with intelligent codec selection
- **Zero-Copy**: Memory-efficient reads and writes
- **Partially Readable**: Footer-based metadata lookup without full file scan
- **Cloud-Optional**: Direct object-storage upload support

## Architecture

```
Rust Core Engine (qrd-core)
  ↓
FFI Layer (qrd-ffi)
  ↓
Language Bindings
  ├─ Python
  ├─ TypeScript/WASM
  ├─ Go
  └─ Java
```

## Repository Structure

```
qrd/
├── README.md
├── LICENSE
├── CHANGELOG.md
├── ROADMAP.md
├── SPECIFICATION.md
├── specs/                    # Format specifications
├── test-vectors/             # Golden test vectors
├── core/
│   ├── qrd-core/            # Rust engine
│   ├── qrd-ffi/             # FFI bindings
│   └── qrd-wasm/            # WASM build
├── sdk/
│   ├── python/
│   ├── typescript/
│   ├── go/
│   └── java/
├── docs/
├── examples/
└── tools/
```

## Quick Start

### Build

```bash
cd core/qrd-core
cargo build --release
```

### Test

```bash
cargo test --all
```

### Run Example

```bash
cargo run --example basic_writer --release
```

## Requirements

- **Rust**: 1.70+
- **Target**: x86_64-unknown-linux-gnu (with cross-platform support)
- **Dependencies**: Minimalist, focused on performance

## Performance Targets

- **Throughput**: GB/s range for local I/O
- **Memory**: O(row-group-size) not O(dataset-size)
- **Compression**: 2-10x depending on data patterns
- **Deterministic**: Identical binary output across all SDKs

## What's NOT Included (Yet)

- ❌ SaaS dashboard
- ❌ Frontend UI
- ❌ Authentication system
- ❌ Billing
- ❌ Cloud orchestration

## Documentation

- [SPECIFICATION.md](./SPECIFICATION.md) - Format semantics
- [specs/binary-layout.md](./specs/binary-layout.md) - Binary format details
- [specs/encoding-spec.md](./specs/encoding-spec.md) - Encoding algorithms
- [specs/compression-spec.md](./specs/compression-spec.md) - Compression rules

## Contributing

This is a foundational project. Quality over speed. All contributions must maintain:
- Binary correctness
- Deterministic output
- Comprehensive testing
- Performance benchmarks

## License

See [LICENSE](./LICENSE)

---

**Built for serious long-term binary data engineering.**
