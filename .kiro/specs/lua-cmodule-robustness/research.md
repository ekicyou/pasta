# ギャップ分析: lua-cmodule-robustness

- 日付: 2026-10-18（base: `5c550856`）
- 対象: `requirements.md`（requirements-generated・未承認）
- 調べ方: リポジトリのコード・文書を読み、`C:\rust\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\` の `luajit-src-210.6.6+707c12b`・`mlua-sys-0.10.0`・`mlua-0.11.6` のソースを読んだ。別のワークツリーにある作り済みの `pasta.dll`（x86・x64）を `dumpbin` で調べた。ビルドとテストは走らせていない。

## 0. 2026-10-18 の方針転換（FFI の道）と設計への申し送り

要件の話し合いで、C モジュールを `require` で読む道をやめ、LuaJIT の FFI の `ffi.load` で DLL を読む道に替えた（brief の「2026-10-18 要件の話し合いの決定」）。**下の 1〜7 節は方針転換の前の C モジュールの道の分析**で、参考に残す。使い回せるのは、2.1 のファイルの表（`searcher.rs`・`module_registry.rs`・`context.rs`・`Cargo.toml` の行）と、5 節のテスト用 DLL の作り方（案 (i)）。2.3・2.4・8 節の C API の話と IAT の書き換えは、FFI の道では要らない。

### 0.1 FFI の道で確かめた事実

- 本番の VM は `RuntimeConfig::new()`（`pasta_shiori/src/shiori.rs:106`）の `default_libs()`（`std_all` ほか）で作る。`std_all` は mlua の `ALL_SAFE`（`(1 << 30) - 1`。FFI と debug を含まない）。`[lua] libs` は `pasta-toml-logging-consistency` で消したので、今は FFI を開く手段が無い。`[loader]`（`LoaderConfig`・`loader/config/mod.rs:274`）は `@pasta_config` に現れない。
- mlua は FFI を `requiref(state, "ffi", luaopen_ffi, 1)`（`mlua-0.11.6/src/state/raw.rs:1609`）で開く。グローバルの `ffi` と `package.loaded.ffi` は同じ表。
- LuaJIT の `ffi.load`（`lib_ffi.c:810`）は `lj_clib_load` を呼び、関数の環境（`curr_func(L)->c.env`）から `CLibrary` のメタ表を得る。元の関数をそのまま呼べば、包んでも型は保たれる。
- `clib_loadlib`（`lj_clib.c:211`）: `LoadLibraryExA(clib_extname(name), NULL, 0)`。`clib_needext` は名前に `/`・`\`・`.` のどれも無いときだけ `.dll` を足す。`global` は使わない。失敗は `clib_error` が `cannot load module '%s': %s`（`FormatMessageA`）を `lj_err_callermsg` で投げる（呼び出し元の位置が付く）。
- `clib_unloadlib` は `CLibrary` の後始末で `FreeLibrary` する。pasta が先に読んだハンドルを別に持てば、参照の数で DLL が保たれる。

### 0.2 実測（2026-10-18・Win11 26300・ACP 932・LongPathsEnabled=1）

スクラッチ（リポジトリの外）で、マニフェストの無い C の試しの EXE を x64 と x86 の両方で作って走らせた。DLL は 539 文字・絵文字とカタカナ入りのフォルダに置いた。`foo.dll` は隣の `dep.dll` を取り込む。

| 手順 | 結果（x64・x86 とも） |
| --- | --- |
| `LoadLibraryExA(ACP のフルパス, NULL, 0)`（今の `ffi.load` と同じ） | 失敗（126） |
| `LoadLibraryExW(フルパス, NULL, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR\|LOAD_LIBRARY_SEARCH_DEFAULT_DIRS)` | 成功 |
| 続けて `LoadLibraryExA("foo.dll", NULL, 0)` | 成功。上と同じハンドル。`dep.dll` を通った関数の値も正しい |
| 別のフォルダの同じ名前の `foo.dll` を `LoadLibraryExW` で読む | 成功。1 つ目と別のハンドル |
| その後の `LoadLibraryExA("foo.dll", NULL, 0)` | **1 つ目が返る**（取り違え） |
| 2 つ目を読む前の `GetModuleHandleW(L"foo.dll")` | 1 つ目を返す（事前に気づける） |

### 0.3 設計への申し送り（FFI の道）

- 置き換えの流れ: 名前を解く → 取り違えの確かめ（`GetModuleHandleW`）→ `LoadLibraryExW` で先に読む → 元の `ffi.load` に短い名前を渡す → 先に読んだハンドルを Lua の状態を閉じ終わるまで保つ。
- 取り違えの決め方（Requirement 5.2）: パスが ANSI で表せるときにフルパスを元の `ffi.load` に渡す案（深いパスで `LoadLibraryExA` が通るかの実測が要る。brief の起票時の計測では 480 文字の `LoadLibraryExA` が通ったが、`LongPathsEnabled=1` の機械）と、いつもファイル名を渡して取り違えなら失敗させる案を比べる。`GetModuleHandleW` が返したモジュールのパス（`GetModuleFileNameW`）と自分のパスの比べ方（8.3 名・大文字小文字・`\\?\`）も決める。再読み込みで前の状態の同じファイルを読む場合（5.4）を含める。
- ファイル名が ANSI で表せるかの判定（2.8）: `WideCharToMultiByte` の `lpUsedDefaultChar` などで確かめる。
- 先に読んだハンドルの解放（7.2）: Lua の状態を閉じた後（`PastaLuaRuntime` の `Drop` の中で、`Lua` を落とした後）に `FreeLibrary` する形か、userdata の `__gc` に任せる形か。`CLibrary` と `ffi.gc` の後始末より後になることを保証する方を選ぶ。
- `LongPathsEnabled=0` での `LoadLibraryExW` の 260 文字超え（2.5）: GitHub の windows-latest ランナーの設定を確かめ、`0` の環境で確かめる道（ランナーの設定・別のジョブ）を決める。開発機の設定は変えない。
- エラーの理由（6.2）: `FormatMessageW` → UTF-8。末尾の `\r\n` の扱い（LuaJIT の `FormatMessageA` は残す）を決める。
- 置き場所: `ffi.load` の置き換えは `searcher.rs` に足さず、新しいファイルに分ける（`searcher.rs` は 622 行で、目安の 600 行を超えている）。Windows 専用の層は `cfg(windows)` で分ける。
- テスト用 DLL: 5 節の案 (i)。FFI の DLL は Lua の C API に頼らないので、取り込み用のライブラリは要らない（依存 DLL の `.lib` は `cl` が作る）。

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

## 8. 設計フェーズの調査と決定（2026-10-18・FFI の道）

### Summary

- **Feature**: `lua-cmodule-robustness`
- **Discovery Scope**: Extension（既存のローダーと VM に、`ffi.load` の置き換えを足す）。light の調べ方に、mlua・LuaJIT のソースの確かめを足した。
- **Key Findings**:
  - mlua の app data（`Lua::set_app_data`）は `ExtraData` に入り、`RawLua` の `Drop` が `lua_close` を呼び終えた後に落ちる。DLL の参照をここに持たせれば、`CLibrary` の `__gc`・`ffi.gc` の後始末がすべて済んでから `FreeLibrary` できる。`PastaLuaRuntime` のフィールドの並びにも、`Lua` の複製の数にも左右されない。
  - LuaJIT の `lua_close` は `cpfinalize` で cdata の後始末（`lj_gc_finalize_cdata`）→ userdata の後始末（`lj_gc_finalize_udata`）の順に最大 10 回回す。userdata の `__gc` で解放する形は、この順と回り方に頼ることになるので採らない。
  - 元の `ffi.load` を末尾呼び出しにすれば、元の関数が投げる文言の位置は作者のコードの位置になる（`lj_err_callermsg` は C の関数の枠の 1 つ前の Lua の枠の位置を付ける）。pasta が投げる失敗は Lua のラッパの `error(msg, 2)` で同じ書式になる。

### Research Log

#### DLL の参照をいつ解放するか（申し送り 7.2）
- **Context**: `PastaLuaRuntime` の `Drop` の後（`Lua` を落とした後）か、userdata の `__gc` か。
- **Sources Consulted**: `mlua-0.11.6/src/state/raw.rs`（`impl Drop for RawLua` が `ffi::lua_close` を呼び、`extra: XRc<UnsafeCell<ExtraData>>` はその後にフィールドとして落ちる）、`src/state/extra.rs`（`app_data: AppData`）、`luajit2/src/lj_state.c`（`lua_close` と `cpfinalize`）。
- **Findings**: 第 3 の道として app data がある。`ExtraData` は `lua_close` が終わった後に落ちるので、そこで `FreeLibrary` すれば後始末の順に頼らない。`PastaLuaRuntime` のフィールドに持たせる形は、`Lua` の強い参照がほかに残ると VM より先に解放してしまう。
- **Implications**: `DllPins` を app data に置く。`PastaLuaRuntime` は変えない。

#### LuaJIT の `ffi.load` の細部
- **Sources Consulted**: `luajit2/src/lj_clib.c`（`clib_error`・`clib_needext`・`clib_extname`・`clib_loadlib`）、`lib_ffi.c:810`、`lj_lib.c`（`lj_lib_checkstr`）、`lj_err.c`（`lj_err_callermsg`）。
- **Findings**:
  - `clib_error` は `FormatMessageA(FROM_SYSTEM | IGNORE_INSERTS, NULL, err, 0, buf, 128, NULL)`。言語の指定は 0、バッファは 128 バイトで、足りなければ理由は空になる。末尾の `\r\n` は残る。文言の `<名前>` は拡張子を足す前の名前。
  - `lj_lib_checkstr` は数を文字列にする。文字列でも数でもなければ `bad argument #1 to 'load'`。ラッパは文字列でない引数を元の関数へそのまま渡せば同じになる。
  - `global` は `L->base+1 < L->top && tvistruecond(L->base+1)` で読むだけ。型の誤りにならない。ラッパは `...` で渡す。
