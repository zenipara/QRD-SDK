#!/usr/bin/env python3
import json
import os
import sys

if len(sys.argv) != 10:
    print("usage: reporting.py <entries> <json> <junit> <mode> <total> <passed> <failed> <skipped> <duration>", file=sys.stderr)
    sys.exit(1)

entries_path, json_path, junit_path, mode, total, passed, failed, skipped, duration = sys.argv[1:10]

total = int(total)
passed = int(passed)
failed = int(failed)
skipped = int(skipped)
duration = int(duration)

entries = []
with open(entries_path, "r", encoding="utf-8") as f:
    for line in f:
        line = line.rstrip("\n")
        if not line:
            continue
        parts = line.split("\x1f")
        if len(parts) != 8:
            continue
        name, status, reason, severity, command, output, duration_ms, timestamp = parts
        entries.append({
            "name": name,
            "status": status,
            "reason": reason,
            "severity": severity,
            "command": command,
            "output": output,
            "duration_ms": int(duration_ms or 0),
            "timestamp": timestamp,
        })

summary = {
    "mode": mode,
    "total_checks": total,
    "passed": passed,
    "failed": failed,
    "skipped": skipped,
    "duration_ms": duration,
    "generation_time": "",
    "entries": entries,
}
with open(json_path, "w", encoding="utf-8") as f:
    json.dump(summary, f, indent=2)

suite_name = "QRD-SDK Validation"
with open(junit_path, "w", encoding="utf-8") as f:
    f.write("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")
    f.write(f"<testsuite name=\"{suite_name}\" tests=\"{total}\" failures=\"{failed}\" skipped=\"{skipped}\" time=\"{duration / 1000:.3f}\">\n")
    for entry in entries:
        name = entry["name"]
        status = entry["status"]
        time_sec = entry["duration_ms"] / 1000.0
        f.write(f"  <testcase classname=\"validation\" name=\"{name}\" time=\"{time_sec:.3f}\">\n")
        if status == "failed":
            reason = entry["reason"] or "failure"
            output = entry["output"] or ""
            f.write(f"    <failure message=\"{reason}\">{output}</failure>\n")
        elif status == "skipped":
            f.write("    <skipped/>\n")
        f.write("  </testcase>\n")
    f.write("</testsuite>\n")
