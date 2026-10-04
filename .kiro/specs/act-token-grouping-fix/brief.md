# Brief: act-token-grouping-fix

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 2（バグ修正）。着手するときは `/kiro-start act-token-grouping-fix` で開始する。

## Problem

ACT は積んだトークンをアクターごとのグループにまとめてからさくらスクリプトに組み立てる。このグループ化（`group_by_actor`）に 2 つの不具合がある。

- **最初の発言より前の表示制御が捨てられる**: 1 回の出力（`yield` またはシーンの終了で区切られる範囲）の中で、まだ `talk` を積んでいないうちに積んだ `surface`・`wait` などが出力されない。`act:yield()` の直後に `act:surface(5):wait(500):talk(…)` と書くと `\s[5]\_w[500]` が消える。
- **スポット変更でグループを閉じない**: `spot`・`clear_spot` が現在のアクターのグループを閉じないため、トークンが別のグループに入り、並びが変わる。

あわせて、LuaJIT で機能しない CT（`ct.lua`）を片付ける（既知負債）。

## Current State

照合記録は `pasta-runtime-internals-doc` の吸収台帳付録 B（「ACT のグループ化がトークンを捨てる・並べ替える」「CT（`ct.lua`）が LuaJIT で機能しない」）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。

- **グループ化**: `crates/pasta_lua/pasta_scripts/pasta/act.lua`。
  - 58–66 行付近は、最初の発言より前の表示トークンを捨てる。
  - 34–35 行付近の `spot`・`clear_spot` は、`current_actor_token`・`current_actor` を戻さない。
  - `sakura_builder.lua` 164 行付近は、アクターが nil のグループを既に受け付ける。
- **マニュアル**:
  - `book/src/internals/talk-output.md` 106・229 行付近は、現行挙動を書く。
  - `book/src/lua/script-api.md#表示制御` は「`talk` より前の表示制御は出力されない」を規則として書く。
  - 作例 `book/src/lua/patterns.md` は、棚卸の即時修正でこの規則に合わせた（`yield` の直後に先に `talk` を積む）。
- **CT**: `crates/pasta_lua/pasta_scripts/ct.lua`。
  - 5 行付近の `IMPL` に `__index` が無い。
  - 21 行付近の `__close` は、LuaJIT に `<close>` が無いため呼ばれない。
  - 利用者はいない。参照は、壊れた挙動を固定する `tests/lua_specs/ct_test.lua`（`tests/lua_specs/init.lua` に登録）と、マニュアル `internals/execution-model.md` 20・274–281・318 行付近だけ。
  - 配布物には入っているが、使えないため依存しているゴーストは無い。

## Desired Outcome

- 1 回の出力の中で、最初の `talk` より前に積んだ表示制御が、決めた規則で出力される（または、出力しない規則を維持すると決めて、警告か明確な記述がある）。
- `spot`・`clear_spot` の前後でトークンの並びが変わらない。
- CT が直っているか、撤去されている。
- マニュアル（`lua/script-api.md#表示制御`・`lua/patterns.md`・`internals/talk-output.md`・`internals/execution-model.md`）が新しい挙動を書き、スキル `references/` を再生成している。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **先頭の表示制御の行き先**: 次の 2 案から選ぶ。
  - (a) アクターの無いグループとして開き、`\p` などの切替の前に現在のスコープへ出す。
  - (b) 次の話者のグループに入れる。
  - (a) は `sakura_builder.lua` が既に nil のアクターを受けるため小さい。段落区切り改行の規則（`sakura-script-newline` の fully-lazy）と、立ち絵の復旧（`actor-surface-restore`）との相互作用を確かめる。
- **スポット変更**: `spot`・`clear_spot` で現在のグループを閉じる（2 行）。
- **CT**: 撤去（推奨）か修正か。
  - 撤去: `ct.lua`・`ct_test.lua`・`tests/lua_specs/init.lua` の登録・マニュアルの節を削る。
  - 修正: `__index` を足すのは 1 行だが、`__close` は LuaJIT では使えないままになる。

## Scope

- **In**:
  - `group_by_actor` の 2 つの不具合の修正
  - CT の撤去または修正
  - 修正を固定するテスト（`sakura_builder`・ACT の Lua テスト、出力のバイト比較）
  - マニュアルの該当章の更新とスキル `references/` の再生成
- **Out**:
  - 段落区切り改行の規則（`sakura-script-newline`・完了）の変更
  - 立ち絵の復旧（`actor-surface-restore`・完了）の変更

## Boundary Candidates

- ACT のトークン列とグループ化（`act.lua` の先頭〜`group_by_actor`）
- さくらスクリプトの組み立て（`sakura_builder.lua`。必要な場合）
- CT（`ct.lua`・そのテスト）
- マニュアル・生成スキル

## Out of Boundary

- `act.lua` の `init_scene`・`call`（Wave 3 の `call-execution-correctness` が持つ）
- `actor.lua`（Wave 2 では `actor-proxy-act-delegation` が持つ）

## Upstream / Downstream

- **Upstream**: `dsl-codegen-runtime-safety`（Wave 1。`act.lua` を先に触る）
- **Downstream**: `call-execution-correctness`（Wave 3。`act.lua` をこの spec の後に触る）

## Existing Spec Touchpoints

- **Adjacent**: `actor-talk-grouping`・`act-token-buffer-refactor`（完了。グループ化の元の設計）、`sakura-script-newline`・`actor-surface-restore`（完了。出力の規則）

## Constraints

- 既存のゴーストの出力（段落区切り改行・立ち絵の復旧を含む）を、修正対象の 2 つの現象以外で変えない。バイト比較のテストで確かめる。
- LuaJIT 2.1 には `<close>` も `coroutine.close` も無い。
- 並走条件（Wave 2）: 編集するソースは `act.lua` のトークン列・グループ化の範囲、`sakura_builder.lua`、`ct.lua` とそのテストに限る。Wave 2 の中で `act.lua` を編集するのはこの spec だけ。
