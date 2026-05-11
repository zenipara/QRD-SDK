#!/usr/bin/env bash
set -euo pipefail

echo "Running lint checks..."

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings

echo "Lint checks passed."