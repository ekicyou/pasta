# Brief: onhour-talk-recount

> **ステータス**: 未着手（2026-11-14 の `/kiro-discovery`「ぱすた開発」セッションからのヒアリング。ghost_dev の窓際のぱすたさんでの観察から。聞き取りの全文は末尾の「聞き取りの記録」）。版は未定（開発者の判断）。要件定義は Opus。

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

## 聞き取りの記録

2026-11-14 の `/kiro-discovery`（開発者の指示「「ぱすた開発」セッションからヒアリングを行い、spec を立ち上げよ」）で集めた材料を、漏れなく残す。上の節はこれを整理したもの。食い違うときはこの記録の原文が正しい。

### 関わったセッション

| セッション | 場所 | 役目 |
|---|---|---|
| 「ぱすた開発」 | ghost_dev（`C:\home\maz\git\ghost_dev`） | 窓際のぱすたさんの担当。現象を見つけ、依頼文を組み立てた。確認用のゴーストを持つ |
| 「Reset next random-talk time when OnHour fires」 | pasta の作業ツリー `.claude/worktrees/kiro-start-ponytail-shiori-8e713a`（ブランチ `claude/reverent-almeida-cf6cc2`） | 「ぱすた開発」から枝分かれしたサブセッション。調査と仮実装（未コミット）。kiro-discovery は走らせていない |
| この discovery のセッション | pasta の作業ツリー `kiro-start-ponytail-manual-ci-80ebb3`（ブランチ `claude/pasta-dev-spec-kickoff-2c3a3c`） | 聞き取りと起票 |

### 経緯

1. 「ぱすた開発」が、作者の依頼で元旦 0 時の時報を SSP の実機で確かめていて、時報のすぐ次にランダムトークが出るのを SSP のスクリプトログで見つけた。作者はそれを聞いて「時報トークが出た地点で、次話発動予定時刻はリセットされないとダメ」と言った。
2. 「ぱすた開発」はサブセッションを立て、調査と仮実装をさせた。最初は kiro-discovery まで頼んだが、作者の指摘（pasta の環境で立ち上げるべき）で「調査と仮実装まで」に改めた。
3. 「ぱすた開発」は pasta の環境の作業の候補（チップ「Discover: recount next talk time after OnHour」）を出したが、作者は pasta の組でこのセッションを開いて `/kiro-discovery` を指示した。チップは要らなくなった。
4. このセッションが「ぱすた開発」とサブセッションの記録を読み、依頼文の全文を頼み、brief を書いた。その後「ぱすた開発」から、依頼文の全文・brief への指摘 3 点・スリープ明けの作者の判断が届き、そのつど brief に入れた。
5. 開発者の指示「起票だけせよ。セッション記録ももれなく起票してね。PRしてスクワッシュマージ」で、この記録を足して起票を閉じた（`/kiro-start` はまだ走らせていない）。

### 現象（「ぱすた開発」・2026-11-14・SSP 実機）

- ゴースト：ghost_dev の `debug/pasta-in-windows-debug`（窓際のぱすたさん DEBUG・pasta 0.5.x）。開発用の `pasta.toml` は `[ghost] talk_interval_min = 15`・`talk_interval_max = 30`（本番の dist は 180〜300）。
- SSP の開発用パレット「現在時刻の仮想的変更」で 2026/12/31 23:59:30 に進め、1/1 0:00 に `＊時報00`（`＆＄month＝1 ＆＄day＝1`）が出た。
- その時報が喋り終わった直後の OnSecondChange で、ランダムトーク（男の子の「傘」）がすぐ続けて出た（SSP のスクリプトログで、時報の次のエントリがランダムトーク・同じ分）。

### 原因（「ぱすた開発」の所見）

ゴーストに配られた `pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` を読んだ。正本は `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua`。

- `M.check_hour` は時報を発行するとき `next_hour_unix` だけを更新し、`next_talk_time` を数え直さない（配布版の 178 行あたり「次の正時を更新」）。
- `M.check_talk` は時報マージン（`time_to_hour > 0 and time_to_hour < cfg.hour_margin`）で見送るとき、`next_talk_time` を更新せずに `return nil` する（配布版の 216〜220 行あたり）。そのため正時の前に期限が来たトークが溜まり、時報が終わって status が空いた最初の OnSecondChange ですぐ OnTalk が出る。
- 仮想時刻の飛びは関係しない（トーク間隔は `CLOCK.steady` で、飛びを差し引いて数えている）。本番の 3〜5 分の間隔でも、正時の前に期限が来れば同じことが起きる。

