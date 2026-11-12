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
- 0.4.3 の `shiori4-v3-portable` が、shiori4 の版 3 の入口（`shiori_get_api`）と、OS に頼らない実行器を入れる（予定）。
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

## 2026-10-24 shiori4 の host-protocol からの知らせ

shiori4 の spec `host-protocol` の要件が、2026-10-24 に確定した（開発者が全議題を推奨案で承認した）。その後、設計で 1 つ決まった。正本の置き場所は、shiori4 の設計が終わった後に知らせが来る。

- **確定した語のうち、上の Approach と違う所**:
  - プロパティの読みは `ID: getproperty`。0.0.1 の `property` から改めた（互換は無い）。書き込みの `ID: setproperty`（`Reference0` が名前、`Reference1` が値。成功は 204、不正は 400）が増えた。
  - `Reason` は付けなくてもよい。`Reason` の無い 400 は「不正」、`Reason: forbidden` は「拒否」、それ以外の語は「理由不明の拒否」と読む。これはすべての 400 に当てはまる。
  - `OnHostComplete` も「post から起きた要求」に入る。`SenderType: internal`・`SecurityLevel: local` が付く。pasta が断るのは `SecurityLevel: external` だけなので、そのまま通る。
  - `OnHostComplete` の `Status` は、結果のステータスコードを運ぶ。ukadoc の `Status`（ベースウェアの状態）とは意味が違う。今の `dispatcher.is_blocked(req.status)` には通さない。
- **pasta の出発点は変わらない**:
  - shiori4 の `Ghost` は、解析済みのヘッダではなく原文 `&str` を受けるようになる。`Headers`・`HeaderWriter`・`PendingTags`・`Properties`／`Lookup` は、芯から便利関数のモジュールへ移る。
  - pasta は `shiori4::Ghost` を使わず、`shiori4_abi::Shiori` を直に使う（`shiori4-v3-portable`）。上の型はどれも使っていない。
  - `Properties` は `ShioriHostHandle` を持つ。`ShioriHostHandle` は Send でないので、アクタースレッドからの中継には使えない。要求の文字列は、便利関数の純粋な関数か、自前の関数で作る。
- **逆向きの変換は shiori4 では作らない**（2026-10-24 の shiori4 の設計。開発者が確定）:
  - 作らないもの: v4 のヘッダ部分を SHIORI/3.0 の全文に直す変換と、SHIORI/3.0 の応答をステータスとヘッダ部分に分ける変換。
  - 理由: この変換が要るのは、pasta の中が SHIORI/3.0 の全文の形で組まれているためである。開始行を求める pest の文法（`util/parsers/req_parser.pest`）や、開始行と `Charset` を必ず付ける Lua の RES（`shiori/res.lua`）がそうなっている。だからこれは pasta の事柄とされた。
  - `shiori4::v3::{parse_request, response}` は公開のまま残る。
- **開発者の意向**:「ここはもう少しモダンに設計してほしい」。
  - v4 の入口で SHIORI/3.0 の全文に詰め替えて今の解析に流すことはしない。
  - pasta の内部の表現を、SHIORI/3.0 の全文に縛られない形（v4 のヘッダ部分をそのまま扱える形）に寄せる。これをこの spec の設計で検討する。
- **設計の出発点の案**（まだ決まっていない）:
  - **中の表現**: 要求は「メソッド（get／notify）＋ヘッダ部分」、応答は「ステータス＋ヘッダ部分」にする。どちらも `Charset` を持たない。
  - **変換する場所**: 開始行の読み書きと、`Charset` の検査・補いは、旧来の入口（HGLOBAL の `load`・`request`）と版 3 の入口だけが行う。版 3 の入口は `shiori4::v3` を使える。
  - **pest の文法**: ヘッダ部分の規則（`key_values`）と開始行の規則（`header`）を分ける。v4 ではヘッダ部分だけを解析する。SHIORI/2.x の開始行（`header2`）は、旧来の入口の側にだけ残す。
  - **Lua の RES と `error.rs`**: 返すのはステータスとヘッダ部分だけにする。開始行と `Charset` は入口が付ける。応答のゴールデン（`actor/marshaling.rs` の標準ヘッダ 3 種）も見直す。
  - **`req.version`**: 版の数をやめ、入口の種類（v4・v3・旧来）を表す印にする方向で考える。印は、上の In の「版 4 で読まれたかどうかの印」と同じものにする。
  - **入れる時期**: 0.4.3 の `shiori4-v3-portable` は今の全文の形のまま入れる。版 3 の口は全文を受けるので、そのままでよい。作り直しは、この spec（0.5.0）で行う。
