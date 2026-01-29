# 6 – 標準ライブラリ

> **Source**: Lua 5.5 Reference Manual - Chapter 6 (Standard Libraries)
> **Translation**: AI-generated based on Lua 5.4 Japanese reference with Lua 5.5 updates
> **Glossary**: See [GLOSSARY.md](../GLOSSARY.md) for terminology

---

標準Luaライブラリは、C APIを介してCで実装された便利な関数を提供します。これらの関数の中には、言語に不可欠なサービスを提供するもの（例：`type`と`getmetatable`）があります。その他は、外部サービス（例：I/O）へのアクセスを提供します。また、Lua自体で実装できるものもありますが、さまざまな理由でCでの実装に値するものもあります（例：`table.sort`）。

すべてのライブラリは公式のC APIを介して実装され、個別のCモジュールとして提供されます。特に断りのない限り、これらのライブラリ関数は、その引数の数を予期されるパラメータに調整しません。たとえば、`foo(arg)`としてドキュメント化された関数は、引数なしで呼び出すべきではありません。

**fail**という表記は、何らかの失敗を表すfalse値を意味します。（現在、**fail**は**nil**と等しいですが、将来のバージョンでは変更される可能性があります。推奨されるのは、`(status == nil)`の代わりに、常に`(not status)`でこれらの関数の成功をテストすることです。）

現在、Luaには次の標準ライブラリがあります：

- 基本ライブラリ（§6.2）
- コルーチンライブラリ（§6.3）
- パッケージライブラリ（§6.4）
- 文字列操作（§6.5）
- 基本的なUTF-8サポート（§6.6）
- テーブル操作（§6.7）
- 数学関数（§6.8）（sin、logなど）
- 入出力（§6.9）
- オペレーティングシステム機能（§6.10）
- デバッグ機能（§6.11）

基本ライブラリとパッケージライブラリを除いて、各ライブラリは、そのすべての関数をグローバルテーブルのフィールドとして、またはそのオブジェクトのメソッドとして提供します。

---

## 6.1 – Cコードでのライブラリのロード

> **Lua 5.5 の新規セクション**: このセクションは5.5で独立したセクションになりました。

Cホストプログラムは、スクリプトが標準ライブラリを使用する場合、明示的にステートにロードする必要があります。そのために、ホストプログラムは関数`luaL_openlibs`を呼び出すことができます。または、`luaL_openselectedlibs`を使用して、どのライブラリを開くかを選択することもできます。両方の関数はヘッダーファイル`lualib.h`で宣言されています。

スタンドアロンインタプリタ`lua`（§7を参照）は、すでにすべての標準ライブラリを開いています。

### luaL_openlibs

```c
void luaL_openlibs (lua_State *L);
```

`[-0, +0, e]`

すべての標準Luaライブラリを指定されたステートに開きます。

### luaL_openselectedlibs

> **Lua 5.5 新規関数**

```c
void luaL_openselectedlibs (lua_State *L, int load, int preload);
```

`[-0, +0, e]`

選択された標準ライブラリをステート`L`に開く（ロードする）および事前ロードします。（*事前ロード*とは、ライブラリローダーをテーブル`package.preload`に追加することで、プログラムが後で`require`できるようにすることを意味します。`require`自体は*package*ライブラリによって提供されることに注意してください。プログラムがそのライブラリをロードしない場合、何もrequireできません。）

整数`load`はどのライブラリをロードするかを選択し、整数`preload`はロードされなかったものの中からどれを事前ロードするかを選択します。両方とも、以下の定数のビット単位ORで形成されたマスクです：

| 定数 | ライブラリ |
|------|------------|
| `LUA_GLIBK` | 基本ライブラリ |
| `LUA_LOADLIBK` | パッケージライブラリ |
| `LUA_COLIBK` | コルーチンライブラリ |
| `LUA_STRLIBK` | 文字列ライブラリ |
| `LUA_UTF8LIBK` | UTF-8ライブラリ |
| `LUA_TABLIBK` | テーブルライブラリ |
| `LUA_MATHLIBK` | 数学ライブラリ |
| `LUA_IOLIBK` | I/Oライブラリ |
| `LUA_OSLIBK` | オペレーティングシステムライブラリ |
| `LUA_DBLIBK` | デバッグライブラリ |

