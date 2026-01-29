# Research & Design Decisions: lua-stdlib-config

## Summary
- **Feature**: `lua-stdlib-config`
- **Discovery Scope**: Extension（既存RuntimeConfigシステムの拡張）
- **Key Findings**:
  1. mlua 0.11の`StdLib::ALL_SAFE`はdebug以外すべてのLua標準ライブラリを含む（io, os, packageもsafe扱い）
  2. 既存RuntimeConfigの個別フラグ（`enable_std_libs`, `enable_testing`等）を`libs`配列に統合することで、Cargo.toml風の直感的な設定が可能
  3. RuntimeConfig主導型アプローチ（Approach C-2）により、変換ロジックの単体テストとAPI利便性を両立

## Research Log

### mlua StdLibフラグ構造
- **Context**: `StdLib::ALL_SAFE`が何を含むか確認
- **Sources Consulted**: 
  - https://docs.rs/mlua/0.11/mlua/struct.StdLib.html
  - mlua GitHubソースコード（lib.rs StdLib定義）
- **Findings**:
  - `ALL_SAFE = StdLib((1 << 30) - 1)` = DEBUG と FFI を除く全ライブラリ
  - `ALL = u32::MAX` = DEBUG と FFI を含む全ライブラリ
  - `DEBUG = 1 << 31` = 唯一のunsafeライブラリ（Lua 5.5）
  - io, os, packageはsafeライブラリとして`ALL_SAFE`に含まれる
- **Implications**: 
  - `std_all` = `ALL_SAFE`（安全なデフォルト）
  - `std_all_unsafe` = `ALL`（debug含む、開発用）
  - 個別ライブラリはビット演算でOR結合

### mlua-stdlib モジュール構造
- **Context**: mlua-stdlib v0.1のモジュール登録パターン確認
- **Sources Consulted**: 
  - Cargo.toml（mlua-stdlib features: json, regex, yaml）
  - 既存runtime/mod.rsのモジュール登録コード
- **Findings**:
  - 各モジュールは`mlua_stdlib::<module>::register(&lua, None)`で登録
  - @assertions, @testing はコア機能（features不要）
  - @env はセキュリティ上opt-in
  - @regex, @json, @yaml はCargo features有効時のみ利用可能
- **Implications**:
  - libs配列で`testing`, `regex`等を指定 → 対応するモジュール登録
  - 減算記法`-testing`で登録スキップ

### 既存RuntimeConfig構造
- **Context**: 現在のフラグベース設計の理解
- **Sources Consulted**: 
  - crates/pasta_lua/src/runtime/mod.rs
  - crates/pasta_lua/src/loader/config.rs
- **Findings**:
  - RuntimeConfig: 7つの個別boolフラグ
  - PastaConfig: `[loader]`, `[logging]`, `[persistence]`セクション対応
  - `custom_fields: toml::Table`で未知セクション保持
- **Implications**:
  - `[lua]`セクション追加は既存パターンに準拠
  - RuntimeConfigのフラグを`libs`配列で完全置換

### TOMLパーサー統合
- **Context**: serde + tomlによる配列解析
- **Sources Consulted**: 
  - toml 0.9.8 documentation
  - 既存LoaderConfig, LoggingConfig実装
- **Findings**:
  - `#[serde(default)]`でデフォルト値適用
  - `Vec<String>`は`libs = ["a", "b"]`形式で自然にデシリアライズ
  - バリデーションはデシリアライズ後に実施
- **Implications**:
  - `LuaConfig`構造体を追加し`PastaConfig::lua()`メソッドで取得
  - デフォルト値関数で`["std_all", ...]`を返す

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 完全置換 | RuntimeConfigの全フラグを`libs`配列に統合 | 設計統一性、拡張性、Cargo風 | 既存テスト影響 | **採用** |
| Option B: 段階的移行 | 既存フラグ残しつつ`libs`優先 | 後方互換 | 二重管理、コード複雑化 | 不採用 |
| Approach C-1: LoaderConfig主導 | 変換ロジックをLoaderに配置 | 責務分離 | API利便性低下 | 不採用 |
| Approach C-2: RuntimeConfig主導 | 変換ロジックをRuntimeConfigに集約 | テスタビリティ、API利便性 | 責務境界曖昧 | **採用** |

