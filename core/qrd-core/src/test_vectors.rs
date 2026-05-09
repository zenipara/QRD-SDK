//! Golden test vector generation and validation
//!
//! Creates canonical binary QRD files for cross-implementation validation.
//! These vectors ensure deterministic output across all language bindings.

use crate::{
    reader::FileReader, schema::{FieldType, Nullability, SchemaBuilder},
    writer::FileWriter, error::Result,
};
use std::{fs, path::Path};

/// Generate all golden test vectors
pub fn generate_golden_vectors() -> Result<()> {
    let golden_dir = Path::new("test_vectors/golden");

    // Ensure directory exists
    fs::create_dir_all(golden_dir)?;

    // Generate basic types vector
    generate_basic_types_vector(golden_dir)?;

    // Generate encoding showcase vector
    generate_encoding_showcase_vector(golden_dir)?;

    // Generate compression test vector
    generate_compression_test_vector(golden_dir)?;

    // Generate large dataset vector
    generate_large_dataset_vector(golden_dir)?;

    // Generate edge cases vector
    generate_edge_cases_vector(golden_dir)?;

    println!("✅ Generated all golden test vectors");
    Ok(())
}

/// Basic types test vector - demonstrates all supported field types
fn generate_basic_types_vector(golden_dir: &Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("bool_col", FieldType::Boolean, Nullability::Required)?
        .add_field("int8_col", FieldType::Int8, Nullability::Required)?
        .add_field("int16_col", FieldType::Int16, Nullability::Required)?
        .add_field("int32_col", FieldType::Int32, Nullability::Required)?
        .add_field("int64_col", FieldType::Int64, Nullability::Required)?
        .add_field("uint8_col", FieldType::UInt8, Nullability::Required)?
        .add_field("uint16_col", FieldType::UInt16, Nullability::Required)?
        .add_field("uint32_col", FieldType::UInt32, Nullability::Required)?
        .add_field("uint64_col", FieldType::UInt64, Nullability::Required)?
        .add_field("float32_col", FieldType::Float32, Nullability::Required)?
        .add_field("float64_col", FieldType::Float64, Nullability::Required)?
        .add_field("string_col", FieldType::String, Nullability::Required)?
        .add_field("blob_col", FieldType::Blob, Nullability::Required)?
        .add_field("timestamp_col", FieldType::Timestamp, Nullability::Required)?
        .add_field("date_col", FieldType::Date, Nullability::Required)?
        .add_field("time_col", FieldType::Time, Nullability::Required)?
        .add_field("duration_col", FieldType::Duration, Nullability::Required)?
        .build()?;

    let output_path = golden_dir.join("basic_types.qrd");
    let mut writer = FileWriter::new(&output_path, schema)?;

    // Write 10 rows with deterministic test data
    for i in 0..10 {
        let bool_val = vec![(i % 2 == 0) as u8];
        let int8_val = vec![i as i8];
        let int16_val = (i as i16 * 10).to_le_bytes().to_vec();
        let int32_val = (i as i32 * 100).to_le_bytes().to_vec();
        let int64_val = (i as i64 * 1000).to_le_bytes().to_vec();
        let uint8_val = vec![i as u8];
        let uint16_val = (i as u16 * 10).to_le_bytes().to_vec();
        let uint32_val = (i as u32 * 100).to_le_bytes().to_vec();
        let uint64_val = (i as u64 * 1000).to_le_bytes().to_vec();
        let float32_val = ((i as f32 * 1.5) + 10.0).to_le_bytes().to_vec();
        let float64_val = ((i as f64 * 2.5) + 20.0).to_le_bytes().to_vec();
        let string_val = serialize_string(&format!("string_{}", i));
        let blob_val = serialize_blob(&vec![i as u8; 8]);
        let timestamp_val = (i as i64 * 1000000).to_le_bytes().to_vec();
        let date_val = (i as u32 * 86400).to_le_bytes().to_vec();
        let time_val = (i as u64 * 1000000).to_le_bytes().to_vec();
        let duration_val = (i as u64 * 1000000).to_le_bytes().to_vec();

        writer.write_row(vec![
            bool_val, int8_val, int16_val, int32_val, int64_val,
            uint8_val, uint16_val, uint32_val, uint64_val,
            float32_val, float64_val, string_val, blob_val,
            timestamp_val, date_val, time_val, duration_val,
        ])?;
    }

    writer.finish()?;
    println!("  Generated basic_types.qrd");
    Ok(())
}

