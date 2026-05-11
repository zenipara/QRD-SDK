#!/usr/bin/env bash
set -euo pipefail

echo "Running unit and integration tests..."

cargo test --workspace --all-targets

echo "Tests completed successfully."