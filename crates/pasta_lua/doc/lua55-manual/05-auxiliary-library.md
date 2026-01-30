[← 前へ: 4 – C API](04-c-api.md) | [目次](./README.md) | [次へ: 6 – 標準ライブラリ →](06-standard-libraries.md)

---

<!--
  原文: https://www.lua.org/manual/5.5/manual.html#5
  参考: https://lua.dokyumento.jp/manual/5.4/manual.html#5
  翻訳日: 2026-01-29
  レビュー: AI Claude Opus 4.5
  用語対照: GLOSSARY.md参照
-->

# 5 – 補助ライブラリ

---

*補助ライブラリ*は、CとLuaをインターフェイスするための便利な関数をいくつか提供します。基本的なAPIが、CとLua間のすべての相互作用のためのプリミティブ関数を提供する一方、補助ライブラリはいくつかの一般的なタスクのためのより高レベルの関数を提供します。

補助ライブラリのすべての関数と型は、ヘッダーファイル`lauxlib.h`で定義され、接頭辞`luaL_`が付いています。

補助ライブラリのすべての関数は、基本的なAPIの上に構築されているため、そのAPIではできないことは何も提供しません。それにもかかわらず、補助ライブラリを使用すると、コードの一貫性が向上します。

補助ライブラリのいくつかの関数は、内部的にいくつかの追加のスタックスロットを使用します。補助ライブラリの関数が5つ未満のスロットを使用する場合、スタックサイズをチェックしません。十分なスロットがあると単純に想定します。

補助ライブラリのいくつかの関数は、C関数の引数をチェックするために使用されます。エラーメッセージは引数用にフォーマットされているため（例："`bad argument #1`"）、これらの関数を他のスタック値に使用しないでください。

`luaL_check*`と呼ばれる関数は、チェックが満たされない場合は常にエラーを発生させます。

---

## 5.1 – 関数と型

ここでは、補助ライブラリのすべての関数と型をアルファベット順にリストします。

> **Note**: 関数リファレンスは膨大なため、ここでは主要なカテゴリと代表的な関数の概要を示します。
> 完全なリファレンスは英語版原文を参照してください。

---

### バッファ操作関数

文字列バッファは、CコードでLua文字列を段階的に構築するための機能です。

#### luaL_Buffer

```c
typedef struct luaL_Buffer luaL_Buffer;
```

*文字列バッファ*の型です。

文字列バッファを使用すると、CコードでLua文字列を段階的に構築できます。その使用パターンは以下のとおりです：

1. まず、`luaL_Buffer`型の変数`b`を宣言します。
2. 次に、`luaL_buffinit(L, &b)`を呼び出して初期化します。
3. 次に、`luaL_add*`関数を呼び出して、文字列の断片をバッファに追加します。
4. 最後に、`luaL_pushresult(&b)`を呼び出して完了します。この呼び出しは、最終的な文字列をスタックの最上位に残します。

結果の文字列の最大サイズを事前に知っている場合は、次のようにバッファを使用できます：

1. まず、`luaL_Buffer`型の変数`b`を宣言します。
2. 次に、`luaL_buffinitsize(L, &b, sz)`を呼び出して初期化し、サイズ`sz`の領域を事前割り当てします。
3. 次に、その領域に文字列を生成します。
4. 最後に、`luaL_pushresultsize(&b, sz)`を呼び出して完了します。`sz`は、その領域にコピーされた結果の文字列の合計サイズです。

通常の操作中、文字列バッファは可変数のスタックスロットを使用します。そのため、バッファを使用している間は、スタックの最上位がどこにあるかわからないことを前提とする必要があります。

| 関数 | 説明 |
|------|------|
| `luaL_addchar` | バイトをバッファに追加 |
| `luaL_addgsub` | 置換を行いながら文字列をバッファに追加 |
| `luaL_addlstring` | 長さ指定で文字列を追加 |
| `luaL_addsize` | バッファ領域からの追加を確定 |
| `luaL_addstring` | ゼロ終端文字列を追加 |
| `luaL_addvalue` | スタック最上位の値を追加 |
| `luaL_buffaddr` | バッファ内容のアドレスを取得 |
| `luaL_buffinit` | バッファを初期化 |
| `luaL_bufflen` | バッファ内容の長さを取得 |
| `luaL_buffinitsize` | サイズ指定でバッファを初期化 |
| `luaL_buffsub` | バッファからバイトを削除 |
| `luaL_prepbuffer` | 定義済みサイズの領域を取得 |
| `luaL_prepbuffsize` | 指定サイズの領域を取得 |
| `luaL_pushresult` | バッファ使用を終了し文字列をプッシュ |
| `luaL_pushresultsize` | サイズ指定でバッファ使用を終了 |

