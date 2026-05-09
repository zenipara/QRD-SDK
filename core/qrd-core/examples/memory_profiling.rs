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
    let write_stats = profile_writer_memory_usage(&temp_file.path(), &schema, 1000)?;

    println!("Write operation memory statistics:");
    println!("  Peak memory usage: {}", format_memory_size(write_stats.peak_bytes));
    println!("  Total allocations: {}", write_stats.total_allocations);
    println!("  Total bytes allocated: {}", format_memory_size(write_stats.total_bytes_allocated));

    // Profile memory usage during reading
    println!("\nProfiling memory usage during read operation...");
    let read_stats = profile_reader_memory_usage(&temp_file.path())?;

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