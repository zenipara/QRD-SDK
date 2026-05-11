<div align="center">

<img src="https://drive.google.com/uc?export=view&id=1Q_-_J8JKuPwO8t3e6HGfW26rB_ZTkAkH" alt="QRD-SDK Logo" width="180"/>

<br/>

# QRD-SDK

### Streaming-First Analytical Binary Container Format

**Edge-native · WASM-capable · Multi-language · Deterministic**

<br/>

[![CI](https://github.com/zenipara/QRD-SDK/actions/workflows/ci.yml/badge.svg)](https://github.com/zenipara/QRD-SDK/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust Edition](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/Version-1.0.0-blue.svg)](CHANGELOG.md)
[![Docs](https://img.shields.io/badge/Docs-docs.qrd.dev-brightgreen.svg)](https://docs.qrd.dev)
[![Crates.io](https://img.shields.io/badge/crates.io-qrd--core-red.svg)](https://crates.io/crates/qrd-core)

<br/>

[Overview](#-overview) · [Why QRD](#-why-qrd) · [Features](#-features) · [Architecture](#-architecture) · [Binary Format](#-binary-format-specification) · [Type System](#-type-system) · [Encoding](#-encoding-algorithms) · [Compression](#-compression) · [Security](#-security--integrity) · [Quick Start](#-quick-start) · [SDKs](#-multi-language-sdk) · [Benchmarks](#-benchmarks) · [Use Cases](#-use-cases) · [Contributing](#-contributing)

</div>

---

## 📌 Overview

**QRD** (Columnar Row Descriptor) adalah format binary container kolumnar yang dirancang untuk analytical workloads di lingkungan **edge, browser, dan offline**. QRD dibangun dengan prinsip streaming-first — data ditulis secara inkremental dalam row group tanpa perlu membuffer seluruh dataset di memori.

Di balik semua binding multi-bahasa, terdapat satu **Rust core engine** yang menjadi sumber kebenaran tunggal. Setiap bahasa hanya menyediakan lapisan tipis di atas FFI atau WASM, memastikan fidelitas format yang identik di semua platform dan runtime.

```
QRD bukan database. QRD bukan pengganti Parquet.
QRD adalah container format untuk sistem yang membutuhkan:
  - streaming data dari edge ke cloud
  - bounded-memory ingestion di perangkat terbatas
  - analytical columnar reads di browser via WASM
  - deterministic binary output lintas bahasa dan platform
```

---

## 🎯 Why QRD

### Masalah yang Dipecahkan

Format yang ada hari ini memiliki trade-off yang tidak cocok untuk edge-native pipelines:

| Masalah | Format Lama | Solusi QRD |
|---|---|---|
| Parquet butuh buffer dataset penuh | Tidak cocok untuk streaming | Row-group streaming dengan bounded memory |
| Arrow IPC bukan file format persisten | In-memory only | Persistent binary container |
| CSV tidak ada schema, encoding, atau kompresi | Terlalu primitif | Self-describing dengan encoding + kompresi |
| SQLite bukan columnar analytics | Row-oriented | Columnar chunks dengan partial reads |
| Format lain tidak support WASM/browser | Server-only | First-class WASM dan browser support |
| Multiple implementasi menyebabkan drift | Inkonsistensi cross-language | Satu Rust engine, semua bahasa via FFI |

### Prinsip Desain

```
1. Deterministic      → Input identik selalu menghasilkan binary identik, di semua bahasa
2. Streaming-First    → Ingestion tak terbatas tanpa materialisasi dataset penuh
3. Columnar           → Transposisi row→kolom untuk kompresi dan selective read yang efisien
4. Bounded Memory     → Memory bergantung pada row group, bukan ukuran dataset
5. Self-Describing    → Schema embedded di footer; tidak perlu schema registry eksternal
6. Little-Endian      → Semua integer multi-byte dalam format little-endian
7. Footer-Driven      → Metadata di footer untuk random access efisien
8. Checksum-Protected → CRC32 di setiap row group dan footer untuk deteksi korupsi
```

---

## ✨ Features

### Core Format

- **Columnar Storage** — Setiap kolom disimpan independen dalam row group; enables skip I/O untuk kolom yang tidak dibutuhkan
- **Row Group Streaming** — Data ditulis dalam chunk dengan ukuran yang dapat dikonfigurasi; memory proporsional terhadap satu row group, bukan seluruh file
- **Footer-Driven Random Access** — Footer di akhir file menyimpan offset semua row group; reader seek langsung ke data yang dibutuhkan
- **Partial Column Reads** — Pilih subset kolom tanpa decompress atau membaca kolom lainnya
- **Self-Describing Schema** — Schema field name, tipe, dan nullability embedded dalam file footer
- **Deterministic Schema ID** — Fingerprint SHA-256 dari schema disimpan di header untuk validasi cross-file

### Encoding

- **7 algoritma encoding** — PLAIN, RLE, BIT_PACKED, DELTA_BINARY, DELTA_BYTE_ARRAY, BYTE_STREAM_SPLIT, DICTIONARY_RLE
- **Per-column encoding** — Setiap kolom dapat menggunakan encoding yang berbeda sesuai karakteristik datanya
- **SIMD-friendly** — Encoding dirancang untuk layout yang kompatibel dengan instruksi SIMD

### Compression

- **Chunk-level compression** — Setiap column chunk dikompresi independen
- **ZSTD** — Untuk archive-quality storage dengan rasio kompresi tinggi
- **LZ4** — Untuk low-latency streaming dan write-heavy workloads
- **Adaptive selection** — Runtime memilih codec berdasarkan entropi data dan profil workload
- **Parallel decompression** — Independent chunks memungkinkan dekompresi paralel

### Security & Integrity

- **CRC32 checksums** — Per column chunk dan per footer block
- **AES-256-GCM encryption** — Per-kolom dengan key granularity individual
- **HKDF key derivation** — Untuk key management yang aman
- **Reed-Solomon ECC** — Parity chunks untuk recovery dari media yang degraded
- **Parser hardening** — Strict bounds checks; tolak input malformed atau terpotong
- **Audit coverage** — Fuzzing targets untuk header, footer, dan row group parsing

### Multi-Language & Runtime

- **Rust native core** — Implementasi referensi; sumber kebenaran format
- **C-compatible FFI** — Semua bahasa lain menggunakan layer ini
- **WASM target** — Browser dan Node.js runtime via TypeScript SDK
- **Python** via PyO3, **Go** via CGO, **Java** via JNI, **C/C++** langsung via FFI

### Edge & Offline

- **Bounded memory** — Writer: proporsional terhadap satu row group; Reader: proporsional terhadap kolom aktif
- **Offline-first metadata** — Schema dan statistik tersedia tanpa baca payload data
- **Sensor telemetry ingestion** — Dirancang untuk streaming data sensor dengan ingestion unbounded
- **Local inference caching** — Feature selection columnar untuk ML preprocessing di edge

---

## 🏗 Architecture

### Layered Architecture

```
+-------------------------------------------------------------+
|                    Application Layer                        |
|  Your app, analytics pipeline, ML inference, telemetry     |
+---------------------------+---------------------------------+
                            |
+---------------------------v---------------------------------+
|                  Language SDK Layer                         |
|                                                             |
|   +----------+  +------------+  +------+  +--------------+ |
|   |  Python  |  | TypeScript |  |  Go  |  | Java / C/C++ | |
|   |  (PyO3)  |  |   (WASM)   |  |(CGO) |  |  (JNI/FFI)  | |
|   +----+-----+  +-----+------+  +--+---+  +------+-------+ |
+--------|--------------|-----------|-----------------+-------+
         |              |           |                 |
+--------v--------------v-----------v-----------------v-------+
|              FFI / WASM Interface Layer                      |
|                    core/qrd-ffi/                             |
|         core/qrd-wasm/  (WebAssembly target)                 |
+---------------------------+---------------------------------+
                            |
+---------------------------v---------------------------------+
|                  Rust Core Engine                            |
|                   core/qrd-core/                             |
|                                                             |
|  +----------+  +----------+  +----------+  +------------+  |
|  |  Schema  |  |  Writer  |  |  Reader  |  |  Encoding  |  |
|  +----------+  +----------+  +----------+  +------------+  |
|  +------------+  +------------+  +-----+  +-------------+  |
|  |Compression |  | Encryption |  | ECC |  |  Metadata   |  |
|  +------------+  +------------+  +-----+  +-------------+  |
+-------------------------------------------------------------+
```

### Streaming Write Pipeline

```
Input Row
    |
    v
[Row Buffer per Group]
    |  [buffer full -> flush]
    v
[Columnar Transpose]
    |
    v
[Per-Column Encoding]
    PLAIN / RLE / DELTA_BINARY
    BIT_PACKED / DICT_RLE / etc.
    |
    v
[Per-Chunk Compression]
    ZSTD  /  LZ4
    |
    v
[AES-256-GCM Encryption]  <- optional
    |
    v
[Reed-Solomon ECC]         <- optional
    |
    v
[Row Group Flush -> File Stream]
```

### Read Modes

```
File
 |
 +-- Footer Parse (always first)
 |       Schema, Row group offsets, Statistics, Checksums
 |
 +-- [Mode 1] Full Scan
 |       Iterate all row groups sequentially
 |
 +-- [Mode 2] Partial Column Read
 |       Seek directly to requested column chunks
 |       Skip unrelated data entirely
 |
 +-- [Mode 3] Row Group Projection
 |       Select specific row groups by range or predicate
 |
 +-- [Mode 4] Footer-Only Inspection
         Schema + statistics without loading any payload
```

### Memory Bounds

```
Writer memory  ~=  row_group_size x avg_row_size
Reader memory  ~=  selected_columns x active_row_groups
```

Memory tidak pernah bergantung pada ukuran total file.

---

## 📄 Binary Format Specification

### File Layout

```
+----------------------------------------+
|           FILE HEADER (32 bytes)       |
+----------------------------------------+
|           ROW GROUP 0                  |
|  +------------------------------------+|
|  | Row Group Header                   ||
|  +------------------------------------+|
|  | Column Chunk 0  [enc|comp|crc32]   ||
|  | Column Chunk 1  [enc|comp|crc32]   ||
|  | ...                                ||
|  | Column Chunk N  [enc|comp|crc32]   ||
|  +------------------------------------+|
|  | [ECC Parity Chunks - optional]     ||
|  +------------------------------------+|
|  | Row Group Footer                   ||
|  +------------------------------------+|
+----------------------------------------+
|           ROW GROUP 1 ... N            |
+----------------------------------------+
|           FILE FOOTER                  |
|  [Schema | Offsets | Stats | CRC32]    |
+----------------------------------------+
|           FOOTER_LENGTH (4 bytes U32)  |
+----------------------------------------+
```

### File Header (32 bytes)

```
Offset  Size  Type    Field             Keterangan
------  ----  ------  ----------------  ----------------------------------------
0       4     U32LE   MAGIC             0x51 0x52 0x44 0x01  ("QRD\1")
4       2     U16LE   VERSION_MAJOR     Major format version
6       2     U16LE   VERSION_MINOR     Minor format version (additive changes)
8       4     U32LE   SCHEMA_ID         4 bytes pertama dari SHA-256 schema
12      4     U32LE   CREATED_AT        Unix timestamp (seconds)
16      4     U32LE   TOTAL_ROW_COUNT   Total logical rows dalam file
20      4     U32LE   COLUMN_COUNT      Jumlah kolom dalam schema
24      4     U32LE   ROW_GROUP_SIZE    Target rows per row group
28      4     U32LE   FLAGS             Reserved; readers harus ignore unknown bits
```

### Column Chunk Layout

```
Offset   Size  Type    Field               Keterangan
-------  ----  ------  ------------------  -----------------------------------------
0        1     U8      ENCODING_ID         ID algoritma encoding
1        1     U8      COMPRESSION_ID      ID codec kompresi
2        4     U32LE   UNCOMPRESSED_LEN    Ukuran payload sebelum kompresi
6        4     U32LE   COMPRESSED_LEN      Ukuran payload setelah kompresi
10       4     U32LE   NULL_COUNT          Jumlah nilai null dalam chunk
14       4     U32LE   DISTINCT_COUNT      Jumlah nilai unik (statistik)
18       B     BYTES   PAYLOAD             Data encoded + compressed
18+B     4     U32LE   CRC32               Checksum payload uncompressed
```

### File Footer Structure

```
Footer Content
--------------------------------------------------
[version: U16LE]
[schema_length: U32LE]
  [schema_version: U16LE]
  [field_count: U16LE]
  For each field:
    [name_len: U16LE]
    [name: UTF-8 bytes]
    [logical_type_id: U8]
    [nullability_id: U8]
    [metadata_count: U16LE]
    For each metadata entry:
      [key_len: U16LE]  [key: UTF-8]
      [value_len: U16LE]  [value: UTF-8]
[row_group_count: U32LE]
[row_group_offsets: U64LE x N]
[statistics_flag: U8]
[statistics_length: U32LE]
[statistics_bytes]
[metadata_length: U32LE]
[metadata_bytes]
[checksum: U32LE]           <- CRC32 seluruh footer
--------------------------------------------------
[FOOTER_LENGTH: U32LE]      <- 4 bytes terakhir file
```

**Footer Parsing Protocol:**
1. Seek ke `file_size - 4`; baca `FOOTER_LENGTH`
2. Seek ke `file_size - 4 - FOOTER_LENGTH`
3. Parse footer; validasi CRC32
4. Load schema, offsets, dan statistik
5. Baru akses row group berdasarkan offset

---

## 🗃 Type System

### Numeric Types

| Type | Bytes | Range | Physical Representation |
|---|---|---|---|
| `BOOLEAN` | 1 | true / false | Bit-packed |
| `INT8` | 1 | -128 ... 127 | Signed byte |
| `INT16` | 2 | -32,768 ... 32,767 | Signed short, LE |
| `INT32` | 4 | -2^31 ... 2^31-1 | Signed int, LE |
| `INT64` | 8 | -2^63 ... 2^63-1 | Signed long, LE |
| `UINT8` | 1 | 0 ... 255 | Unsigned byte |
| `UINT16` | 2 | 0 ... 65,535 | Unsigned short, LE |
| `UINT32` | 4 | 0 ... 2^32-1 | Unsigned int, LE |
| `UINT64` | 8 | 0 ... 2^64-1 | Unsigned long, LE |
| `FLOAT32` | 4 | IEEE 754 single | 4-byte float, LE |
| `FLOAT64` | 8 | IEEE 754 double | 8-byte float, LE |

### Temporal Types

| Type | Bytes | Format | Contoh |
|---|---|---|---|
| `TIMESTAMP` | 8 | Unix microseconds (INT64) | `1609459200000000` |
| `DATE` | 4 | Days since 1970-01-01 (INT32) | `18628` (2021-01-01) |
| `TIME` | 8 | Microseconds since 00:00:00 (INT64) | `43200000000` (12:00 PM) |
| `DURATION` | 8 | Microseconds (INT64) | `3600000000` (1 jam) |

### Text & Binary Types

| Type | Format | Max Size | Catatan |
|---|---|---|---|
| `UTF8_STRING` | Variable length | 2 GB per string | Length-prefixed |
| `ENUM` | UTF-8 dengan index | 65,535 nilai | Dictionary encoded |
| `UUID` | 16 bytes raw | 128-bit | Big-endian byte order |
| `BLOB` | Variable length | 2 GB per blob | Opaque binary |
| `DECIMAL` | Variable length | Arbitrary precision | Exact numeric |

### Composite Types

| Type | Deskripsi | Status |
|---|---|---|
| `STRUCT` | Named field set yang fixed | Planned |
| `ARRAY` | Homogeneous repeated elements | Planned |
| `ANY` | Schema validation disabled | Planned |

### Nullability

| Value | Semantik | Null Bitmap |
|---|---|---|
| `REQUIRED` | Tidak ada nilai null diizinkan | Tidak ada |
| `OPTIONAL` | Bisa mengandung null | Present |
| `REPEATED` | 0 atau lebih elemen per row | Present |

---

## ⚙️ Encoding Algorithms

Encoding diterapkan **sebelum** kompresi. Setiap column chunk memiliki encoding ID yang disimpan dalam header chunk.

### PLAIN

Nilai disimpan dalam bentuk serialized mentah. Baseline untuk semua tipe.

```
[value_0][value_1][value_2]...[value_N]
```

### RLE (Run-Length Encoding)

Nilai berulang disimpan sebagai pasangan `(run_length, value)`. Efektif untuk data low-cardinality atau data yang sudah diurutkan.

```
(5, "active") -> "active" berulang 5 kali
(3, 42)       -> 42 berulang 3 kali
```

### BIT_PACKED

Integer kecil dan boolean dikemas rapat dalam bit sequences. Mengurangi storage overhead secara signifikan untuk nilai yang memiliki bit width kecil.

```
8 boolean values dikemas dalam 1 byte
4-bit integers: 2 nilai per byte
```

### DELTA_BINARY

Menyimpan selisih antara nilai integer berurutan. Sangat efektif untuk timestamp, ID monoton, dan sequence data.

```
[100, 102, 105, 109] -> [100, +2, +3, +4]
```

### DELTA_BYTE_ARRAY

Menyimpan prefix yang sama dari byte arrays berturutan sebagai (prefix_length, suffix). Efektif untuk string yang memiliki prefix serupa seperti URL, path, dan kolom kategorikal panjang.

```
["https://example.com/api/v1", "https://example.com/api/v2"]
-> prefix_len=26, suffix=["v1", "v2"]
```

### BYTE_STREAM_SPLIT

Menyusun ulang bytes floating-point ke dalam stream terpisah per byte position. Meningkatkan compressibility data float secara signifikan.

```
Float bytes sebelum:
  [f0_b0, f0_b1, f0_b2, f0_b3, f1_b0, f1_b1, f1_b2, f1_b3]

Setelah split:
  stream-0: [f0_b0, f1_b0, ...]
  stream-1: [f0_b1, f1_b1, ...]
  stream-2: [f0_b2, f1_b2, ...]
  stream-3: [f0_b3, f1_b3, ...]
```

### DICTIONARY_RLE

Membangun dictionary nilai unik, lalu encode setiap nilai sebagai index RLE. Optimal untuk kolom string berulang seperti kategori, status, dan kode.

```
Dictionary: {0: "active", 1: "inactive", 2: "pending"}
Data:       (3, 0), (2, 1), (1, 2)   <- (count, dict_index)
```

### Encoding Selection Guide

| Tipe Data | Encoding Rekomendasi |
|---|---|
| Integer monoton / timestamp | `DELTA_BINARY` |
| Integer kecil / boolean | `BIT_PACKED` |
| String berulang / kategori | `DICTIONARY_RLE` |
| Float / double | `BYTE_STREAM_SPLIT` |
| String dengan prefix serupa | `DELTA_BYTE_ARRAY` |
| Data dengan run panjang | `RLE` |
| Data acak / BLOB | `PLAIN` |

---

## 🗜 Compression

### Pipeline Kompresi

```
column values
      |
      v  [Encoding]
encoded bytes
      |
      v  [Compression]
compressed payload
      |
      v  [Encryption - optional]
final chunk payload
```

Kompresi diterapkan **setelah** encoding dan **sebelum** enkripsi opsional.

### Codec yang Didukung

| Codec | ID | Use Case | Trade-off |
|---|---|---|---|
| `NONE` | 0x00 | Testing, data sudah terkompresi | Tanpa overhead CPU |
| `LZ4` | 0x01 | Streaming, low-latency writes | Kompresi ringan, sangat cepat |
| `ZSTD` | 0x02 | Archive, analytics storage | Rasio tinggi, sedikit lebih lambat |
| `GZIP` | 0x03 | Reserved untuk kompatibilitas | Belum diimplementasi |

### Chunk-Level Independence

Setiap column chunk dikompresi **secara independen**:

```
Row Group
+-- Column 0 chunk  [ZSTD]   -> dapat didekompresi sendiri
+-- Column 1 chunk  [LZ4]    -> dapat didekompresi sendiri
+-- Column 2 chunk  [NONE]   -> dapat didekompresi sendiri
+-- Column N chunk  [ZSTD]   -> dapat didekompresi sendiri
```

Ini memungkinkan partial reads, parallel decompression, dan per-column codec selection.

### Panduan Pemilihan Codec

```
Streaming / write-heavy   -> gunakan LZ4
Archive / analytics       -> gunakan ZSTD
Sudah compressed (JPEG)   -> gunakan NONE
```

---

## 🔐 Security & Integrity

### CRC32 Integrity Verification

QRD memvalidasi integritas pada dua level:

```
Level 1: Per column chunk
  CRC32(uncompressed_payload) disimpan di column chunk header
  Reader verifikasi sebelum decode

Level 2: Per file footer
  CRC32(footer_content) disimpan sebagai 4 bytes terakhir footer
  Reader verifikasi sebelum parse metadata
```

Reader **harus** menolak file dengan CRC mismatch.

### AES-256-GCM Encryption

```
Per-column encryption dengan granularity key individual:

column_chunk
  +-- NONCE (12 bytes)
  +-- AUTH_TAG (16 bytes)
  +-- KEY_ID (optional, untuk multi-key setup)
  +-- ENCRYPTED_PAYLOAD

Footer menyimpan:
  +-- encryption_algorithm_id
  +-- key_derivation_metadata (HKDF params)
```

Enkripsi hanya pada kolom sensitif; kolom lain tetap plaintext. Auth tag memastikan integritas payload terenkripsi.

### Reed-Solomon ECC

```
Row Group dengan ECC:
  +-- DATA_CHUNKS   (column chunk 0..N)
  +-- PARITY_CHUNKS (derived dari data chunks)
  +-- Row Group Footer

Recovery: chunks yang hilang atau korup dapat di-reconstruct
          jika jumlah parity chunks cukup
```

Cocok untuk penyimpanan di media degraded, transmisi kanal yang tidak reliable, dan cold storage jangka panjang.

### Parser Hardening

- Strict bounds check pada semua input eksternal
- Tolak header/footer malformed atau terpotong
- Fail-fast pada encoding/compression ID yang tidak dikenal
- Semua `unsafe` Rust didokumentasi dan diaudit
- Fuzz targets untuk header, footer, dan row group parsing

---

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) toolchain (2021 Edition atau lebih baru)
- `cargo` tersedia di PATH

### Clone & Build

```bash
git clone https://github.com/zenipara/QRD-SDK.git
cd QRD-SDK
cargo build --workspace --release
```

### Run Tests

```bash
# Core engine tests
cargo test --package qrd-core

# Full workspace
cargo test --workspace
```

### Run Benchmarks

```bash
# Semua benchmark
cargo bench --package qrd-core

# Benchmark spesifik
cargo bench --package qrd-core -- encode
cargo bench --package qrd-core -- streaming
cargo bench --package qrd-core -- compression
```

### Validasi Build

```bash
./scripts/validate.sh --mode=standard
```

---

## 💻 Code Examples

### Rust — Write

```rust
use qrd_core::{Schema, SchemaField, LogicalType, Nullability, StreamingWriter, WriterConfig};
use std::fs::File;
use std::io::BufWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = Schema::builder()
        .field(SchemaField::new("id",        LogicalType::INT64,       Nullability::Required))
        .field(SchemaField::new("name",      LogicalType::UTF8_STRING, Nullability::Optional))
        .field(SchemaField::new("score",     LogicalType::FLOAT64,     Nullability::Optional))
        .field(SchemaField::new("timestamp", LogicalType::TIMESTAMP,   Nullability::Required))
        .build()?;

    let config = WriterConfig {
        row_group_size: 50_000,
        compression: Compression::Zstd,
        ..Default::default()
    };

    let file = BufWriter::new(File::create("output.qrd")?);
    let mut writer = StreamingWriter::new(file, schema, config)?;

    for i in 0..1_000_000u64 {
        writer.write_row(vec![
            Value::Int64(i as i64),
            Value::String(format!("user_{}", i)),
            Value::Float64(100.0 - (i % 100) as f64),
            Value::Timestamp(chrono::Utc::now().timestamp_micros()),
        ])?;
    }

    // Wajib dipanggil -- finalize footer dan flush
    writer.finish()?;
    Ok(())
}
```

### Rust — Read

```rust
use qrd_core::FileReader;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = FileReader::new(BufReader::new(File::open("output.qrd")?))?;

    println!("Schema: {:?}", reader.schema());
    println!("Row count: {}", reader.row_count());

    for row in reader.rows() {
        let row = row?;
        // process row...
    }
    Ok(())
}
```

### Rust — Partial Column Read

```rust
// Hanya baca kolom "id" dan "score" (index 0 dan 2)
// Kolom "name" dan "timestamp" sepenuhnya di-skip
let columns = reader.read_columns(&[0, 2])?;

for (id, score) in columns[0].iter().zip(columns[1].iter()) {
    println!("{}: {}", id, score);
}
```

### Rust — Verify Integrity

```rust
let mut reader = FileReader::new(BufReader::new(File::open("output.qrd")?))?;

match reader.verify() {
    Ok(())  => println!("File valid"),
    Err(e)  => eprintln!("Corrupted: {}", e),
}
```

### Python

```python
import qrd

schema = (qrd.SchemaBuilder()
    .add_field("id",        qrd.FieldType.INT64,     qrd.Nullability.REQUIRED)
    .add_field("name",      qrd.FieldType.STRING,    qrd.Nullability.OPTIONAL)
    .add_field("score",     qrd.FieldType.FLOAT64,   qrd.Nullability.OPTIONAL)
    .add_field("timestamp", qrd.FieldType.TIMESTAMP, qrd.Nullability.REQUIRED)
    .build())

# Write
writer = qrd.FileWriter("output.qrd", schema)
for i in range(100_000):
    writer.write_row({
        "id": i, "name": f"user_{i}",
        "score": 100.0 - (i % 100),
        "timestamp": int(time.time() * 1_000_000),
    })
writer.finish()

# Read
reader = qrd.FileReader("output.qrd")
print(f"Schema: {reader.schema()}")
print(f"Rows: {reader.row_count()}")

for row in reader.rows():
    print(row)

# Partial column read
columns = reader.read_columns(["id", "score"])
```

### TypeScript / JavaScript

```typescript
import * as qrd from 'qrd-sdk';

const schema = new qrd.SchemaBuilder()
    .addField("id",    qrd.FieldType.INT64,   qrd.Nullability.REQUIRED)
    .addField("name",  qrd.FieldType.STRING,  qrd.Nullability.OPTIONAL)
    .addField("score", qrd.FieldType.FLOAT64, qrd.Nullability.OPTIONAL)
    .build();

// Write
const writer = new qrd.FileWriter("output.qrd", schema);
for (let i = 0; i < 10_000; i++) {
    writer.writeRow({ id: i, name: `user_${i}`, score: 100 - (i % 100) });
}
writer.finish();

// Read
const reader = new qrd.FileReader("output.qrd");
for (const row of reader.rows()) { console.log(row); }

// Browser / WASM: inspeksi metadata tanpa load payload
const meta = qrd.inspectFooter(buffer);
console.log("Columns:", meta.schema.fields.map(f => f.name));
```

### Go

```go
package main

import (
    "fmt"
    qrd "github.com/zenipara/QRD-SDK/sdk/go"
)

func main() {
    schema := qrd.NewSchemaBuilder().
        AddField("id",    qrd.FieldTypeInt64,   qrd.NullabilityRequired).
        AddField("name",  qrd.FieldTypeString,  qrd.NullabilityOptional).
        AddField("score", qrd.FieldTypeFloat64, qrd.NullabilityOptional).
        Build()

    writer, _ := qrd.NewFileWriter("output.qrd", schema)
    defer writer.Close()

    for i := 0; i < 100_000; i++ {
        writer.WriteRow(map[string]interface{}{
            "id": int64(i), "name": fmt.Sprintf("user_%d", i),
            "score": 100.0 - float64(i%100),
        })
    }
    writer.Finish()

    reader, _ := qrd.NewFileReader("output.qrd")
    defer reader.Close()
    fmt.Printf("Rows: %d\n", reader.RowCount())
}
```

### Best Practices

```rust
// Selalu panggil finish() -- footer tidak ditulis tanpa ini
writer.finish()?;

// Pilih row group size yang sesuai workload:
//   Memory terbatas:  10_000 rows
//   Standar:          50_000 rows
//   Batch archive:   500_000 rows

// Gunakan batch writes untuk performa optimal
let mut batch = Vec::with_capacity(10_000);
for row in data {
    batch.push(row);
    if batch.len() == 10_000 {
        writer.write_rows(&batch)?;
        batch.clear();
    }
}

// Gunakan partial reads bila hanya butuh sebagian kolom
let cols = reader.read_columns(&["id", "timestamp"])?;
```

---

## 🌐 Multi-Language SDK

### Status SDK

| Language | Path | Mekanisme | Package | Status |
|---|---|---|---|---|
| **Rust** | `core/qrd-core/` | Native | `qrd-core` (crates.io) | Stable / Reference Implementation |
| **Python** | `sdk/python/` | PyO3 | `qrd-sdk` (PyPI) | Stable |
| **TypeScript** | `sdk/typescript/` | WASM | `qrd-sdk` (npm) | Stable |
| **Go** | `sdk/go/` | CGO | `github.com/zenipara/QRD-SDK/sdk/go` | Stable |
| **Java** | `sdk/java/` | JNI/JNA | Maven artifact | Stable |
| **C/C++** | `core/qrd-ffi/` | C FFI | Header + static lib | Stable |

### Instalasi per Bahasa

**Rust**
```bash
cargo build --workspace --release
# Atau sebagai dependency: qrd-core = "0.1"
```

**Python**
```bash
cd sdk/python && pip install --user .
```

**TypeScript / WASM**
```bash
cd sdk/typescript && npm install && npm run build
```

**Go**
```bash
cd sdk/go && go test ./...
```

**Java**
```bash
cd sdk/java && mvn package
```

**C/C++**
```bash
cargo build --package qrd-ffi --release
# Header: core/qrd-ffi/include/qrd.h
# Library: target/release/libqrd_ffi.a
```

### WASM / Browser

```typescript
import { initWasm, inspectFooter } from 'qrd-sdk/browser';

await initWasm();

const buffer = await fetch('/data/telemetry.qrd').then(r => r.arrayBuffer());
const meta = inspectFooter(new Uint8Array(buffer));

console.log(`${meta.rowCount} rows, ${meta.schema.fields.length} columns`);
console.log('Fields:', meta.schema.fields.map(f => `${f.name}:${f.type}`));
```

---

## 📊 Benchmarks

### Design Targets

| Operasi | Workload | Target |
|---|---|---|
| Write throughput | 1 KB row, analytical stream | 1 - 5 GB/s |
| Full scan read | 100 MB dense dataset | 2 - 10 GB/s |
| Partial column read | 10% kolom terpilih | 5 - 20 GB/s |
| ZSTD compression ratio | Integer + string columns | 1.5x - 4x |
| LZ4 compression | Low-latency stream | Overhead minimal |

> Target ini mewakili desain engine. Selalu reproduksi pada hardware target Anda.

### Menjalankan Benchmark

```bash
cargo bench --package qrd-core                    # semua benchmark
cargo bench --package qrd-core -- encode          # encode/decode
cargo bench --package qrd-core -- streaming       # streaming
cargo bench --package qrd-core -- compression     # kompresi
cargo bench --package qrd-core -- --nocapture     # dengan output verbose
```

Setiap perubahan benchmark harus menyertakan spesifikasi hardware, versi toolchain, dan perbandingan before/after. Lihat [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) untuk panduan reproduksibilitas lengkap.

---

## 🧭 Use Cases

### Edge & IoT Telemetry

```
Sensor -> [QRD Writer] -> output.qrd -> [Upload] -> Cloud ingest
               ^
         bounded memory, LZ4 compression, CRC32 integrity
```

Cocok untuk sensor suhu, akselerometer, GPS log, dan event stream dari perangkat embedded.

### Browser Analytics (WASM)

```
Browser -> [WASM QRD Writer] -> IndexedDB / download
        <- [WASM QRD Reader] <- local .qrd file
```

Cocok untuk client-side telemetry collection, offline-capable dashboards, dan browser feature inspection.

### Local AI / ML Inference

```
Feature store (.qrd)
      |
      v  [Partial column read]
Selected features -> ML inference pipeline
```

Cocok untuk preprocessing data model inference, caching feature vectors, dan offline inference tanpa server.

### Cross-Language Data Exchange

```
Rust producer -> output.qrd -> Python consumer
                            -> Go consumer
                            -> TypeScript (browser)
```

Satu format, deterministic output, tidak ada implementasi drift antar bahasa.

### Audit & Compliance Logging

Self-describing schema, deterministic format, CRC32 per chunk, dan immutable row groups menjadikan QRD cocok untuk compliance logging, diagnostics storage, dan reproducible experiment records.

---

---

## 🆚 Format Comparison

| Properti | **QRD** | Parquet | Arrow IPC | CSV | SQLite |
|---|---|---|---|---|---|
| Format type | Columnar binary container | Columnar binary file | In-memory / IPC | Text table | Embedded relational DB |
| Streaming write | Native row-group stream | Requires buffering | Not designed | Yes (no schema) | Limited |
| Offline-first | Yes | Ecosystem-heavy | No | Yes | Yes |
| Partial column read | Yes | Yes | Yes | No | Query-bound |
| Schema embedded | Yes | Yes | Yes | No | Yes |
| Compression built-in | Chunk-level | Yes | Yes | No | Optional |
| Encryption | Metadata-aware | External | External | No | Optional |
| Error correction | Reed-Solomon | None | None | None | None |
| Browser / WASM | First-class | Limited | Arrow JS | Yes | No |
| Cross-language fidelity | Single engine | Multiple impls | Reference impl | Trivial | Single engine |
| Bounded-memory streaming | By design | Not primary goal | Not primary goal | Yes (no schema) | No |

**QRD tidak berkompetisi dengan Parquet atau Arrow** — QRD mengisi niche yang tidak dilayani dengan baik oleh format yang ada: streaming edge analytics dengan browser support dan bounded memory.

---

## 🔄 Compatibility & Versioning

### Semantic Versioning

```
MAJOR.MINOR.PATCH

MAJOR -> Perubahan format binary atau API yang tidak kompatibel
MINOR -> Fitur baru yang backward-compatible
PATCH -> Bug fix tanpa perubahan format
```

### Format Version Compatibility

| Skenario | Behavior |
|---|---|
| Reader versi sama | Penuh kompatibel |
| Reader major lebih rendah | Tolak sebagai unsupported |
| Reader minor lebih rendah | Ignore unknown optional fields |
| Unknown encoding/compression ID | Fail-fast dengan error jelas |
| Unknown optional metadata | Ignore safely |

### Schema Compatibility

| Perubahan | Kompatibel? |
|---|---|
| Menambah kolom opsional | Compatible |
| Menambah optional metadata field | Compatible |
| Rename field | Breaking — schema ID berubah |
| Ubah tipe field | Breaking — schema ID berubah |
| Ubah nullability REQUIRED ke OPTIONAL | Breaking |

---

## 📁 Repository Structure

```
QRD-SDK/
|
+-- core/
|   +-- qrd-core/              # Rust core engine -- authoritative implementation
|   |   +-- src/
|   |   |   +-- schema/        # Schema builder, serialization, fingerprint
|   |   |   +-- writer/        # StreamingWriter, row group flush
|   |   |   +-- reader/        # FileReader, partial reads, footer parse
|   |   |   +-- encoding/      # PLAIN, RLE, BIT_PACKED, DELTA_*, DICT_RLE
|   |   |   +-- compression/   # ZSTD, LZ4, adaptive selection
|   |   |   +-- encryption/    # AES-256-GCM, HKDF
|   |   |   +-- ecc/           # Reed-Solomon ECC
|   |   |   +-- columnar/      # Row-to-column transposition
|   |   +-- benches/           # Criterion benchmark suite
|   |   +-- examples/          # Usage examples
|   |
|   +-- qrd-ffi/               # C-compatible FFI layer
|   +-- qrd-wasm/              # WebAssembly target
|
+-- sdk/
|   +-- python/                # PyO3 Python binding
|   +-- typescript/            # WASM + TypeScript packaging
|   +-- go/                    # CGO Go binding
|   +-- java/                  # JNI Java binding
|
+-- docs/
|   +-- FORMAT_SPEC.md         # Binary format specification (canonical)
|   +-- ARCHITECTURE.md        # System design & component overview
|   +-- SDKS.md                # Language binding status & install
|   +-- BENCHMARKS.md          # Benchmark methodology & results
|   +-- STABILITY.md           # Compatibility & deprecation policy
|   +-- VERSIONING.md          # Semantic versioning policy
|   +-- STREAMING_MODEL.md     # Streaming write/read semantics
|   +-- MEMORY_MODEL.md        # Bounded-memory guarantees
|   +-- COMPRESSION.md         # Compression design & codec guide
|   +-- EDGE_AI.md             # Edge AI & telemetry guidance
|   +-- WASM.md                # WASM & browser runtime docs
|   +-- SECURITY_AUDIT.md      # Audit goals & process
|   +-- THREAT_MODEL.md        # Threat analysis
|   +-- FUZZING.md             # Fuzz coverage guidance
|   +-- COMPETITOR_COMPARISON.md
|   +-- DEPLOYMENT.md
|   +-- USE_CASES.md
|   +-- PERFORMANCE.md
|   +-- COMPATIBILITY.md
|
+-- examples/                  # Top-level usage examples
+-- benches/                   # Top-level benchmark pointers
+-- tests/                     # Integration & regression tests
+-- specs/                     # Format spec supplements
+-- tools/                     # CLI & interoperability tooling
|
+-- Cargo.toml                 # Workspace manifest
+-- Makefile                   # Common dev commands
+-- CHANGELOG.md               # Version history
+-- CONTRIBUTING.md            # Contribution guide
+-- SECURITY.md                # Vulnerability reporting
+-- LICENSE                    # MIT License
```

---

## 🤝 Contributing

QRD menargetkan kualitas infrastructure-grade. Kontribusi diharapkan memenuhi standar berikut:

### Proses Kontribusi

1. **Buka issue** — deskripsikan perubahan yang diusulkan dan referensi docs terkait di `docs/`
2. **Submit PR** — deskripsi jelas, tests, dan benchmark jika relevan
3. **CI harus pass** — semua workflow yang ada harus hijau
4. **Review** — perubahan pada format, enkripsi, FFI, atau ECC membutuhkan security review

### Standar Kode

- Ikuti Rust idioms di `core/qrd-core/`; gunakan `clippy` dan `rustfmt`
- Jaga FFI bindings tetap tipis dan konsisten dengan core interface
- Dokumentasikan semua public API dan format changes
- Setiap fitur baru membutuhkan unit test dan golden test vectors
- Perubahan yang mempengaruhi benchmark harus menyertakan before/after dengan detail hardware

### Testing

```bash
cargo test --package qrd-core          # unit tests
cargo test --workspace                 # full workspace
cargo test --package qrd-core -- proptest  # property-based tests
```

Lihat [`CONTRIBUTING.md`](CONTRIBUTING.md) untuk panduan lengkap termasuk release process dan ekspektasi CI.

---

## 📚 Documentation Index

| Dokumen | Deskripsi |
|---|---|
| [`docs/FORMAT_SPEC.md`](docs/FORMAT_SPEC.md) | Spesifikasi binary format (canonical) |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Desain sistem & overview komponen |
| [`docs/SDKS.md`](docs/SDKS.md) | Status SDK & instruksi instalasi per bahasa |
| [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) | Metodologi benchmark & hasil |
| [`docs/STABILITY.md`](docs/STABILITY.md) | Compatibility & deprecation policy |
| [`docs/VERSIONING.md`](docs/VERSIONING.md) | Semantic versioning policy |
| [`docs/STREAMING_MODEL.md`](docs/STREAMING_MODEL.md) | Semantik streaming write/read |
| [`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md) | Bounded-memory guarantees & row group design |
| [`docs/COMPRESSION.md`](docs/COMPRESSION.md) | Desain kompresi & codec guide |
| [`docs/EDGE_AI.md`](docs/EDGE_AI.md) | Edge AI & telemetry workload guidance |
| [`docs/WASM.md`](docs/WASM.md) | WASM & browser runtime docs |
| [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) | Filosofi & metodologi performa |
| [`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md) | Cross-version compatibility rules |
| [`docs/COMPETITOR_COMPARISON.md`](docs/COMPETITOR_COMPARISON.md) | Perbandingan dengan format lain |
| [`docs/SECURITY_AUDIT.md`](docs/SECURITY_AUDIT.md) | Audit goals & process |
| [`docs/THREAT_MODEL.md`](docs/THREAT_MODEL.md) | Threat analysis |
| [`docs/FUZZING.md`](docs/FUZZING.md) | Fuzz coverage guidance |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history & release notes |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Panduan kontribusi |
| [`SECURITY.md`](SECURITY.md) | Responsible disclosure policy |

---

## 📜 License

Proyek ini dilisensikan di bawah [MIT License](LICENSE).

---

<div align="center">

**QRD-SDK** dibangun untuk sistem yang membutuhkan format binary portabel<br/>
lintas perangkat, browser, dan server — tanpa infrastruktur terpusat.

<br/>

[![GitHub](https://img.shields.io/badge/GitHub-zenipara%2FQRD--SDK-black?logo=github)](https://github.com/zenipara/QRD-SDK)
[![Documentation](https://img.shields.io/badge/Documentation-docs.qrd.dev-brightgreen)](https://docs.qrd.dev)
[![Changelog](https://img.shields.io/badge/Changelog-CHANGELOG.md-blue)](CHANGELOG.md)

<br/>

*Built with Rust · MIT Licensed · Contributions Welcome*

</div>
