# @pasta_persistence

> 【目閉じ】昨日のことを覚えていてくれるゴースト――それだけで、ぐっと愛おしくなりますわよね。

> 【アンソニー】その記憶を守る金庫番が、`@pasta_persistence` でございます。

> 【クローディア】ええ。普段は自動で働いてくれますけれど、扱い方を知っておけば、大切な思い出を取りこぼしませんわ。

---

`@pasta_persistence` は、セーブデータ（Lua の表）を JSON（または gzip 圧縮した JSON）でファイルに保存し、読み込むモジュールである。

```lua
local persistence = require "@pasta_persistence"
```

**モジュールのメタデータ**: `_VERSION = "0.1.0"`、`_DESCRIPTION = "Persistent data storage (JSON/gzip)"`

## 自動で保存される save テーブルとの関係

ゴーストの永続データは、ふつうはこのモジュールを直接呼ばずに扱う。

- シーン関数の `local save, var = act:init_scene(SCENE)` が返す `save` テーブルが永続データである（[scripts/ の記述パターン](../patterns.md)）。DSL のグローバル変数 `＄＊名前` は `save.名前` になる（[グローバル変数の保存先](../../grammar/variables.md#グローバル変数の保存先)）。
- `save` テーブルは、ランタイムで 1 度だけ `load()` で読み込まれ、ランタイムの終了時（ゴーストの終了・辞書の再読込）に、その時点の内容がこのモジュールの保存先へ自動で保存される。
- 終了時の自動保存は、保存先のファイルを `save` テーブルの内容で丸ごと置き換える。`save()` に別の表を渡して保存しても、終了時に `save` テーブルの内容で上書きされる。永続させたい値は `save` テーブルに入れる。
- `save` テーブルに JSON にできない値（関数・循環参照など。[save(data)](#savedata) の失敗する条件と同じ）を入れると、終了時の自動保存が失敗し、ファイルは更新されない（エラーログを出す）。
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
- ファイルが壊れている（JSON として読めない・gzip を展開できない）、または中身が表にならない値 → 空テーブルを返す（警告ログを出す）。エラーは投げない。
- gzip 圧縮されたファイルは、先頭のバイトで自動判別して展開する。`obfuscate` の設定に関係なく、どちらの形式も読める。

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

**戻り値**: 成功時は `true, nil`、失敗時は `nil, error_message`。

**失敗する条件**（`nil, error_message` を返す）:

- Lua の値を JSON へ変換できない。循環参照を含む表（`recursive table detected`）、関数値を含む表（`unsupported value type`）、文字列・数値以外のキー（真偽値など）を持つ表。
- ファイルの書き込みに失敗した（権限不足、ディスク容量不足など）。

`data` に表以外（文字列など）を渡すと、戻り値ではなく Lua のエラーになる。

**書き込み方**:

- `obfuscate = false` のときは整形した JSON、`true` のときは gzip 圧縮した JSON を書く。
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

## pasta.toml 設定

`pasta.toml` の `[persistence]` セクションで動作を変えられる。

```toml
[persistence]
obfuscate = true                              # gzip 圧縮を有効にする（難読化）
file_path = "profile/pasta/save/save.json"    # 保存先パス
debug_mode = false                            # デバッグログを出す
```

| オプション | 型 | 既定値 | 説明 |
| ---- | ---- | ---- | ---- |
| `obfuscate` | bool | `false` | gzip 圧縮を有効にする。保存先の拡張子が `.dat` になる |
| `file_path` | string | `"profile/pasta/save/save.json"` | 保存先パス（`pasta.toml` を置いたディレクトリからの相対パス） |
| `debug_mode` | bool | `false` | 読み込み・保存のたびにデバッグログを出す |

既定値は実装の既定値である。`[persistence]` セクションを書かないときは、すべて既定値になる。`pasta.toml` の全セクションは [pasta.toml リファレンス](../../reference/pasta-toml.md#persistence永続化) を参照する。

- `obfuscate = true` のとき、`file_path` が `.json` で終わればその部分を `.dat` に変え、`.dat` 以外で終われば末尾に `.dat` を足したパスへ保存する（既定では `profile/pasta/save/save.dat`）。
- `file_path` に絶対パスや `..` を含むパスを書くと、ゴーストの読み込みが失敗する（保存先がゴーストのディレクトリの外へ出ることを防ぐ）。

## セーブキーの命名規約

`pasta_` で始まるキーはエンジンの予約領域である（[予約グローバル変数](../../grammar/variables.md#予約グローバル変数pasta_-で始まる名前)）。ゴースト固有のキーには `pasta_` を付けず、`talk_count` や `my_flag` のような任意の名前を使う。

---

> 【にっこり】これで思い出の守り方は完璧ですわね。大切なものは `save` テーブルへ――それさえ覚えておけば安心ですわ。

> 【アンソニー】ゴーストに何もかも忘れられては、お嬢様もお寂しいでしょう。

> 【照れ】……ええ、少しだけ。あなたのゴーストが忘れてしまったら、わたくしだって寂しいのですもの。
