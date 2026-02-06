# Implementation Plan: yield-continuation-token

## Overview

3つの主要コンポーネント（ランタイム実装 + Lua BDD テスト2層 + Rust E2E テスト）を順序立てて実装する。ランタイム実装を最初に行い、その後テストで段階的に検証する。

- **Total**: 4 major tasks with 10 sub-tasks
- **Requirements Coverage**: All 3 requirements (1, 2, 3) mapped across 13 numeric requirement items (1.1–3.4)
- **Average Task Size**: 1–3 hours per sub-task

---

## Tasks

### 1. GLOBAL テーブルへのチェイントーク関数登録

- [ ] 1.1 (P) GLOBAL.チェイントーク と GLOBAL.yield 関数をグローバルモジュールに実装
  - `scripts/pasta/global.lua` を修正し、空テーブルの中に2つの関数定義を追加
  - 両関数は同一シグネチャ `function(act)` をとり、`act:yield()` を呼び出すのみ
  - 既存の `local GLOBAL = {} ... return GLOBAL` パターンを維持
  - Lua BDD テスト実行確認用に一時的なテスト実行を含める（最終検証は後続タスクで実施）
  - _Requirements: 1.1, 1.2_

### 2. Lua BDD ランタイム動作テスト

- [ ] 2.1 Lua BDD テストの初期構成と GLOBAL 関数登録の検証テスト
  - テストファイル `tests/lua_specs/global_chaintalk_call_test.lua` を作成
  - `lua_test.test` フレームワークで describe/test/expect パターンで記述
  - GLOBAL.チェイントーク と GLOBAL.yield が非 nil 関数として存在することを確認
  - `init.lua` の specs テーブルに新テストファイル名を登録
  - _Requirements: 2.1, 2.3_

- [ ] 2.2 (P) GLOBAL 関数の L3 解決と yield 動作の検証
  - `act:call` で GLOBAL.チェイントーク が L3 検索で見つかることを確認
  - コルーチン内での `act:yield()` が正しく `coroutine.yield()` を実行することを検証
  - yield 前のトークン蓄積出力が返ってくることを確認
  - `act_impl_call_test.lua` の L3 解決パターンを参考に実装
  - _Requirements: 2.1, 2.2, 2.4_

### 3. Lua BDD EVENT.fire 統合テスト

- [ ] 3.1 Lua BDD 統合テストの構成と EVENT.fire + コルーチン分割の検証
  - テストファイル `tests/lua_specs/global_chaintalk_integration_test.lua` を作成
  - EVENT.fire のハンドラーとして GLOBAL.チェイントーク を呼び出すシーン関数を登録
  - `coroutine.create` でコルーチンを生成し、その中で GLOBAL.チェイントーク を呼び出し
  - `integration_coroutine_test.lua` の EVENT.fire + STORE.co_scene パターンを踏襲
  - `init.lua` の specs テーブルに新テストファイル名を登録
  - _Requirements: 3.1, 3.3_

- [ ] 3.2 (P) EVENT.fire 経由での中間・最終出力の分割検証
  - 1回目 resume 時に yield 前のトークンのみが中間出力として返ることを確認
  - 2回目 resume 時に yield 後のトークンが最終出力として返ることを確認
  - STORE.co_scene のコルーチン状態が適切に遷移することを検証
  - resume_until_valid による nil 出力スキップの動作を確認
  - _Requirements: 3.2, 3.3_

### 4. Rust E2E 一気通貫テスト

- [ ] 4.1 (P) Rust E2E テストフィクスチャと Pasta DSL コード生成の検証
  - テストフィクスチャ `tests/fixtures/e2e/runtime_e2e_scene_chaintalk.pasta` を作成
  - `＞チェイントーク` を含むシーンを記述し、トランスパイルして Lua コードを生成
  - トランスパイル出力が `act:call(SCENE.__global_name__, "チェイントーク", ...)` を含むことを確認
  - スナップショットテストで トランスパイル出力を検証（`runtime_e2e_scene_chaintalk.lua` expected）
  - _Requirements: 3.4_

- [ ] 4.2 (P) Rust E2E テストケース実装と一気通貫実行検証
  - `tests/runtime_e2e_test.rs` に `test_e2e_chaintalk_transpile_and_execute()` を新規追加
  - 既存ヘルパー（`create_runtime_with_finalize()`, `transpile()`）を利用
  - Pasta DSL → トランスパイル → finalize_scene → GLOBAL 関数呼び出し → yield の完全フロー検証
  - Lua ランタイムで GLOBAL.チェイントーク が呼ばれ、コルーチン yield が正常に機能することを確認
  - _Requirements: 3.4_

---

## Testing & Quality Gates

### Lua BDD テスト実行
```bash
cd crates/pasta_lua && cargo test --test lua_unittest_runner
```
- ✅ `global_chaintalk_call_test.lua` 全テストケース成功
- ✅ `global_chaintalk_integration_test.lua` 全テストケース成功

### Rust E2E テスト実行
```bash
cd crates/pasta_lua && cargo test --test runtime_e2e_test -- --test-threads=1
```
- ✅ `test_e2e_chaintalk_transpile_and_execute()` 成功
- ✅ トランスパイル出力スナップショット確認

### 全テスト実行
```bash
cargo test --all
```
- ✅ 既存テスト全成功（回帰なし）

---

## Final Task: ドキュメント整合性の確認と更新

- [ ] 5. ドキュメント整合性の確認と更新
  - [ ] SOUL.md - コアバリュー・設計原則との整合性確認
  - [ ] doc/spec/ - 言語仕様の更新（該当する場合）
  - [ ] GRAMMAR.md - 文法リファレンスの同期（チェイントーク構文）
  - [ ] TEST_COVERAGE.md - 新規テスト（Lua BDD 2層 + Rust E2E）の記録
  - [ ] crates/pasta_lua/README.md - 継続トーク機能の記載（該当する場合）
  - [ ] steering/* - 該当領域のステアリング更新
  - _Requirements: 1, 2, 3_

---

## Notes

- **パラレル実行候補**: タスク 2.2, 3.2, 4.1, 4.2 は依存関係がなく並列実行可能
- **テスト順序**: ランタイム実装（1.1）→ ユニットテスト（2層）→ E2E テスト の順序必須
- **既存パターン準拠**: 既存テストファイル（`act_impl_call_test.lua`, `integration_coroutine_test.lua`）の構造を踏襲
- **最小変更**: ランタイム実装は `global.lua` 1ファイルのみ、変更量は ~5 行（関数定義の追加）
