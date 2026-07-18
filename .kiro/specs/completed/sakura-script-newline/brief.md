# Brief: sakura-script-newline

## Problem

ゴースト再生時、キャラA→キャラBへ会話が移ると、意図しない1.5行分の改行がバルーンに現れて見える。キャラAでトークが終了する場合はAの最終トークに改行が入らない。

原因は `sakura_builder.lua` の段落区切り改行の**先出し（eager）出力**である。アクター切替を検出した瞬間に、切替先スコープタグ `\p[N]` の**直前**（＝離脱する側のスコープ末尾）へ `\n[150]` を出力するため、生成スクリプトは次の形になる：

```
\p[0]Aのトーク \n[150] \p[1]Bのトーク
```

この `\n[150]` はAのスコープで実行され、Aのバルーン末尾に空行として残る。Aが二度と発言しない場合、この改行は純粋なゴミ出力となる。

なお SSP は「実際に文字がタイプされるまで改行の描画を遅延する」挙動を持つと推定され（ユーザー観察・要検証）、SSP 上では末尾ゴミ改行が可視化されないことがある。ただし出力スクリプトとしては非正規であり、ベースウェアの描画実装に依存せず正しい位置に改行が入るべきである。

## Current State

- 実装: `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` の `emit_actor_switch()`
  - 条件: `allow_break（text_since_break） and last_spot ~= nil and last_spot ~= spot` のとき `\n[N]`（N = `spot_newlines * 100`、デフォルト 1.5 → 150）を出力し、その後 `\p[spot]` を出力
- 先行修正 #21（9ac91f82、main マージ済み）: 直前の手番が全てさくらスクリプト（一般文字列ゼロ）の場合に改行を抑制する `text_since_break` フラグを導入。ただし出力位置（\p の直前）は変わっていない
- テスト: `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua` および `crates/pasta_lua/tests/sakura_script/*.rs` が現行の先出し順序を仕様として固定
- 参考（里々の慣習）: 里々も「＄スコープ切り換え時 = \n[half]」を切替タグの**前**に挿入する同型設計。SSP の描画遅延に依存して破綻を回避していると推定される

## Desired Outcome

段落区切り改行を**遅延（lazy）出力**に変更する：

- アクター切替時は `\p[spot]` のみを出力し、離脱側スコープ末尾に改行を残さない
- 切替**先**のスコープに、同一ビルド内で既に一般文字列が出力済みの場合に限り、`\p[spot]` の**後**に `\n[N]` を出力する（＝「戻ってきた側」の段落先頭に改行が入る）
- 生成スクリプト例（A→B→A）：
  - 現行: `\p[0]A1 \n[150] \p[1]B1 \n[150] \p[0]A2`
  - 修正後: `\p[0]A1 \p[1]B1 \p[0] \n[150] A2`
- A→B で終了しても、どちらのバルーンにも余分な改行が残らない

## Approach

遅延出力方式（ユーザー選択済み）。`BUILDER.build()` にスポットごとの「一般文字列出力済み」状態（per-spot has-text マップ）を導入し、改行の出力判断を切替先スコープ側で行う。#21 の `text_since_break` によるグローバル抑制は per-spot 追跡に統合・包摂できるか設計時に検討する。

却下した代替案：
- 現状維持（SSP の描画遅延に依存）— 出力スクリプトが非正規のままで、他ベースウェア・将来の描画仕様変更に脆弱
- `spot_newlines = 0` による回避 — 段落区切り機能自体が失われる

## Scope

- **In**:
  - `sakura_builder.lua` の改行出力ロジック変更（eager → lazy）
  - per-spot テキスト出力状態の追跡
  - 既存テスト（`sakura_builder_test.lua`、`tests/sakura_script/*.rs`）の期待値更新と新規ケース追加（A→B終了、A→B→A、全さくらスクリプト手番、clear_spot リセット等）
- **Out**:
  - `spot_newlines` 設定の意味・デフォルト値の変更
  - SSP 側描画仕様への対応・回避策
  - budoux 改行、`\n` トークン（明示改行）等、段落区切り以外の改行処理

## Boundary Candidates

- 改行出力位置の決定ロジック（emit_actor_switch 周辺）— 本 spec の中核
- per-spot 状態（has-text マップ）のライフサイクル（build 単位でリセット、clear_spot での扱い）

## Out of Boundary

- トークン生成側（act.lua / トランスパイラ）の変更
- actor_spots の永続化仕様（persist-spot-position の領分）
- バルーンのクリア（\c）やトーク間のバルーン状態管理

## Upstream / Downstream

- **Upstream**:
  - #21 修正（text_since_break 抑制）— 本変更で包摂または共存させる
  - completed spec: persist-spot-position（actor_spots の永続化）、actor-talk-grouping（トークングループ構造）、choice-definition-dsl（choice トークン）
- **Downstream**:
  - サンプルゴースト・実ゴーストのバルーン表示品質
  - pasta-user-manual / pasta-ghost-authoring スキルの挙動記述（該当があれば追随）

## Existing Spec Touchpoints

- **Extends**: なし（新規 spec）
- **Adjacent**: persist-spot-position（spot 状態の永続化と混同しないこと）、budoux-line-breaker（改行挿入だが別レイヤー）

## Constraints

- SSP の改行遅延描画仮説は設計フェーズで検証する（ukadoc / 実機 SSP での目視確認を検証項目に含める）
- 修正前後で SSP 上の見た目（A→B→A の段落区切り）が変わらないこと（回帰防止）
- 既存の Lua ユニットテスト基盤（lua_test）と Rust 統合テストの両方で検証する
- リファクタリングは特性化テスト先行・小ステップで行う（プロジェクト方針）
