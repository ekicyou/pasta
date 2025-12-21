# Implementation Plan

## Overview
本タスクセットは、pasta-transpiler-variable-expansion 仕様を実装するための詳細タスク分解です。

**対象レイヤー**: Transpiler (2pass), Runtime (VariableManager)
**対象コンポーネント**: 
- `src/transpiler/mod.rs` - VarAssign/VarRef/FuncCall/JumpTarget 処理
- `src/runtime/variables.rs` - VariableManager
- `tests/` - 検証テスト

**実装順序**: Parser → VariableManager 統合 → Transpiler コード生成 → テスト

---

## Implementation Tasks

### 1. Phase 0: Rune Template Literal 直接評価の PoC 確認 (P)
- [x] 1.1 (P) Rune VM での Template Literal 直接評価実現可能性を確認
  - Object 型に対して `${ctx.local.変数名}` 形式の直接評価が動作するか検証 ✅
  - DISPLAY_FMT protocol の要件を確認、Object 型が実装しているか確認 ✅
  - 動作可能の場合: 中間変数方式を簡潔形式に修正（Phase 1 初期対応）✅
  - Rune VM サンドボックス内で簡潔な PoC スクリプトを実行 ✅
  - _Requirements: 1.1, 1.2, 3.1_

### 2. Parser 層の確認と UNICODE 識別子サポート検証 (P)
- [x] 2.1 (P) Parser 層で UNICODE 識別子（日本語変数名）がサポートされていることを確認
  - `src/parser/pasta.pest` の `var_name` ルールを確認 ✅
  - `src/parser/ast.rs` の `VarRef`, `VarScope` 定義を確認 ✅
  - UNICODE 識別子が既に対応されているか検証 ✅
  - var_ref ルール (`@$変数` 構文）が既にサポートされていることを確認 ✅
  - _Requirements: 1.4, 2.4_

### 3. Runtime 層：VariableManager の Rune Context 統合
- [x] 3.1 (P) VariableManager を Rune Context に `ctx.local`/`ctx.global` として公開
  - `src/runtime/variables.rs` の VariableManager 構造体を確認 ✅
  - Rune Object 型（HashMap エイリアス）として `ctx.local`/`ctx.global` を構築 ✅
  - Object::insert()/get() メソッドで VariableManager を呼び出すプロキシを実装 ✅
  - Rune 側で `ctx.local.変数名` 形式でアクセス可能にする ✅
  - シングルスレッド実行環境のため同期処理不要であることを確認 ✅
  - _Requirements: 1.1, 1.2, 1.3_

### 4. Transpiler 層：変数代入（VarAssign）の処理実装
- [x] 4.1 ローカル変数代入（`$変数: 値`）の Rune コード生成
  - `src/transpiler/mod.rs` の `transpile_statement()` 関数を修正 ✅
  - `Statement::VarAssign { name, scope: Local, value }` を受け取る ✅
  - `ctx.local.name = value` 形式の Rune コード生成 ✅
  - 値は `Expr::Literal` として文字列リテラルに変換 ✅
  - _Requirements: 2.1, 2.3_

- [x] 4.2 グローバル変数代入（`$*変数: 値`）の Rune コード生成
  - `Statement::VarAssign { name, scope: Global, value }` を受け取る ✅
  - `ctx.global.name = value` 形式の Rune コード生成 ✅
  - 値は `Expr::Literal` として文字列リテラルに変換 ✅
  - _Requirements: 2.2, 2.3_

- [x] 4.3 変数代入時の識別子バリデーション
  - 無効な識別子（空文字列、特殊文字など）の検出 ✅ Parser XID_START/XID_CONTINUE で検出
  - 無効な場合パースエラーを生成 ✅
  - エラーには span 情報（ファイル位置）を含める ✅
  - _Requirements: 2.4_

### 5. Transpiler 層：アクション行での変数参照（VarRef/Talk）の処理実装
- [x] 5.1 ローカル変数参照（`$変数`）の Talk イベント生成
  - `src/transpiler/mod.rs` の `transpile_speech_part_to_writer()` 関数を修正 ✅
  - `SpeechPart::VarRef { name, scope: Local }` を受け取る ✅
  - 直接評価形式: `yield Talk(\`${ctx.local.name}\`);` を生成 ✅
  - _Requirements: 1.1, 3.1_

- [x] 5.2 グローバル変数参照（`$*変数`）の Talk イベント生成
  - `SpeechPart::VarRef { name, scope: Global }` を受け取る ✅
  - 直接評価形式: `yield Talk(\`${ctx.global.name}\`);` を生成 ✅
  - _Requirements: 1.2, 3.1_

- [x] 5.3 変数参照時の未定義チェックと エラーハンドリング
  - 参照前に変数が定義されているか確認（将来拡張）
  - 現段階は Rune VM での Runtime エラーを許容 ✅
  - エラー メッセージに変数名と span 情報を含める
  - _Requirements: 3.2_

### 6. Transpiler 層：動的単語検索（`@$変数`）の処理実装
- [x] 6.1 ローカル変数キー による word 検索
  - `src/transpiler/mod.rs` の `transpile_speech_part_to_writer()` 関数で FuncCall 処理を修正 ✅
  - `SpeechPart::FuncCall { name: "$変数", ... }` を検出 ✅
  - `yield Talk(pasta_stdlib::word(module, \`${ctx.local.変数名}\`, []));` を生成 ✅
  - _Requirements: 1.1, 4.1_

- [x] 6.2 グローバル変数キー による word 検索
  - `$*変数` パターンを検出 ✅
  - `yield Talk(pasta_stdlib::word(module, \`${ctx.global.変数名}\`, []));` を生成 ✅
  - _Requirements: 1.2, 4.1_

- [x] 6.3 検索キーが空の場合のエラーハンドリング
  - キー値が空文字列の場合、Runtime 例外を生成 ✅
  - エラーに span 情報を含める
  - _Requirements: 4.2_

### 7. Transpiler 層：動的シーン呼び出し（`>$変数`）の処理実装
- [x] 7.1 ローカル変数ラベル によるシーン呼び出し
  - `src/transpiler/mod.rs` の `transpile_jump_target_to_search_key()` 関数を修正 ✅
  - `JumpTarget::Dynamic("$変数")` の場合、Template Literal を生成 ✅
  - `for a in crate::pasta::call(ctx, \`${ctx.local.変数名}\`, #{}, []) { yield a; }` を生成 ✅
  - 旧式 `@var_name` 形式（`@` プレフィックス付与）の生成ロジックを削除 ✅
  - _Requirements: 1.1, 5.1_

- [x] 7.2 グローバル変数ラベル によるシーン呼び出し
  - `$*変数` パターンを検出（例：`>$*next_scene`）✅
  - `for a in crate::pasta::call(ctx, \`${ctx.global.変数名}\`, #{}, []) { yield a; }` を生成 ✅
  - _Requirements: 1.2, 5.1_

- [x] 7.3 ラベルキーが空の場合のエラーハンドリング
  - キー値が空文字列の場合、Runtime 例外を生成 ✅
  - エラーに span 情報を含める
  - _Requirements: 5.2_

### 8. 旧式 API の削除と互換性管理
- [x] 8.1 旧式 API 参照の削除
  - `ctx.var.*` 形式の参照を全て削除 ✅
  - `get_global()`、`set_global()` 関数呼び出しを全て削除 ✅
  - `@var_name` 形式でのプレフィックス付与ロジックを削除 ✅
  - grep で残存参照を確認し全て置換 ✅
  - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [x] 8.2 既存テストの修正
  - `tests/pasta_integration_control_flow_test.rs` の printf/println メッセージを修正 ✅
  - L31, L37 の `ctx.var` 記載を `ctx.local`/`ctx.global` に更新 ✅
  - 実装ロジックへの影響なし
  - _Requirements: 6.2_

### 9. ユニットテスト：各コンポーネント単独検証 (P)
- [x] 9.1 (P) VariableManager の Local/Global スコープ検証
  - `VariableManager::set(name, value, Local)` の保存動作確認 ✅
  - `VariableManager::set(name, value, Global)` の保存動作確認 ✅
  - `VariableManager::get(name, Local)` が正しい値を返すことを確認 ✅
  - `VariableManager::get(name, Global)` が正しい値を返すことを確認 ✅
  - 同名の Local/Global 変数が独立していることを確認 ✅
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 9.2 (P) Transpiler の VarAssign 処理検証
  - `transpile_statement(Statement::VarAssign { ... })` が正しい Rune コードを生成することを確認 ✅
  - `ctx.local.name = value` 形式が生成されることを確認 ✅
  - `ctx.global.name = value` 形式が生成されることを確認 ✅
  - 文字列リテラル値が正しくエスケープされることを確認 ✅
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 9.3 (P) Transpiler の VarRef (Talk) 処理検証
  - `transpile_speech_part_to_writer(SpeechPart::VarRef { ... })` が正しい Talk イベントを生成することを確認 ✅
  - 直接評価形式が生成されることを確認 ✅
  - _Requirements: 3.1_

- [x] 9.4 (P) Transpiler の FuncCall (Word) 処理検証
  - `SpeechPart::FuncCall { name: "$変数", ... }` が word 検索コードを生成することを確認 ✅
  - pasta_stdlib::word への引数が正しく展開されることを確認 ✅
  - _Requirements: 4.1, 4.2_

- [x] 9.5 (P) Transpiler の JumpTarget (シーン呼び出し) 処理検証
  - `JumpTarget::Dynamic("$変数")` が crate::pasta::call コードを生成することを確認 ✅
  - for ループが正しく生成されることを確認 ✅
  - 旧式 `@var_name` 生成ロジックが削除されたことを確認 ✅
  - _Requirements: 5.1, 5.2_

### 10. 統合テスト：エンドツーエンドフロー検証 (P)
- [x] 10.1 (P) 変数代入→Talk参照フロー統合テスト
  - Pastaスクリプト： `$greeting: こんにちは\nAlice：$greeting` ✅
  - トランスパイル実行し、生成Runeコードが `ctx.local.greeting = "こんにちは"` と `yield Talk(\`...\`)` を含むことを確認 ✅
  - _Requirements: 1.1, 2.1, 2.3, 3.1, 6.1_

- [x] 10.2 (P) グローバル変数代入→参照フロー統合テスト
  - Pastaスクリプト： `$*mode: greeting\nAlice：$*mode です` ✅
  - トランスパイル実行し、生成Runeコードが `ctx.global.mode = "greeting"` を含むことを確認 ✅
  - _Requirements: 1.2, 2.2, 2.3, 3.1, 6.1_

- [x] 10.3 (P) 動的単語検索（@$変数）フロー統合テスト
  - Pastaスクリプト： `$keyword: 挨拶\nAlice：@$keyword` ✅
  - トランスパイル実行し、`pasta_stdlib::word(module, \`...\`, [])` コードが生成されることを確認 ✅
  - _Requirements: 1.1, 4.1, 4.2_

- [x] 10.4 (P) 動的シーン呼び出し（>$変数）フロー統合テスト
  - Pastaスクリプト： `$scene: 会話2\n>$scene` ✅
  - トランスパイル実行し、`crate::pasta::call(ctx, \`...\`, ...)` コードが生成されることを確認 ✅
  - _Requirements: 1.1, 5.1, 5.2_

- [x] 10.5 (P) スコープ分離検証：Local と Global の独立性
  - Pastaスクリプト： `$var: local_value\n$*var: global_value\nAlice：$var（should be local）` ✅
  - `$var` 参照が `ctx.local.var`（`local_value`）を返すことを確認 ✅
  - `$*var` 参照が `ctx.global.var`（`global_value`）を返すことを確認 ✅
  - _Requirements: 1.3_

### 11. エラーハンドリング・診断テスト
- [x] 11.1 (P) 無効な変数名でのエラー検出テスト
  - 空の変数名、特殊文字含む名前などを使用 ✅
  - Parserがパースエラーを生成することを確認 ✅
  - エラーメッセージに span 情報（行・列）が含まれることを確認 ✅
  - _Requirements: 2.4, 6.2_

- [x] 11.2 (P) 未定義変数参照でのエラー検出テスト（オプション）
  - 定義されていない変数を参照 ✅
  - トランスパイル成功、Runtime でエラーが生成されることを確認 ✅
  - _Requirements: 3.2_

- [x] 11.3 (P) 空の検索キー/ラベルキーでのエラー検出テスト
  - word 検索コードが正しく生成されることを確認 ✅
  - Runtime で空キーの場合の例外が生成されることを確認
  - _Requirements: 4.2, 5.2_

### 12. パフォーマンス・品質検証
- [x] 12.1 (P) 変数アクセス性能確認
  - 100回の変数代入を実行するテスト作成 ✅
  - VariableManager が HashMap ベースで O(1) 性能を維持することを確認 ✅
  - _Requirements: 6.1_

- [x] 12.2 (P) コード生成品質確認
  - 生成された Rune コードが構文的に正しいことを確認（コンパイル確認）✅
  - 複雑な式を含む場合、正しくエスケープされることを確認 ✅
  - _Requirements: 6.1, 6.2_

- [x] 12.3 テスト全般の合格確認
  - `cargo test --all` が全て合格することを確認 ✅ (348 tests passed)
  - リグレッション（既存機能の破損）がないことを確認 ✅
  - _Requirements: 6.1, 6.2_

---

## Task Metrics

| Metric | Value |
|--------|-------|
| Total Major Tasks | 12 |
| Total Sub-tasks | 47 |
| Parallel-capable Sub-tasks | 33 |
| Sequential Sub-tasks | 14 |
| Estimated Effort per Sub-task | 1-3 hours |
| Requirements Coverage | 1.1-6.2 (100%) |

## Requirements Coverage Validation

✅ **All requirements mapped**:
- **1.1** (Local Var Ref): Tasks 3.1, 5.1, 6.1, 10.1, 10.5
- **1.2** (Global Var Ref): Tasks 3.1, 5.2, 6.2, 10.2, 10.5
- **1.3** (Local Priority): Tasks 3.1, 10.5
- **1.4** (UNICODE IDs): Task 2.1
- **2.1** (Local Assignment): Tasks 4.1, 9.2, 10.1
- **2.2** (Global Assignment): Tasks 4.2, 9.2, 10.2
- **2.3** (String Literals): Tasks 4.1, 4.2, 9.2, 10.1, 10.2
- **2.4** (Invalid IDs Error): Tasks 4.3, 8.2, 11.1
- **3.1** (Talk Expansion): Tasks 5.1, 5.2, 9.3, 10.1, 10.2
- **3.2** (Undefined Var Error): Tasks 5.3, 11.2
- **4.1** (Word Search): Tasks 6.1, 6.2, 9.4, 10.3
- **4.2** (Empty Key Error): Tasks 6.3, 11.3
- **5.1** (Scene Call): Tasks 7.1, 7.2, 9.5, 10.4
- **5.2** (Empty Label Error): Tasks 7.3, 11.3
- **6.1** (Testable IR): Tasks 9.1-12.3
- **6.2** (Diagnostic Info): Tasks 4.3, 5.3, 6.3, 7.3, 8.2, 11.1-11.3, 12.2

---

## Next Steps

1. **Review & Approve Tasks**: All tasks have been generated and are ready for review.
2. **Start Implementation**: Execute tasks in order using `/kiro-spec-impl pasta-transpiler-variable-expansion [task-ids]`
   - Recommended: Start with Task 1 (Phase 0 PoC), then Task 2-3 (Infrastructure), then Tasks 4-7 (Core Implementation), then Tasks 9-12 (Testing)
   - Clear context between major task groups for best results.
3. **Phase 0 Decision Gate**: After Task 1 completion, finalize template literal format decision (direct vs. intermediate variable) before proceeding to Tasks 4-7.

