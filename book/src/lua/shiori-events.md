# SHIORI イベントとハンドラ

> 【クローディア】一秒が過ぎた、ダブルクリックされた、ゴーストが起動した――ベースウェアは、そうした出来事をひとつ残らず「イベント」として知らせてきますの。

> 【アンソニー：刮目】一秒ごとに、でございますか。

> 【高笑い】驚くのはまだ早くてよ！ そのすべての受け止め方を、わたくしが余さずお教えいたしますわ。シーンに任せるか、Lua で自ら捌くか――その作法を、しっかり身につけてくださいまし。

---

ベースウェア（SSP など）は、出来事を SHIORI/3.0 のリクエスト（イベント）として送ってくる。pasta は 1 つのイベントを次の順で処理し、結果を SHIORI/3.0 の応答文字列にして返す。

1. 結果を待っているコールバック（[OnPastaCallBack](#onpastacallbackコールバック応答)）への応答なら、待っているシーンを再開する。
2. 仮想時刻の読み取りの返事（[OnPastaClock](#onpastaclock仮想時刻の読み取り)）なら、時計のずれを覚えて 204 No Content を返す。
3. `REG:register` で重ねたハンドラ（鎖）があれば、新しく登録した順に呼ぶ（[ハンドラを重ねる](#ハンドラを重ねるregregister)）。
4. 鎖が無いか、鎖が全部見送れば、`REG` にあるそのイベントのハンドラを呼ぶ（[REG](#reg)）。
5. それも無ければ、イベント名でシーンを探して実行する（[シーン関数フォールバック](#シーン関数フォールバック)）。

処理中に Lua のエラーが起きて止まると、既定では止まった理由を表記にした 200 応答、表記を出さない設定（`balloon_warning = false`）では 500 応答になる（[エラーハンドリング](#エラーハンドリング)）。

## REG

`REG` は、イベント名をキー、ハンドラ関数を値とする表である。

```lua
local REG = require("pasta.shiori.event.register")
```

ハンドラは `function(act)` の形で登録する。引数 `act` からリクエストの内容を `act.req` で読める（[act.req](#actreq)）。

```lua
local REG = require("pasta.shiori.event.register")

REG.OnMouseDoubleClick = function(act)
    if act.req.reference[3] == "0" then
        return "\\0\\s[0]なあに？\\e"
    end
    return "\\1\\s[10]呼んだ？\\e"
end
```

ハンドラの戻り値は、次のように応答になる。

| 戻り値 | 応答 |
| ------ | ---- |
| 文字列（`SHIORI/` で始まらない） | 200 OK。その文字列が `Value` ヘッダの値になる。空文字列なら 204 No Content |
| `SHIORI/` で始まる文字列 | 応答全体として扱い、包まずにそのまま返す（[RES](#res) で作った応答） |
| シーンのコルーチン（thread） | コルーチンを `act` を渡して再開し、得た出力を `Value` にして 200 OK。出力が無ければ 204 No Content。出力が `SHIORI/` で始まっても `Value` になる |
| `nil`（何も返さない） | 204 No Content |

- `SHIORI/` で始まらない文字列は、そのまま `Value` になる。さくらスクリプトへの変換やウェイトの挿入は行われない。
- ハンドラが返した文字列は、`SHIORI/` で始まれば `Value` ではなく応答全体として扱われる。この規則はハンドラの戻り値だけに適用され、コルーチン（シーン）の出力には適用されない。
- 200 以外の応答は、`RES` の関数で作った応答を返せば、そのステータスのままベースウェアへ届く。`RES.warn(理由)` で理由付きの 204、`RES.not_enough()` で 311、`RES.advice()` で 312、`RES.err(理由)` で 500 を返せる。
- シーンの出力を応答にするには、シーンのコルーチンを返す。`SCENE.co_exec(act, 名前)` は、[シーン関数フォールバック](#シーン関数フォールバック)と同じ探し方でシーンを探し、見つかればそのコルーチンを、見つからなければ `nil` を返す。
- コルーチンの途中で[チェイントーク](../grammar/call-jump.md#チェイントーク)により中断した場合、残りは次の自然なおしゃべり（仮想ディスパッチャが起こす OnTalk）の機会に出力される。外から起こした `OnTalk` では続きは出ない（[OnTalk](#ontalk)）。
- `REG` のキーに置けるハンドラは、1 つのイベントに 1 つである。同じキーに代入し直すと、前のハンドラは置き換わる。1 つのイベントにハンドラを重ねるときは、`REG:register` を使う（[ハンドラを重ねる](#ハンドラを重ねるregregister)）。

```lua
local REG = require("pasta.shiori.event.register")
local SCENE = require("pasta.scene")

REG.OnMouseDoubleClick = function(act)
    -- シーン「なでられ」を探してコルーチンを返す（無ければ nil → 204）
    return SCENE.co_exec(act, "なでられ")
end
```

### ハンドラを重ねる（`REG:register`）

`REG:register(イベント名, 関数)` は、1 つのイベントにハンドラを重ねて登録する。`REG` のキーに代入したハンドラを置き換えず、前のハンドラを変数に取っておく必要も無い。イベント名には、`On` で始まらないリソースの ID（`sakura.recommendsites` など）も書ける。

```lua
local REG = require("pasta.shiori.event.register")

REG:register("OnSecondChange", function(act)
    if act.req.reference[3] == "1" then  -- トークを再生できるときだけ
        -- ここに毎秒の処理を書く
    end
    -- 何も返さなければ、次のハンドラへ回る
end)
```

- `:` を付けて `REG:register(…)` と呼ぶ。`REG.register(…)` と書いたとき、イベント名が文字列でないとき、関数でない値を渡したときは、その場でエラーになり、何も登録されない。
- 1 つのイベントに重ねたハンドラの並び（鎖）は、新しく登録したものから順に呼ばれる。
- 呼ばれたハンドラが偽の値（`nil`・`false`）を返すと、次のハンドラ（1 つ前に登録したもの）へ回る。真の値を返すと、そこで止まり、古いハンドラは呼ばれない。
- 鎖のハンドラが全部見送ると、鎖の底として `REG` のキーのハンドラ（既定ハンドラか、ゴーストが代入したもの）が呼ばれ、それも無ければイベント名でシーンを探して実行する（[シーン関数フォールバック](#シーン関数フォールバック)）。
- 既定ハンドラが置き換わるのは、`REG` のキーに代入したときだけである。`REG:register` で重ねても、既定ハンドラは鎖の底に残る。たとえば OnSecondChange に何を重ねても、全部が見送った秒には既定の OnSecondChange が動く。
- ハンドラは、呼ばれるごとに新しい `act` を受け取る。見送ったハンドラが `act` に積んだ台詞は、後のハンドラの出力に混ざらない。
- 見張り役は無い。新しいハンドラが応答した回は、古いハンドラは呼ばれない。毎回必ず動かしたい処理も、新しいハンドラが止めた回は動かない。

鎖のハンドラの戻り値は、次のように読む。

| 戻り値 | 鎖の動き | 応答 |
| ------ | -------- | ---- |
| `nil`（何も返さない）・`false` | 見送って次のハンドラへ回る | 後のハンドラか鎖の底の応答 |
| `true` | ここで止める | 204 No Content |
| 文字列 | ここで止める | [REG の戻り値の表](#reg)のとおり。空文字列 `""` も止まり、204 No Content |
| シーンのコルーチン（thread） | ここで止める | REG の戻り値の表のとおり |
| ほかの真の値（数・表など） | ここで止める | 204 No Content |

`REG:register` は、その登録を外す関数を返す。外した次のイベントから、そのハンドラは呼ばれない。ほかのハンドラの順は変わらず、外す関数を 2 回呼んでも何も起きない。

```lua
local REG = require("pasta.shiori.event.register")

-- 初回の演出の間、毎秒の処理を全部止める門
local off = REG:register("OnSecondChange", function(act)
    return true  -- ここで止めて 204
end)

-- 演出が終わったら門を外す
off()
```

門を置く `true` のハンドラは、OnSecondChange の既定ハンドラ（ランダムトーク・時報・コールバックのタイムアウト処理など）も、ゴーストが重ねたほかのハンドラ（表情を戻す・窓あわせなどの毎秒の処理）も止める。ランダムトークと時報を止めるだけなら、門ではなく [`＞自動トーク停止`](../grammar/call-jump.md#自動トーク停止自動トーク再開) を使う。毎秒の処理まで止めるときに、鎖の門を使う。

### 既定ハンドラと上書き

次の 9 つのイベントには、pasta が既定のハンドラを `REG` に登録している。ゴーストが `REG` の同じキーにハンドラを代入すると、そのイベントでは既定ハンドラの代わりにゴーストのハンドラが呼ばれる。

| イベント | 既定ハンドラの動作 |
| -------- | ------------------ |
| `OnBoot` | イベント名でシーンを探して実行する（シーン関数フォールバックと同じ動作） |
| `OnChoiceSelectEx` | 選ばれた選択肢のシーンを実行する（[OnChoiceSelectEx](#onchoiceselectex)） |
| `OnMouseDoubleClick` | つつきのショートカットのシーンを探して実行する。無ければイベント名でシーンを探して実行する（[OnMouseDoubleClick](#onmousedoubleclick)） |
| `OnMouseMove` | 撫でを判定し、撫でたと決めたときは撫でのショートカットのシーンを探して実行する。それ以外はイベント名でシーンを探して実行する（[OnMouseMove](#onmousemove)） |
| `OnSecondChange` | コールバックのタイムアウト処理と、仮想ディスパッチャによる OnTalk・OnHour の発行を行う（[OnSecondChange](#onsecondchange)） |
| `OnPastaScene` | デバッグが有効なとき、名前で指名したシーンを再生する（[OnPastaScene](#onpastascene名前でシーンを再生)） |
| `OnPastaSceneAt` | デバッグが有効なとき、ファイルと行で指名したシーンを再生する（[OnPastaSceneAt](#onpastasceneat位置でシーンを再生)） |
| `OnPastaCallFunction` | Reference0 の名前を、辞書の `＞名前` と同じ探し方で呼ぶ（[OnPastaCallFunction](#onpastacallfunction名前を外から呼ぶ)） |
| `OnPastaSetSaveVariable` | Reference1 を JSON の値として読み、グローバル変数に代入する（[OnPastaSetSaveVariable](#onpastasetsavevariableグローバル変数に代入)） |

既定ハンドラは、ゴーストが登録したハンドラを上書きしない。既定ハンドラは、エンジンの起動処理（SHIORI 応答関数の読み込み）の中で、`REG` のそのキーがまだ空のときだけ登録される。そのため、ゴーストのハンドラは `scripts/main.lua`（既定ハンドラの登録より前に実行される）に書いても、辞書ファイル（`.pasta`）の Lua ブロック（登録より後に評価される）に書いてもよい（起動の順序は [起動シーケンス](../reference/startup.md#2-起動シーケンス)）。

既定ハンドラを変数に取っておき、自分のハンドラから呼ぶ（既定ハンドラを包む）ときだけ、置き場所に気を付ける。`main.lua` では既定ハンドラがまだ登録されていないので、先に `require("pasta.shiori.event")` を呼んで既定ハンドラを登録させてから `local default = REG.イベント名` で取る。辞書の Lua ブロックでは、そのまま取れる。

```lua
-- scripts/main.lua
require("pasta.shiori.event")  -- 既定ハンドラを先に登録させる
local REG = require("pasta.shiori.event.register")
local default = REG.OnBoot

REG.OnBoot = function(act)
    -- ここに起動時の処理を書く
    return default(act)
end
```

いくつもの場所で同じイベントのハンドラを包み合うと、どれが外側になるかは読み込みの順で決まり、前のハンドラを呼び忘れると既定ハンドラが黙って止まる。包むのをやめて `REG:register` で重ねれば、前のハンドラを取っておく必要が無く、既定ハンドラはいつも鎖の底に残る（[ハンドラを重ねる](#ハンドラを重ねるregregister)）。

毎秒の処理を足す例。包む書き方では、前のハンドラを取っておき、自分の処理の後で呼ぶ。

```lua
local REG = require("pasta.shiori.event.register")
local prev = REG.OnSecondChange

REG.OnSecondChange = function(act)
    -- ここに毎秒の処理を書く
    return prev(act)
end
```

重ねる書き方では、前のハンドラに触れない。何も返さなければ、次のハンドラへ回る。

```lua
local REG = require("pasta.shiori.event.register")

REG:register("OnSecondChange", function(act)
    -- ここに毎秒の処理を書く
end)
```

演出の間だけ毎秒のイベントを全部止める門の例。包む書き方では、門は一番外側に包み、条件を満たさない回は前のハンドラを呼ぶ。

```lua
local REG = require("pasta.shiori.event.register")
local prev = REG.OnSecondChange

REG.OnSecondChange = function(act)
    if SAVE["初回待ち"] then
        return nil  -- 何もしない（204）
    end
    return prev(act)
end
```

重ねる書き方では、門は `true` を返して止め、演出が終わったら外す。後から登録した門は、先に登録したハンドラより先に呼ばれる。

```lua
local REG = require("pasta.shiori.event.register")

local off = REG:register("OnSecondChange", function(act)
    if SAVE["初回待ち"] then
        return true  -- ここで止めて 204
    end
end)

-- 演出が終わったら
off()
```

### act.req

`act.req` は、SHIORI リクエストの内容を持つ表である。

| フィールド | 型 | 内容 |
| ---------- | -- | ---- |
| `act.req.id` | string | イベント名（`"OnBoot"` など） |
| `act.req.method` | string | `"get"` または `"notify"` |
| `act.req.version` | number | `30`（SHIORI/3.0） |
| `act.req.charset` | string または nil | `Charset` ヘッダの値 |
| `act.req.sender` | string または nil | `Sender` ヘッダの値（`"SSP"` など） |
| `act.req.base_id` | string または nil | `BaseID` ヘッダの値 |
| `act.req.status` | string または nil | `Status` ヘッダの値（`"talking,balloon(0=0)"` のようなカンマ区切りの文字列） |
| `act.req.security_level` | string または nil | `SecurityLevel` ヘッダの値 |
| `act.req.reference[N]` | string または nil | `ReferenceN` ヘッダの値。`N` は 0 から数える。送られていない Reference は `nil` |
| `act.req.dic` | table | すべてのヘッダ（ヘッダ名 → 値。`Reference0` なども含む） |
| `act.req.date` | table | リクエストを受けた時点のローカル日時（下表） |

```lua
local ref0 = act.req.reference[0]  -- Reference0 の値
local ref1 = act.req.reference[1]  -- Reference1 の値
if act.req.reference[5] == nil then
    -- Reference5 は送られていない
end
```

`act.req.date` のフィールドは次のとおり（値はすべて数値）。

| フィールド | 内容 |
| ---------- | ---- |
| `unix` | Unix 時刻（秒） |
| `year` / `month` / `day` | 年 / 月（1〜12） / 日 |
| `hour` / `min` / `sec` | 時（0〜23） / 分 / 秒 |
| `ns` | ナノ秒 |
| `yday` / `ordinal` | 年内の通し日（1 月 1 日が 1） |
| `wday` / `num_days_from_sunday` | 曜日（日曜が 0） |

- リクエストに `X-Pasta-Time` ヘッダ（RFC 3339 形式の日時）があると、`act.req.date` は現在時刻ではなくその日時で作られる。形式が正しくないときは、リクエストの処理がエラーになる。
- `act.req.date` の値は、SSP のプロセスの中の OS の時計（`X-Pasta-Time` があればその日時）である。SSP によっては（2.9.18 の「現在時刻の仮想的変更」など）、この OS の時計そのものが仮想時刻になるため、`act.req.date` も仮想時刻に従う。そうでない SSP では、`act.req.date` は仮想時刻に従わない。
- 時報の判定（[OnHour](#onhour)）と[日時変数](../grammar/variables.md#日時変数)は、どちらの場合も仮想時刻に従う（[OnPastaClock](#onpastaclock仮想時刻の読み取り)）。おしゃべりの間隔・撫での判定・コールバックの待ち時間は、時刻の設定し直しによる飛びを差し引いて数える（[時刻の飛び](#時刻の飛び)）。
- `act.req` は読み取り専用として扱う。書き換えたときの動作は定めない。
- DSL のシーンから Reference や日時を使うには、[エンジンが値を入れる変数](../grammar/variables.md#エンジンが値を入れる変数)（[リクエスト変数](../grammar/variables.md#リクエスト変数reference)・[日時変数](../grammar/variables.md#日時変数)）を使う。

## RES

`RES` は、SHIORI/3.0 の応答文字列を作るモジュールである。エンジンは、ハンドラの戻り値やエラーをこのモジュールで応答文字列にする。

```lua
local RES = require("pasta.shiori.res")
```

| 関数 | ステータス | 説明 |
| ---- | ---------- | ---- |
| `RES.ok(value, dic)` | 200 OK | `Value` ヘッダに `value` を入れる。`value` が `nil` か空文字列なら、`RES.no_content(dic)` と同じ 204 No Content |
| `RES.no_content(dic)` | 204 No Content | 返す内容が無い正常終了 |
| `RES.not_enough(dic)` | 311 Not Enough | TEACH の情報が足りない |
| `RES.advice(dic)` | 312 Advice | TEACH を解釈できない |
| `RES.bad_request(dic)` | 400 Bad Request | リクエストの誤り |
| `RES.err(reason, dic)` | 500 Internal Server Error | `X-Error-Reason` ヘッダに `reason` を入れる。`reason` が `nil` か空文字列なら `"Unknown error"` |
| `RES.warn(reason, dic)` | 204 No Content | `X-Warn-Reason` ヘッダに `reason` を入れる |
| `RES.build(code, dic)` | `code` | 任意のステータス（`"200 OK"` など）で応答を作る。上の関数はすべてこれを使う |

- `dic` は追加するヘッダの表（ヘッダ名 → 値）で、省略できる。`RES.ok`・`RES.err`・`RES.warn` は、渡された `dic` の表に `Value` などを書き込んでから使う。
- 応答は、1 行目が `SHIORI/3.0 ステータス`、続いて `Charset`・`Sender`・`SecurityLevel` の 3 ヘッダ、`dic` のヘッダ、最後に空行という形になる。行末は CR LF。`dic` のヘッダの並び順は定まらない。
- 3 つの標準ヘッダの値は `RES.env` の表から取る。既定値は `charset = "UTF-8"`・`sender = "Pasta"`・`security_level = "local"` で、書き換えるとそれ以降のすべての応答に反映される。

```lua
local RES = require("pasta.shiori.res")

local a = RES.ok("\\0\\s[0]こんにちは\\e")
local b = RES.ok("\\0\\s[0]こんにちは\\e", { Reference0 = "追加情報" })
local c = RES.no_content()
local d = RES.err("設定ファイルが見つかりません")
```

`b` の内容は次のとおり（ヘッダの並び順は変わることがある）。

```text
SHIORI/3.0 200 OK
Charset: UTF-8
Sender: Pasta
SecurityLevel: local
Value: \0\s[0]こんにちは\e
Reference0: 追加情報

```

`REG` のハンドラは、`Value` にする文字列を返すほか、この節の関数で作った応答全体を返してもよい。`SHIORI/` で始まる戻り値は、包まれずにそのまま応答になる（[REG](#reg)）。

```lua
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

REG.OnMouseDoubleClick = function(act)
    if act.req.reference[3] ~= "0" then
        return RES.warn("相方へのダブルクリックは扱わない")  -- 204 No Content（X-Warn-Reason 付き）
    end
    return "\\0\\s[0]なあに？\\e"  -- 200 OK（Value に入る）
end
```

## 主要イベント

ベースウェアから送られる主なイベントと、pasta での扱いを示す。各 Reference の意味はベースウェアの仕様であり、すべてのイベントと Reference は [UKADOC の SHIORI Event 一覧](https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html)で確認できる。各イベントの Reference 表の「（SSP）」は SSP など一部のベースウェアだけが送る Reference を表す。

| 分類 | イベント | pasta の既定ハンドラ |
| ---- | -------- | -------------------- |
| 起動・終了 | `OnFirstBoot` | なし（同名のシーンを実行） |
| 起動・終了 | `OnBoot` | あり（同名のシーンを実行） |
| 起動・終了 | `OnClose` | なし（同名のシーンを実行） |
| 起動・終了 | `OnGhostChanged` | なし（同名のシーンを実行） |
| 選択肢 | `OnChoiceSelectEx` | あり（選択肢のルーティング） |
| マウス | `OnMouseDoubleClick` | あり（つつきのショートカット・同名のシーンを実行） |
| マウス | `OnMouseMove` | あり（撫での判定とショートカット・同名のシーンを実行） |
| 時間 | `OnSecondChange` | あり（コールバックのタイムアウト処理・仮想ディスパッチャ） |
| 時間 | `OnMinuteChange` | なし（同名のシーンを実行） |
| コールバック | `OnPastaCallBack{N}` | `REG` より先に処理される |
| 時計 | `OnPastaClock` | `REG` より先に処理される（仮想時刻の読み取り） |
| デバッグ | `OnPastaScene` | あり（名前で指名したシーンを再生。デバッグが有効なときだけ） |
| デバッグ | `OnPastaSceneAt` | あり（位置で指名したシーンを再生。デバッグが有効なときだけ） |
| 外からの操作 | `OnPastaCallFunction` | あり（`＞名前` と同じ呼び出し。いつでも） |
| 外からの操作 | `OnPastaSetSaveVariable` | あり（グローバル変数への代入。いつでも） |

ここに無いイベントも、`REG` にハンドラを登録するか、同名のシーンを書けば応答できる。

### OnFirstBoot

ゴーストが初めて起動されたときに送られる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | vanish された回数 |

```pasta
＊OnFirstBoot
    ぱすた：はじめまして。
```

```lua
REG.OnFirstBoot = function(act)
    local vanished = tonumber(act.req.reference[0]) or 0
    if vanished > 0 then
        return "\\0\\s[0]また会えたね。\\e"
    end
    return "\\0\\s[0]はじめまして。\\e"
end
```

### OnBoot

ゴーストが起動したときに送られる。既定ハンドラは、`OnBoot` という名前でシーンを探して実行する（シーン関数フォールバックと同じ動作）。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 起動時のシェル名 |
| `act.req.reference[6]` | 前回の処理中に落ちたときに `halt`（SSP） |
| `act.req.reference[7]` | 前回の処理中に落ちたゴーストの名前（SSP） |

```pasta
＊OnBoot
    ぱすた：起動しました。
```

```lua
REG.OnBoot = function(act)
    local shell_name = act.req.reference[0] or "不明"
    return "\\0\\s[0]起動しました。シェル: " .. shell_name .. "\\e"
end
```

応えるシーン（`＊OnBoot`）が無いと、既定ハンドラは何も返さない（204 No Content）。このとき SSP は立ち絵を出さず、立ち絵は最初の台詞と一緒に出る。pasta は、書いていない立ち絵を推測で出さない。

`REG.OnBoot` を上書きすると、`＊OnBoot` シーンは自動では実行されなくなる。上書きする代入の置き場所は [既定ハンドラと上書き](#既定ハンドラと上書き) のとおり。

### OnClose

ゴーストの終了が指示されたときに送られる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 終了の理由。ユーザーが終了したときは `user`、シャットダウンのときは `system`（SSP） |
| `act.req.reference[1]` | 終了の操作をしたメニューが属するキャラクターのスコープ番号（SSP） |
| `act.req.reference[2]` | 終了の操作をしたウィンドウのキャラクターのスコープ番号（SSP） |

終了のさくらスクリプト `\-` は、DSL では [`＞ゴースト終了`](../grammar/call-jump.md#ゴースト終了) で出力に加えられる。

```pasta
＊OnClose
    ぱすた：またね。
    ＞ゴースト終了
```

```lua
REG.OnClose = function(act)
    if act.req.reference[0] == "user" then
        return "\\0\\s[0]またね。\\-"
    end
    return "\\0\\s[0]終了します。\\-"
end
```

### OnGhostChanged

ほかのゴーストからこのゴーストに切り替えられたときに送られる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 直前のゴーストの本体側の名前 |
| `act.req.reference[1]` | 直前のゴーストの、切り替え時のスクリプト |
| `act.req.reference[2]` | 直前のゴーストの名前（SSP） |
| `act.req.reference[3]` | 直前のゴーストのパス（SSP） |
| `act.req.reference[7]` | 切り替わったゴースト（このゴースト）のシェル名（SSP） |

```lua
REG.OnGhostChanged = function(act)
    local previous = act.req.reference[0] or "前のゴースト"
    return "\\0\\s[0]" .. previous .. "から交代したよ。\\e"
end
```

### OnChoiceSelectEx

ユーザーが選択肢（さくらスクリプトの `\q[表示テキスト,ID]`）を選んだときに送られる。DSL の選択肢行 `＠？ジャンプ先` は `\![*]\q[表示テキスト,ジャンプ先,グローバルシーン名]` を出力し、ジャンプ先の名前が選択肢の ID に、選択肢行を出したグローバルシーンの登録名が 3 番目の引数になる（構文は [選択肢行](../grammar/block-structure.md#選択肢行)、`act:choice` も同じ。[choice](script-api.md#choicetarget-display)）。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 選択肢のテキスト（ラベル） |
| `act.req.reference[1]` | 選択肢の ID |
| `act.req.reference[2]` 以降 | 拡張情報（`\q` の 3 番目以降の引数）。選択肢行・`act:choice` が出した選択肢では、`reference[2]` が選択肢を出したグローバルシーンの登録名 |

既定ハンドラ（自動ルーティング）は次の順で動く。通常はハンドラの登録は不要である。

1. `＊OnChoiceSelectEx` という名前のシーンがあれば、それを実行する（自動ルーティングより優先する）。シーンの探し方は[シーン関数フォールバック](#シーン関数フォールバック)と同じ。
2. 無ければ、選択 ID と前方一致するローカルシーンを、選択肢を出したグローバルシーンの配下から探し、見つかったシーンを実行する。ローカルシーンが見つからなければ、選択 ID と前方一致するグローバルシーンを探す。候補が複数あれば、シャッフル＆順次消費で 1 つ選ばれる。
   - 選択肢を出したグローバルシーンは、`reference[2]` がグローバルシーンの登録名であればそのシーンである。Call で呼んだ別のグローバルシーンが出した選択肢はそのシーンの、Call から戻った後に出した選択肢は呼び出し元のシーンの配下から探す（[Call から戻った後の名前解決](../grammar/call-jump.md#call-から戻った後の名前解決)）。1 つの応答に別々のシーンが出した選択肢があっても、選択肢ごとに探す範囲が決まる。
   - `reference[2]` が無いか、グローバルシーンの登録名でないとき（手書きのさくらスクリプトで出した選択肢、`reference[2]` を送らないベースウェアなど）は、最後に `init_scene` を呼んだグローバルシーンの配下から探す。このグローバルシーンは、Call から戻っても呼び出し元には戻らない。
3. 見つからなければ 204 No Content を返す。

```pasta
＊メニュー
    ぱすた：どれにする？
    ＠？挨拶
    ＠？天気

    ・挨拶
        ぱすた：こんにちは。

    ・天気
        ぱすた：今日は晴れだよ。
```

自分で選択肢を処理するときは `REG.OnChoiceSelectEx` を上書きする。上書きすると、既定の自動ルーティングは行われない。

```lua
REG.OnChoiceSelectEx = function(act)
    local choice_id = act.req.reference[1]  -- 選択肢の ID
    if choice_id == "やめる" then
        return "\\0\\s[0]わかった。\\e"
    end
    return nil
end
```

### OnMouseDoubleClick

キャラクターが左または右ボタンでダブルクリックされたときに送られる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | マウスカーソルの x 座標（ローカル座標） |
| `act.req.reference[1]` | マウスカーソルの y 座標（ローカル座標） |
| `act.req.reference[2]` | 常に `0` |
| `act.req.reference[3]` | 本体なら `0`、相方なら `1`（SSP では `2` 以降もある） |
| `act.req.reference[4]` | 当たり判定の識別子 |
| `act.req.reference[5]` | 左ボタンなら `0`、右ボタンなら `1` |
| `act.req.reference[6]` | 入力の種類（`touch`・`pen` など） |

既定ハンドラは、つつきのショートカットのシーン（`＊つっつき反応…`・`＊OnPoke…`）を探して実行する。どれも無ければ、`OnMouseDoubleClick` という名前でシーンを探して実行する（[シーン関数フォールバック](#シーン関数フォールバック)と同じ探し方）。探す順と決まりは [つつき・撫でのショートカット](#つつき撫でのショートカット) のとおり。左ボタンと右ボタン、マウス・タッチ・ペンは区別しない。

```pasta
＊つっつき反応Head
    ぱすた：頭をつつかないで。

＊OnMouseDoubleClick
    ぱすた：なあに？
```

`REG.OnMouseDoubleClick` を登録すると、そのハンドラだけが呼ばれ、ショートカットのシーンも `＊OnMouseDoubleClick` も自動では探されなくなる（[自前のハンドラの優先](#自前のハンドラの優先)）。

```lua
REG.OnMouseDoubleClick = function(act)
    local scope = act.req.reference[3]
    local hit_area = act.req.reference[4]
    if scope == "0" and hit_area == "Head" then
        return "\\0\\s[0]頭をつつかないで。\\e"
    end
    return nil
end
```

### OnMouseMove

マウスカーソルがキャラクターの上で動いたときに送られる。動いている間は短い間隔で続けて届く。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | マウスカーソルの x 座標（ローカル座標） |
| `act.req.reference[1]` | マウスカーソルの y 座標（ローカル座標） |
| `act.req.reference[3]` | 本体なら `0`、相方なら `1`（SSP では `2` 以降もある） |
| `act.req.reference[4]` | 当たり判定の識別子 |

既定ハンドラは、届くたびに撫でを判定する（[撫での判定](#撫での判定)）。撫でたと決めたときは、撫でのショートカットのシーン（`＊さわり反応…`・`＊OnTouch…`）を探して実行する。撫でたと決めなかったとき、およびショートカットのシーンが 1 つも見つからなかったときは、`OnMouseMove` という名前でシーンを探して実行する（[シーン関数フォールバック](#シーン関数フォールバック)と同じ探し方）。撫でたと決めなかった回は、`OnMouseMove` のほかの名前を探さない。

```pasta
＊さわり反応Head
    ぱすた：えへへ、なでなでされちゃった。
```

`REG.OnMouseMove` を登録すると、そのハンドラだけが呼ばれ、撫での判定もショートカットのシーンの検索も行われない（[自前のハンドラの優先](#自前のハンドラの優先)）。

### OnSecondChange

1 秒ごとに送られる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | OS の連続起動時間（時間単位） |
| `act.req.reference[1]` | 見切れているとき `1`、それ以外は `0` |
| `act.req.reference[2]` | 重なっているとき `1`、それ以外は `0` |
| `act.req.reference[3]` | トークを再生できるとき `1`、それ以外は `0` |
| `act.req.reference[4]` | OS で何も操作されずに放置されている時間（秒、SSP） |

既定ハンドラは、次の 3 つを順に行う。前の手順が応答を生んだ回は、その応答を返し、後の手順は行わない。

1. 結果を待っているコールバックのうち、待ち時間の上限を過ぎたものをタイムアウトとして処理する（[OnPastaCallBack](#onpastacallbackコールバック応答)）。この処理が応答を生んだ回は、その回の OnHour・OnTalk は発行されない。
2. [仮想ディスパッチャ](#仮想ディスパッチャ)を呼び、OnHour（時報）・OnTalk（ランダムトーク）を発行する。
3. 何も発行しなかった回で、Status に[ブロック対象のキーワード](#ブロック対象-status-キーワード)が無く、仮想時刻の読み取りを予約していれば、読み取りのさくらスクリプトを返す（[OnPastaClock](#onpastaclock仮想時刻の読み取り)）。

`REG.OnSecondChange` を上書きすると、OnTalk・OnHour の発行、コールバックのタイムアウト処理、デバッグのシーン再生、仮想時刻の読み取りがすべて止まる。毎秒の処理を足すときは、`REG:register` で重ねる（[ハンドラを重ねる](#ハンドラを重ねるregregister)）か、既定ハンドラを変数に取っておき、上書きしたハンドラから呼び出す（代入の置き場所は [既定ハンドラと上書き](#既定ハンドラと上書き) のとおり）。

```lua
local REG = require("pasta.shiori.event.register")
local default_on_second_change = REG.OnSecondChange

REG.OnSecondChange = function(act)
    -- ここに毎秒の処理を書く
    return default_on_second_change(act)
end
```

### OnPastaCallBack（コールバック応答）

シーンの中でベースウェアのプロパティを読む（`act:get_property`）と、pasta は `\![get,property,OnPastaCallBack{N},…]` タグだけを応答として返し、シーンのコルーチンを中断して結果を待つ。`{N}` は 1 から増える番号で、呼び出しのたびに別のイベント名になる。ベースウェアは結果を、そのイベント名のイベント（値は Reference）として送り返す。

- このイベントは `REG` より先に処理される。イベント名が待っているものと一致すると、待っているコルーチンを Reference の値で再開し、その出力を `Value` にして 200 OK（出力の無いままシーンが終われば 204 No Content）を返す。このとき `REG` のハンドラは呼ばれない。ハンドラを登録する必要はない。
- 待っているものと一致しない `OnPastaCallBack{N}` は、通常のイベントとして `REG`・シーン関数フォールバックへ進む。

再開したシーンは、通常のイベントで実行したシーンと同じ規則で扱われる。

- 出力の無いまま中断したときは、応答を返さずに同じイベントの中で再開を続け、出力が得られるかシーンが終わるまで進める。
- [チェイントーク](../grammar/call-jump.md#チェイントーク)で中断したときは、残りが次の自然なおしゃべり（仮想ディスパッチャが起こす OnTalk）の機会に出力される。
- もう一度 `act:get_property` を呼んで中断したときは、新しいイベント名の待機になる。

待ち時間の上限（`act:get_property` の既定は 5 秒）を過ぎた待機は、OnSecondChange の既定ハンドラがタイムアウトとして処理する（[get_property](script-api.md#get_propertyname_or_names-timeout-timeout_message)）。

- 待っていたシーンは、`timeout_message`（既定は `"callback timeout: get_property"`）を理由とするエラーで終わり、警告ログにイベント名と理由が出る。
- その回の OnSecondChange の応答は、`X-Error-Reason` に `timeout_message` の文字列そのものを持つ。既定では、途中の台詞の後に `【時間切れ：理由】` の表記を `Value` に持つ 200 OK になる。表記を出さない設定（`balloon_warning = false`）では、`Value` の無い 500 Internal Server Error になる（[時間切れ](../reference/runtime-failures.md#時間切れ)）。
- シーンがそのエラーを `pcall` で捕まえて続けたときは、再開したシーンと同じ規則（前の箇条書き）で扱われ、出力があればその回の OnSecondChange の応答（200 OK）になる。
- 1 回の OnSecondChange で複数の待機がタイムアウトしたときは、`{N}` の小さい順に処理し、最初に応答を生んだ待機の応答を使う。残りの待機の応答は、警告ログを出して捨てられる。
- タイムアウトした後に届いた結果は、待っていない `OnPastaCallBack{N}` として扱われる。

理由付きの時間切れの応答は、表記を出すかどうかで次のように変わる（ヘッダは抜粋）。

```text
表記を出す設定（既定）:
  SHIORI/3.0 200 OK
  Value: （途中の台詞）【時間切れ：callback timeout: get_property】\e
  X-Error-Reason: callback timeout: get_property

表記を出さない設定（balloon_warning = false）:
  SHIORI/3.0 500 Internal Server Error
  X-Error-Reason: callback timeout: get_property
```

### OnPastaClock（仮想時刻の読み取り）

SSP の仮想時刻（開発用パレットなどで進めた時刻）は、SHIORI のリクエストのヘッダには載らない。SSP によっては、SSP のプロセスの中の OS の時計そのものを仮想時刻にするため、`act.req.date` がそのまま仮想時刻になる（[act.req](#actreq)）。そうしない SSP のために、pasta は、SSP に仮想の日時を読ませて、OS の時計とのずれ（秒）を覚える。

1. `OnMinuteChange`・`OnHourTimeSignal` は、毎分・毎時の決まった時機に届く。pasta は、これらが OS の時計で見込んだ時機から外れて届くと、仮想時刻が動いたとみなして読み取りを予約する。
2. 予約の後、何も出さない最初の OnSecondChange の応答で、`\![get,property,OnPastaClock,system.year,system.month,system.day,system.hour,system.minute,system.second]\e` を返す（[OnSecondChange](#onsecondchange)）。
3. SSP は、`OnPastaClock` のイベントで仮想の日時を送り返す。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 年 |
| `act.req.reference[1]` | 月 |
| `act.req.reference[2]` | 日 |
| `act.req.reference[3]` | 時 |
| `act.req.reference[4]` | 分 |
| `act.req.reference[5]` | 秒 |

- このイベントは `REG` より先に処理される。pasta は、返事の日時と OS の時計の差をずれとして覚え、204 No Content を返す。このとき `REG` のハンドラは呼ばれず、シーンも探さない。ハンドラを登録する必要はない。
- ずれが 2 秒以内なら、0（仮想時刻なし）とみなす。SSP が OS の時計そのものを仮想時刻にするときは、返事の日時と `act.req.date` が同じなので、ずれはいつも 0 になる。読み取りは、時刻を設定し直した直後に 1 回走るだけである。
- 値が読めないときは、ずれを 0 に戻し、警告ログを出す。OS の時計で動き続ける。
- `SecurityLevel` が `external` の `OnPastaClock` は、返事として扱わない。通常のイベントとして `REG`・シーン関数フォールバックへ進む。
- `X-Pasta-Time` ヘッダのあるリクエストでは、ずれを使わない。その日時をそのまま使う。
- ずれは保存しない。SHIORI の load から unload までの間だけ保たれる。

覚えたずれは、時報の判定（[OnHour](#onhour)）と[日時変数](../grammar/variables.md#日時変数)に使われる。`act.req.date` には足されない（[act.req](#actreq)）。SSP が OS の時計を書き換えるときは、ずれは 0 のままで、設定し直しは[時刻の飛び](#時刻の飛び)として扱われる。

### OnMinuteChange

1 分ごとに送られる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | OS の連続起動時間（時間単位） |
| `act.req.reference[1]` | 見切れているとき `1`、それ以外は `0` |
| `act.req.reference[2]` | 重なっているとき `1`、それ以外は `0` |
| `act.req.reference[3]` | トークを再生できるとき `1`、それ以外は `0` |
| `act.req.reference[4]` | OS で何も操作されずに放置されている時間（秒、SSP） |

```lua
REG.OnMinuteChange = function(act)
    if act.req.date.min == 30 then
        return "\\0\\s[0]30 分になったよ。\\e"
    end
    return nil
end
```

正時の時報は、OnMinuteChange ではなく [OnHour](#onhour) で書ける。

### OnPastaScene（名前でシーンを再生）

名前で指名したシーンを、その場で再生する。pasta の組み込みのイベントで、ベースウェアが自分から送ることはない。AI エージェントなどが、SSP の MCP サーバーの `raise_event` で送る（[MCP からの呼び方](#mcp-からの呼び方)）。デバッグを有効にしたときだけ動く（[シーンを再生するイベントの共通の決まり](#シーンを再生するイベントの共通の決まり)）。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | グローバルシーンの名前（必須）。`＊会話` なら `会話` |
| `act.req.reference[1]` | ローカルシーンの名前（省略できる）。`・挨拶` なら `挨拶`。空の文字列は省略と同じ |

- Reference1 を省略すると、Reference0 のグローバルシーンを再生する。Reference1 を渡すと、Reference0 のグローバルシーンの中にある Reference1 のローカルシーンを再生する。
- 名前は、辞書に書いたとおりの、番号の付かない名前で渡す。`会話_1` のような登録名では指名できない。
- 名前は完全一致で探す。`会話` で `＊会話2` は再生しない。見つからなくても、前方一致で探し直すことはない。
- 照合用の名前への揃え方と、`[scene.alias]` の別名の当て方は、ふだんのシーンの検索と同じである（[search_scene](modules/pasta-search.md#search_scenename-global_scene_name)・[シーン名の別名](../grammar/call-jump.md#シーン名の別名)）。別名を当てるのはグローバルシーンの名前だけで、ローカルシーンの名前には当てない。既定の別名表では、`会話` を渡すと `OnTalk` に置き換わり、`＊OnTalk` と `＊会話` の両方が候補になる。
- 同じ名前のグローバルシーンが複数あれば、ほかのシーンと同じくシャッフル＆順次消費で 1 つを選ぶ。
- ローカルシーンを指名したときは、その名前のローカルシーンを持つグローバルシーンだけを候補にし、その中からシャッフル＆順次消費で 1 つを選ぶ。同じ名前のグローバルシーンのうち片方だけがローカルシーンを持つときも、持つ方が毎回選ばれる。

見つからないときは再生せず、失敗の表記とログの警告を出す。チェイントークで中断しているシーンは、そのまま残る。

| 場合 | 表記 | ログの警告 |
| ---- | ---- | ---------- |
| グローバルシーンが無い | `【シーンが見つからない：OnPastaScene ＊会話】` | `OnPastaScene: scene not found: global='会話'` |
| グローバルシーンはあるが、その中にローカルシーンが無い | `【シーンが見つからない：OnPastaScene ＊会話 ・挨拶】` | `OnPastaScene: scene not found: global='会話', local='挨拶'` |
| Reference0 が無い | `【名前にならない値：OnPastaScene Reference0（値なし）】` | `OnPastaScene: bad argument: Reference0=nil` |
| Reference0 が空の文字列 | `【名前にならない値：OnPastaScene Reference0（空文字列）】` | `OnPastaScene: bad argument: Reference0=''` |

```pasta
＊会話
    ぱすた：今日はいい天気だね。

    ・挨拶
        ぱすた：こんにちは。
```

この辞書に `OnPastaScene` を Reference0 `会話`・Reference1 `挨拶` で送ると、`・挨拶` を再生し、その台詞が応答の `Value` になる。

### OnPastaSceneAt（位置でシーンを再生）

ファイルと行で指名したシーンを、その場で再生する。`OnPastaScene` と同じく pasta の組み込みのイベントで、デバッグを有効にしたときだけ動く（[シーンを再生するイベントの共通の決まり](#シーンを再生するイベントの共通の決まり)）。同じ名前のシーンが複数あっても、狙った 1 つを再生できる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | `.pasta` ファイルのパス（必須）。`ghost/master/` からの相対パスか、絶対パス |
| `act.req.reference[1]` | 行（必須）。エディタと同じく 1 から数える。半角の数字だけで書いた 1 以上の整数 |

- 相対パスは、ゴーストの `ghost/master/` からの相対パスとして扱う（`dic/boot.pasta`）。SSP の今のフォルダには左右されない。絶対パスは、そのパスのファイルとして扱う。
- パスの区切りは `/` と `\` のどちらでもよい。ファイル名の大文字・小文字は、VSCode からのシーンの再生と同じ規則で突き合わせる（Windows では区別しない）。
- 行からシーンを引く規則は、VSCode の「▶ シーンを実行」と同じである（[シーン再生](../debug/dev-actions.md#シーン再生-シーンを実行)）。その行を含むシーンのうち、最も内側のシーンを再生する。グローバルシーンの中のローカルシーンの行ならローカルシーン、ローカルシーンより前の行ならグローバルシーンになる。どのシーンにも含まれない行（最初のシーンより前の行）では、その行より下にある最も近いシーンを再生する。
- ファイルの最後のシーンはファイルの終わりまで続くとみなすので、辞書に読み込まれたファイルの行は、ファイルの行数を超える行も含めて、必ずどれかのシーンに当たる。位置で見つからないのは、辞書に読み込まれていないファイルを指したときである。
- 同じ名前のシーンが複数あっても、指した位置のシーンそのものを再生する。
- ファイルは開かない。読み込みのときに作った、行からシーンを引くための表だけを使う。
- 行の `"４２"`（全角の数字）・`" 42"`（前後の空白）・`"0"`・`"1.5"`・`"abc"` は誤りになる。

見つからないときと引数の誤りでは再生せず、失敗の表記とログの警告を出す。チェイントークで中断しているシーンは、そのまま残る。表記のパスは渡された文字列のまま出る。

| 場合 | 表記 | ログの警告 |
| ---- | ---- | ---------- |
| 辞書に無いファイルを指した | `【シーンが見つからない：OnPastaSceneAt dic/無い.pasta:1】` | `OnPastaSceneAt: scene not found: file='dic/無い.pasta', line=1` |
| Reference0 が無い | `【名前にならない値：OnPastaSceneAt Reference0（値なし）】` | `OnPastaSceneAt: bad argument: Reference0=nil` |
| Reference0 が空の文字列 | `【名前にならない値：OnPastaSceneAt Reference0（空文字列）】` | `OnPastaSceneAt: bad argument: Reference0=''` |
| 行が 1 以上の整数でない | `【数にできない値：OnPastaSceneAt Reference1（"４２"）】` | `OnPastaSceneAt: bad argument: Reference1='４２'` |
| 行が無い | `【数にできない値：OnPastaSceneAt Reference1（値なし）】` | `OnPastaSceneAt: bad argument: Reference1=nil` |

### シーンを再生するイベントの共通の決まり

`OnPastaScene`・`OnPastaSceneAt` の 2 つに共通する決まりである。

**デバッグが有効なときだけ動く**

- `pasta.toml` の `[debug]` の `enabled = true` か、環境変数 `PASTA_DEBUG` でデバッグを有効にしたときだけ、シーンを再生する（[デバッグの有効化](../debug/index.md#デバッグの有効化)）。
- 有効かどうかは設定だけで決まる。VSCode のデバッグの待ち受けが立たなかった（ポートが取れなかった）ときも再生する。
- デバッグが無効なときは、シーンを再生せずに 204 No Content を返し、ログに警告 `OnPastaScene: debug is disabled; scene not played`（イベント名は届いたもの）を書く。
- `SecurityLevel` が `external` のリクエストは、デバッグが有効でも再生せずに 204 No Content を返し、ログに警告 `OnPastaScene: SecurityLevel external is refused; scene not played` を書く。
- 断るときは引数を見ないので、失敗の表記は出ない。204 の理由は、pasta のログ（既定は `profile/pasta/logs/pasta.log`）でだけ分かる。

**再生と続き**

- 受け取ったその場で再生する。Status に[ブロック対象のキーワード](#ブロック対象-status-キーワード)（選択肢の表示中の `choosing` など）が含まれていても止めない（ほかのイベントのハンドラと同じ）。
- 再生すると、再生したシーンの台詞の有無にかかわらず、チェイントークで待っている続きを捨て、[`＊チェイントーク中断`](../grammar/call-jump.md#チェイントーク中断) も実行しない。VSCode からのシーンの再生と同じである（[続きが捨てられるとき](../grammar/call-jump.md#続きが捨てられるとき)）。
- 再生したシーンが `＞チェイントーク` で区切れば、その残りが新しい続きになる。
- シーンを再生しなかったとき（見つからない・引数の誤り・デバッグが無効・`SecurityLevel` が `external`）は、待っている続きに触れない。
- このイベントの応答には、再生したシーンの最初の[チェイントーク](../grammar/call-jump.md#チェイントーク)までの出力だけが載る。残りは、ほかのイベントで実行したシーンと同じく、次の自然なおしゃべり（仮想ディスパッチャが起こす OnTalk）の機会に出る。
- 再生したシーンの実行がエラーで止まったときは、ほかのイベントと同じく止まるエラーの応答になる（[エラーハンドリング](#エラーハンドリング)）。

**失敗の出方**

- 見つからないときと引数の誤りは、ほかの実行時の失敗と同じ `【種類：対象（説明）】` の形の表記で出す。対象は、イベント名と、探した名前（`＊グローバルシーン名 ・ローカルシーン名`）・位置（`ファイル:行`）・誤った引数（`ReferenceN`）を半角の空白でつないだものである（[実行時の失敗の出力](../reference/runtime-failures.md)）。
- 表記は既定でこのイベントの応答（200 OK の `Value`）に載る。表記を出さない設定（`[logging]` の `balloon_warning = false`）では 204 No Content になる。ログの警告は、どちらの設定でも出る。
- `Value` の中では、表記に入ったパスの `\` が `\\` になる（さくらスクリプトとして働かないように直す）。

**作者のハンドラとシーン関数フォールバック**

- 2 つの既定ハンドラは、ゴーストが同じキーに登録したハンドラを上書きしない（[既定ハンドラと上書き](#既定ハンドラと上書き)）。
- 2 つのイベントには既定ハンドラがあるので、[シーン関数フォールバック](#シーン関数フォールバック)は起きない。デバッグが無効なときも、`＊OnPastaScene` や `＊OnPastaScene朝` のような、イベント名で始まる名前のシーンを前方一致で再生することはない。

### MCP からの呼び方

SSP の MCP サーバーの `raise_event` は、イベントを SHIORI に送り、返った台本を再生して、ツールの結果にも台本を載せる。`event` にイベント名を、`references` に Reference0 から順の値を文字列で渡す。

```text
reload(target = "shiori")

raise_event(event = "OnPastaScene", references = ["会話"])
raise_event(event = "OnPastaScene", references = ["会話", "挨拶"])
raise_event(event = "OnPastaSceneAt", references = ["dic/boot.pasta", "42"])
```

- 1 行目は、辞書を読み込み直す呼び出しである。
- 2 行目は `＊会話` を、3 行目は `＊会話` の中の `・挨拶` を、4 行目は `dic/boot.pasta` の 42 行目のシーンを再生する。
- 再生したときは、ツールの結果に、最初のチェイントークまでの台本が載る。見つからないときと引数の誤りでは、失敗の表記が載る。何も載らないとき（204）は、pasta のログで理由を見る。

辞書を書き換えたあとは、2 つのイベントを呼ぶ前に、MCP の `reload` に `target = "shiori"` を渡して SHIORI を読み込み直す。これは呼び出す側の務めである。`reload` は読み込み直しを頼むだけで、終わるのを待たない。終わる前に `raise_event` を呼ぶと、イベントは SHIORI に届かず、台本も載らない。MCP の `get_status` が答えるようになってから呼ぶ。

- シーンの一覧と、行からシーンを引くための表は、SHIORI の読み込みのときに作られ、辞書のファイルを書き換えても変わらない。
- 読み込み直さないと、書き足したシーンや名前を変えたシーンは見つからず、`OnPastaSceneAt` の行は書き換える前の辞書の行として引かれる。行がずれると、狙ったものとは違うシーンが、失敗も出さずに再生されることがある。

### OnPastaCallFunction（＞名前を外から呼ぶ）

名前で指名したものを、辞書の `＞名前` と同じ探し方で呼ぶ。pasta の組み込みのイベントで、ベースウェアが自分から送ることはない。AI エージェントなどが、SSP の MCP サーバーの `raise_event` で送る。試験の間だけ [`＞自動トーク停止`](../grammar/call-jump.md#自動トーク停止自動トーク再開) を呼ぶ、といった使い方をする。デバッグの有無によらず働く。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 呼ぶ名前（必須）。`＞自動トーク停止` なら `自動トーク停止` |
| `act.req.reference[1]` 以降 | 呼び先に渡す引数（省略できる） |

**探し方**

- 辞書のシーンの外に `＞名前` を書いたのと同じに探す。[スコープ解決アルゴリズム](../grammar/call-jump.md#スコープ解決アルゴリズム)の 5 段のうち、実行中のグローバルシーンが無いので、3 段目以降（act のメソッド・`GLOBAL` テーブル・すべてのグローバルシーン）から探す。
- `GLOBAL` の関数（`自動トーク停止`・`ゴースト終了` など）と、`act` のメソッドにも届く。
- グローバルシーンは前方一致で探す。`初回` で `＊初回起動` も候補になる。候補が複数あれば、シャッフル＆順次消費で 1 つを選ぶ（[候補の選択](../grammar/call-jump.md#候補の選択)）。呼ぶたびに順送りが進む。
- 照合用の名前への揃え方と、`[scene.alias]` の別名の当て方は、`＞名前` と同じである（[シーン名の別名](../grammar/call-jump.md#シーン名の別名)）。
- ローカルシーンは探さない。ローカルシーンを指して再生するときは [OnPastaScene](#onpastascene名前でシーンを再生) を使う（[OnPastaCallFunction と OnPastaScene の違い](#onpastacallfunction-と-onpastascene-の違い)）。

**引数**

- Reference1 から、値の無い最初の Reference の手前までを、受け取った文字列のまま、順に引数として渡す。数に直すことはしない。空の文字列も 1 つの引数として渡す。
- シーンを呼んだときは、Reference1 がシーン引数の `＄０`、Reference2 が `＄１` になる（[シーン引数](../grammar/variables.md#シーン引数)）。`GLOBAL` の関数を呼んだときは、関数が `act, Reference1, Reference2, …` の順に引数を受け取る。

**応答**

- 呼び先が台詞を出せば、その最初の[チェイントーク](../grammar/call-jump.md#チェイントーク)までの出力を `Value` にして 200 OK を返す。残りは、ほかのイベントで実行したシーンと同じく、次の自然なおしゃべり（仮想ディスパッチャが起こす OnTalk）の機会に出る。
- 呼び先が何も出さなければ（`自動トーク停止` など）、204 No Content を返す。
- 呼び先の戻り値は、応答に載らない。
- 受け取ったその場で呼ぶ。Status に[ブロック対象のキーワード](#ブロック対象-status-キーワード)が含まれていても止めない（ほかのイベントのハンドラと同じ）。
- 呼び先の実行がエラーで止まったときは、ほかのイベントと同じく止まるエラーの応答になる（[エラーハンドリング](#エラーハンドリング)）。

**失敗の出方**

見つからないときと引数の誤りでは何も呼ばず、失敗の表記とログの警告を出す。表記の形と、表記を出さない設定（`balloon_warning = false`）で 204 No Content になることは、[シーンを再生するイベントの共通の決まり](#シーンを再生するイベントの共通の決まり)の「失敗の出方」と同じである。ただし、表記はデバッグの有無によらず出る。

| 場合 | 表記 | ログの警告 |
| ---- | ---- | ---------- |
| 名前が見つからない | `【シーンが見つからない：OnPastaCallFunction ＞無い名前】` | `OnPastaCallFunction: handler not found: name='無い名前'` |
| Reference0 が無い | `【名前にならない値：OnPastaCallFunction Reference0（値なし）】` | `OnPastaCallFunction: bad argument: Reference0=nil` |
| Reference0 が空の文字列 | `【名前にならない値：OnPastaCallFunction Reference0（空文字列）】` | `OnPastaCallFunction: bad argument: Reference0=''` |

**断り**

- `SecurityLevel` が `external` のリクエストは、何も呼ばずに 204 No Content を返し、ログに警告 `OnPastaCallFunction: SecurityLevel external is refused; nothing done` を書く。ほかの機械から `ゴースト終了` などを呼べないようにするためである。MCP からは `local` で届くので、断られない。
- 断るときは引数を見ないので、失敗の表記は出ない。

**作者のハンドラで止める**

既定ハンドラは、ゴーストが `REG.OnPastaCallFunction` に登録したハンドラを上書きしない（[既定ハンドラと上書き](#既定ハンドラと上書き)）。外から呼ばせたくないときは、何もしないハンドラを登録する。いつも 204 No Content を返すようになる。

```lua
local REG = require("pasta.shiori.event.register")

REG.OnPastaCallFunction = function() end  -- 外からの呼び出しを受け付けない
```

既定ハンドラがあるので、[シーン関数フォールバック](#シーン関数フォールバック)は起きない。`＊OnPastaCallFunction` のような、イベント名で始まる名前のシーンを前方一致で実行することはない。

**MCP からの呼び方**

```text
raise_event(event = "OnPastaCallFunction", references = ["自動トーク停止"])
raise_event(event = "OnPastaCallFunction", references = ["自動トーク再開"])
raise_event(event = "OnPastaCallFunction", references = ["初回起動"])
```

- 1 行目は自動トークを止め、2 行目は再開する。どちらも台詞を出さないので、ツールの結果には何も載らない（204）。
- 3 行目は `＊初回起動` を呼び、最初のチェイントークまでの台本がツールの結果に載る。
- 辞書を書き換えたあとは、呼ぶ前に `reload(target = "shiori")` で SHIORI を読み込み直す（[MCP からの呼び方](#mcp-からの呼び方)）。

### OnPastaSetSaveVariable（グローバル変数に代入）

グローバル変数 `＄＊名前` に、外から値を代入する。`OnPastaCallFunction` と同じく pasta の組み込みのイベントで、デバッグの有無によらず働く。試験の間だけ [`＄＊pasta_talk_interval_min`](../grammar/variables.md#予約グローバル変数pasta_-で始まる名前) を縮める、といった使い方をする。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 変数の名前（必須）。`＄＊会員番号` なら `会員番号` |
| `act.req.reference[1]` | 代入する値（必須）。JSON の値 1 つで書く |

**値の読み方**

Reference1 を JSON の値 1 つとして読み、その型のまま代入する。

| Reference1 | 代入される値 |
| ---------- | ------------ |
| `60` | 数値の `60` |
| `"01"`（引用符も含めて送る） | 文字列の `01` |
| `true`・`false` | 真偽値 |
| `null` | 変数を消す（値の無い状態に戻す） |

- 文字列にしたいときは、値を `"` で囲んで送る。引用符なしの `01` は JSON として読めないので、失敗になる（下の表）。
- JSON の配列（`[1,2]`）とオブジェクト（`{"a":1}`）は代入しない。失敗になり、変数は変わらない。
- 名前は区切らない。`a.b` は、`a` の中の `b` ではなく、`a.b` という名前の 1 つの変数になる。
- `pasta_` で始まる[予約グローバル変数](../grammar/variables.md#予約グローバル変数pasta_-で始まる名前)にも代入できる。トークの間隔の変数は数値で送る（`60`。`"60"` は文字列なので無視される）。代入した値は、次にトーク間隔を計算するときから反映される。
- 代入した値は、その後に読む `＄＊名前`（Lua の `save.名前`）にすぐ見える。保存は、ほかのグローバル変数と同じく、ランタイムの終了時に行われる（[グローバル変数の保存先](../grammar/variables.md#グローバル変数の保存先)）。代入するたびにファイルへ書き出すことはしない。

**応答**

- 代入したとき・消したときは、204 No Content を返す。ログは出さない。
- 変数の値を読み出して返すことはしない。

**失敗の出方**

失敗では変数を変えず、失敗の表記とログの警告を出す。表記の出方は `OnPastaCallFunction` と同じである（デバッグの有無によらず出る。`balloon_warning = false` では 204 No Content）。表の例は、名前 `会員番号` に送ったときである。

| 場合 | 表記 | ログの警告 |
| ---- | ---- | ---------- |
| Reference1 が JSON として読めない（引用符なしの `01` を送った） | `【JSON として読めない値：OnPastaSetSaveVariable ＄＊会員番号（01）】` | `OnPastaSetSaveVariable: not JSON: name='会員番号', Reference1='01'` |
| Reference1 が無い | `【JSON として読めない値：OnPastaSetSaveVariable ＄＊会員番号（値なし）】` | `OnPastaSetSaveVariable: not JSON: name='会員番号', Reference1=nil` |
| Reference1 が空の文字列 | `【JSON として読めない値：OnPastaSetSaveVariable ＄＊会員番号（空文字列）】` | `OnPastaSetSaveVariable: not JSON: name='会員番号', Reference1=''` |
| Reference1 が配列かオブジェクト | `【代入できない値：OnPastaSetSaveVariable ＄＊会員番号（表）】` | `OnPastaSetSaveVariable: JSON array or object is refused: name='会員番号', Reference1='[1,2]'` |
| JSON を読む部品（`@json`）が読み込めない | `【JSON として読めない値：OnPastaSetSaveVariable ＄＊会員番号（JSON の部品が無い）】` | `OnPastaSetSaveVariable: @json is not available; variable not set: name='会員番号'` |
| Reference0 が無い | `【名前にならない値：OnPastaSetSaveVariable Reference0（値なし）】` | `OnPastaSetSaveVariable: bad argument: Reference0=nil` |
| Reference0 が空の文字列 | `【名前にならない値：OnPastaSetSaveVariable Reference0（空文字列）】` | `OnPastaSetSaveVariable: bad argument: Reference0=''` |

- 読めない値は、受け取った文字列を引用符で包まずに、そのまま表記に入れる。`（01）` と出たら、引用符なしで届いている。
- Reference1 が無いときも、変数は消さない。消すのは `null` を送ったときだけである。

**断り**

- `SecurityLevel` が `external` のリクエストは、代入せずに 204 No Content を返し、ログに警告 `OnPastaSetSaveVariable: SecurityLevel external is refused; nothing done` を書く。MCP からは `local` で届くので、断られない。
- 断るときは引数を見ないので、失敗の表記は出ない。

**作者のハンドラで止める**

`OnPastaCallFunction` と同じく、ゴーストが `REG.OnPastaSetSaveVariable` にハンドラを登録すると、既定ハンドラは動かない（[既定ハンドラと上書き](#既定ハンドラと上書き)）。

```lua
local REG = require("pasta.shiori.event.register")

REG.OnPastaSetSaveVariable = function() end  -- 外からの代入を受け付けない
```

既定ハンドラがあるので、`＊OnPastaSetSaveVariable` のような名前のシーンへの[シーン関数フォールバック](#シーン関数フォールバック)は起きない。

**MCP からの呼び方**

```text
raise_event(event = "OnPastaSetSaveVariable", references = ["pasta_talk_interval_min", "60"])
raise_event(event = "OnPastaSetSaveVariable", references = ["会員番号", "\"01\""])
raise_event(event = "OnPastaSetSaveVariable", references = ["初回済み", "null"])
```

- 1 行目は `＄＊pasta_talk_interval_min` に数値の `60` を代入する。
- 2 行目は `＄＊会員番号` に文字列の `01` を代入する。`references` の中では、JSON の引用符を `\"` と書く。SHIORI には `"01"` が届く。
- 3 行目は `＄＊初回済み` を消す。
- `references = ["会員番号", "01"]` と送ると、SHIORI には引用符なしの `01` が届き、`【JSON として読めない値：OnPastaSetSaveVariable ＄＊会員番号（01）】` がツールの結果に載る。

### OnPastaCallFunction と OnPastaScene の違い

どちらも名前で指名して、その場で動かすイベントである。違いは次のとおり。

| | `OnPastaCallFunction` | `OnPastaScene` |
| - | --------------------- | -------------- |
| いつ働くか | いつでも | デバッグが有効なときだけ |
| 探し方 | `＞名前` と同じ。グローバルシーンは前方一致。`GLOBAL` の関数・`act` のメソッドにも届く | シーンだけを完全一致で探す |
| ローカルシーン | 指せない | Reference1 で指せる |
| 引数 | Reference1 以降を渡せる | 渡せない |
| 見つからないとき | 失敗の表記とログの警告 | 失敗の表記とログの警告 |
| `SecurityLevel` が `external` | 断る（204） | 断る（204） |

- `＊挨拶` と `＊挨拶2` があるとき、`OnPastaCallFunction` の `挨拶` はどちらも候補にするが、`OnPastaScene` の `挨拶` は `＊挨拶` だけを再生する。
- `自動トーク停止` のような `GLOBAL` の関数は、`OnPastaCallFunction` でだけ呼べる。

## つつき・撫でのショートカット

`OnMouseDoubleClick`（つつき）と `OnMouseMove`（撫で）の既定ハンドラは、触られたキャラクターと部位に合わせた名前のシーンを探す。作者は、決まった名前のグローバルシーンを書くだけで、誰のどこを触られたかで反応を変えられる。

### 探す順

次の 9 段を上から順に探し、最初に見つかったシーンを実行する。各段の日本語の名前の次に、英字の名前を探す。

| 段 | つつき | 撫で |
| -- | ------ | ---- |
| 1 | `つっつき反応<アクター名><部位>` | `さわり反応<アクター名><部位>` |
| 2 | `OnPoke<アクター名><部位>` | `OnTouch<アクター名><部位>` |
| 3 | `つっつき反応<アクター名>その他` | `さわり反応<アクター名>その他` |
| 4 | `OnPoke<アクター名>Other` | `OnTouch<アクター名>Other` |
| 5 | `つっつき反応<部位>` | `さわり反応<部位>` |
| 6 | `OnPoke<部位>` | `OnTouch<部位>` |
| 7 | `つっつき反応その他` | `さわり反応その他` |
| 8 | `OnPokeOther` | `OnTouchOther` |
| 9 | `OnMouseDoubleClick` | `OnMouseMove` |

- `<アクター名>` は触られたキャラクターのアクター名（[アクター名の決め方](#アクター名の決め方)）、`<部位>` は当たり判定の識別子（Reference4）である。部位の名前は、シェルが送ってきたとおりに（英字も日本語も変えずに）入れる。
- アクター名が決まらないときは、アクター名を含む段（1〜4）を飛ばす。
- 部位が空（当たり判定の外）のつつきでは、部位を含む段（1・2・5・6）を飛ばす。部位が空のマウスの移動では、撫でたと決めない。
- 9 段のどれにもシーンが無ければ 204 No Content を返す。
- ショートカットのシーンの中でも、今までどおり [`＞transfer_req_to_var`](../grammar/variables.md#リクエスト変数reference) で Reference を読める。

```pasta
＊つっつき反応女の子Head
    女の子：頭をつつかないで。

＊つっつき反応男の子その他
    男の子：わっ、ぼくのこと、つついた？

＊さわり反応その他
    女の子：くすぐったいよ。
```

### アクター名の決め方

スコープ番号（Reference3）から、pasta.toml の `[actor."名前"]` の `spot` がその番号に等しいアクターを探す（[pasta.toml リファレンス](../reference/pasta-toml.md#spot)）。`spot` は読み込み時の設定の値で引く。シーンの `％` 行でスポットを入れ替えても、引く値は変わらない。

| `spot` が等しいアクター | アクター名 |
| ----------------------- | ---------- |
| 1 人 | そのアクターの名前 |
| 0 人 | 決まらない |
| 2 人以上 | そのスポットで最後に話したアクターの名前。まだ誰も話していなければ決まらない |

スコープ番号そのものを入れた名前（`つっつき反応1Head` など）は探さない。

### 撫での判定

- 同じキャラクターの同じ部位の上で、マウスの移動が途切れずに 2 秒続くと、1 回「撫でた」と決める。2 秒を過ぎた後の最初の `OnMouseMove` で決まる。
- 時間は、リクエストの時刻を 1 秒より細かく測る（設定し直しなどの[時刻の飛び](#時刻の飛び)は差し引く）。
- 一度撫でたと決めた後は、数え直すまで次の「撫でた」を決めない。同じ所を撫で続けても 1 回である。
- 次のときは、撫での時間を数え直す。
  - 部位かキャラクターが変わったとき
  - 前の移動から 2 秒より長く途切れたとき
  - 時刻が前の移動より戻ったとき
  - 部位が空（当たり判定の外）の移動を受けたとき
- 判定の状態は 1 つだけ持つ。保存せず、ゴーストを読み直すと初めからになる。
- 2 秒のしきい値と途切れの秒数は、設定では変えられない。

### 抑止

- Status ヘッダに、仮想イベント（OnTalk・OnHour）と同じ[ブロック対象のキーワード](#ブロック対象-status-キーワード)（`talking` など）が含まれている間は、撫でたと決めず、撫での時間を数え直す。抑止が解けてから、改めて 2 秒撫で続けると撫でたと決まる。
- トークが終わってバルーンが残っているだけ（Status が `balloon(…)` だけ）のときは抑止しない。
- 抑止の間も、`OnMouseMove` という名前のシーンは今までどおり探す。抑止するのは撫での判定と撫でのショートカットだけである。
- つつきは抑止しない。しゃべっている途中でも探して返す（トークは打ち切られる）。
- [自動トークを止めている](../grammar/call-jump.md#自動トーク停止自動トーク再開)間は、つつき・撫でとも 1〜8 段を探さず、9 段目の `OnMouseDoubleClick`・`OnMouseMove` という名前のシーンだけを探す。撫では、Status による抑止と同じく撫でたと決めず、撫での時間を数え直す。`REG` に置いた自前のハンドラは変わらない。

### 名前の一致

- 1〜8 段は、名前と完全に一致するシーンだけを候補にする（照合用の名前どうしを比べる。[search_scene_exact](modules/pasta-search.md#search_scene_exactname-global_scene_name)）。`つっつき反応胸` を探すときに `＊つっつき反応胸右` は選ばない。部位 `女` で `＊つっつき反応女の子Head` は選ばない。
- 9 段目の `OnMouseDoubleClick`・`OnMouseMove` は、今までどおり前方一致で探す（[シーン関数フォールバック](#シーン関数フォールバック)）。
- 見つかった段に同じ名前のシーンが複数あれば、ほかのシーンと同じくシャッフル＆順次消費で 1 つを選ぶ。
- 接頭辞だけの名前のシーン（`＊つっつき反応`・`＊OnPoke`・`＊さわり反応`・`＊OnTouch`）は、どの段の名前とも一致しないため呼ばれない。

### `その他`・`Other` という名前の部位

部位の名前が `その他`・`Other` のシェルでも、部位を特別扱いせず、上の表のとおりに名前を作って上から探す。たとえば部位 `その他` のつつきでは、5 段目の名前が 7 段目と同じ `つっつき反応その他` になり、5 段目で見つかる。

### 自前のハンドラの優先

ゴーストが `REG.OnMouseDoubleClick`・`REG.OnMouseMove` にハンドラを登録すると、そのイベントでは既定ハンドラは一切動かない。ショートカットのシーンを探さず、撫での判定もしない。登録の置き場所は `main.lua` でも辞書の Lua ブロックでもよい（[既定ハンドラと上書き](#既定ハンドラと上書き)）。

### 空のシーンで反応を止める

見つかったショートカットのシーンが何も出力しなければ、204 No Content を返し、それより下の段は探さない。本文の無いシーンを部位の名前で書くと、その部位では反応しなくなる。

```pasta
＃ 閉じるボタンの当たり判定では、その他の反応を鳴らさない
＊つっつき反応Close

＊つっつき反応その他
    ぱすた：なあに？
```

## シーン関数フォールバック

鎖のハンドラが無いか全部見送り、`REG` にもハンドラが無いイベントは、イベント名を検索キーにしてシーンを探し、見つかったシーンを実行する。既定の OnBoot ハンドラも同じ動作をする。

```text
イベント到着
  ↓
REG:register で重ねたハンドラ（鎖）を新しい順に呼ぶ
  ├─ どれかが真の値を返した → そこで止めて、戻り値から応答（鎖の戻り値の表のとおり）
  └─ 鎖が無い・全部見送った
  ↓
REG[イベント名] がある？
  ├─ ある → ハンドラを実行 → 戻り値から応答（REG の戻り値の表のとおり）
  └─ ない → イベント名でシーンを探す
              ├─ 見つかった → シーンを実行 → 出力があれば 200 OK、無ければ 204 No Content
              └─ 見つからない → 204 No Content
```

- シーンの探し方は、Call と同じ 5 段の検索である（[スコープ解決アルゴリズム](../grammar/call-jump.md#スコープ解決アルゴリズム)）。イベントの開始時点では実行中のグローバルシーンが無いため、3 段目以降（act のメソッド・`GLOBAL` テーブル・すべてのグローバルシーン）から探される。
- グローバルシーンは前方一致で探される。同名のシーンを複数定義すると、シャッフル＆順次消費で 1 つが選ばれる。`＊OnBoot朝` のように、イベント名で始まる名前のシーンも候補になる。
- 見つかったシーンはコルーチンとして実行され、出力が `Value` になる。シーンがチェイントークで中断した場合、残りは次の自然なおしゃべり（仮想ディスパッチャが起こす OnTalk）の機会に出力される（[チェイントーク](../grammar/call-jump.md#チェイントーク)）。

DSL で書いたシーンは、イベント名と同じ名前にするだけでそのイベントに応答する。

```pasta
＊OnBoot
    ぱすた：こんにちは。

＊OnBoot
    ぱすた：おはよう。
```

この例では、OnBoot のたびに 2 つのシーンのどちらかが実行される。

イベント名に日本語の別名を付けると、別名で宣言したシーンもそのイベントに応答する。別名は pasta.toml の `[scene.alias]` に書く（[シーン名の別名](../grammar/call-jump.md#シーン名の別名)・[pasta.toml リファレンス](../reference/pasta-toml.md#sceneシーン名)）。

```toml
[scene.alias]
OnTalk = ["会話"]
OnBoot = ["起動"]
```

```pasta
＊起動
    ぱすた：こんにちは。
```

この例の `＊起動` は `OnBoot` という名前で登録され、OnBoot のシーンとして実行される。`[scene.alias]` を書くと既定の表（`OnTalk = ["会話"]`）は丸ごと置き換わるため、`＊会話` をランダムトークに使い続けるには `OnTalk = ["会話"]` も書く。

## エラーハンドリング

イベントの処理全体（コールバックの再開・`REG` のハンドラ・シーンの実行）は、SHIORI のリクエスト処理関数の中で `xpcall` により保護されている。エラーが起きて処理が止まると（止まるエラー）、`X-Error-Reason` ヘッダにエラーメッセージの最初の行を入れた応答を返す。エラーの値が文字列でないときは `"Unknown error"` になる。応答の形は、`pasta.toml` の `[logging]` の `balloon_warning` で変わる。

- 表記を出す設定（既定）: 200 OK を返し、`Value` に、エラーまでに積んだ台詞と、その後に止まった理由の表記（`【スクリプトのエラー：止まった場所（エラー文）】`）を入れる。止まった理由はバルーンに出る。
- 表記を出さない設定（`balloon_warning = false`）: これまでどおり、`Value` の無い 500 Internal Server Error を返す。バルーンには何も出ないので、`X-Error-Reason` の行とログの警告で理由を見る。

`scripts/main.lua` に次のハンドラを書いた例を示す（ヘッダは抜粋）。

```lua
local REG = require("pasta.shiori.event.register")

REG.OnBoot = function(act)
    error("何かがおかしい")
end
-- 応答（表記を出す設定。既定）:
--   SHIORI/3.0 200 OK
--   Value: 【スクリプトのエラー：OnBoot（scripts\\main.lua:4: 何かがおかしい）】\e
--   X-Error-Reason: （発生位置）: 何かがおかしい
-- 応答（balloon_warning = false）:
--   SHIORI/3.0 500 Internal Server Error
--   X-Error-Reason: （発生位置）: 何かがおかしい
```

- `error("…")` のメッセージには、通常、発生位置（ファイル名と行番号）が前に付く。`X-Error-Reason` には Lua のエラー文の最初の行がそのまま入り、中身は表記を出すかどうかで変わらない。
- 表記の中の場所は、ゴーストのフォルダーから見た形に直して出す。`Value` の `\\` はさくらスクリプトで `\` を表す書き方で、バルーンには `scripts\main.lua:4:` と出る。
- シーンの実行中にエラーが起きた場合も同じで、表記の止まった場所はシーンの名前になる。止まったシーンは再開されない。
- ID が `On` で始まらない要求（`version` などのリソースの要求）と NOTIFY の要求では、設定にかかわらず表記を出さない。
- 表記の形・止まった場所の示し方・出る応答の詳細は [止まるエラーの表記](../reference/runtime-failures.md#止まるエラーの表記)、設定は [表記の切り替え](../reference/runtime-failures.md#表記の切り替え) を参照。
- ゴーストの読み込みに失敗しているときの 500 応答（表記は出ない）の読み方は [ゴーストが起動しない・喋らないとき](../reference/startup.md#4-ゴーストが起動しない喋らないとき) を参照。

## 仮想ディスパッチャ

OnTalk（ランダムトーク）と OnHour（時報）は、ベースウェアが送るイベントではない。OnSecondChange の既定ハンドラが呼ぶ仮想ディスパッチャが、条件を判定して発行する「仮想イベント」である。発行されたシーンの出力は、その OnSecondChange への応答になる。

- 仮想イベントは `REG` を通らず、シーンを直接探して実行する。`REG.OnTalk`・`REG.OnHour` を登録しても呼ばれない。
- 状態（次の正時・次のトーク時刻・前に見た時刻の飛びの通し番号）は、SHIORI の load から unload までの間だけ保たれる。unload で Lua の実行環境ごと破棄され、次の load で初めからになる。

```lua
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
```

### dispatch(act)

仮想ディスパッチャの入口である。OnSecondChange の `act` を受け取り、発行するシーンのコルーチン（実行はしない）か `nil` を返す。

```lua
--- @param act ShioriAct
--- @return thread|nil
local co = dispatcher.dispatch(act)
```

次の順に判定する。

1. `act.req.date` が無ければ `nil` を返す。
2. `act.req.status` に[ブロック対象のキーワード](#ブロック対象-status-キーワード)が含まれていれば `nil` を返す（`is_blocked` で判定）。
3. デバッグの[シーン再生](../debug/dev-actions.md#シーン再生-シーンを実行)で再生を求められたシーンがあれば、そのシーンを返す。このときに限り、2 の判定を 1 回だけ無視する。
4. [自動トークを止めている](#自動トークを止めている間)なら `nil` を返す。
5. OnHour を判定する（[check_hour](#check_houract)）。発行するならそのシーンを返す。
6. OnTalk を判定する（[check_talk](#check_talkact)）。

### ブロック対象 Status キーワード

`act.req.status` に次のキーワードのいずれかが含まれていると、`dispatch` は何も発行しない。判定は文字列の部分一致で行う。

| キーワード | 意味 | 対応する状態 |
| ---------- | ---- | ------------ |
| `talking` | トーク中 | さくらスクリプトの実行中 |
| `choosing` | 選択肢の表示中 | `\q` の選択肢を待っている |
| `online` | ネットワーク通信中 | 更新の確認など |
| `opening` | 入力ボックスなどが開いている | `opening(communicate)` など |
| `passive` | パッシブモード中 | ほかのゴーストから制御されている |
| `induction` | インダクションモード中 | ほかのゴーストを呼び出している |
| `timecritical` | タイムクリティカルセクション中 | `\![set,timecritical]` |
| `nouserbreak` | ユーザーブレイク禁止中 | `\![set,nouserbreak]` |
| `minimizing` | 最小化中 | バルーンが表示されていない |

ブロックされた OnHour は、ブロックが解けた最初の OnSecondChange で発行される。

### 自動トークを止めている間

[`＞自動トーク停止`](../grammar/call-jump.md#自動トーク停止自動トーク再開)で止めている間は、`dispatch` は OnHour・OnTalk の判定に進まず、止まっているシーンの続き（チェイントークで中断したシーンの残りと、シーン再生で再生したシーンの 2 回目以降の出力）も返さない。待っている続きは捨てない。判定の位置は Status による抑止とシーン再生の後なので、シーン再生で求めたシーンの最初の出力は出る。

Status による抑止とは、時報の扱いが違う。

- Status による抑止: 抑止が解けた最初の OnSecondChange で、その間に過ぎた正時の時報を出す。
- 自動トークの停止: 止めている間に過ぎた正時の時報は、再開した後にも出さない。

`＞自動トーク再開` で再開すると、仮想ディスパッチャの時計を決め直す（`restart_timers(act)`）。

- 次のトーク時刻を未設定に戻す。再開した後の最初の [check_talk](#check_talkact) が、そこから間隔を数え直す。
- 次の正時を、再開した時刻の次の正時にする（今の値より後になるときだけ。仮想時刻の基準が変わっていれば置き換える）。`act.req.date` が無いときは未設定に戻し、次の [check_hour](#check_houract) が決める。

### is_blocked(status)

Status の文字列にブロック対象のキーワードが含まれるかを判定する。`dispatch` が内部で使うほか、ほかのイベントハンドラからも使える。

```lua
--- @param status string|nil act.req.status の値
--- @return boolean true なら発行しない、false なら発行してよい
local blocked = dispatcher.is_blocked(status)
```

```lua
local REG = require("pasta.shiori.event.register")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

REG.OnMouseDoubleClick = function(act)
    if dispatcher.is_blocked(act.req.status) then
        return nil
    end
    return "\\0\\s[0]なあに？\\e"
end
```

### OnHour

正時（毎時 0 分 0 秒）を過ぎた最初の OnSecondChange で、時報のシーンを発行する。正時は SSP の仮想時刻で数える（[OnPastaClock](#onpastaclock仮想時刻の読み取り)）。仮想時刻を使っていなければ、OS の時計で数える。シーンは次の 4 つの候補を順に探し、最初に見つかった候補を実行する。

1. `時報{HH}`（時刻別のシーン。例: `時報12` は正午専用）
2. `OnHour{HH}`（時刻別のシーン、英語名）
3. `時報その他`（汎用の時報シーン）
4. `OnHourOther`（汎用の時報シーン、英語名）

- `{HH}` は、仮想時刻の「時」を 0 埋めした 2 桁（`00`〜`23`）である。仮想時刻を使っていなければ `act.req.date.hour` と同じである。
- 候補の探し方は[シーン関数フォールバック](#シーン関数フォールバック)と同じ（前方一致）。
- 4 つの候補がどれも無ければ、時報は発行せず、その回は OnTalk の判定へ進む（[check_talk](#check_talkact)）。
- `OnHour` という名前そのものは候補に無い。`＊OnHour` という名前のシーンは時報では実行されない。
- 候補を探す前に、[日時変数](../grammar/variables.md#日時変数)（`＄時` など）に仮想の日時の値が入る。

```pasta
＊時報12
    ぱすた：お昼の 12 時です。

＊時報その他
    ぱすた：＄時　になりました。
```

#### check_hour(act)

OnHour の判定と発行を行う。発行するシーンのコルーチンか `nil` を返す。

```lua
--- @param act ShioriAct
--- @return thread|nil
local co = dispatcher.check_hour(act)
```

- 現在時刻は、仮想時刻（OS の時計にずれを足した時刻）である。
- 最初の呼び出しでは、次の正時を計算して記録するだけで、何も発行しない。
- 前の呼び出しから[時刻の飛び](#時刻の飛び)があれば、次の正時を決め直す。決め直しは現在時刻の 70 秒前から数えるので、70 秒ほど前までに過ぎた正時は、1 回だけ時報になる。
- 現在時刻が記録した正時に達していれば、次の正時を記録し直してから、4 つの候補でシーンを探す。
- 一度に何時間分も進んでいても、時報は 1 回だけである。`{HH}` は進んだ先の時になる。

### OnTalk

一定の間隔で、ランダムトークのシーン（`OnTalk`）を発行する。

- 同名の `＊OnTalk` を複数定義すると、シャッフル＆順次消費で 1 つが選ばれる。前方一致で探すため、`OnTalk` で始まる名前のシーンも候補になる。
- `＊会話` も候補になる。既定の別名表（`OnTalk = ["会話"]`）により、`＊会話` は `OnTalk` という名前で登録されるためである。`＊OnTalk` と `＊会話` を混ぜて書いても、すべてが同じ名前の候補になり、シャッフル＆順次消費で 1 つが選ばれる。`＊会話・朝` のように別名で始まるだけの名前は候補にならない（[シーン名の別名](../grammar/call-jump.md#シーン名の別名)）。
- 前のシーンがチェイントークで中断していれば、新しい OnTalk のシーンの代わりに、その続きを出力する（[チェイントーク](../grammar/call-jump.md#チェイントーク)）。続きを出すのは、仮想ディスパッチャが起こす自然なおしゃべりだけである。
- 外から起こした `OnTalk`（`\![raise,OnTalk]` や開発用パレットなど）は、ベースウェアが送る通常のイベントとして扱われ、シーン関数フォールバックで新しいトークを始める。待っている続きは、ほかのイベントと同じく、そのシーンが台詞（字のある台詞か選択肢）を出したときだけ捨てられ、`＊チェイントーク中断` があればその台詞が代わりに出る（[続きが捨てられるとき](../grammar/call-jump.md#続きが捨てられるとき)・[チェイントーク中断](../grammar/call-jump.md#チェイントーク中断)）。
- act を通さずに Lua で作ったコルーチンをハンドラが返したときは、出力が空でなければ台詞を出したものとして続きを捨てるが、[`＊チェイントーク中断`](../grammar/call-jump.md#チェイントーク中断) の台詞は付かない。

```pasta
＊OnTalk
    ぱすた：今日はいい天気だね。

＊会話
    ぱすた：お腹がすいたな。
```

#### check_talk(act)

OnTalk の判定と発行を行う。発行するシーンのコルーチンか `nil` を返す。

```lua
--- @param act ShioriAct
--- @return thread|nil
local co = dispatcher.check_talk(act)
```

- トークの間隔は、[時刻の飛び](#時刻の飛び)を差し引いた時計で数える。仮想時刻を進めても、ランダムトークは早まらない。
- 最初の呼び出しでは、次のトーク時刻（現在時刻に `talk_interval_min`〜`talk_interval_max` の範囲の乱数秒を足した時刻）を決めるだけで、何も発行しない。
- 現在時刻が次のトーク時刻に達していても、次の正時までの残りが `hour_margin` 秒未満のときは発行しない（時報を優先する）。この残りは、正時と同じく仮想時刻で数える。
- 発行するときは、次のトーク時刻を決め直してから、チェイントークの続きか新しい `OnTalk` のシーンを返す。次のトーク時刻は、シーンが見つかったかどうかに関わらず決め直される。

### 時刻の飛び

pasta は、OnSecondChange が届くたびに、前の OnSecondChange からの時刻の差を見る。差が 2 秒より戻ったか 90 秒より進んだら「時刻の飛び」とみなす。開発用パレットの「現在時刻の仮想的変更」で Apply・Reset したときが、これにあたる。

- 飛びがあると、時報の次の正時を決め直す（[check_hour](#check_houract)）。仮想時刻を入れ直しても、余分な時報は出ない。
- おしゃべりの間隔・撫での判定・コールバックの待ち時間（[get_property](script-api.md#get_propertyname_or_names-timeout-timeout_message)）は、飛びの分を差し引いた時計で数える。仮想時刻を 3 時間進めても、次のおしゃべりまでの残りは変わらない。
- スリープから復帰した後の最初の差は、飛びとみなさない（`OnSysSuspend` か `OnSysResume` が届いた後の最初の飛び）。時報は 1 回出て、おしゃべりはすぐ出る。
- `X-Pasta-Time` ヘッダのあるリクエストでは、飛びを見ない。その日時をそのまま使う。
- 飛びの記録は保存しない。SHIORI の load から unload までの間だけ保たれる。

仮想時刻を使わずに OS の時計を手で動かしたときも、90 秒より進めたか 2 秒より戻せば飛びになる。

| 操作 | 時報 | おしゃべり |
| ---- | ---- | ---------- |
| 10 時 30 分から 9 時 30 分へ戻す | 10 時に出る | 残りの間隔を待つ |
| 10 時 30 分から 12 時 10 分へ進める | 13 時まで出ない | 残りの間隔を待つ |

SSP が 90 秒より長く OnSecondChange を送らなかったとき（デバッガでゴーストを止めたときなど）も、その間の時刻の差は飛びになる。止まっている間にまたいだ正時の時報は出ない。

### pasta.toml 設定

仮想ディスパッチャの動作は、`pasta.toml` の `[ghost]` セクションで設定する。

| 設定 | 型 | 既定値 | 説明 |
| ---- | -- | ------ | ---- |
| `talk_interval_min` | 整数 | `180` | ランダムトークの最小間隔（秒） |
| `talk_interval_max` | 整数 | `300` | ランダムトークの最大間隔（秒） |
| `hour_margin` | 整数 | `30` | 正時の何秒前からランダムトークを控えるか（秒） |

```toml
[ghost]
talk_interval_min = 180
talk_interval_max = 300
hour_margin = 30
```

- トークの間隔は、予約グローバル変数 `＄＊pasta_talk_interval_min`・`＄＊pasta_talk_interval_max` → `pasta.toml` の値 → 既定値、の順に決まる。数値でない値は無視され、小数は切り捨て、10 秒未満は 10 秒になり、最小間隔が最大間隔を上回ると最大間隔が最小間隔にそろえられる（[予約グローバル変数](../grammar/variables.md#予約グローバル変数pasta_-で始まる名前)）。
- 設定は、次のトーク時刻を決めるたびに読み直される。
- Lua から設定値を読むには [@pasta_config](modules/pasta-config.md) を使う。
- `[ghost]` の全キー（`spot_newlines` を含む）と既定値は [pasta.toml リファレンス](../reference/pasta-toml.md#ghostゴースト動作) を参照する。

### テスト用関数

仮想ディスパッチャの単体テストのための関数である。

```lua
-- 状態のリセット（セッション開始時と同じ状態にする。シーン実行関数の差し替えも解除する）
dispatcher._reset()

-- 内部状態の取得
local state = dispatcher._get_internal_state()
-- { next_hour_unix = 次の正時の Unix 時刻（仮想時刻）, next_talk_time = 次のトーク時刻, seen_epoch = 前に見た時刻の飛びの通し番号 }

-- シーンを探して実行する関数の差し替え（nil を渡すと解除）
dispatcher._set_scene_executor(function(event_name, act)
    -- event_name は "OnTalk" や時報の候補名（"時報12" など）
    return nil  -- シーンのコルーチンか nil を返す
end)

-- 解決済みの設定値の取得
local cfg = dispatcher._get_config()
-- { talk_interval_min = …, talk_interval_max = …, hour_margin = … }
```

---

> 【目閉じ】シーンに任せるのが基本、細かな捌きは REG、時報とランダムトークは仮想ディスパッチャにお任せなさいまし。これでイベントの受け止め方は一通りですわ。迷ったら、この章へ戻っていらっしゃい。

> 【アンソニー】既定のハンドラにうかつに手を出しますと、時報が黙ってしまいますので、どうぞご用心を。
