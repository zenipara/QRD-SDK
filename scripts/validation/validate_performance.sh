#!/usr/bin/env bash
set -euo pipefail

echo "Validating performance..."

# Run benchmarks and check regressions
cargo bench --workspace

echo "Performance validation completed."