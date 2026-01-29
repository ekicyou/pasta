# Lua 5.4 リファレンスマニュアル（日本語版）参照文書

> **Source**: https://lua.dokyumento.jp/manual/5.4/manual.html
> **Purpose**: Lua 5.5翻訳時の技術用語参考文献
> **Copyright**: Copyright © 2020–2023 Lua.org, PUC-Rio. Luaライセンスの条項に基づき自由に使用できます。

---

## 1 – はじめに

Luaは、強力で、効率的で、軽量で、組み込み可能なスクリプト言語です。手続き型プログラミング、オブジェクト指向プログラミング、関数型プログラミング、データ駆動型プログラミング、およびデータ記述をサポートしています。

Luaは、単純な手続き型構文と、連想配列と拡張可能なセマンティクスに基づいた強力なデータ記述構造を組み合わせています。Luaは動的型付けであり、レジスタベースの仮想マシンでバイトコードを解釈して実行し、世代別ガベージコレクションによる自動メモリ管理を備えているため、構成、スクリプト作成、および迅速なプロトタイピングに最適です。

Luaは、標準CとC++の共通サブセットであるクリーンCで記述されたライブラリとして実装されています。Luaディストリビューションには、Luaライブラリを使用して完全なスタンドアロンのLuaインタープリターを提供するホストプログラム`lua`が含まれており、対話型またはバッチで使用できます。

Luaはフリーソフトウェアであり、そのライセンスに記載されているように、通常、保証なしで提供されます。

---

## 2 – 基本概念

### 2.1 – 値と型

Luaは動的型付け言語です。これは、変数には型がなく、値にのみ型があることを意味します。言語に型定義はありません。すべての値は独自の型を持ちます。

Luaのすべての値は、第一級の値です。これは、すべての値を変数に格納したり、他の関数に引数として渡したり、結果として返したりできることを意味します。

Luaには、**nil**、**boolean**、**number**、**string**、**function**、**userdata**、**thread**、**table**の8つの基本型があります。

- **nil**: 他の任意の値とは異なることを主な特性とする単一の値nil。有用な値の欠如を表します。
- **boolean**: falseとtrueの2つの値。nilとfalseの両方が条件を偽にします（偽の値）。
- **number**: integerとfloatの2つのサブタイプ。標準Luaは64ビット整数と倍精度浮動小数点を使用。
- **string**: 不変のバイトシーケンス。8ビットクリーン。
- **function**: Luaで記述された関数とCで記述された関数の両方を表す。
- **userdata**: 任意のCデータをLua変数に格納できるようにする。フルuserdataとライトuserdata。
- **thread**: 独立した実行スレッド。コルーチンの実装に使用。
- **table**: Luaの唯一のデータ構造メカニズム。連想配列。

### 2.2 – 環境とグローバル環境

Luaは、グローバル環境と呼ばれる特別な環境を保持します。この値は、Cレジストリの特別なインデックスに保持されます。Luaでは、グローバル変数`_G`はこの同じ値で初期化されます。

### 2.4 – メタテーブルとメタメソッド

主要なメタメソッド:
- `__add`: 加算 (`+`)
- `__sub`: 減算 (`-`)
- `__mul`: 乗算 (`*`)
- `__div`: 除算 (`/`)
- `__mod`: 剰余 (`%`)
- `__pow`: べき乗 (`^`)
- `__unm`: 単項マイナス (`-`)
- `__idiv`: 床関数除算 (`//`)
- `__band`: ビット論理積 (`&`)
- `__bor`: ビット論理和 (`|`)
- `__bxor`: ビット排他的論理和 (`~`)
- `__bnot`: ビット否定 (`~`)
- `__shl`: 左シフト (`<<`)
- `__shr`: 右シフト (`>>`)
- `__concat`: 連結 (`..`)
- `__len`: 長さ (`#`)
- `__eq`: 等しい (`==`)
- `__lt`: より小さい (`<`)
- `__le`: 以下 (`<=`)
- `__index`: インデックスアクセス (`table[key]`)
- `__newindex`: インデックス代入
- `__call`: 関数呼び出し

---

## 3 – 言語

### 3.1 – 字句規則

Luaは自由形式の言語です。

**予約語**:
```
and       break     do        else      elseif    end
false     for       function  goto      if        in
local     nil       not       or        repeat    return
then      true      until     while
```

**演算子と区切り文字**:
```
+     -     *     /     %     ^     #
&     ~     |     <<    >>    //
==    ~=    <=    >=    <     >     =
(     )     {     }     [     ]     ::
;     :     ,     .     ..    ...
```

### 3.2 – 変数

変数は値を格納する場所です。Luaには、グローバル変数、ローカル変数、およびテーブルフィールドの3種類の変数があります。

