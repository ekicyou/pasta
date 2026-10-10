# Requirements Document

## Project Description (Input)
pasta の利用者マニュアル（`book/`）の入門ガイド（`book/src/getting-started/`）を、初心者が「こんな表現をしたい」を 1 つずつ叶えていく物語として全面的に書き直す。章立ては段階表（`crates/pasta_sample_ghost/STAGES.md`・13 段）に 1 章ずつ対応させ、各章の終わりでゴーストが起動してその章の表現を試せるようにする。全編を Claudia が語り、文法リファレンスと Lua 章はリファレンスのまま残してリンクで送り出す。執筆規約（`book/AUTHORING.md`）に入門ガイド限定の例外を設け、本文検査（`verify-content.mjs`）と作例の段階ごとの照合（`tutorial-check.mjs`）とそれらの自己テストを追従させ、目次・表紙の案内・旧ページからの転送・README のリンクを更新する。スクリーンショットは別 spec（`getting-started-screenshots`）が持つ。詳細は `brief.md` を参照。

## Introduction

今の入門ガイドは 3 章（`index.md`・`prerequisites.md`・`first-ghost.md`）で、`first-ghost.md` は完成形の辞書 12 ファイルを順に貼る手順書である。初心者には「何ができるのか」「どう表現したいときに何を書くのか」が頭に入らず、案内役の Claudia も導入と締めに顔を出すだけで物語がない。

本 spec は、入門ガイドを段階表（`crates/pasta_sample_ghost/STAGES.md`）の 13 段に 1 章ずつ対応する物語に書き直す。各章は「こんな表現をしたい」という願いから始まり、章の終わりでゴーストが起動してその表現を試せる。全編を Claudia が語り、詳しい文法は文法リファレンスと Lua 章へリンクで送り出す。あわせて、執筆規約に入門ガイド限定の例外を設け、作例の照合を段階ごとに広げ、章の数に依存する自己テストと目次・案内・転送・リンクを追従させる。

本書の読者は「ゴースト作者（初心者を含む）」であり、執筆規約と検査ツールの利用者は「マニュアルの執筆者（AI エージェントを含む）」である。

## Boundary Context

- **In scope**:
  - `book/src/getting-started/` の全章の文章（入口の章・準備の章・13 段の章）
  - 執筆規約（`book/AUTHORING.md`）への、入門ガイドに限った例外
  - 本文検査（`book/tools/verify-content.mjs`）の追従と、作例の照合（`book/tools/tutorial-check.mjs`）の段階ごとの照合への拡張、それらの自己テスト
  - 章の数を決め打ちしている自己テスト（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）の追従
  - 目次（`SUMMARY.md`）、表紙の案内（`introduction.md`）、旧ページからの転送（`book.toml`）、`first-ghost.html` を指す 2 つの README のリンク
- **Out of scope**:
  - スクリーンショットの撮影と掲載、画像ファイル、章への画像の行、新しいシェルでの 8 段目の実機の確かめ（`getting-started-screenshots`）
  - 段階表の内容（どの表現をどの順で教えるか）と段階辞書の中身（`hello-pasta-tutorial-stages` が確定済み。本 spec はそれに従う）
  - 文法リファレンス・Lua 章・リファレンス・内部設計パートの書き直し（リンク先として使うだけ）
  - 文法・API の新しい説明。ガイドに書く事実は、リファレンス章が権威として書いている事実の範囲に留める
  - シェルのファイル（`shell/master/descript.txt`・`surfaces.txt`）の中身の転記と、シェル画像（`hello-pasta-shell-art`）
  - スキル `references/` の生成対象の変更（`getting-started` は生成対象章ではない）
  - マニュアルの自動検査の定義（`.github/workflows/manual.yml`）と出力の検査（`book/tools/verify-static.mjs`）の変更（同じウェーブの `manual-print-media-refs` が持つ）
