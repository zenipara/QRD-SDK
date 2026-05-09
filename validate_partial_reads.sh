#!/bin/bash

# QRD v1.1.0 Partial Reads Validation Script
# Validates all roadmap items for Partial Reads release

set -e

echo "QRD v1.1.0 Partial Reads Validation"
echo "===================================="
echo

cd "$(dirname "$0")"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "core/qrd-core" ]; then
    echo "Error: Run this script from the QRD-SDK root directory"
    exit 1
fi

echo "✓ Directory structure validated"

# Build the project
echo
echo "Building qrd-core..."
cd core/qrd-core
cargo build --release
echo "✓ Build successful"

# Run all tests including new partial reads tests
echo
echo "Running all tests..."
cargo test --release
echo "✓ All tests passed"

# Run partial reads tests specifically
echo
echo "Running partial reads tests..."
cargo test partial_reads_test --release
echo "✓ Partial reads tests passed"

# Run roundtrip tests to ensure compatibility
echo
echo "Running roundtrip tests..."
cargo test roundtrip_test --release
echo "✓ Roundtrip tests passed"

# Test examples
echo
echo "Testing examples..."

echo "  Testing basic examples..."
cargo run --example basic_writer --release > /dev/null
cargo run --example basic_reader --release > /dev/null
echo "  ✓ Basic examples work"

echo "  Testing partial_reads example..."
cargo run --example partial_reads --release > /dev/null
echo "  ✓ Partial reads example works"

echo "  Testing memory_profiling example..."
cargo run --example memory_profiling --release > /dev/null
echo "  ✓ Memory profiling example works"

echo "  Testing encoding_showcase example..."
cargo run --example encoding_showcase --release > /dev/null
echo "  ✓ Encoding showcase example works"

# Generate and validate golden test vectors
echo
echo "Testing golden test vectors..."
cargo run --bin generate_test_vectors --release > /dev/null
cargo run --bin validate_test_vectors --release > /dev/null
echo "✓ Golden test vectors work"

# Test metadata functionality
echo
echo "Testing metadata and statistics collection..."
cargo test test_column_statistics_collection --release
cargo test test_query_pushdown_optimization --release
cargo test test_metadata_index_functionality --release
echo "✓ Metadata functionality validated"

# Run benchmarks to ensure performance
echo
echo "Running performance benchmarks..."
cargo bench -- --test > /dev/null 2>&1
echo "✓ Benchmarks completed"

echo
echo "🎉 QRD v1.1.0 Partial Reads Validation Complete!"
echo
echo "All roadmap items validated:"
echo "  ✓ Footer-based column-selective reads"
echo "  ✓ Column statistics collection"
echo "  ✓ Query pushdown optimization"
echo "  ✓ Metadata indexing"
echo
echo "Key features validated:"
echo "  • Column statistics automatically collected during writing"
echo "  • Query pushdown skips irrelevant row groups"
echo "  • Metadata index enables fast column lookups"
echo "  • Partial readers support selective column access"
echo "  • Backward compatibility maintained"
echo
echo "Ready for Partial Reads release!"