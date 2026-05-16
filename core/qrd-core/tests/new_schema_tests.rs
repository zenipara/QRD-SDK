//! Additional schema and field type tests

use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use tempfile::NamedTempFile;

#[test]
fn test_schema_builder_add_field_int8() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int8, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Int8);
}

#[test]
fn test_schema_builder_add_field_int16() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int16, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Int16);
}

#[test]
fn test_schema_builder_add_field_uint8() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::UInt8, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::UInt8);
}

#[test]
fn test_schema_builder_add_field_uint16() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::UInt16, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::UInt16);
}

#[test]
fn test_schema_builder_add_field_uint32() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::UInt32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::UInt32);
}

#[test]
fn test_schema_builder_add_field_uint64() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::UInt64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::UInt64);
}

#[test]
fn test_schema_builder_add_field_float32() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Float32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Float32);
}

#[test]
fn test_schema_builder_add_field_float64() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Float64);
}

#[test]
fn test_schema_builder_add_field_boolean() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Boolean);
}

#[test]
fn test_schema_builder_add_field_string() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::String, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::String);
}

#[test]
fn test_schema_builder_add_field_blob() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Blob);
}

#[test]
fn test_schema_builder_add_field_uuid() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Uuid, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Uuid);
}

#[test]
fn test_schema_builder_add_field_timestamp() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Timestamp, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Timestamp);
}

#[test]
fn test_schema_builder_add_field_date() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Date, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Date);
}

#[test]
fn test_schema_builder_add_field_time() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Time, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Time);
}

#[test]
fn test_schema_builder_add_field_duration() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Duration, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Duration);
}

#[test]
fn test_schema_builder_add_field_decimal() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Decimal, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Decimal);
}

#[test]
fn test_schema_builder_add_field_enum() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Enum, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].field_type, FieldType::Enum);
}

#[test]
fn test_schema_nullability_required() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].nullability, Nullability::Required);
}

#[test]
fn test_schema_nullability_optional() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].nullability, Nullability::Optional);
}

#[test]
fn test_schema_nullability_repeated() {
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Repeated)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].nullability, Nullability::Repeated);
}

#[test]
fn test_schema_multiple_fields_2() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields.len(), 2);
}

#[test]
fn test_schema_multiple_fields_5() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("age", FieldType::Int32, Nullability::Optional)
        .unwrap()
        .add_field("balance", FieldType::Float64, Nullability::Optional)
        .unwrap()
        .add_field("active", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields.len(), 5);
}

#[test]
fn test_schema_multiple_fields_10() {
    let mut builder = SchemaBuilder::new();
    for i in 0..10 {
        builder = builder.add_field(&format!("field_{}", i), FieldType::Int32, Nullability::Required).unwrap();
    }
    let schema = builder.build().unwrap();
    assert_eq!(schema.fields.len(), 10);
}

#[test]
fn test_schema_multiple_fields_50() {
    let mut builder = SchemaBuilder::new();
    for i in 0..50 {
        builder = builder.add_field(&format!("field_{}", i), FieldType::Int32, Nullability::Required).unwrap();
    }
    let schema = builder.build().unwrap();
    assert_eq!(schema.fields.len(), 50);
}

#[test]
fn test_schema_multiple_fields_100() {
    let mut builder = SchemaBuilder::new();
    for i in 0..100 {
        builder = builder.add_field(&format!("field_{}", i), FieldType::Int32, Nullability::Required).unwrap();
    }
    let schema = builder.build().unwrap();
    assert_eq!(schema.fields.len(), 100);
}

#[test]
fn test_schema_field_name_special_chars() {
    let schema = SchemaBuilder::new()
        .add_field("field_with_underscore", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].name, "field_with_underscore");
}

#[test]
fn test_schema_field_name_with_numbers() {
    let schema = SchemaBuilder::new()
        .add_field("field123", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema.fields[0].name, "field123");
}

#[test]
fn test_schema_duplicate_field_rejected() {
    let result = SchemaBuilder::new()
        .add_field("name", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::Int64, Nullability::Required);
    assert!(result.is_err());
}

#[test]
fn test_schema_empty_field_name_rejected() {
    let result = SchemaBuilder::new()
        .add_field("", FieldType::Int32, Nullability::Required);
    assert!(result.is_err());
}

#[test]
fn test_schema_id_deterministic() {
    let schema1 = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let schema2 = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(schema1.schema_id, schema2.schema_id);
}

#[test]
fn test_schema_id_changes_with_different_field() {
    let schema1 = SchemaBuilder::new()
        .add_field("field_a", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let schema2 = SchemaBuilder::new()
        .add_field("field_b", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_ne!(schema1.schema_id, schema2.schema_id);
}

#[test]
fn test_schema_clone() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let cloned = schema.clone();
    assert_eq!(schema.fields.len(), cloned.fields.len());
    assert_eq!(schema.schema_id, cloned.schema_id);
}

#[test]
fn test_field_type_id_int32() {
    assert_eq!(FieldType::Int32.id(), 4);
}

#[test]
fn test_field_type_id_int64() {
    assert_eq!(FieldType::Int64.id(), 5);
}

#[test]
fn test_field_type_id_float32() {
    assert_eq!(FieldType::Float32.id(), 18);
}

#[test]
fn test_field_type_id_float64() {
    assert_eq!(FieldType::Float64.id(), 19);
}

#[test]
fn test_field_type_id_string() {
    assert_eq!(FieldType::String.id(), 24);
}

#[test]
fn test_field_type_id_blob() {
    assert_eq!(FieldType::Blob.id(), 27);
}

#[test]
fn test_field_type_fixed_size_int32() {
    assert_eq!(FieldType::Int32.fixed_size(), Some(4));
}

#[test]
fn test_field_type_fixed_size_int64() {
    assert_eq!(FieldType::Int64.fixed_size(), Some(8));
}

#[test]
fn test_field_type_fixed_size_float32() {
    assert_eq!(FieldType::Float32.fixed_size(), Some(4));
}

#[test]
fn test_field_type_fixed_size_string_none() {
    assert_eq!(FieldType::String.fixed_size(), None);
}

#[test]
fn test_field_type_fixed_size_blob_none() {
    assert_eq!(FieldType::Blob.fixed_size(), None);
}
// Total: 50 tests
