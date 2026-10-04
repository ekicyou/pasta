# Brief: scene-search-key-normalization

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 1（バグ修正）。着手するときは `/kiro-start scene-search-key-normalization` で開始する。

## Problem

シーン名・アクター名は、登録するときに英数字以外を `_` に置き換えた（サニタイズした）キーで登録されるが、検索は作者が書いた元の名前で行う。そのため、置き換えられる文字を含む名前は、元の名前で Call・検索・アクター単語参照ができない。

- **U30**: 記号を含むグローバルシーン名（例 `＊会話・朝`）が、元の名前で検索・Call できない（「見つからない」警告になり、何も出力しない・204 になる）。
- **ローカルシーン名**: 同じ食い違いがある。
- **アクター名**: アクター単語のキー `:__actor_X__:` はサニタイズした名前で作られるが、`actor.lua` は元の名前で検索する。記号を含むアクター名のアクター単語が引けない。

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（U30）と `pasta-runtime-internals-doc` の吸収台帳付録 B（「シーン・アクター名のサニタイズと検索キーの不一致」）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。

- **登録（サニタイズする）**:
  - `crates/pasta_core/src/registry/scene_registry.rs` 241–243 行付近。
  - `crates/pasta_lua/src/code_gen/scope_gen.rs` 120 行付近は、サニタイズ済みの名前を `create_scene` に渡す。
- **検索（元の名前）**: `crates/pasta_lua/src/search/context.rs` の `search_scene`（68–110 行付近）は、作者が書いた名前をそのまま使う。
- **アクター**:
  - `crates/pasta_core/src/registry/word_registry.rs` 83 行付近がキー `:__actor_X__:` の X をサニタイズする。実行時は `crates/pasta_lua/src/runtime/finalize.rs` 187 行付近、トランスパイル時は `transpiler.rs` 239 行付近で使われる。
  - `crates/pasta_lua/pasta_scripts/pasta/actor.lua` 142 行付近は元の `self.actor.name` で検索する。
- **棚卸の即時修正で修正済み（関連）**: グローバルシーンの検索が `:` で始まるローカルのキーを候補にする問題（`search_scene` が `resolve_scene_id` を使っていた）。
- **マニュアル**: 素朴な名前だけを例にしており、この不具合は書いていない。

## Desired Outcome

- 置き換えられる文字を含むシーン名（グローバル・ローカル）・アクター名でも、作者が書いた元の名前で Call・シーン検索・アクター単語参照ができる。
- 登録キーと検索キーを作る規則が 1 か所にまとまり、二度と食い違わない。
- マニュアル（`grammar/call-jump.md`・`internals/internal-modules.md` ほか）が、名前に使える文字と検索の扱いを書いている。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **シーン名**: 検索の入口（`SearchContext::search_scene`）で、作者が書いた名前を登録と同じ規則でサニタイズする（入口 1 か所の修正で全呼び出し元に効く）。
  - サニタイズするのは作者が書いた名前だけにする。ランタイムのグローバル名（`global_scene_name`）やスコープ引数には適用しない。
  - 後続の `scene-identity-format` がランタイム名の区切り文字を決めるため、その区切りがサニタイズで消えないようにする。
- **アクター名**: `actor.lua` でもサニタイズするか、`register_actor` でサニタイズをやめるか。後者の場合、アクター名に `:` などキー構文と衝突する文字を許すかを確認する。
- **衝突**: 異なる名前がサニタイズ後に同じキーになる場合（`会話・朝` と `会話_朝`）の扱い（警告・前方一致の候補として許す、など）。現行も同じことが起きているため、少なくとも記録する。
- サニタイズ規則を共有関数にし、登録・検索・トランスパイルが同じ関数を呼ぶ。

## Scope

- **In**:
  - シーン名（グローバル・ローカル）とアクター名の、登録キーと検索キーの統一
  - 修正を固定するテスト（記号を含むシーン名の Call・検索、記号を含むアクター名のアクター単語）
  - マニュアルの該当章の更新とスキル `references/` の再生成
- **Out**:
  - 末尾が数字のシーン名・ランタイムのシーン名の形式（`scene-identity-format`。Wave 2）
  - シーン検索アルゴリズム（前方一致・シャッフル）の変更
  - 選択肢の自動ルーティング（即時修正済み）

## Boundary Candidates

- 検索の入口（`pasta_lua` の `search/context.rs`）
- 登録（`pasta_core` の `scene_registry.rs`・`word_registry.rs`）
- アクター単語の検索（`actor.lua` の該当箇所）
- マニュアル・生成スキル

## Out of Boundary

- `act.lua`・`element_gen.rs`（Wave 1 では `dsl-codegen-runtime-safety` が持つ）
- `pasta_core` の `random.rs`（`search-selector-indices` が持つ）

## Upstream / Downstream

- **Upstream**: なし
- **Downstream**: `scene-identity-format`（サニタイズ規則とランタイム名の区切り文字を両立させる）、`actor-proxy-act-delegation`（`actor.lua` をこの spec の後に触る）、`search-selector-indices`（`pasta_core` の registry をこの spec の後に触る）、`call-attribute-filter`（`search_scene` にフィルターを通す）

## Existing Spec Touchpoints

- **Adjacent**: `scene-search-integration`・`pasta_search_module`（完了。検索モジュールの元の設計）

## Constraints

- マニュアルが文法・API の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- 現行実装を正として設計する。
- 並走条件（Wave 1）: 編集するソースは `crates/pasta_lua/src/search/`・`crates/pasta_core/src/registry/`（`random.rs` を除く）・`crates/pasta_lua/pasta_scripts/pasta/actor.lua` のアクター単語検索の箇所に限る。`actor.lua` の `PROXY_IMPL` の呼び出し規約は変えない（Wave 2 の `actor-proxy-act-delegation` が持つ）。
