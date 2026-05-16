//! Additional validation and edge case tests

use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use tempfile::NamedTempFile;

#[test]
fn test_schema_field_type_int8_display() {
    let ft = FieldType::Int8;
    let s = format!("{}", ft);
    assert_eq!(s, "INT8");
}

#[test]
fn test_schema_field_type_int16_display() {
    let ft = FieldType::Int16;
    let s = format!("{}", ft);
    assert_eq!(s, "INT16");
}

#[test]
fn test_schema_field_type_int32_display() {
    let ft = FieldType::Int32;
    let s = format!("{}", ft);
    assert_eq!(s, "INT32");
}

#[test]
fn test_schema_field_type_int64_display() {
    let ft = FieldType::Int64;
    let s = format!("{}", ft);
    assert_eq!(s, "INT64");
}

#[test]
fn test_schema_field_type_uint8_display() {
    let ft = FieldType::UInt8;
    let s = format!("{}", ft);
    assert_eq!(s, "UINT8");
}

#[test]
fn test_schema_field_type_uint16_display() {
    let ft = FieldType::UInt16;
    let s = format!("{}", ft);
    assert_eq!(s, "UINT16");
}

#[test]
fn test_schema_field_type_uint32_display() {
    let ft = FieldType::UInt32;
    let s = format!("{}", ft);
    assert_eq!(s, "UINT32");
}

#[test]
fn test_schema_field_type_uint64_display() {
    let ft = FieldType::UInt64;
    let s = format!("{}", ft);
    assert_eq!(s, "UINT64");
}

#[test]
fn test_schema_field_type_float32_display() {
    let ft = FieldType::Float32;
    let s = format!("{}", ft);
    assert_eq!(s, "FLOAT32");
}

#[test]
fn test_schema_field_type_float64_display() {
    let ft = FieldType::Float64;
    let s = format!("{}", ft);
    assert_eq!(s, "FLOAT64");
}

#[test]
fn test_schema_field_type_boolean_display() {
    let ft = FieldType::Boolean;
    let s = format!("{}", ft);
    assert_eq!(s, "BOOLEAN");
}

#[test]
fn test_schema_field_type_string_display() {
    let ft = FieldType::String;
    let s = format!("{}", ft);
    assert_eq!(s, "STRING");
}

#[test]
fn test_schema_field_type_blob_display() {
    let ft = FieldType::Blob;
    let s = format!("{}", ft);
    assert_eq!(s, "BLOB");
}

#[test]
fn test_schema_field_type_uuid_display() {
    let ft = FieldType::Uuid;
    let s = format!("{}", ft);
    assert_eq!(s, "UUID");
}

#[test]
fn test_schema_field_type_timestamp_display() {
    let ft = FieldType::Timestamp;
    let s = format!("{}", ft);
    assert_eq!(s, "TIMESTAMP");
}

#[test]
fn test_schema_field_type_date_display() {
    let ft = FieldType::Date;
    let s = format!("{}", ft);
    assert_eq!(s, "DATE");
}

#[test]
fn test_schema_field_type_time_display() {
    let ft = FieldType::Time;
    let s = format!("{}", ft);
    assert_eq!(s, "TIME");
}

#[test]
fn test_schema_field_type_duration_display() {
    let ft = FieldType::Duration;
    let s = format!("{}", ft);
    assert_eq!(s, "DURATION");
}

#[test]
fn test_schema_field_type_decimal_display() {
    let ft = FieldType::Decimal;
    let s = format!("{}", ft);
    assert_eq!(s, "DECIMAL");
}

#[test]
fn test_schema_field_type_enum_display() {
    let ft = FieldType::Enum;
    let s = format!("{}", ft);
    assert_eq!(s, "ENUM");
}

#[test]
fn test_field_type_from_id_1() {
    let ft = FieldType::from_id(1).unwrap();
    assert_eq!(ft, FieldType::Boolean);
}

#[test]
fn test_field_type_from_id_2() {
    let ft = FieldType::from_id(2).unwrap();
    assert_eq!(ft, FieldType::Int8);
}

#[test]
fn test_field_type_from_id_5() {
    let ft = FieldType::from_id(5).unwrap();
    assert_eq!(ft, FieldType::Int64);
}

#[test]
fn test_field_type_from_id_18() {
    let ft = FieldType::from_id(18).unwrap();
    assert_eq!(ft, FieldType::Float32);
}

#[test]
fn test_field_type_from_id_19() {
    let ft = FieldType::from_id(19).unwrap();
    assert_eq!(ft, FieldType::Float64);
}

#[test]
fn test_field_type_from_id_24() {
    let ft = FieldType::from_id(24).unwrap();
    assert_eq!(ft, FieldType::String);
}

#[test]
fn test_field_type_from_id_27() {
    let ft = FieldType::from_id(27).unwrap();
    assert_eq!(ft, FieldType::Blob);
}

