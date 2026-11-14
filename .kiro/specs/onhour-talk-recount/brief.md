# Brief: onhour-talk-recount

> **ステータス**: 未着手（2026-11-14 の `/kiro-discovery`「ぱすた開発」セッションからのヒアリング。ghost_dev の窓際のぱすたさんでの観察から）。版は未定（開発者の判断）。要件定義は Opus。

## Problem

時報（OnHour）が出た直後に、ランダムトーク（OnTalk）が続けて出てしまう。

- 窓際のぱすたさん（ghost_dev）で、時報のすぐ後にランダムトークが重なって出るのを作者が見た。確かめたときの間隔は開発用の 15〜30 秒（本番は 180〜300 秒）。
- 作者の期待：「時報トークが出た地点で、次話発動予定時刻はリセットされないとダメ」。

## Current State

2026-11-14（main `c089373d`）に確かめた。正本は `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua`。

- `check_talk` は、正時までの残りが `hour_margin` より短いと発行を見送る。このとき `next_talk_time` を変えないので、期限が過ぎたまま溜まる。
- `M.check_hour`（L188 あたりの候補ループ）は、時報のシーンを返すときに `next_talk_time` を数え直さない。
- そのため、正時の前に期限が来た分が、時報の直後の秒に OnTalk として出る。仮想時刻の飛びは関係ない。
- マニュアル `book/src/lua/shiori-events.md` の「時刻の飛び」節は、スリープ明けを「時報は 1 回出て、おしゃべりはすぐ出る」と書く。これも同じ溜まりから起きている動き。

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

1. **時報のシーンが無かったときも数え直すか**。推奨：数え直さない（何も喋っていないのにトークを遅らせる理由がない）。
2. **時報マージン**。推奨：見送るときに `next_talk_time` を変えない今の作りのまま（溜まった分は時報の数え直しで消える）。
3. **チェイントークの続き・`restart_timers`・止める門との関係**。見立て：続き（`STORE.co_scene`）は捨てず、間隔が延びるだけで次の OnTalk の番に出る。`restart_timers` は `next_talk_time` を 0 に戻すだけでぶつからない。止める門（`auto_talk_paused`）の間は `check_hour` が呼ばれない。
4. **スリープ明けを例外にするか**（開発者の判断が要る）。仮実装は作者の決まりどおり、スリープ明けも時報から数え直す形にした。マニュアルの「おしゃべりはすぐ出る」が変わる。推奨：例外にしない（同じ溜まりから起きている動きで、作者の決まりと揃う）。
5. **SSP 実機での確認**。窓際のぱすたさんで、開発用の短い間隔（15〜30 秒）のまま正時をまたいで確かめる（仮想時刻を正時の少し前に合わせる）。ghost_dev には pasta の側から触らない。

## Constraints

- pasta の作法（steering・`workflow.md`）に従う。コミットとプッシュは作者の合図があってから。ghost_dev には触らない。
- `cargo test` の前に `NoDefaultCurrentDirectoryInExePath` を外す。テストが書き換えた `sample.generated.lua` は戻す。
- spec 1 本としてはとても小さい（コード 4 行）。開発者が spec として立てることを指示した。
