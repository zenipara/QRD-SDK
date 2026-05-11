#!/usr/bin/env bash
set -euo pipefail

echo "Validating core functionality..."

# Run core-specific tests
cd core/qrd-core
cargo test --all-targets

echo "Core validation completed."