## Design Decisions

### Decision: RuntimeConfig構造の刷新

- **Context**: 既存の7つの個別boolフラグをどう扱うか
- **Alternatives Considered**:
  1. 個別フラグ残存 + libs配列追加（段階的移行）
  2. 個別フラグ完全廃止 + libs配列のみ（完全置換）
- **Selected Approach**: 完全置換（Option A）
- **Rationale**: 
  - 設計の一貫性（1つの配列で全ライブラリ制御）
  - 技術的負債削減（二重管理回避）
  - Cargo.toml featuresとの高い親和性
- **Trade-offs**: 
  - ✅ シンプルな設計
  - ✅ 将来の拡張性
  - ❌ 既存テストの修正必要
- **Follow-up**: 既存テストは`RuntimeConfig::default()`使用のため影響軽微

### Decision: 変換ロジック配置

- **Context**: `libs`配列から`StdLib`への変換をどこで行うか
- **Alternatives Considered**:
  1. LoaderConfig（C-1）: 設定ロード時に変換
  2. RuntimeConfig（C-2）: ランタイム構築時に変換
- **Selected Approach**: RuntimeConfig主導型（C-2）
- **Rationale**:
  - RuntimeConfigだけで単体テスト可能
  - プログラムからの直接構築が容易
  - `Default::default()`パターンとの親和性
- **Trade-offs**:
  - ✅ テスタビリティ向上
  - ✅ API利便性
  - ❌ RuntimeConfigが設定解析責務を持つ
- **Follow-up**: LoaderConfigは`LuaConfig` → `RuntimeConfig`変換のみ

### Decision: デフォルト値

- **Context**: `libs`省略時の動作
- **Alternatives Considered**:
  1. 最小構成（空配列）
  2. 既存動作維持（全safe + mlua-stdlib）
- **Selected Approach**: 既存動作維持
- **Rationale**: 後方互換性、ユーザー移行コスト最小化
- **Default Value**: `["std_all", "assertions", "testing", "regex", "json", "yaml"]`
- **Note**: `env`はセキュリティ上デフォルト無効

### Decision: 減算記法の処理順序

- **Context**: `["std_all", "-std_debug"]`の処理方法
- **Selected Approach**: 加算を先に処理し、減算を後に処理
- **Rationale**: 
  - 順序に依存しない直感的な動作
  - `["std_all", "-std_debug"]` = `["-std_debug", "std_all"]` 同等
- **Implementation**: 
  ```rust
  let mut add = StdLib::NONE;
  let mut sub = StdLib::NONE;
  for lib in libs {
      if lib.starts_with('-') { sub |= parse(lib) }
      else { add |= parse(lib) }
  }
  result = add & !sub
  ```

### Decision: バリデーション戦略

- **Context**: 不明なライブラリ名への対応
- **Selected Approach**: 厳格（エラー返却）
- **Rationale**: 設定ミスの早期検出、タイポ防止
- **Error Type**: `ConfigError::UnknownLibrary(String)`

## Risks & Mitigations
- **既存テスト影響** — RuntimeConfig::default()がlibs配列に変更されるが、動作は同等のため影響軽微
- **std_debug誤設定** — 有効化時にtracing::warn出力で注意喚起
- **env誤設定** — デフォルト無効 + 有効化時に警告

## References
- [mlua StdLib documentation](https://docs.rs/mlua/0.11/mlua/struct.StdLib.html)
- [mlua-stdlib crate](https://docs.rs/mlua-stdlib/0.1/)
- [toml crate documentation](https://docs.rs/toml/0.9/)
- [gap-analysis.md](./gap-analysis.md) - 詳細なギャップ分析
