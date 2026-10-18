# Requirements Document

## Project Description (Input)

ゴーストの作者が、C で書かれた Lua の拡張モジュール（DLL）を `require` で読もうとすると、ゴーストのフォルダの場所しだいで読めない。

- ゴーストのフォルダが深い（パスが 260 文字を超える）と読めない。
- フォルダ名に ANSI コードページに無い文字（日本語のロケールでの絵文字や CP932 の外の文字、日本語以外のロケールでの日本語）があると読めない。
- 相対パスで書くと、ゴーストのフォルダではなく SSP のカレントや SSP.exe のフォルダを探してしまう。
- 短い ASCII のパスでも、モジュールの隣に置いた依存 DLL が見つからず失敗する（エラー 126）。

`.lua` の `require` は spec `lua-require-robustness`（完了）で直した。C モジュールの側（`package.loaders[3]`・`[4]`・`package.loadlib`・`package.cpath`）は LuaJIT の実装のままで、ANSI 版の API（`LoadLibraryExA`・`fopen`・`GetModuleFileNameA`）を使っている。

**変えるべきこと**: ゴーストのフォルダが深くても、ANSI に無い文字を含んでも、`require "foo"` が `foo.dll` を見つけて読み込み、`luaopen_foo` を呼べること。`package.cpath` の相対パスはゴーストの `ghost/master` を基点にすること。モジュールの隣の依存 DLL が見つかること。`package.loadlib` も同じ規則で読めること。失敗したときの文言は LuaJIT と同じ形を保つこと。LuaJIT は改造せず、`lua-require-robustness` と同じくホストの側でローダーを差し替える。

出典: `.kiro/specs/lua-cmodule-robustness/brief.md`（2026-10-18 の `/kiro-discovery` で起票）

## Introduction

C で書かれた Lua の拡張モジュール（以下「C モジュール」。`luaopen_<名前>` を持つ DLL）を、ゴーストがどこに置かれていても `require` で読めるようにする。

今は事実上どこに置いても読めない。理由は 4 つ重なっている。

- 深いパス（260 文字を超える）では、ファイルがあるかの確かめで落ちる。
- ANSI コードページに無い文字を含むパスでは、パスの文字が欠ける。
- 既定の `package.cpath` が、SSP のカレントと SSP.exe のフォルダを指している。ゴーストのフォルダを指していない。
- モジュールの隣に置いた依存 DLL を探さない。

これに加えて、C モジュールが Lua の C API（`lua_*`・`luaL_*`）をどこから得るかが決まっていない。pasta.dll は LuaJIT を中に含んでいて、`lua51.dll` は無い。普通の C モジュールは `lua51.dll` を前提に作られる。この spec では、C モジュールが pasta.dll の中の Lua の状態を正しく扱える方法を、作者に示せる形で用意する（下の Requirement 8。方法の選び方は未決。Open Questions の 1）。

方針は `lua-require-robustness` と同じにする。LuaJIT は改造しない。ホストの側で、C モジュールを探して読む部分だけを置き換える。探す順・候補の作り方・失敗の文言は LuaJIT と同じ形に保つ。

## Boundary Context

- **In scope**
  - `require` で C モジュールを探して読む 2 つの経路（モジュール名のとおりのファイルを探す経路と、`a.b` の `a` のファイルから `luaopen_a_b` を探す経路）と、`package.loadlib` の置き換え。深いパス・ANSI に無い文字のパス・隣の依存 DLL で読めること。
  - 既定の `package.cpath` と、相対パス・`!` の基点。
  - 読み込んだ DLL を、Lua の状態を閉じるまで保ち、閉じるときに解放すること。
  - 失敗の文言を LuaJIT と同じ形に保つこと。LuaJIT の経路と並べて比べるテスト。
  - C モジュールが Lua の C API を得る方法（Requirement 8）。
  - テスト用の小さな C モジュール DLL（依存 DLL つき）を、手作業なしで用意すること。
  - 文書の更新: 内部設計の `book/src/internals/loader.md`、利用者向けの `book/src/reference/startup.md`、スキル `pasta-lua-coding` の `references/startup.md` の再生成。
