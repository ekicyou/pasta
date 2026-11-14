# Brief: onhour-talk-recount

> **ステータス**: 未着手（2026-11-14 の `/kiro-discovery`「ぱすた開発」セッションからのヒアリング。ghost_dev の窓際のぱすたさんでの観察から）。版は未定（開発者の判断）。要件定義は Opus。

## Problem

時報（OnHour）が出た直後に、ランダムトーク（OnTalk）が続けて出てしまう。

2026-11-14 に SSP の実機で見つけた。見つけたのは「ぱすた開発」の Claude で、作者の依頼で元旦 0 時の時報を確かめていたときに、SSP のスクリプトログで気づいた。

- ゴースト：ghost_dev の `debug/pasta-in-windows-debug`（窓際のぱすたさん DEBUG・pasta 0.5.x）。開発用の `pasta.toml` は `[ghost] talk_interval_min = 15`・`talk_interval_max = 30`（本番の dist は 180〜300）。
- SSP の開発用パレット「現在時刻の仮想的変更」で 2026/12/31 23:59:30 に進めた。1/1 0:00 に `＊時報00`（`＆＄month＝1 ＆＄day＝1`）が出た。
- その時報が喋り終わった直後の OnSecondChange で、ランダムトーク（男の子の「傘」）がすぐ続けて出た（SSP のスクリプトログで、時報の次のエントリがランダムトーク・同じ分）。
- 本番の 3〜5 分の間隔でも、正時の前に期限が来れば同じことが起きる。
- 作者の期待（報告を聞いての言葉）：「時報トークが出た地点で、次話発動予定時刻はリセットされないとダメ」。

## Current State

2026-11-14（main `c089373d`）に確かめた。正本は `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua`。

- `check_talk` は、正時までの残りが `hour_margin` より短いと発行を見送る。このとき `next_talk_time` を変えないので、期限が過ぎたまま溜まる。
- `M.check_hour`（L188 あたりの候補ループ）は、時報のシーンを返すときに `next_talk_time` を数え直さない。
- そのため、正時の前に期限が来た分が、時報の直後の秒に OnTalk として出る。仮想時刻の飛びは関係ない。
- マニュアル `book/src/lua/shiori-events.md` の「時刻の飛び」節は、スリープ明けを「時報は 1 回出て、おしゃべりはすぐ出る」と書く。これも同じ溜まりから起きている動き。
  - 出どころは completed の `tutorial-ssp-gaps` の design.md L641（「スリープ明けは今と同じ動き（時報は 1 回・おしゃべりはすぐ）のまま（要件 2.4）」）。要件 2.4 の本文は「時報を 1 回だけ出す」だけで、おしゃべりには触れていない。直しても要件 2.4 には反しない。

### 仮実装（未コミット・別の作業ツリー）

調査のサブセッション（「Reset next random-talk time when OnHour fires」）が、作業ツリー `.claude/worktrees/kiro-start-ponytail-shiori-8e713a`（ブランチ `claude/reverent-almeida-cf6cc2`）に置いた。作業ツリーが消えても再現できるよう、要点を写す。

- コード：候補ループで thread が返ったときだけ数え直す。

  ```lua
  if t then
      -- 時報を出したら、次のランダムトークはここから数え直す（正時の前に期限が来た分を溜めない）
      next_talk_time = calculate_next_talk_time(CLOCK.steady(act.req))
      return t
  end
  ```

- テスト（`crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua`）
  - 足した：10:59:40 に `hour_margin` で見送り → 11:00 に時報 → 待ちが 180〜300 秒 → 時報の 179 秒後の `check_talk` が nil。直す前は落ちる。
  - 足した：候補が全部無い（`check_hour - all candidates miss`）とき、`next_talk_time` が 0 のまま。
  - 変えた：スリープ明けのテスト。時報の 1 秒後の `check_talk` を thread から nil の期待へ。
- マニュアル：`shiori-events.md` の `check_hour(act)` 節に 1 行、「時刻の飛び」節のスリープ明けの文、`book/src/internals/shiori.md` の `check_hour` の段落に 1 文。
- `cargo test -p pasta_lua`（Lua 67 スイート）と `cargo test -p pasta_shiori` は通った。SSP 実機では未確認。

## Desired Outcome

- 時報のシーンが出たら、次のランダムトークはその時点から間隔を数え直す。時報の直後に `talk_interval_min` 秒より前の OnTalk は出ない。スリープ明けも例外にしない。
- スリープ明けのおしゃべりは、溜まりに頼らず、`OnSysResume` を合図に意図して出す（新しい既定の動き）。ゴーストが `OnSysResume` を受けていれば出さない。
- マニュアルの動きの説明が、新しい動きと一致している。

