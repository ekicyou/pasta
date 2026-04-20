# Implementation Plan

- [x] 1. act.lua にコアフォールバック検索メソッドを追加する

- [x] 1.1 find_act_handler の6段階フォールバックロジックを実装する
  - `@pasta_search` の利用可否を関数先頭で pcall により1回チェックし変数に保持する
  - Level 1: `current_scene[key]` 完全一致（current_scene が nil の場合はガード）
  - Level 2: ローカル辞書前方一致（word モードなら search_word、scene/expr モードなら SCENE.search を `SCENE.__global_name__` スコープで呼ぶ）
  - Level 3: `self[key]` が function 型の場合のみハンドラーとして採用（act.XX フォールバック）
  - Level 4: `GLOBAL[key]` 完全一致
  - Level 5: グローバル辞書前方一致（word: search_word(nil)、scene/expr: SCENE.search(nil)）
  - @pasta_search 未利用時は前方一致レベルをすべてスキップして nil を返す
  - _Requirements: 1.4, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10, 2.11_

- [x] 1.2 ACT_IMPL.find_handler を find_act_handler への thin wrapper として実装する
  - `return self:find_act_handler(mode, key)` のみを実装する
  - _Requirements: 1.1_

- [x]* 1.3 find_act_handler の直接単体テストを作成する
  - 各フォールバックレベル（scene直接・ローカル辞書・act.XX・GLOBAL・グローバル辞書・nil）をそれぞれ検証する
  - mode 切り替え（word/scene/expr）による辞書種切り替えを検証する
  - _Requirements: 1.4, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10, 2.11_

- [x] 2. actor.lua にコアフォールバック検索メソッドを追加する

- [x] 2.1 (P) find_actor_handler の word 限定検索を実装する
  - mode が `"word"` 以外なら即 nil を返す
  - `proxy.actor[key]` 完全一致を最優先で検索する
  - `@pasta_search` の pcall 保護付きで `"__actor_{name}__"` スコープの前方一致検索を実行する
  - _Requirements: 1.3, 2.1, 2.2, 2.3_

- [x] 2.2 PROXY_IMPL.find_handler を実装する
  - find_actor_handler でアクターレベル検索を先に実行し、マッチすれば即返す
  - マッチしなければ `self.act:find_act_handler(mode, key)` に委譲する
  - _Requirements: 1.2_

- [x] 3. act.lua を find_handler ベースに移行し expr_fn を新設する

- [x] 3.1 ACT_IMPL.expr_fn を新設し expr ポストプロセスとエラーログを実装する
  - `find_handler("expr", key)` でハンドラーを取得する
  - handler が function 型: `h(self, ...)` を呼び出して戻り値を返す（可変引数を伝搬する）
  - handler が nil または非 function: `@pasta_log` 経由で key・mode・経路を含む warn ログを出力し nil を返す
  - _Requirements: 4.1, 4.3, 3.6, 3.7, 7.1, 7.2_

- [x] 3.2 ACT_IMPL.word() を find_handler + word ポストプロセスに書き換える
  - `find_handler("word", name)` でハンドラーを取得する
  - handler が nil: warn ログ出力 + return nil
  - handler が function: `h(self)` の戻り値を返す
  - handler がその他: `tostring(h)` を返す
  - 既存の WORD.resolve_value() 呼び出しと4段階独自検索ロジックを除去する
  - _Requirements: 6.1, 3.1, 3.2, 3.3, 7.1, 7.2_

- [x] 3.3 ACT_IMPL.find_scene() を thin wrapper 化し call() のポストプロセスを整理する
  - find_scene(): `find_handler("scene", key)` を呼び出して結果をそのまま返す（コルーチン化しない）
  - call(): handler が nil または非 function なら warn ログ出力 + return nil
  - call(): handler が function なら `handler(self, ...)` を直接実行する（co_exec 不使用）
  - 5段階独自検索ロジックを find_handler への委譲に置き換える
  - _Requirements: 6.3, 3.4, 3.5, 7.1, 7.2_

- [x] 4. actor.lua を find_handler ベースに移行し expr_fn を新設する

- [x] 4.1 (P) PROXY_IMPL.expr_fn を新設し expr ポストプロセスとエラーログを実装する
  - `find_handler("expr", key)` でハンドラーを取得する
  - handler が function 型: `h(self, ...)` を呼び出して戻り値を返す（可変引数を伝搬する）
  - handler が nil または非 function: warn ログ出力 + return nil
  - caller には proxy 自身（`self`）を渡す
  - _Requirements: 4.2, 4.3, 3.6, 3.7, 7.1, 7.2_

- [x] 4.2 PROXY_IMPL.word() を find_handler + word ポストプロセスに書き換える
  - `find_handler("word", name)` でハンドラーを取得する
  - word ポストプロセスを act.lua の ACT_IMPL.word と同じルールで実装する（caller は `self` = proxy）
  - 既存の `act:word()` 委譲と WORD.resolve_value() 呼び出しを除去する
  - _Requirements: 6.2, 3.1, 3.2, 3.3, 7.1, 7.2_

- [x] 5. トランスパイラの FnScope::Local コード生成を expr_fn 形式に変更する

- [x] 5.1 generate_action() の FnScope::Local 出力を proxy:expr_fn 形式に変更する
  - `act.{actor}:talk(tostring(SCENE.{name}(act, ...)))` を `act.{actor}:talk(tostring(act.{actor}:expr_fn("{name}", ...)))` に変更する
  - 関数名 `{name}` は string_literalizer で Lua 文字列エスケープして渡す
  - 引数リストは既存の generate_args_string() を再利用する
  - _Requirements: 5.1, 5.3_

- [x] 5.2 generate_expr() と generate_expr_to_buffer() の FnScope::Local 出力を act:expr_fn 形式に変更する
  - `SCENE.{name}(act, ...)` を `act:expr_fn("{name}", ...)` に変更する（2箇所）
  - 関数名は文字列リテラルとして渡す
  - _Requirements: 5.2, 5.3_

- [x] 5.3 insta スナップショットテストを差分確認のうえ一括更新する
  - `cargo test` を実行してスナップショット差分を確認する
  - SCENE.func(act, ...) → act:expr_fn("func", ...) への出力変更が意図通りであることを確認する
  - `cargo insta accept` で正しい差分のみを承認する
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 6. 全テストを通過させ最終確認する
  - `cargo test --workspace` で 950+ 件のテストが全て通過することを確認する
  - word のフォールバック順序変更（旧: GLOBAL が L2→新: ローカル辞書が L2）によるリグレッションがないことを確認する
  - SHIORI_ACT_IMPL 継承チェーン経由の act.XX フォールバックが正しく動作することを確認する
  - _Requirements: 6.4_
