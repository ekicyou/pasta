# Brief: hello-pasta-tutorial-stages

## Problem
入門ガイドを「こんな表現をしたい」を順に叶えていく物語に作り直すには、章ごとの段階で読者の手元のゴーストが起動し、その章で覚えた表現を試せる必要がある。今の hello-pasta は完成形が 1 つあるだけである。しかも辞書のコメントはテスト向けの説明（「テスト安定性のため単一シーン」「プロパティスコープ統合テスト」など）で、教材として読ませる形になっていない。

## Current State
- hello-pasta の辞書は `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/` の 5 ファイル（`actors`・`boot`・`talk`・`click`・`choice`、計 187 行）。これが配布物の SSOT である。
- 入っている表現: 起動・初回起動・終了のあいさつ、ランダムトーク（`OnTalk`）、時報（`時報12`・`時報その他`）、クリックへの反応、選択肢、単語のランダム選択、アクター辞書の表情、`＄％` によるプロパティの読み取り。
- 入っていない表現: ゴーストの切り替えなどへの挨拶（`OnGhostChanged` など）、イベントの付加情報の取り出し（`＞transfer_req_to_var`・`＄ｒ０`）、変数を覚えさせる（永続化）、Call/Jump による会話の続き・分岐、Lua との連携。
- `book/src/getting-started/first-ghost.md` は完成形の辞書 5 ファイルを逐語で転記している。`book/tools/tutorial-check.mjs`（`manual.yml` で実行）が、その逐語一致を検査する。
- 構文の妥当性は `cargo test -p pasta_sample_ghost`（`self_deploy_integration_test.rs` が実際に読み込む）が担保する。

## Desired Outcome
- 「こんな表現をしたい」を 1 段ずつ広げる**段階表**が確定している。段階ごとに、新しく覚える表現と使う文法要素が決まっている。
- 段階ごとの辞書一式（段階辞書）がリポジトリにあり、どの段階も、そのまま読み込めて起動できることを CI が確かめる。
- 最終段階の辞書は hello-pasta の辞書と一致する。hello-pasta の辞書は、教材として読める作例とコメントに書き直されている。
- 既存のテスト（`OnBoot` の決定性など）が依存している挙動は、テスト側を直すか作例の形を保つかして、壊さない。

## Approach
段階表のたたき台（確定は要件定義で行う）:

1. しゃべらせたい（起動して一言）
2. 二人で掛け合いさせたい
3. 表情を変えたい
4. 毎回ちがうことを言わせたい（同じ名前のシーン・単語）
5. 暇なときに話しかけてほしい（ランダムトーク）
6. 時刻を知らせたい（時報）
7. 挨拶したい（ゴーストの切り替えなど、ベースウェアからのイベントへの反応）
8. 触ったら反応してほしい
9. 選ばせたい（選択肢）
10. 覚えていてほしい（変数の保存）
11. 話を続けたい・分岐させたい（Call/Jump）
12. もっと凝ったことをしたい（Lua への入り口）
13. 配布したい（`.nar` にする）

「挨拶したい」の段階で、SHIORI イベントとシーンの対応（シーン名＝イベント名で呼ばれること）と、イベントの付加情報（`Reference`）を `＞transfer_req_to_var` で `＄ｒ０`〜`＄ｒ９` に取り出す書き方を初めて扱う。切り替えの相手役のゴーストは **emo2**（ghost_dev リポジトリで開発中。2026-10-06 ユーザー指示）とする。作例は、`OnGhostChanged`（emo2 から切り替わってきた）で直前のゴースト名 `＄ｒ０` を呼んで挨拶する形などを候補とし、扱うイベントは要件で決める。直後の「触ったら反応してほしい」は、この書き方を使って `OnMouseDoubleClick` の部位（`＄ｒ４`）を読む。

段階辞書は「前の段階＋差分」で育つ形にする。読者が前の章の成果に書き足すだけで次の段階になることを原則とする。どうしても書き換えが要る段階は、要件で明示する。段階辞書の置き場所と、CI での検証方法（既存の統合テストの拡張か、新しいテストか）は設計で決める。

