// Basic reader example

use qrd_core::prelude::*;

fn main() -> Result<()> {
    // For this example, we'd open a QRD file created by the writer
    println!("QRD Reader Example");
    println!("------------------");

    // In a real scenario:
    // let reader = FileReader::new("data.qrd")?;
    // println!("Schema: {:?}", reader.schema());
    // println!("Total rows: {}", reader.row_count());
    //
    // for (row_idx, row) in reader.rows()?.iter().enumerate() {
    //     println!("Row {}: {:?}", row_idx, row);
    //     if row_idx >= 10 {
    //         break;
    //     }
    // }

    println!("Reader implementation in progress");

    Ok(())
}
