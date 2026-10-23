# Brief: shiori4-v4-host

> **ステータス**: 未着手（2026-10-23 の `/kiro-discovery` で起票）。0.5.0（ノベルゲームと同じリリース）に入れる（2026-10-23 開発者の方針）。要件定義は Fable（Rust のアクター・Lua の API・SHIORI の時の流れをまたぎ、後の spec が上に乗る取り決めになるため）。

## Problem

SHIORI/3.0 の SHIORI は、ホストに呼ばれたときに文字列を 1 回返すことしかできない。pasta では次のことが困っている。

- MCP からシーンを再生しても（`OnPastaScene`・キック）、台本を出せるのは次の OnSecondChange の応答のときである。すぐには出ない。
- ベースウェアのプロパティ（`currentghost.name` など）を、シーンの中から同期で読めない。
- 「台本を出して、表示が終わるのを待ってから続ける」を、Lua のコルーチンで素直に書けない。

## Current State

2026-10-23（main `c360adb2`）に確かめた。

- pasta.dll は Lua VM をアクタースレッドに固定し、SHIORI のスレッドとは mailbox でやり取りする。NOTIFY はすぐに 204 を返し、後で処理する。
- キックは保留のフラグを立てるだけで、次の要求の応答で台本を出す（`actor/thread.rs` の KICK）。
- 0.4.2 の `shiori4-v3-portable` が、shiori4 の版 3 の入口（`shiori_get_api`）と、OS に頼らない実行器を入れる（予定）。
- shiori4 の版 4: ホストのテーブル `ShioriHostApi` に、同期の `request`（プロパティなど）・任意のスレッドから呼べる `post`・`log` がある。post の結果は、`Tag` 付きの通常のイベント（`OnHostComplete`）として後から届く。

## Desired Outcome

- pasta.dll が `shiori_get_api(4)` に応え、版 4 の SHIORI として動く。
- シーン（DSL）と Lua から、次のことができる。
  - ホストのプロパティを読む
  - 台本を今すぐ再生する（post の SEND）。待つかどうか（`Option: nobreak`）を選べる。完了を待って続きを書ける
  - イベントを起こす（post の NOTIFY）
- MCP からのシーン再生（キック）が、次の OnSecondChange を待たずにすぐ出る。
- 版 3（`shiori4-v3-portable`）と SHIORI/3.0 の入口では、今までどおり動く。版 4 でしか使えないことは、作者に分かる形で伝わる（例: プロパティが読めないときの戻り値・ログ）。

## Approach

2026-10-23 の 3 者会談（pasta・areka・shiori4）で、ABI の規約とホストとの取り決めを合意した。正本は shiori4 の 2 本の spec（`abi-conventions`・`host-protocol`。shiori4 の作業ブランチ `claude/abi-host-protocol` で起票済み）である。pasta に関わる要点:

- **スレッド**: `host->request` は「load・request・notify を呼ばれているスレッドから、その呼び出しが返るまでの間だけ」呼べる（違反は `SHIORI_E_WRONG_THREAD (-7)`）。pasta の Lua はアクタースレッドで動くので、アクターからの問い合わせを、応答を待っている SHIORI のスレッドへ中継して呼ぶ。ABI は緩めない。
- **notify の後回し**: 合法。ただし後回しにした処理の中では `host->request` を使えない（post を使う）。
- **unload**: 「unload が返った後に ShioriHostApi のどの関数も呼ばない」。join しない pasta は、unload を始めた時点で poster を無効にして守る（shiori4 クレートの poster がこの作りになる）。
- **log・release**: どのスレッドから呼んでもよい。
- **post の語（SSTP に揃える）**: `Command: SEND` ＋ `Script:` で台本を再生する。既定では割り込み、`Option: nobreak` のときだけ今の台本が終わるまで待つ。`Command: NOTIFY` ＋ `Event`・`Reference*` で、ホストがそのイベントを SHIORI に送る。post の宛先は post したインスタンスに固定する。
- **完了通知**: `ID: OnHostComplete`・`Tag`・`Status`・`Reference0: <コマンド名>`。request で届く。SEND の完了は再生が終わった時で、200＋Reference1（`completed`／`interrupted`）、204（一度も再生せずに捨てた）、400＋`Reason`（不正・拒否）。NOTIFY は起こせたかどうか（200／400）。OnHostComplete への応答の台本は、`Option: nobreak` の SEND と同じに扱われる。
- **割り込まれた側**: SEND に切られたときに OnBalloonBreak は出ない。知りたい台本は Tag 付きの SEND で出す。
- **要求のヘッダ**: 必須は ID・Sender・SenderType・SecurityLevel・SecurityOrigin。Status・BaseID は状況による。Charset は無い（UTF-8 固定）。post から起きた要求は `SenderType: internal`・`SecurityLevel: local`。
- **拒否**: 400＋`Reason: forbidden`。
- **SHIORI の名前と版**: SHIORI/3.0 と同じ `ID: version`・`name`・`craftman`・`craftmanw` に 200＋`Value`（pasta は今は答えていない。答えるかを要件で決める）。

