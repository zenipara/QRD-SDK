# QRD-SDK CLI Tools Reference

## Overview

QRD-SDK provides command-line tools for file inspection, validation, conversion, and diagnostics.

---

## qrd-inspect

Inspect and display QRD file metadata and content.

```bash
qrd-inspect [OPTIONS] <FILE>
```

### Options

```bash
--schema              Show schema definition
--statistics          Show row/column statistics  
--encoding            Show encoding information
--compression         Show compression details
--row-groups          List all row groups with offsets
--columns N           Show specific columns (comma-separated, or ALL)
--rows N              Show first N rows (default: 10)
--format FORMAT       Output format: json, csv, table (default: table)
```

### Examples

```bash
# Show schema and first 10 rows
qrd-inspect file.qrd

# Show detailed statistics
qrd-inspect file.qrd --statistics --schema

# Export first 100 rows as JSON
qrd-inspect file.qrd --rows 100 --format json > export.json

# Show row group offsets
qrd-inspect file.qrd --row-groups
```

---

## qrd-validate

Validate QRD file format and integrity.

```bash
qrd-validate [OPTIONS] <FILE> [FILE...]
```

### Options

```bash
--checksum            Verify CRC32 checksums
--schema              Validate schema consistency
--offsets             Verify row group offsets
--deep                Full format validation (slow)
--repair              Attempt repair of corrupted file
--output FILE         Write repaired file
--ecc                 Use error correction if available
```

### Examples

```bash
# Quick format check
qrd-validate file.qrd

# Deep validation with checksum verification
qrd-validate file.qrd --deep --checksum

# Attempt repair of corrupted file
qrd-validate corrupted.qrd --repair --output repaired.qrd

# Batch validation
for f in *.qrd; do qrd-validate "$f" || echo "FAILED: $f"; done
```

---

## qrd-convert

Convert between QRD and other formats (CSV, JSON, Parquet).

```bash
qrd-convert [OPTIONS] --input INPUT --output OUTPUT
```

### Options

```bash
--input-format FMT     Input format: qrd, csv, json, parquet (auto-detect)
--output-format FMT    Output format: qrd, csv, json, parquet (required)
--schema FILE          Schema file (for CSV input)
--compression COMP     Compression for QRD output: zstd, lz4, none
--compression-level L  Compression level (0-22)
--row-group-size N     Row group size in bytes (default: 10M)
--sampling N           Sample every Nth row (default: 1 - no sampling)
```

### Examples

```bash
# CSV to QRD
qrd-convert --input data.csv --output data.qrd --output-format qrd

# QRD to JSON (with sampling)
qrd-convert --input data.qrd --output sample.json --sampling 100

# Optimize QRD compression
qrd-convert --input old.qrd --output new.qrd \
  --compression zstd --compression-level 20

# Parquet to QRD
qrd-convert --input data.parquet --output data.qrd --output-format qrd
```

---

## qrd-bench

Run performance benchmarks.

```bash
qrd-bench [OPTIONS] [BENCHMARK]
```

### Benchmarks

- `write` - Write performance (rows/sec)
- `read` - Read performance (rows/sec)
- `compression` - Compression ratio and speed
- `encryption` - Encryption overhead
- `roundtrip` - Write-then-read full cycle

### Options

```bash
--rows N               Number of rows to benchmark (default: 1M)
--row-size N           Bytes per row (default: 1KB)
--compression COMP     Compression algorithm
--encryption          Enable encryption
--parallelism N       Number of threads
--output FILE         Save results to JSON
```

### Examples

```bash
# Full benchmark suite
qrd-bench

# Custom write benchmark
qrd-bench write --rows 10M --row-size 10KB

# Compare compression algorithms
qrd-bench compression --compression zstd
qrd-bench compression --compression lz4

# Output to file for analysis
qrd-bench --output results.json
```

---

## qrd-schema

Extract, validate, and manage QRD schemas.

```bash
qrd-schema [OPTIONS] <COMMAND>
```

### Commands

**extract**
```bash
qrd-schema extract <FILE>          # Extract schema from QRD file
```

**generate**
```bash
qrd-schema generate <DATA_FILE>    # Generate schema from CSV/JSON
```

**validate**
```bash
qrd-schema validate <SCHEMA_FILE>  # Validate schema definition
```

**compare**
```bash
qrd-schema compare <FILE1> <FILE2> # Compare schemas between files
```

### Examples

```bash
# Extract schema from file
qrd-schema extract data.qrd > schema.yaml

# Generate schema from CSV
qrd-schema generate data.csv > schema.yaml

# Validate schema
qrd-schema validate schema.yaml

# Compare schemas
qrd-schema compare v1.qrd v2.qrd
```

---

## qrd-encrypt/decrypt

Manage encryption for QRD files.

```bash
qrd-encrypt [OPTIONS] <INPUT> <OUTPUT>
qrd-decrypt [OPTIONS] <INPUT> <OUTPUT>
```

