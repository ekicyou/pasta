# ギャップ分析: lua-cmodule-robustness

- 日付: 2026-10-18（base: `5c550856`）
- 対象: `requirements.md`（requirements-generated・未承認）
- 調べ方: リポジトリのコード・文書を読み、`C:\rust\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\` の `luajit-src-210.6.6+707c12b`・`mlua-sys-0.10.0`・`mlua-0.11.6` のソースを読んだ。別のワークツリーにある作り済みの `pasta.dll`（x86・x64）を `dumpbin` で調べた。ビルドとテストは走らせていない。

## 1. まとめ

- 探して読む部分（Requirement 1〜7・9）は、`lua-require-robustness` の `searcher.rs` と同じ形で素直に作れる。手本・テストの道具（`NON_ANSI_DIR_NAME`・`make_deep_dir`・標準と並べる比べるテスト）はそろっている。
- **いちばん大きい穴は Requirement 8（Lua の C API）。** 今の pasta.dll は `lua_*`・`luaL_*` を 1 つも公開していない。普通の C モジュールは `lua51.dll` を前提に作られ、pasta には `lua51.dll` が無い。ここを決めないと、ローダーを直しても実際の C モジュールは読めない。
- 公開する案 (a)・(b) には、**同じ SSP で pasta のゴーストを 2 体動かすと、後のゴーストの C モジュールが先のゴーストの pasta.dll に結びつく**という危険がある（Windows の DLL の結びつけの規則。未計測）。要件では「自分の pasta.dll につなぎ直す」完全版で書いた。設計で実測が要る。
- テスト用の C モジュール DLL は、`cc`（lock に既にある）を dev の依存に足し、テストの中で `cl.exe` を呼んで作るのが一番軽い。CI は x86・x64 の両方で `cargo test --target` を走らせているので、そのまま両方で確かめられる。
- 規模は M〜L（Requirement 8 を入れると L 寄り）。危険は中〜高（8.7 のつなぎ直しが未知）。

## 2. 今の姿

### 2.1 関わるファイル

| 場所 | 中身 | この spec との関わり |
| --- | --- | --- |
| `crates/pasta_lua/src/runtime/searcher.rs`（622 行） | `.lua` の searcher（`package.loaders[2]`）の置き換え。`candidate_paths`・`search`・`find_module`・Lua のラッパ（`error(msg, 0)` で文字列のエラー）・`install_module_searcher`（`EXPECTED_LOADERS_LEN = 4` の確かめ）・`@pasta_chunks` | C の searcher の手本。候補の作り方（`candidate_paths`）はそのまま使い回せる。`install_module_searcher` の隣に C 側の設置を置く |
| `crates/pasta_lua/src/runtime/module_registry.rs`（193 行） | `setup_package_path`。`package.path` を設定し、searcher と `@pasta_chunks` を設置する。「`package.cpath` は変えない」とコメントにある | 既定の `package.cpath` の設定と C 側の設置をここに足す |
| `crates/pasta_lua/src/loader/context.rs`（278 行） | `LoaderContext`（`base_dir`・`lua_search_paths`）、`generate_package_path`（`/` 区切り、フォルダごとに `?.lua`・`?/init.lua`） | 既定の `package.cpath` を作る関数を足す（`generate_package_cpath` のようなもの） |
| `crates/pasta_lua/src/loader/config/mod.rs` | `default_lua_search_paths`（`profile/pasta/save/lua`・`scripts`・`profile/pasta/pasta_scripts`・`profile/pasta/cache/lua`・`scriptlibs`） | 既定の cpath の元。変えない |
| `crates/pasta_lua/src/runtime/mod.rs` | VM は `Lua::unsafe_new_with(std_lib, …)` で作る。既定の `libs` は `std_all`（= `StdLib::ALL_SAFE`。FFI・debug を含まない）。`std_package` は必須（`REQUIRED_LIBS`） | 安全な `Lua::new` だと mlua は C の searcher を落とす。pasta は `unsafe_new_with` なので 4 つの searcher がある（`searcher.rs` のテストのコメントにある） |
| `crates/pasta_lua/src/runtime/lifecycle.rs` | `impl Drop for PastaLuaRuntime`（終了時の保存など） | Lua の状態を閉じる時。DLL の解放はここではなく Lua の状態の後始末（userdata の `__gc`）に任せる |
| `crates/pasta_lua/Cargo.toml` | `windows-sys.workspace = true`（機能はルートの `Win32_Foundation`・`Win32_System_Memory`・`Win32_Globalization`） | `Win32_System_LibraryLoader` を crate の側で足す（`pasta_shiori` が `Win32_System_Threading` を足しているのと同じ形）。機能の名前は lock に載らない |
| `crates/pasta_shiori/Cargo.toml` | `[lib] name = "pasta"`・`crate-type = ["cdylib", "rlib"]` | Requirement 8 の公開はこの crate |
| `crates/pasta_shiori/build.rs` | VERSIONINFO の埋め込み（`embed_resource::compile_for_cdylib`）だけ | `cargo:rustc-cdylib-link-arg=/EXPORT:…` を出す場所になる |
| `crates/pasta_shiori/src/windows.rs` | `#[unsafe(no_mangle)] extern "C"` の `loadu`・`load`・`unload`・`request` と `DllMain`。`.def` は無い | 入口は変えない（8.8） |
| `crates/pasta_lua/tests/runtime/module_searcher_test.rs`（340 行） | 標準の searcher と置き換えた searcher を並べて、型と文言をバイトで比べる | C 側の比べるテストの手本（10.4） |
| `crates/pasta_lua/tests/loader/path_robustness_test.rs`・`tests/common/mod.rs` | `NON_ANSI_DIR_NAME`（5 つの文字体系と絵文字）・`make_deep_dir` | 10.1・10.2 でそのまま使う |
| `book/src/internals/loader.md:215` | 「`package.cpath` は変えない」 | 11.4 で直す |
| `book/src/reference/startup.md`（192 行・「3. 既知の制限」は 83 行から） | C モジュールの記述は無い | 11.1〜11.3 |
| `.claude/skills/pasta-lua-coding/references/startup.md` | `gen-skill-refs.mjs` が `reference/startup.md` から作る | 11.5。スキルの書き込みは分類器に拒まれることがある（brief の Constraints） |
| `.github/workflows/build.yml` | x86・x64 の行列で `cargo test --all --target ${{ matrix.target }}`（49 行）。clippy は x64 だけ | テストは両方で走る（10.9）。追加の設定は要らない見込み |

