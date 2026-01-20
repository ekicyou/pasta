# Implementation Tasks

## Overview
本タスクリストは、SHIORI.request関数に解析済みLuaテーブルを渡す機能の実装作業を定義する。要件1-3をカバーし、既存の`lua_request::parse_request`を活用して、Lua側でのリクエスト解析を不要にする。

## Task Breakdown

### 1. 400 Bad Requestレスポンス生成機能の実装

- [ ] 1.1 (P) 400 Bad Requestレスポンス生成機能を実装
  - SHIORI/3.0プロトコル準拠の静的メソッドを追加
  - 既存の`default_204_response`と同様の実装パターンを踏襲
  - Charset: UTF-8、Sender: Pastaを固定で含める
  - レスポンス形式: `SHIORI/3.0 400 Bad Request\r\nCharset: UTF-8\r\nSender: Pasta\r\n\r\n`
  - _Requirements: 1.3_

### 2. リクエスト解析とLua呼び出しの変更

- [ ] 2.1 リクエスト解析処理を統合
  - `call_lua_request`メソッドに`lua_request::parse_request`呼び出しを追加
  - パース成功時: 解析済みテーブルを取得
  - パース失敗時: エラーログ出力後に`default_400_response`を返却してLua呼び出しをスキップ
  - 既存の204レスポンス動作（関数未定義時）を維持
  - _Requirements: 1.1, 1.3, 3.4_

- [ ] 2.2 Lua関数呼び出しシグネチャの変更
  - `request_fn.call`の引数を生テキスト（`&str`）から解析済みテーブル（`mlua::Table`）に変更
  - Lua実行エラー時の既存エラーハンドリングを維持
  - _Requirements: 1.2, 3.1, 3.2, 3.5_

### 3. Luaテストフィクスチャの更新

- [ ] 3.1 (P) Luaテストフィクスチャのシグネチャ変更
  - `main.lua`の`SHIORI.request`関数を更新
  - パラメータ変更: `request_text` → `req`
  - テーブルフィールドアクセスに変更: `req.id`, `req.reference[0]`など
  - 既存のイベント処理ロジックを新しいテーブル構造に適合
  - _Requirements: 3.6_

### 4. テストによる動作検証

- [ ] 4.1 (P) ユニットテスト追加
  - `call_lua_request`のパース成功パステスト: テーブルが正しく渡されることを検証
  - `call_lua_request`のパース失敗パステスト: 400レスポンスが返却されることを検証
  - `default_400_response`のフォーマット検証テスト
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 4.2 統合テスト実行と検証
  - `cargo test -p pasta_shiori`を実行
  - `shiori_lifecycle_test.rs`の全テストが合格することを確認
  - SHIORI.load → SHIORI.request(req) → SHIORI.unloadのフルサイクル動作を検証
  - reqテーブルからの全フィールドアクセス（method, version, id, reference, dic, charset, base_id, sender）を確認
  - 既存の`lua_request_test.rs`が影響を受けずに合格することを確認
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 3.3, 3.7_

## Requirements Coverage Matrix

| Requirement | Covered by Tasks |
|-------------|------------------|
| 1.1 | 2.1, 4.1 |
| 1.2 | 2.2, 4.1 |
| 1.3 | 1.1, 2.1, 4.1 |
| 2.1 | 4.2 |
| 2.2 | 4.2 |
| 2.3 | 4.2 |
| 2.4 | 4.2 |
| 2.5 | 4.2 |
| 2.6 | 4.2 |
| 2.7 | 4.2 |
| 2.8 | 4.2 |
| 3.1 | 2.2 |
| 3.2 | 2.2 |
| 3.3 | 4.2 |
| 3.4 | 2.1 |
| 3.5 | 2.2 |
| 3.6 | 3.1 |
| 3.7 | 4.2 |

## Parallel Execution Notes

- タスク1.1と3.1は独立しており、並列実行可能（ファイル競合なし）
- タスク4.1と4.2は独立したテスト範囲のため並列実行可能
- タスク2.1, 2.2は同一ファイル内の密接な変更のため順次実行を推奨
- 全ての並列タスクは`(P)`マーカーで明示

## Implementation Notes

- 既存の`default_204_response`（L300）との一貫性を保つため、`default_400_response`も静的メソッドとして実装
- `lua_request::parse_request`は既にテスト済みの既存資産のため、新規テスト不要
- フィクスチャ更新対象は1ファイルのみ（調査済み）: `crates/pasta_shiori/tests/fixtures/shiori_lifecycle/scripts/pasta/shiori/main.lua`
- 破壊的変更だが未リリースのため、後方互換性考慮は不要
