## 6.1 – Cコードでのライブラリロード

Cホストプログラムは、スクリプトで標準ライブラリを使用したい場合、明示的にそれらをステートにロードする必要があります。そのために、ホストプログラムは関数 [`luaL_openlibs`](#lual_openlibs) を呼び出すことができます。あるいは、[`luaL_openselectedlibs`](#lual_openselectedlibs) を使用して、どのライブラリを開くかを選択することもできます。両方の関数はヘッダファイル `lualib.h` で宣言されています。

スタンドアロンインタプリタ `lua`（[§7](07-standalone.md) 参照）は、すべての標準ライブラリを既に開いています。

---

### luaL_openlibs

```c
[-0, +0, e]
void luaL_openlibs (lua_State *L);
```

指定されたステートにすべての標準Luaライブラリを開きます。

---

### luaL_openselectedlibs

```c
[-0, +0, e]
void luaL_openselectedlibs (lua_State *L, int load, int preload);
```

選択された標準ライブラリをステート `L` に開く（ロードする）およびプリロードします。（*プリロード*とは、ライブラリローダーをテーブル [`package.preload`](#pdf-packagepreload) に追加することを意味し、これによりプログラムは後でそのライブラリを require できるようになります。[`require`](#pdf-require) 自体は *package* ライブラリによって提供されることに注意してください。プログラムがそのライブラリをロードしない場合、何も require することができません。）

整数 `load` はどのライブラリをロードするかを選択します。整数 `preload` は、ロードされなかったものの中からどのライブラリをプリロードするかを選択します。両方とも、以下の定数のビット単位ORで形成されるマスクです：

| 定数 | 説明 |
|------|------|
| **`LUA_GLIBK`** | 基本ライブラリ |
| **`LUA_LOADLIBK`** | パッケージライブラリ |
| **`LUA_COLIBK`** | コルーチンライブラリ |
| **`LUA_STRLIBK`** | 文字列ライブラリ |
| **`LUA_UTF8LIBK`** | UTF-8ライブラリ |
| **`LUA_TABLIBK`** | テーブルライブラリ |
| **`LUA_MATHLIBK`** | 数学ライブラリ |
| **`LUA_IOLIBK`** | I/Oライブラリ |
| **`LUA_OSLIBK`** | オペレーティングシステムライブラリ |
| **`LUA_DBLIBK`** | デバッグライブラリ |
