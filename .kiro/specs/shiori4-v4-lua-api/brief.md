2026-10-23 の棚卸で `shiori4-v4-host` から分けた（Lua と DSL の口の側。Rust の土台は `shiori4-v4-host` に残した）。

# Brief: shiori4-v4-lua-api

> **ステータス**: 未着手（2026-10-23 の棚卸で起票）。0.5.0（ノベルゲームと同じリリース）に入れる（2026-10-23 開発者の方針）。ノベルの合格には要らないので、ほかの 0.5.0 の spec を待たせない。要件定義は Fable（下の測り）。`shiori4-v4-host` の後。

## Problem

`shiori4-v4-host` が版 4 の土台（ホストのテーブル・問い合わせの中継・post・Tag）を入れても、作者がシーンから使える口が無ければ使えない。作者は今も次のことができない。

- ベースウェアのプロパティ（`currentghost.name` など）を、その場で読んで続きを書く（今は台本を 1 回出して、ベースウェアからの返事を待つ）
- 台本を今すぐ再生し、表示が終わるのを待ってから続きを書く
- 自分にイベントを起こす

## Current State

2026-10-23（main `82176f19`）に確かめた。

- **プロパティの読み出しは既にある**: `act:get_property`（`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua` 294 行）。DSL では `＄x＝＄％prop`（`book/src/grammar/variables.md` 294 行）。`\![get,property,…]` の台本を出してシーンを止め、`OnPastaCallBack{N}` で値が届いたら再開する。待ちの上限は既定 5 秒。
- **待って再開する仕組みも既にある**: 待ちの表 `CALLBACK.pending`（`pasta_scripts/pasta/shiori/event/callback.lua`）。予約（`stage_pending`）→ 止まった直後に登録 → 一致したイベントで再開、期限切れは掃除して時間切れの応答を出す。止まったシーンは `STORE.co_callback` で印を付け、チェイントークの続きの置き場（`event/init.lua` の `set_co_scene`）とは別に持つ。
- **チェイントーク**: キック・`OnPastaScene`・`OnPastaSceneAt` は `STORE.co_preempt` の印で続きを捨て、中断の台詞を出さない（`event/kick.lua`・`event/scene_play.lua`。`chain-talk-interrupt`）。
- **ハンドラ**: `REG:register` で 1 つのイベントにハンドラを重ねられる（`reg-handler-chain`）。
- **外からの呼び出し**: 外から `＞名前` の呼び出しとグローバル変数の代入ができるイベント（`event/external_control.lua`。`external-control-events`）。
- **NOTIFY**: pasta は NOTIFY を全部後回しにする（すぐ 204。`crates/pasta_shiori/src/actor/marshaling.rs`）。shiori4 の正本は「後回しの中では `host->request` を使えない」ので、NOTIFY のハンドラの中ではホストのプロパティを同期で読めない。
- **組み込みの名前の表**: `crates/pasta_check/src/validate/builtins.rs`（`get_property` は 55 行）。
- **土台**: `shiori4-v4-host` は未着手。ホストを呼ぶ口は、まだ pasta に無い。取り決めの正本 shiori4 の `host-protocol` は shiori4 の main にまだ入っていない（PR #7 で起票まで）。

## Desired Outcome

- シーン（DSL）と Lua から、次のことができる。
  - ホストのプロパティを読む。版 4 では、今の `＄％prop`・`get_property` が台本を出さずにその場で値を返す
  - 台本を今すぐ再生する（post の SEND）。待つかどうか（`Option: nobreak`）を選べる
  - 再生の終わりを待って続きを書ける。終わり方（最後まで出た・割り込まれた・一度も出なかった・断られた）がシーンから分かる
  - イベントを起こす（post の NOTIFY）
- 版 3（`shiori4-v3-portable`）と SHIORI/3.0 の入口では、今までどおり動く。`＄％prop` は今の待つ形のまま。SEND・NOTIFY など版 4 でしかできないことは、作者に分かる形で伝わる（戻り値・ログ）。
- `pasta_check` が新しい名前を知っていて、書き間違いを指摘できる。
- マニュアルに、版 4 でできること・版 3 との違い・中継の内側の説明がある。

