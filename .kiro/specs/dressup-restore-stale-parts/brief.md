# Brief: dressup-restore-stale-parts

> **ステータス**: 未着手（2026-10-18 の `/kiro-discovery` で起票）。版は開発者が決める（推奨は 0.4.2。致命的でない不具合で、ゴースト側に回避がある）。着手するときは `/kiro-start dressup-restore-stale-parts` で開始する。**要件定義の最初に、実例で原因を確かめる**（下の「まだ分かっていないこと」）。

## Problem

1 つのスポットを複数のアクターで共有するゴーストで、話者が戻ったときの外見の復旧が、前に着けた別のパーツ（別の腕・目）を出してしまう。2026-10-18 に、ghost_dev の「ぱすた開発」セッション（窓際のぱすたさん）が実機で報告した。

- 報告の見立て: pasta は、着せ替えの軸ごとの排他（`mustselect`）を知らずに、パーツ単位で記録している。そのため ON だけを出すと、過去に ON にしたパーツが全部記録に残り、復旧のときに別の腕や目が出た。
- ゴースト側の回避: 選ばなかったパーツを `,0` で明示的に OFF にしている。`act:raw_script` で出した bind は記録されないことも使っている。
- ネタ帳のとおりに `\![bind,うで,ボール,1]` と書くだけで済まず、作者がすべての OFF を書き並べることになる。

## Current State

外見の復旧は `crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua` が持つ（spec `actor-surface-restore`。完了）。状態は `STORE.appearance` で、保存はしない。

- 記録の形: `binds[カテゴリ][パーツ] = 0|1`。アクターの側には、カテゴリごとの記録の順 `order[カテゴリ]` もある（記録し直すと末尾へ動く）。
- 着けたとき（`1`）:
  - スポットの側では、同じカテゴリのほかのパーツを「不明」にする（SSP が複数選択できないカテゴリで自動で外すため。要件 R3.13）。
  - アクターの側では、ほかのパーツの `1` を残す。
- 復旧の出力（`restore_dressup`）:
  - カテゴリごとに出す。中身は、`dressup` の既定のうちアクターが記録していないものを名前の順に並べ、続けてアクターの記録を記録の順に並べる。
  - そのリストを、OFF も含めて全部 `\![bind-noevent,…]` で出し直す。
  - ねらいは要件 R3.2 のとおり。複数選択できないカテゴリでは、最後に着けたパーツが残る。複数選択できるカテゴリでは、すべてが着く。このため descript.txt を読まずに、どちらでも再現できる。
- pasta は、シェルの `descript.txt`（`bindgroup`・`bindoption`）を実行時に読まない（設計の非ゴール。`actor-surface-restore` の design.md:23）。
  - マニュアルは、`mustselect` の最後の 1 パーツを外す指定を SSP が拒んだときのずれを、非対応と書いている（`book/src/grammar/actor-dictionary.md` の「非対応事項」）。
- 窓際のぱすたさんのシェル（`ghost_dev/project/pasta-in-windows/ghost/pasta-in-windows/shell/master/descript.txt`）:
  - 口・目・眉・うでが `mustselect` で、`multiple` のカテゴリは無い（2026-10-18 に読んだ）。
  - ボールのカテゴリ（おじさん・男の子・ギャル・妹）には、`bindoption` が無い。
- テスト: `crates/pasta_lua/tests/lua_specs/appearance_test.lua`（observe と restore の着せ替えの節）、`sakura_builder_test.lua` の統合（麦わら帽子とリボン）。Rust と SHIORI の結合テストに、`dressup`・`bind-noevent` を確かめるものは無い。

## まだ分かっていないこと

今の設計では、複数選択できないカテゴリでも、記録の順に出し直せば最後の 1 つが残るはずである。報告の症状（別の腕や目が出る）が、どの経路で起きたかは確かめていない。候補は次のとおり。

1. SSP 2.9.18 が、1 回のスクリプトの中で同じカテゴリの `bind-noevent` が続いたとき、「最後に着けたものが残る」動きをしない（`mustselect` との組み合わせを含む）。
2. 記録の順が、作者の意図した最後の状態と違う。例: `dressup` の既定が後から勝つ、記録し直しで末尾へ動く規則の取りこぼし、トグルやカテゴリ指定で「不明」になった後の扱い。
3. `\![bind,…]` と `bind-noevent` の違い、または `mustselect` で OFF を拒まれたことによる、スポットの側の記録と実際の表示のずれ。
4. 報告の見立てのとおり、パーツ単位の記録そのものが原因の場合。そのときは、記録の形をカテゴリごとの「今着けているもの」に寄せる。

2026-10-18 に、「ぱすた開発」セッションへ再現の材料（回避を入れる前の台詞・pasta.dll が返したスクリプトの全文・そのアクターの `dressup` の設定）を頼んだ。要件定義は、その材料と、SSP の実機（読み取り専用の別 SSP。手順は `ssp-ghost-test-launch` のメモ）で、どの候補かを確かめてから始める。

