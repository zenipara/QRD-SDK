//! Fuzz testing for QRD format robustness
//!
//! Tests malformed inputs, corrupted data, and edge cases to ensure
//! the format handles errors gracefully without crashes or undefined behavior.

use qrd_core::error::Result;
use qrd_core::footer::{Footer, FooterParser};
use qrd_core::reader::{FileReader, PartialReader, PartialReadConfig};
use qrd_core::rowgroup::RowGroup;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::validation::CorruptionDetector;
use qrd_core::writer::FileWriter;
use std::io::Cursor;

/// Fuzz test malformed footer data
#[test]
fn fuzz_malformed_footer() {
    // Test various malformed footer scenarios
    let test_cases = vec![
        // Empty footer
        vec![],
        // Invalid length
        vec![0xFF; 10],
        // Corrupted magic
        {
            let mut data = vec![b'Q', b'R', b'D', 0x01];
            data.extend_from_slice(&[0xFF; 100]);
            data
        },
        // Wrong version
        {
            let mut data = vec![b'Q', b'R', b'D', 0x01, 0xFF, 0xFF];
            data.extend_from_slice(&[0x00; 100]);
            data
        },
        // Invalid schema
        {
            let mut data = vec![b'Q', b'R', b'D', 0x01, 0x01, 0x00];
            data.extend_from_slice(&u32::MAX.to_le_bytes()); // Invalid schema ID
            data.extend_from_slice(&[0x00; 100]);
            data
        },
    ];

    for (i, malformed_data) in test_cases.into_iter().enumerate() {
        println!("Testing malformed footer case {}", i);
        let result = Footer::deserialize(&malformed_data);
        // Should either succeed or return a proper error, not panic
        match result {
            Ok(_) => println!("  Case {}: Successfully parsed (unexpected but valid)", i),
            Err(e) => println!("  Case {}: Correctly rejected with error: {}", i, e),
        }
    }
}

/// Fuzz test corrupted row group data
#[test]
fn fuzz_corrupted_row_groups() {
    // Create a valid row group first
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let mut row_group = RowGroup::new(100);
    // Add some dummy column data
    let column_data = vec![42u8; 800]; // 100 rows * 8 bytes each

    // Simulate corrupted data patterns
    let corruption_patterns = vec![
        // Truncated data
        column_data[..50].to_vec(),
        // Extra data
        {
            let mut data = column_data.clone();
            data.extend_from_slice(&[0xFF; 100]);
            data
        },
        // Bit flips in header
        {
            let mut data = column_data.clone();
            if data.len() > 4 {
                data[0] ^= 0xFF; // Corrupt row count
            }
            data
        },
        // Invalid encoding type
        {
            let mut data = column_data.clone();
            if data.len() > 10 {
                data[8] = 0xFF; // Invalid encoding ID
            }
            data
        },
        // Negative sizes
        {
            let mut data = column_data.clone();
            if data.len() > 12 {
                data[12..16].copy_from_slice(&(-1i32).to_le_bytes()); // Negative compressed size
            }
            data
        },
    ];

    for (i, corrupted_data) in corruption_patterns.into_iter().enumerate() {
        println!("Testing corrupted row group case {}", i);
        let result = RowGroup::deserialize(&corrupted_data);
        match result {
            Ok(_) => println!("  Case {}: Successfully parsed (unexpected but valid)", i),
            Err(e) => println!("  Case {}: Correctly rejected with error: {}", i, e),
        }
    }
}

