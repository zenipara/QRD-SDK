<div align="center">

```
 ██████  ██████  ██████
██    ██ ██   ██ ██   ██
██    ██ ██████  ██   ██
██ ▄▄ ██ ██   ██ ██   ██
 ██████  ██   ██ ██████
    ▀▀
```

# QRD — Columnar Binary Data Container SDK

**A production-grade, streaming-first, edge-native columnar binary format.**  
Built in Rust. One format. One truth. All languages.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-Phase%202%20Complete-brightgreen.svg)](#project-status)
[![Spec](https://img.shields.io/badge/spec-v1.0.0--draft-yellow.svg)](./SPECIFICATION.md)

</div>

---

## Vision & Problem Statement

Modern data pipelines are built for the cloud: centralized, always-online, and controlled by infrastructure you don't own. When connectivity is unreliable, data is sensitive, or the edge is the only compute you have — the default toolset breaks down.

**QRD was built to solve a different class of problem:**

> How do you store, stream, and read large structured datasets efficiently — on a device, in a CLI tool, in a browser, or embedded in an application — without a server, without a SaaS dependency, and without reinventing a format every time?

Existing solutions make hard tradeoffs. Arrow is an in-memory format, not a file format. Parquet is cloud-optimized and complex to implement correctly. CSV has no schema, no compression, no streaming story. SQLite is a database, not a container format.

QRD is a **columnar binary file container** with a clean specification, a single Rust implementation, and deterministic output across every language binding. It exists for **offline-first tools, edge pipelines, local analytics, and anywhere that portability and reproducibility matter more than managed infrastructure.**

---

## Key Features

| Feature | Description |
|---|---|
| **Columnar Storage** | Row→column transposition per row group for better compression and partial reads |
| **Streaming-First** | Write unbounded row streams with O(row-group-size) memory, not O(dataset-size) |
| **Offline-First** | Generate, read, and validate locally — no cloud dependency required |
| **Deterministic Output** | Identical input → identical binary output across all SDKs and all platforms |
| **Intelligent Encoding** | Automatic encoding selection: RLE, Delta, Dictionary, Bit-Packed, PLAIN |
| **Compression** | ZSTD + LZ4 with entropy-based adaptive codec selection |
| **Encryption** | AES-256-GCM with HKDF key derivation, per-column keys |
| **Error Correction** | Reed-Solomon ECC with configurable parity chunks — survive up to 50% chunk loss |
| **Partial Reads** | Footer-based metadata access; read one column without scanning the whole file |
| **Zero-Copy Reads** | SIMD-accelerated memory-efficient reads and writes |
| **Self-Describing** | Schema embedded in the file — no external catalog required |
| **Forward Compatible** | Versioned format with graceful degradation for unknown encodings |

---

## Architecture Overview

QRD is organized as a single-source-of-truth Rust engine with a clean FFI layer that powers all language bindings. No reimplementation. No format drift between languages.

```
┌─────────────────────────────────────────────────────────┐
│                 Language SDK Layer                       │
│   Python (PyO3)  │  TypeScript (WASM)  │  Go  │  Java   │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│                   FFI Layer (qrd-ffi)                    │
│         C-compatible interface for all bindings          │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│               Rust Core Engine (qrd-core)                │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │  Schema  │  │  Writer  │  │  Reader  │  │ Footer │  │
│  │  Engine  │  │ (stream) │  │ (random) │  │ Parser │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │ Encoding │  │ Compress │  │  Encrypt │  │  ECC   │  │
│  │ Pipeline │  │ ZSTD/LZ4 │  │ AES-GCM  │  │  R-S   │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │  SIMD Utilities + Bit Ops + Validation + I/O     │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

**Design principle:** The Rust engine is the only implementation. Language bindings are thin wrappers over FFI — they share identical binary behavior, identical encoding decisions, and produce bit-perfect identical output.

---

## Data Format Overview

### File Layout

```
┌─────────────────────────┐
│   FILE HEADER (32B)     │  Magic "QRD\x01", version, schema ID, row count
├─────────────────────────┤
│   ROW GROUP 0           │  Columns: encoded → compressed → encrypted
├─────────────────────────┤
│   ROW GROUP 1           │
├─────────────────────────┤
│   ...                   │
├─────────────────────────┤
│   ROW GROUP N           │
├─────────────────────────┤
│   FOOTER                │  Schema, statistics, row group offsets, CRC32
├─────────────────────────┤
│   FOOTER LENGTH (4B)    │
└─────────────────────────┘
```

### Supported Types

QRD supports 20 logical types organized across five categories:

- **Numeric:** `BOOLEAN`, `INT8/16/32/64`, `UINT8/16/32/64`, `FLOAT32`, `FLOAT64`
- **Temporal:** `TIMESTAMP` (µs), `DATE` (days), `TIME` (µs), `DURATION` (µs)
- **Text:** `UTF8_STRING`, `ENUM`, `UUID`
- **Binary:** `BLOB`, `DECIMAL`
- **Composite:** `STRUCT`, `ARRAY`, `ANY`

### Encoding Algorithms

| Encoding | Best For |
|---|---|
| `PLAIN` | Mixed or unpredictable data |
| `RLE` | Repetitive runs |
| `BIT_PACKED` | Booleans, small integers |
| `DELTA_BINARY` | Sorted or monotonic integers |
| `DELTA_BYTE_ARRAY` | Sorted strings |
| `BYTE_STREAM_SPLIT` | Floating-point columns |
| `DICTIONARY_RLE` | Low-cardinality strings/enums |

Encoding is selected automatically based on logical type, cardinality sampling, entropy analysis, and sortedness detection. No manual hints required.

---

## Performance Targets

These are design targets on modern x86_64 hardware with local I/O. Actual results depend on data shape, encoding path, and compression ratio.

| Operation | Target Throughput |
|---|---|
| Write (1KB rows) | 1 – 5 GB/s |
| Read (full file) | 2 – 10 GB/s |
| Read (partial columns) | 5 – 20 GB/s |
| ZSTD Compression | 500 MB – 2 GB/s |
| ZSTD Decompression | 1 – 5 GB/s |
| SIMD acceleration | 2 – 5× over scalar baseline |

Memory usage is bounded by row group size, not dataset size:

```
Max memory ≈ row_group_size × avg_row_bytes + overhead
```

For a 100K-row group at ~100 bytes/row: roughly 10MB peak. The file can be arbitrarily large.

---

## Benchmark

The benchmark suite lives in `core/qrd-core/benches/` and covers all encoding types, compression algorithms, SIMD operations, encryption throughput, and ECC encode/decode performance. Run benchmarks with:

```bash
cargo bench --package qrd-core
```

### Encoding Performance (Design Targets)

| Encoding | Dataset Size | Expected Throughput |
|---|---|---|
| `PLAIN` | 1M integers | > 3 GB/s |
| `RLE` | 1M repetitive | > 5 GB/s |
| `DELTA_BINARY` | 1M sequential | > 4 GB/s |
| `BIT_PACKED` | 1M small ints | > 3.5 GB/s |
| `DICTIONARY_RLE` | 1M low-cardinality | > 2 GB/s |

### Benchmark Comparison: QRD vs Common Alternatives

| Property | QRD | Parquet | Arrow IPC | CSV | SQLite |
|---|---|---|---|---|---|
| **Format type** | Columnar binary | Columnar binary | Columnar binary | Row text | Row B-tree |
| **Streaming writes** | ✅ Native | ⚠️ Requires buffering | ⚠️ Limited | ✅ (no schema) | ⚠️ Limited |
| **Offline-first** | ✅ Yes | ⚠️ Ecosystem-heavy | ⚠️ In-memory focus | ✅ Yes | ✅ Yes |
| **Deterministic output** | ✅ Bit-perfect | ❌ Implementation varies | ❌ Varies | ✅ (trivial) | ❌ Varies |
| **Partial column reads** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No | ⚠️ Per query |
| **Encryption** | ✅ AES-256-GCM | ❌ External | ❌ External | ❌ No | ⚠️ SEE extension |
| **Error correction** | ✅ Reed-Solomon | ❌ No | ❌ No | ❌ No | ❌ No |
| **Cross-language correctness** | ✅ One engine | ❌ Multiple impls | ⚠️ Reference impls | ✅ (trivial) | ✅ One engine |
| **Edge / embedded target** | ✅ Core design goal | ❌ Not a priority | ❌ Not a priority | ✅ (trivial) | ✅ Yes |
| **Schema embedded** | ✅ Always | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes |
| **Single-file portability** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |

---

## Project Status


---

## Repository Structure

```
qrd/
├── README.md                    # This file
├── LICENSE                      # MIT License
├── CHANGELOG.md                 # Version history
├── ROADMAP.md                   # Release milestones
├── SPECIFICATION.md             # Full format specification
├── IMPLEMENTATION_STATUS.md     # Current implementation state
│
├── specs/                       # Detailed format sub-specifications
│   ├── binary-layout.md         # Binary layout and header/footer format
│   ├── encoding-spec.md         # Encoding algorithm details
│   ├── compression-spec.md      # Compression rules and selection
│   ├── schema-spec.md           # Schema serialization and hashing
│   ├── footer-spec.md           # Footer structure and access patterns
│   └── compatibility.md         # Forward/backward compatibility rules
│
├── core/
│   └── qrd-core/                # Rust core engine
│       ├── src/
│       │   ├── schema/          # Type system and schema hashing
│       │   ├── encoding/        # All encoding algorithms
│       │   ├── compression/     # ZSTD, LZ4, entropy detection
│       │   ├── encryption/      # AES-256-GCM implementation
│       │   ├── ecc/             # Reed-Solomon error correction
│       │   ├── writer/          # Streaming writer
│       │   ├── reader/          # Full, partial, and streaming readers
│       │   ├── footer/          # Footer builder and parser
│       │   ├── rowgroup/        # Row group management
│       │   ├── columnar/        # Row→column transposition
│       │   ├── validation/      # CRC32 and corruption detection
│       │   ├── io/              # Buffered I/O utilities
│       │   └── utils/           # SIMD, bit ops, varint
│       ├── benches/             # Criterion benchmarks
│       └── examples/            # Basic reader/writer examples
│
├── sdk/
│   ├── python/                  # Python binding (PyO3) [Phase 3]
│   ├── typescript/              # TypeScript/WASM binding [Phase 3]
│   ├── go/                      # Go binding (CGO) [Phase 3]
│   └── java/                    # Java binding (JNI) [Phase 3]
│
├── test-vectors/                # Golden binary test vectors
├── docs/                        # Architecture docs and guides
├── examples/                    # Cross-language usage examples
└── tools/                       # CLI tools [Phase 4]
```

---

## Quick Start

### Requirements

- Rust 1.70+
- Cargo (comes with Rust)

### Build

```bash
git clone https://github.com/zenipara/QRD-SDK.git
cd qrd-sdk/core/qrd-core
cargo build --release
```

### Run Tests

```bash
cargo test --all
```

### Run Benchmarks

```bash
cargo bench --package qrd-core
```

### Write a QRD File

```rust
use qrd_core::{Schema, SchemaField, LogicalType, Nullability, StreamingWriter};
use std::fs::File;
use std::io::BufWriter;

fn main() -> qrd_core::Result<()> {
    // Define schema
    let schema = Schema::builder()
        .field(SchemaField::new("id", LogicalType::INT64, Nullability::Required))
        .field(SchemaField::new("name", LogicalType::Utf8String, Nullability::Optional))
        .field(SchemaField::new("value", LogicalType::FLOAT64, Nullability::Required))
        .build()?;

    // Open output file
    let file = BufWriter::new(File::create("output.qrd")?);
    let mut writer = StreamingWriter::new(file, schema, Default::default())?;

    // Write rows incrementally — memory stays bounded
    for i in 0..1_000_000 {
        writer.write_row(vec![
            qrd_core::Value::Int64(i),
            qrd_core::Value::Utf8String(format!("row_{}", i)),
            qrd_core::Value::Float64(i as f64 * 1.5),
        ])?;
    }

    writer.finish()?;
    Ok(())
}
```

Run it:

```bash
cargo run --example basic_writer --release
```

### Read a QRD File

```bash
cargo run --example basic_reader --release -- output.qrd
```

### Streaming Read

```bash
cargo run --example streaming_read --release -- output.qrd
```

---

## Mental Model

Think of a QRD file the way you'd think of a streaming-friendly Parquet, but with three key differences:

**1. Write like a stream, read like a database.**  
You push rows in one at a time. QRD internally batches them into row groups, transposes rows into columns, selects the best encoding per column, and flushes compressed chunks to disk. The memory footprint is a function of row group size, not file size. On the read side, the footer gives you random access to any column in any row group without scanning.

**2. The file is self-contained and verifiable.**  
Schema, statistics, offsets, checksums, and optionally error-correction data are all inside the file. You can hand someone a `.qrd` file and they have everything they need to read it — no sidecar, no catalog, no server.

**3. One engine, all languages.**  
The Rust core is the only implementation. Python, TypeScript, Go, and Java bindings call into it via FFI/WASM. This means encoding decisions, compression, and the final byte sequence are identical regardless of which SDK you write with. If you write a file in Python and read it in Go — it just works, and the bytes are the same as if you'd written it in Rust directly.

---

## First Use Case: Edge Telemetry Pipeline

**Scenario:** A fleet of IoT sensors collects timestamped readings (temperature, pressure, vibration) in environments with intermittent connectivity. Each device needs to buffer data locally, compress it efficiently, and ensure data survives partial storage corruption before uploading when a connection becomes available.

**How QRD solves this:**

```
Sensor hardware
    │
    ▼
QRD StreamingWriter (Rust, embedded)
    │  ← rows arrive one at a time
    │  ← encoding: DELTA_BINARY for timestamps, PLAIN for floats
    │  ← compression: LZ4 (speed-first for stream mode)
    │  ← ECC: 4 data chunks + 2 parity chunks per row group
    ▼
sensor_data_2026_05_09.qrd
    │
    ▼ (on reconnect)
Upload to object storage
    │
    ▼
Analytics service reads partial columns
    (only "timestamp" + "temperature" — skips vibration column)
    using footer offset table, without reading the full file
```

Without QRD: you'd need a custom binary format, manual compression, separate corruption handling, and no cross-language guarantee for the analytics side.

With QRD: the file is the protocol.

---

## What QRD is NOT

QRD is a format and SDK. It is explicitly not:

| Not this | Use something else |
|---|---|
| A database or query engine | DuckDB, SQLite, PostgreSQL |
| A SaaS or cloud platform | Snowflake, BigQuery, Databricks |
| An analytics dashboard | Grafana, Metabase, Superset |
| An in-memory columnar format | Apache Arrow |
| A message queue or event bus | Kafka, NATS, Redpanda |
| A user authentication system | Auth0, Keycloak |
| A general serialization protocol | Protocol Buffers, MessagePack, FlatBuffers |

QRD is the right choice when you need to **write structured data to a file, efficiently, with a schema, deterministically, and read it back selectively** — without a server, without a cloud dependency, and without implementing the format yourself in every language you use.

---

## Roadmap



---

## Adoption Narrative

QRD is not trying to replace Parquet or compete with Arrow. It fills a different space:

**Arrow** is for in-memory columnar processing. It is not a file format designed for streaming writes from embedded systems.

**Parquet** is the gold standard for cloud data lakes. Its implementation complexity, Thrift-encoded metadata, and ecosystem dependencies make it impractical to implement correctly from scratch in a resource-constrained environment or to guarantee bit-perfect output across languages without using the same shared implementation.

**QRD** is for engineers who need to:
- Write structured binary data from a Rust/C process, a Python script, or a TypeScript service
- Be confident that the bytes they write in one language are exactly what they'd get in any other
- Read back specific columns efficiently, without loading the whole file
- Do all of this without a server, a cloud account, or a complex ecosystem

The adoption path is incremental: start with the Rust core directly, add the Python SDK when analytics needs arise, add the Go SDK for your backend services. The file format never changes — it just gains more first-class readers.

---

## Spec Governance

The QRD format specification lives in [`SPECIFICATION.md`](./SPECIFICATION.md) and its sub-specs in [`specs/`](./specs/).

### How the spec evolves

- **Patch (1.0.x):** Bug fixes, clarifications, non-breaking editorial changes. No RFC required.
- **Minor (1.x.0):** Additive features (new optional metadata fields, new encoding types, new compression codecs). Requires a spec PR with a written justification and at least one golden test vector demonstrating the new behavior.
- **Major (x.0.0):** Breaking changes to the binary layout, schema serialization, or footer structure. Requires a deprecation period (minimum 6 months post 1.0 stable), a documented migration path, and a backward compatibility layer.

### Contribution to the spec

Spec changes follow the same PR process as code changes. A spec PR must include:
1. A clear description of the problem being solved
2. The exact binary-level change (offsets, byte values, field ordering)
3. At least one updated or new section in `SPECIFICATION.md`
4. A golden test vector that validates the change

Implementations must track the spec version and fail loudly on version mismatches rather than silently degrading.

---

## Contributing

QRD is a foundational engineering project. Quality over velocity. Contributions are welcome, with some firm requirements:

### Requirements for all contributions

- **Binary correctness:** Format changes must include golden test vectors
- **Determinism:** No contribution may introduce non-deterministic output
- **Performance:** Encoding and compression changes must include benchmarks showing before/after
- **Testing:** New features require unit tests and, where applicable, integration tests
- **Documentation:** Format changes must update `SPECIFICATION.md` and relevant files in `specs/`

### How to contribute

1. Fork the repository
2. Create a branch: `git checkout -b feature/your-feature-name`
3. Write code, tests, and benchmarks
4. Open a pull request with a clear description of what changed and why
5. Wait for review — correctness and specification alignment are checked before merging

### Development conventions

- Use `cargo fmt` and `cargo clippy` before committing
- No `unsafe` code without explicit justification in a comment
- All public APIs must be documented
- Prefer `Result<T>` over panics for recoverable errors

### Good first contributions

- Additional golden test vectors for edge cases
- Documentation improvements and architecture explanations
- Example programs demonstrating real use cases
- Performance analysis and profiling write-ups

---

## License & Copyright

```
MIT License

Copyright (c) 2026 NAFAL FATURIZKI

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

See [`LICENSE`](./LICENSE) for the full license text.

---

<div align="center">

**QRD — Built for serious long-term binary data engineering.**

*Developed and maintained by [NAFAL FATURIZKI](https://github.com/nafalfaturizki)*

</div>
