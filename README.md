# QRD-SDK

QRD-SDK is a streaming-first analytical binary container format and a multi-language SDK family for edge-native data pipelines.

QRD is focused on bounded-memory ingestion, columnar partial reads, WASM/browser portability, and deterministic cross-language compatibility.

## Project Positioning

- **Streaming-first analytical container** with row-group-based write semantics.
- **Edge-native and offline-first** for low-memory devices and intermittent connectivity.
- **WASM-capable runtime** for browser and Node.js edge applications.
- **Multi-language SDK family** with one Rust core engine and thin FFI/WASM bindings.
- **Infrastructure-grade design** for reproducible telemetry, analytics, and local inference.

## Architecture Summary

QRD is implemented as a single authoritative Rust engine in `core/qrd-core/`. All language bindings are thin wrappers built on a C-compatible FFI layer or WASM interface.

- `core/qrd-core/`: Core encoding, compression, schema, metadata, reader/writer, ECC, encryption.
- `core/qrd-ffi/`: C-compatible FFI layer for language bindings.
- `sdk/`: Language-specific bindings for Python, TypeScript, Go, Java, and C/C++.

## Benchmark Highlights

QRD is benchmarked against realistic edge and streaming workloads in `core/qrd-core/benches/`.

- **Write throughput** design target: 1–5 GB/s for 1KB row workloads.
- **Read throughput** design target: 2–10 GB/s for full scans.
- **Partial column read** target: 5–20 GB/s when metadata is used to skip unrelated data.
- **Compression**: ZSTD and LZ4 codecs with adaptive selection based on entropy and workload.

## Compression Results

QRD uses column-aware data encodings and chunk-level compression to improve compression ratio while preserving read performance.

- Typical compressions range from **1.5× to 4×** depending on data shape.
- Compression is applied after encoding and before optional encryption.
- Low-latency paths use LZ4; archive-ready paths use ZSTD.

## Edge AI Positioning

QRD is designed for Edge AI and telemetry workloads:

- bounded-memory serialization for streaming sensors
- schema-aware columnar layout for inference feature selection
- offline-first metadata and partial reads for local preprocessing
- browser/WASM support for client-side analytics and telemetry aggregation

## WASM and Browser Positioning

The repository supports a WASM runtime path through `sdk/typescript/` and the Rust core.

- browser-capable container with deterministic format behavior
- low-overhead binding layer for JavaScript/TypeScript
- suitable for lightweight browser analytics, telemetry, and local inference caching

## Supported Languages

- Rust (`core/qrd-core/`)
- Python (`sdk/python/`)
- TypeScript / WASM (`sdk/typescript/`)
- Go (`sdk/go/`)
- Java (`sdk/java/`)
- C/C++ (`sdk/go/qrd.c`, `core/qrd-ffi/`)

## Installation

### Rust

```bash
cargo build --workspace --release
```

### Python

See `docs/SDKS.md` for the current Python binding status and install instructions.

### TypeScript / WASM

See `docs/SDKS.md` for the TypeScript and WASM build path.

### Go and Java

See `docs/SDKS.md` for Go and Java binding notes.

## Quick Examples

### Rust Writer

```rust
use qrd_core::{Schema, SchemaField, LogicalType, Nullability, StreamingWriter};
use std::fs::File;
use std::io::BufWriter;

let schema = Schema::builder()
    .field(SchemaField::new("id", LogicalType::INT64, Nullability::Required))
    .field(SchemaField::new("value", LogicalType::FLOAT64, Nullability::Optional))
    .build();

let file = BufWriter::new(File::create("output.qrd")?);
let mut writer = StreamingWriter::new(file, schema, Default::default())?;
// write rows...
writer.finish()?;
```

### Rust Reader

```rust
use qrd_core::FileReader;
use std::fs::File;
use std::io::BufReader;

let file = BufReader::new(File::open("output.qrd")?);
let reader = FileReader::new(file)?;
for row in reader.rows() {
    // process row
}
```

## Benchmark Dashboard

See `docs/DASHBOARD.md` for dashboard metrics, workload definitions, and reproducibility guidelines.

## Roadmap Summary

The project is organized around:

- stable core format and row-group semantics
- compression and partial read efficiency
- multi-language binding maturity
- security hardening, fuzzing, and audit coverage

See `docs/ROADMAP.md` for the current development plan.

## Ecosystem Vision

QRD-SDK is built for systems that require a portable binary format across devices, browsers, and servers. It is intended to support telemetry ingestion, local analytics, and AI pipelines without requiring centralized infrastructure.

## Repository Layout

- `core/qrd-core/`: format implementation, encodings, reader/writer, benchmark harness
- `core/qrd-ffi/`: C-compatible interface for bindings
- `sdk/`: language bindings and packaging
- `docs/`: architecture, format, security, benchmarks, and governance
- `examples/`, `benches/`, `tests/`: top-level pointers to implementation locations