## Approach

取り決めの要点（post の語・完了通知 `OnHostComplete` の `Tag`・`Status`・`Reference*`・割り込まれた側に OnBalloonBreak が出ないこと・OnHostComplete への応答の台本は `Option: nobreak` の SEND と同じに扱われること）は、`shiori4-v4-host` の brief の Approach にある。この spec は、その上に作者の口を載せる。

- **既にある仕組みに乗せる**: 完了を待つシーンは、`get_property` と同じ待ちの表 `CALLBACK.pending` に入れる（Tag をキーにして、`OnHostComplete` で一致したものを再開する）。新しい待ちの仕組みを別に作らない。時間切れの掃除・`STORE.co_callback` の印・チェイントークとの関係も、今の決まりをそのまま使う。
- **プロパティ**: 版 4 で読まれたときは、`get_property` の中で `shiori4-v4-host` の下回りの同期の読み出しを使い、台本を出さずに値を返す。版 3 と SHIORI/3.0 では今の待つ形のまま。作者の書き方（`＄％prop`）は変えない。
- **書き方は DSL を先に**: SEND・NOTIFY・待つ書き方を、DSL から書けるようにする（Lua だけの口にしない）。名前と書き方は、外から `＞名前` を呼ぶ口と揃える。
- **既定のハンドラ**: `OnHostComplete` の既定のハンドラ（待ちの表を引いて再開する）は、作者が `REG:register` で重ねたハンドラと並ぶ置き方にする。

## Scope

- **In**:
  - プロパティの読み出しを版 4 の同期の読み出しで答える（`get_property`・`＄％prop`）
  - SEND・NOTIFY の Lua の口と DSL の書き方
  - 完了を待って続きを書く書き方と、`OnHostComplete` で待っているシーンを再開する既定のハンドラ（`REG:register` との並べ方）
  - 待っている間の割り込み（キック・`OnPastaScene` の `STORE.co_preempt`）、後から届いた完了、待ち中の unload の扱い
  - 版 3 と SHIORI/3.0 で読まれたときの返し方。NOTIFY のハンドラの中でプロパティを読めないことの見せ方
  - `pasta_check` の組み込みの名前
  - Lua のテスト
  - マニュアル（`book/src/lua/*.md`・`internals/shiori.md`・`internals/execution-model.md`。`shiori4-v4-host` の中継の図など、内側の説明もここで書く）
- **Out**:
  - Rust の土台（ホストのテーブル・中継・post の受け口・Tag の振り出し・unload での無効化・`log` への転送・キックの即時化・`GET_TIMEOUT`）。`shiori4-v4-host` が持つ
  - shiori4 の ABI とホストの取り決めの変更（shiori4 の spec が持つ）
  - areka の側の実装

## Boundary Candidates

- プロパティの読み出し（`get_property` の版 4 の道）
- SEND・NOTIFY の口と、完了を待つ書き方（Lua と DSL）
- `OnHostComplete` の既定のハンドラと、待ちの表での再開
- 版 3・SHIORI/3.0 での返し方
- `pasta_check` とマニュアル

## Out of Boundary

- `novel-*` の各 spec の演出の通り道（SHIORI とさくらスクリプトのまま。版 4 を前提にしない）
- SHIORI/3.0 の要求の解析
- 待ちの表 `CALLBACK` の作りそのものの変更（使うだけ。直すべき穴が見つかったら、この spec で直す）

## Upstream / Downstream

- **Upstream**: `shiori4-v4-host`（Lua から呼べる下回りの口と、版 4 で読まれたかどうかの印。名前と形はあちらの要件で決まる）。shiori4 の `host-protocol` が main に入り、crates.io に出ること。実機の確認は areka v0.2.0（版 4 に対応する版）
- **Downstream**: ノベルゲームの演出で、表示の終わりを待つ書き方を使いたい spec（あれば）

## Existing Spec Touchpoints

- **Extends**: なし（`get_property` と待ちの表は、完了した spec が入れたものを使う）
- **Adjacent**: `shiori4-v4-host`（土台）、`chain-talk-interrupt`（`STORE.co_preempt`）、`reg-handler-chain`（`REG:register`）、`external-control-events`（`＞名前` の口と名前を揃える）、`ontalk-suppression`（止めている間の決まり）、`recommend-menu-scenes`・`novel-talk-flow`（`shiori/act.lua` を共有する）