### 2.2 LuaJIT の C モジュールの経路（`lib_package.c`）

置き換えで同じに保つ規則（行は `luajit2/src/lib_package.c`）。

- 読み込み: `LoadLibraryExA(path, NULL, 0)`（`lj_arch.h:709`）。依存 DLL はモジュールのフォルダから探さない。
- 存在の確かめ: `readable` が `fopen(filename, "r")`（299 行）。深いパスはここで落ちる。
- 入口の名前: `mksymname` が最初の `-` までを捨て、`.` を `_` にして `luaopen_%s`。見つからなければ `luaJIT_BC_%s` を探し、`luaL_loadbuffer(L, bcdata, ~(size_t)0, name)` でバイトコードとして読む。
- 使い回し: レジストリのキー `"LOADLIB: <path>"` に、ハンドルを入れた userdata（メタ表 `_LOADLIB`・`__gc` で `FreeLibrary`）を置く。同じパス文字列なら 1 回だけ読む。
- パスの長さ: `strlen(path) >= 4096` なら `"path too long"`。置き換えでは持たない（1.6。`lua-require-robustness` の 1.6 と同じ）。
- `package.loadlib(path, init)`: `init` が `*` なら読むだけで `true`。失敗は `nil, msg, "open"`（読めない）か `nil, msg, "init"`（関数が無い）。
- croot（4 番目）: 名前に `.` が無ければ何もしない（`return 0`）。ルートのファイルに関数が無ければ `"\n\tno module '<名前>' in file '<ファイル>'"`。
- 失敗の文言: `loaderror` が `"error loading module '%s' from file '%s':\n\t%s"`。理由は `pusherror` が `FormatMessageA`（ANSI・末尾に `\r\n` が付く）、失敗すれば `"system error %d\n"`。
- `package.cpath` が文字列でなければ `"'package.cpath' must be a string"`。
- `!`: `setprogdir` は `luaopen_package` の `setpath` の中だけで呼ばれる。つまり**起動のときに既定の値（または環境変数 `LUA_CPATH`）の中でだけ置き換え**、後から書いた `package.cpath` では置き換えない。`GetModuleFileNameA(NULL, …)`（`MAX_PATH`）で SSP.exe のフォルダになる。
- 既定の cpath: `.\?.dll;!\?.dll;!\loadall.dll`（`luaconf.h:24-25`）。mlua は `LUA_NOENV` を立てないので、環境変数 `LUA_CPATH` があればそれが使われる。

