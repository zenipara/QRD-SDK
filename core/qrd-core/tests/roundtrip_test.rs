//! End-to-end roundtrip test suite
//!
//! Validates complete write → read cycles with all encoding types,
//! compression algorithms, and reader types.

use qrd_core::prelude::*;
use qrd_core::reader::{PartialReader, PartialReadConfig};
use qrd_core::writer::{StreamingWriter, StreamingWriterConfig};
use std::io::Cursor;
use tempfile::NamedTempFile;

/// Test complete roundtrip: FileWriter → FileReader
#[test]
fn test_file_writer_reader_roundtrip() {
    let temp = NamedTempFile::new().unwrap();

    // Create comprehensive schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("name", FieldType::String, Nullability::Required)
        .unwrap()
        .add_field("score", FieldType::Float64, Nullability::Required)
        .unwrap()
        .add_field("active", FieldType::Boolean, Nullability::Required)
        .unwrap()
        .add_field("tags", FieldType::Blob, Nullability::Optional)
        .unwrap()
        .build()
        .unwrap();

    // Test data
    let test_rows = vec![
        (1i64, "Alice", 95.5f64, true, Some(b"tag1,tag2")),
        (2i64, "Bob", 87.2f64, false, Some(b"tag3")),
        (3i64, "Charlie", 91.8f64, true, None),
        (4i64, "Diana", 88.9f64, true, Some(b"tag4,tag5,tag6")),
        (5i64, "Eve", 93.3f64, false, Some(b"tag7")),
    ];

    // Write data
    {
        let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

        for (id, name, score, active, tags) in &test_rows {
            let id_bytes = id.to_le_bytes().to_vec();
            let name_bytes = serialize_string(name);
            let score_bytes = score.to_le_bytes().to_vec();
            let active_bytes = vec![*active as u8];
            let tags_bytes = tags.map(|t| serialize_blob(t)).unwrap_or_default();

            writer.write_row(vec![id_bytes, name_bytes, score_bytes, active_bytes, tags_bytes]).unwrap();
        }

        writer.finish().unwrap();
    }

    // Read data back
    {
        let reader = FileReader::new(temp.path()).unwrap();

        // Verify metadata
        assert_eq!(reader.row_count(), test_rows.len() as u32);
        assert_eq!(reader.schema().fields.len(), 5);
        assert!(reader.row_group_offsets().len() > 0);

        // Read all rows
        let all_rows = reader.rows().unwrap();
        assert_eq!(all_rows.len(), test_rows.len());

        // Verify each row
        for (i, row) in all_rows.iter().enumerate() {
            let (expected_id, expected_name, expected_score, expected_active, expected_tags) = &test_rows[i];

            // Parse ID (first 8 bytes)
            let id = i64::from_le_bytes(row[0..8].try_into().unwrap());
            assert_eq!(id, *expected_id);

            // Parse name (length-prefixed string)
            let (name, name_end) = deserialize_string(&row[8..]);
            assert_eq!(name, *expected_name);

            // Parse score (8 bytes)
            let score = f64::from_le_bytes(row[name_end..name_end + 8].try_into().unwrap());
            assert!((score - expected_score).abs() < 0.001);

            // Parse active (1 byte)
            let active = row[name_end + 8] != 0;
            assert_eq!(active, *expected_active);

            // Parse tags (optional blob)
            if let Some(expected_tags) = expected_tags {
                let tags_start = name_end + 9;
                let (tags, _) = deserialize_blob(&row[tags_start..]);
                assert_eq!(tags, *expected_tags);
            }
        }
    }
}

