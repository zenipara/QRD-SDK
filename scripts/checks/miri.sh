#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_miri() {
  log_step "Running Miri validation"

  if ! check_tool "cargo-miri"; then
    log_fail "cargo-miri not found"
    return 1
  fi

  local start_time=$(get_timestamp)

  # Run Miri on a subset of tests to check for UB
  cd "$REPO_ROOT/core/qrd-core"
  if cargo miri test --lib validation 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "Miri validation completed in ${duration}s"
    return 0
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "Miri validation failed after ${duration}s"
    return 1
  fi
}