### 2.3 Lua の C API の今（Requirement 8）

- mlua-sys は `luajit_src::Build::new().lua52compat(…).build()` を呼び、MSVC では `msvcbuild.bat … static` で**静的ライブラリ `lua51.lib`** を作って pasta.dll に入れる。static のときは `LUA_BUILD_AS_DLL` が付かないので、`LUA_API` は `extern`（`dllexport` ではない）。
- 作り済みの pasta.dll（x86・x64）の export は 182 個。**`lua_*`・`luaL_*`・`luaopen_*` は 0 個**。あるのは `DllMain`・`load`・`loadu`・`request`・`unload` と、CRT の stdio の関数（約 170 個）。後者は `msvcbuild.bat` の `/D_CRT_STDIO_INLINE=__declspec(dllexport)__inline` が原因で、今も漏れている（この spec の範囲外だが、静的ライブラリの export 指示をリンカーが守る証拠になる）。
- `lua51.lib` の公開の名前（x86）: `lua_*` 87・`luaL_*` 44・`luaopen_*` 12・`luaJIT_*` 5（計 148）。x86 は `_lua_pushstring`（cdecl の飾り）、x64 は飾り無し。x86 の pasta.dll では rustc の `.def` の `load` がリンカーで `_load` に対応づけられているので、`/EXPORT:lua_pushstring` も `_lua_pushstring` に当たる見込み（lua_* では未確認）。
- `luajit52` の互換関数（`lua_absindex`・`lua_rotate`・`luaL_tolstring`・`luaL_requiref` など）は `mlua-sys-0.10.0/src/lua51/compat.rs` の Rust の関数で、C のライブラリには無い。公開はできないが、C モジュールは LuaJIT 2.1 のヘッダーに対して作るので要らない。ヘッダーに `LUA52COMPAT` の分かれ目は無く、ABI は互換モードに左右されない。
- 普通の C モジュールは `lua51.dll` を取り込むので、今は `lua51.dll` が無いだけでエラー 126（`ERROR_MOD_NOT_FOUND`）になる。**brief の表の「隣の依存 DLL で失敗」は、この原因と区別されていない。** 分けるには、Lua を何も取り込まないモジュール（`luaopen_x` が何もせず 0 を返す）＋隣の依存 DLL 1 つ、と、依存 DLL の無い同じモジュール（対照）を並べ、`dumpbin /dependents` で取り込みを確かめる（10.3）。

### 2.4 DLL の結びつけの規則（8.7 の根）

Microsoft の「DLL の探し方」の文書には、同じモジュール名の DLL が既に読み込まれていれば、どのフォルダにあってもそれを使う、とある（リダイレクトとマニフェストだけを先に見る）。pasta.dll を名前で取り込む C モジュールは、プロセスの中で最初に読み込まれた `pasta.dll` に結びつく。

- SSP は各ゴーストの SHIORI をフルパスで読むので、pasta のゴーストが 2 体いれば `pasta.dll` が 2 つ読み込まれる（別の LuaJIT の写し）。
- 後のゴーストの C モジュールは、先のゴーストの `pasta.dll` に結びつく。版が違えば構造体の形が違って壊れる。同じ版でも別の写しのコードで自分の `lua_State` を触ることになる（多くは動きそうだが、保証できない）。
- 転送用の `lua51.dll`（案 (b)）も、転送先 `pasta` を同じ規則で解くので、同じ危険を引き継ぐ。
- **この節は文書から推した結論で、実測していない。** 設計で、2 つのフォルダに置いた同じ pasta.dll を 1 つのプロセスに読み込み、C モジュールの結びつき先を確かめる必要がある。

## 3. 要件と資産の対応表

