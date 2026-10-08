# Requirements Document

## Introduction

入門ガイドを「こんな表現をしたい」を 1 段ずつ叶えていく物語に作り直すため、題材の hello-pasta を**段階ごとに起動できる辞書（段階辞書）**として育て直す。本 spec は、(1) どの表現をどの順で教えるかの**段階表**を確定し、(2) 段階ごとの辞書一式をリポジトリに置いて**どの段階もそのまま読み込めて起動できる**ことを CI で確かめ、(3) 配布物 hello-pasta の辞書（`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/`）を**最終段階と一致する教材**として書き直す。既存のテストが依存する挙動（`OnBoot` の決定性など）は、テスト側を直すか作例の形を保つかして壊さない。

ガイド本文の執筆（Claudia の語り・章の文章）は下流の `getting-started-story-guide` が持つ。本 spec は段階表と段階辞書を、下流が逐語で使える形で確定して申し送る。

> 要件ディスカッション（2026-10-08、議題 1〜11）で、ディスカバリだけでは確定できなかった論点 [OPEN-1]〜[OPEN-11] はすべて解消した。本文中の「[OPEN-n] 解消」「議題 n で決定」は、その決定の出どころを示す。決定の背景は `research.md` の §2.2・§5 を参照。

## Boundary Context

- **In scope**: 段階表の確定、段階辞書一式（全段階）、hello-pasta の辞書の書き直し（教材化・不足する表現の追加）、全段階を実際に読み込む検証、既存テスト・検査ツールの更新（辞書の変更に追従する範囲）、配布物の変化を利用者に知らせる記述
- **Out of scope**: マニュアル本文の執筆と章立て（`getting-started-story-guide`）、`tutorial-check.mjs` の照合方式の変更（段階ごとの照合への拡張。同 spec。本 spec は照合対象のファイル名を実ファイルから導く追従だけを行う）、シェル画像（`hello-pasta-shell-art`）、DSL・ランタイムの機能追加（段階表は現行実装で書ける表現だけを使う）、emo2 側の辞書の変更（ghost_dev リポジトリ）
- **Adjacent expectations**:
  - `getting-started-story-guide` は、本 spec が確定した段階表の順に 1 章ずつ対応させ、段階辞書を逐語で引用する。段階辞書の置き場所と形は本 spec が決めて申し送る。
  - `release-workflow` / `release-ci` は hello-pasta の `.nar` を作る。辞書の中身が変わるため、利用者に変化が分かる記述を本 spec が残す。
  - `hello-pasta-shell-art` は同じクレートの画像側を触る。本 spec はアクター辞書の表情名と `\s[n]` の対応（surface 番号）を変えない。
  - emo2（ghost_dev）は切り替えの相手役。emo2 側の `OnGhostChanging` / `OnGhostChanged` の反応はすでに入っており、hello-pasta 専用の台詞を足すかはユーザーの判断（本 spec の範囲外）。
  - **上流（実装のゲート）**: spec `scene-name-alias`（別セッション「OnTalk シーン名の扱い」。brief 起票済み・実装未着手）が、`pasta.toml` のシーン名別名表（既定「会話 → OnTalk」1 件、**完全一致のみ**、登録・Call/Jump・選択肢・Lua 呼び出しのすべてに効く、既存 `＊OnTalk` は後方互換、マニュアルも同 PR で更新）を入れる。本 spec の段階表はこの別名を前提に、読者が書くランダムトークのシーン名を `＊会話` とする。**同様に、算術で未代入の変数を 0 とみなす上流 spec `arith-unassigned-var-zero`（Requirement 3.6。設計ディスカッション #1 で起票を決定）もゲートに加わる。** **設計（`/kiro-design`）はこれらの PR を待たずに進め、実装（`/kiro-impl`）の着手は `scene-name-alias` が main に取り込まれていることをゲートとする**（議題 6。ユーザーは `scene-name-alias` の PR を最優先で出させる方針）。`＊会話朝` のような派生名は別名の対象外（別シーン）なので、候補を増やすときは `＊会話` を繰り返すか単独 `＊` を使う。

## Requirements

### Requirement 1: 段階表の確定

