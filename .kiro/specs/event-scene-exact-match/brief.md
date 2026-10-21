# Brief: event-scene-exact-match

> **ステータス**: 未着手（2026-10-21 の `/kiro-discovery`「PastaDSL機能ヒアリング」で起票。ghost_dev の「emo2 開発」「ぱすた開発」セッションからの聞き取り）。版は未定（開発者の判断）。要件定義は Opus。

## Problem

SHIORI イベントを受けるシーンが、前方一致で**別のイベント**のシーンを拾う。作者は気づきにくい。

- emo2（ghost_dev）で、`＊OnInstallComplete` と `＊OnInstallCompleteAll` を両方書いたところ、`OnInstallComplete` の候補に `＊OnInstallCompleteAll` が混ざった。`OnInstallCompleteAll` のシーンは Reference を見て 204 を返すので、約半数のインストール完了で何も言わなくなった。
  - 作者は `OnInstallCompleteAll` を `REG`（完全一致）へ逃がした。ユーザーの `REG` を置く場所として `pasta.shiori.event.boot` を上書きしたので、起動と関係の無いハンドラが `boot.lua` に同居している。
  - 両ゴーストの作者は、シーン名を足すたびに全部の名前の前方一致の衝突を総当たりで調べている。
- 窓際のぱすたさん（ghost_dev）でも、`OnMouseEnterAll`／`OnMouseLeaveAll` が前方一致で `＊OnMouseEnter`／`＊OnMouseLeave` に入り、シーンが見かけの約 2 倍動いていた（`chain-talk-interrupt` の brief の Problem）。
- 同じ形の組は SHIORI イベントに多い: `OnMouseClick`／`OnMouseClickEx`、`OnMouseDoubleClick`／`OnMouseDoubleClickEx`、`OnChoiceSelect`／`OnChoiceSelectEx`、`OnInstallComplete`／`OnInstallCompleteAll`、`OnMouseEnter`／`OnMouseEnterAll` など。

## Current State

2026-10-21（main `71e18f42`）に確かめた。

- `REG` にハンドラが無いイベントは、`SCENE.co_exec(act, act.req.id, nil, nil)`（`pasta_scripts/pasta/shiori/event/init.lua` L179）でシーンを探す。グローバルシーンは前方一致で探す。
- マニュアル `book/src/lua/shiori-events.md` L635 は「`＊OnBoot朝` のように、イベント名で始まる名前のシーンも候補になる」と書く。L808 は `OnTalk` で始まる名前も候補になると書く。作者はこれを使って `＊OnTalk_ギャル` のような変化を書いている。**前方一致そのものは作者の道具**であり、全部を完全一致にはできない。
- 完全一致の検索は Lua の API にだけある（`SEARCH:search_scene_exact`、`SCENE.co_exec_exact`。`scene.lua` L227-238、`crates/pasta_lua/src/search/context.rs` L105-111）。DSL から使うのは撫で・つつきのショートカットの 1〜8 段目だけである。
- `[scene.alias]` の既定 `会話 → OnTalk` は完全一致・1 段だけ（`crates/pasta_core/src/registry/scene_alias.rs`）。

## Desired Outcome

- イベント `X` を受けるとき、名前が **別の SHIORI イベント名**になっているシーン（`X` の後ろに英大文字で始まる語が続くもの、など）は候補に入らない。`＊OnInstallComplete` を書いた作者は、`＊OnInstallCompleteAll` を足しても挙動が変わらない。
- `＊OnBoot朝`・`＊OnTalk_ギャル`・`＊OnHour12` のような、作者が変化として書く名前は、今までどおり候補に入る。
- 規則がマニュアル（`lua/shiori-events.md` の「シーン関数フォールバック」）に 1 か所で書かれ、`pasta_check validate` が衝突しうる組を知らせる（入れるかは要件で決める）。

## Approach

要件フェーズで次を決める（推奨つき）。

- **区切りの規則**（推奨: 英大文字の続き）: イベント ID の直後が `[A-Z]` で始まる候補を外す。`All`・`Ex` のような SHIORI の命名の続きは外れ、`朝`・`_`・数字の続きは残る。既に `＊OnTalkMorning` のように英大文字で続けて書いた作者の辞書は挙動が変わる（破壊的変更の扱いを要件で決める）。
- **代わりの案**: (a) 既知の SHIORI イベント名の表と突き合わせて外す（表の保守が要る）、(b) 完全一致を指定する印をシーンに付ける（作者が知らないと防げない）、(c) 検査で知らせるだけ（実行時は変えない）。
- 実行時の外し方は `search/context.rs` の検索に足すか、`event/init.lua` の受け口で `co_exec` の候補を絞るか、設計で決める。

## Scope

- **In**: イベントの受け口のシーン探し（シーン関数フォールバック）の候補の規則、テスト、マニュアル、スキルの `references/` の再生成、`pasta_check validate` の衝突の知らせ（要件で決める）
- **Out**: Call（`＞X`）の前方一致の規則（作者が書いた Call は今のまま）、単語の前方一致、`[scene.alias]` の形（`recommend-menu-scenes` が改める）、撫で・つつきのショートカットの段

## Boundary Candidates

- イベントの受け口（`shiori/event/init.lua`）と検索（`search/context.rs`）
- 辞書の検査（`pasta_check` の `validate`）
- マニュアル・生成スキル

## Out of Boundary

- 条件で候補を絞る書き方（`call-attribute-filter`）
- 同名シーンの選び方（重み・巡回。`scene-pick-modes`）

## Upstream / Downstream

- **Upstream**: なし
- **Downstream**: ゴースト作者が `REG` に逃がしていたイベント（emo2 の `OnInstallCompleteAll`）を DSL に戻せる

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `chain-talk-interrupt`（`event/init.lua` の `EVENT.drive` まわりを触る。同じウェーブに置くなら順番を決める）、`scene-play-event`・`external-control-events`（イベントの既定のハンドラを足す。`OnPastaScene` と `OnPastaSceneAt` も英大文字の続きの組なので、規則の例として確かめる）、`recommend-menu-scenes`（`[scene.alias]` を改める）

## Constraints

- `＊OnBoot朝` のような、マニュアルが書いている使い方は壊さない。
- 規模は 5〜7 タスクの見込み（規則と検索 1〜2、検査 1、テスト 1〜2、マニュアルとスキル 1）。
