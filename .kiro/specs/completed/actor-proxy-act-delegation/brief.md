# Brief: actor-proxy-act-delegation

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 2（バグ修正）。着手するときは `/kiro-start actor-proxy-act-delegation` で開始する。

## Problem

アクション行（`さくら：…`）の中で関数を呼ぶと、名前の解決はアクタープロキシ経由で行われ、見つかった関数にはプロキシが第 1 引数として渡る。ところが、ランタイムが用意する関数の一部は ACT（`act`）を前提にしており、プロキシを渡されるとエラーになる。

- アクション行の `＠yield`・`＠ゴースト終了`、値が `yield` の `＠＄x` などが、実行時エラー（500）になる。

## Current State

照合記録は `pasta-runtime-internals-doc` の吸収台帳付録 B（「アクター付きの名前の解決で ACT を前提とする関数にプロキシが渡る」）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。パスは `crates/pasta_lua/pasta_scripts/pasta/` 基準。

- `actor.lua` 160–168 行付近の `PROXY_IMPL.find_handler` は、act の 3 段目（act のメソッド）・4 段目（`GLOBAL`）へ委ねる。見つかった関数は 179 行付近の `handler(self, …)`・236 行付近の `handler(self)` でプロキシ（`self`）を受け取る。
- ACT を前提とする関数:
  - `global.lua` 21 行付近の `GLOBAL.yield(act)`。
  - `shiori/entry.lua` 17 行付近の `close_ghost`。`act:yield`・`act:wait`・`act:raw_script` を呼ぶ。
  - `act.lua` 467 行付近の `ACT_IMPL.yield` は `self:build()` を呼ぶ。
- マニュアル `book/src/grammar/variables.md#関数スコープの展開先` は「アクション行の関数はプロキシを受け取る」と書いている。`GLOBAL` の関数が受け取るものを変えると、作者に見える変更になる。
- プロキシの `__index` で act へ委ねる案は、抜けがある（`get_property` が `self.token` に代入するなど）ため推奨しない（棚卸の調査結果）。

## Desired Outcome

- アクション行から呼んだランタイムの関数（`yield`・`チェイントーク`・`ゴースト終了` など）が、行内からでも正しく動く。500 にならない。
- 作者が `GLOBAL` に登録した関数がアクション行から呼ばれたときに何を受け取るかが、1 つの規則で決まり、マニュアルに書かれている。

## Approach

要件フェーズで次を決める。

- **委ね方**: 次の 2 案から選ぶ。
  - 案 1: 見つかった場所で渡すものを変える。act 段（3・4 段目）で見つかったら `self.act` を、アクター段（A1・A2）で見つかったらプロキシを渡す。
  - 案 2: ランタイムの組み込み関数（`yield`・`close_ghost` など）の側で、プロキシを受けたら ACT に正規化する。
- **作者の関数**: `GLOBAL` に登録した作者の関数がアクション行から呼ばれたとき、プロキシを受けるか ACT を受けるか。現行のマニュアルの記述（プロキシ）を保つかどうか。
- `dsl-codegen-runtime-safety` が足すプロキシ取得口（アクション行の `act.名前` を置き換える口）を前提にする。

## Scope

- **In**:
  - プロキシ経由で見つかった関数に渡す引数の規則の確定と実装
  - ランタイムの組み込み関数のアクション行からの呼び出しの修正
  - 修正を固定するテスト（アクション行の `＠yield`・`＠ゴースト終了`・`＠＄x` が 500 にならない）
  - マニュアル（`grammar/variables.md#関数スコープの展開先`・`lua/script-api.md#アクタープロキシ`・`internals/internal-modules.md`）の更新とスキル `references/` の再生成
- **Out**:
  - アクター単語の検索キー（`scene-search-key-normalization`）
  - アクション行のコード生成（`dsl-codegen-runtime-safety`）

## Boundary Candidates

- アクタープロキシ（`actor.lua` の `PROXY_IMPL`）
- ランタイムの組み込み関数（`global.lua`・`shiori/entry.lua`）
- マニュアル・生成スキル

## Out of Boundary

- `act.lua`（Wave 2 では `act-token-grouping-fix` が持つ。`ACT_IMPL.yield` の中を変える必要が出たら、その spec と順序を調整する）

## Upstream / Downstream

- **Upstream**:
  - `dsl-codegen-runtime-safety`（Wave 1。アクション行のプロキシ取得口）。
  - `scene-search-key-normalization`（Wave 1。`actor.lua` を先に触る）。
- **Downstream**: なし

## Existing Spec Touchpoints

- **Adjacent**: `actor-word-dictionary`・`actor-spot-refactoring`（完了。プロキシの元の設計）、`dynamic-word-reference`（完了。`＠＄x` の解決規則）

## Constraints

- マニュアルが文法・API の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- 並走条件（Wave 2）: 編集するソースは `actor.lua`・`global.lua`・`shiori/entry.lua` に限る。
