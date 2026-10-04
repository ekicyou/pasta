# scripts/ の記述パターン

道具の名前を覚えたら、次は使いこなしですわ。ここでは、実際の `scripts/` でよく出てくる
書き方の「型」を集めましたの。シーン関数の定型、イベントの受け止め方、単語の一括投入――
どれも一度覚えれば一生もの。型をなぞるだけで、あなたのゴーストはぐっと賢くなりますわよ。

---

この章では `scripts/` 配下のカスタム Lua スクリプト、および Pasta DSL 内の ` ```lua ``` ` ブロックで
頻出する記述パターンを示す。対象方言は **LuaJIT 2.1**（Lua 5.1 系）である。

## scripts/ の役割と優先順位

ゴーストディレクトリ直下の `scripts/` にユーザーカスタム Lua スクリプトを置く。`main.lua` が
エントリーポイントとしてランタイムに読み込まれる。

- `scripts/`（ゴーストカスタム）は `pasta_scripts/`（エンジン標準ランタイム）より**優先**される。
  同名ファイルを置けばエンジン動作を上書きできる。
- `pasta_scripts/` は pasta.dll が起動時に自己展開するエンジン同梱スクリプトで、通常は触らない。

## パターン 1: シーン関数の定型

すべてのシーン関数は `act:init_scene(SCENE)` で始まる。これが必須の定型である。
戻り値の `save`（セッション間で永続するデータ）と `var`（この ACT の間だけ有効な一時変数）を受け取る。

```lua
function SCENE.挨拶(act)
    local save, var = act:init_scene(SCENE)  -- 必須: save / var を取得
    act:talk(act.ぱすた.actor, "こんにちは！")  -- アクター名でトーク
end
```