## Constraints

- ゴーストの振る舞いは DSL で書けるようにする（Lua だけの口にしない）
- 版 4 を使うゴーストでも、版 3 と SHIORI/3.0 で読み込まれたときに壊れない（できないことは、できないと分かる形で返す）
- `host->request` は「load・request・notify を呼ばれているスレッドから、その呼び出しが返るまでの間だけ」呼べる（shiori4 の正本）。同期の読み出しは、pasta が応答を待っている要求の処理の中でしか使えない
- 状態はインスタンスごとに持つ（グローバルな状態を持ち込まない。2026-10-23 開発者の決定）
- 今の `＄％prop`・`get_property` の書き方と、版 3 での動きを変えない

## 2026-10-23 棚卸の測り（main 82176f19）

- **触るファイル**: `pasta_scripts/pasta/shiori/act.lua`（`get_property` の版 4 の道・SEND・NOTIFY・待つ書き方）、`pasta_scripts/pasta/shiori/event/host_complete.lua`（新規。`OnHostComplete` の既定のハンドラ）、`event/init.lua` の `require` の 1 行、要れば `event/callback.lua`（Tag をキーにした登録）、DSL から呼ぶ関数（`shiori/act.lua` か `GLOBAL`）、`pasta_check/src/validate/builtins.rs`、Lua のテスト（新規 1〜2）、マニュアル `book/src/lua/*.md`・`internals/shiori.md`・`internals/execution-model.md`。ファイルの重なり: `recommend-menu-scenes`（`event/init.lua`・`builtins.rs`・`shiori/act.lua`）、`novel-talk-flow`（`shiori/act.lua`）、`ponytail-lua-scripts`（`pasta_scripts/**`）。
- **規模**: 11〜13。待ちの表を使い回すので、新しく作るのは Tag との対応・既定のハンドラ・SEND と NOTIFY の口・版ごとの返し方が中心。
- **先に要るもの**: `shiori4-v4-host`（下回りの口と印）。shiori4 の `host-protocol` が crates.io に出ること。実機は areka v0.2.0。
- **種別**: 機能（版 4 の作者向けの口。開発者の方針で 0.5.0）。
- **要件定義のモデル**: Fable。(i) シーンが SEND の完了を待って続きを書く書き方は、公開の Lua と DSL の取り決めで、後のノベルの spec が上に乗る。(ii) その待ちの時の流れが Rust・Lua・SHIORI をまたぐ（`OnHostComplete` でのコルーチンの再開、キックの `STORE.co_preempt` での割り込み、待ち中の unload、版 3 と SHIORI/3.0 で読まれたときの返し方）。
- **議題**:
  1. SEND・NOTIFY・待つ書き方の DSL の形と、Lua の形（名前を `＞名前` の口とどう揃えるか）
  2. 終わり方（`completed`・`interrupted`・204・400＋`Reason`）をシーンにどう渡すか
  3. `OnHostComplete` の既定のハンドラと、作者が `REG:register` で重ねたハンドラの順番
  4. 待っている間にキック・`OnPastaScene` が来たとき（`STORE.co_preempt`）、待っているシーンを捨てるか。捨てた後に届いた完了をどうするか
  5. 待ち中の unload（待っているシーンは再開しない、でよいか）と、待ちの上限（今の `get_property` の 5 秒に揃えるか、再生の長さに合わせて別にするか）
  6. 版 3 と SHIORI/3.0 での SEND・NOTIFY の返し方（エラーにするか、何もせずにログを出すか）
  7. NOTIFY のハンドラの中で `＄％prop` を書いたときの見せ方（版 4 でも同期では読めない）
- **見つけた穴**: 元の brief は、`get_property`・`＄％prop` と待ちの表 `CALLBACK.pending` が既にあることに触れていなかった。これに乗せる前提で規模を見積もった。`shiori4-v4-host` の下回りの口の名前と形が決まるまで、この spec の要件は細部を詰められない（あちらの要件で、ここから使う形を先に決めてもらう）。
