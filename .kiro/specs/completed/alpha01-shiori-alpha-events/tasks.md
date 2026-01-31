# Implementation Plan

## 概要

本実装計画は、SHIORI EVENT 7種のハンドラ登録・スタブ応答実装を段階的に実現する。既存の EVENT/REG/RES モジュールは変更せず、EVENT.no_entry にシーン関数フォールバックを追加し、テスト・ドキュメントを拡充する。

**要件カバレッジ**: 全8要件を13サブタスクにマッピング

---

## タスク一覧

- [x] 1. EVENT.no_entry にシーン関数フォールバック機構を実装
- [x] 1.1 (P) シーン関数検索ロジックの実装
  - `SCENE.search(req.id, nil, nil)` でグローバルシーン検索
  - シーン関数が見つかった場合は pcall で実行
  - エラー発生時は `RES.err(err)` を返却
  - シーン関数未発見時は `RES.no_content()` を返却（従来動作）
  - _Requirements: 7_

- [x] 1.2 (P) エラーハンドリングの検証
  - pcall でシーン関数実行時の例外をキャッチ
  - エラーメッセージを RES.err で包んで返却
  - 既存の xpcall（EVENT.fire）との一貫性確認
  - _Requirements: 7, 4_

- [x] 2. 7種イベントのテスト拡張
- [x] 2.1 (P) OnFirstBoot/OnBoot/OnClose イベントテスト
  - OnFirstBoot: Reference0（バニッシュ復帰フラグ "0"/"1"）の解析確認
  - OnBoot: Reference0（シェル名）、Reference6（シェルパス）、Reference7（ゴーストパス）の解析確認
  - OnClose: Reference0（終了理由 "user"/"shutdown"）の解析確認
  - ハンドラ登録・呼び出し・レスポンス検証
  - _Requirements: 1, 2, 3, 6_

- [x] 2.2 (P) OnGhostChanged/OnSecondChange/OnMinuteChange イベントテスト
  - OnGhostChanged: Reference0（切り替え先）、Reference1（切り替え元）の解析確認
  - OnSecondChange: Reference0（現在秒）、Reference1（累積秒）の解析確認
  - OnMinuteChange: Reference0（現在分）、Reference1（現在時）の解析確認
  - ハンドラ登録・呼び出し・レスポンス検証
  - _Requirements: 1, 2, 3, 6_

- [x] 2.3 (P) OnMouseDoubleClick イベントテスト
  - Reference0（スコープ 0: sakura, 1: kero）の解析確認
  - Reference4（当たり判定 ID）の解析確認
  - ハンドラ登録・呼び出し・レスポンス検証
  - _Requirements: 1, 2, 3, 6_

- [x] 2.4 未登録イベントのフォールバックテスト
  - REG テーブル未登録時に 204 No Content が返されることを確認
  - 既存の基本動作テストとの一貫性確認
  - _Requirements: 2, 4_

- [x] 2.5 (P) シーン関数フォールバックテスト
  - `＊OnBoot` 等のグローバルシーンが検索・実行されることを確認
  - シーン関数実行時のエラーハンドリング確認
  - シーン関数未発見時の 204 No Content 返却確認
  - alpha01 では戻り値無視・204 返却を確認（alpha03 統合は将来）
  - _Requirements: 6, 7_

- [x] 2.6 (P) エラーハンドリングテスト
  - ハンドラ実行時例外のキャッチ・500 レスポンス生成確認
  - traceback 情報の RES.err 包含確認
  - xpcall（EVENT.fire）と pcall（EVENT.no_entry）の整合性確認
  - _Requirements: 2, 4, 6_

- [x] 3. LUA_API.md に SHIORI EVENT セクションを追加
- [x] 3.1 (P) セクション 2: SHIORI EVENT ハンドラの執筆
  - 2.1 概要: SHIORI/3.0 イベントハンドリング機構の説明
  - 2.2 REG テーブルへの登録: ハンドラ関数登録方法とサンプルコード
  - 2.3 サポートイベント一覧: 7種イベントの発火タイミング・Reference 意味・応答サンプル
  - 2.4 RES レスポンス生成: RES.ok/no_content/err の使用方法
  - 2.5 シーン関数フォールバック: `＊OnBoot` 等のグローバルシーン利用方法
  - 2.6 エラーハンドリング: xpcall/pcall によるエラーキャッチと RES.err 使用
  - _Requirements: 8_