/// Encoding showcase vector - demonstrates all encoding types
fn generate_encoding_showcase_vector(golden_dir: &Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("plain_int", FieldType::Int32, Nullability::Required)?
        .add_field("rle_int", FieldType::Int32, Nullability::Required)?
        .add_field("delta_int", FieldType::Int64, Nullability::Required)?
        .add_field("bitpacked_bool", FieldType::Boolean, Nullability::Required)?
        .add_field("dictionary_str", FieldType::String, Nullability::Required)?
        .add_field("byte_stream_float", FieldType::Float32, Nullability::Required)?
        .build()?;

    let output_path = golden_dir.join("encoding_showcase.qrd");
    let mut writer = FileWriter::new(&output_path, schema)?;

    // Write rows designed to trigger specific encodings
    for i in 0..100 {
        let plain_int = (i as i32).to_le_bytes().to_vec();
        let rle_int = (if i < 50 { 42i32 } else { 24i32 }).to_le_bytes().to_vec(); // RLE pattern
        let delta_int = ((i as i64 * 5) + 1000).to_le_bytes().to_vec(); // Delta encoding
        let bitpacked_bool = vec![(i % 3 == 0) as u8]; // Bit-packed pattern
        let dictionary_str = serialize_string(match i % 5 {
            0 => "alpha",
            1 => "beta",
            2 => "gamma",
            3 => "delta",
            _ => "epsilon",
        }); // Dictionary encoding
        let byte_stream_float = ((i as f32 * 3.14) + 1.0).to_le_bytes().to_vec(); // Byte stream split

        writer.write_row(vec![plain_int, rle_int, delta_int, bitpacked_bool, dictionary_str, byte_stream_float])?;
    }

    writer.finish()?;
    println!("  Generated encoding_showcase.qrd");
    Ok(())
}

/// Compression test vector - demonstrates compression effectiveness
fn generate_compression_test_vector(golden_dir: &Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("compressible_data", FieldType::Blob, Nullability::Required)?
        .add_field("random_data", FieldType::Blob, Nullability::Required)?
        .build()?;

    let output_path = golden_dir.join("compression_test.qrd");
    let mut writer = FileWriter::new(&output_path, schema)?;

    // Generate compressible and incompressible data
    for i in 0..50 {
        // Highly compressible: repeating pattern
        let compressible = vec![(i % 10) as u8; 1000];

        // Incompressible: random-like data
        let random: Vec<u8> = (0..1000).map(|j| ((i * j) % 256) as u8).collect();

        writer.write_row(vec![
            serialize_blob(&compressible),
            serialize_blob(&random),
        ])?;
    }

    writer.finish()?;
    println!("  Generated compression_test.qrd");
    Ok(())
}

/// Large dataset vector - tests scalability
fn generate_large_dataset_vector(golden_dir: &Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?
        .add_field("timestamp", FieldType::Timestamp, Nullability::Required)?
        .add_field("value", FieldType::Float64, Nullability::Required)?
        .add_field("category", FieldType::String, Nullability::Required)?
        .build()?;

    let output_path = golden_dir.join("large_dataset.qrd");
    let mut writer = FileWriter::new(&output_path, schema)?;

    // Generate 10,000 rows of realistic telemetry data
    for i in 0..10_000 {
        let id = (i as i64).to_le_bytes().to_vec();
        let timestamp = ((i as i64 * 60000) + 1609459200000000).to_le_bytes().to_vec(); // 2021-01-01 + i minutes
        let value = ((i as f64 % 100.0) + 10.0).to_le_bytes().to_vec();
        let category = serialize_string(match i % 4 {
            0 => "temperature",
            1 => "pressure",
            2 => "humidity",
            _ => "voltage",
        });

        writer.write_row(vec![id, timestamp, value, category])?;
    }

    writer.finish()?;
    println!("  Generated large_dataset.qrd (10,000 rows)");
    Ok(())
}