- **Out of scope**
  - LuaJIT のソースの改造（fork・`[patch.crates-io]`）。
  - `io.open`・`loadfile`・`dofile`・`package.searchpath` の非 ANSI 対応・深いパス対応。今の既知の制限のまま。
  - `ffi.load`。FFI は既定で無効（`std_all` に含まれない）。
  - ホストが ANSI の `load` しか呼ばないときに失われる文字。pasta.dll に届く前に失われる。
  - サードパーティの C モジュールの中身。モジュールが中で ANSI 版の API を使えば、深いパス・ANSI に無い文字のフォルダではそこで失敗する。この spec が保証するのは「DLL を見つけて読み込み、`luaopen` を呼ぶ」ところまで。
  - C モジュールを配布物（`.nar`）にどう入れるか（`pasta_check release` の扱い）。
  - Windows 以外の OS での C モジュールの読み込み。今の LuaJIT の動きのまま。
- **Adjacent expectations**
  - `.lua` の `require`（`lua-require-robustness` の searcher・`package.path`・`@pasta_chunks` の表）は変えない。この spec は `package.loaders` の 3 番目と 4 番目と `package.loadlib` だけを置き換え、1 番目（preload）と 2 番目（`.lua`）には触れない。
  - ゴーストの基点のフォルダ（`ghost/master`）は、今の起動の入口（`load`・`loadu`）が渡すパスをそのまま使う。基点を受け取る方法は変えない。
  - 起動の失敗の扱い（`package.loaders` の並びが想定と違えば起動を止める、など）は `lua-require-robustness` と `load-error-logging` の決まりに従う。

## Requirements

### Requirement 1: 深いパス・ANSI に無い文字のパスで C モジュールを読む

**Objective:** ゴーストの作者として、ゴーストのフォルダがどこにあっても C モジュールを `require` で読みたい。置き場所で動いたり動かなかったりするのを避けたい。

#### Acceptance Criteria