## Desired Outcome

- 作者が ON だけを書く（`\![bind,うで,ボール,1]`）ゴーストで、話者が戻ったときに、そのアクターが最後に着けていた見た目が戻る。過去に着けた別のパーツは出ない。
- `,0` の明示の OFF を書き並べる回避が要らなくなる。
- 複数選択できるカテゴリ（`multiple`）の再現は、今の約束（すべてが着く）を保つ。保てないなら、要件定義でその扱いを決める。
- マニュアルの「同一スポット共有時の外見の復旧」が、新しい動きと、残る制限を正しく書いている。

## Approach

原因を確かめてから決める。今の時点での候補は次のとおり。

- **A. 出し直しをカテゴリごとに絞る**: 複数選択できないカテゴリでは、最後に着けた 1 つ（と明示の OFF）だけを出す。どのカテゴリが複数選択できるかを知る手段が要る。
  - A1: `descript.txt` を読む。
  - A2: pasta.toml でカテゴリの性質を宣言する。
  - A3: 「1 を出したら同じカテゴリのほかのパーツの記録を消す」を既定にし、`multiple` のカテゴリだけ宣言で外す。
- **B. 記録の形を「今着けているもの」に寄せる**: 着けたときに、アクターの側でも同じカテゴリのほかのパーツの記録を消す。複数選択できるカテゴリでは、着けたパーツが減って見える。
- **C. 原因が SSP の動き（候補 1）なら**、出し直しの順と形（先に全部 OFF にしてから最後のものを ON にする、など）を変える。

推奨は、実例で原因を確かめた後に決める。今の情報で推すなら A3 である。`multiple` のカテゴリは少数で、窓際のぱすたさんのシェルには無い。宣言が要るのは珍しい側だけで済む。

## Scope

- **In**:
  - 原因の確認（実例と実機）。
  - `appearance.lua` の記録と復旧の直し。
  - 必要なら pasta.toml のキー（読み込みの検査と `pasta_check validate` の報告を含む）。
  - `appearance_test.lua`・`sakura_builder_test.lua` の退行テスト。
  - マニュアル（`grammar/actor-dictionary.md`・`reference/pasta-toml.md`・`internals/talk-output.md`）と、スキル references の再生成。
- **Out**:
  - `act:raw_script` の bind を記録すること（今の約束どおり、アクターの記録に入れない）。
  - サーフェス（`\s`）の復旧の規則。
  - シェルの `surfaces.txt` を読むこと。

## Boundary Candidates

- 記録（observe）: 着けたときに、アクターの側の同じカテゴリの記録をどうするか。
- 復旧（restore）: カテゴリごとに何を・どの順で出すか。
- カテゴリの性質を知る手段（`descript.txt`・pasta.toml・既定）。A を採るときだけ要る。

## Out of Boundary

- ゴースト側の辞書の直し（回避を外すのは ghost_dev の側の仕事。ghost_dev は共有のチェックアウトなので、pasta のセッションから書き換えない）。
- 着せ替えのメニューから利用者が変えた着せ替え（`OnDressupChanged`）を記録すること。今の約束どおり、発火させず、追わない。

## Upstream / Downstream

- **Upstream**: `actor-surface-restore`（完了。`.kiro/specs/completed/actor-surface-restore/`）の要件 R3.1〜R3.13・設計 OQ-8。SSP 2.8.23 以上の `bind-noevent`。
- **Downstream**: 1 つのスポットを複数のアクターで共有するゴースト（窓際のぱすたさん。`actor-display-style` で正式に支えた形）。0.5.0 のノベル（登場人物がシェル）。

## Existing Spec Touchpoints

- **Extends**: `actor-surface-restore`（完了）。要件 R3.2・R3.5・R3.13 と、非対応事項の `mustselect` を見直す。
- **Adjacent**:
  - `actor-display-style`（完了）。装いの出し入れは `pasta.shiori.actor_style` が持ち、復旧の後に出す。
  - `sakura_builder.lua` は、`appearance.lua` を呼ぶ所だけを触る見込みである。触るなら、`novel-talk-flow`・`cue-command-runtime`・`novel-checkpoint-rollback`・`scene-anchor-link` と同じウェーブに置かない。

## Constraints

- 着せ替えを書いていないゴーストと、`,0` で明示の OFF を書いているゴースト（窓際のぱすたさんの今の辞書）の出力を壊さないこと。
- ゴーストの作り込みの知見は、推測で書かずに emo2 開発・ぱすた開発のセッションに確かめる。
- 要件定義は Opus でよい。記録の形は保存しないので、決め直せる。ただし、pasta.toml のキーを足すなら、その名前と既定は作者が上に乗る約束になるので、慎重に決める。
