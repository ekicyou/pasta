## 6.4 – モジュール（パートA: require-package.loadlib）

パッケージライブラリはLuaでモジュールをロードするための基本機能を提供します。グローバル環境に直接1つの関数をエクスポートします：[`require`](#require-modname)。それ以外はすべてテーブル `package` にエクスポートされます。

---

### require (modname)

与えられたモジュールをロードします。関数は最初に [`package.loaded`](#packageloaded) テーブルを調べて、`modname` が既にロードされているかどうかを判断します。ロードされている場合、`require` は `package.loaded[modname]` に格納されている値を返します。（この場合に2番目の結果がないことは、この呼び出しがモジュールをロードする必要がなかったことを示します。）そうでなければ、モジュールの*ローダー*を見つけようとします。

ローダーを見つけるために、`require` はテーブル [`package.searchers`](#packagesearchers) によってガイドされます。このテーブルの各項目は、特定の方法でモジュールを検索する検索関数です。このテーブルを変更することで、`require` がモジュールを探す方法を変更できます。以下の説明は [`package.searchers`](#packagesearchers) のデフォルト設定に基づいています。

最初に `require` は `package.preload[modname]` をクエリします。値がある場合、この値（関数でなければならない）がローダーです。そうでなければ、`require` は [`package.path`](#packagepath) に格納されているパスを使用してLuaローダーを検索します。それも失敗した場合、[`package.cpath`](#packagecpath) に格納されているパスを使用してCローダーを検索します。それも失敗した場合、*all-in-one* ローダーを試みます（[`package.searchers`](#packagesearchers) 参照）。

ローダーが見つかると、`require` は2つの引数でローダーを呼び出します：`modname` と追加の値、*ローダーデータ*で、これもサーチャーによって返されます。ローダーデータはモジュールにとって有用な任意の値です。デフォルトのサーチャーでは、ローダーがどこで見つかったかを示します。（例えば、ローダーがファイルから来た場合、この追加の値はファイルパスです。）ローダーが非 nil 値を返した場合、`require` は返された値を `package.loaded[modname]` に割り当てます。ローダーが非 nil 値を返さず、`package.loaded[modname]` に値を割り当てていない場合、`require` はこのエントリに **true** を割り当てます。いずれの場合も、`require` は `package.loaded[modname]` の最終値を返します。その値に加えて、`require` はサーチャーによって返されたローダーデータを2番目の結果として返し、これは `require` がモジュールをどのように見つけたかを示します。

モジュールのロードまたは実行でエラーがある場合、またはモジュールのローダーが見つからない場合、`require` はエラーを発生させます。

---

### package.config

パッケージのコンパイル時設定を記述する文字列。この文字列は一連の行です：

| 行 | 説明 | デフォルト |
|----|------|-----------|
| 1行目 | ディレクトリ区切り文字列 | Windowsは '`\`'、他は '`/`' |
| 2行目 | パス内のテンプレートを区切る文字 | '`;`' |
| 3行目 | テンプレート内の置換ポイントを示す文字列 | '`?`' |
| 4行目 | Windowsでパス内で実行ファイルのディレクトリに置換される文字列 | '`!`' |
| 5行目 | `luaopen_` 関数名を構築する際にそれ以降のすべてのテキストを無視するマーク | '`-`' |

---

### package.cpath

[`require`](#require-modname) がCローダーを検索するために使用するパスを持つ文字列。

Luaは、Luaパス [`package.path`](#packagepath) を初期化するのと同じ方法でCパス [`package.cpath`](#packagecpath) を初期化します。環境変数 `LUA_CPATH_5_5`、または環境変数 `LUA_CPATH`、または `luaconf.h` で定義されたデフォルトパスを使用します。

---

### package.loaded

どのモジュールが既にロードされているかを制御するために [`require`](#require-modname) が使用するテーブル。モジュール `modname` を require し、`package.loaded[modname]` が偽でない場合、[`require`](#require-modname) は単にそこに格納されている値を返します。

この変数は実際のテーブルへの参照に過ぎません。この変数への代入は [`require`](#require-modname) が使用するテーブルを変更しません。実際のテーブルはCレジストリ（[§4.3](04-the-application-program-interface.md#43--レジストリ) 参照）に格納され、キー `LUA_LOADED_TABLE`（文字列）でインデックスされます。

---

### package.loadlib (libname, funcname)

ホストプログラムをCライブラリ `libname` と動的にリンクします。

`funcname` が "`*`" の場合、ライブラリとリンクするだけで、ライブラリによってエクスポートされたシンボルを他の動的にリンクされたライブラリで使用可能にします。そうでなければ、ライブラリ内の関数 `funcname` を探し、この関数をC関数として返します。したがって、`funcname` は [`lua_CFunction`](04-the-application-program-interface.md#lua_cfunction) プロトタイプに従う必要があります。

これは低レベル関数です。パッケージとモジュールシステムを完全にバイパスします。[`require`](#require-modname) とは異なり、パス検索を行わず、自動的に拡張子を追加しません。`libname` は、必要に応じてパスと拡張子を含む、Cライブラリの完全なファイル名でなければなりません。`funcname` は、Cライブラリによってエクスポートされた正確な名前でなければなりません（使用されるCコンパイラとリンカに依存する場合があります）。

この機能はISO Cではサポートされていません。そのため、`loadlib` はいくつかのプラットフォームでのみ利用可能です：Linux、Windows、Mac OS X、Solaris、BSD、およびその他の `dlfcn` 標準をサポートするUnixシステム。

この関数は本質的に安全ではありません。Luaがシステム内の任意の読み取り可能な動的ライブラリ内の任意の関数を呼び出すことを許可するからです。（Luaは関数が適切なプロトタイプを持ち、適切なプロトコルを尊重すると仮定して任意の関数を呼び出します（[`lua_CFunction`](04-the-application-program-interface.md#lua_cfunction) 参照）。したがって、任意の動的ライブラリ内の任意の関数を呼び出すと、アクセス違反になることがほとんどです。）
