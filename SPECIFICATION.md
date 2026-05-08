# QRD Specification

**Version:** 1.0.0-draft  
**Status:** In Development  
**Last Updated:** 2026-05-08

## Table of Contents

1. [Overview](#overview)
2. [Core Principles](#core-principles)
3. [Logical Types](#logical-types)
4. [Binary Format](#binary-format)
5. [Schema](#schema)
6. [Row Groups](#row-groups)
7. [Encoding](#encoding)
8. [Compression](#compression)
9. [Encryption](#encryption)
10. [ECC](#ecc)
11. [Footer](#footer)
12. [Streaming](#streaming)
13. [Reading](#reading)
14. [Determinism](#determinism)

---

## Overview

QRD (Columnar Row Descriptor) is a columnar binary format designed for:

- **Streaming**: Incremental writes without whole-file buffering
- **Efficiency**: Row→column transposition with intelligent encoding/compression
- **Compatibility**: Forward/backward compatible versioning
- **Accessibility**: Partial reads via footer metadata
- **Safety**: Integrity verification with CRC32
- **Performance**: Zero-copy reads, SIMD-friendly encoding

## Core Principles

1. **Deterministic**: Identical input → identical binary output
2. **Columnar**: Row→column transposition for better compression
3. **Streaming-First**: Unbounded row ingestion with configurable row groups
4. **Little-Endian**: All multi-byte integers are little-endian
5. **Versioned**: Format version in header for evolution
6. **Footer-Driven**: Metadata in footer for random access
7. **Self-Describing**: Schema embedded in file
8. **Checksum-Protected**: CRC32 for corruption detection

## Logical Types

### Numeric Types

| Type | Bytes | Range | Physical |
|------|-------|-------|----------|
| BOOLEAN | 1 | true/false | bit-packed |
| INT8 | 1 | -128 to 127 | signed byte |
| INT16 | 2 | -32768 to 32767 | signed short |
| INT32 | 4 | -2³¹ to 2³¹-1 | signed int |
| INT64 | 8 | -2⁶³ to 2⁶³-1 | signed long |
| UINT8 | 1 | 0 to 255 | unsigned byte |
| UINT16 | 2 | 0 to 65535 | unsigned short |
| UINT32 | 4 | 0 to 2³²-1 | unsigned int |
| UINT64 | 8 | 0 to 2⁶⁴-1 | unsigned long |
| FLOAT32 | 4 | IEEE 754 single | 4-byte float |
| FLOAT64 | 8 | IEEE 754 double | 8-byte float |

### Temporal Types

| Type | Bytes | Format | Example |
|------|-------|--------|---------|
| TIMESTAMP | 8 | Unix microseconds (INT64) | 1609459200000000 |
| DATE | 4 | Days since 1970-01-01 (INT32) | 18628 |
| TIME | 8 | Microseconds since 00:00:00 (INT64) | 43200000000 (12:00 PM) |
| DURATION | 8 | Microseconds (INT64) | 3600000000 (1 hour) |

### Text Types

| Type | Format | Max Size |
|------|--------|----------|
| UTF8_STRING | variable | 2GB per string |
| ENUM | UTF8 with index | 65535 values |
| UUID | 16 bytes | 128-bit UUID |

### Binary Types

| Type | Format | Max Size |
|------|--------|----------|
| BLOB | variable | 2GB per blob |
| DECIMAL | variable | arbitrary precision |

### Composite Types

| Type | Description |
|------|-------------|
| STRUCT | Fixed set of named fields |
| ARRAY | Homogeneous repeated elements |
| ANY | Schema validation disabled |

### Nullability

- **REQUIRED**: No null values permitted
- **OPTIONAL**: May contain null value markers
- **REPEATED**: 0 or more elements (array-like)

---

## Binary Format

### File Structure

```
┌─────────────────────────┐
│   FILE HEADER (32B)     │  Magic, version, metadata
├─────────────────────────┤
│   ROW GROUP 0           │  Column chunks + row group metadata
├─────────────────────────┤
│   ROW GROUP 1           │
├─────────────────────────┤
│   ...                   │
├─────────────────────────┤
│   ROW GROUP N           │
├─────────────────────────┤
│   FOOTER                │  Schema, statistics, offsets
├─────────────────────────┤
│   FOOTER LENGTH (4B)    │  Size of footer in bytes
└─────────────────────────┘
```

### File Header (32 bytes)

```
Offset  Size  Field               Description
------  ----  -----               -----------
0       4     MAGIC               "QRD\x01" (0x515244_01)
4       2     VERSION_MAJOR       Current: 1
6       2     VERSION_MINOR       Current: 0
8       4     SCHEMA_ID           Deterministic schema hash
12      4     CREATED_TIMESTAMP   Unix seconds
16      4     ROW_COUNT           Total rows in file
20      4     COLUMN_COUNT        Total columns
24      4     ROW_GROUP_SIZE      Rows per row group (config)
28      4     RESERVED            For future use
```

### Row Group Structure

```
┌─────────────────────────────────┐
│ ROW GROUP HEADER (16B)          │
├─────────────────────────────────┤
│ COLUMN_CHUNK[0]                 │ Type-specific encoding
├─────────────────────────────────┤
│ COLUMN_CHUNK[1]                 │
├─────────────────────────────────┤
│ ...                             │
├─────────────────────────────────┤
│ COLUMN_CHUNK[N]                 │
├─────────────────────────────────┤
│ ROW GROUP METADATA              │
│   - Row count                   │
│   - Column offsets              │
│   - Compression codecs          │
│   - Row group CRC32             │
└─────────────────────────────────┘
```

### Column Chunk Structure

```
Column Chunk:
  Encoding: 1 byte (PLAIN, RLE, BIT_PACKED, etc.)
  Compression: 1 byte (NONE, ZSTD, LZ4)
  Length: 4 bytes (uncompressed)
  Compressed Length: 4 bytes
  Data: variable
  CRC32: 4 bytes (of uncompressed data)
```

### Footer

```
Footer:
  Schema (serialized, variable length)
  Statistics (per column, optional)
  Row Group Offsets (4 bytes × N)
  Column Metadata (per column)
  Footer Checksum (CRC32)
```

---

## Schema

### Schema Serialization

```
Schema:
  Version (2 bytes): 1
  Num Columns (2 bytes)
  
  For each column:
    - Name (UTF8, length-prefixed)
    - Logical Type (1 byte)
    - Physical Type (1 byte)
    - Nullability (1 byte): REQUIRED, OPTIONAL, REPEATED
    - Encoding (1 byte)
    - Compression (1 byte)
    - Metadata (key-value pairs, length-prefixed)
```

### Schema Hashing

Schema ID is deterministic SHA256 hash of:
```
schema_bytes || SCHEMA_VERSION_CONSTANT
```

Used for:
- File validation
- Compatibility checking
- Deduplication

---

## Row Groups

### Row Group Semantics

- Configurable size (e.g., 100K, 1M rows)
- Independent compression across groups
- Parallel encoding opportunity
- Random access via footer offsets

### Row Group Metadata

```
Offset  Size  Field
------  ----  -----
0       4     ROW_COUNT
4       4     TOTAL_COMPRESSED_SIZE
8       4     TOTAL_UNCOMPRESSED_SIZE
12      N×4   COLUMN_OFFSETS
```

---

## Encoding

Automatic encoding selection based on:

1. **Logical Type** → default encoding
2. **Cardinality** → sample column
3. **Entropy** → dictionary vs plain
4. **Sortedness** → delta vs plain

### Available Encodings

| Encoding | Description | Best For |
|----------|-------------|----------|
| PLAIN | Raw bytes | Mixed data |
| RLE | Run-length encoding | Repetitive data |
| BIT_PACKED | Bit-level packing | Booleans, small integers |
| DELTA_BINARY | Delta-of-deltas | Sorted integers |
| DELTA_BYTE_ARRAY | Delta for strings | Sorted strings |
| BYTE_STREAM_SPLIT | Byte-level split | Floating point |
| DICTIONARY_RLE | Dictionary + RLE | Low cardinality |
| PASSTHROUGH | No encoding | Pre-compressed blobs |

### Encoding Selection Algorithm

```
1. If BLOB type → PASSTHROUGH
2. If BOOL type → BIT_PACKED
3. If String type:
   a. Sample 1000 values
   b. If cardinality < 256 → DICTIONARY_RLE
   c. If sorted → DELTA_BYTE_ARRAY
   d. Else → PLAIN
4. If Numeric type:
   a. If sorted → DELTA_BINARY
   b. If all same row → RLE
   c. Else → PLAIN
5. Default → PLAIN
```

---

## Compression

### Compression Rules

1. **Compress After Encoding**
2. **Skip Pre-Compressed**:
   - JPEG, MP4, WebP
   - Detected via entropy threshold
3. **Adaptive**:
   - LZ4 for real-time streams
   - ZSTD for archives
   - NONE for incompressible

### Compression Selection

```
If entropy > 7.5 bits/byte → NONE
  (Already highly compressed)

If stream mode:
  → LZ4 (level 4) for speed
Else:
  → ZSTD (level 3-10, auto-optimized)
```

---

## Encryption

### Algorithm

- **Cipher**: AES-256-GCM
- **Key Derivation**: HKDF-SHA256
- **Authentication**: Built-in GCM tag

### Per-Column Encryption

```
master_key → HKDF-SHA256 → column_key
             input: column_id

Each column encrypted independently:
  plaintext ⟶ AES-256-GCM ⟶ ciphertext + tag
```

### Compression Before Encryption

```
uncompressed ⟶ compress ⟶ encrypt
```

---

## ECC

### Reed-Solomon Code

- **Configurable**: 1-32 parity chunks
- **Per Row Group**: Independent reconstruction
- **Recovery**: Up to 50% chunk loss

### ECC Metadata

```
ECC Level (1 byte)
Data Chunks (1 byte)
Parity Chunks (1 byte)
```

---

## Footer

### Offset Calculation

```
Footer position = File size - 4 - footer_length
Footer length = read 4-byte value at (File size - 4)
```

### Footer Contents

```
Footer {
  schema: Schema,
  row_group_offsets: Vec<u64>,
  column_metadata: Vec<ColumnMeta>,
  statistics: Option<Statistics>,
  created_at: u64,
  modified_at: u64,
  footer_checksum: u32 (CRC32),
}
```

---

## Streaming

### Writer Contract

```
writer.write_row({ column1: v1, column2: v2, ... })
  ↓ Buffers row
  ↓ When buffer hits row_group_size
  ↓ Transposition (rows → columns)
  ↓ Encode each column
  ↓ Compress each column
  ↓ Flush row group
  ↓ Continue...
```

### Memory Bounded

```
Max memory = row_group_size × avg_row_bytes + overhead

Example: 1M rows, 1KB/row ≈ 1.3GB per group
```

---

## Reading

### Read Modes

1. **Full Read**: All rows
2. **Streaming Read**: Iterator over rows
3. **Partial Read**: Specific columns
4. **Footer Lookup**: Metadata without rows

### Random Access

```
1. Seek to EOF - 4
2. Read footer_length (4 bytes)
3. Seek to EOF - 4 - footer_length
4. Read footer (parse offsets)
5. Seek to row_group offsets
6. Decompress, decode, read specific columns
```

---

## Determinism

### Requirements for Deterministic Output

1. **Reproducible Encoding**:
   - Same algorithm given same input
   - No random seeds
   - No timestamps in encoding

2. **Fixed Schema ID**:
   - Schema must hash identically
   - Column order matters
   - Metadata must be canonical

3. **Reproducible Compression**:
   - No random initialization vectors for compression
   - Fixed compression level
   - Deterministic codec selection

4. **Bit-Perfect Matching**:
   - All SDKs produce identical bytes
   - For identical schema, row order, and configuration

### Testable

```
write_file(schema, rows) → file_bytes_1
write_file(schema, rows) → file_bytes_2
assert file_bytes_1 == file_bytes_2  ✓
```

---

## Compatibility

### Forward Compatibility

- Unknown encodings → treat as PLAIN
- Unknown compression → skip decompression
- Extra metadata → ignore

### Backward Compatibility

- Optional fields have defaults
- Version checking before parsing
- Graceful degradation for new features

### Schema Evolution

- Column addition → new column ID
- Column removal → mark as deleted
- Type changes → version bump required

---

## Performance

### Benchmarks

Target on modern x86_64:

| Operation | Throughput |
|-----------|-----------|
| Write (1KB rows) | 1-5 GB/s |
| Read (full) | 2-10 GB/s |
| Read (partial) | 5-20 GB/s |
| Compression | 500MB-2GB/s (ZSTD) |
| Decompression | 1-5 GB/s |

---

## References

- ParquetFormat: columnar format inspiration
- Protocol Buffers: deterministic serialization
- TAR: streaming-friendly layout
- Arrow: columnar memory model

---

**End of Specification**