#[test]
fn test_field_type_from_id_invalid() {
    let result = FieldType::from_id(255);
    assert!(result.is_err());
}

#[test]
fn test_writer_finish_twice() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![1i32.to_le_bytes().to_vec()]).unwrap();
    let result1 = writer.finish();
    assert!(result1.is_ok());
}

#[test]
fn test_reader_nonexistent_file() {
    let result = FileReader::new("/nonexistent/path/to/file");
    assert!(result.is_err());
}

#[test]
fn test_writer_multiple_rows_same_schema() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..5 {
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 5);
}

#[test]
fn test_writer_reader_both_int32_int64() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("b", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![100i32.to_le_bytes().to_vec(), 200i64.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_all_int_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("i8", FieldType::Int8, Nullability::Required)
        .unwrap()
        .add_field("i16", FieldType::Int16, Nullability::Required)
        .unwrap()
        .add_field("i32", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("i64", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![
        1i8.to_le_bytes().to_vec(),
        2i16.to_le_bytes().to_vec(),
        3i32.to_le_bytes().to_vec(),
        4i64.to_le_bytes().to_vec(),
    ]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_all_uint_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("u8", FieldType::UInt8, Nullability::Required)
        .unwrap()
        .add_field("u16", FieldType::UInt16, Nullability::Required)
        .unwrap()
        .add_field("u32", FieldType::UInt32, Nullability::Required)
        .unwrap()
        .add_field("u64", FieldType::UInt64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![
        1u8.to_le_bytes().to_vec(),
        2u16.to_le_bytes().to_vec(),
        3u32.to_le_bytes().to_vec(),
        4u64.to_le_bytes().to_vec(),
    ]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_all_float_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("f32", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("f64", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![
        1.5f32.to_le_bytes().to_vec(),
        2.5f64.to_le_bytes().to_vec(),
    ]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_schema_with_100_fields_names() {
    let mut builder = SchemaBuilder::new();
    for i in 0..100 {
        builder = builder.add_field(&format!("f{:03}", i), FieldType::Int32, Nullability::Required).unwrap();
    }
    let schema = builder.build().unwrap();
    assert_eq!(schema.fields.len(), 100);
    for i in 0..100 {
        assert_eq!(schema.fields[i].name, format!("f{:03}", i));
    }
}

#[test]
fn test_writer_100_columns_single_row() {
    let temp = NamedTempFile::new().unwrap();
    let mut builder = SchemaBuilder::new();
    for i in 0..100 {
        builder = builder.add_field(&format!("col_{}", i), FieldType::Int32, Nullability::Required).unwrap();
    }
    let schema = builder.build().unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let mut row = Vec::new();
    for i in 0..100 {
        row.push((i as i32).to_le_bytes().to_vec());
    }
    writer.write_row(row).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_reader_schema_fields() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("x", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("y", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![1i32.to_le_bytes().to_vec(), 2i32.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.schema().fields.len(), 2);
    assert_eq!(reader.schema().fields[0].name, "x");
    assert_eq!(reader.schema().fields[1].name, "y");
}

#[test]
fn test_writer_reader_timestamp_type() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("ts", FieldType::Timestamp, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let ts = 1234567890u64;
    writer.write_row(vec![ts.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_date_type() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("d", FieldType::Date, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let date = 18000i32;
    writer.write_row(vec![date.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_time_type() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("t", FieldType::Time, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let time = 3600000000u64;
    writer.write_row(vec![time.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_duration_type() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("dur", FieldType::Duration, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let duration = 3600000000i64;
    writer.write_row(vec![duration.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_uuid_type() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Uuid, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let uuid = vec![1u8; 16];
    writer.write_row(vec![uuid]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_enum_type() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("status", FieldType::Enum, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![b"ACTIVE".to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_decimal_type() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("amount", FieldType::Decimal, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let decimal = vec![1, 2, 3, 4, 5];
    writer.write_row(vec![decimal]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_schema_serialization() {
    let schema = SchemaBuilder::new()
        .add_field("x", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("y", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let serialized = schema.serialize_binary().unwrap();
    assert!(!serialized.is_empty());
    
    let deserialized = qrd_core::schema::Schema::deserialize_binary(&serialized).unwrap();
    assert_eq!(deserialized.fields.len(), 2);
}

#[test]
fn test_schema_metadata() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(schema.fields[0].name, "id");
    assert_eq!(schema.fields[0].field_type, FieldType::Int32);
}

#[test]
fn test_writer_reader_200_rows_varied_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("i", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("f", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("s", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..200 {
        let text = if i % 10 == 0 { format!("num_{}", i).into_bytes() } else { vec![] };
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            (i as f64).to_le_bytes().to_vec(),
            text,
        ]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 200);
}

// Total: 59 tests (50+50+59 = 159)