1. When `require "foo"` が実行され、`package.cpath` の候補に `foo.dll` がある場合, the pasta_lua ランタイム shall その DLL を読み込み、`luaopen_foo` を呼び、その戻り値を `require` の結果として返す。
2. When C モジュールの絶対パスが 260 文字を超える場合, the pasta_lua ランタイム shall 短いパスのときと同じく、そのモジュールを見つけて読み込む。
3. When C モジュールのパスに ANSI コードページで表せない文字が含まれる場合, the pasta_lua ランタイム shall そのモジュールを見つけて読み込む。
4. When C モジュールのパスが 260 文字を超え、かつ ANSI コードページで表せない文字を含む場合, the pasta_lua ランタイム shall そのモジュールを見つけて読み込む。
5. Where ホストのプロセスや OS で長いパスの設定（`longPathAware`・`LongPathsEnabled`）が有効でない環境である場合, the pasta_lua ランタイム shall 260 文字を超えるパスの C モジュールを読み込む。
6. The pasta_lua ランタイム shall C モジュールのパスの長さに独自の上限を設けない。
7. The pasta_lua ランタイム shall 長いパスのための内部の書き方（`\\?\` の接頭辞など）を、エラーの文言・ログ・`require` の結果に出さない。

### Requirement 2: 探す規則を LuaJIT と同じにする

**Objective:** ゴーストの作者として、Lua の一般の資料に書いてある C モジュールの探し方が、pasta でもそのまま通じてほしい。

#### Acceptance Criteria

1. The pasta_lua ランタイム shall `package.loaders` の順（preload → `.lua` → C モジュール → C モジュールのルート）を変えない。
2. The pasta_lua ランタイム shall `package.cpath` を `;` で区切り、空の要素を飛ばし、各要素の `?` をすべてモジュール名に置き換えた候補を、書いてある順に試す。
3. The pasta_lua ランタイム shall 候補を作るとき、モジュール名の `.` をパスの区切り `\` に置き換える。
4. The pasta_lua ランタイム shall `package.cpath` とモジュール名を UTF-8 として読む。
5. When モジュール名が `-` を含む場合, the pasta_lua ランタイム shall ファイル名にはモジュール名をそのまま使い、呼ぶ関数の名前には最初の `-` までを捨て `.` を `_` にした名前を使う（例: `require "v2-foo.bar"` は `v2-foo\bar.dll` の `luaopen_foo_bar`）。
6. When `require "a.b"` が実行され、`a\b.dll` が見つからず、`a.dll` が見つかった場合, the pasta_lua ランタイム shall `a.dll` から `luaopen_a_b` を探して呼ぶ（C モジュールのルートの経路）。
7. When 同じファイルの C モジュールが 2 回以上読まれた場合（別の名前の `require`・`package.loadlib` を含む）, the pasta_lua ランタイム shall その DLL を 1 回だけ読み込み、同じものを使い回す。
8. If `package.cpath` が文字列でない場合, the pasta_lua ランタイム shall LuaJIT と同じ文言（`'package.cpath' must be a string`）の Lua の文字列のエラーで `require` を失敗させる。
9. If `package.cpath` に UTF-8 として正しくないバイト列が含まれる場合, the pasta_lua ランタイム shall その候補を開けなかったものとして扱い、`no file` の行に載せて次の候補へ進む。
10. Where DLL が `luaopen_<名前>` を持たず、LuaJIT のバイトコードの入口（`luaJIT_BC_<名前>`）を持つ場合, the pasta_lua ランタイム shall LuaJIT と同じく、そのバイトコードを読み込んでモジュールにする。
    > **仮置き（Open Questions の 3）**: LuaJIT の `ll_loadfunc` が持つ規則なので、同じ形を保つ方針に合わせて範囲に入れた。brief には書かれていない。

### Requirement 3: 既定の `package.cpath` と相対パスの基点

**Objective:** ゴーストの作者として、C モジュールを `.lua` と同じフォルダに置けば、何も設定せずに `require` で読めてほしい。SSP がどこから起動されたかに左右されたくない。

#### Acceptance Criteria

1. The pasta_lua ランタイム shall 既定の `package.cpath` を、`package.path` と同じ探索のフォルダ（`[loader] lua_search_paths`）から、同じ順で、フォルダごとに `?.dll` の形で作る。
2. When `pasta.toml` で `lua_search_paths` が変えられた場合, the pasta_lua ランタイム shall `package.path` と `package.cpath` が同じフォルダを同じ順で指すようにする。
3. The pasta_lua ランタイム shall 既定の `package.cpath` に、SSP のカレントのフォルダと SSP.exe のフォルダを含めない。
4. The pasta_lua ランタイム shall 既定の `package.cpath` を作るとき、環境変数 `LUA_CPATH` を使わない（`package.path` が `LUA_PATH` を使わないのと同じ）。
5. When `package.cpath` の要素が相対パスである場合, the pasta_lua ランタイム shall それをゴーストの基点のフォルダ（`ghost/master`）を起点にして解決する。SSP のカレントのフォルダや SSP.exe のフォルダは起点にしない。
6. When `package.cpath` の要素が `!` を含む場合, the pasta_lua ランタイム shall `!` をゴーストの基点のフォルダに置き換える。
   > **仮置き（Open Questions の 2）**: brief の Approach のとおりにした。ただし LuaJIT は `!` を起動のときに既定の値の中でだけ置き換え、後から書いた `package.cpath` では置き換えない。また、ゴーストのフォルダ名そのものに `!` が含まれると、探すたびに置き換える方式では壊れる。置き換える時と範囲は要件定義の話し合いで決める。
7. When ゴーストの作者が実行中に `package.cpath` を書き換えた場合, the pasta_lua ランタイム shall 次の `require` から、書き換えた値で探す。
8. When ゴーストの基点のフォルダのパスに `!` が含まれる場合, the pasta_lua ランタイム shall 既定の `package.cpath` のままで、C モジュールを見つけて読み込む（6. の置き換えで基点のパスを壊さない）。

### Requirement 4: 隣の依存 DLL を見つける

**Objective:** ゴーストの作者として、ほかの DLL に頼る C モジュールを、その DLL といっしょに同じフォルダに置くだけで動かしたい。

#### Acceptance Criteria

1. When C モジュールが、同じフォルダに置いた別の DLL に頼っている場合, the pasta_lua ランタイム shall その依存 DLL を見つけて、C モジュールを読み込む。
2. When C モジュールが Windows のシステムの DLL（`System32` の DLL）に頼っている場合, the pasta_lua ランタイム shall その DLL を見つけて、C モジュールを読み込む。
3. The pasta_lua ランタイム shall 依存 DLL を探す場所に、ホストのカレントのフォルダと環境変数 `PATH` のフォルダを含めない。
   > **仮置き（Open Questions の 4）**: brief の計測で成功した読み方（モジュールのフォルダ＋既定のフォルダ）に合わせた。LuaJIT の今の読み方は `PATH` とカレントも探すので、`PATH` にある DLL に頼るモジュールはこの変更で読めなくなる。その代わり、ほかの場所の同名の DLL を誤って読む心配が減る。
4. When 1 つの依存 DLL が、C モジュールのフォルダにも別の探索先にもある場合, the pasta_lua ランタイム shall C モジュールのフォルダにあるものを使う。

### Requirement 5: `package.loadlib` も同じ規則で読む

**Objective:** ゴーストの作者として、`package.loadlib` で DLL の関数を直接読むときも、`require` と同じ場所・同じ条件で読めてほしい。

#### Acceptance Criteria

1. When `package.loadlib(パス, 関数名)` が呼ばれた場合, the pasta_lua ランタイム shall そのパスの DLL を読み込み、その名前の関数を Lua の関数として返す。
2. The pasta_lua ランタイム shall `package.loadlib` でも、深いパス・ANSI に無い文字のパス・隣の依存 DLL について、Requirement 1 と Requirement 4 と同じ結果を返す。
3. When `package.loadlib` に相対パスが渡された場合, the pasta_lua ランタイム shall それをゴーストの基点のフォルダを起点にして解決する。
4. When `package.loadlib` の関数名に `*` が渡された場合, the pasta_lua ランタイム shall DLL を読み込むだけにして、`true` を返す。
5. If DLL を読み込めなかった場合, the pasta_lua ランタイム shall LuaJIT と同じく、`nil`・理由の文言・`"open"` の 3 つを返す。
6. If DLL は読み込めたが、その名前の関数が無かった場合, the pasta_lua ランタイム shall LuaJIT と同じく、`nil`・理由の文言・`"init"` の 3 つを返す。

### Requirement 6: 失敗の文言を LuaJIT と同じ形にする

**Objective:** ゴーストの作者として、C モジュールを読めなかったとき、どの候補を探し、なぜ失敗したかを、Lua の一般の資料と同じ読み方で知りたい。

#### Acceptance Criteria

1. When どの候補にも C モジュールが無かった場合, the pasta_lua ランタイム shall 候補ごとに `\n\tno file '<候補>'` の行を、試した順に、`require` の `module '<名前>' not found:` のエラーに載せる。
2. When C モジュールのルートの経路で、ルートのファイルは見つかったがその関数が無かった場合, the pasta_lua ランタイム shall `\n\tno module '<名前>' in file '<ファイル>'` の行を `require` のエラーに載せる。
3. If 候補のファイルはあるが読み込めなかった場合（依存 DLL が無い・DLL として正しくない・`luaopen_<名前>` が無い、など）, the pasta_lua ランタイム shall `error loading module '<名前>' from file '<ファイル>':\n\t<理由>` の文言の Lua の文字列のエラーで `require` を失敗させる。
4. The pasta_lua ランタイム shall 失敗の理由の文言に、Windows が返す理由（例: 指定されたモジュールが見つからない）を、文字化けしない形で入れる。
   > **仮置き**: LuaJIT は Windows の理由を ANSI のまま入れるので、UTF-8 の画面では日本語の理由が化ける。ここは LuaJIT と同じにせず、読める形にする。比べるテストでは、理由の部分は文字コードをそろえてから比べる。
5. The pasta_lua ランタイム shall 失敗の文言とエラーの型（Lua の文字列）を、`pcall(require, …)` で受け取ったときに LuaJIT と同じにする。理由の部分の文字コード（6.4）だけを除く。
6. The pasta_lua ランタイム shall 失敗の文言の中のパスを、ANSI に無い文字も欠けず置き換わらない形で出す。

### Requirement 7: 読み込んだ DLL の寿命

**Objective:** ゴーストの利用者として、C モジュールを使うゴーストを再読み込みしたり終了したりしても、SSP が落ちないでほしい。

#### Acceptance Criteria

1. While Lua の状態が生きている間, the pasta_lua ランタイム shall 読み込んだ C モジュールの DLL を解放しない。
2. When Lua の状態が閉じられた場合（ゴーストの終了・再読み込み）, the pasta_lua ランタイム shall 読み込んだ DLL を解放する。
3. When Lua の状態が閉じられる場合, the pasta_lua ランタイム shall C モジュールが作った値の後始末（`__gc`）を、その DLL を解放するより前に済ませる。
4. When ゴーストが再読み込みされた場合, the pasta_lua ランタイム shall 新しい Lua の状態で、C モジュールを改めて読み込み、`luaopen_<名前>` を改めて呼ぶ。
5. If `luaopen_<名前>` がエラーを投げた場合, the pasta_lua ランタイム shall そのエラーを `require` の呼び出し元へ返し、DLL はほかの読み込み済みの DLL と同じく、Lua の状態を閉じるときに解放する。

### Requirement 8: C モジュールが Lua の C API を得る方法

**Objective:** C モジュールを作る人として、pasta.dll の中の Lua の状態を正しく扱える C モジュールを、示された手順で作りたい。別の Lua の実装を誤ってつないで、ゴーストを壊したくない。

> **仮置き（Open Questions の 1）**: 推奨案 (a)「pasta.dll から Lua の C API を公開し、C モジュールは pasta.dll に対して作る」で書いた。(b)「pasta.dll へつなぎ直す `lua51.dll` を添える」・(c)「範囲外にする」は Open Questions に並べた。今の pasta.dll は Lua の C API を 1 つも公開していない（2026-10-18 に x86・x64 の pasta.dll で確かめた）。

#### Acceptance Criteria

1. The pasta_shiori shall LuaJIT 2.1 の公開の C API（`lua.h`・`lauxlib.h`・`lualib.h`・`luajit.h` に宣言され、LuaJIT が中身を持つ関数）を、pasta.dll の外から呼べる関数として公開する。
2. The pasta_shiori shall 公開した関数が、その pasta.dll の中で動いている Lua の状態を扱うようにする（別の Lua の実装の関数ではない）。
3. The pasta_shiori shall x86 と x64 の両方の pasta.dll で、同じ名前（飾りの無い C の名前）で関数を公開する。
4. The pasta リポジトリ shall C モジュールを pasta.dll に対して作るのに要るもの（公開する関数の一覧と、それから取り込み用のライブラリを作る手順）を、C モジュールを作る人が手に入れられる形で置く。
   > **仮置き（Open Questions の 6）**: 取り込み用のライブラリそのものをリリースの配布物に入れるかは決めていない。
5. When pasta.dll に対して作った C モジュールが `require` で読まれた場合, the pasta_lua ランタイム shall その C モジュールが、渡された Lua の状態で値を作り、関数を登録し、Lua のエラーを投げられるようにする。
6. If C モジュールが `lua51.dll` に対して作られていて、`lua51.dll` が見つからない場合, the pasta_lua ランタイム shall Requirement 6.3 の文言で `require` を失敗させる（SSP は落ちない）。
7. While 同じ SSP の中で pasta を使うほかのゴーストが動いている間, when C モジュールが `require` で読まれた場合, the pasta_lua ランタイム shall その C モジュールが、読んだゴーストの pasta.dll の Lua の C API を使うようにする（先に読み込まれた別のゴーストの pasta.dll を使わない）。
   > **仮置き（Open Questions の 1）**: Windows は、DLL が頼る DLL を、同じ名前で既に読み込まれているものに結びつける（Microsoft の DLL の探し方の文書による。この件では未計測）。何もしなければ、pasta のゴーストを 2 体動かしたとき、後のゴーストの C モジュールは必ず先のゴーストの pasta.dll（LuaJIT の別の写し）に結びつく。版が違えば壊れる。ここでは「自分の pasta.dll につなぎ直す」完全版で書いた。縮めた案は「別の pasta.dll に結びついたら `luaopen` を呼ばずに、分かる文言で `require` を失敗させる」（その場合、C モジュールを使えるのは先に読み込まれたゴーストだけになる）。
8. The pasta_shiori shall 公開する関数を増やしても、今の SHIORI の入口（`load`・`loadu`・`request`・`unload`）の名前と動きを変えない。

### Requirement 9: 変えないこと

**Objective:** ゴーストの作者として、C モジュールを使わないゴーストは、この変更の前と同じに動いてほしい。

#### Acceptance Criteria

1. The pasta_lua ランタイム shall `.lua` の `require` の結果（探す順・チャンクの名前・失敗の文言・`@pasta_chunks` の表）を変えない。
2. The pasta_lua ランタイム shall `package.path` の値を変えない。
3. While C モジュールを使わないゴーストが動いている間, the pasta_shiori shall この変更の前とバイト単位で同じ SHIORI の応答を返す。
4. If `package.loaders` の並びが想定（4 つ）と違う場合, the pasta_lua ランタイム shall どの要素も置き換えず、起動を失敗として扱う（今の動きを保つ）。
5. The pasta_lua ランタイム shall `require` が `package.loaders` と `package.searchers` のどちらの名前から見ても、置き換えた後の同じ表を見るようにする。
6. The pasta_lua ランタイム shall この spec の C モジュールの読み込みについて、x86 と x64 のどちらのビルドでも同じ結果を返す。

### Requirement 10: 確かめるテスト

**Objective:** 保守する人として、深いパス・ANSI に無い文字のパス・依存 DLL という再現しにくい条件が、いつのまにか壊れていないことを毎回確かめたい。

#### Acceptance Criteria

1. The 自動テスト shall 300 文字を超えるパスに置いた C モジュールを `require` で読めることを確かめる。
2. The 自動テスト shall ANSI に無い文字のフォルダ（`tests/common` の `NON_ANSI_DIR_NAME`）に置いた C モジュールを `require` で読めることを確かめる。
3. The 自動テスト shall 隣に置いた依存 DLL に頼る C モジュールを `require` で読めることを確かめる。依存 DLL が見つからない失敗と、Lua の C API が見つからない失敗を、別々のモジュールで分けて確かめる。
4. The 自動テスト shall 短い ASCII のパスで、LuaJIT の今の経路と置き換えた経路を並べ、見つからない・読み込めない・関数が無い・ルートの経路の各場合の、`pcall(require, …)` の結果の型と文言を比べる（`.lua` の searcher の比べるテストと同じ形）。
5. The 自動テスト shall `package.loadlib` の成功・`"open"`・`"init"`・`*` の各場合を確かめる。
6. The 自動テスト shall 本物の pasta.dll を読み込み、pasta.dll に対して作った C モジュールが、その中の Lua の状態で値を返し、関数を登録し、エラーを投げられることを確かめる（Requirement 8）。別のフォルダに置いた pasta.dll の写しを先に読み込んだ状態でも、C モジュールが自分の pasta.dll を使うことを確かめる（8.7）。
7. The 自動テスト shall Lua の状態を閉じたあとに DLL が解放されること、`__gc` を持つ値を残したまま閉じても落ちないこと、閉じてから同じモジュールを読み直せることを確かめる。
8. The 自動テスト shall テスト用の C モジュール DLL を、人の手を介さずに用意する（手で作ったバイナリをテストの前に置く手順を要さない）。
   > **仮置き（Open Questions の 5）**: 用意の方法（テストの中でコンパイラを呼ぶ・テスト用の crate・作ってあるバイナリをコミットする）は要件定義の話し合いで決める。
9. The 自動テスト shall x86 と x64 の両方のターゲットで、上の各テストを CI で毎回走らせる（`#[ignore]` にしない）。
10. The 自動テスト shall `PASTA_DEBUG` の環境変数があってもなくても同じ結果を返す。

