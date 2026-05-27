# Implementation Plan

## shiori-async-talk

---

- [ ] 1. CALLBACKモジュール基盤の構築
- [x] 1.1 IDカウンターとステージング・pending登録機構の実装
  - `pasta/shiori/event/callback.lua` を新規作成し、モジュール局所状態（`_next_id`, `_staged`, `pending`）を定義する
  - `CALLBACK.next_event_id()` を実装: 毎回インクリメントして `"OnPastaCallBack{N}"` 形式の文字列を返す
  - `CALLBACK.stage_pending(event_id, timeout_at, on_timeout)` を実装: 単一スロット（`_staged`）への記録と、`_staged` が非 nil の場合の多重ステージング検出エラーを含む
  - `CALLBACK.consume_staged(co, act)` を実装: `_staged` 消費・`CALLBACK.pending[event_id]` への `{co, act, timeout_at, on_timeout}` 登録・`STORE.co_callback = co` マーカー設定の3ステップを実装（`STORE` を require して使用）
  - `CALLBACK.reset()` を実装: `_next_id`, `_staged`, `pending`, `STORE.co_callback` を初期化してテスト用状態クリアを提供する
  - **完了確認**: `CALLBACK.stage_pending` → `CALLBACK.consume_staged` のラウンドトリップを手動または `reset()` を使ったテストコードで確認すると、`CALLBACK.pending` にエントリが登録され、`STORE.co_callback` にコルーチンが設定される
  - _Requirements: 1.1, 1.2_
  - _Boundary: CALLBACK module (callback.lua)_

- [x] 1.2 コールバックルーティングとタイムアウトsweepの実装
  - `CALLBACK.try_route(req)` を実装: `req.id` が `CALLBACK.pending` のキーと一致するとき、`req.reference[0..N]` を 1-based 配列に詰め替えて `coroutine.resume(entry.co, refs)` を呼び出す。コールバック後に同一コルーチンが再度 `get_property` を呼ぶ（コールバックチェーン R4）場合は `consume_staged` が発火するため、`try_route` 内で一致エントリの削除後に `consume_staged` の追加戻り値を処理してネストした pending 登録に対応する。不一致時は nil を返す
  - `CALLBACK.sweep(now)` を実装: `now > entry.timeout_at` のエントリを走査し、`on_timeout` が文字列の場合は `coroutine.resume(co, nil, on_timeout)` + `@pasta_log.warn(event_id, on_timeout)` + `pasta.shiori.res` で 500 + `X-ERROR-REASON: <on_timeout>` レスポンス文字列を生成して返す。`on_timeout` が nil の場合は `coroutine.resume(co, nil)` のみで nil を返す。いずれの場合もエントリを削除する
  - sweep が複数エントリ走査時は文字列 `on_timeout` の最初のタイムアウトエントリを 500 として返し、他はサイレントに削除する（OnSecondChange は毎秒発火するため次回以降に処理可能）
  - **完了確認**: `try_route` にマッチするリクエストを渡すとコルーチンが resume され non-nil のレスポンス文字列が返る。`sweep` にタイムアウト超過エントリ（文字列 `on_timeout`）を渡すと 500 レスポンスが返り、nil `on_timeout` のエントリを渡すと nil が返り、いずれも pending から削除される
  - _Requirements: 1.1, 1.2, 1.3, 5.1, 5.2, 5.3_
  - _Boundary: CALLBACK module (callback.lua)_

- [ ] 2. イベントディスパッチャへの統合
- [x] 2.1 (P) EVENT.fireへのコールバックルーティング分岐とset_co_scene修正
  - `event/init.lua` の `EVENT.fire(req)` 冒頭に `CALLBACK.try_route(req)` 呼び出しを追加し、非 nil なら即 return するルーティング分岐を実装する（`create_act` より前）
  - `resume_until_valid` 後に `CALLBACK.consume_staged(result, act)` を呼び出す分岐を追加する（戻り値 true/false は `set_co_scene` の挙動制御に使う）
  - `set_co_scene(co)` を修正: `co == STORE.co_callback` の場合は旧 `STORE.co_scene` を close せずにデタッチのみ行い（コールバック待ちコルーチンの誤 close を防ぐ）、`STORE.co_callback` をクリアする。旧 `STORE.co_scene` が `co` と別オブジェクトであれば通常通り close する
  - **完了確認**: コールバック待ちで yield したシーンコルーチンが `STORE.co_scene` に登録されず `CALLBACK.pending` に登録される。既存のチェーントーク（`act:yield()` 利用）は従来通り `STORE.co_scene` に登録され、挙動に変化がない
  - _Requirements: 2.2, 6.1, 6.2, 6.3_
  - _Boundary: EVENT.fire, set_co_scene (event/init.lua)_

- [x] 2.2 (P) OnSecondChangeラッパーへのsweep呼び出し追加
  - `second_change.lua` の `REG.OnSecondChange` を修正: `CALLBACK.sweep(os.time())` を先に呼び出し、非 nil（タイムアウト 500 レスポンス）が返ったら即 return し、nil なら既存の `dispatcher.dispatch(act)` に委譲する
  - **完了確認**: タイムアウト超過エントリが pending にある状態で `OnSecondChange` を発火すると 500 レスポンスが返る。エントリがない場合は通常のディスパッチ結果が返る
  - _Requirements: 5.2, 5.3_
  - _Boundary: REG.OnSecondChange (second_change.lua)_

