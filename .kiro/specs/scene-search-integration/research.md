# Research & Design Decisions: scene-search-integration

---
**Purpose**: `SCENE.search()` 関数実装に関する調査結果と設計決定の記録

---

## Summary
- **Feature**: scene-search-integration
- **Discovery Scope**: Extension（既存 scene.lua への機能追加）
- **Key Findings**:
  1. `@pasta_search` は PastaLuaRuntime 初期化時に最初に登録され、scene.lua ロード時には常に利用可能
  2. 既存の `SCENE.get()` を内部で活用可能（シーン関数取得ロジックの再利用）
  3. `__call` メタメソッドによりテーブルを関数として呼び出し可能にするLuaパターンが既存コードベースと整合

## Research Log

### Topic: @pasta_search モジュールの初期化順序
- **Context**: scene.lua が `@pasta_search` を require できるか確認が必要
- **Sources Consulted**: 
  - `crates/pasta_lua/src/runtime/mod.rs` - PastaLuaRuntime 初期化コード
  - `crates/pasta_lua/src/search/context.rs` - SearchContext UserData 実装
- **Findings**:
  - `PastaLuaRuntime::with_config()` で `@pasta_search` が最初に登録される
  - 初期化順序: `@pasta_search` 登録 → スクリプトロード
  - `package.loaded["@pasta_search"]` に SearchContext UserData が登録される
- **Implications**: 
  - pcall 不要、直接 `require("@pasta_search")` で問題なし
  - テスト時は必ず PastaLuaRuntime を使用する必要あり

### Topic: SearchContext API シグネチャ
- **Context**: `SEARCH:search_scene()` の戻り値形式を確認
- **Sources Consulted**: 
  - `crates/pasta_lua/src/search/context.rs` (ll.50-130)
- **Findings**:
  - `search_scene(name, global_scene_name?)` → `(global_name, local_name) | nil`
  - `global_scene_name` が nil の場合はグローバル検索のみ
  - ローカル検索時はフォールバック戦略（ローカル→グローバル）を内部で処理
  - fn_name パース: `"メイン_1::選択肢_1"` → `("メイン_1", "__選択肢_1__")`
- **Implications**: 
  - 戻り値をそのまま `SCENE.get()` に渡せる
  - `__start__` は特別扱い（そのまま返却）

### Topic: 既存の類似パターン
- **Context**: `@` プレフィックスモジュールの require パターン
- **Sources Consulted**: 
  - `crates/pasta_lua/scripts/pasta/save.lua`
- **Findings**:
  ```lua
  local persistence = require("@pasta_persistence")
  local save = persistence.load()
  ```
  - モジュールスコープで即座に require（遅延ロードなし）
- **Implications**: 
  - 同じパターンを `@pasta_search` にも適用可能
  - scene.lua の既存スタイルと整合

### Topic: Lua メタテーブルパターン
- **Context**: 関数として呼び出し可能なテーブルの実装
- **Sources Consulted**: 
  - Lua 5.4 マニュアル - Metatables and Metamethods
  - 既存 scene.lua の `scene_table_mt` 実装
- **Findings**:
  - `__call` メタメソッドでテーブルを関数のように呼び出し可能
  - 既存の `scene_table_mt` は `__index` のみ使用
  - 新規に `scene_result_mt` を追加してもコンフリクトなし
- **Implications**: 
  - `setmetatable({...}, scene_result_mt)` で実装
  - 呼び出し時: `result(act, ...)` が `result.func(act, ...)` に変換

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: scene.lua 直接追加 | 既存モジュールに `search()` 関数を追加 | 最小変更、単一責任維持、約20行 | scene.lua が @pasta_search に依存 | **採用** |
| B: ヘルパーモジュール | 新規 search_helper.lua を作成 | 責務分離 | ファイル追加、require チェーン増加 | 過剰設計 |
| C: act.lua 統合 | act:call_dynamic() を追加 | ワンステップ呼び出し | 要件スコープ外、2ファイル変更 | 将来検討 |

## Design Decisions

### Decision: 直接 require（pcall 不使用）
- **Context**: `@pasta_search` の require 時にエラーハンドリングが必要か
- **Alternatives Considered**:
  1. `pcall(require, "@pasta_search")` でフォールバック
  2. 直接 `require("@pasta_search")`
- **Selected Approach**: 直接 require
- **Rationale**: 
  - `@pasta_search` は PastaLuaRuntime で必ず登録される
  - pcall は末尾再帰最適化を阻害する（Lua パフォーマンス考慮）
  - テスト時は PastaLuaRuntime 使用が前提
- **Trade-offs**: 
  - ✅ シンプル、高速
  - ❌ PastaLuaRuntime 以外では使用不可（設計上許容）
- **Follow-up**: テストで PastaLuaRuntime を使用することを確認

### Decision: `__call` メタメソッドによる呼び出し可能テーブル
- **Context**: `SCENE.search()` の戻り値形式
- **Alternatives Considered**:
  1. シーン関数のみ返却（メタデータなし）
  2. `{global_name, local_name, func}` テーブル（呼び出し不可）
  3. `{global_name, local_name, func}` + `__call` メタメソッド
- **Selected Approach**: Option 3（`__call` メタメソッド付きテーブル）
- **Rationale**: 
  - 関数として直接呼び出し可能 (`result(act, ...)`)
  - デバッグ情報へのアクセスも可能 (`result.global_name`)
  - 両方のユースケースを満たす
- **Trade-offs**: 
  - ✅ 使いやすさとデバッグ性の両立
  - ✅ 将来の拡張性（メタデータ追加可能）
  - ❌ メタテーブル1つ追加（最小限のオーバーヘッド）
- **Follow-up**: メタテーブルはモジュールスコープで1回のみ定義

### Decision: scene.lua への直接追加（Option A）
- **Context**: 実装場所の選択
- **Alternatives Considered**:
  1. scene.lua 直接追加
  2. 新規ヘルパーモジュール
  3. act.lua 統合
- **Selected Approach**: Option A
- **Rationale**: 
  - 最小変更（約20行追加）
  - シーン検索はシーンモジュールの責務として自然
  - 既存 API と統一されたインターフェース
- **Trade-offs**: 
  - ✅ シンプル、保守しやすい
  - ✅ 既存パターンとの整合性
  - ❌ scene.lua の依存関係が増加（許容範囲）
- **Follow-up**: なし

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| テストで PastaLuaRuntime 未使用 | Low | High | テストガイドラインで明記 |
| search_scene の戻り値形式変更 | Very Low | Medium | Rust側APIは安定 |
| パフォーマンス低下 | Very Low | Low | メタテーブルオーバーヘッドは最小 |
