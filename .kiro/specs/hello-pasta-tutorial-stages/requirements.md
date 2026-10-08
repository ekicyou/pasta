# Requirements Document

## Introduction

入門ガイドを「こんな表現をしたい」を 1 段ずつ叶えていく物語に作り直すため、題材の hello-pasta を**段階ごとに起動できる辞書（段階辞書）**として育て直す。本 spec は、(1) どの表現をどの順で教えるかの**段階表**を確定し、(2) 段階ごとの辞書一式をリポジトリに置いて**どの段階もそのまま読み込めて起動できる**ことを CI で確かめ、(3) 配布物 hello-pasta の辞書（`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/`）を**最終段階と一致する教材**として書き直す。既存のテストが依存する挙動（`OnBoot` の決定性など）は、テスト側を直すか作例の形を保つかして壊さない。

ガイド本文の執筆（Claudia の語り・章の文章）は下流の `getting-started-story-guide` が持つ。本 spec は段階表と段階辞書を、下流が逐語で使える形で確定して申し送る。

> 本書の「[OPEN-n]」は、ディスカバリ（`brief.md`）だけでは確定できず、要件ディスカッションで決める論点である。各論点には暫定の仮定を明記し、その仮定で要件を書いている。

## Boundary Context

- **In scope**: 段階表の確定、段階辞書一式（全段階）、hello-pasta の辞書の書き直し（教材化・不足する表現の追加）、全段階を実際に読み込む検証、既存テスト・検査ツールの更新（辞書の変更に追従する範囲）、配布物の変化を利用者に知らせる記述
- **Out of scope**: マニュアル本文の執筆と章立て（`getting-started-story-guide`）、`tutorial-check.mjs` の照合方式の変更（同 spec）、シェル画像（`hello-pasta-shell-art`）、DSL・ランタイムの機能追加（段階表は現行実装で書ける表現だけを使う）、emo2 側の辞書の変更（ghost_dev リポジトリ）
- **Adjacent expectations**:
  - `getting-started-story-guide` は、本 spec が確定した段階表の順に 1 章ずつ対応させ、段階辞書を逐語で引用する。段階辞書の置き場所と形は本 spec が決めて申し送る。
  - `release-workflow` / `release-ci` は hello-pasta の `.nar` を作る。辞書の中身が変わるため、利用者に変化が分かる記述を本 spec が残す。
  - `hello-pasta-shell-art` は同じクレートの画像側を触る。本 spec はアクター辞書の表情名と `\s[n]` の対応（surface 番号）を変えない。
  - emo2（ghost_dev）は切り替えの相手役。emo2 側の `OnGhostChanging` / `OnGhostChanged` の反応はすでに入っており、hello-pasta 専用の台詞を足すかはユーザーの判断（本 spec の範囲外）。

## Requirements

### Requirement 1: 段階表の確定

**Objective:** As a 入門ガイドの執筆者（`getting-started-story-guide`）, I want 「こんな表現をしたい」を 1 段ずつ広げる段階表が確定していること, so that 物語の章立てと作例を食い違いなく書ける。

#### Acceptance Criteria

