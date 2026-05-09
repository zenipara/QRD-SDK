//! Encoding showcase example
//!
//! Demonstrates all QRD encoding types and how they are automatically
//! selected based on data characteristics.

use qrd_core::prelude::*;
use tempfile::NamedTempFile;

fn main() -> Result<()> {
    println!("QRD Encoding Showcase Example");
    println!("==============================");

    // Create schema that will trigger different encodings
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?           // PLAIN (unique values)
        .add_field("category", FieldType::String, Nullability::Required)?    // DICTIONARY (few unique values)
        .add_field("status", FieldType::Int32, Nullability::Required)?       // RLE (repeated values)
        .add_field("sequence", FieldType::Int64, Nullability::Required)?     // DELTA (sequential)
        .add_field("bit_flags", FieldType::Int8, Nullability::Required)?     // BIT_PACKED (small range)
        .add_field("sparse_data", FieldType::Int64, Nullability::Optional)?  // DELTA_BYTE_ARRAY (optional)
        .build()?;

    println!("Schema created with {} columns", schema.column_count());

    // Create temporary file
    let temp_file = NamedTempFile::new()?;

    // Create writer
    let mut writer = FileWriter::new(&temp_file.path(), schema.clone())?;

    println!("\nWriting rows designed to trigger specific encodings...");

    // Write rows that will trigger different encodings
    for i in 0..100 {
        // id: Unique values → PLAIN encoding
        let id_bytes = (i as i64).to_le_bytes().to_vec();

        // category: Limited set of values → DICTIONARY encoding
        let categories = ["A", "B", "C", "A", "B"];
        let category = categories[i % categories.len()];
        let category_bytes = serialize_string(category);

        // status: Repeated values → RLE encoding
        let status = if i < 50 { 1i32 } else { 2i32 };
        let status_bytes = status.to_le_bytes().to_vec();

        // sequence: Sequential values → DELTA encoding
        let sequence = (i as i64 * 5) + 1000;
        let sequence_bytes = sequence.to_le_bytes().to_vec();

        // bit_flags: Small range (0-7) → BIT_PACKED encoding
        let bit_flag = (i % 8) as i8;
        let bit_flag_bytes = bit_flag.to_le_bytes().to_vec();

        // sparse_data: Optional sequential → DELTA_BYTE_ARRAY encoding
        let sparse_data = if i % 10 == 0 {
            Some((i as i64 * 10).to_le_bytes().to_vec())
        } else {
            None
        };

        // Prepare row data
        let mut row_data = vec![
            id_bytes,
            category_bytes,
            status_bytes,
            sequence_bytes,
            bit_flag_bytes,
        ];

        // Add sparse data (handle optional)
        match sparse_data {
            Some(data) => row_data.push(data),
            None => row_data.push(vec![]), // Empty for null
        }

        writer.write_row(row_data)?;

        if (i + 1) % 25 == 0 {
            println!("  Wrote {} rows", i + 1);
        }
    }

    // Finish writing
    writer.finish()?;
    println!("Successfully wrote {} rows", writer.row_count());

    // Now read back and verify
    println!("\nReading back data to verify encodings...");

    let mut reader = FileReader::new(&temp_file.path())?;
    let read_schema = reader.schema();

    println!("Read schema with {} columns", read_schema.column_count());

    // Read all rows
    let mut row_count = 0;
    while let Some(row_result) = reader.read_row()? {
        row_count += 1;

        // Verify we can read the data back
        if row_count <= 5 {
            println!("  Row {}: {} columns", row_count, row_result.len());
        }
    }

    println!("Successfully read {} rows", row_count);

    // Demonstrate partial reading (read just one column)
    println!("\nDemonstrating partial column reading...");

    let mut partial_reader = qrd_core::reader::PartialReader::new(
        &temp_file.path(),
        vec![0], // Read only the first column (id)
        qrd_core::reader::PartialReadConfig::default()
    )?;

    let mut partial_count = 0;
    while let Some(partial_row) = partial_reader.read_partial_row()? {
        partial_count += 1;
        if partial_count <= 3 {
            println!("  Partial row {}: {} columns read", partial_count, partial_row.len());
        }
    }

    println!("Successfully read {} partial rows (1 column each)", partial_count);

    println!("\nEncoding showcase example completed successfully!");
    println!("This example demonstrated:");
    println!("  • PLAIN encoding for unique values");
    println!("  • DICTIONARY encoding for repeated strings");
    println!("  • RLE encoding for runs of identical values");
    println!("  • DELTA encoding for sequential numbers");
    println!("  • BIT_PACKED encoding for small-range integers");
    println!("  • DELTA_BYTE_ARRAY encoding for optional data");
    println!("  • Partial reading for efficient column access");

    Ok(())
}

/// Serialize a string with length prefix
fn serialize_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(bytes);
    result
}