**Objective:** As a 入門ガイドの執筆者（`getting-started-story-guide`）, I want 「こんな表現をしたい」を 1 段ずつ広げる段階表が確定していること, so that 物語の章立てと作例を食い違いなく書ける。

#### Acceptance Criteria

1. The 段階表 shall リポジトリ内の 1 か所に置かれ、段階ごとに「叶えたい表現（願い）」「新しく覚える表現」「使う文法要素」「その段階で初めて扱うベースウェアのイベント（あれば）」「その段階で追加する辞書ファイル名」「検証で送るイベントと Reference（実イベントを初めて扱う段階のみ。Requirement 4.2a）」を持つ。
2. The 段階表 shall 次の順を採る: 1. しゃべらせたい（`OnBoot` の一言） → 2. 二人で掛け合いさせたい（暇なときのおしゃべり `＊会話`（＝ランダムトーク）に 2 人のコントを書く。brief の 5 段目「暇なときに話しかけてほしい」をここに吸収） → 3. 表情を変えたい → 4. 毎回ちがうことを言わせたい（同名シーンを増やす・単独 `＊`） → 5. 単語でちょこっと変えたい（`＠単語：a、b、c`。brief では 4 段目に同居していたものを独立） → 6. 時報 → 7. 挨拶（ベースウェアのイベント） → 8. 触ったら反応 → 9. 選択肢 → 10. 覚えていてほしい（変数の保存） → 11. 話を続ける・分岐 → 12. Lua への入り口 → 13. 配布。brief のたたき台からの変更理由（議題 5 で `OnBoot` を育てない方針にしたため、2〜4 段目の掛け合いの受け皿としてランダムトークを 2 段目へ繰り上げた）を段階表に記す。2 段目で `＊会話` を導入するときは、「暇なときに pasta が呼ぶシーンの名前」とだけ説明し、イベント名とシーン名の対応の本格的な説明は 7 段目に置く。11 段目「話を続けたい・分岐させたい」で教える「分岐」は、条件（IF）による分岐ではなく **ランダムジャンプ**である: `＞シーン名` で別のシーンへ話を続け、呼び先の名前に同名のシーンが複数あればどれか 1 つがランダムに選ばれ、呼び先の名前は**前方一致**で候補が集められる（`＞挨拶` が `＊挨拶朝`・`＊挨拶夜` の両方を候補にする）。ローカルシーン（`・`）とチェイントークもこの段で扱い、ローカルシーンへのジャンプとグローバルシーンへのジャンプの違い（自分のグローバルシーンの配下のローカルシーンが先に探され、ローカルの候補が 1 つでもあればグローバルシーンは候補にならない。他のグローバルシーンの配下のローカルシーンは候補にならない。マニュアル `call-jump.md` のスコープ解決）を軽く説明する。条件による分岐は Lua による拡張であり、本 spec の作例では Lua を「少し紹介する程度」に留める（12 段目）。ランダムトーク（OnTalk）を説明する段では、同名シーンの候補を増やす書き方として**単独の `＊` 行**（同じファイルで直前に宣言したグローバルシーンと同じ名前の別シーンを開始する。候補が 1 つ増えるだけで前のシーンへは統合されない。ファイル先頭の単独 `＊` はパースエラー。マニュアル `block-structure.md`）の例をさらっと示す。
3. The 段階表 shall 各段階で使う文法要素と API を、現行の版のマニュアル（`book/src/`）に書かれているものだけから選ぶ。マニュアルに無い書き方、「書けるだけ」の書き方、意図の推測で救済される書き方は作例に使わない。
4. The 段階表 shall 「挨拶したい」の段階で、シーン名＝イベント名で呼ばれること、イベントの付加情報を `＞transfer_req_to_var` で `＄ｒ０`〜`＄ｒ９` に取り出す書き方、`OnGhostChanged` に応答すると `OnBoot` は来ないこと（204 を返したときだけ `OnBoot` へ回る。送り出しの `OnGhostChanging` も同様に、応答すれば `OnClose` は来ない）、pasta のマニュアルのイベント一覧に無いベースウェアのイベント（`OnGhostChanging` など）も同名のシーンで応答できることを、初めて扱う内容として位置づける。`OnGhostChanging` の根拠は UKADOC（<https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html#OnGhostChanging>）へのリンクで示す。同じ段で「ベースウェアに聞く」書き方としてプロパティの読み取り（`＄％baseware.name`。Requirement 3.8）も新しく覚える表現に含める。
5. The 段階表 shall 「触ったら反応してほしい」の段階で、前段で覚えた `＞transfer_req_to_var` を使って `OnMouseDoubleClick` の部位（`＄ｒ４`）を読む形を採る。
6. The 段階表 shall 辞書を持つすべての段階を「新しい `.pasta` ファイルを `dic/` に足すだけ」で成立させ、前の段階で書いたファイルを書き換える段階を置かない。とくに 1 段目で書く `OnBoot` のシーンは以後の段階で変えず、掛け合い（2 段目）・表情（3 段目）は `OnBoot` ではなく新しいファイルの別のシーンで教える（議題 5 で決定。pasta は `dic/*.pasta` を全部読み込むため、章 N の辞書は章 1〜N のファイルの集まりとして定義できる）。
7. The 段階表 shall 「配布したい」の段階（13 段目）を、辞書の差分を持たない段階（手順のみ）として扱う。辞書を持つ段階は 1〜12 段目で、12 段目の辞書（＝全ファイル）が hello-pasta の配布辞書そのものである（13 段目で `.nar` にする中身は 12 段目と同じ）。
8. The 段階表 shall 「配布したい」の段階で案内する `.nar` の作り方を、SSP の nar 作成機能（本体設定「一般」で開発者用機能を有効にし、「開発/その他」の「ディレクトリをドロップした際に更新ファイルや NAR を作成」を ON にして、ゴーストのフォルダを SSP にドロップする）とし、手順の正本として UKADOC の SSP ヘルプ（開発者向けヘルプ <https://ssp.shillest.net/ukadoc/ssphelp/dev.html>・設定：開発/その他 <https://ssp.shillest.net/ukadoc/ssphelp/config-dev.html>）へのリンクを持つ。内製ツール（`pasta_check release`・`release.ps1`）は読者に案内しない。