- **Adjacent expectations**:
  - 段階辞書は hello-pasta の配布辞書 `dic/01-boot.pasta`〜`12-lua.pasta`（1 段 1 ファイル）そのもので、全段が起動することは `cargo test -p pasta_sample_ghost` が保証する。本 spec はその保証に乗り、起動の検証を再実装しない
  - 台詞部品の記法・話し手・表情の登録簿・導入と締めの検査（`manual-claudia-theme` の成果）は変えずに使う
  - 辞書を変える spec は、同じ変更で入門の章の作例を直す（照合が CI で落ちるため）
  - 8 段目の部位名は段階表の `Head` を使う。シェルの当たり判定の確定は `hello-pasta-shell-art` が持つ

## Requirements

### Requirement 1: 章立てと目次

**Objective:** As a ゴースト作者（初心者）, I want 段階表の 1 段が 1 章になった順路, so that 「こんな表現をしたい」を 1 つずつ叶えながら読み進められる

#### Acceptance Criteria
1. The 入門ガイド shall 入口の章・準備の章・段階表の 13 段に 1 対 1 で対応する 13 章で構成される。
2. The 入門ガイド shall 各段の章を、段階表の `段階` の順に並べる。
3. The 入門ガイド shall 各段の章の題を、段階表の `願い`（例:「しゃべらせたい」）が読者に分かる形にする。
4. The 目次（`SUMMARY.md`） shall 入門パートに入口の章・準備の章・13 段の章をこの順で載せる。
5. The 入口の章 shall ガイドの進め方（1 章 1 段・章の終わりで起動して試す・詳しい文法はリファレンスへ）と、13 段の章の一覧を示す。
6. If 段階表に辞書を持たない段（13 段目）がある, then the 入門ガイド shall その段も 1 章として扱い、辞書の差分が無いことを本文で示す。
7. The 入門ガイド shall 「途中の段階では起動しないことがある」「`scripts/` に Lua ランタイムを置く」「8 段目の書き方はこれまでと変わらない」「シェルの画像を自動生成する仕組み」などの、現状と食い違う記述を残さない。

### Requirement 2: 準備の章

**Objective:** As a ゴースト作者（初心者）, I want 1 段目に入る前に必要な道具と最小一式がそろう手順, so that 1 段目の辞書を 1 ファイル足すだけでゴーストが起動する

#### Acceptance Criteria
1. The 準備の章 shall 動作環境（Windows・SSP）・テキストエディタ・UTF-8 の約束を、今の `prerequisites.md` の技術的内容を落とさずに示す。
2. The 準備の章 shall `pasta.dll` の入手先を案内し、Lua ランタイムが `pasta.dll` の中にあって `scripts/` は利用者が自分のスクリプトを置く場所であることを正しく示す。
3. The 準備の章 shall ゴーストのフォルダ構成と、1 段目より前に置く最小一式（`install.txt`・`ghost/master/descript.txt`・`ghost/master/pasta.toml`・`pasta.dll`・シェル）を示す。
4. The 準備の章 shall シェルについて「hello-pasta のシェルをそのまま使う」と案内して置き場所へ送り、`shell/master/descript.txt`・`surfaces.txt` の中身を本文に書き写さない。
5. The 準備の章 shall `pasta.toml` について、起動に必須なのが `[actor]` だけであることと、各アクターの `spot` の意味を示す。
6. The 準備の章 shall 準備の終わりの状態（辞書が無いので起動してもしゃべらないこと）を読者に伝え、1 段目へ送る。
7. The 準備の章 shall `install.txt`・`ghost/master/descript.txt` の `name` を読者自身が決めた名前にさせ（`craftman`・`craftmanw` も読者のものに置き換えさせ）、`hello-pasta` を使うと配布版と同じ SSP に入れたとき名前が衝突することを伝える。
8. The 準備の章 shall `sakura.name`・`kero.name` が表示名であり、辞書の中で呼ぶアクター名は `pasta.toml` の `[actor]` で決まることを示す。

### Requirement 3: 段の章の共通の型

