# Schema Evolution Guide

## Overview

Schema evolution enables safe changes to QRD file schemas while maintaining data compatibility. This guide covers:

- **Field renaming** with explicit mapping
- **Type changes** with compatibility checking
- **Forward and backward compatibility** rules
- **Breaking change** detection
- **Migration patterns** for real-world scenarios

## Key Concepts

### Type Compatibility

The schema evolution engine categorizes type changes into four compatibility levels:

| Compatibility | Description | Example | Impact |
|---|---|---|---|
| **Identical** | Same type on both sides | `Int32 → Int32` | No data conversion needed |
| **Widening** | Safe, lossless conversion | `Int32 → Int64` | Safe for all existing values |
| **Narrowing** | Potentially lossy conversion | `Int64 → Int32` | May lose precision on large values |
| **Incompatible** | Cannot convert | `Int64 → Boolean` | Requires special handling |

### Breaking Changes

A migration is **breaking** if:
- A **required field** is removed (old data becomes unreadable)
- A **required field** is added (new schema cannot read old data without defaults)
- **Incompatible type change** occurs on a required field
- Field **nullability** becomes more restrictive

**Example breaking changes:**
```rust
// BREAKING: Required field removed
Old: [user_id (REQUIRED), name (OPTIONAL)]
New: [name (OPTIONAL)]

// BREAKING: Required field added
Old: [user_id (REQUIRED)]
New: [user_id (REQUIRED), email (REQUIRED)]

// BREAKING: Type incompatibility
Old: [value: INT64 (REQUIRED)]
New: [value: BOOLEAN (REQUIRED)]
```

### Compatibility Guarantees

| Scenario | Forward Compatible | Backward Compatible | Meaning |
|---|---|---|---|
| Add optional field | ✅ | ✅ | Old and new readers both work |
| Remove optional field | ✅ | ✅ | Both can read both versions |
| Add required field | ❌ | ✅ | Old reader fails on new data |
| Remove required field | ✅ | ❌ | New reader fails on old data |
| Widen type (Int32→Int64) | ✅ | ✅ | Safe for both directions |
| Narrow type (Int64→Int32) | ✅ | ✅ | Marked as breaking, may lose data |

## Usage Examples

### 1. Safe Field Addition (Backward Compatible)

```rust
use qrd_core::schema::{FieldMetadata, FieldType, Nullability, Schema, SchemaEvolution};

let old_schema = Schema::new(vec![
    FieldMetadata {
        name: "id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "name".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Optional,
        metadata: HashMap::new(),
    },
]);

let new_schema = Schema::new(vec![
    FieldMetadata {
        name: "id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "name".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Optional,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "email".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Optional,  // Key: OPTIONAL field
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema)?;

// Check compatibility
assert!(!migration.is_breaking);
assert!(SchemaEvolution::is_backward_compatible(&migration));
assert!(SchemaEvolution::is_forward_compatible(&migration));
```

### 2. Safe Type Widening

```rust
// Widening Int32 to Int64 is always safe
let old_schema = Schema::new(vec![
    FieldMetadata {
        name: "count".to_string(),
        field_type: FieldType::Int32,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let new_schema = Schema::new(vec![
    FieldMetadata {
        name: "count".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema)?;

// Widening is safe and non-breaking
assert!(!migration.is_breaking);

// Check the specific mapping
let mapping = &migration.field_mappings[0].as_ref().unwrap();
assert_eq!(mapping.compatibility, TypeCompatibility::Widening);
```

### 3. Field Type Narrowing (Potentially Lossy)

```rust
// Narrowing Float64 to Float32 may lose precision
let old_schema = Schema::new(vec![
    FieldMetadata {
        name: "price".to_string(),
        field_type: FieldType::Float64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let new_schema = Schema::new(vec![
    FieldMetadata {
        name: "price".to_string(),
        field_type: FieldType::Float32,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema)?;

// Narrowing alone doesn't mark as breaking, but needs explicit handling
let mapping = &migration.field_mappings[0].as_ref().unwrap();
assert_eq!(mapping.compatibility, TypeCompatibility::Narrowing);

// Application should handle this conversion
// Example: truncate or round values to fit in Float32
```

