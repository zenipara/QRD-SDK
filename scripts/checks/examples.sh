#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_examples() {
  log_step "Running examples validation"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  local start_time=$(get_timestamp)
  local failed=0

  cd "$REPO_ROOT/core/qrd-core"

  # Get list of examples
  local examples
  mapfile -t examples < <(find examples -name "*.rs" -exec basename {} \; | sed 's/\.rs$//')

  for example in "${examples[@]}"; do
    log_info "Testing example: $example"
    if ! timeout 30 cargo run --example "$example" --quiet 2>&1; then
      log_fail "example '$example' failed"
      ((failed++))
    fi
  done

  local duration=$(calculate_duration "$start_time")

  if [ $failed -eq 0 ]; then
    log_pass "examples validation completed in ${duration}s"
    return 0
  else
    log_fail "examples validation failed: $failed examples failed"
    return 1
  fi
}

