# Benchmarks

## Current Baseline

Benchmarks are implemented in `core/qrd-core/benches/`.

### Core benchmark commands

```bash
cargo bench --package qrd-core
```

For targeted benchmark runs:

```bash
cargo bench --package qrd-core -- --nocapture
```

## Example Results

| Operation | Example Workload | Expected Baseline |
|---|---|---|
| Write throughput | 1KB row analytical stream | 1–5 GB/s |
| Full read throughput | 100MB dense dataset | 2–10 GB/s |
| Partial read throughput | 10% columns selected | 5–20 GB/s |
| ZSTD compression | integer/string columns | 1.5×–4× ratio |
| LZ4 compression | low-latency streams | lightweight compression overhead |

These values are representative of the engine design and should be reproduced on target hardware.

## Benchmark Philosophy

- report workloads clearly
- avoid synthetic claims
- document hardware and build environment
- include before/after comparisons for changes
- prefer real columnar data over random bytes

## Reporting

Each benchmark change should include:

- benchmark source or modified benchmark file
- command line used to run it
- hardware details
- interpretation of result changes

## Reproducibility Checklist

- use cargo bench from repository root
- use release mode where appropriate
- document OS, CPU, compiler, and toolchain versions
- do not rely on unpublished crates or local hacks

## Benchmark Caveats

- single-core results may differ from multi-core results
- archive configuration differs from stream configuration
- browser/WASM performance differs from native Rust
