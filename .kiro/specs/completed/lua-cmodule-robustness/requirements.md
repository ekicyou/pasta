# Requirements Document

## Project Description (Input)

ゴーストの作者が、自分で作った DLL（や、ほかの人が作った C の DLL）の関数をゴーストから呼ぼうとすると、ゴーストのフォルダの場所しだいで読めない。そもそも今の pasta.dll では、DLL を呼ぶ手段が事実上無い。

- C モジュールを `require` で読む道は、Lua の C API が pasta.dll から公開されておらず、`lua51.dll` も無いので、どこに置いても読めない。
- LuaJIT の FFI は既定で無効で、pasta.toml から有効にする手段も無い。
- 仮に FFI を有効にしても、`ffi.load` は ANSI 版の `LoadLibraryExA` を flags 0 で呼ぶので、深いパス・ANSI に無い文字のフォルダ・隣の依存 DLL で失敗し、名前だけや相対パスではゴーストのフォルダを探さない。

**変えるべきこと**: pasta.toml の `[loader] ffi = true` で FFI を有効にできること（既定は無効）。FFI を有効にしたゴーストでは、ゴーストのフォルダが深くても、ANSI に無い文字を含んでも、`ffi.load` がゴーストのフォルダの DLL を読めること。名前だけの指定はゴーストの探索のフォルダから探し、相対パスはゴーストの `ghost/master` を基点にすること。隣の依存 DLL が見つかること。別のゴーストの同じ名前の DLL に黙って結びつかないこと。失敗したときの文言は LuaJIT と同じ形を保つこと。LuaJIT は改造せず、`lua-require-robustness` と同じくホストの側で `ffi.load` を包む。

出典: `.kiro/specs/lua-cmodule-robustness/brief.md`（2026-10-18 の `/kiro-discovery` で起票。同じ日の要件の話し合いで、C モジュールの道から FFI の道へ方針を替えた）

## Introduction

ゴーストの作者が、ネイティブの DLL の関数を LuaJIT の FFI（`ffi.cdef` で C の宣言を書き、`ffi.load` で DLL を読んで呼ぶ仕組み）で呼べるようにする。

```c
// foo.c — cl /LD foo.c だけで作れる。Lua のヘッダーもライブラリも要らない
__declspec(dllexport) int add(int a, int b) { return a + b; }
```

```toml
# pasta.toml
[loader]
ffi = true
```

```lua
local ffi = require "ffi"
ffi.cdef [[ int add(int a, int b); ]]
local foo = ffi.load("foo")   -- ghost/master の探索のフォルダから foo.dll を探す
print(foo.add(1, 2))          --> 3
```

今はこれが動かない。理由は 2 つある。

- FFI が既定で無効で、pasta.toml から有効にできない。
- LuaJIT の `ffi.load` は、深いパス・ANSI に無い文字のフォルダでは DLL を読めず、DLL の隣の依存 DLL も探さない。名前だけや相対パスで書くと、ゴーストのフォルダではなく SSP の側を探す。

方針は `lua-require-robustness` と同じにする。LuaJIT は改造しない。ホストの側で `ffi.load` を包み、DLL を先に読んでから元の `ffi.load` に渡す。失敗の文言は LuaJIT と同じ形に保つ。

## Boundary Context

- **In scope**
  - `[loader] ffi` の設定と、`true` のときに FFI を開くこと。
  - `ffi.load` の置き換え。深いパス・ANSI に無い文字のフォルダ・隣の依存 DLL・名前だけの探索・相対パスの基点・取り違えの防止。
  - 先に読んだ DLL を、Lua の状態を閉じ終わるまで保ち、そのあと解放すること。
  - 失敗の文言を LuaJIT と同じ形に保つこと。元の `ffi.load` と並べて比べるテスト。
  - テスト用の小さな DLL（依存 DLL つき）を、手作業なしで用意すること。
  - 文書の更新: `book/src/reference/pasta-toml.md`、`book/src/reference/startup.md`、`book/src/internals/loader.md`、スキル `pasta-lua-coding` の `references/` の再生成。
