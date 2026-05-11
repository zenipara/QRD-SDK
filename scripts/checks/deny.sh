#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_deny() {
  log_step "Running cargo deny"

  if ! check_tool "cargo-deny"; then
    log_fail "cargo-deny not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  if cargo deny check 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "cargo deny completed in ${duration}s"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "cargo deny failed after ${duration}s"
    return 1
  fi
}

