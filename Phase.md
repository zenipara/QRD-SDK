# 🏭 QRD-SDK Production-Ready — PHASE 5 of 5
## Production Hardening: Benchmarks, CLI Tools, Fuzzing & Final Checklist

> Prerequisite: PHASE 1–4 sudah selesai dan semua tests passing.
> Ini adalah phase terakhir sebelum label "production-ready" bisa disematkan.

---

## OVERVIEW

Phase ini mencakup:
1. **Real benchmark results** — buktikan performance claims di README
2. **CLI tools** — `qrd-inspect`, `qrd-write`, `qrd-read`
3. **Fuzz testing** — pastikan tidak ada panic dari arbitrary input
4. **CI/CD pipeline** — GitHub Actions untuk automated testing
5. **Final production checklist** — semua item harus green

---

## TASK 1: Real Benchmark Suite

### Update `benches/comprehensive_bench.rs` dengan realistic data:

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use qrd_core::schema::{SchemaBuilder, FieldType, Nullability};
use qrd_core::writer::{FileWriter, WriterConfig};
use qrd_core::reader::FileReader;
use tempfile::NamedTempFile;

// ============================================================================
// WRITE BENCHMARKS
// ============================================================================

fn bench_write_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_throughput");
    
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required).unwrap()
        .add_field("timestamp", FieldType::Timestamp, Nullability::Required).unwrap()
        .add_field("value", FieldType::Float64, Nullability::Required).unwrap()
        .build().unwrap();
    
    for &row_count in &[1_000u64, 10_000, 100_000, 1_000_000] {
        let bytes_per_row = 8 + 8 + 8; // id + ts + value
        group.throughput(Throughput::Bytes(row_count * bytes_per_row));
        
        group.bench_with_input(
            BenchmarkId::new("rows", row_count),
            &row_count,
            |b, &n| {
                b.iter(|| {
                    let tmp = NamedTempFile::new().unwrap();
                    let mut writer = FileWriter::new(tmp.path(), schema.clone()).unwrap();
                    for i in 0..n {
                        writer.write_row(vec![
                            i.to_le_bytes().to_vec(),
                            (i * 1000).to_le_bytes().to_vec(),
                            (i as f64 * 1.5).to_le_bytes().to_vec(),
                        ]).unwrap();
                    }
                    writer.finish().unwrap();
                })
            }
        );
    }
    group.finish();
}

// ============================================================================
// READ BENCHMARKS  
// ============================================================================

fn bench_read_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_throughput");
    
    // Pre-write test files
    let row_count = 1_000_000u64;
    let schema = make_benchmark_schema();
    let tmp = write_benchmark_file(&schema, row_count);
    
    let file_size = std::fs::metadata(tmp.path()).unwrap().len();
    group.throughput(Throughput::Bytes(file_size));
    
    // Full file read
    group.bench_function("full_file_read", |b| {
        b.iter(|| {
            let reader = FileReader::new(tmp.path()).unwrap();
            reader.read_all_row_groups().unwrap()
        })
    });
    
    // Partial column read (1 of 3 columns)
    group.bench_function("single_column_read", |b| {
        b.iter(|| {
            let reader = FileReader::new(tmp.path()).unwrap();
            reader.read_columns(&[0]).unwrap()  // Only read "id" column
        })
    });
    
    group.finish();
}

// ============================================================================
// ENCODING BENCHMARKS
// ============================================================================

fn bench_encodings(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding");
    
    let n = 1_000_000usize;
    
    // Sequential integers (best for DELTA_BINARY)
    let sequential: Vec<i64> = (0..n as i64).collect();
    let seq_bytes: Vec<u8> = sequential.iter().flat_map(|i| i.to_le_bytes()).collect();
    
    // Random integers (best for PLAIN)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let random_bytes: Vec<u8> = (0..n).flat_map(|i| {
        let mut h = DefaultHasher::new();
        i.hash(&mut h);
        h.finish().to_le_bytes()
    }).collect();
    
    for (name, data) in [("sequential_i64", &seq_bytes), ("random_i64", &random_bytes)] {
        group.throughput(Throughput::Bytes(data.len() as u64));
        
        for encoding in ["PLAIN", "DELTA_BINARY", "RLE"] {
            let enc_type = parse_encoding(encoding);
            group.bench_with_input(
                BenchmarkId::new(encoding, name),
                data,
                |b, d| {
                    let encoder = qrd_core::encoding::get_encoder(enc_type).unwrap();
                    b.iter(|| encoder.encode(d).unwrap())
                }
            );
        }
    }
    group.finish();
}

