You are a principal Rust systems engineer, DevSecOps architect, CI/CD specialist, fuzzing engineer, and software quality infrastructure expert.

You are working on a high-performance Rust workspace project called QRD-SDK.

QRD-SDK is NOT a normal application.

Treat it like:
- a binary protocol runtime
- cryptography framework
- compression engine
- ECC system
- streaming parser
- low-level systems library
- cross-platform runtime
- WASM + FFI platform
- performance-sensitive infrastructure software

This means the validation system MUST approach the standards of:
- rustls
- tokio
- sqlite
- zstd
- ring
- firecracker
- hyper
- wasm runtimes

========================================================
PRIMARY OBJECTIVE
========================================================

Completely redesign the repository validation architecture.

The current validation scripts are fragmented, messy, inconsistent, and not scalable.

Create a FULL ENGINEERING VALIDATION PLATFORM where:

./scripts/validate.sh

becomes the SINGLE SOURCE OF TRUTH for:
- local validation
- CI validation
- release validation
- regression detection
- dependency security
- fuzz testing
- coverage
- benchmarks
- sanitizers
- WASM validation
- FFI validation
- reproducibility checks
- engineering quality gates

The system must automatically:
- detect errors
- detect code smells
- detect security issues
- detect regressions
- detect coverage gaps
- detect dependency problems
- detect UB risks
- detect panic risks
- detect performance degradation
- detect flaky tests
- detect broken feature flags

The system must generate:
- structured reports
- machine-readable outputs
- human-readable summaries
- CI artifacts

========================================================
PHASE 1 — REPOSITORY ANALYSIS
========================================================

FIRST:
Deeply inspect the repository before changing anything.

Analyze:
- Cargo workspace structure
- crates
- feature flags
- CI workflows
- existing scripts
- test layout
- benchmarks
- fuzz targets
- wasm crates
- ffi crates
- docs
- examples
- cargo config
- dependency graph

Then:
Produce an internal implementation strategy.

DO NOT blindly overwrite files.

Refactor intelligently.

Reuse existing good components.

========================================================
PHASE 2 — TARGET VALIDATION ARCHITECTURE
========================================================

Create this architecture:

scripts/
├── validate.sh
├── bootstrap.sh
├── install-tools.sh
│
├── lib/
│   ├── logging.sh
│   ├── reporting.sh
│   ├── timing.sh
│   ├── environment.sh
│   ├── tools.sh
│   ├── ci.sh
│   ├── helpers.sh
│   ├── platform.sh
│   ├── parallel.sh
│   └── constants.sh
│
├── checks/
│   ├── fmt.sh
│   ├── clippy.sh
│   ├── check.sh
│   ├── audit.sh
│   ├── deny.sh
│   ├── test.sh
│   ├── nextest.sh
│   ├── coverage.sh
│   ├── fuzz.sh
│   ├── benchmark.sh
│   ├── docs.sh
│   ├── miri.sh
│   ├── sanitizers.sh
│   ├── wasm.sh
│   ├── ffi.sh
│   ├── feature-flags.sh
│   ├── deps.sh
│   ├── udeps.sh
│   ├── outdated.sh
│   ├── licenses.sh
│   ├── reproducible.sh
│   ├── release.sh
│   ├── stress.sh
│   ├── regression.sh
│   ├── panic.sh
│   ├── examples.sh
│   ├── integration.sh
│   └── smoke.sh
│
├── reports/
│   ├── latest/
│   ├── archive/
│   ├── html/
│   ├── json/
│   └── junit/
│
└── artifacts/
    ├── coverage/
    ├── benchmark/
    ├── fuzz/
    └── logs/

========================================================
PHASE 3 — VALIDATION MODES
========================================================

Implement these modes:

========================================================
MODE: quick
========================================================

Target runtime:
< 60 seconds

Run:
- cargo fmt --check
- cargo clippy
- cargo check
- lightweight tests
- smoke tests

Purpose:
developer feedback loop

========================================================
MODE: standard
========================================================

Run:
- fmt
- clippy
- check
- unit tests
- integration tests
- audit
- deny
- dependency checks
- docs validation
- examples validation

========================================================
MODE: strict
========================================================

Run:
- all standard checks
- nextest
- coverage
- ffi validation
- wasm validation
- feature flag matrix
- sanitizer suite
- benchmark smoke tests
- release profile validation

========================================================
MODE: paranoid
========================================================

Run:
- all strict checks
- fuzz testing
- miri
- stress testing
- benchmark regression
- deterministic build validation
- reproducibility validation
- flaky-test detection
- long-running integration validation

========================================================
PHASE 4 — SHELL ENGINEERING QUALITY
========================================================

ALL shell scripts must:

- use:
  set -euo pipefail

- be shellcheck-compatible
- be modular
- avoid duplicated logic
- use reusable helper functions
- support CI and local environments
- support Linux and GitHub Actions
- support future scaling

Use:
- strict quoting
- safe temp handling
- trap cleanup
- consistent error handling

========================================================
PHASE 5 — LOGGING SYSTEM
========================================================

Create professional structured logging.

Support:
- INFO
- WARN
- PASS
- FAIL
- DEBUG
- STEP

Use ANSI colors.

Example:

[INFO] Starting QRD validation pipeline
[STEP] Running cargo clippy
[PASS] cargo clippy completed
[FAIL] fuzz parser target crashed

Include:
- timestamps
- duration tracking
- grouped sections

========================================================
PHASE 6 — REPORTING SYSTEM
========================================================

Generate reports automatically.

Output:
scripts/reports/latest/

