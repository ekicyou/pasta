# Research and Discovery Log

## Discovery Summary

### Feature Classification
**Type**: Refactoring - Code cleanup and module reorganization  
**Scope**: Straightforward (light discovery)

This specification focuses on **deleting legacy code** after a successful migration to new parser2/transpiler2 modules. The architecture itself does not change; the discovery process is minimal since:
- All legacy code is marked for deletion (no forward-compatibility concerns)
- Target modules (parser2/transpiler2) already exist and are proven
- Module renaming is a mechanical refactoring (parser2 → parser)
- Registry module is already independent and requires no changes

### Discovery Findings
1. **Legacy Code Status**: Confirmed safe to delete - all runtime references have migrated to parser2/transpiler2
2. **Public API Impact**: `src/lib.rs` re-exports 13 types from old parser module; all must be removed
3. **Test Coverage**: 21 test files depend on legacy modules; complete deletion is safe (tested via gap-analysis)
4. **Registry Module**: Already established in `src/registry/` with stable public API; no migration needed
5. **Module Renaming Strategy**: Use intermediate names (parser_new, transpiler_new) to avoid conflicts during the transition

### Design Constraints
- No changes to API contracts during deletion phase
- All deletions must be completed before module renaming phase begins
- Module renaming must be followed by complete source/test rebuild verification

---

## Research Topics

### Topic 1: Legacy Module Dependency Mapping
**Research Question**: Which files have hardcoded references to parser/transpiler modules?

**Investigation**:
- Reviewed `.kiro/specs/legacy-parser-transpiler-cleanup/gap-analysis.md` (lines 1-100)
- Source code layer: 4 files with references (lib.rs, cache.rs, runtime/words.rs, stdlib/mod.rs)
- Test code layer: 22 files with references (21 to delete + 1 to modify)

**Findings**:
- ✅ All references are explicit `use` statements and module declarations
- ✅ No transitive dependencies (legacy modules do not import each other)
- ✅ Registry module is already independent (safe to preserve)

**Implications for Design**:
- Deletion order: src/parser → src/transpiler → test files → update source files
- No circular dependency risks

---

### Topic 2: Module Renaming Safety
**Research Question**: Is it safe to rename parser2 → parser after legacy deletion?

**Investigation**:
- Confirmed parser2/transpiler2 are production-ready (used by all runtime layers)
- Confirmed no other modules import from parser2/transpiler2 with version-specific names
- Intermediate naming strategy (parser2 → parser_new → parser) avoids conflicts

**Findings**:
- ✅ All imports use `use pasta::parser2::*` or `use pasta::transpiler2::*` syntax
- ✅ No hardcoded paths or feature flags depend on exact directory names
- ✅ Cargo.toml does not enforce version-specific module names

**Implications for Design**:
- Renaming can proceed in two sub-phases: intermediate rename (safety checkpoint), then final rename (stabilization)
- Build verification required after each sub-phase

---

### Topic 3: Test File Deletion Strategy
**Research Question**: Can we safely delete 21 test files without losing coverage?

**Investigation**:
- Reviewed test file categorization in gap-analysis.md
- 12 parser-only tests → Legacy parser features (all in parser2 now)
- 7 transpiler-only tests → Legacy transpiler features (all in transpiler2 now)
- 3 integration tests → E2E tests using legacy modules directly
- 1 test file (pasta_stdlib_call_jump_separation_test.rs) → Tests registry module, needs import fix only

**Findings**:
- ✅ No test duplication (each test covers legacy features, not shared logic)
- ✅ Integration tests covered by parser2/transpiler2 test suites
- ✅ One test (pasta_stdlib_call_jump_separation_test.rs) validates design principle (Call/Jump don't access word dict)

**Implications for Design**:
- Complete deletion is safe and recommended
- One test must be modified to import from registry instead of transpiler
- No new tests need to be created during this specification

---

### Topic 4: Cargo Build Recovery Strategy
**Research Question**: What is the order of operations for build recovery?

**Investigation**:
- Reviewed requirements.md phases (1-8)
- Analyzed dependency chain: deletion → source fix → test fix → rename → final build

**Findings**:
- Phase 1: Delete src/parser, src/transpiler directories
- Phase 2: Fix source code references → `cargo check` success
- Phase 3: Delete test files + fix pasta_stdlib_call_jump_separation_test.rs → `cargo check --all` success  
- Phase 4: Commit
- Phase 5: Run tests → `cargo test --all` success
- Phase 6: Commit
- Phase 7-8: Rename parser2→parser, transpiler2→transpiler + rebuild/retest

**Implications for Design**:
- Clean separation between deletion and renaming prevents merge conflicts
- `cargo check` is sufficient for source-layer validation
- `cargo test --all` is final validation point

---

## Architecture Pattern Notes

No new architecture patterns introduced in this specification. This is pure code cleanup:
- Same layering (Parser → Transpiler → Runtime)
- Same public API contracts maintained by registry module
- Same error handling patterns (PastaError enum unchanged)

---

## Technology Stack Impact

**No new dependencies introduced**.

Affected Stack Components:
| Layer        | Tool/Version      | Change                                          |
| ------------ | ----------------- | ----------------------------------------------- |
| Build System | Cargo (standard)  | None                                            |
| Compiler     | Rust 2024 edition | None                                            |
| Parser Gen   | Pest 2.8          | Rename grammar.pest location (parser2 → parser) |
| Backend VM   | Rune 0.14         | None                                            |

---

## Risk Assessment

### Risk 1: Incomplete Reference Detection
**Severity**: Medium  
**Mitigation**: Thorough grep search completed in gap-analysis; verified all 22 files

### Risk 2: Build Regression During Rename Phase
**Severity**: Medium  
**Mitigation**: Intermediate rename (parser_new) provides checkpoint to rollback if needed

### Risk 3: Test Fixture Data Loss
**Severity**: Low  
**Mitigation**: Only *.rs test files are targeted; fixture data (*.pasta) checked separately per Req 7

**Overall Risk Level**: Low (straightforward refactoring with clear rollback strategy)

---

## Design Decision Rationale

### Decision: Delete vs Archive Legacy Tests
**Options**:
- A) Complete deletion (saves disk space, clean history)
- B) Archive to separate branch (preserves reference)
- C) Keep with #[ignore] (available for debugging)

**Selected**: Option A (Complete deletion)  
**Rationale**: Legacy parser/transpiler are fully replaced by parser2/transpiler2; archive is available in git history if needed

### Decision: Rename Strategy (Direct vs Intermediate)
**Options**:
- A) Direct rename: parser2 → parser (faster, risky if issue detected)
- B) Intermediate rename: parser2 → parser_new → parser (safer, extra step)

**Selected**: Option B (Intermediate rename)  
**Rationale**: Allows `cargo check` validation between sub-phases; supports rollback if new issues emerge

### Decision: README.md Legacy Documentation
**Options**:
- A) Complete removal (cleaner, may lose reference)
- B) Archive as "Legacy Stack" section (preserves knowledge)
- C) Keep as-is (confused users about current state)

**Selected**: Option A (Complete removal)  
**Rationale**: README is user-facing; showing only production-ready stack prevents confusion

