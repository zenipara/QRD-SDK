#[path = "../../../bench/common.rs"]
mod common;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::from_str;
use std::time::Instant;

fn json_to_qrd_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_to_qrd");

    for row_count in common::ROW_COUNTS {
        let telemetry = common::telemetry_dataset(row_count);
        let ai_events = common::ai_event_dataset(row_count);

        for dataset in [&telemetry, &ai_events] {
            group.throughput(Throughput::Bytes(dataset.json_bytes as u64));
            group.bench_with_input(
                BenchmarkId::new(dataset.name, row_count),
                dataset,
                |b, dataset| {
                    b.iter_custom(|iters| {
                        let started = Instant::now();
                        let dataset = black_box(dataset);
                        let mut qrd_bytes = 0usize;

                        for _ in 0..iters {
                            let mut rows = Vec::with_capacity(dataset.json_rows.len());

                            if dataset.name == "telemetry" {
                                for json_row in &dataset.json_rows {
                                    let parsed: common::TelemetryRecord =
                                        from_str(black_box(json_row))
                                            .expect("telemetry JSON must parse");
                                    rows.push(parsed.to_row());
                                }
                            } else {
                                for json_row in &dataset.json_rows {
                                    let parsed: common::AiEventRecord =
                                        from_str(black_box(json_row))
                                            .expect("AI event JSON must parse");
                                    rows.push(parsed.to_row());
                                }
                            }

                            qrd_bytes = common::write_rows(
                                &dataset.schema,
                                &rows,
                                common::DEFAULT_ROW_GROUP_SIZE,
                                common::DEFAULT_COMPRESSION_LEVEL,
                            )
                            .expect("JSON to QRD conversion should succeed");
                        }

                        let elapsed = started.elapsed();
                        common::report_metrics(
                            "json_to_qrd",
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
    targets = json_to_qrd_bench
);
criterion_main!(benches);
