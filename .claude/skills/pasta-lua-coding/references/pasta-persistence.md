<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta マニュアル「@pasta_persistence」（https://ekicyou.github.io/pasta/lua/modules/pasta-persistence.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# @pasta_persistence

`@pasta_persistence` は、セーブデータ（Lua の表）を TOML（または gzip 圧縮した TOML）でファイルに保存し、読み込むモジュールである。既定の保存先は、ゴーストのフォルダからの相対パス `profile/pasta/save/save.toml` である（[pasta.toml 設定](#pastatoml-設定)）。

```lua
local persistence = require "@pasta_persistence"
```

**モジュールのメタデータ**: `_VERSION = "0.1.0"`、`_DESCRIPTION = "Persistent data storage (TOML/gzip)"`

## 自動で保存される save テーブルとの関係

ゴーストの永続データは、ふつうはこのモジュールを直接呼ばずに扱う。

- シーン関数の `local save, var = act:init_scene(SCENE)` が返す `save` テーブルが永続データである（[scripts/ の記述パターン](https://ekicyou.github.io/pasta/lua/patterns.html)）。DSL のグローバル変数 `＄＊名前` は `save.名前` になる（[グローバル変数の保存先](https://ekicyou.github.io/pasta/grammar/variables.html#グローバル変数の保存先)）。
- `save` テーブルは、ランタイムで 1 度だけ `load()` で読み込まれ、ランタイムの終了時（ゴーストの終了・辞書の再読込）に、その時点の内容がこのモジュールの保存先へ自動で保存される。
- 終了時の自動保存は、保存先のファイルを `save` テーブルの内容で丸ごと置き換える。`save()` に別の表を渡して保存しても、終了時に `save` テーブルの内容で上書きされる。永続させたい値は `save` テーブルに入れる。
- `save` テーブルに TOML で書けない値（関数・循環参照など。[書ける値と書けない値](#書ける値と書けない値)）を入れると、終了時の自動保存は、そのキーだけを保存せずに警告ログを出し、ほかのキーを保存する。
- 途中で確実にファイルへ書き出したい場合は、`save` テーブルそのものを `save()` に渡す（`persistence.save(save)`）。

```lua
local persistence = require "@pasta_persistence"

function SCENE.カウント(act)
    local save, var = act:init_scene(SCENE)
    save.play_count = (save.play_count or 0) + 1   -- 終了時に自動で保存される

    local ok, err = persistence.save(save)         -- 今すぐ書き出す場合
    if not ok then
        print("保存失敗:", err)
    end
end
```

## 関数

### load()

保存先のファイルからデータを読み込む。

```lua
persistence.load() -> table
```

**戻り値**: 保存されていたデータ（Lua の表）。呼ぶたびにファイルを読み、新しい表を返す（`save` テーブルとは別の表になる）。

- ファイルが無い（初回起動など）→ 空テーブル `{}` を返す（警告ログを出す）。
- ファイルが空 → 空テーブルを返す。
- ファイルが読めない（TOML として読めない・gzip を展開できない）→ ファイルを `.broken` の名前に移して、空テーブルを返す（警告ログを出す）。エラーは投げない（[読めないセーブの退避](#読めないセーブの退避)）。
- gzip 圧縮されたファイルは、先頭のバイトで自動判別して展開する。`obfuscate` の設定に関係なく、どちらの形式も読める。
- TOML の日時（`1979-05-27T07:32:00Z` など）は、文字列として読む。

```lua
local data = persistence.load()
data.play_count = data.play_count or 0
data.player_name = data.player_name or "Guest"
```

### save(data)

データを保存先のファイルへ書き込む。

```lua
persistence.save(data) -> true, nil | nil, error_message
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `data` | table | ✅ | 保存するデータ（Lua の表） |

**戻り値**: 成功時は `true, nil`、失敗時は `nil, error_message`（`"Failed to save: …"`）。

TOML で書けない値を持つキーは、そのキーだけを保存せず、警告ログを出してほかのキーを保存する。このときも戻り値は `true, nil` である（[書ける値と書けない値](#書ける値と書けない値)）。

**失敗する条件**（`nil, error_message` を返す）:

- ファイルの書き込みに失敗した（権限不足、ディスク容量不足など）。
- 読めないセーブを退避できず、そのランタイムでは保存しないことになっている（[読めないセーブの退避](#読めないセーブの退避)）。

`data` に表以外（文字列など）を渡すと、戻り値ではなく Lua のエラーになる。

**書き込み方**:

- `obfuscate = false` のときは TOML、`true` のときは gzip 圧縮した TOML を書く。
- そのランタイムでまだ保存先を読んでいなければ、先に読む（読めなければ退避する）。読めないセーブを確かめずに上書きしないためである。
- 保存先のディレクトリが無ければ作る。
- いったん拡張子を `.tmp` にした一時ファイルへ書き、書き終えてから保存先の名前に置き換える。書き込みの途中で失敗しても、元のファイルは壊れない。

```lua
local data = persistence.load()
data.play_count = (data.play_count or 0) + 1
data.last_played = os.date()

local ok, err = persistence.save(data)
if not ok then
    print("保存失敗:", err)
end
```

## 書ける値と書けない値

セーブデータは TOML の文書として書く。Lua の値は次のように書く。

| Lua の値 | 書き方 |
| ---- | ---- |
| 文字列 | TOML の文字列 |
| 真偽値 | TOML の真偽値 |
| 数 | 64 ビットの整数の範囲にある整数は TOML の整数、それ以外（小数・`nan`・`inf` など）は TOML の浮動小数点数 |
| キーがすべて文字列の表 | TOML の表 |
| キーが 1 から穴なしに並ぶ表（`{"a", "b", "c"}`） | TOML の配列 |
| 空の表 `{}` | 空の表（`load()` で読んだ空の配列は、空の配列のまま書き戻る） |

次の値は書けない。書けない値を持つキーは、そのキーだけを保存しない。

| 書けない値 | 理由の言葉 |
| ---- | ---- |
| 関数・コルーチン・ユーザーデータなど | `unsupported value type (function)` など |
| 自分を含む表（循環参照） | `recursive table` |
| 入れ子が 32 段より深い表 | `nested deeper than 32 levels` |
| 番号のキーに穴がある表（`{1, nil, 3}`） | `hole in array` |
| 番号のキーと名前のキーが混ざる表（`{1, 2, x = 3}`） | `table mixes array items and named keys` |
| 文字列でないキーを持つ表（`{[5] = "x"}`・`{[true] = 1}` など） | `non-string key (integer)`・`non-string key (boolean)` など |
| UTF-8 として正しくない文字列 | `string is not valid UTF-8` |

- 入れ子の段数は、保存する表（`save` テーブル、または `save()` に渡した表）を 1 段目と数える。32 段目の表までは書ける。
- 配列の要素が 1 つでも書けなければ、その配列を持つキーごと保存しない。
- 名前が UTF-8 として正しくないキーは、そのキーだけを保存しない（理由 `key is not valid UTF-8`）。
- 保存する表そのものに文字列でないキー（`save[1]` など）があれば、そのキーだけを保存しない（理由 `non-string key at root (integer)` など）。
- LuaJIT の数は倍精度なので、手で書いた `2^53` を超える整数は丸まる。

キーを保存しなかったときは、保存のたびに 1 キー 1 件の警告ログを出す。

```lua
save.name = "ぱすた"
save.on_click = function() end
save.flags = { seen = true, secret = coroutine.create(function() end) }
persistence.save(save)   -- true, nil（name と flags.seen は保存される）
```

```text
key 'on_click' was not saved: unsupported value type (function)
key 'flags.secret' was not saved: unsupported value type (thread)
```

警告のキーは、保存する表からの道筋を `.` でつなぎ、配列の要素を `[1]` のように書く（`inventory[1].f`）。

## 読めないセーブの退避

保存先のファイルが TOML として読めない（gzip を展開できない場合を含む）とき、ファイルを消さずに `<保存先>.broken` の名前に移し、空の表から始める（警告ログを出す）。`.broken` が既にあれば、`.broken.1`、`.broken.2`、… のまだ無い最初の名前に移す。

- 既定の設定なら、`save.toml` が `save.toml.broken` に移る。
- 移せないとき（ほかのプログラムがファイルを開いたままのときなど）は、エラーログを出し、そのランタイムの間は保存しない。`save()` は `nil, error_message` を返し、終了時の自動保存もしない（エラーログを出す）。読めないファイルはそのまま残る。後で `load()` が読めても、そのランタイムの間は保存しない。
- 次のランタイム（ゴーストの再起動・辞書の再読込）では、改めて読み直す。

## 0.4.3 からの形式の変更

0.4.3 から、保存の形式が TOML に、既定の保存先が `profile/pasta/save/save.toml` に変わった。0.4.2 までのセーブは引き継がない。

- 0.4.2 までの既定の保存先 `profile/pasta/save/save.json` は読まない。ファイルは消さず、名前も中身も変えずに残る。
- `obfuscate = true` で使っていた古い `save.dat` は、保存先が同じ `save.dat` になるとき（既定の `file_path`、または末尾が `.toml`・`.dat` の `file_path`）、読めないセーブとして扱い、`save.dat.broken` に移す（中身は残る）。保存先が別の名前になるとき（末尾が `.json` の `file_path` なら `save.json.dat`）は、古い `save.dat` を読まず、そのまま残す。
- `file_path` に古いセーブの名前を書いているときも、同じように `.broken` の名前に移す。

## pasta.toml 設定

`pasta.toml` の `[persistence]` セクションで動作を変えられる。

```toml
[persistence]
obfuscate = true                              # gzip 圧縮を有効にする（難読化）
file_path = "profile/pasta/save/save.toml"    # 保存先パス
debug_mode = false                            # デバッグログを出す
```

| オプション | 型 | 既定値 | 説明 |
| ---- | ---- | ---- | ---- |
| `obfuscate` | bool | `false` | gzip 圧縮を有効にする。保存先の拡張子が `.dat` になる |
| `file_path` | string | `"profile/pasta/save/save.toml"` | 保存先パス（`pasta.toml` を置いたディレクトリからの相対パス） |
| `debug_mode` | bool | `false` | 読み込み・保存のたびにデバッグログを出す |

既定値は実装の既定値である。`[persistence]` セクションを書かないときは、すべて既定値になる。`pasta.toml` の全セクションは [pasta.toml リファレンス](https://ekicyou.github.io/pasta/reference/pasta-toml.html#persistence永続化) を参照する。

- `obfuscate = true` のとき、`file_path` の末尾が `.toml` ならその末尾だけを `.dat` に変え、`.dat` ならそのまま、それ以外なら末尾に `.dat` を足したパスへ保存する。途中のフォルダの名前は変えない。

| `file_path` | `obfuscate = false` | `obfuscate = true` |
| ---- | ---- | ---- |
| `profile/pasta/save/save.toml`（既定） | `profile/pasta/save/save.toml` | `profile/pasta/save/save.dat` |
| `my.toml.d/save.toml` | `my.toml.d/save.toml` | `my.toml.d/save.dat` |
| `save.json` | `save.json`（中身は TOML） | `save.json.dat` |
| `data.bin` | `data.bin` | `data.bin.dat` |
| `save.dat` | `save.dat` | `save.dat` |

- 拡張子に関係なく、中身は TOML（`obfuscate = true` なら gzip 圧縮した TOML）である。
- `file_path` に絶対パスや `..` を含むパスを書くと、ゴーストの読み込みが失敗する（保存先がゴーストのディレクトリの外へ出ることを防ぐ）。

## セーブキーの命名規約

`pasta_` で始まるキーはエンジンの予約領域である（[予約グローバル変数](https://ekicyou.github.io/pasta/grammar/variables.html#予約グローバル変数pasta_-で始まる名前)）。ゴースト固有のキーには `pasta_` を付けず、`talk_count` や `my_flag` のような任意の名前を使う。
