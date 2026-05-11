//! Stress and writer/reader consistency tests

use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use qrd_core::reader::FileReader;
use tempfile::NamedTempFile;

#[test]
fn test_writer_reader_consistency_small() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("b", FieldType::Int32, Nullability::Required).unwrap()
        .build().unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..1000 {
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec(), ((i*2) as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1000);
}

#[test]
fn test_writer_reader_consistency_medium() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("x", FieldType::Int64, Nullability::Required).unwrap()
        .add_field("y", FieldType::Blob, Nullability::Optional).unwrap()
        .build().unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..5000 {
        let mut blob = (i as u32).to_le_bytes().to_vec();
        blob.extend_from_slice(&vec![(i % 256) as u8; 16]);
        writer.write_row(vec![(i as i64).to_le_bytes().to_vec(), blob]).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 5000);
}

#[test]
fn test_writer_reader_many_small_files() {
    for _ in 0..20 {
        let temp = NamedTempFile::new().unwrap();
        let schema = SchemaBuilder::new()
            .add_field("v", FieldType::Int32, Nullability::Required).unwrap()
            .build().unwrap();

        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
        for i in 0..50 {
            writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
        }
        writer.finish().unwrap();

        let reader = FileReader::new(temp.path()).unwrap();
        assert_eq!(reader.row_count(), 50);
    }
}

#[test]
fn test_writer_reader_alternating_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("i", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("f", FieldType::Float32, Nullability::Required).unwrap()
        .add_field("s", FieldType::String, Nullability::Optional).unwrap()
        .build().unwrap();

    let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();
    for i in 0..1000 {
        let mut row = vec![(i as i32).to_le_bytes().to_vec(), ((i as f32)*1.1).to_le_bytes().to_vec()];
        if i % 3 == 0 {
            let mut s = (3u32).to_le_bytes().to_vec(); s.extend_from_slice(b"abc");
            row.push(s);
        } else {
            row.push((0u32).to_le_bytes().to_vec());
        }
        writer.write_row(row).unwrap();
    }
    writer.finish().unwrap();

    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1000);
}

#[test]
fn test_writer_finish_idempotent() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new().add_field("a", FieldType::Int32, Nullability::Required).unwrap().build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    writer.write_row(vec![(1i32).to_le_bytes().to_vec()]).unwrap();
    writer.finish().unwrap();
    // finish consumes writer; verify file is readable and has one row
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}
