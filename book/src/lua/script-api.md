# スクリプト用ランタイム API

ごきげんよう。`scripts/` から呼べるランタイムの道具――ACT、WORD、GLOBAL、SAVE を、ここで一望にいたしますわ。
どの道具で何ができるのか、手元に置いておけば迷うことはございませんの。さあ、参りましょう。

---

この章は、ゴースト作者が `scripts/` 配下の Lua スクリプトから呼ぶランタイム API のリファレンスである。作例と記述の型は [scripts/ の記述パターン](patterns.md) で扱う。

## ACT

ACT は、シーン関数が受け取るトークの組立役のオブジェクトである。この節では、`init_scene` の呼び方、トーク系メソッド（`talk`・`raw_script`）、SHIORI 固有のメソッド（`set_property`・`get_property`）、表示制御、スポット操作、検索と呼び出し（`word`・`find_handler`・`find_act_handler`・`expr_fn`・`find_scene`・`call`）、`yield`・`choice`・`choice_timeout` を扱う。SHIORI リクエストの内容を表す `act.req` のフィールドは [SHIORI イベントとハンドラ](shiori-events.md#actreq) を参照する。

## WORD

`pasta.word` は、Lua から単語を定義するモジュールである。この節では、ファクトリ関数、ビルダーによる単語の登録、大量投入の使用例を扱う。

## GLOBAL

`pasta.global` は、ユーザー定義のグローバル関数を登録するテーブルを返すモジュールである。登録した関数は DSL から `＠＊関数名()` で呼び出せる。この節では、その登録方法を扱う。

## SAVE

`pasta.save` は、セッションをまたいで保持される永続化データのテーブルを返すモジュールである。この節では、ACT 経由のアクセスと `require` による直接のアクセスを扱う。セーブキーの命名規約は [@pasta_persistence](modules/pasta-persistence.md#セーブキーの命名規約) を参照する。

---

道具の一覧は手に入りましたわね。フンッ、あとは使いこなすだけですわよ。
さあ、あなたのゴーストに存分に働いてもらいましょう！
