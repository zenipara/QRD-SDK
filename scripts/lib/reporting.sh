#!/usr/bin/env bash
set -euo pipefail

REPORTING_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_ENTRIES=()
FIELD_SEPARATOR=$'\x1F'

report_init() {
  local report_dir="$1"
  mkdir -p "$report_dir"
  REPORT_ENTRIES=()
  : > "$report_dir/summary.txt"
  : > "$report_dir/failures.log"
  : > "$report_dir/summary.json"
  : > "$report_dir/junit.xml"
}

report_add_result() {
  local name="$1"
  local status="$2"
  local reason="$3"
  local severity="$4"
  local command="$5"
  local output="$6"
  local duration_ms="$7"
  local report_dir="$8"
  local timestamp

  timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  output="${output//$FIELD_SEPARATOR/ }"
  REPORT_ENTRIES+=("$name$FIELD_SEPARATOR$status$FIELD_SEPARATOR$reason$FIELD_SEPARATOR$severity$FIELD_SEPARATOR$command$FIELD_SEPARATOR$output$FIELD_SEPARATOR$duration_ms$FIELD_SEPARATOR$timestamp")

  if [[ "$status" != "passed" ]]; then
    printf "%s | %s | %s | %s\n" "$timestamp" "$name" "$status" "$reason" >> "$report_dir/failures.log"
  fi
}

report_count_failed() {
  local report_dir="$1"
  grep -cE "\bfailed\b|\berror\b" "$report_dir/failures.log" || true
}

report_finalize() {
  local report_dir="$1"
  local summary_file="$report_dir/summary.txt"
  local json_file="$report_dir/summary.json"
  local junit_file="$report_dir/junit.xml"
  local total=${#REPORT_ENTRIES[@]}
  local passed=0
  local failed=0
  local skipped=0
  local duration=0

  for entry in "${REPORT_ENTRIES[@]}"; do
    IFS="$FIELD_SEPARATOR" read -r name status reason severity command output duration_ms timestamp <<< "$entry"
    duration=$((duration + duration_ms))
    case "$status" in
      passed) passed=$((passed + 1)) ;;
      skipped) skipped=$((skipped + 1)) ;;
      *) failed=$((failed + 1)) ;;
    esac
  done

  printf $'mode: %s\ntotal_checks: %s\npassed: %s\nfailed: %s\nskipped: %s\nduration_ms: %s\ngeneration_time: %s\n' \
    "$MODE" "$total" "$passed" "$failed" "$skipped" "$duration" "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" > "$summary_file"

  local entry_file
  entry_file="$(mktemp)"
  printf '%s\n' "${REPORT_ENTRIES[@]}" > "$entry_file"
  python3 "$REPORTING_DIR/reporting.py" "$entry_file" "$json_file" "$junit_file" "$MODE" "$total" "$passed" "$failed" "$skipped" "$duration"
  rm -f "$entry_file"
}