### 3.3 – ステートメント

#### 3.3.4 – 制御構造

```lua
-- while文
while exp do block end

-- repeat文
repeat block until exp

-- if文
if exp then block {elseif exp then block} [else block] end

-- 数値for文
for Name = exp, exp [, exp] do block end

-- 汎用for文
for namelist in explist do block end
```

### 3.4 – 式

#### 3.4.1 – 算術演算子

- `+`: 加算
- `-`: 減算
- `*`: 乗算
- `/`: 浮動小数点除算
- `//`: 床関数除算
- `%`: 剰余
- `^`: べき乗
- `-`: 単項マイナス

#### 3.4.5 – 論理演算子

Luaの論理演算子は、`and`、`or`、および`not`です。

---

## 4 – アプリケーションプログラミングインターフェイス

### 4.1 – スタック

Luaは、値をCとの間で受け渡すために、仮想スタックを使用します。このスタック内の各要素は、Lua値（nil、数値、文字列など）を表します。

### 4.3 – レジストリ

Luaは、任意のCコードが必要なLua値を格納するために使用できる事前定義されたテーブルであるレジストリを提供します。

### 4.6 – 関数と型（主要なもの）

| 関数 | 説明 |
|------|------|
| `lua_absindex` | 許容可能なインデックスを絶対インデックスに変換 |
| `lua_call` | 関数を呼び出す |
| `lua_checkstack` | スタックサイズを拡張 |
| `lua_close` | Lua状態を閉じる |
| `lua_compare` | 2つのLua値を比較 |
| `lua_createtable` | 新しい空のテーブルを作成 |
| `lua_error` | Luaエラーを発生させる |
| `lua_gc` | ガベージコレクタを制御 |
| `lua_getfield` | `t[k]`の値をプッシュ |
| `lua_getglobal` | グローバル変数の値をプッシュ |
| `lua_gettable` | テーブルから値を取得 |
| `lua_gettop` | スタックの最上部のインデックスを返す |
| `lua_isboolean` | 値がブール値かどうかをチェック |
| `lua_isfunction` | 値が関数かどうかをチェック |
| `lua_isnil` | 値がnilかどうかをチェック |
| `lua_isnumber` | 値が数値かどうかをチェック |
| `lua_isstring` | 値が文字列かどうかをチェック |
| `lua_istable` | 値がテーブルかどうかをチェック |
| `lua_len` | 値の長さを返す |
| `lua_newtable` | 新しい空のテーブルを作成 |
| `lua_newthread` | 新しいスレッドを作成 |
| `lua_next` | テーブルのトラバース |
| `lua_pcall` | 保護モードで関数を呼び出す |
| `lua_pop` | スタックから要素をポップ |
| `lua_pushboolean` | ブール値をプッシュ |
| `lua_pushinteger` | 整数をプッシュ |
| `lua_pushnil` | nilをプッシュ |
| `lua_pushnumber` | 浮動小数点数をプッシュ |
| `lua_pushstring` | 文字列をプッシュ |
| `lua_rawget` | 生のテーブルアクセス |
| `lua_rawset` | 生のテーブル代入 |
| `lua_setfield` | `t[k] = v`を設定 |
| `lua_setglobal` | グローバル変数を設定 |
| `lua_settable` | テーブルに値を設定 |
| `lua_settop` | スタックの最上部を設定 |
| `lua_toboolean` | 値をブール値に変換 |
| `lua_tointeger` | 値を整数に変換 |
| `lua_tolstring` | 値を文字列に変換 |
| `lua_tonumber` | 値を数値に変換 |
| `lua_tostring` | 値をC文字列に変換 |
| `lua_type` | 値の型を返す |
| `lua_typename` | 型の名前を返す |

---

## 5 – 補助ライブラリ

補助ライブラリは、CとLuaをインターフェイスするための便利な関数をいくつか提供します。すべての関数と型は、ヘッダーファイル`lauxlib.h`で定義され、接頭辞`luaL_`が付いています。

### 主要な関数

| 関数 | 説明 |
|------|------|
| `luaL_checkinteger` | 引数が整数かどうかをチェック |
| `luaL_checknumber` | 引数が数値かどうかをチェック |
| `luaL_checkstring` | 引数が文字列かどうかをチェック |
| `luaL_checktype` | 引数の型をチェック |
| `luaL_checkudata` | 引数がuserdataかどうかをチェック |
| `luaL_dofile` | ファイルをロードして実行 |
| `luaL_dostring` | 文字列をロードして実行 |
| `luaL_error` | エラーを発生させる |
| `luaL_loadfile` | ファイルをロード |
| `luaL_loadstring` | 文字列をロード |
| `luaL_newlib` | 新しいライブラリテーブルを作成 |
| `luaL_newmetatable` | 新しいメタテーブルを作成 |
| `luaL_newstate` | 新しいLua状態を作成 |
| `luaL_openlibs` | すべての標準ライブラリを開く |
| `luaL_ref` | 参照を作成 |
| `luaL_unref` | 参照を解放 |

