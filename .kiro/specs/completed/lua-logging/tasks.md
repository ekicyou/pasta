# Implementation Plan

## Overview
`@pasta_log` モジュールの実装タスク。Lua実行環境からRust側のtracingロギングインフラへログ出力をブリッジする機能を提供する。

**実装方針**:
- 最初にスタックレベル検証プロトタイプで `inspect_stack` の正しいレベル値を確定
- ValueConverter、CallerInfo、LogModule の順に実装
- 各コンポーネント実装後にユニットテストで検証
- 最後にモジュール登録とE2Eテストで統合確認

**並列実行**: `(P)` マークのあるタスクは並列実行可能

---

## Tasks

### Phase 1: スタックレベル検証とコア実装

- [x] 1. スタックレベル検証プロトタイプ
  - mlua `create_function` でクロージャを作成し、`inspect_stack(1)` と `inspect_stack(2)` を試行
  - Luaスクリプト `local function test() log_fn("msg") end; test()` で呼び出し、各レベルで取得される source/line を確認
  - 正しいレベル値を確定（期待: Lua呼び出し元の情報が取得できるレベル）
  - 検証結果をコメントまたはテストコードに記録
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 2. (P) ValueConverter実装
- [x] 2.1 (P) 基本的な値変換ロジック
  - String → そのまま使用
  - Integer, Number, Boolean → tostring() 相当の変換
  - Nil, 引数なし → 空文字列
  - Function, UserData, Thread → tostring() フォールバック
  - 最終フォールバック（tostring失敗時）→ `"<unconvertible value>"`
  - _Requirements: 1.3, 1.5_

- [x] 2.2 (P) テーブルのJSON変換とサイズ制限
  - テーブル要素数が1000を超える場合 → `"<table: N elements>"` 形式で省略表示
  - serde_json で JSON 変換（`lua.from_value::<serde_json::Value>()`）
  - ネスト深度制限10段階（`DeserializeOptions::default().max_depth(10)`）
  - JSON変換失敗時（循環参照・深度超過等）→ tostring() フォールバック
  - _Requirements: 1.4_

- [x] 2.3 (P) ValueConverterのユニットテスト
  - 各型の変換確認（文字列、数値、真偽値、nil）
  - テーブルJSON変換確認（正常ケース、期待フォーマット `{"key":"value"}`, `[1,2,3]`）
  - サイズ超過テーブル（1001要素）の省略表示確認
  - ネスト深度超過（11段階）のフォールバック確認
  - 循環参照テーブルのフォールバック確認
  - _Requirements: 1.3, 1.4, 1.5_

- [x] 3. (P) CallerInfo実装
- [x] 3.1 (P) スタック情報取得ロジック
  - タスク1で確定したレベル値で `Lua::inspect_stack(level, ...)` 呼び出し
  - DebugSource から source（ソースファイル名）取得
  - Debug::current_line() から line（行番号）取得
  - DebugNames から fn_name（関数名）取得
  - 情報取得失敗時はデフォルト値（空文字列/0）で補完、エラーを返さない
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 3.2 (P) CallerInfoのユニットテスト
  - 直接呼び出し（main chunk）でのsource/line取得確認
  - Lua関数内からの呼び出しでfn_name取得確認
  - スタックレベル検証（期待: 呼び出し元の正しい行番号）
  - 情報取得失敗時のデフォルト値補完確認
  - _Requirements: 2.1, 2.2, 2.4, 5.1_

### Phase 2: ログモジュール実装と統合

- [x] 4. LogModule実装（runtime/log.rs新規作成）
- [x] 4.1 5レベルのログ関数実装
  - trace, debug, info, warn, error の5関数を Lua テーブルに登録
  - 各関数は ValueConverter で値を文字列化
  - CallerInfo で呼び出し元情報を取得
  - 対応する tracing マクロ（`tracing::trace!`, `tracing::debug!` 等）を呼び出し
  - structured fields として `lua_source`, `lua_line`, `lua_fn` を埋め込み
  - _Requirements: 1.1, 1.2, 4.1, 4.2_

- [x] 4.2 register() 関数実装
  - `pub fn register(lua: &Lua) -> LuaResult<Table>` シグネチャ
  - Luaテーブル作成、5レベルログ関数を登録
  - `_VERSION`, `_DESCRIPTION` メタデータ設定
  - 引数チェック不要（ValueConverter/CallerInfoがエラーを返さない）
  - _Requirements: 3.3, 3.4_

- [x] 4.3* LogModuleのユニットテスト
  - 各ログレベル関数の呼び出し成功確認（tracing-test使用）
  - 文字列引数の出力確認（`logs_contain("test message")`）
  - 数値・真偽値・テーブル・nilの変換出力確認
  - 呼び出し元情報のstructured fields確認（`logs_contain("lua_source=")`, `lua_line=`, `lua_fn=`）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2_

- [x] 5. モジュール登録と統合
- [x] 5.1 mod.rsへの統合
  - `mod log;` 宣言を追加
  - `register_log_module()` メソッドを PastaLuaRuntime に追加
  - `from_loader_with_scene_dic()` 内の `register_persistence_module()` 直後に呼び出し
  - `with_config()` 内にも登録追加（テスト用ランタイムで利用可能に）
  - 登録成功後 `tracing::debug!("Registered @pasta_log module")` 出力
  - _Requirements: 3.1, 3.2, 5.3_

- [x] 5.2 統合テスト
  - `require "@pasta_log"` でモジュール取得可能であること確認
  - `_VERSION` と `_DESCRIPTION` の存在確認
  - 他の `@pasta_*` モジュールとの共存確認（@pasta_persistence との順序依存なし）
  - main.lua 内からのログ出力が正常に動作すること確認
  - PastaLogger設定時にインスタンス固有ファイルへの書き込み確認
  - PastaLogger未設定時でもエラーなく動作すること確認（tracing出力のみ）
  - _Requirements: 3.1, 3.2, 4.3, 4.4_

### Phase 3: ドキュメント整合性確認

- [x] 6. ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（UNICODE識別子、yield型等）
  - doc/spec/ - 言語仕様の更新（該当する場合）
  - GRAMMAR.md - 文法リファレンスの同期（該当する場合）
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - crates/pasta_lua/README.md - API変更の反映（@pasta_log モジュール追加）
  - steering/* - 該当領域のステアリング更新（runtime/log.rs 追加）
  - _Requirements: 全要件_

---

## Summary

**合計**: 6メジャータスク、13サブタスク  
**並列実行可**: タスク2.1-2.3, 3.1-3.2（Phase 1コア実装）  
**平均作業時間**: 1-3時間/サブタスク  
**全要件カバレッジ**: 15要件（R1.1-R5.3）全てマッピング済み