---

### 引数チェック関数

C関数の引数を検証するための関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_argcheck` | 条件が真であるかチェック |
| `luaL_argerror` | 引数エラーを発生 |
| `luaL_argexpected` | 型エラーを発生 |
| `luaL_checkany` | 任意の型の引数があるかチェック |
| `luaL_checkinteger` | 整数であるかチェックして返す |
| `luaL_checklstring` | 文字列であるかチェックして長さ付きで返す |
| `luaL_checknumber` | 数値であるかチェックして返す |
| `luaL_checkoption` | 文字列が選択肢リストにあるかチェック |
| `luaL_checkstack` | スタックサイズを確保 |
| `luaL_checkstring` | 文字列であるかチェックして返す |
| `luaL_checktype` | 指定した型であるかチェック |
| `luaL_checkudata` | ユーザーデータであるかチェック |
| `luaL_checkversion` | Luaバージョンの一致をチェック |

---

### オプショナル引数関数

引数が省略可能な場合に使用する関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_opt` | 引数がnilまたは不在ならデフォルトを返すマクロ |
| `luaL_optinteger` | 整数またはデフォルトを返す |
| `luaL_optlstring` | 文字列またはデフォルトを返す |
| `luaL_optnumber` | 数値またはデフォルトを返す |
| `luaL_optstring` | 文字列またはデフォルトを返す |

---

### ロード・実行関数

Luaコードのロードと実行を行う関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_dofile` | ファイルをロードして実行 |
| `luaL_dostring` | 文字列をロードして実行 |
| `luaL_loadbuffer` | バッファをチャンクとしてロード（mode=NULL） |
| `luaL_loadbufferx` | バッファをチャンクとしてロード（モード指定） |
| `luaL_loadfile` | ファイルをチャンクとしてロード（mode=NULL） |
| `luaL_loadfilex` | ファイルをチャンクとしてロード（モード指定） |
| `luaL_loadstring` | 文字列をチャンクとしてロード |

#### luaL_dofile

```c
int luaL_dofile (lua_State *L, const char *filename);
```

指定されたファイルをロードして実行します。これは、次のマクロとして定義されます：

```c
(luaL_loadfile(L, filename) || lua_pcall(L, 0, LUA_MULTRET, 0))
```

エラーがない場合は0（`LUA_OK`）を返し、エラーの場合は1を返します。
（メモリ不足エラーは例外として発生させます。）

---

### メタテーブル操作関数

メタテーブルの取得・設定を行う関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_callmeta` | メタメソッドを呼び出す |
| `luaL_getmetafield` | メタテーブルからフィールドを取得 |
| `luaL_getmetatable` | 名前でメタテーブルを取得 |
| `luaL_newmetatable` | 新しいメタテーブルを作成 |
| `luaL_setmetatable` | メタテーブルを設定 |

#### luaL_newmetatable

```c
int luaL_newmetatable (lua_State *L, const char *tname);
```

レジストリにすでにキー`tname`がある場合は0を返します。そうでない場合は、userdataのメタテーブルとして使用する新しいテーブルを作成し、この新しいテーブルにペア`__name = tname`を追加し、レジストリにペア`[tname] = new table`を追加して1を返します。

どちらの場合も、この関数は、レジストリ内の`tname`に関連付けられた最終的な値をスタックにプッシュします。

---

### ライブラリ登録関数

