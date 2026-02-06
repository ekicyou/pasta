# Research & Design Decisions: yield-continuation-token

## Summary
- **Feature**: yield-continuation-token
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - `global.lua` は空テーブルのみで、関数追加は既存パターン完全準拠
  - `ACT_IMPL.call` L3 → `ACT_IMPL.yield` のコールチェーンは既存テスト済み
  - トランスパイラは `＞チェイン` を `act:call(SCENE.__global_name__, "チェイン", {}, table.unpack(args))` に変換する

## Research Log

### GLOBAL テーブルへの関数登録パターン
- **Context**: `global.lua` に関数を追加する際の既存パターンを調査
- **Sources Consulted**: `scripts/pasta/global.lua`, `steering/lua-coding.md`
- **Findings**:
  - `global.lua` は `local GLOBAL = {} ... return GLOBAL` の標準モジュール構造
  - lua-coding.md で日本語識別子は内部変数・GLOBAL エントリとして許可されている
  - 関数シグネチャは `function(act, ...)` パターン（`ACT_IMPL.call` が第1引数に `self` を渡す）
- **Implications**: `GLOBAL.チェイン = function(act) ... end` の形式で追加可能

### ACT_IMPL.call の検索フロー
- **Context**: `＞チェイン` が GLOBAL 関数に正しく解決されることを検証
- **Sources Consulted**: `scripts/pasta/act.lua` L313-L347, `crates/pasta_lua/src/code_generator.rs` L408-L455
- **Findings**:
  - トランスパイラ出力: `act:call(SCENE.__global_name__, "チェイン", {}, table.unpack(args))`
  - L1（current_scene）→ L2（SCENE.search）→ L3（GLOBAL[key]）→ L4（SCENE.search fallback）
  - L3 は完全一致: `handler = GLOBAL[key]`
  - ハンドラー実行: `handler(self, ...)` — `self` は act オブジェクト
- **Implications**: GLOBAL 関数の第1引数は act オブジェクト。`act:yield()` で `self:build()` → `coroutine.yield(result)` を実行

### コルーチンコンテキストの検証
- **Context**: GLOBAL 関数内の `act:yield()` が正しくコルーチン yield するか
- **Sources Consulted**: `scripts/pasta/act.lua` L289-L293, `scripts/pasta/scene.lua` SCENE.co_exec
- **Findings**:
  - `ACT_IMPL.call` は通常の関数呼び出し（新コルーチン非生成）
  - GLOBAL 関数内の `act:yield()` は、呼び出し元コルーチン（`SCENE.co_exec` 生成）の `coroutine.yield()` を実行
  - `EVENT.fire` の `resume_until_valid` が yield 結果を受け取り、`set_co_scene` でコルーチン状態を管理
- **Implications**: 追加のコルーチン制御は不要。既存メカニズムが完全に機能する

### テストインフラの調査
- **Context**: Req 2, Req 3 のテスト実装方法を検討
- **Sources Consulted**: `tests/common/e2e_helpers.rs`, `tests/runtime_e2e_test.rs`, `tests/lua_specs/init.lua`, `tests/lua_specs/integration_coroutine_test.lua`
- **Findings**:
  - **Rust E2E**: `transpile()` → `lua.load()` → `finalize_scene()` → scene 実行のパイプラインが `e2e_helpers` に整備済み
  - **Lua BDD**: `lua_test.test` フレームワーク (`describe`, `test`, `expect`)、`package.loaded` リセットによるモジュール再初期化パターン
  - `init.lua` の specs テーブルに新テスト名を追加するだけで自動実行される
- **Implications**: 両テスト層とも追加インフラ不要。既存パターンの踏襲のみ

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: global.lua 直接拡張 | 空テーブルに関数定義を追加 | 最小変更量、パターン完全準拠、テスト容易 | なし | **採用** |
| B: 別モジュール分離 | 新規 builtins.lua に分離 | 責務分離が明確 | 不要な複雑化、読み込み順制御 | 却下 |
| C: Rust 側からの登録 | Rust バインディングで GLOBAL に登録 | 型安全性 | 過剰設計、責務境界不明確 | 却下 |

## Design Decisions

### Decision: global.lua 直接拡張アプローチ
- **Context**: GLOBAL テーブルへのデフォルト関数登録方法の選択
- **Alternatives Considered**:
  1. Option A — global.lua に直接定義
  2. Option B — 新規 builtins.lua に分離
  3. Option C — Rust 側から登録
- **Selected Approach**: Option A — `global.lua` に `GLOBAL.チェイン` と `GLOBAL.yield` を直接定義
- **Rationale**: 変更量最小（実質3行追加）、既存の「GLOBAL はユーザー拡張可能なテーブル」という設計思想と完全に整合、新ファイル・新モジュール・新バインディング一切不要
- **Trade-offs**: 将来デフォルト関数が大量に増えた場合の管理性 → 現時点では2関数のみで問題なし
- **Follow-up**: なし

### Decision: テスト戦略の2層分離
- **Context**: Req 2（ランタイム動作試験）と Req 3（EVENT.fire 統合テスト）のテスト層選択
- **Alternatives Considered**:
  1. 両方 Rust E2E
  2. 両方 Lua BDD
  3. Req 2 を Lua BDD、Req 3 を Lua BDD（同一テストファイル）
  4. Req 2 を Lua BDD、Req 3 を Lua BDD（別テストファイル）
- **Selected Approach**: Option 4 — 両テストとも Lua BDD で実装、別テストファイルに分離
- **Rationale**:
  - Req 2 は `ACT_IMPL.call` → `GLOBAL.チェイン` → `act:yield()` のユニット的な検証。Lua 層で完結
  - Req 3 は `EVENT.fire` → コルーチン分割の統合検証。`integration_coroutine_test.lua` と同じパターン
  - Pasta DSL → トランスパイル経路は `ACT_IMPL.call` のテストで既にカバーされており、Rust E2E の追加価値は薄い
  - ただし、トランスパイラの `＞チェイン` 出力を検証するスナップショットテストは既存テストでカバー可能
- **Trade-offs**: Rust E2E による DSL→トランスパイル→実行の一気通貫検証は省略 → 既存の ACT_IMPL.call テスト + トランスパイラスナップショットでカバー済み
- **Follow-up**: 設計で具体的なテストファイル名・構成を決定

## Risks & Mitigations
- **Risk 1**: ユーザーが `GLOBAL.チェイン` を意図せず上書きしてしまう — `main.lua` での明示的代入のみなので低リスク。ドキュメントで注意喚起
- **Risk 2**: 将来の GLOBAL 関数追加時にファイル肥大化 — 現時点では2関数のみ。増加時は別途モジュール分離を検討

## References
- [lua-coding.md](.kiro/steering/lua-coding.md) — Lua コーディング規約
- [act.lua](crates/pasta_lua/scripts/pasta/act.lua) — ACT_IMPL.call, ACT_IMPL.yield 実装
- [global.lua](crates/pasta_lua/scripts/pasta/global.lua) — GLOBAL テーブル（変更対象）
- [integration_coroutine_test.lua](crates/pasta_lua/tests/lua_specs/integration_coroutine_test.lua) — 既存コルーチン統合テスト