**Objective:** As a ゴースト作者（初心者）, I want どの段の章も同じ流れで読める, so that 章を進めるごとにゴーストにできることが増えるのを実感できる

#### Acceptance Criteria
1. The 段の章 shall 段階表の `願い` から話を起こし、その段で新しく覚える表現を、段階表の `新しく覚える表現` の範囲で説明する。
2. The 段の章 shall その段で追加する辞書ファイルの中身を、辞書ファイル 1 つにつき 1 つの ```` ```pasta ```` ブロックで、辞書ファイルと逐語で一致する形で示す。
9. The 段の章 shall 辞書の転記以外に ```` ```pasta ```` ブロックを置くときは、その段の辞書ファイルの連続した行の抜き出し（逐語）だけを置き、辞書に無い形の例示を ```` ```pasta ```` ブロックで書かない。
3. The 段の章 shall 辞書ファイルの置き場所とファイル名（`dic/NN-name.pasta`）を示し、前の段のファイルに手を入れずに新しいファイルを足すだけで次の段になることを伝える。
4. The 段の章 shall 章の終わりに、ゴーストを起動（または再読み込み）してその段の表現を確かめる手順と、何が見えれば成功かを示す。
5. When 段階表の `確かめるための道具` がその段に道具（開発者用機能の有効化・開発用パレット・スクリプト入力）を指定している, the 段の章 shall その道具の使い方を手順の中で示す。
6. The 段の章 shall その段で学んだ表現の詳しい文法へ、段階表の `使う文法要素` が指すリンク先（文法リファレンス・Lua 章・`pasta.toml` リファレンス）と同じ先へリンクで送り出す。
7. The 段の章 shall 段階表の `使う文法要素` に無い文法・API を新しく教えない。
8. While 辞書ファイルの中身がその段の説明と重なる, the 段の章 shall 辞書ファイルのコメント（`＃` 行）に書いてある事実と食い違う説明をしない。

### Requirement 4: 段ごとの固有の内容

**Objective:** As a ゴースト作者（初心者）, I want 段階表と上流の申し送りが定めた注意点をその段で教わる, so that よくある失敗（二重の挨拶・数分待つ・部位名が空）で止まらない

#### Acceptance Criteria
1. The 2 段目の章 shall ランダムトークの間隔を `pasta.toml` の `[ghost]`（`talk_interval_min`・`talk_interval_max`）で短くする 2 行を書き足させ、既定のままだと数分待つことを伝える。
2. The 6 段目の章 shall SSP の本体設定で開発者用機能を有効にする手順と、開発用パレットの「現在時刻の仮想的変更」で正時を待たずに時報を確かめる手順を示す。
3. The 7 段目の章 shall イベントとシーンの対応の仕組みを初めて説明する章として、(a) シーン名をイベント名にすると呼ばれること、(b) マニュアルの一覧に無いイベントも UKADOC で探して同名シーンで応答できること、(c) `＞transfer_req_to_var` で付加情報を `＄ｒ０`〜`＄ｒ９` に取り出して台詞に使えること、(d) `＄％baseware.name` のようなプロパティ変数でベースウェアに聞けること、を順を追って語る。
4. The 7 段目の章 shall `OnGhostChanged`・`OnGhostChanging` に応答すると `OnBoot`・`OnClose` が来ないこと（何も話さずに終えて 204 を返したときだけ回ること）を示し、起動の挨拶を二重に書く失敗を防ぐ。
5. The 7 段目の章 shall 切り替えの相手役を emo2 とし、読者が試す手順を SSP のメニューからの切り替えで書く（ゴースト自身の `\![change,ghost]` では `OnGhostChanging` が届かないため、その手順で試させない）。
6. The 7 段目の章 shall 切り替えの相手が手元に無くても試せるよう、開発用パレットの「スクリプト入力」で `\![raise,イベント名,…]` を送って確かめる手順を示す。
7. The 7 段目の章 shall 読者は 1 段目で初回起動を済ませているため `OnFirstBoot` が自然には呼ばれないことを伝え、`profile/` を消して初期状態に戻す方法は案内しない。
8. The 8 段目の章 shall 触られた部位の名前が `＄ｒ４` に入ること、部位名は段階表のとおり `Head` であること、当たり判定の外では空になることを示す。
9. The 9 段目の章 shall 選択肢行と `!select(秒)`、選ばれた先のシーンへの自動ルーティングを、`OnChoiceSelectEx` との対応とともに示す。
10. The 10 段目の章 shall 未代入のグローバル変数が算術で 0 とみなされることを示し、`＄＊回数` が終了しても残ることを確かめる手順を示す。
11. The 12 段目の章 shall シーン内の Lua ブロックと `＞＠名前（）` の呼び出しを示し、より込み入ったことは Lua 章へ送り出す。
12. The 13 段目の章 shall SSP の NAR 作成機能で `.nar` にする手順を段階表の「13 段目：配布したい」のとおり示し、手順の正本として UKADOC の SSP ヘルプへリンクする。

### Requirement 5: SHIORI イベントの出典

**Objective:** As a ゴースト作者, I want シーンがどのイベントで呼ばれるかと、そのイベントの正式な説明の在りか, so that マニュアルに無いイベントも自分で調べられる

#### Acceptance Criteria
1. When 段の章が段階表の `初めて扱うイベント` に載るイベント（`OnBoot`・`OnGhostChanged`・`OnGhostChanging`・`OnFirstBoot`・`OnClose`・`OnMouseDoubleClick`・`OnChoiceSelectEx`）を初めて扱う, the 段の章 shall そのイベントの UKADOC の項へリンクし、イベントの説明と `Reference` の意味を出典付きで短く引用する。
2. The 入門ガイド shall UKADOC からの引用を短い引用に留め、`Reference` の一覧などを丸ごと転載しない（全体は UKADOC へのリンクで見せる）。
3. The 入門ガイド shall SHIORI イベントの一覧と仕組みの詳しい説明は Lua 章 `lua/shiori-events.md` へ、リクエスト変数の詳しい文法は `grammar/variables.md` の「リクエスト変数（Reference）」へ送り出す。

### Requirement 6: emo2 の扱い

**Objective:** As a ゴースト作者（初心者）, I want 作りこんだゴーストの実例を見る, so that 「この表現を作りこむとこうなる」が想像できる

#### Acceptance Criteria
1. The 入門ガイド shall emo2（<https://ekicyou.github.io/ghost_dev/emo2/>）を pasta.dll のショーケースゴーストとして紹介し、7 段目の章では切り替えの相手役として使う。
2. Where 段の章が emo2 を「この表現を作りこむとこうなる」実例として紹介する, the 段の章 shall 本文の作例を hello-pasta の辞書で書き、emo2 の辞書を作例の照合対象にしない。
3. The 入門ガイド shall emo2 について書く事実（名前 `むらさき` が Reference0 に入ること、`OnGhostChanging`・`OnGhostChanged` に応答してくること など）を、`hello-pasta-tutorial-stages` の「emo2 からの知見」に記録された範囲に留める。

### Requirement 7: Claudia の語り

**Objective:** As a ゴースト作者（初心者）, I want Claudia が全編を語って引っ張ってくれる, so that 文法書を読むのでなく物語として最後まで読み通せる

#### Acceptance Criteria
1. The 入門ガイド shall 全章で、説明本体の語りをすべて台詞部品（`> 【表情】…`）で書き、Claudia が前面に出て読者を次の手順へ導く。説明の地の文の段落は置かない。
2. The 入門ガイド shall 各章の導入と締めを、既存の規約どおり台詞部品による Claudia とアンソニーの掛け合いで書く（両方の話し手を置く）。
3. While 説明本体を台詞部品で語る, the 入門ガイド shall 技術的な事実（構文・手順・値・ファイル名）を、台詞を読み飛ばしても台詞以外の部分だけで追える形で示す。台詞以外に置けるのは、操作を指す普通文体の指示の一文（例:「`dic/02-talk.pasta` を作り、次の内容を貼る。」）・箇条書き・表・コードブロック・見出しに限る。
4. The 入門ガイド shall コードブロック内・構文定義・コマンド例・サンプルコード・表のセルにキャラ口調を持ち込まない。
5. The 入門ガイド shall 語りのせいで誤読の余地が生まれる書き方をせず、技術的正確さを語りより優先する。
6. The 入門ガイド shall 各章の最初の台詞を、その章に固有の言葉で始める（ありふれた句で始めない）。
7. The 入門ガイド shall 台詞部品の記法（話し手・表情・空行の規則）を守り、台詞部品の検査（`T-syntax`・`T-intro`・`T-outro`）に通る。
8. The 入門ガイド shall アンソニーを、読者の疑問を代わりに尋ねる役・手綱を引く役として本体にも登場させてよいが、技術的な事実の説明をアンソニーの台詞にだけ置かない。

### Requirement 8: 執筆規約の入門ガイド向けの例外

**Objective:** As a マニュアルの執筆者, I want 入門ガイドだけに許される書き方が規約に書いてある, so that ガイドの章を足したり直したりするときに、他の章の規則と混同しない

#### Acceptance Criteria
1. The 執筆規約（`book/AUTHORING.md`） shall 入門ガイド（`book/src/getting-started/`）に限った例外を、台詞部品の節（第 7 節）とは別の節に書く。
2. The 執筆規約 shall 例外の内容として、説明本体の語りを台詞部品だけで書き説明の地の文の段落を置かないこと、台詞以外に置けるもの（指示の一文・箇条書き・表・コードブロック・見出し）、各章の型（願い → 表現 → 辞書ファイル → 試す → 送り出し）を示す。
3. The 執筆規約 shall キャラ口調を持ち込まない場所（コードブロック内・構文定義・コマンド例・サンプルコード・表のセル）と、技術的正確さを最優先する鉄則を、入門ガイドでも維持することを明記する。
4. The 執筆規約 shall 文体の使い分け表（第 2 節）と執筆チェックリスト（第 5 節）から、入門ガイドの例外の節へ案内する。
5. The 執筆規約 shall 作例の ```` ```pasta ```` ブロックが段階辞書と逐語で一致しなければならないこと、断片を見せる ```` ```pasta ```` ブロックも辞書の連続した抜き出しに限ること、辞書を変えるときは同じ変更で章の作例を直すことを、入門ガイドの執筆者向けに示す。
6. The 執筆規約 shall 生成対象章・内部設計章の規則（本文に台詞と口調を置かない）を変えない。
7. The 執筆規約 shall 辞書以外の転記（`install.txt`・`descript.txt`・`pasta.toml` の断片）は照合されないため、項目名と既定値をリファレンス章（`reference/`）に合わせて執筆者が確かめることを、入門ガイドの執筆者向けに示す。

### Requirement 9: 作例の段階ごとの照合

**Objective:** As a マニュアルの執筆者, I want 各段の章の作例がその段の辞書ファイルと逐語で一致することを CI が確かめる, so that 辞書か章のどちらかが変わっても食い違いが公開前に止まる

#### Acceptance Criteria
1. The 作例の照合 shall 段階辞書の各ファイル（`dic/NN-name.pasta`）について、対応する段の章の ```` ```pasta ```` ブロックと逐語で一致するかを照合する。
2. The 作例の照合 shall 段と章の対応を、入門ガイドの 1 か所に作例がどこかにあればよいとする今の方式でなく、辞書ファイルの段階番号 `NN` とその段の章の対応で判定する。
3. If 段の章にその段の辞書ファイルと一致するブロックが無い, then the 作例の照合 shall 章と辞書ファイルを名指しして失敗（終了コード 1）し、対処（章の作例を辞書の現内容に合わせる）を示す。
4. If 段階辞書にあるファイルに対応する段の章が無い, then the 作例の照合 shall その食い違いを報告して失敗する。
5. While 段が辞書ファイルを持たない（13 段目）, the 作例の照合 shall その段の章に ```` ```pasta ```` ブロックが無いことを失敗にしない。
6. The 作例の照合 shall 改行コード（CRLF/LF/CR）と末尾の空白行だけを正規化して比較し、内容（全角空白・コメント・台詞）を変えない。
7. The 作例の照合 shall 段階辞書のファイルを足したときに固定の一覧を直さなくても照合対象に入る、今の性質を保つ。
8. The 作例の照合の自己テスト（`tutorial-check-test.mjs`） shall 段ごとの照合の成功・章の作例の不一致・章の欠落・辞書の欠落・抜き出しブロックの一致と不一致の各場合を確かめる。
9. The 作例の照合 shall 辞書ファイル以外（`install.txt`・`descript.txt`・`pasta.toml`・シェルのファイル）の転記を照合対象にしない。
10. The 作例の照合 shall 段の章にある ```` ```pasta ```` ブロックのそれぞれについて、その段の辞書ファイル全体と一致するか、辞書ファイルの連続した行の抜き出しと一致するか、のどちらかであることを確かめる。
11. If 段の章の ```` ```pasta ```` ブロックが、その段の辞書ファイル全体とも連続した抜き出しとも一致しない, then the 作例の照合 shall 章とブロック（先頭行）を名指しして失敗する。

### Requirement 10: 本文検査と自己テストの追従

**Objective:** As a マニュアルの執筆者, I want 章を 3 枚から 16 枚に増やしても既存の検査がすべて通る, so that 公開の自動検査が止まらない

#### Acceptance Criteria
1. The 本文検査（`verify-content.mjs`） shall `first-ghost.md` を名指ししている検査を、新しい章立てに合わせて直す。
2. The 本文検査 shall 入門ガイドの説明本体に Claudia の語り（台詞部品）があることを失敗にしない。
3. The 本文検査 shall 入門ガイドの各章についても、台詞部品の記法・導入と締めの検査（`T-syntax`・`T-intro`・`T-outro`）・コードフェンス内に口調が無いことの検査を、従来どおり行う。
4. When 章の数が変わる, the 章の数を決め打ちしている自己テスト（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`） shall 新しい章の数で通る。
5. The マニュアルの検査一式（`verify-content`・`gen-skill-refs --check`・`link-check`・`tutorial-check`・`verify-static`・`verify-search`・`book/tools` の自己テスト全件） shall 本 spec の変更後にすべて成功する。
6. The 入門ガイド shall 各章の最初の台詞の語でその章が検索できるという検索の検査（`verify-search`）を、章を足した後も満たす。

