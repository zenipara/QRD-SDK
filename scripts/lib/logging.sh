#!/usr/bin/env bash
set -euo pipefail

GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
CYAN="\033[0;36m"
MAGENTA="\033[0;35m"
NC="\033[0m"

log_prefix() {
  printf "[%s]" "$(date +"%Y-%m-%d %H:%M:%S")"
}

log() {
  local color="$1" level="$2" message="$3"
  printf "%s %b[%s]%b %s\n" "$(log_prefix)" "$color" "$level" "$NC" "$message"
}

log_info() {
  log "$BLUE" "INFO" "$1"
}

log_warn() {
  log "$YELLOW" "WARN" "$1"
}

log_pass() {
  log "$GREEN" "PASS" "$1"
}

log_fail() {
  log "$RED" "FAIL" "$1"
}

log_step() {
  log "$CYAN" "STEP" "$1"
}

log_debug() {
  if [[ "${VERBOSE:-false}" == true ]]; then
    log "$MAGENTA" "DEBUG" "$1"
  fi
}
