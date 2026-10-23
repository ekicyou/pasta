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
  - 1 プロセスに複数のインスタンスを置くこと。`shiori4-v3-portable` の要件で決める（推奨はあちらで入れる）。あちらで入らなかったときだけ、この spec の In に移す

## Boundary Candidates

- Rust 側の版 4 の土台（ホストのテーブルの保持・スレッドの中継・post と Tag・unload での無効化）
- Lua の API と DSL の書き方（作者が触る口）
- キックの即時化（デバッグの経路）

規模が 20 タスクを超えるなら、Rust 側の土台と、Lua・DSL の口とで分ける。

## Out of Boundary

- `novel-*` の各 spec の演出の通り道（SHIORI とさくらスクリプトのまま。版 4 を前提にしない）
- SHIORI/3.0 の要求の解析

## Upstream / Downstream

- **Upstream**: `shiori4-v3-portable`（入口と実行器）。shiori4 の `abi-conventions`・`host-protocol`（公開された版に依存する）。areka の版 4 の対応（実機の確認に要る）
- **Downstream**: ノベルゲームの演出で、表示の終わりを待つ書き方を使いたい spec（あれば）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `scene-play-event`（完了。キックの経路）、`chain-talk-interrupt`（チェイントークの続きは pasta の中にあり、SEND の割り込みとは別物）、`ponytail-shiori`（`actor/` を共有する）

## Constraints

- 版 4 を使うゴーストでも、版 3 と SHIORI/3.0 で読み込まれたときに壊れない（できないことは、できないと分かる形で返す）
- ゴーストの振る舞いは DSL で書けるようにする（Lua だけの口にしない）
- pasta.dll の出荷物は 1 種類のまま。静的 CRT リンクを保つ
- shiori4 の spec が main に入り、crates.io に出た版を使う