### 作者の期待

「時報トークが出た地点で、次話発動予定時刻はリセットされないとダメ」。時報を発行したら、その時点から `calculate_next_talk_time(CLOCK.steady(act.req))` で次のランダムトーク時刻を数え直す。

### 「ぱすた開発」が discovery に預けた依頼文

`/kiro-discovery 時報（OnHour）を出したら、次のランダムトーク（OnTalk）の発動予定時刻を数え直す`

決めてほしいこと（5 つ）：

1. 時報の候補シーン（時報HH → OnHourHH → 時報その他 → OnHourOther）がどれも無く、何も出さなかったときにも数え直すか（出したときだけが自然という見立て。仮実装もそうしている）。
2. 時報マージンで見送った分の扱い（時報の発行での数え直しで足りるか。仮実装は見送り時は今のまま）。
3. チェイントークの続き（`STORE.co_scene`）・`M.restart_timers`・止める門（`auto_talk_paused`）との関係を壊さないこと。
4. テスト：時報を出した直後は `talk_interval_min` が過ぎるまで OnTalk が出ないことを確かめるもの（リポジトリの流儀で）。
5. 利用者マニュアル（仮想ディスパッチャ・時報マージンの節）に書くべき振る舞いの変更か。

加えて、サブセッションが見つけた議題：既存 spec `tutorial-ssp-gaps` の要件 2.4 とマニュアルの「スリープ明けはおしゃべりがすぐ出る」が変わる。スリープ明けだけ例外にするかは作者の判断。既存 spec の拡張として扱うかもここで決まる。

守ること：

- pasta の作法（kiro の仕様駆動・ブランチ運用・CHANGELOG・リリースノート）はリポジトリの CLAUDE.md とステアリングに従う。
- コミット・プッシュは作者の合図があってから。
- 確認用のゴースト（ghost_dev）は「ぱすた開発」が持っているので触らない。実機での確かめが要るときは、作者か「ぱすた開発」に声をかける（仮想時刻で 12/31 23:59:30 → 1/1 0:00 を通して確かめる）。

### サブセッションの報告（調査と仮実装）

#### 正本と差分の要点

- 正本：`crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua`。
- `M.check_hour` の候補ループ（190 行あたり）で、`create_scene_thread` が thread を返したときだけ、`next_talk_time = calculate_next_talk_time(CLOCK.steady(act.req))` を入れてから `return t` する。`check_talk`・`restart_timers`・`dispatch` は触っていない。
- マニュアル：`book/src/lua/shiori-events.md` の「check_hour(act)」節に 1 行（時報のシーンが見つかったら次のトーク時刻を決め直す。直後は `talk_interval_min` 秒より前に OnTalk は出ない。候補が無ければ変えない）。同じファイルの「時刻の飛び」節の「スリープ明けは時報 1 回・おしゃべりはすぐ出る」を「おしゃべりは時報から数え直して待つ（時報のシーンが無ければすぐ出る）」に。`book/src/internals/shiori.md` の `check_hour` の段落にも同じ旨を 1 文。

#### 見立て

- 時報のシーンが無かったとき：数え直さない（出したときだけ）。何も喋っていないのに次のトークを遅らせる理由がない。テストで確かめた。
- 時報マージン：`check_talk` が見送るときに `next_talk_time` を変えないのは今のまま。溜まった分は時報の発行で数え直されて消えるので、これで足りる。見送りのたびに数え直すと、正時の直前に毎秒数え直すことになり、時報が無い場合にもトークが遅れるので採らない。
- `co_scene`（チェイントークの続き）：捨てない。次に OnTalk の番が来たときに続きが出る（間隔が延びるだけ）。`restart_timers` は `next_talk_time` を 0 に戻すだけなので衝突しない。止める門（`auto_talk_paused`）の間は `check_hour` が呼ばれないので影響しない。
- 決めることが 1 つ：`tutorial-ssp-gaps` の要件 2.4 と、マニュアルの「スリープ明けはおしゃべりがすぐ出る」が変わる。あれも同じ溜まりで起きていた動き。作者の決まりどおりにスリープ明けも数え直す形にしたが、スリープ明けだけ例外にするかは作者の判断待ち。

