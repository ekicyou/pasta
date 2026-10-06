# Brief: getting-started-story-guide

## Problem
pasta のマニュアルは初心者向けになっていない。入門ガイドは 2 章（`prerequisites.md`・`first-ghost.md`）しかない。`first-ghost.md` は完成形の辞書 5 ファイルを順に貼り付ける手順書で、途中の段階では起動しないこともある。その先は文法リファレンスへ送られ、DSL の文法を体系的に読むことになる。初心者には「何ができるのか」「どう表現したいときに何を書くのか」が頭に入らない。案内役の Claudia も、各章の導入と締めに一言ずつ顔を出すだけで、物語がない。

## Current State
- `book/AUTHORING.md` の執筆規約は、全章に「軽い導入（キャラ口調）→ 淡々とした本体（普通文体）→ ひとことの締め（キャラ口調）」を課す。説明本体での「〜ですわ」口調は禁止されている。
- `book/tools/verify-content.mjs` の D 系検査が、`getting-started` を含む全章について、散文部にキャラ口調があることと、コードフェンス内に口調がないことを見る。
- `getting-started` は `gen-skill-refs.mjs` の `GENERATION_MAP`（スキル `references/` の生成元）に含まれない。本文の口調を変えても、スキルの生成物には影響しない。
- `book/tools/tutorial-check.mjs` は、`first-ghost.md` の ```pasta ブロックが hello-pasta の `dic/*.pasta` と逐語一致することを検査する。
- 文法リファレンス（`grammar/`）と Lua 章（`lua/`）は、リファレンスとして体系的に書かれている。

## Desired Outcome
- 入門ガイドは「こんな表現をしたい」を 1 つずつ叶えていく物語になっている。章を進めるごとに、読者のゴーストにできることが増える。各章の終わりでゴーストは起動し、その章の表現を試せる。
- ガイドの間は、Claudia が全編を語る。説明本体も Claudia の語りで書き、Claudia が前面に出て読者を引っ張る。
- 文法リファレンスと Lua 章はリファレンスのまま残る。ガイドの各章は、学んだ表現の詳しい文法へリンクで送り出す。
- ガイドの作例は段階辞書（`hello-pasta-tutorial-stages`）と逐語で一致し、CI がそれを検査する。
- 新しいシェル（`hello-pasta-shell-art`）で、ゴーストが実際にしゃべる様子をスクリーンショットで示す。
- emo2（<https://ekicyou.github.io/ghost_dev/emo2/>）は、ユーザーが開発する pasta.dll のショーケースゴーストである。存分に使ってよい（2026-10-06 ユーザー確認）。「挨拶したい」の章の切り替えの相手役のほか、各章で「この表現を作りこむとこうなる」実例として紹介してよい。ただし本文の作例は hello-pasta で書く。
- イベントに反応する章では、シーンがどの SHIORI イベントで呼ばれるかを示す。UKADOC の該当イベントの項へリンクし、そのイベントの説明と `Reference` の意味を出典付きで短く引用する。
- 「挨拶したい」の章は、emo2 との切り替えを題材にする（作例の確定は `hello-pasta-tutorial-stages` が emo2 開発と相談して行う）。この章は、イベントとシーンの対応の仕組みを初めて説明する章とする。シーン名をイベント名にすると呼ばれること、ほかに来るイベントは UKADOC で探せること、イベントの付加情報を `＞transfer_req_to_var` で `＄ｒ０`〜`＄ｒ９` に取り出して台詞に使えること（ショートカットアクセス）を、Claudia が順を追って語る。詳しくは文法章 `grammar/variables.md` の「リクエスト変数（Reference）」と、Lua 章 `lua/shiori-events.md` へ送り出す。
- 「挨拶したい」の章では、`OnGhostChanged` に応答すると `OnBoot` が来ないこと（204 を返したときだけ `OnBoot` へ回る）も語る。読者が起動の挨拶を二重に書く失敗を防ぐため（emo2 開発からの指摘）。

## Approach
- 章立ては、`hello-pasta-tutorial-stages` が確定する段階表に 1 章ずつ対応させる。準備の章（前提環境・最小一式の配置）を先頭に置く。
- 執筆規約に、`getting-started` に限った例外を設ける。説明本体も Claudia の語りで書いてよい。
  - キャラ口調を持ち込まない場所は維持する: コードブロック内・構文定義・コマンド例・サンプルコード。
  - 表のセルの扱いは要件で決める。
  - 技術的正確さを最優先するという鉄則も維持する。語りのせいで誤読の余地が生まれる書き方はしない。
- `verify-content.mjs` の口調検査を、ガイドの例外に合わせて直す。
- `tutorial-check.mjs` の照合を、完成形の 1 か所から、段階ごとの照合に広げる。
- `introduction.md` の「このマニュアルの歩き方」を、ガイドが物語で導き、文法・Lua がリファレンスであるという位置づけに直す。

## Scope
- **In**: `book/src/getting-started/` の全面的な書き直しと章の追加、`SUMMARY.md` の入門パート、`introduction.md` の案内、`book/AUTHORING.md` のガイド向けの例外、`verify-content.mjs`・`tutorial-check.mjs` とそのテスト、スクリーンショットの撮影と掲載
- **Out**: 文法リファレンス・Lua 章・内部設計パートの書き直し（リンク先として使うだけ）、段階辞書と hello-pasta の辞書（`hello-pasta-tutorial-stages`）、シェル画像（`hello-pasta-shell-art`）、スキル `references/` の生成対象の変更

## Boundary Candidates
- ガイドの章の本文（Claudia の語り）
- ガイド向けの執筆規約の例外と、その機械検査（`verify-content.mjs`）
- 段階辞書との逐語照合（`tutorial-check.mjs`）

## Out of Boundary
- 段階表の内容（どの表現をどの順で教えるか）の決定 — `hello-pasta-tutorial-stages` が持ち、本 spec はそれに従う
- 文法・API の新しい説明。ガイドに書く事実は、リファレンス章が権威として書いている事実の範囲に留める

## Upstream / Downstream
- **Upstream**: emo2 開発（ghost_dev。ゴースト作りこみの知見の出どころ。Claudia が語るコツや注意は、ここから引く。要点と置き場所は `hello-pasta-tutorial-stages/brief.md` の「emo2 からの知見」）、`hello-pasta-tutorial-stages`（段階表・段階辞書）、`manual-claudia-theme`（Claudia の台詞の部品と、その記法の執筆規約。ガイドの全編の語りはこの部品で書く）、`hello-pasta-shell-art`（スクリーンショットの絵）、`book/` の生成・検査ツール群
- **Downstream**: なし（後で他の章の初心者向け改訂を起票するなら、その手本になる）

## Existing Spec Touchpoints
- **Extends**: `pasta-user-manual`（完了済み。入門ガイドと Claudia 令嬢ボイスの規約を定めた）、`manual-ssot-authority`（完了済み。権威と生成の規約）— どちらも再オープンせず、本 spec が規約の例外を足す
- **Adjacent**: Phase 11 の未完了 spec がマニュアルの文法章を触ることがある。入門ガイドとはページが分かれる

## Constraints
- 現行の版で実装されている文法・API だけを書く。回避レシピは載せない。
- UKADOC からの引用は、出典リンク付きの短い引用に留める。`Reference` の一覧などを丸ごと転載しない（全体は UKADOC へのリンクで見せる）。
- マニュアルは利用者向けの唯一の権威である。ガイドの記述がリファレンス章と食い違ってはならない。
- スクリーンショットの撮影手段（実機の SSP で手動か、自動化か）は設計で決める。撮影できない環境でも CI は通ること。
- 完成度を優先する。一部の章だけ物語化して残りを旧形式のまま出す、といった部分出荷はしない。
