# pasta.toml リファレンス

ゴーストの性格を決めるのが辞書なら、その暮らし方を決めるのが `pasta.toml` ですわ。
わたくしが全セクション・全キーを、ひとつ残らず並べて差し上げますの。
とはいえ、書かねばならないのは `[actor]` だけ。あとは必要になったときに、この頁を引けばよろしくてよ。

---

`pasta.toml` は、ゴーストの設置ディレクトリ（`ghost/master/`。`pasta.dll` と同じ場所）に置く設定ファイルである。このページは全セクション・全キーの型・既定値・用途を示す。

## 概要

`pasta.toml` は、ゴーストの読み込みの最初に読まれる。省略したセクション・キーには実装の既定値（SSOT）が使われ、明示した値は既定値で上書きされない。`[ghost]` の 4 キーは、書かなかったものが読み込み時に既定値で補完されるため、Lua から `@pasta_config` で読んでも同じ値になる（[@pasta_config](../lua/modules/pasta-config.md)）。

- `pasta.toml` ファイル自体は必須である。ファイルが無い場合、または TOML として読めない場合、ゴーストの読み込みは失敗する。最小化できるのは記述量であって、ファイルの存在ではない。
- 以下の表の「既定値」は、いずれも実装の既定値（SSOT）である。

### プロファイルと3分類

既定値は、用途ごとの 2 系統を概念として持つ。伺かゴーストとして動かすための **SHIORI プロファイル**と、ノベルゲーム・ツールなど SHIORI 以外の用途を想定した**エンジンプロファイル**である。このページが定める既定値は SHIORI プロファイルのものだけで、エンジンプロファイルの既定値は定められていない。

各セクション・各キーは、次の 3 分類のどれか 1 つに属する（重複しない）。

1. **SHIORI デフォルト有（省略可）** — 省略すると既定値が使われる。最小構成では書く必要がない。
2. **必須（デフォルト不能）** — ゴースト固有の値で既定値を決められないため、作者が書く。`[actor]`（1 つ以上）と、その `spot` が該当する。
3. **エンジンプロファイル専用** — SHIORI 用途では使われず、書く必要もない。`[package]` が該当する。

### 値の型が合わないとき

キーに型の合わない値（整数のキーに文字列など）を書いたときの扱いは、セクションによって異なる。

| セクション | 型の合わない値があるとき |
| ---------- | ------------------------ |
| `[loader]` | `pasta.toml` の読み込みエラーになり、ゴーストの読み込みが失敗する |
| `[talk]`・`[persistence]`・`[logging]`・`[debug]` | そのセクション全体を書かなかったときと同じ扱いになる（同じセクションの正しく書いたキーも含め、すべて既定値になる）。エラーも警告も出ない |

例として、`[talk]` に `script_wait_period = 300` と `script_wait_normal = "x"` を並べて書くと、`script_wait_period` も既定値（`1000`）になる。設定が効いていないと感じたら、同じセクションの他のキーの型を確かめる。

## 3分類表

全セクション・代表キーの分類と、SHIORI プロファイルの既定値を示す。既定値は実装の既定値（SSOT）である。

