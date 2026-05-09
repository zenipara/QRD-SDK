# Binary Layout Specification

**Document Version:** 1.0.0  
**Format Version:** 1.0  
**Status:** Draft

## Overview

This document specifies the precise binary layout of QRD files, including all headers, encoding formats, and data structures.

## Byte Order

**All integers are little-endian unless explicitly specified.**

```
// Example: 0x12345678 stored as:
78 56 34 12 (little-endian)
```

## File Header (32 bytes)

```
Offset  Size  Type    Field                Notes
------  ----  ----    -----                -----
0       4     U32LE   MAGIC                0x515244_01 ("QRD\x01")
4       2     U16LE   VERSION_MAJOR        Current: 1
6       2     U16LE   VERSION_MINOR        Current: 0
8       4     U32LE   SCHEMA_ID            SHA256 first 4 bytes
12      4     U32LE   CREATED_UNIX_SEC     Unix timestamp
16      4     U32LE   TOTAL_ROW_COUNT      All rows in file
20      4     U32LE   COLUMN_COUNT         Number of columns
24      4     U32LE   ROW_GROUP_SIZE       Rows per row group
28      4     U32LE   FLAGS                Reserved (currently zero)
```

### Magic Verification

```rust
const MAGIC: [u8; 4] = [0x51, 0x52, 0x44, 0x01]; // "QRD\x01"

fn verify_header(data: &[u8]) -> Result<()> {
    if data[0..4] != MAGIC {
        return Err("Invalid magic number");
    }
    let version_major = u16::from_le_bytes([data[4], data[5]]);
    if version_major > CURRENT_VERSION_MAJOR {
        return Err("Unsupported version");
    }
    Ok(())
}
```

## Row Group Structure

Each row group contains columns for a subset of rows.

```
Row Group Layout:

┌─────────────────────────────────┐
│ ROW_GROUP_HEADER (16B)          │
│ ├─ row_count (U32LE)            │
│ ├─ total_size (U32LE)           │ Uncompressed
│ ├─ compressed_size (U32LE)      │
│ └─ column_count (U16LE)         │
├─────────────────────────────────┤
│ COLUMN_CHUNK[0]                 │
├─────────────────────────────────┤
│ COLUMN_CHUNK[1]                 │
├─────────────────────────────────┤
│ ...                             │
├─────────────────────────────────┤
│ ROW_GROUP_METADATA (12 + 8*N)   │
│ ├─ row_count (U32LE)            │ Repeated
│ ├─ reserved (U32LE)             │
│ ├─ checksum (U32LE) CRC32       │
│ └─ column_offsets[n] (U64LE×N)  │
└─────────────────────────────────┘
```

## Column Chunk Layout

Each column chunk contains encoding and compression metadata followed by data.

```
Column Chunk:

Offset  Size  Type    Field               Notes
------  ----  ----    -----               -----
0       1     U8      ENCODING_ID         (See encoding table)
1       1     U8      COMPRESSION_ID      (See compression table)
2       4     U32LE   UNCOMPRESSED_LEN    Original data size
6       4     U32LE   COMPRESSED_LEN      Stored data size
10      4     U32LE   NULL_COUNT          Number of nulls (0 if required)
14      4     U32LE   DISTINCT_COUNT      Cardinality estimate
18      CS    BYTES   DATA                Compressed/encoded bytes
18+CS   4     U32LE   CRC32               Checksum of uncompressed data
```

### Encoding ID Table

```
ID  Name                Type
--  ----                ----
0   PLAIN               Unencoded
1   RLE                 Run-length encoding
2   BIT_PACKED          Packed binary
3   DELTA_BINARY        Delta-of-deltas
4   DELTA_BYTE_ARRAY    Delta for variable-length
5   BYTE_STREAM_SPLIT   Byte-level rearrangement
6   DICTIONARY_RLE      Dictionary with RLE
7   PASSTHROUGH         No encoding (pre-encoded)
```

### Compression ID Table

```
ID  Name               Block Size  Best For
--  ----               ----------  --------
0   NONE               N/A         Already compressed
1   ZSTD               64KB        Archive/high compression
2   LZ4                64KB        Real-time/low latency
3   GZIP               64KB        Compatibility (future)
```

## PLAIN Encoding

Raw column data stored sequentially.

```
For REQUIRED columns:
  [value_1_bytes] [value_2_bytes] ... [value_n_bytes]

For OPTIONAL columns:
  [null_bitmap] [value_1_bytes] [value_2_bytes] ...
  where null_bitmap is (n_rows + 7) / 8 bytes

Null bitmap:
  - 1 bit per value
  - 1 = NOT null
  - 0 = null
  - Padded to byte boundary
```

## RLE Encoding

Run-length encoding for repetitive data.

```
Format:
  [run_length_1: U32LE] [value_1] [run_length_2: U32LE] [value_2] ...

Example (INT32):
  run_length=5, value=42
  → [0x05, 0x00, 0x00, 0x00] [0x2A, 0x00, 0x00, 0x00]
```

## BIT_PACKED Encoding

For booleans and small integers (INT8 max).

```
Format:
  Bits packed into bytes, 8 bits per byte

Example (BOOLEAN):
  true, false, true, true, false, false, true, false
  → [0b10110100] = 0xB4
     1     0 1 1  0       1     0   0 1 0 0

Padded with zeros for incomplete bytes.
```

## Dictionary Encoding

For low-cardinality columns (DICTIONARY_RLE).

