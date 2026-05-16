# Schema Evolution API Reference

## Module: `qrd_core::schema::evolution`

The schema evolution module provides tools for planning and validating schema changes in QRD files.

## Types

### `TypeCompatibility`

```rust
pub enum TypeCompatibility {
    /// Types are identical, no conversion needed
    Identical,
    
    /// Lossless conversion (widening)
    /// Example: Int32 → Int64
    Widening,
    
    /// Potentially lossy conversion (narrowing)
    /// Example: Int64 → Int32
    Narrowing,
    
    /// Cannot convert between types
    /// Example: Int64 → Boolean
    Incompatible,
}
```

**Methods:**
- `is_safe(&self) -> bool` - Returns true for Identical and Widening
- `is_possible(&self) -> bool` - Returns true for all except Incompatible

---

### `FieldMapping`

Describes how a field is mapped from old schema to new schema.

```rust
pub struct FieldMapping {
    /// Original field name in old schema
    pub old_name: String,
    
    /// New field name in new schema
    pub new_name: String,
    
    /// Old field type
    pub old_type: FieldType,
    
    /// New field type
    pub new_type: FieldType,
    
    /// Type compatibility
    pub compatibility: TypeCompatibility,
    
    /// Optional name of conversion function
    pub converter: Option<String>,
    
    /// Whether nullability changed
    pub nullability_changed: bool,
}
```

---

### `SchemaMigration`

Complete migration plan from one schema to another.

```rust
pub struct SchemaMigration {
    /// Old schema ID
    pub old_schema_id: u32,
    
    /// New schema ID
    pub new_schema_id: u32,
    
    /// Field mappings (index in old schema → mapping)
    /// None if field is removed and not mapped
    pub field_mappings: Vec<Option<FieldMapping>>,
    
    /// New fields in new schema that don't exist in old schema
    pub new_fields: Vec<FieldMetadata>,
    
    /// Indices of removed fields from old schema
    pub removed_field_indices: Vec<usize>,
    
    /// Whether this is a breaking change
    pub is_breaking: bool,
    
    /// Migration version (currently 1)
    pub migration_version: u32,
}
```

---

### `TypeConversion`

Describes a supported type conversion.

```rust
pub struct TypeConversion {
    /// Source type
    pub from_type: FieldType,
    
    /// Target type
    pub to_type: FieldType,
    
    /// Conversion name (e.g., "int32_to_int64")
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Whether conversion is safe (non-lossy)
    pub is_safe: bool,
}
```

---

## Functions

### `SchemaEvolution::check_type_compatibility`

```rust
pub fn check_type_compatibility(from: FieldType, to: FieldType) -> TypeCompatibility
```

Checks compatibility when converting between two types.

**Returns:** One of `Identical`, `Widening`, `Narrowing`, or `Incompatible`

**Example:**
```rust
use qrd_core::schema::{FieldType, SchemaEvolution, TypeCompatibility};

let compat = SchemaEvolution::check_type_compatibility(
    FieldType::Int32,
    FieldType::Int64
);
assert_eq!(compat, TypeCompatibility::Widening);
```

---

### `SchemaEvolution::validate_field_rename`

```rust
pub fn validate_field_rename(
    old_field: &FieldMetadata,
    new_field: &FieldMetadata,
) -> Result<()>
```

Validates that a field can be renamed.

**Constraints:**
- Field type must be identical
- Nullability should remain the same

**Returns:** `Ok(())` if rename is valid, `Err` otherwise

**Example:**
```rust
use qrd_core::schema::{FieldMetadata, FieldType, Nullability, SchemaEvolution};
use std::collections::HashMap;

let old = FieldMetadata {
    name: "user_id".to_string(),
    field_type: FieldType::Int64,
    nullability: Nullability::Required,
    metadata: HashMap::new(),
};

let new = FieldMetadata {
    name: "id".to_string(),
    field_type: FieldType::Int64,
    nullability: Nullability::Required,
    metadata: HashMap::new(),
};

SchemaEvolution::validate_field_rename(&old, &new)?;
```

---

### `SchemaEvolution::validate_type_change`

```rust
pub fn validate_type_change(
    old_type: FieldType,
    new_type: FieldType,
    allow_breaking: bool,
) -> Result<TypeCompatibility>
```

Validates a type change with optional breaking change detection.

**Parameters:**
- `old_type` - Source type
- `new_type` - Target type
- `allow_breaking` - Whether to allow narrowing conversions

**Returns:** `Ok(compatibility)` on success, `Err` if incompatible or breaking and not allowed

**Example:**
```rust
use qrd_core::schema::{FieldType, SchemaEvolution, TypeCompatibility};

// Allow breaking change
let compat = SchemaEvolution::validate_type_change(
    FieldType::Int64,
    FieldType::Int32,
    true  // Allow narrowing
)?;
assert_eq!(compat, TypeCompatibility::Narrowing);

// Disallow breaking change
let result = SchemaEvolution::validate_type_change(
    FieldType::Int64,
    FieldType::Int32,
    false  // Disallow narrowing
);
assert!(result.is_err());
```

---

### `SchemaEvolution::plan_migration`

```rust
pub fn plan_migration(
    old_schema: &Schema,
    new_schema: &Schema,
) -> Result<SchemaMigration>
```

Plans a complete migration from one schema to another.

**Algorithm:**
1. Match fields by name
2. Check type compatibility for matched fields
3. Identify removed fields (in old but not in new)
4. Identify added fields (in new but not in old)
5. Determine if migration is breaking

**Returns:** `SchemaMigration` struct with complete migration plan