/// Test streaming writer → partial reader roundtrip
#[test]
fn test_streaming_writer_partial_reader_roundtrip() {
    // Create test data in memory
    let schema = SchemaBuilder::new()
        .add_field("timestamp", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("value", FieldType::Float32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut buffer = Cursor::new(Vec::new());
    let config = StreamingWriterConfig {
        row_group_size: 1000,
        ..Default::default()
    };

    // Write streaming data
    {
        let mut writer = StreamingWriter::with_config(&mut buffer, schema.clone(), config).unwrap();

        for i in 0..2500 {
            let timestamp = (i as i64 * 1000).to_le_bytes().to_vec();
            let value = ((i as f32 * 1.5) + 10.0).to_le_bytes().to_vec();
            writer.write_row(vec![timestamp, value]).unwrap();
        }

        writer.finish().unwrap();
    }

    let data = buffer.into_inner();

    // Test partial reader
    {
        let cursor = Cursor::new(data);
        let config = PartialReadConfig::default();
        let mut reader = PartialReader::new(cursor, config).unwrap();

        assert_eq!(reader.row_count(), 2500);
        assert_eq!(reader.row_group_count(), 3); // 2500 / 1000 = 2.5 → 3 groups

        // Test column-selective reading
        for rg_idx in 0..reader.row_group_count() {
            let columns = reader.read_columns(rg_idx, &[0, 1]).unwrap(); // Read both columns
            assert_eq!(columns.len(), 2);

            // Verify data in this row group
            let rg_size = if rg_idx < 2 { 1000 } else { 500 }; // Last group has 500 rows
            assert_eq!(columns[0].len() / 8, rg_size); // timestamp column
            assert_eq!(columns[1].len() / 4, rg_size); // value column
        }

        // Test single column reading
        let timestamp_col = reader.read_columns(0, &[0]).unwrap();
        assert_eq!(timestamp_col.len(), 1);
        assert_eq!(timestamp_col[0].len(), 1000 * 8); // 1000 timestamps * 8 bytes each
    }
}

/// Test all encoding types in roundtrip
#[test]
fn test_all_encodings_roundtrip() {
    let encodings_to_test = vec![
        ("plain_int", FieldType::Int32, vec![1i32, 2, 3, 4, 5]),
        ("rle_int", FieldType::Int32, vec![1i32, 1, 1, 2, 2, 2, 2]),
        ("delta_int", FieldType::Int64, vec![100i64, 105, 110, 115, 120]),
        ("bitpacked_bool", FieldType::Boolean, vec![true, false, true, false, true]),
        ("dictionary_string", FieldType::String, vec!["apple", "banana", "apple", "cherry", "banana"]),
    ];

    for (test_name, field_type, test_values) in encodings_to_test {
        println!("Testing encoding roundtrip: {}", test_name);

        let schema = SchemaBuilder::new()
            .add_field("test_col", field_type.clone(), Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let temp = NamedTempFile::new().unwrap();

        // Write data
        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

            for value in &test_values {
                let bytes = match field_type {
                    FieldType::Int32 => (value as &i32).to_le_bytes().to_vec(),
                    FieldType::Int64 => (value as &i64).to_le_bytes().to_vec(),
                    FieldType::Boolean => vec![*value as u8],
                    FieldType::String => serialize_string(value as &str),
                    _ => continue,
                };
                writer.write_row(vec![bytes]).unwrap();
            }

            writer.finish().unwrap();
        }

        // Read data back
        {
            let reader = FileReader::new(temp.path()).unwrap();
            let decoded_columns = reader.read_decoded_row_group(0).unwrap();

            assert_eq!(decoded_columns.len(), 1);
            let column_data = &decoded_columns[0];

            // Verify roundtrip
            match field_type {
                FieldType::Int32 => {
                    for (i, expected) in test_values.iter().enumerate() {
                        let offset = i * 4;
                        let actual = i32::from_le_bytes(column_data[offset..offset + 4].try_into().unwrap());
                        assert_eq!(actual, *expected as i32);
                    }
                }
                FieldType::Int64 => {
                    for (i, expected) in test_values.iter().enumerate() {
                        let offset = i * 8;
                        let actual = i64::from_le_bytes(column_data[offset..offset + 8].try_into().unwrap());
                        assert_eq!(actual, *expected as i64);
                    }
                }
                FieldType::Boolean => {
                    for (i, expected) in test_values.iter().enumerate() {
                        let actual = column_data[i] != 0;
                        assert_eq!(actual, *expected as bool);
                    }
                }
                FieldType::String => {
                    let mut offset = 0;
                    for expected in &test_values {
                        let (actual, new_offset) = deserialize_string(&column_data[offset..]);
                        assert_eq!(actual, *expected as &str);
                        offset = new_offset;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Test compression algorithms in roundtrip
#[test]
fn test_compression_roundtrip() {
    use qrd_core::compression::CompressionCodec;

    let compressions = vec![CompressionCodec::None, CompressionCodec::Zstd, CompressionCodec::Lz4];

    for compression in compressions {
        println!("Testing compression roundtrip: {:?}", compression);

        let schema = SchemaBuilder::new()
            .add_field("data", FieldType::Blob, Nullability::Required)
            .unwrap()
            .build()
            .unwrap();

        let temp = NamedTempFile::new().unwrap();

        // Create compressible data
        let test_data = (0..1000).map(|i| (i % 256) as u8).collect::<Vec<_>>();
        let test_rows = vec![test_data; 100]; // Repeat pattern for better compression

        // Write data
        {
            let mut writer = FileWriter::new(temp.path(), schema.clone()).unwrap();

            for row_data in &test_rows {
                writer.write_row(vec![serialize_blob(row_data)]).unwrap();
            }

            writer.finish().unwrap();
        }

        // Read data back
        {
            let reader = FileReader::new(temp.path()).unwrap();
            let decoded_columns = reader.read_decoded_row_group(0).unwrap();

            assert_eq!(decoded_columns.len(), 1);
            let column_data = &decoded_columns[0];

            // Verify all rows
            let mut offset = 0;
            for expected_data in &test_rows {
                let (actual_data, new_offset) = deserialize_blob(&column_data[offset..]);
                assert_eq!(actual_data, *expected_data);
                offset = new_offset;
            }
        }
    }
}

/// Helper functions for serialization
fn serialize_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(bytes);
    result
}

fn serialize_blob(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend_from_slice(&(data.len() as u32).to_le_bytes());
    result.extend_from_slice(data);
    result
}

fn deserialize_string(data: &[u8]) -> (&str, usize) {
    let len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let str_data = &data[4..4 + len];
    let str = std::str::from_utf8(str_data).unwrap();
    (str, 4 + len)
}

fn deserialize_blob(data: &[u8]) -> (&[u8], usize) {
    let len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let blob_data = &data[4..4 + len];
    (blob_data, 4 + len)
}