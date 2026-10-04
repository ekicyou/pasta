# Brief: string-concat-operator

> **ステータス**: 未着手（`dsl-codegen-runtime-safety` の要件ディスカッション議題 4 で起票、2026-10-04）。Phase 11 Wave 2（機能）。着手するときは `/kiro-start string-concat-operator` で開始する。
>
> **前提**: `dsl-codegen-runtime-safety` が完成した状態を前提とする。同 spec は `＋` を数値専用のまま維持し（R3.5）、算術を実行時ヘルパー経由・失敗時は警告＋値なしにする。本 spec はその成果物の上に連結を足し、必要な調整（ヘルパー・生成形・マニュアル）は本 spec 側で行う。`dsl-codegen-runtime-safety` には遡って手を入れない。

## Problem

Pasta DSL の式には文字列を連結する演算子が無い（マニュアル `grammar/variables.md` 153 行付近「文字列を連結する演算子は無い。`＋` などはすべて数値の演算である」）。変数に「合計 3 個」のような組み立てた文字列を入れたいゴースト作者は、Lua ブロックに降りるしかない。アクション行なら `＄姓＄名` と並べて話せるが、代入の右辺・関数の引数・Call の引数では連結できない。

`＋` を文字列どうしで連結にする案は採らない。`「1」＋「2」` が 3 か `"12"` かが値しだいで変わる曖昧さを生むため（Lua が `+` と `..` を分けている理由と同じ）。

## Current State

- 文法: `crates/pasta_dsl/src/parser/grammar.pest` の `bin_op = _{ add_op | sub_op | mul_op | div_op | modulo_op }`（62 行付近）。`amp = _{ "＆" | "&" }`（27 行）は現在、行頭の属性マーカー `attr_marker`（47 行）にだけ使われている。
- コード生成: `crates/pasta_lua/src/code_gen/element_gen.rs` の `generate_expr_to_buffer` の Binary アーム。`dsl-codegen-runtime-safety` の後は、算術が存在確認付きの実行時ヘルパー経由（失敗時は警告＋値なし）になっている見込み（形は同 spec の設計で確定する）。
- マニュアル: `grammar/variables.md`（演算子の表・Lua 展開の表・153 行付近の注記）。

## Desired Outcome

- 式の中で `＆`（全角・半角 `&` の両方）が文字列の連結になる。例: `＄表示＝「合計」＆＄n＆「個」`。
- 数値は文字列にしてから連結する。
- 優先順位は算術（`＋`・`－`・`＊`・`／`・`％`）より低い（VB・Excel と同じ）。`「合計」＆＄a＋＄b` は「合計」と（a＋b）の連結。
- 連結できない被演算子（値なしなど）の扱いを決め、500 にならない（`dsl-codegen-runtime-safety` の「警告＋値なし」の考え方にそろえるかを要件で決める）。
- マニュアル（`grammar/variables.md` ほか演算子・生成コードを書く章）を更新し、スキル `references/` を再生成する。VSCode 拡張・マニュアルのシンタックスハイライトが `＆` 演算子を扱えるかも確認する。

## Approach

- 文字は `＆`／`&` で確定（議題 4 の決定。Excel・VB の連結演算子で、ゴースト作者に馴染みがある）。
- `pasta_dsl` の文法に連結の演算子と優先順位の段を足す。AST の Binary に連結の演算子を足すか、別ノードにするかは設計で決める。
- コード生成は `dsl-codegen-runtime-safety` の算術ヘルパーと同じ流儀で、実行時ヘルパー経由（存在確認・警告付き）にするのが第一候補。

## Scope

- **In**:
  - 式の中の `＆` 連結（文法・AST・コード生成・実行時ヘルパー）
  - 優先順位（算術より低い）と括弧との組み合わせ
  - 被演算子の型ごとの扱い（文字列・数値・値なし・真偽値）
  - テスト（パーサ・スナップショット・Lua 実行・SHIORI 経由で 500 にならないこと）
  - マニュアル・スキル `references/`・シンタックスハイライトの更新
- **Out**:
  - `＋` の意味の変更（数値専用のまま）
  - 比較演算子・論理演算子など、ほかの演算子の追加
  - Call の属性フィルター `＞シーン＆k＝v`（`call-attribute-filter`）。ただし下記「Constraints」の構文上の接点は本 spec で扱う

## Boundary Candidates

- 文法・AST（`pasta_dsl`）: 演算子と優先順位の段
- コード生成（`element_gen.rs` の式生成）
- ランタイム（`act.lua` などの連結ヘルパー。置き場所は `dsl-codegen-runtime-safety` の算術ヘルパーに合わせる）
- マニュアル・生成スキル・シンタックスハイライト

## Out of Boundary

- 属性の文法（行頭 `＆k＝v`）と属性の実行時保持（`scene-attribute-store`）
- Call の属性フィルターの意味論（`call-attribute-filter`）

## Upstream / Downstream

- **Upstream**:
  - `dsl-codegen-runtime-safety`（算術ヘルパーと「警告＋値なし」の生成形。完成を前提とする）
  - `dsl-literal-fixes`（Wave 1 で `pasta_dsl` の文法・パーサを持つ。その後に文法を触る）
- **Downstream**:
  - `call-attribute-filter`（`＆` を Call 行で使う。本 spec が先に式の `＆` を入れるため、フィルター構文との切り分けを前提として引き継ぐ）

## Existing Spec Touchpoints

- **Extends**: なし（新しい文法要素）
- **Adjacent**: `dsl-codegen-runtime-safety`（式の生成・算術ヘルパー）、`scene-attribute-store`・`call-attribute-filter`（`＆` の属性用途）、`act-token-grouping-fix`（Wave 2 で `act.lua` のグループ化の領域を持つ。本 spec は算術・連結ヘルパーの領域だけを触る）

## Constraints

- マニュアルが文法・API の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- **`＆` の構文上の接点**: `call_scene = { call_marker ~ (id | call_target_expr) ~ s ~ args? }` の `call_target_expr` は式であり、本 spec の後は `＞＄名前＆＄x`（動的ターゲットの連結）が式として読める。`call-attribute-filter` の `＞＄名前＆k＝v`（動的ターゲット＋フィルター）と衝突しうる。`＞シーン名＆k＝v`（`id` 分岐）は式に入らないため衝突しない。本 spec の要件・設計で、動的コールのターゲット式における `＆` の扱い（括弧を必須にする・フィルターを優先するなど）を決め、`call-attribute-filter` へ申し送る。
- 単語参照への属性フィルター `＠単語名＆category＝food`（バックログ）とも、`＆` の読みが衝突しないことを確かめる。
- 並走条件（Wave 2）: `act.lua` は `act-token-grouping-fix` と同じウェーブになるため、本 spec は算術・連結ヘルパーの領域だけを触り、グループ化の領域には触れない。`element_gen.rs` は式生成（Binary）だけを触る（Call の生成形は Wave 3 の `call-execution-correctness`）。
