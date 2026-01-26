# Research & Design Decisions

## Summary
- **Feature**: `store-save-persistence`
- **Discovery Scope**: Extension（既存システムへの機能追加）
- **Key Findings**:
  - mluaの`serialize`フィーチャーがすでに有効化済み（Lua Value ↔ serde変換可能）
  - enc.rsモジュールパターンを参考にした`@pasta_persistence`モジュール実装
  - PastaShiori::Dropの既存パターンでDrop時保存を実装可能

## Research Log

### mluaのserializeフィーチャー
- **Context**: LuaテーブルをRust側でJSON/バイナリにシリアライズする方法の調査
- **Sources Consulted**: 
  - Cargo.toml workspace依存関係
  - mlua 0.11 ドキュメント
- **Findings**:
  - `mlua = { version = "0.11", features = ["lua54", "vendored", "serialize"] }` が既に設定済み
  - `mlua::LuaSerdeExt`トレイトで`lua.from_value::<T>(value)`と`lua.to_value(&data)`が使用可能
  - 追加クレートなしでLua Value ↔ serde_json::Value変換が可能
- **Implications**: serde_jsonを直接使用し、新規依存なしで実装可能

### 既存Rustモジュール登録パターン（enc.rs）
- **Context**: `@pasta_persistence`モジュールの実装パターン調査
- **Sources Consulted**: 
  - `crates/pasta_lua/src/runtime/enc.rs`
  - `crates/pasta_lua/src/runtime/mod.rs`
- **Findings**:
  - `register(&Lua) -> LuaResult<Table>`関数でモジュールテーブル作成
  - `package.loaded["@module_name"]`に登録
  - `_VERSION`, `_DESCRIPTION`メタデータを含める
  - エラーは`(Option<T>, Option<String>)`タプルで返す
- **Implications**: 同じパターンで`persistence.rs`を実装

### Drop時保存パターン（PastaShiori）
- **Context**: ランタイムDrop時の永続化処理パターン調査
- **Sources Consulted**: 
  - `crates/pasta_shiori/src/shiori.rs` Drop実装
- **Findings**:
  - `impl Drop for PastaShiori`でLua関数呼び出し後にリソース解放
  - `call_lua_unload()`でLua側の終了処理を呼び出し
  - エラーはログ出力のみ（パニックしない）
- **Implications**: PastaLuaRuntimeにDrop実装を追加し、`ctx.save`を保存

### 設定ファイルパターン（PastaConfig）
- **Context**: `[persistence]`セクションの追加方法調査
- **Sources Consulted**: 
  - `crates/pasta_lua/src/loader/config.rs`
- **Findings**:
  - `custom_fields: toml::Table`で任意セクションを保持
  - `PastaConfig::logging()`のようにヘルパーメソッドで抽出
  - `serde::Deserialize`でデシリアライズ
- **Implications**: `PastaConfig::persistence()`メソッドを追加

### 難読化方式の選定
- **Context**: 追加クレートなしでの難読化方式調査
- **Sources Consulted**: 
  - 既存依存（serde, serde_json）
  - 標準ライブラリ
- **Findings**:
  - XOR難読化 + Base64エンコードで簡易難読化可能
  - 標準ライブラリのみで実装可能（追加クレートなし）
  - Magic headerで形式判別可能
- **Implications**: JSON → UTF-8バイト → XOR（固定キー） → Base64で難読化

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: Runtime層統合 | persistence.rsをruntime/に追加 | enc.rsと同じパターン、一貫性 | runtime層が肥大化 | **採用** |
| B: Loader層統合 | persistence.rsをloader/に追加 | 設定との親和性 | ロード/セーブの責務分離が曖昧 | 不採用 |
| C: 独立モジュール | persistence/ディレクトリを新設 | 責務明確 | 過剰な分離 | 将来検討 |

## Design Decisions

### Decision: ファイル形式とパス
- **Context**: 永続化ファイルの形式と保存先の決定
- **Alternatives Considered**:
  1. TOML形式 — Luaテーブル構造との互換性に制限
  2. MessagePack — 追加クレート必要
  3. JSON形式 — 人間可読、既存依存で対応可能
- **Selected Approach**: 
  - 非難読化: JSON形式 (`.json`)
  - 難読化: XOR + Base64バイナリ (`.dat`)
- **Rationale**: JSON形式はデバッグ容易、serde_jsonが既に依存関係に含まれる
- **Trade-offs**: バイナリ形式より若干大きいが、可読性とデバッグ性を優先
- **Follow-up**: ファイルサイズが問題になる場合はMessagePack導入を検討

### Decision: 保存タイミング
- **Context**: いつ永続化ファイルに書き込むか
- **Alternatives Considered**:
  1. Drop時のみ — 異常終了時にデータ損失
  2. 定期保存 — 複雑性増加、パフォーマンス影響
  3. 明示的 + Drop — 柔軟性と安全性のバランス
- **Selected Approach**: `@pasta_persistence.save()`による明示的保存 + Drop時強制保存
- **Rationale**: スクリプト開発者に保存タイミング制御を提供しつつ、安全なフォールバック
- **Trade-offs**: 2箇所で保存ロジックが呼ばれる
- **Follow-up**: 重複保存を防ぐためのダーティフラグ検討

### Decision: 難読化レベル
- **Context**: 保存データの難読化要件
- **Alternatives Considered**:
  1. 暗号学的暗号化 — 追加クレート必要、過剰
  2. XOR難読化 — 軽量、カジュアル改ざん抑止
  3. Base64のみ — 容易に解読可能
- **Selected Approach**: XOR（固定キー） + Base64エンコード
- **Rationale**: 「テキストエディタで開いても読めない」程度で十分
- **Trade-offs**: 解析者には簡単に解読可能
- **Follow-up**: なし（要件として暗号学的安全性は不要と確認済み）

### Decision: モジュール設計
- **Context**: STORE.saveの代替設計
- **Alternatives Considered**:
  1. STORE.save維持 — 循環参照リスク
  2. ctx.save直接定義 — 初期化順序の問題
  3. pasta.save新規モジュール — クリーンな分離
- **Selected Approach**: `pasta.save`モジュール作成、`ctx.save = require "pasta.save"`
- **Rationale**: 循環参照回避、ロード順序制御、責務分離
- **Trade-offs**: STORE.saveからの移行が必要
- **Follow-up**: STORE.saveフィールドを削除（非推奨化不要、直接削除）

## Risks & Mitigations

- **Risk 1**: Drop時にLua VMがすでに無効な状態 → Luaアクセス前に状態チェック、エラーログのみ
- **Risk 2**: 大きなテーブルでのシリアライズ性能 → 通常使用では問題なし、必要時にストリーミング検討
- **Risk 3**: ファイル書き込み失敗 → アトミック書き込み（一時ファイル+リネーム）、エラーログ
- **Risk 4**: 難読化形式と非難読化形式の混在 → Magic headerで形式自動判別

## References

- [mlua 0.11 Serialize Feature](https://docs.rs/mlua/latest/mlua/serde/index.html) — LuaSerdeExtトレイト
- [serde_json](https://docs.rs/serde_json/latest/) — JSON シリアライズ
- [Rust Drop Trait](https://doc.rust-lang.org/std/ops/trait.Drop.html) — リソース解放パターン