| 要件 | 使える資産 | 状態 |
| --- | --- | --- |
| 1.1〜1.7 深いパス・非 ANSI | `searcher.rs` の `candidate_paths`・std の存在確認（wide API）。brief の計測で `LoadLibraryExW`＋`LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR\|LOAD_LIBRARY_SEARCH_DEFAULT_DIRS`（`\` 区切りの絶対パス）が 480 文字・絵文字で成功 | **Missing**（読み込みの層・設置）。**Constraint**: `LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR` は完全なパスを要し、`/` を含むと失敗する（一般に `ERROR_INVALID_PARAMETER`。未計測）。候補は `/` と `\` が混ざる（`package.path` と同じ作り）ので、読み込みの前に `\` へそろえる。`\\?\` を付けないこと（brief）。 |
| 2.1〜2.9 探す規則 | `candidate_paths`（`.`→`\`・`;` 区切り・空を飛ばす・`?` をすべて置き換え）・Lua のラッパ（文字列のエラー） | **Missing**（`-` の規則・croot・使い回しのキー）。使い回しは LuaJIT と同じく「パス文字列」をキーにするのが素直（同じファイルの別の書き方は OS が同じハンドルを返す） |
| 2.10 `luaJIT_BC_` | — | **Missing**・**Unknown**: 長さ不明のバイトコード（`~(size_t)0`）を読むには `mlua::ffi::luaL_loadbuffer` を生の状態で呼ぶ必要がある。mlua の安全な API では長さが要る |
| 3.1〜3.8 既定の cpath・基点 | `LoaderContext::generate_package_path`・`base_dir`（`std::path::absolute`・8.3 名を変えない） | **Missing**（`generate_package_cpath` 相当・相対の判定）。**Constraint**: 「相対か」の判定（ドライブ文字・UNC・`\` で始まるもの）を決める。`;` を含むフォルダ名は `package.path` と同じく扱えない（今の制限と同じ）。`!` は Open Questions の 2 |
| 4.1〜4.4 依存 DLL | brief の計測 | **Missing**。**Constraint**: `LOAD_LIBRARY_SEARCH_DEFAULT_DIRS` は SSP.exe のフォルダ・`System32`・`AddDllDirectory` のフォルダを探し、`PATH` とカレントを探さない（4.3） |
| 5.1〜5.6 `package.loadlib` | — | **Missing** |
| 6.1〜6.6 文言 | `searcher.rs` の `load_error`・ラッパ・`module_searcher_test.rs` | **Missing**。**Constraint**: 理由は `FormatMessageW`→UTF-8 にする（6.4）。末尾の `\r\n` を残すかは設計で決める（LuaJIT は残す） |
| 7.1〜7.5 寿命 | LuaJIT の `_LOADLIB` と同じ形（ハンドルの userdata をレジストリに置き、`__gc` で解放） | **Missing**。**Unknown**: 閉じるときの `__gc` の順。LuaJIT は閉じるときに userdata を新しいものから順に後始末する見込みなので、`luaopen` より前に作ったハンドルの userdata は、モジュールが作った userdata より後に解放される（brief の方針）。実測が要る |
| 8.1〜8.3 C API の公開 | `pasta_shiori/build.rs`（link-arg を足せる）・`lua51.lib` の名前の一覧 | **Missing**。**Constraint**: rustc は自分の `/DEF:` を渡すので、2 つ目の `/DEF:` は衝突しうる。`/EXPORT:<名前>` を並べるのが安全。`/EXPORT` があれば `/OPT:REF` で使っていない API も残る（未計測） |
| 8.4 作る人へ渡すもの | — | **Missing**。`lib /def:<一覧>.def /name:pasta.dll /machine:x86\|x64` で取り込み用のライブラリが作れる。一覧を 1 つの正にし、build.rs もそれを読む形にすると食い違わない |
| 8.5〜8.6 | — | **Missing** |
| 8.7 自分の pasta.dll を使う | — | **Unknown（高）**。案: 読み込みの後、`luaopen` の前に、モジュールの取り込みの表（IAT）の `pasta.dll` の項を、読んだゴーストの pasta.dll（`GetModuleHandleExW` の `FROM_ADDRESS` で自分を得る）の関数に書き換える。`VirtualProtect` が要る。縮めた案は「行き先を確かめて違えば失敗」。どちらも PE の取り込みの表を読む処理が要る |
| 8.8 入口を変えない | `windows.rs` | 足すだけなら変わらない。`DllMain` の名前（x86 の `_DllMain@12`）も変えない |
| 9.1〜9.6 変えないこと | `module_searcher_test.rs`・`byte_invariant_test.rs`（pasta_shiori） | 既存のテストで守れる。9.4 は `install_module_searcher` の確かめを C 側でも共有する |
| 10.1〜10.10 テスト | `NON_ANSI_DIR_NAME`・`make_deep_dir`・`#[ctor]` の `PASTA_DEBUG` 中和 | **Missing**（テスト用 DLL の作り方）。10.6 は本物の pasta.dll を読む必要がある。**Unknown**: `cargo test` が `cdylib` を作るか・その場所の求め方（`target/<triple>/debug/pasta.dll`）。今の `pasta_shiori` のテストは pasta.dll を読んでいない（rlib を直接使う） |
| 11.1〜11.5 文書 | `book/`・`gen-skill-refs.mjs` | **Missing**。スキルの書き込みは拒まれることがある |

