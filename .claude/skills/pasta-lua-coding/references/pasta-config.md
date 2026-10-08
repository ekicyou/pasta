<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta マニュアル「@pasta_config」（https://ekicyou.github.io/pasta/lua/modules/pasta-config.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# @pasta_config

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

`[scene]` には何も補われない。`pasta.toml` に書いたとおりに現れ、書かなければ `config.scene` は `nil` になる。`[scene.alias]` を書かなかったときに使われる既定の別名表（`OnTalk = ["会話"]`）も補完されない。そのため `config.scene` からは、実際に使われている別名表を読み取れないことがある（[[scene]](https://ekicyou.github.io/pasta/reference/pasta-toml.html#sceneシーン名)）。

`[ghost]`・`[actor]` に書けるキーと既定値は、[pasta.toml リファレンス](https://ekicyou.github.io/pasta/reference/pasta-toml.html) の [[ghost]](https://ekicyou.github.io/pasta/reference/pasta-toml.html#ghostゴースト動作)・[[actor."名前"]](https://ekicyou.github.io/pasta/reference/pasta-toml.html#actor名前アクター設定) を参照する。

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
