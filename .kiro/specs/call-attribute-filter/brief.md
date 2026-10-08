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
- 申し送り（`call-execution-correctness` より、2026-10-05）: Call の実行経路が次のように変わった。「実行時の経路」と「候補なし」の論点はこれを前提に決め直す。詳細は `.kiro/specs/completed/call-execution-correctness/design.md` の CallCodeGen・ActCall・DynamicCallKey。
  - 生成形が 4 通りになった。末尾の Call は `return act:call(SCENE.__global_name__, <キー>, {}, <引数>)`、途中の Call は `act:call_restore(…同じ並び…)`。`attrs` はどちらも第 3 引数で、`call_restore` は `act:call` へそのまま渡す。フィルターを出すときは両方の形に同じ位置で出せる。
  - 動的コールのキーは `act:call_key(値, 変数の経路, 説明)` が判定する。値が nil・空文字列・使えない型なら「呼ばない」印を返し、`act:call` は検索の前（フィルターより前）に nil を返す。
  - ターゲットが見つからないとき、`act:call` は現行の警告に加えて失敗表記 `【Call失敗：「名前」が見つからない】` を出して次の行へ進むようになった（同 spec の要件 4.6）。「一致する候補が無いとき」をこれと同じ扱いにするか、フィルター条件を表記に含めるかを決める。

## 2026-10-07 棚卸の再測定（main 2cbaf510）

- **前提の変化**: `scene-search-key-normalization`・`call-execution-correctness`・`string-concat-operator` は完了した（申し送りは上の Constraints のとおりで、現行の main と一致する）。未完了の前提は `scene-attribute-store` だけ。
- **触るファイル**: `pasta_dsl` の `parser/grammar.pest`（272 行。`call_scene` 170 行）・`parse_action.rs`（456 行）・AST、`pasta_lua` の `code_gen/element_gen.rs`（555 行。Call の生成は 196 行付近）、`pasta_scripts/pasta/act.lua`（774 行。`call`・`find_act_handler`）、`scene.lua`（`SCENE.search`）、`search/context.rs`（614 行）・`search/mod.rs`、`pasta_core` の `registry/scene_table.rs`（434 行）、`editors/vscode/syntaxes/pasta.tmLanguage.json`、マニュアル `grammar/call-jump.md`・`lua/script-api.md`・`internals/registry-search.md` と生成スキル。`act.lua` は 774 行で、1,000 行に近づいている。
- **規模**: 約 16〜19 タスク（文法・パーサ・AST 3、生成 2、`act.lua`→`scene.lua`→`search_scene` の受け渡し 3、`pasta_core` の型付き比較と候補キャッシュのキー 3、候補なしの扱い 1、ハイライト 1、マニュアルと生成 2、テスト 2〜3）。上限の 20 に近いが、一度分割した spec なので再分割はしない。
- **先に要るもの**: `scene-attribute-store`（保持と型の解釈）。ファイルの重なり: `failure-output-unification`（`act.lua`）、`scene-attribute-store`（`scene.lua`・`pasta_core` の registry）。どちらとも同じウェーブに置けない。Phase 12・`release-ci` とはソースが重ならない（マニュアルは別の節）。
- **種別**: 機能（現在 `＞X＆k＝v` はパースエラー）。
- **要件定義のモデル**: Fable（新しい構文と意味。式の `＆`（連結）との切り分け、比較演算子、AND/OR の決定がある）。
- **分割の案**: なし。
- **見つけた穴・古くなった記述**:
  - `attrs` は `act:call` の中で捨てられる。`self:find_handler("scene", key)`（`act.lua` 670 行）は `attrs` を受けず、`search_dictionary` の `SCENE.search(key, scene_name)`（307 行）も渡さない。`SCENE.search` は `attrs` を受けるが `SEARCH:search_scene(name, global_scene_name)`（`scene.lua` 160 行）に渡さず、`search_scene` にはフィルターの引数が無い（`context.rs` 73〜78 行）。Current State の「途中で `attrs` 引数は受け取る」は、`act:call` の入口までの話である。
  - `find_act_handler` の 5 段の検索のうち、L1（実行中のシーン表の完全一致）・L3（`act` のメソッド）・L4（`GLOBAL`）はシーンの登録表を通らないので、属性で絞り込めない。フィルター付きの Call がこれらに当たったときの扱い（L2・L5 だけを探すか）を要件で決める。
  - 動的コールでは `act:call_key` の「呼ばない」印が先に返るので、フィルターは評価されない（申し送りどおり）。
  - `register_global_raw` がグローバルの属性をローカルシーンに複製する件は `scene-attribute-store` の brief に記した。その結論が、ローカルシーンを絞り込むときの前提になる。
  - マニュアル `lua/script-api.md` 304・317 行は「`attrs` は使わない」と書いている。本 spec で書き換える対象。
- **順序の提案**: 本 spec を `failure-output-unification` の後に置く。そうすれば「一致する候補が無い」の失敗表記を、最初から一本化した仕組みに載せられる（現在のロードマップは逆順）。
