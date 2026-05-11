#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_deps() {
  log_step "Running dependency analysis"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)

  # Check for duplicate dependencies
  log_info "Checking for duplicate dependencies"
  if cargo tree --workspace --duplicates 2>&1 | grep -q "duplicate"; then
    log_fail "duplicate dependencies found"
    return 1
  fi

  # Check dependency tree
  if cargo tree --workspace 2>&1; then
    log_pass "dependency analysis completed"
  else
    log_fail "dependency analysis failed"
    return 1
  fi

  local duration=$(calculate_duration "$start_time")
  log_pass "dependency analysis completed in ${duration}s"
  return 0
}

