# Requirements Document

## Project Description (Input)
**誰が困っているか**: ゴーストの作者（`＄＊` のグローバル変数や `@pasta_persistence` で、起動をまたいで値を残したい人）と、pasta の開発者。

**現状**: pasta の既定のシリアライズは TOML である（2026-10-24 開発者の方針「ユーザー要望で json が要求されるまではシリアライズは toml」）。ところが、セーブファイルは `profile/pasta/save/save.json`（`obfuscate = true` なら gzip した `save.dat`）に JSON で書かれる。`@pasta_persistence` は TOML でなければならず（2026-10-24 開発者の決定）、今の JSON は仕様からの逸脱で、**致命的な不具合**として最優先で直す。ほかに、`@pasta_log` の表の出力と、デバッグ用のソースマップのサイドカー（`<lua_path>.map`）も JSON である。

**何を変えるか**: セーブファイルを TOML にする（既定 `profile/pasta/save/save.toml`）。今ある `save.json`・`save.dat` は読まず、移し替えず、消さない（リリースノートで詫びる）。TOML で書けない値は、そのキーだけ保存せず警告する。読めないセーブは `.broken` へ退避してから空のセーブで始め、空で上書きしない。`@pasta_log` の表は TOML の書き方で 1 行に出す。サイドカーも TOML にする。マニュアル・スキルの references・`tech.md` を TOML の記述に直す。詳細は `brief.md` を参照。

## Introduction

本仕様は、pasta が自分で読み書きするファイルと出力から JSON を追い出し、TOML にそろえる。いちばん大事なのはセーブファイルで、ゴーストの作者が残した値（`＄＊名前`、Lua の `save.名前`）を TOML のファイルに保存し、次の起動で読み戻す。

本仕様の後のセーブファイルの例（`profile/pasta/save/save.toml`）:

```toml
"会員番号" = "01"
play_count = 42
last_played = "2026-10-24"

[flags]
tutorial_complete = true

[[inventory]]
name = "剣"
count = 1
```

`＄＊` の値に TOML で書けない形（例: `＄＊混ざり` に `{1, 2, x = 3}`）を入れたとき、ゴーストの終了時の保存は、そのキーだけ外してほかのキーを保存し、ログに警告を残す:

```text
WARN  persistence: key '混ざり' was not saved: table mixes array items and named keys
```

`@pasta_log` の表の出力の例（本仕様の後）:

```text
log.info({ player = "ユーザー", score = 100 })   → { player = "ユーザー", score = 100 }
log.info({ 1, 2, 3 })                           → [1, 2, 3]
log.info({})                                    → {}
```

用語:
- **セーブ**: `＄＊` のグローバル変数が入る Lua の `save` テーブル（`require("pasta.save")`）を、ファイルに残すしくみ。`@pasta_persistence` の `load()`・`save(data)` と、ランタイムの終了時（ゴーストの終了・辞書の再読込）の自動保存から成る。本書では、この全体を **セーブ機能** と呼ぶ。
- **セーブファイル**: セーブ機能が読み書きするファイル。保存先は `pasta.toml` の `[persistence]` で決まる。
- **保存先**: `[persistence]` の `file_path` と `obfuscate` から決まる、セーブファイルのパス（ゴーストのフォルダからの相対パス）。
- **古いセーブ**: 本仕様の前の pasta が書いた JSON のセーブファイル（`save.json`、または gzip した JSON の `save.dat`）。
- **書けない値**: TOML のセーブファイルに、意味を変えずに書き表せない Lua の値（Requirement 4 で線を引く）。
- **退避**: 読めないセーブファイルを、消さずに別の名前（`<保存先>.broken`）へ移すこと。
- **サイドカー**: デバッグのソースマップを、生成した `.lua` の隣に任意で書き出すファイル（`<lua_path>.map`。`[debug] source_map_sidecar = true` などで有効）。

## Boundary Context

