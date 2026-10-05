# スクリプト用ランタイム API

ごきげんよう。`scripts/` から呼べるランタイムの道具――ACT、WORD、GLOBAL、SAVE を、ここで一望にいたしますわ。
どの道具で何ができるのか、手元に置いておけば迷うことはございませんの。さあ、参りましょう。

---

この章は、ゴースト作者が `scripts/` 配下の Lua スクリプトと Pasta DSL の Lua ブロックから呼ぶランタイム API のリファレンスである。作例と記述の型は [scripts/ の記述パターン](patterns.md) で扱う。対象方言は LuaJIT 2.1（Lua 5.1 系）である。

## ACT

ACT（`act`）は、シーン関数とイベントハンドラが第 1 引数で受け取るオブジェクトである。台詞や表示制御をトークンとして積み、ランタイムがそれをさくらスクリプトに組み立てて応答にする。SHIORI のイベントごとに新しい ACT が作られる。SHIORI リクエストの内容を表す `act.req` のフィールドは [act.req](shiori-events.md#actreq) を参照する。

### ACT のフィールド

| フィールド | 内容 |
| ---------- | ---- |
| `act.actors` | アクター名 → アクターオブジェクトの表（pasta.toml の `[actor]` とアクター辞書で定義したアクター） |
| `act.save` | 永続化データの表（[SAVE](#save)） |
| `act.var` | ローカル変数の表。DSL の `＄名前` は `var.名前` になる。ACT ごとに新しい表で、Call で呼んだ先のシーンやチェイントークの続きとも共有される（[ローカル変数](../grammar/variables.md#ローカル変数)） |
| `act.app_ctx` | ゴーストの実行中（辞書の再読込まで）保たれる汎用の表。全 ACT で同じ表であり、永続化はされない |
| `act.req` | SHIORI リクエストの内容（[act.req](shiori-events.md#actreq)） |
| `act.アクター名` | そのアクターのアクタープロキシ（[アクタープロキシ](#アクタープロキシ)） |

- 上記以外のフィールド（`act.token`・`act.current_scene` など）はランタイムが使う。スクリプトから書き換えない。

### init_scene

```lua
local save, var = act:init_scene(SCENE)
```

- Lua ブロックで定義するシーン関数（`function SCENE.名前(act, ...)`）の先頭で呼ぶ。`SCENE` は、その Lua ブロックが属するグローバルシーンのシーンテーブルである。
- 戻り値の `save`・`var` は `act.save`・`act.var` と同じ表である。
- 呼ぶと、そのシーンが実行中のシーンになる。名前の検索（[検索と呼び出し](#検索と呼び出し)）の 1・2 段目と、選択肢が記録する「選択肢を出したグローバルシーン」（[choice](#choicetarget-display)）は、実行中のシーンを基準にする。
- 実行中のシーンは、次に `init_scene` が呼ばれるまで変わらない。ただし、`call_restore`・`word`・`expr_fn`・`expr_fn_var`・`global_fn` で呼んだ関数から戻ると、呼ぶ前のシーンに戻る（[検索と呼び出し](#検索と呼び出し)）。
- シーン関数でない関数（`GLOBAL` の関数など）で `save`・`var` が必要なときは、`init_scene` を呼ばずに `act.save`・`act.var` を使う。
- シーン関数は、アクション行の `＠名前（…）`・`＠単語`・`＠＄変数名（…）`・`＠＄変数名` から呼ばれた場合も、第 1 引数に ACT を受け取る。アクタープロキシを受け取るのは、アクション行のアクターの表の関数だけである（[関数スコープの展開先](../grammar/variables.md#関数スコープの展開先)）。

```lua
function SCENE.カウント(act)
    local save, var = act:init_scene(SCENE)
    save.count = (save.count or 0) + 1   -- 永続化される
    var.temp = "一時データ"               -- この ACT の間だけ有効
end
```

### トーク

| メソッド | 積むもの | 戻り値 |
| -------- | -------- | ------ |
| `act:talk(actor, text)` | アクターの台詞 | `act` |
| `act:sakura_script(actor, text)` | アクターに属するさくらスクリプト | `act` |
| `act:raw_script(text)` | 変換しないさくらスクリプト | `act` |

#### talk(actor, text)

```lua
act:talk(actor, text) -> act
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `actor` | Actor | 発言するアクターのアクターオブジェクト（`act.アクター名.actor`） |
| `text` | any | 台詞。`nil` のときは何も積まない。それ以外の値は `tostring` で文字列にする |

- 台詞は、応答を組み立てるときに、アクターの表を使って `@pasta_sakura_script` の `talk_to_script` でウェイト付きのさくらスクリプトに変換される（[act:talk との関係](modules/pasta-sakura-script.md#acttalk-との関係)）。
- 発言するアクターが替わると、そのアクターの立ち位置を示す `\p[番号]` が台詞の前に出力される（[スポット操作](#スポット操作)）。同じアクターの続けての台詞は 1 つにつながる。
- 第 3 引数は生成コードが使う（変数参照の値が `nil` のときに警告ログへ出す変数名）。

```lua
act:talk(act.さくら.actor, "こんにちは")
act.さくら:talk("こんにちは")   -- アクタープロキシ経由。同じ台詞を積む
```

#### sakura_script(actor, text)

```lua
act:sakura_script(actor, text) -> act
```

アクターに属するさくらスクリプトを積む。アクション行に書いたさくらスクリプトの生成先である。`talk` と同じく、組み立て時に `talk_to_script` を通る。

#### raw_script(text)

```lua
act:raw_script(text) -> act
```

さくらスクリプトを、変換せずに（ウェイトの挿入を経ずに）積んだ順の位置へ出力する。アクターに属さないため、`\p[番号]` の出力やアクターの切り替えには関わらない。

```lua
act:raw_script("\\![raise,OnMyEvent]")
```

### SHIORI 固有のメソッド

SHIORI のイベントでランタイムが渡す ACT は SHIORI 用の ACT であり、`pasta.act` の共通のメソッドに加えて次のメソッドを持つ。テストなどで `pasta.act` の `ACT.new` から作った ACT には無い。

| メソッド | 内容 | 戻り値 |
| -------- | ---- | ------ |
| `act:set_property(name, value)` | ベースウェアのプロパティへの書き込み（次節） | `act` |
| `act:get_property(name_or_names, timeout, timeout_message)` | ベースウェアのプロパティの読み取り（次節） | プロパティの値 |
| `act:transfer_date_to_var()` | `act.req.date` の日時を `var` の日時変数に入れる。`act.req` か `act.req.date` が無ければ何もしない（[日時変数](../grammar/variables.md#日時変数)） | `act` |
| `act:transfer_req_to_var()` | Reference 0〜9 と `act.req.id`・`act.req.base_id` を `var` のリクエスト変数に入れる。`act.req` が無ければ何もしない（[リクエスト変数](../grammar/variables.md#リクエスト変数reference)） | `act` |

#### set_property(name, value)

```lua
act:set_property(name, value) -> act
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `name` | string | プロパティ名。`nil`・空文字列は不可 |
| `value` | any | 値。`nil` は空文字列として扱い、それ以外は `tostring` で文字列にする |

- プロパティの書き込みタグ `\![set,property,名前,値]` を、[raw_script](#raw_scripttext) と同じ扱いのトークンとして積む。
- `name` が `nil` か空文字列のときは `error` になる。
- 名前と値はタグの引数として自動でエスケープされる。`\` は `\\`、`%` は `\%`、`]` は `\]` になり、`,` か `"` を含む場合は全体を `"` で囲んで中の `"` を `""` にする。
- DSL の `＄％名前＝値` は `act:set_property("名前", 値)` になる（[プロパティ変数](../grammar/variables.md#プロパティ変数)）。

```lua
act:set_property("sakura.name", "Alice")
act:set_property("score", 100)   -- 数値は tostring される
act:set_property("flag", "on"):talk(act.さくら.actor, "設定しました")
```

#### get_property(name_or_names, timeout, timeout_message)

```lua
act:get_property(name_or_names, timeout?, timeout_message?) -> string | nil, ...
```

| パラメータ | 型 | 既定 | 説明 |
| ---------- | -- | ---- | ---- |
| `name_or_names` | string または string の配列 | — | プロパティ名、または名前の配列（1 つ以上） |
| `timeout` | number | `5` | 結果を待つ秒数 |
| `timeout_message` | string | `"callback timeout: get_property"` | タイムアウトしたときのエラーの理由 |

**戻り値**: 名前の数だけの値。名前を配列で渡したときは、配列の順の多値で返る。値が空文字列のものと、結果に含まれなかったものは `nil` になる。

- シーンのコルーチンの中でだけ呼べる。コルーチンの外（`REG` のハンドラの本体など）で呼ぶと `error` になる。
- `name_or_names` が `nil`・空文字列・空の配列のとき、配列に `nil` や空文字列の名前があるとき、文字列でも表でもないときは `error` になる。
- 呼ぶと `\![get,property,OnPastaCallBack{N},名前,…]` のタグだけを応答として返してシーンを中断し、ベースウェアから結果が届くと再開して値を返す（[OnPastaCallBack](shiori-events.md#onpastacallbackコールバック応答)）。
- 呼ぶ前に積んだトークンは、中断の応答には含まれない。再開した後にそのまま残り、以降に積んだトークンと一緒に出力される。
- `timeout` 秒を過ぎても結果が届かないと、シーンの中で `timeout_message` を理由とするエラーが発生し、シーンはそこで終わる（警告ログが出る）。期限は呼び出し時の `os.time()` に `timeout` を足した時刻で、判定は OnSecondChange の既定ハンドラが行う。
- 名前は `set_property` と同じ規則でエスケープされる。
- DSL の `＄x＝＄％名前` は `var.x = act:get_property("名前")` になる（[プロパティ変数](../grammar/variables.md#プロパティ変数)）。

```lua
local version = act:get_property("baseware.version")

local w, h = act:get_property({
    "currentghost.balloon.scope(0).validwidth.initial",
    "currentghost.balloon.scope(0).validheight.initial",
})

local name = act:get_property("sakura.name", 10, "name取得タイムアウト")
```

### 表示制御

| メソッド | 積むもの | 出力 |
| -------- | -------- | ---- |
| `act:surface(id)` | サーフェスの変更（`id` は数値または文字列） | `\s[id]` |
| `act:wait(ms)` | ウェイト。`ms` を切り捨てた整数で、負の値と `nil` は 0 | `\_w[ms]` |
| `act:newline(n)` | 改行。`n` を省くと 1 | `\n` を `n` 個 |
| `act:clear()` | 表示のクリア | `\c` |

- いずれも `act` を返し、続けて呼べる。
- 表示制御のトークンは、直前に積まれた `talk`・`sakura_script` のアクターの出力の中に入る（そのアクターの立ち位置に効く）。
- 1 回の出力（`yield` またはシーンの終了で区切られる範囲）の中で、まだ `talk` も `sakura_script` も積まれていないうちに積んだ表示制御は、どのアクターにも結び付かず、スコープ切替タグを付けずに、積んだ位置にそのまま出力される。`clear_spot` の後、まだ発言を積んでいないうちに積んだ表示制御も同じである。`choice`・`choice_timeout` も同じ規則に従う。
- 発言者の表情は、発言の中か発言の後に書く。発言より前に積んだ `surface` は、次の発言者の表情にはならない。

```lua
act:talk(act.さくら.actor, "えっ")
act:surface(5):wait(500):talk(act.さくら.actor, "驚いた！"):newline()
```

### スポット操作

| メソッド | 内容 | 戻り値 |
| -------- | ---- | ------ |
| `act:set_spot(name, number)` | アクター `name` の立ち位置（スポット番号）を `number` にする。`act.actors` に無い名前は無視する | `nil` |
| `act:clear_spot()` | すべてのアクターの立ち位置を消す | `nil` |

- 立ち位置の変更は、応答を組み立てるときに積んだ順に反映され、以降の発言の `\p[番号]` が変わる。設定はイベントをまたいで保たれる。
- 立ち位置が無いアクターは、スポット 0 で話す（警告ログが出る）。
- 1 回の出力の中で、同じアクターの発言が `set_spot` の前後に続くと、後の発言も変更前の立ち位置で出力される（変更はそのアクターの続く発言の後に反映される）。
- `clear_spot` の後の発言は、前の発言と同じアクターでも、`clear_spot` とその後の `set_spot` を反映した立ち位置で出力される。
- 戻り値が `nil` のため、続けて呼べない。
- DSL の `％` 行は、シーンの先頭で `clear_spot()` と、並べたアクターごとの `set_spot(名前, 番号)` を生成する（[シーンスコープ内でのアクター指定](../grammar/actor-dictionary.md#シーンスコープ内でのアクター指定)）。

### アクタープロキシ

`act.アクター名`（`act.さくら` など）は、そのアクターを添えて ACT のメソッドを呼ぶアクタープロキシを返す。

| 呼び方 | 内容 | 戻り値 |
| ------ | ---- | ------ |
| `act.さくら:talk(text)` | `act:talk(act.さくら.actor, text)` と同じ | `nil` |
| `act.さくら:sakura_script(text)` | `act:sakura_script(act.さくら.actor, text)` と同じ | `nil` |
| `act.さくら:word(name, var_path)` | アクター辞書を先に探してから単語を探す（[アクタースコープと単語参照の統合](../grammar/actor-dictionary.md#アクタースコープと単語参照の統合)）。見つかった関数には、アクターの表で見つかったときはプロキシを、それ以外の段で見つかったときは ACT を渡す。`var_path` の扱いは `act:word` と同じで、動的参照のときアクターの表は、表自身のフィールド（`name` と pasta.toml の `[actor.名前]` で設定した値など。`create_word` などのメソッドは含まない）だけを探す | 単語、関数の戻り値、または `nil` |
| `act.さくら:expr_fn(key, ...)` | `act:expr_fn` と同じ検索で関数を探して呼ぶ（アクターの表は探さない）。関数の第 1 引数は ACT | 関数の戻り値、または `nil` |
| `act.さくら:expr_fn_var(value, var_path, ...)` | `act:expr_fn_var` と同じ検索で関数を探して呼ぶ（アクターの表は探さない）。関数の第 1 引数は ACT | 関数の戻り値、または `nil` |
| `act.さくら.actor` | アクターオブジェクト | — |
| `act.さくら.act` | 元の ACT | — |

- `word`・`expr_fn`・`expr_fn_var` は、呼んだ関数の先頭の戻り値が ACT またはそのプロキシそのものなら `nil` を返す。それ以外の戻り値は、複数の値も含めてそのまま返す。ACT の `word`・`expr_fn`・`expr_fn_var`・`global_fn` は戻り値を変えない（[関数スコープの展開先](../grammar/variables.md#関数スコープの展開先)）。
- `talk`・`sakura_script` は `nil` を返すため、プロキシの呼び出しは続けて書けない。
- メソッド名（`talk`・`wait`・`yield` など）やフィールド名（`save`・`var`・`actors` など）と同じ名前のアクターは、`act.名前` ではプロキシにならない。
- 生成コードは、アクション行の発言とアクター付きの単語参照・関数呼び出しを、`act.名前` ではなく [actor_proxy](#actor_proxyname) で得たプロキシで書く（`ぱすた：こんにちは` は `act:actor_proxy("ぱすた"):talk("こんにちは")`）。名前を文字列で渡すため、メソッド名・フィールド名と同じ名前のアクターも、アクション行では話せる。

### 検索と呼び出し

以下のメソッドは、名前を [スコープ解決アルゴリズム](../grammar/call-jump.md#スコープ解決アルゴリズム) の 5 段で探す。モード（`mode`）によって、2 段目と 5 段目で探す対象が変わる。

| モード | 2 段目（前方一致） | 5 段目（前方一致） | 使うメソッド |
| ------ | ------------------ | ------------------ | ------------ |
| `"word"` | 実行中のシーンのローカル単語 | グローバル単語 | `word` |
| `"scene"` | 実行中のシーンのローカルシーン | グローバルシーン | `find_scene`・`call`・`call_restore` |
| `"expr"` | 実行中のシーンのローカルシーン | グローバルシーン | `expr_fn`・`expr_fn_var` |

- 1 段目（実行中のシーンのシーンテーブル）・3 段目（act のメソッド。関数の値だけ）・4 段目（`GLOBAL` テーブル）は、どのモードでも同じである。
- `@pasta_search` を読み込めない環境（テストなど）では、2 段目と 5 段目を飛ばす。
- `word`・`expr_fn`・`expr_fn_var`・`call_restore` と、アクタープロキシの `word`・`expr_fn`・`expr_fn_var` は、見つけた関数から戻った後、実行中のシーンを呼ぶ前のシーンに戻す。関数の中で別のシーンが `init_scene` を呼んでも、戻った後の名前の検索は呼び出し元のシーンを基準にする。`call` は戻さない（[call](#callglobal_scene_name-key-attrs-)）。
- 変数の値を名前にする動的参照（`var_path` を渡した `word` と `expr_fn_var`。DSL の [動的単語参照](../grammar/words.md#動的単語参照)）は、3 段目を探さず、1 段目はシーンテーブル自身のキー（`__global_name__`・シーン関数・Lua ブロックで定義した関数）だけを探す。4 段目の `GLOBAL` は探すため、値が `GLOBAL` に登録された名前（ランタイムが登録する `yield`・`チェイントーク` を含む）と同じなら、その関数が見つかって呼ばれる。

#### word(name, var_path)

```lua
act:word(name, var_path?) -> string | nil
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `name` | string（`var_path` があるときは任意の値） | 単語名。`var_path` があるときは変数の値 |
| `var_path` | string または nil | 動的参照のときの変数の場所（`"var.x"`・`"save.x"`・`"args[1]"`）。警告ログに使う |

- `var_path` が `nil` のとき、`name` が `nil` か空文字列なら、何もせずに `nil` を返す。
- `var_path` があるときは、`name` を `WORD.dynamic_key` で単語名に直してから探す（[WORD.dynamic_key](#worddynamic_keyvalue-var_path-via)）。直せない値なら、警告ログを出して探さずに `nil` を返す。探すときは動的参照の探し方（[検索と呼び出し](#検索と呼び出し)）になる。
- `"word"` モードで探し、見つかったのが関数なら `関数(act)` を呼んでその戻り値をそのまま返す。関数以外の値なら `tostring` した文字列を返す。
- 見つからなければ警告ログを出して `nil` を返す。
- DSL の `＄x＝＠単語` は `var.x = act:word("単語")`、`＄x＝＠＄y` は `var.x = act:word(var.y, "var.y")` になる。アクション行の `＠単語`・`＠＄y` はアクタープロキシの `word` を使う（[DSL と Lua の対応表](../grammar/variables.md#dsl-と-lua-の対応表)）。

#### find_handler(mode, key, skip_methods) と find_act_handler(mode, key, skip_methods)

```lua
act:find_handler(mode, key, skip_methods?) -> any | nil
act:find_act_handler(mode, key, skip_methods?) -> any | nil
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `mode` | string | `"word"`・`"scene"`・`"expr"` |
| `key` | string | 検索する名前 |
| `skip_methods` | boolean または nil | 真なら動的参照の探し方（3 段目を探さず、1 段目はシーンテーブル自身のキーだけ）にする。省略時は通常の探し方 |

- 5 段で探し、最初に見つかった値（関数・文字列など）を、呼ばずにそのまま返す。見つからなければ `nil` を返す（警告ログは出さない）。
- ACT の `find_handler` は `find_act_handler` と同じ結果を返す。アクタープロキシの `find_handler` は、`"word"` モードのときアクター辞書を先に探す。
- `"word"` モードの 2・5 段目は、単語辞書から候補の値（文字列）を 1 つ返す。候補のシャッフル＆順次消費はこの呼び出しでも 1 つ進む（[シャッフル＆順次消費](../grammar/words.md#シャッフル順次消費)）。
- `"scene"`・`"expr"` モードの 2・5 段目は、見つかったシーンのシーン関数を返す。

#### expr_fn(key, ...)

```lua
act:expr_fn(key, ...) -> any
```

- `"expr"` モードで探し、見つかったのが関数なら `関数(act, ...)` を呼んでその戻り値を返す。2・5 段目でシーンが見つかった場合も、そのシーン関数を同じく呼ぶ。
- 関数以外の値が見つかったとき、または見つからないときは、警告ログを出して `nil` を返す。
- DSL の式の中の `＠関数（…）` は `act:expr_fn("関数", …)` になる。アクション行の中ではアクタープロキシの `expr_fn` を使う（[関数スコープの展開先](../grammar/variables.md#関数スコープの展開先)）。

#### expr_fn_var(value, var_path, ...)

```lua
act:expr_fn_var(value, var_path, ...) -> any
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `value` | any | 関数名にする変数の値 |
| `var_path` | string | 変数の場所（`"var.f"` など）。警告ログに使う |
| `...` | any | 呼び出す関数に渡す引数 |

- `value` を `WORD.dynamic_key` で関数名に直し（[WORD.dynamic_key](#worddynamic_keyvalue-var_path-via)）、動的参照の探し方（[検索と呼び出し](#検索と呼び出し)）で `"expr"` モードで探す。見つかった後の扱いは `expr_fn` と同じである。
- `value` を関数名に直せないときは、警告ログを出して探さずに `nil` を返す。
- DSL の式の中の `＠＄f（…）` は `act:expr_fn_var(var.f, "var.f", …)` になる。アクション行の中ではアクタープロキシの `expr_fn_var` を使う（[動的関数呼び出し](../grammar/words.md#動的関数呼び出し)）。

#### find_scene(key)

```lua
act:find_scene(key) -> function | nil
```

- `"scene"` モードで探し、見つかった値（通常は関数）を呼ばずに返す。`act:find_handler("scene", key)` と同じである。
- 第 2・第 3 引数（`global_scene_name`・`attrs`）は受け取るが使わない。
- `SCENE.co_exec(act, 名前)` は、この方法で名前を解決する（[REG](shiori-events.md#reg)）。

#### call(global_scene_name, key, attrs, ...)

```lua
act:call(global_scene_name, key, attrs, ...) -> any
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `global_scene_name` | string または nil | 使わない（生成コードは `SCENE.__global_name__` を渡す） |
| `key` | string または nil | 検索する名前。動的ターゲットの生成コードは [call_key](#call_keyvalue-var_path-desc) の戻り値を渡す |
| `attrs` | table または nil | 使わない（生成コードは `{}` を渡す） |
| `...` | any | 呼び出す関数に渡す引数 |

- `"scene"` モードで `key` を探し、見つかったのが関数なら `関数(act, ...)` を呼んでその戻り値を返す。関数は末尾位置で呼ぶ（`return 関数(act, ...)`）ため、`return act:call(…)` の形で呼び出しをつないでも、呼び出しは深くならない。
- 関数以外の値が見つかったとき、または見つからないときは、警告ログ `act:call - handler not found: key='名前', mode='scene', via=act` を出し、失敗表記 `【Call失敗：「名前」が見つからない】` を積んで（[failure](#failuretext-warning)）、`nil` を返す。Lua から直接呼んだ場合も、DSL の Call 行と同じく失敗表記を積む。
- `key` が `nil` のときは、検索せずに警告ログ `act:call - nil key (undefined variable?), skipping scene search` を出して `nil` を返す。失敗表記は積まない。
- `key` が `call_key` の返した「呼ばない」印のときは、検索も警告もせずに `nil` を返す（警告と失敗表記は `call_key` が出している）。
- 呼び出しはコルーチンを新しく作らず、実行中のシーンのコルーチンの中で行う。呼んだ先で `yield` すると、呼び出し元のシーンごと中断する。
- 実行中のシーンは戻さない。呼んだ先のシーン関数が `init_scene` を呼ぶと、戻った後も実行中のシーンは呼んだ先のシーンのままになる。戻った後に呼ぶ前のシーンへ戻す呼び出しは [call_restore](#call_restoreglobal_scene_name-key-attrs-) である。
- DSL の Call 行は、ローカルシーンの最後の行なら `return act:call(SCENE.__global_name__, "名前", {}, 引数…)`、それ以外の行なら `act:call_restore(SCENE.__global_name__, "名前", {}, 引数…)` になる。動的ターゲット（`＞＄名前` など）では、`"名前"` の代わりに `act:call_key(…)` が入る（[Call / Jump](../grammar/call-jump.md)）。

```lua
act:call(nil, "挨拶", nil)           -- 「挨拶」で前方一致するシーンを探して呼ぶ
local fn = act:find_scene("挨拶")    -- 呼ばずに関数だけを得る
```

#### call_restore(global_scene_name, key, attrs, ...)

```lua
act:call_restore(global_scene_name, key, attrs, ...) -> any
```

- 引数は [call](#callglobal_scene_name-key-attrs-) と同じである。`act:call` を呼び、その戻り値をすべてそのまま返す。
- 戻った後、実行中のシーンを呼ぶ前のシーンに戻す。呼んだ先がシーン関数でも Lua の関数でも、見つからなかった場合や `key` が `nil`・「呼ばない」印の場合でも戻す。呼んだ先が `yield` で中断した場合は、再開して戻った時点で戻す。
- 戻すのは実行中のシーンだけである。記録の無い選択 ID の行き先の検索に使う「最後に `init_scene` を呼んだグローバルシーン」（[OnChoiceSelectEx](shiori-events.md#onchoiceselectex)）は戻さない。
- 戻った後に処理が残るため、呼び出しの段が 1 つ残る（`return act:call(…)` は末尾呼び出しで段を残さない）。呼んだ先の中で `return act:call(…)` でつないだ呼び出しは深くならない。
- DSL の Call 行のうち、ローカルシーンの最後の行でないものはこの呼び出しになる。

```lua
act:call_restore(nil, "雑談", nil)   -- 「雑談」を呼び、戻った後も実行中のシーンは呼ぶ前のまま
```

#### call_key(value, var_path, desc)

```lua
act:call_key(value, var_path?, desc?) -> string | 「呼ばない」印
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `value` | any | 動的ターゲットの式の値 |
| `var_path` | string または nil | 式が変数参照 1 つのときの変数の場所（`"var.x"`・`"save.x"`・`"args[1]"`）。警告ログと失敗表記に使う |
| `desc` | string または nil | 式が関数呼び出し 1 つのときの表記（`"@名前()"`・`"@*名前()"`・`"@$var.名前()"`）。警告ログと失敗表記に使う |

- 空でない文字列（文字列 `"nil"` を含む）はそのまま、数値は `tostring(value)` を検索キーとして返す。
- それ以外の値（`nil`・空文字列・真偽値・表など）では、警告ログと失敗表記（[failure](#failuretext-warning)）を出し、「呼ばない」印を返す。印を受け取った `act:call`・`act:call_restore` は、検索せずに `nil` を返す。印は外から作れない値で、文字列 `"nil"` とも `false` とも区別される。
- 警告ログは、`var_path` があれば `WORD.dynamic_key(value, var_path, "act:call")` の警告（[WORD.dynamic_key](#worddynamic_keyvalue-var_path-via)）、無ければ `act:call - key is not a string or number: operand='@名前()', value=nil` の形である。`operand=` は `desc` があるときだけ付き、`value=` の表記は [arith](#arithop-lhs-rhs-lhs_desc-rhs_desc) と同じである。値が `nil` で `var_path` も `desc` も無いとき（内側の `act:arith`・`act:concat` が警告して `nil` を返した場合）は警告しない。
- 失敗表記の文言は [動的ターゲットの値](../grammar/call-jump.md#動的ターゲットの値) の表のとおりである。
- DSL の動的ターゲットは、`＞＄x` が `act:call_key(var.x, "var.x")`、`＞＠f（）` が `act:call_key(act:expr_fn("f"), nil, "@f()")`、それ以外の式が `act:call_key(式)` になる。

#### restore_scene(scene, ...)

```lua
act:restore_scene(scene, ...) -> ...
```

- 実行中のシーンを `scene`（シーンテーブルまたは `nil`）にし、2 番目以降の引数をそのまま返す。
- `call_restore`・`word`・`expr_fn`・`expr_fn_var`・`global_fn` とアクタープロキシの `word`・`expr_fn`・`expr_fn_var` が、関数から戻った後に実行中のシーンを戻すのに使う。
- `init_scene` と違い、最後に `init_scene` を呼んだグローバルシーンの記録は変えない。

#### failure(text, warning)

```lua
act:failure(text, warning?) -> nil
```

- `warning` があれば警告ログに出し、`【text】` を生のさくらスクリプト（`act:raw_script` と同じトークン）として積む。句読点のウェイトも budoux の改行も入らず、`text` はエスケープしない。
- スコープ切替タグを付けずに積んだ位置へ出力される。直前に話したアクターのバルーンに続けて出て、次に同じアクターが話してもスコープ切替タグは増えない。出力の先頭では、その時点のスコープのバルーンに出る（[Call が失敗したとき](../grammar/call-jump.md#call-が失敗したとき)）。
- `act:call`（見つからないとき）と `act:call_key`（検索キーにならない値のとき）が、Call の失敗表記を出すのに使う。

`call_restore`・`call_key`・`restore_scene`・`failure` は act のメソッドのため、ほかの act のメソッドと同じく、`＠名前（…）`・`＠名前`・`＞名前` の検索の 3 段目で名前から見つかる（[検索と呼び出し](#検索と呼び出し)）。

### アクター・グローバル関数・算術・連結

アクション行のアクター、`＠＊名前（…）`、式の算術と連結の生成コードは、次のメソッドを呼ぶ。どれも、アクターや関数が無いとき・値が数値や文字列にできないときに Lua のエラーにせず、警告ログを出して続ける。手書きの Lua からも呼べる。

| メソッド | 内容 | 戻り値 |
| -------- | ---- | ------ |
| `act:actor_proxy(name)` | アクター `name` のアクタープロキシを得る | アクタープロキシ（常に `nil` でない） |
| `act:global_fn(name, ...)` | `GLOBAL` の関数 `name` を呼ぶ | 関数の戻り値、または `nil` |
| `act:arith(op, lhs, rhs, lhs_desc, rhs_desc)` | 数値の二項演算 | 演算結果、または `nil` |
| `act:concat(lhs, rhs, lhs_desc, rhs_desc)` | 文字列の連結 | 連結した文字列、または `nil` |

- 4 つの名前は act のメソッドのため、ほかの act のメソッド（`talk`・`wait` など）と同じ制限を受ける。手書き Lua の `act.actor_proxy`・`act.global_fn`・`act.arith`・`act.concat` は、同じ名前のアクターがいてもプロキシにならない（[アクタープロキシ](#アクタープロキシ)）。また、`＠名前（…）`・`＠名前` の検索の 3 段目で見つかるため、`GLOBAL` に同じ名前の関数があっても `＠名前（…）` では届かない（`＠＊名前（…）` では届く）。1・2 段目（実行中のシーンのシーンテーブル、ローカル単語・ローカルシーン）にある同じ名前は、3 段目より先に見つかる（[検索と呼び出し](#検索と呼び出し)）。

#### actor_proxy(name)

```lua
act:actor_proxy(name) -> ActorProxy
```

- `act.actors` に `name` があれば、`act.名前` と同じアクタープロキシを返す。ログもトークンも出さない。名前を文字列で受け取るため、メソッド名・フィールド名と同じ名前のアクターでもプロキシになる。
- 無ければ、名前だけを持つその場限りのアクターのプロキシを返す。このアクターは登録されず、`act.actors` にも立ち位置にも書かれない（立ち位置が無いため、スポット 0 で話す）。直前に積まれた `talk`・`sakura_script` が同じ名前のその場限りのアクターのものでなければ、目印の台詞 `【未登録アクター：名前】` を積み、警告ログ `act:actor_proxy - unregistered actor: name='名前'` を出す。
- アクション行の生成コードは、アクターをこの形で参照する。DSL での見え方は [登録していないアクター名](../grammar/action-line.md#登録していないアクター名) を参照する。

#### global_fn(name, ...)

```lua
act:global_fn(name, ...) -> any
```

- `GLOBAL[name]` が関数なら `関数(act, ...)` を呼び、その戻り値をすべて返す。関数の中で起きたエラーは、そのまま伝わる。
- 無いとき、関数でない値のときは、警告ログ `act:global_fn - function not found: key='名前'` を出して `nil` を返す。
- 5 段の検索は行わず、`GLOBAL` だけを見る。
- 関数から戻った後、実行中のシーンを呼ぶ前のシーンに戻す（[検索と呼び出し](#検索と呼び出し)）。
- DSL の `＠＊名前（…）` は `act:global_fn("名前", …)` になる。アクション行の中でも、関数の第 1 引数は act である（[関数スコープの展開先](../grammar/variables.md#関数スコープの展開先)）。

#### arith(op, lhs, rhs, lhs_desc, rhs_desc)

```lua
act:arith(op, lhs, rhs, lhs_desc?, rhs_desc?) -> number | nil
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `op` | string | `"+"`・`"-"`・`"*"`・`"/"`・`"%"` |
| `lhs`・`rhs` | any | 左と右の被演算子 |
| `lhs_desc`・`rhs_desc` | string または nil | 警告ログに出す被演算子の説明（`"var.x"`・`"@f()"` など） |

- 被演算子は、数値ならそのまま、文字列なら `tonumber` で数値にする。両方が数値になれば、演算の結果を返す。
- 数値にできない被演算子（`nil`・数字でない文字列・真偽値・表など）があれば `nil` を返し、その被演算子ごとに警告ログ `act:arith - operand is not a number: op='+', operand='var.x', value=nil` を出す。`operand=` は説明があるときだけ付く。`value=` は値の種類を表し、`nil`・`'abc' (string)`・`true (boolean)`、それ以外は `(table)` のように型名だけになる（表の `__tostring` は呼ばない）。ただし、値が `nil` で説明も `nil` の被演算子では警告しない（内側の `act:arith`・`act:concat` が既に警告して `nil` を返した場合）。
- 表の被演算子に `__add` などのメタメソッドがあっても呼ばない。
- `op` が上のどれでもないときは、警告ログを出して `nil` を返す。
- DSL の式の算術は、演算ごとに `act:arith` になる。優先順位と括弧は入れ子で表される（[算術の評価](../grammar/variables.md#算術の評価)・[DSL と Lua の対応表](../grammar/variables.md#dsl-と-lua-の対応表)）。

#### concat(lhs, rhs, lhs_desc, rhs_desc)

```lua
act:concat(lhs, rhs, lhs_desc?, rhs_desc?) -> string | nil
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `lhs`・`rhs` | any | 左と右の被演算子 |
| `lhs_desc`・`rhs_desc` | string または nil | 警告ログに出す被演算子の説明（`"var.x"`・`"@f()"` など） |

- 被演算子は、文字列ならそのまま、数値なら `tostring` で文字列にする（アクション行で表示したときと同じ表記）。両方が文字列になれば、左の直後に右をつないだ文字列を返す。区切りの文字は入れない。
- 文字列と数値以外の被演算子（`nil`・真偽値・表・関数など）があれば `nil` を返し、その被演算子ごとに警告ログ `act:concat - operand is not a string or number: op='&', operand='var.x', value=nil` を出す。`op` は DSL で全角・半角のどちらを書いても `'&'` である。`operand=` は説明があるときだけ付き、`value=` の表記は [arith](#arithop-lhs-rhs-lhs_desc-rhs_desc) と同じである。
- 値が `nil` で説明も `nil` の被演算子では警告しない（内側の `act:concat`・`act:arith` が既に警告して `nil` を返した場合）。外側の演算は黙って `nil` を返す。
- 表の被演算子に `__concat`・`__tostring` などのメタメソッドがあっても呼ばない。
- DSL の式の連結は、演算ごとに `act:concat` になる。連結の連鎖・算術との組み合わせ・括弧は `act:concat` と `act:arith` の入れ子で表される（[連結の評価](../grammar/variables.md#連結の評価)・[DSL と Lua の対応表](../grammar/variables.md#dsl-と-lua-の対応表)）。

```lua
act:concat("合計", 3)                -- "合計3"
act:concat("a", nil, nil, "var.x")   -- nil（警告を 1 行出す）
act:concat(nil, "個")                -- nil（警告しない）
```

### yield

```lua
act:yield() -> act
```

- ここまでに積んだトークンをさくらスクリプトに組み立て、それを応答としてシーンのコルーチンを中断する。積んだトークンは空に戻る。
- 積んだトークンが無いときは何も返さずに中断し、ランタイムはすぐに再開する。
- 続きは、次の OnTalk の機会に再開される。DSL の `＞yield`・`＞チェイントーク` と同じ動作である（[チェイントーク](../grammar/call-jump.md#チェイントーク)）。
- シーン関数が終わると、残っているトークンは自動で組み立てられて出力される。そのため、シーン関数の最後に `yield` は要らない。最後に `yield` を置くと、シーンは中断した状態で残り、次の OnTalk の機会に再開されて何も出力せずに終わる。
- シーンのコルーチンの外（`REG` のハンドラの本体など）で呼ぶと、Lua のエラーになる。

```lua
function SCENE.物語(act)
    local save, var = act:init_scene(SCENE)
    act:talk(act.さくら.actor, "最初のセリフ")
    act:yield()   -- ここまでを出力して中断する
    act:talk(act.さくら.actor, "次のセリフ")
    -- 残りはシーンの終了時に出力される
end
```

### choice と choice_timeout

#### choice(target, display)

```lua
act:choice(target, display?) -> act
```

| パラメータ | 型 | 説明 |
| ---------- | -- | ---- |
| `target` | string | 選択肢の ID（行き先のシーン名） |
| `display` | string または nil | 表示テキスト。`nil` なら `target` を表示する |

- `\![*]\q[表示テキスト,ID,グローバルシーン名]` を出力する選択肢を積む。グローバルシーン名は、`choice` を呼んだ時点の実行中のシーンのグローバルシーンの登録名（`メイン_1` の形）で、選択肢を出したグローバルシーンを表す。表示テキスト・ID・グローバルシーン名の中の `\`・`]`・`,` は `\` でエスケープされる。
- 実行中のシーンが無いとき（シーンの外で呼んだとき）と、ID が `On` または `script:` で始まるときは、3 番目の引数を付けずに `\![*]\q[表示テキスト,ID]` を出力する。
- 選ばれると、OnChoiceSelectEx の既定の処理が、ID と前方一致するローカルシーンを、選択肢を出したグローバルシーンの配下から探して実行する。見つからなければ、ID と前方一致するグローバルシーンを探す（[OnChoiceSelectEx](shiori-events.md#onchoiceselectex)）。
- 表示制御と同じく、1 回の出力の中で `talk`・`sakura_script` より前に積むと、どのアクターにも結び付かず、積んだ位置にそのまま出力される（[表示制御](#表示制御)）。
- DSL の選択肢行 `＠？行き先` も同じ選択肢を出力する（[選択肢行](../grammar/block-structure.md#選択肢行)）。

```lua
act:talk(act.さくら.actor, "どうする？")
act:choice("挨拶", "挨拶する")
act:choice("自己紹介")   -- 表示テキストは「自己紹介」
```

#### choice_timeout(seconds)

```lua
act:choice_timeout(seconds?) -> act
```

- 選択肢のタイムアウトを設定するタグ `\![set,choicetimeout,ミリ秒]` を積む。ミリ秒は `seconds` の 1000 倍を切り捨てた整数で、`seconds` が `nil` なら 0（タイムアウトなし）である。
- `choice` と同じく、1 回の出力の中で `talk`・`sakura_script` より前に積むと、どのアクターにも結び付かず、積んだ位置にそのまま出力される（[表示制御](#表示制御)）。

```lua
act:choice_timeout(30)   -- 30 秒でタイムアウト
act:choice_timeout()     -- タイムアウトなし
```

## WORD

`pasta.word` は、Lua から単語を定義するモジュールである。DSL の単語定義と同じ辞書に登録される。

```lua
local WORD = require("pasta.word")
```

### ファクトリ関数

| 関数 | 登録先 | 探されるとき |
| ---- | ------ | ------------ |
| `WORD.create_global(key)` | グローバル単語 | 単語参照の 5 段目 |
| `WORD.create_local(scene_name, key)` | `scene_name` のローカル単語 | 実行中のシーンが `scene_name` のときの単語参照の 2 段目 |
| `WORD.create_actor(actor_name, key)` | `actor_name` のアクター単語 | そのアクターを付けた単語参照（[アクタースコープと単語参照の統合](../grammar/actor-dictionary.md#アクタースコープと単語参照の統合)） |
| `WORD.create_word(key)` | グローバル単語（`create_global` の別名） | 単語参照の 5 段目 |

- `require("pasta").create_word(key)` も `WORD.create_global(key)` と同じである。
- `scene_name` には、グローバルシーンの登録名（シーン名の照合用の名前の後ろに `_` と、照合用の名前が同じシーンの通し番号を付けた名前。1 つ目の `＊メイン` なら `"メイン_1"`、1 つ目の `＊会話・朝` なら `"会話_朝_1"`）を渡す（[search_scene](modules/pasta-search.md#search_scenename-global_scene_name)）。
- どの関数もビルダーを返す。値はビルダーの `entry` で足す。
- 単語の検索対象は、シーン辞書の読み込みの最後に確定する（[利用できる時期](modules/pasta-search.md#利用できる時期)）。`main.lua` や Lua ブロックのトップレベルで登録した単語は検索できる。シーン関数やイベントハンドラの実行中に登録した単語は、検索の対象にならない。

### ビルダー（entry）

```lua
builder:entry(...) -> builder
```

- 引数の値（文字列）が、それぞれ単語の候補になる。引数が 0 個なら何もしない。
- ビルダー自身を返すため、続けて呼べる。
- 同じキーのビルダーを何度作っても、同じキーに候補が足されていく。DSL で定義した同じキーの単語とも候補が合わさる。
- 候補は前方一致で探され、シャッフル＆順次消費で選ばれる（[前方一致検索](../grammar/words.md#前方一致検索)・[シャッフル＆順次消費](../grammar/words.md#シャッフル順次消費)）。

```lua
WORD.create_global("好きな食べ物")
    :entry("ラーメン", "カレー")
    :entry("寿司")
    :entry("焼肉", "パスタ")
```

### 大量投入の使用例

```lua
local WORD = require("pasta.word")

-- ループによる一括投入
local foods = { "ラーメン", "カレー", "寿司", "焼肉", "パスタ" }
local builder = WORD.create_global("好きな食べ物")
for _, food in ipairs(foods) do
    builder:entry(food)
end

-- ローカル単語の投入（1 つ目の ＊メイン のローカル単語）
WORD.create_local("メイン_1", "返事")
    :entry("はい", "ええ")
    :entry("そうね")

-- アクター単語の投入
WORD.create_actor("さくら", "一人称")
    :entry("わたし")
    :entry("あたし")
```

### WORD.dynamic_key(value, var_path, via)

```lua
WORD.dynamic_key(value, var_path, via) -> string | nil
```

動的参照の変数の値を、単語名・関数名に直す補助関数である。`act:word`（`var_path` を渡したとき）と `act:expr_fn_var` が使う。`act:call_key`（`var_path` を渡したとき）は、値を検索キーにできないときの警告にだけ使う。

| `value` | 戻り値 |
| ------- | ------ |
| 空でない文字列 | その文字列 |
| 数値 | `tostring(value)`（`3` なら `"3"`） |
| `nil`・空文字列・それ以外の型 | `nil`（警告ログを出す） |

- 文字列は DSL として読み直さない。`__tostring` を持つ表も文字列にしない。
- 警告ログは `{via} - undefined variable: '{var_path}'`（`nil`）・`{via} - empty variable: '{var_path}'`（空文字列）・`{via} - unsupported value type: '{var_path}' ({型名})`（それ以外の型）である。ランタイムは `via` に `"act:word"`・`"act:expr_fn"`・`"act:call"`・`"proxy:word"`・`"proxy:expr_fn"` を渡す。

### WORD.resolve_value(value, act)

```lua
WORD.resolve_value(value, act) -> any
```

検索で得た値を単語の値に直す補助関数である。`value` が `nil` なら `nil`、関数なら `value(act)` の戻り値、表なら最初の要素（空の表なら `nil`）、それ以外なら `tostring(value)` を返す。

## GLOBAL

`pasta.global` は、ユーザー定義のグローバル関数を登録するテーブルを返すモジュールである。

```lua
local GLOBAL = require("pasta.global")

GLOBAL.時報 = function(act)
    return os.date("%H") .. "時です"
end

-- DSL からの呼び出し: ＠＊時報（）
-- 変数への代入:       ＄result＝＠＊時報（）
-- 式文（戻り値不要）: ＄＝＠＊時報（）
```

戻り値を使わない呼び出し `＄＝式` は [式文](../grammar/variables.md#式文exprstmt) で扱う。

- DSL の `＠＊名前（引数…）` は `act:global_fn("名前", 引数…)` になり、`GLOBAL` の関数を `(act, 引数…)` で呼ぶ。5 段の検索は行わない。未定義の名前・関数でない値のときは、警告ログを出して `nil` になる（[global_fn](#global_fnname-)）。アクション行で呼んだ場合は戻り値が出力され（`nil` なら何も出力しない）、変数代入の右辺なら戻り値が代入される（[関数スコープの展開先](../grammar/variables.md#関数スコープの展開先)）。
- `GLOBAL` は 5 段の検索の 4 段目（完全一致）でもある。`＠名前（…）`（ローカル呼び出し）・`＠名前`（単語参照）・`＞名前`（Call）は、1〜3 段目に無ければ `GLOBAL` の値を見つける（[検索と呼び出し](#検索と呼び出し)）。関数は `(act, 引数…)` で呼ばれる（アクション行の中の単語参照と関数呼び出しでも第 1 引数は act である。[関数スコープの展開先](../grammar/variables.md#関数スコープの展開先)）。関数以外の値は、単語参照では文字列として出力される。
- DSL の `＠名前（）` は `GLOBAL` の関数を直接は呼ばない。グローバル関数を確実に呼ぶには `＊` を付けた `＠＊名前（）` を使う。
- 関数は、呼ばれる前であればいつ登録してもよい（`main.lua`・Lua ブロックなど）。

ランタイムはあらかじめ次の名前を登録している。

| 名前 | 内容 |
| ---- | ---- |
| `GLOBAL.yield` | `act:yield()` を呼ぶ |
| `GLOBAL["チェイントーク"]` | `GLOBAL.yield` と同じ関数。`＞チェイントーク` が呼ぶ（[チェイントーク](../grammar/call-jump.md#チェイントーク)） |
| `GLOBAL.close_ghost` | `close_ghost(act, ms)`。`ms` が 1 以上の数値なら `act:wait(ms)` を積み、ゴーストを終了させる `\-` を `raw_script` で積む |
| `GLOBAL["ゴースト終了"]` | `GLOBAL.close_ghost` と同じ関数。`＞ゴースト終了` が呼ぶ（[ゴースト終了](../grammar/call-jump.md#ゴースト終了)） |

- `GLOBAL.close_ghost`・`GLOBAL["ゴースト終了"]` は、`main.lua` の後に読み込まれる `pasta.shiori.entry` が登録する。`main.lua` でこれらの名前に代入しても上書きされる（読み込み順は [起動シーケンスとモジュール解決](../reference/startup.md)）。

## SAVE

`pasta.save` は、セッションをまたいで保持される永続化データのテーブルを返すモジュールである。

- セーブキーの命名規約（`pasta_` で始まるキーはエンジンの予約領域）は [@pasta_persistence](modules/pasta-persistence.md#セーブキーの命名規約) を参照する。
- テーブルの読み込みと終了時の自動保存、入れてよい値は [自動で保存される save テーブルとの関係](modules/pasta-persistence.md#自動で保存される-save-テーブルとの関係) を参照する。
- DSL のグローバル変数 `＄＊名前` は `save.名前` になる（[グローバル変数の保存先](../grammar/variables.md#グローバル変数の保存先)）。

### ACT 経由のアクセス

シーン関数では `init_scene` の戻り値の `save` を使う。シーン関数でない関数では `act.save` を使う。

```lua
function SCENE.カウント(act)
    local save, var = act:init_scene(SCENE)
    save.count = (save.count or 0) + 1   -- セッションをまたいで保持される
end
```

### require による直接のアクセス

```lua
local save = require("pasta.save")
save.talk_count = (save.talk_count or 0) + 1
```

`require("pasta.save")` が返す表は、`act.save`・`init_scene` の戻り値の `save` と同じ表である。ACT を受け取らない場所（`main.lua` のトップレベルなど）から永続化データを読み書きするときに使う。

---

道具の一覧は手に入りましたわね。フンッ、あとは使いこなすだけですわよ。
さあ、あなたのゴーストに存分に働いてもらいましょう！
