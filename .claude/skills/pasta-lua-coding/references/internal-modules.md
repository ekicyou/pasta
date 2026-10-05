<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta マニュアル「Lua ランタイム内部モジュール」（https://ekicyou.github.io/pasta/internals/internal-modules.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# Lua ランタイム内部モジュール

この章は、Lua ランタイムの内部モジュールのモジュール単位のリファレンスである。モジュール間の関係と実行の流れは [ランタイム実行モデル](https://ekicyou.github.io/pasta/internals/execution-model.html) で扱う。ゴースト作者が `scripts/` から呼ぶ API（ACT のトーク系メソッド・WORD・GLOBAL・SAVE の使い方）は [スクリプト用ランタイム API](script-api.md) で扱う。

この章が扱うモジュールのソースの所在は [ランタイム実行モデル](https://ekicyou.github.io/pasta/internals/execution-model.html#ソースの所在) を参照する。

## STORE パターン

`pasta.store` は、ランタイムが共有するデータを 1 つの表に集めて保持するモジュールである。他の `pasta.*` モジュールを `require` しないため、どのモジュールからも循環なく `require` できる。

```lua
local STORE = require("pasta.store")
```

### STORE のフィールド

| フィールド | 型 | 内容 | 書き手と読み手 |
| ---------- | -- | ---- | -------------- |
| `actors` | `table<string, Actor>` | アクター名 → アクターオブジェクト | 初期値は `@pasta_config` の `actor`。`pasta.actor` の `get_or_create` が追加する。ACT の生成時に `act.actors` として渡る |
| `actor_spots` | `table<string, integer>` | アクター名 → スポット番号 | 初期値は `@pasta_config` の `actor.*.spot`。`pasta.shiori.act` の `build` が `sakura_builder` に渡し、組立が `spot`・`clear_spot` のトークンで書き換える |
| `appearance` | `table` | 外見の状態 `{ actors, spots, owners, last_spots }`（アクターの既知状態・スポットの表示中状態・スポットの直前の発話アクター・アクターの前回の発話スポット）。セッション中だけ保持し、永続化しない | `pasta.shiori.act` の `build` が `sakura_builder` に渡す（[アピアランスの観測と復旧](https://ekicyou.github.io/pasta/internals/talk-output.html#アピアランスの観測と復旧)） |
| `scenes` | `table<string, SceneTable>` | グローバルシーン名 → シーンテーブル | `pasta.scene` が作り、`get_all_scenes` が返す |
| `counters` | `table<string, number>` | シーンの基本名 → 最後に振った番号 | `pasta.scene` の `get_or_increment_counter` |
| `global_words` | `table<string, table>` | 単語キー → 値リストの配列 | `pasta.word` の `create_global` のビルダー |
| `local_words` | `table<string, table>` | グローバルシーン名 → { 単語キー → 値リストの配列 } | `pasta.word` の `create_local` のビルダー |
| `actor_words` | `table<string, table>` | アクター名 → { 単語キー → 値リストの配列 } | `pasta.word` の `create_actor` のビルダー |
| `app_ctx` | `table` | セッション中の汎用の表 | ACT の生成時に同じ表が `act.app_ctx` に入る |
| `co_scene` | `thread\|nil` | 中断中の継続待ちのコルーチン | `pasta.shiori.event` の `set_co_scene`（[継続トーク（チェイントーク）と co_scene の更新](https://ekicyou.github.io/pasta/internals/execution-model.html#継続トークチェイントークと-co_scene-の更新)） |
| `last_global_scene` | `string\|nil` | 最後に `init_scene` したシーンテーブルの `__global_name__` | `ACT_IMPL.init_scene` が書き、OnChoiceSelectEx の既定ハンドラが選択 ID の検索の親に使う |
| `kick_pending` | `string\|nil` | 保留中のキック対象のシーン名 | `KICK.install` が書き、`KICK.try_dispatch` が消費する（[キックの保留と起動](https://ekicyou.github.io/pasta/internals/debug.html#キックの保留と起動kicklua)） |
| `kick_force` | `boolean` | キックの割り込み許可（既定 `false`） | `KICK.install` が立て、仮想イベントの `dispatch` の入口が 1 回で下ろす |
| `co_callback` | `thread\|nil` | コールバック待ちとして登録したコルーチンの印 | `pasta.store` は初期化しない（未設定のときは `nil`）。`CALLBACK.consume_staged` が書き、直後の `set_co_scene`（どちらも `EVENT.drive` の中）が `nil` に戻す。`CALLBACK.reset` も `nil` に戻す（[コールバック待ちとの関係](https://ekicyou.github.io/pasta/internals/execution-model.html#コールバック待ちとの関係)） |

単語の 3 つのフィールドの値の形は [finalize_scene](#finalize_scene) の「単語収集データ構造」で扱う。

### @pasta_config からの初期化

`pasta.store` は、最初に `require` されたときに `@pasta_config` を `pcall(require, "@pasta_config")` で読み、`actor` が表なら次の 2 つを行う。

- `STORE.actors` に `CONFIG.actor` の表そのものを入れる（複製しない）。以後 `get_or_create` が足すアクターも同じ表に入る。
- `CONFIG.actor` の各要素のうち、表であり `spot` が数値のものについて、`STORE.actor_spots[アクター名] = spot` とする。

`@pasta_config` を取得できない環境（`@pasta_config` を登録しない単体テストなど）でも、`pcall` で保護しているため読み込みは失敗せず、両フィールドは空の表のままになる。ランタイムの構築では `@pasta_config` の登録が `pasta.store` の最初の `require` より先に行われる（[VM の構築とモジュール登録](https://ekicyou.github.io/pasta/internals/execution-model.html#vm-の構築とモジュール登録)）。`CONFIG.actor` の要素にアクターのメタテーブルを付けるのは `pasta.actor` である（[アクターオブジェクト](#アクターオブジェクト)）。

### reset()

`STORE.reset()` は、STORE の状態を初期値に戻す。ランタイムのモジュールは呼ばず、テストが状態を消すために使う。

```lua
STORE.reset()
```

- `co_scene` は、`coroutine.close` があり中断中なら閉じてから `nil` にする。LuaJIT 2.1 には `coroutine.close` が無いため、ランタイムでは閉じる処理は実行されず、参照を外すだけになる（[LuaJIT 2.1 の制約](https://ekicyou.github.io/pasta/internals/execution-model.html#luajit-21-の制約)）。
- `actors`・`actor_spots`・`scenes`・`app_ctx`・`counters`・`global_words`・`local_words`・`actor_words`・`appearance` を新しい空の表（`appearance` は空の 4 つの表を持つ表）にし、`last_global_scene`・`kick_pending` を `nil`、`kick_force` を `false` にする。
- `@pasta_config` からの初期化はやり直さない。`actors`・`actor_spots` は空の表になる。
- `co_callback` は変えない。コールバックの状態を消すのは `CALLBACK.reset()` である。
- フィールドの表を新しい表に置き換えるため、すでに作られた ACT が持つ `act.app_ctx`・`act.actors` は古い表を指したまま残る。

### 循環参照回避の原則

共有するデータは STORE に集め、他のモジュールが STORE を `require` する一方向の依存を保つ。STORE 自身は他の `pasta.*` モジュールを `require` しない。

```lua
-- 正しい依存の向き: 他のモジュール → STORE
-- pasta.store
local STORE = {}
STORE.actors = {}
return STORE

-- pasta.actor
local STORE = require("pasta.store")
STORE.actors["さくら"] = { name = "さくら" }
```

例外は Rust 製のモジュール `@pasta_config` だけであり、`pcall` を通して `require` する（前節）。`@pasta_config` は Lua のモジュールを `require` しないため、循環は生じない。モジュール全体の `require` の関係は [Lua 側のモジュールの関係](https://ekicyou.github.io/pasta/internals/execution-model.html#lua-側のモジュールの関係) で扱う。

## ACT の内部

ACT は、シーン関数が第 1 引数で受け取るオブジェクトである。`pasta.act` が基本の ACT を、`pasta.shiori.act` が SHIORI 用の ACT を作る。SHIORI のイベントごとに `SHIORI_ACT.new(STORE.actors, req)` で新しい ACT が作られる（[イベントからシーンへ](https://ekicyou.github.io/pasta/internals/execution-model.html#イベントからシーンへ)）。この節では ACT の構造と、メソッド・アクタープロキシの解決、`init_scene`、名前の解決のメソッド、生成コード用のメソッドと動的参照のキーの変換を扱う。トーク系メソッドなど個々のメソッドの使い方は [スクリプト用ランタイム API](script-api.md) で、トークンの蓄積と組立は [トーク出力とアピアランス](https://ekicyou.github.io/pasta/internals/talk-output.html#トークンの蓄積) で扱う。

### ACT オブジェクトの構造

`ACT.new(actors)` は次のフィールドを持つ表を作り、メタテーブル `ACT_IMPL` を付ける。ここでは各フィールドの作られ方を示す。スクリプトからの使い方は [ACT のフィールド](script-api.md#act-のフィールド) が正である。

| フィールド | 値 |
| ---------- | -- |
| `actors` | 引数の `actors`（`nil` なら空の表）。SHIORI 用の ACT では `STORE.actors` そのもの |
| `save` | `require("pasta.save")` の結果。全 ACT で同じ表（[SAVE モジュールの内部](#save-モジュールの内部)） |
| `app_ctx` | `STORE.app_ctx` と同じ表 |
| `var` | ACT ごとの新しい空の表 |
| `token` | 積んだトークンの配列。`build` が新しい空の配列に置き換える |
| `current_scene` | 実行中のシーンテーブル。初期値は `nil` で、`init_scene` が設定する |

`SHIORI_ACT.new(actors, req)` は `ACT.new(actors)` で作った表に、`_spot_newlines`（`pasta.config` から読む `[ghost]` の `spot_newlines`。既定 1.5）と `req`（SHIORI のリクエストの表）を足し、メタテーブルを `SHIORI_ACT_IMPL` に付け替える。`act.req` のフィールドは [act.req](shiori-events.md#actreq) が正である。

シーンのコルーチンに渡った ACT は、そのコルーチンが終わるまで使われ続ける（[再開のループ](https://ekicyou.github.io/pasta/internals/execution-model.html#再開のループresume_until_valid)）。Call で呼んだ先のシーン関数も同じ ACT を受け取るため、`var` と `token` は呼び出しの連鎖の全体で共有される。

### メソッドとアクタープロキシの解決（__index）

ACT のフィールドに無いキーの参照は、メタテーブルの `__index` 関数が解決する。

```text
act[key]（ACT のフィールドに無いとき）
  SHIORI 用の ACT: rawget(SHIORI_ACT_IMPL, key) があればそれ
                   無ければ ACT_IMPL.__index(act, key) へ
  ACT_IMPL.__index:
    1. ACT_IMPL[key] があればそれ（メソッド）
    2. act.actors[key] があれば ACTOR.create_proxy(アクター, act)（アクタープロキシ）
    3. nil
```

- 解決の優先順はフィールド → メソッド → アクター名である。この優先順により、メソッド名やフィールド名と同じ名前のアクターがプロキシにならないこと（利用者から見た振る舞い）は [アクタープロキシ](script-api.md#アクタープロキシ) が正である。
- アクタープロキシは参照のたびに新しく作られ、キャッシュされない（[PROXY パターン](#proxy-パターン)）。
- 生成コードはアクターをこの経路で引かず、`act:actor_proxy("名前")` を呼ぶ（[生成コード用のメソッド](#生成コード用のメソッドactor_proxyglobal_fnarith)）。`actor_proxy` は `act.actors[名前]` を直接引くため、メソッド名・フィールド名と同じ名前のアクターもプロキシになる。
- `SHIORI_ACT_IMPL` 自身にも `__index = ACT.IMPL` のメタテーブルが付いており、`SHIORI_ACT_IMPL.talk` のように実装表から直接引いても `ACT_IMPL` のメソッドが得られる。

### 継承（ACT.IMPL）

`pasta.act` は実装表を `ACT.IMPL` として、`pasta.shiori.act` は `SHIORI_ACT.IMPL` として公開する。`pasta.shiori.act` は `ACT.IMPL` を継承し、`build` をさくらスクリプト文字列を返す処理に差し替え（[トーク出力とアピアランス](https://ekicyou.github.io/pasta/internals/talk-output.html)）、`get_property`・`set_property`・`transfer_date_to_var`・`transfer_req_to_var` を加える。差し替えた `build` の中では、親の処理を `ACT.IMPL.build(self)` で呼ぶ。

### init_scene

`ACT_IMPL.init_scene(self, scene)` は、シーン関数の実行を始めるときに ACT を初期化し、永続化データの表とローカル変数の表を返す。

```lua
--- @param scene SceneTable シーンテーブル（生成コードの SCENE）
--- @return table save 永続化データの表（act.save）
--- @return table var ローカル変数の表（act.var）
function ACT_IMPL.init_scene(self, scene) end
```

```lua
function SCENE.__start__(act, ...)
    local args = { ... }
    local save, var = act:init_scene(SCENE)
    -- 以降のアクション
end
```

処理は次の 3 つだけである。

1. `scene.__global_name__` があれば、`STORE.last_global_scene` に記録する。`pasta.scene` が作るシーンテーブルはすべて `__global_name__` を持つため、生成コードからの呼び出しでは常に記録される。
2. `self.current_scene = scene` とする。
3. `self.save, self.var` を返す。新しい表は作らない。

- トランスパイラは、すべてのシーン関数の先頭に `local save, var = act:init_scene(SCENE)` を生成する（[生成される Lua コードの形](https://ekicyou.github.io/pasta/internals/transpiler.html#生成される-lua-コードの形)）。
- `current_scene` は、単語参照・Call・式関数の名前の解決のうち、L1（`current_scene[key]` の完全一致）と L2（`current_scene.__global_name__` をスコープとするローカル辞書の前方一致）が使う（[ローカル優先の検索順](https://ekicyou.github.io/pasta/internals/registry-search.html#ローカル優先の検索順)）。`init_scene` を呼ばない関数の中では、`current_scene` は直前に `init_scene` したシーンのままである。
- `init_scene` は `current_scene` と `last_global_scene` を上書きするだけで、呼び出し元へ戻ったときに元へ戻す処理は無い。別のグローバルシーンを Call すると、戻った後の呼び出し元の名前の解決と、それ以降の選択肢の検索の親は、呼び出し先のシーンのものになる。

### 名前の解決のメソッド

ACT の `word`・`expr_fn`・`expr_fn_var`・`find_scene`・`call` と `find_handler`・`find_act_handler` の引数・戻り値・警告は [検索と呼び出し](script-api.md#検索と呼び出し)（[word(name, var_path)](script-api.md#wordname-var_path)・[expr_fn_var(value, var_path, ...)](script-api.md#expr_fn_varvalue-var_path-) を含む）が、検索の各段の意味は [ローカル優先の検索順](https://ekicyou.github.io/pasta/internals/registry-search.html#ローカル優先の検索順) が正である。ここでは実装の内部の構成だけを扱う。

- `expr_fn(self, key, ...)` と `expr_fn_var(self, value, var_path, ...)` は、`pasta.act` の局所関数 `call_expr(self, key, skip_methods, ...)` を共有する。`expr_fn` は `call_expr(self, key, nil, ...)`、`expr_fn_var` はキーに直した後に `call_expr(self, キー, true, ...)` を呼ぶ。`call_expr` は `find_handler("expr", key, skip_methods)` が関数なら `h(self, ...)` の戻り値を返し、それ以外は接頭辞 `act:expr_fn` の警告ログ（`handler not found`）を出して `nil` を返す。アクタープロキシは `pasta.actor` に同じ形の別の局所関数を持つ（[PROXY_IMPL のメソッド](#proxy_impl-のメソッド)）。
- `skip_methods` を真にするのは、`var_path` を受け取った `word` と `expr_fn_var`（生成コードの動的参照）だけである。`find_scene`・`call` と、`var_path` の無い `word`・`expr_fn` は `skip_methods` を渡さない。`find_handler` は `find_act_handler` に引数をそのまま渡す。

### 生成コード用のメソッド（actor_proxy・global_fn・arith）

アクション行のアクター（`act:actor_proxy("名前")`）、`＠＊名前（…）`（`act:global_fn("名前", …)`）、式の算術（`act:arith(…)`）の生成コードが呼ぶメソッドである。引数・戻り値・警告の文言は [アクター・グローバル関数・算術](script-api.md#アクターグローバル関数算術) が、生成コードの形は [生成される Lua コードの形](https://ekicyou.github.io/pasta/internals/transpiler.html#生成される-lua-コードの形) が正である。ここでは実装の内部の構成だけを扱う。

- `actor_proxy(self, name)` は、`self.actors[name]` があれば `ACTOR.create_proxy(アクター, self)` を返す。無ければ `self.token` を末尾から見て、最初に当たる `talk`・`sakura_script` のトークンの `actor.name` が `name` と同じならその `actor` の表を再利用してプロキシを作る。そうでなければ `{ name = name }`（メタテーブルなし）を作り、警告ログを出し、目印の `talk` トークン（`text` は `【未登録アクター：名前】`）を積んでからプロキシを作る。再利用によって同じ未登録の名前の連続する発言は同じ表を持ち、`build` のグループ化（表の同一性で判定する）で 1 つのグループになる（[グループ化トークン](https://ekicyou.github.io/pasta/internals/talk-output.html#グループ化トークン)）。
- その場限りのアクターは `STORE.actors`・`self.actors`・`STORE.actor_spots` のどれにも書かれず、ACT にもフィールドを足さない。状態は `self.token` の中にしか無いため、`build`・`yield` でトークンが空になると、次の発言で目印がまた付く。`name` だけの表のため、プロキシの検索の A1 は `name` にしか一致せず、A2 のアクター単語も無い（[PROXY_IMPL のメソッド](#proxy_impl-のメソッド)）。
- `global_fn(self, name, ...)` は `GLOBAL[name]` が関数なら `f(self, ...)` の戻り値をすべて返し、関数でなければ警告ログを出して `nil` を返す。名前の解決の 5 段の検索（`find_act_handler`）は通らない。関数の中で起きたエラーは捕まえない。
- `arith(self, op, lhs, rhs, lhs_desc, rhs_desc)` は、局所関数 `arith_operand` で被演算子を数値にし（`number` はそのまま、`string` は `tonumber`、それ以外は数値にできない）、両方が数値になったときだけ局所の表 `ARITH_OPS` の関数で Lua の演算子を適用する。演算子に渡るのは数値だけのため、表の `__add` などのメタメソッドは呼ばれない。数値にできない被演算子ごとに警告ログを出すが、値も説明も `nil` の被演算子（内側の `arith` が既に失敗したもの）では出さない。`self` は使わず、ACT の状態を読み書きしない。

### 動的参照のキー（WORD.dynamic_key）

`WORD.dynamic_key` の変換規則と警告は [WORD.dynamic_key(value, var_path, via)](script-api.md#worddynamic_keyvalue-var_path-via) が正である。この関数は、`pasta.act`（`word`・`expr_fn_var`）と `pasta.actor`（同名のプロキシのメソッド）の両方が使うため、どちらも `require` しない `pasta.word` に置かれている。`pasta.word` が `require` するのは `pasta.store` と `@pasta_log` だけであり、`pasta.act`・`pasta.actor` が `pasta.word` を `require` しても循環は生じない。

## PROXY パターン

アクタープロキシは、`act:actor_proxy("さくら")` を呼んだとき、または `act.さくら` のようにアクター名で ACT を参照したときに作られる小さな表である。アクターと ACT への参照を持ち、アクターを添えて ACT のメソッドに委ねる。生成コードはアクターの発言と、アクター修飾付きの単語参照・関数呼び出しを、`actor_proxy` で得たプロキシで書く。

```lua
act:actor_proxy("さくら"):talk("こんにちは")
act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("名前"))   -- アクター単語から探す
act:actor_proxy("さくら"):talk((act:actor_proxy("さくら"):expr_fn("関数名", 引数)))   -- アクター修飾付きの関数呼び出し
act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word(var.x, "var.x"))   -- 動的単語参照（変数の値をキーにする）
```

プロキシの実装（`PROXY_IMPL`）とアクターオブジェクトは、どちらも `pasta.actor` にある。

### アクターオブジェクト

アクターオブジェクトは `name`（アクター名）と `spot`（`nil` または数値）を持つ表で、メタテーブル `ACTOR_IMPL` を持つ。

| 関数・メソッド | 内容 |
| -------------- | ---- |
| `ACTOR.get_or_create(name)` | `STORE.actors[name]` が無ければ `{ name = name, spot = nil }` を作ってメタテーブルを付け、登録する。あればそれを返す。同じ名前には常に同じオブジェクトを返す。`pasta` の `create_actor` はこの関数そのもの |
| `ACTOR_IMPL.create_word(self, key)` | アクター単語のビルダーを返す。生成コードの `ACTOR:create_word(キー):entry(値, …)` が使う |
| `ACTOR.create_proxy(actor, act)` | アクタープロキシを作る（次節） |

- `create_word` が返すビルダー（`ACTOR_WORD_BUILDER_IMPL`）は、内部に `WORD.create_actor(アクター名, キー)` のビルダーを `_word_builder` として持ち、`entry(...)` を値が 1 つ以上あるときだけそれに渡して自身を返す。登録先は `STORE.actor_words` だけであり、アクターオブジェクトのフィールドには値を設定しない。
- `pasta.actor` は読み込み時に、`STORE.actors` の要素のうち表であるもの（`@pasta_config` の `actor` から来たアクター）に、`name` が無ければ表のキーを `name` として補い、メタテーブル `ACTOR_IMPL` を付ける。そのため `pasta.toml` の `[actor.名前]` に書いたキー（`spot` など）は、アクターオブジェクトのフィールドとして見える。

### プロキシの構造と生成

`ACTOR.create_proxy(actor, act)` は `{ actor = アクター, act = ACT }` にメタテーブル `PROXY_IMPL`（`__index` は `PROXY_IMPL` 自身）を付けて返す。ACT の `actor_proxy` と `__index` が、呼び出し・参照のたびにこの関数を呼ぶ（[生成コード用のメソッド](#生成コード用のメソッドactor_proxyglobal_fnarith)・[メソッドとアクタープロキシの解決](#メソッドとアクタープロキシの解決__index)）。プロキシは状態を持たず、同じアクターのプロキシを何度作っても振る舞いは変わらない。

### PROXY_IMPL のメソッド

| メソッド | 処理 | 戻り値 |
| -------- | ---- | ------ |
| `talk(self, text, var_name)` | `self.act:talk(self.actor, text, var_name)` | `nil`（メソッドチェーンはできない） |
| `sakura_script(self, text)` | `self.act:sakura_script(self.actor, text)` | `nil` |
| `find_actor_handler(self, mode, key, skip_methods)` | `mode` が `"word"` でなければ `nil`。`self.actor[key]`（`skip_methods` が真なら `rawget(self.actor, key)`）が `nil` でなければそれ（A1）。次に `@pasta_search` を `pcall(require, …)` で得られれば `SEARCH:search_word(key, "__actor_" .. アクター名 .. "__")`（A2） | 見つかった値、または `nil` |
| `find_handler(self, mode, key, skip_methods)` | `find_actor_handler` で見つからなければ `self.act:find_act_handler(mode, key, skip_methods)` に委ねる。`skip_methods` は両方に渡す | 見つかった値、または `nil` |
| `word(self, name, var_path)` | `var_path` が `nil` なら、`name` が `nil` か空文字列のとき `nil` を返す。`var_path` があれば `WORD.dynamic_key(name, var_path, "proxy:word")` でキーにし（`nil` ならそこで `nil` を返す）、`skip_methods` を真にする。そのうえで `find_handler` と同じ順序で、`find_actor_handler("word", …)`（アクターの段）、見つからなければ `self.act:find_act_handler("word", …)`（ACT の段）を引く（ACT の `word` と同じ規則。[word(name, var_path)](script-api.md#wordname-var_path)）。結果が関数なら、アクターの段で見つかったときは `h(self)`（引数はプロキシ）、ACT の段で見つかったときは `h(self.act)`（引数は ACT）を呼び、戻り値を `drop_self` で正規化して返す。それ以外の値なら `tostring(h)`。見つからなければ警告ログ（`via=proxy(アクター名)`）を出す | 単語の文字列、関数の戻り値（正規化後）、または `nil` |
| `expr_fn(self, key, ...)` | 局所関数 `call_expr(self, key, nil, ...)` | 関数の戻り値、または `nil` |
| `expr_fn_var(self, value, var_path, ...)` | `value` を `WORD.dynamic_key(value, var_path, "proxy:expr_fn")` でキーにし（`nil` ならそこで `nil` を返す）、`call_expr(self, キー, true, ...)` | 関数の戻り値、または `nil` |

- `call_expr` は `pasta.actor` の局所関数で（`pasta.act` の同名の局所関数とは別）、`find_handler("expr", key, skip_methods)` の結果が関数なら `h(self.act, ...)`（第 1 引数は ACT）を呼び、戻り値を `drop_self` で正規化して返す。関数でなければ警告ログ（`proxy:expr_fn - handler not found`）を出す。`"expr"` モードでは `find_actor_handler` が `nil` を返すため、見つかる関数は常に ACT の段のものである。
- `drop_self(self, r, ...)` は `pasta.actor` の局所関数で、先頭の戻り値 `r` が `self.act` または `self`（プロキシ）と `==` で等しければ `nil` を返し、そうでなければ `r, ...` を複数の戻り値も含めてそのまま返す。ACT・プロキシは `__eq` を持たないため、比較は同一性である。正規化はプロキシの `word`・`expr_fn`・`expr_fn_var` だけが行い、ACT の `word`・`expr_fn`・`expr_fn_var`・`global_fn` は関数の戻り値をそのまま返す。
- A1 は通常の添字参照であるため、アクターオブジェクトのフィールド（`name`・`spot`・`pasta.toml` の `[actor.名前]` のキー）と、メタテーブル経由の `create_word` も一致の対象になる。`skip_methods` が真のとき（動的参照）は `rawget` で引くため、アクターオブジェクト自身のフィールドだけが対象になり、`create_word` などのメソッドには一致しない。
- A2 が渡すスコープ名は元のアクター名から組み立てるが、`search_word` の入口でサニタイズされてから照合されるため、記号を含むアクター名（`さくら・改`）でも、`register_actor` がサニタイズした名前で登録したキー（`:__actor_さくら_改__:…`）に一致する（[照合規則の共有](https://ekicyou.github.io/pasta/internals/registry-search.html#照合規則の共有)）。
- 検索の全体の順序（A1 → A2 → L1〜L5）と各段の意味は [ローカル優先の検索順](https://ekicyou.github.io/pasta/internals/registry-search.html#ローカル優先の検索順) で扱う。
- `word` が見つけた関数に渡す第 1 引数は、アクターの段で見つかったときはプロキシ、ACT の段で見つかったときは ACT である。`expr_fn`・`expr_fn_var` が見つけた関数には常に ACT を渡す。ACT の `word`・`expr_fn`・`expr_fn_var`・`call` が見つけた関数にも ACT を渡す。利用者から見た規則は [関数スコープの展開先](https://ekicyou.github.io/pasta/grammar/variables.html#関数スコープの展開先) が正である。

## SCENE モジュール

`pasta.scene` は、シーンテーブルの作成と登録、検索結果からシーン関数への解決、シーン関数のコルーチン化を担うモジュールである。

```lua
local SCENE = require("pasta.scene")
```

### シーンテーブル

シーンテーブルは、グローバルシーン 1 つにつき 1 つ作られ、`STORE.scenes[グローバルシーン名]` に入る。

```text
STORE.scenes = {
  ["メイン1"] = {                         -- シーンテーブル
    __global_name__ = "メイン1",
    __start__        = function(act, ...) … end,   -- グローバルシーンの本体
    ["選択肢_1"]     = function(act, ...) … end,   -- ローカルシーン
    -- メタテーブル: __index = SCENE_TABLE_IMPL（create_word を持つ）
  },
}
```

- グローバルシーン名は、トランスパイラが渡す基本名（サニタイズ済みのシーン名）に、`create_scene` が振った番号を区切り無しで付けたものである（`メイン` → `メイン1`）。
- `__global_name__` 以外のキーはすべてシーン関数として扱われ、辞書確定で `(グローバルシーン名, キー)` の組として集められる（[finalize_scene](#finalize_scene)）。
- メタテーブルの `__index` が `SCENE_TABLE_IMPL` を指すため、`scene:create_word(キー)` は `WORD.create_local(scene.__global_name__, キー)` のビルダーを返す。生成コードの `SCENE:create_word(キー):entry(値, …)` がこれを使う。通常の添字参照（ACT の L1 の完全一致を含む）でも、キー `create_word` はこのメソッドに一致する。動的参照の L1 は `rawget` で引くため、シーンテーブル自身のキー（`__global_name__` とシーン関数）だけが対象になり、`create_word` には一致しない。

### SCENE の関数

| 関数 | 内容 |
| ---- | ---- |
| `create_scene(base_name, local_name, scene_func)` | `get_or_increment_counter(base_name)` で番号を得て、グローバルシーン名 `base_name .. 番号` を作る。`local_name` と `scene_func` が両方あれば `register` で登録する。そのグローバルシーン名のシーンテーブルを（無ければ作って）返す。`pasta` の `create_scene` はこの関数そのもの |
| `get_or_increment_counter(base_name)` | `STORE.counters[base_name]` を 1 増やして返す（最初は 1） |
| `register(global_name, local_name, scene_func)` | シーンテーブルが無ければ作り、`[local_name] = scene_func` とする |
| `create_global_table(global_name)` | シーンテーブルが無ければ作って返す |
| `get_global_table(global_name)` | `STORE.scenes[global_name]` を返す |
| `get(global_name, local_name)` | シーン関数を返す。無ければ `nil` |
| `get_start(global_name)` | `get(global_name, "__start__")` |
| `get_global_name(scene_table)` | `scene_table.__global_name__` を返す |
| `get_all_scenes()` | `STORE.scenes` そのもの（複製しない）を返す |
| `search(name, global_scene_name, attrs)` | 前方一致の検索でシーンを探し、結果オブジェクトを返す（後述） |
| `co_exec(act, name, global_scene_name, attrs)` | `act:find_scene` で探したシーン関数をコルーチンに包んで返す（後述） |

- 生成コードは `create_scene` を基本名だけで呼び（`local SCENE = PASTA.create_scene("メイン")`）、返ったシーンテーブルに `function SCENE.__start__(act, ...)` の形で関数を直接定義する。`register`・`create_global_table` などは `pasta.scene` の内部とテストが使う。
- 番号は基本名ごとに全モジュールを通して振られる。`pasta.scene_dic` がモジュールを名前順に `require` するため、同じ基本名のシーンの番号はその順（モジュールの中では出現順）で決まる。

### search と結果オブジェクト

`SCENE.search(name, global_scene_name, attrs)` は次の順に処理する。`attrs` は使わない。

1. `name` が文字列でなければ `nil` を返す。
2. `@pasta_search` を `require` し（呼び出し時に読み込む）、`SEARCH:search_scene(name, global_scene_name)` を呼ぶ。`global_scene_name` が `nil` ならグローバルシーンだけを探してローカル名 `"__start__"` を、文字列ならそのグローバルシーンの中のローカルシーンだけを探す。
3. 見つからなければ `nil`。見つかった `(グローバルシーン名, ローカル名)` で `SCENE.get` を引き、関数が無ければ `nil` を返す。
4. 次の結果オブジェクトを返す。

```lua
--- @class SceneSearchResult
--- @field global_name string グローバルシーン名
--- @field local_name string ローカル名（グローバルシーンなら "__start__"）
--- @field func function シーン関数
-- メタテーブルの __call により result(...) は result.func(...) と同じ
```

前方一致の候補の集め方と選び方は [シーン・単語レジストリとシーン検索](https://ekicyou.github.io/pasta/internals/registry-search.html#前方一致による候補の収集) で扱う。ACT の名前の解決（L2・L5）は結果オブジェクトの `func` を使う。

### co_exec

`SCENE.co_exec(act, name, global_scene_name, attrs)` は、`act:find_scene(name, global_scene_name, attrs)` でシーン関数を探し、関数でなければ `nil` を、関数ならそれを包んだコルーチンを返す。`act:find_scene` は `global_scene_name` と `attrs` を使わず `find_handler("scene", name)` に委ねるため、`co_exec` で親のグローバルシーンを指定してローカルシーンを引くことはできない。第 1 引数の `act` は名前の解決にだけ使い、シーン関数には `coroutine.resume` で渡された ACT が渡る。コルーチンの包み方と再開は [シーンコルーチンの作り方](https://ekicyou.github.io/pasta/internals/execution-model.html#シーンコルーチンの作り方) で扱う。

### DSL の Lua ブロックとの接続

トランスパイラは、グローバルシーンとその配下に書かれた Lua ブロックを、そのグローバルシーンの `do … end` の中、すべてのシーン関数の定義の後に出力する（[生成される Lua コードの形](https://ekicyou.github.io/pasta/internals/transpiler.html#生成される-lua-コードの形)）。この位置ではローカル変数 `SCENE` がそのシーンテーブルを指すため、Lua ブロックで定義した関数はシーンテーブルに入る。

```lua
function SCENE.関数名(act, ...)
    local save, var = act:init_scene(SCENE)
    act.さくら:talk("セリフ")
end
```

- こうして定義した関数は、生成されたシーン関数と同じくシーンテーブルのキーになり、辞書確定でローカルシーンとして集められる。
- DSL の `＠関数名(…)` は `act:expr_fn("関数名", …)` などに、`＞関数名` は `act:call(…)` になり、どちらも L1 の完全一致（`current_scene[key]`）でこの関数に一致する。
- 関数の中で `init_scene` を呼ぶと `current_scene` と `last_global_scene` がこのシーンテーブルになり、`save`・`var` が得られる（[init_scene](#init_scene)）。書き方の利用者向けの説明は [スクリプト用ランタイム API](script-api.md) で扱う。

## SAVE モジュールの内部

`pasta.save` は、`@pasta_persistence` の `load()` の結果をそのまま返すモジュールである。実装は次のとおりである。

```lua
local persistence = require("@pasta_persistence")
local save = persistence.load()
return save
```

- `load()` が呼ばれるのは、`pasta.save` が最初に `require` されたときの 1 回だけである。結果の表は `package.loaded["pasta.save"]` に入り、以後の `require` は同じ表を返す。
- `ACT.new` が ACT ごとに `require("pasta.save")` を `act.save` に入れるため、全 ACT と `init_scene` の戻り値 `save` は同じ表を指す。
- ランタイムの破棄時に保存されるのは `package.loaded["pasta.save"]` の表である。読み書きの時点と保存の処理は [永続化](https://ekicyou.github.io/pasta/internals/execution-model.html#永続化) で、`@pasta_persistence` の API と設定は [@pasta_persistence](pasta-persistence.md) で扱う。キーの命名規約とアクセスの方法は [スクリプト用ランタイム API](script-api.md) で扱う。

## finalize_scene

`finalize_scene` は、Lua 側に登録されたシーンと単語から `@pasta_search` を作り直す関数である。この節では Lua 側の窓口と、Rust 側に渡す収集データの構造を扱う。Rust 側での収集・レジストリの構築・`@pasta_search` の登録は [辞書確定](https://ekicyou.github.io/pasta/internals/registry-search.html#辞書確定) で扱う。

```lua
require("pasta").finalize_scene()
```

### Lua 側の窓口

- `pasta`（`pasta` モジュール）は、何もしないスタブの `PASTA.finalize_scene` を持つ。
- ランタイムの構築は、`main`・`pasta.shiori.entry`・`pasta.scene_dic` を読み込む前に、Rust の `register_finalize_scene` でこのスタブを Rust の関数に置き換える。`pasta` がまだ読み込まれていなければ、その場で `require` してから置き換える。
- そのため、ランタイムの上で `finalize_scene` を呼ぶと常に Rust 側の処理が動く。Rust を通さずに `pasta` を読み込む環境（単体テストなど）では、スタブが呼ばれて何も起きない。
- `@pasta_search` は VM の構築時にも、トランスパイル時のレジストリから一度登録されている。`finalize_scene` はそれを、Lua 側の登録から作った検索表で置き換える。

### 呼び出しタイミング

`finalize_scene` を呼ぶのは、ローダが生成する `pasta.scene_dic`（`scene_dic.lua`）の末尾である。`pasta.scene_dic` は全シーンモジュールを名前順に `require` してから `finalize_scene` を呼ぶ。

```lua
-- scene_dic.lua（自動生成）
require("pasta.scene.main")
require("pasta.scene.sub")

require("pasta").finalize_scene()
```

- シーンモジュールの実行（生成コードの `create_actor`・`create_scene`・`create_word` と、シーンテーブルへの関数定義）で、Lua 側のテーブルが埋まる。`finalize_scene` はその後に 1 回だけ呼ばれる。
- `main` は `pasta.scene_dic` より前に読み込まれるため、`main` で `WORD.create_global` などを使って登録した単語も収集の対象になる。`finalize_scene` より後に登録したシーン・単語は、もう一度 `finalize_scene` を呼ぶまで検索の対象にならない。
- `scene_dic.lua` の生成は [トランスパイル結果キャッシュ](https://ekicyou.github.io/pasta/internals/transpiler.html#トランスパイル結果キャッシュ) で扱う。

### シーン収集データ構造

Rust 側は `pasta.scene` の `get_all_scenes()` を呼び、`STORE.scenes` を受け取る。

```lua
{
  ["メイン1"] = {
    __global_name__ = "メイン1",
    __start__ = function(act, ...) … end,
    ["選択肢_1"] = function(act, ...) … end,
  },
}
```

- 外側のキーはグローバルシーン名、内側はシーンテーブル（[シーンテーブル](#シーンテーブル)）である。
- Rust 側での集め方（`__global_name__` 以外のキーを組にすること、関数を写さないこと）は [辞書確定](https://ekicyou.github.io/pasta/internals/registry-search.html#辞書確定) で扱う。

### 単語収集データ構造

Rust 側は `pasta.word` の `get_all_words()` を呼ぶ。この関数は、STORE の 3 つの表をまとめた新しい表を返す。

```lua
{
  global = {                       -- STORE.global_words
    ["キー"] = { { "値1", "値2" }, { "値3" } },
  },
  ["local"] = {                    -- STORE.local_words
    ["メイン1"] = {                -- グローバルシーン名
      ["キー"] = { { "ローカル値" } },
    },
  },
  actor = {                        -- STORE.actor_words
    ["さくら"] = {                 -- アクター名
      ["キー"] = { { "アクター値" } },
    },
  },
}
```

- 単語キーの値は「値リストの配列」である。ビルダーの `entry(...)` を 1 回呼ぶごとに、その引数を値リスト 1 つとして配列の末尾に足す。引数が 0 個の `entry()` は何も足さない。
- Rust 側での集め方（値リスト 1 つにつき 1 つの収集項目を作り、収集した表でスコープが決まること）は [辞書確定](https://ekicyou.github.io/pasta/internals/registry-search.html#辞書確定) で扱う。
- `local` は Lua の予約語であるため、表のキーは `["local"]` と書く。

### 単語ビルダーの内部

`pasta.word` のビルダー（`WORD_BUILDER_IMPL`）は、登録先の表 `_registry` と単語キー `_key` だけを持つ。

| 生成 | 登録先（`_registry`） |
| ---- | --------------------- |
| `WORD.create_global(key)`（別名 `WORD.create_word`・`pasta` の `create_word`） | `STORE.global_words` |
| `WORD.create_local(scene_name, key)`（`scene:create_word(key)` が使う） | `STORE.local_words[scene_name]`（無ければ作る） |
| `WORD.create_actor(actor_name, key)`（アクターの `create_word` が使う） | `STORE.actor_words[actor_name]`（無ければ作る） |

- ビルダーの生成時に `_registry[key]` が無ければ空の配列を作る。そのため、`entry` を呼ばずに作っただけのビルダーでも、値リストを 1 つも持たないキーが残る。
- `entry(...)` は `table.insert(_registry[_key], { ... })` を行って自身を返す。同じキーのビルダーを何度作っても、同じ配列に値リストが足されていく。
- `get_global_words()`・`get_local_words(scene_name)`・`get_actor_words(actor_name)` は STORE の該当する表をそのまま返す。

## ユーティリティモジュール

### pasta.buf

`pasta.buf` は、文字列を連結するためのバッファを作るモジュールである。LuaJIT の String Buffer Library（`string.buffer`）があればそれを使い、無ければ同じメソッドを持つ最小実装を使う。`pasta.shiori.sakura_builder` がさくらスクリプトの組立に使う（[さくらスクリプトの組立](https://ekicyou.github.io/pasta/internals/talk-output.html#さくらスクリプトの組立)）。

```lua
local buf = require("pasta.buf")
local b = buf.new()
b:put("Hello"):put(", "):put("World")
local s = b:tostring()  -- "Hello, World"
```

| API | 内容 |
| --- | ---- |
| `buf.new()` | バッファを作る。LuaJIT のときは `string.buffer` の `new` そのもの、それ以外は `new_fallback` |
| `buf.backend` | 採用したバックエンド（`"luajit"` か `"fallback"`） |
| `buf.new_fallback()` | 最小実装のバッファを明示的に作る（環境に依らないテストのため） |
| `buffer:put(s)` | 文字列を追記して自身を返す |
| `buffer:tostring()` | 追記した順に連結した文字列を返す。バッファの中身は消えない |

- バックエンドの選択は、モジュールの読み込み時に 1 回だけ `pcall(require, "string.buffer")` で行う。`require` に失敗しても、`new` が関数でなくても、例外は送出せず最小実装を選ぶ。
- LuaJIT のときは `string.buffer` のオブジェクトをラップせず生成関数を直接束ねるため、間接呼び出しの負担は無い。呼び出し側は `put`・`tostring` 以外のメソッドに依存しない。
- 最小実装は内部の配列 `_parts` に追記し、`tostring` で `table.concat` する。

### pasta.lua_version

`pasta.lua_version` は、実行中のランタイムの種類（標準 Lua か LuaJIT か）と版を 1 つの整数で返す。

```lua
local lua_version = require("pasta.lua_version")
local v = lua_version.get()   -- このランタイム（LuaJIT 2.1）では 221
if v >= 200 then
    -- LuaJIT 固有の機能（string.buffer など）を使える
end
```

| 戻り値 | 意味 |
| ------ | ---- |
| `1xy` | 標準 Lua x.y（例: `154` は Lua 5.4、`155` は Lua 5.5） |
| `2xy` | LuaJIT x.y（例: `220` は LuaJIT 2.0、`221` は LuaJIT 2.1） |

- 種類は `rawget(_G, "jit")` の有無で判定する。LuaJIT でも `_VERSION` は `"Lua 5.1"` を返すため、種類の判定には使わない。
- LuaJIT の版は `jit.version_num`（例 `20100`）から求め、数値でなければ `jit.version`（`"LuaJIT 2.1.…"`）の `major.minor` を解析する。どちらも得られなければ `221` を返す。
- 標準 Lua の版は `_VERSION` の `major.minor` を解析し、得られなければ `151` を返す。
- 例外を送出せず、常に整数を返す。
