<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta マニュアル「@pasta_sakura_script」（https://ekicyou.github.io/pasta/lua/modules/pasta-sakura-script.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# @pasta_sakura_script

`@pasta_sakura_script` は、台詞のテキストにウェイトタグ（`\_w[ミリ秒]`）を挿入してさくらスクリプトへ変換するモジュールである。budoux による日本語の分かち書きを使った自動改行（`\n` の挿入）も提供する。

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"
```

**モジュールのメタデータ**: `_VERSION = "1.0.0"`、`_DESCRIPTION = "Sakura Script wait insertion module for natural conversation tempo"`

## act:talk との関係

アクション行の台詞や、シーン関数で `act:talk(...)` に渡したテキストは、さくらスクリプトを組み立てるときに、発言したアクターの表を引数にしてこのモジュールの `talk_to_script` で変換される。アクターの表は pasta.toml の `[actor."名前"]` に書いた値を持つ（`CONFIG.actor["名前"]` と同じ表）。アクション行に登録していないアクター名を書いた行は、名前だけを持つ表で変換されるため、`[talk]` の値と実装の既定値を使い、自動改行はしない（[登録していないアクター名](https://ekicyou.github.io/pasta/grammar/action-line.html#登録していないアクター名)）。

ふつうのゴーストでは、このモジュールを直接 `require` する必要はない。ウェイトの長さや自動改行は、pasta.toml の `[talk]` セクションと `[actor."名前"]` のキーで調整する（[pasta.toml での設定](#pastatoml-での設定)・[pasta.toml での budoux 設定](#pastatoml-での-budoux-設定)）。

## talk_to_script(actor, talk)

テキストにウェイトタグを挿入して返す。アクターの表に `budoux` があれば、続けて自動改行も行う。

```lua
SAKURA_SCRIPT.talk_to_script(actor, talk) -> string
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `actor` | table \| nil | ✅ | アクターの表。ウェイトの設定（[アクター表のウェイト設定](#アクター表のウェイト設定)）と `budoux` をこの表の直下から読む。表でない値（`nil` など）を渡すと、`[talk]` の値と実装の既定値を使い、自動改行はしない |
| `talk` | string \| nil | ✅ | 変換する台詞のテキスト |

**戻り値**: 変換後の文字列。

- `talk` が `nil` または `""` のときは `""` を返す。
- `talk` に数値を渡すと文字列にしてから変換する。表など文字列にできない値を渡すと Lua のエラーになる。

## アクター表のウェイト設定

ウェイトの長さ（ミリ秒）は、アクター表の直下にある次のキーで決まる。キーごとに「アクター表の値 → pasta.toml の `[talk]` の値 → 実装の既定値」の順で最初に見つかった値を使う。

| キー | 対象の文字 | 既定値 |
| ---- | ---- | ---- |
| `script_wait_normal` | 一般の文字（下の分類に当たらない文字） | `50` |
| `script_wait_period` | 句点 | `1000` |
| `script_wait_comma` | 読点 | `500` |
| `script_wait_strong` | 感嘆符・疑問符 | `500` |
| `script_wait_leader` | リーダー（`…` など） | `200` |

- 既定値は実装の既定値である。
- アクター表の値が数値（数値に変換できる文字列を含む）でないキーは、無いものとして扱う。小数は整数に切り捨てる。
- 実際に挿入されるウェイトは、設定値から 50 を引いた値（`値 - 50`）である。計算した値が 0 以下のときは何も挿入しない。既定の `script_wait_normal = 50` では、一般の文字の後ろにウェイトは入らない。
- どの文字を句点・読点などとみなすかは、アクター表では変えられない。pasta.toml の `[talk]` で変える（[pasta.toml での設定](#pastatoml-での設定)）。

## 動作仕様

### 文字の分類

テキストを先頭から読み、次の順で分類する。

1. **さくらスクリプトタグ**: `\` に英数字・`_`・`!`・`+`・`*`・`?`・`&`・`-` が 1 文字以上続き、その後ろに `[…]` が続いてもよい形（`\h`・`\s[5]`・`\_w[100]`・`\![open,inputbox]` など）と、`\` を表示するエスケープ `\\`。そのまま出力し、ウェイトは入れない。`\\` は 2 文字で 1 つのタグとして読み、後ろの文字とつなげない（`\\n` は `\\` と `n` で、改行タグ `\n` にはならない）。アクション行の `\\` はこの形で出力される（[インライン要素](https://ekicyou.github.io/pasta/grammar/action-line.html#インライン要素)）。
2. それ以外の文字は 1 文字ずつ、次の文字の集合に含まれるかを上から順に調べる（複数の集合に含まれる文字は、先に当たった分類になる）。

| 分類 | 文字の集合（`[talk]` のキー） | 既定の文字 |
| ---- | ---- | ---- |
| 句点 | `chars_period` | `｡。．.` |
| 読点 | `chars_comma` | `、，,` |
| 感嘆符・疑問符 | `chars_strong` | `？！!?` |
| リーダー | `chars_leader` | `･・‥…` |
| 行頭禁則文字 | `chars_line_start_prohibited` | `゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ` |
| 行末禁則文字 | `chars_line_end_prohibited` | `（［｛「『([{｢` |
| 一般の文字 | （上のどれにも当たらない文字） | — |

### ウェイトの挿入

- **一般の文字**: 1 文字ごとに、その後ろへ `script_wait_normal - 50` のウェイトを入れる。
- **リーダー**: 1 文字ごとに、その後ろへ `script_wait_leader - 50` のウェイトを入れる。
- **句点・読点・感嘆符・疑問符・行頭禁則文字**: 連続する並び（`」！？。` など）をひとまとめにし、並びの最後に 1 回だけウェイトを入れる。値は並びの中の文字の設定値の最大値から 50 を引いた値である。行頭禁則文字は並びを延ばすだけで、自分の値を持たない（行頭禁則文字だけの並びにはウェイトが入らない）。
- **行末禁則文字**: ウェイトを入れない。直前の並びはそこで終わる。
- **さくらスクリプトタグ**: ウェイトを入れない。直前の並びはそこで終わる。

## 使用例

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"

-- 既定値（一般の文字 50 → ウェイトなし、句点 1000 → 950）
SAKURA_SCRIPT.talk_to_script(nil, "こんにちは。")
-- → こんにちは。\_w[950]

-- アクター表の直下でウェイトを変える
local actor = { script_wait_normal = 100 }
SAKURA_SCRIPT.talk_to_script(actor, "こんにちは。")
-- → こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]。\_w[950]

local actor2 = { script_wait_normal = 80, script_wait_period = 200 }
SAKURA_SCRIPT.talk_to_script(actor2, "やあ。")
-- → や\_w[30]あ\_w[30]。\_w[150]

-- さくらスクリプトタグはそのまま残る
SAKURA_SCRIPT.talk_to_script(actor, "こんにちは\\s[5]元気？")
-- → こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]\s[5]元\_w[50]気\_w[50]？\_w[450]

-- 連続する句読点は最大値（感嘆符・疑問符 500 → 450）
SAKURA_SCRIPT.talk_to_script(nil, "え！？")
-- → え！？\_w[450]

-- 読点・句点・行頭禁則文字の並び（句点 1000 が最大 → 950）
SAKURA_SCRIPT.talk_to_script(nil, "あ、」。")
-- → あ、」。\_w[950]

-- リーダーは 1 文字ごと（200 → 150）
SAKURA_SCRIPT.talk_to_script(nil, "そう…なの？」")
-- → そう…\_w[150]なの？」\_w[450]
```

## pasta.toml での設定

`[talk]` セクションで、全アクター共通のウェイトの長さと文字の集合を設定する。書かなかったキーは実装の既定値になる。アクター表（`[actor."名前"]`）に同じ名前のウェイトのキーを書くと、そのアクターではアクター表の値が優先される。

```toml
[talk]
# ウェイト（ミリ秒）
script_wait_normal = 50
script_wait_period = 1000
script_wait_comma = 500
script_wait_strong = 500
script_wait_leader = 200

# 文字の集合
chars_period = "｡。．."
chars_comma = "、，,"
chars_strong = "？！!?"
chars_leader = "･・‥…"
chars_line_start_prohibited = "゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ"
chars_line_end_prohibited = "（［｛「『([{｢"
```

上の値はすべて実装の既定値である。アクターごとに句点の間だけ変える例:

```toml
[talk]
script_wait_normal = 100

[actor."さくら"]
spot = 0
script_wait_period = 300

[actor."うにゅう"]
spot = 1
```

この設定で、さくらの「やあ。」は `や\_w[50]あ\_w[50]。\_w[250]`、うにゅうの「やあ。」は `や\_w[50]あ\_w[50]。\_w[950]` になる。

## break_lines(text, widths)

budoux の日本語分割モデルで求めた語の区切り位置に、さくらスクリプトの改行タグ `\n` を挿入する。

```lua
SAKURA_SCRIPT.break_lines(text, widths) -> string
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `text` | string \| nil | ✅ | 改行を入れるテキスト。`nil` または `""` のときは `""` を返す |
| `widths` | table \| nil | ✅ | 行ごとの幅の上限の配列。`nil`・表でない値・空の表のときは `text` をそのまま返す |

**widths の仕様**:

- `widths[1]` が 1 行目、`widths[2]` が 2 行目の幅の上限である。配列の最後の値が、それ以降の行すべてに使われる（`{10, 12}` なら 1 行目は 10 まで、2 行目以降は 12 まで）。
- 幅は半角 1 文字を 1、全角 1 文字を 2 と数える（東アジアの文字幅で幅が曖昧な文字も 2）。`10` は全角 5 文字分である。
- 要素は数値で書く。小数は整数に切り捨てる。数値でない要素があると Lua のエラーになる。

**改行の入れ方**:

- 語を先頭から順に行へ積み、次の語を足すと幅の上限を超えるときに、その語の前へ `\n` を入れて次の行へ移る。
- 1 語だけで上限を超える場合は、語の途中で分割しない（その行は上限を超える）。
- さくらスクリプトタグ（`\_w[50]`・`\\` など）は幅に数えず、元の文字との位置関係を保ったまま出力する。文字の直後にあるタグはその文字の側に残り、`\n` はタグの後ろに入る。
- テキストに元からある `\n` は行の区切りとして数えない。

通常は `talk_to_script` と組み合わせて呼ぶ必要はない。アクター表に `budoux` があれば、`talk_to_script` がウェイトの挿入後に自動で `break_lines` を適用する。`break_lines` を直接呼ぶのは、独自の変換手順を組む場合である。

## pasta.toml での budoux 設定

アクター表に `budoux` を書くと、そのアクターの台詞に自動改行が入る。値は `break_lines` の `widths` と同じ配列である。

```toml
[actor."女の子"]
spot = 0
budoux = [10, 12]
# 1 行目は幅 10（全角 5 文字）まで、2 行目以降は幅 12（全角 6 文字）まで
```

`budoux` を書かないアクター、または空の配列を書いたアクターには自動改行は入らない。

## 使用例（直接呼び出し）

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"

SAKURA_SCRIPT.break_lines("今日はいい天気ですね", {6})
-- → 今日は\nいい\n天気ですね

SAKURA_SCRIPT.break_lines("今日はいい天気ですね", {10})
-- → 今日はいい\n天気ですね

-- さくらスクリプトタグは幅に数えず、出力に残る
SAKURA_SCRIPT.break_lines("こ\\_w[50]れ\\_w[50]は\\_w[50]テ\\_w[50]ス\\_w[50]ト", {6})
-- → こ\_w[50]れ\_w[50]は\_w[50]\nテ\_w[50]ス\_w[50]ト

-- nil・空の表は安全
SAKURA_SCRIPT.break_lines(nil, {10})    -- → ""
SAKURA_SCRIPT.break_lines("テスト", {})  -- → "テスト"（変更なし）
SAKURA_SCRIPT.break_lines("テスト", nil) -- → "テスト"（変更なし）

-- talk_to_script 経由の自動適用（アクター表に budoux がある場合）
local ok, CONFIG = pcall(require, "@pasta_config")
local actor = ok and CONFIG.actor and CONFIG.actor["女の子"]  -- budoux = [10, 12] を書いたアクター
SAKURA_SCRIPT.talk_to_script(actor, "今日はいい天気ですね。")
-- → 今日はいい\n天気ですね。\_w[950]（ウェイトを挿入した後で改行が入る）
```

改行の位置は budoux の分割結果に依存する。上の出力は現行の分割モデルでの結果である。
