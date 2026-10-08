# @pasta_config

> 【したり顔】覗き窓をひとつ、ご紹介いたしますわ。`pasta.toml` に書いた設定を、Lua からも眺められる窓――それが `@pasta_config` ですの。

> 【アンソニー】覗き見とは、あまり行儀のよろしいことではございませんね。

> 【不機嫌】人聞きの悪いことをおっしゃらないで。キャラクターの名前でも自作のフラグでも、書いたとおりの形で受け取れますのよ。ただし読み込み方にひとつだけ作法がございますから、しっかり覚えてくださいまし。

---

`@pasta_config` は、`pasta.toml` の内容（`[loader]` セクションを除く）を Lua の表として公開するモジュールである。

```lua
local ok, config = pcall(require, "@pasta_config")
```

## 読み込み方

`pcall(require, "@pasta_config")` で読み、`ok` を確かめてから使う。

- ゴーストとして読み込まれたランタイムでは、`@pasta_config` は常に登録されている（`pasta.toml` が無いとゴーストの読み込み自体が失敗するため、登録されない状態にはならない）。
- ランタイムの外（Lua だけの単体テストなど）でスクリプトを動かすと、`@pasta_config` は登録されていない。素の `require "@pasta_config"` はエラーで止まる。
- `pcall` で保護しておけば、どちらの環境でも同じスクリプトが動く。ランタイム同梱のスクリプトも、すべてこの形で読んでいる。

## 公開されるフィールド

`pasta.toml` の `[loader]` セクション**以外**のすべてのセクション・フィールドが公開される。セクション名とキー名が、そのまま表のキーになる。

```toml
# pasta.toml の例
[loader]
# loader セクションは @pasta_config に含まれない
pasta_patterns = ["dic/*/*.pasta"]

[character]
name = "まゆら"
age = 17

[character.appearance]
hair_color = "黒"
eye_color = "茶"

[system]
debug = true
version = "1.0.0"
```

`pasta.toml` に書いた値のほかに、ランタイムが次の値を補う。

- **`[ghost]`**: 書かなくても必ず存在する。`talk_interval_min`・`talk_interval_max`・`hour_margin`・`spot_newlines` のうち書かなかったキーには、実装の既定値が入る（書いたキーはそのまま）。
- **`[actor]`**: `[actor."名前"]` の各サブテーブルに、キー名と同じ値の `name` フィールドが入る（`config.actor["さくら"].name` は `"さくら"`）。`name` を書いていても、キー名で上書きされる。

`[ghost]`・`[actor]` に書けるキーと既定値は、[pasta.toml リファレンス](../../reference/pasta-toml.md) の [[ghost]](../../reference/pasta-toml.md#ghostゴースト動作)・[[actor."名前"]](../../reference/pasta-toml.md#actor名前アクター設定) を参照する。

`[loader]` 以外のセクションを 1 つも書かなかった場合、表は `ghost` だけを持つ。

## アクセス例

```lua
local ok, config = pcall(require, "@pasta_config")
if ok then
    -- セクションのフィールド
    print(config.character.name)  -- "まゆら"
    print(config.character.age)   -- 17

    -- ネストしたテーブル
    print(config.character.appearance.hair_color)  -- "黒"

    -- 書かれていないかもしれない値は nil ガードする
    local version = config.system and config.system.version or "unknown"
end
```

書かれていないセクションは `nil` になる。`config.system.version` のように 2 段以上たどるときは、途中のセクションが無い場合に備えて nil ガードを書く。

## 注意事項

- **TOML 構造の保持**: ネストしたテーブル構造（`[character.appearance]` など）は、そのまま入れ子の表になる。
- **型の対応**: TOML の型は次のように Lua の値になる。

  | TOML の型 | Lua の値 |
  | ---- | ---- |
  | 文字列 | 文字列 |
  | 整数・浮動小数点数 | 数値（整数も Lua の数値。`17` は `17`） |
  | 真偽値 | 真偽値（`true` / `false`） |
  | 日時（`2020-01-02` など） | 文字列（TOML に書いた形。`"2020-01-02"`） |
  | 配列 | 1 から始まる連番の表 |
  | テーブル | 表 |

- **`[loader]` 除外**: `[loader]` はローダーの内部設定のため公開されない（`config.loader` は `nil`）。
- **読み取り専用ではない**: 公開される表は普通の Lua の表で、書き換えられる。書き換えた値は、同じランタイムで後から `require` した表にも見える（同じ表を返すため）が、`pasta.toml` には書き戻されない。ランタイム同梱のスクリプトも同じ表から設定を読むため、書き換えるとエンジンの動作が変わることがある。設定の読み取りにだけ使う。

---

> 【にっこり】設定を読み取る窓口、これでばっちりですわね。`pcall` の作法さえ守れば、怖いものはございませんわ。

> 【アンソニー】表を書き換えても、`pasta.toml` には書き戻されないのでございましたね。

> 【不機嫌】フンッ、ですから設定は読むだけになさいまし。困ったときは、この章を読み返せばよろしくてよ。