### 4. Field Removal (Optional Fields Only)

```rust
let old_schema = Schema::new(vec![
    FieldMetadata {
        name: "id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "deprecated_field".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Optional,  // Safe to remove if OPTIONAL
        metadata: HashMap::new(),
    },
]);

let new_schema = Schema::new(vec![
    FieldMetadata {
        name: "id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema)?;

// Removing optional field is safe
assert!(!migration.is_breaking);
assert!(SchemaEvolution::is_forward_compatible(&migration));
```

### 5. Incompatible Type Change (Breaking)

```rust
// Changing Int64 to Boolean is incompatible
let old_schema = Schema::new(vec![
    FieldMetadata {
        name: "value".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let new_schema = Schema::new(vec![
    FieldMetadata {
        name: "value".to_string(),
        field_type: FieldType::Boolean,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema)?;

// This is a breaking change
assert!(migration.is_breaking);

let mapping = &migration.field_mappings[0].as_ref().unwrap();
assert_eq!(mapping.compatibility, TypeCompatibility::Incompatible);
```

## Type Compatibility Matrix

### Supported Type Conversions

#### Numeric Types
```
Widening (safe):
  Int8    → Int16, Int32, Int64
  Int16   → Int32, Int64
  Int32   → Int64
  UInt8   → UInt16, UInt32, UInt64
  UInt16  → UInt32, UInt64
  UInt32  → UInt64
  Float32 → Float64

Narrowing (lossy):
  Int64   → Int32, Int16, Int8
  Int32   → Int16, Int8
  Float64 → Float32
  Float64 → Int64, Int32, Int16, Int8
  Float32 → Int32, Int16, Int8

Incompatible:
  Any numeric ↔ Boolean
  Any numeric ↔ String (without explicit converter)
```

#### String Conversions
```
Widening (safe):
  UUID    → STRING (serialize to canonical form)
  STRING  → BLOB (encode as UTF-8)

Narrowing (lossy):
  STRING  → UUID (parse, may fail)
  BLOB    → STRING (decode UTF-8, may fail)
```

#### Temporal Types
```
Widening (safe):
  TIMESTAMP, DATE, TIME → STRING (serialize)

Narrowing (lossy):
  STRING  → TIMESTAMP (parse, may fail)
  STRING  → DATE (parse, may fail)
  STRING  → TIME (parse, may fail)
```

## Migration Planning API

The `SchemaEvolution` struct provides utilities for migration planning:

```rust
use qrd_core::schema::{Schema, SchemaEvolution, TypeCompatibility};

// Plan a migration
let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema)?;

// Inspect the migration
println!("Breaking change: {}", migration.is_breaking);
println!("Removed fields: {:?}", migration.removed_field_indices);
println!("New fields: {:?}", migration.new_fields);

// Check compatibility
if SchemaEvolution::is_backward_compatible(&migration) {
    println!("New readers can read old files");
}

if SchemaEvolution::is_forward_compatible(&migration) {
    println!("Old readers can read new files");
}

// Inspect field mappings
for (old_idx, mapping_opt) in migration.field_mappings.iter().enumerate() {
    if let Some(mapping) = mapping_opt {
        println!("Field {} ({}) → {} ({})",
            old_idx,
            mapping.old_name,
            mapping.new_name,
            mapping.compatibility
        );
    }
}
```

## Field Mapping Details

Each `FieldMapping` contains:

```rust
pub struct FieldMapping {
    pub old_name: String,                    // Original field name
    pub new_name: String,                    // New field name
    pub old_type: FieldType,                 // Original type
    pub new_type: FieldType,                 // New type
    pub compatibility: TypeCompatibility,    // Compatibility level
    pub converter: Option<String>,           // Custom converter function (if needed)
    pub nullability_changed: bool,           // Whether nullability changed
}
```

## Recommended Practices

### 1. **Plan Before Migrating**

Always use `plan_migration()` to understand the impact:

