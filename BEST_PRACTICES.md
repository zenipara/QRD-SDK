# Deprecated Best Practices

This file has been superseded by `CONTRIBUTING.md` and the new documentation system under `docs/`.

---

## Table of Contents

1. [Best Practices](#best-practices)
2. [Common Patterns](#common-patterns)
3. [Performance Optimization](#performance-optimization)
4. [Troubleshooting](#troubleshooting)
5. [Security Checklist](#security-checklist)
6. [Migration Recipes](#migration-recipes)

---

## Best Practices

### Writing Files

#### 1. Always Call finish()
```rust
// ❌ WRONG - file is incomplete
{
    let mut writer = FileWriter::new("data.qrd", schema)?;
    writer.write_row(vec![...])?;
}  // File is invalid!

// ✅ CORRECT
let mut writer = FileWriter::new("data.qrd", schema)?;
writer.write_row(vec![...])?;
writer.finish()?;  // Don't forget!
```

#### 2. Choose Appropriate Row Group Size
```rust
// ❌ TOO SMALL - too many row groups
const ROW_GROUP_SIZE: usize = 100;  // Bad for compression

// ✅ GOOD - balanced
const ROW_GROUP_SIZE: usize = 10_000;  // Standard
const ROW_GROUP_SIZE: usize = 50_000;  // Large files
```

#### 3. Handle Errors Properly
```rust
// ❌ INCOMPLETE - may leave file in inconsistent state
if let Err(e) = writer.write_row(row) {
    eprintln!("Error: {}", e);
}
writer.finish()?;  // Still runs despite error

// ✅ CORRECT - abort on error
match writer.write_row(row) {
    Ok(_) => {},
    Err(e) => {
        eprintln!("Fatal error: {}", e);
        return Err(e);  // Abort immediately
    }
}
writer.finish()?;
```

#### 4. Batch Small Writes
```rust
// ❌ SLOW - individual writes
for i in 0..1_000_000 {
    writer.write_row(vec![serialize_int(i)])?;
}

// ✅ FASTER - batch writes
let mut batch = Vec::new();
for i in 0..1_000_000 {
    batch.push(vec![serialize_int(i)]);
    if batch.len() == 10_000 {
        writer.write_rows(&batch)?;
        batch.clear();
    }
}
if !batch.is_empty() {
    writer.write_rows(&batch)?;
}
```

### Reading Files

#### 1. Check Schema First
```rust
// ✅ GOOD - validate schema
let reader = FileReader::new("data.qrd")?;
let schema = reader.schema();

for field in &schema.fields {
    println!("Field: {} (type: {:?})", field.name, field.field_type);
}

// Then read with confidence
let data = reader.read_all()?;
```

#### 2. Use Partial Reads for Large Files
```rust
// ❌ BAD - loads entire file
let reader = FileReader::new("huge_file.qrd")?;
let all_data = reader.read_all()?;  // Out of memory!

// ✅ GOOD - read row groups one at a time
let reader = FileReader::new("huge_file.qrd")?;
for i in 0..reader.row_group_count() {
    let row_group = reader.read_decoded_row_group(i)?;
    process_row_group(row_group)?;  // Only in-memory one at a time
}
```

#### 3. Cache Headers
```rust
// ✅ GOOD - read schema once, reuse
let reader = FileReader::new("data.qrd")?;
let schema = reader.schema().clone();  // Cache
let row_count = reader.row_count();    // Cache
let columns = schema.fields.len();

// Use cached values
for _ in 0..row_count {
    // ... process
}
```

### Encryption

#### 1. Never Hardcode Keys
```rust
// ❌ DANGEROUS
const KEY: &[u8] = b"my_super_secret_key_12345678901";

// ✅ SAFE - load from environment
let key = std::env::var("QRD_ENCRYPTION_KEY")?
    .as_bytes()
    .to_vec();

// ✅ SAFER - load from vault
let key = load_from_vault("qrd_key")?;
```

#### 2. Generate Random Salts
```rust
// ❌ BAD - predictable salt
let salt = b"same_salt_always_";

// ✅ GOOD - random salt for each password
let salt = EncryptionConfig::generate_salt();
let config = EncryptionConfig::derive_from_password(password, &salt)?;

// Store salt with encrypted data (it's not secret)
save_with_salt(data, &salt)?;
```

#### 3. Use Password Derivation for User Encryption
```rust
// ✅ GOOD - user-friendly password-based
let password = get_user_password()?;
let salt = EncryptionConfig::generate_salt();
let config = EncryptionConfig::derive_from_password(password, &salt)?;

// ✅ BETTER - use key file for machine-to-machine
let key = EncryptionConfig::generate_key();
let config = EncryptionConfig::new(key)?;
store_key_securely(key)?;
```

### Error Correction

#### 1. Choose Appropriate Parity Level
```rust
// Risk Profile Mapping:
match reliability_level {
    // Acceptable Loss: Single network packet
    Standard => EccConfig::with_chunk_size(1, 1024)?,      // 12.5% overhead
    
    // Acceptable Loss: Few blocks
    High => EccConfig::with_chunk_size(2, 512)?,          // 25% overhead
    
    // Acceptable Loss: Up to 25% of data
    Critical => EccConfig::with_chunk_size(3, 256)?,      // 50% overhead
    
    // Acceptable Loss: None - critical data
    Archive => EccConfig::with_chunk_size(5, 256)?,       // 100% overhead
}
```

#### 2. Don't Combine with Inappropriate Compression
```rust
// ECC + Encryption can reduce compression effectiveness
// Compression → Encryption order recommended:
// 1. Original data
// 2. Apply compression
// 3. Apply ECC
// 4. Apply encryption (random output doesn't compress!)
```

---

## Common Patterns

### Pattern 1: Stream Large CSV to QRD

```rust
use std::fs::File;
use csv::Reader;

fn csv_to_qrd(csv_path: &str, qrd_path: &str) -> Result<()> {
    // Read CSV header to determine schema
    let file = File::open(csv_path)?;
    let mut reader = Reader::from_reader(file);
    
    let headers = reader.headers()?.clone();
    
    // Build schema (assume all String for simplicity)
    let mut builder = SchemaBuilder::new();
    for header in &headers {
        builder = builder.add_field(
            header,
            FieldType::String,
            Nullability::Optional
        )?;
    }
    let schema = builder.build()?;
    
    // Write QRD file
    let mut writer = FileWriter::new(Path::new(qrd_path), schema)?;
    
    for record in reader.records() {
        let record = record?;
        let row: Vec<Vec<u8>> = record
            .iter()
            .map(|field| serialize_string(field))
            .collect();
        writer.write_row(row)?;
    }
    
    writer.finish()?;
    Ok(())
}
```

### Pattern 2: Encrypted Backup

```rust
fn backup_with_encryption(
    source_path: &str,
    backup_path: &str,
    password: &str,
) -> Result<()> {
    // Read original file
    let data = std::fs::read(source_path)?;
    
    // Generate random salt for this backup
    let salt = EncryptionConfig::generate_salt();
    let config = EncryptionConfig::derive_from_password(password, &salt)?;
    
    // Encrypt
    let encrypted = encrypt(&data, &config)?;
    
    // Write backup with salt prefix
    let mut backup_data = salt.clone();
    backup_data.extend_from_slice(&encrypted);
    std::fs::write(backup_path, backup_data)?;
    
    Ok(())
}

fn restore_from_backup(
    backup_path: &str,
    output_path: &str,
    password: &str,
) -> Result<()> {
    // Read backup
    let backup_data = std::fs::read(backup_path)?;
    
    // Extract salt (first 32 bytes)
    if backup_data.len() < 32 {
        return Err("Backup too small".into());
    }
    let (salt_slice, encrypted) = backup_data.split_at(32);
    let salt = salt_slice.to_vec();
    
    // Decrypt
    let config = EncryptionConfig::derive_from_password(password, &salt)?;
    let original = decrypt(encrypted, &config)?;
    
    // Write restored file
    std::fs::write(output_path, original)?;
    
    Ok(())
}
```

### Pattern 3: Resilient File Transfer

```rust
fn transfer_with_ecc(
    source_path: &str,
    dest_path: &str,
    reliability: EccReliability,
) -> Result<()> {
    let config = match reliability {
        EccReliability::Standard => EccConfig::with_chunk_size(2, 1024)?,
        EccReliability::High => EccConfig::with_chunk_size(3, 512)?,
    };
    
    // Read original
    let data = std::fs::read(source_path)?;
    
    // Add ECC
    let mut codec = EccCodec::new(config)?;
    let encoded = codec.encode(&data)?;
    
    // Serialize shards to file
    let shards = encoded.shards();
    for (i, shard) in shards.iter().enumerate() {
        let shard_path = format!("{}.shard.{}", dest_path, i);
        std::fs::write(shard_path, shard)?;
    }
    
    Ok(())
}

fn recover_from_transfer(
    dest_path_pattern: &str,
    config: &EccConfig,
) -> Result<Vec<u8>> {
    // Read available shards
    let mut shards: Vec<Option<Vec<u8>>> = Vec::new();
    
    for i in 0..10 {  // Assume max 10 shards
        let shard_path = format!("{}.shard.{}", dest_path_pattern, i);
        match std::fs::read(&shard_path) {
            Ok(shard) => shards.push(Some(shard)),
            Err(_) => shards.push(None),
        }
    }
    
    // Recover if enough shards present
    decode_and_recover(&shards, config)
}
```

### Pattern 4: Multi-Language Processing

```rust
// Rust creates file with specific encodings
fn create_canonical_file(path: &str) -> Result<()> {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)?
        .add_field("tags", FieldType::String, Nullability::Required)?
        .build()?;
    
    let mut writer = FileWriter::new(Path::new(path), schema)?;
    
    // Write deterministic data
    for i in 0..1000 {
        let id = (i as i64).to_le_bytes().to_vec();
        let tags = serialize_string("tag_set_1,tag_set_2,tag_set_3");
        writer.write_row(vec![id, tags])?;
    }
    
    writer.finish()?;
    Ok(())
}

// Python reads and validates
// import qrd
// reader = qrd.FileReader("data.qrd")
// assert reader.row_count == 1000
// ...

// TypeScript verifies consistency
// const reader = new FileReader("data.qrd");
// assert(reader.rowCount() === 1000);
// ...
```

---

## Performance Optimization

### Memory Profiling

```rust
fn profile_memory(file_path: &str) -> Result<()> {
    use qrd_core::memory_profiling::MemoryProfiler;
    
    let mut profiler = MemoryProfiler::new();
    profiler.start();
    
    // Your code here
    let reader = FileReader::new(file_path)?;
    let _ = reader.read_all()?;
    
    profiler.stop();
    profiler.report();
    
    Ok(())
}
```

### CPU Profiling

```bash
# Flamegraph: measure where time is spent
cargo install flamegraph
cargo flamegraph --bin my_app

# Criterion: automated benchmarking
cargo bench
```

### Optimization Checklist

- [ ] Batch writes (not individual rows)
- [ ] Use partial reads for large files
- [ ] Choose appropriate row group size
- [ ] Enable SIMD (default on x86_64)
- [ ] Cache schema/metadata
- [ ] Avoid unnecessary encryption/ECC
- [ ] Profile on representative data

---

## Troubleshooting

### Issue: Out of Memory

**Symptom:** `Error: Memory allocation failed`

**Diagnosis:**
```rust
let file_size = std::fs::metadata("data.qrd")?.len();
let available_memory = get_available_memory();

if file_size > available_memory {
    println!("File larger than available RAM!");
}
```

**Solutions:**
1. Use partial reads instead of `read_all()`
2. Increase row group size
3. Process files on system with more RAM
4. Stream processing instead of loading entire file

### Issue: Slow Write Performance

**Symptom:** Write speed much slower than expected

**Diagnosis:**
```rust
// Measure write speed
let start = Instant::now();
for i in 0..100_000 {
    writer.write_row(vec![...])?;
}
let elapsed = start.elapsed();
let rows_per_sec = 100_000 / elapsed.as_secs_f64();
println!("Write speed: {:.0} rows/sec", rows_per_sec);
```

**Solutions:**
1. Batch writes: `write_rows()` instead of `write_row()`
2. Reduce row group size if compression takes too long
3. Disable compression for testing
4. Check disk I/O speed (`iostat`)

### Issue: File Larger Than Expected

**Symptom:** Compressed file still very large

**Diagnosis:**
```rust
let original_size = calculate_original_size();
let file_size = std::fs::metadata("data.qrd")?.len();
let ratio = file_size as f64 / original_size as f64;

println!("Compression ratio: {:.1}%", ratio * 100.0);
```

**Root Causes & Solutions:**
- Data already compressed → Use NONE codec
- Random/encrypted data → Compression ineffective
- Mixed compressible/incompressible → Split into separate files

### Issue: Encryption Failed

**Symptom:** `Error: Encryption failed`

**Diagnosis:**
```rust
// Check key size
if key.len() != 32 {
    println!("Key must be 32 bytes, got {}", key.len());
}

// Check password strength
if password.len() < 8 {
    println!("Password too short!");
}
```

**Solutions:**
1. Verify key is exactly 32 bytes
2. Use stronger passwords (16+ chars)
3. Check salt is 32 bytes
4. Regenerate key/salt

### Issue: Reconstruction Failed

**Symptom:** `Error: Cannot recover - too many losses`

**Diagnosis:**
```rust
let lost_shards = count_none_shards(&shards);
let required_shards = data_chunks;
let available_shards = shards.len() - lost_shards;

if available_shards < required_shards {
    println!("Have {}, need {}", available_shards, required_shards);
}
```

**Solutions:**
1. Increase parity level before encoding
2. Accept that too much data was lost
3. Use backup from earlier checkpoint

### Issue: File Corruption Detected

**Symptom:** `Error: CRC32 mismatch`

**Diagnosis:**
```rust
// Validate file integrity
let reader = FileReader::new("data.qrd")?;
let report = reader.validate_integrity()?;

for error in report.errors {
    println!("Row group {}: {}", error.row_group, error.message);
}
```

**Solutions:**
1. Restore from backup
2. Attempt recovery with ECC (if available)
3. Partial recovery of undamaged row groups
4. Investigate storage/network reliability

---

## Security Checklist

### Development
- [ ] No hardcoded secrets
- [ ] Secrets from environment variables
- [ ] Secure random number generation
- [ ] Input validation on all user data
- [ ] No debug prints of sensitive data

### Production
- [ ] Use encryption for sensitive data
- [ ] Strong passwords (16+ chars, mixed case, numbers, symbols)
- [ ] Secure key storage (vault, HSM, KMS)
- [ ] Regular backups of encryption keys
- [ ] Log access to sensitive files
- [ ] Monitor for anomalies
- [ ] Disable debug logging

### Operations
- [ ] Encrypt files at rest
- [ ] Encrypt transmission (HTTPS/TLS)
- [ ] Restrict file permissions (600 or 0o600)
- [ ] Regular security audits
- [ ] Update to latest SDK version
- [ ] Monitor for CVE alerts

---

## Migration Recipes

### From Parquet

```python
import pandas as pd
import pyarrow.parquet as pq
import qrd

# Read Parquet
pq_table = pq.read_table("data.parquet")
df = pq_table.to_pandas()

# Map types
type_map = {
    'int64': qrd.FieldType.Int64,
    'float64': qrd.FieldType.Float64,
    'object': qrd.FieldType.String,
    'bool': qrd.FieldType.Boolean,
}

# Create schema
schema = qrd.SchemaBuilder()
for col in df.columns:
    qrd_type = type_map.get(str(df[col].dtype), qrd.FieldType.String)
    schema.add_field(col, qrd_type)
schema = schema.build()

# Write QRD
writer = qrd.FileWriter("data.qrd", schema)
for _, row in df.iterrows():
    writer.write_row(list(row))
writer.finish()
```

### From SQLite

```rust
use rusqlite::Connection;
use qrd_core::{schema::*, writer::FileWriter};

fn sqlite_to_qrd(db_path: &str, table: &str, qrd_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    // Get column info
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    
    let mut builder = SchemaBuilder::new();
    let columns: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    
    for (name, type_str) in &columns {
        let field_type = match type_str.as_str() {
            "INTEGER" => FieldType::Int64,
            "REAL" => FieldType::Float64,
            "TEXT" => FieldType::String,
            _ => FieldType::Blob,
        };
        builder = builder.add_field(name, field_type, Nullability::Optional)?;
    }
    
    let schema = builder.build()?;
    let mut writer = FileWriter::new(Path::new(qrd_path), schema)?;
    
    // Extract all rows
    let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table))?;
    let rows = stmt.query_map([], |row| {
        // Serialize each column...
        Ok(vec![])
    })?;
    
    for row in rows {
        writer.write_row(row?)?;
    }
    
    writer.finish()?;
    Ok(())
}
```

---

**Version:** 1.2.0  
**Last Updated:** May 2026  
**Feedback:** Please report issues at https://github.com/zenipara/QRD-SDK/issues
