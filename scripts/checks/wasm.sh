#!/usr/bin/env bash
set -euo pipefail

CHECK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$CHECK_DIR/../lib/constants.sh"
source "$CHECK_DIR/../lib/logging.sh"
source "$CHECK_DIR/../lib/timing.sh"
source "$CHECK_DIR/../lib/environment.sh"
source "$CHECK_DIR/../lib/tools.sh"
source "$CHECK_DIR/../lib/helpers.sh"

check_wasm() {
  log_step "Running WASM validation"

  if ! command -v cargo &> /dev/null; then
    log_fail "cargo not found"
    return 1
  fi

  # Check if wasm-pack is available
  if ! command -v wasm-pack &> /dev/null; then
    log_warn "wasm-pack not found, skipping WASM validation"
    return 0
  fi

  local start_time=$(get_timestamp)

  # Build WASM package
  cd "$REPO_ROOT/core/qrd-wasm"
  if wasm-pack build --target web --out-dir pkg 2>&1; then
    local duration=$(calculate_duration "$start_time")
    log_pass "WASM build completed in ${duration}s"

    # Verify package contents
    if [ -f "pkg/qrd_wasm.js" ] && [ -f "pkg/qrd_wasm_bg.wasm" ]; then
      log_info "WASM package files verified"
      return 0
    else
      log_fail "WASM package files missing"
      return 1
    fi
  else
    local duration=$(calculate_duration "$start_time")
    log_fail "WASM build failed after ${duration}s"
    return 1
  fi
}

