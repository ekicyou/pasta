# テスト層別化・リネーム計画

## 現状分析

現在の tests/ ディレクトリには約38ファイルのテストがあり、層別に一貫した命名規則がありません。

### 現在のテストファイル分類

#### Parser 層テスト（パース・AST生成）
- `parser_tests.rs` — メインパーサーテスト
- `parser_error_tests.rs` — エラーケース
- `parser_line_types.rs` — 行タイプ分類
- `sakura_script_tests.rs` — さくらスクリプト（パース側）
- `pest_sakura_test.rs` — Pest さくら処理
- `grammar_diagnostic.rs` — 文法診断
- `parser_debug.rs` — デバッグ用
- `parser_sakura_debug.rs` — さくら デバッグ用
- `pest_debug.rs` — Pest デバッグ用

#### Transpiler 層テスト（AST → Rune IR変換）
- `transpile_comprehensive_test.rs` — 総合トランスパイル
- `two_pass_transpiler_test.rs` — 2パス処理
- `label_registry_test.rs` — ラベル管理
- `actor_assignment_test.rs` — アクター変数生成
- `phase3_test.rs` — Phase 3（基本的なcall/jump/local label）

#### Engine/Runtime 層テスト（実行・生成・変数管理）
- `comprehensive_rune_vm_test.rs` — Rune VM基本
- `rune_block_integration_test.rs` — Runeブロック統合
- `rune_closure_test.rs` — Rune クロージャ
- `rune_compile_test.rs` — Rune コンパイル
- `rune_module_memory_test.rs` — メモリ管理
- `rune_module_merge_test.rs` — モジュールマージ
- `rune_rust_module_test.rs` — Rust モジュール統合
- `simple_rune_test.rs` — 基本 Rune
- `label_resolution_runtime_test.rs` — ラベル解決（ランタイム）
- `function_scope_tests.rs` — 関数スコープ
- `persistence_test.rs` — 永続化
- `random.rs` または内部？ — ランダム選択
- `variable*.rs` または内部？ — 変数管理

#### 統合・E2E テスト
- `engine_integration_test.rs` — Engine 統合
- `engine_independence_test.rs` — Engine 独立性
- `engine_two_pass_test.rs` — Engine 2パス
- `end_to_end_simple_test.rs` — E2E シンプル
- `comprehensive_control_flow_test.rs` — 制御フロー総合
- `concurrent_execution_test.rs` — 並行実行
- `stdlib_integration_test.rs` — 標準ライブラリ
- `directory_loader_test.rs` — ディレクトリローダー
- `error_handling_tests.rs` — エラーハンドリング

#### Debug/Legacy（削除・統合対象）
- `sakura_debug_test.rs` — さくら デバッグ（重複?）
- `label_id_consistency_test.rs` — ラベル ID 一貫性
- その他 debug/diagnostic 類

---

## リネーム規則（提案）

```
パーサー層:        pasta_parser_XXX_test.rs
トランスパイラ層:   pasta_transpiler_XXX_test.rs
エンジン/ランタイム層: pasta_engine_XXX_test.rs
統合テスト:        pasta_integration_XXX_test.rs
```

---

## リネーム計画（Phase 0: Pre-Implementation Preparation）

### Step 0.1: 層別化・リネーム

#### Parser 層
- `parser_tests.rs` → `pasta_parser_main_test.rs`
- `parser_error_tests.rs` → `pasta_parser_error_test.rs`
- `parser_line_types.rs` → `pasta_parser_line_types_test.rs`
- `sakura_script_tests.rs` → `pasta_parser_sakura_script_test.rs`
- `pest_sakura_test.rs` → `pasta_parser_pest_sakura_test.rs`
- `grammar_diagnostic.rs` → `pasta_parser_grammar_diagnostic_test.rs`
- `parser_debug.rs` → `pasta_parser_debug_test.rs`
- `parser_sakura_debug.rs` → `pasta_parser_sakura_debug_test.rs`
- `pest_debug.rs` → `pasta_parser_pest_debug_test.rs`

#### Transpiler 層
- `transpile_comprehensive_test.rs` → `pasta_transpiler_comprehensive_test.rs`
- `two_pass_transpiler_test.rs` → `pasta_transpiler_two_pass_test.rs`
- `label_registry_test.rs` → `pasta_transpiler_label_registry_test.rs`
- `actor_assignment_test.rs` → `pasta_transpiler_actor_assignment_test.rs`
- `phase3_test.rs` → `pasta_transpiler_phase3_test.rs`（または移動）

