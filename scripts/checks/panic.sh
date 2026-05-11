#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_panic() {
  log_step "Running panic detection"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)

  # Run tests with panic detection
  if RUST_BACKTRACE=1 cargo test --workspace --lib --quiet 2>&1 | grep -q "panic\|thread.*panicked"; then
    local duration=$(calculate_duration "$start_time")
    log_fail "panic detected in tests after ${duration}s"
    return 1
  else
    local duration=$(calculate_duration "$start_time")
    log_pass "no panics detected in ${duration}s"
    return 0
  fi
}