- **In scope**:
  - セーブファイルの形式（TOML。`obfuscate = true` なら gzip した TOML）、保存先の既定と拡張子の決まり、読み込みと書き込み。
  - 古いセーブを引き継がないこと（読まない・移し替えない・消さない）。
  - TOML で書けない値の線引きと、そのキーだけ保存しない扱い（`save(data)` と終了時の自動保存の両方）。
  - 読めないセーブの退避（`ponytail-lua-loader` の Must fix から取り込んだ。2026-10-24 開発者の決定）。
  - `@pasta_log` の表の出力の書式。
  - サイドカーの形式。
  - 文書: マニュアル（`lua/modules/pasta-persistence.md`・`reference/pasta-toml.md` の `[persistence]`・`grammar/variables.md` のグローバル変数の保存先・`lua/modules/pasta-log.md` の値の変換規則・`internals/execution-model.md` の永続化・`internals/logging-encoding.md` の表の変換・`internals/debug.md` と `debug/source-level.md` のサイドカー）、スキルの references の作り直し、steering の `tech.md`。
  - リリースノートの詫びの申し送り（本仕様のコミットを「動きが変わる変更」として示し、詫びの文案を渡す）。
- **Out of scope**:
  - `@json` モジュールと `OnPastaSetSaveVariable` の値の形式（`mlua-stdlib-removal` が持つ）。
  - DAP（デバッガーとの通信）と LSP の JSON。プロトコルが JSON を決めているので残す。
  - 保存の仕組みの作り直し（いつ保存するか、何を保存するか）。セーブの中身の範囲（`novel-variable-scope` が、セーブごとと全セーブ共通に分ける）。
  - `novel-save-slots` のスロットのファイル（その spec が、ここで決めた形式に揃える）。
  - 古いセーブの移し替え（2026-10-24 開発者の決定「ごめんなさいして破棄するだけでよい」）。
  - リリースノートの本文（`doc/release-notes/v0.4.3.md`）を書くこと。`release` スキルが書く。
  - 見本のゴースト `hello-pasta` の `pasta.toml` の読まれないキー `[persistence] data_dir` の整理（見出しの有無を見るテストが `ponytail-check-release` の持ち場にあるため、本仕様では動きを変えない。直す場合も見出しは残す）。
- **Adjacent expectations**:
  - `mlua-stdlib-removal`（同じウェーブ）: Lua の値を TOML へ変える口（`@toml`）を別に足す見込みだが、本仕様のセーブ・ログ・サイドカーはそれに頼らない。
  - `ponytail-lua-loader`（0.4.4）: 退避は本仕様が持つ。あちらは本仕様の後のセーブの形を前提に測り直す。
  - `novel-variable-scope`・`novel-checkpoint-rollback`・`novel-save-slots`（0.5.0）: 本仕様の TOML のセーブと、書けない値の扱いを前提にする。
  - `mlua-0-12-upgrade`: 本仕様の後のセーブ・ログの変換を前提に、版を上げる。

## Requirements

### Requirement 1: セーブファイルを TOML で書いて読む

**Objective:** ゴーストの作者として、`＄＊` の値が TOML のセーブファイルに残り、次の起動で戻ってきてほしい。そうすれば、pasta の決まり（シリアライズは TOML）どおりに、人が読めて手でも直せるセーブを持てる。

#### Acceptance Criteria

1. When セーブ機能がセーブファイルを書くとき（`save(data)` と終了時の自動保存）, the セーブ機能 shall 表の中身を TOML の文書として書く。`obfuscate = false` のときは TOML の文字そのもの（UTF-8）を、`obfuscate = true` のときはその TOML を gzip で圧縮したものを書く。
2. The セーブ機能 shall JSON でセーブファイルを書かない。
3. When セーブ機能がセーブファイルを読むとき, the セーブ機能 shall 先頭が gzip の印なら展開してから、そうでなければそのまま、TOML として読む。`obfuscate` の設定に関係なく、どちらの形も読める。
4. The セーブ機能 shall JSON のセーブファイルを読む道を持たない。
5. When セーブファイルが空のとき, the セーブ機能 shall 空の表を返す（警告は出さない）。
6. If セーブファイルが無いとき（初回の起動など）, the セーブ機能 shall 空の表を返し、警告のログを出す（今の動きのまま）。
7. When セーブ機能が書いたセーブファイルを読み戻すとき, the セーブ機能 shall 書いた時点の表と同じ値（Requirement 4 で外したキーを除く）を返す。文字列・真偽値・数・入れ子の表・配列が、同じ形で戻る。
8. The セーブ機能 shall キーの並びと書き方が毎回同じセーブファイルを書く（同じ中身の表からは、同じ中身のファイルができる）。
9. The セーブ機能 shall 書き込みの決まりを今のまま保つ。保存先のフォルダが無ければ作る。いったん一時ファイルに書き終えてから保存先の名前に置き換え、途中で失敗しても元のセーブファイルを壊さない。
10. The セーブ機能 shall `load()`・`save(data)` の呼び方と戻り値（`load()` は表、`save(data)` は成功で `true, nil`、失敗で `nil, エラーメッセージ`）を変えない。
11. When 作者が手で直したセーブファイルに UTF-8 の BOM が付いているとき, the セーブ機能 shall BOM を無視して読む。
12. When 作者が手で書いたセーブファイルに TOML の日時の値があるとき, the セーブ機能 shall その値を、日時を TOML の標準の書き方（`1979-05-27T07:32:00Z` など）にした文字列として読む。次に保存するときは、その値を TOML の文字列として書く（日時の値には戻さない）。

