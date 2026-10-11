# Brief: choice-line-layout

`getting-started-story-guide` の完了時の棚卸で起票（2026-10-10・main `f193c9cb`）。

## Problem

入門ガイドの 9 段目（選ばせたい）を辞書のとおりに書いて動かすと、1 つ目の選択肢が問いかけの台詞と同じ行に続いて描かれ、吹き出しの右端で文字が切れて見える。読者は書いてあるとおりに写しただけなので、自分の間違いなのか区別できない。配布している見本の hello-pasta でも同じに見える。

## Current State

- 辞書（`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/09-choice.pasta`）:

  ```pasta
  ＊OnMouseDoubleClick
  　女の子：＠笑顔　ねえ、ユーザーさん。なんの話をしよっか？
  　＠？おやつの話「おやつの話をする」
  　＠？おでかけの話
  　!select(10)
  ```

- 実機の確認（2026-10-10・SSP・v0.3.8 の `pasta.dll` と今のシェル）: 「ねえ、ユーザーさん。なんの話をしよっか？」のすぐ後ろに「おやつの話をする」が続き、「をする」が吹き出しの右端で切れる。2 つ目の「おでかけの話」は次の行に出る。選択肢そのものは働く（クリックで正しいシーンへ進む。10 秒で時間切れになる）。
- 選択肢の出力は `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua` が `\![*]\q[表示,飛び先]` に変える。1 つ目の選択肢の前に改行を入れているかは調べていない。
- 原因の候補は 3 つ: 選択肢の前に改行が出ていない／出ているが吹き出しの折り返しと重なっている／バルーンの幅が足りない。
- 入門ガイドの `book/src/getting-started/09-choice.md` は、この辞書を逐語で写している（`tutorial-check.mjs` が照合する）。

## Desired Outcome

- 9 段目を書いたとおりに動かすと、選択肢が 1 つずつ別の行に、切れずに見える。
- 直した場所（辞書・出力・バルーン）と理由が記録される。

## Approach

決めていない。最初に、実際に出ているさくらスクリプトを見て原因を 1 つに絞る。出力の側（選択肢の前の改行）が原因なら、選択肢を書いた人が改行を足さなくても済むように出力で直す。辞書の書き方で避ける手順は、ガイドに載せない。

## Scope

- **In**: 原因の特定。選択肢の出力、または見本の辞書・バルーンの修正。直した後の実機での見え方の確認。辞書を変えた場合は、入門ガイド 9 段目の作例と段階表の同時の更新。
- **Out**: 選択肢の文法の変更。バルーンの絵の描き直し。文節での折り返し（budoux）の方針。

## Boundary Candidates

- 選択肢の出力（`sakura_builder.lua`）。
- 見本の辞書と、それを写す入門ガイドの章。

## Out of Boundary

- 台詞の中のアンカー（`scene-anchor-link`。選択肢の振り分けを共有するので、順番に注意する）。
- シェルの絵（`hello-pasta-shell-art`）。

## Upstream / Downstream

- **Upstream**: なし。
- **Downstream**: `getting-started-screenshots`（9 段目の絵に、切れた選択肢がそのまま写る。先に直すと撮り直しが要らない）。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `scene-anchor-link`（`sakura_builder.lua` と選択肢の振り分け）、`getting-started-screenshots`、`hello-pasta-shell-art`（辞書と `book/` を触らない約束）。

## Constraints

- 出力を変えるなら、既にあるゴーストの選択肢の見え方が変わる（改行が 1 つ増える）。影響を要件で確かめる。
- 辞書を変えるなら、`tutorial-check.mjs` の逐語の照合と `crates/pasta_sample_ghost/tests/tutorial_stages_test.rs` が同時に通ること。

## 2026-10-10 起票時の測定（main f193c9cb）

- **規模**: 2〜4 タスク。
- **種別**: バグ（見え方）。入門の読者が最初に踏む。
- **要件定義のモデル**: Opus（原因が決まれば直し方は絞れる）。
- **要件定義の議題**:
  1. 原因はどれか（実際のさくらスクリプトで確かめる）。
  2. 出力で直す場合、台詞の直後の選択肢の前に必ず改行を入れるか。選択肢だけのシーン・台詞の無い行の後ではどうするか。
  3. emo2 など既にあるゴーストへの影響。
