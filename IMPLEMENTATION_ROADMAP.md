# Action Items — QRD-SDK Audit Fixes

**Created:** 9 May 2026  
**Status:** 74% Complete (14/19 findings)  
**Priority:** HIGH — Blocking 1.0.0 Release

---

## Immediate Actions (This Week)

### ✅ Already Completed
- [x] Update audit.md with fix status
- [x] Remove unused tokio dependency
- [x] Add bytemuck crate for safer SIMD
- [x] Improve schema mismatch error messages
- [x] Document Nullability::Repeated enum
- [x] Clarify ECC feature flag semantics
- [x] Create AUDIT_STATUS.md as authoritative source
- [x] Create AUDIT_FIXES_SUMMARY.md

### ⏳ This Week
- [ ] Run `cargo test --all` and verify all tests pass
- [ ] Build all language bindings:
  - [ ] `cargo build -p qrd-ffi --release`
  - [ ] `cargo build -p qrd-wasm --target wasm32-unknown-unknown`
  - `pushd sdk/go && go build ./... && popd`
  - [ ] `mvn clean package` (Java)
  - [ ] `pip install -e .` (Python)
  - [ ] `npm run build` (TypeScript)
- [ ] Review and merge all changes above
- [ ] Create PR with title: "fix: address audit findings (14/19 complete)"

---

## Serious Work — Next 2 Weeks

### S2: Per-Column Encryption Keys (HIGH PRIORITY)

**Effort:** 3-4 days | **Complexity:** HIGH

**Implementation Plan:**

1. **Design** (0.5 day)
   - Finalize per-column key derivation strategy
   - Decide on key storage location (header metadata vs footer)
   - Define binary format for per-column key indices

2. **Implementation** (2 days)
   - Add `ColumnEncryptionInfo` struct to store per-column metadata
   - Implement `derive_column_key(master_key, column_name) -> Vec<u8>`
   - Update `FileWriter.flush_row_group()` to encrypt with per-column keys
   - Update `FileReader.read_row_group()` to decrypt with per-column keys
   - Update footer serialization to include column key metadata

3. **Testing** (1 day)
   - Write roundtrip tests: write with per-column keys → read with per-column keys
   - Verify columns encrypted with different keys
   - Test key derivation consistency
   - Add integration test with different passwords per operation

4. **Documentation** (0.5 day)
   - Update SPECIFICATION.md with per-column encryption section
   - Add examples in README.md
   - Document in rustdoc comments

**Files to Modify:**
- `core/qrd-core/src/encryption/mod.rs` — Add per-column key derivation
- `core/qrd-core/src/writer/mod.rs` — Per-column key handling in flush_row_group
- `core/qrd-core/src/reader/mod.rs` — Per-column key handling in read_row_group
- `core/qrd-core/src/metadata/mod.rs` — Store column key metadata
- `SPECIFICATION.md` — Document format changes

**Testing Checklist:**
- [ ] Unit test: per-column key derivation
- [ ] Integration test: roundtrip with per-column encryption
- [ ] Fuzz test: corrupt one column key, verify others decrypt correctly
- [ ] Spec validation: format matches SPECIFICATION.md

---

### S5: TypeScript Reader Implementation (MEDIUM PRIORITY)

**Effort:** 2-3 days | **Complexity:** MEDIUM

**Implementation Plan:**

1. **API Design** (0.5 day)
   ```typescript
   // Design WASM bindings
   export class QrdReader {
     static fromBytes(data: Uint8Array): QrdReader
     rowCount(): number
     columnCount(): number
     columnNames(): string[]
     readAllRows(): Row[]
     readRows(start: number, count: number): Row[]
     readColumn(name: string): any[]
     readColumnSlice(name: string, start: number, count: number): any[]
   }
   ```

2. **WASM Binding Enhancement** (0.5 day)
   - Extend `sdk/typescript-wasm/src/lib.rs` with reader exports
   - Implement `QrdMemReader` struct in WASM
   - Export to JavaScript/TypeScript

3. **TypeScript Wrapper** (1 day)
   - Implement `QrdReader` class in `sdk/typescript/src/index.ts`
   - Handle Uint8Array → WASM memory transfer
   - Implement all read methods with proper error handling
   - Add proper typing for Row objects