| セクション / キー | 分類 | SHIORI デフォルト |
| ----------------- | ---- | ----------------- |
| `[actor."名前"]`（1 つ以上） | **必須（デフォルト不能）** | （なし。作者が必ず書く） |
| `[actor]` › `spot` | 必須（デフォルト不能） | （なし。ゴースト固有） |
| `[actor]` › `budoux` / `surface` / `dressup` | SHIORI デフォルト有（省略可） | （なし＝未設定） |
| `[loader]` | SHIORI デフォルト有（省略可） | （下記キー参照） |
| `[loader]` › `pasta_patterns` | SHIORI デフォルト有 | `["dic/**/*.pasta"]` |
| `[loader]` › `lua_search_paths` | SHIORI デフォルト有 | （[lua_search_paths](#lua_search_paths) 参照） |
| `[loader]` › `transpiled_output_dir` | SHIORI デフォルト有 | `"profile/pasta/cache/lua"` |
| `[loader]` › `debug_mode` | SHIORI デフォルト有 | `true` |
| `[ghost]` | SHIORI デフォルト有（省略可） | （下記キー参照） |
| `[ghost]` › `talk_interval_min` | SHIORI デフォルト有 | `180` |
| `[ghost]` › `talk_interval_max` | SHIORI デフォルト有 | `300` |
| `[ghost]` › `hour_margin` | SHIORI デフォルト有 | `30` |
| `[ghost]` › `spot_newlines` | SHIORI デフォルト有 | `1.5` |
| `[talk]` | SHIORI デフォルト有（省略可） | （[[talk]](#talkトーク表示制御) 参照） |
| `[persistence]` | SHIORI デフォルト有（省略可） | （[[persistence]](#persistence永続化) 参照） |
| `[logging]` | SHIORI デフォルト有（省略可） | （[[logging]](#loggingログ出力) 参照） |
| `[lua]` | SHIORI デフォルト有（省略可） | （[[lua]](#lualua-ライブラリ) 参照） |
| `[debug]` | SHIORI デフォルト有（省略可） | `enabled = false` / `port = 9276` |
| `[package]` | **エンジンプロファイル専用** | （SHIORI では不要。[予約注記](#package-予約注記) 参照） |

分類は一意で、同じセクション・キーが複数の分類に属することはない。

## 最小テンプレート

SHIORI として起動するために必須なのは `[actor]` だけである。他のセクションはすべて省略でき、既定値が使われる。`[package]`・`[loader]` を書く必要はない。

```toml
# 最小構成: 必須の [actor] のみ。他は SHIORI デフォルトが使われる。
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

- `"名前"` は、辞書（`.pasta`）の会話行・アクター辞書で使うアクター名と一致させる。`descript.txt` の `sakura.name`・`kero.name` とそろえておくと対応が分かりやすい。
- `spot` はゴースト固有で既定値が無いため、アクターごとに書く（`0`=sakura 側、`1`=kero 側）。
- `dic/` 配下に置いた辞書は、`[loader]` を書かなくても `pasta_patterns` の既定値で読み込まれる（[pasta_patterns](#pasta_patterns)）。

`[actor]` を 1 つも書かなかった場合も起動は止まらないが、SHIORI として正しく動かない可能性があるため、警告がログに 1 回出る。

## フルリファレンステンプレート

全セクション・全キーを、分類と既定値の注記付きで並べたテンプレートである。値は、書いてある限り既定値を示す（必要な項目だけ抜き出して使う）。

```toml
# ============================================================
# pasta.toml フルリファレンステンプレート
# 各行の注記 = 分類 / SHIORI デフォルト値（実装の既定値）
# 「省略可」のセクションは丸ごと削除しても既定値が使われる。
# ============================================================

# --- 必須（デフォルト不能）: 最小構成で唯一必須 ---
[actor."女の子"]
spot = 0                # 必須（デフォルト不能）: 0=sakura側 / 1=kero側
budoux = [10, 12]       # 省略可: 未設定=自動改行なし
surface = 0             # 省略可: 未設定=既定サーフェスなし

[actor."男の子"]
spot = 1                # 必須（デフォルト不能）

# --- 省略可（SHIORI デフォルト有）: ファイル読み込み ---
[loader]
pasta_patterns = ["dic/**/*.pasta"]            # 既定 ["dic/**/*.pasta"]
lua_search_paths = [                            # 既定（優先順位順）:
  "profile/pasta/save/lua",                     #   保存領域の Lua
  "scripts",                                    #   ゴースト作者のスクリプト
  "profile/pasta/pasta_scripts",                #   pasta の内蔵スクリプト
  "profile/pasta/cache/lua",                    #   トランスパイル済みキャッシュ
  "scriptlibs",                                 #   スクリプトライブラリ
]
transpiled_output_dir = "profile/pasta/cache/lua"  # 既定 "profile/pasta/cache/lua"
debug_mode = true                                  # 既定 true

# --- 省略可（SHIORI デフォルト有）: ゴースト動作 ---
[ghost]
talk_interval_min = 180   # 既定 180（秒）
talk_interval_max = 300   # 既定 300（秒）
hour_margin = 30          # 既定 30（秒）
spot_newlines = 1.5       # 既定 1.5

# --- 省略可（SHIORI デフォルト有）: トーク表示制御 ---
[talk]
script_wait_normal = 50     # 既定 50（ミリ秒）
script_wait_period = 1000   # 既定 1000（ミリ秒）
script_wait_comma = 500     # 既定 500（ミリ秒）
script_wait_strong = 500    # 既定 500（ミリ秒）
script_wait_leader = 200    # 既定 200（ミリ秒）
chars_period = "｡。．."      # 既定 "｡。．."
chars_comma = "、，,"        # 既定 "、，,"
chars_strong = "？！!?"      # 既定 "？！!?"
chars_leader = "･・‥…"       # 既定 "･・‥…"
chars_line_start_prohibited = "゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ"  # 既定（行頭禁則文字）
chars_line_end_prohibited = "（［｛「『([{｢"                           # 既定（行末禁則文字）

# --- 省略可（SHIORI デフォルト有）: 永続化 ---
[persistence]
obfuscate = false                                   # 既定 false
file_path = "profile/pasta/save/save.json"          # 既定 "profile/pasta/save/save.json"
debug_mode = false                                  # 既定 false

# --- 省略可（SHIORI デフォルト有）: ログ出力 ---
[logging]
file_path = "profile/pasta/logs/pasta.log"   # 既定 "profile/pasta/logs/pasta.log"
level = "info"                               # 既定 "info"
# filter = "debug,pasta_shiori=info"         # 未設定（設定時は level より優先）

# --- 省略可（SHIORI デフォルト有）: Lua ライブラリ ---
# [lua] は上級者向け。「[lua]（Lua ライブラリ）」の節を参照。

# --- 省略可（SHIORI デフォルト有）: デバッグバックエンド ---
[debug]
enabled = false             # 既定 false
port = 9276                 # 既定 9276
# present_as = "lua"        # 未設定（既定 .pasta）
source_map_sidecar = false  # 既定 false

# --- エンジンプロファイル専用: SHIORI 用途では記述不要（[package] 予約注記 参照） ---
# [package] は SHIORI では使われない。書いても無視される。
```

## [package] 予約注記

`[package]`（`name`・`version`・`edition`）は**エンジンプロファイル専用**に分類される。

- **SHIORI 用途では不要**: 伺かゴーストでは `install.txt`・`readme.txt` などでメタデータを管理できるため、`[package]` を書く必要はない。最小テンプレートにも、配布しているサンプルゴースト（hello-pasta）の設定にも含めていない。
- **書いても無視される**: `[package]` を含む `pasta.toml` も、エラーや警告を出さずにこれまでどおり起動する（後方互換）。エンジンは `[package]` の値を使わない。他のセクションと同じく `@pasta_config` からは読める。
- **予約**: エンジンプロファイルでの `[package]` の既定値と用途は定められておらず、予約されている。

| キー | 型 | 説明 |
| ---- | -- | ---- |
| `name` | 文字列 | パッケージ名（エンジンプロファイル専用） |
| `version` | 文字列 | セマンティックバージョン（エンジンプロファイル専用） |
| `edition` | 文字列 | エディション（例: `"2024"`。エンジンプロファイル専用） |

## 各セクション詳細

ここから先は、各セクション・各キーの型・既定値・用途を示す。

### [loader]（ファイル読み込み）

辞書ファイルと Lua モジュールの読み込みを制御する。**分類: SHIORI デフォルト有（省略可）**。`[loader]` は `@pasta_config` に含まれない。

| キー | 型 | 既定値 | 説明 |
| ---- | -- | ------ | ---- |
| `pasta_patterns` | 文字列の配列 | `["dic/**/*.pasta"]` | 読み込む `.pasta` ファイルの glob パターン |
| `lua_search_paths` | 文字列の配列 | （[lua_search_paths](#lua_search_paths) 参照） | Lua モジュールの検索パス（優先順位順） |
| `transpiled_output_dir` | 文字列 | `"profile/pasta/cache/lua"` | トランスパイルした Lua（キャッシュ）の出力先 |
| `debug_mode` | 真偽値 | `true` | `true` のとき、読み込み時の処理件数（トランスパイル・スキップ・失敗・コピー）を info ログに出す |

パスはすべて設置ディレクトリからの相対パスで書く。トランスパイル結果のキャッシュへの保存は、`debug_mode` の値に関わらず行われる。元のファイルが無くなったキャッシュ（孤立キャッシュ）は、`debug_mode` の値に関わらず warn ログに出る（削除はしない）。`true` のときは各パスがもう一度出る。

#### pasta_patterns

辞書ファイル（`.pasta`）を探す glob パターンの配列である。設置ディレクトリからの相対パスで書き、どれかのパターンに一致した `.pasta` がすべて読み込まれる。既定の `["dic/**/*.pasta"]` は、`dic/` の直下とその下のすべての階層に一致する。慣例どおり `dic/` 配下に辞書を置けば、`[loader]` を書かずに起動できる。

`.pasta` で終わる各パターンからは、末尾を `.lua` に変えたパターン（既定では `dic/**/*.lua`）も作られ、それに一致した `.lua` ファイルも辞書モジュールとして読み込まれる。`.lua` ファイルはトランスパイルされずにそのまま読み込まれる。同じ場所に同じ名前の `.pasta` があるときは `.pasta` が優先され、`.lua` は警告を出して読み込まれない。`.pasta` で終わらないパターンからは `.lua` 用のパターンは作られない（警告が出る）。ファイル名が `init.lua` のものは `init.pasta` と同じく読み込みに失敗する。

```toml
[loader]
pasta_patterns = ["dic/**/*.pasta"]
```

- 辞書は役割ごとに複数のファイルへ分けられる（例: アクター辞書を `actors.pasta`、起動・終了イベントを `boot.pasta`、ランダムトークと時報を `talk.pasta`、マウス反応を `click.pasta`）。パターンに一致するファイルはすべて自動で読み込まれ、ファイル間の取り込み指定は要らない。グローバルなアクター辞書・単語・シーンは、別のファイルからも参照できる。
- `dic/*.pasta` のように `**` を使わないパターンは、`dic/` の直下だけに一致する（`dic/talk/a.pasta` は読み込まれない）。
- `..` を含むパターンは使われず、警告がログに出る。`profile/` 配下のファイルは、パターンに一致しても読み込まれない。
- 一致する `.pasta` が 1 つも無くても起動は止まらず、警告がログに出る。
- `init.pasta` という名前のファイルが一致すると、ゴーストの読み込みが失敗する。

#### lua_search_paths

Lua の `require` がモジュールを探すディレクトリの一覧である。前にあるものが優先される。既定値（優先順位順）:

1. `profile/pasta/save/lua` — 保存領域に置かれた Lua（最優先）
2. `scripts` — ゴースト作者が書くスクリプト
3. `profile/pasta/pasta_scripts` — pasta が起動時に展開する内蔵スクリプト
4. `profile/pasta/cache/lua` — `.pasta` をトランスパイルしたキャッシュ
5. `scriptlibs` — スクリプトライブラリ

各ディレクトリでのファイル名の探し方とモジュール解決の詳細は [モジュール検索パス](../reference/startup.md#1-モジュール検索パス) を参照する。

---

### [ghost]（ゴースト動作）

ゴーストの動作パラメータを設定する。**分類: SHIORI デフォルト有（省略可）**。`[ghost]` は書かなくても必ず `@pasta_config` に現れ、書かなかったキーには既定値が入る。表に無いキーを書いた場合も、そのまま `@pasta_config` に現れる。

| キー | 型 | 既定値 | 説明 |
| ---- | -- | ------ | ---- |
| `talk_interval_min` | 整数 | `180` | ランダムトーク（OnTalk）の最小間隔（秒） |
| `talk_interval_max` | 整数 | `300` | ランダムトーク（OnTalk）の最大間隔（秒） |
| `hour_margin` | 整数 | `30` | 次の正時までの残りがこの秒数未満のとき、ランダムトークを発行しない（時報を優先する）（秒） |
| `spot_newlines` | 数値 | `1.5` | 話し手の切り替えで台詞のあるバルーンへ戻るときの改行幅（`\n[値×100]`） |

#### talk_interval_min / talk_interval_max

ランダムトーク（OnTalk）の発火間隔を秒で指定する。次のトーク時刻は、この範囲の乱数秒で決まる。

```toml
[ghost]
talk_interval_min = 120
talk_interval_max = 240
```

- 実行中は、予約グローバル変数 `＄＊pasta_talk_interval_min`・`＄＊pasta_talk_interval_max` に入れた値が `pasta.toml` の値より優先される（[予約グローバル変数](../grammar/variables.md#予約グローバル変数pasta_-で始まる名前)）。
- 数値でない値の扱い、下限（10 秒）、最小間隔が最大間隔を上回ったときの扱いは [SHIORI イベントとハンドラの pasta.toml 設定](../lua/shiori-events.md#pastatoml-設定) を参照する。

#### spot_newlines

1 回のトークの出力（さくらスクリプト 1 本）の中で、話し手が切り替わり、すでに台詞を出したスポット（バルーン）へ戻るとき、`\p[スポット番号]` の直後（次の台詞の前）に改行 `\n[N]` を出力する。`N` は値を 100 倍して小数点以下を切り捨てた整数で、行の高さに対する百分率の改行幅である。既定の `1.5` では `\n[150]` になる。

```toml
[ghost]
spot_newlines = 2.0
```

```pasta
＊OnTalk
　さくら：やあ。
　うにゅう：はい。
　さくら：また。
```

この例の出力は、既定値では `\p[0]やあ。\_w[950]\p[1]はい。\_w[950]\p[0]\n[150]また。\_w[950]\e`、`spot_newlines = 2.0` では `\n[150]` の位置が `\n[200]` になる（`さくら` を spot 0、`うにゅう` を spot 1 とした場合）。

- そのスポットで初めて台詞を出すときは改行を出さない。
- `\c` でバルーンを消去した後、そのスポットで最初に出す台詞の前にも改行を出さない。

---

### [actor."名前"]（アクター設定）

アクターごとの設定である。`"名前"` は辞書で使うアクター名と一致させる（`descript.txt` の `sakura.name`・`kero.name` とそろえておくと分かりやすい）。複数のアクターを書ける。

**分類: 必須（デフォルト不能）**。SHIORI として動かすには `[actor]` が 1 つ以上必要で、`spot` はゴースト固有のため既定値が無い。`budoux`・`surface`・`dressup` はアクター内の省略可のキー（未設定が既定）である。

| キー | 型 | 既定値 | 分類 | 説明 |
| ---- | -- | ------ | ---- | ---- |
| `spot` | 整数 | （なし） | 必須（デフォルト不能） | バルーン位置（`0`=sakura 側、`1`=kero 側） |
| `budoux` | 整数の配列 | （なし） | 省略可 | BudouX による自動改行の幅 |
| `surface` | 整数 または 文字列 | （なし） | 省略可 | 既定サーフェス ID（同一スポット共有時の外見の復旧に使う） |
| `dressup` | テーブル（カテゴリ → パーツ → `0`/`1`） | （なし） | 省略可 | 既定の着せ替え（同一スポット共有時の外見の復旧に使う） |
| `script_wait_normal` ほか `script_wait_*` の 5 キー | 整数 | （なし＝`[talk]` の値） | 省略可 | このアクターだけのウェイト（[アクター表のウェイト設定](../lua/modules/pasta-sakura-script.md#アクター表のウェイト設定)） |

- 表に無いキーも書ける。アクター設定のキーと値は Lua から `@pasta_config` の `actor` で読め、各アクターにはキー名と同じ値の `name` が入る（[@pasta_config](../lua/modules/pasta-config.md)）。
- アクター設定のキーは、そのアクターの会話行で `＠キー名` を書いたときの単語検索で最優先される（[副作用: トーク中の ＠surface](../grammar/actor-dictionary.md#副作用-トーク中の-surface)）。

#### spot

台詞を出すバルーン（スコープ）を決める。`0` がメイン（sakura 側）、`1` がサブ（kero 側）、`2` 以上は 3 人目以降のキャラクターである。アクターごとに書く。

```toml
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

- シーンの `％` 行を実行すると、そのアクターの立ち位置は `％` 行の指定に置き換わる。
- `spot` も `％` 行の指定も無いアクターは、スポット 0 で話し、警告がログに出る。
- 立ち位置と出力の関係は [バルーン連携](../grammar/actor-dictionary.md#バルーン連携) を参照する。

#### budoux

BudouX による自動改行の幅を、配列 `[1 行目の幅, 2 行目以降の幅]` で指定する。設定すると、そのアクターの台詞に指定の幅で自動的に `\n` が入る。

- 幅は半角 1 文字を 1、全角 1 文字を 2 と数える（`10` は全角 5 文字分）。
- 要素が 1 つの場合: すべての行に同じ幅を使う（例: `budoux = [10]`）。
- 要素が 2 つの場合: 1 行目と 2 行目以降で別の幅を使う（例: `budoux = [10, 12]`）。配列の最後の値が、それ以降のすべての行に使われる。
- BudouX は日本語の自然な分かち書きの位置で改行するため、語の途中では改行しない（1 語だけで幅を超える行は、幅を超えたままになる）。
- `budoux` を書かないアクター、または空の配列を書いたアクターには自動改行が入らない。

```toml
[actor."女の子"]
spot = 0
budoux = [10, 12]   # 1 行目は幅 10（全角 5 文字）まで、2 行目以降は幅 12（全角 6 文字）まで

[actor."男の子"]
spot = 1
budoux = [10]       # すべての行を幅 10（全角 5 文字）まで
```

改行の入れ方の詳細は [pasta.toml での budoux 設定](../lua/modules/pasta-sakura-script.md#pastatoml-での-budoux-設定) と [break_lines(text, widths)](../lua/modules/pasta-sakura-script.md#break_linestext-widths) を参照する。

#### surface / dressup

複数のアクターが同じ `spot` を共有して交代で話すとき、pasta は切替先のアクターが最後に出したサーフェス・着せ替えを `\p[spot]` の直後へ自動で再出力（復旧）する。`surface` と `dressup` は、そのアクターがまだ一度もサーフェス変更・着せ替えを出していないときに使う既定値である。起動時に自動で適用されるのではなく、復旧が必要になったとき（専用スポットのアクターでは、起動・再読込後の最初の発話）に出力される。

- `surface`: サーフェス ID（数値またはエイリアスの文字列）。`\s[ID]` として出力される。
- `dressup`: カテゴリ → パーツ → `0`（外す）/`1`（着ける）の入れ子のテーブル。値は数値 `0`・`1` だけが有効で、それ以外は無視される。`\![bind-noevent,カテゴリ,パーツ,値]` として出力される（SSP 2.8.23 以上で動作する）。

```toml
[actor."女の子"]
spot = 0
surface = 0

[actor."女の子".dressup."帽子"]
"麦わら" = 1
```

- キー名 `surface`・`dressup`・`spot` は、アクター単語の検索で最優先される。トーク中の `＠surface` は設定値（例: 文字 `0`）に解決されるため、`surface`・`dressup`・`spot` という名前の単語は作らない。
- 既定サーフェスを決めるキーは `surface` である。`default_surface` を書いても既定サーフェスにはならない。
- 復旧の振る舞いの詳細は [同一スポット共有時の外見の復旧](../grammar/actor-dictionary.md#同一スポット共有時の外見の復旧) を参照する。

---

### [talk]（トーク表示制御）

台詞をさくらスクリプトにするときのウェイトの挿入を制御する。**分類: SHIORI デフォルト有（省略可）**。

| キー | 型 | 既定値 | 説明 |
| ---- | -- | ------ | ---- |
| `script_wait_normal` | 整数 | `50` | 通常の文字のウェイト（ミリ秒） |
| `script_wait_period` | 整数 | `1000` | 句点のウェイト（ミリ秒） |
| `script_wait_comma` | 整数 | `500` | 読点のウェイト（ミリ秒） |
| `script_wait_strong` | 整数 | `500` | 感嘆符・疑問符のウェイト（ミリ秒） |
| `script_wait_leader` | 整数 | `200` | リーダーのウェイト（ミリ秒） |
| `chars_period` | 文字列 | `"｡。．."` | 句点として扱う文字 |
| `chars_comma` | 文字列 | `"、，,"` | 読点として扱う文字 |
| `chars_strong` | 文字列 | `"？！!?"` | 感嘆符・疑問符として扱う文字 |
| `chars_leader` | 文字列 | `"･・‥…"` | リーダーとして扱う文字 |
| `chars_line_start_prohibited` | 文字列 | `"゛゜ヽヾゝゞ々ー）］｝」』):;]}｣､･ｰﾞﾟ"` | 行頭禁則文字。直前の句読点の並びに含め、並びの最後にまとめてウェイトを入れる |
| `chars_line_end_prohibited` | 文字列 | `"（［｛「『([{｢"` | 行末禁則文字。ウェイトを入れない |

- 実際に挿入されるウェイトは、設定値から 50 を引いた値である（0 以下なら挿入しない。既定の通常文字ではウェイトが入らない）。
- アクター設定（`[actor."名前"]`）に同じ名前の `script_wait_*` キーを書くと、そのアクターではアクターの値が優先される。文字の集合はアクターごとには変えられない。
- 文字の分類とウェイトの入れ方の詳細は [@pasta_sakura_script の動作仕様](../lua/modules/pasta-sakura-script.md#動作仕様) を参照する。

**使用例**: 表示速度を速くする場合。

```toml
[talk]
script_wait_normal = 30
script_wait_period = 600
script_wait_comma = 300
```

---

### [persistence]（永続化）

セーブデータ（`save` テーブル）の保存先と保存形式を制御する。**分類: SHIORI デフォルト有（省略可）**。

| キー | 型 | 既定値 | 説明 |
| ---- | -- | ------ | ---- |
| `obfuscate` | 真偽値 | `false` | gzip 圧縮して保存する（難読化）。保存先の拡張子は `.dat` になる |
| `file_path` | 文字列 | `"profile/pasta/save/save.json"` | 保存先のパス（設置ディレクトリからの相対パス） |
| `debug_mode` | 真偽値 | `false` | 読み込み・保存のたびにデバッグログを出す |

```toml
[persistence]
obfuscate = true
file_path = "profile/pasta/save/save.json"
```

- `obfuscate = true` のとき、`file_path` が `.json` で終わればその部分を `.dat` に変え、それ以外なら末尾に `.dat` を足したパスへ保存する（既定では `profile/pasta/save/save.dat`）。
- `file_path` に絶対パスや `..` を含むパスを書くと、ゴーストの読み込みが失敗する。
- 保存のしくみと API は [@pasta_persistence](../lua/modules/pasta-persistence.md) を参照する。

---

### [logging]（ログ出力）

ログファイルの出力を制御する。**分類: SHIORI デフォルト有（省略可）**。

| キー | 型 | 既定値 | 説明 |
| ---- | -- | ------ | ---- |
| `file_path` | 文字列 | `"profile/pasta/logs/pasta.log"` | ログファイルのパス（設置ディレクトリからの相対パス） |
| `level` | 文字列 | `"info"` | 記録するログレベル（`error`/`warn`/`info`/`debug`/`trace`） |
| `filter` | 文字列 | （なし） | ログのフィルター指定（tracing の EnvFilter の書式）。設定すると `level` より優先される |

```toml
[logging]
level = "debug"
filter = "debug,pasta_shiori=info"
```

- `file_path` に書けるのは、`profile` で始まり `..` を含まない相対パスだけである。条件を満たさない値のときはログファイルが作られない（ゴーストの起動は続き、`level`・`filter` は反映される）。
- `filter`・`level` として解釈できない値のときは `info` になる。
- 環境変数 `PASTA_LOG` が設定されていると、`filter`・`level` より優先される。
- Lua からのログ出力は [@pasta_log](../lua/modules/pasta-log.md)、起動に失敗したときのログの読み方は [ゴーストが起動しない・喋らないとき](../reference/startup.md#4-ゴーストが起動しない喋らないとき) を参照する。

---

### [lua]（Lua ライブラリ）

上級者向けのセクションである。**分類: SHIORI デフォルト有（省略可）**。

ゴーストの Lua で使える標準ライブラリと mlua-stdlib のモジュールは、[mlua-stdlib 統合モジュール](../lua/modules/mlua-stdlib.md) に示すとおりである。`@env` は通常のゴーストから有効にできない。

---

### [debug]（デバッグバックエンド）

pasta に組み込まれた DAP デバッグバックエンドを制御する。上級者向けのセクションである。**分類: SHIORI デフォルト有（省略可）**。省略するとデバッグは無効で、本番の動作に負荷をかけない。

| キー | 型 | 既定値 | 説明 |
| ---- | -- | ------ | ---- |
| `enabled` | 真偽値 | `false` | デバッグバックエンドを有効にする |
| `port` | 整数 | `9276` | DAP リスナーが待ち受ける TCP ポート |
| `present_as` | 文字列 | （なし＝`.pasta`） | ソースの提示モード（`"pasta"` / `"lua"`） |
| `source_map_sidecar` | 真偽値 | `false` | `.lua.map` サイドカーファイルも出力する |

```toml
[debug]
enabled = true
port = 9276
present_as = "lua"
```

- 環境変数（`PASTA_DEBUG`・`PASTA_DEBUG_PORT` など）が設定されていると、`[debug]` の値より優先される。
- 有効化の手順と優先順位は [デバッグ概要](../debug/index.md#pastatoml-による有効化)、提示モードとサイドカーは [.pasta ソースレベルのデバッグ操作](../debug/source-level.md#提示モードの切替) を参照する。

---

### [package]（パッケージ情報）

**エンジンプロファイル専用**で、SHIORI 用途では書く必要がない。書いても無視され、これまでどおり起動する。分類と予約の詳細は [[package] 予約注記](#package-予約注記) を参照する。

---

これで `pasta.toml` の隅から隅まで、すっかり見渡せましたわね。
フンッ、全部を一度に覚えようなんて欲張らなくて結構。困ったときにこの頁へ戻ってくれば、わたくしがいつでもお待ちしておりますわ。
さあ、設定が整ったなら、あとは辞書を書くだけ。熱く参りましょう！
