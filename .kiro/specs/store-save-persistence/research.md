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
- **Context**: 難読化とファイルサイズ削減を両立する方式調査
- **Sources Consulted**: 
  - flate2 (gzip/deflate圧縮) - crates.io
  - zstd (Zstandard圧縮) - crates.io
- **Findings**:
  - XOR難読化よりも圧縮の方が実用的（難読化＋サイズ削減）
  - flate2: rust-lang公式、最終更新14日前、活発なメンテナンス
  - zstd: 高圧縮率だが最終更新11ヶ月前、個人メンテナ
  - flate2はRustエコシステムで最も標準的な圧縮ライブラリ
- **Implications**: flate2によるgzip圧縮を採用（JSON → gzip → .datファイル）

## Architecture Pattern Evaluation

| Option            | Description                    | Strengths                    | Risks / Limitations           | Notes    |
| ----------------- | ------------------------------ | ---------------------------- | ----------------------------- | -------- |
| A: Runtime層統合  | persistence.rsをruntime/に追加 | enc.rsと同じパターン、一貫性 | runtime層が肥大化             | **採用** |
| B: Loader層統合   | persistence.rsをloader/に追加  | 設定との親和性               | ロード/セーブの責務分離が曖昧 | 不採用   |
| C: 独立モジュール | persistence/ディレクトリを新設 | 責務明確                     | 過剰な分離                    | 将来検討 |

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

### Decision: 難読化と圧縮の統合
- **Context**: 保存データの難読化要件とファイルサイズ削減
- **Alternatives Considered**:
  1. 暗号学的暗号化 — 追加クレート必要、過剰
  2. XOR難読化 — 軽量だがファイルサイズ削減なし
  3. gzip圧縮 — 難読化とサイズ削減を両立
  4. zstd圧縮 — 高圧縮率だがメンテナンス懸念
- **Selected Approach**: flate2によるgzip圧縮
- **Rationale**: 
  - テキストエディタで開いても読めない（難読化要件を満たす）
  - 70-80%のファイルサイズ削減
  - rust-lang公式で活発にメンテナンス
  - 追加依存は軽量（純Rustバックエンド）
- **Trade-offs**: 暗号学的安全性はないが、要件として不要
- **Follow-up**: なし

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
