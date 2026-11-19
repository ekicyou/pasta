# 起動シーケンスとモジュール解決

> 【不安】ゴーストが何も言わずに黙り込む――これほど不気味で、これほど心細いことはございませんわね。

> 【アンソニー】原因の当てもつかぬまま、途方に暮れる方が多うございます。

> 【クローディア】けれど安心なさいまし。pasta は黙って死んだりはいたしませんの。どこを探し、どの順に読み込み、どこで力尽きたのか、必ず足跡を残しますわ。その足跡の読み方を、わたくしが手ほどきいたします。

---

このページでは、pasta.dll がゴーストを起動するときに何をどの順で読み込むかと、起動に失敗したときにどこを読めば原因が分かるかを示す。

相対パスはすべて、`pasta.dll` を置いた `ghost/master/`（以下、基準ディレクトリ）からの相対である。

## 1. モジュール検索パス

Lua の `require` は、次の 5 つのディレクトリを**この順**で探す。先に見つかったものが勝つ。同じ名前のモジュールを上のディレクトリに置けば、下のものを上書きできる。

| 順 | ディレクトリ | 役割 |
| -- | ------------ | ---- |
| 1 | `profile/pasta/save/lua` | 保存領域に置かれた Lua |
| 2 | `scripts` | ゴースト作者が書くスクリプト |
| 3 | `profile/pasta/pasta_scripts` | pasta.dll が起動時に展開する内蔵スクリプト |
| 4 | `profile/pasta/cache/lua` | `.pasta` を変換したシーンのモジュール |
| 5 | `scriptlibs` | スクリプトライブラリ |

上の表は既定値で、`pasta.toml` の `[loader] lua_search_paths` で変えられる。

各ディレクトリでは、次の 2 つのファイル名を**この順**で試す。`?` にはモジュール名が入り、モジュール名の `.` はフォルダの区切りになる。

| 順 | パターン | `require "pasta.shiori.entry"` の例（`scripts` の場合） |
| -- | -------- | ------------------------------------------------------- |
| 1 | `?.lua` | `scripts/pasta/shiori/entry.lua` |
| 2 | `?/init.lua` | `scripts/pasta/shiori/entry/init.lua` |

モジュールが見つからないときのエラーには、探した候補が `no file '…'` の行として探索順に並ぶ。次は `require "mylib.util"` が見つからなかったときの例である。

```text
module 'mylib.util' not found:
	…
	no file 'C:/SSP/ghost/my-ghost/ghost/master/profile/pasta/save/lua/mylib\util.lua'
	no file 'C:/SSP/ghost/my-ghost/ghost/master/profile/pasta/save/lua/mylib\util/init.lua'
	no file 'C:/SSP/ghost/my-ghost/ghost/master/scripts/mylib\util.lua'
	no file 'C:/SSP/ghost/my-ghost/ghost/master/scripts/mylib\util/init.lua'
	…
```

