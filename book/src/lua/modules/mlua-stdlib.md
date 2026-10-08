# mlua-stdlib 統合モジュール

> 【したり顔】汎用の用事のために、わざわざ道具を自作する必要はございませんわ。JSON の読み書きも、正規表現での文字列調べも――Pasta には mlua-stdlib という外部ライブラリのモジュールが、最初から組み込まれておりますの。すぐに使える子たちと、使えない子の見分け方を、ここでお伝えいたしますわ。

> 【アンソニー】では、昨夜から私が書いております JSON の読み取り係は……。

> 【驚き】もう自作に手を付けていましたの！？

---

Pasta ランタイムは、Rust のライブラリ [mlua-stdlib](https://docs.rs/mlua-stdlib)（0.1 系）のモジュールを組み込んでいる。`require` で読み込んで使う。各関数の詳細は [mlua-stdlib の API ドキュメント](https://docs.rs/mlua-stdlib) を参照する。

| モジュール | 用途 | 既定 |
| ---- | ---- | ---- |
| `@json` | JSON のエンコード・デコード | 有効 |
| `@yaml` | YAML のエンコード・デコード | 有効 |
| `@regex` | 正規表現 | 有効 |
| `@assertions` | アサーション関数 | 有効 |
| `@testing` | テストの登録と実行 | 有効 |
| `@env` | 環境変数・ファイルシステムのパス | 無効 |

## デフォルトで有効なモジュール

失敗を返す関数（`encode`・`decode`・`regex.new` など）は、成功時に値を 1 つ、失敗時に `nil, エラーメッセージ` を返す。

### @json

JSON のエンコード・デコード。

```lua
local json = require "@json"

local str = json.encode({ name = "test", value = 42 })  -- {"name":"test","value":42}
local obj = json.decode('{"name": "test"}')             -- obj.name == "test"
```

- `json.encode(value, opts?)`: Lua の値を JSON の文字列にする。配列の表は `[1,2,3]`、空の表は `{}` になる。`opts.pretty = true` で整形し（キーを並べ替える）、`opts.relaxed = true` で関数などの変換できない値と循環参照を飛ばす。変換できない値があると `nil, エラーメッセージ`（`cannot serialize <function>` など）を返す。
- `json.decode(str, opts?)`: JSON の文字列を Lua の値にする。JSON の `null` は既定で専用の値（`nil` ではない）になり、`opts.null_as_nil = true` で `nil` になる。`opts.set_array_metatable = true` で配列に印のメタテーブルを付ける。JSON として正しくないと `nil, エラーメッセージ` を返す。
- `json.decode_native(str)`: Lua の表に変換せず、元の JSON を保持したユーザーデータを返す。フィールドは `obj.a.b[1]` のように読め、`obj:pointer("/a/b/1")`（JSON Pointer）・`obj:dump()`（Lua の値への変換）・`obj:iter()` を持つ。

### @yaml

YAML のエンコード・デコード。

```lua
local yaml = require "@yaml"

local str = yaml.encode({ name = "test" })  -- "name: test\n"
local obj = yaml.decode("name: test")       -- obj.name == "test"
```

`yaml.encode(value, opts?)`・`yaml.decode(str, opts?)`・`yaml.decode_native(str)` を持つ。オプションと失敗の返し方は `@json` と同じ（`encode` の `pretty` を除く）。

### @regex

正規表現（Rust の regex クレートの構文）。

```lua
local regex = require "@regex"

local re = regex.new("(\\d+)")
re:is_match("abc123def456")            -- true
local m = re:match("abc123def456")
print(m[0], m[1])                      -- 123  123（最初の一致と 1 番目のグループ）
local parts = re:split("abc123def456ghi")  -- { "abc", "def", "ghi" }
re:replace("abc123def456", "<$1>")     -- "abc<123>def456"（最初の一致だけを置き換える）
```

- `regex.new(pattern)`: 正規表現をコンパイルする。構文が誤っていると `nil, エラーメッセージ` を返す。
- 正規表現オブジェクトのメソッド: `is_match(text)`（真偽値）、`match(text)`（最初の一致。番号 `m[0]`・`m[1]`… と名前付きグループ `m.名前` で読める。一致しなければ `nil`）、`split(text)`・`splitn(text, n)`（分割した文字列の配列）、`replace(text, rep)`（最初の一致を置き換える。`rep` の `$1` などはグループに展開される）、`captures_read(text)`（グループの位置）。
- `regex.is_match(pattern, text)`・`regex.match(pattern, text)`: コンパイルせずに 1 回だけ調べる。`regex.match` は一致全体を `[0]`、グループを `[1]` 以降に持つ表を返す（名前付きグループは名前では読めない）。
- `regex.escape(text)`: 文字列を正規表現の中でそのまま一致させるためにエスケープする（`"a.b*c"` → `a\.b\*c`）。
- `regex.RegexSet.new(patterns)`: 複数の正規表現の集合。`set:is_match(text)`・`set:matches(text)`（一致した正規表現の番号の配列。1 始まり）・`set:len()` を持つ。

### @assertions

アサーション関数。条件が成り立たないと Lua のエラーを投げる。

```lua
local assertions = require "@assertions"

assertions.assert_eq(1 + 1, 2)                          -- ==
assertions.assert_ne(1, 2)                              -- ~=
assertions.assert_same({ x = { 1 } }, { x = { 1 } })    -- 表の中身を再帰的に比べる
```

- `assert_eq(left, right, message?)`・`assert_ne(left, right, message?)`・`assert_same(left, right, message?)` の 3 つを持つ。
- 失敗時のエラーメッセージは `assertion` で始まり、比べた左右の値を含む。`message` を渡すと、その文字列もメッセージに入る。

### @testing

テストの登録と実行。

```lua
local testing = require "@testing"

local t = testing.new("sample")
t:test("足し算", function(ctx)
    ctx.assert_eq(1 + 1, 2)
end)
t:test("後で書く", function(ctx)
    ctx.skip("未実装")
end)

local ok, results = t:run({ quiet = true })
-- ok: 失敗したテストが無ければ true
-- results[i]: { name, passed, skipped, error, duration }
```

- `testing.new(name?)` でテストの集まりを作り、`t:test(name, func)` で登録する。`func` は文脈 `ctx` を受け取り、`ctx.assert_eq`・`ctx.assert_same`・`ctx.assert(cond, msg?)`・`ctx.skip(reason?)` を使える。
- `t:before_all(func)`・`t:after_all(func)`・`t:before_each(func)`・`t:after_each(func)` で前後処理を登録する。
- `t:run(opts?)` で実行し、`ok, results` を返す。`opts.pattern` で名前が一致するテストだけを実行する。結果の一覧は標準出力に表示する（ゴーストとして動いているときは表示先が無い）。`opts.quiet = true` で表示を止める。

## デフォルトで無効なモジュール

### @env

環境変数・カレントディレクトリ・実行ファイルのパスなどへアクセスするモジュールである。既定で無効で、`require "@env"` はエラーになる（`pcall(require, "@env")` は `false` を返す）。環境変数やファイルシステムに触れられるため、セキュリティ上の理由から既定で無効にしてある。

## Lua 標準ライブラリ

ランタイムは既定で、Lua 標準ライブラリのうち安全なものだけを読み込む。

- 既定で使える: `string`・`table`・`math`・`io`・`os`・`package`・`coroutine`・`bit`・`jit`
- 既定では使えない: `debug`・`ffi`（`require "ffi"` はエラーになる）

ここに示した構成（mlua-stdlib のモジュールと Lua 標準ライブラリの有効・無効）は pasta が固定しており、ゴーストから変える手段は無い。

---

> 【不機嫌】フンッ、`@env` が使えなくても拗ねないでくださいまし。あなたのゴーストを守るためですもの。道具箱の中身は、これですべてですわ。

> 【アンソニー】私の読み取り係は、どうやら日の目を見ずに終わりそうでございます。

> 【にっこり】その腕は、実際の `scripts/` で道具を組み合わせるときに振るってくださいまし。
