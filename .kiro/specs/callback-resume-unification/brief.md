# Brief: callback-resume-unification

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 1（バグ修正・最優先）。着手するときは `/kiro-start callback-resume-unification` で開始する。

## Problem

非同期コールバック（`act:get_property` などの SHIORI 非同期通信）でコルーチンを再開する経路が、通常のイベント（`EVENT.fire`）の再開ループ・予約の消費・継続の保存を通らない。また `EVENT.fire` は、ハンドラが返した文字列を常に `RES.ok` で包む。その結果、次の不具合が起きる。

| 項目 | 現象 |
| ---- | ---- |
| コールバック再開後の継続の消失 | `get_property` の後のチェイントークの続きと、出力の無い最初の中断が失われる |
| タイムアウト掃引の結果の破棄 | `CALLBACK.sweep` が `coroutine.resume` の結果を捨てる。再度 `get_property` した予約（`_staged`）が残り、次のイベントで「multiple staging」エラーか、古いイベント ID での誤登録になる（潜在的な 500） |
| タイムアウト応答の二重包み | `CALLBACK.sweep` の `RES.err(…)` 全文を `EVENT.fire` が `RES.ok` で包み、`Value` に 500 応答を持つ 200 になる |
| U23 REG ハンドラの戻り値の二重包み | ハンドラが `RES.ok(…)` などの応答全体を返すと、さらに `RES.ok` で包まれて不正な SHIORI 応答になる。ハンドラから 204（理由付き）・311・312 を返す手段も無い |

## Current State

照合記録は `pasta-runtime-internals-doc` の吸収台帳（`.kiro/specs/completed/pasta-runtime-internals-doc/absorption-ledger.md` 付録 B）と `manual-ssot-authority` の吸収台帳（U23）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。パスは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/` 基準。

- **継続の消失**: `callback.lua` の `try_route`（93–103 行付近）が 1 回だけ resume し、`resume_until_valid` を使わない。
  - コルーチンが再予約せずに中断すると、どこにも保存されない（`set_co_scene` は `init.lua` 89 行付近のローカル関数）。
  - 最初の中断が nil なら 204 になり、コルーチンが失われる。
- **掃引**: `callback.lua` の `sweep`（112–127 行付近）が `coroutine.resume` の戻り値（116・124 行付近）を捨てる。
  - `on_timeout` が文字列でない経路で再度 `get_property` すると、`_staged` が残る。
  - review-improvement-loop のマトリクス 3.49 は、`pairs` ループ中に `consume_staged` が `pending` へ挿入する問題を見て修正を見送った。
- **タイムアウトの二重包み**: `callback.lua` 121 行付近が `RES.err(...)` を返し、`second_change.lua` 18–19 行付近がそれを返し、`init.lua` 202–204 行付近が `RES.ok` で包む。
- **U23**: `init.lua` 202–204 行付近が、すべての文字列を `RES.ok` で包む。
  - `register.lua` 25 行付近・`init.lua` 36 行付近のコメントと、マニュアル `book/src/lua/shiori-events.md` 37–43・172 行付近は「ハンドラは `Value` にする文字列を返す」を設計として書く。
  - 一方、`crates/pasta_lua/tests/shiori/event_dispatch_test.rs`・`event_handler_test.rs` は `return RES.ok(…)` と書いている。応答を `find` で部分一致検査しているため通っている。
- **マニュアル**: `internals/execution-model.md` 102・198 行付近と `lua/shiori-events.md` 389 行付近は、現行挙動（継続が残らない）を書く。

## Desired Outcome

- コールバックで再開したコルーチンが、通常のイベントと同じ再開ループ（`resume_until_valid`）・予約の消費（`consume_staged`）・継続の保存（`set_co_scene`）を通る。`get_property` の後のチェイントークの続きが失われない。
- タイムアウト掃引が、再開結果を捨てず、予約を残さない。`pairs` ループ中の挿入の問題も起きない（期限切れを集めて取り除いてから、ループの外で再開する）。
- タイムアウト応答が正しい SHIORI 応答（500、または決めた形）になり、二重に包まれない。
- U23 の扱いが決まり、コード・テスト・マニュアルがそろっている。
- マニュアル（`lua/shiori-events.md`・`internals/execution-model.md`）が新しい挙動を書いている。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **再開の一本化**: `try_route` がコルーチンと参照を `EVENT.fire` に返し、`EVENT.fire` が同じ再開ループで駆動する。再開ヘルパーは、最初の resume の引数として参照（refs）を受けられるようにする。
- **既存の継続との関係**: コールバックの続きが中断したとき、既にある `co_scene`（別のチェイントークの続き）をどうするか。置き換えるか、残すか、片方を捨てて警告するか。
- **掃引の出力**: 掃引で再開したコルーチンの出力を、その回の OnSecondChange の応答にするか。
- **U23**: 次の 2 案から選ぶ。
  - 推奨は「`SHIORI/` で始まる文字列は応答全体として素通しする」（1 行。タイムアウトの二重包みも同時に直る）。
  - もう 1 案は「設計どおり（`Value` の文字列だけを返す）として、テストの `RES.ok` を直し、タイムアウトは `error(reason)` で `SHIORI.request` の xpcall に 500 を作らせる」。
  - 素通しを選ぶ場合は、ハンドラから 204・311・312 を返す方法として `RES` を使えることをマニュアルに書く。

## Scope

- **In**:
  - 上の 4 項目の修正（`shiori/event/init.lua`・`callback.lua`・`second_change.lua`）
  - 修正を固定するテスト（`crates/pasta_lua/tests/shiori/`・`tests/lua_specs/` のコールバック・エントリのテスト。`EVENT.fire` を通る経路で検査する）
  - テストが `RES.ok` を返している箇所の整理（U23 の決定に従う）
  - マニュアルの該当章の更新
- **Out**:
  - `get_property` の API・プロパティ名の仕様
  - アクター・シーン検索

## Boundary Candidates

- イベント配送（`init.lua` の `EVENT.fire`・再開ループ・`set_co_scene`）
- コールバック管理（`callback.lua` の `try_route`・`sweep`・予約）
- 定期イベント（`second_change.lua`）
- マニュアル

## Out of Boundary

- `act.lua`・`element_gen.rs`（Wave 1 では `dsl-codegen-runtime-safety` が持つ）
- 選択肢の自動ルーティング（`choice_select.lua`。棚卸の即時修正で U27・U31 を修正済み）

## Upstream / Downstream

- **Upstream**: `shiori-async-talk`（完了。コールバック基盤）、`shiori-event-test-framework`（完了。SHIORI イベントの試験基盤）
- **Downstream**: なし

## Existing Spec Touchpoints

- **Adjacent**: `review-improvement-loop`（マトリクス 3.49 で掃引の修正を見送った記録）

## Constraints

- LuaJIT 2.1 には `coroutine.close` が無い。中断したコルーチンを強制終了できないため、捨てるときは参照を外して GC に任せる。
- マニュアルが API・内部設計の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新する。
- 並走条件（Wave 1）: 編集するソースは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/`（`choice_select.lua` を除く）と、そのテストに限る。
