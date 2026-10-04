# Brief: dsl-literal-fixes

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 1（バグ修正・最優先）。着手するときは `/kiro-start dsl-literal-fixes` で開始する。

## Problem

Pasta DSL の文字列リテラルと単語定義の値の解析に、3 つの不具合がある。いずれもマニュアルの規範記述（「`#` から行末まではコメント」「引用符で囲まれた中身が文字列」）と食い違う。

- **U24（起動不能・最重要）**: 改行を含む引用文字列（`「a`＋改行＋`b」`）や、1 ファイルに 2 つ以上書いた `""` で、生成 Lua が構文エラーになり、`pasta.scene_dic` のロードが失敗してゴーストのランタイム全体が起動しない。パースもトランスパイルも通るため、作者は原因に気づけない。
- **U25**: 単語定義の値に書いた `「」`・`""` が空文字列にならず、囲み文字そのもの（`「」`・`""` の 2 文字）が候補の値になる。式の中の `「」` は空文字列になるため、同じリテラルが場所で意味を変える。
- **U06**: 引用なしの単語値の後ろの行末コメント（`＠w：あ、い ＃c`）がコメントにならず、最後の値 `い ＃c` に取り込まれる。

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（`.kiro/specs/completed/manual-ssot-authority/absorption-ledger.md` の付録「未記載の実装事実」U06・U24・U25、食い違い grep 記録 D07）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。

- **U24**:
  - 文法: `crates/pasta_dsl/src/parser/grammar.pest` の `string_contents = @{ (!PEEK ~ ANY)+ }`（123 行付近）が改行をまたぐ。`string_literal` は `slfence_en = PUSH("\""+)`（138 行付近）を `string_blank`（118 行付近）より先に試すため、最初の `""` が囲みの開始になり、次の `""` まで（改行を含めて）取り込む。
  - 出力: `crates/pasta_lua/src/string_literalizer.rs` の `needs_long_string`（58–59 行付近）は `\` と `"` だけを見る。改行を含む値も `"…"` で出力され、Lua の `unfinished string` になる。
  - `choice_label` も `string_contents` を使う。
- **U25**: `crates/pasta_dsl/src/parser/parse_elements.rs` の単語値（64–66 行付近）は `Rule::string_blank` の字面 `as_str()` をそのまま値にする。属性値の `parse_attr`（21 行付近）も同じ。式側（`parse_action.rs`）は `Expr::BlankString` で空になる。
- **U06**: `grammar.pest` の `word_nofenced = @{ (!(comma_sep | "\r" | "\n") ~ ANY)+ }`（154 行付近）が `comment_marker` を除外しない。同じ引用なしの値の `attr_string`（165 行付近）は除外している（前例あり）。
- **マニュアル**: 上の不具合を避けて書いている（不具合自体は書いていない）。
  - `book/src/grammar/block-structure.md` 275–283 行付近は、行末コメントを「値を「」または"で囲んだ単語定義」に限って書く。
  - 空文字列の例は `「」` だけで書く（`""` は 2 つ目で U24 に当たるため）。
  - 単語値の空文字列の書き方は載せていない。
  - 引用文字列が改行を含められるとは書いていない。

## Desired Outcome

- 引用文字列を含むどんな `.pasta` も、生成 Lua の構文エラーでランタイムの初期化を壊さない。改行を含む引用文字列は、パースエラー（行番号付き）になるか、改行を含む文字列として正しく動くかのどちらかに確定している。
- `""` を 1 ファイルに何度書いても、それぞれが空文字列になる。
- 単語値・属性値の `「」`・`""` が空文字列になる（または、空の候補を許さないと決めて、明示的なエラーか警告になる）。
- 引用なしの単語値の後ろの `＃…` が行末コメントになる。
- マニュアル（`grammar/literals.md`・`grammar/words.md`・`grammar/block-structure.md`）が新しい挙動を書き、`node book/tools/gen-skill-refs.mjs` でスキル `references/` を再生成している。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **改行を含む引用文字列**: 禁止（パースエラー）か、許可（改行入りの文字列）か。
  - 推奨は禁止。`string_contents` から改行を除けば、行末の `""` は `string_blank` に落ち、「1 ファイルに `""` を 2 つ」も同時に直る。
  - あわせて `string_literalizer.rs` で `\r`・`\n` をエスケープする（多層防御）。長い文字列 `[[…]]` に切り替える場合は、Lua が `[[` 直後の改行を捨てるため、先頭に改行を足す必要がある。
  - 1 行の中の `""…""` は可変長フェンスと本質的に曖昧になる。仕様上の注記として残す。
- **単語値の空文字列**: 空の候補を許すか。許すなら `string_blank` を `String::new()` にする（属性値の `parse_attr` も同じ）。U24 の文法修正が前提になる（修正しないと 2 つ目の `""` がフェンスとして消費される）。
- **引用なしの単語値の `#`・`＃`**: コメントとして扱うと、引用なしの値に `#` を含められなくなる（引用すれば書ける）。この非互換を受け入れるか。`attr_string` と同じ除外にそろえるのが推奨。

## Scope

- **In**:
  - U24・U25・U06 の修正（`pasta_dsl` の文法・パーサ、`pasta_lua` の `string_literalizer.rs`）
  - 修正を固定するテスト（`pasta_dsl` のパーサテスト、`pasta_lua` のトランスパイル・スナップショット、改行入り文字列のロード）
  - マニュアルの該当章の更新とスキル `references/` の再生成
  - VSCode 拡張の TextMate 文法（ハイライト）に影響がある場合はその追従
- **Out**:
  - アクション行の `\\` エスケープ（U08）。コード生成側の修正のため `dsl-codegen-runtime-safety` が持つ。
  - 括弧式の中の演算（U12）。棚卸の即時修正で修正済み。
  - さくらスクリプトの `\]`・`\%`（R4・保留）

## Boundary Candidates

- 文法（`grammar.pest`）: `string_contents`・`string_literal` のフェンス優先順位・`word_nofenced`
- パーサ（`parse_elements.rs`）: 単語値・属性値の `string_blank`
- 出力（`string_literalizer.rs`）: 改行の扱い
- マニュアル（文法章）・生成スキル・ハイライト文法

## Out of Boundary

- `element_gen.rs`（コード生成）の変更。Wave 1 で `dsl-codegen-runtime-safety` が持つ。
- 単語検索・選択の挙動

## Upstream / Downstream

- **Upstream**: `manual-ssot-authority`（吸収台帳の照合記録 U06・U24・U25・D07）
- **Downstream**: `scene-attribute-store`（属性値の型解釈で、空文字列の属性値の扱いを前提にする）

## Existing Spec Touchpoints

- **Adjacent**: `dsl-codegen-runtime-safety`（同じ Wave。`element_gen.rs` を持つ）、`pasta-manual-syntax-highlight`（TextMate 文法の再利用）

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する（`--check` が CI で鮮度を見る）。
- 現行実装を正として設計する。吸収台帳の記述は材料であり規範ではない。
- 並走条件（Wave 1）: 編集するソースは `crates/pasta_dsl/src/parser/`（`grammar.pest`・`parse_elements.rs`）と `crates/pasta_lua/src/string_literalizer.rs` に限る。`element_gen.rs`・`act.lua` は同じ Wave の別 spec が持つため触らない。スナップショットの更新は、この spec が足す・変える入力に関わるものに限る。
