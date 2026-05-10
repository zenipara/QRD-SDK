# Compatibility Specification

**Document Version:** 1.0.0  
**Status:** Draft

## Design Principle

**First:** QRD files written by version X MUST be readable by version X and later.  
**Second:** Simple readers should work with complex files (graceful degradation).

## Version Strategy

### Semantic Versioning

```
QRD vX.Y.Z

X = Major version (breaking format changes)
Y = Minor version (additive features, backward compatible)
Z = Patch version (bug fixes, no format changes)
```

### Compatibility Rules

| Writer Version | Reader Version | Result |
|---|---|---|
| 1.0.0 | 1.0.0 | ✓ Read perfectly |
| 1.0.0 | 1.1.0 | ✓ Read (ignore new features) |
| 1.0.0 | 2.0.0 | ✗ Error (major version) |
| 1.1.0 | 1.0.0 | ✗ Error (reader too old) |

### Version Checking

```
File Header [VERSION_MAJOR: U16LE][VERSION_MINOR: U16LE]

Reader check:
  if file_major > READER_MAJOR:
    error "Format version too new"
  if file_major == READER_MAJOR && file_minor > READER_MINOR:
    warn "File has features we don't understand, gracefully skipping"
```

## Forward Compatibility

Forward compatible means: new writer files can be read by old readers.

### New Encoding Types

Old reader encounters unknown encoding ID → treat as PLAIN:

```rust
match encoding_id {
    0 => EncodingType::Plain,
    1 => EncodingType::Rle,
    // ... known encodings
    99 => {
        // Unknown encoding in future version
        log::warn!("Unknown encoding {}, treating as PLAIN", encoding_id);
        EncodingType::Plain  // Fallback
    }
}
```

**Impact:** Data is readable but not optimally decoded.

### New Compression Codecs

Old reader encounters unknown compression → treat as NONE:

```rust
match compression_id {
    0 => CompressionCodec::None,
    1 => CompressionCodec::Zstd,
    // ... known codecs
    99 => {
        log::warn!("Unknown compression {}, treating as NONE", compression_id);
        CompressionCodec::None
    }
}
```

### New Field Types

Unknown logical type → treat as BLOB:

```rust
match type_id {
    1 => LogicalType::Boolean,
    // ... known types
    99 => {
        log::warn!("Unknown type {}, treating as BLOB", type_id);
        LogicalType::Blob  // Generic binary
    }
}
```

### Extra Metadata

Unknown metadata keys → ignore:

```rust
for (key, value) in field.metadata {
    match key.as_str() {
        "description" => process_description(value),
        "encoding_hint" => process_hint(value),
        // ...
        unknown => {
            // Unknown metadata is silently ignored
            log::debug!("Unknown metadata key: {}", unknown);
        }
    }
}
```

## Backward Compatibility

Files written by old versions should work with new readers.

### Handling Old Formats

```rust
match file_version {
    (1, 0) => parse_v1_0_format(),
    (1, 1) => parse_v1_1_format(),
    (1, _) => parse_v1_x_format(),  // Minor version compatible
    _ => error("Version not supported"),
}
```

### Default Values

When old format lacks a field:

```rust
let compression = read_compression()
    .unwrap_or(CompressionCodec::None);

let statistics = read_statistics()
    .unwrap_or_else(|| compute_statistics());
```

## Schema Evolution

### Column Addition

Old files don't have new columns:

```
Old schema:  [id, name]
New schema:  [id, name, score]  ← column 2 added

When reading old file with new schema:
- Column 0: id
- Column 1: name
- Column 2: DEFAULT (null for optional)
```

### Column Removal

New files don't write removed column:

```
Old schema:  [id, name, email]
New schema:  [id, name]         ← email removed

When reading old file with new schema:
- Column 0: id
- Column 1: name
- Rest ignored
```

### Column Type Change

Not supported (requires new schema ID):

