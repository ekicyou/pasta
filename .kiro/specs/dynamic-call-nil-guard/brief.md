# Brief: dynamic-call-nil-guard

> **ステータス**: 未着手（`dynamic-word-reference` のギャップ分析 RN-4 で発見・2026-10-02 起票）。優先度は低い。着手するときは `/kiro-start dynamic-call-nil-guard` で開始する。

## Problem

動的コール `＞式` の値が nil のとき（代入していない変数の参照、値を返さない関数など）、`dynamic-call-variable` が定めた nil ガード（R3-AC5「シーン検索を行わずに早期リターンし、警告ログを出力する」）が働かない。トランスパイラが式を `tostring(<式>)` で包んでから `act:call` に渡すため、nil は文字列 `"nil"` になり、`ACT_IMPL.call` の `if key == nil` に届かない。結果として `"nil"` という名前のシーンを探し、「見つからない」警告（`handler not found: key='nil'`）になる。

作者から見ると、警告に参照した変数名が出ず、「`nil` というシーンが無い」という紛らわしい文言になる。`ACT_IMPL.call` の nil ガードは、DSL から到達できない死んだコードになっている。

## Current State

- 生成コード: `act:call(SCENE.__global_name__, tostring(<式>), {}, …)`（`crates/pasta_lua/src/code_gen/element_gen.rs` の `CallTarget::Dynamic` アーム、202–208 行付近）。
- ランタイム: `ACT_IMPL.call` の先頭に `if key == nil then log.warn("act:call - nil key (undefined variable?), skipping scene search") return nil end`（`crates/pasta_lua/pasta_scripts/pasta/act.lua` 458–463 行付近）。
- **マニュアルは現行の挙動を書いている**: `book/src/grammar/call-jump.md` 191 行付近は「値が `nil` になる場合は文字列 `"nil"` を検索キーにして検索する。通常はそのような名前のシーンが無いため、『見つからない』場合と同じく警告をログに出し、何も出力せずに次の行へ進む」と書く。マニュアルは文法の唯一の権威のため、現状は「仕様どおり」とも読める。
- 元の要件: `.kiro/specs/completed/dynamic-call-variable/requirements.md` R3-AC5（nil なら早期リターン＋警告）。マニュアルと食い違っている。
- 実行が止まることはない（どちらの経路でも警告＋何も出力せず次の行へ進む）。違いは警告の文言と、`"nil"` という名前のシーンが実在した場合にそれが呼ばれることだけ。

## Desired Outcome

次のどちらかに決め、コード・マニュアル・テストを揃える。

- **ガードを生かす**: nil の値はシーン検索をせず、nil であることが分かる警告（できれば参照した変数のパス付き）を出す。マニュアル `call-jump.md` の該当記述を新しい挙動に書き換え、スキル `references/` を `node book/tools/gen-skill-refs.mjs` で再生成する。
- **現行を正とする**: マニュアルの記述を仕様として確定し、到達できない `ACT_IMPL.call` の nil ガード（Lua から直接 `act:call(…, nil)` を呼ぶ場合のために残すかも含めて）を整理する。

## Approach

要件フェーズで決める（未決定）。

- 上の 2 方向のどちらにするか。
- ガードを生かす場合の文字列化の位置: 生成コードで nil 以外だけを `tostring` するか、`ACT_IMPL.call` 側で nil 判定の後に `tostring` するか。
- 警告に参照変数のパスを含めるか（変数展開の `act:talk(value, "var.x")` と同じ型で、変数パスを渡す経路を足すか）。式が変数参照でない場合（関数呼び出し・算術）の文言。
- `dynamic-word-reference`（動的単語参照 `＠＄変数名`）の未代入時の警告と、考え方・文言を揃えるか。

## Scope

- **In**:
  - 動的コール `＞式` の値が nil のときの挙動の確定と実装
  - マニュアル `grammar/call-jump.md` の更新とスキル `references/` の再生成
  - トランスパイラのスナップショット（`crates/pasta_lua/tests/transpiler/snapshot_test.rs`・`dynamic_call_test.rs`）と、nil 時の警告を固定するテスト
- **Out**:
  - 値が nil 以外のとき（文字列・数値など）の動的コールの挙動
  - シーン検索アルゴリズムの変更
  - 静的なシーン名の Call

## Boundary Candidates

- トランスパイラ（`pasta_lua` code_gen の `CallTarget::Dynamic`）
- ランタイム（`act.lua` の `ACT_IMPL.call`）
- マニュアル・生成スキル

## Upstream / Downstream

- **Upstream**: `dynamic-call-variable`（完了済み。nil ガードの元の要件）
- **Downstream**: なし

## Existing Spec Touchpoints

- **Adjacent**: `dynamic-word-reference`（未代入時の警告の考え方を揃える。同 spec の `research.md` RN-4 が発見元）、`manual-ssot-authority`（マニュアルの権威とスキル生成の仕組み）

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたらマニュアルを同じ変更で更新し、生成スキルを再生成する（`--check` が CI で鮮度を見る）。
- 現行実装を正として設計する。`dynamic-call-variable` の R3-AC5 は材料であり規範ではない。