```
Format:
  [dict_size: U32LE]
  [value_size_1: U32LE] [value_1_bytes]
  [value_size_2: U32LE] [value_2_bytes]
  ...
  [value_size_n: U32LE] [value_n_bytes]
  [indices...] (1-4 bytes per index, encoded as RLE or plain)

Dictionary indices are 0-based.
```

## Null Handling

### PLAIN Encoding with Nulls

```
Total bytes = (row_count + 7) / 8 + sum(value_sizes)

[0 bytes      to    (row_count+7)/8 - 1]:  Null bitmap
[(row_count+7)/8  to   end          ]:  Actual values (skipped for nulls)
```

### Null Bitmap Format

```
Bit i corresponds to row i.
Row i is null if bit[i] == 0.
Row i is not null if bit[i] == 1.

Example 13 rows:
  Bitmap: 0x3F, 0x1F (13 bits needed, padded to 16)
  Binary: 0011_1111 0001_1111
  Rows:   col0=N, col1=V, col2=V, col3=V, col4=V, col5=V, col6=V, col7=V,
           col8=N, col9=N, col10=N, col11=N, col12=V, [padded]

  (In the example: value bits are numbered LSB-first within bytes)
```

## Footer Structure

The footer contains schema, row group offsets, and metadata.

```
File Structure:
  [FILE HEADER: 32 bytes]
  [ROW_GROUP_0]
  [ROW_GROUP_1]
  ...
  [ROW_GROUP_N]
  [FOOTER: variable]
  [FOOTER_LENGTH: 4 bytes] ← Points back to FOOTER

To read footer:
  seek(file_size - 4)
  footer_length = read_u32_le()
  seek(file_size - 4 - footer_length)
  footer_data = read(footer_length)
```

### Footer Content

```
Footer {
  version: U16LE,
  meta_len: U32LE,           // Length of serialized metadata below
  schema_len: U32LE,
  schema: [schema_bytes],
  row_group_count: U32LE,
  row_group_offsets: [U64LE × N],
  column_stats: [stats],     // Optional
  footer_checksum: U32LE,    // CRC32 of footer_data[0..len-4]
}
```

## Schema Serialization

```
Schema Format:

[version: U16LE]
[num_columns: U16LE]

For each column:
  [name_len: U16LE]
  [name_utf8: bytes]
  [logical_type: U8]
  [physical_type: U8]
  [nullability: U8]          // 0=REQUIRED, 1=OPTIONAL, 2=REPEATED
  [encoding: U8]
  [compression: U8]
  [metadata_count: U16LE]
  
  For each metadata:
    [key_len: U16LE] [key_utf8]
    [val_len: U16LE] [val_utf8]
```

### Type ID Tables

```
Logical Types:
  1 = BOOL
  2 = INT8, ..., 9 = INT64
  10 = UINT8, ..., 17 = UINT64
  18 = FLOAT32, 19 = FLOAT64
  20 = TIMESTAMP
  21 = DATE
  22 = TIME
  23 = DURATION
  24 = UTF8_STRING
  25 = ENUM
  26 = UUID
  27 = BLOB
  28 = DECIMAL
  50 = STRUCT
  51 = ARRAY
  99 = ANY

Physical Types:
  1 = BYTE (1 byte)
  2 = SHORT (2 bytes)
  4 = INT (4 bytes)
  8 = LONG (8 bytes)
  16 = FLOAT (4 bytes)
  32 = DOUBLE (8 bytes)
  0 = VARIABLE (string/blob)
```

## CRC32 Calculation

```rust
// Standard CRC32-IEEE (0x04C11DB7 polynomial)
// Applied to uncompressed column chunk data

fn calculate_crc32(data: &[u8]) -> u32 {
    // Use standard CRC32 algorithm
    // Initial value: 0xFFFFFFFF
    // Final XOR: 0xFFFFFFFF
}

// Stored as little-endian U32
```

## Example: 2-Row File

```
Hex Dump of minimal QRD file:

00000000: 51 52 44 01  01 00 00 00  AB CD 12 34  00 00 00 00
          M A G I C    VER MAJ MIN  SCHEMA_ID    TIMESTAMP

00000010: 02 00 00 00  02 00 00 00  01 00 00 00
          ROW_COUNT=2  COL_COUNT=2  ROW_GRP_SZ=1

Row Group 0 (2 rows, 2 columns):
00000020: 02 00 00 00  08 00 00 00  08 00 00 00  02 00
          ROW_CNT   UNCOMP_SZ     COMP_SZ      COL_CNT

Column 0 (INT32, PLAIN, no compression):
00000030: 00 01 04 00  00 00 04 00  00 00 00 00  00 00
          ENC PLN COMP  LEN        CLEN       NULL CNT

00000038: 2A 00 00 00  3B 00 00 00  12 34 56 78
          value_1=42   value_2=59   CRC32

Column 1 (UTF8_STRING):
00000040: 00 00 08 00  00 00 0B 00  00 00 00 00
          ENC PLN COMP  LEN=8      CLEN=11

00000048: 03 00 00 00  48 65 6C 6C  6F 03 00 00
          str_len=3    "Hel"       len=3

00000050: 00 00 57 6F  72 6C 64 87  65 43 21
          "World"                   CRC32

Footer:
00000058: ... (schema, offsets, metadata)
```

## Alignment and Padding

- **No required alignment** between chunks
- **No padding between fields** in structures
- **Null bitmaps:** Padded to byte boundary with zeros

## Versioning and Extensions

New format versions:
- Increment VERSION_MAJOR for breaking changes
- Increment VERSION_MINOR for additive changes
- Unknown encodings → treated as PLAIN
- Unknown compression → treated as NONE

---

**End of Binary Layout Specification**