1. The 段階表 shall リポジトリ内の 1 か所に置かれ、段階ごとに「叶えたい表現（願い）」「新しく覚える表現」「使う文法要素」「その段階で初めて扱うベースウェアのイベント（あれば）」「前段階からの差分の種別（書き足しのみ／書き換えあり）」を持つ。
2. The 段階表 shall `brief.md` の段階のたたき台（1. しゃべらせたい → 2. 二人で掛け合い → 3. 表情 → 4. 毎回ちがうこと → 5. ランダムトーク → 6. 時報 → 7. 挨拶（ベースウェアのイベント） → 8. 触ったら反応 → 9. 選択肢 → 10. 覚えていてほしい（変数の保存） → 11. 話を続ける・分岐 → 12. Lua への入り口 → 13. 配布）の順を基本とし、順を変える場合はその理由を段階表に記す。**[OPEN-11]** 仮定: DSL 単体には条件分岐が無いため、11 段目は Call・ローカルシーン・チェイントークによる「話の続き」とし、条件による分岐は 12 段目の Lua で扱う。
3. The 段階表 shall 各段階で使う文法要素と API を、現行の版のマニュアル（`book/src/`）に書かれているものだけから選ぶ。マニュアルに無い書き方、「書けるだけ」の書き方、意図の推測で救済される書き方は作例に使わない。
4. The 段階表 shall 「挨拶したい」の段階で、シーン名＝イベント名で呼ばれること、イベントの付加情報を `＞transfer_req_to_var` で `＄ｒ０`〜`＄ｒ９` に取り出す書き方、`OnGhostChanged` に応答すると `OnBoot` は来ないこと（204 を返したときだけ `OnBoot` へ回る）を、初めて扱う内容として位置づける。
5. The 段階表 shall 「触ったら反応してほしい」の段階で、前段で覚えた `＞transfer_req_to_var` を使って `OnMouseDoubleClick` の部位（`＄ｒ４`）を読む形を採る。
6. When ある段階が前段階の辞書への書き足しだけでは成立せず既存の行の書き換えを要する場合, the 段階表 shall その段階と書き換える箇所を明示する。
7. The 段階表 shall 「配布したい」の段階を、辞書の差分を持たない段階（手順のみ）として扱う。**[OPEN-1]** 仮定: 段階辞書は「配布したい」を除く 12 段とし、12 段目（Lua への入り口）の辞書が hello-pasta と一致する。

### Requirement 2: 段階辞書一式

**Objective:** As a 入門ガイドの読者, I want 各章の終わりに自分のゴーストが起動して、その章で覚えた表現を試せること, so that 「書いたら動いた」を毎章で体験できる。

#### Acceptance Criteria

1. The 段階辞書 shall 段階表の辞書を持つ各段階について、その段階だけで完結して読み込める辞書一式（`.pasta` ファイル群）としてリポジトリに置かれる。
2. The 段階辞書 shall 「前の段階＋差分」で育つ形を原則とし、読者が前段階の成果に書き足すだけで次の段階になる。書き換えを要する段階は Requirement 1.6 で明示されたものに限る。
3. The 最終段階の段階辞書 shall hello-pasta の配布辞書（`ghosts/hello-pasta/ghost/master/dic/*.pasta`）と、ファイル構成・ファイル名・内容のすべてにおいて一致する。
4. The 段階辞書 shall 各段階で、アクター辞書の表情名と surface 番号の対応（`女の子`: `\s[0]`〜`\s[8]`、`男の子`: `\s[10]`〜`\s[18]`）を hello-pasta と同じに保つ。
5. The 段階辞書 shall 新しく付けるシーン名を、既存のシーン名で始まる名前（前方一致で既存の呼び出しの候補に混ざる名前）にしない。
6. While 段階辞書が 1 段階ずつ育つ, the 各段階の辞書 shall hello-pasta の `pasta.toml`・`descript.txt`・`install.txt`・シェルをそのまま使って起動できる（段階ごとに設定ファイルの差し替えを要しない）。**[OPEN-2]** 仮定: 段階辞書は辞書（`dic/`）だけを持ち、設定・シェルは hello-pasta のものを共用する。「Lua への入り口」で `scripts/` のファイルを置く場合はその段階だけが `scripts/` を持つ。
7. The 段階辞書の置き場所と形（ディレクトリ構成・段階番号の付け方・各段階の差分の読み取り方） shall 下流の `getting-started-story-guide` が逐語で参照できるよう、クレート内の説明ファイルに記される。

### Requirement 3: hello-pasta 辞書の教材化

**Objective:** As a 入門ガイドの読者, I want hello-pasta の辞書そのものが、読んで学べる作例とコメントになっていること, so that 配布物を開いたときに教材の最終形として読める。

#### Acceptance Criteria