C関数をLuaに登録するための関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_newlib` | 新しいテーブルを作成し関数を登録 |
| `luaL_newlibtable` | ライブラリ用テーブルを作成 |
| `luaL_setfuncs` | 関数配列をテーブルに登録 |
| `luaL_requiref` | モジュールをロード/登録 |

#### luaL_Reg

```c
typedef struct luaL_Reg {
  const char *name;
  lua_CFunction func;
} luaL_Reg;
```

`luaL_setfuncs`によって登録される関数の配列の型です。`name`は関数名であり、`func`は関数へのポインタです。`luaL_Reg`の配列は、`name`と`func`の両方が`NULL`である番兵エントリで終わる必要があります。

#### luaL_newlib

```c
void luaL_newlib (lua_State *L, const luaL_Reg l[]);
```

新しいテーブルを作成し、リスト`l`内の関数を登録します。これは、次のマクロとして実装されます：

```c
(luaL_newlibtable(L,l), luaL_setfuncs(L,l,0))
```

配列`l`は、実際の配列である必要があり、ポインタであってはなりません。

---

### 参照システム関数

オブジェクト参照を管理するための関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_ref` | 参照を作成 |
| `luaL_unref` | 参照を解放 |

#### luaL_ref

```c
int luaL_ref (lua_State *L, int t);
```

スタックの一番上にあるオブジェクトに対する*参照*を、インデックス`t`にあるテーブル内に作成して返します（そしてオブジェクトをポップします）。

参照システムはテーブルの整数キーを使用します。参照は一意の整数キーです。`luaL_ref`は返されるキーの一意性を保証します。エントリ1は内部使用のために予約されています。

参照`r`で参照されるオブジェクトを取得するには、`lua_rawgeti(L, t, r)`または`lua_geti(L, t, r)`を呼び出します。関数`luaL_unref`は参照を解放します。

スタックの最上部にあるオブジェクトが**nil**の場合、`luaL_ref`は定数`LUA_REFNIL`を返します。定数`LUA_NOREF`は、`luaL_ref`によって返されるどの参照とも異なることが保証されています。

---

### エラー処理関数

エラー発生と結果処理のための関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_error` | エラーを発生 |
| `luaL_typeerror` | 型エラーを発生 |
| `luaL_execresult` | プロセス関連関数の戻り値を生成 |
| `luaL_fileresult` | ファイル関連関数の戻り値を生成 |
| `luaL_pushfail` | fail値をプッシュ |

#### luaL_error

```c
int luaL_error (lua_State *L, const char *fmt, ...);
```

エラーを発生させます。エラーメッセージの形式は、`lua_pushfstring`と同じルールに従って、`fmt`と追加の引数によって指定されます。また、メッセージの先頭に、エラーが発生したファイル名と行番号を追加します（この情報が利用可能な場合）。

この関数は決して戻りませんが、C関数では`return luaL_error(args)`として使用するのが慣例です。

---

### ステート管理関数

Luaステートの作成と管理のための関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_newstate` | 新しいLuaステートを作成 |
| `luaL_openlibs` | すべての標準ライブラリを開く |

#### luaL_newstate

```c
lua_State *luaL_newstate (void);
```

新しいLuaステートを作成します。`luaL_alloc`をアロケータ関数として、`luaL_makeseed(NULL)`の結果をシードとして`lua_newstate`を呼び出し、その後、標準エラー出力にメッセージを出力する警告関数とパニック関数（§4.4を参照）を設定します。

新しいステートを返します。メモリ割り当てエラーが発生した場合は`NULL`を返します。

---

### ユーティリティ関数

その他のユーティリティ関数群です。

| 関数 | 説明 |
|------|------|
| `luaL_alloc` | 標準アロケータ関数 |
| `luaL_getsubtable` | サブテーブルを取得または作成 |
| `luaL_gsub` | 文字列置換 |
| `luaL_len` | 長さを取得 |
| `luaL_makeseed` | 乱数シードを生成 |
| `luaL_testudata` | ユーザーデータをテスト |
| `luaL_tolstring` | 値を文字列に変換 |
| `luaL_traceback` | トレースバックを作成 |
| `luaL_typename` | 型名を取得 |
| `luaL_where` | 現在位置を識別する文字列を生成 |

#### luaL_gsub

```c
const char *luaL_gsub (lua_State *L,
                       const char *s,
                       const char *p,
                       const char *r);
```

文字列`s`のコピーを作成し、文字列`p`の出現箇所をすべて文字列`r`に置き換えます。結果の文字列をスタックにプッシュして返します。

#### luaL_tolstring

```c
const char *luaL_tolstring (lua_State *L, int idx, size_t *len);
```

指定されたインデックスにある任意のLua値を、妥当な形式のC文字列に変換します。結果の文字列はスタックにプッシュされ、関数からも返されます（§4.1.3を参照）。`len`が`NULL`でない場合、関数は`*len`に文字列の長さも設定します。

