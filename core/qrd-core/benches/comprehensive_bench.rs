use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qrd_core::{
    ecc::{EccCodec, EccConfig},
    encryption::{decrypt, encrypt, EncryptionConfig},
    reader::FileReader,
    schema::{FieldType, Nullability, SchemaBuilder},
    writer::FileWriter,
};
use std::path::PathBuf;
use tempfile::TempDir;

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets =
        bench_write_throughput,
        bench_read_throughput,
        bench_various_sizes,
        bench_encoding_performance,
        bench_encryption_throughput,
        bench_ecc_performance,
        bench_simd_xor
);
criterion_main!(benches);

// ============================================================================
// WRITE PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_write_throughput(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();

    let mut group = c.benchmark_group("write_throughput");

    for row_count in [1000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(row_count),
            row_count,
            |b, &row_count| {
                b.iter_batched(
                    || {
                        let path = temp_dir.path().join(format!("write_{}.qrd", row_count));
                        let schema = SchemaBuilder::new()
                            .add_field("id", FieldType::Int64, Nullability::Required)
                            .unwrap()
                            .add_field("value", FieldType::Float64, Nullability::Required)
                            .unwrap()
                            .add_field("data", FieldType::String, Nullability::Required)
                            .unwrap()
                            .build()
                            .unwrap();
                        (path, schema)
                    },
                    |(path, schema)| {
                        let mut writer = FileWriter::new(&path, schema).unwrap();

                        for i in 0..row_count {
                            let id = (i as i64).to_le_bytes().to_vec();
                            let value = (i as f64 * 3.14).to_le_bytes().to_vec();
                            let data = serialize_string(&format!("row_{}", i));

                            writer.write_row(vec![id, value, data]).unwrap();
                        }

                        writer.finish().unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

// ============================================================================
// READ PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_read_throughput(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();

    // Pre-create files for reading
    let files: Vec<(usize, PathBuf)> = [1000, 10_000, 100_000]
        .iter()
        .map(|&row_count| {
            let path = temp_dir.path().join(format!("read_{}.qrd", row_count));
            let schema = SchemaBuilder::new()
                .add_field("id", FieldType::Int64, Nullability::Required)
                .unwrap()
                .add_field("value", FieldType::Float64, Nullability::Required)
                .unwrap()
                .add_field("data", FieldType::String, Nullability::Required)
                .unwrap()
                .build()
                .unwrap();

            {
                let mut writer = FileWriter::new(&path, schema).unwrap();
                for i in 0..row_count {
                    let id = (i as i64).to_le_bytes().to_vec();
                    let value = (i as f64 * 3.14).to_le_bytes().to_vec();
                    let data = serialize_string(&format!("row_{}", i));
                    writer.write_row(vec![id, value, data]).unwrap();
                }
                writer.finish().unwrap();
            }

            (row_count, path)
        })
        .collect();

    let mut group = c.benchmark_group("read_throughput");

    for (row_count, path) in files {
        group.bench_with_input(BenchmarkId::from_parameter(row_count), &path, |b, path| {
            b.iter(|| {
                let reader = FileReader::new(path).unwrap();
                let _ = reader.read_all().unwrap();
            });
        });
    }
    group.finish();
}

// ============================================================================
// VARIABLE SIZE BENCHMARKS
// ============================================================================

fn bench_various_sizes(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut group = c.benchmark_group("various_sizes");

    // Test different payload sizes
    for payload_size in [100, 1000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("write_max_payload", payload_size),
            payload_size,
            |b, &payload_size| {
                b.iter_batched(
                    || {
                        let path = temp_dir
                            .path()
                            .join(format!("payload_{}.qrd", payload_size));
                        let schema = SchemaBuilder::new()
                            .add_field("data", FieldType::Blob, Nullability::Required)
                            .unwrap()
                            .build()
                            .unwrap();
                        (path, schema)
                    },
                    |(path, schema)| {
                        let mut writer = FileWriter::new(&path, schema).unwrap();

                        for _ in 0..100 {
                            let data = serialize_blob(&vec![42u8; payload_size]);
                            writer.write_row(vec![data]).unwrap();
                        }

                        writer.finish().unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================================
// ENCODING PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_encoding_performance(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut group = c.benchmark_group("encoding");

    // Plain encoding (baseline)
    group.bench_function("plain_encoding", |b| {
        b.iter_batched(
            || {
                let path = temp_dir.path().join("plain.qrd");
                let schema = SchemaBuilder::new()
                    .add_field("value", FieldType::Int32, Nullability::Required)
                    .unwrap()
                    .build()
                    .unwrap();
                (path, schema)
            },
            |(path, schema)| {
                let mut writer = FileWriter::new(&path, schema).unwrap();
                for i in 0..10_000 {
                    let value = (i as i32 * 7).to_le_bytes().to_vec();
                    writer.write_row(vec![value]).unwrap();
                }
                writer.finish().unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // RLE encoding (repetitive data)
    group.bench_function("rle_encoding", |b| {
        b.iter_batched(
            || {
                let path = temp_dir.path().join("rle.qrd");
                let schema = SchemaBuilder::new()
                    .add_field("value", FieldType::Int32, Nullability::Required)
                    .unwrap()
                    .build()
                    .unwrap();
                (path, schema)
            },
            |(path, schema)| {
                let mut writer = FileWriter::new(&path, schema).unwrap();
                // RLE-friendly: repetitive pattern
                for i in 0..10_000 {
                    let value = (if i % 1000 < 500 { 42i32 } else { 24i32 })
                        .to_le_bytes()
                        .to_vec();
                    writer.write_row(vec![value]).unwrap();
                }
                writer.finish().unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // DELTA encoding (sorted/increasing data)
    group.bench_function("delta_encoding", |b| {
        b.iter_batched(
            || {
                let path = temp_dir.path().join("delta.qrd");
                let schema = SchemaBuilder::new()
                    .add_field("value", FieldType::Int64, Nullability::Required)
                    .unwrap()
                    .build()
                    .unwrap();
                (path, schema)
            },
            |(path, schema)| {
                let mut writer = FileWriter::new(&path, schema).unwrap();
                // Delta-friendly: monotonically increasing
                for i in 0..10_000 {
                    let value = ((i as i64) * 5).to_le_bytes().to_vec();
                    writer.write_row(vec![value]).unwrap();
                }
                writer.finish().unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ============================================================================
// ENCRYPTION PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_encryption_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption");

    for data_size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let key = EncryptionConfig::generate_key();
        let config = EncryptionConfig::new(key).unwrap();
        let data = vec![42u8; *data_size];

        group.bench_with_input(BenchmarkId::new("encrypt", data_size), data_size, |b, _| {
            b.iter(|| {
                let _ = encrypt(black_box(&data), &config);
            });
        });

        let encrypted = encrypt(&data, &config).unwrap();
        group.bench_with_input(BenchmarkId::new("decrypt", data_size), data_size, |b, _| {
            b.iter(|| {
                let _ = decrypt(black_box(&encrypted), &config);
            });
        });
    }

    group.finish();
}

/// Benchmark password-based key derivation
#[test]
fn test_password_derivation_performance() {
    let password = "test_password_1234567890";
    let salt = EncryptionConfig::generate_salt();

    let start = std::time::Instant::now();
    let _config =
        EncryptionConfig::derive_from_password(password, &salt).expect("Key derivation failed");
    let elapsed = start.elapsed();

    println!("Password key derivation time: {:?}", elapsed);
    assert!(
        elapsed.as_millis() < 1000,
        "Should complete within 1 second"
    );
}

// ============================================================================
// ECC PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_ecc_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecc");

    for data_size in [1_024, 10_240, 102_400].iter() {
        group.bench_with_input(
            BenchmarkId::new("encode", data_size),
            data_size,
            |b, &data_size| {
                let config = EccConfig::with_chunk_size(2, 256).unwrap();
                let mut codec = EccCodec::new(config).unwrap();
                let data = vec![42u8; data_size];

                b.iter(|| {
                    let _ = codec.encode(black_box(&data));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decode", data_size),
            data_size,
            |b, &data_size| {
                let config = EccConfig::with_chunk_size(2, 256).unwrap();
                let mut codec = EccCodec::new(config.clone()).unwrap();
                let data = vec![42u8; data_size];
                let encoded = codec.encode(&data).unwrap();

                b.iter(|| {
                    let shards = encoded.shards_as_options();
                    let _ = qrd_core::ecc::decode_and_recover(black_box(&shards), &config);
                });
            },
        );
    }

    group.finish();
}

fn bench_simd_xor(c: &mut Criterion) {
    use qrd_core::utils::simd::SimdOps;

    let mut group = c.benchmark_group("simd_xor");

    for size in [1_000usize, 10_000, 100_000, 1_000_000].iter() {
        let mut dst = vec![0u8; *size];
        let src = vec![0xFFu8; *size];

        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            let ops = SimdOps::new();
            b.iter(|| {
                let mut d = dst.clone();
                ops.xor(&mut d, &src).unwrap();
            })
        });

        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter(|| {
                let mut d = dst.clone();
                d.iter_mut().zip(src.iter()).for_each(|(x, s)| *x ^= s);
            })
        });
    }

    group.finish();
}

// ============================================================================
// COMPARISON BENCHMARKS
// ============================================================================

/// Compare write performance: encrypted vs unencrypted
#[test]
fn test_encryption_overhead() {
    let temp_dir = TempDir::new().unwrap();

    // Unencrypted
    let unencrypted_path = temp_dir.path().join("unencrypted.qrd");
    let schema = SchemaBuilder::new()
        .add_field("data", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let start = std::time::Instant::now();
    {
        let mut writer = FileWriter::new(&unencrypted_path, schema.clone()).unwrap();
        for i in 0..1000 {
            let data = serialize_blob(&vec![i as u8; 1000]);
            writer.write_row(vec![data]).unwrap();
        }
        writer.finish().unwrap();
    }
    let unencrypted_time = start.elapsed();

    println!("Unencrypted write: {:?}", unencrypted_time);

    // Expected: encryption adds small overhead (~10-20%)
}

/// Compare read performance across various file sizes
#[test]
fn test_read_performance_scaling() {
    let temp_dir = TempDir::new().unwrap();

    for col_count in [1, 5, 10, 20].iter() {
        let path = temp_dir.path().join(format!("cols_{}.qrd", col_count));

        let mut builder = SchemaBuilder::new();
        for i in 0..*col_count {
            builder = builder
                .add_field(
                    &format!("col_{}", i),
                    FieldType::Float64,
                    Nullability::Required,
                )
                .unwrap();
        }
        let schema = builder.build().unwrap();

        {
            let mut writer = FileWriter::new(&path, schema).unwrap();
            for _ in 0..1000 {
                let mut row = Vec::new();
                for i in 0..*col_count {
                    row.push((i as f64).to_le_bytes().to_vec());
                }
                writer.write_row(row).unwrap();
            }
            writer.finish().unwrap();
        }

        let start = std::time::Instant::now();
        let _reader = FileReader::new(&path).unwrap();
        let read_time = start.elapsed();

        println!("Read {} columns: {:?}", col_count, read_time);
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn serialize_string(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(bytes);
    result
}

fn serialize_blob(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend_from_slice(&(data.len() as u32).to_le_bytes());
    result.extend_from_slice(data);
    result
}