### Requirement 2: 古いセーブを引き継がない

**Objective:** pasta の開発者として、本仕様の前の JSON のセーブを読まず、移し替えず、消さないでほしい。そうすれば、移し替えの道を作らずに TOML へ切り替えられ、作者のファイルも失わない（今、セーブを積極的に使うゴーストは居ない。2026-10-24 開発者の決定）。

#### Acceptance Criteria

1. When 本仕様の後の pasta が、古いセーブ `profile/pasta/save/save.json` のあるゴーストを起動したとき, the セーブ機能 shall `save.json` を読まず、空の表から始める（既定の保存先が `save.toml` になるため）。
2. The セーブ機能 shall 保存先でない古いセーブ（既定の設定のゴーストの `save.json` など）を、消さず、名前を変えず、中身を書き換えない。
3. If 古いセーブが保存先と同じ名前で残っているとき（`obfuscate = true` のゴーストの gzip した JSON の `save.dat`、または作者が `file_path` に古いセーブの名前を書いているとき）, the セーブ機能 shall それを読めないセーブとして扱い（Requirement 5）、`<保存先>.broken`（例: `save.dat.broken`）へ退避して空の表から始める。中身は消さない。
4. The マニュアル shall 0.4.3 から保存先と形式が TOML に変わり、古いセーブが引き継がれないこと、古いファイルが残ることを、`@pasta_persistence` の頁に書く。
5. The 本仕様の成果物 shall 本仕様のコミットを「動きが変わる変更」として示し（詳しいリリースノートの該当の欄に出るように）、セーブが引き継がれないことを詫びるリリースノートの文案を、`release` スキルへ申し送る。

### Requirement 3: 保存先の既定と拡張子の決まり

**Objective:** ゴーストの作者として、保存先の既定と `obfuscate` のときの名前を、TOML に合った分かりやすい決まりで知りたい。そうすれば、`pasta.toml` を書かなくても、書いても、セーブファイルの場所が分かる。

#### Acceptance Criteria

1. The セーブ機能 shall `[persistence] file_path` を書かないときの保存先を `profile/pasta/save/save.toml` にする。
2. While `obfuscate = true` のとき, the セーブ機能 shall `file_path` の末尾が `.toml` ならその末尾だけを `.dat` に替え、末尾が `.dat` ならそのまま、それ以外なら末尾に `.dat` を足したパスを保存先にする（既定では `profile/pasta/save/save.dat`）。
3. While `obfuscate = true` のとき, the セーブ機能 shall 拡張子を替えるのを、パスの末尾の 1 か所だけにする（途中のフォルダ名に同じ文字列があっても替えない）。
4. Where 作者が `file_path` に `.json` で終わる名前を書いているとき, the セーブ機能 shall その名前を書いたとおりのパスとして使い、中身は TOML で読み書きする（`.json` を特別に扱わない。`obfuscate = true` なら末尾に `.dat` を足す）。
5. If `file_path` が絶対パス、または `..`・ルート・ドライブの指定を含むとき, the セーブ機能 shall 今のとおりゴーストの読み込みを失敗させる。
6. The セーブ機能 shall `[persistence]` のキー（`obfuscate`・`file_path`・`debug_mode`）の名前と意味を変えない。

