# 実装タスク: scene-coroutine-execution

## タスク概要

シーン関数をコルーチンとして実行し、チェイントーク機能を実現する。全9要件を6つの主要タスクに分解し、段階的に実装する。

---

## 実装タスクリスト

- [ ] 1. STORE.co_scene フィールド追加
- [ ] 1.1 (P) co_scene フィールドと初期化処理を実装
  - STORE.co_sceneフィールドをnilで初期化
  - STOREモジュールのフィールド定義にco_sceneを追加
  - _Requirements: 7.1, 7.3_

- [ ] 1.2 (P) STORE.reset() にクリーンアップ処理を追加
  - reset()内でco_sceneがsuspended状態ならcoroutine.close()を呼び出し
  - close後にco_sceneをnilにクリア
  - 既存のreset()ロジックを維持
  - _Requirements: 7.2_

- [ ] 2. EVENT.fire のコルーチン対応拡張
- [ ] 2.1 set_co_scene() ローカル関数を実装
  - 引数coがsuspended以外の状態の場合、coroutine.close(co)して co = nil
  - STORE.co_scene == co の場合は何もせずreturn
  - 既存STORE.co_sceneが存在する場合、coroutine.close()で解放
  - STORE.co_scene = co で上書き
  - _Requirements: 2.4_

- [ ] 2.2 EVENT.fire にthread判定とresume処理を追加
  - handler(act)の戻り値の型判定（thread/string/nil）
  - threadの場合: coroutine.resume(result, act)を実行
  - resume成功時: yielded_valueをRES.ok()に渡す
  - resume失敗時: set_co_scene()でclose、error()で伝搬
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3_

- [ ] 2.3 EVENT.fire にコルーチン状態管理を追加
  - resume後にset_co_scene(result)を呼び出し
  - string戻り値の処理: RES.ok(result)を返す
  - nil戻り値の処理: RES.no_content()を返す
  - 既存のhandler呼び出しロジックを維持
  - _Requirements: 2.5, 2.6, 2.7, 2.8, 3.4_

- [ ] 3. virtual_dispatcher のthread返却対応
- [ ] 3.1 create_scene_thread() ヘルパー関数を実装
  - event_name（OnTalk/OnHour）を受け取りシーン検索
  - scene_fnが見つかった場合、coroutine.create(scene_fn)でthreadを生成
  - scene_fnが見つからない場合、nilを返す
  - 既存のexecute_scene()ロジックを置き換え
  - _Requirements: 3.1, 4.4_

- [ ] 3.2 check_hour() をthread返却形式に変更
  - 既存の時刻判定ロジックを維持
  - シーン実行の代わりにcreate_scene_thread("OnHour", act)を呼び出し
  - threadまたはnilを返す
  - _Requirements: 4.1_

- [ ] 3.3 check_talk() にチェイントーク継続ロジックを追加
  - STORE.co_sceneが存在する場合、そのthreadを返す（継続優先）
  - STORE.co_sceneがnilの場合、create_scene_thread("OnTalk", act)を呼び出し
  - 既存の時刻判定・トーク中判定ロジックを維持
  - _Requirements: 4.2, 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ] 3.4 (P) dispatch() 戻り値をthread対応に変更
  - check_hour()とcheck_talk()からthreadまたはnilを受け取り
  - 受け取った値をそのまま返す（実行しない）
  - _Requirements: 4.3_

- [ ] 4. EVENT.no_entry のthread返却対応
- [ ] 4.1 (P) EVENT.no_entry をthread返却形式に変更
  - SCENE.search()でシーン検索
  - scene_fnが見つかった場合、coroutine.create(scene_fn)でthreadを返す
  - scene_fnが見つからない場合、nilを返す
  - 既存のシーン実行ロジックを削除
  - _Requirements: 3.2_

- [ ] 5. OnSecondChange ハンドラの統合
- [ ] 5.1 (P) OnSecondChange をthread橋渡し形式に変更
  - dispatcher.dispatch(act)を呼び出し
  - 戻り値（threadまたはnil）をそのまま返す
  - 既存のRES.no_content()ロジックを削除（EVENT.fireに委譲）
  - _Requirements: 3.3_

- [ ] 6. RES.ok() の空文字列処理拡張
- [ ] 6.1 (P) RES.ok() にnil/空文字列チェックを追加
  - valueがnilまたは空文字列""の場合、RES.no_content()を返す
  - 有効な文字列の場合、既存のレスポンス生成を実行
  - _Requirements: 9.1, 9.2, 9.3_

- [ ] 7. 統合テストの実装
- [ ] 7.1* E2Eテストでact:yield()とチェイントークを検証
  - act:yield()を含むシーン関数を実行し、STORE.co_scene設定を確認
  - 次回OnTalkイベントで継続が再開されることを検証
  - 継続後のシーン完了時にSTORE.co_sceneがクリアされることを確認
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 8.1, 8.2, 8.3_

- [ ] 7.2* エラー処理の統合テストを実装
  - シーン関数内エラー発生時のcoroutine.close()とエラー伝搬を検証
  - 既存suspendedコルーチンがある状態での新規コルーチン設定時のclose処理を検証
  - _Requirements: 8.4, 8.5_

---

## タスク進捗

- 合計: 7つの主要タスク、13のサブタスク
- 全9要件をカバー
- 平均タスクサイズ: 1-2時間/サブタスク
- 並列実行可能: タスク1.1, 1.2, 3.4, 4.1, 5.1, 6.1（データ依存なし、ファイル競合なし）

## 実装順序の推奨

1. **基盤構築**: タスク1（STORE拡張）→ タスク2（EVENT.fire拡張）
2. **ハンドラ統合**: タスク3（virtual_dispatcher）→ タスク4（EVENT.no_entry）→ タスク5（OnSecondChange）
3. **補助機能**: タスク6（RES.ok拡張）
4. **検証**: タスク7（統合テスト）

並列実行可能なタスクは `(P)` マーカーで識別。ファイル競合がないため、同時実装可能。
