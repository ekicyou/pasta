# Brief: actor-line-newlines

## Problem

縦書きバルーンのゴースト（ghost_dev「窓際のぱすたさん」）は、1 行を 1 列として台詞を書く。ネタ帳（台詞の正本）も 1 行 1 列で書いている。ところが pasta は、同じ話し手が続く行の間に何も挟まない。そのため辞書では、列の切れ目ごとに行の頭へ `＠改行`（`\n[180]`）を手で書き足すことになり、辞書の形がネタ帳と違ってしまう。2026-11-05 時点で、ぱすたの独り言（ポエム・起動の挨拶・時報など）の行の頭に `＠改行` が並んでいる（ぱすた側は同じ日に 187 行を「行の頭に ＠改行」の形へそろえた）。

## Current State

- トランスパイラ（`pasta_lua` の `code_gen/element_gen.rs`）は、行の境目を捨てる。継続行 `　：…`（`ContinueAction`）も、同じアクターを書き直した行（`ぱすた：A` → `ぱすた：B`）も、前の行と同じ `talk` の呼び出しが並ぶだけになる。出力側には「ここで行が替わった」が届かない。
- 出力側（`pasta_scripts/pasta/shiori/sakura_builder.lua` の `BUILDER.build`）が改行を入れるのは、話し手（スポット）が替わり、その吹き出しに字が既にあるときだけである（`[ghost] spot_newlines`。保留して、次の字の直前に出す遅延方式）。ほかに入れるのは選択肢の前後（`choice_newlines`・`after_choice_newlines`）と、budoux の自動改行（`[actor."名前"] budoux_newlines`）だけ。
- マニュアル「アクション行」の「行継続」と「改行」は、継続行が前の発言に続けて出力されること（改行にならないこと）を書いている。
- アクターごとの改行の幅には先例がある（`[actor."名前"] budoux_newlines`。`\n[値×100]`、`1.0` なら `\n`）。

## Desired Outcome

`pasta.toml` に

```toml
[actor."ぱすた"]
line_newlines = 1.8
```

と書くと、

```pasta
　ぱすた：＠笑顔　今日が最後の日なら。
　　　　：＠注目　みんなに会ってみたいかも。
```

の 2 行目の字の前に `\n[180]` が入る。書かなければ今と同じ出力（何も入れない）。ぱすた側は、行の頭の `＠改行` を消して、辞書をネタ帳と同じ形に戻せる。

## Approach

**単純なモデル**: トランスパイラが「行の頭」の印を出す。出力側は、その印を、spot_newlines と同じ遅延方式の保留として持ち、同じ話し手が字を出す直前に `\n[値×100]` を 1 つ入れる。値は話し手の `[actor."名前"] line_newlines`。

ghost_dev「ぱすた開発」セッションとのやりとり（2026-11-05）で、次を決めた。

1. **対象の行**: 継続行と、同じアクターを書き直した行の両方。ポエムの「（間）」の後で `ぱすた：` を書き直す形にも入る。
2. **設定の場所**: `[actor."名前"] line_newlines` だけ。`[ghost]` には置かない。相手役 4 人もぱすたと同じスポット 0 の吹き出しに出るので、ゴースト全体の値は相手役にも効いて困る。未設定のアクターは何も入れない（今と同じ）。値の換算と型の検査は `budoux_newlines` と同じ（`N` が 100 なら `\n`、それ以外は `\n[N]`）。
3. **話し手が替わった行**: `spot_newlines` だけを出す。`line_newlines` は重ねない。
4. **Call をまたぐ**: 行の境目は出力の並びで見る。Call 行の前の行・呼ばれた先の最初の行・戻った後の行も、同じ話し手が同じ吹き出しで続いていれば入る（例: ＊雑談_ポエム_暑い寒い の `ぱすた：窓の下の公園、` → `＞＄季節` → 戻って `ぱすた：＠左　…`。＊OnBoot → ＞起動挨拶 → ＞起動_朝 → ＞レベルアップ）。
5. **選択肢の前後**: 今の `choice_newlines`・`after_choice_newlines` を優先する。`line_newlines` は足さない。
6. **行の頭に位置を決めるタグがある行**: その行の最初の見える字より前に、改行（`\n`・`\n[N]`）・`\_l[…]`・`\c` のどれかがあれば入れない。表情の単語（`\![bind,…]`・`\s[…]`）を先に置いてから `＠改行` を書く行（`：＠笑顔　＠改行　…`）も、この規則で二重にならない。`＠改ページ`（`\_w[2000]\c`）や `＠左`（`\_l[-283,0]`）で始まる行の前にも入れない。
7. **前の行の末尾が改行で終わる**とき（`…\n` で終わる行の次の行）も入れない。
8. **budoux**: 入れた `\n[N]` で列を 1 つ進める（`spot_newlines` と同じく `put_break` を通す）。budoux が行の途中で折る改行とは独立。
9. **行に数えないもの**: 空行・コメント行・変数代入行・Call 行そのもの。字を出す行の間だけが対象。
10. **字の無い行**（表情の単語だけの `ぱすた：＠笑顔` など）: 保留は溜めない。次の字の行の前に 1 つだけ入る。
11. **吹き出しが空のとき**: `\c` の後、出力の最初の行、`clear_spot` の後は入れない（区切る相手の字が無い）。

