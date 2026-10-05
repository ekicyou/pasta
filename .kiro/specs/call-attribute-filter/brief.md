# Brief: call-attribute-filter

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で `scene-attribute-semantics` から分割して起票）。Wave 5（機能拡張）。着手するときは `/kiro-start call-attribute-filter` で開始する。

## Problem

Call の属性フィルター（`＞シーン＆k＝v`）は、旧文法仕様（ch04 §4.2・ch12 §12.5）とマニュアル旧版で「将来予約」として定義されていたが、文法として受理されない（パースエラー）。シーンに属性を付けても、それで候補を絞り込む手段が無い。`scene-attribute-store` が属性を実行時に保持し型解釈を決めた後に、それを使う絞り込みを足す。

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（仕分け表 B1、食い違い grep 記録 D05）と 2026-10-04 の棚卸の再照合による。旧仕様の原文の要旨は `.kiro/specs/scene-attribute-store/brief.md` の「吸収元の内容」節にある（ch04 §4.2 フィルター・§12.5・§12.10・マニュアル旧版 `call-jump.md` の「フィルター」）。

- **構文（受理されない）**: `call_scene = { call_marker ~ (id | call_target_expr) ~ s ~ args? }` に `＆` を受ける規則が無く、`＞X＆k＝v` はパースエラー（`expected args`）。単語参照 `word_ref` も名前だけを受ける。式に比較演算子は無い（X04）。
- **実行時の経路**: `ACT_IMPL.call` → `find_handler` → `SCENE.search` → `search_scene` と進む。
  - 途中で `attrs` 引数は受け取るが、使われないと文書化されている。
  - `search_scene`（`crates/pasta_lua/src/search/context.rs` 73 行付近）は常に空の `filters` で検索する。
- **絞り込み**: `pasta_core` の `scene_table.rs` の `filter_by_attributes` は、`HashMap<String,String>` の文字列一致・AND だけ。比較演算子と型付きの値には新しいフィルター型が要る。候補キャッシュのキーにはフィルターが含まれる。
- **マニュアル**: `grammar/call-jump.md` の「フィルター」節は削除済み。

## Desired Outcome

- Call の属性フィルター構文が文法として受理され、実行時に候補を絞り込む。比較演算子と複数条件の結合（AND/OR）の規則が決まっている。
- 一致する候補が無いときの挙動が決まっている。前方一致・シャッフル消費（候補キャッシュ）と矛盾しない。
- マニュアル（`grammar/call-jump.md` ほか）が新しい構文と挙動を書き、スキル `references/` を再生成している。VSCode 拡張の TextMate 文法が新しい構文をハイライトする。

## Approach

要件フェーズで次を決める（未決定）。

- **構文**: `＞シーン＆k＝v` の文法。
  - 全角・半角の扱い。
  - `call_target_expr`（動的 Call）との組み合わせ。
  - 引数リストとの順序。
- **比較**: 比較演算子の種類（`＝`・`＞`・`＜` など）と、値の型解釈（`scene-attribute-store` が決めたもの）に基づく数値比較と文字列比較。
- **結合**: 複数条件の結合（AND/OR）。
- **候補なし**: 一致する候補が無いときの挙動（「見つからない」警告と同じか）。
- **単語参照へのフィルター**: §4.2 の `＠単語名＆category＝food` は、単語に属性を付ける構文が無いため対象外とし、バックログに置く（推奨）。
- **実行時の経路**: コード生成が `act:call` の `attrs` 引数にフィルターを出力し、`ACT_IMPL.call` → `find_handler` → `SCENE.search` → `search_scene` へ通す。

## Scope

- **In**:
  - Call の属性フィルター構文（文法・パーサ・AST）
  - コード生成と、`act.lua`・`scene.lua`・`search/context.rs` を通る受け渡し
  - `pasta_core` の型付き比較フィルターと候補キャッシュのキー
  - 修正を固定するテスト、マニュアル章の更新とスキル `references/` の再生成、TextMate 文法の追従
- **Out**:
  - 属性の保持・継承・読み出し（`scene-attribute-store`）
  - 単語参照への属性フィルター（バックログ）
  - 属性と無関係な Call・単語検索の挙動変更

## Boundary Candidates

- 文法（`grammar.pest`）とパーサ（`pasta_dsl`）: フィルター構文
- コード生成（`element_gen.rs` の Call の出力）
- ランタイム（`act.lua` の `call`・`find_handler`、`scene.lua` の `SCENE.search`、`search/context.rs`、`pasta_core` の `scene_table`）
- マニュアル・生成スキル・ハイライト文法

## Out of Boundary

- マニュアル権威化とスキル生成の仕組み（`manual-ssot-authority` が提供）

## Upstream / Downstream

- **Upstream**:
  - `scene-attribute-store`（Wave 4。属性の実行時の保持と型解釈）。
  - `scene-search-key-normalization`（Wave 1。`search_scene` の入口）。
  - `call-execution-correctness`（Wave 3。Call のコード生成と `ACT_IMPL.call`）。
- **Downstream**: なし（属性を使うゴースト）

## Existing Spec Touchpoints

- **Adjacent**: `manual-ssot-authority`（吸収台帳 D05）、`pasta-manual-syntax-highlight`（TextMate 文法の再利用）

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- 現在 `＞X＆k＝v` はパースエラーのため、フィルター構文を足しても既存の辞書に非互換は生まれない。
- 現行実装を正として設計する。旧仕様の記述は材料であり規範ではない。
- 並走条件（Wave 5）: Wave 5 はこの spec だけ。
- `string-concat-operator` からの申し送り: 式の `＆` は連結演算子になった（`＞＄種類＆「_挨拶」` は受理される）。式の `＆` の後は項の開始文字（`＄`・`＠`・数字・`－`・文字列の開き・括弧の開き）だけ。フィルターのキーは識別子で始める。`＞＄名前＆k＝v`・`＞シーン名＆k＝v`・`＄x＝＠単語＆k＝v` は `string-concat-operator` の後もパースエラー（マニュアル `grammar/call-jump.md`・`grammar/variables.md` の「連結の評価」）。
