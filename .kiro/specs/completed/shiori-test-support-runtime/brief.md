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

## 2026-10-10 棚卸の再測定（main add05022）

- **前提の変化**: 写しはそのまま残っている（`crates/pasta_shiori/tests/support/scripts/pasta/` の 10 ファイル）。起票の後に main へ入った 5 本は、どれも写しを直していない。本物の側だけが変わった（式の中の値なしの扱いで、`crates/pasta_lua/pasta_scripts/pasta/` の `act.lua` と `init.lua`）ので、ずれは広がった。
  - 本物の `act.lua` は 726 行になった（写しは 209 行のまま）。
  - 回避の設定を持つフィクスチャは 3 つから 4 つに増えた。シーン名の別名の結合テスト（`tests/fixtures/scene_alias_ontalk/pasta.toml`）が、同じ回避と「理由は同じ」というコメントを書き足した。結合テストを足すたびに、回避が写されていく。
  - 写しを今も読み込むのは、回避をしていないフィクスチャ `shiori_lifecycle` を、テストの共通部品（`tests/common/mod.rs` の `copy_fixture_to_temp`・`copy_fixture_into`。写しをコピーするのは 58〜64 行と 156 行から）で使う 8 ファイルのテスト。
  - `async_callback`・`debug_preservation` のフィクスチャは、別のコピー処理（`tests/common/async_callback_support.rs`・`tests/debug_preservation_test.rs`・`tests/actor_thread_vm_test.rs`）で使われ、写しを通らない。
  - この 3 つのフィクスチャは、テスト用の入口のスクリプト（`scripts/pasta/shiori/entry.lua`）を自分で持つ。「利用者のスクリプトが標準より優先される」本番の読み込み順を使った差し替えなので、残す。
- **触るファイル**: `tests/support/scripts/**`（消す）、`tests/common/mod.rs`（172 行）、回避をしている 4 つの `pasta.toml`、コピーの結果を確かめるテスト（`tests/test_path_helpers_test.rs` の 46〜53 行は、`scripts` と `scriptlibs` が置かれることを確かめている）、古いランタイムの結果を期待値にしていたテスト（調べて直す）。`src/` の中のテスト（`src/shiori_request_tests.rs`）は別のコピー処理で本物を使うので、触らない。1,000 行に近いファイルは無い。
- **規模**: 6〜8 タスク（写しに頼るテストの洗い出し、写しの撤去、コピー処理、期待値の直し、4 つの回避の取り外し、通しの確認）。
- **先に要るもの**: 無い。他の未完了 spec とファイルは重ならない。ただし Phase 11 の残りと `scene-anchor-link` は結合テストとフィクスチャを新しく足す見込みで、本 spec より先に入ると回避がまた写される。早いウェーブで入れるほど、後片付けが減る。
- **種別**: 基盤（テストの正しさ。動作は変えない）。
- **要件定義のモデル**: Opus。
- **分割の案**: なし。
- **見つけた穴・古くなった記述**:
  - Problem の「回避は 3 か所」は 4 か所、「本物は 774 行」は 726 行。
  - 同じ場所にあるテスト用ライブラリの写し（`tests/support/scriptlibs/lua_test/`）も古い（3 ファイルのうち 2 つが、本物の `crates/pasta_lua/scriptlibs/lua_test/` と違う）。使っているテストは見当たらない。一緒に消すかを要件で決める。
  - 別のコピー処理の 3 つは、本物のランタイムをゴーストの直下の `pasta_scripts/` へコピーしている。今の読み込み先は `profile/pasta/pasta_scripts`（`pasta.dll` が自分で展開する）なので、このコピーが効いているかを要件で確かめる。
  - `tests/common/mod.rs` の説明（42 行目・67 行目）は、差し替えの例に `main.lua` を挙げるが、実際に差し替えているのは `entry.lua` だけ。