## Approach

仮実装の形（`check_hour` が時報のシーンを返すときだけ `next_talk_time` を決め直す）を推奨案とする。`check_talk`・`restart_timers`・`dispatch` は触らない。

スリープ明けの既定の動きの推奨案（discovery の見立て・要件で詰める）：ゴーストが `OnSysResume` を受けていないとき、`OnSysResume` で `next_talk_time` を「今」（`CLOCK.steady`）にするだけにする。OnSysResume の応答にはスクリプトを返さない。こうすると、次の `check_talk` が今の門（喋っている間・時報マージン・止める門・チェイントークの続き）を全部通したうえでおしゃべりを出す。時報を喋っている最中に割り込むおそれもない。

採らない案：`check_talk` が `hour_margin` で見送るたびに数え直す形。正時の直前に毎秒数え直すことになり、時報のシーンが無いゴーストでもトークが遅れる。

## Scope

- **In**: `check_hour` の数え直し、スリープ明け（`OnSysResume`）のおしゃべりの既定の動き、Lua のテスト、マニュアル（`lua/shiori-events.md`・`internals/shiori.md`）。
- **Out**: 間隔の決め方そのもの（`talk_interval_min`/`max`・`hour_margin` の値と読み方）、自動トークの停止・再開の口、チェイントークの仕組み。

## Boundary Candidates

- 数え直しの条件（時報のシーンを出したときだけか）
- スリープ明けの扱い（例外にするか）
- マニュアルの記述

## Out of Boundary

- 時報のシーンの探し方（4 段の候補）と日時変数の転記
- OnTalk の抑止（`ontalk-suppression`・`suppress-ontalk-on-choosing`）の条件

## Upstream / Downstream

- **Upstream**: `alpha02-virtual-event-dispatcher`・`tutorial-ssp-gaps`（仮想時刻・時刻の飛び・`CLOCK.steady`）・`ontalk-suppression`（`自動トーク停止`／`再開`・`restart_timers`）・`chain-talk-interrupt`（`STORE.co_scene`）。すべて completed。
- **Downstream**: ghost_dev のゴースト（窓際のぱすたさん・emo2）。`novel-talk-flow` のランダムトークと時報の停止とは別の道（あちらは止める、こちらは間隔）。

## Existing Spec Touchpoints

- **Extends**: なし（completed の `tutorial-ssp-gaps` のスリープ明けの動きが変わるが、completed は書き換えない）。
- **Adjacent**: `novel-talk-flow`（`virtual_dispatcher.lua` を触る可能性。同じウェーブに置くなら持ち場分け）。

## 要件ディスカッションの議題（ヒアリングで残ったもの）

「ぱすた開発」が discovery に預けた 5 つと、サブセッションが見つけた 1 つ。

1. **時報のシーンが無かったときも数え直すか**（候補 `時報HH` → `OnHourHH` → `時報その他` → `OnHourOther` がどれも無く、何も出さなかったとき）。推奨：数え直さない（何も喋っていないのにトークを遅らせる理由がない）。仮実装もそうしている。
2. **時報マージンで見送った分の扱い**。推奨：見送るときに `next_talk_time` を変えない今の作りのまま（溜まった分は時報の数え直しで消える）。
3. **チェイントークの続き（`STORE.co_scene`）・`M.restart_timers`・止める門（`auto_talk_paused`）を壊さないこと**。見立て：続きは捨てず、間隔が延びるだけで次の OnTalk の番に出る。`restart_timers` は `next_talk_time` を 0 に戻すだけでぶつからない。止める門の間は `check_hour` が呼ばれない。
4. **テスト**：時報を出した直後は `talk_interval_min` が過ぎるまで OnTalk が出ないことを、リポジトリの流儀（`virtual_dispatcher_spec.lua`）で確かめる。仮実装の 3 つ（上の Current State）が下書き。
5. **利用者マニュアルに書く振る舞いの変更か**。推奨：書く（仮想ディスパッチャの `check_hour(act)` 節と「時刻の飛び」節、`internals/shiori.md`）。仮実装の 3 か所が下書き。
6. **スリープ明けを例外にするか** → **決定済み**（2026-11-14 作者。「ぱすた開発」経由）。時報の後の数え直しはスリープ明けも例外にしない（仮実装のまま）。そのうえで、スリープ明けのおしゃべりを `OnSysResume` を合図に出す既定の動きを足す。要件 2.4 の本文には反しないので、completed の `tutorial-ssp-gaps` を開き直す（Path A）必要はなく、この spec で扱う。作者の言葉（原文）：

   > スリープ明けならイベント来るはずだから、ゴースト側でイベントキャプチャしてないなら pasta.dll が OnTalk を即時に投げてしまえばよいかなと思う。スリープ直後はユーザーが見てる可能性が高いから喋らせるのによいタイミングなので

