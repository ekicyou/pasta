# Brief: lua-cmodule-robustness

> **ステータス**: 未着手（2026-10-18 の `/kiro-discovery` で起票）。0.4.2 に入れる（2026-10-18 開発者の決定）。着手するときは `/kiro-start lua-cmodule-robustness` で開始する。

## Problem

ゴーストの作者が、C で書かれた Lua の拡張モジュール（DLL）を `require` で読もうとすると、ゴーストのフォルダの場所しだいで読めない。

- ゴーストのフォルダが深い（パスが 260 文字を超える）と読めない。
- フォルダ名に ANSI コードページに無い文字（日本語のロケールでの絵文字や CP932 の外の文字、日本語以外のロケールでの日本語）があると読めない。
- 相対パスで書くと、ゴーストのフォルダではなく SSP のカレントや SSP.exe のフォルダを探してしまう。
- 短い ASCII のパスでも、モジュールの隣に置いた依存 DLL が見つからず失敗する（エラー 126）。

## Current State

- `.lua` の `require` は解決済み。spec `lua-require-robustness`（完了）が `package.loaders[2]` を Rust の searcher に差し替え（`crates/pasta_lua/src/runtime/searcher.rs`）、深いパスと非 ANSI のパスでも読める。`package.path` は `LoaderContext::generate_package_path`（`crates/pasta_lua/src/loader/context.rs`）がゴーストの `ghost/master` を基点にした絶対パスで作る。
- C モジュールの側は手つかず。`package.loaders[3]`（C searcher）・`[4]`（croot searcher）・`package.loadlib` は LuaJIT の実装のまま。`package.cpath` は設定しておらず、LuaJIT の既定（`.\?.dll;!\?.dll;!\loadall.dll`）のまま。設計の文書 `book/src/internals/loader.md` にも「`package.cpath` は変えない」とある。
- LuaJIT（`luajit-src 210.6.6+707c12b`）の `lib_package.c` は ANSI 版の API を使う（2026-10-18 に確かめた）。
  - 読み込み: `LoadLibraryExA(path, NULL, 0)`（`lj_arch.h` の `LJ_WIN_LOADLIBA`）。flags が 0 なので、モジュールの隣の依存 DLL を探さない。
  - 存在の確認: `fopen(filename, "r")`。深いパスはここで落ちる（`LoadLibraryExA` 自体は、マニフェストの無い EXE でも 480 文字のパスを読めた）。
  - `!` の置き換え: `GetModuleFileNameA(NULL, …)` でホストの EXE（SSP.exe）のフォルダになる。バッファは `MAX_PATH`。
- pasta.dll の側から、プロセス単位の設定（マニフェストの `longPathAware`・`activeCodePage=UTF-8`）は変えられない。どちらもホストの EXE が決める。
- 書き手向けの文書と作例に、C モジュールを `require` する例は無い。
- 試しの計測は、調査のときにリポジトリの外で行った（Win11 26300・ACP 932・LongPathsEnabled=1）。

