# Brief: toml-serialization

> **ステータス**: 未着手（2026-10-24 の `/kiro-discovery` で起票。開発者の方針「pasta から json は一旦追い出して。ユーザー要望で json が要求されるまではシリアライズは toml」）。**セーブファイルが TOML でないのは致命的な不具合で、最優先で直す**（2026-10-24 開発者の決定「toml じゃないということは、致命的です。最優先対応事項の不具合だと考えてください」）。ほかのどの spec よりも先に入れる。0.4.3 の先頭（2026-10-24 開発者の決定）。要件定義は Opus。ログの表とサイドカーは致命的ではないので、セーブファイルの直しが遅れるなら別の spec に分けてよい。

## Problem

pasta の既定のシリアライズは TOML である（2026-10-24 開発者の方針）。ところが、pasta が自分で読み書きするファイルと出力に、JSON が残っている。

**`@pasta_persistence` は TOML でなければならない**（2026-10-24 開発者の決定「pasta_persistence は toml です」「toml でないとだめ」）。今の JSON は仕様からの逸脱で、直すべき不具合として扱う。

- 経緯: Rune 版の pasta（`crates/pasta`、〜2025-12）のセーブは TOML だった（`src/stdlib/persistence.rs` の `toml_to_string`／`toml_from_string`、テストの材料 `sample_save.toml`）。Rune 版は 2026-01-01 に廃止された（`23978966`）。2026-01-26 に Lua 版の `@pasta_persistence` を作った spec `store-save-persistence`（`c09d4612`）の要件が「JSON 形式で保存する」（requirements.md 55・67 行）と書き、TOML から替える理由も比べた跡も無いまま、JSON になった。

- **セーブファイル**: `＄＊` のグローバル変数（Lua の `save` テーブル）は、終了時に `profile/pasta/save/save.json`（`obfuscate = true` なら gzip した `save.dat`）へ JSON で保存される。
- **ログの表**: `@pasta_log` は、引数の Lua の表を JSON の文字列にしてログに出す。
- **ソースマップのサイドカー**: デバッグ用に生成 `.lua` の隣へ書く `<lua_path>.map` は、pasta 独自の形の JSON である。

`@json` と `OnPastaSetSaveVariable` の値は、`mlua-stdlib-removal` が持つ。この spec は、残りの 3 つを持つ。

## Current State

2026-10-24（main `543e0437`）に確かめた。

- **セーブファイル**:
  - `crates/pasta_lua/src/runtime/persistence.rs`。`load_from_file` は先頭のバイトで gzip か素の JSON かを見分けて読む。`save_to_file` は `serde_json::Value` を書く。読めないファイルは空の表を返し、警告を出す（エラーは投げない）。
  - `runtime/lifecycle.rs` 38〜40 行が、終了時に `save` テーブルを `serde_json::Value` にする。
  - `loader/config/sections.rs` の `[persistence]`: `file_path` の既定は `"profile/pasta/save/save.json"`。`obfuscate = true` なら、`.json` を `.dat` に替える（`effective_file_path`）。
  - マニュアル: `lua/modules/pasta-persistence.md`（「JSON（または gzip 圧縮した JSON）でファイルに保存」）、`reference/pasta-toml.md` の `[persistence]`（165〜167 行・706〜713 行）、`grammar/variables.md` など。
  - `ponytail-lua-loader` が、読めないセーブを `.broken` に退避する直しを持つ（同じ `persistence.rs`）。
- **ログの表**: `runtime/log.rs` 108〜129 行。入れ子の深さを先に確かめ（`table_depth_exceeds`）、`from_value_with::<serde_json::Value>` で変換して `serde_json::to_string` で 1 行にする。変換できなければ `tostring` に戻す。
- **サイドカー**: `debug/source_map/sidecar.rs`。`SidecarFile`（`version` 付き）を JSON で書き（`write_sidecar`）、読み戻す（`read_sidecar`）。任意のディスク出力で、出荷物の振る舞いは変えない。
- **残す JSON**: DAP（`debug/dap/`・`debug/transport/`・`debug/wiring/`・`debug/enable.rs`）と LSP（`pasta_lsp/src/transport.rs`）は、プロトコルが JSON を決めているので対象外。`serde_json` の依存は、これらのために残る。

## Desired Outcome

- セーブファイルが TOML で書かれる（既定 `profile/pasta/save/save.toml`）。今あるゴーストの `save.json`・`save.dat` は読まない（移し替えない）。リリースノートで詫びる。
- `@pasta_log` の表が、TOML の書き方で 1 行に出る。
- サイドカーが TOML になる。
- マニュアルが TOML で書いている。

