# Brief: baseware-virtual-time

`getting-started-story-guide` の完了時の棚卸で起票（2026-10-10・main `f193c9cb`）。

## Problem

辞書を書く人は、時報（`＊時報12`・`＊時報その他`）を書いても、実際の時計が正時になるまで確かめられない。SSP の開発用パレットには「現在時刻の仮想的変更」があるが、pasta の時報には効かない。入門ガイドの 6 段目は、このため「次の正時を待つ」と教えている。最長で 1 時間待つ手順は、入門の途中に置くには重い。

同じ種類の困りごとがもう 1 つある。チェイントーク（11 段目）の続きは、外から `\![raise,OnTalk]` で起こしたおしゃべりでは出ない。自然なおしゃべりを待つしかない。

## Current State

- pasta は、リクエストごとに OS の時計を読む（`crates/pasta_shiori/src/lua_request.rs` の `OffsetDateTime::now_local()`）。上書きの口は `X-Pasta-Time` ヘッダーだけで、SSP はこのヘッダーを送らない。
- 実機の確認（2026-10-10・SSP・v0.3.8 の `pasta.dll`）: 開発用パレットの「現在時刻の仮想的変更」（ダイアログの題は TimeMachine）で時刻を「明日 12:00」に進めて 90 秒待っても、時報は出なかった。暇なときのおしゃべりは、実時間の間隔のままだった。実際の 17:00 には「午後5時になったよ。」が出た。
- SSP が仮想の時刻を SHIORI へどう伝えるか（`OnSecondChange`・`OnMinuteChange` の Reference、別のイベント、何も伝えない）は調べていない。
- 時報とおしゃべりの振り分けは `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` にある。チェイントークの続きもここが持つ。外から起こした `OnTalk` は、この振り分けを通らずに同名のシーンへ直接届く。
- 入門ガイド 6 段目（`book/src/getting-started/06-hour.md`）と段階表（`crates/pasta_sample_ghost/STAGES.md` の「確かめるための道具」）は、「仮想時刻は pasta の時報に効かない」と書いている。

## Desired Outcome

- 辞書を書く人が、正時を待たずに時報を確かめられる。
- 入門ガイド 6 段目の手順を、待たない形へ戻せる。
- できないと分かった場合は、その理由が記録され、代わりの確かめ方（あれば）が示される。

## Approach

決めていない。最初に、SSP が仮想の時刻を SHIORI に伝えているかを実機とベースウェアの仕様で調べる。伝えていれば、pasta がその値を時刻として使う。伝えていなければ、SSP の仮想時刻に乗る道は無いので、別の確かめ方（外から起こせるイベントなど）を出すか、やらないと決める。

## Scope

- **In**: 仮想の時刻の伝わり方の調査。時報の判定に使う時刻の取り方。外から起こした `OnTalk` とチェイントークの関係の整理。入門ガイド 6 段目と段階表の記述の更新。
- **Out**: SSP の側を変えること。おしゃべりの間隔の設定（`[talk]`）の見直し。デバッグ用の待受（debug transport）の拡張。

## Boundary Candidates

- 時刻の取り方（`pasta_shiori` のリクエストの組み立て）。
- 時報・おしゃべりの振り分け（`virtual_dispatcher.lua`）。
- 入門ガイドと段階表の記述。

## Out of Boundary

- 時報の文法やシーン名の別名（バックログの「既定の別名の追加」）。
- hello-pasta の辞書（6 段目の作例は変えない）。

## Upstream / Downstream

- **Upstream**: なし。
- **Downstream**: `getting-started-screenshots`（6 段目の絵を撮るとき、狙った時報を出せるかに効く）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `shiori-test-support-runtime`（`pasta_shiori` の結合テスト）、`getting-started-screenshots`。完了した `getting-started-story-guide`（6 段目の本文と要件 4.2 の訂正の経緯。`.kiro/specs/completed/getting-started-story-guide/tasks.md` の Implementation Notes（5.3））。

## Constraints

- `X-Pasta-Time` の今の働き（テストが使う上書き）を壊さない。
- 配布する `pasta.dll` は 1 種類のまま（辞書を確かめる人のための別ビルドは作らない）。

## 2026-10-10 起票時の測定（main f193c9cb）

- **規模**: 調査の結果による（伝わっていれば 4〜6 タスク）。
- **種別**: 作成支援（辞書を確かめる道具）。急ぐバグではない。
- **要件定義のモデル**: Fable（やれるか・やるかを最初に決める）。
- **要件定義の議題**:
  1. SSP は仮想の時刻を SHIORI に伝えているか。伝えていなければ、この spec をどうするか（別の確かめ方を出す・取り下げる）。
  2. 仮想の時刻を使うとき、時報だけに効かせるか、おしゃべりの間隔や `＄時１２` などの時刻の変数にも効かせるか。
  3. 外から起こした `OnTalk` でチェイントークの続きを出すか。今の動きを仕様として書くだけにするか。
  4. 入門ガイド 6 段目を仮想時刻の手順へ戻すか（戻すなら、開発者用機能を有効にする節の置き場所も 7 段目から動く）。
