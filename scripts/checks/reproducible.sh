#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_reproducible() {
  log_step "Running reproducible build validation"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  local temp_dir
  temp_dir=$(mktemp -d)

  # Build once
  log_info "Building first instance"
  if ! cargo build --release --package qrd-core 2>&1; then
    log_fail "first build failed"
    rm -rf "$temp_dir"
    return 1
  fi

  # Copy the binary
  cp "target/release/libqrd_core.rlib" "$temp_dir/first.rlib" 2>/dev/null || cp "target/release/qrd-core" "$temp_dir/first" 2>/dev/null || true

  # Clean and rebuild
  log_info "Building second instance"
  cargo clean
  if ! cargo build --release --package qrd-core 2>&1; then
    log_fail "second build failed"
    rm -rf "$temp_dir"
    return 1
  fi

  # Compare binaries
  local second_file
  if [ -f "target/release/libqrd_core.rlib" ]; then
    second_file="target/release/libqrd_core.rlib"
  else
    second_file="target/release/qrd-core"
  fi

  if cmp "$temp_dir/first" "$second_file" 2>/dev/null; then
    local duration=$(calculate_duration "$start_time")
    log_pass "reproducible build validation completed in ${duration}s"
    rm -rf "$temp_dir"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "builds are not reproducible"
    rm -rf "$temp_dir"
    return 1
  fi
}

