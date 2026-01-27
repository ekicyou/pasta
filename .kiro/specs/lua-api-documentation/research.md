# Research & Design Decisions

## Summary
- **Feature**: `lua-api-documentation`
- **Discovery Scope**: Simple Addition（ドキュメント作成のみ、コード変更なし）
- **Key Findings**:
  - pasta_luaは4つのカスタムモジュール（`@pasta_search`, `@pasta_config`, `@pasta_persistence`, `@enc`）を公開
  - mlua-stdlibから6つの追加モジュール（`@assertions`, `@testing`, `@regex`, `@json`, `@yaml`, `@env`）を統合
  - `finalize_scene()`はRust側バインディングとしてLua stubを上書きする特殊な関数

## Research Log

### Lua公開モジュールの完全リスト調査
- **Context**: Rust側からLuaに公開されているすべてのモジュールを特定する必要があった
- **Sources Consulted**: 
  - [runtime/mod.rs](../../crates/pasta_lua/src/runtime/mod.rs) - RuntimeConfig, モジュール登録
  - [search/mod.rs](../../crates/pasta_lua/src/search/mod.rs) - @pasta_search登録
  - [runtime/persistence.rs](../../crates/pasta_lua/src/runtime/persistence.rs) - @pasta_persistence登録
  - [runtime/enc.rs](../../crates/pasta_lua/src/runtime/enc.rs) - @enc登録
  - [runtime/finalize.rs](../../crates/pasta_lua/src/runtime/finalize.rs) - finalize_scene登録
- **Findings**:
  - pasta_luaカスタムモジュール: `@pasta_search`, `@pasta_config`, `@pasta_persistence`, `@enc`
  - mlua-stdlib統合: `@assertions`, `@testing`, `@env`, `@regex`, `@json`, `@yaml`
  - `@env`はセキュリティ上デフォルト無効
- **Implications**: ドキュメントは2つのカテゴリ（pasta_lua固有 / mlua-stdlib統合）に分けて構成すべき

### API署名の抽出
- **Context**: 各モジュールの公開関数シグネチャを正確に文書化する
- **Sources Consulted**:
  - [search/context.rs](../../crates/pasta_lua/src/search/context.rs) - UserData methods
  - [runtime/persistence.rs](../../crates/pasta_lua/src/runtime/persistence.rs) - load/save関数
  - [runtime/enc.rs](../../crates/pasta_lua/src/runtime/enc.rs) - to_ansi/to_utf8関数
- **Findings**:
  - `@pasta_search`: `search_scene(name, global_scene_name?)`, `search_word(name, global_scene_name?)`, `set_scene_selector(...)`, `set_word_selector(...)`
  - `@pasta_persistence`: `load()`, `save(data)`, `_VERSION`, `_DESCRIPTION`
  - `@enc`: `to_ansi(utf8_str)`, `to_utf8(ansi_str)`, `_VERSION`, `_DESCRIPTION`
- **Implications**: 全関数のシグネチャ、戻り値、エラーケースを文書化可能

### 既存ドキュメント構造の確認
- **Context**: 新規ドキュメントを既存体系と整合させる
- **Sources Consulted**: [crates/pasta_lua/README.md](../../crates/pasta_lua/README.md)
- **Findings**:
  - README.mdは日本語で記述
  - ディレクトリ構成、設定ファイル形式、アーキテクチャ概要を含む
  - APIリファレンスセクションは未存在
- **Implications**: README.mdから新規APIリファレンスへのリンクを追加、または別ファイル（LUA_API.md等）として作成

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 単一ファイル（LUA_API.md） | 全モジュールを1つのMarkdownに集約 | シンプル、検索しやすい | 長大化の可能性 | 推奨: 現時点のモジュール数なら管理可能 |
| 複数ファイル分割 | モジュールごとに個別ファイル | 拡張性が高い | 散在してナビゲーションが複雑化 | 将来的に検討 |
| README.md内セクション | 既存READMEに追記 | ファイル増加なし | READMEが肥大化 | 非推奨: 既に328行 |

## Design Decisions

### Decision: 単一ファイル（LUA_API.md）での作成
- **Context**: API参照ドキュメントの配置場所を決定
- **Alternatives Considered**:
  1. README.mdへの追記 — 既に長大（328行）
  2. 複数ファイル分割 — 現時点では過剰
- **Selected Approach**: `crates/pasta_lua/LUA_API.md` として新規作成
- **Rationale**: モジュール数（約10）は単一ファイルで管理可能、README.mdからリンク
- **Trade-offs**: 将来モジュール増加時には分割を検討
- **Follow-up**: README.mdに参照リンクを追加

### Decision: 日本語での記述
- **Context**: ドキュメント言語の選択
- **Alternatives Considered**:
  1. 英語 — 国際的だが主要ユーザーは日本語話者
  2. 日本語 — 既存README.mdに合わせる
- **Selected Approach**: 日本語（spec.json.language = "ja"）
- **Rationale**: 既存ドキュメント体系との一貫性、ターゲットユーザーのニーズ
- **Trade-offs**: 国際ユーザーには不便
- **Follow-up**: 必要に応じて英語版を別途作成

## Risks & Mitigations
- Risk 1: APIシグネチャの変更時にドキュメントが陳腐化 → 変更時にドキュメント更新をチェックリスト化
- Risk 2: mlua-stdlibの外部ドキュメントリンク切れ → 定期的なリンクチェック

## References
- [mlua-stdlib GitHub](https://github.com/mluastdlib/mlua-stdlib) — 外部モジュールの公式ドキュメント
- [mlua Documentation](https://docs.rs/mlua) — Lua-Rustバインディングリファレンス
