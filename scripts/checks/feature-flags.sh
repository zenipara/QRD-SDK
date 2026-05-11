#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_feature_flags() {
  log_step "Running feature flag validation"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  local failed=0

  # Test default features
  log_info "Testing default features"
  if ! cargo check --package qrd-core 2>&1; then
    log_fail "default features check failed"
    ((failed++))
  fi

  # Test no-default-features
  log_info "Testing no-default-features"
  if ! cargo check --package qrd-core --no-default-features 2>&1; then
    log_fail "no-default-features check failed"
    ((failed++))
  fi

  # Test all-features
  log_info "Testing all-features"
  if ! cargo check --package qrd-core --all-features 2>&1; then
    log_fail "all-features check failed"
    ((failed++))
  fi

  # Test specific feature combinations
  local features=("threading" "compression" "encryption" "ecc" "wasm")
  for feature in "${features[@]}"; do
    log_info "Testing feature: $feature"
    if ! cargo check --package qrd-core --no-default-features --features "$feature" 2>&1; then
      log_fail "feature '$feature' check failed"
      ((failed++))
    fi
  done

  local duration=$(calculate_duration "$start_time")

  if [ $failed -eq 0 ]; then
    log_pass "feature flag validation completed in ${duration}s"
    return 0
  else
    log_fail "feature flag validation failed: $failed combinations failed"
    return 1
  fi
}

