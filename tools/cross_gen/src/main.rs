use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::writer::FileWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = env::args().nth(1).unwrap_or_else(|| "/tmp/cross_rust.qrd".to_string());
    let path = Path::new(&out);

    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?
        .add_field("name", FieldType::String, Nullability::Required)?
        .build()?;

    let mut writer = FileWriter::new(path, schema)?;

    for i in 0..100u64 {
        let id = (i as i64).to_le_bytes().to_vec();
        let name = format!("user_{}", i);
        let mut name_ser = (name.len() as u32).to_le_bytes().to_vec();
        name_ser.extend_from_slice(name.as_bytes());
        writer.write_row(vec![id, name_ser])?;
    }

    writer.finish()?;

    println!("Wrote {} rows to {}", 100, out);
    Ok(())
}
