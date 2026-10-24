# Brief: toml-serialization

> **ステータス**: 未着手（2026-10-24 の `/kiro-discovery` で起票。開発者の方針「pasta から json は一旦追い出して。ユーザー要望で json が要求されるまではシリアライズは toml」）。**セーブファイルが TOML でないのは致命的な不具合で、最優先で直す**（2026-10-24 開発者の決定「toml じゃないということは、致命的です。最優先対応事項の不具合だと考えてください」）。ほかのどの spec よりも先に入れる。要件定義は Opus。ログの表とサイドカーは致命的ではないので、セーブファイルの直しが遅れるなら別の spec に分けてよい。

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
  - `ponytail-lua-loader`（`persistence.rs` を共有する。同じウェーブに置かない）
  - `ponytail-lua-debug`（`debug/` を共有する）
  - `novel-variable-scope`（セーブの形と互換を扱う。こちらが先に入るのが推奨）
  - `mlua-stdlib-removal`（マニュアルの Lua の部品の頁を共有しうる）

## Constraints

- TOML のセーブの内容を失わない。読めないときに空の表で上書きしない。JSON のセーブを引き継がないのは、開発者が決めた例外である（今 save を積極的に使っているゴーストが居ない）。
- TOML で書けない Lua の値がある。`nil` の穴のある配列、配列と表の混ざった表、数のキーなど。どう書くか（断る・文字列のキーにする・警告して落とす）を要件で決める。今の JSON でも書けない値（関数・循環）は今どおり断る。
- `obfuscate`（gzip）は残す。
- 作者が `[persistence] file_path` に `.json` の名前を書いているときの扱いを、要件で決める。

## 開発者の判断が要るもの（要件の前）

- 移し替えは決まった（しない。2026-10-24 開発者の決定）。
- **TOML で書けない値**: 推奨は「書けない値を含むキーは保存せず、警告のログを出す（ほかのキーは保存する）」。
