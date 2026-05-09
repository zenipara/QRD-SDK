// Basic reader example

use qrd_core::prelude::*;
use std::path::Path;

fn main() -> Result<()> {
    println!("QRD Reader Example");
    println!("------------------");

    let test_file = "test_output.qrd";

    if !Path::new(test_file).exists() {
        println!("Test file '{}' not found. Run basic_writer first.", test_file);
        return Ok(());
    }

    let reader = FileReader::new(test_file)?;
    println!("Opened file: {}", test_file);
    println!("Schema: {} columns", reader.schema().fields.len());
    println!("Total rows: {}", reader.row_count());
    println!("Row groups: {}", reader.row_group_offsets().len());

    // Read first row group
    if reader.row_group_offsets().len() > 0 {
        println!("\nReading first row group...");
        let decoded_columns = reader.read_decoded_row_group(0)?;

        println!("Columns in row group: {}", decoded_columns.len());
        for (col_idx, column_data) in decoded_columns.iter().enumerate() {
            println!("Column {}: {} bytes", col_idx, column_data.len());
        }

        // Try to interpret first few values from first column
        if !decoded_columns.is_empty() {
            let first_col = &decoded_columns[0];
            println!("\nFirst column data (first 20 bytes): {:?}", &first_col[..first_col.len().min(20)]);
        }
    }

    Ok(())
}
