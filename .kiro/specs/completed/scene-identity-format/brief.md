# Brief: scene-identity-format

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 2（バグ修正）。着手するときは `/kiro-start scene-identity-format` で開始する。

## Problem

グローバルシーンのランタイム名は「サニタイズした名前＋通し番号」を区切り無しでつないで作る（`メイン` の 1 つ目は `メイン1`）。そのため、末尾が数字のシーン名で、内部名が別のシーンと重なる。

- **U21**: `＞A1` が `A` の 1 つ目（内部名 `A1`）を選ぶ。2 つの名前の内部名が完全に一致する場合（`A1` の 1 つ目と `A` の 11 個目がどちらも `A11`）は、片方のシーン表がもう片方を上書きし、シーンが消える。
- **デバッガのシーン identity の索引漏れ**: `split_runtime_global` が名前の末尾の数字まで通し番号とみなす（`章11` を (`章`, 11) と読む）。カーソル位置のキックが別のシーンを選ぶ。
- **位置からのキックの前方一致**: `KICK.try_dispatch` が前方一致の検索（`SCENE.search`・`SCENE.co_exec`）で引くため、名前が接頭辞になる別のシーン（`会話1` に対する `会話10`）が再生されうる。キックの対象はデバッガが解決した厳密な identity なので、完全一致で引くべきである。

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（U21）と `pasta-runtime-internals-doc` の吸収台帳付録 B（「末尾が数字のシーン名のシーン identity 索引漏れ」「位置からのキックの前方一致」）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。

- **ランタイム名の生成**:
  - `crates/pasta_lua/pasta_scripts/pasta/scene.lua` 132 行付近（`base_name .. counter`）。
  - `crates/pasta_lua/src/transpiler.rs` 206 行付近（`format!("{}{}", sanitize_name, counter)`）。
  - 検索はこの名前に対する前方一致。
- **索引**: `crates/pasta_lua/src/debug/source_map/scene_join.rs` 62–75 行付近の `split_runtime_global`。結合は 89–103 行付近。
- **キック**: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua` 140–150 行付近。
  - `kick_pending` は、デバッガが解決した厳密な identity（`debug/playscene.rs` の `build_kick_scene`）からだけ設定される。
  - 完全一致の口は既にある（`scene.lua` 92・110 行付近の `SCENE.get(parent, local)`・`SCENE.get_start(global)`）。
- **マニュアル（現行の形式を公開 API として書いている）**:
  - `book/src/lua/modules/pasta-search.md` 60・72–78・106・115・139 行付近（`"メイン1"`）。
  - `book/src/lua/patterns.md` の `WORD.create_local("メイン1", …)`（棚卸の即時修正で `メイン_1` から直した）。
  - `book/src/internals/internal-modules.md` 235 行付近（「区切り無し」）・`internals/debug.md`（キック・DAP の番兵 `:` を含む）。

## Desired Outcome

- どんなシーン名（末尾が数字のものを含む）でも、ランタイム名が一意で、名前と通し番号に曖昧さなく分けられる。`＞A1` は `A1` という名前のシーンだけを選び、シーン表の上書きでシーンが消えることが無い。
- デバッガのシーン identity の索引と、位置からのキックが、正しいシーンを選ぶ。キックは完全一致で引く。
- ランタイム名の形式を作る・分ける処理が 1 つの関数（Rust と Lua それぞれ 1 か所）にまとまっている。
- マニュアル（`lua/modules/pasta-search.md`・`lua/patterns.md`・`lua/script-api.md`・`internals/internal-modules.md`・`internals/debug.md`）が新しい形式を書き、スキル `references/` を再生成している。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **区切り**: 名前と通し番号の間に区切りを入れる。
  - 推奨は `_`（最後の `_` で分ければ一意に分けられる。サニタイズ後の名前は `_` を含みうるが、通し番号は数字だけなので「最後の `_` の後ろが数字だけ」で分けられる）。
  - キック・DAP の番兵 `:` を含まないこと。
  - `scene-search-key-normalization` が決めたサニタイズ規則と両立すること。
- **公開 API の互換**: `WORD.create_local("メイン1", …)` のようにランタイム名を直接書く Lua コードが壊れる。移行の扱いを決める（旧形式を受け付けるか、リリースノートで告知するか）。
- **キックの完全一致と索引の修正**は、形式の変更と独立して先に出せる（小さな独立スライス）。ただし、内部名が完全に重なる場合は形式の変更でしか直らない。
- `split_runtime_global` は名前を分けるのをやめ、ランタイムのグローバル名の集合を持ち、記録ごとに「名前＋区切り＋番号」を組み立てて結合する。組み立ては形式の関数を共有する。

## Scope

- **In**:
  - ランタイム名の形式の変更（`scene.lua`・`transpiler.rs`）
  - デバッガの索引（`scene_join.rs`・必要なら `scene_index.rs`・`playscene.rs`）の修正
  - 位置からのキックの完全一致化（`kick.lua`）
  - トランスパイラのスナップショット・統合テストの更新、修正を固定するテスト
  - マニュアルの該当章の更新とスキル `references/` の再生成
- **Out**:
  - サニタイズ規則（`scene-search-key-normalization`）
  - シーン検索アルゴリズム（前方一致・シャッフル）

## Boundary Candidates

- ランタイム名の形式（`scene.lua`・`transpiler.rs`）
- デバッガのソースマップ・キック（`debug/source_map/`・`debug/playscene.rs`・`kick.lua`）
- マニュアル・生成スキル

## Out of Boundary

- `act.lua`（Wave 2 では `act-token-grouping-fix` が持つ）
- `actor.lua`（Wave 2 では `actor-proxy-act-delegation` が持つ）

## Upstream / Downstream

- **Upstream**:
  - `scene-search-key-normalization`（Wave 1。サニタイズ規則と区切り文字を両立させる）。
  - `dsl-codegen-runtime-safety`（Wave 1。トランスパイラのスナップショットを広く変えるため、その後に着手する）。
- **Downstream**: `call-execution-correctness`（Wave 3。スナップショットをこの spec の後に変える）、`scene-attribute-store`（`scene.lua` をこの spec の後に触る）

## Existing Spec Touchpoints

- **Adjacent**: `pasta-scene-kick`・`pasta-scene-kick-from-cursor`（完了。キックの設計）、`pasta-source-map`（完了。ソースマップ）

## Constraints

- マニュアルが API・内部設計の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- ソースマップ（`.pasta` 行 ↔ 生成 Lua 行）とデバッガの既存テストを壊さない。
- 並走条件（Wave 2）: 編集するソースは上の境界候補に限る。トランスパイラのスナップショットを広く更新するのは Wave 2 ではこの spec だけ。