### Requirement 11: 文書

**Objective:** ゴーストの作者と C モジュールを作る人として、C モジュールをどこに置けば読めるか、何が保証され何が保証されないかを知りたい。

#### Acceptance Criteria

1. The 利用者向けマニュアル（`book/src/reference/startup.md`） shall C モジュールを探すフォルダと順、既定の `package.cpath`、相対パスの基点を示す。
2. The 利用者向けマニュアル shall 保証の範囲（DLL を見つけて読み込み、`luaopen` を呼ぶところまで）と、保証しないこと（C モジュールの中で ANSI 版の API を使った場合の失敗・`PATH` にある依存 DLL）を、既知の制限の節に書く。
3. The 利用者向けマニュアル shall C モジュールを pasta 向けに作る方法（Requirement 8 で決めた方法）と、作るときの注意（C の実行時ライブラリをまたいでメモリや `FILE*` を渡さないこと、など）を書く。
4. The 内部設計の文書（`book/src/internals/loader.md`） shall 「`package.cpath` は変えない」の記述を改め、C モジュールの経路の置き換えと、LuaJIT と違う点を書く。
5. The スキル `pasta-lua-coding` の `references/startup.md` shall マニュアルから再生成され、鮮度の照合（`gen-skill-refs.mjs --check`）を通る。

