# Implementation Plan

## Task Format

- `(P)` — 前のタスクと並行実行可能
- `*` — 任意（後回し可能なテスト補完）

---

- [ ] 1. `act:find_scene()` の抽出と `act:call()` へのリファクタリング

- [ ] 1.1 `act.lua` に `ACT_IMPL.find_scene()` メソッドを実装する
  - `ACT_IMPL.call()` の L1〜L5 フォールバック検索ロジックをそのまま `find_scene()` として抽出する
  - シグネチャは `ACT_IMPL.find_scene(self, key, global_scene_name?, attrs?)` とし、関数オブジェクトまたは `nil` を返す（実行しない）
  - L1 の `if self.current_scene then` nil ガードをそのまま保持する
  - 既存コードの外部インターフェースには一切触れない
  - EmmyLua 型注釈（`@param`, `@return`）を整備する
  - _Requirements: 1.1, 1.3, 2.3, 2.4_

- [ ] 1.2 `act:call()` の内部実装を `find_scene()` ベースに書き換える
  - `ACT_IMPL.call()` 内の L1〜L5 検索ロジックを `self:find_scene(key, global_scene_name, attrs)` 呼び出しに置換する
  - 外部パラメータ順序 `(self, global_scene_name, key, attrs, ...)` および戻り値型は変更しない
  - `find_scene` が nil を返した場合の `log.error` + `return nil` は維持する
  - 既存の `act:call()` テストが全パスすることを確認する
  - _Requirements: 1.3, 4.3, 5.1, 5.2, 5.4_

- [ ] 2. (P) `SCENE.co_exec()` のシグネチャ変更と `find_scene` への委譲
  - `SCENE.co_exec(name, global_scene_name, attrs)` → `SCENE.co_exec(act, name, global_scene_name, attrs)` にシグネチャを変更する（`act` を第1引数に追加）
  - 関数内の `SCENE.search(name, global_scene_name, attrs)` 呼び出しを `act:find_scene(name, global_scene_name, attrs)` に置換する
  - `scene_result.func` の取得と nil チェックを削除し、`find_scene` の戻り値（`function|nil`）をそのまま使用する
  - `wrapped_fn` 内の `fn(act, ...)` + `act:build()` ロジックおよびコルーチン生成は変更しない
  - タスク 1 完了後に開始する（`act:find_scene()` が必要）、タスク 4 と並行実行可能
  - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.3_

- [ ] 3. イベントディスパッチ呼び出し元の一斉更新（タスク 2 完了後）

- [ ] 3.1 (P) `EVENT.no_entry()` の `SCENE.co_exec()` 呼び出しを新シグネチャに更新する
  - `event/init.lua` の `SCENE.co_exec(act.req.id, nil, nil)` → `SCENE.co_exec(act, act.req.id, nil, nil)` に変更する
  - `act` はすでに `EVENT.no_entry(act)` の引数として利用可能であることを確認する
  - _Requirements: 1.1, 1.2, 3.2_

- [ ] 3.2 (P) `REG.OnBoot` デフォルトハンドラの `SCENE.co_exec()` 呼び出しを更新する
  - `event/boot.lua` の `SCENE.co_exec(act.req.id, nil, nil)` → `SCENE.co_exec(act, act.req.id, nil, nil)` に変更する
  - `act` はすでにハンドラ引数として利用可能であることを確認する
  - _Requirements: 3.1, 3.2_

- [ ] 3.3 (P) 仮想ディスパッチャの `create_scene_thread()` 内の `SCENE.co_exec()` 呼び出しを更新する
  - `event/virtual_dispatcher.lua` の `SCENE.co_exec(event_name, nil, nil)` → `SCENE.co_exec(act, event_name, nil, nil)` に変更する
  - `act` はすでに `create_scene_thread(event_name, act)` の第2引数として利用可能であることを確認する
  - _Requirements: 2.1, 2.2_

- [ ] 4. (P) `act:find_scene()` 単体テストの実装
  - タスク 1 完了後に開始可能、タスク 2・3 と並行実行可能（テストファイルは実装ファイルと独立）
  - 各フォールバックレベルが正しく機能することを個別に検証するテストケースを実装する:
    - L1: `current_scene` に直接登録されたローカル関数の解決
    - L2: `SCENE.search()` でのスコープ付き前方一致検索での解決
    - L3: `GLOBAL` テーブルに登録された関数の解決（本仕様の主要修正点）
    - L5: スコープなし全体検索でのフォールバック解決
    - 全レベル未発見で `nil` が返ること
  - _Requirements: 1.1, 1.3, 2.3, 2.4_

- [ ] 5. イベントディスパッチ統合テストの実装（タスク 1〜4 完了後）

- [ ] 5.1 `GLOBAL` フォールバック統合テストを実装する
  - `GLOBAL.OnHour` に関数を登録し、DSLラベル `＊OnHour` が未定義の状態で OnHour イベントを発火 → GLOBAL の関数が呼ばれることを確認する
  - `GLOBAL.OnBoot` 同様のパターンで確認する
  - _Requirements: 1.1, 2.3_

- [ ] 5.2 DSL ラベルと `GLOBAL` が共存するときの優先順位テストを実装する
  - `＊OnHour` DSL ラベルと `GLOBAL.OnHour` の両方を定義した状態で OnHour を発火 → DSL ラベル側が呼ばれることを確認する
  - _Requirements: 2.4_

- [ ] 5.3 既存テスト全パスによるリグレッション検証を行う
  - `cargo test --all` を実行し、以下の既存テストが全パスすることを確認する：`event_dispatch_test`、`event_handler_test`、`virtual_event_dispatch_test`、`virtual_event_config_test`、`act:call()` 関連テスト全件
  - チェイントーク継続（`STORE.co_scene` 進行中に新規イベントがスキップされる）の動作も確認する
  - REG 登録済みハンドラが依然として最優先で実行されることを確認する
  - _Requirements: 3.1, 3.3, 4.2, 5.1, 5.2, 5.3, 5.4_

- [ ] 6. ドキュメント整合性の確認と更新
  - 実装完了後、以下のドキュメントとの整合性を確認・更新する：
    1. [ ] SOUL.md — コアバリュー・設計原則との整合性確認（コードパス1本化原則に沿った変更であることを確認）
    2. [ ] `doc/spec/` — 言語仕様への影響確認（今回は内部リファクタリングのみにつき変更不要の可能性が高い）
    3. [ ] TEST_COVERAGE.md — `act:find_scene()` 単体テスト・GLOBAL フォールバック統合テストの追加をマッピング
    4. [ ] `crates/pasta_lua/README.md` — `act:find_scene()` API の追加を反映（該当する場合）
    5. [ ] `.agents/skills/pasta-lua-coding/` — `act:find_scene(key, scope?, attrs?)` の API ドキュメントと解決優先順位（L1〜L5）の説明を追加する（Req 1.4）
    6. [ ] `steering/*` — 該当領域のステアリング更新
  - _Requirements: 1.4_