#### テスト（`crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua`）

- 足した：「正時の前に期限が来たおしゃべりは、時報の直後に出ず、時報から talk_interval_min 秒待つ」。10:59:40 に `hour_margin` で見送り → 11:00 に時報 → 待ちが 180〜300 秒の範囲 → 時報の 179 秒後に `check_talk` が nil。直す前のコードでは落ちる形。
- 足した：候補が全部無いとき（`check_hour - all candidates miss`）に `next_talk_time` が 0 のまま変わらないこと。
- 変えた：スリープ明けのテスト。時報の 1 秒後の `check_talk` を thread から nil の期待に変え、名前も変えた。
- 結果：`cargo test -p pasta_lua`（Lua 67 スイートを含む）と `cargo test -p pasta_shiori` が全部通った。テストが書き換えた `sample.generated.lua` は `git checkout` で戻した。SSP 実機では確かめていない（ghost_dev には触っていない）。

### この discovery で確かめたこと

- 「スリープ明けはおしゃべりがすぐ出る」の出どころは、completed の `tutorial-ssp-gaps` の design.md L641（「スリープ明けは今と同じ動き（時報は 1 回・おしゃべりはすぐ）のまま（要件 2.4）」）。要件 2.4 の本文は「時報を 1 回だけ、進んだ先の時刻で出す」だけで、おしゃべりには触れていない。直しても要件 2.4 には反しないので、Path A（completed の spec を開き直す）にせず、新しい spec 1 本（Path C）にした。

### brief への指摘（「ぱすた開発」・3 点）

1. 見つけたのは作者でなく「ぱすた開発」の Claude。作者の依頼で元旦 0 時の時報を確かめていて、SSP のスクリプトログで時報のすぐ次にランダムトーク（男の子「傘」）が出ているのを見た。作者はそれを聞いて「時報トークが出た地点で、次話発動予定時刻はリセットされないとダメ」と言った。
2. 確かめた条件：SSP 開発用パレットの仮想時刻で 2026/12/31 23:59:30 に進め、1/1 0:00 の `＊時報00`（`＆＄month＝1 ＆＄day＝1`）を出したとき。実機確認もこの手順（正時の 30 秒前に合わせる）で再現できる。
3. 実機確認の段取り：ghost_dev のゴーストが読む `virtual_dispatcher.lua` は、pasta.dll が配る `profile/pasta/pasta_scripts` の写し。直したものを実機で確かめるには、新しい pasta.dll（または確認用のビルド）を ghost_dev の `debug/pasta-in-windows-debug` へ入れる手順が要る。入れるのは ghost_dev 側（「ぱすた開発」）の仕事なので、pasta 側は確認に使う dll のありかを知らせる。

### スリープ明けの作者の判断（「ぱすた開発」経由・2026-11-14）

原文：

> スリープ明けならイベント来るはずだから、ゴースト側でイベントキャプチャしてないなら pasta.dll が OnTalk を即時に投げてしまえばよいかなと思う。スリープ直後はユーザーが見てる可能性が高いから喋らせるのによいタイミングなので

「ぱすた開発」の読み：時報の後の数え直しはスリープ明けも例外にしない（仮実装のまま）。そのうえで、スリープ明けのおしゃべりは、溜まりに頼らず `OnSysResume` を合図に意図して出す、という新しい既定の動きを足す。

#### 「ぱすた開発」が調べた材料

- `OnSysResume`（ukadoc）：サスペンド（スリープ・休止状態）の解除で発生。GET なのでスクリプトを返せる。Reference0 は解除理由で、`normal`（通常）・`auto`（システムのスケジュールによる自動解除）・`critical`（Vista 以降廃止）。対になる `OnSysSuspend` は NOTIFY。
- pasta の今の扱い：`crates/pasta_lua/pasta_scripts/pasta/shiori/clock.lua` が `OnSysResume`・`OnSysSuspend` を observe して、スリープ明けを時刻の飛びに数えないようにしている。注釈に「SSP は OnSysSuspend を復帰してから送り、OnSysResume は復帰後の最初の秒より数秒遅れて送る（2026-10-18 実機）」とある（この discovery で L128・L155 を確かめた）。
- ghost_dev の 2 体（窓際のぱすたさん・emo2）は、どちらも `OnSysResume` を辞書でも `REG` でも受けていない（grep で確認）。既定の動きがそのまま効く。