### Requirement 11: 表紙の案内・転送・リンク

**Objective:** As a ゴースト作者, I want 古いリンクや表紙から迷わず新しいガイドへ入れる, so that ブックマークや README からの導線が切れない

#### Acceptance Criteria
1. The 表紙（`introduction.md`）の「このマニュアルの歩き方」 shall 入門ガイドが物語で導き、文法・Lua 章がリファレンスであるという位置づけで案内する。
2. The 表紙の扉（`claudia-hero`）のパート案内 shall 入門ガイドの入口の章を指し続ける。
3. When 読者が廃止するページ（`getting-started/first-ghost.html`）の URL を開く, the マニュアルサイト shall 新しいガイドの該当ページへ転送する。
4. The `crates/pasta_lua/README.md`・`crates/pasta_shiori/README.md` shall `first-ghost.html#ゴーストのフォルダ構成` を指すリンクを、新しいガイドでフォルダ構成を説明する章の実在する見出しへ向け直す（README のリンク検証は転送では通らず、ページと見出しの実在を見る）。
5. The 入門ガイド shall 既存のリンク検証（`link-check.mjs`）に通る相対リンクだけを使う。

### Requirement 12: 技術的正確さと権威

**Objective:** As a ゴースト作者, I want ガイドに書いてあることがそのまま動く, so that リファレンス章と読み比べて混乱しない

