# SHIORI イベントとハンドラ

ゴーストが起動した、ダブルクリックされた、一秒が過ぎた――ベースウェアは、そうした出来事をひとつ残らず「イベント」として知らせてきますの。
この章では、わたくしがイベントの受け止め方を余さずお教えいたしますわ。シーンに任せるか、Lua で自ら捌くか、その作法をしっかり身につけてくださいまし。

---

ベースウェア（SSP など）は、出来事を SHIORI/3.0 のリクエスト（イベント）として送ってくる。pasta は 1 つのイベントを次の順で処理し、結果を SHIORI/3.0 の応答文字列にして返す。

1. 結果を待っているコールバック（[OnPastaCallBack](#onpastacallbackコールバック応答)）への応答なら、待っているシーンを再開する。
2. `REG` にそのイベントのハンドラがあれば、ハンドラを呼ぶ（[REG](#reg)）。
3. ハンドラが無ければ、イベント名でシーンを探して実行する（[シーン関数フォールバック](#シーン関数フォールバック)）。

処理中にエラーが起きると 500 応答になる（[エラーハンドリング](#エラーハンドリング)）。

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
| 文字列 | 200 OK。その文字列が `Value` ヘッダの値になる。空文字列なら 204 No Content |
| シーンのコルーチン（thread） | コルーチンを `act` を渡して再開し、得た出力を `Value` にして 200 OK。出力が無ければ 204 No Content |
| `nil`（何も返さない） | 204 No Content |

- 返した文字列は、そのまま `Value` になる。さくらスクリプトへの変換やウェイトの挿入は行われない。
- シーンの出力を応答にするには、シーンのコルーチンを返す。`SCENE.co_exec(act, 名前)` は、[シーン関数フォールバック](#シーン関数フォールバック)と同じ探し方でシーンを探し、見つかればそのコルーチンを、見つからなければ `nil` を返す。
- コルーチンの途中で[チェイントーク](../grammar/call-jump.md#チェイントーク)により中断した場合、残りは次の OnTalk の機会に出力される。
- 1 つのイベントに登録できるハンドラは 1 つである。同じキーに代入し直すと、前のハンドラは置き換わる。

```lua
local REG = require("pasta.shiori.event.register")
local SCENE = require("pasta.scene")

REG.OnMouseDoubleClick = function(act)
    -- シーン「なでられ」を探してコルーチンを返す（無ければ nil → 204）
    return SCENE.co_exec(act, "なでられ")
end
```

### 既定ハンドラと上書き

次の 3 つのイベントには、pasta が既定のハンドラを `REG` に登録している。`REG` の同じキーに代入すると、既定ハンドラは置き換わる。

| イベント | 既定ハンドラの動作 |
| -------- | ------------------ |
| `OnBoot` | イベント名でシーンを探して実行する（シーン関数フォールバックと同じ動作） |
| `OnChoiceSelectEx` | 選ばれた選択肢のシーンを実行する（[OnChoiceSelectEx](#onchoiceselectex)） |
| `OnSecondChange` | コールバックのタイムアウト処理と、仮想ディスパッチャによる OnTalk・OnHour の発行を行う（[OnSecondChange](#onsecondchange)） |

既定ハンドラは、エンジンの起動処理（SHIORI 応答関数の読み込み）の中で登録される。既定ハンドラを上書きする代入は、それより後に実行されるように置く（起動の順序は [起動シーケンス](../reference/startup.md#2-起動シーケンス)）。

- 辞書ファイル（`.pasta`）の Lua ブロックは、既定ハンドラの登録より後に評価される。そこで代入すれば上書きできる。
- `scripts/main.lua` は、既定ハンドラの登録より前に実行される。`main.lua` で上書きするときは、先に `require("pasta.shiori.event")` を呼んで既定ハンドラを登録させてから代入する。

```lua
-- scripts/main.lua
require("pasta.shiori.event")  -- 既定ハンドラを先に登録させる
local REG = require("pasta.shiori.event.register")

REG.OnBoot = function(act)
    return "\\0\\s[0]起動しました。\\e"
end
```

既定ハンドラを持たないイベントのハンドラは、どちらに書いてもよい。

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

`REG` のハンドラは、応答全体ではなく `Value` にする文字列を返す（[REG](#reg)）。

## 主要イベント

ベースウェアから送られる主なイベントと、pasta での扱いを示す。各 Reference の意味はベースウェアの仕様であり、すべてのイベントと Reference は [UKADOC の SHIORI Event 一覧](https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html)で確認できる。各イベントの Reference 表の「（SSP）」は SSP など一部のベースウェアだけが送る Reference を表す。

| 分類 | イベント | pasta の既定ハンドラ |
| ---- | -------- | -------------------- |
| 起動・終了 | `OnFirstBoot` | なし（同名のシーンを実行） |
| 起動・終了 | `OnBoot` | あり（同名のシーンを実行） |
| 起動・終了 | `OnClose` | なし（同名のシーンを実行） |
| 起動・終了 | `OnGhostChanged` | なし（同名のシーンを実行） |
| 選択肢 | `OnChoiceSelectEx` | あり（選択肢のルーティング） |
| マウス | `OnMouseDoubleClick` | なし（同名のシーンを実行） |
| 時間 | `OnSecondChange` | あり（コールバックのタイムアウト処理・仮想ディスパッチャ） |
| 時間 | `OnMinuteChange` | なし（同名のシーンを実行） |
| コールバック | `OnPastaCallBack{N}` | `REG` より先に処理される |

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

ユーザーが選択肢（さくらスクリプトの `\q[表示テキスト,ID]`）を選んだときに送られる。DSL の選択肢行 `＠？ジャンプ先` は `\![*]\q[表示テキスト,ジャンプ先]` を出力し、ジャンプ先の名前が選択肢の ID になる（構文は [選択肢行](../grammar/block-structure.md#選択肢行)）。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | 選択肢のテキスト（ラベル） |
| `act.req.reference[1]` | 選択肢の ID |
| `act.req.reference[2]` 以降 | 拡張情報（`\q` の 3 番目以降の引数） |

既定ハンドラ（自動ルーティング）は次の順で動く。通常はハンドラの登録は不要である。

1. `＊OnChoiceSelectEx` という名前のシーンがあれば、それを実行する（自動ルーティングより優先する）。シーンの探し方は[シーン関数フォールバック](#シーン関数フォールバック)と同じ。
2. 無ければ、選択 ID と前方一致するローカルシーンを、直前に実行したグローバルシーン（通常は選択肢を出したシーン）の配下から探し、見つかったシーンを実行する。候補が複数あれば、シャッフル＆順次消費で 1 つ選ばれる。
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

```pasta
＊OnMouseDoubleClick
    ぱすた：なあに？
```

```lua
REG.OnMouseDoubleClick = function(act)
    local scope = act.req.reference[3]
    local hit_area = act.req.reference[4]
    if scope == "0" and hit_area == "Head" then
        return "\\0\\s[0]頭をなでないで。\\e"
    end
    return nil
end
```

### OnSecondChange

1 秒ごとに送られる。

| Reference | 内容 |
| --------- | ---- |
| `act.req.reference[0]` | OS の連続起動時間（時間単位） |
| `act.req.reference[1]` | 見切れているとき `1`、それ以外は `0` |
| `act.req.reference[2]` | 重なっているとき `1`、それ以外は `0` |
| `act.req.reference[3]` | トークを再生できるとき `1`、それ以外は `0` |
| `act.req.reference[4]` | OS で何も操作されずに放置されている時間（秒、SSP） |

既定ハンドラは、次の 2 つを順に行う。

1. 結果を待っているコールバックのうち、待ち時間の上限を過ぎたものをタイムアウトとして処理する（[OnPastaCallBack](#onpastacallbackコールバック応答)）。
2. [仮想ディスパッチャ](#仮想ディスパッチャ)を呼び、OnHour（時報）・OnTalk（ランダムトーク）を発行する。

`REG.OnSecondChange` を上書きすると、OnTalk・OnHour の発行、コールバックのタイムアウト処理、デバッグのシーン再生がすべて止まる。毎秒の処理を足すときは、既定ハンドラを変数に取っておき、上書きしたハンドラから呼び出す（代入の置き場所は [既定ハンドラと上書き](#既定ハンドラと上書き) のとおり）。

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

- このイベントは `REG` より先に処理される。イベント名が待っているものと一致すると、待っているコルーチンを Reference の値で再開し、その出力を `Value` にして 200 OK（出力が無ければ 204 No Content）を返す。このとき `REG` のハンドラは呼ばれない。ハンドラを登録する必要はない。
- 待っているものと一致しない `OnPastaCallBack{N}` は、通常のイベントとして `REG`・シーン関数フォールバックへ進む。
- 待ち時間の上限（`act:get_property` の既定は 5 秒）を過ぎた待機は、OnSecondChange の既定ハンドラがタイムアウトとして処理する。

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

## シーン関数フォールバック

`REG` にハンドラが無いイベントは、イベント名を検索キーにしてシーンを探し、見つかったシーンを実行する。既定の OnBoot ハンドラも同じ動作をする。

```text
イベント到着
  ↓
REG[イベント名] がある？
  ├─ ある → ハンドラを実行 → 戻り値から応答（200 / 204）
  └─ ない → イベント名でシーンを探す
              ├─ 見つかった → シーンを実行 → 出力があれば 200 OK、無ければ 204 No Content
              └─ 見つからない → 204 No Content
```

- シーンの探し方は、Call と同じ 5 段の検索である（[スコープ解決アルゴリズム](../grammar/call-jump.md#スコープ解決アルゴリズム)）。イベントの開始時点では実行中のグローバルシーンが無いため、3 段目以降（act のメソッド・`GLOBAL` テーブル・すべてのグローバルシーン）から探される。
- グローバルシーンは前方一致で探される。同名のシーンを複数定義すると、シャッフル＆順次消費で 1 つが選ばれる。`＊OnBoot朝` のように、イベント名で始まる名前のシーンも候補になる。
- 見つかったシーンはコルーチンとして実行され、出力が `Value` になる。シーンがチェイントークで中断した場合、残りは次の OnTalk の機会に出力される（[チェイントーク](../grammar/call-jump.md#チェイントーク)）。

DSL で書いたシーンは、イベント名と同じ名前にするだけでそのイベントに応答する。

```pasta
＊OnBoot
    ぱすた：こんにちは。

＊OnBoot
    ぱすた：おはよう。
```

この例では、OnBoot のたびに 2 つのシーンのどちらかが実行される。

## エラーハンドリング

イベントの処理全体（コールバックの再開・`REG` のハンドラ・シーンの実行）は、SHIORI のリクエスト処理関数の中で `xpcall` により保護されている。エラーが起きると 500 Internal Server Error を返し、`X-Error-Reason` ヘッダにエラーメッセージの最初の行を入れる。エラーの値が文字列でないときは `"Unknown error"` になる。

```lua
REG.OnBoot = function(act)
    error("何かがおかしい")
end
-- 応答: SHIORI/3.0 500 Internal Server Error
--       X-Error-Reason: （発生位置）: 何かがおかしい
```

- `error("…")` のメッセージには、通常、発生位置（ファイル名と行番号）が前に付く。
- シーンの実行中にエラーが起きた場合も 500 になり、そのシーンは再開されない。
- 500 応答の読み方は [ゴーストが起動しない・喋らないとき](../reference/startup.md#4-ゴーストが起動しない喋らないとき) を参照。

## 仮想ディスパッチャ

OnTalk（ランダムトーク）と OnHour（時報）は、ベースウェアが送るイベントではない。OnSecondChange の既定ハンドラが呼ぶ仮想ディスパッチャが、条件を判定して発行する「仮想イベント」である。発行されたシーンの出力は、その OnSecondChange への応答になる。

- 仮想イベントは `REG` を通らず、シーンを直接探して実行する。`REG.OnTalk`・`REG.OnHour` を登録しても呼ばれない。
- 状態（次の正時・次のトーク時刻）は、SHIORI の load から unload までの間だけ保たれる。unload で Lua の実行環境ごと破棄され、次の load で初めからになる。

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
4. OnHour を判定する（[check_hour](#check_houract)）。発行するならそのシーンを返す。
5. OnTalk を判定する（[check_talk](#check_talkact)）。

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

正時（毎時 0 分 0 秒）を過ぎた最初の OnSecondChange で、時報のシーンを発行する。シーンは次の 4 つの候補を順に探し、最初に見つかった候補を実行する。

1. `時報{HH}`（時刻別のシーン。例: `時報12` は正午専用）
2. `OnHour{HH}`（時刻別のシーン、英語名）
3. `時報その他`（汎用の時報シーン）
4. `OnHourOther`（汎用の時報シーン、英語名）

- `{HH}` は `act.req.date.hour` を 0 埋めした 2 桁（`00`〜`23`）である。
- 候補の探し方は[シーン関数フォールバック](#シーン関数フォールバック)と同じ（前方一致）。
- 4 つの候補がどれも無ければ、何も発行しない。
- `OnHour` という名前そのものは候補に無い。`＊OnHour` という名前のシーンは時報では実行されない。
- 候補を探す前に、[日時変数](../grammar/variables.md#日時変数)（`＄時` など）に値が入る。

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

- 最初の呼び出しでは、次の正時を計算して記録するだけで、何も発行しない。
- 現在時刻が記録した正時に達していれば、次の正時を記録し直してから、4 つの候補でシーンを探す。

### OnTalk

一定の間隔で、ランダムトークのシーン（`OnTalk`）を発行する。

- 同名の `＊OnTalk` を複数定義すると、シャッフル＆順次消費で 1 つが選ばれる。前方一致で探すため、`OnTalk` で始まる名前のシーンも候補になる。
- 前のシーンがチェイントークで中断していれば、新しい OnTalk のシーンの代わりに、その続きを出力する（[チェイントーク](../grammar/call-jump.md#チェイントーク)）。

```pasta
＊OnTalk
    ぱすた：今日はいい天気だね。

＊OnTalk
    ぱすた：お腹がすいたな。
```

#### check_talk(act)

OnTalk の判定と発行を行う。発行するシーンのコルーチンか `nil` を返す。

```lua
--- @param act ShioriAct
--- @return thread|nil
local co = dispatcher.check_talk(act)
```

- 最初の呼び出しでは、次のトーク時刻（現在時刻に `talk_interval_min`〜`talk_interval_max` の範囲の乱数秒を足した時刻）を決めるだけで、何も発行しない。
- 現在時刻が次のトーク時刻に達していても、次の正時までの残りが `hour_margin` 秒未満のときは発行しない（時報を優先する）。
- 発行するときは、次のトーク時刻を決め直してから、チェイントークの続きか新しい `OnTalk` のシーンを返す。次のトーク時刻は、シーンが見つかったかどうかに関わらず決め直される。

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

### テスト用関数

仮想ディスパッチャの単体テストのための関数である。

```lua
-- 状態のリセット（セッション開始時と同じ状態にする。シーン実行関数の差し替えも解除する）
dispatcher._reset()

-- 内部状態の取得
local state = dispatcher._get_internal_state()
-- { next_hour_unix = 次の正時の Unix 時刻, next_talk_time = 次のトーク時刻 }

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

これでイベントの受け止め方は一通りですわ。シーンに任せるのが基本、細かな捌きは REG で、時報とランダムトークは仮想ディスパッチャにお任せなさいまし。
フンッ、既定ハンドラを不用意に上書きして時報が鳴らなくなっても、わたくしのせいではありませんわよ？ 迷ったら、この章へ戻っていらっしゃい。
