<!-- GENERATED FROM PASTA MANUAL - DO NOT EDIT -->
<!-- このファイルは pasta 利用者マニュアル「@enc」（https://ekicyou.github.io/pasta/lua/modules/enc.html）から自動生成されたものです。手で編集しないでください。修正はマニュアルの該当章で行い、pasta リポジトリで再生成してください。 -->

# @enc

`@enc` は、UTF-8 と ANSI（システムのロケールの文字コード）の間で文字列を変換するモジュールである。主に Windows で、日本語を含むファイルパスを Lua 標準の入出力関数（`io.open` など）へ渡すときに使う。

```lua
local enc = require "@enc"
```

**モジュールのメタデータ**: `_VERSION = "0.1.0"`、`_DESCRIPTION = "Encoding conversion (UTF-8 <-> ANSI)"`

Pasta の文字列（DSL の台詞・変数、`scripts/` の Lua ソース）はすべて UTF-8 である。一方、Windows 上の Lua 標準の入出力関数はファイルパスを ANSI のバイト列として解釈する。日本語版 Windows の ANSI は CP932（Shift_JIS）である。Lua 標準の入出力関数が長いパス・ANSI で表せないパスを扱えない点は [起動シーケンスとモジュール解決](startup.md) の既知の制限で扱う。

## to_ansi(utf8_str)

UTF-8 の文字列を ANSI のバイト列へ変換する。

```lua
enc.to_ansi(utf8_str) -> ansi_string, nil | nil, error_message
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `utf8_str` | string | ✅ | 変換元の UTF-8 文字列 |

**戻り値**: 成功時は `ansi_string, nil`、失敗時は `nil, error_message`。`""` は `""` になる。

**失敗する条件**:

- `utf8_str` が文字列でない（`expected string, got integer` など）。
- 入力が正しい UTF-8 でない（`invalid UTF-8 input: …`）。
- ANSI で表現できない文字を含む（`ANSI conversion failed: …`。日本語版 Windows で絵文字を渡した場合など）。

## to_utf8(ansi_str)

ANSI のバイト列を UTF-8 の文字列へ変換する。

```lua
enc.to_utf8(ansi_str) -> utf8_string, nil | nil, error_message
```

| パラメータ | 型 | 必須 | 説明 |
| ---- | ---- | ---- | ---- |
| `ansi_str` | string | ✅ | 変換元の ANSI のバイト列 |

**戻り値**: 成功時は `utf8_string, nil`、失敗時は `nil, error_message`。`""` は `""` になる。

**失敗する条件**:

- `ansi_str` が文字列でない（`expected string, got nil` など）。
- ANSI のバイト列として正しくない（`UTF-8 conversion failed: …`。メッセージの後半は OS のエラーメッセージ）。

## 戻り値パターン

2 つの関数は、成功時に `値, nil`、失敗時に `nil, エラーメッセージ` の 2 値を返す。引数の型が合わない場合も Lua のエラーにはせず、この形で失敗を返す。失敗したときは、あわせて警告ログを出す。

```lua
local enc = require "@enc"

-- UTF-8 → ANSI（日本語のパスを Lua 標準の入出力関数へ渡す）
local ansi_path, err = enc.to_ansi("C:/ユーザー/設定.txt")
if ansi_path then
    local file = io.open(ansi_path, "r")
    if file then
        local content = file:read("*a")
        file:close()
    end
else
    print("変換エラー:", err)
end

-- ANSI → UTF-8（ANSI で返ってきた文字列を Pasta の文字列として扱う）
local utf8_path, err2 = enc.to_utf8(ansi_path)
if utf8_path then
    print("ファイルパス:", utf8_path)  -- C:/ユーザー/設定.txt
end
```

## プラットフォームによる違い

- **Windows**: システムの ANSI コードページ（日本語版では CP932）との間で変換する。結果は OS のロケール設定に依存する。
- **Windows 以外**: 変換しない。`to_ansi` は入力をそのまま返す。`to_utf8` は入力が正しい UTF-8 ならそのまま返し、そうでなければ失敗する。

複数のプラットフォームで動かすコードでは、変換結果の違いに注意する。