#### Acceptance Criteria
1. The 入門ガイド shall 現行の版で実装されている文法・API だけを書き、回避レシピ（書けない形の代わりの書き方）を載せない。
2. The 入門ガイド shall 文法・公開 Lua API・`pasta.toml` について、リファレンス章（`grammar/`・`lua/`・`reference/`）が権威として書いている事実と食い違わない。
3. The 入門ガイド shall ベースウェアの規則（`OnGhostChanged`・`OnGhostChanging` と `OnBoot`・`OnClose` の関係、NAR 作成の手順、開発用パレットの操作）の根拠として UKADOC へリンクする。
4. The 入門ガイド shall 画像を置かず、スクリーンショットに依存しない文章で各章を完結させる（画像は `getting-started-screenshots` が後から足す）。
5. The 入門ガイド shall 一部の章だけを物語化して残りを旧形式のまま出す部分出荷をせず、全章を新しい型でそろえる。

## 前提と決定の記録

brief.md で決めきれなかった点は、2026-10-10 の要件ディスカッションで「閉じた項目」のとおり決めた。残りは設計フェーズで決める。

### 設計フェーズ（`/kiro-design`）で決めること

研究書（`research.md`）の「Recommendations for Design Phase」に引き継ぐ。

- **準備の章の数**（Requirement 1.1・2）: 前提は「前提環境」と「最小一式の配置」で 2 章（入口 1＋準備 2＋段 13＝16 章）。1 章にまとめるかは設計で決める。
- **廃止ページの転送先と章のファイル名**（Requirement 11.3）: 前提は「`first-ghost.html` → 入門ガイドの入口の章」。`prerequisites.md` はファイル名を変えずに残す前提。
- **段と章の対応の決め方**（Requirement 9.2）: 章のファイル名で段階番号を持たせるか、章の中の印で持たせるかは設計で決める。
- **章の数を決め打ちしている自己テストの直し方**（Requirement 10.4）: 3 か所の数字を直すか、目次から数えるかは設計で決める。