- `…` の所には、残りのディレクトリの行と、Lua 標準のほかの探索の行が並ぶ。pasta が探した場所は、`.lua` で終わる行である。
- 候補のパスは、ディレクトリの部分の区切りが `/`、モジュール名の部分の区切りが `\` になる。表記が混ざるだけで、探している場所は 1 つである。
- 置いたつもりのファイルがどの行とも一致しなければ、置き場所かモジュール名のどちらかが違う。

### 設置パスの長さ・文字種に依存しない

`require` は、ゴーストをどこに置いても同じように解決する。

- 絶対パスが 260 文字を超える場所に置いても成功する。
- システムの ANSI コードページで表せない文字（各国語の利用者名・フォルダ名など）がパスにあっても成功する。`no file '…'` の行にも、その文字がそのまま出る。

例外は `;` と `?` である。Lua の検索パス（`package.path`）が `;` を区切り、`?` をモジュール名の場所として使うためである。基準ディレクトリの絶対パスか `lua_search_paths` の項目にこの 2 つの文字があると、辞書の読み込みより前に読み込みエラーになる。

```text
Ghost folder path contains ';', which Lua package.path cannot hold: 'C:\SSP\ghost\a;b\ghost\master'
[loader] lua_search_paths entry contains '?', which Lua package.path cannot hold: 'lib?'
```

ゴーストのフォルダ名か `lua_search_paths` の項目から、その文字を外す。

`package.path` は UTF-8 の文字列として解釈される。スクリプトから `package.path` に足すときは、UTF-8 の文字列を足す。

ここまでは `require` についての保証である。スクリプトが直接呼ぶ Lua 標準のファイル入出力は対象外である（第 3 節）。

## 2. 起動シーケンス

SHIORI の `load`（か `loadu`）を受けると、ランタイムは次の順にモジュールを読み込む。どれか 1 つでもロードに失敗すると、起動はそこで止まる。

| 順 | 読み込むもの | 失敗時の扱い | 備考 |
| -- | ------------ | ------------ | ---- |
| 1 | `main`（作者の初期化スクリプト） | 致命 | `scripts/main.lua` を置かなければ、内蔵の `main.lua` が読まれる |
| 2 | `pasta.shiori.entry`（SHIORI 応答関数） | 致命 | `SHIORI.load` / `SHIORI.request` / `SHIORI.unload` を定義する |
| 3 | `pasta.scene_dic`（シーン辞書） | 致命 | `.pasta` を変換したシーンのモジュールを読み込む |

`.pasta` の変換は、この 3 つより前に行われる。文法エラーのある `.pasta` が 1 つでもあれば、`main` を読む前に起動が止まる。

起動が止まると、pasta は次の 3 つで原因を示す。読み方は第 4 節にある。

1. `load` の戻り値が失敗になる。
2. その後の GET のリクエストに、500 応答と `X-ERROR-REASON` ヘッダで原因を返す。
3. ログファイルに、原因の全文を残す。起動モジュールのロード失敗は `fatal=true` の行になる。

## 3. 既知の制限

### `loadu` を呼ばないホストでは非 ANSI の設置パスを扱えない

`loadu` は、設置パスを UTF-8 で受け取る初期化の入口である（SSP は 2.6.92 以降が対応する）。`loadu` を呼ばないホストは設置パスを ANSI コードページで渡すので、ANSI で表せない文字は pasta.dll に届く前に欠ける。そのホストでは、ANSI で表せる文字だけのパスにゴーストを置く。どちらの入口で呼ばれたかは、ログの `entry` で分かる（第 4 節の手順 1）。

### 設置パスが 260 文字を超えると、ホストが短縮名を渡すことがある

ホストが、設置パスを 8.3 短縮名（`MYGHOS~1` の形）に置き換えて渡すことがある。ランタイムは、どちらのパスでも同じように動く。

### Lua 標準のファイル入出力は長パス・非 ANSI パスに対応しない

スクリプトが直接呼ぶ次の関数は、絶対パスが 260 文字を超えるファイルと、ANSI コードページに無い文字を含むパスのファイルを扱えない。

- `io.open`
- `loadfile`
- `dofile`
- `package.searchpath`

データの保存には、これらではなく [@pasta_persistence](../lua/modules/pasta-persistence.md) を使う。設置パスの長さ・文字種の影響を受けない。

### `ffi.load` で読む DLL の制限

`ffi.load`（第 5 節）が保証するのは、DLL を見つけて読み込み、`ffi.load` の値として返すところまでである。次のことは保証しない。

- **DLL のファイル名は ANSI コードページで表せる文字に限る。** フォルダの部分は深くても ANSI に無い文字を含んでもよいが、ファイル名（最後の部分）は LuaJIT の `ffi.load` へそのまま渡るため、ANSI で表せなければならない。似た文字への置き換えでしか表せない名前も表せないものとして扱う。この場合は `cannot load module '…': the file name '…' cannot be represented in the ANSI code page` で失敗する。
- **同じファイル名の別の DLL がプロセスに読み込み済みなら失敗する。** 同じ SSP で動く別のゴーストが同名の DLL を読んでいる場合などである。黙ってそちらの DLL に結びつくことはなく、`cannot load module '…': a different DLL named '…' is already loaded in this process` で失敗する。同じファイルを読み直すのは失敗ではない。ほかと重ならないファイル名にすれば起きない。
- **依存 DLL の同名の取り違えは防がない。** 確かめるのは `ffi.load` に渡した DLL 自身だけである。依存 DLL と同じ名前の DLL がプロセスに読み込み済みなら、Windows はそちらを使う。
- **名前だけの指定で OS の探索に任せた DLL は、Windows の規則のまま読まれる。** 検索パスのどこにも無かった名前（第 5 節）は、探す場所も依存 DLL を探す場所も Windows の既定の探索に従う。この節のほかの保証（深いパス・ANSI に無い文字・依存 DLL の探し方）は及ばない。
- **DLL の中で ANSI 版の API を使うと失敗しうる。** DLL が自分の置き場所などのパスを ANSI 版の API（`…A` の関数）で扱うと、設置パスが深い場合や ANSI に無い文字を含む場合に、DLL の中で失敗する。読み込みまでは成功しているので、`ffi.load` は失敗を報告しない。
- **`PATH` にある依存 DLL は見つからない。** カレントディレクトリも探さない。依存 DLL を探す場所は第 5 節のとおりである。

## 4. ゴーストが起動しない・喋らないとき

ゴーストがまったく反応しないときは、起動のどこかで失敗している。次の順で調べる。

この節が扱うのは、起動・読み込みの失敗である。このとき、バルーンには何も出ない。起動した後にイベントの処理中に止まったとき（止まるエラー）は、既定ではバルーンに `【スクリプトのエラー：…】`・`【時間切れ：…】` が出る。そちらの読み方は [止まるエラーの表記](runtime-failures.md#止まるエラーの表記) にある。

### 手順 1: ログファイルの `fatal=true` の行を読む

ログファイルは次の場所にある。新しい行は末尾に足される。

```text
profile/pasta/logs/pasta.log
```

場所は `pasta.toml` の `[logging] file_path` で変えられる。書けるのは、基準ディレクトリからの相対パスで、`profile/` の下だけである。それ以外の値（例: `logs/pasta.log`）を書くと、ログは上の既定の場所に書かれ、次の警告が残る。

```text
2026-10-18T00:35:49.801358Z  WARN pasta_lua::loader: Cannot use [logging] file_path; logging to the default log file instead file_path=logs/pasta.log fallback=profile/pasta/logs/pasta.log error=Log path must be within profile/ directory: "…\\logs/pasta.log"
```

ログを開いたら、`fatal=true` を含む行を探す。起動モジュール（第 2 節の表の 3 つ）のロードに失敗すると、必ずこの行が残る。次は、辞書 `dic/talk.pasta` を変換したシーンのモジュールが、Lua の構文エラーで読めなかったときの例である。

```text
2026-10-18T00:35:49.901358Z ERROR pasta_lua::runtime::factory: Failed to load startup module module="pasta.scene_dic" fatal=true error=runtime error: error loading module 'pasta.scene.talk' from file 'C:/SSP/ghost/my-ghost/ghost/master/profile/pasta/cache/lua/pasta\scene\talk.lua':
	...host/master/profile/pasta/cache/lua/pasta\scene\talk.lua:25: invalid escape sequence near '"'
