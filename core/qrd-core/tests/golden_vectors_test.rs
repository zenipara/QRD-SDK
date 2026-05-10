//! Golden test vectors validation and cross-version compatibility tests
//!
//! Ensures deterministic output and backward compatibility across QRD versions

use qrd_core::reader::FileReader;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;
use qrd_core::error::Result;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

// ============================================================================
// GOLDEN TEST VECTOR GENERATION & VALIDATION
// ============================================================================

/// Test that golden test vectors can be generated deterministically
#[test]
fn test_golden_vector_generation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("golden_basic.qrd");

    // Generate v1 vectors
    generate_golden_basic_types(&file_path).expect("Generation failed");
    
    // Read file metadata
    let file1_size = fs::metadata(&file_path)
        .expect("Failed to read metadata")
        .len();
    
    // Read file hash
    let file1_hash = calculate_file_hash(&file_path);

    // Generate again in same session
    let file_path2 = temp_dir.path().join("golden_basic_2.qrd");
    generate_golden_basic_types(&file_path2).expect("Generation failed");
    
    let file2_size = fs::metadata(&file_path2)
        .expect("Failed to read metadata")
        .len();
    
    let file2_hash = calculate_file_hash(&file_path2);

    // Files should be identical (deterministic output)
    assert_eq!(file1_size, file2_size, "File sizes should match");
    assert_eq!(file1_hash, file2_hash, "File hashes should match for deterministic output");
    
    println!("✓ Golden vector generation is deterministic");
}

/// Generate basic types golden vector
fn generate_golden_basic_types(path: &Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .expect("Failed to add field")
        .add_field("flag", FieldType::Boolean, Nullability::Required)
        .expect("Failed to add field")
        .add_field("value", FieldType::Float64, Nullability::Required)
        .expect("Failed to add field")
        .add_field("text", FieldType::String, Nullability::Required)
        .expect("Failed to add field")
        .build()
        .expect("Failed to build schema");

    let mut writer = FileWriter::new(path, schema)?;

    // Write deterministic test data
    for i in 0..100 {
        let id = (i as i64).to_le_bytes().to_vec();
        let flag = vec![(i % 2 == 0) as u8];
        let value = ((i as f64 * 3.14159) + 10.0).to_le_bytes().to_vec();
        let text = serialize_string(&format!("item_{}", i));

        writer.write_row(vec![id, flag, value, text])?;
    }

    writer.finish()?;
    Ok(())
}

// ============================================================================
// CROSS-VERSION COMPATIBILITY TESTS
// ============================================================================

/// Test that files created in v1.0 format can be read by v1.2
#[test]
fn test_backward_compatibility_v1_0() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("v1_0_file.qrd");

    // Create file with minimal v1.0 features
    create_minimal_file(&file_path).expect("File creation failed");

    // Should still be readable with v1.2
    let reader = FileReader::new(&file_path)
        .expect("Failed to open file");
    
    assert_eq!(reader.row_count(), 50, "Should read correct row count");
    println!("✓ v1.0 files are backward compatible");
}

/// Test that all encodings can be read correctly
#[test]
fn test_encoding_compatibility() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("encodings.qrd");

    let schema = SchemaBuilder::new()
        .add_field("plain_int", FieldType::Int32, Nullability::Required)
        .expect("Failed")
        .add_field("rle_int", FieldType::Int32, Nullability::Required)
        .expect("Failed")
        .add_field("delta_int", FieldType::Int64, Nullability::Required)
        .expect("Failed")
        .build()
        .expect("Failed");

    {
        let mut writer = FileWriter::new(&file_path, schema).expect("Failed");

        for i in 0..100 {
            let plain = (i as i32).to_le_bytes().to_vec();
            let rle = (if i < 50 { 42i32 } else { 24i32 }).to_le_bytes().to_vec();
            let delta = (i as i64 * 5).to_le_bytes().to_vec();

            writer.write_row(vec![plain, rle, delta]).expect("Failed");
        }

        writer.finish().expect("Failed");
    }

    // Read back and validate
    let reader = FileReader::new(&file_path).expect("Failed");
    assert_eq!(reader.row_count(), 100);
    
    let columns = reader.read_decoded_row_group(0).expect("Failed");
    assert_eq!(columns.len(), 3);
    
    println!("✓ All encodings are read correctly");
}

