#[path = "../../../bench/common.rs"]
mod common;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Instant;

fn write_throughput_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_throughput");

    for row_count in common::ROW_COUNTS {
        let telemetry = common::telemetry_dataset(row_count);
        group.throughput(Throughput::Bytes(telemetry.logical_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new(telemetry.name, row_count),
            &telemetry,
            |b, dataset| {
                b.iter_custom(|iters| {
                    let started = Instant::now();
                    let dataset = black_box(dataset);
                    let mut qrd_bytes = 0usize;

                    for _ in 0..iters {
                        qrd_bytes = common::write_rows(
                            &dataset.schema,
                            &dataset.rows,
                            common::DEFAULT_ROW_GROUP_SIZE,
                            common::DEFAULT_COMPRESSION_LEVEL,
                        )
                        .expect("write benchmark should succeed");
                    }

                    let elapsed = started.elapsed();
                    common::report_metrics(
                        "write_throughput",
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

        let ai_events = common::ai_event_dataset(row_count);
        group.throughput(Throughput::Bytes(ai_events.logical_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new(ai_events.name, row_count),
            &ai_events,
            |b, dataset| {
                b.iter_custom(|iters| {
                    let started = Instant::now();
                    let dataset = black_box(dataset);
                    let mut qrd_bytes = 0usize;

                    for _ in 0..iters {
                        qrd_bytes = common::write_rows(
                            &dataset.schema,
                            &dataset.rows,
                            common::DEFAULT_ROW_GROUP_SIZE,
                            common::DEFAULT_COMPRESSION_LEVEL,
                        )
                        .expect("write benchmark should succeed");
                    }

                    let elapsed = started.elapsed();
                    common::report_metrics(
                        "write_throughput",
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

    group.finish();
}

criterion_group!(
    name = benches;
    config = common::criterion_config();
    targets = write_throughput_bench
);
criterion_main!(benches);
