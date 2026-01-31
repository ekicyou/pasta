# Implementation Plan: act-req-parameter

## タスク概要

本実装計画は、`pasta.shiori.act` に `req` フィールドを追加し、イベントディスパッチ処理で `act` オブジェクトを生成してハンドラ・シーン関数に引き渡す機能を実装します。すべてのハンドラシグネチャを `function(act)` に統一し、情報源を一元化します。

---

## 実装タスク

### 1. SHIORI_ACT への req フィールド追加

- [ ] 1.1 (P) SHIORI_ACT.new に req パラメータを追加
  - `SHIORI_ACT.new(actors, req)` シグネチャに変更
  - `base.req = req` で req を格納
  - LuaDoc コメントを更新（`@param req table SHIORIリクエストテーブル`）
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 1.2 (P) act.req の読み取り専用推奨をドキュメント化
  - LuaDoc に読み取り専用推奨を明記
  - `@field req table|nil SHIORIリクエストテーブル（読み取り専用推奨）`
  - _Requirements: 1.4_

### 2. EVENT モジュールでの act 生成・引き渡し

- [ ] 2.1 create_act ローカル関数の実装
  - STORE と SHIORI_ACT を require
  - `STORE.actors` を取得して `SHIORI_ACT.new(STORE.actors, req)` を呼び出し
  - act オブジェクトを返却
  - _Requirements: 2.1, 2.3, 3.1, 3.2, 3.3_

- [ ] 2.2 EVENT.fire でのハンドラ呼び出しを handler(act) に変更
  - `EVENT.fire()` 冒頭で `create_act(req)` を呼び出し
  - ハンドラ呼び出しを `handler(req, act)` から `handler(act)` に変更
  - xpcall のエラーハンドリングを維持
  - _Requirements: 2.1_

- [ ] 2.3 EVENT.no_entry のシグネチャを function(act) に変更
  - 引数を `(req, act)` から `(act)` に変更
  - req が必要な箇所は `act.req` から取得
  - シーン検索処理に act を渡す
  - _Requirements: 2.2_

### 3. 既存ハンドラのシグネチャ変更

- [ ] 3.1 (P) boot.lua のハンドラを function(act) に変更
  - `REG.OnBoot = function(req)` → `function(act)` に変更
  - req 参照箇所を `act.req` に変更（存在する場合）
  - _Requirements: 2.1_

- [ ] 3.2 (P) second_change.lua のハンドラを function(act) に変更
  - `REG.OnSecondChange = function(req)` → `function(act)` に変更
  - `dispatcher.dispatch(req)` → `dispatcher.dispatch(act)` に変更
  - _Requirements: 2.1, 2.2_

### 4. virtual_dispatcher の act 対応

- [ ] 4.1 M.dispatch のシグネチャを function(act) に変更
  - 引数を `(req, act)` から `(act)` に変更
  - req が必要な箇所は `act.req` から取得（イベント名、日時情報等）
  - _Requirements: 2.2_

- [ ] 4.2 execute_scene, check_hour, check_talk を act ベースに変更
  - すべての関数で req 引数を削除し、act のみを受け取る
  - `act.req` から必要な情報（イベント ID、日時等）を取得
  - シーン関数呼び出しを `scene_fn(act)` に統一
  - _Requirements: 2.2_

### 5. テスト実装

- [ ] 5.1 (P) SHIORI_ACT.new の単体テスト
  - req パラメータが正しく `act.req` に設定されることを確認
  - `act.req` への参照が正しいことを確認
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 5.2 (P) EVENT.fire の統合テスト
  - act がハンドラに正しく渡されることを確認
  - `act.req` からリクエスト情報が取得できることを確認
  - ハンドラ内で `act.req.id` などの参照が動作することを確認
  - _Requirements: 2.1, 2.3_

- [ ] 5.3 (P) EVENT.no_entry の統合テスト
  - act がシーン関数に正しく渡されることを確認
  - シーン関数内で `act.req` にアクセスできることを確認
  - _Requirements: 2.2_

- [ ] 5.4 (P) virtual_dispatcher の統合テスト
  - `dispatcher.dispatch(act)` が正しく動作することを確認
  - シーン関数に act が渡されることを確認
  - OnSecondChange → dispatcher → scene の完全フローをテスト
  - _Requirements: 2.2_

- [ ] 5.5* リグレッションテストの確認
  - 既存の `shiori_act_test.lua` が全パスすることを確認
  - 既存の `shiori_event_test.rs` が全パスすることを確認（シグネチャ変更後）
  - テスト失敗がある場合、テストコードを新しいシグネチャに合わせて修正
  - _Requirements: すべて（1.1-3.3）_

### 6. ドキュメント整合性の確認と更新

- [ ] 6.1 SOUL.md との整合性確認
  - コアバリュー（日本語フレンドリー、UNICODE識別子、yield型、宣言的フロー）との整合性確認
  - 設計原則（行指向文法、前方一致、UI独立性）との整合性確認
  - 必要に応じて SOUL.md を更新
  - _Requirements: すべて_

- [ ] 6.2 SPECIFICATION.md の更新確認
  - 言語仕様に影響がないことを確認（今回は Lua 実装層のみ）
  - 必要に応じて補足を追加
  - _Requirements: すべて_

- [ ] 6.3 クレート README とステアリングの更新
  - `crates/pasta_lua/README.md` に API 変更を反映（該当する場合）
  - `.kiro/steering/lua-coding.md` にハンドラシグネチャパターンを更新
  - 必要に応じて他のステアリングファイルを更新
  - _Requirements: すべて_

---

## 要件カバレッジ

| 要件 | 対応タスク |
|------|----------|
| 1.1, 1.2, 1.3 | 1.1, 5.1 |
| 1.4 | 1.2 |
| 2.1 | 2.1, 2.2, 3.1, 3.2, 5.2 |
| 2.2 | 2.3, 3.2, 4.1, 4.2, 5.3, 5.4 |
| 2.3 | 2.1 |
| 2.4 | 2.2（既存パターン維持）|
| 3.1, 3.2 | 2.1 |
| 3.3 | 2.1 |

---

## 実装順序の推奨

1. **Phase 1 (並列可能)**: タスク 1.1, 1.2（SHIORI_ACT 拡張）
2. **Phase 2**: タスク 2.1, 2.2, 2.3（EVENT モジュール変更）
3. **Phase 3 (並列可能)**: タスク 3.1, 3.2, 4.1, 4.2（既存コード変更）
4. **Phase 4 (並列可能)**: タスク 5.1, 5.2, 5.3, 5.4（テスト実装）
5. **Phase 5**: タスク 5.5（リグレッション確認）
6. **Phase 6**: タスク 6.1, 6.2, 6.3（ドキュメント更新）

---

## 注意事項

- すべてのハンドラシグネチャを `function(act)` に統一する破壊的変更です
- req 情報は `act.req` から取得するよう、すべての呼び出し元を変更する必要があります
- テストコードも新しいシグネチャに合わせて更新が必要です
