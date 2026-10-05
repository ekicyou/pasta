# Brief: call-execution-correctness

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 3（バグ修正）。着手するときは `/kiro-start call-execution-correctness` で開始する。
>
> 旧 brief `dynamic-call-nil-guard`（2026-10-02 起票）を統合した。両者とも `ACT_IMPL.call` と Call のコード生成を触り、別 spec にすると同じ箇所を順に 2 回直すことになるため。

## Problem

Call（`＞シーン`・`＞式`）の実行に 2 つの不具合がある。

- **Call から戻った後のシーン文脈（U28）**: 別のグローバルシーンへ Call して戻ると、呼び出し元のシーン文脈が復元されない。呼び出し元の以後のローカル単語参照・Call・式の 1・2 段目と、以後の選択肢が、呼ばれた側のシーンで解決される。
- **動的コールの nil ガード（旧 `dynamic-call-nil-guard`）**: `＞式` の値が nil のとき（代入していない変数の参照、値を返さない関数など）、`dynamic-call-variable` が定めた nil ガード（R3-AC5「シーン検索をせずに早期リターンし、警告を出す」）が働かない。
  - トランスパイラが式を `tostring(<式>)` で包むため、nil は文字列 `"nil"` になり、`"nil"` という名前のシーンを探して「見つからない」警告（`handler not found: key='nil'`）になる。
  - 警告に参照した変数名が出ず、紛らわしい。
  - `ACT_IMPL.call` の nil ガードは、DSL から到達できない死んだコードになっている（Lua から直接 `act:call(…, nil)` を呼ぶ場合だけ届く）。

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（U28）、`pasta-runtime-internals-doc` の吸収台帳付録 B（「Call から戻った後のシーン文脈」）、`dynamic-word-reference` の `research.md` RN-4（nil ガードの発見元）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。

- **シーン文脈**:
  - `crates/pasta_lua/pasta_scripts/pasta/act.lua` 180–186 行付近の `init_scene` が、`self.current_scene` と `STORE.last_global_scene` を上書きする。
  - `ACT_IMPL.call`（499–516 行付近）はそれを復元しない。
  - `return handler(self, ...)` は意図した末尾呼び出しで（`element_gen.rs` 206・247 行付近、マニュアル `internals/transpiler.md` 234 行付近）、`call` の中で保存・復元すると末尾呼び出しが壊れる。
  - マニュアル `internals/internal-modules.md` 152 行付近は現行挙動を書く。
- **nil ガード**:
  - 生成コードは `act:call(SCENE.__global_name__, tostring(<式>), {}, …)`（`crates/pasta_lua/src/code_gen/element_gen.rs` の `CallTarget::Dynamic` アーム、237–243 行付近）。`Expr::VarRef` は生の `var.x` を出力する（471–472 行付近）ため、未代入の変数は `tostring(nil)` で `"nil"` になる。
  - ランタイムの `ACT_IMPL.call` 先頭（`act.lua` 499–504 行付近）に `if key == nil then log.warn(…) return nil end` がある。
  - マニュアル `book/src/grammar/call-jump.md` 191 行付近は現行の挙動（`"nil"` を検索キーにする）を書き、193 行付近はガードが Lua からの直接呼び出しだけに効くと書く。マニュアルは文法の唯一の権威のため、現状は「仕様どおり」とも読める。
  - 未定義の変数・単語・関数の参照を空文字で展開する変更（be9bbed7、2026-09-26）は、アクション行の展開と `ACT_IMPL.talk` だけを変え、Call のターゲットは変えていない。
  - 再利用できる口: `dynamic-word-reference` が足した `dynamic_ref_args`（`element_gen.rs` 56 行付近）は、`value, "var.path"` を渡す。警告に変数パスを含める経路として使える。
  - 関連するテスト: `crates/pasta_lua/tests/transpiler/snapshot_test.rs`（`dynamic_call_local_var`・`dynamic_call_binary_expr`）・`dynamic_call_test.rs`・スナップショット `transpiler__final_regression_test__r7_1_off_path__kind_dynamic_call.snap`。

## Desired Outcome