---

## 6.2 – 基本関数

基本ライブラリは、Luaへのコア関数を提供します。このライブラリをアプリケーションに含めない場合は、その機能の一部に実装を提供する必要があるかどうかを注意深く確認する必要があります。

### 主要な基本関数

| 関数 | 説明 |
|------|------|
| `assert(v [, message])` | vがfalseならエラーを発生、それ以外は全引数を返す |
| `collectgarbage([opt [, arg]])` | ガベージコレクタへのインターフェース |
| `dofile([filename])` | ファイルをロードして実行 |
| `error(message [, level])` | エラーを発生 |
| `_G` | グローバル環境を保持するグローバル変数 |
| `getmetatable(object)` | メタテーブルを取得 |
| `ipairs(t)` | 整数インデックスでテーブルを反復 |
| `load(chunk [, chunkname [, mode [, env]]])` | チャンクをロード |
| `loadfile([filename [, mode [, env]]])` | ファイルからチャンクをロード |
| `next(table [, index])` | テーブルの次のインデックスと値を返す |
| `pairs(t)` | テーブルの全キー・値ペアを反復 |
| `pcall(f [, arg1, ...])` | 保護モードで関数を呼び出す |
| `print(...)` | 引数を文字列に変換して出力 |
| `rawequal(v1, v2)` | メタメソッドなしで等価性をチェック |
| `rawget(table, index)` | メタメソッドなしでテーブル値を取得 |
| `rawlen(v)` | メタメソッドなしで長さを取得 |
| `rawset(table, index, value)` | メタメソッドなしでテーブル値を設定 |
| `select(index, ...)` | 可変長引数の一部を返す |
| `setmetatable(table, metatable)` | メタテーブルを設定 |
| `tonumber(e [, base])` | 数値に変換 |
| `tostring(v)` | 文字列に変換 |
| `type(v)` | 型を文字列で返す |
| `_VERSION` | Luaバージョン文字列（"Lua 5.5"） |
| `warn(msg1, ...)` | 警告を発行 |
| `xpcall(f, msgh [, arg1, ...])` | メッセージハンドラ付きで保護呼び出し |

### collectgarbage の Lua 5.5 での変更

> **Lua 5.5 変更点**: パラメータの扱いが大幅に変更されました。

```lua
collectgarbage([opt [, arg]])
```

Lua 5.5では、`"incremental"`と`"generational"`オプションは単にモードを変更し、**以前のモードを返す**ようになりました。パラメータの設定は新しい`"param"`オプションで行います：

**`"param"`オプション（Lua 5.5 新規）**:
コレクタのパラメータの値を変更または取得します。このオプションの後に1つまたは2つの追加引数が必要です：変更または取得するパラメータの名前（文字列）と、オプションでそのパラメータの新しい値（範囲[0,100000]の整数）。

| パラメータ名 | 説明 |
|--------------|------|
| `"minormul"` | マイナー乗数 |
| `"majorminor"` | メジャー・マイナー乗数 |
| `"minormajor"` | マイナー・メジャー乗数 |
| `"pause"` | ガベージコレクタ一時停止 |
| `"stepmul"` | ステップ乗数 |
| `"stepsize"` | ステップサイズ |

呼び出しは常にパラメータの以前の値を返します。新しい値を指定しない場合、値は変更されません。

**`"step"`オプションの変更**:
- サイズが正の`n`の場合、コレクタは`n`バイトの新規割り当てが行われたかのように動作
- サイズがゼロの場合、基本ステップを実行
- インクリメンタルモードでは、ステップがコレクションサイクルを終了した場合に**true**を返す
- 世代別モードでは、ステップがメジャーコレクションを終了した場合に**true**を返す

---

## 6.3 – コルーチン操作

このライブラリは、`coroutine`テーブルに含まれるコルーチンを操作する操作で構成されています。コルーチンの概要については、§2.6を参照してください。