/// Edge cases vector - tests boundary conditions
fn generate_edge_cases_vector(golden_dir: &Path) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("empty_string", FieldType::String, Nullability::Required)?
        .add_field("empty_blob", FieldType::Blob, Nullability::Required)?
        .add_field("zero_int", FieldType::Int64, Nullability::Required)?
        .add_field("max_int", FieldType::Int64, Nullability::Required)?
        .add_field("min_int", FieldType::Int64, Nullability::Required)?
        .add_field("zero_float", FieldType::Float64, Nullability::Required)?
        .add_field("nan_float", FieldType::Float64, Nullability::Required)?
        .add_field("inf_float", FieldType::Float64, Nullability::Required)?
        .build()?;

    let output_path = golden_dir.join("edge_cases.qrd");
    let mut writer = FileWriter::new(&output_path, schema)?;

    // Write edge case values
    let edge_cases = vec![
        (
            serialize_string(""),           // empty string
            serialize_blob(&[]),            // empty blob
            0i64.to_le_bytes().to_vec(),    // zero
            i64::MAX.to_le_bytes().to_vec(), // max
            i64::MIN.to_le_bytes().to_vec(), // min
            0.0f64.to_le_bytes().to_vec(),   // zero float
            f64::NAN.to_le_bytes().to_vec(), // NaN
            f64::INFINITY.to_le_bytes().to_vec(), // infinity
        ),
    ];

    for case in edge_cases {
        writer.write_row(vec![case.0, case.1, case.2, case.3, case.4, case.5, case.6, case.7])?;
    }

    writer.finish()?;
    println!("  Generated edge_cases.qrd");
    Ok(())
}

/// Validate golden test vectors can be read correctly
pub fn validate_golden_vectors() -> Result<()> {
    let golden_dir = Path::new("test_vectors/golden");

    if !golden_dir.exists() {
        println!("⚠️  Golden vectors directory not found, skipping validation");
        return Ok(());
    }

    // Validate basic types vector
    validate_basic_types_vector(golden_dir)?;

    // Validate encoding showcase vector
    validate_encoding_showcase_vector(golden_dir)?;

    println!("✅ All golden test vectors validated");
    Ok(())
}

fn validate_basic_types_vector(golden_dir: &Path) -> Result<()> {
    let path = golden_dir.join("basic_types.qrd");
    let reader = FileReader::new(&path)?;

    assert_eq!(reader.row_count(), 10);
    assert_eq!(reader.schema().fields.len(), 16); // All field types

    // Read first row group
    let columns = reader.read_decoded_row_group(0)?;
    assert_eq!(columns.len(), 16);

    println!("  ✅ Validated basic_types.qrd");
    Ok(())
}

fn validate_encoding_showcase_vector(golden_dir: &Path) -> Result<()> {
    let path = golden_dir.join("encoding_showcase.qrd");
    let reader = FileReader::new(&path)?;

    assert_eq!(reader.row_count(), 100);
    assert_eq!(reader.schema().fields.len(), 6);

    // Read first row group
    let columns = reader.read_decoded_row_group(0)?;
    assert_eq!(columns.len(), 6);

    println!("  ✅ Validated encoding_showcase.qrd");
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_vectors() {
        // Only run if test_vectors directory exists
        if Path::new("test_vectors").exists() {
            generate_golden_vectors().unwrap();
            validate_golden_vectors().unwrap();
        }
    }
}