### Requirement 4: TOML で書けない値の扱い

**Objective:** ゴーストの作者として、`save` テーブルに TOML で書けない値が混ざっても、ほかの値は保存されてほしい。そして、何が保存されなかったかを知りたい。そうすれば、1 つの値のせいでセーブ全体を失わない（2026-10-24 開発者の決定）。

#### Acceptance Criteria

1. The セーブ機能 shall 次の Lua の値を **書ける値** として保存する。
   - 文字列（UTF-8 として正しいもの）、真偽値。
   - 数。整数で表せる数は TOML の整数、それ以外は TOML の浮動小数点数（`nan`・`inf` を含む）として書く。
   - キーが 1 から順に穴なく並び、ほかのキーを持たない表（**配列**）。中身がすべて書ける値なら、TOML の配列として書く。
   - キーがすべて文字列の表。中身がすべて書ける値なら、TOML の表として書く。
   - 空の表（TOML の空の表として書き、読み戻すと空の表になる）。読み込んだセーブの空の配列は、空の配列のまま書き戻す。
2. The セーブ機能 shall 次の値を **書けない値** とする。
   - `nil` の穴のある配列（`{1, nil, 3}` など）。
   - 配列の並びと文字列のキーが混ざった表（`{1, 2, x = 3}` など）。
   - 文字列でないキー（数・真偽値・表など）を持つ、配列でない表（`{[5] = "x"}`、`{[true] = 1}` など）。
   - 関数・コルーチン・ユーザーデータ・ライトユーザーデータ。
   - 自分自身を中に含む表（循環）。
   - 上限（Requirement 4.7）を超えて入れ子になった表。
   - UTF-8 として正しくない文字列（キーを含む）。
3. When 保存する表の中で、文字列のキーの値が書けない値のとき, the セーブ機能 shall そのキーだけを保存せず、同じ表のほかのキーと、ほかの表のキーは保存する。
4. When 配列の要素そのものが書けない値のとき（要素が関数である、要素の表が混ざった表である、配列に穴がある など）, the セーブ機能 shall その配列を持つキーごと保存しない（要素だけを抜くと番号がずれるため）。要素の表の中の文字列のキーの値だけが書けないときは、Requirement 4.3 のとおりそのキーだけを外し、配列は保存する。
5. When セーブ機能がキーを保存しなかったとき, the セーブ機能 shall 保存しなかったキーごとに 1 件、警告のログを出す。警告には、保存の表の根からのキーの道筋（`flags.secret`、`inventory[1].f` のように）と、書けない理由（`function`・`mixed table`・`hole in array` などが分かる言葉）を入れる。
6. When 書けない値のために一部のキーを保存しなかったとき, the セーブ機能 shall それでも残りの内容でセーブファイルを書き、`save(data)` は成功（`true, nil`）を返す。
7. The セーブ機能 shall 入れ子の深さに上限を置き、上限を超えた表を持つキーを書けない値として扱う。上限の段数はマニュアルに書く。
8. The セーブ機能 shall どんな表を渡されても（深い入れ子・循環・大きな表を含む）、ゴーストを止める異常終了（スタックの溢れなど）を起こさない。
9. The セーブ機能 shall 終了時の自動保存でも、`save(data)` と同じ線引きと扱いをする。
10. When 保存する表の根（`save` テーブル、または `save(data)` に渡した表）に文字列でないキーがあるとき（`save[1] = "x"` など）, the セーブ機能 shall 根を配列としては書かず、文字列でないキーだけを Requirement 4.5 の警告つきで保存しない（TOML のファイルの根は、いつもキーのある表であるため）。

### Requirement 5: 読めないセーブの退避

**Objective:** ゴーストの作者として、セーブファイルが壊れていたり一時的に読めなかったりしたときに、元のファイルを失わないでほしい。そうすれば、手で直した書き間違いや、ロックで読めなかったときも、元のセーブを取り戻せる（2026-10-19・2026-10-24 開発者の決定）。

#### Acceptance Criteria