stack traceback:
	[C]: in ?
	[C]: in function 'error'
	pasta_searcher:6: in function <pasta_searcher:3>
	[C]: in function 'require'
	...ghost/master/profile/pasta/cache/lua/pasta\scene_dic.lua:17: in main chunk
	[C]: in function 'require'
2026-10-18T00:35:49.966478Z ERROR pasta::shiori: PastaShiori load failed load_dir=C:\SSP\ghost\my-ghost\ghost\master\ error=Failed to initialize Lua runtime: failed to load startup module 'pasta.scene_dic'
runtime error: error loading module 'pasta.scene.talk' from file 'C:/SSP/ghost/my-ghost/ghost/master/profile/pasta/cache/lua/pasta\scene\talk.lua':
	…
2026-10-18T00:35:49.966478Z  INFO pasta::load_entry: SHIORI load entry completed entry="loadu" loaded=false
```

| 読む所 | 分かること |
| ------ | ---------- |
| `module="pasta.scene_dic"` | 失敗した起動モジュール |
| `fatal=true` | 起動を止めた失敗である |
| `error=` から次の時刻の行まで | 原因の全文。この例では、読めなかったファイルと、その 25 行目の構文エラーである。`stack traceback:` から下は、そこへ至った呼び出しの経路である |
| `PastaShiori load failed` の行 | ロード全体が失敗したこと。`load_dir` は基準ディレクトリである。同じ原因がもう一度続く（例では `…` で省いた） |
| `SHIORI load entry completed` の行 | ロードの締めくくり。`entry` はホストが呼んだ入口（`loadu` か `load`）、`loaded` はロードの成否である |

- `fatal=true` の行が無く `loaded=false` なら、`.pasta` の文法エラーなど、起動モジュールを読む前の失敗である。`ERROR` の行を探す（手順 3）。
- `SHIORI load entry completed` の行がまったく無ければ、ロードの処理に入る前（DLL の読み込み・設置パスの受け取り）で止まっている。

### 手順 2: ホストの SHIORI 通信ログで 500 応答を読む

ロードに失敗した後も、pasta は GET のリクエストに 500 応答を返し、`X-ERROR-REASON` ヘッダに原因を入れる。ホストの SHIORI 通信ログで、この応答を読める。ホストがリクエストを送らなければ応答は出ないので、そのときは手順 1 のログだけで調べる。

手順 1 の例と同じ失敗では、応答は次の形になる。

```text
SHIORI/3.0 500 Internal Server Error
Charset: UTF-8
X-ERROR-REASON: Load error: Failed to initialize Lua runtime: failed to load startup module 'pasta.scene_dic' runtime error: error loading module 'pasta.scene.talk' from file 'C:/SSP/ghost/my-ghost/ghost/master/profile/pasta/cache/lua/pasta\scene\talk.lua': ...host/master/profile/pasta/cache/lua/pasta\scene\talk.lua:25: invalid escape sequence near '"'
```

`X-ERROR-REASON` の値は必ず 1 行である。ログの原因から改行を除き、スタックトレースを落としたものが入る。全文はログファイルで読む。

### 手順 3: 典型的な原因を読み分ける

| メッセージに現れるもの | 原因 | 対処 |
| ---------------------- | ---- | ---- |
| `failed to load startup module 'main'` と `error loading module 'main' from file '…'` | `scripts/main.lua` の構文エラー | 続く行に出るファイル名と行番号の所を直す |
| `failed to load startup module 'pasta.shiori.entry'` | `scripts/pasta/shiori/entry.lua` を置いて内蔵の `entry.lua` を上書きしており、その中身に誤りがある | 上書きが要らなければ、そのファイルを消して内蔵のものに戻す |
| `module 'X' not found:` と `no file '…'` の並び | モジュールが見つからない | `no file` の行と、置いた場所・ファイル名・モジュール名の綴りを見比べる（第 1 節） |
| `error loading module 'X' from file '…'` | ファイルは見つかったが、構文エラーがあるか読めなかった | 続く行に出るファイル名と行番号の所を直す |
| `Partial transpilation failure: …` | `.pasta` の文法エラー | ログの `Process failure` の行で、ファイル・行・列を確かめて直す（下記「パースエラー」） |
| `Module name collision: …` | 2 つ以上の `.pasta`、または辞書のフォルダの 2 つ以上の `.lua` が、同じモジュール名になる。大文字と小文字だけの違いも同じ名前として扱う | 示されたファイルのどれかの名前を変える（下記「モジュール名の衝突」） |
| `which Lua package.path cannot hold` | 基準ディレクトリのパスか `lua_search_paths` の項目に `;` か `?` がある | フォルダ名か項目から、その文字を外す（第 1 節） |
| `load directory not found: …` | 渡された設置パスのフォルダが無い | 示されたパスの文字が欠けていれば、第 3 節の `loadu` の制限に当たる |

#### パースエラー

`.pasta` に文法エラーがあると、ロード全体が失敗する。失敗したファイルごとに、`Process failure` の行がログに残る。次は、`dic/talk_boy.pasta` の 123 行目に誤りがあったときの例である。

```text
2026-11-08T11:04:20.912891Z ERROR pasta_lua::loader::process: Process failure path=C:\SSP\ghost\my-ghost\ghost\master\dic\talk_boy.pasta error=Parse error: Parse error: C:\SSP\ghost\my-ghost\ghost\master\dic\talk_boy.pasta:123:22: Parse error in C:\SSP\ghost\my-ghost\ghost\master\dic\talk_boy.pasta at 123:22:    --> 123:22
    |
