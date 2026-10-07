# Brief: shiori-test-support-runtime

起点: 2026-10-07 の棚卸。完了した spec（dsl-codegen-runtime-safety・actor-proxy-act-delegation・call-execution-correctness）の実装メモに残っていた積み残しを起票した。

## Problem

`pasta_shiori` の結合テストは、一時ゴーストを作るときに `crates/pasta_shiori/tests/support/scripts/` を `scripts/` へコピーする（`tests/common/mod.rs` の `copy_fixture_into`）。そこにある `pasta/*.lua` は、本物のランタイム（`crates/pasta_lua/pasta_scripts/pasta/`）の古い写しで、`act.lua` は 209 行（本物は 774 行）。`act:actor_proxy` など最近の関数を持たない。新しいテストはこの写しを避けるために、フィクスチャの `pasta.toml` で `lua_search_paths` から `scripts` を外し、その理由をコメントで書いている（3 か所）。写しに気づかないテストは、古いランタイムで動いた結果を確かめてしまう。

## Current State

- 写し: `crates/pasta_shiori/tests/support/scripts/pasta/`（`act.lua`・`actor.lua`・`global.lua`・`init.lua`・`scene.lua`・`store.lua`・`word.lua`・`shiori/`・`areka/`）。
- 回避しているフィクスチャ: `tests/fixtures/{codegen_runtime_safety,actor_proxy_act_delegation,call_execution_correctness}/pasta.toml`。残りのフィクスチャは写しを読み込んでいる。
- 本物のランタイムは `pasta_lua` に埋め込まれ、既定プロファイルで読み込まれる。

## Desired Outcome

結合テストが本物のランタイムだけで動き、回避用の設定とコメントが要らなくなる。

## Approach

写しを消して、ハーネスが本物のランタイムを使うようにする。写しにしか無い差し替え（テスト専用の `main.lua` など）が要るテストは、フィクスチャ側に必要な最小限だけを置く。写しを残す理由が見つかった場合は、本物から生成する形に変える。

## Scope

- **In**: `tests/support/scripts/` の扱い、`tests/common/mod.rs` のコピー処理、写しを読み込んでいるフィクスチャとテストの追従、回避コメントの削除。
- **Out**: ランタイムそのものの変更、テストの意図の変更。

## Boundary Candidates

- 写しの撤去（ハーネスとフィクスチャ）
- 写しに依存していたテストの期待値の見直し

## Out of Boundary

- `pasta_lua` 側のテスト基盤（`lua_test`・モック）。

## Upstream / Downstream

- Upstream: なし。
- Downstream: なし。以後の結合テストが回避設定を書かずに済む。

## Existing Spec Touchpoints

- 完了済みの dsl-codegen-runtime-safety・actor-proxy-act-delegation・call-execution-correctness のフィクスチャ（回避コメントを外す）。

## Constraints

- 外部の挙動は変えない。テストだけの変更。
- `scripts/` に置いた利用者のスクリプトが標準ランタイムより優先される読み込み順は、本番の挙動として保つ。

## 測定（2026-10-07 棚卸・main 2cbaf510）

- 触るファイル: `crates/pasta_shiori/tests/support/scripts/**`（削除）、`tests/common/mod.rs`、`tests/fixtures/*/pasta.toml`、写しに依存していたテスト。
- 規模: 約 5〜8 タスク。
- 先に要るもの: なし。他の未完了 spec とファイルの重なりなし。
- 種別: 基盤（テストの正しさ）。
- 要件定義のモデル: Opus。
- 論点: 写しにしか無い振る舞いに依存しているテストがあるか。
