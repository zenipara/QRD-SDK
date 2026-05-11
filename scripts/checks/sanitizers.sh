#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_sanitizers() {
  log_step "Running sanitizer validation"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  # Check if we're on Linux (sanitizers work best there)
  if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    log_warn "sanitizers not supported on $OSTYPE, skipping"
    return 0
  fi

  local start_time=$(get_timestamp)
  local failed=0

  # AddressSanitizer
  log_info "Testing AddressSanitizer"
  if RUSTFLAGS="-Zsanitizer=address" cargo test --lib --quiet 2>&1; then
    log_pass "AddressSanitizer passed"
  else
    log_fail "AddressSanitizer failed"
    ((failed++))
  fi

  # LeakSanitizer (if available)
  log_info "Testing LeakSanitizer"
  if RUSTFLAGS="-Zsanitizer=leak" cargo test --lib --quiet 2>&1; then
    log_pass "LeakSanitizer passed"
  else
    log_fail "LeakSanitizer failed"
    ((failed++))
  fi

  local duration=$(calculate_duration "$start_time")

  if [ $failed -eq 0 ]; then
    log_pass "sanitizer validation completed in ${duration}s"
    return 0
  else
    log_fail "sanitizer validation failed: $failed sanitizers failed"
    return 1
  fi
}

