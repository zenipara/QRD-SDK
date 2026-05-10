# Roadmap

## Current Focus

QRD-SDK is progressing through a sequence of engineering milestones that move the project from format definition to secure, multi-language adoption.

1. **Core format stabilization**
   - complete row group streaming write and footer semantics
   - validate schema serialization determinism
   - finalize column chunk metadata and checksum behavior

2. **Security and reliability**
   - verify parser and reader hardening against malformed inputs
   - validate optional ECC recovery and checksum failure modes
   - extend fuzz coverage for header/footer/row group parsing

3. **SDK maturity**
   - deliver production-ready bindings for Python, TypeScript/WASM, Go, Java, and C/C++
   - align bindings with the Rust core format behavior
   - provide packaging and platform guidelines

4. **Governance and documentation**
   - define release, compatibility, and deprecation policies
   - centralize docs in `docs/`
   - establish benchmark and security reporting conventions

## Phase Breakdown

### Phase 1: Core Format Stabilization

- establish file header and footer invariants
- implement stable schema ID generation
- support streaming writes with bounded memory
- document format behavior in `docs/FORMAT_SPEC.md`

### Phase 2: Security Hardening

- integrate encryption metadata and validation
- support optional Reed-Solomon ECC recovery
- audit parser boundaries and format assumptions
- document threats, fuzzing, and audit guidance

### Phase 3: Multi-language SDK delivery

- package Python, TypeScript/WASM, Go, Java, and C/C++ bindings
- provide example integrations and language-specific docs
- publish language stability status in `docs/SDKS.md`

### Phase 4: Ecosystem adoption

- establish CLI and interoperability tooling
- define compatibility and release policies
- create benchmark dashboards and reproducible workflows

### Phase 5: Enterprise readiness

- finalize release criteria and compatibility tests
- complete security review and audit closure
- ensure docs, changelog, and governance are aligned

## Release Criteria

A release is eligible when:

- core format tests and regression suites pass
- benchmark baselines are documented and reproducible
- compatibility policy is updated for any format changes
- security guidance and audit findings are addressed
- documentation reflects the current status and user-facing contract

## How to Use This Roadmap

- reference `docs/STABILITY.md` for compatibility guarantees
- reference `docs/VERSIONING.md` for versioning decisions
- update this document when release or phase scope changes
- use `docs/SDKS.md` for language binding status

## Notes

This roadmap is a living planning artifact. It is intended to guide contributors, not to substitute for issue tracking or release notes.