## Open Questions

要件定義の話し合いで決める。各項に推奨案を付けた。調べた事実は `research.md` にある。

1. **C モジュールが Lua の C API をどこから得るか**（Requirement 8）
   - **推奨: (a) pasta.dll から公開する ＋ C モジュールを読んだゴーストの pasta.dll につなぎ直す（8.7）。**
   - (a) pasta.dll から `lua_*`・`luaL_*`・`luaopen_*`・`luaJIT_*` を公開する（約 150 個）。C モジュールは pasta.dll に対して作る。理由: pasta.dll の中の LuaJIT と確実に同じものを使える。リンカーへの指示だけで足り、LuaJIT は改造しない。
   - (b) (a) に加えて、pasta.dll へつなぎ直す `lua51.dll` を添える。利点: 普通の `lua51.lib` で作ったモジュールがそのまま動く。欠点: (a) が前提で、x86・x64 の DLL を 2 つ配る手間が増える。`lua51.dll` という名前がほかと重なるおそれもある。
   - (c) 範囲外にし、「DLL を見つけて読み、`luaopen` を呼ぶ」までを保証する。欠点: 今ある C モジュールはどれも Lua の C API を使うので、この spec だけでは実際には何も読めない。「完成度が高くないなら仕様ごと見送る」の方針に照らすと、(c) を選ぶならこの spec そのものの値打ちを開発者に確かめる必要がある。
   - (a)・(b) に共通の危険: Windows は、頼る DLL を、同じ名前で既に読み込まれたものに結びつける。同じ SSP で pasta のゴーストを 2 体動かすと、後のゴーストの C モジュールは必ず先のゴーストの pasta.dll（LuaJIT の別の写し）に結びつく。版が違えば壊れる。版が同じでも、先のゴーストが閉じたあとの扱いなどが保証できない。
     - **推奨: 自分の pasta.dll につなぎ直す（8.7 の完全版）。** 読み込んだあと、`luaopen` を呼ぶ前に、C モジュールが pasta.dll から取り込んだ関数の行き先を、読んだゴーストの pasta.dll に書き換える。ゴーストが何体いても、それぞれ自分の LuaJIT を使える。方法と安全さは設計で実測する。
     - 縮めた案: 行き先を確かめるだけにし、別の pasta.dll なら `luaopen` を呼ばずに失敗させる。作りは小さいが、C モジュールを使えるのは先に読み込まれたゴーストだけになる。
   - 規模: brief の見積もりでは、(a) か (b) を入れると 10〜13 タスクが 14〜17 タスクになる。0.4.2 に入れたままでよいかも確かめる。