- **触るファイル（足すもの）**: `pasta_shiori/src/lua_request.rs`・`util/parsers/{req_parser.pest,req.rs}`・`error.rs`・`actor/marshaling.rs`（ゴールデン）・`pasta_lua/pasta_scripts/pasta/shiori/res.lua`。ファイルの重なりも見直す。`ponytail-shiori`（`windows.rs`・`actor/`）と `ponytail-lua-scripts`（`res.lua`）が重なる。

## shiori4 0.1.0 への合わせ（2026-10-24 の `/kiro-discovery`）

shiori4 の 0.1.0 が 2026-10-24 に出た（crates.io の `shiori4_abi`・`shiori4`・`shiori4_host`、GitHub のタグ `v0.1.0`。shiori4 の PR や教科書の「0.0.3」は 0.1.0 のこと）。`abi-conventions`・`host-protocol` は shiori4 の main に入り、この版で crates.io に出た。上の Constraints の「shiori4 の spec が main に入り、crates.io に出た版を使う」と、2026-10-23 の再測定の「先に要るもの」の shiori4 の分は満ちた。残る先に要るものは `shiori4-v3-portable` と、実機の areka v0.2.0。

- **正本の置き場所**（上の 2026-10-24 の節の「正本の置き場所は後で知らせが来る」の答え）: 規約の正本は C ヘッダ `shiori4_abi/include/shiori4_abi.h` の冒頭と、`shiori4_abi` のクレート冒頭の `//!`。ホストとの語は `doc/host-protocol.md`。読みやすく言い直したものが教科書 `doc/textbook/`（規約の一覧 `rules.md`・移行の手引き `migration.md`・試験用のホスト `testhost.md` など）。食い違ったら正本が正しい。
- **二進互換でない変更（shiori4 #11）と、この spec への効き目**:
  - 応答の `out` は常にホストが作り、SHIORI は `ShioriHostApi` の末尾の `copy_from_slice` で写す。`shiori4_abi` の `ShioriApi::new` が作るテーブルの関数がこれを行うので、pasta は写しのコードを書かない。版 4 の `load` も `copy_from_slice` の欄が無いテーブル（`struct_size` が足りない）なら、`SHIORI_E_UNSUPPORTED` の 1 行を `log` に出して NULL を返す（`shiori4_abi` が行う）。
  - `release` は常にホストの関数で、unload の後でも、どのスレッドからでも呼べる（スレッド規約 6）。join しない pasta のアクターが unload の後に応答を捨てても、規約の内である。
  - Rust の `ShioriHostApi` の `request`・`post`・`log` の欄は `Option` になった。pasta は `ShioriHostHandle`（同期の `request`。SHIORI のスレッドだけ）と `ShioriHostPoster`（`post`・`log`。任意のスレッド）を通して呼ぶので、欄を直に読まない。欄を直に読む作りにするなら、NULL を確かめる。