各メソッドの引数・戻り値は [スクリプト用ランタイム API](script-api.md#act) を参照する。シーン関数が終わると、積んだトークンは自動で組み立てられて出力される。そのため、シーン関数の最後に `act:yield()` は書かない（最後に置くと、シーンが中断したまま残り、次の OnTalk の機会に何も出力せずに終わる。[yield](script-api.md#yield)）。

`save` は `@pasta_persistence` 管理の永続変数で、セッションをまたいで保持される。`var` は ACT ごとの一時変数で、Call で呼んだ先のシーンやチェイントークの続きとも共有される（[ACT のフィールド](script-api.md#act-のフィールド)）。

```lua
function SCENE.カウント(act)
    local save, var = act:init_scene(SCENE)
    save.count = (save.count or 0) + 1      -- セッション間で累積
    var.temp = "一時データ"                  -- この ACT の間だけ有効
    act:talk(act.ぱすた.actor, save.count .. "回目ですね")
end
```

### 複数トークと表示制御

`act` のメソッドはチェーンできる。途中で `act:yield()` を呼ぶと、そこまでを 1 回の出力として区切る。

```lua
function SCENE.物語(act)
    local save, var = act:init_scene(SCENE)
    act:talk(act.ぱすた.actor, "最初のセリフ")
    act:yield()  -- ここで一区切り
    act:talk(act.ぱすた.actor, "えっ")
    act:surface(5):wait(500):talk(act.ぱすた.actor, "驚いた！"):newline()
    -- 残りはシーンの終了時に出力される
end
```

表示制御メソッド（`surface`・`wait`・`newline`・`clear`）は [表示制御](script-api.md#表示制御) を参照する。1 回の出力（`yield` またはシーンの終了で区切られる範囲）の中で、まだ `talk` を積んでいないうちに積んだ表示制御は出力されない。`yield` の直後は、先に `talk` を積んでから表示制御を続ける。

### 選択肢

```lua
function SCENE.分岐(act)
    local save, var = act:init_scene(SCENE)
    act:talk(act.ぱすた.actor, "どうする？")
    act:choice("挨拶", "挨拶する")   -- ジャンプ先シーン名, 表示テキスト
    act:choice("自己紹介")           -- 表示テキスト省略時はシーン名を表示
    act:choice_timeout(30)           -- 30秒でタイムアウト
end
```

選択肢が選ばれると、ランタイムが `OnChoiceSelectEx` を発火し、選択 ID（`choice` の第1引数）を
シーン名として前方一致検索して該当シーンを実行する。通常はこの自動ルーティングに任せればよい（[choice と choice_timeout](script-api.md#choice-と-choice_timeout)・[OnChoiceSelectEx](shiori-events.md#onchoiceselectex)）。

## パターン 2: イベントハンドラの登録

カスタム SHIORI イベント処理は `REG` テーブルにハンドラを登録する。ハンドラは `function(act)` の形で、
`Value` にする文字列（または `RES` で作った応答全体）を返す。詳細は [SHIORI イベントとハンドラ](shiori-events.md#reg) を参照。

```lua
local REG = require("pasta.shiori.event.register")
local SCENE = require("pasta.scene")

REG.OnClose = function(act)
    local reason = act.req.reference[0]  -- Reference0: 終了の理由
    if reason == "user" then
        return "\\0\\s[0]またね。\\-"
    end
    return "\\0\\s[0]終了します。\\-"
end

REG.OnMouseDoubleClick = function(act)
    -- シーン「なでられ」のコルーチンを返す（見つからなければ nil → 204 No Content）
    return SCENE.co_exec(act, "なでられ")
end
```

ハンドラの戻り値は、エンジンが SHIORI の応答にする。

| 戻り値 | 応答 |
| ---- | ---- |
| 文字列（`SHIORI/` で始まらない） | 200 OK。その文字列が `Value` になる（空文字列なら 204 No Content） |
| `SHIORI/` で始まる文字列 | 応答全体として扱い、包まずにそのまま返す |
| シーンのコルーチン | コルーチンを実行し、出力を `Value` にして 200 OK（出力が無ければ 204 No Content。出力が `SHIORI/` で始まっても `Value` になる） |
| `nil`（何も返さない） | 204 No Content（表示なし） |

応答文字列は、エンジンが `RES`（`pasta.shiori.res`）で組み立てる。ハンドラは `Value` にする文字列を返すほか、`RES` で作った応答全体（`SHIORI/` で始まる文字列）を返してもよい（[RES](shiori-events.md#res)）。

リクエストの内容は `act.req` で読む。主なフィールド:

| フィールド | 説明 |
| ---- | ---- |
| `act.req.id` | イベント名（`"OnBoot"` 等） |
| `act.req.reference[N]` | Reference ヘッダ（0 始まり）。未送信時は `nil` |
| `act.req.date` | リクエストを受けた時点の日時（`year`・`hour` 等のフィールドを持つ表） |
| `act.req.status` | `Status` ヘッダの値（`"talking,balloon(0=0)"` のようなカンマ区切りの文字列） |

全フィールドは [act.req](shiori-events.md#actreq) を参照。

DSL のシーンから `Reference` や日時を読むときは、Lua を書かずに `＄ｒ０` や `＄時１２` を使える（[エンジンが値を入れる変数](../grammar/variables.md#エンジンが値を入れる変数)を参照）。

`OnBoot`・`OnChoiceSelectEx`・`OnSecondChange` には pasta の既定ハンドラがある。`scripts/main.lua` でこれらを上書きするときは、先に既定ハンドラを登録させる手順が要る（[既定ハンドラと上書き](shiori-events.md#既定ハンドラと上書き)）。

### REG 未登録時のフォールバック

`REG` にハンドラが無いイベントは、同名のグローバルシーンが自動的に検索・実行される。
つまり DSL で `＊OnBoot` シーンを定義しておけば、`REG.OnBoot` を書かなくても起動時に呼ばれる。
凝った分岐が要るときだけ `REG` でハンドラを書く、というのが基本方針である
（[シーン関数フォールバック](shiori-events.md#シーン関数フォールバック)）。

`OnTalk`（ランダムトーク）と `OnHour`（時報）は、ランタイムの仮想ディスパッチャが
`OnSecondChange` を起点に自動発行する。これらも対応するシーンを定義しておけば呼ばれる。

## パターン 3: 単語の一括投入

数十〜数百件の単語を投入したいときは、DSL の単語定義より Lua のループが向く。
`pasta.word` モジュールのビルダーパターンを使う。

```lua
local WORD = require("pasta.word")

-- グローバル単語: ループで一括投入
local foods = { "ラーメン", "カレー", "寿司", "焼肉", "パスタ" }
local builder = WORD.create_global("好きな食べ物")
for _, food in ipairs(foods) do
    builder:entry(food)
end

-- メソッドチェーンでまとめて
WORD.create_global("挨拶")
    :entry("こんにちは", "やあ")
    :entry("ごきげんよう")

-- ローカル単語（グローバルシーンの登録名。1 つ目の ＊メイン なら "メイン1"）
WORD.create_local("メイン1", "返事")
    :entry("はい", "ええ")
    :entry("そうね")

-- アクター単語（アクター名スコープ）
WORD.create_actor("ぱすた", "一人称")
    :entry("わたし")
    :entry("あたし")
```

各ファクトリ関数のスコープと引数は [ファクトリ関数](script-api.md#ファクトリ関数) を参照する。

`entry(...)` は可変長引数で値を追加し、`self` を返すのでチェーンできる。外部 JSON / YAML を
`@json` / `@yaml` で読み込んでループ投入すれば、データ駆動の大規模辞書も構築できる。

## パターン 4: グローバル関数の定義

DSL から呼び出せるユーザー定義関数は `pasta.global`（`GLOBAL`）テーブルに登録する。

```lua
local GLOBAL = require("pasta.global")

GLOBAL.時報 = function(act)
    return os.date("%H") .. "時です"
end
-- DSL から呼び出し: ＠＊時報()  （グローバル関数は ＊ 付きで呼ぶ）
```

DSL の `＠関数名（）` は、名前を 5 段（実行中のシーンのシーンテーブル → ローカルシーン → act のメソッド → `GLOBAL` → グローバルシーン）で探して呼ぶ。`GLOBAL` は 4 段目のため、同じ名前のローカルシーンや act のメソッド（`talk`・`actor_proxy`・`global_fn`・`arith` など）があるとそちらが呼ばれる。`＠＊関数名（）` は検索せずに `GLOBAL` の関数だけを呼ぶ（生成コードは `act:global_fn("関数名")`。関数が無ければ警告ログを出して値なしになる）。グローバル関数を確実に呼ぶには `＊` を付ける（[検索と呼び出し](script-api.md#検索と呼び出し)・[GLOBAL](script-api.md#global)）。

---

型は出そろいましたわ。あとはこれらを組み合わせて、あなただけのゴーストを織り上げるだけ。
でも――Lua で書くべきか、DSL で済ませるべきか、迷う場面もございましょう？
次の章で、その見極め方をきっちりお教えいたしますわ。さあ、最後まで参りましょう！