- **Out of scope**
  - LuaJIT のソースの改造（fork・`[patch.crates-io]`）。
  - C モジュールを `require` で読む道（`package.loaders[3]`・`[4]`・`package.loadlib`・`package.cpath`）。LuaJIT のままにする。Lua の C API の公開もしない。
  - `io.open`・`loadfile`・`dofile`・`package.searchpath` の非 ANSI 対応・深いパス対応。今の既知の制限のまま。
  - ホストが ANSI の `load` しか呼ばないときに失われる文字。pasta.dll に届く前に失われる。
  - サードパーティの DLL の中身。DLL が中で ANSI 版の API を使えば、深いパス・ANSI に無い文字のフォルダではそこで失敗する。この spec が保証するのは「DLL を見つけて読み込み、`ffi.load` の値として返す」ところまで。
  - DLL を配布物（`.nar`）にどう入れるか（`pasta_check release` の扱い）。
  - FFI 以外のライブラリの組み合わせを pasta.toml で選べるようにすること（`[lua] libs` の復活）。
  - Windows 以外の OS での `ffi.load`。今の LuaJIT の動きのまま。
- **Adjacent expectations**
  - `.lua` の `require`（`lua-require-robustness` の searcher・`package.path`・`@pasta_chunks` の表）は変えない。
  - ゴーストの基点のフォルダ（`ghost/master`）は、今の起動の入口（`load`・`loadu`）が渡すパスをそのまま使う。
  - pasta.toml の読み込みの失敗の扱いは、今の `[loader]` のほかのキーと同じ決まり（`load-error-logging` など）に従う。

## Requirements

### Requirement 1: FFI を pasta.toml で有効にする

**Objective:** ゴーストの作者として、DLL を呼ぶゴーストだけが、明示して FFI を使えるようにしたい。DLL を呼ばないゴーストには、任意のメモリに触れる道具を開きたくない。

#### Acceptance Criteria

1. The pasta_lua ランタイム shall pasta.toml の `[loader]` の節に、真偽値のキー `ffi` を受け付ける。既定は `false` とする。
2. When `[loader] ffi` が `true` の場合, the pasta_lua ランタイム shall LuaJIT の FFI のライブラリを開き、`require "ffi"` とグローバルの `ffi` から同じ表を使えるようにする。
3. While `[loader] ffi` が `false` であるか書かれていない間, the pasta_lua ランタイム shall FFI のライブラリを開かず、`require "ffi"` とグローバルの `ffi` をこの変更の前と同じ結果にする。
4. If `[loader] ffi` に真偽値でない値が書かれた場合, the pasta_lua ランタイム shall `[loader]` のほかのキーの型の誤りと同じく、pasta.toml の読み込みの失敗として扱う。
5. The pasta_lua ランタイム shall `[loader] ffi` を `@pasta_config` に載せない（`[loader]` のほかのキーと同じ）。
6. When `[loader] ffi` が `true` の場合, the pasta_lua ランタイム shall `ffi.cdef`・`ffi.new`・`ffi.C` など、`ffi.load` 以外の FFI の機能を LuaJIT のまま使えるようにする。

### Requirement 2: 深いパス・ANSI に無い文字のフォルダの DLL を読む

**Objective:** ゴーストの作者として、ゴーストのフォルダがどこにあっても DLL を読みたい。置き場所で動いたり動かなかったりするのを避けたい。

#### Acceptance Criteria

1. When `ffi.load` に DLL のパスが渡され、そのファイルがある場合, the pasta_lua ランタイム shall その DLL を読み込み、元の `ffi.load` と同じ種類の値（`ffi.cdef` で宣言した関数をその DLL から引ける値）を返す。
2. When DLL の絶対パスが 260 文字を超える場合, the pasta_lua ランタイム shall 短いパスのときと同じく、その DLL を読み込む。
3. When DLL のフォルダのパスに ANSI コードページで表せない文字が含まれる場合, the pasta_lua ランタイム shall その DLL を読み込む。
4. When DLL の絶対パスが 260 文字を超え、かつフォルダのパスに ANSI コードページで表せない文字を含む場合, the pasta_lua ランタイム shall その DLL を読み込む。
5. Where ホストのプロセスや OS で長いパスの設定（`longPathAware`・`LongPathsEnabled`）が有効でない環境である場合, the pasta_lua ランタイム shall 260 文字を超えるパスの DLL を読み込む。
   > **確かめ方は設計で決める**: 2026-10-18 の実測は `LongPathsEnabled=1` の機械で、マニフェストの無い EXE で行った。`LongPathsEnabled=0` の機械では未計測（brief の「残る制限」）。