### Requirement 2: 段階辞書一式

**Objective:** As a 入門ガイドの読者, I want 各章の終わりに自分のゴーストが起動して、その章で覚えた表現を試せること, so that 「書いたら動いた」を毎章で体験できる。

#### Acceptance Criteria

1. The 段階辞書 shall hello-pasta の配布辞書（`ghosts/hello-pasta/ghost/master/dic/*.pasta`）そのものを正本とし、各 `.pasta` ファイルが「どの段階で追加されるか」を段階表（Requirement 1）が定める。段階 N の辞書は、段階 1〜N で追加されるファイルの集まりとして定義される（別置きの段階ディレクトリや複製は持たない）。
2. The 段階辞書 shall 段階ごとに新しいファイルを足すだけで育つ。読者は前の段階のファイルに手を入れず、新しいファイルを `dic/` に加えるだけで次の段階になる（Requirement 1.6）。
3. The 段階辞書の各ファイル名 shall 読み込み順と段階の順が一目で分かるよう、段階番号を接頭にしたテーマ別の名前（例: `01-boot.pasta`）を持つ。番号の桁数と区切りの規則は設計で決める。
4. The 段階辞書 shall 各段階で、アクター辞書の表情名と surface 番号の対応（`女の子`: `\s[0]`〜`\s[8]`、`男の子`: `\s[10]`〜`\s[18]`）を hello-pasta と同じに保つ。
5. The 段階辞書 shall 新しく付けるシーン名を、既存のシーン名で始まる名前（前方一致で既存の呼び出しの候補に混ざる名前）にしない。
6. While 段階辞書が 1 段階ずつ育つ, the 各段階の辞書 shall hello-pasta の `pasta.toml`・`descript.txt`・`install.txt`・シェルをそのまま使って起動できる（段階ごとに設定ファイルの差し替えを要しない）。段階ごとに変わるのは `dic/` の `.pasta` ファイルの集まりだけで、設定・シェル・`scripts/` は全段階で共通である。検証（Requirement 4）は hello-pasta の設定・シェルと段階 N までのファイルを一時ディレクトリで合成して起動する。
7. The 段階辞書の形（ファイル名の規則・段階とファイルの対応・段階 N の辞書の組み立て方） shall 下流の `getting-started-story-guide` が逐語で参照できるよう、クレート内の説明ファイル（段階表と同じ場所）に記される。

