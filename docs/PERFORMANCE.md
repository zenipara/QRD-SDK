# Performance

## Philosophy

QRD performance documentation prioritizes realistic workloads, transparent methodology, and well-defined caveats.

The objective is to describe how the format behaves in bounded-memory and streaming environments, not to make unverified claims.

## Measurement Methodology

Benchmarks should be derived from the core engine under reproducible conditions.

- use `cargo bench --package qrd-core`
- document the command line and dataset shape
- capture hardware details and build profile
- compare before/after for performance-sensitive changes

### Hardware Notes

Performance depends on:

- CPU architecture and available ISA extensions
- memory bandwidth and cache behavior
- storage or I/O subsystem characteristics
- compiler flags and build profile

Always reproduce benchmarks on the target platform before drawing conclusions.

## Throughput Categories

- **Write throughput**: how quickly QRD assembles and flushes row groups.
- **Read throughput**: how quickly QRD can decode a full file.
- **Partial read throughput**: how quickly QRD can read selected columns.
- **Compression throughput**: codec performance across payloads.
- **Decompression throughput**: speed to recover original data.

## Reproducibility

- keep the same CPU and build profile
- run benchmarks from the repository root
- avoid background tasks that affect I/O
- record `RUSTFLAGS`, `cargo`, and OS version

## Caveats

- results depend heavily on data shape, cardinality, and sortedness
- compression ratios vary with workload
- small files are dominated by metadata overhead
- browser/WASM results are different from native results

## Edge and Streaming Implications

QRD target workloads prioritize:

- low peak memory
- sequential ingestion
- selective read efficiency
- predictable behavior on constrained hardware