| 関数 | 説明 |
|------|------|
| `coroutine.close(co)` | コルーチンを閉じる |
| `coroutine.create(f)` | 新しいコルーチンを作成 |
| `coroutine.isyieldable([co])` | yieldできるかどうかを返す |
| `coroutine.resume(co [, val1, ...])` | コルーチンを再開 |
| `coroutine.running()` | 実行中のコルーチンを返す |
| `coroutine.status(co)` | コルーチンのステータスを返す |
| `coroutine.wrap(f)` | コルーチンをラップした関数を返す |
| `coroutine.yield(...)` | コルーチンの実行を中断 |

---

## 6.4 – モジュール

packageライブラリは、Luaでモジュールをロードするための基本的な機能を提供します。グローバル環境に1つの関数`require`を直接エクスポートします。他のすべては、`package`テーブルにエクスポートされます。

| 関数/変数 | 説明 |
|-----------|------|
| `require(modname)` | モジュールをロード |
| `package.config` | パッケージ構成文字列 |
| `package.cpath` | Cローダーのパス |
| `package.loaded` | ロード済みモジュールのテーブル |
| `package.loadlib(libname, funcname)` | Cライブラリをロード |
| `package.path` | Luaローダーのパス |
| `package.preload` | 事前ロードテーブル |
| `package.searchers` | 検索関数のテーブル |
| `package.searchpath(name, path [, sep [, rep]])` | パス検索 |

---

## 6.5 – 文字列操作

このライブラリは、`string`テーブルに文字列操作の汎用関数を提供します。また、文字列のメタテーブルを設定して、`string.byte(s,i)`の代わりに`s:byte(i)`のように使用できるようにします。

| 関数 | 説明 |
|------|------|
| `string.byte(s [, i [, j]])` | 文字のバイト値を返す |
| `string.char(...)` | バイト値から文字列を作成 |
| `string.dump(function [, strip])` | 関数をバイナリにダンプ |
| `string.find(s, pattern [, init [, plain]])` | パターンを検索 |
| `string.format(formatstring, ...)` | 書式化 |
| `string.gmatch(s, pattern [, init])` | パターンマッチイテレータ |
| `string.gsub(s, pattern, repl [, n])` | パターン置換 |
| `string.len(s)` | 文字列長を返す |
| `string.lower(s)` | 小文字に変換 |
| `string.match(s, pattern [, init])` | パターンマッチ |
| `string.pack(fmt, v1, v2, ...)` | 値をバイナリ文字列にパック |
| `string.packsize(fmt)` | パックサイズを返す |
| `string.rep(s, n [, sep])` | 文字列を繰り返す |
| `string.reverse(s)` | 文字列を反転 |
| `string.sub(s, i [, j])` | 部分文字列を抽出 |
| `string.unpack(fmt, s [, pos])` | バイナリ文字列をアンパック |
| `string.upper(s)` | 大文字に変換 |

### パターンマッチング

文字列ライブラリは正規表現の代わりにパターンを使用します。主なパターン要素：

- `.` 任意の文字
- `%a` 英字
- `%d` 数字
- `%s` 空白文字
- `%w` 英数字
- `[set]` 文字クラス
- `*` 0回以上の繰り返し
- `+` 1回以上の繰り返し
- `-` 0回以上の最短繰り返し
- `?` 0回または1回

---

## 6.6 – UTF-8サポート

このライブラリは、`utf8`テーブルにUTF-8エンコーディングの基本的なサポートを提供します。このライブラリは、文字列のエンコーディングに関してUnicodeサポート以外のサポートを提供しません。

| 関数/変数 | 説明 |
|-----------|------|
| `utf8.char(...)` | コードポイントから文字列を作成 |
| `utf8.charpattern` | UTF-8文字のパターン |
| `utf8.codes(s [, lax])` | コードポイントイテレータ |
| `utf8.codepoint(s [, i [, j [, lax]]])` | コードポイントを返す |
| `utf8.len(s [, i [, j [, lax]]])` | UTF-8文字数を返す |
| `utf8.offset(s, n [, i])` | n番目の文字のバイト位置を返す |