### Requirement 3: hello-pasta 辞書の教材化

**Objective:** As a 入門ガイドの読者, I want hello-pasta の辞書そのものが、読んで学べる作例とコメントになっていること, so that 配布物を開いたときに教材の最終形として読める。

#### Acceptance Criteria

1. The hello-pasta の辞書 shall テスト向けの説明（「テスト安定性のため単一シーン」「プロパティスコープ統合テスト」「7種以上」など）を含まず、コメントは読者に向けた「何を表現しているか・どう書くか」の説明になっている。
2. The hello-pasta の辞書 shall 既存の表現（起動・初回起動・終了のあいさつ、ランダムトーク、時報、クリックへの反応、選択肢、単語のランダム選択、アクター辞書の表情）を引き続き含む。
3. The hello-pasta の辞書 shall 次の表現を新たに含む: ベースウェアからの切り替えイベントへの挨拶（`OnGhostChanged`。相手役は emo2）、イベントの付加情報の取り出し（`＞transfer_req_to_var`・`＄ｒ０`・`＄ｒ４`）、変数の保存（`＄＊名前`）、会話の続きと分岐（`＞シーン名` の Call、同名・前方一致の候補からのランダム選択による分岐、ローカルシーンまたはチェイントーク）、Lua との連携の入り口（紹介程度の最小の作例。条件分岐の作り込みは求めない）。
4. The 切り替えの作例 shall emo2 からの切り替え（`OnGhostChanged`）で `＄ｒ０`（直前のゴーストの本体側の名前。emo2 では `むらさき`）を台詞に使い、`＄ｒ０` の直後に空白を置く。作例は `OnGhostChanged`（emo2 から迎え入れ）と `OnGhostChanging`（emo2 へ送り出し）の両方を含み、送り出しの台詞は `＄ｒ０`（切り替え先の本体側の名前。emo2 では `むらさき`）に向けて一言で引き継ぐ形にする（議題 7 で決定）。
5. The 切り替えの作例の台詞 shall emo2 の制約に従う: むらさき・エモを冷たく・いじわるに描かない、「おばあちゃん」に触れない、ユーザーは「ユーザーさん」と呼ぶ。
6. The 変数の保存の作例 shall `＄＊名前` に入れた値がゴーストを終了しても残ることを読者が確かめられる形にする: 10 段目のファイルに `＊会話` を 1 つ足し、その中で `＄＊回数` を 1 増やして「この話をするのは `＄＊回数` 回目」と自分の回数を言う（その候補が選ばれた回数そのものを数えるので意味が正しく、再起動しても数が巻き戻らないことで保存を確かめられる。議題 9 で決定）。全発生を数える値（触られた回数・起動回数）は、同名候補のランダム選択と `OnBoot` 不変の制約で正しく数えられないため採らない。作例は `＄＊回数＝＄＊回数＋１` の 1 行で 1 増やす形とし、初期値の代入行や Lua を書かない。未代入の変数に `＋１` すると現行では値なしになるため、算術の被演算子の未代入変数を 0 とみなす上流 spec `arith-unassigned-var-zero`を本 spec の実装着手のゲートに加える（設計ディスカッション #1 で決定。この程度で Lua が要るなら DSL の問題として扱う）。
7. The Lua との連携の作例 shall 「少し紹介する程度」の最小の形に留め、シーン内の Lua ブロックに小さな関数を 1 つ書いて `＞＠関数（）` で呼ぶ、マニュアルの記述パターンに従う。`scripts/` にファイルを置かず、12 段目の辞書も `dic/` だけで完結する。
8. The hello-pasta の辞書 shall プロパティの読み取り（`＄％プロパティ名`）の作例を、ゴースト自身が知りえない情報を読む形で 1 つ持つ: 7 段目「挨拶したい」のファイルに `＊会話` を 1 つ足し、`＄％baseware.name`（必要なら `＄％baseware.version` も）を台詞に使う（例: 「わたしたち、`＄％baseware.name` の上で動いてるんだね」）。現行の `＄％currentghost.name` で自分のゴースト名を名乗る作例は削る（ゴーストが自分の名前を知らないのは不自然。議題 8 で決定）。根拠は UKADOC プロパティシステム <https://ssp.shillest.net/ukadoc/manual/list_propertysystem.html#baseware.name> とマニュアル `variables.md` のプロパティ変数。
9. The hello-pasta の辞書 shall 現行の版の文法・API だけで書かれ、`cargo test -p pasta_sample_ghost` の実ローダーによる読み込みを通る。