- [x] 2.3 (P) get_propertyメソッドの実装
  - `act.lua` の `SHIORI_ACT_IMPL` に `get_property(self, name_or_names, timeout, timeout_message)` メソッドを追加する
  - 引数正規化: `name_or_names` が string なら 1 要素配列、table ならそのまま利用し、それ以外の型ではエラーを発生させる
  - バリデーション（`stage_pending` より前に完了させること）: 配列長 0 のエラー・各要素の nil または空文字列エラー・`coroutine.running()` で `is_main == true` または `co == nil` のコルーチン外呼び出しエラー
  - `timeout = timeout or 5`・`timeout_message = timeout_message or "callback timeout: get_property"` でデフォルト適用
  - `CALLBACK.next_event_id()` でユニーク ID 生成、`CALLBACK.stage_pending(id, os.time() + timeout, timeout_message)` でステージング
  - `\![get,property,{id},{names...}]` タグを既存の `escape_tag_arg` でエスケープして `self.token` に蓄積
  - `local refs, reason = coroutine.yield(self:build())` で一時停止し、`reason` があれば `error(reason)`、`refs == nil` なら全 nil 多値返却、それ以外は `refs[i]` の空文字列→nil 変換を行って多値で返す
  - **完了確認**: `act:get_property("name")` の呼び出しでタグが蓄積されコールバック到着後に文字列値が返る。`act:get_property({"a","b"})` でプロパティ名ごとの 2 値が多値として返る。引数なし・nil・空文字列・不正型・コルーチン外で明確なエラーが発生する
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 4.1, 4.2, 4.3, 5.4_
  - _Boundary: SHIORI_ACT.get_property (act.lua)_

- [ ] 3. テスト実装
- [x] 3.1 (P) CALLBACKモジュール単体テスト
  - `crates/pasta_lua/tests/callback_module_test.lua` を新規作成し、既存の Lua テストパターンと `CALLBACK.reset()` を使って各関数を検証する
  - `next_event_id` の連番確認（OnPastaCallBack1, OnPastaCallBack2, ...）
  - `stage_pending` → `consume_staged` のラウンドトリップで `CALLBACK.pending` にエントリが登録されることを確認
  - 多重 `stage_pending`（`consume_staged` 前）がエラーを発生させることを確認
  - `try_route` の一致・不一致の両パスを確認（一致時はコルーチンが resume され pending が削除される）
  - `sweep` が `on_timeout` 文字列エントリで 500 レスポンスを返し、ログ出力が発生し、pending が削除されることを確認
  - `sweep` が `on_timeout` nil エントリで nil を返し、ログ出力がなく、pending が削除されることを確認
  - **完了確認**: `lua_test` または相当するテスト実行コマンドで `callback_module_test.lua` の全アサーションが通過する
  - _Requirements: 1.1, 1.2, 5.1, 5.2, 5.3_
  - _Boundary: CALLBACK module tests (pasta_lua/tests/)_

- [ ] 3.2 (P) get_propertyバリデーション・タグ発行・多値返却テスト
  - `crates/pasta_lua/tests/get_property_test.lua` を新規作成する
  - 引数なし・nil・空文字列・不正型（数値、boolean）で明確なエラーが発生することを確認
  - メインスレッド呼び出しでエラーが発生することを確認
  - 単一 string 引数で `\![get,property,OnPastaCallBack{N},name]` タグが蓄積され、ステージングが発生することを確認
  - table 引数 `{"n1","n2"}` で `\![get,property,OnPastaCallBack{N},n1,n2]` タグが蓄積されることを確認
  - カンマ・引用符を含むプロパティ名が `escape_tag_arg` でエスケープされることを確認
  - `timeout` のみ指定時にデフォルト `timeout_message` がステージングに反映されることを確認
  - **完了確認**: `lua_test` または相当するテスト実行コマンドで `get_property_test.lua` の全アサーションが通過する
  - _Requirements: 2.1, 3.1, 4.1, 4.2, 4.3, 5.4_
  - _Boundary: SHIORI_ACT tests (pasta_lua/tests/)_

- [ ] 3.3 SHIORIプロトコルレベル統合テスト
  - `crates/pasta_shiori/tests/async_callback_integration_test.rs` を新規作成する（既存統合テストのモックパターンを参考にする）
  - **Scenario 1**: 2 ラウンドで `baseware.version: 2.6.77\e` が取得できることを確認（Round 1 の Value に `\e` なし）
  - **Scenario 2**: Round 1 の Value に蓄積済みトークンと get タグが同一 Value に含まれることを確認
  - **Scenario 3**: 3 ラウンドで `STORE.co_scene` と `CALLBACK.pending` の切り替えが正しく行われることを確認（チェーントーク→コールバック待ち遷移）
  - **Scenario 4**: コールバック待機中に `OnTalk` 等の無関係イベントが `CALLBACK.pending` を保持したまま正常にディスパッチされることを確認
  - 複数プロパティの `Reference0/1` マッピングが正しく多値返却されることを確認
  - 空文字列 Reference が nil に変換されることを確認
  - タイムアウト sweep 後に待機コルーチンが解放され、遅延コールバック到着時に 500 レスポンスが返ることを確認
  - **完了確認**: `cargo test async_callback_integration` で全シナリオが通過する
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 5.2, 6.1, 6.2, 6.3_

- [ ] 3.4 既存テストのリグレッション確認
  - `cargo test` で `event_coroutine_test` と `integration_coroutine_test` が変更なしで通過することを確認する
  - **完了確認**: 既存の全テストスイートが green のまま維持され、本機能導入による既存フローへの影響がないことが確認される
  - _Requirements: 6.1, 6.2_