---

## 6.7 – テーブル操作

このライブラリは、`table`テーブルにテーブル操作の汎用関数を提供します。

| 関数 | 説明 |
|------|------|
| `table.concat(list [, sep [, i [, j]]])` | テーブルを連結 |
| `table.insert(list, [pos,] value)` | 要素を挿入 |
| `table.move(a1, f, e, t [,a2])` | 要素を移動 |
| `table.pack(...)` | 可変長引数をテーブルにパック |
| `table.remove(list [, pos])` | 要素を削除 |
| `table.sort(list [, comp])` | テーブルをソート |
| `table.unpack(list [, i [, j]])` | テーブルを展開 |

---

## 6.8 – 数学関数

このライブラリは、`math`テーブルに標準的なC数学ライブラリのサブセットに加えて、いくつかの追加関数を提供します。

| 関数/定数 | 説明 |
|-----------|------|
| `math.abs(x)` | 絶対値 |
| `math.acos(x)` | 逆余弦 |
| `math.asin(x)` | 逆正弦 |
| `math.atan(y [, x])` | 逆正接 |
| `math.ceil(x)` | 切り上げ |
| `math.cos(x)` | 余弦 |
| `math.deg(x)` | ラジアンを度に変換 |
| `math.exp(x)` | e^x |
| `math.floor(x)` | 切り捨て |
| `math.fmod(x, y)` | 浮動小数点剰余 |
| `math.huge` | 正の無限大 |
| `math.log(x [, base])` | 対数 |
| `math.max(x, ...)` | 最大値 |
| `math.maxinteger` | 最大整数値 |
| `math.min(x, ...)` | 最小値 |
| `math.mininteger` | 最小整数値 |
| `math.modf(x)` | 整数部と小数部 |
| `math.pi` | π |
| `math.rad(x)` | 度をラジアンに変換 |
| `math.random([m [, n]])` | 乱数生成 |
| `math.randomseed([x [, y]])` | 乱数シード設定 |
| `math.sin(x)` | 正弦 |
| `math.sqrt(x)` | 平方根 |
| `math.tan(x)` | 正接 |
| `math.tointeger(x)` | 整数に変換 |
| `math.type(x)` | 数値の型を返す |
| `math.ult(m, n)` | 符号なし比較 |

---

## 6.9 – 入出力機能

I/Oライブラリは、`io`テーブルにファイル操作のための2つのスタイルを提供します：

1. **暗黙的なファイルハンドル**: デフォルトの入力ファイルと出力ファイルを使用
2. **明示的なファイルハンドル**: ファイルハンドルを直接操作

| 関数 | 説明 |
|------|------|
| `io.close([file])` | ファイルを閉じる |
| `io.flush()` | 出力バッファをフラッシュ |
| `io.input([file])` | デフォルト入力ファイルを設定/取得 |
| `io.lines([filename, ...])` | 行イテレータを返す |
| `io.open(filename [, mode])` | ファイルを開く |
| `io.output([file])` | デフォルト出力ファイルを設定/取得 |
| `io.popen(prog [, mode])` | パイプを開く |
| `io.read(...)` | デフォルト入力から読み込み |
| `io.stderr` | 標準エラー出力 |
| `io.stdin` | 標準入力 |
| `io.stdout` | 標準出力 |
| `io.tmpfile()` | 一時ファイルを作成 |
| `io.type(obj)` | ファイルハンドルかどうかをチェック |
| `io.write(...)` | デフォルト出力に書き込み |

### ファイルハンドルメソッド

| メソッド | 説明 |
|----------|------|
| `file:close()` | ファイルを閉じる |
| `file:flush()` | バッファをフラッシュ |
| `file:lines(...)` | 行イテレータを返す |
| `file:read(...)` | ファイルから読み込み |
| `file:seek([whence [, offset]])` | ファイル位置を移動 |
| `file:setvbuf(mode [, size])` | バッファリングモードを設定 |
| `file:write(...)` | ファイルに書き込み |