### スリープ明けのおしゃべりの議題（「ぱすた開発」の見立て）

材料：

- `OnSysResume` は、サスペンド（スリープ・休止状態）の解除で来る GET（スクリプトを返せる）。Reference0 は解除の理由で、`normal`（通常）・`auto`（システムのスケジュールによる自動解除）・`critical`（Vista 以降廃止）。対の `OnSysSuspend` は NOTIFY（ukadoc）。
- `crates/pasta_lua/pasta_scripts/pasta/shiori/clock.lua`（L128・L155）は、`OnSysResume`・`OnSysSuspend` を見て、スリープ明けを時刻の飛びに数えない。注釈：SSP は `OnSysSuspend` を復帰してから送り、`OnSysResume` は復帰後の最初の秒より数秒遅れて送る（2026-10-18 実機）。
- ghost_dev の 2 体（窓際のぱすたさん・emo2）は、どちらも `OnSysResume` を辞書でも `REG` でも受けていない（grep で確かめた）。既定の動きがそのまま効く。

議題：

7. **「ゴーストが受けていない」の判定**。見立て：`＊OnSysResume` のシーン（前方一致で探す今の受け口に当たるもの）も `REG.OnSysResume` も無いとき。受けていれば既定の動きは出さない。
8. **順番と割り込み**。正時をまたいだスリープなら、復帰後の最初の OnSecondChange で時報が先に出て、その数秒後に `OnSysResume` が来る。時報を喋っている最中に OnTalk のスクリプトを返すと、SSP が時報を切るおそれがある。見立て：OnSysResume では返さず、`next_talk_time` を今にして次の `check_talk` で出す（上の Approach の推奨案）。時報を出した直後なら、議題 1 の数え直しとどちらを勝たせるかもここで決める。
9. **Reference0 が `auto` のとき**。見立て：ユーザーが見ていない見込みが高いので、即時のおしゃべりを出さない。
10. **チェイントークの続き（`STORE.co_scene`）があるとき**。新しい OnTalk でなく続きを出すか。推奨案の形なら `check_talk` の今の扱い（続きを先に出す）に従う。
11. **止める門（`auto_talk_paused`）の間**。見立て：出さない。推奨案の形なら、門の間は `check_talk` が呼ばれないので自然にそうなる。

## 実機での確かめ

確認用のゴースト（ghost_dev）は「ぱすた開発」セッションが持っている。pasta の側からは触らない。実機で確かめるときは、作者か「ぱすた開発」に声をかける。手順は、見つけたときと同じく、SSP の開発用パレットの仮想時刻で 2026/12/31 23:59:30（正時の 30 秒前）に合わせて 1/1 0:00 の `＊時報00` を出し、時報の直後に `talk_interval_min`（開発用は 15 秒）より前にランダムトークが出ないことを見る。

- ghost_dev のゴーストが読む `virtual_dispatcher.lua` は、pasta.dll が配る `profile/pasta/pasta_scripts` の写しである。直したものを確かめるには、新しい pasta.dll（または確認用のビルド）を `debug/pasta-in-windows-debug` へ入れる必要がある。
- スリープ明けのおしゃべりは、PC を実際にスリープさせて確かめる。作者に頼むことになる（「ぱすた開発」の見込み）。
- 入れるのは ghost_dev 側（「ぱすた開発」）の仕事。pasta 側は、確認に使う dll のありか（パス）を「ぱすた開発」に知らせる。SSP は 32bit なので、x86 のビルド（`i686-pc-windows-msvc`）を渡す。

## Constraints

- pasta の作法（steering・`workflow.md`・CHANGELOG・リリースノート）に従う。プッシュと main への統合は作者の合図があってから。ghost_dev には触らない。
- `cargo test` の前に `NoDefaultCurrentDirectoryInExePath` を外す。テストが書き換えた `sample.generated.lua` は戻す。
- 小さな spec（時報の数え直しはコード 4 行。スリープ明けの既定の動きを足しても、`virtual_dispatcher.lua` とイベントの受け口の周りで収まる見込み）。開発者が spec として立てることを指示した。