6. The pasta_lua ランタイム shall DLL のパスの長さに独自の上限を設けない。
7. The pasta_lua ランタイム shall 長いパスのための内部の書き方（`\\?\` の接頭辞など）を、エラーの文言・ログ・`ffi.load` の結果に出さない。
8. If DLL のファイル名（フォルダを除いた部分）に ANSI コードページで表せない文字が含まれる場合, the pasta_lua ランタイム shall その DLL を読み込まず、ファイル名に使えない文字があることが分かる文言で `ffi.load` を失敗させる。
9. The pasta_lua ランタイム shall `ffi.load` に渡されたパスの区切りとして、`/` と `\` のどちらも受け付ける。

### Requirement 3: 名前の解決と相対パスの基点

**Objective:** ゴーストの作者として、DLL を `.lua` と同じフォルダに置けば、名前だけで読めてほしい。SSP がどこから起動されたかに左右されたくない。

#### Acceptance Criteria

1. When `ffi.load` に渡された名前が `/`・`\`・`.` のどれも含まない場合, the pasta_lua ランタイム shall LuaJIT と同じく、末尾に `.dll` を足したファイル名で探す。
2. When `ffi.load` に渡された名前がパスの区切り（`/`・`\`）を含まない場合, the pasta_lua ランタイム shall `package.path` と同じ探索のフォルダ（`[loader] lua_search_paths`）を同じ順で探し、最初に見つかったファイルを読む。
3. When `pasta.toml` で `lua_search_paths` が変えられた場合, the pasta_lua ランタイム shall `package.path` と、名前だけの `ffi.load` の探索が、同じフォルダを同じ順で指すようにする。
4. When 名前だけの `ffi.load` で、探索のフォルダのどこにもそのファイルが無い場合, the pasta_lua ランタイム shall その名前を LuaJIT と同じ OS の既定の探索で読み、`ffi.load("user32")` のように OS の DLL を今までどおり読めるようにする（失敗の理由は Requirement 6 の UTF-8 で返す）。
5. When `ffi.load` に渡された名前が相対パス（パスの区切りを含み、ドライブ文字や `\\` で始まらないもの）である場合, the pasta_lua ランタイム shall それをゴーストの基点のフォルダ（`ghost/master`）を起点にして解決する。SSP のカレントのフォルダや SSP.exe のフォルダは起点にしない。
6. When `ffi.load` に渡された名前が絶対パスである場合, the pasta_lua ランタイム shall そのパスをそのまま使う。
7. If パスの区切りを含む名前（相対・絶対）のファイルが無い場合, the pasta_lua ランタイム shall 元の `ffi.load` へ渡さず、Requirement 6 の文言で `ffi.load` を失敗させる。
8. The pasta_lua ランタイム shall `ffi.load` の 2 番目の引数（`global`）を受け付け、Windows の LuaJIT と同じく結果に影響させない。

### Requirement 4: 隣の依存 DLL を見つける

**Objective:** ゴーストの作者として、ほかの DLL に頼る DLL を、その DLL といっしょに同じフォルダに置くだけで動かしたい。

#### Acceptance Criteria

1. When ゴーストのフォルダから読む DLL が、同じフォルダに置いた別の DLL に頼っている場合, the pasta_lua ランタイム shall その依存 DLL を見つけて、DLL を読み込む。
2. When ゴーストのフォルダから読む DLL が Windows のシステムの DLL（`System32` の DLL）に頼っている場合, the pasta_lua ランタイム shall その DLL を見つけて、DLL を読み込む。
3. The pasta_lua ランタイム shall ゴーストのフォルダから読む DLL の依存 DLL を探す場所に、ホストのカレントのフォルダと環境変数 `PATH` のフォルダを含めない。
4. When 1 つの依存 DLL が、DLL のフォルダにも別の探索先にもある場合, the pasta_lua ランタイム shall DLL のフォルダにあるものを使う。
5. If 依存 DLL が見つからない場合, the pasta_lua ランタイム shall Requirement 6 の文言で `ffi.load` を失敗させる（SSP は落ちない）。

### Requirement 5: 別のゴーストの DLL と取り違えない

**Objective:** ゴーストの作者として、同じ SSP でほかのゴーストが同じファイル名の DLL を使っていても、自分のゴーストの DLL を呼びたい。黙って別の DLL の関数を呼んで、ゴーストを壊したくない。

#### Acceptance Criteria

1. While 同じプロセスの中に、同じファイル名で別のファイルの DLL が既に読み込まれている間, when `ffi.load` がゴーストのフォルダの DLL を読む場合, the pasta_lua ランタイム shall 既に読み込まれている別のファイルの DLL を、`ffi.load` の結果として返さない。
2. When 1. の場合に自分のゴーストの DLL を読めない場合, the pasta_lua ランタイム shall 同じファイル名の別の DLL が既に読み込まれていることが分かる文言で `ffi.load` を失敗させる。
   > **決め方は設計で詰める**: 実測で、取り違えは事前に `GetModuleHandleW` で気づけることを確かめた。パスが ANSI で表せるときはフルパスを元の `ffi.load` に渡して取り違えを避け、表せないときだけ 2. で失敗させる、などの方法がある。どの場合に失敗させるかは設計で決め、マニュアルに書く。
3. When 同じゴーストが、同じファイルの DLL を 2 回以上 `ffi.load` で読んだ場合, the pasta_lua ランタイム shall 2 回目以降も成功させ、同じ DLL を使う。
4. When ゴーストが再読み込みされ、前の Lua の状態で読んだ同じファイルの DLL を読む場合, the pasta_lua ランタイム shall それを別のファイルの DLL とは扱わず、読み込みを成功させる。

### Requirement 6: 失敗の文言を LuaJIT と同じ形にする

**Objective:** ゴーストの作者として、DLL を読めなかったとき、どの名前で何が原因かを、LuaJIT の資料と同じ読み方で知りたい。

#### Acceptance Criteria

1. If DLL を読み込めなかった場合, the pasta_lua ランタイム shall LuaJIT と同じく、呼び出し元の位置に続けて `cannot load module '<名前>': <理由>` の文言の Lua のエラーを投げる。`<名前>` は作者が `ffi.load` に渡したとおりの文字列とする。
2. The pasta_lua ランタイム shall 失敗の理由に、Windows が返す理由（例: 指定されたモジュールが見つからない）を UTF-8 で入れる。
   > LuaJIT は Windows の理由を ANSI のまま入れるので、UTF-8 の画面では日本語の理由が化ける。ここは LuaJIT とわざと違える（2026-10-18 の話し合いの決定）。比べるテストでは、理由の部分は文字コードをそろえてから比べる。
3. The pasta_lua ランタイム shall pasta が自分で見つけた失敗（Requirement 2.8・3.7・5.2）にも、1. と同じ形の文言を使い、理由の部分に原因を書く。
4. The pasta_lua ランタイム shall 失敗の文言とエラーの型を、`pcall(ffi.load, …)` で受け取ったときに LuaJIT と同じにする。理由の部分（2.・3.）だけを除く。
5. The pasta_lua ランタイム shall 失敗の文言の中の名前とパスを、ANSI に無い文字も欠けず置き換わらない形で出す。

### Requirement 7: 読み込んだ DLL の寿命

**Objective:** ゴーストの利用者として、DLL を使うゴーストを再読み込みしたり終了したりしても、SSP が落ちないでほしい。

#### Acceptance Criteria

1. While Lua の状態が生きている間, the pasta_lua ランタイム shall `ffi.load` のために読み込んだ DLL を解放しない（`ffi.load` が返した値が捨てられても解放しない）。
2. When Lua の状態が閉じられた場合（ゴーストの終了・再読み込み）, the pasta_lua ランタイム shall Lua の状態の後始末（`__gc`・`ffi.gc` の後始末を含む）がすべて終わってから、読み込んだ DLL を解放する。
3. When ゴーストが再読み込みされた場合, the pasta_lua ランタイム shall 新しい Lua の状態で、DLL を改めて読み込めるようにする。
4. If DLL の読み込みに失敗した場合, the pasta_lua ランタイム shall 途中で読み込んだものを残さず解放する。ただし pasta が先に読み終えたあとで元の `ffi.load` が失敗した場合（同じ名前で読み込み済みなので起きない見込み）は、先に読んだ DLL を Lua の状態を閉じるまで残す。

### Requirement 8: 変えないこと

**Objective:** ゴーストの作者として、FFI を有効にしないゴーストは、この変更の前と同じに動いてほしい。

#### Acceptance Criteria

1. While `[loader] ffi` が `true` でない間, the pasta_shiori shall この変更の前とバイト単位で同じ SHIORI の応答を返す。
2. The pasta_lua ランタイム shall `.lua` の `require` の結果（探す順・チャンクの名前・失敗の文言・`@pasta_chunks` の表）を変えない。
3. The pasta_lua ランタイム shall `package.path`・`package.cpath`・`package.loaders`・`package.loadlib` を変えない。
4. The pasta_lua ランタイム shall FFI 以外の既定のライブラリの組み合わせ（`default_libs`）を変えない。
5. The pasta_lua ランタイム shall この spec の DLL の読み込みについて、x86 と x64 のどちらのビルドでも同じ結果を返す。

### Requirement 9: 確かめるテスト

**Objective:** 保守する人として、深いパス・ANSI に無い文字のフォルダ・依存 DLL・ゴーストの取り違えという再現しにくい条件が、いつのまにか壊れていないことを毎回確かめたい。

#### Acceptance Criteria

1. The 自動テスト shall 300 文字を超えるパスに置いた DLL を `ffi.load` で読み、関数を呼べることを確かめる。
2. The 自動テスト shall ANSI に無い文字のフォルダ（`tests/common` の `NON_ANSI_DIR_NAME`）に置いた DLL を `ffi.load` で読み、関数を呼べることを確かめる。
3. The 自動テスト shall 隣に置いた依存 DLL に頼る DLL を `ffi.load` で読めること、依存 DLL が無いときに Requirement 6 の文言で失敗することを確かめる。
4. The 自動テスト shall 名前だけの `ffi.load` が `lua_search_paths` の順で探すこと、見つからない名前が元の `ffi.load` に渡ってシステムの DLL（`kernel32` など）を読めること、相対パスがゴーストの基点から解決されることを確かめる。
5. The 自動テスト shall 同じプロセスで 2 つのゴーストの Lua の状態を作り、それぞれのフォルダの同じファイル名の別の DLL を読んだとき、Requirement 5 のとおりになることを確かめる。
6. The 自動テスト shall 短い ASCII のパスで、元の `ffi.load` と置き換えた `ffi.load` を並べ、見つからない場合などの `pcall(ffi.load, …)` の結果の型と文言を比べる（`.lua` の searcher の比べるテストと同じ形）。
7. The 自動テスト shall `[loader] ffi` が `false`・未記入・`true`・真偽値でない値の各場合を確かめる。
8. The 自動テスト shall Lua の状態を閉じたあとに DLL が解放されること、`ffi.gc` の後始末を残したまま閉じても落ちないこと、閉じてから同じ DLL を読み直せることを確かめる。
9. The 自動テスト shall テスト用の DLL を、人の手を介さずに用意する。`cc` を dev の依存に足し、テストの中で `cl` を呼んで作る（2026-10-18 の話し合いの決定）。
10. The 自動テスト shall x86 と x64 の両方のターゲットで、上の各テストを CI で毎回走らせる（`#[ignore]` にしない）。
11. The 自動テスト shall `PASTA_DEBUG` の環境変数があってもなくても同じ結果を返す。