#### 「ぱすた開発」の見立て（要件で決めてほしいこと）

1. 「ゴースト側でイベントキャプチャしていない」の判定：`＊OnSysResume` のシーンも `REG.OnSysResume` も無いとき、が自然。ゴーストが受けていれば既定の動きは出さない。
2. 順番：注釈どおりなら、復帰後の最初の OnSecondChange で時報が先に出る（正時をまたいだスリープなら）。その数秒後に `OnSysResume` が来る。時報を喋っている最中に OnTalk のスクリプトを返すと、SSP が時報を切ってしまうおそれがある。喋っている間（status に talking など）は、返さずに次の `check_talk` で出す（`next_talk_time` を今にする）形が安全かもしれない。
3. Reference0 が `auto`（スケジュールによる自動復帰）のときは、ユーザーが見ていない見込みが高いので、即時のおしゃべりを出さない案。
4. チェイントークの続き（`STORE.co_scene`）があるときは、新しい OnTalk でなく続きを出すか。
5. 止める門（`auto_talk_paused`）の間は出さない。

ghost_dev 側：実機確認は、時報の数え直しと一緒に「ぱすた開発」が受け持つ。スリープ明けは実機で PC をスリープさせる必要があるので、作者に頼むことになるかもしれない。

### まだ確かめていないこと（ギャップ分析へ）

- `＊OnSysResume` のシーンも `REG.OnSysResume` も無いとき、今のイベントの受け口（`pasta_scripts/pasta/shiori/event/init.lua` の `SCENE.co_exec` の前方一致の探し方）が何を返すか。「受けていない」の判定の置き場所。
- 仮実装のテストの結果は、サブセッションの報告だけ。この discovery では走らせていない。

### 仮実装の差分（全文）

サブセッションの作業ツリー（base `c089373d`）の `git diff` をそのまま写す。作業ツリーが消えても、ここから戻せる。