## Scope

- **In**:
  - 版 4 の入口（`load` でホストのテーブルを受け、unload まで持つ）
  - アクタースレッドから `host->request` への中継（SHIORI のスレッドで呼ぶ）
  - post の受け口（任意のスレッドから。unload で無効にする）と Tag の管理
  - OnHostComplete を受けて、待っているシーン（コルーチン）を再開する仕組み
  - Lua の API と、DSL からの書き方（プロパティの読み出し・SEND・NOTIFY・完了を待つ）
  - キックを post の SEND で即座に出す
  - ホストの `log` へのログの転送
  - マニュアル（版 4 でできること・版 3 との違い）
- **Out**:
  - shiori4 の ABI とホストの取り決めの変更（shiori4 の spec が持つ）
  - areka の側の実装（再生の列・SSTP の受け口）
  - 状態をインスタンスごとに持つエンジンの作りそのもの（`shiori4-v3-portable` が入れる）。この spec が足すホストのテーブル・post の受け口・Tag の表も、インスタンスごとに持つ（グローバルな状態を持ち込まない。2026-10-23 開発者の決定）

## Boundary Candidates

- Rust 側の版 4 の土台（ホストのテーブルの保持・スレッドの中継・post と Tag・unload での無効化）
- Lua の API と DSL の書き方（作者が触る口）
- キックの即時化（デバッグの経路）

規模が 20 タスクを超えるなら、Rust 側の土台と、Lua・DSL の口とで分ける。

## Out of Boundary

- `novel-*` の各 spec の演出の通り道（SHIORI とさくらスクリプトのまま。版 4 を前提にしない）
- SHIORI/3.0 の要求の解析

## Upstream / Downstream

- **Upstream**: `shiori4-v3-portable`（入口と実行器）。shiori4 の `abi-conventions`・`host-protocol`（公開された版に依存する）。areka の版 4 の対応（実機の確認に要る。2026-10-23 の areka の開発者の決定: areka は COM の作りかけを消して shiori4 に移り、版 4 には areka v0.2.0 で対応する。どちらが先に来ても困らないよう、shiori4 の正本どおりに作る）
- **Downstream**: ノベルゲームの演出で、表示の終わりを待つ書き方を使いたい spec（あれば）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `scene-play-event`（完了。キックの経路）、`chain-talk-interrupt`（チェイントークの続きは pasta の中にあり、SEND の割り込みとは別物）、`ponytail-shiori`（`actor/` を共有する）

## Constraints

- ホストは SHIORI を専用のプロセスで読むことがある（areka はそうする。2026-10-23 の shiori4 の abi-conventions の論点 11）。ホストと同じプロセスにいることを前提にしない。`host->request` は IPC を往復して遅くなりうるので、GET の応答を待つ上限（アクターのタイムアウト）はそれを見込んで決める
- 版 4 の `load` が NULL を返すと、ホストは SHIORI/3.0 として読み直さない（2026-10-23 の shiori4 の裁定。`shiori4_host` の `from_library`）。版 4 の `load` で失敗すると、ゴーストは起動しない。版 4 の準備に失敗しても版 3 相当で動けるなら、NULL を返さずに動く道を要件で考える
- 版 4 を使うゴーストでも、版 3 と SHIORI/3.0 で読み込まれたときに壊れない（できないことは、できないと分かる形で返す）
- ゴーストの振る舞いは DSL で書けるようにする（Lua だけの口にしない）
- pasta.dll の出荷物は 1 種類のまま。静的 CRT リンクを保つ
- shiori4 の spec が main に入り、crates.io に出た版を使う

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**: 起票の後に 0.4.2 の 4 本が入った。この spec に効くのは 3 本。
  - `chain-talk-interrupt`: キック・`OnPastaScene`・`OnPastaSceneAt` は `kick.lua`・`scene_play.lua` で印を付け、`set_co_scene` がチェイントークの続きを捨てる（`STORE.co_preempt`）。キックを post の SEND で即座に出すときも、この「捨てる・中断の台詞を出さない」を保つ。
  - `reg-handler-chain`: `REG:register` でハンドラを重ねられる。OnHostComplete の既定のハンドラは、作者の `register` と並ぶ置き方を決める。
  - `external-control-events`: 外からの `＞名前` の呼び出しがある。post の NOTIFY で自分にイベントを起こす口と、名前や書き方を揃える。
  - crates.io の `shiori4` 系は 0.0.1 だけ。`shiori4_abi` 0.0.1 には `ShioriHostApi`（`request`・`post`・`log`）と `ShioriLoadMode::V4` が既にあるが、host-protocol の spec はまだ shiori4 の main に入っていない（PR #7 で起票まで）。pasta が `version`・`name`・`craftman` に答えていないのは今も同じ。
