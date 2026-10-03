<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta マニュアル「@pasta_log」（https://ekicyou.github.io/pasta/lua/modules/pasta-log.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# @pasta_log

`@pasta_log` は、Lua からのログ出力をランタイムのログ基盤（Rust の tracing）へ渡すモジュールである。呼び出し元の Lua のソース・行番号・関数名を自動で取り、ログの項目として付ける。

```lua
local log = require "@pasta_log"
```

**モジュールのメタデータ**: `_VERSION = "0.1.0"`、`_DESCRIPTION = "Lua logging bridge to Rust tracing"`

ランタイムのモジュール構成に関係なく、常に登録される（[mlua-stdlib 統合モジュール](mlua-stdlib.md) の有効・無効に左右されない）。

## 出力先とログレベル

- ログはゴーストのログファイルに書かれる。既定の場所は `profile/pasta/logs/pasta.log` である（[起動シーケンスとモジュール解決](startup.md) の「ゴーストが起動しない・喋らないとき」）。
- どのレベルまで記録するかは pasta.toml の `[logging]` の `level` で決まる。既定は `info` で、`trace`・`debug` のログは記録されない。開発中に `debug` 以下も残したいときは `level = "debug"`（または `"trace"`）を書く。
- 1 件のログは次の形の 1 行になる。ログの発生元（target）は `pasta_lua::runtime::log` である。

```text
2026-10-01T10:52:25.353586Z  INFO pasta_lua::runtime::log: ゴースト起動完了 lua_source=....ghost/profile/pasta/cache/lua/pasta\scene\test\case.lua lua_line=17 lua_fn=handler
```

## 関数

5 つの関数は、記録するレベルだけが違う。どれも引数を 1 つ取り（2 つ目以降の引数は無視する）、戻り値は無い。

### trace(value)

TRACE レベルでログを出力する。

```lua
log.trace(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `value` | any | ❌ | ログメッセージ（省略時・`nil` のときは空文字列） |

### debug(value)

DEBUG レベルでログを出力する。

```lua
log.debug(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `value` | any | ❌ | ログメッセージ |

### info(value)

INFO レベルでログを出力する。

```lua
log.info(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `value` | any | ❌ | ログメッセージ |

### warn(value)

WARN レベルでログを出力する。

```lua
log.warn(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `value` | any | ❌ | ログメッセージ |

### error(value)

ERROR レベルでログを出力する。

```lua
log.error(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `value` | any | ❌ | ログメッセージ |

## 値の変換規則

各関数は任意の Lua の値を受け取り、次の規則で文字列にしてから記録する。

| Lua の型 | 記録される文字列 |
| ---- | ---- |
| 文字列 | そのまま |
| 数値・真偽値 | 文字列にした形（`42` → `"42"`、`3.5` → `"3.5"`、`true` → `"true"`） |
| `nil`（引数の省略を含む） | 空文字列 `""` |
| 表 | JSON（`{"player":"ユーザー","score":100}`、配列は `[1,2,3]`、空の表は `{}`） |
| 関数・ユーザーデータ・コルーチンなど | `tostring()` の結果（`function: 0x…` など） |

表の扱いには上限がある。

- キーと値の組が 1000 個を超える表は、`<table: 要素数 elements>`（`<table: 1001 elements>` など）になる。
- 入れ子が 10 段を超える表、循環参照を含む表、JSON にできない値（関数など）を含む表は、`tostring()` の結果（`table: 0x…`）になる。

`tostring()` も失敗した場合は `"<unconvertible value>"` を記録する。どの値を渡しても、ログの呼び出しがエラーになってゴーストを止めることはない。

## 構造化ログフィールド

各ログには次の項目が自動で付く。

| フィールド | 説明 | 例 |
| ---- | ---- | ---- |
| `lua_source` | 呼び出し元のソースの名前。長いパスは先頭を `...` で省略した形になる。DSL の ` ```lua ``` ` ブロックからの呼び出しでは、トランスパイル結果のキャッシュファイルの名前になる | `....ghost/profile/pasta/cache/lua/pasta\scene\test\case.lua` |
| `lua_line` | 呼び出し元の行番号（取れないときは `0`） | `17` |
| `lua_fn` | 呼び出し元の関数の名前（取れないとき・ファイルの最上位からの呼び出しでは空） | `handler`、`on_boot` |

## 使用例

```lua
local log = require "@pasta_log"

-- 基本的なログ出力
log.info("ゴースト起動完了")
log.debug("変数の値を確認: " .. tostring(some_var))
log.warn("非推奨の書き方が使われています")
log.error("設定ファイルの読み込みに失敗")

-- 任意の型をそのまま渡せる
log.info({ player = "ユーザー", score = 100 })  -- {"player":"ユーザー","score":100}
log.debug(42)                                   -- 42
log.trace(nil)                                  -- （空文字列）
log.warn(true)                                  -- true
```