2. **`!` をいつ・どこで置き換えるか**（Requirement 3.6・3.8）
   - **推奨: `!` は置き換えない（ただの文字として扱う）。3.6 を削り、3.8 は残す。**
   - LuaJIT は `!` を、起動のときに既定の値と環境変数 `LUA_CPATH` の中でだけ置き換える。後から書いた `package.cpath` では置き換えない。pasta の既定の値は `!` を使わず、環境変数も使わない（3.4）ので、置き換える場面が無い。探すたびに置き換えると、フォルダ名に `!` を含むゴーストで既定の値が壊れる。
   - brief のとおり置き換える場合は、作者が書いた部分だけを置き換え、基点のパスを壊さない仕組みが要る（3.8）。
3. **LuaJIT のバイトコードの入口（`luaJIT_BC_<名前>`）を扱うか**（Requirement 2.10）
   - **推奨: 扱う。** LuaJIT と同じ形を保つ方針に合うし、`luaopen` が無いときに 1 回多く探すだけで済む。扱わない場合は、マニュアルに LuaJIT との違いとして書く。
4. **依存 DLL を探す場所から `PATH` とカレントを外してよいか**（Requirement 4.3）
   - **推奨: 外す（brief の計測で成功した読み方のまま）。** モジュールのフォルダ・SSP.exe のフォルダ・`System32` を探す。ほかの場所の同名の DLL を誤って読む心配が減る。
   - 別の案: 今の読み方の「`PATH` とカレントも探す」を保つ読み方（モジュールのフォルダを先頭にして、そのあと標準の順で探す）もある。`PATH` に置いた DLL に頼るモジュールは動き続けるが、SSP のカレントの DLL を拾うおそれが残る。
