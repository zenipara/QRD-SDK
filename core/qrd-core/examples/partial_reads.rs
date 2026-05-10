//! Partial reads and query pushdown example
//!
//! Demonstrates column-selective reads, query pushdown optimization,
//! and metadata indexing for efficient data access.

use qrd_core::prelude::*;
use qrd_core::metadata::{ColumnFilter, ColumnFilterSpec};
use qrd_core::reader::PartialReader;
use tempfile::NamedTempFile;

fn main() -> Result<()> {
    println!("QRD Partial Reads Example");
    println!("=========================");
    println!();

    // Create a comprehensive dataset
    let temp_file = create_sample_dataset()?;
    println!("✓ Created sample dataset");

    // Demonstrate column-selective reads
    demonstrate_column_selective_reads(&temp_file)?;

    // Demonstrate query pushdown optimization
    demonstrate_query_pushdown(&temp_file)?;

    // Demonstrate metadata indexing
    demonstrate_metadata_indexing(&temp_file)?;

    println!();
    println!("🎉 Partial reads example completed successfully!");
    println!();
    println!("Key benefits demonstrated:");
    println!("  • Column-selective reads: Only load needed columns");
    println!("  • Query pushdown: Skip irrelevant row groups");
    println!("  • Metadata indexing: Fast column lookup and statistics");
    println!("  • Efficient I/O: Reduced memory usage and faster queries");

    Ok(())
}

fn create_sample_dataset() -> Result<NamedTempFile> {
    let temp = NamedTempFile::new()?;

    // Create schema for a user analytics dataset
    let schema = SchemaBuilder::new()
        .add_field("user_id", FieldType::Int64, Nullability::Required)?
        .add_field("username", FieldType::String, Nullability::Required)?
        .add_field("email_domain", FieldType::String, Nullability::Required)?
        .add_field("signup_date", FieldType::Int64, Nullability::Required)? // Unix timestamp
        .add_field("last_login", FieldType::Int64, Nullability::Optional)?
        .add_field("login_count", FieldType::Int32, Nullability::Required)?
        .add_field("is_active", FieldType::Boolean, Nullability::Required)?
        .add_field("account_type", FieldType::String, Nullability::Required)? // "free", "premium", "enterprise"
        .build()?;

    println!("  Schema: {} columns", schema.fields.len());

    // Create writer with small row groups for demonstration
    let mut config = qrd_core::writer::WriterConfig::default();
    config.row_group_size = 100; // Small groups to show pushdown
    
    let mut writer = qrd_core::writer::FileWriter::with_config(
        std::fs::File::create(&temp)?,
        schema,
        config
    )?;

    // Generate sample data
    let domains = ["gmail.com", "yahoo.com", "outlook.com", "company.com"];
    let account_types = ["free", "premium", "enterprise"];

    for i in 0..1000 {
        let user_id = (1000 + i) as i64;
        let username = format!("user_{}", i);
        let email_domain = domains[i % domains.len()];
        let signup_date = 1609459200 + (i as i64 * 86400); // Jan 2021 + i days
        let last_login = if i % 3 != 0 { Some(1640995200 + (i as i64 * 3600)) } else { None }; // Some users never logged in
        let login_count = (i % 100) as i32;
        let is_active = i % 5 != 0; // 80% active users
        let account_type = account_types[i % account_types.len()];

        // Serialize data
        let user_id_bytes = user_id.to_le_bytes().to_vec();
        let username_bytes = serialize_string(&username);
        let email_domain_bytes = serialize_string(email_domain);
        let signup_date_bytes = signup_date.to_le_bytes().to_vec();
        let last_login_bytes = last_login.map(|ts| ts.to_le_bytes().to_vec()).unwrap_or_default();
        let login_count_bytes = login_count.to_le_bytes().to_vec();
        let is_active_bytes = vec![is_active as u8];
        let account_type_bytes = serialize_string(account_type);

        writer.write_row(vec![
            user_id_bytes,
            username_bytes,
            email_domain_bytes,
            signup_date_bytes,
            last_login_bytes,
            login_count_bytes,
            is_active_bytes,
            account_type_bytes,
        ])?;
    }

    let rg_count = writer.row_group_count();
    writer.finish()?;
    println!("  Generated {} user records in {} row groups", 1000, rg_count);

    Ok(temp)
}