## 4. 実装の選択肢

### Option A: `searcher.rs` を広げる

C の searcher・`loadlib`・寿命をすべて `searcher.rs` に足す。

- ✅ 手本の隣で読みやすい。設置（`install_module_searcher`）を 1 か所にできる。
- ❌ 622 行が 1,000 行を超える見込み（目安の上限は 600 行）。Windows 専用の unsafe（`LoadLibraryExW`・`GetProcAddress`・IAT の書き換え）が、OS に依らない `.lua` の searcher と混ざる。

### Option B: 新しいファイルに分ける

- `runtime/c_searcher.rs`（仮）: 候補の展開（`candidate_paths` を `pub(crate)` にして使い回す）・`-` と croot の規則・文言の組み立て・Lua への登録（`loaders[3]`・`[4]`・`package.loadlib`）。OS に依らずテストできる部分。
- `runtime/dll.rs`（仮・`cfg(windows)`）: `LoadLibraryExW`・`GetProcAddress`・`FreeLibrary`・`FormatMessageW`・IAT の書き換え（8.7）。薄い unsafe の層。
- `loader/context.rs`: 既定の cpath を作る関数。
- `pasta_shiori/build.rs`＋公開する関数の一覧（`.def`）: Requirement 8。
- ✅ brief の Boundary Candidates（探索・読み込み・登録）とそのまま合う。unsafe の範囲が狭い。
- ❌ ファイルが増える。`searcher.rs` との共有部分（`candidate_paths`・`DIR_SEP`・ラッパ）の見せ方を決める必要がある。

### Option C: 混ぜる（推奨に近い）

設置は `install_module_searcher` を広げて 1 回の確かめ（4 要素）で 2〜4 番目を置き換え、中身は Option B の新しいファイルに置く。

- ✅ 「並びの確かめ」が 1 か所に残る（9.4）。ファイルの大きさも保てる。
- ❌ `install_module_searcher` の名前と役目が広がる（名前を変えるなら公開の API なので注意。`pasta_lua::runtime::install_module_searcher` はテストから使われている）。

### C API（Requirement 8）の選択肢

| 案 | 中身 | 利点 | 欠点 |
| --- | --- | --- | --- |
| (a) pasta.dll から公開 | build.rs で `/EXPORT` を約 148 個。作る人は pasta.dll 用の取り込みライブラリでリンクする | LuaJIT と確実に同じもの。リンカーの指示だけ | 2.4 の危険。x86・x64 の取り込みライブラリが要る |
| (b) (a)＋転送用の `lua51.dll` | `lua_pushstring = pasta.lua_pushstring` の形の DLL を添える | 普通の `lua51.lib` で作ったものが動く | (a) が前提。DLL を 2 つ配る。`lua51.dll` の名前の衝突。2.4 の危険はそのまま |
| (c) 範囲外 | ローダーだけ | 小さい | 実際の C モジュールは読めない。spec の値打ちが下がる |
| (a′) (a)＋つなぎ直し | (a) に 8.7 の IAT の書き換えを足す | ゴーストが何体いても自分の LuaJIT を使う | PE の取り込みの表を扱う unsafe の処理が増える。危険は設計で実測 |

`lua51.dll` に対して作ったモジュールまで IAT の書き換えで救う案も考えられるが、読み込みの時点で `lua51.dll` が見つからなければ読み込み自体が失敗する（取り込みを解かずに読む `DONT_RESOLVE_DLL_REFERENCES` は非推奨で、`DllMain` も走らない）。救うなら (b) の転送用 DLL が要る。

## 5. テスト用の C モジュール DLL

