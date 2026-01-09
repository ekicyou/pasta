# Implementation Plan

## Task Overview

**Total**: 6 major tasks, 17 sub-tasks  
**Coverage**: All 9 requirements (1, 2, 3, 4, 5, 6, 7, 8, 9)  
**Estimated Duration**: 4-5 hours total (1-3 hours per major task)

---

## Major Tasks

### 1. Update Scene Function Signature and Initialize Session

- [ ] 1.1 (P) Change scene function signature from `ctx` to `act` parameter
  - Replace the parameter name in function declaration at L253
  - Update function naming pattern (entry point and named local scenes)
  - Verify all actor context references use the new `act` parameter
  - Run `cargo build` to validate syntax
  - _Requirements: 1, 2_

- [ ] 1.2 (P) Replace `PASTA.create_session()` call with `act:init_scene()` 
  - Update the call format at L278 to use `act:init_scene(SCENE)`
  - Adjust return value handling from 3-tuple to 2-tuple (save, var)
  - Ensure save/var variables are properly captured from init_scene return
  - Run `cargo build` to validate syntax
  - _Requirements: 2, 7_

- [ ] 1.3 Update `generate_local_scene()` function documentation
  - Add docstring explaining the new `act` parameter convention
  - Include example output showing the updated signature
  - Document the init_scene call pattern
  - _Requirements: 9_

### 2. Implement Spot Management API Changes

- [ ] 2.1 (P) Replace `PASTA.clear_spot(ctx)` with `act:clear_spot()` call
  - Update the method call format at L268
  - Remove the `ctx` parameter from the call
  - Ensure this change precedes init_scene() in generated code
  - Run `cargo build` to validate syntax
  - _Requirements: 3, 7_

- [ ] 2.2 (P) Replace `PASTA.set_spot()` calls with `act:set_spot()` format
  - Update the method call format at L270
  - Remove the `ctx` parameter and adjust argument positions
  - Ensure each actor's spot is set in correct order after clear_spot()
  - Run `cargo build` to validate syntax
  - _Requirements: 3, 7_

- [ ] 2.3 Update spot management logic in documentation
  - Add docstring notes about clear_spot and set_spot API
  - Include examples of the new format in generate_local_scene documentation
  - _Requirements: 9_

### 3. Implement Actor Proxy Calling Patterns

- [ ] 3.1 (P) Wrap word() calls with talk() at L510
  - Change output format from `act.actor:word()` to `act.actor:talk(act.actor:word())`
  - Ensure current actor context is maintained
  - Apply StringLiteralizer to word name argument via literalize()
  - Run `cargo build` to validate syntax
  - _Requirements: 4, 7_

- [ ] 3.2 (P) Apply StringLiteralizer to word() arguments at L362 and L472
  - Update VarSet word() processing at L362 to wrap word name with StringLiteralizer::literalize()
  - Update Action::WordRef processing at L472 to wrap word name with StringLiteralizer::literalize()
  - Ensure all string literals are processed uniformly
  - Run `cargo build` to validate syntax
  - _Requirements: 7, 4_

- [ ] 3.3 Update actor proxy documentation
  - Add docstring explaining word() wrapping pattern
  - Document the talk() + word() nested call format
  - Include StringLiteralizer requirement explanation
  - _Requirements: 9_

### 4. Implement SakuraScript and Escape Character Handling

- [ ] 4.1 (P) Replace actor-scoped sakura_script with `act:sakura_script()` at L509
  - Change output format to use act-scoped method instead of actor-scoped talk()
  - Ensure SakuraScript is output as standalone (not wrapped in talk)
  - Maintain StringLiteralizer processing for script text
  - Run `cargo build` to validate syntax
  - _Requirements: 6, 7_

- [ ] 4.2 (P) Apply StringLiteralizer to escape character output at L511
  - Update Action::Escape match branch to wrap escape character with StringLiteralizer::literalize()
  - Ensure character is properly escaped before output
  - Run `cargo build` to validate syntax
  - _Requirements: 7, 6_

- [ ] 4.3 Update sakura_script documentation
  - Add docstring explaining the act:sakura_script() method
  - Document escape character handling with StringLiteralizer
  - _Requirements: 9_

### 5. Update Tests for New Output Format

- [ ] 5.1 Update transpiler integration test assertions
  - Modify existing test cases in `transpiler_integration_test.rs` to match new output format
  - Update assertions for scene signature (ctx → act)
  - Update assertions for init_scene call format
  - Update assertions for spot management methods
  - Verify test count (~20-25 affected tests)
  - _Requirements: 8, 1, 2, 3_

- [ ] 5.2 Update actor proxy and sakura_script test assertions
  - Update assertions for word() wrapping pattern
  - Update assertions for act:sakura_script() format
  - Update assertions for escape character handling
  - Verify all affected tests pass
  - _Requirements: 8, 4, 6, 7_

- [ ] 5.3 Update fixture files and expected outputs
  - Update files in `crates/pasta_lua/tests/fixtures/`
  - Update Lua test expectations in `crates/pasta_lua/tests/lua_specs/`
  - Ensure all expected output reflects new API format
  - Run `cargo test --lib` to validate
  - _Requirements: 8_

- [ ] 5.4 (P) Execute full test suite
  - Run `cargo test --all` to verify no regressions
  - Verify all new assertions pass
  - Confirm test coverage for new code patterns
  - Document any test failures and corrections needed
  - _Requirements: 8_

### 6. Verify Code Generation and Document Changes

- [ ] 6.1 (P) Validate generated Lua code correctness
  - Run integration tests and inspect generated output samples
  - Verify scene function signature matches expected format
  - Verify API calls (init_scene, clear_spot, set_spot, sakura_script) are correct
  - Verify word() wrapping and actor proxy calls are correct
  - _Requirements: 1, 2, 3, 4, 5, 6, 7_

- [ ] 6.2 Update code_generator.rs module documentation
  - Update module-level docstring with overview of new functionality
  - Update method documentation for generate_local_scene() and generate_action()
  - Add examples showing new output format for each major change
  - Ensure documentation reflects StringLiteralizer unified rule
  - _Requirements: 9_

- [ ] 6.3 Final validation and commit
  - Verify `cargo check --all` passes
  - Verify `cargo test --all` passes with no failures
  - Confirm all changes are committed with clear commit messages
  - Document any deviations from design in implementation notes
  - _Requirements: 1, 2, 3, 4, 5, 6, 7, 8, 9_

---

## Requirement Mapping Summary

| Requirement | Tasks | Coverage |
|-------------|-------|----------|
| 1 | 1.1, 5.1, 6.1 | Scene signature change implementation and verification |
| 2 | 1.2, 5.1, 6.1 | init_scene implementation and verification |
| 3 | 2.1, 2.2, 5.1, 6.1 | Spot management implementation and verification |
| 4 | 3.1, 3.2, 5.2, 6.1 | Actor proxy implementation and verification |
| 5 | - | (Already implemented, skipped) |
| 6 | 4.1, 4.2, 5.2, 6.1 | SakuraScript implementation and verification |
| 7 | 1.2, 2.1, 2.2, 3.1, 3.2, 4.1, 4.2, 6.1 | StringLiteralizer and variable access throughout |
| 8 | 5.1, 5.2, 5.3, 5.4, 6.1 | Test updates and coverage |
| 9 | 1.3, 2.3, 3.3, 4.3, 6.2 | Documentation updates |