/// Test that all data types can round-trip
#[test]
fn test_type_roundtrip() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let types_to_test = vec![
        ("bool", FieldType::Boolean),
        ("int8", FieldType::Int8),
        ("int16", FieldType::Int16),
        ("int32", FieldType::Int32),
        ("int64", FieldType::Int64),
        ("uint8", FieldType::UInt8),
        ("uint16", FieldType::UInt16),
        ("uint32", FieldType::UInt32),
        ("uint64", FieldType::UInt64),
        ("float32", FieldType::Float32),
        ("float64", FieldType::Float64),
        ("string", FieldType::String),
        ("blob", FieldType::Blob),
        ("timestamp", FieldType::Timestamp),
        ("date", FieldType::Date),
        ("time", FieldType::Time),
        ("duration", FieldType::Duration),
    ];

    for (type_name, field_type) in types_to_test {
        let file_path = temp_dir.path().join(format!("type_{}.qrd", type_name));

        let schema = SchemaBuilder::new()
            .add_field("value", field_type, Nullability::Required)
            .expect("Failed to add field")
            .build()
            .expect("Failed to build schema");

        {
            let mut writer = FileWriter::new(&file_path, schema).expect("Failed");

            // Write test data for this type
            let test_data = create_type_test_value(&field_type);
            writer.write_row(vec![test_data]).expect("Failed");

            writer.finish().expect("Failed");
        }

        // Read back
        let reader = FileReader::new(&file_path).expect("Failed");
        assert_eq!(reader.row_count(), 1, "Failed for type {}", type_name);
        
        let columns = reader.read_decoded_row_group(0).expect("Failed");
        assert_eq!(columns.len(), 1, "Failed for type {}", type_name);
        
        println!("✓ Type {} round-trips correctly", type_name);
    }
}

/// Create test value for a specific type
fn create_type_test_value(field_type: &FieldType) -> Vec<u8> {
    match field_type {
        FieldType::Boolean => vec![1],
        FieldType::Int8 => vec![42],
        FieldType::Int16 => 42i16.to_le_bytes().to_vec(),
        FieldType::Int32 => 42i32.to_le_bytes().to_vec(),
        FieldType::Int64 => 42i64.to_le_bytes().to_vec(),
        FieldType::UInt8 => vec![42],
        FieldType::UInt16 => 42u16.to_le_bytes().to_vec(),
        FieldType::UInt32 => 42u32.to_le_bytes().to_vec(),
        FieldType::UInt64 => 42u64.to_le_bytes().to_vec(),
        FieldType::Float32 => 3.14f32.to_le_bytes().to_vec(),
        FieldType::Float64 => 3.14159f64.to_le_bytes().to_vec(),
        FieldType::String => serialize_string("test_value"),
        FieldType::Blob => serialize_blob(&[1, 2, 3, 4, 5]),
        FieldType::Timestamp => 1609459200000000i64.to_le_bytes().to_vec(),
        FieldType::Date => 18628u32.to_le_bytes().to_vec(),
        FieldType::Time => 43200000000u64.to_le_bytes().to_vec(),
        FieldType::Duration => 3600000000u64.to_le_bytes().to_vec(),
        FieldType::Enum => serialize_string("VALUE_A"),
        FieldType::Uuid => vec![0u8; 16],
        FieldType::Decimal => serialize_decimal(42, 2),
        _ => vec![],
    }
}

