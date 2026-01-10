# Research & Design Decisions

## Summary
- **Feature**: `pasta-lua-startup-sequence`
- **Discovery Scope**: Extension（既存のLuaTranspiler/PastaLuaRuntimeを統合する新規loaderモジュール）
- **Key Findings**:
  - 既存コンポーネント（LuaTranspiler, PastaLuaRuntime, TranspileContext）は複数ファイル統合に対応可能
  - `glob` + `toml` + `serde` 依存関係は追加済み
  - Luaの `package.path` 設定は既存テストコードにパターンあり

## Research Log

### globパターン実装
- **Context**: Requirement 1で`dic/*/*.pasta`パターン探索が必要
- **Sources Consulted**: 
  - pasta_runeの既存実装（`loader.rs`）
  - glob 0.3 ドキュメント
- **Findings**:
  - `glob::glob()` でパターンマッチングを実行
  - Windows/Unix両対応（パス区切り文字は自動処理）
  - エラーハンドリングは `GlobError` を `LoaderError` にラップ
- **Implications**: pasta_runeの実装パターンを踏襲し、一貫性を確保

### TOML設定ファイル解析
- **Context**: Requirement 2で`pasta.toml`解析が必要
- **Sources Consulted**:
  - toml 0.9.8 ドキュメント
  - serde derive ドキュメント
- **Findings**:
  - `#[serde(flatten)]` で未知のフィールドを `toml::Table` にキャッチ可能
  - `#[serde(default)]` でオプショナルフィールドにデフォルト値設定
  - `toml::from_str()` で文字列からデシリアライズ
- **Implications**: 
  - `PastaConfig` 構造体に `#[derive(Deserialize)]` 使用
  - `[loader]` セクション以外は `custom_fields: toml::Table` で受け取り

### Luaモジュール検索パス設定
- **Context**: Requirement 4で`package.path`設定が必要
- **Sources Consulted**:
  - 既存テストコード `lua_unittest_runner.rs`
  - mlua ドキュメント
- **Findings**:
  - `lua.globals().set("package", ...)` でpackageテーブルにアクセス
  - `package.path` は `;` 区切りの文字列（Windows/Unix共通）
  - 既存 `package.path` に追加する形式がベストプラクティス
- **Implications**: 
  - 起動ディレクトリを絶対パスで保持
  - 4階層のパスを連結して `package.path` にプリペンド

### toml::Value から mlua::Value への変換
- **Context**: Requirement 7で`@pasta_config`モジュール実装が必要
- **Sources Consulted**:
  - mlua serialize feature ドキュメント
  - toml::Value API
- **Findings**:
  - mlua `serialize` feature を有効化済み
  - `toml::Value` を手動で `mlua::Value` に変換する関数が必要
  - 再帰的に Table, Array, String, Integer, Float, Boolean, Datetime を変換
- **Implications**:
  - `fn toml_to_lua(lua: &Lua, value: &toml::Value) -> mlua::Result<mlua::Value>` を実装
  - Datetime は文字列として表現

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 既存拡張 | PastaLuaRuntime にディレクトリ探索を追加 | 変更箇所少 | 責務過多、テスト困難 | 却下 |
| Option B: 新規loaderモジュール | PastaLoader を新規作成、既存を利用 | 責務分離、テスト容易 | コード増加 | **採用** |

**選択理由**: Option B - 既存コンポーネントの責務を変更せず、新規 `loader` モジュールで統合ロジックを実装。tech.mdの「Engine層」に相当。

## Design Decisions

### Decision: ローダーモジュール構造

- **Context**: 統合起動APIをどこに配置するか
- **Alternatives Considered**:
  1. `runtime/mod.rs` に追加 — ランタイム責務と混在
  2. `loader.rs` 新規作成 — 単一ファイル構成
  3. `loader/` ディレクトリ新規作成 — サブモジュール分割
- **Selected Approach**: `loader/` ディレクトリ新規作成
- **Rationale**: 
  - `loader/mod.rs` - PastaLoader 公開API
  - `loader/config.rs` - PastaConfig, LoaderConfig
  - `loader/error.rs` - LoaderError
  - `loader/discovery.rs` - ファイル探索
- **Trade-offs**: ファイル数増加 vs 責務分離・テスト容易性
- **Follow-up**: 必要に応じてサブモジュールを統合可能

### Decision: PastaLuaRuntime API拡張

- **Context**: ランタイム初期化時に`package.path`と`@pasta_config`を設定する必要
- **Alternatives Considered**:
  1. `PastaLuaRuntime::new()` シグネチャ変更 — 既存コード破壊
  2. 新規 `with_loader_context()` メソッド追加 — 追加APIのみ
  3. `RuntimeConfig` 拡張 — 設定が肥大化
- **Selected Approach**: 新規ファクトリメソッド追加
  - `PastaLuaRuntime::from_loader(context, loader_context) -> LuaResult<Self>`
- **Rationale**: 既存API互換性を維持しつつ、loaderからの初期化パスを追加
- **Trade-offs**: API表面積増加 vs 互換性維持
- **Follow-up**: ドキュメント更新、マイグレーションガイド

### Decision: エラー型設計

- **Context**: LoaderError の型階層設計
- **Alternatives Considered**:
  1. 単一 `LoaderError` enum — シンプル
  2. 既存 `TranspileError` を拡張 — 型混在
  3. 新規 `LoaderError` + From トレイト — 相互変換
- **Selected Approach**: 新規 `LoaderError` enum + From トレイト
- **Rationale**: 
  - `LoaderError::Io(PathBuf, io::Error)` — ファイルIO
  - `LoaderError::Config(PathBuf, toml::de::Error)` — 設定解析
  - `LoaderError::Parse(PathBuf, pasta_core::ParseError)` — パース
  - `LoaderError::Transpile(TranspileError)` — トランスパイル
  - `LoaderError::Runtime(mlua::Error)` — Luaランタイム
- **Trade-offs**: エラー型増加 vs 診断情報の充実
- **Follow-up**: Display実装でユーザーフレンドリーなメッセージ

## Risks & Mitigations
- **Risk**: 複数ファイルトランスパイル時のメモリ使用量増加
  - **Mitigation**: 各ファイルを順次処理、生成コードは即座にファイル書き込み
- **Risk**: Windowsパス区切り文字の取り扱い
  - **Mitigation**: `std::path::Path` APIを使用、手動文字列操作を避ける
- **Risk**: 既存テストへの影響
  - **Mitigation**: 新規API追加のみ、既存シグネチャ変更なし

## References
- [glob crate docs](https://docs.rs/glob/0.3/glob/)
- [toml crate docs](https://docs.rs/toml/0.9/toml/)
- [serde derive docs](https://serde.rs/derive.html)
- [mlua docs](https://docs.rs/mlua/0.11/mlua/)
