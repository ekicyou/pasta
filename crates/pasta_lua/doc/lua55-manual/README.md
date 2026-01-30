<!--
  原文: https://www.lua.org/manual/5.5/
  翻訳日: 2026-01-29
  レビュー: AI Claude Opus 4.5
-->

# Lua 5.5 リファレンスマニュアル

> リファレンスマニュアルはLua言語の公式定義です。  
> Luaプログラミングの完全な入門については、書籍「*Programming in Lua*」をご覧ください。

**原文**: https://www.lua.org/manual/5.5/  
**翻訳日**: 2026年1月29日  
**ライセンス**: [Lua license](LICENSE.md) の条件に基づき自由に利用可能

---

## ❖ 目次

[1 – はじめに](01-introduction.md)

[2 – 基本概念](02-basic-concepts.md)
- [2.1 – 値と型](02-basic-concepts.md#21--値と型)
- [2.2 – スコープ、変数、および環境](02-basic-concepts.md#22--スコープ変数および環境)
- [2.3 – エラー処理](02-basic-concepts.md#23--エラー処理)
- [2.4 – メタテーブルとメタメソッド](02-basic-concepts.md#24--メタテーブルとメタメソッド)
- [2.5 – ガベージコレクション](02-basic-concepts.md#25--ガベージコレクション)
  - [2.5.1 – インクリメンタルガベージコレクション](02-basic-concepts.md#251--インクリメンタルガベージコレクション)
  - [2.5.2 – ジェネレーショナルガベージコレクション](02-basic-concepts.md#252--ジェネレーショナルガベージコレクション)
  - [2.5.3 – ガベージコレクションメタメソッド](02-basic-concepts.md#253--ガベージコレクションメタメソッド)
  - [2.5.4 – ウィークテーブル](02-basic-concepts.md#254--ウィークテーブル)
- [2.6 – コルーチン](02-basic-concepts.md#26--コルーチン)

[3 – 言語](03-language.md)
- [3.1 – 字句規則](03-language.md#31--字句規則)
- [3.2 – 変数](03-language.md#32--変数)
- [3.3 – ステートメント](03-language.md#33--ステートメント)
  - [3.3.1 – ブロック](03-language.md#331--ブロック)
  - [3.3.2 – チャンク](03-language.md#332--チャンク)
  - [3.3.3 – 代入](03-language.md#333--代入)
  - [3.3.4 – 制御構造](03-language.md#334--制御構造)
  - [3.3.5 – For文](03-language.md#335--for文)
  - [3.3.6 – 関数呼び出しステートメント](03-language.md#336--関数呼び出しステートメント)
  - [3.3.7 – 変数宣言](03-language.md#337--変数宣言) ⭐ *5.5: global追加*
  - [3.3.8 – クローズされる変数](03-language.md#338--クローズされる変数)
- [3.4 – 式](03-language.md#34--式)
  - [3.4.1 – 算術演算子](03-language.md#341--算術演算子)
  - [3.4.2 – ビット単位演算子](03-language.md#342--ビット単位演算子)
  - [3.4.3 – 強制変換と変換](03-language.md#343--強制変換と変換)
  - [3.4.4 – 関係演算子](03-language.md#344--関係演算子)
  - [3.4.5 – 論理演算子](03-language.md#345--論理演算子)
  - [3.4.6 – 連結](03-language.md#346--連結)
  - [3.4.7 – 長さ演算子](03-language.md#347--長さ演算子)
  - [3.4.8 – 優先順位](03-language.md#348--優先順位)
  - [3.4.9 – テーブルコンストラクタ](03-language.md#349--テーブルコンストラクタ)
  - [3.4.10 – 関数呼び出し](03-language.md#3410--関数呼び出し)
  - [3.4.11 – 関数定義](03-language.md#3411--関数定義) ⭐ *5.5: vararg名前付け*
  - [3.4.12 – 式のリスト、複数の結果、調整](03-language.md#3412--式のリスト複数の結果調整)
- [3.5 – 可視性ルール](03-language.md#35--可視性ルール)

[4 – アプリケーションプログラムインターフェース](04-c-api.md)
- [4.1 – スタック](04-c-api.md#41--スタック)
  - [4.1.1 – スタックサイズ](04-c-api.md#411--スタックサイズ)
  - [4.1.2 – 有効なインデックスと許容可能なインデックス](04-c-api.md#412--有効なインデックスと許容可能なインデックス)
  - [4.1.3 – 文字列へのポインタ](04-c-api.md#413--文字列へのポインタ)
- [4.2 – Cクロージャ](04-c-api.md#42--cクロージャ)
- [4.3 – レジストリ](04-c-api.md#43--レジストリ)
- [4.4 – Cでのエラー処理](04-c-api.md#44--cでのエラー処理)
  - [4.4.1 – ステータスコード](04-c-api.md#441--ステータスコード)
- [4.5 – CでのYieldの処理](04-c-api.md#45--cでのyieldの処理)
- [4.6 – 関数と型](04-c-api.md#46--関数と型)
- [4.7 – デバッグインターフェース](04-c-api.md#47--デバッグインターフェース)

[5 – 補助ライブラリ](05-auxiliary-library.md)
- [5.1 – 関数と型](05-auxiliary-library.md#51--関数と型)

[6 – 標準ライブラリ](06-standard-libraries.md)
- [6.1 – Cコードでのライブラリロード](06-standard-libraries.md#61--cコードでのライブラリロード) ⭐ *5.5新規セクション*
- [6.2 – 基本関数](06-standard-libraries.md#62--基本関数)
- [6.3 – コルーチン操作](06-standard-libraries.md#63--コルーチン操作)
- [6.4 – モジュール](06-standard-libraries.md#64--モジュール)
- [6.5 – 文字列操作](06-standard-libraries.md#65--文字列操作)
  - [6.5.1 – パターン](06-standard-libraries.md#651--パターン)
  - [6.5.2 – PackとUnpackの書式文字列](06-standard-libraries.md#652--packとunpackの書式文字列)
- [6.6 – UTF-8サポート](06-standard-libraries.md#66--utf-8サポート)
- [6.7 – テーブル操作](06-standard-libraries.md#67--テーブル操作)
- [6.8 – 数学関数](06-standard-libraries.md#68--数学関数)
- [6.9 – 入出力機能](06-standard-libraries.md#69--入出力機能)
- [6.10 – オペレーティングシステム機能](06-standard-libraries.md#610--オペレーティングシステム機能)
- [6.11 – デバッグライブラリ](06-standard-libraries.md#611--デバッグライブラリ)

[7 – Luaスタンドアロン](07-standalone.md)

[8 – 以前のバージョンとの非互換性](08-incompatibilities.md)
- [8.1 – 言語の非互換性](08-incompatibilities.md#81--言語の非互換性)
- [8.2 – ライブラリの非互換性](08-incompatibilities.md#82--ライブラリの非互換性)
- [8.3 – APIの非互換性](08-incompatibilities.md#83--apiの非互換性)

[9 – Luaの完全な構文](09-complete-syntax.md)

---

## ❖ 索引

### Lua関数

#### 基本関数
[_G](06-standard-libraries.md#_G) · 
[_VERSION](06-standard-libraries.md#_VERSION) · 
[assert](06-standard-libraries.md#assert) · 
[collectgarbage](06-standard-libraries.md#collectgarbage) · 
[dofile](06-standard-libraries.md#dofile) · 
[error](06-standard-libraries.md#error) · 
[getmetatable](06-standard-libraries.md#getmetatable) · 
[ipairs](06-standard-libraries.md#ipairs) · 
[load](06-standard-libraries.md#load) · 
[loadfile](06-standard-libraries.md#loadfile) · 
[next](06-standard-libraries.md#next) · 
[pairs](06-standard-libraries.md#pairs) · 
[pcall](06-standard-libraries.md#pcall) · 
[print](06-standard-libraries.md#print) · 
[rawequal](06-standard-libraries.md#rawequal) · 
[rawget](06-standard-libraries.md#rawget) · 
[rawlen](06-standard-libraries.md#rawlen) · 
[rawset](06-standard-libraries.md#rawset) · 
[select](06-standard-libraries.md#select) · 
[setmetatable](06-standard-libraries.md#setmetatable) · 
[tonumber](06-standard-libraries.md#tonumber) · 
[tostring](06-standard-libraries.md#tostring) · 
[type](06-standard-libraries.md#type) · 
[warn](06-standard-libraries.md#warn) · 
[xpcall](06-standard-libraries.md#xpcall)

#### coroutine
[coroutine.close](06-standard-libraries.md#63--コルーチン操作) · 
[coroutine.create](06-standard-libraries.md#63--コルーチン操作) · 
[coroutine.isyieldable](06-standard-libraries.md#63--コルーチン操作) · 
[coroutine.resume](06-standard-libraries.md#63--コルーチン操作) · 
[coroutine.running](06-standard-libraries.md#63--コルーチン操作) · 
[coroutine.status](06-standard-libraries.md#63--コルーチン操作) · 
[coroutine.wrap](06-standard-libraries.md#63--コルーチン操作) · 
[coroutine.yield](06-standard-libraries.md#63--コルーチン操作)

#### debug
[debug.debug](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.gethook](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.getinfo](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.getlocal](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.getmetatable](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.getregistry](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.getupvalue](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.getuservalue](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.sethook](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.setlocal](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.setmetatable](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.setupvalue](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.setuservalue](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.traceback](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.upvalueid](06-standard-libraries.md#611--デバッグライブラリ) · 
[debug.upvaluejoin](06-standard-libraries.md#611--デバッグライブラリ)

#### io
[io.close](06-standard-libraries.md#69--入出力機能) · 
[io.flush](06-standard-libraries.md#69--入出力機能) · 
[io.input](06-standard-libraries.md#69--入出力機能) · 
[io.lines](06-standard-libraries.md#69--入出力機能) · 
[io.open](06-standard-libraries.md#69--入出力機能) · 
[io.output](06-standard-libraries.md#69--入出力機能) · 
[io.popen](06-standard-libraries.md#69--入出力機能) · 
[io.read](06-standard-libraries.md#69--入出力機能) · 
[io.stderr](06-standard-libraries.md#69--入出力機能) · 
[io.stdin](06-standard-libraries.md#69--入出力機能) · 
[io.stdout](06-standard-libraries.md#69--入出力機能) · 
[io.tmpfile](06-standard-libraries.md#69--入出力機能) · 
[io.type](06-standard-libraries.md#69--入出力機能) · 
[io.write](06-standard-libraries.md#69--入出力機能)

#### math
[math.abs](06-standard-libraries.md#68--数学関数) · 
[math.acos](06-standard-libraries.md#68--数学関数) · 
[math.asin](06-standard-libraries.md#68--数学関数) · 
[math.atan](06-standard-libraries.md#68--数学関数) · 
[math.ceil](06-standard-libraries.md#68--数学関数) · 
[math.cos](06-standard-libraries.md#68--数学関数) · 
[math.deg](06-standard-libraries.md#68--数学関数) · 
[math.exp](06-standard-libraries.md#68--数学関数) · 
[math.floor](06-standard-libraries.md#68--数学関数) · 
[math.fmod](06-standard-libraries.md#68--数学関数) · 
[math.huge](06-standard-libraries.md#68--数学関数) · 
[math.log](06-standard-libraries.md#68--数学関数) · 
[math.max](06-standard-libraries.md#68--数学関数) · 
[math.maxinteger](06-standard-libraries.md#68--数学関数) · 
[math.min](06-standard-libraries.md#68--数学関数) · 
[math.mininteger](06-standard-libraries.md#68--数学関数) · 
[math.modf](06-standard-libraries.md#68--数学関数) · 
[math.pi](06-standard-libraries.md#68--数学関数) · 
[math.rad](06-standard-libraries.md#68--数学関数) · 
[math.random](06-standard-libraries.md#68--数学関数) · 
[math.randomseed](06-standard-libraries.md#68--数学関数) · 
[math.sin](06-standard-libraries.md#68--数学関数) · 
[math.sqrt](06-standard-libraries.md#68--数学関数) · 
[math.tan](06-standard-libraries.md#68--数学関数) · 
[math.tointeger](06-standard-libraries.md#68--数学関数) · 
[math.type](06-standard-libraries.md#68--数学関数) · 
[math.ult](06-standard-libraries.md#68--数学関数)

#### os
[os.clock](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.date](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.difftime](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.execute](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.exit](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.getenv](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.remove](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.rename](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.setlocale](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.time](06-standard-libraries.md#610--オペレーティングシステム機能) · 
[os.tmpname](06-standard-libraries.md#610--オペレーティングシステム機能)

#### package
[package.config](06-standard-libraries.md#64--モジュール) · 
[package.cpath](06-standard-libraries.md#64--モジュール) · 
[package.loaded](06-standard-libraries.md#64--モジュール) · 
[package.loadlib](06-standard-libraries.md#64--モジュール) · 
[package.path](06-standard-libraries.md#64--モジュール) · 
[package.preload](06-standard-libraries.md#64--モジュール) · 
[package.searchers](06-standard-libraries.md#64--モジュール) · 
[package.searchpath](06-standard-libraries.md#64--モジュール)

#### string
[string.byte](06-standard-libraries.md#65--文字列操作) · 
[string.char](06-standard-libraries.md#65--文字列操作) · 
[string.dump](06-standard-libraries.md#65--文字列操作) · 
[string.find](06-standard-libraries.md#65--文字列操作) · 
[string.format](06-standard-libraries.md#65--文字列操作) · 
[string.gmatch](06-standard-libraries.md#65--文字列操作) · 
[string.gsub](06-standard-libraries.md#65--文字列操作) · 
[string.len](06-standard-libraries.md#65--文字列操作) · 
[string.lower](06-standard-libraries.md#65--文字列操作) · 
[string.match](06-standard-libraries.md#65--文字列操作) · 
[string.pack](06-standard-libraries.md#65--文字列操作) · 
[string.packsize](06-standard-libraries.md#65--文字列操作) · 
[string.rep](06-standard-libraries.md#65--文字列操作) · 
[string.reverse](06-standard-libraries.md#65--文字列操作) · 
[string.sub](06-standard-libraries.md#65--文字列操作) · 
[string.unpack](06-standard-libraries.md#65--文字列操作) · 
[string.upper](06-standard-libraries.md#65--文字列操作)

#### table
[table.concat](06-standard-libraries.md#67--テーブル操作) · 
[table.create](06-standard-libraries.md#67--テーブル操作) · 
[table.insert](06-standard-libraries.md#67--テーブル操作) · 
[table.move](06-standard-libraries.md#67--テーブル操作) · 
[table.pack](06-standard-libraries.md#67--テーブル操作) · 
[table.remove](06-standard-libraries.md#67--テーブル操作) · 
[table.sort](06-standard-libraries.md#67--テーブル操作) · 
[table.unpack](06-standard-libraries.md#67--テーブル操作)

#### utf8
[utf8.char](06-standard-libraries.md#66--utf-8サポート) · 
[utf8.charpattern](06-standard-libraries.md#66--utf-8サポート) · 
[utf8.codepoint](06-standard-libraries.md#66--utf-8サポート) · 
[utf8.codes](06-standard-libraries.md#66--utf-8サポート) · 
[utf8.len](06-standard-libraries.md#66--utf-8サポート) · 
[utf8.offset](06-standard-libraries.md#66--utf-8サポート)

---

### メタメソッド

[__add](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__band](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__bnot](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__bor](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__bxor](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__call](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__close](03-language.md#338--クローズされる変数) · 
[__concat](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__div](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__eq](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__gc](02-basic-concepts.md#25--ガベージコレクション) · 
[__idiv](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__index](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__le](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__len](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__lt](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__mod](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__mode](02-basic-concepts.md#25--ガベージコレクション) · 
[__mul](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__name](06-standard-libraries.md#62--基本関数) · 
[__newindex](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__pairs](06-standard-libraries.md#62--基本関数) · 
[__pow](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__shl](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__shr](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__sub](02-basic-concepts.md#24--メタテーブルとメタメソッド) · 
[__tostring](06-standard-libraries.md#62--基本関数) · 
[__unm](02-basic-concepts.md#24--メタテーブルとメタメソッド)

---

### 環境変数

[LUA_CPATH](06-standard-libraries.md#64--モジュール) · 
[LUA_CPATH_5_5](06-standard-libraries.md#64--モジュール) · 
[LUA_INIT](07-standalone.md) · 
[LUA_INIT_5_5](07-standalone.md) · 
[LUA_PATH](06-standard-libraries.md#64--モジュール) · 
[LUA_PATH_5_5](06-standard-libraries.md#64--モジュール)

---

### C API

#### 型
[lua_Alloc](04-c-api.md#46--関数と型) · 
[lua_CFunction](04-c-api.md#46--関数と型) · 
[lua_Debug](04-c-api.md#47--デバッグインターフェース) · 
[lua_Hook](04-c-api.md#47--デバッグインターフェース) · 
[lua_Integer](04-c-api.md#46--関数と型) · 
[lua_KContext](04-c-api.md#45--cでのyieldの処理) · 
[lua_KFunction](04-c-api.md#45--cでのyieldの処理) · 
[lua_Number](04-c-api.md#46--関数と型) · 
[lua_Reader](04-c-api.md#46--関数と型) · 
[lua_State](04-c-api.md#46--関数と型) · 
[lua_Unsigned](04-c-api.md#46--関数と型) · 
[lua_WarnFunction](04-c-api.md#46--関数と型) · 
[lua_Writer](04-c-api.md#46--関数と型)

#### 主要関数
[lua_call](04-c-api.md#46--関数と型) · 
[lua_checkstack](04-c-api.md#411--スタックサイズ) · 
[lua_close](04-c-api.md#46--関数と型) · 
[lua_error](04-c-api.md#44--cでのエラー処理) · 
[lua_gc](04-c-api.md#46--関数と型) · 
[lua_getfield](04-c-api.md#46--関数と型) · 
[lua_getglobal](04-c-api.md#46--関数と型) · 
[lua_gettable](04-c-api.md#46--関数と型) · 
[lua_gettop](04-c-api.md#41--スタック) · 
[lua_newstate](04-c-api.md#46--関数と型) · 
[lua_newthread](04-c-api.md#46--関数と型) · 
[lua_pcall](04-c-api.md#46--関数と型) · 
[lua_pop](04-c-api.md#41--スタック) · 
[lua_pushcclosure](04-c-api.md#42--cクロージャ) · 
[lua_pushinteger](04-c-api.md#46--関数と型) · 
[lua_pushlstring](04-c-api.md#413--文字列へのポインタ) · 
[lua_pushnil](04-c-api.md#46--関数と型) · 
[lua_pushnumber](04-c-api.md#46--関数と型) · 
[lua_pushstring](04-c-api.md#413--文字列へのポインタ) · 
[lua_setfield](04-c-api.md#46--関数と型) · 
[lua_setglobal](04-c-api.md#46--関数と型) · 
[lua_settable](04-c-api.md#46--関数と型) · 
[lua_settop](04-c-api.md#41--スタック) · 
[lua_toboolean](04-c-api.md#46--関数と型) · 
[lua_tointeger](04-c-api.md#46--関数と型) · 
[lua_tolstring](04-c-api.md#413--文字列へのポインタ) · 
[lua_tonumber](04-c-api.md#46--関数と型) · 
[lua_type](04-c-api.md#46--関数と型) · 
[lua_upvalueindex](04-c-api.md#42--cクロージャ) · 
[lua_yieldk](04-c-api.md#45--cでのyieldの処理)

---

### 補助ライブラリ

#### 型
[luaL_Buffer](05-auxiliary-library.md#51--関数と型) · 
[luaL_Reg](05-auxiliary-library.md#51--関数と型) · 
[luaL_Stream](05-auxiliary-library.md#51--関数と型)

#### 主要関数
[luaL_argcheck](05-auxiliary-library.md#51--関数と型) · 
[luaL_argerror](05-auxiliary-library.md#51--関数と型) · 
[luaL_buffinit](05-auxiliary-library.md#51--関数と型) · 
[luaL_checkinteger](05-auxiliary-library.md#51--関数と型) · 
[luaL_checklstring](05-auxiliary-library.md#51--関数と型) · 
[luaL_checknumber](05-auxiliary-library.md#51--関数と型) · 
[luaL_checkstring](05-auxiliary-library.md#51--関数と型) · 
[luaL_checktype](05-auxiliary-library.md#51--関数と型) · 
[luaL_checkudata](05-auxiliary-library.md#51--関数と型) · 
[luaL_dofile](05-auxiliary-library.md#51--関数と型) · 
[luaL_dostring](05-auxiliary-library.md#51--関数と型) · 
[luaL_error](05-auxiliary-library.md#51--関数と型) · 
[luaL_loadbuffer](05-auxiliary-library.md#51--関数と型) · 
[luaL_loadfile](05-auxiliary-library.md#51--関数と型) · 
[luaL_loadstring](05-auxiliary-library.md#51--関数と型) · 
[luaL_newlib](05-auxiliary-library.md#51--関数と型) · 
[luaL_newmetatable](05-auxiliary-library.md#51--関数と型) · 
[luaL_newstate](05-auxiliary-library.md#51--関数と型) · 
[luaL_openlibs](05-auxiliary-library.md#51--関数と型) · 
[luaL_openselectedlibs](05-auxiliary-library.md#51--関数と型) ⭐ *5.5新規* · 
[luaL_optinteger](05-auxiliary-library.md#51--関数と型) · 
[luaL_optnumber](05-auxiliary-library.md#51--関数と型) · 
[luaL_optstring](05-auxiliary-library.md#51--関数と型) · 
[luaL_pushresult](05-auxiliary-library.md#51--関数と型) · 
[luaL_ref](05-auxiliary-library.md#51--関数と型) · 
[luaL_unref](05-auxiliary-library.md#51--関数と型)

---

### 定数

[LUA_ERRERR](04-c-api.md#441--ステータスコード) · 
[LUA_ERRFILE](04-c-api.md#441--ステータスコード) · 
[LUA_ERRMEM](04-c-api.md#441--ステータスコード) · 
[LUA_ERRRUN](04-c-api.md#441--ステータスコード) · 
[LUA_ERRSYNTAX](04-c-api.md#441--ステータスコード) · 
[LUA_MINSTACK](04-c-api.md#411--スタックサイズ) · 
[LUA_MULTRET](04-c-api.md#46--関数と型) · 
[LUA_OK](04-c-api.md#441--ステータスコード) · 
[LUA_REGISTRYINDEX](04-c-api.md#43--レジストリ) · 
[LUA_RIDX_GLOBALS](04-c-api.md#43--レジストリ) · 
[LUA_RIDX_MAINTHREAD](04-c-api.md#43--レジストリ) · 
[LUA_YIELD](04-c-api.md#441--ステータスコード)

---

### 標準ライブラリ定数 ⭐ *5.5新規*

[LUA_COLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_DBLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_GLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_IOLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_LOADLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_MATHLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_OSLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_STRLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_TABLIBK](06-standard-libraries.md#61--cコードでのライブラリロード) · 
[LUA_UTF8LIBK](06-standard-libraries.md#61--cコードでのライブラリロード)

---

## 付録

| ドキュメント | 説明 |
|-------------|------|
| [ABOUT.md](ABOUT.md) | 翻訳について・利用案内 |
| [GLOSSARY.md](GLOSSARY.md) | 用語対応表 |
| [LICENSE.md](LICENSE.md) | ライセンス情報 |

---

## 翻訳について

翻訳についての詳細は[ABOUT.md](ABOUT.md)を参照してください。

---

*最終更新: 2026年1月29日*

*Copyright © 1994–2025 Lua.org, PUC-Rio.*  
*[Lua license](LICENSE.md) の条件に基づき自由に利用可能。*
