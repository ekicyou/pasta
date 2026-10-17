# Brief: tutorial-ssp-gaps

2026-10-17 の `/kiro-discovery` で、`baseware-virtual-time` と `boot-surface-without-dic` をこの spec にまとめた。どちらも `getting-started-story-guide` の完了時の棚卸で起票した（2026-10-10・main `f193c9cb`）。入門ガイドを SSP で確かめたときに見つかった「SSP の動きとのずれ」で、入門ガイドが SSP の動きに合わせた書き方で逃げている 2 か所である。

- 6 段目「時報を確かめるには、次の正時を待つ」（SSP の仮想の時刻が pasta に届かない）
- 準備の章「何も表示されないのが正しい」（辞書が無いと立ち絵が出ない）

ソースのファイルは別々（時刻は `lua_request.rs`・`virtual_dispatcher.lua`、立ち絵は `boot.lua`）だが、どちらも `pasta_scripts/pasta/shiori/event/` の層で、直す先のマニュアルも同じ入門の章と `STAGES.md` である。どちらも「やるかどうか」から決める。1 回の要件定義で 2 つを並べて決め、やらないと決めた側は要件から外す。両方やらないと決まったら、この spec は却下する。

## Problem

### 時報を待たずに確かめられない（旧 `baseware-virtual-time`）

辞書を書く人は、時報（`＊時報12`・`＊時報その他`）を書いても、実際の時計が正時になるまで確かめられない。SSP の開発用パレットには「現在時刻の仮想的変更」があるが、pasta の時報には効かない。入門ガイドの 6 段目は、このため「次の正時を待つ」と教えている。最長で 1 時間待つ手順は、入門の途中に置くには重い。

同じ種類の困りごとがもう 1 つある。チェイントーク（11 段目）の続きは、外から `\![raise,OnTalk]` で起こしたおしゃべりでは出ない。自然なおしゃべりを待つしかない。

### 辞書が無いと何も表示されない（旧 `boot-surface-without-dic`）

初めてゴーストを作る人は、入門ガイドの準備の章で最小一式を置いて SSP で起動する。辞書（`.pasta`）がまだ 1 枚も無いので、立ち絵も吹き出しも出ない。画面には何も現れず、うまくいったのか失敗したのかを見た目で区別できない。ガイドは「何も表示されないのが正しい」と教え、ログのファイルで確かめさせている。最初の成功体験が「何も起きない」になっている。

## Current State

### 時刻

