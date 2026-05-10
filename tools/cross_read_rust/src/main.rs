use std::env;
use std::path::Path;

use qrd_core::reader::FileReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).unwrap_or_else(|| "/tmp/cross_go.qrd".to_string());
    let p = Path::new(&path);
    let reader = FileReader::new(p)?;
    println!("rust row_count={}", reader.row_count());
    Ok(())
}