- 別のグローバルシーンへ Call して戻った後、呼び出し元のシーン文脈（ローカル単語・Call・式の解決、以後の選択肢の探索範囲）が呼び出し元のものに戻る。
- 動的コール `＞式` の値が nil のときの挙動が次のどちらかに確定し、コード・マニュアル・テストがそろっている。
  - **ガードを生かす**: シーン検索をせず、nil であることが分かる警告（できれば参照した変数のパス付き）を出す。
  - **現行を正とする**: マニュアルの記述を仕様として確定し、到達できない nil ガードを整理する。
- マニュアル（`grammar/call-jump.md`・`internals/internal-modules.md`・`internals/transpiler.md`）が新しい挙動を書き、スキル `references/` を再生成している。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **シーン文脈の復元の位置**: 推奨は、コード生成が末尾でない `act:call(...)` の後に文脈を戻す呼び出し（`act:init_scene(SCENE)` か、小さな `act:restore_scene(SCENE)`）を出力すること。末尾呼び出しは戻らないため対象外になる。
- **`last_global_scene` も戻すか**: 呼ばれた側が選択肢を出した場合、戻すとその選択肢のルーティングが壊れる。選択肢ごとに探索範囲を記録する案は中規模になる。選択肢の自動ルーティングは棚卸の即時修正でグローバルシーンへのフォールバックを得たため、影響範囲は狭まっている。
- **nil の方向**: ガードを生かすか、現行を正とするか。
  - ガードを生かす場合の文字列化の位置: 生成コードで nil 以外だけを `tostring` するか、`ACT_IMPL.call` 側で nil 判定の後に `tostring` するか。
  - 警告に参照変数のパスを含めるか（`dynamic_ref_args` と同じ型で変数パスを渡す）。式が変数参照でない場合（関数呼び出し・算術）の文言。
  - `dynamic-word-reference`（`＠＄変数名`）の未代入時の警告と、考え方・文言を揃えるか。同 spec は、参照変数の値が文字列・数値以外のときは検索せず空＋型を含む警告にすると決めた（要件ディスカッション #8）。動的コールの型の扱いもそろえるかを合わせて決める。

## Scope

- **In**:
  - Call から戻った後のシーン文脈の復元
  - 動的コールの値が nil のときの挙動の確定と実装
  - トランスパイラのスナップショット（Call を含むもの）の更新と、修正を固定するテスト
  - マニュアルの該当章の更新とスキル `references/` の再生成
- **Out**:
  - 値が nil 以外のときの動的コールの挙動（型の扱いを揃えると決めた場合を除く）
  - シーン検索アルゴリズムの変更
  - Call の属性フィルター（`call-attribute-filter`）

## Boundary Candidates

- コード生成（`element_gen.rs` の Call の出力・`CallTarget::Dynamic`）
- ランタイム（`act.lua` の `init_scene`・`call`）
- マニュアル・生成スキル

## Out of Boundary

- 選択肢の自動ルーティング（`choice_select.lua`。即時修正済み）
- アクション行・式のコード生成（`dsl-codegen-runtime-safety`）

## Upstream / Downstream

- **Upstream**:
  - `dynamic-call-variable`（完了。nil ガードの元の要件）。
  - `dsl-codegen-runtime-safety`（Wave 1）と `act-token-grouping-fix`（Wave 2）。どちらも `element_gen.rs`・`act.lua` を先に触る。
  - `scene-identity-format`（Wave 2。スナップショットを先に広く変える）。
- **Downstream**: `scene-attribute-store`・`call-attribute-filter`（Call のコード生成・`ACT_IMPL.call` をこの spec の後に触る）

## Existing Spec Touchpoints

- **Adjacent**: `dynamic-word-reference`（完了。未代入時の警告の考え方、`dynamic_ref_args`）、`choice-definition-dsl`（完了。選択肢の探索範囲）、`local-scene-act-call`・`act-impl-call`（完了。Call の元の設計）

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する（`--check` が CI で鮮度を見る）。
- 現行実装を正として設計する。`dynamic-call-variable` の R3-AC5 は材料であり規範ではない。
- 末尾呼び出し（`return act:call(…)`）の性質を壊さない。
- 並走条件（Wave 3）: Wave 3 はこの spec だけ。
