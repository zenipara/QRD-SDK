#[path = "../../../bench/common.rs"]
mod common;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Instant;

fn compression_ratio_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_ratio");

    for row_count in common::ROW_COUNTS {
        let telemetry = common::telemetry_dataset(row_count);
        let repetitive = common::repetitive_dataset(row_count);
        let entropy = common::entropy_dataset(row_count);
        let ai_events = common::ai_event_dataset(row_count);

        for dataset in [&telemetry, &repetitive, &entropy, &ai_events] {
            group.throughput(Throughput::Bytes(dataset.logical_bytes as u64));
            group.bench_with_input(
                BenchmarkId::new(dataset.name, row_count),
                dataset,
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
                            .expect("compression benchmark should succeed");
                        }

                        let elapsed = started.elapsed();
                        common::report_metrics(
                            "compression_ratio",
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

    group.finish();
}

criterion_group!(
    name = benches;
    config = common::criterion_config();
    targets = compression_ratio_bench
);
criterion_main!(benches);
