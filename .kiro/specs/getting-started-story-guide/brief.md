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
- **起動時の台詞の説明**: `first-ghost.md` ステップ 8-3 の「起動すると `OnBoot`（初回は `OnFirstBoot`）のセリフが表示される」は、7 段目の `＊OnGhostChanged` が応答するため、別のゴーストから切り替えたときは OnBoot ではなく OnGhostChanged の台詞が出る。5.2 は機械的な追従に限ったので直していない。本文を書き直すときに合わせる。

## 2026-10-10 棚卸の再測定（main add05022）

- **前提の変化**: 先に要る 3 本のうち、`manual-claudia-theme` と `hello-pasta-tutorial-stages` が main に入った。残るのは `hello-pasta-shell-art` だけ。
  - 段階表は `crates/pasta_sample_ghost/STAGES.md`（13 段。辞書を持つのは 1〜12 段目で、13 段目は手順だけ）。段階の辞書は配布版の辞書そのもの（`ghosts/hello-pasta/ghost/master/dic/` の `01-boot.pasta`〜`12-lua.pasta`、1 段 1 ファイル）で、全段が起動することをテスト（`tests/tutorial_stages_test.rs`）が確かめている。
  - `first-ghost.md` は 589 行になり、12 ファイルを順に貼る形へ機械的に直してある。語りと説明は古いまま。
  - 作例の照合（`book/tools/tutorial-check.mjs`、178 行）は、今も「`first-ghost.md` 1 枚のどこかに、辞書の各ファイルと同じ中身のブロックがあるか」だけを見る。段階と章の対応は見ない。
  - マニュアルの自動検査（`.github/workflows/manual.yml`）は、辞書の変更でも走るようになった。roadmap の「照合の落とし穴」は解消した。
  - 入門の章では、本文に台詞の部品を置いてよい（`book/AUTHORING.md` 第 7 節「置ける場所」）。本文の口調を禁じる機械検査は、入門の章にはかかっていない。規約の例外は主に `AUTHORING.md` の第 1・2・5 節の文章の話になり、検査（`book/tools/verify-content.mjs`、622 行）の直しは小さい（`first-ghost.md` を名指しする検査と、章の数）。
- **触るファイル**: `book/src/getting-started/`（3 枚を 16 枚前後に）、`book/src/SUMMARY.md`・`introduction.md`、`book/AUTHORING.md`、`book/tools/` の `tutorial-check.mjs`・`tutorial-check-test.mjs`・`verify-content.mjs`・`verify-scripts-test.mjs`・`talk/talk-test.mjs`（後ろの 2 つは章数 47 を決め打ちしている）、`book/book.toml`（古いページからの転送）、`crates/pasta_lua/README.md`・`crates/pasta_shiori/README.md`（`first-ghost.html` へのリンク）、スクリーンショット（新規）。段階表を照合に使うなら `manual.yml` の対象パスも。1,000 行に近いファイルは無い。
- **規模**: 22〜24 タスク（規約 1、検査ツールと自己テスト 4、目次・案内・転送 2、準備の章 2、段階の章 13 段で 9〜11、スクリーンショット 2〜3、通しの確認 1）。20 を超える。
- **先に要るもの**: 本文は今すぐ始められる。`hello-pasta-shell-art` を待つのは、スクリーンショットと 8 段目の実機の動き（今のシェルには当たり判定が無く、触った部位の名前が空になる）。他の未完了 spec とファイルは重ならない（`hello-pasta-shell-art` が辞書と `book/` を触らない場合）。
- **種別**: 文書（入門ガイドの書き直し。方針は開発者が 2026-10-06 に決めた）。
- **要件定義のモデル**: 本文の側は Fable（前回は Opus）。「全編を Claudia が語る」を、台詞の部品だけで書くか、地の文も Claudia の口調にするかで、16 枚全部の形と執筆規約が決まる。導入と締めに二人とも要る規則との折り合いも、開発者の判断になる。スクリーンショットの側は Opus。
- **分割の案**: 本文と絵の 2 つに分ける。
  - `getting-started-story-guide`（本文。今のウェーブ）: 規約の例外、検査ツール、全章の文章、目次と案内。18〜20 タスク。シェルのファイルの中身は本文に書き写さず（「hello-pasta のシェルをそのまま使う」とだけ書く）、画像は置かない。
  - `getting-started-screenshots`（新規。`hello-pasta-shell-art` と本文の後）: 撮影の手順、画像、各章への画像の行、新しいシェルでの 8 段目の実機確認。4〜6 タスク。
  - 本文を main に入れるのは `hello-pasta-shell-art` の後にする（8 段目が書いたとおりに動くため）。絵の無いガイドを先に公開してよいかは、開発者に確かめる（Constraints の「部分出荷はしない」との兼ね合い）。
- **見つけた穴・古くなった記述**:
  - 「途中の段階では起動しないことがある」（`first-ghost.md` 13 行目、`index.md` 15 行目）は、もう正しくない。
  - `first-ghost.md` 559 行目の「Lua ランタイム（`scripts/` 配下）も配置する」は誤り。ランタイムは `pasta.dll` の中にあり、`scripts/` は利用者が自分のスクリプトを置く場所。
  - `first-ghost.md` 378 行目は 8 段目を「書き方はこれまでと変わらない」と説明するが、今の作例は付加情報（`＄ｒ４`）を使う。
  - 前回の再測定の行数（`first-ghost.md` 447 行・`verify-content.mjs` 531 行・`tutorial-check.mjs` 152 行）と「先に要るものはすべて未完了」は古い。