## Approach

`toml` クレート（既に `pasta_lua` の依存）で読み書きする。Lua の値と TOML の値の変換は、既にある `toml_to_lua`（`module_registry.rs` 216 行）と、mlua の serde の変換を使う。

セーブの移し替えはしない（2026-10-24 開発者の決定「今のところ、save を積極的に使ってるゴーストは居ないので、ごめんなさいして破棄するだけでよい」）。古い `save.json`・`save.dat` は読まず、`save.toml` から新しく始める。古いファイルは消さずに放っておく（pasta が作者のファイルを消さない）。リリースノートで、セーブが引き継がれないことを詫びる。

## Scope

- **In**:
  - セーブファイルの形式（`persistence.rs`・`lifecycle.rs`・`[persistence]` の既定と拡張子の決まり・gzip の扱い）。JSON の読み取りは残さない
  - リリースノートの詫び（セーブが引き継がれない）
  - 読めないセーブの退避（`ponytail-lua-loader` の Must fix から取り込んだ。2026-10-24 開発者の決定）。壊れた `save.toml` は `save.toml.broken` へ rename してから空の表を返し、警告を出す。空で上書きしない（2026-10-19 の決定を引き継ぐ）
  - TOML で書けない値の扱い（そのキーだけ保存せず、警告のログを出す。ほかのキーは保存する。2026-10-24 開発者の決定）
  - `@pasta_log` の表の出力
  - ソースマップのサイドカー
  - マニュアル（`pasta-persistence.md`・`pasta-toml.md` の `[persistence]`・変数の章・ログの章・内部設計のデバッグの頁）とスキルの references の作り直し、`tech.md`
- **Out**:
  - `@json`・`OnPastaSetSaveVariable`（`mlua-stdlib-removal`）
  - DAP・LSP の JSON
  - セーブの中身の範囲（`novel-variable-scope` がセーブごとと全セーブ共通に分ける）

## Boundary Candidates

- セーブファイル（形式）
- 出力の書式（ログの表・サイドカー）

## Out of Boundary

- 保存の仕組みの作り直し（いつ保存するか、何を保存するか）
- `novel-save-slots` のスロットのファイル（その spec が、ここで決めた形式に揃える）

## Upstream / Downstream