### 閉じた項目

- **辞書の転記でない ```` ```pasta ```` ブロック**（Requirement 3.9・9.10・9.11・8.5、議題 5）: 許すが、その段の辞書ファイルの連続した抜き出しに限り、照合がそれを確かめると決定（2026-10-10）。辞書に無い形の例示は `pasta` ブロックで書けない。

- **辞書ファイル以外の転記の照合**（Requirement 9.9・8.7、議題 4）: 照合しないと決定（2026-10-10）。読者は自分の名前を付け（議題 3）、`pasta.toml` は段階的に書くため配布版と一致しない。項目名と既定値の鮮度はリファレンス章との整合（12.2）と執筆規約（8.7）で守る。

- **読者のゴーストの名前**（Requirement 2.7・2.8、議題 3）: 読者自身に名前を付けさせると決定（2026-10-10）。`hello-pasta` は配布版と衝突するため使わせない。`descript.txt` は照合対象外（9.9）なので配布版と違ってよい。

- **表のセルの口調**（Requirement 7.4、議題 2）: 従来どおり持ち込まないと決定（2026-10-10）。口調が宿るのは台詞部品だけで、表のセルは普通文体。

- **「全編を Claudia が語る」の形**（Requirement 7.1・7.3・8.2、議題 1）: 台詞部品だけで語る形に決定（2026-10-10）。説明の地の文の段落は置かず、台詞以外は指示の一文・箇条書き・表・コードブロック・見出しだけ。地の文を令嬢口調で書く形と、普通文体の地の文に台詞を差し込む混合形は採らない。導入・締めの「両方の話し手が要る」規則は変えない。

- **8 段目の当たり判定の過渡期**（Requirement 4.8）: 「当たり判定の外では空になる」は今のシェルでも新しいシェルでも正しいので、4.8 のままで過渡期の状態を別に書かない。