| 読み方 | 480 文字のパス | 絵文字を含むパス | 隣の依存 DLL |
|---|---|---|---|
| LuaJIT の今の経路（A 版・flags 0・`fopen`） | 失敗 | 失敗 | 失敗 |
| `LoadLibraryExW`＋`LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR\|LOAD_LIBRARY_SEARCH_DEFAULT_DIRS`（`\` 区切りの絶対パス） | 成功 | 成功 | 成功 |

## Desired Outcome

- ゴーストのフォルダが深くても、ANSI に無い文字を含んでも、`require "foo"` が `foo.dll` を見つけて読み込み、`luaopen_foo` を呼べる。
- `package.cpath` に書いた相対パスは、pasta.dll の置いてあるフォルダ（ゴーストの `ghost/master`）を基点にして解決される。
- モジュールの隣に置いた依存 DLL が解決される。
- `package.loadlib` も同じ規則で読める。
- 失敗したときのエラーの文言は、LuaJIT のものと同じ形を保つ（`lua-require-robustness` と同じ方針）。

## Approach

LuaJIT は改造しない。`lua-require-robustness` と同じく、ホストの Rust 側で C モジュールのローダーを差し替える（2026-10-18 開発者の決定。推奨案のまま）。

- `package.loaders[3]`・`[4]`・`package.loadlib` を、Rust で作った関数に置き換える。テーブルごと入れ替えず、中身を書き換える（`require` が読むのは `package.loaders`。luajit52 では `package.searchers` と同じテーブル）。
- 探索: `package.cpath` を UTF-8 として読み、`;` で区切り、`?` を置き換える（モジュール名の `.` は `\`）。相対のテンプレートはゴーストの基点のフォルダに結合する。存在の確認は Rust std（wide API）で行う。
- 読み込み: `\` 区切りの絶対パス（`\\?\` は付けない。`GetModuleFileNameW` の戻り値にも付いてしまうため）を UTF-16 にして、`LoadLibraryExW(path, NULL, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS)` で読む。
- 入口: `GetProcAddress` で `luaopen_<名前>` を引く（最初の `-` までを捨て、`.` を `_` にする LuaJIT の規則のとおり）。得た `lua_CFunction` を mlua の `create_c_function`（unsafe）で包んで返す。
- DLL の解放: LuaJIT と同じく、Lua の状態を閉じるときに `FreeLibrary` する（解放役の userdata をレジストリに置き、`luaopen` を呼ぶ前に作る）。
- 既定の `package.cpath`: `package.path` と同じ探索フォルダ（`lua_search_paths`）から `?.dll` の形で作る（LuaJIT の既定は SSP のカレントと SSP.exe のフォルダを指すので使わない）。テンプレートの中の `!` は、ゴーストの基点のフォルダに置き換える。
- `windows-sys` に `Win32_System_LibraryLoader` を足す。

## Scope

- **In**:
  - `package.loaders[3]`・`[4]`・`package.loadlib` の置き換え（深いパス・非 ANSI のパス・隣の依存 DLL）。
  - 既定の `package.cpath` と、相対パス・`!` の基点。
  - DLL のハンドルの寿命の管理。
  - エラーの文言を LuaJIT と揃えること、LuaJIT の経路との比較テスト（`module_searcher_test.rs` と同じ形）。テスト用の小さな C モジュール DLL（依存 DLL つき）の用意。
  - 文書の更新: `book/src/internals/loader.md`（cpath を変えない、の記述）、`book/src/reference/startup.md`（既知の制限の節）。スキル `pasta-lua-coding` の `references/startup.md` の再生成。
- **Out**:
  - LuaJIT のソースの改造（fork・`[patch.crates-io]`）。
  - `io.open`・`loadfile`・`dofile`・`package.searchpath` の非 ANSI 対応（今の既知の制限のまま。`@enc`・`@pasta_persistence` で回避できる）。
  - `ffi.load`（FFI は `StdLib::ALL_SAFE` に含まれず無効）。
  - ホストが ANSI の `load` しか呼ばないときに失われる文字（pasta.dll に届く前に失われる）。

## Boundary Candidates

- 探索（cpath のテンプレートの展開と、ファイルの存在の確認）— 純粋な Rust で、OS に依らずテストできる。
- 読み込み（`LoadLibraryExW`・`GetProcAddress`・`FreeLibrary`）— Windows 専用の薄い層。
- Lua への登録（loaders の書き換え、`create_c_function`、エラーの文言を揃えるラッパー）— `searcher.rs` の `install_module_searcher` の隣。

## Out of Boundary

- サードパーティの C モジュールの中身。モジュールが内部で `fopen` などの ANSI 版の API を使っていれば、深いパス・非 ANSI のフォルダではそこで失敗する。保証するのは「DLL を見つけて読み込み、`luaopen` を呼ぶ」ところまで。
- C モジュールを配布物にどう同梱するか（`pasta_check release` の扱い）。動機が出たら別に起票する。

## Upstream / Downstream

- **Upstream**: `lua-require-robustness`（完了）の `searcher.rs` と `LoaderContext`。mlua 0.11.6 の `create_c_function`・`mlua::ffi`。LuaJIT の `lib_package.c` の規則（シンボル名・エラーの文言）。
- **Downstream**: C モジュールを使うゴーストと、その作り方の文書（今は無い）。

## Existing Spec Touchpoints

- **Extends**: `lua-require-robustness`（完了。`.kiro/specs/completed/lua-require-robustness/`）。その design.md が範囲外にした「`package.cpath`・`loaders[3]`/`[4]` の変更」を、この spec が引き受ける。
- **Adjacent**: `crates/pasta_lua/src/runtime/module_registry.rs`（`setup_package_path`）、`crates/pasta_lua/src/runtime/searcher.rs`（`EXPECTED_LOADERS_LEN = 4` の前提）、`crates/pasta_lua/src/loader/context.rs`。

## Constraints

- x86 と x64（MSVC・静的 CRT）の両方で動くこと。
- `lua_search_paths` を pasta.toml で上書きしたときも、`package.path` と `package.cpath` が同じフォルダを指すこと。
- LuaJIT の `fopen` を経由しないこと（深いパスで落ちる原因）。
- テストは非 ANSI のフィクスチャ `NON_ANSI_DIR_NAME`（`crates/pasta_lua/tests/common/mod.rs`）と、300 文字を超えるパスで確かめる。
- スキル（`.claude/skills/`）の編集は分類器に拒まれることがあるので、そのときは開発者に頼む。

## 2026-10-18 棚卸の再測定（main facd6cf4）

- **前提の変化**: 起票した日の再測定。「Current State」の場所は main と合う（`runtime/searcher.rs` の `EXPECTED_LOADERS_LEN = 4`（37 行）と `install_module_searcher`（280 行）、`runtime/module_registry.rs` 44 行の `setup_package_path` が cpath を変えないこと、`book/src/internals/loader.md:215`）。LuaJIT の `lib_package.c` の `LJ_WIN_LOADLIBA`（80 行）・`GetModuleFileNameA`（100 行）・`fopen`（299 行）と、`luaconf.h:24-25` の既定の cpath も書いたとおり。0.4.1 までに入った spec は、この範囲に触れていない。
- **触るファイル**: `crates/pasta_lua/src/runtime/searcher.rs`（622 行）、`runtime/module_registry.rs`（193）、`loader/context.rs`（278。既定の cpath を作るなら）、C のローダーの新しいファイル（Windows 専用の層）、`crates/pasta_lua/Cargo.toml`（39 行の `windows-sys` に `Win32_System_LibraryLoader`。`pasta_shiori` と同じく crate の側で足せば、ルートの `Cargo.toml` と `Cargo.lock` は変わらない。機能の名前は lock に載らない）、`.kiro/steering/tech.md:47`、テスト（`tests/runtime/module_searcher_test.rs` の隣）、`book/src/internals/loader.md`（302）・`book/src/reference/startup.md`（192。「3. 既知の制限」は 83 行から。C モジュールの記述は今は無い）、スキルの `references/startup.md` の再生成。1,000 行に近いファイルは無い。
- **規模**: 10〜13 タスク（探索・読み込み・登録・既定の cpath と `!`・寿命・文言の比較・テスト用 DLL・文書）。下の穴の (a) か (b) を範囲に入れると 14〜17。
- **先に要るもの**: 機能の上では無い（`lua-require-robustness` は完了）。ファイルの重なり: 未完了の spec のどれとも、ソースは重ならない（`searcher.rs`・`module_registry.rs`・`loader/context.rs` を挙げる brief は無い）。`talk-grapheme-clusters` とは `crates/pasta_lua/Cargo.toml` の別の行と `tech.md` の依存の表だけ。`Cargo.lock` の席は向こうに渡す（こちらは、テスト用 DLL を新しい crate で作らない限り lock を触らない）。
- **種別**: 機能（C モジュールの `require` は今どこに置いても事実上使えず、新しく読めるようにする）。版は 0.4.2（開発者の決定）。
- **要件定義のモデル**: Opus。下の「Lua の C API をどこから得るか」は外へ出す約束（ABI）になるが、案は 3 つに絞れ、時刻や並行の意味論も絡まない。境目なので Opus とする。
- **分割の案**: 要らない。
- **見つけた穴・古くなった記述**:
  - **C モジュールが Lua の C API（`lua_*`・`luaL_*`）をどこから得るかが書いてない。** mlua は `vendored`（ルートの `Cargo.toml:32`）で LuaJIT を pasta.dll に静的にリンクし、`lua51.dll` は無い。普通の C モジュールは `lua51.dll` を import するので、無ければエラー 126、別の `lua51.dll` を置けば別の VM の関数で pasta の `lua_State` を触ることになり壊れる。表の「隣の依存 DLL」の失敗が同じ原因でないかも確かめる。案: (a) pasta.dll から `lua_*`・`luaL_*` を export し、モジュールは pasta.dll に対して作る、(b) pasta.dll へ転送する `lua51.dll` を用意する、(c) 範囲外とし「DLL を見つけて読み、`luaopen` を呼ぶ」までを保証する。要件定義の最初に決める（(c) なら、この spec の値打ちを開発者に確かめる）。
  - テスト用の C モジュール DLL も同じ問題に当たる（テストの EXE も LuaJIT を export しない）。作り方（`cc` を dev の依存にしてテストの中で `cl` を呼ぶ・テスト用の新しい crate・ビルド済みの DLL を置く）で、`Cargo.lock` と CI の手間が変わる。x86 と x64 の両方が要る。`cc` は LuaJIT のビルド用に、lock に間接で既にある。
  - 「Current State」の `module_searcher_test.rs` は `crates/pasta_lua/tests/runtime/` にある。
- **即時修正**: なし。
