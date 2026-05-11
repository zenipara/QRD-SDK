#!/usr/bin/env bash
set -euo pipefail

echo "Running bootstrap checks..."

# Install required tools if not present
./scripts/install-tools.sh

# Bootstrap workspace
cargo check --workspace

echo "Bootstrap completed successfully."