## 2026-10-10 棚卸の分割

上の「分割の案」のとおり、本文と絵の 2 つに分けた。この spec は名前をそのままにして本文を持ち、絵は新しい `getting-started-screenshots` が持つ。上の Scope・Desired Outcome・Upstream のうち、絵にかかわる記述は、この節の内容で読み替える。

- **分割後の In**:
  - 執筆規約（`book/AUTHORING.md`）への、入門ガイドに限った例外
  - 本文の検査（`book/tools/verify-content.mjs`）の追従
  - 作例の照合（`book/tools/tutorial-check.mjs`）を、段階ごとの照合に広げることと、そのテスト（`book/tools/tutorial-check-test.mjs`）
  - 章の数を決め打ちしている自己テストの追従（`book/tools/verify-scripts-test.mjs` 72 行・`book/tools/talk/talk-test.mjs` 454 行。どちらも 47 章と書いている）。再測定の節に無かったものが、もう 1 か所ある。スキル用の文書を生成する道具の自己テスト（`book/tools/gen-skill-refs-test.mjs` 599 行）も、`book/src` の章を 47 と数えている。章を足すと 3 か所とも落ちる。
  - 全部の章の文章（`book/src/getting-started/` の入口の章・準備の章・13 段の章）
  - 目次（`book/src/SUMMARY.md`）、表紙の案内（`book/src/introduction.md`）、古いページからの転送（`book/book.toml`）、`first-ghost.html` を指している 2 つのリンク（`crates/pasta_lua/README.md`・`crates/pasta_shiori/README.md`）
- **分割後の Out**（これまでの Out に足す）:
  - スクリーンショットの撮影と掲載、画像ファイル、章への画像の行（`getting-started-screenshots` へ）
  - 新しいシェルでの 8 段目の実機の確かめ（同上）
- **移したもの**: Scope の「スクリーンショットの撮影と掲載」、Desired Outcome の「新しいシェルで、ゴーストが実際にしゃべる様子をスクリーンショットで示す」、Constraints の「スクリーンショットの撮影手段」。先に要るものから `hello-pasta-shell-art` が外れ、未完了の前提は無くなった。
- **境界の決まり**:
  - シェルのファイル（`descript.txt`・`surfaces.txt`）の中身を、本文に書き写さない。「hello-pasta のシェルをそのまま使う」と書いて、置き場所へ案内する。今の `first-ghost.md` は 69 行目と 84 行目から書き写しているので、書き直しで消す。
  - 本文に画像を置かない。
  - 触った部位の名前は、段階表（`crates/pasta_sample_ghost/STAGES.md`）から取る。`hello-pasta-shell-art` は、頭の部位名を今の `Head` のまま確定し、hello-pasta の辞書（`dic/`）と `book/` を触らない。
  - このウェーブでは、マニュアルの自動検査の定義（`.github/workflows/manual.yml`）と、出力の検査（`book/tools/verify-static.mjs`）を触らない。この 2 つは、同じウェーブの `manual-print-media-refs` が持つ。段階表を照合に使うために自動検査の対象を広げたくなったら、次のウェーブへ回す。
- **順番**: 本文は今のウェーブで始める。`hello-pasta-shell-art` と同じ時期に進められる（上の境界の決まりを守れば、ファイルは重ならない）。絵は、新しいシェルと本文の両方が main に入った後に `getting-started-screenshots` が足す。本文が新しいシェルより先に main に入ると、それまでの間、8 段目は当たり判定の無い今のシェルで動く。触った部位の名前は空になるが、8 段目の辞書（`dic/08-touch.pasta`）は空でも読めるように書いてある。
- **規模**: 本文が 18〜20 タスク（規約 1、検査ツールと自己テスト 4、目次・案内・転送 2、準備の章 2、段階の章 9〜11、通しの確認 1）。絵が 4〜6 タスク。
- **触るファイル**:
  - 本文（この spec）: `book/src/getting-started/`（3 枚を 16 枚前後に）、`book/src/SUMMARY.md`、`book/src/introduction.md`、`book/AUTHORING.md`、`book/book.toml`、`book/tools/` の `verify-content.mjs`・`tutorial-check.mjs`・`tutorial-check-test.mjs`・`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`（章の数の 1 行だけ）、`crates/pasta_lua/README.md`、`crates/pasta_shiori/README.md`。
  - 絵（`getting-started-screenshots`）: 画像ファイル（新規）、撮影の手順の記録（新規）、`book/src/getting-started/` の各章の画像の行。
- **要件定義のモデル**: 本文は Fable、絵は Opus（再測定の節のとおり）。
- **開発者に確かめること**: 絵の無い入門ガイドを、先に公開してよいか。Constraints は「部分出荷はしない」と決めている。分割の後は、本文が先に main に入り、絵が後から付く。本文は全部の章が物語の形になっていて、旧形式のまま残る章は無い。今の入門ガイドにも絵は無い。これを部分出荷と見るなら、本文を main に入れるのを、`hello-pasta-shell-art` と絵の後まで待つ。
- **申し送り（完了した `hello-pasta-tutorial-stages` の実機の確かめから）**: ゴースト自身がさくらスクリプトの `\![change,ghost]` で切り替えたときは、`OnGhostChanging` が届かない。SSP のメニューから切り替えたときは届く（`.kiro/specs/completed/hello-pasta-tutorial-stages/tasks.md` の実装メモ 6.1 の追記）。切り替えを説明する 7 段目「挨拶したい」の章で、読者に試してもらう手順は、メニューからの切り替えで書く。
