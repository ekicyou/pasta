# Brief: lua-cmodule-robustness

> **ステータス**: 要件定義中（2026-10-18 の `/kiro-discovery` で起票し、同じ日の `/kiro-start` で始めた）。0.4.2 に入れる（2026-10-18 開発者の決定）。
>
> **2026-10-18 の方針転換**: 要件の話し合いで、C モジュールを `require` で読む道（`package.loaders[3]`・`[4]`・`package.loadlib`）をやめ、**LuaJIT の FFI の `ffi.load` で DLL を読む道**に替えた（開発者の承認）。理由と実測は末尾の「2026-10-18 要件の話し合いの決定」にある。spec の名前はそのままにする。

## Problem

ゴーストの作者が、自分で作った DLL（や、ほかの人が作った C の DLL）の関数をゴーストから呼ぼうとすると、ゴーストのフォルダの場所しだいで読めない。そもそも今の pasta.dll では、DLL を呼ぶ手段が事実上無い。

- C モジュール（`luaopen_<名前>` を持つ DLL）を `require` で読む道は、Lua の C API（`lua_*`・`luaL_*`）が pasta.dll から 1 つも公開されておらず、`lua51.dll` も無いので、どこに置いても読めない。
- LuaJIT の FFI（`ffi.load`・`ffi.cdef`）は、既定のライブラリ `std_all`（mlua の `ALL_SAFE`）に含まれず、pasta.toml から有効にする手段も無い（`[lua] libs` は spec `pasta-toml-logging-consistency` で、効かない設定として消した）。
- 仮に FFI を有効にしても、`ffi.load` は次の場合に読めない。
  - ゴーストのフォルダが深い（パスが 260 文字を超える）。
  - フォルダ名に ANSI コードページに無い文字（日本語のロケールでの絵文字や CP932 の外の文字、日本語以外のロケールでの日本語）がある。
  - 名前だけ（`ffi.load("foo")`）や相対パスで書くと、ゴーストのフォルダではなく SSP.exe のフォルダ・SSP のカレント・`PATH` を探してしまう。
  - DLL の隣に置いた依存 DLL が見つからない（エラー 126）。

## Current State