---

## 6 – 標準ライブラリ

### 6.1 – 基本関数

| 関数 | 説明 |
|------|------|
| `assert` | アサーション |
| `collectgarbage` | ガベージコレクション制御 |
| `dofile` | ファイルを実行 |
| `error` | エラーを発生 |
| `getmetatable` | メタテーブルを取得 |
| `ipairs` | 整数キーイテレータ |
| `load` | チャンクをロード |
| `loadfile` | ファイルをロード |
| `next` | テーブルトラバース |
| `pairs` | テーブルイテレータ |
| `pcall` | 保護モード呼び出し |
| `print` | 出力 |
| `rawequal` | 生の等価比較 |
| `rawget` | 生のテーブルアクセス |
| `rawlen` | 生の長さ |
| `rawset` | 生のテーブル代入 |
| `select` | 引数選択 |
| `setmetatable` | メタテーブルを設定 |
| `tonumber` | 数値に変換 |
| `tostring` | 文字列に変換 |
| `type` | 型を取得 |
| `warn` | 警告を発行 |
| `xpcall` | 拡張保護モード呼び出し |

### 6.2 – コルーチン操作 (coroutine)

| 関数 | 説明 |
|------|------|
| `coroutine.create` | コルーチンを作成 |
| `coroutine.isyieldable` | yieldできるかチェック |
| `coroutine.resume` | コルーチンを再開 |
| `coroutine.running` | 実行中のコルーチンを取得 |
| `coroutine.status` | コルーチンの状態を取得 |
| `coroutine.wrap` | コルーチンをラップ |
| `coroutine.yield` | コルーチンを中断 |

### 6.3 – モジュール (package)

| 関数/変数 | 説明 |
|-----------|------|
| `require` | モジュールをロード |
| `package.config` | パッケージ構成文字列 |
| `package.cpath` | Cローダーの検索パス |
| `package.loaded` | ロード済みモジュール |
| `package.loadlib` | Cライブラリをロード |
| `package.path` | Luaローダーの検索パス |
| `package.preload` | プリロードテーブル |
| `package.searchers` | 検索関数のリスト |
| `package.searchpath` | パスを検索 |

### 6.4 – 文字列操作 (string)

| 関数 | 説明 |
|------|------|
| `string.byte` | 文字コードを取得 |
| `string.char` | 文字コードから文字列を作成 |
| `string.dump` | 関数をバイナリにダンプ |
| `string.find` | パターン検索 |
| `string.format` | 書式付き文字列 |
| `string.gmatch` | グローバルパターンマッチ |
| `string.gsub` | グローバル置換 |
| `string.len` | 文字列長 |
| `string.lower` | 小文字に変換 |
| `string.match` | パターンマッチ |
| `string.pack` | バイナリパック |
| `string.packsize` | パックサイズ計算 |
| `string.rep` | 文字列を繰り返し |
| `string.reverse` | 文字列を逆順 |
| `string.sub` | 部分文字列 |
| `string.unpack` | バイナリアンパック |
| `string.upper` | 大文字に変換 |

### 6.5 – UTF-8サポート (utf8)

| 関数/定数 | 説明 |
|-----------|------|
| `utf8.char` | コードポイントから文字列 |
| `utf8.charpattern` | UTF-8文字パターン |
| `utf8.codepoint` | コードポイントを取得 |
| `utf8.codes` | コードポイントイテレータ |
| `utf8.len` | UTF-8文字列長 |
| `utf8.offset` | オフセット計算 |

### 6.6 – テーブル操作 (table)

| 関数 | 説明 |
|------|------|
| `table.concat` | テーブルを連結 |
| `table.insert` | 要素を挿入 |
| `table.move` | 要素を移動 |
| `table.pack` | 引数をテーブルにパック |
| `table.remove` | 要素を削除 |
| `table.sort` | ソート |
| `table.unpack` | テーブルをアンパック |

### 6.7 – 数学関数 (math)