## emo2 からの知見（2026-10-06「emo2 開発」より）

要件定義で段階表と作例を決めるときの材料。どれも読み取りで使う。現行の pasta と食い違わないかは、マニュアル（権威）で確かめてから採る。

### 置き場所（`C:\home\maz\git\ghost_dev` 配下）
- `.claude/skills/emo2-authoring/SKILL.md` — 辞書の約束ごと（OnTalk をカテゴリのディスパッチャで振り分ける形、`dic/` の構成、アクター行の `：` の位置合わせ）
- `.claude/skills/emo2-characters/`（＋`references/`）— 性格・口調・掛け合いのパターン表・禁忌・トーク生成の制約
- `.kiro/steering/characters.md`・`tech.md`・`workflow.md`、`.kiro/specs/completed/` の次の spec
  - `emo2-talk-port`（時報）
  - `emo2-talk-enrichment`・`emo2-te-*`（ランダムトーク・スキット・季節）
  - `emo2-menu-pasta`（選択肢）
  - `emo2-actors-diversity`・`murasaki1000-script-builder`（表情）
  - `emo2-word-dic`・`emo2-word-kaiki`（単語）
  - `pasta-no-recipe`
- 実際の辞書: `project/emo2/ghost/emo2/ghost/master/dic/*.pasta`、`scripts/`（`boot.lua`・`murasaki1000.lua`・`touch_detect.lua`）
- ハマりどころのメモ（`C:\Users\maz-o\.claude\projects\C--home-maz-git-ghost-dev\memory\`）: `pasta-sakura-script-emission.md`・`pasta-scene-prefix-collision.md`・`pasta-budoux-kinsoku-pending.md`

### 段ごとの要点
- **掛け合い**: アクター行の `：` を縦にそろえる（短い名前の前に全角空白を足す）。1 トークは 1〜2 往復のショートコントに留め、「ボケ→ツッコミ→オチ」で組む。
- **表情**:
  - 台詞の頭に `＠表情名` を置く。1 行の中で `＠驚き　…\w9＠照れ　…` のように表情を切り替える「表情チェイン」がよく効く。
  - 表情の定義はアクター辞書でも Lua の関数でもよい。入門ではアクター辞書で教える。
  - emo2 開発は「字のない `＠通常` だけの行が余分な空行を生む」と言っていたが、これは `paragraph-break-tag-only-talk`（#69）で修正済み。現行の main では起きない。
- **毎回ちがう台詞**: 同じ名前のシーンを複数書くと、ランダムに選ばれる。1 場面に 3 通り程度が目安。
- **ランダムトーク**:
  - `OnTalk` はディスパッチ専用にする。`＞通常トーク` などの複数定義で比率を決め（emo2 は 5:2:1）、中身は別名のシーンに書く。
  - 間隔は `pasta.toml` の `[ghost] talk_interval_min`・`talk_interval_max` で決める。`＄＊pasta_talk_interval_min`・`_max` を書き換えれば、実行中にも変えられる。
- **時報**: `＊時報00`〜`＊時報23` から、時間帯のシーン（`＞時報深夜` など、各 3 通り）へ回す二段構え。
- **挨拶・イベント**:
  - 変数の直後には空白を置く（`＄ｒ１　さんが来たで！`）。マニュアルの規則どおり。
  - 黙らせたいとき（204 にしたいとき）は、何も話さずに終える。204 なら `OnGhostChanging`・`OnGhostChanged` は `OnClose`・`OnBoot` へ回る。
  - イベントの性質に合わせた長さにする（`OnInstallBegin` は一言だけ、など）。
- **ハマりどころ**:
  - シーン検索の前方一致。`＊OnInstallComplete` と `＊OnInstallCompleteAll`、`＊時報深夜` と `＊時報深夜0時` で、約半分が外れた。
  - Lua の文字列でさくらスクリプトを書くときは `[=[ ... ]=]` を使う。
  - 長い台詞はバルーンからあふれる。名前が入る台詞は短めにする。
- **変数の保存**: `＄＊名前` は保存される（emo2 はおしゃべりの頻度に使っている）。

emo2 の辞書を作例や Claudia の語りに引用してよい（emo2 開発の了承済み）。個別のトークの実例が要るときは、emo2 開発に頼めば出してもらえる。

## Scope
- **In**: 段階表の確定、段階辞書一式、hello-pasta の辞書の書き直し（教材化・不足する表現の追加）、全段階を読み込む検証、既存テストの更新
- **Out**: マニュアル本文の執筆（`getting-started-story-guide`）、シェル画像（`hello-pasta-shell-art`）、DSL・ランタイムの機能追加（段階表は現行実装で書ける表現だけを使う）

## Boundary Candidates
- 段階表（どの表現をどの順で教えるか）— ガイド本文との共有の接点
- 段階辞書とその検証（`crates/pasta_sample_ghost` とテスト）

## Out of Boundary
- 章の文章・Claudia の語り・執筆規約（`getting-started-story-guide`）
- `tutorial-check.mjs` の照合方式の変更（ガイド本文の構成に依存するので `getting-started-story-guide` が持つ。本 spec は段階辞書の置き場所と形を申し送る）

## Upstream / Downstream
- **Upstream**: emo2（ghost_dev。切り替えの相手役）、現行の Pasta DSL・ランタイム（段階表は現行実装で書けることを確かめてから確定する）、`crates/pasta_sample_ghost` のテスト
- **Downstream**: `getting-started-story-guide`（段階表と段階辞書を逐語で使う）

## Existing Spec Touchpoints
- **Extends**: なし
- **Adjacent**: `hello-pasta-shell-art`（同じクレートの画像側）、`release-workflow`（hello-pasta の `.nar` の中身が変わる）、Phase 11 の未完了 spec（`scene-attribute-store` など。hello-pasta の辞書は触らない見込み）

## Constraints
- emo2 との切り替えの作例は、「emo2 開発」セッション（ghost_dev）と相談して決める。2026-10-06 の回答の要点は次のとおり。
  - **名前**: 配布版の `name` は `えも？？`（Reference2 に入る）、`sakura.name` は `むらさき`（Reference0 に入る）、`kero.name` は `エモ`。デバッグ版は `name` が `えも2DEBUG` になるので、名前で分岐するなら Reference0（`むらさき`）で分ける。emo2 が受け取る hello-pasta の Reference0 は `女の子`。
  - **入手**: GitHub Pages で公開されている（<https://ekicyou.github.io/ghost_dev/emo2/>・`emo2.nar`）。areka のアルファにも既定のゴーストとして同梱されている。emo2 はユーザー自身が開発する pasta.dll のショーケースゴーストであり、ガイドで公開ゴーストとして案内してよい（2026-10-06 ユーザー確認。「存分に使ってください」）。
  - **emo2 側の反応**: もう入っている。`OnGhostChanging`（送り出し）と `OnGhostChanged`（迎え入れ）が 3 通りずつあり、`＄ｒ０` で相手の名前を呼ぶ（例「＄ｒ０　にバトンタッチやな！」「＄ｒ０　からバトン受け取ったで！」）。汎用の文なので、「女の子さん」のような少し不自然な言い回しが出る。hello-pasta 専用の台詞を emo2 側に足すかどうかはユーザーの判断。
  - **台詞の制約（hello-pasta 側）**: むらさきは関西弁、エモは標準語。エモは「むらさきにしか見えない」感情の妖精なので、話しかける相手はむらさきにするほうが自然。むらさきの「おばあちゃん」には触れない。二人を冷たい・いじわるに描かない。二人は互いに呼び捨てで、ユーザーを「ユーザーさん」と呼ぶ。
- 作例で教える注意（emo2 開発からの指摘）: `OnGhostChanged` に応答すると `OnBoot` は来ない（SSP・areka とも、204 を返したときだけ `OnBoot` へ回る）。読者が起動の挨拶を二重に書かないよう、この段で説明する。
- シーン名の付け方の注意（同じく emo2 開発からの指摘）: シーン検索は前方一致の候補をまとめて抽選する。既存のシーン名で始まる名前を付けると、既存の呼び出しの候補に混ざる。作例のシーン名はこれを踏まえて付ける。
- ゴーストとしての作りこみの知見は、emo2 開発（ghost_dev）から引く（2026-10-06 ユーザー方針）。表情の付け方、掛け合いのテンポ、ランダムトークの量と間隔、反応するイベントの選び方、変数の使いどころ、pasta で書くときのハマりどころなどが対象。段階表と作例は、この知見に照らして決める。回答は「emo2 からの知見」の節にまとめた。
- 現行の版で実装されている文法・API だけを使う（マニュアルの方針）。「書けるだけ」の書き方や、意図を推測して救済されるような書き方を作例にしない。
- 時報など時刻やランダムに依存する段階も、検証では「読み込めて起動できる」ことを確かめる（決定的な出力の照合までは求めない）。
- 配布物 hello-pasta の辞書が変わるので、サンプルゴーストのリリースで中身の変化が利用者に分かるようにする。

## 2026-10-07 棚卸の再測定（main 2cbaf510）

- **前提の変化**: 起票文の記述はそのまま正しい（`dic/` 5 ファイル・計 187 行。`＞transfer_req_to_var`・`＄ｒ０`・`＄＊pasta_talk_interval_min` はマニュアルの記述どおり）。シーン検索は `scene-search-key-normalization`・`scene-identity-format` の後も前方一致のまま（`grammar/call-jump.md` 119 行）。`paragraph-break-tag-only-talk` も完了している。
- **触るファイル**: `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/*.pasta`、段階辞書（新規。置き場所は設計）、`tests/self_deploy_integration_test.rs`（333 行）か新しいテスト、`tests/integration_test.rs` の辞書のテスト（269〜365 行）、`tests/dist_src_validation_test.rs`、`src/scripts.rs`（辞書の単体テスト）。1,000 行に近いファイルは無い。
- **規模**: 15〜18 タスク（段階表、13 前後の段階を 2〜3 段ずつ、検証の仕組み、hello-pasta の書き直し、既存テスト、マニュアルとの同期）。20 を超えそうなら段階をまとめる。
- **先に要るもの**: 機能の依存は無い。
- **ファイルの重なり**: `hello-pasta-shell-art`（`tests/integration_test.rs`。どちらもウェーブ 1 なので順序を決める）。
- **もう一つの重なり**: hello-pasta の辞書を書き換えると、`tutorial-check.mjs` が `first-ghost.md` の ```pasta ブロックとの逐語一致で落ちる。しかも `manual.yml` の paths は `crates/pasta_sample_ghost/ghosts/**` を含まないので、本 spec の PR では検査が走らない。次に `book/` を触る別の PR（`manual-claudia-theme` など）で初めて赤くなる。本 spec の中で `first-ghost.md` の作例を新しい辞書に合わせる（`manual-claudia-theme` とは同じページの別の節）か、`manual.yml` の paths に辞書を足すかを要件で決める。
- **種別**: 機能（教材としての段階辞書と、その CI 検証。作例の相手役 emo2 は開発者の指示）。
- **要件定義のモデル**: Fable（段階表は後続のガイドが逐語で使う土台。扱うイベントや emo2 との作例に開発者の判断が要る）。
- **分割の案**: なし（Phase 12 で一度切り分けている）。
- **見つけた穴・古くなった記述**:
  - `tests/dist_src_validation_test.rs:8-17` の必須ファイルに `dic/choice.pasta` が無い。
  - 「Lua への入り口」の段で `scripts/` に Lua を置く場合、今は `release.ps1` が `crates/pasta_lua/scripts`（README だけ）を robocopy の `/MIR` で上書きする。段階辞書・hello-pasta に Lua を置くなら、`scripts/` の扱いを `release-ci` と合わせる。