#### Engine/Runtime 層
- `comprehensive_rune_vm_test.rs` → `pasta_engine_rune_vm_comprehensive_test.rs`
- `rune_block_integration_test.rs` → `pasta_engine_rune_block_test.rs`
- `rune_closure_test.rs` → `pasta_engine_rune_closure_test.rs`
- `rune_compile_test.rs` → `pasta_engine_rune_compile_test.rs`
- `rune_module_memory_test.rs` → `pasta_engine_rune_module_memory_test.rs`
- `rune_module_merge_test.rs` → `pasta_engine_rune_module_merge_test.rs`
- `rune_rust_module_test.rs` → `pasta_engine_rune_rust_module_test.rs`
- `simple_rune_test.rs` → `pasta_engine_rune_simple_test.rs`
- `label_resolution_runtime_test.rs` → `pasta_engine_label_resolution_test.rs`
- `function_scope_tests.rs` → `pasta_engine_function_scope_test.rs`
- `persistence_test.rs` → `pasta_engine_persistence_test.rs`

#### 統合テスト
- `engine_integration_test.rs` → `pasta_integration_engine_test.rs`
- `engine_independence_test.rs` → `pasta_integration_engine_independence_test.rs`
- `engine_two_pass_test.rs` → `pasta_integration_engine_two_pass_test.rs`
- `end_to_end_simple_test.rs` → `pasta_integration_e2e_simple_test.rs`
- `comprehensive_control_flow_test.rs` → `pasta_integration_control_flow_test.rs`
- `concurrent_execution_test.rs` → `pasta_integration_concurrent_execution_test.rs`
- `stdlib_integration_test.rs` → `pasta_integration_stdlib_test.rs`
- `directory_loader_test.rs` → `pasta_integration_directory_loader_test.rs`
- `error_handling_tests.rs` → `pasta_integration_error_handling_test.rs`

#### Debug・Legacy（統合/削除候補）
- `sakura_debug_test.rs` → `pasta_debug_sakura_test.rs`（または削除）
- `label_id_consistency_test.rs` → `pasta_integration_label_id_consistency_test.rs`（または削除）

### Step 0.2: 事前グリーン確認

1. リネーム完了後、全テスト実行：
   ```bash
   cargo test --all
   ```
2. 全テストが `test result: ok` となることを確認
3. テスト結果を記録：
   ```bash
   cargo test --all > .kiro/specs/pasta-grammar-specification/test-baseline.log
   ```

### Step 0.3: Commit（グリーン状態を保存）

```bash
git add tests/*.rs
git commit -m "Refactor: Organize test files by layer hierarchy (parser/transpiler/engine/integration) for regression tracking

- Rename test files with layer prefix (pasta_parser_*, pasta_transpiler_*, pasta_engine_*, pasta_integration_*)
- Improves regression detection: easier to identify which layer has failures
- All tests pass before implementing DSL grammar changes
- Baseline for regression comparison in Phase 1-3 implementation"
```

---

## Phase 0 の成果物

- リネーム済みテストファイル（約38ファイル）
- `.kiro/specs/pasta-grammar-specification/test-baseline.log` — 全テスト実行ログ
- Git commit — グリーン状態の記録
- `test-hierarchy-plan.md` — このドキュメント

---

## 使用方法（Phase 1-3 での活用）

- **Phase 1 後**: Parser 層テストのみ失敗 → `pasta_parser_*.rs` で修正
- **Phase 2 後**: Transpiler 層テストの失敗 → `pasta_transpiler_*.rs` で修正
- **Phase 3 後**: Engine/Integration 層テストの失敗 → `pasta_engine_*.rs`, `pasta_integration_*.rs` で修正
- 回帰チェック: `cargo test --all -- --nocapture | grep -E "^test (pasta_parser|pasta_transpiler|pasta_engine|pasta_integration)"`

---

## リスク・注意点

- テストファイルのリネームは機械的だが、`mod.rs` (tests/common/) への参照がある場合は要確認
- `Cargo.toml` や CI/CD 設定に明示的なテスト指定がある場合は更新必須
- tests/fixtures/ は変更不要

