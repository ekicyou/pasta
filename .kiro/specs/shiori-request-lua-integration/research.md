# Research & Design Decisions

## Summary
- **Feature**: `shiori-request-lua-integration`
- **Discovery Scope**: Extension（既存システムへの拡張）
- **Key Findings**:
  - lua_request.rsは既に119行の機能的なコードが存在し、主要な修正はインポート変更とライフタイム削除
  - `time` crate v0.3.45 は `local-offset` feature で `OffsetDateTime::now_local()` を提供（Result型）
  - mlua 0.11+では`<'lua>`ライフタイムが不要、`Table`型はライフタイムパラメータなし

## Research Log

### time crate API調査
- **Context**: Requirement 2 の現在時刻テーブル生成に必要なDateTime依存の選定
- **Sources Consulted**:
  - https://crates.io/crates/time - メタデータ確認
  - https://docs.rs/time/latest/time/struct.OffsetDateTime.html - API仕様
- **Findings**:
  - `OffsetDateTime::now_local()` は `Result<OffsetDateTime, IndeterminateOffset>` を返却
  - 各コンポーネントメソッド: `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`, `nanosecond()`, `ordinal()`, `weekday()`
  - `month()`は`Month` enum、`weekday()`は`Weekday` enum（数値変換必要）
  - `local-offset` feature flagが必須
- **Implications**: 
  - chronoの`Local::now()`と異なりResult型なのでエラーハンドリングが必要
  - enumから数値への変換ロジックが追加で必要

### mlua ライフタイム調査
- **Context**: 移植元コードの`<'lua>`ライフタイムが現行mluaで必要か確認
- **Sources Consulted**:
  - pasta_lua::mlua 再エクスポート構造（lib.rs）
  - mlua 0.11 API ドキュメント
- **Findings**:
  - mlua 0.11+ではTable, Functionなどにライフタイムパラメータが不要
  - 既存のshiori.rsでは`Table`を直接使用（ライフタイムなし）
  - `lua.create_table()` の戻り値は`mlua::Result<Table>`
- **Implications**:
  - 全ての`<'lua>`を削除可能
  - `LuaTable<'lua>` → `Table` に変更

### 既存パーサー統合ポイント調査
- **Context**: lua_request.rsが使用するパーサーの確認
- **Sources Consulted**:
  - `crates/pasta_shiori/src/util/parsers/req.rs` - 既存パーサー
  - `crates/pasta_shiori/src/lua_request.rs` - 移植元コード
- **Findings**:
  - 現在のコードは直接`Parser::parse(Rule::req, text)`を使用
  - `ShioriRequest::parse(text)`という高レベルAPIも存在するが未使用
  - Rule, Parserは`crate::util::parsers::req`から既にインポート可能
- **Implications**:
  - parse_request関数内でのパーサー使用パターンは維持可能
  - 低レベルPest APIを直接使用する現行アプローチを継続

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 直接修正 | 既存lua_request.rsを修正・統合 | 最小変更、既存パターン活用 | なし | **選択** - gap-analysisで推奨 |
| Option B: 新モジュール | util::lua_helpers作成 | 明確な責任分離 | ファイル数増加 | 将来的リファクタリングで検討 |

## Design Decisions

### Decision: time crateの採用
- **Context**: Requirement 2の現在時刻テーブル生成にDateTime依存が必要
- **Alternatives Considered**:
  1. chrono - 歴史あるが過去にセキュリティ問題、メンテナンス不安定期あり
  2. time - モダン設計、活発なメンテナンス、軽量
- **Selected Approach**: time v0.3.x + local-offset feature
- **Rationale**: 
  - 520M DL、125バージョン、7日前更新と活発
  - ローカルオフセット取得がResult型で安全
  - 17K SLoC vs chrono 20K SLoCで軽量
- **Trade-offs**: 
  - `now_local()`がResultを返すためエラーハンドリング追加（chronoより冗長）
  - Month/Weekday enumから数値への変換が必要
- **Follow-up**: Cargo.tomlへの依存追加、MyErrorへの変換実装

### Decision: `<'lua>`ライフタイム削除
- **Context**: 移植元コードのライフタイム記法が現行mluaで不要
- **Alternatives Considered**:
  1. 保持 - 安全確実だが冗長
  2. 削除 - モダン、シンプル
- **Selected Approach**: 全ての`<'lua>`を削除
- **Rationale**: mlua 0.11+ API DOCにライフタイム記載なし、既存shiori.rsでも使用していない
- **Trade-offs**: コンパイルエラー発生時は再検討（低リスク）
- **Follow-up**: 削除後のコンパイル確認

### Decision: エラーハンドリング戦略
- **Context**: time::IndeterminateOffset等の新エラー型をMyErrorに統合
- **Alternatives Considered**:
  1. 新MyError variant追加 - 明確だが変更範囲増
  2. 既存Script variantで代用 - 変更最小
- **Selected Approach**: 実装時に適切なvariantへ変換（Script variant推奨）
- **Rationale**: 時刻取得エラーは稀、専用variantは過剰
- **Trade-offs**: エラー型の意味が若干曖昧になる
- **Follow-up**: 実装時に具体的なマッピング決定

## Risks & Mitigations
- **Risk 1**: time crate local-offset がマルチスレッド環境で問題発生 → pasta_shioriはDLL単一スレッド想定、低リスク
- **Risk 2**: ライフタイム削除後にコンパイルエラー → 段階的に削除し検証、容易に戻せる
- **Risk 3**: 既存パーサーAPIとの互換性問題 → 既にコード内で使用済み、低リスク

## References
- [time crate documentation](https://docs.rs/time/latest/time/) - OffsetDateTime API
- [mlua documentation](https://docs.rs/mlua/latest/mlua/) - Table型仕様
- [Rust 2024 Edition](https://doc.rust-lang.org/edition-guide/rust-2024/) - 言語仕様