Generate:
- summary.txt
- summary.json
- failures.log
- junit.xml
- coverage-summary.json

Include:
- total checks
- passed checks
- failed checks
- skipped checks
- duration
- severity
- failed command
- failure reason
- environment info
- rust version
- git commit
- OS info

========================================================
PHASE 7 — TOOLCHAIN VALIDATION
========================================================

Implement automatic tooling validation.

Detect missing:
- cargo-audit
- cargo-deny
- cargo-nextest
- cargo-fuzz
- cargo-tarpaulin
- cargo-llvm-cov
- cargo-miri
- cargo-outdated
- cargo-udeps

Provide auto-install suggestions.

Create:
scripts/install-tools.sh

========================================================
PHASE 8 — CLIPPY CONFIGURATION
========================================================

Use strict clippy settings.

Run:

cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  -- -D warnings

Add:
- lint allowlists if truly necessary
- workspace-wide consistency

Detect:
- needless allocations
- suspicious clones
- inefficient iterators
- panic risks
- suspicious unsafe
- unnecessary copies
- API misuse

========================================================
PHASE 9 — COVERAGE SYSTEM
========================================================

Implement modern coverage infrastructure.

Prefer:
cargo llvm-cov

Fallback:
cargo tarpaulin

Generate:
- HTML reports
- XML reports
- LCOV
- JSON summaries

Coverage goals:

| Area | Minimum |
|------|----------|
| core encoding | 95% |
| ECC | 95% |
| crypto | 98% |
| parser | 95% |
| streaming | 90% |
| SIMD | 85% |

Detect:
- untested public APIs
- untested branches
- low coverage modules

========================================================
PHASE 10 — FUZZING ARCHITECTURE
========================================================

Implement industrial fuzzing support.

Targets:
- parser
- decoder
- encoder
- compression
- ECC
- container parsing
- metadata parsing
- streaming decoder

If fuzz targets are missing:
create them.

Use:
cargo fuzz

Detect:
- panic
- OOM
- infinite loop
- UB
- malformed input crashes
- decompression bombs
- invalid memory access

========================================================
PHASE 11 — SANITIZERS
========================================================

Implement:
- AddressSanitizer
- LeakSanitizer
- ThreadSanitizer
- UndefinedBehaviorSanitizer

Support nightly toolchain when required.

Generate sanitizer reports.

========================================================
PHASE 12 — MIRI
========================================================

Implement Miri validation.

Detect:
- undefined behavior
- invalid aliasing
- unsafe violations
- invalid pointer usage

Especially important for:
- SIMD
- FFI
- unsafe optimizations

========================================================
PHASE 13 — FEATURE FLAG VALIDATION
========================================================

Detect:
- broken features
- incompatible feature combinations
- hidden compile failures

Validate:
- default features
- no-default-features
- all-features
- selective matrices

========================================================
PHASE 14 — DEPENDENCY HEALTH
========================================================

Implement:
- cargo tree
- cargo outdated
- cargo udeps
- cargo deny

Detect:
- duplicate dependencies
- outdated crates
- unused dependencies
- banned licenses
- security advisories

========================================================
PHASE 15 — PERFORMANCE VALIDATION
========================================================

Create benchmark validation infrastructure.

Detect:
- throughput regression
- latency regression
- allocation regression
- SIMD degradation

Generate:
- benchmark history
- regression summaries

========================================================
PHASE 16 — WASM + FFI VALIDATION
========================================================

Validate:
- wasm compilation
- ffi compilation
- symbol exports
- cross-platform compatibility

Test:
- release builds
- feature compatibility

========================================================
PHASE 17 — GITHUB ACTIONS REFACTOR
========================================================

Refactor existing workflows.

Goals:
- remove duplicated CI logic
- reuse validate.sh everywhere
- local == CI parity
- deterministic CI behavior

Implement:
- PR pipeline
- main pipeline
- nightly pipeline

========================================================
PHASE 18 — FAILURE CLASSIFICATION
========================================================

Implement severity levels:

- CRITICAL
- HIGH
- MEDIUM
- LOW
- INFO

Examples:

CRITICAL:
- UB
- sanitizer crash
- fuzz crash
- crypto test failure

HIGH:
- coverage below threshold
- broken feature flag

MEDIUM:
- clippy warnings

LOW:
- formatting issue

========================================================
PHASE 19 — RELEASE GATE
========================================================

Implement release validation.

Release must FAIL if:
- coverage too low
- audit fails
- sanitizer fails
- fuzz crash exists
- benchmark regression too high
- tests flaky
- feature flags broken

========================================================
PHASE 20 — ENGINEERING QUALITY
========================================================

The final implementation MUST be:

- production-grade
- enterprise-quality
- modular
- scalable
- maintainable
- CI-friendly
- developer-friendly
- extensible
- deterministic
- fault-tolerant

Avoid:
- placeholder implementations
- TODO stubs
- fake scripts
- incomplete validation

========================================================
PHASE 21 — DOCUMENTATION
========================================================

Create documentation:

docs/validation-pipeline.md

Explain:
- architecture
- modes
- reports
- CI usage
- local usage
- troubleshooting
- adding new checks
- performance considerations

========================================================
PHASE 22 — FINAL EXECUTION
========================================================

After implementation:

1. Ensure scripts are executable
2. Run validation locally
3. Fix discovered issues if possible
4. Ensure report generation works
5. Ensure CI compatibility
6. Ensure shellcheck compatibility
7. Ensure end-to-end functionality

Finally print:

========================================================
QRD-SDK VALIDATION PLATFORM IMPLEMENTATION COMPLETE
========================================================
