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
戻り値の `save`（セッション間で永続するデータ）と `var`（このアクション内だけの一時変数）を受け取る。

```lua
function SCENE.挨拶(act)
    local save, var = act:init_scene(SCENE)  -- 必須: save / var を取得
    act:talk(act.ぱすた.actor, "こんにちは！")  -- アクター名でトーク
    act:yield()                                 -- 蓄積トークンを送出
end
```

| 要素 | 役割 |
| ---- | ---- |
| `act:init_scene(SCENE)` | シーン初期化。`save`（永続）と `var`（一時）を返す。必須 |
| `act:talk(actor, text)` | アクターのセリフを出力（さくらスクリプトへ自動変換） |
| `act:yield()` | 蓄積したトークンを送出し、トークの区切りを作る |

`save` は `@pasta_persistence` 管理の永続変数で、セッションをまたいで保持される。`var` は
そのアクション実行中のみ有効な一時変数である。

```lua
function SCENE.カウント(act)
    local save, var = act:init_scene(SCENE)
    save.count = (save.count or 0) + 1      -- セッション間で累積
    var.temp = "一時データ"                  -- このアクション内のみ
    act:talk(act.ぱすた.actor, save.count .. "回目ですね")
    act:yield()
end
```

### 複数トークと表示制御

`act` のメソッドはチェーンできる。複数のセリフ区切りは `yield()` を複数回呼んで作る。

```lua
function SCENE.物語(act)
    local save, var = act:init_scene(SCENE)
    act:talk(act.ぱすた.actor, "最初のセリフ")
    act:yield()  -- ここで一区切り
    act:surface(5):wait(500):talk(act.ぱすた.actor, "驚いた！"):newline()
    act:yield()  -- 2区切り目
end
```

主な表示制御メソッド: `surface(id)`（サーフェス変更）、`wait(ms)`（ウェイト）、`newline(n?)`（改行）、
`clear()`（表示クリア）。いずれも `self` を返すのでチェーン可能。

### 選択肢

```lua
function SCENE.分岐(act)
    local save, var = act:init_scene(SCENE)
    act:talk(act.ぱすた.actor, "どうする？")
    act:choice("挨拶", "挨拶する")   -- ジャンプ先シーン名, 表示テキスト
    act:choice("自己紹介")           -- 表示テキスト省略時はシーン名を表示
    act:choice_timeout(30)           -- 30秒でタイムアウト
    act:yield()
end
```

選択肢が選ばれると、ランタイムが `OnChoiceSelectEx` を発火し、選択 ID（`choice` の第1引数）を
シーン名として前方一致検索して該当シーンを実行する。通常はこの自動ルーティングに任せればよい。

## パターン 2: イベントハンドラの登録

カスタム SHIORI イベント処理は `REG` テーブルにハンドラを登録し、`RES` でレスポンスを返す。

```lua
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

REG.OnBoot = function(req)
    local shell_name = req.reference[0]  -- Reference0
    return RES.ok("\\h\\s[0]起動しました。\\e")
end

REG.OnClose = function(req)
    local reason = req.reference[0]
    if reason == "user" then
        return RES.ok("\\h\\s[0]またね。\\e")
    end
    return RES.no_content()  -- 表示なしで処理完了
end
```

ハンドラは `req`（SHIORI リクエスト情報）を受け取る。主なフィールド:

| フィールド | 説明 |
| ---- | ---- |
| `req.id` | イベント名（`"OnBoot"` 等） |
| `req.reference[N]` | Reference ヘッダ（0 始まり）。未送信時は `nil` |
| `req.date` | 日時情報 |
| `req.status` | ステータス（`"talking"` 等） |

レスポンス生成 API:

| 関数 | 説明 |
| ---- | ---- |
| `RES.ok(value)` | 200 OK + さくらスクリプト |
| `RES.ok_with(headers)` | 200 OK + 複数ヘッダ |
| `RES.no_content()` | 204 No Content（表示なし） |
| `RES.err(message)` | 500 エラー |

### REG 未登録時のフォールバック

`REG` にハンドラが無いイベントは、同名のグローバルシーンが自動的に検索・実行される。
つまり DSL で `＊OnBoot` シーンを定義しておけば、`REG.OnBoot` を書かなくても起動時に呼ばれる。
凝った分岐が要るときだけ `REG` でハンドラを書く、というのが基本方針である。

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

-- ローカル単語（シーン名スコープ）
WORD.create_local("メイン_1", "返事")
    :entry("はい", "ええ")
    :entry("そうね")

-- アクター単語（アクター名スコープ）
WORD.create_actor("ぱすた", "一人称")
    :entry("わたし")
    :entry("あたし")
```

| ファクトリ関数 | スコープ |
| ---- | ---- |
| `WORD.create_global(key)` | グローバル |
| `WORD.create_local(scene_name, key)` | ローカル（指定シーン内） |
| `WORD.create_actor(actor_name, key)` | アクター |

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

DSL の `＠関数名()` はローカル（`SCENE.`）呼び出し、`＠＊関数名()` がグローバル（`GLOBAL.`）呼び出しである点に注意する。

---

型は出そろいましたわ。あとはこれらを組み合わせて、あなただけのゴーストを織り上げるだけ。
でも――Lua で書くべきか、DSL で済ませるべきか、迷う場面もございましょう？
次の章で、その見極め方をきっちりお教えいたしますわ。さあ、最後まで参りましょう！
