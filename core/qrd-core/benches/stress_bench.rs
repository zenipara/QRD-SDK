use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use qrd_core::writer::{StreamingWriter, StreamingWriterConfig};
use qrd_core::schema::{SchemaBuilder, FieldType, Nullability};
use std::io::Cursor;

/// Create streaming schema
fn make_schema(cols: usize) -> qrd_core::schema::Schema {
    let mut builder = SchemaBuilder::new();
    for i in 0..cols {
        builder = builder
            .add_field(&format!("col{}", i), FieldType::Blob, Nullability::Required)
            .unwrap();
    }
    builder.build().unwrap()
}

/// Benchmark 1M row chunk - tests memory scalability
fn benchmark_million_rows(c: &mut Criterion) {
    c.bench_function("stream_1m_rows_single_column", |b| {
        b.iter(|| {
            let schema = make_schema(1);
            let buffer = Cursor::new(Vec::new());
            let mut config = StreamingWriterConfig::default();
            config.row_group_size = 10000; // 100 row groups

            let mut writer = StreamingWriter::with_config(buffer, schema, config).unwrap();

            for i in 0..1_000_000 {
                let val = (i as u64).to_le_bytes().to_vec();
                writer.write_row(vec![val]).unwrap();
            }

            writer.finish().unwrap();
        });
    });
}

/// Benchmark unbounded row writing - tests continuous ingestion
fn benchmark_unbounded_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("unbounded_streaming");
    group.sample_size(10); // Fewer samples for large datasets

    for row_count in [100_000, 500_000, 1_000_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(row_count),
            row_count,
            |b, &row_count| {
                b.iter(|| {
                    let schema = make_schema(3);
                    let buffer = Cursor::new(Vec::new());
                    let mut config = StreamingWriterConfig::default();
                    config.row_group_size = 5000;

                    let mut writer =
                        StreamingWriter::with_config(buffer, schema, config).unwrap();

                    for i in 0..row_count {
                        let v1 = (i as u32).to_le_bytes().to_vec();
                        let v2 = (i as u64).to_le_bytes().to_vec();
                        let v3 = format!("data_{}", i).into_bytes();
                        writer.write_row(vec![v1, v2, v3]).unwrap();
                    }

                    writer.finish().unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark row group boundary transitions - tests flushing stability
fn benchmark_row_group_transitions(c: &mut Criterion) {
    c.bench_function("rg_transitions_100k_rows", |b| {
        b.iter(|| {
            let schema = make_schema(2);
            let buffer = Cursor::new(Vec::new());
            let mut config = StreamingWriterConfig::default();
            config.row_group_size = 1000; // 100 transitions

            let mut writer = StreamingWriter::with_config(buffer, schema, config).unwrap();

            for i in 0..100_000 {
                let v1 = (i as u32).to_le_bytes().to_vec();
                let v2 = (i as u64).to_le_bytes().to_vec();
                writer.write_row(vec![v1, v2]).unwrap();
            }

            writer.finish().unwrap();
        });
    });
}

/// Benchmark memory efficiency - verify buffer reuse
fn benchmark_memory_efficiency(c: &mut Criterion) {
    c.bench_function("memory_efficiency_500k_rows", |b| {
        b.iter(|| {
            let schema = make_schema(4);
            let buffer = Cursor::new(Vec::new());
            let mut config = StreamingWriterConfig::default();
            config.row_group_size = 5000;

            let mut writer = StreamingWriter::with_config(buffer, schema, config).unwrap();

            for i in 0..500_000 {
                let v1 = (i as u32).to_le_bytes().to_vec();
                let v2 = (i as u64).to_le_bytes().to_vec();
                let v3 = (i as u32).wrapping_mul(3).to_le_bytes().to_vec();
                let v4 = format!("val_{}", i).into_bytes();
                writer.write_row(vec![v1, v2, v3, v4]).unwrap();
            }

            let (cached, cap) = writer.buffer_pool_stats();
            println!("  Buffer pool: {} buffers, {} bytes", cached, cap);

            writer.finish().unwrap();
        });
    });
}

/// Test many small rows - stress allocation patterns
fn benchmark_many_small_rows(c: &mut Criterion) {
    c.bench_function("many_small_rows_2m", |b| {
        b.iter(|| {
            let schema = make_schema(1);
            let buffer = Cursor::new(Vec::new());
            let mut config = StreamingWriterConfig::default();
            config.row_group_size = 1000;

            let mut writer = StreamingWriter::with_config(buffer, schema, config).unwrap();

            for i in 0..2_000_000 {
                // Very small rows
                let val = vec![(i % 256) as u8];
                writer.write_row(vec![val]).unwrap();
            }

            writer.finish().unwrap();
        });
    });
}

/// Test variable-size rows - allocation variability
fn benchmark_variable_row_sizes(c: &mut Criterion) {
    c.bench_function("variable_row_sizes_500k", |b| {
        b.iter(|| {
            let schema = make_schema(2);
            let buffer = Cursor::new(Vec::new());
            let mut config = StreamingWriterConfig::default();
            config.row_group_size = 10000;

            let mut writer = StreamingWriter::with_config(buffer, schema, config).unwrap();

            for i in 0..500_000 {
                // Variable size: 10-1000 bytes
                let size = ((i * 7) % 990) + 10;
                let v1 = vec![((i % 256) as u8); size];
                let v2 = (i as u64).to_le_bytes().to_vec();
                writer.write_row(vec![v1, v2]).unwrap();
            }

            writer.finish().unwrap();
        });
    });
}

/// Test wide schemas - many columns
fn benchmark_wide_schema(c: &mut Criterion) {
    c.bench_function("wide_schema_20cols_100k", |b| {
        b.iter(|| {
            let schema = make_schema(20); // 20 columns
            let buffer = Cursor::new(Vec::new());
            let mut config = StreamingWriterConfig::default();
            config.row_group_size = 5000;

            let mut writer = StreamingWriter::with_config(buffer, schema, config).unwrap();

            for i in 0..100_000 {
                let mut row = Vec::with_capacity(20);
                for j in 0..20 {
                    row.push(((i + j as u32) as u64).to_le_bytes().to_vec());
                }
                writer.write_row(row).unwrap();
            }

            writer.finish().unwrap();
        });
    });
}

criterion_group!(
    benches,
    benchmark_million_rows,
    benchmark_unbounded_streaming,
    benchmark_row_group_transitions,
    benchmark_memory_efficiency,
    benchmark_many_small_rows,
    benchmark_variable_row_sizes,
    benchmark_wide_schema
);
criterion_main!(benches);