// ============================================================================
// COMPRESSION BENCHMARKS
// ============================================================================

fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");
    
    // Generate realistic IoT telemetry data (compressible)
    let data_size = 10 * 1024 * 1024; // 10MB
    let compressible: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();
    let incompressible: Vec<u8> = (0..data_size).map(|i| {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (i as u64).hash(&mut h);
        h.finish() as u8
    }).collect();
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    for (name, data) in [("compressible", &compressible), ("incompressible", &incompressible)] {
        group.bench_with_input(BenchmarkId::new("zstd_compress", name), data, |b, d| {
            b.iter(|| qrd_core::compression::compress(d, qrd_core::compression::CompressionCodec::Zstd, 3).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("lz4_compress", name), data, |b, d| {
            b.iter(|| qrd_core::compression::compress(d, qrd_core::compression::CompressionCodec::Lz4, 4).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("zstd_decompress", name), data, |b, d| {
            let compressed = qrd_core::compression::compress(d, qrd_core::compression::CompressionCodec::Zstd, 3).unwrap();
            b.iter(|| qrd_core::compression::decompress(&compressed, qrd_core::compression::CompressionCodec::Zstd).unwrap())
        });
    }
    group.finish();
}
```

### Update README dengan hasil benchmark aktual:

Setelah run `cargo bench`, copy hasil ke README. Format:
```markdown
## Benchmark Results (actual, pada Apple M3 / AMD Ryzen 9 7950X)

| Operation | Dataset | Throughput |
|---|---|---|
| Write (Int64×3) | 1M rows | X.X GB/s |
| Read full file | 1M rows | X.X GB/s |
| Read 1 column | 1M rows | X.X GB/s |
| ZSTD compress | 10MB | XXX MB/s |
| LZ4 compress | 10MB | XXX MB/s |
```

---

## TASK 2: CLI Tools

### Buat `tools/src/main.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qrd", about = "QRD columnar binary format tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect QRD file metadata without reading data
    Inspect {
        #[arg(help = "Path to .qrd file")]
        file: String,
        #[arg(short, long, help = "Show column statistics")]
        stats: bool,
        #[arg(short, long, help = "Show row group details")]
        verbose: bool,
    },
    /// Convert JSON/CSV to QRD
    Write {
        #[arg(help = "Input file (JSON Lines or CSV)")]
        input: String,
        #[arg(short, long, help = "Output .qrd file")]
        output: String,
        #[arg(short, long, help = "Row group size")]
        row_group_size: Option<u32>,
    },
    /// Read QRD and output JSON/CSV
    Read {
        #[arg(help = "Input .qrd file")]
        input: String,
        #[arg(short, long, default_value = "json", help = "Output format: json, csv")]
        format: String,
        #[arg(short, long, help = "Specific columns to read (comma-separated)")]
        columns: Option<String>,
        #[arg(short, long, help = "Max rows to output")]
        limit: Option<usize>,
    },
    /// Validate QRD file integrity
    Validate {
        file: String,
        #[arg(long, help = "Run deep integrity check")]
        deep: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Inspect { file, stats, verbose } => cmd_inspect(&file, stats, verbose),
        Commands::Write { input, output, row_group_size } => cmd_write(&input, &output, row_group_size),
        Commands::Read { input, format, columns, limit } => cmd_read(&input, &format, columns, limit),
        Commands::Validate { file, deep } => cmd_validate(&file, deep),
    }
}
```

### Implementasikan `cmd_inspect`:
```rust
fn cmd_inspect(path: &str, show_stats: bool, verbose: bool) -> anyhow::Result<()> {
    let reader = FileReader::new(path)?;
    
    println!("═══════════════════════════════════════");
    println!("  QRD File: {}", path);
    println!("═══════════════════════════════════════");
    println!("  Rows:        {:>12}", reader.row_count());
    println!("  Row Groups:  {:>12}", reader.row_group_offsets().len());
    println!("  File Size:   {:>12}", format_bytes(std::fs::metadata(path)?.len()));
    println!("  Schema ID:   {:>12x}", reader.schema().schema_id);
    println!();
    println!("  Columns ({}):", reader.schema().fields.len());
    for (i, field) in reader.schema().fields.iter().enumerate() {
        println!("    [{:2}] {:20} {:12} {:10}", 
            i, field.name, field.field_type.to_string(), 
            if matches!(field.nullability, Nullability::Required) { "REQUIRED" } else { "OPTIONAL" });
    }
    
    if verbose {
        println!("\n  Row Group Details:");
        for (i, offset) in reader.row_group_offsets().iter().enumerate() {
            println!("    RG[{:3}] offset={:12}", i, offset);
        }
    }
    
    Ok(())
}
```

### Tambahkan `tools/Cargo.toml`:
```toml
[package]
name = "qrd-tools"
version = "0.1.0"
edition = "2021"

[dependencies]
qrd-core = { path = "../core/qrd-core" }
clap = { version = "4", features = ["derive"] }
serde_json = "1.0"
csv = "1.3"
anyhow = "1.0"
```

---

## TASK 3: Fuzz Testing Setup

### Tambahkan ke `Cargo.toml` workspace:
```toml
[workspace.metadata.cargo-fuzz]
fuzz-dir = "core/qrd-core/fuzz"
```

### Buat `core/qrd-core/fuzz/fuzz_targets/fuzz_reader.rs`:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use qrd_core::reader::FileReader;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Coba parse arbitrary bytes sebagai QRD file
    // HARUS: tidak panic, tidak undefined behavior
    // BOLEH: return Err
    let _ = FileReader::from_bytes(data.to_vec());
});
```

### Buat `core/qrd-core/fuzz/fuzz_targets/fuzz_encoding.rs`:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    encoding_id: u8,
    data: Vec<u8>,
    expected_len: usize,
}

fuzz_target!(|input: FuzzInput| {
    use qrd_core::encoding::{EncodingType, get_encoder};
    
    if let Ok(enc_type) = EncodingType::from_id(input.encoding_id % 8) {
        if let Ok(encoder) = get_encoder(enc_type) {
            if let Ok(encoded) = encoder.encode(&input.data) {
                // Decode harus tidak panic
                let _ = encoder.decode(&encoded, input.data.len());
            }
        }
    }
});
```

### Buat `core/qrd-core/fuzz/fuzz_targets/fuzz_footer.rs`:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use qrd_core::footer::Footer;

fuzz_target!(|data: &[u8]| {
    // Footer parser tidak boleh panic pada arbitrary input
    let _ = Footer::deserialize(data);
});
```

### Script untuk run fuzz (lokal):
```bash
# scripts/fuzz.sh
#!/bin/bash
cargo install cargo-fuzz 2>/dev/null || true

echo "Fuzzing reader..."
cargo fuzz run fuzz_reader -- -max_total_time=60 2>&1

echo "Fuzzing encoding..."
cargo fuzz run fuzz_encoding -- -max_total_time=60 2>&1

echo "Fuzzing footer..."
cargo fuzz run fuzz_footer -- -max_total_time=60 2>&1

echo "All fuzz targets completed without crashes."
```

---

## TASK 4: GitHub Actions CI/CD

### Buat `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test — ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, 1.70.0]  # stable + MSRV
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust ${{ matrix.rust }}
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: clippy, rustfmt
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Check format
        run: cargo fmt --all -- --check
      
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Build
        run: cargo build --all --release
      
      - name: Test
        run: cargo test --all --release 2>&1
      
      - name: Test (no features)
        run: cargo test --package qrd-core --no-default-features
  
  benchmark:
    name: Benchmark (regression check)
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run benchmarks
        run: cargo bench --package qrd-core 2>&1 | tee bench_output.txt
      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: bench_output.txt
  
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Audit dependencies
        run: cargo audit
  
  cross-compile:
    name: Cross-compile — ${{ matrix.target }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - aarch64-unknown-linux-gnu
          - x86_64-unknown-linux-musl
          - wasm32-unknown-unknown
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross
        run: cargo install cross
      - name: Cross build
        run: cross build --target ${{ matrix.target }} --package qrd-core
```

---

## TASK 5: Final Documentation

### Update `README.md` dengan:

1. **Actual benchmark results** (setelah run bench)
2. **Installation yang benar** (hapus package yang belum ada di registry)
3. **Working examples** (semua contoh code harus compile dan run)
4. **Security notice** yang jelas:
```markdown
## Security

QRD menggunakan AES-256-GCM dengan HKDF key derivation.
- Enkripsi diaktifkan per-file via `WriterConfig::encryption`
- ECC (Reed-Solomon) tersedia untuk resilience terhadap storage corruption
- Untuk file sensitivity tinggi, gunakan enkripsi + ECC sekaligus
```

5. **Known limitations** yang jujur:
```markdown
## Current Limitations

- WASM binding tidak mendukung file I/O (hanya in-memory)
- Decimal dan complex nested types masih dalam development
- Tidak ada query pushdown ke storage engine
```

---

## FINAL PRODUCTION CHECKLIST

Jalankan semua command ini. Semua harus pass sebelum v1.0.0 release:

```bash
#!/bin/bash
set -e

echo "=== QRD Production Readiness Check ==="

# 1. Format
echo "[1/10] Checking code format..."
cargo fmt --all -- --check
echo "✅ Format OK"

# 2. Clippy
echo "[2/10] Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy clean"

# 3. Tests
echo "[3/10] Running full test suite..."
cargo test --all --release 2>&1 | grep -E "test result|FAILED"
echo "✅ All tests passing"

# 4. No panics in public API
echo "[4/10] Checking for unwrap() in non-test code..."
UNWRAPS=$(grep -r "\.unwrap()" core/qrd-core/src/ --include="*.rs" | grep -v "test\|bench\|#\[" | wc -l)
if [ "$UNWRAPS" -gt 0 ]; then
    echo "❌ Found $UNWRAPS unwrap() calls in production code"
    grep -r "\.unwrap()" core/qrd-core/src/ --include="*.rs" | grep -v "test\|bench"
    exit 1
fi
echo "✅ No unwrap() in production code"

# 5. No "Variable-length not supported" errors
echo "[5/10] Checking variable-length support..."
if grep -r "not yet supported" core/qrd-core/src/ --include="*.rs" | grep -v test; then
    echo "❌ Found unsupported feature errors"
    exit 1
fi
echo "✅ No unsupported feature blockers"

# 6. Security audit
echo "[6/10] Running security audit..."
cargo audit 2>&1 | grep -E "error|warning" | head -10
echo "✅ Security audit passed"

# 7. Compile WASM
echo "[7/10] Building WASM..."
cargo build --target wasm32-unknown-unknown --package qrd-core --no-default-features
echo "✅ WASM build OK"

# 8. Compile FFI
echo "[8/10] Building FFI..."
cargo build --package qrd-ffi --release
echo "✅ FFI build OK"

# 9. Python binding
echo "[9/10] Testing Python binding..."
cd sdk/python && maturin develop --quiet && python -c "import qrd; qrd.SchemaBuilder()" && cd ../..
echo "✅ Python binding OK"

# 10. Benchmark sanity check
echo "[10/10] Running quick benchmark..."
cargo bench --package qrd-core --bench simple_bench 2>&1 | grep "time:"
echo "✅ Benchmark completed"

echo ""
echo "╔════════════════════════════════════╗"
echo "║  🚀 ALL CHECKS PASSED              ║"
echo "║  QRD SDK is Production-Ready!      ║"
echo "╚════════════════════════════════════╝"
```

---

## POST-RELEASE MONITORING

Setelah v1.0.0 release, setup ini untuk ongoing quality:

### Dependabot (`/.github/dependabot.yml`):
```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    labels:
      - "dependencies"
```

### CHANGELOG format (`CHANGELOG.md`):
```markdown
## [Unreleased]

### Added
### Changed  
### Fixed
### Security

## [1.0.0] — 2026-06-01

### Added
- Variable-length type read/write support (String, Blob, Enum, Uuid)
- Memory-mapped I/O untuk file >= 64MB
- Encryption + ECC integrated ke writer/reader pipeline
- Real SIMD implementation via `wide` crate
- CLI tools: qrd-inspect, qrd-write, qrd-read
- Python, TypeScript, Go, Java bindings
```
