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

- 時報のシーンが出たら、次のランダムトークはその時点から間隔を数え直す。時報の直後に `talk_interval_min` 秒より前の OnTalk は出ない。
- マニュアルの動きの説明が、新しい動きと一致している。

## Approach

仮実装の形（`check_hour` が時報のシーンを返すときだけ `next_talk_time` を決め直す）を推奨案とする。`check_talk`・`restart_timers`・`dispatch` は触らない。

採らない案：`check_talk` が `hour_margin` で見送るたびに数え直す形。正時の直前に毎秒数え直すことになり、時報のシーンが無いゴーストでもトークが遅れる。

## Scope

- **In**: `check_hour` の数え直し、Lua のテスト、マニュアル（`lua/shiori-events.md`・`internals/shiori.md`）、要件の議題になったスリープ明けの扱い。
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
6. **スリープ明けを例外にするか**（開発者の判断が要る）。仮実装は作者の決まりどおり、スリープ明けも時報から数え直す形にした。マニュアルの「おしゃべりはすぐ出る」が変わる。推奨：例外にしない（同じ溜まりから起きている動きで、作者の決まりと揃う）。要件 2.4 の本文には反しないので、completed の `tutorial-ssp-gaps` を開き直す（Path A）必要はなく、この spec で扱う。

## 実機での確かめ

確認用のゴースト（ghost_dev）は「ぱすた開発」セッションが持っている。pasta の側からは触らない。実機で確かめるときは、作者か「ぱすた開発」に声をかける。手順は、見つけたときと同じく、SSP の開発用パレットの仮想時刻で 2026/12/31 23:59:30（正時の 30 秒前）に合わせて 1/1 0:00 の `＊時報00` を出し、時報の直後に `talk_interval_min`（開発用は 15 秒）より前にランダムトークが出ないことを見る。

- ghost_dev のゴーストが読む `virtual_dispatcher.lua` は、pasta.dll が配る `profile/pasta/pasta_scripts` の写しである。直したものを確かめるには、新しい pasta.dll（または確認用のビルド）を `debug/pasta-in-windows-debug` へ入れる必要がある。
- 入れるのは ghost_dev 側（「ぱすた開発」）の仕事。pasta 側は、確認に使う dll のありか（パス）を「ぱすた開発」に知らせる。SSP は 32bit なので、x86 のビルド（`i686-pc-windows-msvc`）を渡す。

## Constraints

- pasta の作法（steering・`workflow.md`・CHANGELOG・リリースノート）に従う。プッシュと main への統合は作者の合図があってから。ghost_dev には触らない。
- `cargo test` の前に `NoDefaultCurrentDirectoryInExePath` を外す。テストが書き換えた `sample.generated.lua` は戻す。
- spec 1 本としてはとても小さい（コード 4 行）。開発者が spec として立てることを指示した。
