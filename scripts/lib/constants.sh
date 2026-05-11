#!/usr/bin/env bash
set -euo pipefail

LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$LIB_DIR/../.." && pwd)"
DEFAULT_REPORT_DIR="$ROOT_DIR/scripts/reports/latest"
REPORT_ARCHIVE_DIR="$ROOT_DIR/scripts/reports/archive"
REPORT_HTML_DIR="$ROOT_DIR/scripts/reports/html"
REPORT_JSON_DIR="$ROOT_DIR/scripts/reports/json"
REPORT_JUNIT_DIR="$ROOT_DIR/scripts/reports/junit"
ARTIFACTS_DIR="$ROOT_DIR/scripts/artifacts"

SEVERITY_CRITICAL="CRITICAL"
SEVERITY_HIGH="HIGH"
SEVERITY_MEDIUM="MEDIUM"
SEVERITY_LOW="LOW"
SEVERITY_INFO="INFO"