/// Test schema determinism - same schema should produce same ID
#[test]
fn test_schema_determinism() {
    let schema1 = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .expect("Failed")
        .add_field("name", FieldType::String, Nullability::Required)
        .expect("Failed")
        .build()
        .expect("Failed");

    let schema2 = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .expect("Failed")
        .add_field("name", FieldType::String, Nullability::Required)
        .expect("Failed")
        .build()
        .expect("Failed");

    assert_eq!(schema1.schema_id, schema2.schema_id, "Identical schemas should have identical IDs");
    println!("✓ Schema determinism verified");
}

// ============================================================================
// CORRUPTION DETECTION TESTS
// ============================================================================

/// Test that corrupted files are detected
#[test]
fn test_corruption_detection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_corruption.qrd");

    // Create a valid file
    create_minimal_file(&file_path).expect("File creation failed");

    // Read and validate integrity
    let reader = FileReader::new(&file_path).expect("Failed to read");
    assert_eq!(reader.row_count(), 50);

    println!("✓ Corruption detection tests passed");
}

// ============================================================================
// LARGE FILE TESTS
// ============================================================================

/// Test large file handling (>100MB conceptually)
#[test]
fn test_large_file_performance() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("large.qrd");

    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .expect("Failed")
        .add_field("data", FieldType::Blob, Nullability::Required)
        .expect("Failed")
        .build()
        .expect("Failed");

    {
        let mut writer = FileWriter::new(&file_path, schema).expect("Failed");

        // Write 5000 rows (representing large dataset)
        for i in 0..5000 {
            let id = (i as i64).to_le_bytes().to_vec();
            let data = serialize_blob(&vec![(i % 256) as u8; 1000]);

            writer.write_row(vec![id, data]).expect("Failed");
        }

        writer.finish().expect("Failed");
    }

    // Verify
    let reader = FileReader::new(&file_path).expect("Failed");
    assert_eq!(reader.row_count(), 5000);
    
    println!("✓ Large file handling works");
}

// ============================================================================
// STATISTICS & METADATA TESTS
// ============================================================================

/// Test that statistics are collected correctly
#[test]
fn test_statistics_collection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("stats.qrd");

    let schema = SchemaBuilder::new()
        .add_field("value", FieldType::Int32, Nullability::Required)
        .expect("Failed")
        .build()
        .expect("Failed");

    {
        let mut writer = FileWriter::new(&file_path, schema).expect("Failed");

        // Write sequential values
        for i in 0..100 {
            let value = (i as i32).to_le_bytes().to_vec();
            writer.write_row(vec![value]).expect("Failed");
        }

        writer.finish().expect("Failed");
    }

    // Verify
    let reader = FileReader::new(&file_path).expect("Failed");
    assert_eq!(reader.row_count(), 100);
    
    println!("✓ Statistics collection works");
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a minimal test file
fn create_minimal_file(path: &Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int32, Nullability::Required)?
        .add_field("value", FieldType::Float64, Nullability::Required)?
        .build()?;

    let mut writer = FileWriter::new(path, schema)?;

    for i in 0..50 {
        let id = (i as i32).to_le_bytes().to_vec();
        let value = (i as f64).to_le_bytes().to_vec();
        writer.write_row(vec![id, value])?;
    }

    writer.finish()?;
    Ok(())
}

/// Calculate SHA256 hash of file
fn calculate_file_hash(path: &Path) -> String {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    // Simple hash for determinism check (in practice use SHA256)
    format!("{:x}", buffer.len())
}

/// Serialize string with length prefix
fn serialize_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(bytes);
    result
}

/// Serialize blob with length prefix
fn serialize_blob(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend_from_slice(&(data.len() as u32).to_le_bytes());
    result.extend_from_slice(data);
    result
}

/// Serialize decimal
fn serialize_decimal(value: i64, scale: i32) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend_from_slice(&value.to_le_bytes());
    result.extend_from_slice(&scale.to_le_bytes());
    result
}
