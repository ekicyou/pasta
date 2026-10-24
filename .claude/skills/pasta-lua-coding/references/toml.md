<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta マニュアル「@toml」（https://ekicyou.github.io/pasta/lua/modules/toml.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# @toml

`@toml` は、TOML の文字列を Lua の表に変える（デコードする）モジュールである。持つ関数は `decode` の 1 つだけである。

```lua
local toml = require "@toml"

local t, err = toml.decode('name = "test"\nvalue = 42')
-- t.name == "test"、t.value == 42
```

## 読み込み方

`require "@toml"` で読む。設定は要らず、ランタイムの初期化の時（`main.lua` を読み込む前）から常に登録されている。

## decode

`toml.decode(text)` は、TOML の文書を表す文字列 `text` を受け取り、一番外側のテーブルを Lua の表にして返す。

```lua
local toml = require "@toml"

local t = toml.decode([[
title = "設定"

[owner]
name = "クローディア"
born = 1979-05-27T07:32:00

[[items]]
id = 1

[[items]]
id = 2
]])

print(t.title)        -- 設定
print(t.owner.name)   -- クローディア
print(t.owner.born)   -- 1979-05-27T07:32:00（文字列）
print(t.items[2].id)  -- 2
```

TOML の型は、[`@pasta_config`](pasta-config.md) と同じ対応で Lua の値になる。

| TOML の型 | Lua の値 |
| ---- | ---- |
| 文字列 | 文字列 |
| 整数・浮動小数点数 | 数値（整数も Lua の数値。2^53 を超える整数は丸まる） |
| 真偽値 | 真偽値（`true` / `false`） |
| 日時（`1979-05-27T07:32:00` など） | 文字列（RFC 3339 の形。`"1979-05-27T07:32:00"`） |
| 配列 | 1 から始まる連番の表 |
| テーブル | 表 |

## 失敗の返し方

- TOML として正しくない文字列を渡すと、`nil, エラーメッセージ` の 2 値を返す。メッセージは誤りの位置を含み、複数行になることがある。
- 文字列でない引数（数値・表・`nil` など）を渡すと、戻り値ではなく Lua のエラーになる（`bad argument #1 to 'decode' (string expected, got number)` など）。

```lua
local t, err = toml.decode("key = ")
if not t then
    -- err に誤りの位置と理由が入る
end
```

## encode は無い

`@toml` は `encode` を持たない。Lua の表を TOML の文字列に変える関数は無い。

## JSON・YAML・正規表現の部品は無い

JSON・YAML を読み書きするモジュールと、正規表現のモジュールは無い。文字列は、Lua の標準の文字列の関数（`string.find`・`string.match`・`string.gmatch`・`string.gsub`）とそのパターンで調べる。使える標準ライブラリは [Lua 標準ライブラリ](modules-index.md#lua-標準ライブラリ) にまとめてある。
