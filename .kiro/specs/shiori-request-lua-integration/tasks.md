# Implementation Plan

## Overview
本タスクリストは、SHIORI requestをLuaテーブルに変換する機能をpasta_shioriクレートに統合するための実装計画です。既存の`lua_request.rs`を現行mlua環境に適合させ、time crateへの依存を追加し、統合テストを実装します。

---

## Tasks

- [ ] 1. 依存関係とモジュール宣言の準備
- [ ] 1.1 (P) Cargo.tomlにtime crateを追加
  - `crates/pasta_shiori/Cargo.toml`に`time = { version = "0.3", features = ["local-offset"] }`を追加
  - バージョン指定はtime 0.3.x系を使用
  - _Requirements: 2.3_

- [ ] 1.2 lib.rsにlua_requestモジュールを宣言
  - `crates/pasta_shiori/src/lib.rs`に`mod lua_request;`を追加
  - 公開設定（`pub mod`または`mod`のみ）は設計書の指示に従う
  - _Requirements: 5.1_

- [ ] 2. lua_request.rsのインポート修正
- [ ] 2.1 chrono依存をtime crateに置換
  - `use chrono;`, `use chrono::Datelike;`, `use chrono::Timelike;`を削除
  - `use time::OffsetDateTime;`を追加
  - _Requirements: 2.3, 3.2_

- [ ] 2.2 外部クレート依存を内部パーサーに変更
  - `shiori3::req`への参照をすべて削除
  - `crate::util::parsers::req`を使用する形に修正
  - _Requirements: 3.3_

- [ ] 2.3 prelude依存を明示的インポートに置換
  - `crate::prelude::*`への参照を削除
  - 必要な型を個別にインポート（`pasta_lua::mlua::{Lua, Table}`, `crate::error::MyResult`など）
  - _Requirements: 3.4_

- [ ] 3. lua_date関数の実装変更
- [ ] 3.1 ライフタイムパラメータを削除
  - 関数シグネチャから`<'lua>`を削除: `pub fn lua_date(lua: &Lua) -> MyResult<Table>`
  - `LuaTable<'lua>`を`Table`に変更
  - _Requirements: 3.1_

- [ ] 3.2 chrono APIをtime APIに置換
  - `chrono::Local::now()`を`OffsetDateTime::now_local()?`に変更（Result型処理）
  - `now.month()`を`now.month() as u8`に変更（enum → u8キャスト）
  - `now.weekday().num_days_from_sunday()`を`now.weekday().number_days_from_sunday()`に変更
  - 日時フィールドの取得メソッドを確認（year, day, hour, minute, second, nanosecond, ordinal）
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 4. parse_request関数およびヘルパー関数の更新
- [ ] 4.1 ライフタイムパラメータを削除
  - `parse_request`, `parse1`, `parse_key_value`関数から`<'lua>`を削除
  - `LuaTable<'lua>`を`Table`に変更
  - 関数シグネチャ: `pub fn parse_request(lua: &Lua, text: &str) -> MyResult<Table>`
  - _Requirements: 3.1_

- [ ] 4.2 既存パーサーとの統合を確認
  - `Parser::parse(Rule::req, text)?`の呼び出しを維持
  - パーサーエラーがMyError::ParseRequestに自動変換されることを確認（From impl既存）
  - _Requirements: 1.1, 1.5, 4.3_

- [ ] 5. error.rsにtime crateエラー変換を追加
- [ ] 5.1 (P) From<IndeterminateOffset>実装を追加
  - `crates/pasta_shiori/src/error.rs`に`From<time::error::IndeterminateOffset>`実装を追加
  - MyError::Script variant経由で変換（メッセージ: "Failed to get local time: {}"）
  - 既存の`From<mlua::Error>`, `From<ParseError>`パターンに準拠
  - _Requirements: 2.4, 4.1, 4.2_

- [ ] 6. コンパイル検証と修正
- [ ] 6.1 cargo checkでエラー解消
  - `cargo check -p pasta_shiori`を実行
  - コンパイルエラーをすべて解消（型不一致、未定義参照など）
  - 警告も可能な限り対応（#[allow(dead_code)]など既存の許可属性は維持）
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 4.1_

- [ ] 7. 統合テストの実装
- [ ] 7.1 テストファイルの作成
  - `crates/pasta_shiori/tests/lua_request_test.rs`を新規作成
  - 必要に応じて`crates/pasta_shiori/tests/common/mod.rs`に共通ユーティリティを追加
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2_

- [ ] 7.2 lua_date関数のテストケース実装
  - Luaインスタンスを生成し、lua_date関数を呼び出し
  - 返却されたテーブルに以下のフィールドが存在することを検証：
    - year, month, day, hour, min, sec, ns（基本日時フィールド）
    - yday/ordinal, wday/num_days_from_sunday（別名フィールド）
  - 各フィールドの型が正しいこと（Lua数値型）を確認
  - _Requirements: 2.1, 2.2_

- [ ] 7.3 parse_request関数のテストケース実装（基本機能）
  - SHIORI 3.0形式のリクエストをパースし、Luaテーブルに変換
  - method, version, charset, id, sender, security_level, status, base_idフィールドが正しく設定されることを検証
  - reference配列（reference[1], reference[2]など）が正しくLua配列に変換されることを確認
  - dicサブテーブルに全キー・バリューが格納されることを確認
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 7.4 parse_request関数のテストケース実装（エラーハンドリング）
  - 不正なSHIORI request形式をパースし、MyError::ParseRequestが返却されることを確認
  - 空文字列をパースした際のエラーハンドリングを検証
  - _Requirements: 1.5_

- [ ] 7.5 SHIORI 2.x形式のパーステスト実装
  - SHIORI 2.x形式のリクエストをパースし、正しく変換されることを確認
  - バージョン20-30の範囲で動作することを検証
  - _Requirements: 1.1, 1.2_

- [ ] 7.6* エッジケースのテスト実装
  - 大量のreference（10個以上）を含むリクエストをパース
  - 日本語値を含むキー・バリューのパーステスト
  - これらのテストは受け入れ基準には直接記載されていないが、堅牢性確保のため実装
  - MVP後に延期可能な追加カバレッジ
  - _Requirements: 1.3, 1.4_

- [ ] 8. 最終検証とコミット
- [ ] 8.1 全テストの実行と合格確認
  - `cargo test -p pasta_shiori`を実行し、すべてのテストが合格することを確認
  - 既存のpasta_shioriテストにリグレッションがないことを確認
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 5.1_

- [ ] 8.2 ワークスペース全体のテスト実行
  - `cargo test --all`を実行し、他のクレートへの影響がないことを確認
  - DoD Test Gate「cargo test --all 成功」を満たす
  - _Requirements: 全要件_

---

## Task Summary

- **Total Tasks**: 8 major tasks, 16 sub-tasks
- **Requirements Coverage**: All 17 sub-requirements (1.1-1.5, 2.1-2.4, 3.1-3.4, 4.1-4.3, 5.1) covered
- **Parallel Tasks**: 2 tasks marked with (P) - 依存関係がなく並行実行可能
- **Optional Tasks**: 1 task marked with * - MVP後に延期可能なエッジケーステスト

## Task Dependencies

1. Task 1 → Task 2-5（依存追加とモジュール宣言が前提）
2. Task 2-4 → Task 6（実装変更後にコンパイル検証）
3. Task 5は並行実行可能（error.rsへの追加は独立）
4. Task 6 → Task 7（コンパイル通過後にテスト実装）
5. Task 7 → Task 8（テスト実装後に最終検証）
