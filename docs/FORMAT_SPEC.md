# QRD Format Specification

## 1. Introduction

QRD is a streaming-first, columnar binary container designed for analytical workloads in edge, browser, and offline environments.

This specification is implementation-neutral and defines the binary format, encoding rules, metadata layout, and compatibility guarantees.

## 2. File Signature and Versioning

### 2.1 File Signature

Every QRD file begins with a fixed 4-byte magic sequence and a version header.

```
0x51 0x52 0x44 0x01  // "QRD"
```

### 2.2 Version Header

The file header includes a format version encoded as major/minor values.

```
Offset  Size  Field
0       4     MAGIC
4       2     VERSION_MAJOR
6       2     VERSION_MINOR
```

Readers must accept files with a major version equal to the supported version and use minor version compatibility rules for additive changes.

## 3. File Header

The file header is fixed-size and appears at byte offset zero.

```
Offset  Size  Type    Field
0       4     U32LE   MAGIC
4       2     U16LE   VERSION_MAJOR
6       2     U16LE   VERSION_MINOR
8       4     U32LE   SCHEMA_ID
12      4     U32LE   CREATED_AT
16      4     U32LE   TOTAL_ROW_COUNT
20      4     U32LE   COLUMN_COUNT
24      4     U32LE   ROW_GROUP_SIZE
28      4     U32LE   FLAGS
```

- `SCHEMA_ID`: deterministic schema fingerprint (first 4 bytes of SHA256).
- `CREATED_AT`: Unix seconds timestamp.
- `TOTAL_ROW_COUNT`: total logical rows in the file.
- `COLUMN_COUNT`: number of columns in the schema.
- `ROW_GROUP_SIZE`: configured target rows per group.
- `FLAGS`: reserved for future features; readers must ignore unknown bits.

## 4. Row Groups and Column Storage

### 4.1 Row Group Layout

QRD files are partitioned into row groups. Each row group contains one section per column.

```
[ROW_GROUP_HEADER]
[COLUMN_CHUNK_0]
[COLUMN_CHUNK_1]
...
[COLUMN_CHUNK_N]
[ROW_GROUP_FOOTER]
```

Row groups enable bounded-memory streaming writes and selective partial reads.

### 4.2 Column Chunk Layout

Each column chunk is stored as a self-contained block with encoding, compression, and optional checksums.

```
Offset  Size  Type    Field
0       1     U8      ENCODING_ID
1       1     U8      COMPRESSION_ID
2       4     U32LE   UNCOMPRESSED_LEN
6       4     U32LE   COMPRESSED_LEN
10      4     U32LE   NULL_COUNT
14      4     U32LE   DISTINCT_COUNT
18      B     BYTES   PAYLOAD
18+B    4     U32LE   CRC32
```

The payload contains the encoded column values, optionally compressed.

### 4.3 Columnar Storage

Columns are stored independently inside each row group. This enables:

- partial column reads without scanning unrelated data
- independent encoding selection per column
- better compression through homogeneous data layout

## 5. Schema Encoding

### 5.1 Schema Document

The file schema is serialized in the footer and describes each field.

Schema fields include:

- name
- logical type
- nullability
- default encoding hint
- default compression hint
- optional metadata

### 5.2 Logical Types

QRD supports the following logical types:

- `BOOLEAN`
- `INT8`, `INT16`, `INT32`, `INT64`
- `UINT8`, `UINT16`, `UINT32`, `UINT64`
- `FLOAT32`, `FLOAT64`
- `TIMESTAMP`
- `DATE`
- `TIME`
- `DURATION`
- `UTF8_STRING`
- `ENUM`
- `UUID`
- `BLOB`
- `DECIMAL`
- `STRUCT` (future)
- `ARRAY` (future)

### 5.3 Nullability

Supported nullability values:

- `REQUIRED`: no nulls allowed
- `OPTIONAL`: nullable values with bitmap
- `REPEATED`: zero or more values per row

### 5.4 Schema Serialization Format

```
[schema_version: U16LE]
[field_count: U16LE]
For each field:
  [name_len: U16LE]
  [name: UTF-8 bytes]
  [logical_type_id: U8]
  [nullability_id: U8]
  [metadata_count: U16LE]
  For each metadata entry:
    [key_len: U16LE]
    [key: UTF-8]
    [value_len: U16LE]
    [value: UTF-8]
```

The schema must be deterministic across all implementations.

### 5.5 Schema ID