1. If セーブファイルがあるのに読めないとき（TOML として読めない、gzip を展開できない、そのほか「ファイルが無い」以外の読み込みの誤り）, the セーブ機能 shall セーブファイルを `<保存先>.broken`（例: `save.toml.broken`）へ名前を変えて退避してから、空の表を返す。
2. When 退避したとき, the セーブ機能 shall 退避元と退避先のパスと、読めなかった理由を入れた警告のログを出す。
3. The セーブ機能 shall 読めないセーブファイルを、空の表や新しい内容で上書きしない。
4. If 退避先の名前のファイルが既にあるとき, the セーブ機能 shall 既にある退避ファイルを上書きせず、別の名前に退避する。
5. If 退避の名前の変更が失敗したとき, the セーブ機能 shall 空の表を返し、そのランタイムの間は保存先へ書かない。`save(data)` は失敗（`nil, エラーメッセージ`）を返し、終了時の自動保存は書かずにエラーのログを出す。
6. When 退避の後に保存するとき, the セーブ機能 shall 新しいセーブファイルを保存先に書く（退避したファイルには触らない）。
7. The セーブ機能 shall エラーを投げず、読み込みの失敗でゴーストの起動を止めない（今の動きのまま）。

### Requirement 6: @pasta_log の表を TOML の書き方で 1 行に出す

**Objective:** ゴーストの作者として、`@pasta_log` に渡した表の中身を、pasta のほかの所と同じ TOML の書き方で、1 行で読みたい。そうすれば、ログの 1 件が 1 行のまま、表の中身を確かめられる。

#### Acceptance Criteria

1. When `@pasta_log` の関数（`trace`・`debug`・`info`・`warn`・`error`）に表を渡したとき, the `@pasta_log` shall 表を TOML の値の書き方（キーのある表は `{ key = "value" }`、配列は `[1, 2, 3]`、空の表は `{}`）の文字列にして記録する。
2. The `@pasta_log` shall 表の文字列を必ず 1 行にする。文字列の中の改行などの制御文字は、TOML のエスケープ（`\n` など）で書く。
3. The `@pasta_log` shall JSON の書き方で表を記録しない。
4. The `@pasta_log` shall 表の上限を今のまま保つ。キーと値の組が 1000 を超える表は `<table: 要素数 elements>`、入れ子が 10 段を超える表と循環を含む表は `tostring()` の結果にする。
5. If 表が書けない値（Requirement 4.2 の線引き。入れ子の上限はこの Requirement 6.4 の 10 段）を含むとき, the `@pasta_log` shall 表全体を `tostring()` の結果（`table: 0x…`）にする。表の一部だけを黙って省いた文字列は出さない。
6. The `@pasta_log` shall 表のほかの値（文字列・数・真偽値・`nil`・関数など）の記録のしかたを変えない。
7. The `@pasta_log` shall どの値を渡されても、エラーでゴーストを止めない（今の動きのまま）。

### Requirement 7: サイドカーを TOML で書く

**Objective:** デバッグ機能を使う作者と開発者として、サイドカーも TOML で読みたい。そうすれば、pasta が書く出力が TOML にそろう。

#### Acceptance Criteria

1. Where サイドカーの出力が有効なとき, the サイドカー出力 shall 生成した `.lua` の隣の `<lua_path>.map` に、TOML の文書を書く。
2. The サイドカー出力 shall 今と同じ情報（形の版 `version`、由来の `.pasta` のパス `pasta_file`、`[.lua の行, .pasta の行]` の組の並び `pairs`）を書く。組は `.lua` の行の昇順に並べる。
3. The サイドカー出力 shall 同じソースマップからは、同じ中身のファイルを書く。
4. When 書いたサイドカーを読み戻すとき, the サイドカー出力 shall 書いたときと同じ対応（`.lua` の行 → `.pasta` の行）を返す。対応が 0 件のサイドカーも読み戻せる。
5. If サイドカーを書けない・読めないとき, the サイドカー出力 shall 今のとおり失敗を返すだけにし、メモリの中のソースマップとデバッグの動きを変えない。
6. The サイドカー出力 shall ファイル名の決まり（`<lua_path>.map`）と、有効にする設定（`[debug] source_map_sidecar`・`PASTA_DEBUG_SOURCE_MAP_SIDECAR`）を変えない。

