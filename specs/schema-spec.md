# Schema Specification

**Document Version:** 1.0.0  
**Status:** Draft

## Overview

Schema defines the structure of QRD files: column names, types, and nullability. This document specifies schema design, serialization, and validation.

## Core Concepts

### Fields

Each column is defined by a field:

```rust
pub struct Field {
    pub name: String,              // Column name (unique)
    pub logical_type: LogicalType, // Type (INT64, STRING, etc.)
    pub nullability: Nullability,  // REQUIRED, OPTIONAL, REPEATED
    pub metadata: Map<String, String>,
}
```

### Logical Types

QRD supports these logical types:

**Numeric:**
- BOOLEAN (1 bit/value)
- INT8, INT16, INT32, INT64 (signed)
- UINT8, UINT16, UINT32, UINT64 (unsigned)
- FLOAT32, FLOAT64 (IEEE 754)
- DECIMAL (arbitrary precision)

**Temporal:**
- TIMESTAMP (microseconds since epoch, INT64)
- DATE (days since 1970-01-01, INT32)
- TIME (microseconds since 00:00:00, INT64)
- DURATION (microseconds, INT64)

**Text:**
- UTF8_STRING (variable-length UTF-8)
- ENUM (categorical, limited values)
- UUID (128-bit identifier)

**Binary:**
- BLOB (variable-length binary)

**Composite (future):**
- STRUCT (fixed set of named fields)
- ARRAY (homogeneous repeated)
- MAP (key-value pairs)

### Nullability

```rust
pub enum Nullability {
    REQUIRED,   // No nulls permitted
    OPTIONAL,   // May contain null
    REPEATED,   // 0 or more elements (array)
}
```

**Implications:**

- **REQUIRED**: No null bitmap needed
- **OPTIONAL**: Null bitmap (1 bit per row)
- **REPEATED**: Array of values per row

## Schema Serialization

### Format

```
Schema Header:
  [version: U16LE]              = 1
  [field_count: U16LE]
  
For each field:
  [name_length: U16LE]
  [name: UTF-8 bytes]
  [logical_type_id: U8]         (0-31)
  [nullability_id: U8]          (0=REQUIRED, 1=OPTIONAL, 2=REPEATED)
  [metadata_count: U16LE]
  
  For each metadata key-value:
    [key_length: U16LE]
    [key: UTF-8 bytes]
    [value_length: U16LE]
    [value: UTF-8 bytes]
```

### Example

```
Schema with 3 columns:
  id (INT64, REQUIRED)
  name (UTF8_STRING, OPTIONAL)
  tags (UTF8_STRING, REPEATED)

Serialized:
  [0x01, 0x00]        Version = 1
  [0x03, 0x00]        Field count = 3
  
  Field 0 (id):
    [0x02, 0x00]      Name length = 2
    "id"              Name bytes
    [9]               Type = INT64 (9)
    [0]               Nullability = REQUIRED
    [0, 0]            No metadata
  
  Field 1 (name):
    [0x04, 0x00]      Name length = 4
    "name"            Name bytes
    [24]              Type = UTF8_STRING (24)
    [1]               Nullability = OPTIONAL
    [0, 0]            No metadata
  
  Field 2 (tags):
    [0x04, 0x00]      Name length = 4
    "tags"            Name bytes
    [24]              Type = UTF8_STRING (24)
    [2]               Nullability = REPEATED
    [0, 0]            No metadata
```

## Schema ID

Schema ID is deterministic SHA256 hash:

```rust
fn calculate_schema_id(fields: &[Field]) -> u32 {
    let mut hasher = Sha256::new();
    
    for field in fields {
        hasher.update(field.name.as_bytes());
        hasher.update(&[field.logical_type as u8]);
        hasher.update(&[field.nullability as u8]);
    }
    
    let hash = hasher.finalize();
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}
```

**Properties:**
- Identical schemas → identical IDs
- Used for validation
- Stored in file header
- Enables schema versioning

