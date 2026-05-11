#!/usr/bin/env bash
set -euo pipefail

create_report_directories() {
  local base_dir="$1"
  local report_root
  report_root="$(dirname "$base_dir")"
  mkdir -p "$base_dir"
  mkdir -p "$report_root/archive"
  mkdir -p "$report_root/html"
  mkdir -p "$report_root/json"
  mkdir -p "$report_root/junit"
}

run_command() {
  local label="$1"
  shift
  local start
  local elapsed
  local status=0
  start=$(timer_start)
  log_step "Running: $label"
  if "$@"; then
    status=0
    elapsed=$(timer_elapsed_ms "$start")
    log_pass "$label completed in $(format_duration "$elapsed")"
  else
    status=$?
    elapsed=$(timer_elapsed_ms "$start")
    log_fail "$label failed after $(format_duration "$elapsed")"
  fi
  echo "$status"
}

run_check() {
  local check_name="$1"
  local function_name="$2"
  local report_dir="$3"
  local start
  local duration
  local status
  local tmp_file

  tmp_file=$(mktemp)
  start=$(timer_start)

  log_step "Executing check: $check_name"
  if "$function_name" >"$tmp_file" 2>&1; then
    status=0
    log_pass "$check_name passed"
  else
    status=$?
    log_fail "$check_name failed"
  fi

  duration=$(timer_elapsed_ms "$start")
  local output
  output=$(sed 's/"/\"/g' "$tmp_file" | tr '\n' ' ' | sed -E 's/  +/ /g' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
  rm -f "$tmp_file"

  local result="passed"
  local reason=""
  local severity="$SEVERITY_HIGH"
  if [[ $status -ne 0 ]]; then
    result="failed"
    reason="command failed"
  fi

  report_add_result "$check_name" "$result" "$reason" "$severity" "$function_name" "$output" "$duration" "$report_dir"
  return "$status"
}

safe_tempdir() {
  mktemp -d 2>/dev/null || mktemp -d -t qrd-sdk-validate
}