値に`__tostring`フィールドを持つメタテーブルがある場合、`luaL_tolstring`は値を引数として対応するメタメソッドを呼び出し、呼び出しの結果を結果として使用します。

---

### ファイルハンドル型

#### luaL_Stream

```c
typedef struct luaL_Stream {
  FILE *f;
  lua_CFunction closef;
} luaL_Stream;
```

標準I/Oライブラリで使用されるファイルハンドルの標準表現です。

ファイルハンドルは、`LUA_FILEHANDLE`というメタテーブルを持つフルユーザーデータとして実装されます（`LUA_FILEHANDLE`は実際のメタテーブルの名前を持つマクロです）。メタテーブルはI/Oライブラリによって作成されます（`luaL_newmetatable`を参照）。

このユーザーデータは構造体`luaL_Stream`で始まる必要があります。この初期構造体の後に他のデータを含めることができます：

- フィールド`f`は対応するCストリームを指します（または、不完全に作成されたハンドルを示すために`NULL`にすることもできます）
- フィールド`closef`は、ハンドルが閉じられたとき、または収集されたときにストリームを閉じるために呼び出されるLua関数を指します

この関数は、ファイルハンドルを唯一の引数として受け取り、成功の場合はtrue値、エラーの場合はfalse値とエラーメッセージを返す必要があります。Luaがこのフィールドを呼び出すと、ハンドルが閉じられたことを示すために、フィールド値を`NULL`に変更します。

---

## 関数リファレンス一覧

以下は補助ライブラリの全関数のアルファベット順リストです：