5. **テスト用の C モジュール DLL をどう用意するか**（Requirement 10.8）
   - **推奨: `cc` を `pasta_lua` の dev の依存に足し、テストの中で `cc` が見つける `cl.exe`（と `lib.exe`）を呼んで、その場で DLL を作る。** `cc` 1.6.0 は lock に既にある（LuaJIT のビルド用）ので、新しい package は増えず、lock の `pasta_lua` の依存の行が 1 つ増えるだけ。CI の windows-latest には Visual Studio があり、x86・x64 の両方のジョブでそのターゲットの DLL ができる。
   - 別の案: テスト用の新しい crate（lock とファイルが増える）、作ってあるバイナリをコミットする（x86・x64 の 2 組。中身の確かめと作り直しの手順が要る）。
   - 注意: brief では `Cargo.lock` の席を `talk-grapheme-clusters` に渡している。推奨案は lock の 1 行を変えるので、どちらが先にマージするかを決める。
6. **取り込み用のライブラリをどう渡すか**（Requirement 8.4）
   - **推奨: 公開する関数の一覧（`.def` の形）をリポジトリに置き、それから取り込み用のライブラリを作る手順をマニュアルに書く。** 一覧は pasta.dll のビルドにも使い、1 つの正にする。
   - 別の案: x86・x64 の取り込み用のライブラリをリリースの `pasta.dll.zip` に入れる。作る人の手間は減るが、リリース CI の中身の検査（`pasta.dll` と `THIRD_PARTY_LICENSES.txt` の 2 つだけ）を変えることになる。
7. **失敗の理由の文字コード**（Requirement 6.4）
   - **推奨: Windows が返す理由は UTF-8 で入れる（LuaJIT とわざと違える）。** LuaJIT は ANSI のまま入れるので、pasta の UTF-8 の世界では日本語の理由が化ける。文言の形（前後の決まった部分）は LuaJIT と同じに保つ。
8. **Windows 以外の OS**（Boundary Context）
   - **推奨: 置き換えず、LuaJIT のままにする。** pasta.dll は Windows 専用で、C モジュールを読む場面は Windows にしかない。