- `.lua` の `require` は解決済み。spec `lua-require-robustness`（完了）が `package.loaders[2]` を Rust の searcher に差し替え（`crates/pasta_lua/src/runtime/searcher.rs`）、深いパスと非 ANSI のパスでも読める。`package.path` は `LoaderContext::generate_package_path`（`crates/pasta_lua/src/loader/context.rs`）がゴーストの `ghost/master` を基点にした絶対パスで作る。
- 実際の pasta.dll は、いつも `RuntimeConfig::new()`（`crates/pasta_shiori/src/shiori.rs:106`）で VM を作る。既定のライブラリは `default_libs()`（`crates/pasta_lua/src/runtime/runtime_config.rs:16`）の `std_all` ほか。FFI は入らない。
- LuaJIT（`luajit-src 210.6.6+707c12b`）の `ffi.load` は `lj_clib.c` の `clib_loadlib`。
  - 読み込み: `LoadLibraryExA(name, NULL, 0)`（`LJ_WIN_LOADLIBA`・214 行）。ANSI 版で、flags が 0 なので DLL の隣の依存 DLL を探さない。
  - 名前に `/`・`\`・`.` のどれも無ければ、末尾に `.dll` を足す（`clib_needext`・`clib_extname`）。
  - 2 番目の引数（`global`）は Windows では使わない（`UNUSED(global)`）。
  - 失敗: `cannot load module '<名前>': <理由>`（`clib_error`。理由は `FormatMessageA`＝ANSI）。呼び出し元の位置が前に付く（`lj_err_callermsg`）。
  - `ffi.load` が返す値（`CLibrary`）の後始末で `FreeLibrary` する。
- mlua は FFI を `requiref(…, "ffi", luaopen_ffi, 1)` で開く（`mlua-0.11.6/src/state/raw.rs:1609`）。グローバルの `ffi` と `package.loaded.ffi` は同じ表になる。
- pasta.dll の側から、プロセス単位の設定（マニフェストの `longPathAware`・`activeCodePage=UTF-8`）は変えられない。どちらもホストの EXE が決める。
- 書き手向けの文書と作例に、DLL を呼ぶ例は無い。

## Desired Outcome

- ゴーストの作者が pasta.toml の `[loader] ffi = true` で FFI を有効にできる。既定は無効のまま。
- FFI を有効にしたゴーストでは、ゴーストのフォルダが深くても、ANSI に無い文字を含んでも、`ffi.load("foo")` がゴーストのフォルダの `foo.dll` を読み、`ffi.cdef` で宣言した関数を呼べる。
- 名前だけで書いたときは、`package.path` と同じ探索のフォルダ（`lua_search_paths`）から探す。見つからなければ、今の `ffi.load` のまま OS に探させる（`ffi.load("user32")` などは今までどおり動く）。
- 相対パスは、ゴーストの `ghost/master` を基点にして解決される。
- DLL の隣に置いた依存 DLL が解決される。
- 同じファイル名の別の DLL を別のゴーストが先に読んでいても、黙ってそちらに結びつかない。
- 失敗したときの文言は、LuaJIT のものと同じ形を保つ（理由の部分だけは UTF-8 にする）。

## Approach

LuaJIT は改造しない。`lua-require-robustness` と同じく、ホストの Rust 側で `ffi.load` を包む。

- `[loader] ffi = true` のときだけ、VM を作るときに FFI を開く（`StdLib::FFI` を足す）。
- 開いた `ffi` の表の `load` を、Rust で作った関数に置き換える。中で元の `ffi.load` を呼ぶ（元の関数の環境（`c.env`）が `CLibrary` の型を持つので、元の関数をそのまま呼ぶ）。
- 置き換えた `ffi.load(name, global)` の流れ（2026-10-18 に実測した形）:
  1. 名前を解く。名前だけなら `.dll` を足す規則を LuaJIT と同じにし、`lua_search_paths` のフォルダを順に探す。相対パスは `ghost/master` に結合する。どこにも無い名前だけの指定は、元の `ffi.load` へそのまま渡す。
  2. 見つけた DLL を、`\` 区切りの絶対パス（`\\?\` は付けない）の UTF-16 にして、`LoadLibraryExW(path, NULL, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS)` で先に読む。依存 DLL はここで解ける。
  3. 元の `ffi.load` には、ANSI で表せる短い名前を渡す。Windows は「同じモジュール名の DLL が既に読み込まれていれば、それを使う」ので、2. で読んだものが返る。
  4. 取り違えを防ぐ。2. の前に、同じモジュール名の別のファイルが既に読み込まれていないかを `GetModuleHandleW` で確かめる。決め方は設計で詰める（分かる文言で失敗させる、パスが ANSI で表せるならフルパスを渡して取り違えを避ける、など）。
- 2. で得たハンドルは、Lua の状態を閉じ終わってから `FreeLibrary` する（`CLibrary` の後始末より後）。
- `windows-sys` に `Win32_System_LibraryLoader` を足す。

## Scope

- **In**:
  - `[loader] ffi`（真偽値・既定 `false`）と、`true` のときに FFI を開くこと。
  - `ffi.load` の置き換え（深いパス・非 ANSI のフォルダ・隣の依存 DLL・名前だけの探索・相対パスの基点・取り違えの防止）。
  - 先に読んだ DLL のハンドルの寿命の管理。
  - エラーの文言を LuaJIT と揃えること、元の `ffi.load` との比較テスト。テスト用の小さな DLL（依存 DLL つき）を `cc` でテストの中で作ること。
  - 文書の更新: `book/src/reference/pasta-toml.md`（`[loader] ffi`）、`book/src/reference/startup.md`（DLL を FFI で使う・探す場所・既知の制限）、`book/src/internals/loader.md`（`ffi.load` の置き換え）。スキル `pasta-lua-coding` の `references/` の再生成。
- **Out**:
  - LuaJIT のソースの改造（fork・`[patch.crates-io]`）。
  - C モジュールを `require` で読む道（`package.loaders[3]`・`[4]`・`package.loadlib`・`package.cpath`）。LuaJIT のままにし、マニュアルに「C の DLL は FFI で使う」と書く。Lua の C API の公開もしない。動機が出たら別に起票する。
  - `io.open`・`loadfile`・`dofile`・`package.searchpath` の非 ANSI 対応（今の既知の制限のまま。`@enc`・`@pasta_persistence` で回避できる）。
  - ホストが ANSI の `load` しか呼ばないときに失われる文字（pasta.dll に届く前に失われる）。
  - FFI 以外のライブラリの組み合わせを pasta.toml で選べるようにすること（`[lua] libs` の復活）。

## Boundary Candidates

- 設定（`[loader] ffi` を読み、VM に FFI を開く）— `LoaderConfig` と `RuntimeConfig` の間。
- 名前の解決（`.dll` を足す規則・`lua_search_paths` の探索・相対パスの基点・存在の確認）— 純粋な Rust で、OS に依らずテストできる。
- 読み込み（`LoadLibraryExW`・`GetModuleHandleW`・`FreeLibrary`・`FormatMessageW`）— Windows 専用の薄い層。
- Lua への登録（`ffi.load` の置き換え、元の `ffi.load` を呼ぶラッパー、エラーの文言を揃える部分）— `searcher.rs` の `install_module_searcher` の隣。

## Out of Boundary

- サードパーティの DLL の中身。DLL が内部で `fopen` などの ANSI 版の API を使っていれば、深いパス・非 ANSI のフォルダではそこで失敗する。保証するのは「DLL を見つけて読み込み、`ffi.load` の値として返す」ところまで。
- DLL を配布物にどう同梱するか（`pasta_check release` の扱い）。動機が出たら別に起票する。

## Upstream / Downstream

- **Upstream**: `lua-require-robustness`（完了）の `searcher.rs` と `LoaderContext`。`pasta-toml-logging-consistency`（完了）が消した `[lua] libs` の経緯。mlua 0.11.6 の `StdLib::FFI`。LuaJIT の `lj_clib.c` の規則（`.dll` を足す規則・エラーの文言）。
- **Downstream**: DLL を使うゴーストと、その作り方の文書（今は無い）。

## Existing Spec Touchpoints

- **Extends**: `lua-require-robustness`（完了。`.kiro/specs/completed/lua-require-robustness/`）。深いパス・非 ANSI のパスの対応を、`.lua` から DLL へ広げる。
- **Adjacent**: `crates/pasta_lua/src/runtime/runtime_config.rs`（`default_libs`・`to_stdlib`）、`crates/pasta_lua/src/loader/config/mod.rs`（`LoaderConfig`）、`crates/pasta_lua/src/runtime/module_registry.rs`、`crates/pasta_lua/src/runtime/searcher.rs`、`crates/pasta_lua/src/loader/context.rs`。

## Constraints

- x86 と x64（MSVC・静的 CRT）の両方で動くこと。
- `lua_search_paths` を pasta.toml で上書きしたときも、`package.path` と `ffi.load` の名前だけの探索が同じフォルダを同じ順で指すこと。
- LuaJIT の `LoadLibraryExA` にパスを渡さないこと（深いパス・非 ANSI のパスで落ちる原因）。
- テストは非 ANSI のフィクスチャ `NON_ANSI_DIR_NAME`（`crates/pasta_lua/tests/common/mod.rs`）と、300 文字を超えるパス（`make_deep_dir`）で確かめる。
- スキル（`.claude/skills/`）の編集は分類器に拒まれることがあるので、そのときは開発者に頼む。

## 2026-10-18 要件の話し合いの決定

- **C モジュールの道をやめ、FFI の道にする（開発者の承認）。** 開発者の指摘「LuaJIT の FFI なら `.lib` も `.def` も要らないのでは」から。FFI で呼ぶ DLL は Lua を知らない普通の C の DLL で、作るときに Lua の C API へリンクしない。そのため、C モジュールの道で要った次のものが無くなる。
  - pasta.dll から Lua の C API（約 148 個）を公開すること。
  - 同じ SSP で pasta のゴーストを 2 体動かしたとき、後のゴーストの C モジュールが先のゴーストの pasta.dll に結びつく問題と、そのつなぎ直し（IAT の書き換え）。
  - 取り込み用のライブラリ（`.lib`）を作る人に渡すこと。
- **FFI は `[loader] ffi = true` で有効にし、既定は無効（開発者の承認）。** FFI は Lua から任意のメモリに触れる道具で、書き間違いで SSP ごと落ちうる。`debug`・`env` を既定で閉じているのと同じ考え方。`[lua]` の節は生き返らせない。`[loader]` は `@pasta_config` に現れない。
- **実測（2026-10-18・Win11 26300・ACP 932・LongPathsEnabled=1・リポジトリの外のスクラッチ）。** マニフェストの無い試しの EXE で、x64 と x86 の両方で同じ結果になった。DLL は 539 文字・絵文字とカタカナ入りのフォルダに置き、その `foo.dll` は隣の `dep.dll` に頼る。

  | 読み方 | 結果 |
  |---|---|
  | 今の `ffi.load(フルパス)` と同じ呼び方（`LoadLibraryExA(ACP のフルパス, NULL, 0)`） | 失敗（エラー 126） |
  | `LoadLibraryExW(フルパス, NULL, SEARCH_DLL_LOAD_DIR\|SEARCH_DEFAULT_DIRS)` で先に読む → `LoadLibraryExA("foo.dll", NULL, 0)` | 成功。同じハンドルが返り、隣の `dep.dll` を通った関数の値も正しい |
  | 別のフォルダの同じ名前の `foo.dll` を 2 つ目として `LoadLibraryExW` で読む → `LoadLibraryExA("foo.dll", NULL, 0)` | 1 つ目が返る（取り違え）。2 つ目を読む前の `GetModuleHandleW("foo.dll")` は 1 つ目を返すので、事前に気づける |

- **残る制限**:
  - DLL のファイル名そのものは ANSI で表せる文字に限る（元の `ffi.load` に渡すのがファイル名のため）。フォルダ名は自由。
  - `LongPathsEnabled=0` の機械で `LoadLibraryExW` が 260 文字を超えるパスを読めるかは未計測（システムの設定を変えて試すことはしない）。設計で確かめる道を決める。
- 話し合いのほかの決定: 依存 DLL を探す場所から `PATH` とカレントを外す。テスト用の DLL は `cc` を dev の依存に足してテストの中で作る（`Cargo.lock` は後からマージするほうが作り直す）。失敗の理由は UTF-8 にする。

## 2026-10-18 棚卸の再測定（main facd6cf4。方針転換の前の C モジュールの道について）

- この節は、方針転換の前に書いた。触るファイル・規模・穴の記述は C モジュールの道のもので、FFI の道では次のように変わる。
  - **触るファイル**: `runtime_config.rs`・`loader/config/mod.rs`（`ffi` のキー）、`searcher.rs` か新しいファイル（`ffi.load` の置き換え）、`module_registry.rs`、Windows 専用の新しいファイル、`crates/pasta_lua/Cargo.toml`（`Win32_System_LibraryLoader` と dev の依存 `cc`）、`.kiro/steering/tech.md`、テスト、`book/src/reference/pasta-toml.md`・`startup.md`・`internals/loader.md`、スキルの `references/` の再生成。
  - **規模**: 10 タスク前後（設定・名前の解決・読み込み・取り違えの防止・登録と文言・寿命・テスト用 DLL・比較テスト・文書）。
  - 元の「Lua の C API をどこから得るか」の穴は、FFI の道では無くなった。
- 元の記述（参考）: `searcher.rs` の `EXPECTED_LOADERS_LEN = 4`（37 行）と `install_module_searcher`（280 行）、`module_registry.rs` 44 行の `setup_package_path` が cpath を変えないこと、`book/src/internals/loader.md:215` は main と合っていた。`talk-grapheme-clusters` とは `crates/pasta_lua/Cargo.toml` の別の行と `tech.md` の依存の表だけが重なる。
