
# audit.md — Repository Structure Audit

# Repository Structure Evaluation

## Strengths
Repository sudah menunjukkan pola enterprise modern:
- modular core architecture
- reusable GitHub Actions
- dedicated benchmarks
- language SDK separation
- tooling isolation

---

# 1. Root Directory Audit

## Current Condition
Root terlalu padat untuk repository enterprise.

### Current Issues
| File/Folder | Status | Recommendation |
|---|---|---|
| STATUS.md | Operational noise | Move to docs/status |
| COMPLETION_REPORT.md | Temporary report | Archive |
| IMPLEMENTATION_SUMMARY.md | Duplicate reporting | Merge |
| IMPLEMENTATION_SUMMARY_COVERAGE.md | Redundant | Merge |
| TEST_COVERAGE_DETAILED.md | Generated-style artifact | docs/reports |
| engineer.md | unclear governance | docs/internal |
| roadmap.md | valid | keep |
| specs/ | valid | keep |
| docs/archive | valid | keep |
| scripts/reports/latest | generated artifact | gitignore |

---

# 2. Core Layout Audit

## Positive
```text
core/
 ├── qrd-core
 ├── qrd-ffi
 └── qrd-wasm
```

Ini sudah benar dan scalable.

## Recommendation
Tambahkan:
```text
core/qrd-cli
core/qrd-bench
core/qrd-tools
```

untuk memisahkan:
- CLI tooling
- benchmark executable
- internal utilities

---

# 3. SDK Layout Audit

## Current
```text
sdk/
 ├── go
 ├── python
 └── typescript
```

## Gap
Belum ada:
```text
sdk/contracts/
sdk/shared/
sdk/schema/
```

### Dampak
Schema drift sangat mungkin terjadi.

---

# 4. Documentation Audit

## Problem
Dokumentasi tersebar:
- root
- docs/
- archive/
- specs/

### Enterprise Recommendation
Gunakan struktur:

```text
docs/
 ├── architecture/
 ├── sdk/
 ├── specs/
 ├── reports/
 ├── governance/
 ├── security/
 ├── benchmarks/
 └── archive/
```

---

# 5. Script Governance Audit

## Current Problem
Validation scripts tersebar di root.

### Current
```text
validate_phase2.sh
validate_phase4.sh
quick_validate.sh
measure_coverage.sh
```

### Enterprise Structure
```text
scripts/
 ├── ci/
 ├── benchmark/
 ├── validation/
 ├── release/
 ├── tooling/
 └── reporting/
```

---

# 6. CI Workflow Audit

## Positive
Reusable workflows sangat baik:
- _core-rust.yml
- _core-heavy.yml
- _core-release.yml

Ini mendekati standar platform engineering modern.

## Weakness
Workflow count mulai terlalu besar dan fragmented.

### Recommendation
Kelompokkan:
```text
.github/workflows/
 ├── ci/
 ├── release/
 ├── security/
 ├── benchmark/
 └── maintenance/
```

---

# 7. Artifact Governance Audit

## Problem
Repository menghasilkan banyak report generated:
- summary.json
- summary.txt
- reports/latest

### Risiko
- dirty git history
- merge conflict
- unstable repository state

### Recommendation
Tambahkan:
```gitignore
scripts/reports/latest/*
coverage/
artifacts/
tmp/
```

---

# 8. Enterprise Readiness Score

| Category | Score |
|---|---|
| Modularity | 8.5/10 |
| Repository Hygiene | 6/10 |
| CI Engineering | 8/10 |
| Release Engineering | 5/10 |
| SDK Governance | 5.5/10 |
| Security Governance | 6/10 |
| Documentation Governance | 5/10 |
| Scalability | 8/10 |

---

# Final Structural Recommendation

Target struktur ideal:

```text
/ci
/core
/sdk
/tools
/docs
/tests
/benchmarks
/scripts
/compliance
/examples
```

Dan root hanya berisi:
- README
- LICENSE
- Cargo.toml
- SECURITY
- CONTRIBUTING
- CHANGELOG

Semua file status/report dipindahkan ke `/docs/reports`.