/// Fuzz test partial reader with corrupted files
#[test]
fn fuzz_partial_reader_corruption() {
    // Create a valid file first
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .add_field("data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    {
        let mut writer = FileWriter::new(temp_file.path(), schema.clone()).unwrap();
        for i in 0..100 {
            let id_bytes = (i as i64).to_le_bytes().to_vec();
            let data_bytes = format!("data_{}", i).into_bytes();
            writer.write_row(vec![id_bytes, data_bytes]).unwrap();
        }
        writer.finish().unwrap();
    }

    // Read the file and corrupt it in various ways
    let mut file_data = std::fs::read(temp_file.path()).unwrap();

    let corruption_scenarios = vec![
        // Corrupt footer length
        {
            let len = file_data.len();
            if len > 8 {
                file_data[len - 8..len - 4].copy_from_slice(&[0xFF; 4]);
            }
            file_data.clone()
        },
        // Corrupt CRC
        {
            let len = file_data.len();
            if len > 4 {
                file_data[len - 4..].copy_from_slice(&[0xFF; 4]);
            }
            file_data.clone()
        },
        // Truncate file
        file_data[..file_data.len() / 2].to_vec(),
        // Corrupt row group offset
        {
            let mut corrupted = file_data.clone();
            // Find footer and corrupt first row group offset
            if let Ok(footer) = FooterParser::parse_from_slice(&file_data) {
                if !footer.row_group_offsets.is_empty() {
                    let offset_pos = 32 + 8; // Approximate position in footer
                    if corrupted.len() > offset_pos + 8 {
                        corrupted[offset_pos..offset_pos + 8].copy_from_slice(&[0xFF; 8]);
                    }
                }
            }
            corrupted
        },
    ];

    for (i, corrupted_data) in corruption_scenarios.into_iter().enumerate() {
        println!("Testing partial reader corruption case {}", i);
        let cursor = Cursor::new(corrupted_data);
        let config = PartialReadConfig::default();

        let result = PartialReader::new(cursor, config);
        match result {
            Ok(mut reader) => {
                println!("  Case {}: Reader created successfully", i);
                // Try to read and see if it fails gracefully
                let read_result = reader.read_row_group(0);
                match read_result {
                    Ok(_) => println!("    Read succeeded (unexpected)"),
                    Err(e) => println!("    Read failed gracefully: {}", e),
                }
            }
            Err(e) => println!("  Case {}: Reader creation failed: {}", i, e),
        }
    }
}

/// Fuzz test validation functions with edge cases
#[test]
fn fuzz_validation_edge_cases() {
    let test_cases = vec![
        // Empty data
        vec![],
        // Very small data
        vec![0x00; 4],
        // Large offsets
        vec![0xFF; 1000],
        // Non-monotonic offsets
        {
            let mut data = vec![0, 10, 5, 20]; // 5 < 10, non-monotonic
            data.into_iter().flat_map(|x| (x as u64).to_le_bytes()).collect()
        },
        // Overlapping ranges
        {
            let mut data = vec![0, 5, 3, 10]; // 3-10 overlaps with 0-5
            data.into_iter().flat_map(|x| (x as u64).to_le_bytes()).collect()
        },
    ];

    for (i, test_data) in test_cases.into_iter().enumerate() {
        println!("Testing validation case {}", i);

        // Test monotonic validation
        let offsets: Vec<u64> = test_data.chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let monotonic_result = CorruptionDetector::validate_monotonic_offsets(&offsets);
        match monotonic_result {
            Ok(_) => println!("  Monotonic: Valid"),
            Err(e) => println!("  Monotonic: Invalid - {}", e),
        }

        // Test bounds validation (need file size)
        let file_size = 1000u64;
        let bounds_result = CorruptionDetector::validate_row_group_bounds(&offsets, file_size);
        match bounds_result {
            Ok(_) => println!("  Bounds: Valid"),
            Err(e) => println!("  Bounds: Invalid - {}", e),
        }
    }
}

/// Fuzz test encoding/decoding with malformed data
#[test]
fn fuzz_encoding_malformed_data() {
    use qrd_core::encoding::{EncodingType, get_encoder};

    let malformed_inputs = vec![
        // Empty data
        vec![],
        // Odd number of bytes for some encodings
        vec![0x00, 0x01, 0x02],
        // Very large data that might cause overflows
        vec![0xFF; 100000],
        // Data with null bytes
        vec![0x00; 1000],
        // Alternating pattern
        (0..1000).map(|i| if i % 2 == 0 { 0x00 } else { 0xFF }).collect(),
    ];

    let encoder = get_encoder(EncodingType::Plain).expect("Failed to get encoder");

    for (i, input) in malformed_inputs.into_iter().enumerate() {
        println!("Testing encoding case {}", i);

        let encode_result = encoder.encode(&input);
        match encode_result {
            Ok(encoded) => {
                println!("  Encoded successfully");
                // Try to decode
                let decode_result = encoder.decode(&encoded, input.len());
                match decode_result {
                    Ok(_decoded) => println!("  Decoded successfully"),
                    Err(e) => println!("  Decode failed: {}", e),
                }
            }
            Err(e) => println!("  Encode failed: {}", e),
        }
    }
}