1. The hello-pasta の辞書 shall テスト向けの説明（「テスト安定性のため単一シーン」「プロパティスコープ統合テスト」「7種以上」など）を含まず、コメントは読者に向けた「何を表現しているか・どう書くか」の説明になっている。
2. The hello-pasta の辞書 shall 既存の表現（起動・初回起動・終了のあいさつ、ランダムトーク、時報、クリックへの反応、選択肢、単語のランダム選択、アクター辞書の表情）を引き続き含む。
3. The hello-pasta の辞書 shall 次の表現を新たに含む: ベースウェアからの切り替えイベントへの挨拶（`OnGhostChanged`。相手役は emo2）、イベントの付加情報の取り出し（`＞transfer_req_to_var`・`＄ｒ０`・`＄ｒ４`）、変数の保存（`＄＊名前`）、会話の続きと分岐（Call・ローカルシーン・チェイントークのいずれかを含む）、Lua との連携の入り口。
4. The 切り替えの作例 shall emo2 からの切り替え（`OnGhostChanged`）で `＄ｒ０`（直前のゴーストの本体側の名前。emo2 では `むらさき`）を台詞に使い、`＄ｒ０` の直後に空白を置く。**[OPEN-3]** 仮定: `OnGhostChanged`（迎え入れ）に加えて `OnGhostChanging`（emo2 への送り出し）も作例に含める。
5. The 切り替えの作例の台詞 shall emo2 の制約に従う: むらさき・エモを冷たく・いじわるに描かない、「おばあちゃん」に触れない、ユーザーは「ユーザーさん」と呼ぶ。
6. The 変数の保存の作例 shall `＄＊名前` に入れた値がゴーストを終了しても残ることを読者が確かめられる形（次回以降の起動や反応で値が台詞に現れる）にする。**[OPEN-4]** 仮定: 保存する値は「触られた回数」とし、`OnMouseDoubleClick` で増やして台詞に出す（`OnBoot` の出力は変えない）。
7. The Lua との連携の作例 shall マニュアルの「条件分岐の実現」（Lua の関数が返す名前を `＞＠関数（）` で呼ぶ）か `scripts/` の記述パターンのいずれかに従う。**[OPEN-5]** 仮定: シーン内の Lua ブロックで分岐先を決める関数を書き、`scripts/` は使わない。
8. The hello-pasta の辞書 shall `＄％currentghost.name` によるプロパティの読み取りの作例を保つか、段階表のどこにも属さないなら削る。**[OPEN-6]** 仮定: 「挨拶したい」の段階に移し、自分のゴースト名を名乗る形で保つ。
9. The hello-pasta の辞書 shall 現行の版の文法・API だけで書かれ、`cargo test -p pasta_sample_ghost` の実ローダーによる読み込みを通る。

### Requirement 4: 全段階の検証

**Objective:** As a リポジトリの保守者, I want どの段階の辞書も実際に読み込めて起動できることを CI が確かめること, so that 辞書やエンジンが変わっても教材の作例が壊れたまま公開されない。

#### Acceptance Criteria

1. When `cargo test -p pasta_sample_ghost` が実行される, the 検証 shall 段階辞書の全段階について、実ローダーで読み込み（パース・トランスパイル・Lua 起動）が成功することを確かめる。
2. When 段階辞書の読み込みが成功した, the 検証 shall その段階の辞書に対して `OnBoot` のリクエストを送り、エラーにならずに応答が返ること（200 OK で空でない `Value`、または 204 No Content）を確かめる。**[OPEN-7]** 仮定: 全段階で `OnBoot` の疎通だけを確かめ、段階ごとの目玉イベント（`OnGhostChanged`・`OnMouseDoubleClick` など）の疎通は求めない。
3. The 検証 shall 最終段階の段階辞書と hello-pasta の配布辞書が一致すること（Requirement 2.3）を確かめる。
4. If いずれかの段階の読み込みまたは応答が失敗した, the 検証 shall 失敗した段階の番号とファイルを報告して失敗する。
5. The 検証 shall 時刻や乱数に依存する段階（ランダムトーク・時報）についても「読み込めて起動できる」ことだけを確かめ、決定的な出力の照合は求めない。
6. The 検証 shall 既存の統合テストと同じく、コミット済みの辞書をその場でロードせず、一時ディレクトリへコピーして行う（自己展開の書き込みをリポジトリに残さない）。
7. While `manual.yml` が `cargo test -p pasta_sample_ghost` をチュートリアル構文ガードとして実行している, the 段階辞書の検証 shall 同じコマンドの中で実行される（マニュアルの公開 CI が追加の手順なしに全段階を検証する）。