- **0.1.0 にあって、この spec で使えるもの**:
  - `ShioriHostPoster` は、unload を始めた時点で無効になる（`post` は `SHIORI_E_SHUTDOWN`、`log` は何もしない。どちらもホストに触れない。スレッド規約 5）。上の Approach の「unload を始めた時点で poster を無効にする」は、shiori4 のクレートがこの作りで持っている。pasta が自前で無効化を持つかは、要件で決める。
  - `shiori4` の `Ghost` の文書に、「アクターは `host.request` を呼ばず、問い合わせを channel で応答を待っている SHIORI のスレッドへ送り、SHIORI のスレッドが中継する」例がある（`shiori4/src/lib.rs` の「例: アクターの中継（スレッド規約 2）」）。pasta の中継はこの形で作る。
  - 試験用のホスト `shiori4-testhost`（`shiori4_host` の example）は、`--entry` を省くと版 4 から試し、`-H "名前: 値"` でヘッダを足せる。版 4 を足したら、CI の段（`shiori4-v3-portable` が足す）で版 4 も 1 往復させられる。
  - 書き出しのマクロ `export_shiori!` は、版 4 を求められると版 4 のテーブルを返し、`SUPPORTS_V3` なら版 3 も返す。版 4 を足すこの spec で、`shiori4-v3-portable` の自前の `shiori_get_api`（版 3 だけを返す）を、このマクロに替えてよい。
- **旧来の入口の段の分け方**（`shiori4-v3-portable` の 2026-10-24 の節で決めた）: HGLOBAL の入口は shiori4 のクレートへ移さず pasta に残す。この spec で、従来の口と版 3 の口の開始行と `Charset` の変換に `shiori4::v3::{parse_request, response}` を使えるかを測る（使えば変換は shiori4 に寄る）。`shiori4::v3::parse_request` は SHIORI/2.x の開始行（今の pest の `header2`）を 400 にするので、従来の口で 2.x を受け続けるかも、ここで決める。

## 申し送り（shiori4-v3-portable より・2026-10-25）

- **芯の型**: 版 3 の口と旧来の入口は、どちらも `actor/instance.rs` の `ActorInstance`（1 インスタンス＝1 アクタースレッド・`spawn_with(ClaimMode)`・`get`・`notify`・`kick`・`shutdown(JoinPolicy)`）の上にある。版 4 はこの型に足す（あちらの design.md の Revalidation Triggers）。旧来の入口の置き場は `static LEGACY: ArcSwapOption<ActorInstance>` だけ。
- **書き出し**: `shiori_get_api` は `v3.rs` に自前で書き、版 3 だけにテーブルを返す（版 4 は NULL）。`export_shiori!` へ替えるとき、CI の 1 往復の段（`build.yml`。`--entry v3` と省いた形）と、版 4 が NULL であることの試験（`tests/shiori4_v3_entry_test.rs`）を直す。
- **unload の後にスレッドを残さない**: 版 3 の unload（`ActorInstance` の Drop）は Stop の ack の後に join する。ロガーも書き出し用のスレッドを持たなくなった（`tracing_appender::non_blocking` をやめて同期で書く。ワーカースレッドは guard を落としても join されず、ホストが unload の直後に pasta.dll を解放すると外れたコードの上で落ちていた。shiori4-testhost でほぼ毎回 exit 139）。版 4 で中継やホストへの log の転送のためにスレッドを足すなら、unload の中で join するか、DLL の外のコードだけで動く形にする。
- **ログ**: 登録簿の項目はディレクトリごとにロガー・量の設定・占有を持つ（`try_claim`・`take_over`・`release_claim`）。ホストの `log` へ転送するなら、この振り分け（`CURRENT_LOAD_DIR`）の後ろに足すかを設計で決める。
- **試験用のホスト**: CI は `shiori4_host` 0.1.1 の `shiori4-testhost` を使う（0.1.1 は x86 の Windows に `--entry legacy` を足した。版 3 の道筋は 0.1.0 と同じ）。

## 申し送り（2026-10-29 の版の組み替えより）

- ノベルゲーム（Phase 13）は 0.5.0 から 0.6.0 へ順延した（開発者の決定）。この brief の「0.5.0」は、ノベルゲームの版のことで、今は 0.6.0 と読む。0.5.0 は「属性関係の整備」（`scene-attribute-store`・`scene-condition-attributes`・`scene-pick-modes`）になり、0.4.5（ponytail）の次に入る。

