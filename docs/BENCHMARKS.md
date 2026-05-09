# QRD Benchmarks

This repository uses Criterion.rs to benchmark the QRD core writer pipeline, focusing on write throughput, compression behavior, streaming append, and JSON to QRD conversion.

## Local execution

Run the full suite from the workspace root:

```bash
make benchmark
```

Run the release build and all QRD core benches explicitly:

```bash
make benchmark-release
```

Run only the JSON conversion or streaming append suites:

```bash
make benchmark-json
make benchmark-streaming
```

You can also run Criterion directly:

```bash
cd core/qrd-core
cargo bench --workspace
```

## Benchmark layout

The benchmark targets live in `core/qrd-core/benches/` and share reusable dataset helpers from `bench/common.rs`.

Current targets:

`write_benchmark` - end to end write throughput across telemetry and AI-style records.
`compression_benchmark` - compression ratio behavior for repetitive, telemetry, entropy, and AI datasets.
`streaming_benchmark` - streaming append performance with batch-driven ingestion.
`json_to_qrd_benchmark` - JSON parsing plus QRD encoding for realistic payloads.

## GitHub Actions workflow

The workflow in `.github/workflows/benchmark.yml` runs on every push and pull request on `ubuntu-latest`.

It performs these steps:

1. Installs stable Rust.
2. Caches Cargo and Criterion data.
3. Builds the workspace in release mode.
4. Runs `cargo bench --workspace`.
5. Uploads `core/qrd-core/target/criterion` and the benchmark log as artifacts.

Criterion history is cached per branch to preserve comparison compatibility between runs.

## Reading Criterion reports

Criterion writes HTML reports under `core/qrd-core/target/criterion/<bench>/report/index.html`.

The most useful signals are:

`ns/iter` - execution time per benchmark iteration.
`change` - regression or improvement compared with the stored baseline.
`throughput` - bytes or elements processed per second.

The console output also prints a summary line for each benchmark sample with:

`rows/sec`
`logical MB/sec`
`compression ratio`
`JSON size`
`QRD size`
`bytes per row`

## Reproducibility notes

The benchmark helpers use fixed seeds and deterministic data generation.

To keep results stable:

1. Run on an otherwise idle machine.
2. Keep CPU frequency scaling consistent when comparing runs.
3. Avoid background filesystem-heavy workloads.
4. Prefer the release profile used by Criterion and the GitHub Actions workflow.

The benchmark pipeline avoids filesystem bottlenecks by writing to an in-memory sink rather than temporary files.

## Best practices

When adding a new benchmark, reuse the helpers in `bench/common.rs` and keep data generation deterministic.

If you introduce a new dataset shape, prefer extending the shared helpers over duplicating setup code.

## Future extension points

This layout is ready for:

`target/criterion` history publication on GitHub Pages.
Branch-aware regression dashboards.
Competitor format benchmarks.
ARM and Raspberry Pi benchmark jobs.
WASM benchmark jobs.
