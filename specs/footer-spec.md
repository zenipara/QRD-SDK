# Footer Specification

**Document Version:** 1.0.0  
**Status:** Draft

## Overview

The footer is the metadata section at the end of a QRD file. It enables random access to row groups and columns without scanning the entire file.

## Footer Location

```
File Layout:
  [FILE_HEADER: 32 bytes]
  [ROW_GROUP_0]
  [ROW_GROUP_1]
  ...
  [ROW_GROUP_N]
  [FOOTER: variable]
  [FOOTER_LENGTH: 4 bytes] ← Points to FOOTER
  
To read footer:
  1. Seek to (file_size - 4)
  2. Read footer_length as U32LE
  3. Seek to (file_size - 4 - footer_length)
  4. Read footer_length bytes
  5. Parse footer
```

## Footer Structure

```
Footer {
  // Metadata
  [version: U16LE]              Version 1
  [schema_length: U16LE]
  [schema: bytes]               Serialized schema
  
  // Row groups
  [row_group_count: U32LE]
  [row_group_offsets: U64LE×N]  Byte offset to each row group
  
  // Statistics (optional)
  [has_statistics: U8]          0 or 1
  [statistics_length: U32LE]
  [statistics: bytes]
  
  // Metadata
  [created_at: U32LE]           Unix timestamp
  [modified_at: U32LE]
  [num_rows: U32LE]             Total rows in file
  
  // Checksum
  [footer_checksum: U32LE]      CRC32 of all above
}
```

**Total size:** Typically 1-10 KB for reasonable files

## Schema in Footer

The schema is serialized once in the footer (not repeated per row group):

```
[schema_version: U16LE]       = 1
[field_count: U16LE]

For each field:
  [name_length: U16LE]
  [name: UTF-8]
  [logical_type: U8]
  [nullability: U8]
  [encoding: U8]               (default for column, can be overridden per row group)
  [compression: U8]            (default, can be overridden)
  [metadata_count: U16LE]
  
  For each metadata pair:
    [key_length: U16LE]
    [key: UTF-8]
    [value_length: U16LE]
    [value: UTF-8]
```

## Row Group Offsets

Array of U64LE offsets pointing to the start of each row group:

```
[offset_0: U64LE]  → Start of row group 0 (includes its header)
[offset_1: U64LE]  → Start of row group 1
...
[offset_N: U64LE]  → Start of row group N
```

**Uses:**
- Seek directly to any row group
- Parallel decompression
- Streaming read efficiency

## Statistics (Optional)

Per-column statistics for query optimization:

```
Statistics {
  [column_count: U16LE]
  
  For each column:
    [has_min_max: U8]          1 if min/max present
    
    if has_min_max:
      [min_length: U16LE]
      [min: bytes]
      [max_length: U16LE]
      [max: bytes]
    
    [null_count: U32LE]        Nulls in column
    [distinct_count: U32LE]    Cardinality estimate
}
```

**Benefits:**
- Query pushdown (filter before reading)
- Cardinality estimates
- Missing value detection
- Sorted data detection

## Example Footer (40 columns, 4 row groups)

```
Typical sizes:
  Schema:              1.5 KB   (40 columns × ~38 bytes each)
  Row group offsets:   32 bytes (4 groups × 8 bytes)
  Statistics:          ~800 bytes
  Checksum & metadata: ~100 bytes
  ─────────────────────────────
  Total:              ~2.5 KB
```

## Footer Validation

```rust
fn validate_footer(footer: &Footer, file_size: u64) -> Result<()> {
    // Verify schema
    validate_schema(&footer.schema)?;
    
    // Verify row group offsets
    for (i, offset) in footer.row_group_offsets.iter().enumerate() {
        if *offset > file_size {
            return Err("Row group offset beyond file end");
        }
        
        if i > 0 && *offset <= footer.row_group_offsets[i-1] {
            return Err("Row group offsets not monotonic");
        }
    }
    
    // Verify checksum
    let calculated_crc = calculate_crc32(&footer_data[..footer_data.len()-4]);
    if calculated_crc != footer.checksum {
        return Err("Footer CRC mismatch");
    }
    
    Ok(())
}
```

## Reading Strategies

### Strategy 1: Full Read

```
1. Read file header
2. Seek to footer
3. Parse footer (get schema + row group offsets)
4. For each row group:
   a. Seek to row_group_offset
   b. Read and decompress
   c. Decode columns
   d. Yield rows
```

### Strategy 2: Partial Read (Specific Columns)

```
1. Read file header
2. Seek to footer
3. Parse footer
4. Find column IDs to read
5. For each row group:
   a. Seek to row_group_offset
   b. Read only specified columns
   c. Skip others (partial I/O)
```

### Strategy 3: Footer-Only (Metadata)

```
1. Read file header
2. Seek to footer
3. Parse footer
4. Get schema, row count, statistics
5. Use for query planning
6. Optionally read specific row groups
```

## Multi-File Operations

For distributed reads across multiple QRD files:

```
File 1 footer → schema + statistics
File 2 footer → schema + statistics
...
File N footer → schema + statistics

Union schema (all files)
Combine statistics
Parallel row group reads
```

## Checksum and Verification

```rust
pub fn verify_footer_integrity(
    file: &mut BufReader,
    footer_length: u32,
) -> Result<Footer> {
    // Read footer (excluding checksum)
    let footer_data = read_bytes(file, footer_length - 4)?;
    let stored_checksum = read_u32_le(file)?;
    
    // Calculate expected checksum
    let calculated = calculate_crc32(&footer_data);
    
    if calculated != stored_checksum {
        return Err("Footer checksum mismatch");
    }
    
    // Parse and return footer
    parse_footer(&footer_data)
}
```

## Memory Efficiency

Footer is typically small (< 10 KB) even for:
- Millions of rows (via row groups)
- Hundreds of columns (schema once)
- Large files (offsets only)

**Benefit**: Can load entire metadata into memory without buffering data

## Forward Compatibility

- Unknown metadata keys: Ignored
- Extra row groups: Skip to next known offset
- New statistics: Parsed, old readers ignore
- Version mismatch: Error on major version

---

**End of Footer Specification**