## Validation

### Field Validation

1. **Name**:
   - Not empty
   - Unique within schema
   - Valid UTF-8
   - Reasonable length (<256 chars)

2. **Type**:
   - Valid logical type ID
   - Supported by this QRD version

3. **Nullability**:
   - Valid enum value
   - Consistent with metadata

### Schema Validation

```rust
fn validate_schema(schema: &Schema) -> Result<()> {
    if schema.fields.is_empty() {
        return Err("Schema must have at least one field");
    }
    
    let mut seen_names = HashSet::new();
    
    for field in &schema.fields {
        // Validate field name
        if field.name.is_empty() {
            return Err("Field name cannot be empty");
        }
        
        if !seen_names.insert(&field.name) {
            return Err(format!("Duplicate field name: {}", field.name));
        }
        
        // Validate type
        validate_logical_type(field.logical_type)?;
        
        // Validate encoding for type
        validate_encoding_for_type(field.logical_type, field.encoding)?;
    }
    
    Ok(())
}
```

## Metadata

Fields can have optional metadata key-value pairs:

```rust
pub struct Field {
    pub name: String,
    pub logical_type: LogicalType,
    pub nullability: Nullability,
    pub metadata: Map<String, String>,  // Optional custom metadata
}
```

**Uses:**
- User-defined descriptions
- Encoding hints
- Compression hints
- Column-specific config
- Comments
- Statistics

**Example:**

```json
{
  "name": "user_age",
  "type": "INT32",
  "nullability": "OPTIONAL",
  "metadata": {
    "description": "Age in years",
    "min": "0",
    "max": "150",
    "comment": "Validated at ingestion time"
  }
}
```

## Type Mappings

### To/From Physical Types

**Logical → Physical:**

| Logical | Physical | Size |
|---------|----------|------|
| BOOLEAN | BYTE | 1 |
| INT8 | BYTE | 1 |
| INT16 | SHORT | 2 |
| INT32 | INT | 4 |
| INT64 | LONG | 8 |
| FLOAT32 | FLOAT | 4 |
| FLOAT64 | DOUBLE | 8 |
| UUID | FIXED(16) | 16 |
| STRING | VARIABLE | N/A |
| BLOB | VARIABLE | N/A |

## Encoding Selection by Type

Default encodings per logical type:

| Type | Default | Alternatives |
|------|---------|--------------|
| BOOLEAN | BIT_PACKED | PLAIN, RLE |
| INT8-64 | DELTA_BINARY (if sorted) | PLAIN, RLE, DeltaBinary |
| UINT8-64 | DELTA_BINARY (if sorted) | PLAIN, RLE |
| FLOAT* | BYTE_STREAM_SPLIT | PLAIN |
| STRING | DELTA_BYTE_ARRAY (if sorted) | PLAIN, DICTIONARY_RLE |
| BLOB | PASSTHROUGH | PLAIN |
| Timestamp | DELTA_BINARY | PLAIN |

## Evolution

### Forward Compatibility

- New logical types: Reader treats unknown type as BLOG
- New encodings: Treat as PLAIN
- New compression: Treat as NONE
- Extra metadata: Ignored

### Backward Compatibility

- Removed fields: Column offsets shift
- Type changes: Not allowed (requires new schema version)
- Renamed fields: Not allowedNeed new schema ID

### Version Bumping

When schema changes:

```
1. Old schema ID → cannot read new files
2. New schema ID → cannot read old files
3. Explicit version in file header
4. Reader must handle version mismatch
```

## Determinism

Schema serialization must be deterministic:

```rust
// Same fields → identical serialized bytes
let schema1 = build_schema(&fields);
let schema2 = build_schema(&fields);
assert_eq!(schema1.serialize(), schema2.serialize());
```

Requirements:
- Field order matters (no reordering)
- No random UUIDs in schema
- Metadata keys sorted (for consistency)
- Same version always

---

**End of Schema Specification**
