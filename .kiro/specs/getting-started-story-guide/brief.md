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

## 2026-10-07 棚卸の再測定（main 2cbaf510）

- **前提の変化**: 無い。`getting-started/` は `index.md`・`prerequisites.md`・`first-ghost.md`（447 行）の 3 ファイル。`tutorial-check.mjs` は `first-ghost.md` 1 か所と hello-pasta の `dic/` を照合し（`TUTORIAL_REL`・`HELLO_DIC_REL`）、`verify-content.mjs` もそれを import して使う。口調の判定の実体は `gen-skill-refs.mjs` の `findVoice`。
- **触るファイル**: `book/src/getting-started/`（全面の書き直しと新しい章）、`book/src/SUMMARY.md`、`book/src/introduction.md`、`book/AUTHORING.md`、`book/tools/` の `verify-content.mjs`（531 行）・`tutorial-check.mjs`（152 行）・`tutorial-check-test.mjs`・`verify-scripts-test.mjs`、スクリーンショット（新規）。起票文に無いが、口調の例外を判定に入れるなら `gen-skill-refs.mjs` の `findVoice` の周りも、段階辞書の照合を CI で確実に走らせるなら `.github/workflows/manual.yml` の paths も触る。
- **規模**: 18〜20 タスク（規約の例外・検査ツール 2 本・準備の章・13 前後の段階の章を 1〜2 章ずつ・目次と案内・スクリーンショット・通しの確認）。超えそうなら章をまとめる。
- **先に要るもの**: `hello-pasta-tutorial-stages`・`hello-pasta-shell-art`・`manual-claudia-theme`（すべて未完了）。`hello-pasta-shell-art` が `release-ci` の後なので、`release-ci` も間接の前提になる。
- **ファイルの重なり**: `manual-claudia-theme`（`AUTHORING.md`・`introduction.md`・`getting-started/*`・`verify-content.mjs`）、`hello-pasta-tutorial-stages`（`first-ghost.md` の作例を同期した場合）。どちらも先のウェーブなので順序で解決している。
- **種別**: 文書（入門ガイドの書き直し。emo2 の扱いなどは開発者の確認済み）。
- **要件定義のモデル**: Opus（段階表・記法・絵は上流の spec が決める。残る判断は表のセルの口調と撮影の手段くらい）。
- **分割の案**: なし。仮に 20 を大きく超えたら、「規約の例外・検査ツール・準備の章・前半の段階」と「後半の段階・スクリーンショット・目次と案内」に分ける。
- **見つけた穴・古くなった記述**:
  - `first-ghost.md` の ```text ブロック（シェルの `descript.txt` など）は `tutorial-check.mjs` の照合の外にある。`hello-pasta-shell-art` でシェルが変わると黙って古くなる。照合の対象を広げるかを要件で決める。
  - `verify-content.mjs` には専用のテストが無く、`verify-scripts-test.mjs` が子プロセスで走らせるスモークだけ。Scope の「そのテスト」は、このスモークと `tutorial-check-test.mjs` を指すことになる。

## 申し送り（manual-claudia-theme より、2026-10-09）

`manual-claudia-theme` が完了し、ガイドの語りに使う台詞の部品と、その検査が入った。要件・設計を決めるときの前提として、変わった事実と見直しの論点だけを渡す（決めるのはこの spec）。参照先は `.kiro/specs/completed/manual-claudia-theme/design.md`。

- **記法と登録簿**: 台詞は `> 【表情】本文`／`> 【話し手：表情】本文`（全角コロン、話し手名はカタカナ）。話し手と表情の唯一の定義は `book/tools/talk/talk.mjs` の `SPEAKERS`（クローディア 14 表情・アンソニー 2 表情）。書き方・表情の使いどころ・掛け合いの目安は `book/AUTHORING.md` 第 7 節「台詞部品（Claudia とアンソニー）」。ガイド向けの例外はこれとは別の節に書く（同じ節を書き換えない）。
- **入門の章の今の導入・締め**: `getting-started/` の 3 章は、導入と締めが既に二人の掛け合いになっている（趣旨は元の Claudia の台詞のまま）。ガイドの全面書き直しはこれを置き換えてよい。
- **全章にかかる検査**: SUMMARY に載る章はすべて、`verify-content` の T-syntax・T-intro・T-outro（導入と締めは台詞だけ・両方の話し手が要る）、`verify-static` の台詞部品の検査（全章と `print.html` に両方の話し手の部品がある）、`verify-search` の台詞の検索の検査（各章の最初の台詞の 4 文字以上の日本語片で、その章がヒットする。その語は本文に出ない語にする）にかかる。章を足すとそのまま対象になる。
  - 論点: ガイドを「全編 Claudia が語る」形にすると、導入・締めの規則（両方の話し手が要る）と本文の扱いをどう両立させるか。`getting-started/` は生成対象章でも内部設計章でもないので、本文に台詞を置いても `talk-in-body` にはならない（T-syntax はかかる）。
  - 論点: 章数 47 が自己テストに書かれている（`verify-scripts-test.mjs` の T 系の件数、`talk/talk-test.mjs` の J-9）。章を足すときは合わせて直す。
- **表紙**: `introduction.md` の先頭は扉（`<section class="claudia-hero">`）になり、パート案内（`hero-toc`）が入門の最初の章 `getting-started/index.md` を指している。扉と締めのクラス契約は `book/theme/claudia.css` の冒頭のコメント。入門の案内を変えるときはこの契約に沿う。
- **CI の段**: `manual.yml` は着色の後に台詞の変換（`talk/talk-html.mjs`）を挟み、verify-static/search は `--self-test` 付きで動く。最後に着せ替え前の版を `classic/` に作る段がある（新版の検査には混ざらない）。

## 申し送り（hello-pasta-tutorial-stages より、2026-10-09）

- **ランダムトークの間隔**: 配布版 hello-pasta の `pasta.toml` は `[ghost]` の `talk_interval_min = 45`・`talk_interval_max = 75`（平均 1 分）になる。読者が `first-ghost.md` ステップ 7 の最小構成（`[actor]` だけ）で作ると既定の 180〜300 秒のままで、2 段目の `＊会話` を確かめるのに数分待つ。段階表（`crates/pasta_sample_ghost/STAGES.md`）の 2 段目は「すぐ確かめたいときは `[ghost]` で間隔を短くする」を新しく覚える表現に含めている。設定の段は独立させていない（設定の段は辞書ファイルを持たず、1 段 1 ファイルの組み立て方が崩れるため）。ガイド本文では 2 段目で `[ghost]` の 2 行を書き足させるのが自然。
