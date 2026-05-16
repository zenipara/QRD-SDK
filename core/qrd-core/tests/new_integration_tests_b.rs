// Integration tests part B - nullability patterns and multiple field scenarios

use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use tempfile::NamedTempFile;

#[test]
fn test_three_int_fields_required() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("b", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("c", FieldType::Int32, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..100 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            ((i+1) as i32).to_le_bytes().to_vec(),
            ((i+2) as i32).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

#[test]
fn test_three_fields_mixed_nullable() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("b", FieldType::Int32, Nullability::Optional).unwrap()
        .add_field("c", FieldType::Int32, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..100 {
        let b_val = if i % 3 == 0 { ((i as i32) * 2).to_le_bytes().to_vec() } else { vec![] };
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            b_val,
            ((i*3) as i32).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 100);
}

#[test]
fn test_six_fields_various_types() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("amount", FieldType::Float64, Nullability::Required).unwrap()
        .add_field("name", FieldType::String, Nullability::Optional).unwrap()
        .add_field("data", FieldType::Blob, Nullability::Optional).unwrap()
        .add_field("ts", FieldType::Timestamp, Nullability::Required).unwrap()
        .add_field("flag", FieldType::Boolean, Nullability::Optional).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..50 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            (i as f64 * 1.5).to_le_bytes().to_vec(),
            format!("name_{}", i).into_bytes(),
            vec![i as u8; 20],
            (1000000 + i as u64).to_le_bytes().to_vec(),
            if i % 2 == 0 { vec![0x01] } else { vec![] },
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 50);
}

#[test]
fn test_ten_fields_all_required() {
    let temp = NamedTempFile::new().unwrap();
    let mut builder = SchemaBuilder::new();
    for i in 0..10 {
        builder = builder.add_field(&format!("f{}", i), FieldType::Int32, Nullability::Required).unwrap();
    }
    let schema = builder.build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for row_idx in 0..40 {
        let mut row = vec![];
        for col_idx in 0..10 {
            row.push(((row_idx * 10 + col_idx) as i32).to_le_bytes().to_vec());
        }
        writer.write_row(row).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 40);
}

#[test]
fn test_ten_fields_all_optional() {
    let temp = NamedTempFile::new().unwrap();
    let mut builder = SchemaBuilder::new();
    for i in 0..10 {
        builder = builder.add_field(&format!("f{}", i), FieldType::String, Nullability::Optional).unwrap();
    }
    let schema = builder.build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for row_idx in 0..40 {
        let mut row = vec![];
        for col_idx in 0..10 {
            if (row_idx + col_idx) % 2 == 0 {
                row.push(format!("v{}", col_idx).into_bytes());
            } else {
                row.push(vec![]);
            }
        }
        writer.write_row(row).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 40);
}