123 | 　　　　：＠内緒　先生にはないしょですよ？
    |                      ^---
    |
    = expected var_ref_local, var_ref_global, …, talk, or sakura_script
2026-11-08T11:04:20.912891Z ERROR pasta::shiori: PastaShiori load failed load_dir=C:\SSP\ghost\my-ghost\ghost\master\ error=Partial transpilation failure: 18 succeeded, 1 failed [C:\SSP\ghost\my-ghost\ghost\master\dic\talk_boy.pasta]
2026-11-08T11:04:20.912891Z  INFO pasta::load_entry: SHIORI load entry completed entry="loadu" loaded=false
```

| 読む所 | 分かること |
| ------ | ---------- |
| `path=` | 失敗した `.pasta` ファイル |
| `…talk_boy.pasta:123:22:` | 誤りの行と列（どちらも 1 から数える） |
| `123 \| …` と `^---` | その行の中身と、誤りの位置 |
| `= expected …` | その位置に書けるものの一覧（例では `…` で省いた） |
| `Partial transpilation failure: 18 succeeded, 1 failed […]` | 読めた数・読めなかった数と、読めなかったファイルの一覧 |

`X-ERROR-REASON` に入るのは `Partial transpilation failure: …` の部分で、行と列は入らない。行と列はログの `Process failure` の行で確かめる。

#### モジュール名の衝突

辞書のファイル名からはモジュール名が作られる。違うファイルが同じモジュール名になると、片方の内容が失われるため、辞書の変換と Lua の実行より前に読み込みエラーになる。`Module name collision:` の後には、ぶつかったモジュール名とファイル（基準ディレクトリから見たパス）が並ぶ。大文字と小文字だけが違う組は、ファイルごとにモジュール名が付く。組が複数あれば ` / ` でつながる。

```text
Module name collision: pasta.scene.会話_朝: dic\会話-朝.pasta, dic\会話_朝.pasta
Module name collision: dic\A-b.pasta (pasta.scene.A_b), dic\a_b.pasta (pasta.scene.a_b)
```

モジュール名の作り方は [辞書の検査](../debug/validate.md#モジュール名) にある。`pasta_check validate` は同じ組を `module-pasta-collision`・`module-lua-collision` の誤りとして報告するので、ゴーストを起動する前に見つけられる。

## 5. DLL を FFI で使う

C で書いた DLL の関数は、LuaJIT の FFI を通して呼ぶ。FFI は既定では開いていない。`pasta.toml` の `[loader] ffi` を `true` にしたゴーストでだけ使える（[ffi](pasta-toml.md#ffi) を参照。有効にしたときの注意もそこにある）。

C モジュールを `require` で読む道（`package.cpath`・`package.loadlib`）は使えない。pasta.dll は Lua の C API を外に公開しておらず、`lua51.dll` も無いため、C モジュールの DLL はどこに置いても読めない。C の DLL は FFI で使う。

### 例

関数を 1 つ持つ DLL を C で書く。

```c
/* foo.c */
__declspec(dllexport) int add(int a, int b) { return a + b; }
```

これを DLL に作り（Visual Studio なら `cl /LD foo.c`）、できた `foo.dll` をゴーストの `scripts/` に置く。DLL は pasta.dll と同じ 32 ビット・64 ビットの別で作る。

`pasta.toml` で FFI を開く。

```toml
[loader]
ffi = true
```

Lua から関数を宣言し、`ffi.load` で DLL を読んで呼ぶ。

```lua
local ffi = require "ffi"
ffi.cdef [[ int add(int a, int b); ]]
local foo = ffi.load("foo")
local sum = foo.add(1, 2)  -- 3
```

`ffi.load` の 2 番目の引数は、LuaJIT の `ffi.load` へそのまま渡る。

### 探すフォルダと順

`ffi.load` に渡す名前は、書き方によって次のように探す。名前の最後の部分に `.` が無ければ `.dll` を足す（`foo` は `foo.dll`、`foo.bin` はそのまま）。

| 書き方 | 例 | 探す場所 |
| ------ | -- | -------- |
| 名前だけ（`/`・`\`・`:` を含まない） | `"foo"`・`"foo.dll"` | 第 1 節のモジュール検索パスのディレクトリを同じ順で探し、最初に見つかったものを読む。どこにも無ければ OS の既定の探索に任せる |
| 相対パス | `"lib/foo"`・`"lib\\foo.dll"` | 基準ディレクトリからの相対として解く |
| 絶対パス | `"C:/dll/foo.dll"` | そのまま読む |

名前だけの指定で探すディレクトリは、`require` と同じ `[loader] lua_search_paths` から作る。`lua_search_paths` を変えれば `require` と `ffi.load` の両方の探索が変わる。実行中に `package.path` を書き換えても、`ffi.load` の探索は変わらない。基準ディレクトリそのものは、`lua_search_paths` に含めない限り探さない。どのディレクトリにも無い名前（たとえば `"kernel32"`）は、OS の既定の探索で読む。このときの探し方は Windows の規則のままである（第 3 節の [`ffi.load` で読む DLL の制限](#ffiload-で読む-dll-の制限)）。

相対パスの基点は、カレントディレクトリではなく基準ディレクトリである。区切りは `/` と `\` のどちらでもよい。`\foo`・`/foo`・`C:foo` も、ドライブのルートやドライブのカレントではなく、基準ディレクトリの下として解く。絶対パスとして扱うのは、ドライブ文字と区切りで始まるもの（`C:\…`・`C:/…`）と `\\` で始まるものである。

相対パス・絶対パスで指したファイルが無ければ、そこで失敗する。モジュール検索パスのディレクトリや OS の既定の探索には回さない。

ゴーストの設置場所が深くても、フォルダの名前に ANSI コードページに無い文字が含まれていても、ゴーストのフォルダの DLL は読める。ただしファイル名は ANSI で表せる文字に限る（第 3 節）。

### 依存 DLL を探す場所

ゴーストのフォルダから読んだ DLL が頼る依存 DLL は、次の順で探す。

1. 読んだ DLL と同じフォルダ
2. ホストの実行ファイル（`ssp.exe` など）のフォルダ、System32 などの Windows の既定のフォルダ

カレントディレクトリと環境変数 `PATH` のフォルダは探さない。依存 DLL は、読む DLL と同じフォルダに置く。

### 失敗したとき

`ffi.load` が失敗すると、`cannot load module '名前': 理由` の形のエラーを、`ffi.load` を呼んだ位置で投げる。`名前` は `ffi.load` に渡したままの名前である。`理由` は Windows のエラーメッセージ（OS の表示言語、UTF-8）か、第 3 節に挙げた pasta の英語の文言である。DLL や依存 DLL が見つからないときは、「指定されたモジュールが見つかりません。」に当たる Windows のメッセージになる。失敗してもプロセスは落ちず、`pcall` で受け取れる。

---

> 【したり顔】ね、黙り込んだように見えても、ゴーストはちゃんと悲鳴を上げておりますのよ。あとはそれを聞き取ってあげるだけ。原因が分かってしまえば、直すのは造作もないことですわ。

> 【アンソニー】悲鳴と申しますより、ログでございますね。

> 【落胆】……風情がありませんわね。まあよろしいですわ。さあ、胸を張って起動させてやりましょう！
