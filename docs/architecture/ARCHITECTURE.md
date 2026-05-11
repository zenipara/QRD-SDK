# Architecture

## Overview

QRD is built around one authoritative Rust core engine with language-specific bindings and a binary container format that supports streaming writes, partial reads, bounded-memory processing, and cross-platform determinism.

The architecture separates responsibilities clearly:

- `core/qrd-core/` implements the core format, schema, writer, reader, encoding, compression, encryption, and ECC logic.
- `core/qrd-ffi/` exposes a stable C-compatible interface for downstream bindings.
- `core/qrd-wasm/` compiles the Rust core to WebAssembly for browser and Node.js runtimes.
- `sdk/*/` packages language-specific bindings that reuse the Rust core through FFI or WASM.

## Architecture Diagram

```
+---------------------------------------------+
| Application / Analytics / ML Layer          |
+----------------------+----------------------+
                       |
+----------------------+----------------------+
| Language SDK Layer (Python, TypeScript, Go, Java, C/C++) |
+----------------------+----------------------+
                       |
+----------------------+----------------------+
| FFI / WASM Interface Layer (`core/qrd-ffi/`, `core/qrd-wasm/`) |
+----------------------+----------------------+
                       |
+----------------------+----------------------+
| Rust Core Engine (`core/qrd-core/`)         |
|  - schema                                   |
|  - writer                                   |
|  - reader                                   |
|  - encoding                                 |
|  - compression                              |
|  - encryption                               |
|  - ecc                                      |
|  - metadata                                 |
+---------------------------------------------+
```

## Streaming Write Pipeline

QRD writes data incrementally in row groups. Each row is buffered until the configured row group size is reached and then flushed as a columnar chunk set.

```
Input rows
    ↓
Row group buffer
    ↓
Columnar transpose
    ↓
Per-column encoding
    ↓
Per-column compression
    ↓
Optional encryption / ECC
    ↓
Row group flush to output stream
```

### Writer Responsibilities

The writer is responsible for:

- validating schema and row shape
- buffering rows until the row group threshold is met
- transposing rows into columnar chunks
- selecting encoding per column
- compressing column chunks independently
- optionally applying encryption and ECC
- writing row group headers, payloads, and footer metadata

This design keeps memory usage bounded by the active row group rather than the total dataset.

## Reader Architecture

QRD readers support multiple modes:

- full file scan
- footer-only metadata inspection
- selective column reads
- row-group streaming iteration

Readers always parse the footer first to discover schema, row group offsets, and metadata. They then seek directly to the required row groups and column chunks.

### Partial Column Reads

Partial reads are enabled by independent column chunk layout. Readers can:

- skip unrelated column payloads
- decrypt only selected chunks when encryption is enabled
- decompress only the requested column chunk

This reduces I/O and memory pressure for analytical queries.

## Schema and Metadata

QRD embeds schema metadata inside the footer and stores a deterministic schema fingerprint in the file header.

Schema metadata includes:

- field names
- logical types
- nullability
- optional per-field metadata
- schema fingerprint

The schema format is deterministic across bindings and the schema ID is used for file validation and compatibility checks.

## Row Group and Column Storage

Each row group contains one chunk per column. Column chunks are self-contained and include:

- encoding ID
- compression ID
- uncompressed and compressed lengths
- null counts and statistics
- payload bytes
- CRC32 checksum

This chunk structure enables independent I/O, compression, and validation.

## Compression and Encoding Layers

QRD separates logical encoding from physical compression.

- Encoding transforms data into a representation optimized for compressibility.
- Compression reduces the encoded bytes for storage efficiency.

Supported encodings include PLAIN, RLE, BIT_PACKED, DELTA_BINARY, DELTA_BYTE_ARRAY, BYTE_STREAM_SPLIT, and DICTIONARY_RLE. Supported compression codecs include NONE, ZSTD, and LZ4.

## Memory Flow

Memory is bounded on both read and write paths:

- Write: row group buffer → column buffers → encoded chunks → compressed payloads
- Read: footer metadata → selected row group → selected column chunk → decoded values

This ensures peak memory is based on row group size and the number of active columns rather than total file size.

## FFI and WASM Layers

### FFI Layer

The FFI layer exposes core functionality to external languages without duplicating format logic:

- schema builder and schema serialization
- writer creation and row ingestion
- reader creation and row/column access
- metadata inspection
- optional ECC and encryption configuration

This keeps bindings thin and aligned with the authoritative Rust core.

### WASM Layer

The WASM target compiles the Rust engine into a portable binary that can run in browsers and Node.js. The TypeScript SDK wraps this runtime and provides a JavaScript-friendly API.

## Extension Points

The architecture reserves explicit extension points for future features:

- new compression codecs
- new encodings
- optional encryption metadata fields
- ECC parity schemes
- schema metadata extensions

The format is designed to keep extensions opt-in and preserve backward compatibility whenever possible.

## Implementation Mapping

Key repository mappings:

- `core/qrd-core/src/schema/` — schema model, serialization, fingerprinting
- `core/qrd-core/src/writer/` — streaming writer and row group flush logic
- `core/qrd-core/src/reader/` — footer parsing, row group seeks, partial reads
- `core/qrd-core/src/encoding/` — logical encoding implementations
- `core/qrd-core/src/compression/` — codec wrappers and adaptive selection
- `core/qrd-core/src/encryption/` — AES-GCM and key metadata handling
- `core/qrd-core/src/ecc/` — Reed-Solomon parity and recovery support
- `core/qrd-ffi/src/` — C-compatible bindings
- `core/qrd-wasm/src/` — WebAssembly target glue
- `sdk/typescript/`, `sdk/python/`, `sdk/go/`, `sdk/java/` — language packaging and examples
