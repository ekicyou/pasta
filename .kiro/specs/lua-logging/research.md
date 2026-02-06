# Research & Design Decisions

## Summary
- **Feature**: `lua-logging`
- **Discovery Scope**: Extension（既存 `@pasta_*` モジュール登録パターンの拡張）
- **Key Findings**:
  - mlua 0.11 の `Lua::inspect_stack()` は Lua debug ライブラリ不要で呼び出し元情報を取得可能
  - tracing マクロは structured fields でランタイム変数を埋め込み可能（`lua_source`, `lua_line`, `lua_fn`）
  - 既存の GlobalLoggerRegistry + RoutingWriter により、tracing 経由で PastaLogger への自動ルーティングが実現済み

## Research Log

### mlua inspect_stack API
- **Context**: R2（呼び出し元情報の自動付与）の実現可能性調査
- **Sources Consulted**: [docs.rs/mlua/0.11 - Lua::inspect_stack](https://docs.rs/mlua/0.11/mlua/struct.Lua.html), Debug/DebugNames/DebugSource 各struct
- **Findings**:
  - `Lua::inspect_stack(level: usize, f: impl FnOnce(&Debug) -> R) -> Option<R>`
  - Lua C API (`lua_getinfo`) を直接使用、debug ライブラリ不要
  - `Debug::names()` → `DebugNames { name, name_what }` — 関数名
  - `Debug::source()` → `DebugSource { source, short_src, line_defined, last_line_defined, what }` — ソースファイル
  - `Debug::current_line()` → `Option<usize>` — 現在の行番号
  - スタックレベル: 0=現在のRust関数、1=Lua呼び出し元
- **Implications**: R2 の全 AC を完全にRust側から実現可能。debug ライブラリ依存なし。

### tracing structured fields
- **Context**: R4（Rust-side logging infrastructure への統合）での呼び出し元情報埋め込み方法
- **Sources Consulted**: [docs.rs/tracing - event macro](https://docs.rs/tracing/latest/tracing/macro.event.html), [Metadata](https://docs.rs/tracing/latest/tracing/struct.Metadata.html)
- **Findings**:
  - `tracing::info!(lua_source = %src, lua_line = line, lua_fn = %fn_name, "{}", msg)` でランタイム値を埋め込み可能
  - Metadata の file/line/module_path はコンパイル時固定（Rustのマクロ呼び出し箇所）
  - structured fields はサブスクライバーが自由にフォーマット可能
- **Implications**: Lua呼び出し元情報は structured fields として保持。tracing subscriber のフォーマットで表示。

### 既存 @pasta_* モジュール登録パターン
- **Context**: R3（モジュール登録と利用方法）の実装テンプレート確定
- **Sources Consulted**: `runtime/persistence.rs`, `runtime/enc.rs`, `runtime/mod.rs`
- **Findings**:
  - パターン: `pub fn register(lua: &Lua, ...) -> LuaResult<Table>`
  - テーブル作成 → `_VERSION`/`_DESCRIPTION` 設定 → クロージャ登録 → テーブル返却
  - `register_*_module()` で `package.loaded["@module"]` に登録
  - 状態はクロージャの upvalue として保持（`PersistenceState` パターン）
- **Implications**: `@pasta_log` も同じパターンに従う。状態は不要（ログ関数はステートレス）。

### PastaLogger ルーティング機構
- **Context**: R4 AC-3（既存ルーティング機構との統合）の確認
- **Sources Consulted**: `logging/registry.rs`, `logging/logger.rs`
- **Findings**:
  - `GlobalLoggerRegistry` がシングルトンとして `load_dir → PastaLogger` マッピングを管理
  - `RoutingWriter` が `MakeWriter` を実装し、thread-local `CURRENT_LOAD_DIR` でルーティング
  - tracing subscriber に `RoutingWriter` を接続すれば、`tracing::info!()` の出力が自動的にPastaLoggerに到達
  - PastaLogger 未登録の場合は no-op（`RoutingWriter` が silently discard）
- **Implications**: `@pasta_log` は tracing マクロを呼ぶだけで、PastaLogger への統合は自動的に実現される。追加の直接書き込みは不要。

### テーブルの構造化表示
- **Context**: R1 AC-4（複雑型の構造化形式展開）の方法選択
- **Sources Consulted**: `serde_json` (Cargo.toml確認済み)、mlua `LuaSerdeExt`
- **Findings**:
  - `serde_json` は既に pasta_lua の依存関係に存在
  - `lua.from_value::<serde_json::Value>(value)` で Lua テーブル → JSON 変換可能（persistence.rs の save_impl で使用実績あり）
  - 循環参照テーブルでは `serde_json` 変換が失敗する → フォールバック必要
- **Implications**: テーブルは `serde_json` で JSON 文字列に変換。変換失敗時は `tostring()` にフォールバック。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: mod.rs 内 inline 実装 | ログ関数を mod.rs に直接記述 | ファイル追加不要 | mod.rs がさらに肥大化（1152行） | ❌ 不採用 |
| B: 独立 runtime/log.rs | persistence.rs と同じパターンで分離 | 既存パターン準拠、テスト容易 | 特になし | ✅ 採用 |
| C: logging/ モジュール拡張 | 既存の logging/ ディレクトリに追加 | ログ関連コードの集約 | logging/ は Rust 側ロガー専用 | ❌ レイヤー混在 |

## Design Decisions

### Decision: 独立ファイル runtime/log.rs
- **Context**: `@pasta_log` モジュールの実装場所
- **Alternatives Considered**:
  1. mod.rs にインラインで実装
  2. runtime/log.rs として分離
  3. logging/ ディレクトリに配置
- **Selected Approach**: runtime/log.rs として分離
- **Rationale**: persistence.rs/enc.rs と同じレイヤー・同じパターンに従う。テスト容易性が高い。
- **Trade-offs**: ファイル追加されるが、mod.rs 肥大化を回避
- **Follow-up**: `mod log;` を mod.rs に追加、`register_log_module()` を実装

### Decision: JSON 形式でテーブル展開
- **Context**: R1 AC-4 のテーブル変換フォーマット選択
- **Alternatives Considered**:
  1. JSON（serde_json）
  2. YAML
  3. Lua inspect ライクなカスタムフォーマット
- **Selected Approach**: JSON（serde_json）
- **Rationale**: 既に依存関係に存在、persistence.rs で使用実績あり、Lua ↔ JSON 変換パスが確立済み
- **Trade-offs**: 深いネスト時の可読性がやや劣るが、ログ用途には十分
- **Follow-up**: 循環参照テーブルでの変換失敗時は `tostring()` フォールバック

### Decision: tracing マクロ経由の統一出力
- **Context**: R4 PastaLogger 統合方式
- **Alternatives Considered**:
  1. tracing マクロ + PastaLogger 直接書き込み（二重出力）
  2. tracing マクロ経由のみ（既存ルーティング活用）
  3. PastaLogger 直接書き込みのみ
- **Selected Approach**: tracing マクロ経由のみ
- **Rationale**: GlobalLoggerRegistry + RoutingWriter が自動的にルーティングする。二重出力問題を回避。
- **Trade-offs**: tracing subscriber の設定に依存するが、既存インフラで完結
- **Follow-up**: なし（既存機構をそのまま利用）

## Risks & Mitigations
- `inspect_stack(1)` のレベル値がクロージャ呼び出しの深さによって変わるリスク → 実装時にテストで検証
- `serde_json` 変換が大きなテーブルで遅延するリスク → ログ用途であり、パフォーマンスクリティカルではない。必要なら depth 制限を追加
- 循環参照テーブルでの `serde_json` パニック → `lua.from_value()` は `Err` を返すのでフォールバック可能

## References
- [mlua 0.11 - Lua::inspect_stack](https://docs.rs/mlua/0.11/mlua/struct.Lua.html#method.inspect_stack)
- [mlua 0.11 - Debug struct](https://docs.rs/mlua/0.11/mlua/struct.Debug.html)
- [tracing - event macro](https://docs.rs/tracing/latest/tracing/macro.event.html)
- [tracing - structured fields](https://docs.rs/tracing/latest/tracing/field/index.html)
- `crates/pasta_lua/src/runtime/persistence.rs` — モジュール登録テンプレート
- `crates/pasta_lua/src/logging/registry.rs` — GlobalLoggerRegistry + RoutingWriter