- **触るファイル**: `pasta_shiori` の `actor/{thread,mailbox,marshaling,lifecycle}.rs`（ホストへの問い合わせの中継・post・Tag の表）と、`shiori4-v3-portable` が作る版 3 の入口。`pasta_lua` に、ホストを呼ぶ口（`runtime/module_registry.rs` の新しいモジュールか、キックと同じ `RuntimeConfig` の差し込み）。Lua は `pasta/shiori/event/kick.lua`・新しい `event/host_complete.lua`（新規）・`event/init.lua` の `require` の 1 行・DSL から呼ぶ関数（`shiori/act.lua` か `GLOBAL`）。`pasta_check/src/validate/builtins.rs`（新しい名前）。マニュアル `lua/*.md`・`internals/shiori.md`・`internals/execution-model.md`。
- **規模**: 22〜28。Rust の土台と、Lua・DSL の口とで分ける（下）。
- **先に要るもの**: `shiori4-v3-portable`（入口と OS に頼らない実行器）。shiori4 の `abi-conventions`・`host-protocol` が main に入り、crates.io に出ること。実機は areka v0.2.0。ファイルの重なり: `ponytail-shiori`（`actor/`）、`recommend-menu-scenes`（`event/init.lua`・`builtins.rs`・`shiori/act.lua`）、`ponytail-lua-scripts`（`pasta_scripts/**`）、`ponytail-lua-loader`（`runtime/module_registry.rs`）、`novel-talk-flow`（`kick.lua`・`shiori/act.lua`）、`novel-checkpoint-rollback`（`marshaling.rs` の 5 秒の待ち）。
- **種別**: 機能（版 4 に本格対応。開発者の方針で 0.5.0）。
- **要件定義のモデル**: 分けたら、Lua・DSL の口の側が Fable。決めるのは、シーンが SEND の完了を待って続きを書く取り決め（公開の Lua と DSL の口で、後のノベルの spec が上に乗る＝(i)）と、その待ちの時の流れ（OnHostComplete でのコルーチンの再開・チェイントークの続きや `co_preempt` との関係・待ち中の unload・版 3 と SHIORI/3.0 で読まれたときの返し方。Rust・Lua・SHIORI をまたぐ時の流れ＝(ii)）。Rust の土台の側は Opus（ABI とスレッドの決まりは shiori4 の正本が決めていて、残りは仕組みの選び方）。分けないなら全体を Fable。
- **分割の案**: `shiori4-v4-host`（Rust の土台。版 4 の `load` でテーブルを持つ・アクターからの `host->request` を SHIORI のスレッドへ中継・post の受け口と Tag の表・unload での無効化・`log` への転送・キックの即時化。11〜13）→ `shiori4-v4-lua-api`（新規。プロパティの読み出し・SEND・NOTIFY・完了を待つ書き方・版 4 でしかできないことの伝え方・マニュアル。11〜13）。
- **見つけた穴・古くなった記述**: NOTIFY は pasta が全部後回しにする（すぐ 204。`marshaling.rs`）ので、NOTIFY のハンドラの中ではホストのプロパティを読めない（正本の「後回しの中では `host->request` を使えない」）。作者にどう見せるかを要件で決める。GET の待ちの上限 `GET_TIMEOUT`（`marshaling.rs` 71 行、5 秒）は、中継の往復（IPC）を見込んで決め直す。`ontalk-suppression` の止めている間は、キックの続き（2 ビート目から）も出さない決まりで、SEND で即座に出す道も同じに従う。Current State の基準の `c360adb2` は古いが、`crates/pasta_shiori` はその後も変わっていない。