## 申し送り（ponytail-shiori より・2026-11-03）

上の「芯の型」と再測定の節の記述のうち、ponytail-shiori（0.4.5）で次が変わった。測り直すときはこちらを正とする。

- **芯の型 `ActorInstance`**: `kick`（インスタンスの kick）と `shutdown(JoinPolicy)` は消えた。後始末は `shutdown(self)`・`shutdown_with_timeout(self, timeout)` で、ack が届くか受け口が既に閉じていれば join し、ack が来ないときだけ手放す。`JoinPolicy`・`teardown_actor_detached`・DllMain の後始末の道は無い（DllMain は何もせず 1 を返す）。kick の送り先は `kick_sink_for(tx)`（`pub`）で作る。
- **モジュールの参照と join の約束**: アクタースレッドは起こす前に pasta.dll の参照を 1 つ取り（`actor/module_ref.rs` の `ModuleRef`）、約束のチャネル（容量 1）で返す人を決める。join する親が返すか、手放されたスレッドが `FreeLibraryAndExitThread` で返して終わる。版 4 で中継やログ転送のスレッドを足すなら、同じく参照を持たせるか、unload の中で join する。
- **要求の判定**: `determine_method` は消え、`actor/marshaling.rs` の `classify_request`（`RequestKind`: Get・Notify・Shiori2(メソッド)、解けなければ位置つきの誤り）の 1 つになった。解けない要求は両方の入口で warn を 1 行（`seam="request.unreadable"`）残す。
- **文法**: `req_parser.pest` の区切り `_tag` は `":" ~ " "?`（`Key:`・`Key:value` を受ける）。使われない規則 `sp`・`eol`・`key_sep` と `_GRAMMAR` は消えた。`req.rs` のテスト用の写しも消えた。
- **ほか**: `hinst` は `PastaShiori::load(load_dir)` まで消えた（Lua へは定数の 0）。`Shiori` トレイト・`Reply`・`MyError::Poison`・`ActorMsg` の `#[non_exhaustive]`・メッセージの作り手（`stop` 以外）・`spawn_actor_thread`・`actor_thread_id`・`debug_local_addr` は消えた。応答のチャネルは `String`。Rust が作る 204 は `default_204()` だけ。通常の依存の `windows-sys` は `Win32_System_LibraryLoader` だけ（Threading は dev-dependencies）。

## 申し送り（ponytail-lua-debug を閉じた、2026-11-02）

- 上の「先に要るもの」に挙げた `ponytail-lua-debug` は閉じた（roadmap の「統合して閉じたもの（2026-11-02）」）。`module_registry.rs` の重なりは無くなった。0.5.1 で DAP を捨てて、`pasta_lua/src/debug` の大半が消える見込みである。

## 申し送り（2026-11-02 の discovery「Pasta DSL MCP移行検討」より）

- 0.5.0 の `debug-mcp-runtime` が、DAP と一緒に**キックの経路を丸ごと消す**（`event/kick.lua`・`SHIORI.kick`・`STORE.kick_*`・`ActorMsg::Kick`・`kick_sink_for` ほか）。キックを生んでいたのは DAP の `pasta/playSceneAt`・`pasta/reloadShiori` だけである。
- 指名した再生は `OnPastaScene`・`OnPastaSceneAt` に一本化する。MCP の `raise_event` で起こせば、応答でそのまま台本を返す。
- この brief の「キックの即時化」（Problem の 1 つ目・Desired Outcome の「MCP からのシーン再生（キック）が…すぐ出る」・触るファイルの `kick.lua`）は、対象が無くなる。要件を作るときに範囲から外す。post の SEND で台本を今すぐ出す仕組みそのもの（シーンと Lua から）は残る。
- 上の 2026-11-02 の申し送りの「0.5.1 で DAP を捨てて」は、0.5.0 と読み替える。

## 申し送り（debug-mcp-runtime より・2026-11-03）

