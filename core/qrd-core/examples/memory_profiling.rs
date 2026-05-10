//! Memory profiling example
//!
//! Demonstrates how to use the memory profiling utilities to track
//! memory usage during QRD operations.

use qrd_core::prelude::*;
use qrd_core::memory_profiling::{profile_writer_memory_usage, profile_reader_memory_usage, MemoryProfileScope};
use tempfile::NamedTempFile;

fn main() -> Result<()> {
    println!("QRD Memory Profiling Example");
    println!("============================");

    // Create a comprehensive schema
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?
        .add_field("name", FieldType::String, Nullability::Required)?
        .add_field("score", FieldType::Float64, Nullability::Required)?
        .add_field("active", FieldType::Boolean, Nullability::Required)?
        .add_field("tags", FieldType::Blob, Nullability::Optional)?
        .build()?;

    println!("Schema created with {} columns", schema.column_count());

    // Create temporary file
    let temp_file = NamedTempFile::new()?;

    // Profile memory usage during writing
    println!("\nProfiling memory usage during write operation...");
    let (_res, write_stats) = qrd_core::memory_profiling::MemoryProfiler::profile(|| {
        // Perform a write workload
        let mut writer = qrd_core::writer::FileWriter::new(temp_file.path(), schema.clone()).unwrap();
        for i in 0..1000 {
            let blob = vec![(i % 256) as u8; 128];
            let data = serialize_blob(&blob);
            writer.write_row(vec![data]).unwrap();
        }
        writer.finish().unwrap();
    });

    println!("Write operation memory statistics:");
    println!("  Peak memory usage: {}", format_memory_size(write_stats.peak_bytes));
    println!("  Total allocations: {}", write_stats.total_allocations);
    println!("  Total bytes allocated: {}", format_memory_size(write_stats.total_bytes_allocated));

    // Profile memory usage during reading
    println!("\nProfiling memory usage during read operation...");
    let (_rres, read_stats) = qrd_core::memory_profiling::MemoryProfiler::profile(|| {
        let reader = qrd_core::reader::FileReader::new(temp_file.path()).unwrap();
        // Trigger a read-all (may be noop if file empty)
        let _ = reader.schema();
    });

    println!("Read operation memory statistics:");
    println!("  Peak memory usage: {}", format_memory_size(read_stats.peak_bytes));
    println!("  Total allocations: {}", read_stats.total_allocations);
    println!("  Total bytes allocated: {}", format_memory_size(read_stats.total_bytes_allocated));

    // Demonstrate scoped profiling
    println!("\nDemonstrating scoped memory profiling...");
    {
        let _scope = MemoryProfileScope::new("scoped_operation");

        // Perform some operations within the scope
        let mut data = Vec::with_capacity(10000);
        for i in 0..10000 {
            data.push(i as u8);
        }

        // The scope will automatically report memory usage when dropped
    }

    println!("\nMemory profiling example completed successfully!");
    Ok(())
}

/// Format memory size in human-readable format
fn format_memory_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Serialize raw blob with length prefix (u32 little-endian)
fn serialize_blob(b: &Vec<u8>) -> Vec<u8> {
    let mut res = Vec::new();
    let len = b.len() as u32;
    res.extend_from_slice(&len.to_le_bytes());
    res.extend_from_slice(b);
    res
}