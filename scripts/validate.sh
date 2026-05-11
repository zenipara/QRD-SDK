#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/constants.sh"
source "$SCRIPT_DIR/lib/logging.sh"
source "$SCRIPT_DIR/lib/timing.sh"
source "$SCRIPT_DIR/lib/environment.sh"
source "$SCRIPT_DIR/lib/tools.sh"
source "$SCRIPT_DIR/lib/helpers.sh"
source "$SCRIPT_DIR/lib/parallel.sh"
source "$SCRIPT_DIR/lib/reporting.sh"

MODE="standard"
CHECKS=()
VERBOSE=false
REPORT_DIR="$SCRIPT_DIR/reports/latest"
OUTPUT_JUNIT="${REPORT_DIR}/junit.xml"
KEEP_ARTIFACTS=false

usage() {
  cat <<EOF
Usage: $0 [--mode=quick|standard|strict|paranoid|release] [--check=<name>] [--list] [--verbose] [--report-dir=<dir>]

Modes:
  quick      Minimal developer loop: fmt, clippy, check, smoke
  standard   Full workspace validation: tests, audit, deny, deps, docs, wasm, ffi
  strict     Full standard checks + coverage, sanitizers, feature matrix, release
  paranoid   Strict + fuzz, miri, stress, reproducibility, regression
  release    Release readiness validation

Examples:
  $0 --mode=quick
  $0 --check=clippy
  $0 --mode=strict --report-dir=scripts/reports/latest
EOF
}

register_check() {
  local name="$1"
  CHECKS+=("$name")
}

list_checks() {
  cat <<EOF
Available checks:
  fmt
  clippy
  check
  test
  nextest
  coverage
  audit
  deny
  udeps
  outdated
  deps
  licenses
  docs
  wasm
  ffi
  feature-flags
  release
  reproducible
  fuzz
  sanitizers
  miri
  stress
  regression
  panic
  benchmark
  examples
  integration
  smoke
EOF
}

resolve_mode() {
  case "$MODE" in
    quick)
      register_check fmt
      register_check clippy
      register_check check
      register_check smoke
      ;;
    standard)
      register_check fmt
      register_check clippy
      register_check check
      register_check test
      register_check audit
      register_check deny
      register_check udeps
      register_check outdated
      register_check docs
      register_check examples
      register_check integration
      register_check deps
      register_check ffi
      register_check wasm
      register_check feature-flags
      ;;
    strict)
      resolve_mode standard
      register_check nextest
      register_check coverage
      register_check sanitizers
      register_check benchmark
      register_check release
      register_check reproducible
      ;;
    paranoid)
      resolve_mode strict
      register_check fuzz
      register_check miri
      register_check stress
      register_check regression
      register_check panic
      ;;
    release)
      register_check fmt
      register_check clippy
      register_check check
      register_check test
      register_check audit
      register_check deny
      register_check feature-flags
      register_check release
      register_check coverage
      register_check fuzz
      register_check sanitizers
      register_check reproducible
      ;;
    *)
      log_fail "Unknown mode: $MODE"
      usage
      exit 1
      ;;
  esac
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --mode=*) MODE="${1#*=}" ;;
      --mode) shift; MODE="${1:-}" ;;
      --check=*) register_check "${1#*=}" ;;
      --check) shift; register_check "${1:-}" ;;
      --list) list_checks; exit 0 ;;
      --report-dir=*) REPORT_DIR="${1#*=}" ;;
      --verbose) VERBOSE=true ;;
      --keep-artifacts) KEEP_ARTIFACTS=true ;;
      -h|--help) usage; exit 0 ;;
      *) log_warn "Ignoring unknown argument: $1" ;;
    esac
    shift
  done
}

main() {
  parse_args "$@"
  log_step "Starting QRD-SDK validation pipeline"
  log_info "Mode: $MODE"
  log_info "Report directory: $REPORT_DIR"
  create_report_directories "$REPORT_DIR"
  write_environment_report "$REPORT_DIR"
  resolve_mode

  if [[ ${#CHECKS[@]} -eq 0 ]]; then
    log_warn "No checks selected, falling back to standard mode"
    MODE=standard
    resolve_mode
  fi

  local total=0
  local failed=0
  local skipped=0
  report_init "$REPORT_DIR"

  for check in "${CHECKS[@]}"; do
    if [[ -f "$SCRIPT_DIR/checks/$check.sh" ]]; then
      source "$SCRIPT_DIR/checks/$check.sh"
      local fn="check_${check}"
      if declare -F "$fn" > /dev/null; then
        total=$((total + 1))
        run_check "$check" "$fn" "$REPORT_DIR" || true
      else
        log_warn "Check implementation missing for $check"
        report_add_result "$check" "skipped" "function missing" "LOW" "" "" "" "$REPORT_DIR"
        skipped=$((skipped + 1))
      fi
    else
      log_warn "Unknown check "$check""
      report_add_result "$check" "skipped" "script missing" "LOW" "" "" "" "$REPORT_DIR"
      skipped=$((skipped + 1))
    fi
  done

  report_finalize "$REPORT_DIR"
  log_info "Validation complete: $total checks processed"
  log_info "Reports written to: $REPORT_DIR"

  if [[ $(report_count_failed "$REPORT_DIR") -gt 0 ]]; then
    log_fail "Validation failed"
    exit 1
  fi

  log_pass "Validation succeeded"
}

main "$@"