- pasta は、リクエストごとに OS の時計を読む（`crates/pasta_shiori/src/lua_request.rs` の `OffsetDateTime::now_local()`）。上書きの口は `X-Pasta-Time` ヘッダーだけで、SSP はこのヘッダーを送らない。
- 実機の確認（2026-10-10・SSP・v0.3.8 の `pasta.dll`）: 開発用パレットの「現在時刻の仮想的変更」（ダイアログの題は TimeMachine）で時刻を「明日 12:00」に進めて 90 秒待っても、時報は出なかった。暇なときのおしゃべりは、実時間の間隔のままだった。実際の 17:00 には「午後5時になったよ。」が出た。
- SSP が仮想の時刻を SHIORI へどう伝えるか（`OnSecondChange`・`OnMinuteChange` の Reference、別のイベント、何も伝えない）は調べていない。
- 時報とおしゃべりの振り分けは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` にある。チェイントークの続きもここが持つ。外から起こした `OnTalk` は、この振り分けを通らずに同名のシーンへ直接届く。
- 入門ガイド 6 段目（`book/src/getting-started/06-hour.md`）と段階表（`crates/pasta_sample_ghost/STAGES.md` の「確かめるための道具」）は、「仮想時刻は pasta の時報に効かない」と書いている。

### 起動の立ち絵

- 実機の確認（2026-10-10・SSP・v0.3.8 の `pasta.dll`）: 辞書が無いゴーストは、pasta の読み込みには成功する（`profile/pasta/logs/pasta.log` ができる）。`OnBoot` はスクリプトを返さず、サーフェス番号は 2 人とも -1 のまま。立ち絵は、最初の台詞と一緒に出る。タスクバーに SSP のアイコンは出る。
- `OnBoot` の既定の処理は `crates/pasta_lua/pasta_scripts/pasta/shiori/event/boot.lua` にあり、同名のシーンを探すだけである。
- 入門ガイドの `book/src/getting-started/setup.md`「SSP に入れて起動する」は、成功の目印を「何も表示されず、何もしゃべらない」とし、立ち絵が無い間はタスクバーの SSP のアイコンからメニューを開くと案内している。
- 辞書はあるが `＊OnBoot` が無いゴーストでも、同じことが起きる。

## Desired Outcome

- 辞書を書く人が、正時を待たずに時報を確かめられる。入門ガイド 6 段目の手順を、待たない形へ戻せる。
- 起動の立ち絵は、やるなら、辞書や `＊OnBoot` が無くても起動した時点で立ち絵が出る。
- どちらも、やらない・できないと決めた場合は、その判断と理由が記録され、代わりの確かめ方（あれば）が示される。入門ガイドの記述は、決めた結果に合っている。

## Approach

決めていない。要件定義で、2 つを並べて決める。

- **時刻**: 最初に、SSP が仮想の時刻を SHIORI に伝えているかを実機とベースウェアの仕様で調べる。伝えていれば、pasta がその値を時刻として使う。伝えていなければ、SSP の仮想時刻に乗る道は無いので、別の確かめ方（外から起こせるイベントなど）を出すか、やらないと決める。
- **起動の立ち絵**: 案は、`OnBoot`（と `OnGhostChanged`）の既定の処理が、応えるシーンが無いときに立ち絵だけを出すスクリプトを返すこと。どのサーフェスを出すかは、`pasta.toml` の `[actor]` からは決まらない（表情とサーフェス番号の対応は辞書が持つ）ので、そこが論点になる。開発者は、エンジンの既定の動きではなく、ガイドの側で手当てする方向を示している（下の「2026-10-16 開発者の決定」）。

## Scope

- **In**: 仮想の時刻の伝わり方の調査。時報の判定に使う時刻の取り方。外から起こした `OnTalk` とチェイントークの関係の整理。起動の立ち絵をやるかどうかの判断と、やる場合の既定の出力。入門ガイド（6 段目・準備の章・うまく起動しないときの節）と段階表の記述の更新。
- **Out**: SSP の側を変えること。おしゃべりの間隔の設定（`[talk]`）の見直し。デバッグ用の待受（debug transport）の拡張。シェルの絵。辞書があるときの起動の挨拶。SSP の側の表示の決まり。

## Boundary Candidates

- 時刻の取り方（`pasta_shiori` のリクエストの組み立て）。
- 時報・おしゃべりの振り分け（`virtual_dispatcher.lua`）。
- 起動のイベントの既定の処理（`boot.lua` ほか）。
- 入門ガイドと段階表の記述。

## Out of Boundary

- 時報の文法やシーン名の別名（バックログの「既定の別名の追加」）。
- hello-pasta の辞書（6 段目の作例は変えない）。
- 実行時の失敗をバルーンへ出すこと（`failure-output-unification`・`runtime-error-balloon`）。
- `pasta_check` での辞書の検査（`pasta-check-dic-validate`・`pasta-check-validate-edges`）。

## Upstream / Downstream

- **Upstream**: なし。
- **Downstream**: 入門ガイドのスクリーンショット（6 段目と準備の章に載せられる絵が変わる。`getting-started-screenshots` は 0.4.0 で、この spec を待たずに撮る）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `getting-started-screenshots`、`hello-pasta-shell-art`（サーフェス番号）。完了した `getting-started-story-guide`（6 段目の本文と準備の章の終わりを直した経緯。`.kiro/specs/completed/getting-started-story-guide/tasks.md` の Implementation Notes（5.3））、`shiori-test-support-runtime`（`pasta_shiori` の結合テスト）。

## Constraints

- `X-Pasta-Time` の今の働き（テストが使う上書き）を壊さない。
- 配布する `pasta.dll` は 1 種類のまま（辞書を確かめる人のための別ビルドは作らない）。
- 辞書を書いた人が出したくない絵を、pasta が勝手に出さない（意図を推測して救済しない）。
- 既に配布されているゴーストの起動の見え方を変えない。

## 測定

- **規模**: 合わせて 7〜11 タスクの見当（時刻は、伝わっていれば 4〜6。起動の立ち絵は、やる場合で 3〜5）。片方だけやるなら 3〜6。
- **種別**: 作成支援（辞書を確かめる道具）と機能。どちらも、やるかどうかから。急ぐバグではない。
- **要件定義のモデル**: Opus（2026-10-17 の棚卸で Fable から直した。下の再測定を参照。やれるか・やるかを最初に決める）。
- **要件定義の議題**:
  1. SSP は仮想の時刻を SHIORI に伝えているか。伝えていなければ、時刻の分をどうするか（別の確かめ方を出す・取り下げる）。
  2. 仮想の時刻を使うとき、時報だけに効かせるか、おしゃべりの間隔や `＄時１２` などの時刻の変数にも効かせるか。
  3. 外から起こした `OnTalk` でチェイントークの続きを出すか。今の動きを仕様として書くだけにするか。
  4. 入門ガイド 6 段目を仮想時刻の手順へ戻すか（戻すなら、開発者用機能を有効にする節の置き場所も 7 段目から動く）。
  5. 起動の立ち絵をやるか。「辞書が無ければ何も出ないのは正しい」とも言える（下の開発者の決定）。
  6. やるなら、どのサーフェスを出すか（`0`・`10` の決め打ち、`pasta.toml` に書かせる、シェルの既定に任せる）。辞書はあるが `＊OnBoot` が無いときも同じに扱うか。
  7. 入門ガイドの準備の章の成功の目印と、タスクバーからの操作の案内を直すか（エンジンを変えずに、準備の章から `＊OnBoot` を書かせる案も含む）。

## 2026-10-16 開発者の決定（旧 `boot-surface-without-dic`）

- 優先度を下げ、0.4.0 から外した。0.4.0 には要らない。
- 「何も書かなければ何も表示しない」が既定の動きである（開発者）。辞書が無いと、どのサーフェスを出すかも決められず、当て推量になる。
- 入門の体験は、エンジンの既定の動きではなく、ガイドの側で手当てする方向で考える。開発者の見方では、入門の最初の段から `＊OnBoot` を書かせてもよかった（今は、準備の章では何も出ず、1 段目（`01-boot.md`）で最初の辞書を足す）。
- `getting-started-screenshots` の「先に要るもの」から外した。
- やるかどうかは、0.4.0 を出した後の棚卸で見直す。

## 申し送り（touch-poke-shortcuts より、2026-10-16）

- `boot.lua` の登録が `REG.OnBoot = REG.OnBoot or function(act) … end` になった（中身は変えていない）。`choice_select.lua`・`second_change.lua`・新しい `mouse.lua` も同じ形で、既定ハンドラはゴーストが先に登録したハンドラを上書きしない（要件 8.7）。
- `boot.lua` を触るときは、この 1 行の形を保つこと。マニュアル `lua/shiori-events.md` の「既定ハンドラと上書き」は 5 つ共通の決まりとして書いてある。参照: `.kiro/specs/completed/touch-poke-shortcuts/design.md` の MouseRegistration。

## 申し送り（getting-started-screenshots より、2026-10-17）

- 入門ガイドの 13 章に絵が入った（`book/src/img/getting-started/` の 16 枚。撮り方の正本は `book/SCREENSHOTS.md`、置き方は `book/AUTHORING.md` 第 8 節「画像の行」）。執筆規約の撮り直す決まりにより、6 段目の確かめ方や準備の終わりの見え方を変えて画面が変わるときは、影響する絵を同じ変更で撮り直すか、roadmap の「人手の確認が残っている項目」に撮り直しが要る絵（ファイル名）と理由を足す。影響しうる絵: `06-hour.png`（16:00 の実際の正時で撮った）、11 段目の 2 枚（`11-jump-detour.png`・`11-jump-continue.png`。外から起こした `OnTalk` が挟まると続きが出ないので、寄り道も続きも自然なおしゃべりで撮った。論点 3 を変えると撮り方が変わる）。準備の章（`setup.md`）には絵を置いていないので、論点 7 で準備の章の見え方を変えても撮り直す絵は無い（置くかどうかは別に決める）。

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**: 入門ガイドに絵が入った（上の申し送り）。止まるエラーをバルーンに出す変更（`runtime-error-balloon`）は、イベントの振り分けの本体 `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` に控えを 2 か所足した（`EVENT.drive` が失敗したときの `STOP.hold`、`EVENT.fire` でハンドラを呼ぶ直前の `STOP.hold_handler`）。新しいファイル `shiori/stop.lua` は止まった場所の表記を作るだけで、時刻と起動の処理には関わらない。この spec が予定するファイル（時刻を作る `crates/pasta_shiori/src/lua_request.rs`、時報とおしゃべりの振り分け `shiori/event/virtual_dispatcher.lua`、起動の既定の処理 `shiori/event/boot.lua`）は、どれも変わっていない。「Current State」は今のコードと合う（時刻は `lua_request.rs` 34 行で OS の時計から取り、上書きは 59 行の `X-Pasta-Time` だけ）。
- **外から起こした OnTalk の続きが出ない訳**: `\![raise,OnTalk]` は振り分けを通らず、`init.lua` の `EVENT.fire` から `EVENT.no_entry`（175 行）で新しい場面として始まる。その場面が、止まっていた場面（チェイントークの続き）を差し替える。続きを拾うのは `virtual_dispatcher.lua` 202〜204 行（自然なおしゃべりのときだけ通る）だけである。論点 3 で動きを変えるなら `init.lua` の `EVENT.fire`・`EVENT.no_entry` を触ることになり、上の `STOP.hold_handler` をハンドラの直前に残す。
- **触るファイル**: 時刻の分は `lua_request.rs`（159 行）と `virtual_dispatcher.lua`（288 行）、論点 3 をやるなら `event/init.lua`（260 行）。起動の立ち絵の分は `boot.lua`（22 行）。テストは `crates/pasta_lua/tests/lua_specs/` と `crates/pasta_shiori/tests/`。マニュアルは入門の `06-hour.md`・`setup.md`・`11-jump.md`、段階表 `crates/pasta_sample_ghost/STAGES.md`。撮り直すなら `book/src/img/getting-started/06-hour.png` と 11 段目の 2 枚。新しいファイルは無い見込み。1,000 行に近いものは無い。
- **規模**: 7〜11 タスク（前回と同じ）。SSP が仮想の時刻を伝えていなければ、時刻の分は 2〜3 タスク（調べた結果の記録と文書）に縮む。
- **先に要るもの**: 機能として要るものは無い。ファイルの重なり: `novel-talk-flow`（`virtual_dispatcher.lua`・`event/init.lua`、起動時の場面で `boot.lua` の見込み）、`novel-checkpoint-rollback`（`virtual_dispatcher.lua`・`event/init.lua`）、`shell-element-click`（`event/init.lua`）。この 3 本とは同じウェーブに置かない。0.5.0 の先頭の 2 本（`novel-areka-contract`・`scene-attribute-store`）とは重ならない。
- **種別**: 機能（辞書を確かめる道具）と文書。バグではない。今の動きは誤りではなく（pasta は OS の時計で正時を判定し、ガイドもそう書いている）、辞書が無ければ何も出さないのは開発者が決めた既定の動きである。
- **要件定義のモデル**: Opus に下げる。SSP が仮想の時刻を SHIORI に伝えるかは、公式の文書では決まらない（UKADOC の開発用パレットの説明は「SSP が認識している現在日時を置き換える」とだけ書き、影響を受けると書いてあるのは日付だけを送る `OnNotifyHolidayInfo` だけ。`OnMinuteChange` などの Reference に時刻は無い）。けれども、決めることは時刻の取り方と起動の既定の出力で、新しい文法・保存の形・公開の Lua の約束ではなく、後から戻せる。実機で確かめれば決まる。
- **分割の案**: 分けない（両方やっても 11 タスク）。
- **見つけた穴・古くなった記述**:
  - 「測定」の「要件定義のモデル: Fable」は、上のとおり Opus に直す。
  - 時刻の分の最初の調べは、開発用パレットの「SHIORI呼出ログ有効」で、仮想の時刻を入れた後のリクエストに時刻らしい値が載るかを見るのが早い。載らなければ、時刻の分は「別の確かめ方を示す」か「やらない」に絞られる。
  - 仮想の時刻を使うと、`act.req.date` と日時の変数（`＄時１２` など）の値の出どころが変わる。テストが使う `X-Pasta-Time` の上書きとの優先順位を、要件に書く。

## 版の決定（2026-10-17 の棚卸の後）

- 開発者の決定で 0.4.1 に入れた。`pasta-check-validate-edges` と同じウェーブに並べる（互いのソースもマニュアルのページも重ならない）。
- 起動の立ち絵の分も含め、やるかどうかは要件定義の最初に決める（上の「0.4.0 を出した後の棚卸で見直す」は、この決定で置き換える）。
