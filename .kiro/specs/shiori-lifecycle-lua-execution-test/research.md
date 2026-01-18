# Research & Design Decisions

## Summary
- **Feature**: `shiori-lifecycle-lua-execution-test`
- **Discovery Scope**: Extension（既存テストインフラへのテスト追加）
- **Key Findings**:
  - 既存`copy_fixture_to_temp`ヘルパー関数がpasta_lua固有パスを使用 → pasta_shiori専用版を作成
  - `@pasta_search`モジュールは`SEARCH:search_scene(name, parent)`で`(global_name, local_name)`を返す
  - 既存`test_unload_called_on_drop`がファイルマーカーパターンを実証済み

## Research Log

### `@pasta_search` API調査
- **Context**: Requirement 4（Pasta DSL読み込み確認）の実装方法を特定するため
- **Sources Consulted**:
  - [search/mod.rs](../../crates/pasta_lua/src/search/mod.rs#L1-L75)
  - [search/context.rs](../../crates/pasta_lua/src/search/context.rs#L1-L200)
  - [search_module_test.rs](../../crates/pasta_lua/tests/search_module_test.rs#L67-L100)
- **Findings**:
  - `require "@pasta_search"`で`SearchContext` UserDataを取得
  - `SEARCH:search_scene(name, global_scene_name)` → `(global_name, local_name)` or `nil`
  - `global_name`は`"メイン_1"`形式、`local_name`は`"__start__"`または`"__選択肢_1__"`形式
  - シーン関数呼び出し: `_G[global_name][local_name]()`
- **Implications**: main.luaでシーン検索・呼び出し・応答生成が可能

### 既存フィクスチャ構造調査
- **Context**: Requirement 5（テストフィクスチャ整備）の設計を特定するため
- **Sources Consulted**:
  - [minimal/pasta.toml](../../crates/pasta_lua/tests/fixtures/loader/minimal/pasta.toml)
  - [minimal/dic/test/hello.pasta](../../crates/pasta_lua/tests/fixtures/loader/minimal/dic/test/hello.pasta)
  - [scripts/pasta/shiori/main.lua](../../crates/pasta_lua/scripts/pasta/shiori/main.lua)
- **Findings**:
  - フィクスチャ構造: `pasta.toml` + `dic/<module>/*.pasta`
  - `scripts/pasta/shiori/main.lua`は`copy_fixture_to_temp`でコピーされる
  - 既存main.luaは固定204応答のみ（観測可能な副作用なし）
- **Implications**: 専用フィクスチャには観測可能なSHIORI関数を静的定義

### copy_fixture_to_temp実装調査
- **Context**: テストヘルパー関数の再利用可能性評価
- **Sources Consulted**:
  - [shiori.rs](../../crates/pasta_shiori/src/shiori.rs#L310-L360)
- **Findings**:
  - `pasta_lua/tests/fixtures/loader`への相対パスがハードコード
  - `scripts/`、`scriptlibs/`は`pasta_lua`クレートからコピー
  - pasta_shiori専用フィクスチャには対応していない
- **Implications**: pasta_shiori/tests用のヘルパー関数を新規作成

### ファイルマーカーパターン調査
- **Context**: Requirement 3（SHIORI.unload検証）の実装パターン確認
- **Sources Consulted**:
  - [shiori.rs test_unload_called_on_drop](../../crates/pasta_shiori/src/shiori.rs#L615-L680)
- **Findings**:
  - `io.open(load_dir .. "/unload_called.marker", "w")`でマーカー作成
  - Rust側で`marker_path.exists()`を検証
  - `drop(shiori)`後もTempDirはスコープ内で有効
- **Implications**: 既存パターンを踏襲、Unicode path制約を記録

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: Integration Test | `tests/shiori_lifecycle_test.rs`に独立テスト | 責務分離、保守性 | ファイル数増加 | **採用** |
| B: Extend shiori.rs | 既存テストモジュールに追加 | 変更最小 | ファイル肥大化 | 不採用 |
| C: pasta_lua依存継続 | pasta_luaフィクスチャ使用 | 変更最小 | クロスクレート依存 | 不採用 |

## Design Decisions

### Decision: 静的フィクスチャ採用
- **Context**: Requirement 5でテストフィクスチャの配置方法を決定
- **Alternatives Considered**:
  1. 動的生成（テストコード内でLua生成）— 柔軟だが保守性低下
  2. 静的フィクスチャ（事前定義ファイル）— 可読性高い
- **Selected Approach**: 静的フィクスチャ
- **Rationale**: Luaコード・Pastaコードが事前にレビュー可能、変更追跡が容易
- **Trade-offs**: ファイル数増加 vs 可読性・保守性向上
- **Follow-up**: フィクスチャ変更時はテストも同時レビュー

### Decision: インテグレーションテスト配置
- **Context**: テストコードの配置場所を決定
- **Alternatives Considered**:
  1. `src/shiori.rs` mod tests — 既存パターン
  2. `tests/shiori_lifecycle_test.rs` — 独立テストファイル
- **Selected Approach**: `tests/shiori_lifecycle_test.rs`
- **Rationale**: E2Eライフサイクルテストは統合テストに相応しい
- **Trade-offs**: ファイル追加 vs 責務明確化
- **Follow-up**: common/mod.rsでヘルパー共有を検討

### Decision: ファイルマーカー方式維持
- **Context**: Requirement 3のSHIORI.unload検証方法
- **Alternatives Considered**:
  1. Luaグローバル変数 — runtimeドロップ後アクセス不可
  2. ファイルマーカー — 永続化により検証可能
- **Selected Approach**: ファイルマーカー
- **Rationale**: 既存test_unload_called_on_dropで実証済み
- **Trade-offs**: Unicode path制約あり（開発環境では問題なし）
- **Follow-up**: 本番Unicode path対応は別仕様で検討

## Risks & Mitigations
- **Lua io.open() Unicode path制約** — TempDirパスは通常ASCIIのため問題発生可能性低
- **@pasta_search API変更リスク** — テストがAPI変更の早期検出に貢献
- **フィクスチャ肥大化** — 最小構成を維持、必要最小限のシーン定義

## References
- [mlua crate documentation](https://docs.rs/mlua/) — Lua bindings for Rust
- [tempfile crate](https://docs.rs/tempfile/) — TempDir for test isolation
- [SHIORI/3.0 Protocol](https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html) — プロトコル仕様
