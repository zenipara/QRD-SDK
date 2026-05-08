# QRD Architecture Guide

## System Overview

QRD is built on a single-core principle: **Rust is the source of truth**.

```
┌─────────────────────────────┐
│     Language Bindings       │
│  (Python, Go, TypeScript)   │
└──────────┬──────────────────┘
           │
┌──────────▼──────────────────┐
│   C FFI Layer (qrd-ffi)     │
│  - C-compatible interfaces  │
│  - Memory management        │
│  - Error handling shim      │
└──────────┬──────────────────┘
           │
┌──────────▼──────────────────┐
│   Rust Core Engine          │
│                             │
│  Schema → Writer → Encoder  │
│       ↓                     │
│    Compressor → Footer      │
│       ↓                     │
│    Binary File              │
│                             │
│  Reader ← Decompressor      │
│    ↑ ← Decoder              │
└─────────────────────────────┘
```

## Module Architecture

### Core Modules

```
qrd-core/
├── schema/         - Type system and schema validation
├── encoding/       - Automatic encoding selection
├── compression/    - ZSTD/LZ4 codecs
├── writer/         - Streaming row ingestion
├── reader/         - Multi-mode read access
├── footer/         - Metadata serialization
├── validation/     - CRC32 and integrity checks
├── io/             - Buffered I/O
└── utils/          - Helpers (varint, bits, etc.)
```

### Data Flow: Writing

```
User Code
   ↓
write_row(values)
   ↓
Row Buffer (accumulate rows)
   ↓ [when row_group_size reached]
Row → Column Transposition
   ↓
For each column:
  - Select encoding (cardinality, entropy)
  - Encode data
  - Select compression (entropy threshold)
  - Compress data
  - Calculate CRC32
  ↓
Build row group metadata
  ↓
Flush to file
   ↓
[repeat until finish()]
   ↓
Build footer
  - Embed schema
  - Row group offsets
  - Statistics
  - Footer checksum
   ↓
Final flush
```

### Data Flow: Reading

#### Mode 1: Full Read

```
open_file()
   ↓
Read header (32 bytes)
   ↓
Seek to footer (file_size - 4)
   ↓
Parse footer:
  - Schema
  - Row group offsets
  - Statistics
   ↓
For each row group:
  - Read column chunks
  - Decompress (using codec from metadata)
  - Decode (using encoding from metadata)
  - Build row records
  - Yield rows
```

#### Mode 2: Partial Read (Specific Columns)

```
open_file()
   ↓
Read footer
   ↓
Select columns to read
   ↓
For each row group:
  - Skip unrequested columns
  - Read only requested columns
  - Decompress
  - Decode
  - Yield values
```

#### Mode 3: Footer-Only (Metadata)

```
open_file()
   ↓
Seek to footer
   ↓
Parse footer
   ↓
Return:
  - Schema
  - Row count
  - Statistics
  [No row data read]
```

## Encoding Selection Algorithm

```
For each column:
  1. Sample 1000 values (or all if < 1000)
  
  2. Detect cardinality
     if cardinality < 256 and is_lowcardinality:
       → DICTIONARY_RLE
  
  3. Detect sortedness
     if is_sorted:
       → DELTA_BINARY (numerics)
       → DELTA_BYTE_ARRAY (strings)
  
  4. Check data patterns
     if is_repetitive:
       → RLE
  
  5. Type-specific
     if type == BOOLEAN:
       → BIT_PACKED
     if type == BLOB or type == PASSTHROUGH:
       → PASSTHROUGH
  
  6. Default:
     → PLAIN
```

## Compression Selection Algorithm

```
For each encoded column chunk:
  1. Calculate entropy of first 1KB
     entropy = -sum(p_i * log2(p_i))
  
  2. High entropy (> 7.5)?
     → NONE (already compressed or random)
  
  3. Pre-compressed format detected?
     (magic bytes for JPEG, MP4, etc.)
     → NONE
  
  4. Streaming write mode?
     → LZ4 (ultra-fast)
  
  5. Default (archive/batch mode):
     → ZSTD (best ratio)
```

## Memory Management

### Writer Memory Usage

```
Total = row_group_size × avg_row_bytes + overhead

Example:
  row_group_size = 1M rows
  avg_row_bytes = 1KB
  ────────────────────────
  ≈ 1.3 GB

When row_group_size reached:
  1. Flush row group to file
  2. Clear buffers
  3. Continue accumulating
```

### Reader Memory Usage

```
Full read:  O(row_group_size) - buffers one group
Streaming:  O(row_size) - buffers one row
Footer:     O(footer_size) ≈ 1-10 KB
```

## Performance Characteristics

### Write Performance

- Row buffer: O(1) per row
- Encoding: O(n) where n = column size
- Compression: depends on codec (200-500 MB/s typical)
- Total throughput: 1-5 GB/s on modern x86_64

### Read Performance

- Full sequential: 2-10 GB/s
- Partial (specific columns): 5-20 GB/s
- Footer only: <1ms regardless of file size
- Random row access: O(row_group_size)

### Storage Compression

| Data Type | Ratio |
|-----------|-------|
| JSON | 8-15x |
| CSV | 10-20x |
| Text | 5-10x |
| Binary | 1.2-3x |
| Already compressed | 1.0-1.1x |

## Determinism

All operations must be **deterministic** and **reproducible**.

```
write_file(schema, rows) → bytes_1
write_file(schema, rows) → bytes_2

bytes_1 == bytes_2  ✓ Always true
```

This guarantees:
- Cache validation
- Diff detection
- Cross-SDK compatibility
- Version control friendly

## Error Handling

### Recoverable Errors

- Compression failed → store uncompressed
- Unknown encoding → treat as PLAIN
- Unknown compression → treat as NONE
- Extra metadata → ignore

### Unrecoverable Errors

- Invalid magic bytes
- Major version mismatch
- CRC checksum failed
- Corrupted schema
- Truncated file

## Threading Model

### Current (Single-threaded)

- All operations sequential
- Simple error handling
- Suitable for embedded/mobile

### Future (Multi-threaded)

```
Writer:
  - Main thread: row buffering
  - Worker threads: encode/compress
  - Channel-based work queue

Reader:
  - Parallel decompress multiple row groups
  - Parallel decode within group
```

## Security

### Current (None)

- No authentication
- No access control
- No built-in encryption (optional)

### Future Phases

- Optional AES-256-GCM (per-column)
- HKDF key derivation
- Authenticated encryption

## Extensibility

### Adding New Features

1. **New Encoding**:
   - Create `encoding/new_encoding.rs`
   - Implement `Encoder` trait
   - Register in `EncodingType`
   - Update selection algorithm
   - Add tests

2. **New Compression**:
   - Add to `CompressionCodec` enum
   - Implement compress/decompress
   - Add to selection algorithm

3. **New Type**:
   - Add to `LogicalType` enum
   - Define default encoding
   - Add validation rules

### Backward Compatibility

- Unknown features → graceful fallback
- Version numbers gate behaviors
- Test vectors ensure compatibility

---

**QRD is designed for long-term stability and wide adoption.**