- キックの経路は消えた。`spawn_actor_thread_with` の引数は `(load_dir, rx, claim)` の 3 つになり、スレッドは既定の `PastaShiori` と既定の `RuntimeConfig` を使う。`RuntimeConfig` に残る欄は `libs`・`scene_aliases` だけ（`kick_sink`・`debug` は無い）。版 4 の口で `RuntimeConfig` に差し込みを足すなら、ここから足す。
- `ActorMsg` は Get・Notify・Stop の 3 つだけ。参照先は `.kiro/specs/completed/debug-mcp-runtime/design.md` の Revalidation Triggers。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化**（前回 2026-11-03・`c575febd` から）: `debug-mcp-runtime` が入った（#233）。キックの道は `actor/` と `shiori.rs` から消え、上の申し送り（`debug-mcp-runtime` より）は main と合っている。`actor/instance.rs` は 271・`mailbox.rs` は 332・`thread.rs` は 263 行に縮んだ。`v3.rs`（207 行）の読み込みのログは `pasta_lua::logging::LOAD_ENTRY_TARGET` を的にした（どのフィルターでも通る info。試験は新しい `tests/v3_load_entry_log_test.rs`）。`shiori_get_api` は 25 行、版 4 が NULL の試験は 185〜190 行、`hinst` に 0 を渡すのは `shiori.rs` 247 行。`act:get_property` は `shiori/act.lua` 322 行（`actor-line-newlines` が `_line_newlines` を足してずれた）。`GET_TIMEOUT`（`marshaling.rs` 559 行）・`build.yml`・`res.lua`・`tests/shiori4_v3_entry_test.rs` は動いていない。#251 は `src/windows_tests.rs` の leak probe の試験に `lock_global_state` を取らせただけで、持ち場に掛からない。
- **触るファイル**: `actor/{instance,thread,mailbox,marshaling,lifecycle,teardown,module_ref}.rs`（271・263・332・559・190・144・123 行）、`v3.rs` か隣の新しい版 4 の口、`shiori.rs`（338）、`lua_request.rs`（151）・`util/parsers/{req_parser.pest,req_parser.rs}`（51・456）・`error.rs`（388）・`pasta_lua/pasta_scripts/pasta/shiori/res.lua`（154）、`pasta_lua` の下回りの口、`.github/workflows/build.yml`（225。版 4 の 1 往復）・`tests/shiori4_v3_entry_test.rs`（283。版 4 が NULL の試験）。1,000 行に近いものは無い。
- **規模**: 14〜17 のまま。
- **先に要るもの**: 働きの上では実機の areka v0.2.0 だけ。ファイルの重なり（未完了）: `novel-checkpoint-rollback`・`novel-areka-contract`（`marshaling.rs`）、`novel-save-slots`・`novel-variable-scope`（`actor/lifecycle.rs`）、`pasta-dll-multiarch`・`pasta-recipe`（`build.yml`）。`debug-mcp-runtime` は入ったので外れた。
- **種別**: 機能（版 4 に本格対応。開発者の方針）。
- **要件定義のモデル**: Opus のまま（スレッドと ABI の決まりは shiori4 の正本が決めていて、`req.version` を入口の印に変える案も 1 つに絞れている）。
- **分割の案**: 要らない。20 を超えるなら、中の表現の作り直しを先の spec に分ける（前のまま）。
- **見つけた穴・古くなった記述**: 版 4 の口の読み込みのログも `LOAD_ENTRY_TARGET` を的にする（`entry = "v4"`。しないとログのフィルターで消える）。前から残るもの: Current State の「`shiori4-v3-portable` が入れる（予定）」・基準の `c360adb2`・「キックは…次の要求の応答で台本を出す（`actor/thread.rs` の KICK）」は古い（入った・消えた）。Existing Spec Touchpoints の Adjacent に `shiori4-v3-portable`（完了。芯の型）が無い。旧来の口で SHIORI/2.x の開始行を受け続けるかは残る議題（`classify_request` の `Shiori2`）。roadmap の台帳の行の「キックの即時化」は対象が無い。