| 案 | 中身 | lock・CI | 欠点 |
| --- | --- | --- | --- |
| (i) テストの中でコンパイル（推奨） | `cc` を `pasta_lua`（と 10.6 用に `pasta_shiori`）の dev の依存に足す。`cc::windows_registry::find_tool(target, "cl.exe")` で環境変数つきの `cl` を得て、`/LD x.c /Fe:…` で DLL を作る。Lua の C API を使うモジュールは、公開の一覧から `lib.exe` で取り込みライブラリを作ってリンクする | `cc` 1.6.0 は lock に既にある（luajit-src などの依存）。lock は `pasta_lua` の依存の行が 1 つ増えるだけ。CI の windows-latest には VS があり、x86・x64 のジョブでそのターゲットの DLL ができる（`cfg!(target_arch)` で選ぶ） | テストの初回が遅くなる。`cc::Build` は静的ライブラリしか作らないので、コマンドを自分で組む。ビルドスクリプトの外なので target・host・out_dir を明示する |
| (ii) テスト用の crate | `crates/` に cdylib の crate を足す | lock とワークスペースが変わる。cdylib のパスの求め方が要る | ファイルが増える。publish の対象外にする手間 |
| (iii) バイナリをコミット | x86・x64 の DLL を `tests/fixtures/` に置く | lock は変わらない | 中身の確かめ・作り直しの手順・バイナリのコミット |

brief は `Cargo.lock` の席を `talk-grapheme-clusters` に渡している。(i) でも lock の 1 行が変わるので、マージの順を決める必要がある（Open Questions の 5）。

この機械には `cl.exe`（VS 2022 Professional・`14.44.35207`・`Hostx64\x86` と `Hostx64\x64`）と `dumpbin.exe` がある。

## 6. 規模と危険

- **規模: L（1〜2 週間）。** 探して読む部分だけなら M（brief の 10〜13 タスク）。Requirement 8（公開・作る人へ渡すもの・8.7 のつなぎ直し・本物の pasta.dll を読むテスト）を入れて L。brief の見積もり（14〜17 タスク）と合う。
- **危険: 中〜高。** 探して読む部分は手本があり低い。8.7（IAT の書き換え）と、閉じるときの `__gc` の順（7.3）は未計測で高い。x86 の `/EXPORT` と cdecl の飾りの対応も未確認。

## 7. 設計への申し送り

### 推奨の進め方

- Option C（設置は 1 か所、中身は新しいファイル）。C API は案 (a′)（公開＋つなぎ直し）。テスト用 DLL は (i)。
- 候補の作り方は `candidate_paths` を使い回し、`package.path` と同じ書き方（`/` と `\` が混ざる）で `no file` の行に出す。読み込みのときだけ `\` へそろえる。
- 理由の文言は `FormatMessageW` の UTF-8。比べるテストは理由の部分を Windows の理由の文言（同じエラー番号）で置き換えてから比べる。

### Research Needed

1. 同じ版の pasta.dll を 2 つのフォルダから 1 つのプロセスに読み込み、`pasta.dll` を取り込む C モジュールの結びつき先を実測する（2.4・8.7）。
2. IAT の書き換えの方法と安全さ（読み込みの直後・`luaopen` の前・`DllMain` の中でモジュールが Lua の関数を呼ばない前提）。縮めた案（確かめて止める）との比べ。
3. x86 の `/EXPORT:lua_pushstring` が `_lua_pushstring` に当たること、`/OPT:REF` で落ちないこと、rustc の `.def` と並べて衝突しないことを、作った pasta.dll の `dumpbin /exports` で確かめる。
4. Lua の状態を閉じるときの userdata の後始末の順（7.3）。`luaopen` の前に作ったハンドルの userdata が、モジュールの userdata より後に解放されることを実測する。
5. `LoadLibraryExW` に `/` を含むパスや相対パスを渡したときの失敗の仕方（`LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR` は完全なパスを要す）。
6. `cargo test --target i686-pc-windows-msvc` で `pasta_shiori` の `cdylib`（pasta.dll）ができる場所と、テストからの求め方（10.6）。
7. `luaJIT_BC_` を生の `luaL_loadbuffer` で読む方法（mlua の中から生の状態を得る道。`create_c_function` の中なら得られる）。
8. 公開する関数の一覧の決め方（`lua51.lib` から取るか、ヘッダーから取るか）と、LuaJIT の更新で増減したときに気づく仕組み（一覧と `lua51.lib` の照合テストなど）。