---

## 6.10 – オペレーティングシステム機能

このライブラリは、`os`テーブルにオペレーティングシステム機能を提供します。

| 関数 | 説明 |
|------|------|
| `os.clock()` | CPU時間を返す |
| `os.date([format [, time]])` | 日時を書式化 |
| `os.difftime(t2, t1)` | 時間差を計算 |
| `os.execute([command])` | シェルコマンドを実行 |
| `os.exit([code [, close]])` | プログラムを終了 |
| `os.getenv(varname)` | 環境変数を取得 |
| `os.remove(filename)` | ファイルを削除 |
| `os.rename(oldname, newname)` | ファイルをリネーム |
| `os.setlocale(locale [, category])` | ロケールを設定 |
| `os.time([table])` | 現在時刻を取得 |
| `os.tmpname()` | 一時ファイル名を生成 |

---

## 6.11 – デバッグライブラリ

このライブラリは、`debug`テーブルにデバッグインターフェースの機能を提供します。これらの関数の一部はLuaプログラムの基本的な前提に違反する可能性があるため、使用には注意が必要です。

| 関数 | 説明 |
|------|------|
| `debug.debug()` | 対話的デバッグモードに入る |
| `debug.gethook([thread])` | フックを取得 |
| `debug.getinfo([thread,] f [, what])` | 関数情報を取得 |
| `debug.getlocal([thread,] f, local)` | ローカル変数を取得 |
| `debug.getmetatable(value)` | メタテーブルを取得 |
| `debug.getregistry()` | レジストリを取得 |
| `debug.getupvalue(f, up)` | アップバリューを取得 |
| `debug.getuservalue(u, n)` | ユーザー値を取得 |
| `debug.sethook([thread,] hook, mask [, count])` | フックを設定 |
| `debug.setlocal([thread,] level, local, value)` | ローカル変数を設定 |
| `debug.setmetatable(value, table)` | メタテーブルを設定 |
| `debug.setupvalue(f, up, value)` | アップバリューを設定 |
| `debug.setuservalue(udata, value, n)` | ユーザー値を設定 |
| `debug.traceback([thread,] [message [, level]])` | トレースバックを取得 |
| `debug.upvalueid(f, n)` | アップバリューIDを取得 |
| `debug.upvaluejoin(f1, n1, f2, n2)` | アップバリューを結合 |

---

## Lua 5.5 での主な変更点まとめ

### 新規追加

1. **§6.1 セクションの独立**: 「Cコードでのライブラリのロード」が独立したセクションに
2. **`luaL_openselectedlibs`**: 選択的にライブラリをロードする新関数
3. **ライブラリ定数**: `LUA_GLIBK`, `LUA_LOADLIBK`などのビットマスク定数

### collectgarbage の変更

1. **`"param"`オプション新規**: パラメータの取得/設定専用オプション
2. **モードオプションの簡素化**: `"incremental"`と`"generational"`は単にモードを変更し、以前のモードを返す
3. **新パラメータ名**: `"majorminor"`, `"minormajor"`などの新しいパラメータ名

### セクション番号の変更

| Lua 5.4 | Lua 5.5 | 内容 |
|---------|---------|------|
| §6.1 | §6.2 | 基本関数 |
| §6.2 | §6.3 | コルーチン操作 |
| §6.3 | §6.4 | モジュール |
| §6.4 | §6.5 | 文字列操作 |
| §6.5 | §6.6 | UTF-8サポート |
| §6.6 | §6.7 | テーブル操作 |
| §6.7 | §6.8 | 数学関数 |
| §6.8 | §6.9 | 入出力機能 |
| §6.9 | §6.10 | OS機能 |
| §6.10 | §6.11 | デバッグライブラリ |
| – | §6.1 | **新規**: Cコードでのライブラリのロード |

---

> **完全なリファレンス**: 個々の関数の詳細なパラメータと動作については、
> [Lua 5.5 Reference Manual (英語)](https://www.lua.org/manual/5.5/) を参照してください。
