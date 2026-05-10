# Architecture

## Overview

QRD is designed as a single authoritative core engine with language-agnostic bindings and a binary format that supports streaming, partial reads, and bounded-memory processing.

## Architecture Diagram

```
+----------------------------+
| Language SDK Layer         |
|  Python  TypeScript  Go    |
|  Java    C/C++            |
+-------------+--------------+
              |
+-------------v--------------+
| FFI / WASM Interface       |
|  core/qrd-ffi/             |
+-------------+--------------+
              |
+-------------v--------------+
| Rust Core Engine           |
|  core/qrd-core/            |
|  - schema                  |
|  - writer                  |
|  - reader                  |
|  - encoding                |
|  - compression             |
|  - encryption              |
|  - ecc                     |
+----------------------------+
```

## Streaming Pipeline

QRD processes input as a stream of rows and emits row groups once the target row group size is reached.

```
Input rows → Row group buffer → Columnar transpose → Encoding → Compression → Optional encryption → Row group flush
```

### Writer

The writer architecture is organized around row-group boundaries:

- ingest rows one at a time
- buffer rows until a row group is complete
- transpose rows into column chunks
- select an encoding per column
- compress chunk payloads
- optionally encrypt and add ECC
- append row group to file stream

This design keeps memory bounded by row group capacity rather than dataset size.

### Reader

The reader architecture supports several modes:

- full file read
- partial column read
- footer-only metadata inspection
- streaming read via row group iteration

Readers parse the footer first, then seek directly to row groups and column chunks. Selective column reads skip unrelated data and avoid unnecessary decompression.

## Compression Pipeline

The compression pipeline is separate from encoding:

- Encode data into a representation that is more compressible
- Select a codec based on data shape and workload
- Compress each column chunk independently
- Store codec metadata in the chunk header

Independent chunk compression enables parallel decompression and partial reads.

## Schema System

The schema system is embedded in the file footer and is deterministic by design.

- field names and types are serialized in a fixed format
- nullability is explicit
- optional metadata is included per field
- schema fingerprint is stored in the header

This schema system provides self-description without an external catalog.

## SDK Architecture

The SDK architecture is layered:

- `core/qrd-core/`: authoritative engine and binary format implementation
- `core/qrd-ffi/`: C-compatible API that exposes core features
- `sdk/*/`: language bindings that call into the FFI or WASM runtime

The bindings are thin wrappers; the Rust engine contains the actual implementation logic.

## FFI Architecture

The FFI layer exposes:

- schema construction and serialization
- writer creation and row ingestion
- reader creation and row or column access
- metadata inspection
- ECC and encryption configuration

Language bindings use the FFI to avoid duplicate format logic.

## WASM Architecture

WASM builds compile the Rust core into a portable module and expose a small runtime API.

- `sdk/typescript/` contains the build and packaging layer
- `qrd-wasm/` is the WebAssembly target in the workspace
- browser and Node.js runtimes reuse the same core implementation

## Memory Flow

Memory is bounded along the write and read paths:

- Write: row group buffer → column buffer → encoded chunk → compressed payload
- Read: row group metadata → chunk payload → decoded row / selected columns

Memory usage remains proportional to row group size and selected columns.

## Zero-Copy Concepts

QRD is designed to minimize copies when possible:

- row group metadata is parsed from the footer without deserializing entire file
- partial reads load only selected column chunks
- readers may map or buffer compressed payloads directly when the runtime supports it

## Future Extensibility

The architecture reserves extension points for:

- new compression codecs
- new encodings
- encryption metadata fields
- ECC parity schemes
- additional schema metadata

The core engine, FFI, and docs are structured to allow incremental extension without format drift.
