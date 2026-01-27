# Implementation Tasks: shiori-res-module

**Feature**: SHIORI/3.0レスポンスビルダーモジュール  
**Phase**: Implementation  
**Total Tasks**: 4 major tasks, 11 sub-tasks  
**Language**: ja

---

## Task List

- [ ] 1. RES モジュールファイル作成と基本構造実装
- [ ] 1.1 (P) モジュールファイルとテーブル宣言を実装
  - `crates/pasta_lua/scripts/pasta/shiori/res.lua` を新規作成
  - モジュールヘッダーに `--- @module pasta.shiori.res` LuaDoc アノテーションを記述
  - `local RES = {}` でモジュールテーブルを宣言
  - 定数 `CRLF = "\r\n"`, `SPLIT = ": "` を定義
  - ファイル末尾で `return RES` によりモジュールを返却
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 1.2 (P) RES.env テーブルを実装
  - `RES.env` テーブルを作成し、3つのフィールドを初期化
  - `charset = "UTF-8"` をデフォルト値として設定
  - `sender = "Pasta"` をデフォルト値として設定
  - `security_level = "local"` をデフォルト値として設定
  - LuaDoc で `@class RESEnv` および各フィールドの `@field` アノテーションを記述
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ] 2. 汎用ビルダー関数 RES.build の実装
- [ ] 2.1 RES.build 関数を実装
  - 引数 `code` (ステータスコード) と `dic` (ヘッダー辞書) を受け取る
  - ステータス行 `"SHIORI/3.0 " .. code .. CRLF` を生成
  - 標準ヘッダー3種（Charset, Sender, SecurityLevel）を `RES.env` から取得して順序通りに出力
  - `dic` が nil の場合は空テーブルとして扱う defensive パターンを実装
  - `dic` がテーブルの場合、各キー・値ペアをヘッダー行として追加
  - 最終行に空行 (`CRLF`) を追加して終端
  - LuaDoc で `@param code string`, `@param dic HeaderDic|nil`, `@return string` を記述
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 9.1, 9.2_

- [ ] 3. ステータス別便利関数の実装
- [ ] 3.1 (P) RES.ok 関数を実装
  - 引数 `value` (Value ヘッダーの値) と `dic` (追加ヘッダー辞書) を受け取る
  - `dic = dic or {}` で nil を防御
  - `dic["Value"] = value` を設定
  - `RES.build("200 OK", dic)` を呼び出して結果を返却
  - LuaDoc アノテーションを記述
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 9.1, 9.2, 9.4_

- [ ] 3.2 (P) RES.no_content 関数を実装
  - 引数 `dic` (追加ヘッダー辞書) を受け取る
  - `RES.build("204 No Content", dic)` を呼び出して結果を返却
  - LuaDoc アノテーションを記述
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 3.3 (P) TEACH 関連レスポンス関数を実装
  - `RES.not_enough(dic)` — `RES.build("311 Not Enough", dic)` を返却
  - `RES.advice(dic)` — `RES.build("312 Advice", dic)` を返却
  - 各関数に LuaDoc アノテーションを記述
  - _Requirements: 6.1, 6.2, 6.3_

- [ ] 3.4 (P) エラーレスポンス関数を実装
  - `RES.bad_request(dic)` — `RES.build("400 Bad Request", dic)` を返却
  - `RES.err(reason, dic)` — `dic = dic or {}` で防御し、`dic["X-Error-Reason"] = reason` を設定後、`RES.build("500 Internal Server Error", dic)` を返却
  - 各関数に LuaDoc アノテーションを記述
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 9.1, 9.2, 9.4_

- [ ] 3.5 (P) ワーニング付きレスポンス関数を実装
  - `RES.warn(reason, dic)` — `dic = dic or {}` で防御し、`dic["X-Warn-Reason"] = reason` を設定後、`RES.no_content(dic)` を返却
  - LuaDoc アノテーションを記述
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 9.1, 9.2, 9.4_

- [ ] 4. テストと統合検証
- [ ] 4.1 RES モジュールの単体テストを実装
  - `crates/pasta_lua/tests/shiori_res_test.rs` を新規作成
  - `RES.ok("test")` が 200 OK レスポンスを返すことを検証
  - `RES.no_content()` が 204 No Content レスポンスを返すことを検証
  - `RES.no_content({["X-Custom"]="val"})` がカスタムヘッダーを含むことを検証
  - `RES.err("reason")` が 500 レスポンスと X-Error-Reason ヘッダーを返すことを検証
  - `RES.warn("warning")` が 204 レスポンスと X-Warn-Reason ヘッダーを返すことを検証
  - `RES.env.charset` の変更が出力に反映されることを検証
  - _Requirements: 4.1, 5.1, 5.3, 7.2, 8.1, 2.5_

- [ ] 4.2 main.lua 統合テストを実装
  - `crates/pasta_lua/tests/shiori_res_test.rs` に統合テストケースを追加
  - `main.lua` から `RES.no_content()` を呼び出せることを検証
  - 標準ヘッダー3種（Charset, Sender, SecurityLevel）が正しい順序で出力されることを検証
  - レスポンスが `\r\n\r\n` で終端することを検証
  - _Requirements: 3.1, 5.1, 3.3, 3.6_

- [ ] 4.3 ドキュメント整合性の確認と更新
  - SOUL.md — コアバリュー・設計原則との整合性確認（DSL拡張なし、Pure Luaユーティリティのため影響なし）
  - SPECIFICATION.md — 言語仕様の更新（該当なし、DSLには影響しない）
  - GRAMMAR.md — 文法リファレンスの同期（該当なし）
  - TEST_COVERAGE.md — 新規テストのマッピング追加（shiori_res_test.rs の6単体テスト + 3統合テストを追加）
  - crates/pasta_lua/README.md — pasta.shiori.res モジュールの説明追加（新規 API として記載）
  - steering/* — 該当領域のステアリング更新（該当なし、既存のlua-coding.mdに準拠）
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4, 8.1, 8.2, 8.3, 8.4, 9.1, 9.2, 9.3, 9.4_

---

## Requirements Coverage Summary

| Requirement | Summary | Covered by Tasks |
|-------------|---------|------------------|
| 1.1-1.3 | モジュール構造 | 1.1 |
| 2.1-2.5 | 環境設定テーブル | 1.2, 4.1 |
| 3.1-3.6 | 汎用ビルダー | 2.1, 4.2 |
| 4.1-4.4 | 200 OK レスポンス | 3.1, 4.1 |
| 5.1-5.3 | 204 No Content | 3.2, 4.1, 4.2 |
| 6.1-6.3 | TEACH 関連 | 3.3 |
| 7.1-7.4 | エラーレスポンス | 3.4, 4.1 |
| 8.1-8.4 | ワーニング | 3.5, 4.1 |
| 9.1-9.4 | エラーハンドリング | 2.1, 3.1, 3.4, 3.5 |

**Total**: 全9要件グループ（36個のAcceptance Criteria）を11タスクでカバー

---

## Notes

- **Parallel Execution**: タスク 1.1, 1.2, 3.1-3.5 は並列実行可能 (`(P)` マーク付き)
- **Dependencies**: タスク2.1は1.1に依存（RES.env を参照）、タスク3.xは2.1に依存（RES.build を呼び出し）、タスク4.xは全実装完了後に実行
- **Test Strategy**: TDD アプローチ推奨 — タスク4.1の単体テストを先に実装し、機能実装を進める
- **Average Task Size**: 各サブタスク 1-2 時間を想定
