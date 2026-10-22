# Brief: ponytail-lua-transpiler

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域に `/ponytail:ponytail-audit` をやり直す。領域は `crates/pasta_lua/src` の `code_gen/`・`normalize.rs`・`config.rs`・`context.rs`・`transpiler.rs`。控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、決めごとにする。
- **修正は設計以降**で行う。

## Problem

ゴーストの作者が辞書に Lua ブロックを書くと、トランスパイラがその中身を書き換えてしまう。長い文字列（`[[ ]]`）の中身に空白が入り、空行が消える。

加えて、効かない設定と死んだ状態が、変換器を読む人の手間を増やしている。

## Current State

2026-10-19 の監査で確かめた。

1. **Lua ブロックの中身の書き換え**
   - 字下げ：`code_gen/element_gen.rs` L474-492 が、ブロックを `do` の中・字下げ 1 段で出し、各行に空白 4 つを足す。
     - ```` ```lua ```` の中の `local t = [[\n\\s[0]こんにちは\n]]` が、`"    \s[0]こんにちは"` になる。
   - 空行：`normalize.rs` L108-140 は、次の行が `end` だと、長い文字列の中の空行も消す。
   - ブロックの中の長い文字列を確かめるテストは無い。
2. **死んだ変換の設定 `LineEnding`・`comment_mode`**
   - 場所：`config.rs`（`LineEnding` の enum 全部・`comment_mode`・`without_comments`）、`code_gen/mod.rs`（`line_ending` の欄・`with_line_ending`）。
   - 出力は常に LF。`normalize` が CRLF を置き換え、`fill_line_maps` が `"\n"` でつなぐため。`comment_mode` はどこからも読まれない。
   - `book/src/internals/transpiler.md:102` は、すでに「どちらも効かない」と書いている。
3. **`TranspileContext` の死んだ状態**
   - 場所：`context.rs` L17-36 の `current_module`（読まれない）、L41 の `register_global_scene`（テストだけ）。
   - 呼び手：`transpiler.rs` L159・L238、`code_gen/scope_gen.rs` L120-121（`_context`・`_file_attrs`）。
4. **素通し**
   - `LuaTranspiler::with_defaults` は `Default` の重複。
   - `transpile_with_sink` は、テストだけが使う素通し。

## Desired Outcome

- Lua ブロックの中身が、書いたとおりに出る。長い文字列も、空行もそのまま残る。
- 効かない設定・死んだ状態・素通しが無くなる。

## Approach

監査の最小の直しを採る。

- ブロックの行は、字下げせずに `write_raw(line)`＋行末で出す。
- `normalize` の根は、`end_block` が `end` の前に空行を作ることにある。それを作らないようにすれば、`normalize` は末尾を整えるだけになる。

## Scope

- **In**: 上の 1〜4 と、ギャップ分析で増えた同じ領域の指摘。snapshot（`tests/transpiler`・`src/code_gen/*_tests.rs`）の作り直し、`book/src/internals/transpiler.md:99-102`。
- **Out**: `TranspileContext` の、ファイルの属性の欄と関数は残す（2026-10-19 開発者の決定）。対象は `accumulate_file_attr`・`file_attrs`・`merge_attrs`。0.5.0 の `scene-attribute-store` が、ファイルレベル属性の継承に使うため。

## Boundary Candidates

- ブロックの出力（`element_gen.rs`・`code_gen/mod.rs` の `end_block`）
- 後処理（`normalize.rs`）
- 設定と文脈の削り（`config.rs`・`context.rs`・`transpiler.rs`・`scope_gen.rs`）

## Out of Boundary

- `loader/`・`runtime/`（`ponytail-lua-loader`）
- ファイルの属性の仕組み（上の Out）

## Upstream / Downstream

- **Upstream**: `pasta_dsl`（AST と span）
- **Downstream**: 実行時の Lua、デバッガのソースの対応表（行の対応が変わらないこと）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-lua-loader`（`crates/pasta_lua/src/lib.rs` の再輸出を共有する。行が別なので、後から入る側が合わせる）
  - `ponytail-dsl-parser`（span を作る側。値は変わらない）
  - `scene-attribute-store`（0.5.0）

## Constraints

- 公開クレートの pub API を消してよい（2026-10-19 開発者の決定）。README と `tech.md` の断り書きは、10 本のうち先に入る 1 本が 1 回だけ書く。
- ブロックの行の対応（デバッガのソースの対応表）を壊さない。
- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_lua`。cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。`sample.generated.lua` の改行だけの差分は戻す。
