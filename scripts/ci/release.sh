#!/usr/bin/env bash
set -euo pipefail

echo "Running release validation..."

cargo build --release --workspace
cargo test --release --workspace

echo "Release validation passed."