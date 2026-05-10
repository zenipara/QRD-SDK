# Benchmark Dashboard

## Purpose

The benchmark dashboard captures the long-term performance profile of QRD across representative workloads and runtime targets.

It is intended to support:

- release-quality performance tracking
- regression detection for format or implementation changes
- edge and WASM performance guidance
- empirical comparison across codec and read/write modes

## Recommended Metrics

- write throughput (GB/s)
- read throughput (GB/s)
- partial read throughput (GB/s)
- compression ratio
- ECC encode/decode overhead
- memory usage per row group
- WASM runtime latency
- end-to-end latency for selected workloads

## Workload Categories

- synthetic integer streams
- low-cardinality telemetry
- mixed numeric / string analytics
- large text and binary columns
- browser/WASM partial read cases

## Reporting Guidelines

Each dashboard entry should include:

- benchmark command
- hardware and OS details
- build profile and toolchain
- dataset description and cardinality
- selected row group size
- observed metrics and variance
- notes on configuration differences

## Example Dashboard Entry

| Metric | Workload | Value | Notes |
|---|---|---|---|
| Write throughput | 1KB telemetry rows | 3.2 GB/s | ZSTD level 3, row group 10k |
| Read throughput | 100MB dense dataset | 4.8 GB/s | full scan |
| Partial read | 3 of 20 columns | 9.1 GB/s | skip unrelated columns |
| Compression ratio | structured telemetry | 2.8× | ZSTD adaptive |

## Reproducibility

- run benchmarks from the repository root
- use release mode builds
- record CPU model, RAM, OS, and toolchain versions
- avoid background workloads that bias I/O or CPU
- include both native and WASM results when relevant

## Dashboard Practices

- update the dashboard whenever benchmark-relevant changes are merged
- treat benchmark numbers as design reference, not absolute guarantees
- avoid optimistic extrapolation beyond the measured workload