| 関数 | スタック | 説明 |
|------|----------|------|
| `luaL_addchar` | [-?, +?, m] | バイトをバッファに追加 |
| `luaL_addgsub` | [-?, +?, m] | 置換しながら文字列を追加 |
| `luaL_addlstring` | [-?, +?, m] | 長さ指定で文字列を追加 |
| `luaL_addsize` | [-?, +?, –] | バッファ領域からの追加を確定 |
| `luaL_addstring` | [-?, +?, m] | ゼロ終端文字列を追加 |
| `luaL_addvalue` | [-?, +?, m] | スタック最上位の値を追加 |
| `luaL_alloc` | – | 標準アロケータ関数 |
| `luaL_argcheck` | [-0, +0, v] | 条件チェック |
| `luaL_argerror` | [-0, +0, v] | 引数エラーを発生 |
| `luaL_argexpected` | [-0, +0, v] | 型エラーを発生 |
| `luaL_buffaddr` | [-0, +0, –] | バッファアドレスを取得 |
| `luaL_buffinit` | [-0, +?, –] | バッファを初期化 |
| `luaL_bufflen` | [-0, +0, –] | バッファ長を取得 |
| `luaL_buffinitsize` | [-?, +?, m] | サイズ指定で初期化 |
| `luaL_buffsub` | [-?, +?, –] | バッファからバイトを削除 |
| `luaL_callmeta` | [-0, +(0\|1), e] | メタメソッドを呼び出す |
| `luaL_checkany` | [-0, +0, v] | 任意の型をチェック |
| `luaL_checkinteger` | [-0, +0, v] | 整数をチェック |
| `luaL_checklstring` | [-0, +0, v] | 文字列をチェック（長さ付き） |
| `luaL_checknumber` | [-0, +0, v] | 数値をチェック |
| `luaL_checkoption` | [-0, +0, v] | オプションをチェック |
| `luaL_checkstack` | [-0, +0, v] | スタックサイズを確保 |
| `luaL_checkstring` | [-0, +0, v] | 文字列をチェック |
| `luaL_checktype` | [-0, +0, v] | 型をチェック |
| `luaL_checkudata` | [-0, +0, v] | ユーザーデータをチェック |
| `luaL_checkversion` | [-0, +0, v] | バージョンをチェック |
| `luaL_dofile` | [-0, +?, m] | ファイルをロード・実行 |
| `luaL_dostring` | [-0, +?, –] | 文字列をロード・実行 |
| `luaL_error` | [-0, +0, v] | エラーを発生 |
| `luaL_execresult` | [-0, +3, m] | プロセス結果を生成 |
| `luaL_fileresult` | [-0, +(1\|3), m] | ファイル結果を生成 |
| `luaL_getmetafield` | [-0, +(0\|1), m] | メタフィールドを取得 |
| `luaL_getmetatable` | [-0, +1, m] | メタテーブルを取得 |
| `luaL_getsubtable` | [-0, +1, e] | サブテーブルを取得 |
| `luaL_gsub` | [-0, +1, m] | 文字列置換 |
| `luaL_len` | [-0, +0, e] | 長さを取得 |
| `luaL_loadbuffer` | [-0, +1, –] | バッファをロード |
| `luaL_loadbufferx` | [-0, +1, –] | バッファをロード（モード指定） |
| `luaL_loadfile` | [-0, +1, m] | ファイルをロード |
| `luaL_loadfilex` | [-0, +1, m] | ファイルをロード（モード指定） |
| `luaL_loadstring` | [-0, +1, –] | 文字列をロード |
| `luaL_makeseed` | [-0, +0, –] | 乱数シードを生成 |
| `luaL_newlib` | [-0, +1, m] | ライブラリテーブルを作成 |
| `luaL_newlibtable` | [-0, +1, m] | ライブラリテーブルを作成 |
| `luaL_newmetatable` | [-0, +1, m] | メタテーブルを作成 |
| `luaL_newstate` | [-0, +0, –] | ステートを作成 |
| `luaL_openlibs` | [-0, +0, e] | 標準ライブラリを開く |
| `luaL_opt` | [-0, +0, –] | オプショナルマクロ |
| `luaL_optinteger` | [-0, +0, v] | オプショナル整数 |
| `luaL_optlstring` | [-0, +0, v] | オプショナル文字列（長さ付き） |
| `luaL_optnumber` | [-0, +0, v] | オプショナル数値 |
| `luaL_optstring` | [-0, +0, v] | オプショナル文字列 |
| `luaL_prepbuffer` | [-?, +?, m] | バッファ領域を取得 |
| `luaL_prepbuffsize` | [-?, +?, m] | サイズ指定で領域を取得 |
| `luaL_pushfail` | [-0, +1, –] | fail値をプッシュ |
| `luaL_pushresult` | [-?, +1, m] | バッファ結果をプッシュ |
| `luaL_pushresultsize` | [-?, +1, m] | サイズ指定で結果をプッシュ |
| `luaL_ref` | [-1, +0, m] | 参照を作成 |
| `luaL_requiref` | [-0, +1, e] | モジュールをロード |
| `luaL_setfuncs` | [-nup, +0, m] | 関数を登録 |
| `luaL_setmetatable` | [-0, +0, –] | メタテーブルを設定 |
| `luaL_testudata` | [-0, +0, m] | ユーザーデータをテスト |
| `luaL_tolstring` | [-0, +1, e] | 文字列に変換 |
| `luaL_traceback` | [-0, +1, m] | トレースバックを作成 |
| `luaL_typeerror` | [-0, +0, v] | 型エラーを発生 |
| `luaL_typename` | [-0, +0, –] | 型名を取得 |
| `luaL_unref` | [-0, +0, –] | 参照を解放 |
| `luaL_where` | [-0, +1, m] | 現在位置を取得 |

---

## スタック表記について

関数プロトタイプの右側にある表記 `[-o, +p, x]` は以下を意味します：

- **o**: 関数がスタックからポップする要素数
- **p**: 関数がスタックにプッシュする要素数
- **x**: エラー発生の可能性
  - `–`: 関数はエラーを発生させない
  - `m`: メモリ不足エラーのみ発生する可能性
  - `e`: その他のエラーが発生する可能性
  - `v`: エラー発生時にメッセージを出力

`?` は、要素数がパラメータや状況に依存することを示します。

---

## 定数

| 定数 | 説明 |
|------|------|
| `LUAL_BUFFERSIZE` | `luaL_prepbuffer`で使用される定義済みバッファサイズ |
| `LUA_NOREF` | どの参照とも異なることが保証された定数 |
| `LUA_REFNIL` | nilオブジェクトに対して返される参照 |
| `LUA_FILEHANDLE` | ファイルハンドルのメタテーブル名 |

---

> **完全なリファレンス**: 個々の関数の詳細なパラメータと動作については、
> [Lua 5.5 Reference Manual (英語)](https://www.lua.org/manual/5.5/) を参照してください。
