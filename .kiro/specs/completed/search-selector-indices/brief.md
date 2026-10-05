# Brief: search-selector-indices

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 2（バグ修正）。着手するときは `/kiro-start search-selector-indices` で開始する。

## Problem

`@pasta_search` の `set_scene_selector(…)`・`set_word_selector(…)` は、テストでシーン・単語の選択を決め打ちするための API で、整数の並びを受け取る。ところが渡した整数は選択に使われず、シャッフルが止まるだけになる（U29）。作者が Lua のテストで「2 番目の候補を選ばせる」ことができない。マニュアルの API 章（`lua/modules/pasta-search.md`）は整数を受けるように読め、内部設計章は「使われない」と書いていて、両者の書き方もそろっていない。

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（U29）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。

- `crates/pasta_core/src/registry/random.rs` 97 行付近の `MockRandomSelector::shuffle_usize` が何もしない。
- シーン表・単語表は `shuffle_usize` だけを呼ぶ（`scene_table.rs` 268・286 行付近、`word_table.rs` 210 行付近）。`select_index` は `random.rs` の外で使われていない。
- 呼び出し元はテストだけ（`crates/pasta_lua/tests/runtime/scene_test.rs`・`tests/search/module_test.rs`）。
- マニュアル:
  - `book/src/internals/registry-search.md` 154 行付近は、整数が使われないことを書く。
  - `book/src/lua/modules/pasta-search.md` 120–135 行付近は API を説明する。

## Desired Outcome

- `set_scene_selector`・`set_word_selector` に渡した整数の並びが、決めた意味（例: 候補の並びの添字を順に使う）で選択に効く。テストで特定の候補を選ばせられる。
- 利用者章と内部設計章が同じ意味を書いている。

## Approach

要件フェーズで次を決める。

- 整数の意味: 候補の添字（0 始まりか 1 始まりか）を順に使うのか、シャッフル後の並びを決めるのか。並びを使い切った後の挙動（繰り返す・最後を使い続ける・通常の乱数に戻る）。
- 実装の位置: モックが整数の並びに従って並べ替える（`random.rs` だけで閉じる）か、API を定義し直してシーン表・単語表が `select_index` を使うか。前者を優先する。
- 本番の選択（乱数・シャッフル消費）は変えない。

## Scope

- **In**:
  - セレクタの整数の意味の確定と実装
  - 修正を固定するテスト
  - マニュアル（`lua/modules/pasta-search.md`・`internals/registry-search.md`）の更新
- **Out**:
  - 本番のシーン・単語選択のアルゴリズム
  - 検索キーの正規化（`scene-search-key-normalization`）

## Boundary Candidates

- `pasta_core` の `registry/random.rs`（モックのセレクタ）
- `pasta_lua` の `@pasta_search` の API 口（必要なら）
- マニュアル

## Out of Boundary

- `pasta_core` の `scene_registry.rs`・`word_registry.rs`（`scene-search-key-normalization` が持つ）

## Upstream / Downstream

- **Upstream**: `scene-search-key-normalization`（Wave 1。`pasta_core` の registry を先に触るため、その完了後に着手する）
- **Downstream**: なし

## Existing Spec Touchpoints

- **Adjacent**: `pasta_search_module`・`pasta-lua-unit-test-framework`（完了。テスト用 API の元の設計）

## Constraints

- マニュアルが API の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新する。
- 並走条件（Wave 2）: 編集するソースは `crates/pasta_core/src/registry/random.rs` を中心とし、シーン表・単語表（`scene_table.rs`・`word_table.rs`）に触れる場合は Wave 2 の他 spec と重ならないことを確かめる（Wave 2 の他 spec は `pasta_core` を触らない想定）。
