//! Golden test vector generation

use qrd_core::prelude::*;
use std::fs;

/// Generate golden test vectors for reproducibility testing
pub fn generate_golden_vectors() -> Result<()> {
    // Create test_vectors directory if it doesn't exist
    fs::create_dir_all("test_vectors")?;

    generate_basic_types_vector()?;
    generate_sorted_integers_vector()?;
    generate_strings_vector()?;
    Ok(())
}

/// Basic types test vector
fn generate_basic_types_vector() -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("bool_col", FieldType::Boolean, Nullability::Required)?
        .add_field("int32_col", FieldType::Int32, Nullability::Required)?
        .add_field("float64_col", FieldType::Float64, Nullability::Required)?
        .build()?;

    let mut writer = FileWriter::new(Path::new("test_vectors/basic_types.qrd"), schema)?;

    for i in 0..100 {
        let bool_val = if i % 2 == 0 { 1u8 } else { 0u8 };
        let int32_val = (i as i32 * 10).to_le_bytes().to_vec();
        let float64_val = (i as f64 * 3.14).to_le_bytes().to_vec();

        writer.write_row(vec![vec![bool_val], int32_val, float64_val])?;
    }

    writer.finish()?;
    Ok(())
}

/// Sorted integers for DELTA_BINARY testing
fn generate_sorted_integers_vector() -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("sorted_int64", FieldType::Int64, Nullability::Required)?
        .build()?;

    let mut writer = FileWriter::new(Path::new("test_vectors/sorted_int64.qrd"), schema)?;

    for i in 0..1000 {
        let value = (i * i).to_le_bytes().to_vec(); // Quadratic growth, sorted
        writer.write_row(vec![value])?;
    }

    writer.finish()?;
    Ok(())
}

/// String data for variable-length testing
fn generate_strings_vector() -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("name", FieldType::String, Nullability::Required)?
        .add_field("description", FieldType::String, Nullability::Optional)?
        .build()?;

    let mut writer = FileWriter::new(Path::new("test_vectors/strings.qrd"), schema)?;

    let names = vec![
        "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry",
        "Ivy", "Jack", "Kate", "Liam", "Mia", "Noah", "Olivia", "Peter",
        "Quinn", "Ryan", "Sophia", "Tyler", "Uma", "Victor", "Wendy", "Xavier",
        "Yara", "Zach"
    ];

    for (i, name) in names.iter().enumerate() {
        let name_bytes = serialize_string(name);
        let desc = format!("Description for {}", name);
        let desc_bytes = serialize_string(&desc);

        writer.write_row(vec![name_bytes, desc_bytes])?;
    }

    writer.finish()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_vectors() {
        // Only run if test_vectors directory exists
        if Path::new("test_vectors").exists() {
            generate_golden_vectors().unwrap();
        }
    }
}