4. **Testing** (0.5 day)
   - Write round-trip test: create file with writer → read with reader
   - Test various column types (Int64, String, Float64, etc.)
   - Test partial reads and column filtering
   - Test with encrypted/ECC-protected files

5. **Documentation** (0.5 day)
   - Add examples to `sdk/typescript/README.md`
   - Document API in JSDoc comments
   - Add quick start guide

**Files to Modify:**
- `core/qrd-wasm/src/lib.rs` — Add reader exports
- `sdk/typescript/src/index.ts` — Implement QrdReader class
- `sdk/typescript/src/index.test.ts` — Add comprehensive tests
- `sdk/typescript/README.md` — Document reader API

**Testing Checklist:**
- [ ] Unit test: QrdReader creation and parsing
- [ ] Integration test: write + read roundtrip
- [ ] Test with various schema types
- [ ] Test error handling (invalid files, truncated data)
- [ ] Test with encryption/ECC enabled

---

## Quality Assurance

### Code Review Checklist
- [ ] All changes follow Rust/TypeScript idioms
- [ ] Error messages are clear and actionable
- [ ] Documentation is comprehensive (rustdoc, JSDoc, comments)
- [ ] Security implications reviewed
- [ ] Performance impact assessed

### Testing Checklist
- [ ] All existing tests still pass
- [ ] New tests cover new functionality (>90% coverage)
- [ ] Fuzzing tests run successfully
- [ ] Cross-language compatibility tests pass
- [ ] Performance benchmarks stable

### Documentation Checklist
- [ ] SPECIFICATION.md updated
- [ ] API documentation complete
- [ ] README examples current
- [ ] CHANGELOG.md updated with new features
- [ ] Migration guide (if needed)

---

## Sprint Planning

### Sprint 1 (This Week)
- [ ] Verify all completed fixes
- [ ] Run full test suite
- [ ] Build all language bindings
- [ ] Merge all changes to main branch

### Sprint 2 (Next 2 Weeks)
- [ ] Complete S2 (per-column encryption)
- [ ] Complete S5 (TypeScript reader)
- [ ] Full integration testing
- [ ] Security audit for new features

### Sprint 3 (Final Week)
- [ ] Polish documentation
- [ ] Performance optimization
- [ ] Release candidate testing
- [ ] Beta 1.0.0 release

---

## Success Criteria

### Launch Readiness (All boxes must be ✅)
- [x] All critical findings fixed (C1-C5)
- [ ] All serious findings fixed except S2 (per-column keys)
- [x] Encryption and ECC working end-to-end
- [ ] All language bindings functional
- [ ] Comprehensive test coverage >85%
- [ ] Documentation complete and accurate
- [ ] Performance benchmarks within targets
- [ ] Security audit passed

### Production Readiness (Target: 2-4 weeks)
- [ ] Zero critical issues in issue tracker
- [ ] All CVEs addressed
- [ ] Load testing completed
- [ ] Disaster recovery tested
- [ ] Monitoring and logging in place

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Per-column encryption adds complexity | HIGH | MEDIUM | Thorough testing, security review |
| TypeScript reader incomplete | MEDIUM | MEDIUM | Clear scope, iterative delivery |
| Performance regression | LOW | HIGH | Benchmark all changes |
| Backward compatibility break | LOW | HIGH | Version in format version field |

---

## Communication Plan

### Team Updates
- [ ] Daily standup on progress
- [ ] Weekly summary email to stakeholders
- [ ] Draft release notes as work progresses

### User Communication
- [ ] Beta release announcement
- [ ] Migration guide (if needed)
- [ ] Support documentation
- [ ] FAQ for common issues

---

## Timeline

```
Week 1: Code review + testing + PR merge
↓
Week 2: Per-column encryption implementation
↓
Week 3: TypeScript reader implementation + integration testing
↓
Week 4: Polish + release candidate
↓
1.0.0 Release Target: Mid-June 2026
```

---

**Owner:** @zenipara (Project Lead)  
**Reviewer:** @nafalfaturizki (Security)  
**QA:** @copilot-ai (Automated Testing)

Last updated: 9 May 2026
Next review: 12 May 2026