### Requirement 4: 全段階の検証

**Objective:** As a リポジトリの保守者, I want どの段階の辞書も実際に読み込めて起動できることを CI が確かめること, so that 辞書やエンジンが変わっても教材の作例が壊れたまま公開されない。

#### Acceptance Criteria

1. When `cargo test -p pasta_sample_ghost` が実行される, the 検証 shall 段階表の辞書を持つ全段階（N = 1〜12）について、段階 1〜N のファイルだけを `dic/` に置いた辞書を実ローダーで読み込み（パース・トランスパイル・Lua 起動）、成功することを確かめる。
2. When 段階辞書の読み込みが成功した, the 検証 shall その段階の辞書に対して `OnBoot` のリクエストを送り、エラーにならずに応答が返ること（200 OK で空でない `Value`、または 204 No Content）を確かめる。
2a. When 段階表がその段階に「検証で送るイベント」を定めている, the 検証 shall `OnBoot` に加えてそのイベントを段階表の Reference 付きで 1 回送り、エラーにならずに応答が返ること（200 OK で空でない `Value`、または 204 No Content）を確かめる。応答の内容は照合しない（同名候補のランダム選択があるため）。対象はベースウェアから来る実イベントに限り（7 段目 `OnGhostChanged`（Reference0 ＝ `むらさき`）・`OnGhostChanging`、8 段目 `OnMouseDoubleClick`（Reference4 ＝ 部位名）、9 段目 選択肢の `OnChoiceSelectEx` など）、仮想イベント（ランダムトーク・時報）は対象外とする（議題 10 で決定。[OPEN-7] 解消）。
3. The 検証 shall 段階表が列挙するファイルの集まりと `dic/` に実在する `.pasta` ファイルの集まりが一致すること（段階表に無いファイル・段階表にあって存在しないファイルがどちらも無いこと）を確かめる。
4. If いずれかの段階の読み込みまたは応答が失敗した, the 検証 shall 失敗した段階の番号とその段階で追加されたファイル名を報告して失敗する。
5. The 検証 shall 時刻や乱数に依存する段階（ランダムトーク・時報）についても「読み込めて起動できる」ことだけを確かめ、決定的な出力の照合は求めない。
6. The 検証 shall 既存の統合テストと同じく、コミット済みの辞書をその場でロードせず、一時ディレクトリへコピーして行う（自己展開の書き込みをリポジトリに残さない）。
7. While `manual.yml` が `cargo test -p pasta_sample_ghost` をチュートリアル構文ガードとして実行している, the 段階辞書の検証 shall 同じコマンドの中で実行される（マニュアルの公開 CI が追加の手順なしに全段階を検証する）。

### Requirement 5: 既存テスト・検査ツールとの整合

**Objective:** As a リポジトリの保守者, I want 辞書の書き直しで既存のテストと CI が壊れないこと, so that 本 spec の完了時点で `cargo test --all` とマニュアル CI が通る。

#### Acceptance Criteria

