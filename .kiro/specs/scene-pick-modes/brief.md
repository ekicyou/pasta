# Brief: scene-pick-modes

> **ステータス**: 未着手（2026-10-21 の `/kiro-discovery`「PastaDSL機能ヒアリング」で起票。ghost_dev の「emo2 開発」「ぱすた開発」セッションからの聞き取り）。版は未定（開発者の判断）。`scene-attribute-store` の後に入る。要件定義は Opus。

## Problem

同名シーンの選び方が「シャッフル＆順次消費」の 1 つしか無い。作者は比率や順番を、シーンの複写と Lua で作っている。

- **重み**: emo2 の `dispatch.pasta` は「通常トーク：スキット：季節トーク」の比率を、同名の `＊OnTalk` の個数で表している（5:3:1）。コメントには 5:2:1 とあり、複写の数を数え間違えても気づけない。窓際のぱすたさんの `talk_gal.pasta` は `＊OnTalk_ギャル` を 15 個複写している。表情の候補でも、emo2 は同じ表情コードを重ねて書いて腕の向きを 2:1 にしている（`murasaki1000.lua`）。
- **定義順の巡回**: 窓際のぱすたさんの `ball.pasta` は、12 本のシーンを決まった順に 1 本ずつ出す。位置を `save["ボール巡回"]` に持つ Lua（`ball_next`）で書いている。同名シーンにするとシャッフルになるからである。

## Current State

2026-10-21（main `71e18f42`）に確かめた。

- 同名シーンの候補は、シャッフル＆順次消費で 1 つ選ぶ（`book/src/lua/modules/pasta-search.md` L40-42）。1 巡の間は重ならない。
- `SEARCH:set_scene_selector(0)` で定義の順にできるが、テストの道具で、全部の単語・Call・OnTalk に効く（同 L159-195）。シーンごとには選べない。
- 属性（`＆重み：5` など）は解析して記録するが、実行時に何の効果も無い（`book/src/grammar/block-structure.md` L271。`scene.lua` L148・L206 と `act.lua` L678 の `attrs` は使われない）。
- 重みを付ける手段は、シーンや単語の値（`＠w：A、A、B`）の複写だけである。

## Desired Outcome

- 同名シーン（Call の前方一致の候補を含む）に、属性で重みを付けられる。`＊OnTalk　＆重み：5` と書けば、複写しなくても比率が付く。
- 候補を定義の順に 1 本ずつ巡回させる書き方がある。位置はゴーストを閉じても続く（保存する）か、セッションの間だけかを要件で決める。
- マニュアルとスキルの `references/` が書き方を載せる。

## Approach

要件フェーズで次を決める。

- **書く場所**: 重みは候補の側（`＊OnTalk　＆重み：5`）。巡回は候補の側か、呼ぶ側（`＞ボール巡回`）か。属性の名前（日本語・英語の別名）。
- **重みとシャッフル消費の関係**: 重みを付けた候補を 1 巡の中で重みの数だけ出すのか、毎回の抽選の確率にするのか。候補の控え（キャッシュ）のキーとの関係。
- **巡回の位置**: 保存の範囲（`＄＊` と同じ保存ファイルか、`novel-variable-scope` が決める範囲か）。候補が増減したときの続き方。
- **単語の重み**: 単語の値の重み（`＠w：A×2、B`）を範囲に入れるか。表情の候補（アクター辞書の単語）にも効くので、作者の要望はある。入れないならバックログに置く。

## Scope

- **In**: 同名シーンの選び方の重みと巡回、その属性の読み方、テスト、マニュアル、スキルの `references/` の再生成
- **Out**: 属性の保持・継承（`scene-attribute-store`）、条件による絞り込み（`call-attribute-filter`）、「一度だけ」のシーン（フラグと絞り込みで書く。`call-attribute-filter` へ申し送った）

## Boundary Candidates

- 候補の選び方（`pasta_core` の `RandomSelector`・シーン表、`pasta_lua` の検索の文脈）
- 巡回の位置の保存
- マニュアル・生成スキル

## Out of Boundary

- 属性の構文と保持（`scene-attribute-store`）
- イベントの受け口のシーン探しの規則（`scene-exact-match`）

## Upstream / Downstream

- **Upstream**: `scene-attribute-store`（属性を実行時に読めること）
- **Downstream**: `call-attribute-filter`（絞り込んだ後に残った候補から、この選び方で選ぶ）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**: `scene-attribute-store`・`call-attribute-filter`（候補の控えのキーと選び方を共有する。同じウェーブに置かない）、`ponytail-core-registry`（`pasta_core` のシーン表を `BTreeMap` に替える。先に入る）

## Constraints

- 重みも巡回も書かないシーンの選び方は、今のシャッフル＆順次消費のまま変えない。
- 規模は 6〜8 タスクの見込み。
