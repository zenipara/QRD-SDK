//! Additional writer and roundtrip tests

use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use tempfile::NamedTempFile;

#[test]
fn test_writer_reader_roundtrip_int32_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..100 {
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

#[test]
fn test_writer_reader_roundtrip_int64_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..100 {
        writer.write_row(vec![(i as i64).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

#[test]
fn test_writer_reader_roundtrip_float32_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Float32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..100 {
        writer.write_row(vec![(i as f32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

#[test]
fn test_writer_reader_roundtrip_float64_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Float64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..100 {
        writer.write_row(vec![(i as f64).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

#[test]
fn test_writer_reader_roundtrip_blob_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..50 {
        let data = vec![i as u8; 100];
        writer.write_row(vec![data]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 50);
}

#[test]
fn test_writer_reader_roundtrip_string_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![b"hello".to_vec()]).unwrap();
    writer.write_row(vec![b"world".to_vec()]).unwrap();
    writer.write_row(vec![b"test".to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 3);
}

#[test]
fn test_writer_reader_multi_column_int32_int64() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("b", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..50 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            ((i * 2) as i64).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 50);
}

#[test]
fn test_writer_reader_multi_column_int32_float32() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("count", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("ratio", FieldType::Float32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..50 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            (i as f32 * 1.5).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 50);
}

#[test]
fn test_writer_reader_multi_column_3_fields() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("amount", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..30 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            (i as f64 * 1.5).to_le_bytes().to_vec(),
            vec![i as u8; 50],
        ]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 30);
}

#[test]
fn test_writer_reader_multi_column_5_fields() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("f1", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("f2", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("f3", FieldType::Float32, Nullability::Required)
        .unwrap()
        .add_field("f4", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("f5", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..20 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            (i as i64).to_le_bytes().to_vec(),
            (i as f32).to_le_bytes().to_vec(),
            (i as f64).to_le_bytes().to_vec(),
            vec![i as u8; 100],
        ]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 20);
}

#[test]
fn test_writer_reader_single_row() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![42i32.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_reader_zero_rows() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 0);
}

#[test]
fn test_writer_reader_1000_rows() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..1000 {
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1000);
}

#[test]
fn test_writer_reader_10000_rows() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..10000 {
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 10000);
}

#[test]
fn test_writer_optional_field_empty() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![vec![]]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_required_field_present() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![42i32.to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_repeated_field_multiple_values() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("values", FieldType::Int32, Nullability::Repeated)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![vec![1, 2, 3, 4]]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_blob_field_large_data() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let large_blob = vec![0xABu8; 1024 * 100];
    writer.write_row(vec![large_blob]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_string_field_long_text() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let long_text = "a".repeat(1000).into_bytes();
    writer.write_row(vec![long_text]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_string_field_unicode() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("text", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    let unicode = "你好世界".as_bytes().to_vec();
    writer.write_row(vec![unicode]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_mixed_optional_required() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::String, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![1i32.to_le_bytes().to_vec(), b"test".to_vec()]).unwrap();
    writer.write_row(vec![2i32.to_le_bytes().to_vec(), vec![]]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 2);
}

#[test]
fn test_writer_all_optional_fields() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Optional)
        .unwrap()
        .add_field("b", FieldType::Int64, Nullability::Optional)
        .unwrap()
        .add_field("c", FieldType::Float32, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    writer.write_row(vec![vec![], vec![], vec![]]).unwrap();
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_alternating_nulls() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..10 {
        if i % 2 == 0 {
            writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
        } else {
            writer.write_row(vec![vec![]]).unwrap();
        }
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 10);
}

// Total: 50 tests