## Scope

- **In**:
  - トランスパイラが行の頭の印を出す（`ActionLine`・`ContinueAction`）。生成する Lua の形と、`sample.generated.lua` などの生成物・コード生成のテストの更新。
  - 出力側（`sakura_builder.lua`）の保留と、上の 1〜11 の決まり。`act.lua` から builder への設定の受け渡し。
  - `[actor."名前"] line_newlines` の読み込みと型の検査（`budoux_newlines` の作りに合わせる）。
  - テスト（Rust の読み込み・Lua の builder・行の頭の規則・Call をまたぐ例・選択肢の前後・budoux の列）。
  - マニュアル（`book/src/reference/pasta-toml.md` のアクター設定の表と節、`book/src/grammar/action-line.md` の「行継続」「改行」、`book/src/internals/talk-output.md`）と、スキルの参照（`pasta-ghost-authoring/references/pasta-toml.md` ほか、マニュアルの写し）。
- **Out**:
  - `[ghost]` 全体の既定値（相手役にも効いて困るため。必要になったら別に起票する）。
  - 改行の後に付ける文字列（相手役の字下げ `\_u[0x3000]` など）。相手役は `＠改行` の手書きのまま使う。将来の拡張候補としてバックログに残す。
  - ぱすた側の辞書の書き換え（ghost_dev の作業。行の頭の `＠改行` を機械的に消す台本がある）。

## Boundary Candidates

- トランスパイラ: 行の頭の印を生成する所（`element_gen.rs`・`scope_gen.rs`）。印の名前と、生成する Lua の 1 行の形。
- 出力側: `BUILDER.build` の状態機械に「行の保留」を足す所。既存の `pending_break`（spot_newlines）・`spot_tail`（選択肢）・`columns`（budoux）との優先順位。
- 設定: アクター設定の読み込みと型の検査。

## Out of Boundary

- 話し手が替わったときの改行（`spot_newlines`）の挙動は変えない。
- 選択肢の前後の区切り（`choice_newlines`・`after_choice_newlines`）の挙動は変えない。
- budoux の自動改行の折り方は変えない。
- 行の途中の `\n` の扱い（作者が書いたものはそのまま出す）。

## Upstream / Downstream

- **Upstream**: 完了済みの `sakura-script-newline`（遅延方式の改行）・`vertical-balloon-lines`（budoux の列）・`choice-line-layout`（選択肢の前後）・`actor-talk-grouping`（同じアクターのトークンのまとめ）。
- **Downstream**: ghost_dev「窓際のぱすたさん」の辞書の移行。将来の「改行の後に付ける文字列」の拡張。

## Existing Spec Touchpoints

- **Extends**: なし（新しい spec）。
- **Adjacent**: `cue-command-runtime`（柱の属性・アクター指定行を、場面に入ったときと Call から戻ったときに台本へ流す。行の頭の印と同じ生成の場所を触る。同じウェーブに置かない）。`scene-anchor-link`（`act.lua` を触る）。`novel-talk-flow`（台詞ごとのクリック待ち。行の単位を使う可能性がある）。

## Constraints

- 未設定のときの出力は、1 バイトも変わらないこと（今のゴーストを壊さない）。トランスパイラが印を出しても、出力側が値の無いアクターでは何も出さない。
- `sakura_builder.lua` の列の動かし方は Rust の `ColumnMove` の写しである。片方だけ直さない。
- 生成する Lua を変えるので、`pasta_lua` のテストが書き換える `sample.generated.lua` の改行コードだけの差分をコミットに混ぜない。
- 判断に迷う所（特に 6・7）は、ghost_dev「窓際のぱすたさん」セッションへ返して確かめる（ぱすた開発セッションの依頼）。
- 版は未定（開発者が決める）。要件定義のモデルは Opus（土台の決定ではなく、論点は上で閉じている）。
