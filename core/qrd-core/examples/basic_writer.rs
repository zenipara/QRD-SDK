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
    let temp_file = NamedTempFile::new()?;
    let path = temp_file.path().to_path_buf();

    // Create writer
    let mut writer = FileWriter::new(&path, schema)?;

    // Simulate writing rows
    println!("Writing rows...");
    for i in 0..100 {
        // In a real implementation, we'd write actual row data
        // For now, this is a placeholder
        writer.write_row(vec![FieldType::Int64; 3])?;

        if (i + 1) % 25 == 0 {
            println!("  Wrote {} rows", i + 1);
        }
    }

    let row_count = writer.row_count();

    // Finish writing
    writer.finish()?;
    println!("Successfully wrote {} rows", row_count);

    Ok(())
}