### Requirement 8: 文書

**Objective:** ゴーストの作者と、AI にゴーストを書かせる作者として、マニュアルとスキルが、本仕様の後の動き（TOML のセーブ、書けない値、退避、ログの表、サイドカー）を正しく書いていてほしい。そうすれば、書いてあるとおりに使える。

#### Acceptance Criteria

1. The マニュアル shall `lua/modules/pasta-persistence.md` に、TOML（または gzip した TOML）で保存すること、既定の保存先 `profile/pasta/save/save.toml`、`obfuscate` のときの名前の決まり、書ける値と書けない値の線引き、書けない値のキーだけを保存しない扱いと警告、入れ子の上限、読めないセーブの退避、古いセーブを引き継がないことを書く。`_DESCRIPTION` の文字列も本仕様の後の値に合わせる。
2. The マニュアル shall `reference/pasta-toml.md` の `[persistence]`（全体の例と節の両方）、`grammar/variables.md` のグローバル変数の保存先を、TOML と新しい既定の保存先に直す。
3. The マニュアル shall `lua/modules/pasta-log.md` の値の変換規則と、`internals/logging-encoding.md` の表の変換を、TOML の書き方と本仕様の後の上限・戻し方に直す。
4. The マニュアル shall `internals/execution-model.md` の永続化の節、`internals/debug.md` と `debug/source-level.md` のサイドカーの記述を、本仕様の後の形式と動きに直す。
5. The マニュアル shall セーブ・ログの表・サイドカーについて「JSON」と書いた記述を残さない（DAP・LSP、`@json`・`OnPastaSetSaveVariable`、外部データの読み込みの記述は本仕様の範囲外として残る）。
6. The 本仕様の成果物 shall マニュアルから作るスキルの references を作り直し、マニュアルと食い違わない状態にする。
7. The 本仕様の成果物 shall steering の `tech.md` の依存の説明（`toml` と `serde_json` の用途）を、本仕様の後の使い方に合わせる。
8. Where スキル `pasta-ghost-authoring` の `SKILL.md` にセーブを「JSON ファイルに保存」と書いた行が残るとき, the 本仕様の成果物 shall その行を TOML の記述に直す。直すのに開発者の許可が要るときは、許可を求めて直す。

## 前提とした事柄（Assumptions）

要件ディスカッションで確かめる。対応する OPEN QUESTION は research.md の最後にまとめる。

- **A1（キーを外す粒度）**: 書けない値は、いちばん近い「文字列のキー」の単位で外す。配列の要素が書けないときは、配列ごと外す（Requirement 4.3・4.4）。根のキーだけで外す案より、失う値が少ない。
- **A2（入れ子の上限）**: セーブの入れ子の上限は 32 段を仮の値とする（ログの 10 段とは別）。設計で決め、マニュアルに書く（Requirement 4.7）。
- **A3（`.json` の名前）**: 作者が `file_path` に `.json` の名前を書いていても、書いたとおりのパスとして TOML を読み書きする（Requirement 3.4）。そのファイルに古い JSON があれば、読めないセーブとして退避される。
- **A4（`obfuscate` の名前）**: `obfuscate = true` の既定の保存先は今と同じ `save.dat` のまま。古い `save.dat` は退避される（Requirement 2.3）。
- **A5（退避の名前の重なり）**: 既にある `.broken` は上書きしない（Requirement 5.4）。別の名前の付け方は設計で決める。
- **A6（退避できないとき）**: 退避もできないときは、そのランタイムの間は保存先へ書かない（Requirement 5.5）。そのランタイムで変えた値は残らないが、元のセーブは残る。
- **A7（`save(data)` の戻り値）**: キーを外して保存したときも `true, nil` を返し、外したことはログだけで知らせる（Requirement 4.6）。
- **A8（ログの表）**: ログの表に書けない値があれば、表全体を `tostring()` にする（今の「変換できなければ tostring」と同じ。Requirement 6.5）。
- **A9（サイドカー）**: ファイル名 `<lua_path>.map` は変えない（Requirement 7.6）。`version` を上げるかは設計で決める。
