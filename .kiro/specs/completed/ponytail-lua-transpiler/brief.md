# Brief: ponytail-lua-transpiler

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。0.4.5（2026-10-24 開発者の決定「ポニーテールは 0.0.1 後ろからの着手に変更」。経緯: 2026-10-19 に 0.4.3、2026-10-22 に版に縛らない隙間、2026-10-23 に 0.4.3）。0.4.3 の mlua 0.12 の後の main で測り直す。同じ監査の 10 本と同じウェーブで並走できる。

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

## 申し送り（2026-11-01、`ponytail-lua-loader` の設計より）

- `ponytail-lua-loader` は、finalize 前の `@pasta_search` に常に空の `TranspileContext` を渡す形にし、`loader/process.rs` の `process_incremental` で `TranspileContext::merge_from` を呼ばなくなる。入った後は、`merge_from` の本番の使い手が無くなる。消すか残すかは、この spec が決める（`src/context.rs` はこの spec の持ち場）。
- `lib.rs` の再輸出の行を、`ponytail-lua-loader` と共有する（`presentation`・`Encoder`/`Encoding`・`TranspileResult` の行が消える）。後から入る側が合わせる。

## 2026-11-02 棚卸の再測定（main 491cc23c）

- **前提の変化**
  - この spec の領域（`code_gen/`・`normalize.rs`・`config.rs`・`context.rs`・`transpiler.rs`）は、2026-10-19 の監査から変わっていない（mlua 0.12 も `ponytail-lua-loader` も触れていない）。Current State の場所（`element_gen.rs` 474〜492 行・`normalize.rs` 108〜140 行・`context.rs` 17〜36・41 行・`transpiler.rs` 42・94 行）はそのまま。
  - `ponytail-lua-loader` が入った。`lib.rs` の再輸出の `presentation`・`Encoder`/`Encoding`・`TranspileResult` は消えた。この spec が `lib.rs` で触るのは 50 行の `LineEnding` だけで、ほかの未完了の spec の brief は `lib.rs` を挙げていない。
  - 申し送りの `merge_from`: `TranspileContext::merge_from`（`context.rs` 115〜126 行）は、テストも含めて呼び手が 0 になった。死んだ状態（3 番）に足して消す。消すと `pasta_core` の `SceneRegistry::merge_from`・`WordDefRegistry::merge_from` がテストだけになる。そちらは `ponytail-core-registry` の持ち場なので、後から入る側が消す（こちらが後ならこの spec が消す）。
  - README と `tech.md` の断り書きは `ponytail-lua-loader` が書いた。この spec は書かない。
- **触るファイル**: `code_gen/element_gen.rs` 564・`code_gen/mod.rs` 436・`code_gen/scope_gen.rs` 445 と `code_gen/*_tests.rs`、`normalize.rs` 463・`config.rs` 164・`context.rs` 454・`transpiler.rs` 339・`transpiler_tests.rs`・`lib.rs` 69、`tests/transpiler/`、`tests/fixtures/sample.generated.lua`、`book/src/internals/transpiler.md` 99〜102 行。1,000 行に近いものは無い。新しいファイルは無い見込み。
- **規模**: 8〜10 タスク。
- **先に要るもの**: 機能の上では無い。ponytail のほかの 7 本とはソースが重ならない（`ponytail-core-registry` とは上の `merge_from` の順だけ）。`element_gen.rs` は 1 ウェーブに 1 本の決まり。0.5.0 以降の `scene-attribute-store`・`cue-command-runtime`（`scope_gen.rs`・`context.rs`・`transpiler.rs`）、`scene-condition-attributes`・`scene-exact-match`（`context.rs` か `transpiler.rs` と `element_gen.rs`）、`scene-pick-modes`・`scene-anchor-link`・`hello-novel-sample`・`novel-talk-flow`・`novel-variable-scope`（`element_gen.rs` か `scope_gen.rs`）とは同じウェーブに置かない。
- **種別**: バグ（Lua ブロックの長い文字列に空白が入り、空行が消える）。残りは効かない設定と死んだ状態の整理。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**: `context.rs` 117 行の説明「Used by PastaLoader to combine contexts from multiple files.」は古い（ローダはもう呼ばない）。Adjacent の「`ponytail-lua-loader` と `lib.rs` の再輸出を共有する」は、あちらが入ったので済んだ。

## 申し送り（2026-11-02、`ponytail-dsl-parser` の要件より）

- **`＄２５５` で番号があふれる（バグ）**: `code_gen/element_gen.rs` 40 行付近の `format!("args[{}]", index + 1)` は、`index` が `u8` なので、`＄２５５` で debug はパニック、release は `args[0]`（常に値なし）になる。マニュアル（`grammar/variables.md` 93 行）は 0〜255 を使えると書いている。`u32::from(*index) + 1` などで直る。読んで確かめた（実行はしていない）。`ponytail-dsl-parser` は `＄２５６` 以上を構文の誤りにする（2026-11-02 確定）ので、届く番号は 0〜255 のまま。
- **大きな数の `!select`（小）**: `code_gen/scope_gen.rs` 424〜430 行は `!select` の小数の引数を `(*f as i64)` で整数にする。`ponytail-dsl-parser` が「i64 に収まらない整数は、いちばん近い小数」にする（2026-11-02 確定）と、`!select(10000000000000000000)` が `9223372036854775807` に飽和して出る。害は小さい。
- **span の値は変わらない**: `ponytail-dsl-parser` は解析を速くするが、AST の span の値は今と同じに保つ（要件 1.3）。ソースの対応表は直さずに済む。