```rust
let migration = SchemaEvolution::plan_migration(&old_schema, &new_schema)?;

if migration.is_breaking {
    eprintln!("WARNING: This is a breaking change!");
    eprintln!("Removed required fields: {:?}", migration.removed_field_indices);
    
    // Decide how to handle the migration
    // Options:
    // 1. Use a new schema ID (creates new format version)
    // 2. Add default values for new required fields
    // 3. Update your application logic
}
```

### 2. **Document Schema Changes**

Keep a migration history in your application:

```rust
pub enum SchemaVersion {
    V1 = 1,  // 2024-01: Initial schema
    V2 = 2,  // 2024-06: Added 'email' field (optional)
    V3 = 3,  // 2024-09: Renamed 'user_id' to 'id'
    V4 = 4,  // 2024-12: Changed 'count' from Int32 to Int64
}
```

### 3. **Use Optional Fields for New Additions**

Always make new fields OPTIONAL to maintain backward compatibility:

```rust
// ✅ GOOD - backward compatible
FieldMetadata {
    name: "new_field".to_string(),
    field_type: FieldType::String,
    nullability: Nullability::Optional,
    metadata: HashMap::new(),
}

// ❌ BAD - breaks backward compatibility
FieldMetadata {
    name: "new_field".to_string(),
    field_type: FieldType::String,
    nullability: Nullability::Required,  // Old files cannot satisfy this
    metadata: HashMap::new(),
}
```

### 4. **Avoid Incompatible Type Changes**

Prefer safe conversions:

```rust
// ✅ GOOD - widening is safe
Int32 → Int64  // Always safe

// ⚠️ CAUTION - narrowing may lose data
Int64 → Int32  // Will fail for values > 2^31-1

// ❌ BAD - incompatible
Int64 → Boolean  // Cannot be converted
```

### 5. **Version Your Formats**

Increment the schema ID when making breaking changes:

```rust
// Old schema with ID 0x12345678
let old = Schema::new(vec![...]);
assert_eq!(old.schema_id, 0x12345678);

// Modified schema will have different ID
let new = Schema::new(vec![...]);
assert_ne!(new.schema_id, 0x12345678);

// Files are now incompatible at the format level
// Readers must handle version mismatch
```

## Real-World Example: Evolving a User Schema

### Phase 1: Initial Schema (v1)

```rust
let v1_schema = Schema::new(vec![
    FieldMetadata {
        name: "user_id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "username".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);
```

### Phase 2: Add Email (Backward Compatible)

```rust
let v2_schema = Schema::new(vec![
    FieldMetadata {
        name: "user_id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "username".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "email".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Optional,  // ← Key: Optional for compatibility
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&v1_schema, &v2_schema)?;
assert!(!migration.is_breaking);  // ✅ Safe to deploy
```

### Phase 3: Improve Type (Widening)

```rust
let v3_schema = Schema::new(vec![
    FieldMetadata {
        name: "user_id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "username".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "email".to_string(),
        field_type: FieldType::String,
        nullability: Nullability::Optional,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "created_at".to_string(),
        field_type: FieldType::Timestamp,
        nullability: Nullability::Optional,
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&v2_schema, &v3_schema)?;
assert!(!migration.is_breaking);  // ✅ Still safe
```

## Error Handling

The schema evolution API uses standard Rust error handling:

```rust
use qrd_core::schema::{SchemaEvolution};
use qrd_core::error::{Error, Result};

fn validate_migration(old: &Schema, new: &Schema) -> Result<()> {
    match SchemaEvolution::plan_migration(old, new) {
        Ok(migration) => {
            if migration.is_breaking {
                Err(Error::SchemaError(
                    "Breaking schema change detected".to_string()
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}
```

## Future Enhancements

The schema evolution system is extensible:

- **Explicit rename mappings**: Map renamed fields without treating as removal+addition
- **Custom converters**: User-defined functions for complex type conversions
- **Migration chains**: Automatically apply multiple sequential migrations
- **Schema registry**: Central repository for schema versions and compatibility rules
- **Streaming migration**: Transform data while reading/writing

## References

- [Schema Specification](../specs/schema-spec.md)
- [Compatibility Guarantees](../specs/compatibility.md)
- [QRD Format Specification](../docs/FORMAT_SPEC.md)
