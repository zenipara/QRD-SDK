#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_nextest() {
  log_step "Running cargo nextest"

  if ! check_tool "cargo-nextest"; then
    log_fail "cargo-nextest not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  if cargo nextest run --workspace --all-features 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "cargo nextest completed in ${duration}s"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "cargo nextest failed after ${duration}s"
    return 1
  fi
}