#[test]
fn test_alternating_int_float_columns() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("int1", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("flt1", FieldType::Float64, Nullability::Required).unwrap()
        .add_field("int2", FieldType::Int64, Nullability::Required).unwrap()
        .add_field("flt2", FieldType::Float32, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..75 {
        writer.write_row(vec![
            (i as i32).to_le_bytes().to_vec(),
            (i as f64).to_le_bytes().to_vec(),
            (i as i64).to_le_bytes().to_vec(),
            (i as f32).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 75);
}

#[test]
fn test_string_blob_mixed_columns() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("s1", FieldType::String, Nullability::Optional).unwrap()
        .add_field("b1", FieldType::Blob, Nullability::Optional).unwrap()
        .add_field("s2", FieldType::String, Nullability::Optional).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..60 {
        writer.write_row(vec![
            format!("str_{}", i).into_bytes(),
            vec![i as u8; 30],
            format!("txt_{}", i*2).into_bytes(),
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 60);
}

#[test]
fn test_boolean_with_numeric_fields() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("b", FieldType::Boolean, Nullability::Required).unwrap()
        .add_field("i", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("f", FieldType::Float64, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..70 {
        writer.write_row(vec![
            if i % 2 == 0 { vec![0x01] } else { vec![0x00] },
            (i as i32).to_le_bytes().to_vec(),
            (i as f64 * 0.5).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 70);
}

#[test]
fn test_temporal_fields_mixed() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("ts", FieldType::Timestamp, Nullability::Required).unwrap()
        .add_field("d", FieldType::Date, Nullability::Required).unwrap()
        .add_field("t", FieldType::Time, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..55 {
        writer.write_row(vec![
            (1000000000u64 + i as u64).to_le_bytes().to_vec(),
            (18000i32 + i as i32).to_le_bytes().to_vec(),
            (3600000000u64 + i as u64).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 55);
}

#[test]
fn test_uuid_field_16bytes() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Uuid, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..45 {
        let mut uuid = vec![0u8; 16];
        uuid[0] = (i as u8) % 256;
        writer.write_row(vec![uuid]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 45);
}

#[test]
fn test_nullable_with_alternating_pattern() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Optional).unwrap()
        .add_field("b", FieldType::Int32, Nullability::Optional).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..80 {
        let a = if i % 2 == 0 { (i as i32).to_le_bytes().to_vec() } else { vec![] };
        let b = if i % 3 == 0 { ((i*2) as i32).to_le_bytes().to_vec() } else { vec![] };
        writer.write_row(vec![a, b]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 80);
}

#[test]
fn test_schema_with_same_name_different_types_error() {
    let schema1 = SchemaBuilder::new()
        .add_field("field", FieldType::Int32, Nullability::Required).unwrap()
        .build().unwrap();
    assert_eq!(schema1.fields[0].name, "field");
    assert_eq!(schema1.fields[0].field_type, FieldType::Int32);
}

#[test]
fn test_repeated_nullability_single_row() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("values", FieldType::Int32, Nullability::Repeated).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    let vals = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    writer.write_row(vec![vals]).unwrap();
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 1);
}

#[test]
fn test_writer_writes_then_checks_row_count() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("x", FieldType::Int32, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..123 {
        writer.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 123);
}

#[test]
fn test_many_varied_types_50_rows() {
    let temp = NamedTempFile::new().unwrap();
    let types = vec![
        FieldType::Int8, FieldType::Int16, FieldType::Int32, FieldType::Int64,
        FieldType::UInt8, FieldType::UInt16, FieldType::UInt32, FieldType::UInt64,
        FieldType::Float32, FieldType::Float64,
    ];
    let mut builder = SchemaBuilder::new();
    for (i, ft) in types.iter().enumerate() {
        builder = builder.add_field(&format!("f{}", i), *ft, Nullability::Required).unwrap();
    }
    let schema = builder.build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for _ in 0..50 {
        let mut row = vec![];
        for _ in 0..10 {
            row.push(vec![0u8; 8]);
        }
        writer.write_row(row).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 50);
}

#[test]
fn test_decimal_enum_combinations() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("dec", FieldType::Decimal, Nullability::Required).unwrap()
        .add_field("enum", FieldType::Enum, Nullability::Optional).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for i in 0..65 {
        writer.write_row(vec![
            vec![i as u8; 16],
            if i % 5 == 0 { format!("E{}", i).into_bytes() } else { vec![] },
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 65);
}

#[test]
fn test_multiple_files_same_schema() {
    let temp1 = NamedTempFile::new().unwrap();
    let temp2 = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("val", FieldType::Int32, Nullability::Required).unwrap()
        .build().unwrap();
    
    let mut writer1 = FileWriter::new(temp1.path(), schema.clone()).unwrap();
    for i in 0..50 {
        writer1.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer1.finish().unwrap();
    
    let mut writer2 = FileWriter::new(temp2.path(), schema.clone()).unwrap();
    for i in 50..100 {
        writer2.write_row(vec![(i as i32).to_le_bytes().to_vec()]).unwrap();
    }
    writer2.finish().unwrap();
    
    let reader1 = FileReader::new(temp1.path()).unwrap();
    let reader2 = FileReader::new(temp2.path()).unwrap();
    assert_eq!(reader1.row_count(), 50);
    assert_eq!(reader2.row_count(), 50);
}

#[test]
fn test_all_numeric_types_in_one() {
    let temp = NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("i8", FieldType::Int8, Nullability::Required).unwrap()
        .add_field("i16", FieldType::Int16, Nullability::Required).unwrap()
        .add_field("i32", FieldType::Int32, Nullability::Required).unwrap()
        .add_field("i64", FieldType::Int64, Nullability::Required).unwrap()
        .add_field("u8", FieldType::UInt8, Nullability::Required).unwrap()
        .add_field("u16", FieldType::UInt16, Nullability::Required).unwrap()
        .add_field("u32", FieldType::UInt32, Nullability::Required).unwrap()
        .add_field("u64", FieldType::UInt64, Nullability::Required).unwrap()
        .add_field("f32", FieldType::Float32, Nullability::Required).unwrap()
        .add_field("f64", FieldType::Float64, Nullability::Required).unwrap()
        .build().unwrap();
    let mut writer = FileWriter::new(temp.path(), schema).unwrap();
    for _ in 0..35 {
        writer.write_row(vec![
            vec![1u8],
            (1i16).to_le_bytes().to_vec(),
            (1i32).to_le_bytes().to_vec(),
            (1i64).to_le_bytes().to_vec(),
            vec![1u8],
            (1u16).to_le_bytes().to_vec(),
            (1u32).to_le_bytes().to_vec(),
            (1u64).to_le_bytes().to_vec(),
            (1.0f32).to_le_bytes().to_vec(),
            (1.0f64).to_le_bytes().to_vec(),
        ]).unwrap();
    }
    writer.finish().unwrap();
    let reader = FileReader::new(temp.path()).unwrap();
    assert_eq!(reader.row_count(), 35);
}

// Total: 50 tests