- **Upstream**: なし（`mlua-stdlib-removal` とは独立に入れられる）
- **Downstream**:
  - `novel-variable-scope`・`novel-checkpoint-rollback`・`novel-save-slots`（0.5.0。保存の範囲を分け、控えとスロットを書く。セーブの形式は TOML を前提にする）
  - `ponytail-lua-loader`（`.broken` に退避する直しを、TOML のセーブに合わせる）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-lua-loader`（0.4.4。`persistence.rs` を共有する。読めないセーブの退避はこの spec が引き取った）
  - `ponytail-lua-debug`（`debug/` を共有する）
  - `novel-variable-scope`（セーブの形と互換を扱う。こちらが先に入るのが推奨）
  - `mlua-stdlib-removal`（マニュアルの Lua の部品の頁を共有しうる）

## Constraints

- TOML のセーブの内容を失わない。読めないときに空の表で上書きしない。JSON のセーブを引き継がないのは、開発者が決めた例外である（今 save を積極的に使っているゴーストが居ない）。
- TOML で書けない Lua の値がある。`nil` の穴のある配列、配列と表の混ざった表、数のキーなど。そのキーだけ保存せず、警告のログを出す（2026-10-24 開発者の決定）。どの値を「書けない」とするかの線引きは要件で決める。関数・循環も同じ扱い。
- `obfuscate`（gzip）は残す。
- 作者が `[persistence] file_path` に `.json` の名前を書いているときの扱いを、要件で決める。

## 開発者の判断が要るもの（要件の前）

- すべて決まった（2026-10-24 開発者の決定）。移し替えはしない。TOML で書けない値は、そのキーだけ保存せず警告する。読めないセーブの退避を取り込む。

## 2026-10-24 棚卸の再測定（main c20ad104）

- **前提の変化**: 起票（main `543e0437`）の後に入ったのは文書だけで、Current State の行の位置は今も合っている。依存の更新（PR #187）は、ここで触るファイルに触れていない。読めないセーブの退避は `ponytail-lua-loader` からこの spec へ移った。
- **触るファイル**: セーブの読み書き `crates/pasta_lua/src/runtime/persistence.rs`（568 行。作者が呼ぶ `save` の `save_impl`・`load` の `load_impl` もここ）、終了時の保存 `runtime/lifecycle.rs`（63）、`[persistence]` の既定と拡張子の決まり `loader/config/sections.rs`（356。60〜115 行あたり）、ログの表 `runtime/log.rs`（241。108〜129 行）、サイドカー `debug/source_map/sidecar.rs`（145）。試験は `tests/runtime/persistence_integration_test.rs`・`tests/loader/config_test.rs`（325〜391 行）・`tests/loader/config_sections_test.rs`・`debug/source_map_sidecar_tests.rs`・`tests/loader_source_map_build_test.rs`。マニュアルは `lua/modules/pasta-persistence.md`・`reference/pasta-toml.md`（167・713〜722 行）・`grammar/variables.md`（318〜320 行）・`lua/modules/pasta-log.md`（104〜110 行）・`internals/execution-model.md`（440〜458 行）・`internals/logging-encoding.md`（196〜201 行）・`internals/debug.md`（349 行）・`debug/source-level.md`（146 行）。`tech.md` 37 行。新しいファイルは要らない見込み。1,000 行に近いものは無い。
- **依存**: `toml` 1.1.2 は既に `pasta_lua` の依存で、書き出しも読み込みもできる。`serde_json` はデバッガーとの通信が使い続けるので外さない。ルートの `Cargo.toml`・`Cargo.lock` は触らない（席を取らない）。
- **規模**: 12〜14 タスク（セーブ 8 前後、ログの表 2、サイドカー 2、文書 2〜3）。
- **先に要るもの**: 働きの上では無い。0.4.3 の中のファイルの重なり:
  - `mlua-stdlib-removal`: 試験 `tests/loader/config_test.rs`（あちらの `@env` の試験 396〜436 行は、モジュールが消えても通るので、あちらが手を付けなければ重ならない）。`runtime/module_registry.rs`（TOML を Lua の表にする `toml_to_lua` が 216 行にあり、あちらは `@toml` をここに足す見込み。こちらは触らず、mlua の値の変換か `persistence.rs` の中の変換で読む約束にする）。マニュアル `pasta-log.md`・`execution-model.md` と `tech.md` は節が別。
  - `recommend-menu-scenes`: `loader/config/sections.rs`（あちらが別名の出どころの印 `SceneAliasSource`（349〜356 行）を変えるときだけ）。`reference/pasta-toml.md` は節が別。
  - `ponytail-check-release`・`shiori4-v3-portable`: 重ならない。
  - `mlua-0-12-upgrade`: mlua の値の変換を使う 3 ファイル（`persistence.rs`・`log.rs`・`lifecycle.rs`）は、どれもこの spec の持ち場。こちらが先に入る。
- **種別**: バグ（セーブファイルが JSON で、TOML という決まりに反する。開発者が致命的と決めた）。
- **要件定義のモデル**: Opus。形式は TOML、移し替えをしない、書けない値はそのキーだけ落とす、退避する、まで決まった。残る線引き（数のキー・穴のある配列・空の表）は、今の JSON と同じ mlua の値の変換の決まりに揃える案が立つ。
- **分割の案**: 不要。ログの表とサイドカーは合わせて 4 タスクほどで、分けるとマニュアルと `tech.md` を 2 本で取り合う。起票のときの「遅れるなら分けてよい」は、規模からは要らない。
- **見つけた穴・古くなった記述**
  - Current State の「`ponytail-lua-loader` が、読めないセーブを退避する直しを持つ」と、Downstream の同じ趣旨の行は古い（退避はこの spec の In）。
  - スキル `pasta-ghost-authoring/SKILL.md` 175 行が「JSON ファイルに保存」と書いている。Scope に無い。直すとき、分類器が止めたら開発者の許可を求める。
  - 見本のゴーストの `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml` 56〜58 行の `[persistence] data_dir` は、pasta が読まないキーである。直すなら見出しは残す（`pasta_sample_ghost/tests/integration_test.rs` 52 行が見出しの有無を見る。その試験は `ponytail-check-release` の持ち場なので触らない）。
  - 保存の変換には、ログの表にある入れ子の深さの先の確かめ（`log.rs` の `table_depth_exceeds`）が無い。キーごとに落とす変換を書くとき、深い入れ子と循環の守りも置く。
  - 「リリースノートで詫びる」は spec の中では書けない。見どころのファイル `doc/release-notes/v0.4.3.md` は `release` スキルが書く。spec はコミットの件名に `!` を付け（詳しいリリースノートの「動きが変わる変更」に出る）、詫びの文案を申し送る。
  - `pasta_shiori/src/shiori_lifecycle_tests.rs` 552 行の `blocker/save.json` は、保存を失敗させるための名前で、拡張子に意味は無い。直さなくても通る。