1. The hello-pasta の辞書 shall `OnBoot` を単一シーンに保ち、その出力が決定的である（`pasta_shiori` のゴールデン応答テスト `byte_invariant_test.rs`・`kick_unused_byte_invariant_test.rs` と `shiori_sample_ghost_test.rs` が依存）。`OnBoot` は 1 段目「しゃべらせたい」で書いた形（女の子の一言）のまま配布辞書に残るため、`OnBoot` の台詞は現行（`起動したよ～。` / `さあ、始めようか。`）から変わる。本 spec はこれら 3 テストのゴールデン文字列と assert 条件を新しい `OnBoot` の決定的な出力に更新し、テストの意図（応答のバイト不変・単一シーン）は保つ（議題 5 で決定。[OPEN-8] 解消）。
2. When 辞書が章ごとのファイルに組み替わる（Requirement 2.1・2.3）, the 本 spec shall ファイル名に依存する既存テスト（`dist_src_validation_test.rs` の必須ファイル一覧、`integration_test.rs`・`src/scripts.rs` のファイル単位の構造検査: `talk.pasta` の `OnTalk` 5〜10 個、`click.pasta` の `OnMouseDoubleClick` 7 個以上、`時報12`・`時報その他`・`＄時１２` の存在、イベント辞書にグローバルアクター辞書を置かない、シーン内の表情名がアクター辞書に定義済み）を、新しいファイル構成に合わせてテストの意図（必須ファイルの存在・構造の保証）を保ったまま更新する。
3. While `pasta_shiori` の e2e テスト（`scene_kick_*_e2e_test.rs`）が hello-pasta を一時ディレクトリへコピーしてシーンを追加している, the hello-pasta の辞書 shall これらが追加するシーン名と衝突せず、`pasta.toml` の読み替え（talk 間隔の上書き）を妨げない。
4. When hello-pasta の辞書が章ごとのファイルに組み替わる, the `book/src/getting-started/first-ghost.md` の ```` ```pasta ```` ブロック shall 変更後の全ファイルと逐語一致する状態に保たれ、`node book/tools/tutorial-check.mjs` が exit 0 で終わる。本 spec が行う追従は**機械的な範囲**に限る: pasta ブロックを新しいファイル群に差し替える、辞書の行を引用している本文（`起動したよ～`・`＄ゴースト名` など）と見出しのファイル名を直す。章立て・Claudia の語り・照合方式（完成形 1 か所との照合）は変えず、全面的な書き直しは `getting-started-story-guide` に残す（議題 11 で決定。[OPEN-9] 解消）。
4a. When hello-pasta の辞書のファイル名が変わる, the `book/tools/tutorial-check.mjs` shall 固定で列挙しているファイル名（`DIC_FILES`）に依存せず、hello-pasta の `dic/` に実在する `.pasta` ファイル（または段階表）から照合対象を導く。照合方式そのもの（各ファイルがいずれかのブロックと逐語一致）は変えない。`tutorial-check-test.mjs` も同じ前提に合わせる。
4b. The `.github/workflows/manual.yml` shall `paths` に hello-pasta の `dic/`（`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/**`）を加え、辞書だけを変える PR でも tutorial-check とチュートリアル構文検証（`cargo test -p pasta_sample_ghost`）が走るようにする。
5. The 本 spec shall `cargo test --all` と `cargo clippy --all-targets --workspace -- -D warnings`、`node book/tools/tutorial-check.mjs` が完了時点で成功する状態で終わる。

### Requirement 6: 配布物の変化の周知

**Objective:** As a hello-pasta の利用者, I want サンプルゴーストの辞書が大きく変わったことがリリースで分かること, so that 以前の辞書を手本にしていた人が差分に気づける。

#### Acceptance Criteria

1. When hello-pasta の辞書が書き直される, the 本 spec shall 書き直しを取り込むマージコミット（PR タイトル）を、リリースノートの生成（git log を Conventional Commits の種類で分類する `release-workflow` の方式）に拾われる `feat(pasta_sample_ghost): …` の形で、辞書が教材として書き直されたこと・emo2 との切り替えの作例が入ったことが利用者に分かる言葉で記す。
2. When hello-pasta の辞書が書き直される, the `crates/pasta_sample_ghost/README.md` shall 辞書の構成（ファイル一覧と各ファイルの役割）の記述を変更後の辞書に合わせる。
3. The 本 spec shall `.nar` に同梱する文書を増やさない（`install.txt`・`descript.txt` は変えず、`RELEASE.md` はリリース手順書のまま利用者向けの変更履歴を持たせない）。
4. The hello-pasta の配布物 shall 辞書以外のファイル（`install.txt`・`descript.txt`・`pasta.toml`・シェル）を本 spec では変えない。