| 関数/定数 | 説明 |
|-----------|------|
| `math.abs` | 絶対値 |
| `math.acos` | 逆余弦 |
| `math.asin` | 逆正弦 |
| `math.atan` | 逆正接 |
| `math.ceil` | 切り上げ |
| `math.cos` | 余弦 |
| `math.deg` | 度に変換 |
| `math.exp` | 指数関数 |
| `math.floor` | 切り捨て |
| `math.fmod` | 浮動小数点剰余 |
| `math.huge` | 無限大 |
| `math.log` | 対数 |
| `math.max` | 最大値 |
| `math.maxinteger` | 最大整数 |
| `math.min` | 最小値 |
| `math.mininteger` | 最小整数 |
| `math.modf` | 整数部と小数部 |
| `math.pi` | 円周率 |
| `math.rad` | ラジアンに変換 |
| `math.random` | 乱数 |
| `math.randomseed` | 乱数シード |
| `math.sin` | 正弦 |
| `math.sqrt` | 平方根 |
| `math.tan` | 正接 |
| `math.tointeger` | 整数に変換 |
| `math.type` | 数値型を取得 |
| `math.ult` | 符号なし比較 |

### 6.8 – 入出力 (io)

| 関数 | 説明 |
|------|------|
| `io.close` | ファイルを閉じる |
| `io.flush` | バッファをフラッシュ |
| `io.input` | 入力ファイルを設定 |
| `io.lines` | 行イテレータ |
| `io.open` | ファイルを開く |
| `io.output` | 出力ファイルを設定 |
| `io.popen` | プロセスを開く |
| `io.read` | 読み込み |
| `io.tmpfile` | 一時ファイル |
| `io.type` | ファイルタイプ |
| `io.write` | 書き込み |

### 6.9 – OS機能 (os)

| 関数 | 説明 |
|------|------|
| `os.clock` | CPU時間 |
| `os.date` | 日付/時刻 |
| `os.difftime` | 時間差 |
| `os.execute` | コマンド実行 |
| `os.exit` | 終了 |
| `os.getenv` | 環境変数取得 |
| `os.remove` | ファイル削除 |
| `os.rename` | ファイル名変更 |
| `os.setlocale` | ロケール設定 |
| `os.time` | 時刻取得 |
| `os.tmpname` | 一時ファイル名 |

### 6.10 – デバッグ (debug)

| 関数 | 説明 |
|------|------|
| `debug.debug` | 対話デバッガ |
| `debug.gethook` | フック取得 |
| `debug.getinfo` | 関数情報取得 |
| `debug.getlocal` | ローカル変数取得 |
| `debug.getmetatable` | メタテーブル取得 |
| `debug.getregistry` | レジストリ取得 |
| `debug.getupvalue` | アップバリュー取得 |
| `debug.getuservalue` | ユーザー値取得 |
| `debug.sethook` | フック設定 |
| `debug.setlocal` | ローカル変数設定 |
| `debug.setmetatable` | メタテーブル設定 |
| `debug.setupvalue` | アップバリュー設定 |
| `debug.setuservalue` | ユーザー値設定 |
| `debug.traceback` | トレースバック |
| `debug.upvalueid` | アップバリューID |
| `debug.upvaluejoin` | アップバリュー結合 |

---

## 9 – Luaの完全な構文

```ebnf
chunk ::= block

block ::= {stat} [retstat]

stat ::=  ';' | 
     varlist '=' explist | 
     functioncall | 
     label | 
     break | 
     goto Name | 
     do block end | 
     while exp do block end | 
     repeat block until exp | 
     if exp then block {elseif exp then block} [else block] end | 
     for Name '=' exp ',' exp [',' exp] do block end | 
     for namelist in explist do block end | 
     function funcname funcbody | 
     local function Name funcbody | 
     local attnamelist ['=' explist] 

attnamelist ::=  Name attrib {',' Name attrib}

attrib ::= ['<' Name '>']

retstat ::= return [explist] [';']

label ::= '::' Name '::'

funcname ::= Name {'.' Name} [':' Name]

varlist ::= var {',' var}

var ::=  Name | prefixexp '[' exp ']' | prefixexp '.' Name 

namelist ::= Name {',' Name}

explist ::= exp {',' exp}

exp ::=  nil | false | true | Numeral | LiteralString | '...' | functiondef | 
     prefixexp | tableconstructor | exp binop exp | unop exp

prefixexp ::= var | functioncall | '(' exp ')'

functioncall ::=  prefixexp args | prefixexp ':' Name args 

args ::=  '(' [explist] ')' | tableconstructor | LiteralString 

functiondef ::= function funcbody

funcbody ::= '(' [parlist] ')' block end

parlist ::= namelist [',' '...'] | '...'

tableconstructor ::= '{' [fieldlist] '}'

fieldlist ::= field {fieldsep field} [fieldsep]

field ::= '[' exp ']' '=' exp | Name '=' exp | exp

fieldsep ::= ',' | ';'

binop ::=  '+' | '-' | '*' | '/' | '//' | '^' | '%' | 
     '&' | '~' | '|' | '>>' | '<<' | '..' | 
     '<' | '<=' | '>' | '>=' | '==' | '~=' | 
     and | or

unop ::= '-' | not | '#' | '~'
```

---

最終更新日: 2023年5月2日
