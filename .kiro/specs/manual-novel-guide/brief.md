# Brief: manual-novel-guide

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 9 番目（最後）。

## Problem

ノベルゲームを作れる機能が揃っても、作り方がマニュアルに無ければ、作者は作れない。機能ごとの説明は、先行する各 spec が文法とリファレンスの章に書く。しかし、それをつなげて「1 本のノベルをどう書き始め、どう仕上げるか」を導く文章は、どの spec の持ち場でもない。

## Current State

- マニュアル（`book/`）は、利用者向けの情報の唯一の権威である。入門ガイド（`book/src/getting-started/`）は、デスクトップマスコットの hello-pasta を題材に、段階ごとに進む 16 章である。
- ノベルゲームについての章は無い。
- マニュアルに章を足すと、目次（`SUMMARY.md`）と、章の数を決め打ちする検査（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）が動く。

## Desired Outcome

- マニュアルに、ノベルゲームの作り方の部がある。見本のノベル（`hello-novel-sample`）を題材に、脚本の 3 層に沿って進む。
  - 柱: 場面と、その性質（背景・曲・誰がいるか）
  - ト書き: その瞬間に起きること（効果音・揺れ・登場と退場）
  - 台詞: 話者と表情
  - 分岐: 選択肢と、フラグによる場面の選び分け
  - システム画面: 見本を写して、絵と文を差し替える
  - 配布
- 画面を組む人（シェルを作る人）向けに、pasta の場面と噛み合う所が書いてある。
- areka との取り決め（語彙の一覧）へ、リンクで送り出す。
- 生成スキル（`pasta-ghost-authoring` ほか）が、ノベルの書き方を知っている。

## Approach

要件定義で決めること。

- **置き場所と形**: 入門ガイドの続きにするか、別の部にするか。Claudia が語る形（入門ガイドの例外の決まり）を、こちらにも当てるか。
- **章の割り方**: 見本のノベルを、段階ごとに起動できる形にするか（hello-pasta の段階表と同じ流儀）。
- **スクリーンショット**: areka の実機の絵を載せるか。載せるなら、撮り方。
- **シェルの説明との分担**: `manual-shell-guide`（デスクトップマスコットのシェルの説明）と、ノベルの画面の組み方を、どう分けるか。
- **回避の書き方を載せない**: 書けない形の「代わりの書き方」は、個別に載せない。部品の能力を示す。

## Scope

- **In**: ノベルゲームの作り方の章。目次と、章の数を決め打ちする検査の更新。本文と見本の辞書の照合。スキル references の再生成。
- **Out**: 機能ごとの文法とリファレンスの章（先行する各 spec が、挙動と同じ変更で書く）。見本のノベルそのもの（`hello-novel-sample`）。areka の使い方の説明（areka の文書）。

## Boundary Candidates

- 章の本文。
- 本文と見本の辞書の照合（`book/tools/tutorial-check.mjs` の流儀）。
- 生成スキルへの反映。

## Out of Boundary

- マニュアルの見た目と台詞の部品（`manual-claudia-theme` で完了）。
- 入門ガイドの既存の章を変えること。

## Upstream / Downstream

- **Upstream**: `hello-novel-sample`（題材）。先行する各 spec が書いた文法とリファレンスの章。
- **Downstream**: 0.5.0 のリリース。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `manual-shell-guide`（シェルの説明）、`getting-started-screenshots`（絵の載せ方と、本文の検査）、`manual-link-anchor-check`（見出しへのリンクの検査）。

## areka への依頼

無い見込みである。areka の実機の絵を載せるなら、撮るときに areka のセッションと一緒に進める。

## Constraints

- マニュアルが権威である。挙動の説明は、機能を入れた spec の章と食い違わないようにする。
- 章を足すほかの spec と、目次と章の数の検査が重なる。後から入る側が、数を合わせる。
- 辞書を変える spec は、同じ変更で、この章の作例を直す。