- [x] 3.2 (P) Reference パラメータ仕様の記載
  - 各イベントの Reference0～Reference7 の意味を明記
  - OnFirstBoot/OnBoot/OnClose/OnGhostChanged/OnSecondChange/OnMinuteChange/OnMouseDoubleClick 各々の Reference 構造をテーブル化
  - `req.reference[n]` アクセス方法とサンプルコード
  - _Requirements: 3, 8_

- [x] 3.3 (P) スタブ応答サンプルコードの追加
  - OnFirstBoot/OnBoot/OnClose/OnGhostChanged/OnMouseDoubleClick の応答例
  - OnSecondChange/OnMinuteChange の空応答例（alpha02 での仮想イベント発行への言及）
  - RES モジュール API の使用パターン
  - alpha03（さくらスクリプト組み立て）との将来統合を想定したコメント
  - _Requirements: 5, 8_

- [x] 4. ドキュメント整合性の確認と更新
  - SOUL.md: コアバリュー・設計原則との整合性確認（SHIORI基盤として適合）
  - SPECIFICATION.md: 言語仕様への影響なし（Lua側実装のみ）
  - GRAMMAR.md: 文法リファレンスへの影響なし
  - TEST_COVERAGE.md: 新規テスト（shiori_event_test.rs 拡張）のマッピング追加
  - クレート README: pasta_lua/LUA_API.md への追記のみ、README.md 変更不要
  - steering/*: 該当領域のステアリング更新不要（既存 SHIORI 基盤の拡張のみ）
  - _Requirements: 全要件_

---

## タスク実行戦略

### 並列実行可能なタスク

以下のタスクは依存関係がなく並列実行可能:
- **Task 1.1, 1.2**: シーン関数フォールバック実装（独立した機能追加）
- **Task 2.1, 2.2, 2.3, 2.5, 2.6**: 各イベント・機能のテスト（独立したテストケース）
- **Task 3.1, 3.2, 3.3**: ドキュメントセクション執筆（独立した執筆作業）

### 依存関係

- Task 2.4（未登録イベントテスト）: 既存実装のテストのため Task 1 と並列可能
- Task 4（ドキュメント整合性確認）: 全実装完了後に実施（最終タスク）

---

## 受入基準検証マトリクス

| Requirement | Tasks | 受入基準 |
|------------|-------|---------|
| 1 | 2.1, 2.2, 2.3 | 7種イベントのハンドラ登録・上書き動作 |
| 2 | 2.1-2.4, 2.6 | イベントディスパッチ・未登録時204・エラー時500 |
| 3 | 2.1, 2.2, 2.3, 3.2 | Reference パラメータアクセス・nil 処理 |
| 4 | 1.2, 2.4, 2.6 | デフォルトハンドラ・上書き許容 |
| 5 | 3.3 | スタブ応答サンプルコード |
| 6 | 2.1-2.3, 2.5, 2.6 | 全テスト要件（7種・Reference・未登録・シーン・エラー） |
| 7 | 1.1, 1.2, 2.5 | シーン関数フォールバック（検索・実行・エラー・204） |
| 8 | 3.1, 3.2, 3.3 | LUA_API.md ドキュメント（全セクション・Reference・サンプル） |

---

## 実装完了後のチェックリスト

- [x] `cargo test --all` が成功
- [x] 既存の 470 行テストスイート（shiori_event_test.rs）にリグレッションなし
- [x] 7種イベント各々のテストケースが追加されている
- [x] LUA_API.md に SHIORI EVENT セクションが追加されている
- [x] TEST_COVERAGE.md に新規テストがマッピングされている
- [x] spec.json の phase が "implementation-complete" に更新されている