## 2026-10-23 棚卸の分割

- **分けた先**: 作者が触る Lua と DSL の口を、新しい `shiori4-v4-lua-api`（`.kiro/specs/shiori4-v4-lua-api/brief.md`）へ移した。この spec は Rust の土台だけを持つ。上の Desired Outcome・Approach・Scope のうち、移した部分の行は新しい brief が持つ（ここの本文は書き換えない）。冒頭のステータスの「要件定義は Fable」は、分ける前の全体の話である。
- **In（この spec に残るもの）**:
  - 版 4 の `load` でホストのテーブルを受け、unload まで持つ（インスタンスごと）
  - アクタースレッドからの `host->request` を、応答を待っている SHIORI のスレッドへ中継する
  - post（`SEND`＋`Script`・`Option: nobreak`、`NOTIFY`）の受け口と、`Tag` の振り出し（Rust の側）。unload を始めた時点で post を無効にする
  - `OnHostComplete` を、ほかのイベントと同じ道で Lua に渡す（Rust の側で飲み込まない）
  - ホストの `log` へのログの転送
  - キックを post の SEND で即座に出す。`chain-talk-interrupt` の「チェイントークの続きを捨て、中断の台詞を出さない」（`STORE.co_preempt`）と、`ontalk-suppression` の「止めている間は 2 ビート目から出さない」を保つ
  - GET の待ちの上限 `GET_TIMEOUT`（`marshaling.rs` 71 行、5 秒）を、中継の往復（IPC）を見込んで決め直す
  - Lua から呼べる下回りの口（作者向けの名前ではない）と、版 4 で読まれたかどうかを Lua に知らせる印。`shiori4-v4-lua-api` がそのまま上に乗れるよう、名前と形をこの spec の要件で決める
  - `version`・`name`・`craftman`・`craftmanw` に答えるか（Approach の最後の行）
- **Out（`shiori4-v4-lua-api` へ移したもの）**: シーンと Lua からプロパティを読む・SEND・NOTIFY を使う書き方。完了を待って続きを書く書き方と、`OnHostComplete` で待っているシーンを再開する仕組み（既定のハンドラと `REG:register` の並べ方、`STORE.co_preempt` との関係、待ち中の unload）。版 3 と SHIORI/3.0 で読まれたときの作者への伝え方（NOTIFY のハンドラの中でプロパティを読めないことも）。`pasta_check` の新しい名前。マニュアル（`lua/*.md`・`internals/shiori.md`・`internals/execution-model.md`。中継の図など内側の説明も、あちらでまとめて書く）。
- **順番**: この spec が先、`shiori4-v4-lua-api` が後。どちらも 0.5.0（ノベルの合格には要らないので、ほかの 0.5.0 の spec を待たせない）。
- **規模**: 11〜13。
- **触るファイル**: `pasta_shiori` の `actor/{thread,mailbox,marshaling,lifecycle}.rs` と、`shiori4-v3-portable` が作る版 3 の入口の隣に置く版 4 の入口。`pasta_lua` の下回りの口（`runtime/module_registry.rs` の新しいモジュールか、キックと同じ `RuntimeConfig` の差し込み）。キックの即時化で要れば `pasta/shiori/event/kick.lua`。Rust のテスト。ファイルの重なり: `ponytail-shiori`（`actor/`）、`novel-checkpoint-rollback`（`marshaling.rs` の 5 秒）、`ponytail-lua-loader`（`module_registry.rs`）、`novel-talk-flow`・`ponytail-lua-scripts`（`kick.lua`）。
- **要件定義のモデル**: Opus（上の再測定の行のとおり。ABI とスレッドの決まりは shiori4 の正本が決めていて、残りは仕組みの選び方）。
- **見つけた穴**: プロパティの読み出しと、待って再開する仕組みは、pasta に既にある。`act:get_property`（`pasta/shiori/act.lua` 294 行。DSL では `＄x＝＄％prop`）は `\![get,property]` を出して `OnPastaCallBack` を待ち、待ちの表 `CALLBACK.pending`（`pasta/shiori/event/callback.lua`。時間切れの掃除つき）で再開する。上の Current State と Scope はこれに触れていない。`shiori4-v4-lua-api` はこの上に作る（あちらの brief に書いた）。この spec からは、版 4 のときに同じ `get_property` を中継の同期の読み出しで答えられる下回りを用意する。
- **マニュアルの間**: この spec だけが main に入った間は、作者が使える口がまだ無いので、マニュアルに版 4 のことはまだ書かない。
