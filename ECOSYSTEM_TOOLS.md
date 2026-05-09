# QRD Ecosystem & Tools

**Author:** NAFAL FATURIZKI  
**Version:** 1.0.0  
**Status:** Ecosystem Design Specification  
**Last Updated:** 2026-05-09

---

## Daftar Isi

1. [Filosofi Ecosystem](#1-filosofi-ecosystem)
2. [Peta Ecosystem](#2-peta-ecosystem)
3. [Tier 1 — Core Tooling (Wajib)](#3-tier-1--core-tooling-wajib)
   - [qrd-cli](#31-qrd-cli)
   - [qrd-inspect](#32-qrd-inspect)
   - [qrd-convert](#33-qrd-convert)
4. [Tier 2 — Language SDKs](#4-tier-2--language-sdks)
   - [qrd-python](#41-qrd-python)
   - [qrd-typescript](#42-qrd-typescript)
   - [qrd-go](#43-qrd-go)
   - [qrd-java](#44-qrd-java)
5. [Tier 3 — Integrasi Ekosistem Data](#5-tier-3--integrasi-ekosistem-data)
   - [DuckDB Extension](#51-duckdb-extension)
   - [Apache Arrow Bridge](#52-apache-arrow-bridge)
   - [Pandas I/O Plugin](#53-pandas-io-plugin)
   - [Polars Plugin](#54-polars-plugin)
6. [Tier 4 — Developer Experience](#6-tier-4--developer-experience)
   - [VS Code Extension](#61-vs-code-extension)
   - [qrd-web-inspector](#62-qrd-web-inspector)
   - [qrd-schema-registry](#63-qrd-schema-registry)
7. [Tier 5 — Infrastructure & Operasional](#7-tier-5--infrastructure--operasional)
   - [qrd-server](#71-qrd-server)
   - [qrd-fuse](#72-qrd-fuse)
   - [qrd-kafka-connector](#73-qrd-kafka-connector)
   - [qrd-s3-sync](#74-qrd-s3-sync)
8. [Tier 6 — Testing & Validasi](#8-tier-6--testing--validasi)
   - [qrd-test-vectors](#81-qrd-test-vectors)
   - [qrd-fuzz](#82-qrd-fuzz)
   - [qrd-compat-suite](#83-qrd-compat-suite)
9. [Roadmap Adopsi Ecosystem](#9-roadmap-adopsi-ecosystem)
10. [Dependency Graph Ecosystem](#10-dependency-graph-ecosystem)
11. [Standar Kontribusi Ecosystem](#11-standar-kontribusi-ecosystem)

---

## 1. Filosofi Ecosystem

QRD bukan hanya format — ini adalah **kontrak biner yang bisa dipercaya lintas bahasa dan lintas platform**. Ecosystem yang dibangun di atasnya harus mencerminkan tiga nilai inti:

**Satu engine, banyak pintu masuk.**  
Setiap tool, setiap SDK, setiap integrasi memanggil Rust core yang sama. Tidak ada reimplementasi. Tidak ada format drift. Tidak ada "versi Python yang sedikit berbeda."

**Setiap tool harus bisa berdiri sendiri.**  
`qrd-cli` bisa dipakai tanpa Python SDK. DuckDB extension bisa dipakai tanpa `qrd-cli`. Tidak ada circular dependency di antara komponen ecosystem.

**Tooling adalah dokumentasi yang bisa dijalankan.**  
Cara terbaik untuk menjelaskan QRD adalah membiarkan orang menjalankan `qrd inspect myfile.qrd` dan melihat sendiri. Setiap tool adalah demonstrasi langsung dari nilai format.

---

## 2. Peta Ecosystem

```
                        ┌─────────────────────────────────────┐
                        │         qrd-core (Rust Engine)      │
                        │   Schema · Encoding · Compression    │
                        │   Encryption · ECC · SIMD · I/O     │
                        └──────────────────┬──────────────────┘
                                           │ FFI / WASM
              ┌────────────────────────────┼────────────────────────────┐
              │                            │                            │
   ┌──────────▼───────┐        ┌──────────▼───────┐        ┌──────────▼──────────┐
   │   TIER 1         │        │   TIER 2          │        │   TIER 3            │
   │   Core Tooling   │        │   Language SDKs   │        │   Data Ecosystem    │
   │                  │        │                   │        │                     │
   │  qrd-cli         │        │  qrd-python       │        │  DuckDB Extension   │
   │  qrd-inspect     │        │  qrd-typescript   │        │  Arrow Bridge       │
   │  qrd-convert     │        │  qrd-go           │        │  Pandas Plugin      │
   └──────────────────┘        │  qrd-java         │        │  Polars Plugin      │
                               └───────────────────┘        └─────────────────────┘
              ┌────────────────────────────┬────────────────────────────┐
              │                            │                            │
   ┌──────────▼───────┐        ┌──────────▼───────┐        ┌──────────▼──────────┐
   │   TIER 4         │        │   TIER 5          │        │   TIER 6            │
   │   Dev Experience │        │   Infrastructure  │        │   Testing & Validasi│
   │                  │        │                   │        │                     │
   │  VS Code Ext.    │        │  qrd-server       │        │  test-vectors       │
   │  Web Inspector   │        │  qrd-fuse         │        │  qrd-fuzz           │
   │  Schema Registry │        │  Kafka Connector  │        │  compat-suite       │
   └──────────────────┘        │  S3 Sync          │        └─────────────────────┘
                               └───────────────────┘
```

---

## 3. Tier 1 — Core Tooling (Wajib)

Tier ini adalah **prasyarat adopsi**. Tanpa tools ini, QRD tidak bisa diinspeksi, tidak bisa didebug, dan tidak bisa didemonstrasikan kepada orang lain. Harus selesai sebelum atau bersamaan dengan v1.0.0.

---

### 3.1 `qrd-cli`

**Satu binary. Semua operasi dasar.**

`qrd-cli` adalah entry point utama ekosistem QRD. Dibangun di atas `qrd-core` langsung — tidak ada layer tambahan. Target engineer adalah: data engineer, backend developer, dan siapapun yang perlu bekerja dengan file QRD dari terminal.

#### Instalasi

```bash
# via cargo
cargo install qrd-cli

# via homebrew (macOS/Linux)
brew install qrd

# via winget (Windows)
winget install qrd

# Download binary langsung
curl -fsSL https://install.qrd.dev | sh
```

#### Subcommand: `qrd write`

Baca data dari stdin (JSON Lines / CSV), tulis ke file QRD.

```bash
# JSON Lines → QRD
cat data.jsonl | qrd write output.qrd

# CSV → QRD (schema diturunkan otomatis)
cat data.csv | qrd write --format csv output.qrd

# CSV dengan schema eksplisit
cat data.csv | qrd write \
  --format csv \
  --schema '{"id":"INT64","name":"STRING","ts":"TIMESTAMP"}' \
  output.qrd

# Dengan kompresi dan enkripsi
cat data.jsonl | qrd write \
  --compression zstd \
  --encrypt --key-file secret.key \
  output.qrd

# Dengan row group size kustom
cat data.jsonl | qrd write \
  --row-group-size 500000 \
  output.qrd
```

#### Subcommand: `qrd read`

Baca file QRD, output ke stdout.

```bash
# Output semua row sebagai JSON Lines
qrd read input.qrd

# Output sebagai CSV
qrd read --format csv input.qrd

# Baca kolom tertentu saja (partial read)
qrd read --columns "id,timestamp,value" input.qrd

# Filter row group tertentu
qrd read --row-group 0 input.qrd

# Limit jumlah row
qrd read --limit 1000 input.qrd

# Output ke file
qrd read input.qrd > output.jsonl

# Dekripsi on-the-fly
qrd read --key-file secret.key input.qrd
```

#### Subcommand: `qrd inspect`

Tampilkan metadata file tanpa membaca row data. Operasi ini hanya membaca footer — sangat cepat bahkan untuk file besar.

```bash
qrd inspect input.qrd
```

Output:

```
QRD File: input.qrd
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Format Version  : 1.0
File Size       : 2.34 GB
Created At      : 2026-05-09 14:23:11 UTC
Schema ID       : a3f7c2d1

Schema (6 columns)
─────────────────────────────────────────────────
  # │ Name          │ Type        │ Nullable │ Encoding        │ Compression
  0 │ id            │ INT64       │ No       │ DELTA_BINARY    │ ZSTD
  1 │ user_id       │ UUID        │ No       │ PLAIN           │ ZSTD
  2 │ event_name    │ STRING      │ Yes      │ DICTIONARY_RLE  │ ZSTD
  3 │ timestamp     │ TIMESTAMP   │ No       │ DELTA_BINARY    │ LZ4
  4 │ value         │ FLOAT64     │ Yes      │ BYTE_STREAM_SPLIT│ ZSTD
  5 │ metadata      │ BLOB        │ Yes      │ PASSTHROUGH     │ NONE

Row Groups (12 total)
─────────────────────────────────────────────────
  # │ Rows      │ Compressed │ Uncompressed │ Ratio
  0 │ 1,000,000 │ 145.2 MB   │ 412.3 MB     │ 2.84x
  1 │ 1,000,000 │ 143.7 MB   │ 409.1 MB     │ 2.85x
  ...
 11 │   847,231 │ 122.8 MB   │ 349.2 MB     │ 2.84x

Summary
─────────────────────────────────────────────────
  Total Rows        : 11,847,231
  Avg Compression   : 2.84x
  ECC               : Enabled (4 data + 2 parity)
  Encryption        : AES-256-GCM (per-column)
  Footer Size       : 4.2 KB
  Schema CRC32      : ✓ Valid
  Footer CRC32      : ✓ Valid
```

#### Subcommand: `qrd verify`

Validasi integritas file secara menyeluruh.

```bash
qrd verify input.qrd

# Output:
# ✓ Magic number valid
# ✓ Version: 1.0 (supported)
# ✓ Schema CRC32 valid
# ✓ Footer CRC32 valid
# ✓ Row group 0/12: CRC32 valid
# ✓ Row group 1/12: CRC32 valid
# ...
# ✓ Row group 12/12: CRC32 valid
# ✓ ECC check: all parity chunks consistent
# ✓ File: VALID (12 row groups, 11,847,231 rows)
```

```bash
# Verifikasi cepat (footer + header saja)
qrd verify --quick input.qrd

# Dengan verbose per-chunk
qrd verify --verbose input.qrd

# Repair file jika ada korupsi (menggunakan ECC)
qrd verify --repair input.qrd --output repaired.qrd
```

#### Subcommand: `qrd schema`

Operasi terkait schema.

```bash
# Print schema sebagai JSON
qrd schema input.qrd

# Print schema sebagai DDL (SQL-style)
qrd schema --format ddl input.qrd

# Bandingkan schema dua file
qrd schema diff file1.qrd file2.qrd

# Export schema ke file
qrd schema input.qrd > schema.json

# Validasi data JSON Lines terhadap schema
cat data.jsonl | qrd schema validate schema.json
```

#### Subcommand: `qrd stat`

Statistik kolom dari data aktual.

```bash
qrd stat input.qrd

# Output per kolom:
# column: timestamp
#   min: 2026-01-01 00:00:00
#   max: 2026-05-09 14:23:11
#   null_count: 0
#   distinct_estimate: ~11.8M
#
# column: value
#   min: -1024.5
#   max: 99847.2
#   mean: 4821.3
#   stddev: 12043.7
#   null_count: 14,231
```

#### Subcommand: `qrd bench`

Benchmark operasi baca/tulis pada sistem saat ini.

```bash
# Benchmark tulis dengan data sintetis
qrd bench write --rows 1000000 --columns 6

# Benchmark baca
qrd bench read input.qrd

# Benchmark kompresi
qrd bench compress --codec zstd --level 3 input.qrd
```

#### Flags Global

```bash
qrd [subcommand] \
  --verbose              # Output detail proses
  --quiet                # Hanya output data, tanpa log
  --no-color             # Disable ANSI color
  --json-errors          # Error output sebagai JSON (untuk scripting)
  --threads N            # Jumlah thread (default: CPU count)
```

---

### 3.2 `qrd-inspect`

**GUI terminal (TUI) untuk eksplorasi file QRD interaktif.**

`qrd-inspect` adalah mode interaktif dari operasi inspect — dibangun dengan `ratatui` (Rust TUI framework). Dirancang untuk engineer yang perlu menjelajahi file besar secara interaktif tanpa menulis kode.

```bash
cargo install qrd-inspect
qrd-inspect input.qrd
```

#### Tampilan TUI

```
┌─ QRD Inspector ─────────────────────────────────────────────┐
│ File: sensor_data_2026.qrd │ 2.34 GB │ 11.8M rows │ v1.0    │
├─────────────────────────────────────────────────────────────┤
│ [Schema] [Row Groups] [Data Preview] [Stats] [ECC] [Help]   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Schema                                                      │
│  ──────────────────────────────────────────────────────      │
│  ▶ id          INT64       REQUIRED    DELTA_BINARY / ZSTD   │
│  ▶ user_id     UUID        REQUIRED    PLAIN / ZSTD          │
│  ▶ event_name  STRING      OPTIONAL    DICTIONARY_RLE / ZSTD │
│  ▶ timestamp   TIMESTAMP   REQUIRED    DELTA_BINARY / LZ4    │
│  ▶ value       FLOAT64     OPTIONAL    BYTE_STREAM_SPLIT/ZSTD│
│  ▶ metadata    BLOB        OPTIONAL    PASSTHROUGH / NONE     │
│                                                              │
│  [↑↓] Navigate  [Enter] Expand  [P] Preview  [Q] Quit       │
└─────────────────────────────────────────────────────────────┘
```

#### Fitur

- Navigate schema, row groups, dan statistik kolom dengan keyboard
- Preview 10–100 baris data tanpa load full file
- Visualisasi distribusi nilai kolom numerik (ASCII histogram)
- Highlight kolom dengan anomali (null rate tinggi, CRC mismatch)
- Export view saat ini ke JSON/CSV

---

### 3.3 `qrd-convert`

**Konversi dua arah antara QRD dan format data umum.**

Ini adalah pintu masuk paling natural untuk adopsi: engineer punya data di format lain, ingin mencoba QRD, tanpa perlu menulis kode.

```bash
cargo install qrd-convert
```

#### Format yang didukung

| Format | Read | Write | Notes |
|---|---|---|---|
| CSV | ✅ | ✅ | Schema inference otomatis |
| JSON Lines | ✅ | ✅ | Satu objek per baris |
| Apache Parquet | ✅ | ✅ | Via Arrow IPC bridge |
| Apache Arrow IPC | ✅ | ✅ | Columnar memory compatible |
| TSV | ✅ | ✅ | Tab-separated |
| Avro | ✅ | 🔄 | Read: Phase 1, Write: Phase 2 |
| ORC | ✅ | ❌ | Read only |
| MessagePack | ✅ | ✅ | Binary JSON |
| SQLite (dump) | ✅ | ❌ | Via SQL SELECT output |
| Excel (.xlsx) | ✅ | ❌ | Read only, via calamine |

#### Penggunaan

```bash
# CSV → QRD
qrd-convert input.csv output.qrd

# QRD → CSV
qrd-convert input.qrd output.csv

# Parquet → QRD
qrd-convert input.parquet output.qrd

# QRD → Parquet
qrd-convert input.qrd output.parquet

# JSON Lines → QRD dengan opsi
qrd-convert input.jsonl output.qrd \
  --compression zstd \
  --row-group-size 1000000 \
  --encoding auto

# Multi-file: merge beberapa CSV → satu QRD
qrd-convert --merge data/*.csv merged.qrd

# Split QRD → multiple Parquet (per row group)
qrd-convert input.qrd output_dir/ \
  --split-by row-group \
  --format parquet

# Streaming: pipe dari proses lain
pg_dump --data-only mydb mytable | \
  qrd-convert --from sql --to qrd - output.qrd

# Preview konversi tanpa tulis (dry run)
qrd-convert input.csv output.qrd --dry-run

# Benchmark: bandingkan ukuran sebelum dan sesudah
qrd-convert input.parquet output.qrd --report
```

Output `--report`:

```
Conversion Report
─────────────────────────────────────────────────
  Source        : input.parquet (Parquet v2)
  Destination   : output.qrd (QRD v1.0)
  
  Rows          : 11,847,231
  Columns       : 6
  
  Source size   : 2,891 MB
  Output size   : 2,338 MB
  Size delta    : -19.1%
  
  Encoding decisions:
    id          → DELTA_BINARY (was PLAIN)
    event_name  → DICTIONARY_RLE (was RLE_DICTIONARY)
    timestamp   → DELTA_BINARY (was PLAIN)
    value       → BYTE_STREAM_SPLIT (was PLAIN)
  
  Duration      : 14.2s
  Throughput    : 203 MB/s (write)
  Schema valid  : ✓
  Checksums     : ✓ All valid
```

---

## 4. Tier 2 — Language SDKs

Setiap SDK dibangun di atas FFI layer yang sama — bukan reimplementasi. Ini menjamin output bit-identical di semua bahasa.

---

### 4.1 `qrd-python`

**Python SDK via PyO3 — zero-copy di mana mungkin.**

Target: data engineers, ML engineers, siapapun yang bekerja dengan Python dan data.

#### Instalasi

```bash
pip install qrd-python

# Atau dengan extras
pip install qrd-python[pandas]
pip install qrd-python[polars]
pip install qrd-python[numpy]
```

#### API Dasar

```python
import qrd

# ── WRITE ──────────────────────────────────────────────────
schema = qrd.Schema([
    qrd.Field("id",         qrd.INT64,     nullable=False),
    qrd.Field("user_id",    qrd.UUID,      nullable=False),
    qrd.Field("event",      qrd.STRING,    nullable=True),
    qrd.Field("timestamp",  qrd.TIMESTAMP, nullable=False),
    qrd.Field("value",      qrd.FLOAT64,   nullable=True),
])

writer = qrd.Writer("output.qrd", schema)

for i in range(1_000_000):
    writer.write({
        "id": i,
        "user_id": "550e8400-e29b-41d4-a716-446655440000",
        "event": "click",
        "timestamp": 1715267391000000,
        "value": 3.14,
    })

writer.close()

# ── READ ───────────────────────────────────────────────────
reader = qrd.Reader("output.qrd")

# Baca semua sebagai list of dicts
rows = reader.read_all()

# Streaming iterator (memory-efficient)
for row in reader.iter_rows():
    process(row)

# Baca kolom tertentu saja
for row in reader.iter_rows(columns=["id", "timestamp", "value"]):
    process(row)

# Akses schema
print(reader.schema)
print(reader.row_count)
print(reader.row_groups)
```

#### Integrasi Pandas

```python
import qrd
import pandas as pd

# QRD → DataFrame (zero-copy untuk numerik)
df = qrd.read_dataframe("input.qrd")

# Kolom tertentu saja
df = qrd.read_dataframe("input.qrd", columns=["timestamp", "value"])

# DataFrame → QRD
qrd.write_dataframe(df, "output.qrd")

# Dengan opsi
qrd.write_dataframe(
    df,
    "output.qrd",
    compression="zstd",
    row_group_size=500_000,
    encrypt=True,
    key_file="secret.key"
)

# Append ke file yang sudah ada
qrd.append_dataframe(df, "existing.qrd")
```

#### Integrasi NumPy

```python
import qrd
import numpy as np

# Baca satu kolom sebagai NumPy array
arr = qrd.read_column("input.qrd", "value")  # → np.ndarray, dtype=float64

# Baca banyak kolom
arrays = qrd.read_columns("input.qrd", ["id", "value"])

# Tulis dari dict of arrays
qrd.write_arrays(
    {
        "id": np.arange(1_000_000, dtype=np.int64),
        "value": np.random.rand(1_000_000).astype(np.float64),
    },
    schema=schema,
    path="output.qrd"
)
```

#### Context Manager & Async

```python
# Context manager
with qrd.Writer("output.qrd", schema) as writer:
    for row in data_source:
        writer.write(row)

# Async reader (untuk aplikasi async)
import asyncio

async def read_async():
    async with qrd.AsyncReader("input.qrd") as reader:
        async for row in reader.iter_rows():
            await process(row)
```

#### Inspect dari Python

```python
info = qrd.inspect("input.qrd")
print(info.schema)
print(info.row_count)
print(info.row_groups)
print(info.file_size)
print(info.compression_ratio)
```

---

### 4.2 `qrd-typescript`

**TypeScript SDK via WebAssembly — browser, Node.js, dan edge runtime.**

Target: frontend engineers, fullstack engineers, Cloudflare Workers, Deno Deploy.

#### Instalasi

```bash
npm install @qrd/sdk
# atau
yarn add @qrd/sdk
# atau
bun add @qrd/sdk
```

#### Node.js / Server

```typescript
import { QrdWriter, QrdReader, Schema, FieldType } from '@qrd/sdk';

// ── WRITE ──────────────────────────────────────────────────
const schema = new Schema([
  { name: 'id',        type: FieldType.INT64,     nullable: false },
  { name: 'event',     type: FieldType.STRING,    nullable: true  },
  { name: 'timestamp', type: FieldType.TIMESTAMP, nullable: false },
  { name: 'value',     type: FieldType.FLOAT64,   nullable: true  },
]);

const writer = new QrdWriter('output.qrd', schema);

for (const row of dataSource) {
  await writer.writeRow(row);
}

await writer.close();

// ── READ ───────────────────────────────────────────────────
const reader = new QrdReader('input.qrd');
const info = await reader.inspect();
console.log(info.schema, info.rowCount);

// Async iterator
for await (const row of reader.iterRows()) {
  process(row);
}

// Baca kolom spesifik
for await (const row of reader.iterRows({ columns: ['id', 'value'] })) {
  process(row);
}
```

#### Browser (via WASM)

```typescript
import { QrdReader, initWasm } from '@qrd/sdk/browser';

// Inisialisasi WASM satu kali
await initWasm();

// Baca dari File input
const fileInput = document.getElementById('file') as HTMLInputElement;
fileInput.addEventListener('change', async (e) => {
  const file = fileInput.files![0];
  const buffer = await file.arrayBuffer();
  
  const reader = QrdReader.fromBuffer(new Uint8Array(buffer));
  const info = reader.inspect();
  
  console.log(`${info.rowCount} rows, ${info.schema.fields.length} columns`);
  
  for (const row of reader.iterRows({ limit: 100 })) {
    console.log(row);
  }
});
```

#### Streaming Write (Node.js)

```typescript
import { QrdStreamWriter } from '@qrd/sdk';
import { createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';

const out = createWriteStream('output.qrd');
const writer = new QrdStreamWriter(schema, { compression: 'zstd' });

// Pipe dari Node.js Readable stream
await pipeline(dataReadable, writer, out);
```

#### Cloudflare Workers / Deno

```typescript
// Kompatibel dengan Web Streams API
import { QrdReader } from '@qrd/sdk/worker';

export default {
  async fetch(request: Request): Promise<Response> {
    const body = await request.arrayBuffer();
    const reader = QrdReader.fromBuffer(new Uint8Array(body));
    const rows = reader.readAll();
    return Response.json({ rows, count: rows.length });
  }
};
```

---

### 4.3 `qrd-go`

**Go SDK via CGO — idiomatic Go dengan `io.Reader`/`io.Writer`.**

Target: backend services, CLI tools, dan sistem yang sudah menggunakan Go.

#### Instalasi

```bash
go get github.com/nafalfaturizki/qrd-go
```

#### Penggunaan

```go
package main

import (
    "os"
    "github.com/nafalfaturizki/qrd-go/qrd"
)

func main() {
    // ── WRITE ──────────────────────────────────────────────
    schema := qrd.NewSchema(
        qrd.Field{Name: "id",        Type: qrd.INT64,     Nullable: false},
        qrd.Field{Name: "event",     Type: qrd.STRING,    Nullable: true},
        qrd.Field{Name: "timestamp", Type: qrd.TIMESTAMP, Nullable: false},
        qrd.Field{Name: "value",     Type: qrd.FLOAT64,   Nullable: true},
    )

    f, _ := os.Create("output.qrd")
    writer, _ := qrd.NewWriter(f, schema, qrd.DefaultConfig())
    defer writer.Close()

    writer.WriteRow(qrd.Row{
        "id":        int64(1),
        "event":     "click",
        "timestamp": int64(1715267391000000),
        "value":     3.14,
    })

    // ── READ ───────────────────────────────────────────────
    r, _ := os.Open("input.qrd")
    reader, _ := qrd.NewReader(r)

    info := reader.Inspect()
    fmt.Printf("Rows: %d, Columns: %d\n", info.RowCount, len(info.Schema.Fields))

    // Iterator
    iter := reader.Iter()
    for iter.Next() {
        row := iter.Row()
        process(row)
    }

    // Baca kolom tertentu
    iter = reader.IterColumns([]string{"id", "value"})
    for iter.Next() {
        row := iter.Row()
        process(row)
    }
}
```

#### io.Reader / io.Writer Compatible

```go
// Kompatibel dengan io.Reader dan io.Writer standar
// Bisa digabung dengan compress/gzip, net/http, dll.

resp, _ := http.Get("https://example.com/data.qrd")
reader, _ := qrd.NewReader(resp.Body)  // langsung dari HTTP response
defer resp.Body.Close()

// Tulis langsung ke HTTP response
w.Header().Set("Content-Type", "application/qrd")
writer, _ := qrd.NewWriter(w, schema, qrd.DefaultConfig())
// ...
```

---

### 4.4 `qrd-java`

**Java SDK via JNI — Maven/Gradle compatible, Stream API support.**

Target: JVM ecosystem, Spark integration, enterprise Java services.

#### Instalasi (Maven)

```xml
<dependency>
  <groupId>dev.qrd</groupId>
  <artifactId>qrd-java</artifactId>
  <version>1.0.0</version>
</dependency>
```

#### Instalasi (Gradle)

```groovy
implementation 'dev.qrd:qrd-java:1.0.0'
```

#### Penggunaan

```java
import dev.qrd.*;

// ── WRITE ──────────────────────────────────────────────────
Schema schema = Schema.builder()
    .field("id",        FieldType.INT64,     false)
    .field("event",     FieldType.STRING,    true)
    .field("timestamp", FieldType.TIMESTAMP, false)
    .field("value",     FieldType.FLOAT64,   true)
    .build();

try (QrdWriter writer = QrdWriter.open("output.qrd", schema)) {
    writer.writeRow(Map.of(
        "id",        1L,
        "event",     "click",
        "timestamp", 1715267391000000L,
        "value",     3.14
    ));
}

// ── READ ───────────────────────────────────────────────────
try (QrdReader reader = QrdReader.open("input.qrd")) {
    QrdInfo info = reader.inspect();
    System.out.printf("Rows: %d%n", info.getRowCount());

    // Java Stream API
    reader.stream()
          .filter(row -> (double) row.get("value") > 100.0)
          .limit(1000)
          .forEach(this::process);

    // Kolom spesifik
    reader.streamColumns(List.of("id", "value"))
          .forEach(this::process);
}
```

#### Apache Spark Integration

```java
// Read QRD dalam Spark
Dataset<Row> df = spark.read()
    .format("qrd")
    .load("hdfs:///data/events/*.qrd");

// Write Spark DataFrame ke QRD
df.write()
  .format("qrd")
  .option("compression", "zstd")
  .option("rowGroupSize", "1000000")
  .save("hdfs:///output/events.qrd");
```

---

## 5. Tier 3 — Integrasi Ekosistem Data

---

### 5.1 DuckDB Extension

**Baca dan query file QRD langsung dari DuckDB dengan SQL.**

Ini adalah multiplier terbesar untuk adopsi. Dengan DuckDB extension, siapapun yang sudah pakai DuckDB bisa langsung query file QRD tanpa belajar API baru.

#### Instalasi

```sql
-- Di dalam DuckDB
INSTALL qrd FROM community;
LOAD qrd;
```

#### Penggunaan

```sql
-- Query langsung dari file QRD
SELECT * FROM read_qrd('data.qrd');

-- Dengan filter (predicate pushdown ke column reads)
SELECT id, timestamp, value
FROM read_qrd('data.qrd')
WHERE value > 100.0
  AND timestamp >= '2026-01-01'::TIMESTAMP;

-- Multiple files (glob)
SELECT * FROM read_qrd('data/*.qrd');

-- Registrasi sebagai virtual table
CREATE VIEW events AS SELECT * FROM read_qrd('events.qrd');
SELECT COUNT(*) FROM events WHERE event_name = 'purchase';

-- Konversi QRD → Parquet via DuckDB
COPY (SELECT * FROM read_qrd('input.qrd'))
TO 'output.parquet' (FORMAT PARQUET);

-- Inspect schema
DESCRIBE SELECT * FROM read_qrd('data.qrd');

-- Statistik kolom
SELECT
    column_name,
    min(value),
    max(value),
    avg(value),
    count(*) FILTER (value IS NULL) AS null_count
FROM read_qrd('data.qrd')
GROUP BY column_name;
```

#### Fitur DuckDB Extension

- **Predicate pushdown:** Filter WHERE dikirim ke QRD reader, hanya kolom yang dibutuhkan yang dibaca dari disk
- **Column pruning:** SELECT tertentu hanya membaca kolom tersebut, memanfaatkan partial read QRD
- **Parallel scan:** Row groups dibaca secara paralel menggunakan thread pool DuckDB
- **Schema inference:** Schema QRD otomatis di-map ke DuckDB types
- **Glob support:** `read_qrd('logs/**/*.qrd')` untuk batch processing

---

### 5.2 Apache Arrow Bridge

**Konversi dua arah antara QRD dan Apache Arrow format.**

Arrow adalah standar de-facto untuk pertukaran data in-memory. Bridge ini memungkinkan QRD bekerja sebagai persistent storage layer untuk aplikasi yang menggunakan Arrow.

```python
# Python
import qrd
import pyarrow as pa

# QRD → Arrow Table (zero-copy untuk kolom numerik)
table: pa.Table = qrd.to_arrow("input.qrd")

# Arrow Table → QRD
qrd.from_arrow(table, "output.qrd")

# Arrow RecordBatch streaming → QRD
writer = qrd.Writer("output.qrd", schema=qrd.schema_from_arrow(arrow_schema))
for batch in arrow_reader:
    writer.write_arrow_batch(batch)
writer.close()
```

```rust
// Rust
use qrd_core::arrow_bridge::ArrowBridge;

let bridge = ArrowBridge::new("input.qrd")?;
let record_batch = bridge.read_batch(0)?;  // Row group 0 sebagai RecordBatch
```

---

### 5.3 Pandas I/O Plugin

**Integrasi Pandas sebagai first-class I/O engine.**

```python
# Setelah install qrd-python[pandas], ini tersedia:
import pandas as pd

# Menggunakan pd.read_qrd() dan df.to_qrd()
df = pd.read_qrd("input.qrd")
df.to_qrd("output.qrd", compression="zstd")

# Filter kolom
df = pd.read_qrd("input.qrd", columns=["id", "timestamp", "value"])

# Chunk reading untuk dataset besar
for chunk in pd.read_qrd("input.qrd", chunksize=100_000):
    process(chunk)
```

---

### 5.4 Polars Plugin

**Integrasi Polars — mendukung lazy evaluation dan scan.**

```python
import polars as pl

# Eager read
df = pl.read_qrd("input.qrd")

# Lazy scan dengan predicate pushdown
df = (
    pl.scan_qrd("input.qrd")
    .filter(pl.col("value") > 100)
    .select(["id", "timestamp", "value"])
    .collect()
)

# Write
df.write_qrd("output.qrd", compression="zstd")

# Multiple files
df = pl.scan_qrd("data/*.qrd").collect()
```

---

## 6. Tier 4 — Developer Experience

---

### 6.1 VS Code Extension

**File explorer, schema viewer, dan data preview untuk file `.qrd` di VS Code.**

#### Instalasi

```
ext install qrd.vscode-qrd
```

#### Fitur

**File Explorer Integration:**
- File `.qrd` tampil dengan icon khusus di file tree
- Hover tooltip menampilkan: ukuran file, jumlah row, jumlah kolom, versi format

**Schema Panel:**
- Klik file `.qrd` → terbuka panel Schema di sidebar
- Tampilkan semua field dengan tipe, nullability, encoding, compression

**Data Preview:**
- Klik "Preview" → tabel 100 baris pertama di panel editor
- Sort kolom, filter basic, export selection ke clipboard (JSON/CSV)

**IntelliSense untuk Schema JSON:**
- Autocomplete saat menulis schema QRD di JSON
- Validasi tipe field, nullability values

**Terminal Integration:**
```
> QRD: Inspect File          → jalankan qrd inspect di terminal terintegrasi
> QRD: Verify File           → jalankan qrd verify
> QRD: Convert to CSV        → konversi via qrd-convert
> QRD: Open in Inspector     → buka qrd-inspect TUI
```

**Syntax Highlight:**
- `.qrdschema` (JSON schema file) mendapat syntax highlighting khusus

---

### 6.2 `qrd-web-inspector`

**Web-based file inspector — upload dan eksplorasi file QRD di browser tanpa install apapun.**

Dihosting di `https://inspect.qrd.dev`. Berjalan 100% di browser menggunakan WASM — file tidak dikirim ke server.

#### Fitur

```
┌─────────────────────────────────────────────────────────────┐
│  🔍 QRD Web Inspector                    inspect.qrd.dev    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   [ Drop your .qrd file here or click to browse ]           │
│                                                              │
│   ─────────────────────────────────────────────────         │
│   After upload:                                             │
│                                                              │
│   📋 Schema Tab                                             │
│      → Semua kolom, tipe, encoding, compression             │
│      → Download schema sebagai JSON                         │
│                                                              │
│   📊 Statistics Tab                                         │
│      → Min/max/mean/null count per kolom                    │
│      → Distribusi histogram (ASCII dan visual)              │
│                                                              │
│   👁 Preview Tab                                            │
│      → 100 baris pertama dalam tabel interaktif             │
│      → Sort dan filter kolom                                │
│      → Export selection ke CSV/JSON                         │
│                                                              │
│   🔧 Row Groups Tab                                         │
│      → Daftar row group, ukuran, kompresi                   │
│      → Visualisasi kompresi ratio per row group             │
│                                                              │
│   ✅ Integrity Tab                                          │
│      → Validasi magic, version, CRC32                       │
│      → Status ECC jika enabled                              │
└─────────────────────────────────────────────────────────────┘
```

**Privasi:** File tidak meninggalkan browser. WASM engine berjalan lokal, tidak ada network request setelah halaman dimuat.

**Embedding:** Inspector dapat di-embed di dokumentasi atau aplikasi lain:
```html
<iframe 
  src="https://inspect.qrd.dev/embed" 
  allow="clipboard-write"
  style="width:100%; height:600px">
</iframe>
```

---

### 6.3 `qrd-schema-registry`

**Registry terpusat (self-hosted) untuk manajemen schema QRD lintas tim.**

Target: tim yang berbagi schema QRD antara beberapa service atau bahasa, dan ingin menghindari drift schema.

#### Konsep

Schema QRD memiliki Schema ID deterministik (SHA256 hash dari binary schema). Registry menyimpan schema berdasarkan ID ini dan menyediakan API untuk lookup, validasi, dan version tracking.

#### Self-hosted (Docker)

```bash
docker run -p 8080:8080 \
  -v ./schemas:/data \
  ghcr.io/nafalfaturizki/qrd-schema-registry:latest
```

#### API

```bash
# Register schema baru
curl -X POST https://registry.internal/schemas \
  -H "Content-Type: application/json" \
  -d @schema.json

# Lookup schema by ID
curl https://registry.internal/schemas/a3f7c2d1

# List semua schema
curl https://registry.internal/schemas

# Validasi file terhadap schema yang terdaftar
curl -X POST https://registry.internal/validate/a3f7c2d1 \
  --data-binary @file.qrd
```

#### Integrasi dengan CLI

```bash
# Push schema dari file ke registry
qrd schema input.qrd | \
  curl -X POST https://registry.internal/schemas -d @-

# Verifikasi file menggunakan schema dari registry
qrd verify --schema-registry https://registry.internal input.qrd
```

---

## 7. Tier 5 — Infrastructure & Operasional

---

### 7.1 `qrd-server`

**HTTP server minimalis untuk serve file QRD via REST API.**

Target: pipeline sederhana yang butuh mengekspos data QRD melalui HTTP tanpa setup database.

```bash
cargo install qrd-server
qrd-server --dir ./data --port 8080
```

#### Endpoints

```
GET  /files                    → List semua file .qrd di direktori
GET  /files/{name}/schema      → Schema sebagai JSON
GET  /files/{name}/inspect     → Metadata lengkap
GET  /files/{name}/rows        → Stream rows (JSON Lines)
GET  /files/{name}/rows?cols=id,value&limit=1000  → Partial read
GET  /files/{name}/row-group/{n}  → Row group tertentu
GET  /files/{name}/download    → Download raw file
POST /files/{name}             → Upload/replace file
```

#### Contoh

```bash
# Serve semua file QRD di direktori saat ini
qrd-server --dir . --port 8080

# Query dari klien
curl http://localhost:8080/files/events.qrd/rows?cols=id,timestamp&limit=100

# Download untuk diproses lokal
wget http://localhost:8080/files/events.qrd/download
```

---

### 7.2 `qrd-fuse`

**Mount direktori file QRD sebagai filesystem virtual (Linux/macOS).**

Dengan `qrd-fuse`, file QRD bisa di-mount dan diakses seolah-olah mereka adalah file CSV atau JSON Lines biasa — memungkinkan tool yang tidak tahu QRD untuk membaca data.

```bash
cargo install qrd-fuse

# Mount satu file
qrd-fuse mount data.qrd /mnt/mydata
ls /mnt/mydata/
# → schema.json  rows.jsonl  rows.csv  rowgroup_0.jsonl  rowgroup_1.jsonl ...

# Baca dengan tool biasa
head -100 /mnt/mydata/rows.jsonl
cat /mnt/mydata/schema.json
wc -l /mnt/mydata/rows.csv

# Unmount
qrd-fuse umount /mnt/mydata

# Mount direktori dengan banyak file QRD
qrd-fuse mount ./qrd-data/ /mnt/alldata
ls /mnt/alldata/
# → events.qrd/  metrics.qrd/  logs.qrd/
ls /mnt/alldata/events.qrd/
# → schema.json  rows.jsonl  rows.csv ...
```

---

### 7.3 `qrd-kafka-connector`

**Kafka Connect sink/source untuk menulis dan membaca QRD.**

Target: pipeline streaming yang menggunakan Kafka dan perlu persistensi ke QRD.

```json
{
  "name": "qrd-sink",
  "config": {
    "connector.class": "dev.qrd.kafka.QrdSinkConnector",
    "tasks.max": "4",
    "topics": "events,metrics",
    "qrd.output.dir": "/data/qrd/",
    "qrd.row.group.size": "1000000",
    "qrd.compression": "zstd",
    "qrd.file.rotation": "hourly",
    "qrd.encrypt": "false"
  }
}
```

```json
{
  "name": "qrd-source",
  "config": {
    "connector.class": "dev.qrd.kafka.QrdSourceConnector",
    "tasks.max": "2",
    "qrd.input.dir": "/data/qrd/",
    "qrd.topic.prefix": "qrd-replay-",
    "qrd.batch.size": "10000"
  }
}
```

---

### 7.4 `qrd-s3-sync`

**Sync dan upload file QRD ke object storage (S3, GCS, Azure Blob).**

```bash
cargo install qrd-s3-sync

# Upload file ke S3
qrd-s3-sync push data.qrd s3://mybucket/data/events/2026-05-09.qrd

# Download dari S3
qrd-s3-sync pull s3://mybucket/data/events/2026-05-09.qrd ./local/

# Sync direktori
qrd-s3-sync sync ./data/ s3://mybucket/data/

# Read langsung dari S3 (range request, tidak download full)
qrd-s3-sync inspect s3://mybucket/data/events.qrd

# Integrasi dengan qrd-cli
qrd-s3-sync pull s3://mybucket/data.qrd - | qrd read --format csv > output.csv
```

**Fitur:**
- Range requests untuk partial read dari object storage tanpa download full file
- Checksum verification setelah upload (membandingkan CRC32 QRD dengan ETag S3)
- Multi-part upload untuk file besar (>100MB)
- Support S3, GCS, Azure Blob, Cloudflare R2, MinIO

---

## 8. Tier 6 — Testing & Validasi

---

### 8.1 `qrd-test-vectors`

**Koleksi file QRD golden — ground truth untuk validasi semua implementasi.**

Setiap implementasi (SDK, tool, integrasi) harus bisa membaca dan memverifikasi semua test vector dengan benar. File golden ini adalah kontrak biner QRD yang paling otoritatif.

#### Struktur

```
test-vectors/
├── README.md                        # Deskripsi setiap vector
├── v1.0/
│   ├── minimal/
│   │   ├── empty.qrd                # File kosong, 0 rows
│   │   ├── single_row.qrd           # 1 row, 1 kolom
│   │   └── single_column.qrd        # 1 kolom, 1000 rows
│   │
│   ├── types/
│   │   ├── all_numeric_types.qrd    # Semua tipe numerik
│   │   ├── temporal_types.qrd       # TIMESTAMP, DATE, TIME, DURATION
│   │   ├── string_types.qrd         # STRING, ENUM, UUID
│   │   └── nullable_columns.qrd     # Semua kolom dengan null values
│   │
│   ├── encoding/
│   │   ├── plain.qrd                # Semua kolom: PLAIN encoding
│   │   ├── rle.qrd                  # RLE encoding
│   │   ├── delta_binary.qrd         # DELTA_BINARY
│   │   ├── bit_packed.qrd           # BIT_PACKED
│   │   ├── dictionary_rle.qrd       # DICTIONARY_RLE
│   │   ├── delta_byte_array.qrd     # DELTA_BYTE_ARRAY
│   │   └── byte_stream_split.qrd    # BYTE_STREAM_SPLIT
│   │
│   ├── compression/
│   │   ├── no_compression.qrd       # NONE
│   │   ├── zstd_level_3.qrd         # ZSTD level 3
│   │   └── lz4.qrd                  # LZ4
│   │
│   ├── encryption/
│   │   ├── aes256gcm.qrd            # File terenkripsi
│   │   ├── aes256gcm.key            # Key untuk decrypt
│   │   └── per_column_keys.qrd      # Per-column encryption
│   │
│   ├── ecc/
│   │   ├── ecc_4_2.qrd              # 4 data + 2 parity chunks
│   │   └── ecc_corrupted.qrd        # File dengan korupsi simulasi (bisa di-repair)
│   │
│   ├── scale/
│   │   ├── large_10M_rows.qrd       # 10 juta rows
│   │   ├── many_columns_100.qrd     # 100 kolom
│   │   └── multi_rowgroup.qrd       # 12 row groups
│   │
│   └── edge_cases/
│       ├── unicode_strings.qrd      # String UTF-8 dengan karakter multi-byte
│       ├── max_values.qrd           # INT64_MAX, FLOAT64_MAX, dll.
│       ├── all_nulls.qrd            # Semua nilai null di setiap kolom
│       └── empty_strings.qrd        # String kosong vs null
│
└── checksums.sha256                 # SHA256 dari setiap file vector
```

#### Penggunaan

```bash
# Clone test vectors
git clone https://github.com/nafalfaturizki/qrd-test-vectors

# Verifikasi implementasi baru terhadap semua vectors
qrd verify test-vectors/v1.0/**/*.qrd

# Jalankan test suite (dari dalam SDK)
cargo test --test golden_vectors
```

---

### 8.2 `qrd-fuzz`

**Fuzzing suite untuk menemukan bug parser, korupsi memori, dan panic.**

```bash
cargo install cargo-fuzz
cargo fuzz run fuzz_reader        # Fuzz FileReader dengan input acak
cargo fuzz run fuzz_footer        # Fuzz footer parser
cargo fuzz run fuzz_encoding      # Fuzz encoding/decoding pipeline
cargo fuzz run fuzz_schema        # Fuzz schema deserializer
cargo fuzz run fuzz_ecc_recover   # Fuzz ECC recovery dengan data rusak
```

Target fuzzing:
- Parser tidak boleh panic pada input apapun — harus return `Err`
- Tidak ada memory leak atau undefined behavior (terdeteksi via AddressSanitizer)
- ECC recovery tidak boleh silent-fail
- Tidak ada integer overflow di offset calculation

Corpus fuzzing dijalankan otomatis di CI untuk setiap PR yang menyentuh `reader/`, `footer/`, atau `encoding/`.

---

### 8.3 `qrd-compat-suite`

**Cross-SDK determinism dan compatibility test suite.**

Memverifikasi bahwa semua SDK menghasilkan output bit-identical untuk input yang sama.

```bash
# Jalankan full compatibility suite
qrd-compat-suite run --all

# Output:
# Testing write determinism across SDKs...
#
#   Rust   → write test_001.qrd: OK (sha256: a3f7c2d1...)
#   Python → write test_001.qrd: OK (sha256: a3f7c2d1...) ✓ MATCH
#   TypeScript → write test_001.qrd: OK (sha256: a3f7c2d1...) ✓ MATCH
#   Go     → write test_001.qrd: OK (sha256: a3f7c2d1...) ✓ MATCH
#   Java   → write test_001.qrd: OK (sha256: a3f7c2d1...) ✓ MATCH
#
# Testing read compatibility across SDKs...
#   All SDKs read Rust-generated files: ✓
#   All SDKs read Python-generated files: ✓
#   All SDKs read Go-generated files: ✓
#
# Cross-SDK Determinism: ✅ PASS (42/42 tests)
```

Test ini dijalankan di CI untuk setiap rilis SDK baru.

---

## 9. Roadmap Adopsi Ecosystem

```
2026 Q2 — Foundation
  ✅ qrd-core Phase 2 Complete
  🔄 qrd-cli (write, read, inspect, verify)
  🔄 qrd-convert (CSV, JSON Lines, Parquet)
  🔄 qrd-test-vectors v1.0

2026 Q3 — First Adopters
  🔲 qrd-python (PyO3, Pandas, NumPy)
  🔲 qrd-inspect (TUI)
  🔲 qrd-web-inspector (WASM)
  🔲 DuckDB extension v0.1
  🔲 qrd-fuzz (dasar)

2026 Q4 — Ecosystem Growth
  🔲 qrd-typescript (WASM, Node.js, Browser)
  🔲 qrd-go (CGO)
  🔲 Arrow bridge
  🔲 Polars plugin
  🔲 VS Code extension
  🔲 qrd-compat-suite

2027 Q1 — Enterprise Ready
  🔲 qrd-java (JNI, Spark integration)
  🔲 qrd-server
  🔲 qrd-schema-registry
  🔲 qrd-s3-sync
  🔲 DuckDB extension v1.0 (community registry)

2027 Q2+ — Infrastructure Layer
  🔲 qrd-kafka-connector
  🔲 qrd-fuse
  🔲 Grafana data source plugin
  🔲 Apache Superset connector
```

---

## 10. Dependency Graph Ecosystem

Setiap komponen ecosystem bergantung pada apa:

```
qrd-core (Rust)
  └── qrd-ffi
        ├── qrd-cli            (depends: qrd-core directly)
        ├── qrd-inspect        (depends: qrd-core)
        ├── qrd-convert        (depends: qrd-core + arrow2 + parquet2)
        ├── qrd-python         (depends: qrd-ffi via PyO3)
        │     ├── pandas plugin
        │     └── polars plugin
        ├── qrd-typescript     (depends: qrd-core via WASM)
        │     └── qrd-web-inspector (depends: qrd-typescript)
        ├── qrd-go             (depends: qrd-ffi via CGO)
        ├── qrd-java           (depends: qrd-ffi via JNI)
        │     └── Spark connector
        ├── DuckDB extension   (depends: qrd-ffi via C API)
        ├── qrd-server         (depends: qrd-core)
        ├── qrd-fuse           (depends: qrd-core + fuser)
        ├── qrd-s3-sync        (depends: qrd-core + aws-sdk-rust)
        └── qrd-kafka-connector (depends: qrd-java)
```

**Aturan:** Tidak ada komponen ecosystem yang boleh bergantung pada komponen ecosystem lain (kecuali SDK bergantung pada qrd-ffi). Setiap tool adalah standalone.

---

## 11. Standar Kontribusi Ecosystem

Setiap komponen ecosystem yang masuk ke organisasi resmi harus memenuhi:

**Teknis:**
- Dibangun di atas `qrd-core` atau `qrd-ffi` — tidak ada reimplementasi format
- Lulus semua golden test vectors di `qrd-test-vectors`
- Benchmark sebelum dan sesudah untuk operasi I/O
- Tidak ada dependency yang tidak diperlukan (minimalis)

**Kualitas:**
- README yang menjelaskan use case, instalasi, dan contoh penggunaan
- Error message yang jelas dan actionable — bukan "error occurred"
- Documented public API (rustdoc / pydoc / godoc / javadoc)
- Changelog mengikuti format Keep a Changelog

**Keamanan:**
- Tidak menerima input jaringan tanpa validasi
- Tidak ada hardcoded credentials atau keys
- Fuzz-tested untuk input adversarial (setidaknya 24 jam corpus)

**Cross-platform:**
- Linux (x86_64, ARM64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64) — kecuali `qrd-fuse` yang Linux/macOS only

---

*Ecosystem ini adalah jangka panjang. Kualitas setiap komponen lebih penting dari jumlah komponen. Satu tool yang bekerja dengan sempurna lebih bernilai dari sepuluh tool yang setengah jalan.*

---

**© 2026 NAFAL FATURIZKI — QRD Project**