fn demonstrate_column_selective_reads(temp_file: &NamedTempFile) -> Result<()> {
    println!("\n📊 Column-Selective Reads");
    println!("-------------------------");

    let mut reader = PartialReader::new(std::fs::File::open(temp_file.path())?, Default::default())?;

    // Read only user_id and login_count columns (indices 0 and 5)
    let column_indices = vec![0, 5];
    let start_time = std::time::Instant::now();

    let result = reader.read_columns_with_filters(&column_indices, &[])?;
    let elapsed = start_time.elapsed();

    println!("  Read {} columns from {} rows in {:?}", column_indices.len(), result.len() / column_indices.len(), elapsed);
    println!("  Memory efficient: Only loaded required columns");

    // Show sample data
    println!("  Sample results:");
    for i in (0..result.len()).step_by(column_indices.len()).take(3) {
        if let (Some(user_id_bytes), Some(login_count_bytes)) = (result.get(i), result.get(i + 1)) {
            let user_id = i64::from_le_bytes(user_id_bytes[..8].try_into().unwrap());
            let login_count = i32::from_le_bytes(login_count_bytes[..4].try_into().unwrap());
            println!("    User {}: {} logins", user_id, login_count);
        }
    }

    Ok(())
}

fn demonstrate_query_pushdown(temp_file: &NamedTempFile) -> Result<()> {
    println!("\n🚀 Query Pushdown Optimization");
    println!("------------------------------");

    let mut reader = PartialReader::new(std::fs::File::open(temp_file.path())?, Default::default())?;

    // Query: Find active premium users with high login counts
    let filters = vec![
        // is_active = true (column 6)
        ColumnFilterSpec {
            column_index: 6,
            filter: ColumnFilter::Equal(vec![1]), // true = 1
        },
        // account_type = "premium" (column 7)
        ColumnFilterSpec {
            column_index: 7,
            filter: ColumnFilter::Equal(serialize_string("premium")),
        },
        // login_count > 50 (column 5)
        ColumnFilterSpec {
            column_index: 5,
            filter: ColumnFilter::GreaterThan(50i32.to_le_bytes().to_vec()),
        },
    ];

    // Estimate result count before executing query
    let estimated_count = reader.estimate_query_result_count(&filters);
    println!("  Estimated results: {} rows", estimated_count);

    // Execute query with pushdown
    let start_time = std::time::Instant::now();
    let result = reader.read_columns_with_filters(&[0, 1, 5, 7], &filters)?; // user_id, username, login_count, account_type
    let elapsed = start_time.elapsed();

    let actual_count = result.len() / 4; // 4 columns per row
    println!("  Actual results: {} rows in {:?}", actual_count, elapsed);
    println!("  Query pushdown: Skipped irrelevant row groups based on statistics");

    // Show sample results
    println!("  Sample premium users with high activity:");
    for i in (0..result.len()).step_by(4).take(3) {
        if let (Some(user_id_bytes), Some(username_bytes), Some(login_count_bytes)) =
            (result.get(i), result.get(i + 1), result.get(i + 2)) {

            let user_id = i64::from_le_bytes(user_id_bytes[..8].try_into().unwrap());
            let username = deserialize_string(username_bytes);
            let login_count = i32::from_le_bytes(login_count_bytes[..4].try_into().unwrap());

            println!("    {} ({}): {} logins", username, user_id, login_count);
        }
    }

    Ok(())
}

fn demonstrate_metadata_indexing(temp_file: &NamedTempFile) -> Result<()> {
    println!("\n📋 Metadata Indexing");
    println!("--------------------");

    let mut reader = PartialReader::new(std::fs::File::open(temp_file.path())?, Default::default())?;
    let metadata_index = reader.metadata_index().unwrap();

    // Show column index mapping
    println!("  Column index mapping:");
    for (name, idx) in [
        ("user_id", "user_id"),
        ("username", "username"),
        ("login_count", "login_count"),
        ("is_active", "is_active"),
    ] {
        if let Some(col_idx) = metadata_index.get_column_index(name) {
            println!("    {} → column {}", name, col_idx);
        }
    }

    // Show row group statistics
    println!("  Row group statistics:");
    println!("    Total row groups: {}", metadata_index.row_group_offsets.len());
    for (i, rg_stats) in metadata_index.row_group_stats.iter().enumerate() {
        println!("    Row group {}: {} rows", i, rg_stats.row_count);
    }

    // Show column statistics for login_count
    let login_stats = metadata_index.get_column_stats(5); // login_count column
    println!("  Login count statistics across row groups:");
    for (i, col_stats) in login_stats.iter().enumerate() {
        println!("    Row group {}: min={}, max={}, nulls={}",
            i,
            col_stats.min_value.as_ref().map(|v| i32::from_le_bytes(v[..4].try_into().unwrap())).unwrap_or(0),
            col_stats.max_value.as_ref().map(|v| i32::from_le_bytes(v[..4].try_into().unwrap())).unwrap_or(0),
            col_stats.null_count
        );
    }

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

/// Deserialize a string with length prefix
fn deserialize_string(data: &[u8]) -> String {
    if data.len() < 4 {
        return String::new();
    }
    let len = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
    if data.len() < 4 + len {
        return String::new();
    }
    String::from_utf8_lossy(&data[4..4 + len]).to_string()
}