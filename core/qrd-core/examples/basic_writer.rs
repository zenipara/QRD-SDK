// Basic writer example

use qrd_core::prelude::*;
use tempfile::NamedTempFile;

fn main() -> Result<()> {
    // Create schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?
        .add_field("name", FieldType::String, Nullability::Optional)?
        .add_field("score", FieldType::Float64, Nullability::Optional)?
        .build()?;

    println!("Schema created with {} columns", schema.column_count());

    // Create temporary file
    let temp_file = "test_output.qrd";

    // Create writer
    let mut writer = FileWriter::new(temp_file, schema)?;

    // Write some sample rows
    println!("Writing rows...");
    for i in 0..10 {
        // Serialize data for each column
        let id_bytes = (i as i64).to_le_bytes().to_vec();
        let name_str = format!("user_{}", i);
        let name_bytes = serialize_string(&name_str);
        let score_bytes = (i as f64 * 1.5).to_le_bytes().to_vec();

        writer.write_row(vec![id_bytes, name_bytes, score_bytes])?;

        if (i + 1) % 5 == 0 {
            println!("  Wrote {} rows", i + 1);
        }
    }

    let row_count = writer.row_count();

    // Finish writing
    writer.finish()?;
    println!("Successfully wrote {} rows", row_count);

    Ok(())
}

/// Serialize a string with length prefix
fn serialize_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(bytes);
    result
}