```diff
diff --git a/book/src/internals/shiori.md b/book/src/internals/shiori.md
index 9d5fc3b6..b6002788 100644
--- a/book/src/internals/shiori.md
+++ b/book/src/internals/shiori.md
@@ -515,7 +515,7 @@ dispatch(act)
 
 利用者から見た判定の順・ブロック対象のキーワードと照合の仕方・OnHour のシーン候補の順・トーク間隔の決まり方・テスト用の関数は、[仮想ディスパッチャ](../lua/shiori-events.md#仮想ディスパッチャ)（[ブロック対象 Status キーワード](../lua/shiori-events.md#ブロック対象-status-キーワード)・[OnHour](../lua/shiori-events.md#onhour)・[pasta.toml 設定](../lua/shiori-events.md#pastatoml-設定)・[テスト用関数](../lua/shiori-events.md#テスト用関数)）が正である。ここではそれを実現する仕組みだけを書く。
 
-- `check_hour` の現在時刻は `CLOCK.now(act.req)`（仮想時刻。下の「仮想時刻の時計」）である。`next_hour_unix` が 0 なら次の正時（`unix - unix % 3600 + 3600`）を記録するだけで終わる。`CLOCK.epoch()` が `seen_epoch` と違えば（時刻の飛びがあれば）、`seen_epoch` を更新し、現在時刻の 70 秒前（ローカル定数 `REBASE_GRACE`）から同じ式で次の正時を決め直す。現在時刻がそれに達していれば、同じ式で次の正時を記録し直し、`act:transfer_date_to_var()` で日時変数を転記してからシーンを探す。シーン名の `{HH}` は `CLOCK.date(act.req).hour` から作る。
+- `check_hour` の現在時刻は `CLOCK.now(act.req)`（仮想時刻。下の「仮想時刻の時計」）である。`next_hour_unix` が 0 なら次の正時（`unix - unix % 3600 + 3600`）を記録するだけで終わる。`CLOCK.epoch()` が `seen_epoch` と違えば（時刻の飛びがあれば）、`seen_epoch` を更新し、現在時刻の 70 秒前（ローカル定数 `REBASE_GRACE`）から同じ式で次の正時を決め直す。現在時刻がそれに達していれば、同じ式で次の正時を記録し直し、`act:transfer_date_to_var()` で日時変数を転記してからシーンを探す。シーン名の `{HH}` は `CLOCK.date(act.req).hour` から作る。シーンが見つかったときだけ、`next_talk_time` を `CLOCK.steady(act.req)` から決め直してから返す（正時の前に `hour_margin` で見送ったトーク時刻を、時報の直後に持ち越さない）。
 - `check_talk` は、`next_talk_time` が 0 なら次のトーク時刻を決めるだけで終わる。トーク時刻は `CLOCK.steady(act.req)`（飛びを差し引いた時計）で数え、仮想時刻の設定し直しに左右されない。正時までの残りを `hour_margin` と比べる判定は、`check_hour` が記録した `next_hour_unix` と `CLOCK.now(act.req)` の差を使う（仮想時刻で数える）。この判定で発行を見送るときは `next_talk_time` を変えないため、次の秒にまた判定する。発行するときは、シーンを探す前に `next_talk_time` を決め直す。
 - 次のトーク時刻は「現在時刻 + `math.random(最小間隔, 最大間隔)`」で決める。間隔は、決めるたびにローカル関数 `get_config` が `pasta.save` と `@pasta_config` を読み直して求める（キャッシュしない）。`hour_margin` は `[ghost].hour_margin` が有限の数ならその値を、切り捨てや下限の補正をせずにそのまま使う。無いとき、数でないとき、NaN・無限大のときは 30 を使う（ローカル関数 `finite_or`）。`[ghost]` の既定値の補完は [設定読込](loader.md#設定読込) で扱う。
 - ブロック中（2 で終わる間）は `check_hour`・`check_talk` を呼ばないため、`next_hour_unix`・`next_talk_time` は進まない。そのため、ブロックが解けた最初の判定で、過ぎていた正時やトーク時刻の条件が成り立つ。
diff --git a/book/src/lua/shiori-events.md b/book/src/lua/shiori-events.md
index 8e7f2c14..b738063b 100644
--- a/book/src/lua/shiori-events.md
+++ b/book/src/lua/shiori-events.md
@@ -1404,6 +1404,7 @@ local co = dispatcher.check_hour(act)
 - 前の呼び出しから[時刻の飛び](#時刻の飛び)があれば、次の正時を決め直す。決め直しは現在時刻の 70 秒前から数えるので、70 秒ほど前までに過ぎた正時は、1 回だけ時報になる。
 - 現在時刻が記録した正時に達していれば、次の正時を記録し直してから、4 つの候補でシーンを探す。
 - 一度に何時間分も進んでいても、時報は 1 回だけである。`{HH}` は進んだ先の時になる。
+- 時報のシーンが見つかったら、次のトーク時刻をその時点から決め直す（[check_talk](#check_talkact)）。時報の直後には、`talk_interval_min` 秒より前にランダムトークは出ない。候補が 1 つも無く時報を出さなかったときは、次のトーク時刻を変えない。
 
 ### OnTalk
 
@@ -1444,7 +1445,7 @@ pasta は、OnSecondChange が届くたびに、前の OnSecondChange からの
 
 - 飛びがあると、時報の次の正時を決め直す（[check_hour](#check_houract)）。仮想時刻を入れ直しても、余分な時報は出ない。
 - おしゃべりの間隔・撫での判定・コールバックの待ち時間（[get_property](script-api.md#get_propertyname_or_names-timeout-timeout_message)）は、飛びの分を差し引いた時計で数える。仮想時刻を 3 時間進めても、次のおしゃべりまでの残りは変わらない。
-- スリープから復帰した後の最初の差は、飛びとみなさない（`OnSysSuspend` か `OnSysResume` が届いた後の最初の飛び）。時報は 1 回出て、おしゃべりはすぐ出る。
+- スリープから復帰した後の最初の差は、飛びとみなさない（`OnSysSuspend` か `OnSysResume` が届いた後の最初の飛び）。時報は 1 回出て、おしゃべりは時報から間隔を数え直して待つ（時報のシーンが無ければ、おしゃべりはすぐ出る）。
 - `X-Pasta-Time` ヘッダのあるリクエストでは、飛びを見ない。その日時をそのまま使う。
 - 飛びの記録は保存しない。SHIORI の load から unload までの間だけ保たれる。
 
diff --git a/crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua b/crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua
index 7bc12a76..25694699 100644
--- a/crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua
+++ b/crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua
@@ -188,7 +188,11 @@ function M.check_hour(act)
     local candidates = { "時報" .. hh, "OnHour" .. hh, "時報その他", "OnHourOther" }
     for _, name in ipairs(candidates) do
         local t = create_scene_thread(name, act)
-        if t then return t end
+        if t then
+            -- 時報を出したら、次のランダムトークはここから数え直す（正時の前に期限が来た分を溜めない）
+            next_talk_time = calculate_next_talk_time(CLOCK.steady(act.req))
+            return t
+        end
     end
     return nil
 end
diff --git a/crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua b/crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua
index 7593758e..efac3bb2 100644
--- a/crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua
+++ b/crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua
@@ -613,6 +613,8 @@ describe("check_hour - all candidates miss", function()
         local result = dispatcher.check_hour(act)
 
         expect(result):toBe(nil)
+        -- 時報を出さなかったときは、次のトーク時刻を数え直さない
+        expect(dispatcher._get_internal_state().next_talk_time):toBe(0)
     end)
 end)
 
@@ -975,7 +977,7 @@ describe("check_hour / check_talk - 仮想の時刻", function()
         expect(found):toBe(nil)
     end)
 
-    vtest("実機の順番のスリープ明け（OnSysSuspend → 最初の秒 → OnSysResume）でも時報は 1 回出て、おしゃべりはすぐ出る", function()
+    vtest("実機の順番のスリープ明け（OnSysSuspend → 最初の秒 → OnSysResume）でも時報は 1 回出て、おしゃべりは時報から数え直す", function()
         dispatcher.check_hour(mk_act(R0)) -- 初回: 11:00 を記録
         dispatcher.check_talk(mk_act(R0)) -- 初回: 次のおしゃべりを記録
         local talk_at = dispatcher._get_internal_state().next_talk_time
@@ -984,7 +986,7 @@ describe("check_hour / check_talk - 仮想の時刻", function()
         expect(type(dispatcher.check_hour(mk_act(real)))):toBe("thread")
         expect(found):toBe("時報" .. hh_of(real))
         expect(talk_at < CLOCK.steady({ date = mk_date(real), dic = {} })):toBe(true)
-        expect(type(dispatcher.check_talk(mk_act(real + 1)))):toBe("thread")
+        expect(dispatcher.check_talk(mk_act(real + 1))):toBe(nil)
         CLOCK.observe({ id = "OnSysResume", date = mk_date(real + 4), dic = {} }) -- 4 秒遅れて届く
         found = nil
         expect(dispatcher.check_hour(mk_act(real + 5))):toBe(nil)
@@ -1003,6 +1005,20 @@ describe("check_hour / check_talk - 仮想の時刻", function()
         expect(dispatcher._get_internal_state().next_talk_time):toBe(talk_at)
     end)
 
+    vtest("正時の前に期限が来たおしゃべりは、時報の直後に出ず、時報から talk_interval_min 秒待つ", function()
+        dispatcher.check_hour(mk_act(R0))
+        dispatcher.check_talk(mk_act(R0))
+        local hour = BASE + H
+        pass(R0 + 1, hour - 21)
+        expect(dispatcher.check_talk(mk_act(hour - 20))):toBe(nil) -- 10:59:40: 期限は過ぎたが hour_margin で控える
+        expect(type(dispatcher.check_hour(mk_act(hour)))):toBe("thread")
+        local steady = CLOCK.steady({ date = mk_date(hour), dic = {} })
+        local wait = dispatcher._get_internal_state().next_talk_time - steady
+        expect(wait >= 180 and wait <= 300):toBe(true)
+        pass(hour + 1, hour + 178)
+        expect(dispatcher.check_talk(mk_act(hour + 179))):toBe(nil)
+    end)
+
     vtest("next_talk_time はなめらかな時計で数える", function()
         set_virtual(R0, R0 + 5 * H)
         local act = mk_act(R0)
```