A schema ID is derived from a deterministic hash of field names, types, and nullability.

- `SCHEMA_ID` is stored in the header.
- Readers use it to validate file identity and detect schema mismatches.

## 6. Encoding Algorithms

QRD uses logical encodings before compression. Supported encoding types:

- `PLAIN`
- `RLE`
- `BIT_PACKED`
- `DELTA_BINARY`
- `DELTA_BYTE_ARRAY`
- `BYTE_STREAM_SPLIT`
- `DICTIONARY_RLE`

### 6.1 PLAIN

PLAIN stores values in their raw serialized form.

### 6.2 RLE

RLE stores repeated values as `(run_length, value)` pairs.

### 6.3 BIT_PACKED

BIT_PACKED packs small integers and booleans into tightly packed bit sequences.

### 6.4 DELTA_BINARY

DELTA_BINARY stores differences between consecutive integer values.

### 6.5 DELTA_BYTE_ARRAY

DELTA_BYTE_ARRAY stores prefix-similar byte arrays as shared prefix lengths plus suffixes.

### 6.6 BYTE_STREAM_SPLIT

BYTE_STREAM_SPLIT rearranges floating-point bytes to improve compressibility.

### 6.7 DICTIONARY_RLE

DICTIONARY_RLE uses a dictionary for low-cardinality values and encodes indices with RLE.

## 7. Compression Sections

Compression is applied after encoding. Supported compression IDs:

- `NONE`
- `ZSTD`
- `LZ4`
- `GZIP` (reserved)

Compression selection is adaptive and may vary per column chunk.

## 8. Checksums and Integrity

Each column chunk includes a CRC32 of the uncompressed payload. The footer includes a checksum for its metadata section.

- Column chunk CRC protects column payloads.
- Footer CRC protects metadata, row group offsets, and schema.

Readers must verify checksums and reject corrupted files.

## 9. ECC and Parity Sections

QRD supports optional Reed-Solomon ECC at the row group level.

### 9.1 ECC Layout

When enabled, a row group may include:

- `DATA_CHUNKS`
- `PARITY_CHUNKS`

The ECC section is stored after column chunks and before the row group footer.

### 9.2 ECC Semantics

- ECC is optional and configured per file.
- Parity chunks are derived from column chunk payloads.
- Readers may recover missing or corrupted chunks when sufficient parity is available.

## 10. Encryption Metadata

Encryption is optional and applied after compression.

### 10.1 Encryption Fields

Encrypted column chunks include:

- `NONCE`
- `AUTH_TAG`
- optional `KEY_ID`

The footer includes encryption metadata describing the algorithm and key derivation.

### 10.2 Supported Encryption

- AES-256-GCM
- HKDF key derivation

Encrypted files remain readable by readers that support the same format version and key metadata.

## 11. Footer

The footer is located at the end of the file and contains the schema, row group offsets, optional statistics, and checksum.

```
[FOOTER]
[FOOTER_LENGTH: U32LE]
```

### 11.1 Footer Parsing

To parse:

1. Seek to end-of-file minus 4 bytes.
2. Read `FOOTER_LENGTH`.
3. Seek to `file_size - 4 - FOOTER_LENGTH`.
4. Read and parse the footer.

### 11.2 Footer Content

```
[version: U16LE]
[schema_length: U32LE]
[schema_bytes]
[row_group_count: U32LE]
[row_group_offsets: U64LE × N]
[statistics_flag: U8]
[statistics_length: U32LE]
[statistics_bytes]
[metadata_length: U32LE]
[metadata_bytes]
[checksum: U32LE]
```

## 12. Compatibility Guarantees

### 12.1 Forward Compatibility

- Readers may ignore unknown footer metadata fields.
- New schema fields are allowed if readers can tolerate unknown columns.
- Unknown compression or encoding IDs MUST be treated as unsupported, not corrupted.

### 12.2 Backward Compatibility

- Older readers must handle missing optional metadata.
- Files with newer minor versions may still be readable if no incompatible features are used.
- Major version bumps indicate incompatible binary layout changes.

### 12.3 Schema Compatibility

- Schema evolution is safe for additive metadata and new optional fields.
- Changing a field name or type is a breaking change and requires a new schema ID.

## 13. Extension Points

The format reserves:

- `FLAGS` in the header
- `COMPRESSION_ID` for future codecs
- footer metadata fields
- optional encryption and ECC metadata blocks

Extensions must remain opt-in and maintain reader safety.