### Options

```bash
--key KEY              Encryption key (32-byte hex string)
--key-file FILE        Read key from file
--key-provider TYPE    Provider: env, file, vault, aws-kms
--algorithm ALG        Encryption algorithm (default: aes-256-gcm)
--kdf-iterations N     KDF iterations if deriving key
```

### Examples

```bash
# Generate and encrypt a file
export QRD_KEY=$(openssl rand -hex 32)
qrd-encrypt plaintext.qrd encrypted.qrd --key $QRD_KEY

# Decrypt with key file
qrd-decrypt encrypted.qrd decrypted.qrd --key-file /etc/qrd/master.key

# Encrypt with AWS KMS
qrd-encrypt plaintext.qrd encrypted.qrd \
  --key-provider aws-kms \
  --kms-key-id arn:aws:kms:us-east-1:123456789012:key/xxxxx
```

---

## qrd-backup

Create and restore file backups with verification.

```bash
qrd-backup [OPTIONS] <COMMAND>
```

### Commands

**create**
```bash
qrd-backup create <SOURCE_DIR> <BACKUP_DIR>
```

**verify**
```bash
qrd-backup verify <BACKUP_DIR>
```

**restore**
```bash
qrd-backup restore <BACKUP_DIR> <TARGET_DIR>
```

### Options

```bash
--compress               Compress backup
--verify                 Verify after backup
--incremental           Only backup changed files
--prune-days N          Delete backups older than N days
--threads N             Number of parallel threads
```

### Examples

```bash
# Full backup with verification
qrd-backup create /data/qrd /backups/qrd-$(date +%Y%m%d) --verify

# Verify backup integrity
qrd-backup verify /backups/qrd-20260510

# Restore from backup
qrd-backup restore /backups/qrd-20260510 /data/qrd-restored

# Incremental backup with pruning
qrd-backup create /data/qrd /backups/qrd-latest \
  --incremental --prune-days 30
```

---

## qrd-diagnostic

Collect diagnostic information.

```bash
qrd-diagnostic [OPTIONS]
```

### Options

```bash
--system               Include system information
--environment          Include environment variables
--dependencies        Check library dependencies
--performance         Run quick performance checks
--output FILE         Save diagnostic bundle
```

### Examples

```bash
# Collect full diagnostics
qrd-diagnostic --system --environment --dependencies --performance

# Save to file for reporting
qrd-diagnostic --output diagnostic-report.json
```

---

## qrd-repair

Repair corrupted QRD files using ECC or other recovery methods.

```bash
qrd-repair [OPTIONS] <INPUT> <OUTPUT>
```

### Options

```bash
--ecc                  Use error correction codes
--force                Force repair even if risky
--keep-corrupt         Keep unrecoverable sections
--log FILE             Save repair log
```

### Examples

```bash
# Attempt repair with ECC
qrd-repair corrupted.qrd repaired.qrd --ecc

# Aggressive recovery
qrd-repair corrupted.qrd repaired.qrd --force --keep-corrupt

# Save logs
qrd-repair corrupted.qrd repaired.qrd --log repair.log
```

---

## Installation

All CLI tools are included in the main QRD-SDK release:

```bash
# Install with Rust
cargo install qrd-cli

# Or from release
wget https://github.com/zenipara/QRD-SDK/releases/latest/download/qrd-cli-linux-x86_64
chmod +x qrd-cli-linux-x86_64
```

---

## Tips & Tricks

### Batch Operations

```bash
# Validate all QRD files
find /data/qrd -name "*.qrd" -exec qrd-validate {} \;

# Convert multiple files
for f in *.csv; do
  qrd-convert --input "$f" --output "${f%.csv}.qrd"
done

# Backup with timestamp
for dir in /data/*/; do
  qrd-backup create "$dir" "/backups/$(basename $dir)-$(date +%Y%m%d-%H%M%S)"
done
```

### Performance Analysis

```bash
# Run full benchmark suite
qrd-bench > benchmark.json

# Generate comparison report
python scripts/analyze-benchmarks.py benchmark.json
```

### Troubleshooting

```bash
# Enable verbose logging
RUST_LOG=debug qrd-tool <command>

# Collect diagnostics
qrd-diagnostic --output debug.json
```

---

## Environment Variables

Global QRD environment variables:

```bash
QRD_LOG_LEVEL           # Logging level: error, warn, info, debug
QRD_PARALLELISM         # Number of parallel threads
QRD_ROW_GROUP_SIZE      # Default row group size in bytes
QRD_COMPRESSION         # Default compression algorithm
QRD_COMPRESSION_LEVEL   # Default compression level
QRD_ENCRYPTION_KEY      # Default encryption key
QRD_KEY_PROVIDER        # Default key provider
```

---

## Getting Help

```bash
# Show help for any command
qrd-inspect --help
qrd-validate --help
qrd-convert --help

# Check version
qrd --version

# Interactive help
qrd help
```