### Requirement 10: 文書

**Objective:** ゴーストの作者として、DLL をどう作り、どこに置けば読めるか、何が保証され何が保証されないかを知りたい。

#### Acceptance Criteria

1. The 利用者向けマニュアル（`book/src/reference/pasta-toml.md`） shall `[loader] ffi` のキー・型・既定値と、有効にすると Lua から任意のメモリに触れられるようになる注意を示す。
2. The 利用者向けマニュアル（`book/src/reference/startup.md`） shall `ffi.load` が DLL を探すフォルダと順、相対パスの基点、依存 DLL を探す場所を示す。
3. The 利用者向けマニュアル shall 保証の範囲（DLL を見つけて読み込み、`ffi.load` の値として返すところまで）と、保証しないこと（DLL のファイル名の ANSI の制限、取り違えを防ぐために失敗する場合、DLL の中で ANSI 版の API を使った場合の失敗、`PATH` にある依存 DLL）を、既知の制限の節に書く。
4. The 利用者向けマニュアル shall C モジュールを `require` で読む道は使えないこと、C の DLL は FFI で使うことを書く。
5. The 内部設計の文書（`book/src/internals/loader.md`） shall `ffi.load` の置き換えの流れと、LuaJIT と違う点を書く。
6. The スキル `pasta-lua-coding` の `references/` shall マニュアルから再生成され、鮮度の照合（`gen-skill-refs.mjs --check`）を通る。

## Open Questions

なし。2026-10-18 の要件の話し合いで、すべての議題を確定した（brief の「2026-10-18 要件の話し合いの決定」）。設計で詰めることは `research.md` の「設計への申し送り」にある。