```
Old schema:  id (INT64)
New schema:  id (STRING)        ← Type changed

Result: Different schema IDs, files incompatible
Error: Schema mismatch
```

### Column Rename

Not supported (name is part of field):

```
Old schema:  identifier (INT64)
New schema:  id (INT64)         ← Name changed

Result: Different schema signatures
Error: Schema mismatch
```

## SDK Compatibility

### Cross-SDK Format Guarantee

**All SDKs with same version produce identical bytes:**

```python
# Python
import qrd
writer = qrd.FileWriter("data.qrd", schema)
writer.write_rows(rows)
writer.finish()

# Go
import github.com/zenipara/QRD-SDK/sdk/go
writer := qrd.NewFileWriter("data.qrd", schema)
writer.WriteRows(rows)
writer.Finish()

# JavaScript
const qrd = require('qrd-sdk');
const writer = new qrd.FileWriter("data.qrd", schema);
writer.writeRows(rows);
writer.finish();

// Output: identical byte-for-byte
```

### Cross-SDK Determinism Tests

```rust
#[test]
fn test_cross_sdk_compatibility() {
    // Write data with Rust SDK
    let rust_bytes = write_with_rust_sdk(test_data);
    
    // Write same data with Python SDK
    let python_bytes = write_with_python_sdk(test_data);
    
    // Write same data with Go SDK
    let go_bytes = write_with_go_sdk(test_data);
    
    // All must be identical
    assert_eq!(rust_bytes, python_bytes);
    assert_eq!(rust_bytes, go_bytes);
}
```

### Version Mismatch Between SDKs

```
Python SDK v1.0 ↔ Rust SDK v1.1
- Rust writes with new compression codec
- Python reads with fallback to NONE
- Data is readable but not optimally compressed

Python SDK v2.0 ↔ Rust SDK v1.0
- Python writes major version 2
- Rust reads major version 1
- Error: version mismatch

Solution: Upgrade one SDK to match
```

## Test Vectors

QRD provides reference test vectors for compatibility testing:

```
test-vectors/
├── golden/
│   ├── v1_0/
│   │   ├── simple.qrd
│   │   ├── complex.qrd
│   │   └── large.qrd
│   └── v1_1/
│       ├── with_encryption.qrd
│       └── with_ecc.qrd
├── corrupted/
│   ├── truncated.qrd
│   ├── bad_checksum.qrd
│   └── invalid_header.qrd
└── README.md
```

**Usage:**

```rust
#[test]
fn test_read_cross_version_file() {
    let file = open_test_vector("v1_0/simple.qrd");
    let reader = FileReader::new(file)?;
    assert_eq!(reader.row_count(), 1000);
}
```

## Error Handling

### Version Mismatch

```rust
pub enum VersionError {
    TooNew,        // file_major > reader_major
    TooOld,        // file_major < reader_major (shouldn't happen)
    BadFormat,     // Invalid magic, corrupted header
}

match validate_version(&header) {
    Err(VersionError::TooNew) => {
        eprintln!("File from future QRD version - please upgrade");
        exit(1);
    }
    Err(VersionError::BadFormat) => {
        eprintln!("Corrupted or not a QRD file");
        exit(1);
    }
    Ok(()) => { /* proceed */ }
}
```

### Graceful Degradation

```rust
// Try to use optimization, fall back if unavailable
let data = match read_with_optimization(&reader) {
    Ok(data) => data,
    Err(OptimizationNotSupported) => {
        warn!("Optimization not available, using standard path");
        read_standard(&reader)?
    }
}
```

## Migration Path

When breaking changes occur:

### Phase 1: Announce
- Deprecation warning in v1.9
- Documentation of breaking change
- Migration guide

### Phase 2: Support Both
- v1.10-1.15: Support old and new formats
- Readers handle both transparently
- Writers use new format, warn on old

### Phase 3: Remove Old
- v2.0: Drop support for v1.x format
- All readers must upgrade
- Document in migration guide

---

**End of Compatibility Specification**