### Requirement 5: 既存テスト・検査ツールとの整合

**Objective:** As a リポジトリの保守者, I want 辞書の書き直しで既存のテストと CI が壊れないこと, so that 本 spec の完了時点で `cargo test --all` とマニュアル CI が通る。

#### Acceptance Criteria

1. The hello-pasta の辞書 shall `OnBoot` を単一シーンに保ち、その出力が決定的である（`pasta_shiori` のゴールデン応答テスト `byte_invariant_test.rs`・`kick_unused_byte_invariant_test.rs` と `shiori_sample_ghost_test.rs` が依存）。**[OPEN-8]** 仮定: `OnBoot` の台詞と表情（`女の子：＠通常　起動したよ～。` / `男の子：＠通常　さあ、始めようか。`）を変えず、ゴールデンを更新しない。
2. If 教材化のために `OnBoot` 以外の既存テストが固定している形（`talk.pasta` の `OnTalk` が 5〜10 個、`click.pasta` の `OnMouseDoubleClick` が 7 個以上、`時報12`・`時報その他`・`＄時１２` の存在、イベント辞書にグローバルアクター辞書を置かない、`dist_src_validation_test.rs` の必須ファイル）を変える必要がある, the 本 spec shall 作例の形を保つかテスト側を直すかを段階表の確定時に決め、テスト側を直す場合はテストの意図（決定性・構造の保証）を保ったまま更新する。
3. While `pasta_shiori` の e2e テスト（`scene_kick_*_e2e_test.rs`）が hello-pasta を一時ディレクトリへコピーしてシーンを追加している, the hello-pasta の辞書 shall これらが追加するシーン名と衝突せず、`pasta.toml` の読み替え（talk 間隔の上書き）を妨げない。
4. When hello-pasta の辞書が変わる, the `book/src/getting-started/first-ghost.md` の ```` ```pasta ```` ブロック shall 変更後の辞書と逐語一致する状態に保たれ、`node book/tools/tutorial-check.mjs` が exit 0 で終わる。**[OPEN-9]** 仮定: 本 spec は `first-ghost.md` のコードブロックだけを変更後の辞書に逐語で差し替える（文章・章立て・照合方式は変えない）。
5. The 本 spec shall `cargo test --all` と `cargo clippy --all-targets --workspace -- -D warnings`、`node book/tools/tutorial-check.mjs` が完了時点で成功する状態で終わる。

### Requirement 6: 配布物の変化の周知

**Objective:** As a hello-pasta の利用者, I want サンプルゴーストの辞書が大きく変わったことがリリースで分かること, so that 以前の辞書を手本にしていた人が差分に気づける。

#### Acceptance Criteria

1. When hello-pasta の辞書が書き直される, the 本 spec shall 書き直しを取り込むマージコミット（PR タイトル）を、リリースノートの生成（git log を Conventional Commits の種類で分類する `release-workflow` の方式）に拾われる `feat(pasta_sample_ghost): …` の形で、辞書が教材として書き直されたこと・emo2 との切り替えの作例が入ったことが利用者に分かる言葉で記す。
2. When hello-pasta の辞書が書き直される, the `crates/pasta_sample_ghost/README.md` shall 辞書の構成（ファイル一覧と各ファイルの役割）の記述を変更後の辞書に合わせる。
3. The 本 spec shall `.nar` に同梱する文書を増やさない（`install.txt`・`descript.txt` は変えず、`RELEASE.md` はリリース手順書のまま利用者向けの変更履歴を持たせない）。
4. The hello-pasta の配布物 shall 辞書以外のファイル（`install.txt`・`descript.txt`・`pasta.toml`・シェル）を本 spec では変えない。