- **Implications**: 理由の取り出しは `FormatMessageW` を同じ引数（言語 0）で呼び、末尾の `\r\n` を残す。std の `io::Error` の表示は末尾を削り `(os error N)` を足すので使わない。

#### Windows の拡張子の規則
- **Context**: 区切りのある名前（`ffi.load("lib/foo")`）で、LuaJIT は `.dll` を足さない（`clib_needext` が偽）。
- **Findings**: `LoadLibraryEx` は、パスの有無によらず拡張子の無いファイル名に `.dll` を補う（末尾の `.` は「拡張子なし」の印）。今の `ffi.load("lib/foo")` は `lib/foo.dll` を読む。
- **Implications**: 「最後の部分に `.` が無ければ `.dll` を足す」を 1 つの規則にする。区切りの無い名前では `clib_needext` と同じ結果になる。

#### 取り違えの判定（申し送り 5.2）
- **Findings**: 読んだ後の `GetModuleHandleW(<ファイル名>)` が今回のハンドルと同じかを見れば、元の `ffi.load` が名前で開いたときに返るものそのものを確かめられる。読む前に確かめるより競合の窓が狭く、パスの文字列の比較（8.3 名・大文字小文字・`\\?\`）が要らない。
- **Implications**: パスの比較の規則は作らない。8.3 名と長い名前の綴りの違いをローダーが同じモジュールと見るかは未計測（設計の Open Questions の 2）。

#### ANSI で表せるかの判定（申し送り 2.8）
- **Findings**: `crate::encoding` の `Encoding::ANSI.to_bytes` は `WC_COMPOSITECHECK` だけを付けて `WideCharToMultiByte` を呼び、既定の文字が使われたら失敗にする。best-fit の置き換え（似た文字への黙った置き換え）は既定の文字の使用にならない。
- **Implications**: 新しい Win32 の呼び出しは作らず、`to_bytes` の結果を `to_string` で戻して元と比べる（往復の比較）。

#### 長いパスの設定が無効な環境（申し送り 2.5）
- **Findings**: マニフェストの無い EXE（`cargo test` の EXE も同じ）は、文書の上では `longPathAware` でないので、レジストリの `LongPathsEnabled` の値によらず長いパスが無効になるはず。2026-10-18 の実測（`LongPathsEnabled=1`・マニフェストなし）で 539 文字の `LoadLibraryExW` が通ったのは、文書の説明と食い違う。どちらが正しいかは、`LongPathsEnabled=0` の機械で確かめるまで分からない。
- **Implications**: 260 文字以上だけ `\\?\` の形にする（Node.js の `process.dlopen` が `\\?\` の形を `LoadLibraryExW` に渡しているのと同じ考え方）。CI の各行列で `LongPathsEnabled=0` にして FFI のテストを走らせ、テストの最初に「接頭辞なしの長いパスが本当に通らない」ことを確かめて、設定が効いていないのに緑になるのを防ぐ。開発機のレジストリは変えない。
- **追記（2026-10-19・タスク 1.2 の試し打ち）**:
  - ローダー（`LoadLibrary*`）は長いパスの設定の外にある。MS の文書が設定の対象に挙げるのはファイルとフォルダの API だけで、ローダーは一覧に無い。開発機（マニフェストなしの EXE と `longPathAware` の EXE）の実測でも、CI（run 39139127106・`LongPathsEnabled=0`）でも、接頭辞なしの長いパスの DLL が読めた。上の「食い違い」はこれで説明が付く。
  - そのため「接頭辞なしの長いパスが通らない」ことは前提の確かめに使えない。`GetFileAttributesW` もマニフェストの無い EXE ではレジストリによらず失敗するので、それだけでは手順が効いた証にならない。前提の確かめは `GetFileAttributesW`（プロセスの側）に加え、`RegGetValueW` で `LongPathsEnabled` を直に読む（OS の側。無いか 0 なら成り立つ）。
  - `\\?\` の形で読むと、隣の `dep.dll` も解けて `via_dep()` が呼べる（開発機の x86・x64 で実測）。`LongPathsEnabled=0` の CI（run 39141129339）でも x86・x64 とも `\\?\` の形で読め、`via_dep()` が呼べた。8.3 名の道は使わない（タスク 1.2 で確定）。
  - `\\?\` の形で読んだモジュールに `GetModuleFileNameW` が返すパスは、`\\?\` の付いた形になる（要件 2.7 のエラーの文言・パスの見せ方に関わる）。
  - 0.2 の実測で `LoadLibraryExA` が失敗したのは、パスに ANSI に無い文字が入っていたため。ASCII だけの 344 文字のパスなら `LoadLibraryExA` で読める（開発機・`LongPathsEnabled=1`）。

#### テストの並列とプロセスの中の DLL
- **Findings**: 同じテストのバイナリの中のテストは同じプロセスの別のスレッドで並列に走る。同じ名前の `foo.dll`・`dep.dll` を別のフォルダから読むと、2 つ目は取り違えになり、依存 DLL の無いことのテストは別のテストが読んだ `dep.dll` で通ってしまう。
- **Implications**: FFI のテストは新しいバイナリ `tests/ffi/` に分け、中では 1 つの `Mutex` で直列にする。

#### テスト用 DLL のビルド
- **Sources Consulted**: `cc-1.6.0/src/lib.rs:343`（`windows_registry::find_tool(arch_or_target, tool) -> Option<Tool>`）。`Cargo.lock` の `cc 1.6.0`。
- **Findings**: `find_tool` は対象（x86・x64）の `cl.exe` を、`INCLUDE`・`LIB`・`PATH` つきの `Tool` で返す。`cl` の既定のランタイムは `/MT`（静的）だが、明示する。CI の windows-latest は `Set-WinSystemLocale ja-JP` を再起動なしで走らせているので、ANSI のコードページは 1252 のままの見込み（理由の文言は英語になる）。
- **Implications**: `dll_fixture.rs` が `OnceLock` で 1 回だけ作る。

### Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 先読み＋短い名前（採用） | `LoadLibraryExW` で先に読み、元の `ffi.load` にファイル名を渡す | LuaJIT を改造しない。`CLibrary` の型・後始末は元のまま。実測で確かめた | ファイル名は ANSI に限る。同名の別 DLL があれば失敗 | brief の方針 |
| ANSI ならフルパスを渡す | 表せるときだけ元の `ffi.load` にフルパス | 取り違えを避けられる場合が増える | フォルダ名の文字種で結果が変わる。深いパスでは `LoadLibraryExA` の長さの制限 | 採らない |
| `CLibrary` を自分で作る | `lj_clib` の内部を真似る | 名前の制限が無い | LuaJIT の内部の構造に依存（改造と同じ危険） | 範囲外 |

### Design Decisions

#### Decision: 置き場所を `runtime/ffi_load/` の 3 ファイルにする
- **Context**: `searcher.rs` は 622 行。Windows 専用の unsafe を、OS に依らない規則から分けたい（申し送り）。
- **Alternatives Considered**: 1. `searcher.rs` に足す 2. 1 ファイル 3. 規則・Win32・登録の 3 ファイル
- **Selected Approach**: 3（`mod.rs`・`resolve.rs`・`win32.rs`）。モジュール全体を `cfg(windows)` にする（Windows 以外では LuaJIT のまま）。
- **Rationale**: brief の境界の候補（名前の解決・読み込み・登録）とそのまま合い、unsafe を 1 ファイルに閉じ込められる。
- **Trade-offs**: ファイルが 3 つ増える。
- **Follow-up**: 各ファイル 600 行未満を保つ。

#### Decision: 名前だけで見つからない場合も先に読む
- **Context**: 3.4（そのまま元へ渡す）と 6.2（理由は UTF-8）がぶつかる。元へそのまま渡すと、失敗の理由が ANSI になる。
- **Selected Approach**: `LoadLibraryExW(name, NULL, 0)`（LuaJIT と同じ探索）で先に読み、成功したらファイル名を元へ渡す。
- **Rationale**: OS の DLL が今までどおり読めるという 3.4 の目的を保ち、すべての失敗の理由を UTF-8 にできる。
- **Follow-up**: 話し合いで確かめる（設計の Open Questions の 4）。

#### Decision: 生成（Synthesis）の結果
- **Generalization**: 3 つの名前の形（名前だけ・相対・絶対）を、1 つの `resolve` と 1 つの拡張子の規則にまとめた。pasta の 3 つの失敗も、LuaJIT と同じ 1 つの文言の形の理由の部分に入れる。
- **Build vs Adopt**: 探索のフォルダは `LoaderContext::absolute_search_paths()`、ANSI の判定は `crate::encoding`、FFI の有効化は既存の `std_ffi` を使う。新しい依存は dev の `cc` だけ（lock に既にある）。
- **Simplification**: パスの文字列の比較（8.3 名・大文字小文字・`\\?\`）をやめ、ハンドルの一致だけで判定する。DLL の参照の寿命は app data に任せ、`PastaLuaRuntime` を変えない。FFI を有効にしたときの警告のログは足さない（既存の `libs` の debug ログで足りる）。

### Risks & Mitigations

- `\\?\` の形での依存 DLL の解決が `LongPathsEnabled=0` で失敗する — CI の手順で毎回確かめる。失敗したら既知の制限にする要件の変更を話し合う。
- 8.3 名と長い名前の綴りの違いで、同じファイルを別のモジュールとして読む — `collision_test.rs` で確かめ、必要ならファイルの識別子の比較を足す。
- 確かめと元の `ffi.load` の間の競合（別のスレッドが同名の別 DLL を読む） — 窓は数命令。既知の制限として書く。
- 末尾呼び出しの位置が LuaJIT と同じになるかは枠の作りに頼る — `compare_test.rs` でバイト比較する。

### 使ったスキルと指針

- `kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、`kiro-validate-design`（`design-review.md`）。
- `ponytail`（最小の差分・既存の部品の使い回し・推測の抽象を作らない）。

## 9. 設計の話し合いの決定（2026-10-18）

8 件の議題をすべて推奨案で確定した。中身は design.md の「決定事項」にある。根拠の要点:

- 取り違えは失敗にする。フルパスを渡す案は、フォルダ名の文字種で結果が変わり、深いパスでは `LoadLibraryExA` の長さの制限にかかる。
- 同じファイルの判定はローダーのハンドルの一致に任せる。8.3 名を解いて文字列で比べる道は、CI だけで出る不具合（`RUNNER~1`）の元になる。8.3 名のテストは、短い名前が本当に違う文字列であることを先に確かめる。
- `LongPathsEnabled=0` は未計測のため、実装の最初のタスクを CI での試し打ちにする。駄目なら 8.3 名のパスを試し、それでも駄目なら要件を弱めずに開発者に戻す。
- 3.4 は「そのまま渡す」から「OS の既定の探索で読む」へ要件の文言を直した（失敗の理由を UTF-8 にするため）。
- 7.4 に「先読みの後に元の `ffi.load` が失敗した場合は状態を閉じるまで残す」の例外を足した。
- 理由の末尾の `\r\n` は残す。pasta の理由は英語。依存 DLL の同名の取り違えは防がず、既知の制限に書く。
