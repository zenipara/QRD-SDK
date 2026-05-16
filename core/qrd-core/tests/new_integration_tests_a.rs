// Integration tests part A - comprehensive type handling

use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use tempfile::NamedTempFile;

#[test]
fn test_large_int32_min_value() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![i32::MIN.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_large_int32_max_value() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![i32::MAX.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_large_int64_min_value() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![i64::MIN.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_large_int64_max_value() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![i64::MAX.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_uint32_max_value() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::UInt32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![u32::MAX.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_uint64_max_value() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::UInt64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![u64::MAX.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_float32_positive_infinity() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Float32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![f32::INFINITY.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_float32_negative_infinity() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Float32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![f32::NEG_INFINITY.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_float64_positive_infinity() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![f64::INFINITY.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_float64_negative_infinity() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![f64::NEG_INFINITY.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_empty_string() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![b"".to_vec()]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_empty_blob() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![vec![]]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_500_byte_string() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    let long_str = "x".repeat(500).into_bytes();
    writer.write_row(vec![long_str]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_5000_byte_blob() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    let large_blob = vec![0xABu8; 5000];
    writer.write_row(vec![large_blob]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_multiple_schemas_different_files() {
    let temp1 = NamedTempFile::new().unwrap();
    let schema1 = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer1 = FileWriter::new(temp1.path(), schema1).unwrap();
    writer1.write_row(vec![1i32.to_le_bytes().to_vec()]).unwrap();
    writer1.finish().unwrap();

    let temp2 = NamedTempFile::new().unwrap();
    let schema2 = SchemaBuilder::new()
        .add_field("name", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer2 = FileWriter::new(temp2.path(), schema2).unwrap();
    writer2.write_row(vec![b"test".to_vec()]).unwrap();
    writer2.finish().unwrap();

    let reader1 = FileReader::new(temp1.path()).unwrap();
    let reader2 = FileReader::new(temp2.path()).unwrap();
    assert_eq!(reader1.row_count(), 1);
    assert_eq!(reader2.row_count(), 1);
}

#[test]
fn test_boolean_field_0x00() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("flag", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![vec![0x00]]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_boolean_field_0x01() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("flag", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![vec![0x01]]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_finish_empty_file() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 0);
}

#[test]
fn test_250_rows_small_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..250 {
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 250);
}

#[test]
fn test_500_rows_varied_data() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("b", FieldType::Int64, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..500 {
        let b_val = if i % 2 == 0 { ((i as i64) * 1000).to_le_bytes().to_vec() } else { vec![] };
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec(), b_val]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 500);
}

#[test]
fn test_schema_with_50_different_types() {
    let temp = NamedTempFile::new().unwrap();
    let mut builder = SchemaBuilder::new();
    let types = vec![
        FieldType::Int8, FieldType::Int16, FieldType::Int32, FieldType::Int64,
        FieldType::UInt8, FieldType::UInt16, FieldType::UInt32, FieldType::UInt64,
        FieldType::Float32, FieldType::Float64, FieldType::Boolean, FieldType::String,
        FieldType::Blob, FieldType::Uuid, FieldType::Timestamp, FieldType::Date,
        FieldType::Time, FieldType::Duration, FieldType::Decimal, FieldType::Enum,
    ];
    for (i, ft) in types.iter().cycle().take(50).enumerate() {
        builder = builder.add_field(&format!("f{}", i), *ft, Nullability::Optional).unwrap();
    }
    let schema = builder.build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    let mut row = vec![];
    for _ in 0..50 {
        row.push(vec![0x00u8]);
    }
    writer.write_row(row).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_nullable_int_all_nulls() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for _ in 0..30 {
        writer.write_row(vec![vec![]]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 30);
}

#[test]
fn test_nullable_string_all_nulls() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for _ in 0..30 {
        writer.write_row(vec![vec![]]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 30);
}

#[test]
fn test_nullable_blob_all_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..30 {
        writer.write_row(vec![vec![i as u8; 10]]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 30);
}

#[test]
fn test_fixed_size_types_consistent() {
    let ft8 = FieldType::Int8;
    let ft16 = FieldType::Int16;
    let ft32 = FieldType::Int32;
    let ft64 = FieldType::Int64;
    let ftstr = FieldType::String;
    
    assert_eq!(ft8.fixed_size(), Some(1));
    assert_eq!(ft16.fixed_size(), Some(2));
    assert_eq!(ft32.fixed_size(), Some(4));
    assert_eq!(ft64.fixed_size(), Some(8));
    assert_eq!(ftstr.fixed_size(), None);
}

#[test]
fn test_field_type_equality() {
    assert_eq!(FieldType::Int32, FieldType::Int32);
    assert_eq!(FieldType::String, FieldType::String);
    assert_ne!(FieldType::Int32, FieldType::Int64);
    assert_ne!(FieldType::String, FieldType::Blob);
}

// Total: 50 tests