**Example:**
```rust
use qrd_core::schema::{FieldMetadata, FieldType, Nullability, Schema, SchemaEvolution};
use std::collections::HashMap;

let old = Schema::new(vec![
    FieldMetadata {
        name: "id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
]);

let new = Schema::new(vec![
    FieldMetadata {
        name: "id".to_string(),
        field_type: FieldType::Int64,
        nullability: Nullability::Required,
        metadata: HashMap::new(),
    },
    FieldMetadata {
        name: "timestamp".to_string(),
        field_type: FieldType::Timestamp,
        nullability: Nullability::Optional,
        metadata: HashMap::new(),
    },
]);

let migration = SchemaEvolution::plan_migration(&old, &new)?;
assert!(!migration.is_breaking);
```

---

### `SchemaEvolution::is_forward_compatible`

```rust
pub fn is_forward_compatible(migration: &SchemaMigration) -> bool
```

Checks if a migration is forward compatible (old reader can read new data).

**Conditions for forward compatibility:**
- No incompatible type changes
- Removed required fields don't affect reading

**Example:**
```rust
if SchemaEvolution::is_forward_compatible(&migration) {
    println!("Old code can read new files");
}
```

---

### `SchemaEvolution::is_backward_compatible`

```rust
pub fn is_backward_compatible(migration: &SchemaMigration) -> bool
```

Checks if a migration is backward compatible (new reader can read old data).

**Conditions for backward compatibility:**
- No required fields added (old data won't have values)
- No incompatible type changes

**Example:**
```rust
if SchemaEvolution::is_backward_compatible(&migration) {
    println!("New code can read old files");
}
```

---

### `SchemaEvolution::get_type_conversions`

```rust
pub fn get_type_conversions() -> Vec<TypeConversion>
```

Returns all supported type conversions.

**Includes conversions for:**
- Numeric type widening/narrowing
- String ↔ UUID
- String ↔ Blob
- Temporal ↔ String

**Example:**
```rust
let conversions = SchemaEvolution::get_type_conversions();

for conv in conversions {
    println!("{} ({}) → {} ({})",
        conv.name,
        if conv.is_safe { "safe" } else { "unsafe" },
        conv.from_type,
        conv.to_type
    );
}
```

---

## Common Patterns

### Check if Migration is Safe

```rust
fn is_migration_safe(old: &Schema, new: &Schema) -> Result<bool> {
    let migration = SchemaEvolution::plan_migration(old, new)?;
    
    Ok(!migration.is_breaking 
        && SchemaEvolution::is_backward_compatible(&migration)
        && SchemaEvolution::is_forward_compatible(&migration))
}
```

### Get Detailed Migration Report

```rust
fn report_migration(old: &Schema, new: &Schema) -> Result<()> {
    let migration = SchemaEvolution::plan_migration(old, new)?;
    
    println!("Migration Report");
    println!("===============");
    println!("Breaking change: {}", migration.is_breaking);
    println!("Forward compatible: {}", 
        SchemaEvolution::is_forward_compatible(&migration));
    println!("Backward compatible: {}",
        SchemaEvolution::is_backward_compatible(&migration));
    
    println!("\nRemoved fields:");
    for idx in &migration.removed_field_indices {
        println!("  - {} (index {})", 
            old.fields[*idx].name, idx);
    }
    
    println!("\nNew fields:");
    for field in &migration.new_fields {
        println!("  + {} ({})", field.name, field.field_type);
    }
    
    println!("\nType changes:");
    for (idx, mapping_opt) in migration.field_mappings.iter().enumerate() {
        if let Some(mapping) = mapping_opt {
            if mapping.old_type != mapping.new_type {
                println!("  {} {} → {}",
                    mapping.old_name,
                    mapping.old_type,
                    mapping.new_type);
            }
        }
    }
    
    Ok(())
}
```

### Conditional Migration with Approval

```rust
fn migrate_schema_safe(old: &Schema, new: &Schema) -> Result<bool> {
    let migration = SchemaEvolution::plan_migration(old, new)?;
    
    if migration.is_breaking {
        eprintln!("WARNING: Breaking change detected");
        eprintln!("This will require coordination with all consumers");
        // In real code, prompt user or check environment variable
        return Ok(false);
    }
    
    Ok(true)
}
```

## Testing

Schema evolution is tested in `core/qrd-core/tests/schema_evolution_test.rs`:

```bash
# Run all schema evolution tests
cargo test -p qrd-core schema_evolution_test

# Run specific test
cargo test -p qrd-core schema_evolution_test test_simple_field_rename
```

18 comprehensive tests cover:
- Field renaming
- Type widening/narrowing/incompatible changes
- Field addition/removal
- Breaking change detection
- Compatibility assessment
- Nullability changes
- Real-world migration scenarios

## Error Handling

The evolution API returns `crate::error::Error::SchemaError` for validation failures:

```rust
use qrd_core::error::Error;

match SchemaEvolution::validate_type_change(from, to, false) {
    Ok(compat) => {
        // Handle success
    }
    Err(Error::SchemaError(msg)) => {
        eprintln!("Schema validation failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

## Performance Characteristics

- `check_type_compatibility`: O(1)
- `plan_migration`: O(n) where n = total fields in both schemas
- `is_forward_compatible`: O(m) where m = number of field mappings
- `is_backward_compatible`: O(n + m)

For typical schemas (10-100 fields), these operations are instant.

## Limitations

Current implementation:
- Field matching is **name-based only** (no positional or semantic matching)
- No **explicit rename mappings** (renames appear as removal + addition)
- No **custom converter functions** in production (structure is available for future)
- No **schema registry** for version management
- No **migration chains** for sequential migrations

Future versions will address these limitations.
