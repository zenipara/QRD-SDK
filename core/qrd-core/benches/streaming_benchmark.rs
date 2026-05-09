#[path = "../../../bench/common.rs"]
mod common;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Instant;

fn streaming_append_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_append");

    for row_count in common::ROW_COUNTS {
        let telemetry = common::telemetry_dataset(row_count);
        let repetitive = common::repetitive_dataset(row_count);

        for dataset in [&telemetry, &repetitive] {
            for batch_size in common::STREAMING_BATCH_SIZES {
                group.throughput(Throughput::Elements(dataset.rows.len() as u64));
                group.bench_with_input(
                    BenchmarkId::new(format!("{}_batch", dataset.name), batch_size),
                    dataset,
                    |b, dataset| {
                        b.iter_custom(|iters| {
                            let started = Instant::now();
                            let dataset = black_box(dataset);
                            let mut qrd_bytes = 0usize;

                            for _ in 0..iters {
                                let sink = common::SharedVecWriter::new();
                                let capture = sink.clone();
                                let mut config = qrd_core::writer::StreamingWriterConfig::default();
                                config.row_group_size = common::DEFAULT_ROW_GROUP_SIZE;
                                config.compression_level = common::DEFAULT_COMPRESSION_LEVEL;

                                let mut writer = qrd_core::writer::StreamingWriter::with_config(
                                    sink,
                                    dataset.schema.clone(),
                                    config,
                                )
                                .expect("streaming writer must initialize");

                                for chunk in dataset.rows.chunks(batch_size) {
                                    for row in chunk {
                                        writer
                                            .write_row(black_box(row.clone()))
                                            .expect("streaming append should succeed");
                                    }
                                }

                                writer.finish().expect("streaming writer should finish");
                                qrd_bytes = capture.len();
                            }

                            let elapsed = started.elapsed();
                            common::report_metrics(
                                "streaming_append",
                                dataset.name,
                                dataset.rows.len() * iters as usize,
                                dataset.logical_bytes,
                                dataset.json_bytes,
                                qrd_bytes,
                                elapsed,
                            );
                            elapsed
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = common::criterion_config();
    targets = streaming_append_bench
);
criterion_main!(benches);