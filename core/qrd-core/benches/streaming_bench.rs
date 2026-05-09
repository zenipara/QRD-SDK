use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use qrd_core::writer::StreamingWriter;
use qrd_core::schema::{Schema, SchemaBuilder, FieldType, Nullability};
use std::io::Cursor;

/// Create simple test schema
fn make_schema(fields: Vec<&str>) -> Schema {
    let mut builder = SchemaBuilder::new();
    for name in fields {
        builder = builder.add_field(name, FieldType::Blob, Nullability::Required).unwrap();
    }
    builder.build().unwrap()
}

/// Benchmark streaming writer with varying row counts
fn benchmark_streaming_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_write");

    for row_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(row_count),
            row_count,
            |b, &row_count| {
                b.iter(|| {
                    let schema = black_box(make_schema(vec!["data"]));
                    let buffer = Cursor::new(Vec::new());
                    let mut writer = StreamingWriter::new(buffer, schema).unwrap();

                    for i in 0..row_count {
                        let value = (i as u32).to_le_bytes().to_vec();
                        writer.write_row(vec![value]).unwrap();
                    }

                    writer.finish().unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark row group auto-flush behavior
fn benchmark_auto_flush(c: &mut Criterion) {
    let mut group = c.benchmark_group("auto_flush");

    for rg_size in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(rg_size),
            rg_size,
            |b, &rg_size| {
                b.iter(|| {
                    let schema = black_box(make_schema(vec!["x", "y"]));
                    let buffer = Cursor::new(Vec::new());
                    let mut config = qrd_core::writer::StreamingWriterConfig::default();
                    config.row_group_size = rg_size as u32;

                    let mut writer =
                        StreamingWriter::with_config(buffer, schema, config).unwrap();

                    for i in 0..(rg_size * 3) {
                        let val1 = (i as u32).to_le_bytes().to_vec();
                        let val2 = (i as u64).to_le_bytes().to_vec();
                        writer.write_row(vec![val1, val2]).unwrap();
                    }

                    writer.finish().unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark compression codec selection
fn benchmark_compression_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_detection");

    // Low entropy (compressible) data
    let low_entropy = black_box(vec![1u8; 10000]);
    group.bench_function("detect_low_entropy", |b| {
        b.iter(|| {
            qrd_core::compression::EntropyCalculator::calculate(&low_entropy)
        });
    });

    // High entropy (incompressible) data
    let high_entropy: Vec<u8> = black_box((0..10000).map(|i| (i % 256) as u8).collect());
    group.bench_function("detect_high_entropy", |b| {
        b.iter(|| qrd_core::compression::EntropyCalculator::calculate(&high_entropy));
    });

    // Real data with pattern
    let patterned: Vec<u8> = black_box(
        (0..10000)
            .map(|i| if i % 8 == 0 { 255 } else { 0 })
            .collect(),
    );
    group.bench_function("detect_patterned", |b| {
        b.iter(|| qrd_core::compression::EntropyCalculator::calculate(&patterned));
    });

    group.finish();
}

/// Benchmark entropy-based decision making
fn benchmark_should_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("should_compress");

    let scenarios = vec![
        ("repetitive", vec![1u8; 50000]),
        ("random", (0..50000).map(|i| (i as u8).wrapping_mul(7)).collect()),
        ("mixed", {
            let mut v = vec![0u8; 50000];
            for i in 0..5000 {
                v[i * 10] = 255;
            }
            v
        }),
    ];

    for (name, data) in scenarios {
        group.bench_function(name, |b| {
            b.iter(|| {
                qrd_core::compression::CompressionSelector::should_compress(
                    black_box(&data),
                )
            });
        });
    }

    group.finish();
}

/// Benchmark memory efficiency of streaming writer
fn benchmark_memory_efficiency(c: &mut Criterion) {
    c.bench_function("memory_pool_acquire_release", |b| {
        let pool = qrd_core::writer::BufferPool::new();
        b.iter(|| {
            let buf = pool.acquire(10000);
            pool.release(buf);
        });
    });

    c.bench_function("memory_pool_reuse", |b| {
        let pool = qrd_core::writer::BufferPool::new();
        let buf = pool.acquire(10000);
        pool.release(buf);

        b.iter(|| {
            let reused = black_box(pool.acquire(5000));
            pool.release(reused);
        });
    });
}

/// Benchmark footer parsing and building
fn benchmark_footer_ops(c: &mut Criterion) {
    let schema = make_schema(vec!["col1", "col2"]);

    c.bench_function("footer_builder_100_groups", |b| {
        b.iter(|| {
            let mut builder = qrd_core::footer::FooterBuilder::new(schema.clone(), 100000);
            for i in 0..100 {
                builder.add_row_group_offset(32 + i * 1000);
            }
            builder.build().unwrap();
        });
    });

    c.bench_function("footer_serialize", |b| {
        let mut builder = qrd_core::footer::FooterBuilder::new(schema.clone(), 100000);
        for i in 0..100 {
            builder.add_row_group_offset(32 + i * 1000);
        }
        let footer = builder.build().unwrap();

        b.iter(|| footer.serialize().unwrap());
    });
}

criterion_group!(
    benches,
    benchmark_streaming_write,
    benchmark_auto_flush,
    benchmark_compression_detection,
    benchmark_should_compress,
    benchmark_memory_efficiency,
    benchmark_footer_ops
);
criterion_main!(benches);
