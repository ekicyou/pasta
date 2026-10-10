# Research & Design Decisions

## Summary
- **Feature**: `getting-started-story-guide`
- **Discovery Scope**: Extension（既存のマニュアル基盤・検査ツールの上で、入門ガイドを 3 章から 16 章前後に書き直す）
- **Key Findings**:
  - 上流はすべて main に入っている。段階表（`crates/pasta_sample_ghost/STAGES.md`・13 段）と段階辞書（`dic/01-boot.pasta`〜`12-lua.pasta`）は確定済みで、全段の起動は `tests/tutorial_stages_test.rs` が保証する。台詞部品の記法・登録簿・検査も完成している。本文の執筆は今すぐ始められる。
  - 機械検査の側で「本文に Claudia の語りを置く」ことを止めるものは無い。本文の台詞・口調を禁じる `extractBody`（`talk-in-body`・`voice-in-body`）は生成対象章と内部設計章だけにかかり、`getting-started` は対象外（`gen-skill-refs-test.mjs` K-9 がそれを確かめている）。止めているのは執筆規約（`AUTHORING.md` 第 1・2・3 節の文章）だけである。
  - 作例の照合 `tutorial-check.mjs` は `first-ghost.md` 1 枚への固定（`TUTORIAL_REL`）と「どのブロックでもよい」照合（`matchDicFile`）で、段と章の対応を見ない。段ごとの照合に広げるには、照合の単位を「辞書ファイル ↔ 段の章」に変え、自己テストのサンドボックス（`makeSandbox`）も作り直す。
  - 章を足すと落ちる決め打ちが 3 か所（`gen-skill-refs-test.mjs:599` K-10、`talk/talk-test.mjs:453-454` J-9、`verify-scripts-test.mjs:72` T 系の件数。すべて 47）。`verify-content.mjs` の `C-steps`（`first-ghost.md` の実在と 3000 文字）も名指しで落ちる。
  - README のリンク検証 `readme-manual-url`（`link-check.mjs`）は `first-ghost.md` にある見出し `ゴーストのフォルダ構成` の実在を見る。`book.toml` の転送だけでは通らず、2 つの README を新しい章の実在する見出しへ向け直す必要がある。

## Research Log

### 既存の入門ガイドと段階表の対応
- **Context**: 今の 3 章を 16 章前後に分けるときの、章と段階・辞書の対応。
- **Sources Consulted**: `book/src/getting-started/{index,prerequisites,first-ghost}.md`、`crates/pasta_sample_ghost/STAGES.md`、`dic/*.pasta`、`.kiro/specs/completed/hello-pasta-tutorial-stages/brief.md`
- **Findings**:
  - `first-ghost.md`（589 行）は「フォルダ構成 → シェル（`descript.txt`・`surfaces.txt` を転記）→ 辞書 12 ファイルを順に転記 → 設定ファイル（`install.txt`・`descript.txt`・`pasta.toml` を転記）→ 起動 → トラブルシュート」の構成。13 段目（配布）の節は無い。
  - 辞書の各ファイルは先頭行が `＃ N 段目：<願い>` で、続く `＃` コメントがその段の説明を含む。章の説明は辞書のコメントと二重になるが、食い違わなければよい（Requirement 3.8）。
  - 段階表の `使う文法要素` 列は、各段で送り出すべきリファレンス章のリンク先を既に相対パスで持っている（`../../book/src/grammar/...`）。章のリンクはこれを `../grammar/...` に読み替えれば揃う。
  - 段階表の「確かめるための道具」節が、6 段目（開発者用機能・時刻の仮想変更）・7 段目（スクリプト入力の `\![raise,…]`・`\-` で終了しない）・13 段目（NAR 作成）の操作を指定している。章の「試す」節の材料はここにある。
  - 古い記述 4 か所（途中では起動しない／`scripts/` に Lua ランタイム／8 段目は書き方が変わらない／シェル画像の自動生成）は書き直しで消える。
- **Implications**: 章立ては「入口・準備（1〜2 章）・13 段」で機械的に決まる。章の本文の素材（説明・リンク先・試す手順）は段階表と辞書コメントにほぼ揃っており、執筆は「語り」の付与が主な仕事になる。

### 検査ツールが入門の章にかける規則
- **Context**: 「全編を Claudia が語る」が機械検査と衝突するか。
- **Sources Consulted**: `book/tools/verify-content.mjs`、`gen-skill-refs.mjs`（`extractBody`・`findVoice`・`chapterRegions`）、`talk/talk.mjs`（`scanTalk`・`checkRegion`）、`verify-static.mjs`、`verify-search.mjs`、`book/AUTHORING.md`
- **Findings**:
  - 入門の章にかかる検査: `C-utf8`・`C-sjis`・`C-env`（`getting-started/*.md` を連結した正規表現）、`C-steps`（`first-ghost.md` 名指し）、`C-tutorial-check`、`D-voice`（散文に口調が 1 つ以上「ある」こと。禁止ではない）、`D-codevoice`（フェンス内に `わたくし`・`おほほ`・`フンッ` 等が無い）、`T-syntax`・`T-intro`・`T-outro`（先頭 H1・`---` 2 本以上・導入と締めは台詞だけで両方の話し手）。
  - 本文の台詞・口調を禁じる `extractBody` は `GENERATION_MAP` の章と `internals/` にだけかかる。入門の章は対象外。
  - 出力の検査 `verify-static` は、全ページに両方の話し手の台詞部品があることを見る（導入・締めの規則を守れば自動的に満たす）。`verify-search` は各章の最初の台詞の 4 文字以上の日本語片でその章がヒットすることを見る（最初の台詞を章に固有の言葉で始める必要。Requirement 7.6）。
  - `D-codevoice` の禁止語は Claudia の語りの語なので、辞書の作例（コードフェンス内）に `わたくし` 等を入れられない。段階辞書は「女の子」「男の子」の台詞なので衝突しない。
  - 規約の側: `AUTHORING.md` 46 行（本体は普通文体）・113 行（本体に台詞を置かない）・147 行（サンプル B）・第 2 節の使い分け表・第 7 節「置ける場所」（入門の本体には置ける、と既に書いてある）。語りを本文に入れるには第 1〜3・5 節の文章を直す。第 7 節は書き換えない（上流の申し送り）。
- **Implications**: 機械検査の直しは小さい（`C-steps` と章の数）。規約の直しは文章の話で、新しい節を 1 つ足して第 2・5 節から案内する形が最小。

### 作例の照合（tutorial-check）の現状と拡張の余地
- **Context**: Requirement 9（段ごとの照合）。
- **Sources Consulted**: `book/tools/tutorial-check.mjs`、`tutorial-check-test.mjs`、`verify-content.mjs:241-247`、`.github/workflows/manual.yml:112-119`
- **Findings**:
  - 公開関数: `listDicFiles`（`dic/*.pasta` を辞書順・固定一覧なし）、`extractPastaBlocks`（```` ```pasta ```` 限定・4 本の ```` ````pasta ```` で内側の ```` ```lua ```` を保持）、`normalizeForCompare`（改行と末尾空白）、`matchDicFile`（どのブロックでもよい）、`runTutorialCheck`、`reportTutorialCheck`（対処文に `first-ghost.md` を決め打ち）。
  - 使う側は `verify-content.mjs`（`runTutorialCheck` を呼ぶ）、`manual.yml`（CLI）、自己テスト（import と CLI）。`TUTORIAL_REL` を外部から使うものは無い。
  - 自己テスト（298 行・B-1〜B-9）は 1 ファイルのサンドボックスを前提にしている。段ごとの照合にすると B-1・B-2・B-3・B-6・B-7 とサンドボックスの作り方が変わる。
  - 13 段目は辞書を持たない。段ごとの照合は「辞書 → 章」の向きで回せば 13 段目を自然に素通りできる（章 → 辞書の向きで回すと例外が要る）。
  - `manual.yml` の起動条件に `STAGES.md` は無い。段階表を照合に使うなら本来は足すべきだが、`manual.yml` はこのウェーブで触らない決まり（`manual-print-media-refs` が持つ）。照合が段階表を読まなければ問題にならない。
- **Implications**: 照合の単位を「辞書ファイル 1 つ ↔ 段の章 1 つ」に変えるのが本質。段と章の対応の持たせ方（章のファイル名の `NN`、章の中の印、固定の対応表）が設計の論点。段階表を読まない方式なら `manual.yml` を触らずに済む。

### 章の数を決め打ちしている自己テスト
- **Context**: Requirement 10.4。
- **Sources Consulted**: `gen-skill-refs-test.mjs:599-606`、`talk/talk-test.mjs:453-454`、`verify-scripts-test.mjs:56,69-72`、`verify-content.mjs:21`
- **Findings**:
  - K-10（`gen-skill-refs-test.mjs`）は 47 章を数えるだけでなく、全章で `chapterRegions` が成功し（H1＋`---` 2 本）、本体の範囲が旧来の判定と一致することも見る。新しい章も同じ型で書けば件数だけ直せばよい。
  - J-9（`talk-test.mjs`）は `--stats` の章の行数＝47。T 系（`verify-scripts-test.mjs`）は `T-syntax`・`T-intro`・`T-outro` の件数＝47 と、`verify-content` の検査総数 50 以上。
  - 3 か所とも「章の数」を 1 つの数字で持つ。数字を 60 に直すのが最小。`SUMMARY.md` から数える形にすれば以後の章の追加で落ちなくなるが、検査が弱まる（章を消しても気づかない）。
- **Implications**: 最小は数字の 3 か所の書き換え（`47` → 新しい章の数）。導出に変えるかは設計の判断（`pasta-check-dic-validate` が後で章を足す可能性がロードマップにある）。

### 旧ページからの導線
- **Context**: Requirement 11。
- **Sources Consulted**: `book/book.toml`、`book/tools/link-check.mjs:372-400`、`crates/pasta_lua/README.md:32`、`crates/pasta_shiori/README.md:77`、`book/src/introduction.md`、`book/theme/claudia.css:47`
- **Findings**:
  - `book.toml` には `[output.html.redirect]` が既にあり（`/lua/modules.html`）、`first-ghost.html` の転送を 1 行足せる。
  - 2 つの README は `getting-started/first-ghost.html#ゴーストのフォルダ構成` を指す。`readme-manual-url` 検査は対応する `.md` の実在と見出し slug の実在を見るので、転送では通らない。新しい準備の章に `## ゴーストのフォルダ構成` の見出しを置き、README をそこへ向け直す。
  - `introduction.md` の扉（`hero-toc`）は `getting-started/index.md` を指しており、入口の章のファイル名を変えなければ触らずに済む。「このマニュアルの歩き方」の文は書き換える。
  - `claudia.css:47` のコメントは `getting-started/index.md` の目次行を例示しているだけで、影響しない。
- **Implications**: 入口の章 `index.md` と `prerequisites.md` のファイル名は残すのが最小。消すのは `first-ghost.md` だけで、転送 1 行と README 2 行の直しで足りる。

### 上流からの申し送り（7・8 段目の実機の事実）
- **Context**: Requirement 4.5・4.8。
- **Sources Consulted**: `brief.md`（2026-10-09・2026-10-10 の申し送り）、`STAGES.md`、`dic/07-greeting.pasta`、`dic/08-touch.pasta`
- **Findings**:
  - ゴースト自身の `\![change,ghost]` では `OnGhostChanging` が届かない。SSP のメニューからの切り替えでは届く。読者に試させる手順はメニューからにする。
  - 8 段目: 今のシェルには当たり判定が無く、`＄ｒ４` は空になる。辞書は空でも読めるように書いてある。部位名は `Head` で確定（`hello-pasta-shell-art` は変えない）。
  - 7 段目の辞書コメントは、既に「このシーンで話すと OnBoot は来ない／何も話さずに終えると（204）次に OnBoot が来る」を書いている。章はこれと同じ事実を語ればよい。
- **Implications**: 8 段目の過渡期（当たり判定が無い）を本文に書くかは要件の未決事項 6。書かない前提なら、章は段階表の `Head` で完結する。

## Requirement-to-Asset Map

| 要件 | 既存の資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 章立て・目次 | `SUMMARY.md` 入門パート 3 行、`index.md` | Missing: 13 段の章と準備の章。`SUMMARY.md` の入門パートを 16 行前後に |
| 2 準備の章 | `prerequisites.md`（64 行・技術内容は正しい）、`first-ghost.md` のフォルダ構成・設定ファイルの節 | Constraint: シェルの転記を消す。`scripts/` の誤りを直す。`## ゴーストのフォルダ構成` の見出しを残す（README のリンク） |
| 3 段の章の型 | 辞書の `＃` コメント、段階表の `使う文法要素`・`確かめるための道具` | Missing: 章そのもの。Constraint: 作例は辞書と逐語一致 |
| 4 段ごとの固有の内容 | 段階表・上流の申し送り・辞書コメント | Missing: 文章。Unknown: 8 段目の過渡期の扱い（未決 6） |
| 5 イベントの出典 | `lua/shiori-events.md`、`grammar/variables.md`、UKADOC（外部） | Missing: UKADOC の該当項の URL とアンカー（`list_shiori_event.html#OnBoot` 等）の確認。Research Needed: 各イベントの項のアンカー名 |
| 6 emo2 | `hello-pasta-tutorial-stages/brief.md` の「emo2 からの知見」 | Constraint: 書く事実はその範囲。emo2 の辞書は照合しない |
| 7 Claudia の語り | 台詞部品（`talk.mjs`・`talk-html.mjs`）、14＋2 表情、第 7 節の記法 | Unknown: 語りの形（未決 1）。Constraint: 最初の台詞は固有の語、両方の話し手が導入・締めに要る |
| 8 執筆規約の例外 | `AUTHORING.md` 第 1〜3・5・7 節 | Missing: 例外の節。Constraint: 第 7 節は書き換えない。`T-authoring` は第 7 節の表を見る（新しい節は影響しない） |
| 9 段ごとの照合 | `tutorial-check.mjs`（公開関数は再利用できる）、自己テスト | Missing: 段 ↔ 章の対応、章の欠落の検出、13 段目の素通り、自己テストの作り直し、`reportTutorialCheck` の対処文 |
| 10 検査・自己テストの追従 | `verify-content.mjs` C 系・D 系・T 系、3 つの自己テスト | Missing: `C-steps` の直し、`47` の 3 か所。Constraint: `manual.yml`・`verify-static.mjs` は触らない |
| 11 案内・転送・リンク | `book.toml` の redirect、`introduction.md`、README 2 つ | Missing: redirect 1 行、「歩き方」の文、README の向け直し |
| 12 正確さ・権威 | リファレンス章、UKADOC | Constraint: 現行版の文法だけ。Research Needed: 各章の事実をリファレンス章と突き合わせる作業は実装時 |

## Architecture Pattern Evaluation

### 段と章の対応の持たせ方（Requirement 9.2）

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 章のファイル名に段階番号 | `getting-started/NN-name.md`（辞書の `NN-name.pasta` と同じ `NN`） | 照合が `listDicFiles` の `NN` だけで章を引ける。固定の一覧が要らない。URL が段の順に並ぶ | 章のファイル名が ASCII の規則に縛られる。辞書と章で `name` が食い違うと混乱する（照合は `NN` だけ見れば防げる） | 辞書の命名規則（`STAGES.md`「ファイル名の規則」）と対称 |
| 章の中の印 | 章の先頭か ```` ```pasta ```` フェンスの前に、どの辞書ファイルかを示す行（例: 見出し `### dic/07-greeting.pasta`） | 章のファイル名を自由に付けられる | 印の記法を新しく決めて規約に書く。印が無い章・重複する章の検出が要る | 今の `first-ghost.md` が `### 01-boot.pasta` の見出しで同じことをしている |
| 固定の対応表 | ツール内に `{ '01-boot.pasta': 'getting-started/boot.md', … }` | 単純 | 辞書を足すたびにツールを直す（Requirement 9.7 に反する） | 採らない |

### 章の数の決め打ち（Requirement 10.4）

| Option | Description | Strengths | Risks / Limitations |
|--------|-------------|-----------|---------------------|
| 数字を直す | 3 か所の `47` を新しい数に | 最小の差分。検査の強さは今のまま | 次に章を足す spec も 3 か所を直す（ロードマップは既にそれを前提にしている） |
| `SUMMARY.md` から数える | 自己テストが目次の章数を数えて期待値にする | 以後の章の追加で落ちない | 章を消しても気づかない。3 つの自己テストで数え方を共有する仕組みが要る |

### 執筆規約の例外の置き方（Requirement 8）

| Option | Description | Strengths | Risks / Limitations |
|--------|-------------|-----------|---------------------|
| 新しい節を足す | 第 8 節「入門ガイドの執筆規約」を足し、第 2・5 節から案内 | 第 7 節を触らない（申し送り）。他の章の規則が変わらない | 第 1〜3 節の「本体は普通文体」の文に「入門ガイドを除く」の但し書きが要る |
| 第 2 節の表を書き換える | 使い分け表に「入門ガイドの本体」の行を足す | 1 か所で分かる | 表だけでは章の型（願い → 表現 → 辞書 → 試す）を書けない。結局 節が要る |

## Implementation Approach Options

### Option A: 既存の構成を拡張する
- `tutorial-check.mjs` の公開関数（`listDicFiles`・`extractPastaBlocks`・`normalizeForCompare`）を残し、`runTutorialCheck` の照合の単位だけを「辞書 ↔ 章」に変える。`TUTORIAL_REL` を章のディレクトリ（`book/src/getting-started`）に置き換える。
- `verify-content.mjs` は `C-steps` を「段の章がすべて実在し、各章が十分な長さを持つ」に直す。
- 自己テストは 3 か所の数字を直す。
- ✅ 差分が小さく、CI の段（`manual.yml`）を触らない。
- ❌ `tutorial-check-test.mjs` のサンドボックスは作り直しになる（避けられない）。

### Option B: 照合ツールを新しく作る
- 段ごとの照合を別のスクリプトにし、旧 `tutorial-check.mjs` を消す。
- ✅ 旧方式の名残が残らない。
- ❌ `manual.yml` のコマンド名が変わる（このウェーブで触れない）。`verify-content.mjs` の import も変わる。採る理由が無い。

### Option C: 混合
- ツールは A。章の本文は全部を新規に書き（`first-ghost.md` は消す）、`prerequisites.md` の技術内容と `first-ghost.md` のフォルダ構成・設定ファイル・トラブルシュートの節を準備の章へ移す。
- ✅ 本文の「全面書き直し」と、ツールの「最小の拡張」を両立する。
- ❌ 準備の章に移す内容の取捨（トラブルシュートをどこに置くか）が設計の論点になる。

## Implementation Complexity & Risk
- **Effort**: L（1〜2 週間）。章 16 枚前後の執筆が大半で、各章に語り・作例・試す手順・リンク・出典を揃える。ツールの直しは S 相当。
- **Risk**: Medium。技術は既知だが、(1) 語りの形の決定が 16 枚全部の形を決める、(2) 各章の事実をリファレンス章と突き合わせる作業は機械検査されない、(3) 検査の規則（導入・締めの両方の話し手、最初の台詞の固有語、`D-codevoice`、`chapterRegions`）を 16 枚で守る必要がある。

## Recommendations for Design Phase
- **推奨**: Option C（ツールは既存を拡張、本文は新規）。段と章の対応は「章のファイル名に段階番号」が最小（固定の一覧も新しい記法も要らない）。章の数は数字の 3 か所を直す。
- **設計で決めること**:
  - 準備の章を 1 章にするか 2 章にするか（`prerequisites.md` を残して「最小一式の配置」を足すのが最小）。
  - 章のファイル名の規則と、`first-ghost.html` の転送先。
  - 「試す」節の共通の型（起動／再読み込みの操作、何が見えれば成功か）と、SSP の再読み込みの操作の書き方。
  - 語りの形（未決 1）に応じた章の雛形。雛形は `AUTHORING.md` の新しい節に載せる。
  - 各章の UKADOC リンク（`list_shiori_event.html#OnBoot` 等）と引用の形。
- **Research Needed**:
  - UKADOC の各イベントの項のアンカー名と、`Reference` の説明文の確認（`OnBoot`・`OnFirstBoot`・`OnClose`・`OnGhostChanged`・`OnGhostChanging`・`OnMouseDoubleClick`・`OnChoiceSelectEx`）。
  - SSP の開発用パレットの操作名（「現在時刻の仮想的変更」「スクリプト入力」「`\-` タグで終了しない」）の UKADOC SSP ヘルプの項。
  - SSP でのゴーストの再読み込み操作（辞書を足した後に起動し直す最短の手順）。
  - `verify-search` の最初の台詞の語が 16 枚で互いに衝突しないことの確かめ方（`--self-test` で分かる）。

## Risks & Mitigations
- 語りの形が決まらないまま章を書き始めると 16 枚の書き直しになる — 要件ディスカッション（未決 1）で先に決め、1 章の雛形を設計で固めてから量産する。
- 作例の照合が「辞書 → 章」の向きで回るため、章に余分な ```` ```pasta ```` ブロック（段の辞書でない例示）があっても検出されない — 要件ディスカッション（議題 5）で決定: 段の章の `pasta` ブロックはすべて「辞書全体」か「辞書の連続した抜き出し」のどちらかでなければならず、照合がそれを確かめる（Requirement 9.10・9.11）。
- リファレンス章との食い違いは機械検査されない — 執筆時に段階表の `使う文法要素` のリンク先を開いて事実を写す手順をタスクに入れる。
- `hello-pasta-shell-art` より先に本文が main に入ると、8 段目は当たり判定が無いシェルで動く — 本文は段階表の `Head` で書き、過渡期の注記は置かない（未決 6）。

## References
- `crates/pasta_sample_ghost/STAGES.md` — 段階表の正本（章立て・リンク先・確かめる道具）
- `.kiro/specs/completed/hello-pasta-tutorial-stages/brief.md` — emo2 からの知見・切り替えの作例の制約
- `.kiro/specs/completed/manual-claudia-theme/design.md` — 台詞部品と検査の設計
- `book/AUTHORING.md` 第 7 節 — 台詞部品の記法と検査
- [UKADOC SHIORI イベント一覧](https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html) — イベントの出典
- [UKADOC SSP ヘルプ 開発用パレット](https://ssp.shillest.net/ukadoc/ssphelp/dev-palette.html) — 試す手順の出典

---

# 設計フェーズの追補（2026-10-10）

ここから下は `/kiro-design` の設計フェーズで足した記録である。上の節（要件フェーズのギャップ分析）は書き換えていない。

## 設計フェーズの Summary
- **Discovery Scope**: Extension（Light Discovery）。新しい依存・外部サービスは無い。既存のツール 2 本（`tutorial-check.mjs`・`verify-content.mjs`）の拡張と、文章の新規執筆である。
- **Key Findings**:
  - 段と章の対応は「辞書と章を同じ名前にする」だけで決まる（`dic/07-greeting.pasta` ↔ `getting-started/07-greeting.md`）。段階番号を切り出す正規表現も、固定の対応表も、章の中の印も要らない。
  - 転送ページは既存の `/lua/modules.html` と同じ形（相対の転送先）にすれば、触れない `verify-static.mjs` をそのまま通る。
  - 読者が開発者用機能を有効にするのは 6 段目である。1〜5 段目では開発用パレットの「リロード」を使えないので、章末の確かめ方の共通操作は「SSP を終了して起動し直す」になる。
  - 辞書が 1 つも無い状態でも読み込みは成功する（`crates/pasta_lua/tests/loader/startup_test.rs` の `test_load_empty_dic`。警告 `No .pasta or .lua files found` を出して続行）。準備の章の終わりの状態（要件 2.6）は、読み込みの段では成り立つ。

## 参照したスキルと指針
- `kiro-spec-design` の `rules/`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）。境界を先に決める・要件 ID の対応表・ファイル構成の計画に使った。
- `ponytail`（最小の設計）。新しい抽象を足さず、既存の関数とパターンを使い回す判断に使った。
- UKADOC・SSP ヘルプの検索（伺かドキュメントの MCP）。イベントの項のアンカーと、開発用パレットの操作名の確認に使った。
- `pasta-ghost-authoring`・`pasta-lua-coding` は読んでいない（辞書と Lua を変えないため）。

## Research Log（設計フェーズ）

### UKADOC のイベントの項のアンカー
- **Context**: 要件 5.1。各イベントの UKADOC の項へリンクし、説明と `Reference` を短く引用する。
- **Sources Consulted**: UKADOC `list_shiori_event.html`（`OnBoot`・`OnFirstBoot`・`OnClose`・`OnGhostChanged`・`OnGhostChanging`・`OnMouseDoubleClick`・`OnChoiceSelectEx` の 7 項を確認）、`list_sakura_script.html`（`\![change,ghost,…]`）、SSP ヘルプ `config-dev.html`（「ディレクトリをドロップした際に更新ファイルやNARを作成」）
- **Findings**:
  - アンカーはイベント名そのまま（`https://ssp.shillest.net/ukadoc/manual/list_shiori_event.html#OnGhostChanged`）。
  - `OnGhostChanged`・`OnFirstBoot` の項は「スクリプトが返されなかった（204）場合、続けて OnBoot が発生する」、`OnGhostChanging` の項は「続けて OnClose が発生する」と書いている。要件 4.4 の根拠はここにある。
  - `OnGhostChanging` の Reference0 は「切り替わるゴーストの本体側の名前」、`OnGhostChanged` の Reference0 は「直前のゴーストの本体側の名前」、`OnMouseDoubleClick` の Reference4 は「当たり判定の識別子」、`OnChoiceSelectEx` の Reference0・1 は「選択肢のテキスト（ラベル）」「選択肢の ID」。
  - `\![change,ghost,ゴースト名]` は `--option=raise-event` を付けない限り `OnGhostChanging` を通知しない、と UKADOC に書いてある。上流の申し送り（ゴースト自身の切り替えでは届かない）と一致する。
- **Implications**: 引用は「イベントの説明の 1 文」と「その段で使う Reference の説明」だけにする。引用の置き場所は表のセル（要件 7.3 が台詞以外に許すのは指示の一文・箇条書き・表・コードブロック・見出しだけで、ふつうの引用ブロックは入っていない）。`OnBoot` の項は「OnGhostChanged、OnGhostCalled、OnFirstBoot、OnVanished に対してスクリプトが返されなかった（204）場合、続けてこのイベントが発生する」と書いており、7 段目の説明の根拠にもなる。

### SSP の開発用パレットと再読み込み
- **Context**: 要件 3.4・3.5・4.2・4.6。章末の「起動して確かめる」の共通操作。
- **Sources Consulted**: SSP ヘルプ `dev.html`・`dev-palette.html`・`shortcut.html`、`book/src/debug/dev-actions.md`
- **Findings**:
  - 開発用パレットは本体設定「一般」の「開発者用機能を有効にする」が ON のときだけ開ける（`Ctrl+Shift+D`）。項目名は「スクリプト入力」「現在時刻の仮想的変更」「`\-`タグで終了しない」「リロード」。
  - 「リロード」はメニューから選んだ内容を読み込み直す。`\![reload,shiori]` でも SHIORI が読み込み直される（`debug/dev-actions.md`）。
  - 段階表は開発者用機能の有効化を 6 段目で初めて扱う。1〜5 段目の読者は開発用パレットを使えない。
- **Implications**: 全段で使える確かめ方は「SSP を終了して起動し直す」だけである。これを共通操作にする。速い再読み込み（6 段目以降の「リロード」）を案内するかは設計ディスカッションで決める。

### 辞書が 1 つも無い起動
- **Context**: 要件 2.6。準備の終わりは「辞書が無いので起動してもしゃべらない」。
- **Sources Consulted**: `crates/pasta_lua/src/loader/mod.rs`、`crates/pasta_lua/tests/loader/startup_test.rs`、`crates/pasta_sample_ghost/tests/tutorial_stages_test.rs`
- **Findings**: 読み込みは警告を出して成功する（`test_load_empty_dic`）。段階の検証（`tutorial_stages_test.rs`）は 1〜13 段目を見ており、辞書が無い段は見ていない。SSP の上で「立ち絵は出るがしゃべらない」になることは、機械検査では確かめていない。
- **Implications**: 準備の章の終わりの状態は、実装の通しの確認（実機の SSP）で 1 回確かめる。

### 転送ページと出力の検査
- **Context**: 要件 11.3。`verify-static.mjs` は触れない。
- **Sources Consulted**: `book/book.toml`、`book/tools/verify-static.mjs`（`listFiles`・相対参照の検査）
- **Findings**: 既存の転送は `"/lua/modules.html" = "modules/index.html"` で、転送先を転送元のフォルダからの相対で書いている。`verify-static` は全 HTML の相対参照が解決することと、ルート絶対（`/…`）の参照が無いことを見る。転送ページは章の一覧（SUMMARY 由来）に入らない。
- **Implications**: `"/getting-started/first-ghost.html" = "index.html"` と相対で書けば、既存の転送と同じ形になる。転送先を `/pasta/…` のように絶対で書くと落ちる。実際に通ることは、実装でビルドして確かめる。

### 口調の検査のフェンスの数え方
- **Context**: 12 段目の辞書は ```` ```lua ```` を含むので、章では 4 本のバッククォートで囲む。
- **Sources Consulted**: `book/tools/verify-content.mjs`（`extractCodeFences`・`stripCodeFences`）、`book/tools/link-check.mjs`（`maskFences`）
- **Findings**: `verify-content` の口調の検査（`D-codevoice`）は、3 本のバッククォートを単純に 2 つずつ対にして「フェンスの中」を決める。4 本のフェンスの中に ```` ```lua ```` と ```` ``` ```` が 1 組あれば数は偶数で、対はずれない（今の `first-ghost.md` が通っている理由）。抜き出しが ```` ```lua ```` の開きだけ、または閉じだけを含むと数が奇数になり、その後ろの台詞が「フェンスの中」と見なされて `わたくし` などが誤検出される。台詞部品の検査（`scanTalk`）と章構造（`chapterRegions`）は CommonMark どおりの `maskFences` を使うので、この問題は無い。
- **Implications**: 検査を直さず、執筆規約に「```` ```lua ```` を含む抜き出しは、開きと閉じの両方を含める」と書く。全 60 章にかかる検査の判定を変えるより安全である。

### 章の数の検算
- **Findings**: 今は 47 章（うち入門 3 章）。入門を 16 章（入口 1・準備 2・段 13）にすると 47 − 3 ＋ 16 ＝ 60 章。3 つの自己テストはどれも同じ集合（`book/src` の章、`debug/` を含む）を数えている。

### `manual.yml` に残る古いコメント
- **Findings**: `.github/workflows/manual.yml` 114 行目のコメントが `first-ghost.md` を名指ししている。実行される行（`node book/tools/tutorial-check.mjs`）は変わらない。このウェーブでは `manual.yml` を触れない。
- **Implications**: コメントの直しは `manual-print-media-refs`（`manual.yml` を持つ spec）へ申し送る。

## Design Decisions

### Decision: 準備の章は 2 章にする
- **Context**: 要件 1.1・2。設計で決める項目 (1)。
- **Alternatives Considered**:
  1. 1 章にまとめる — `prerequisites.md` に最小一式の配置まで入れる。
  2. 2 章に分ける — `prerequisites.md`（道具と約束）と `setup.md`（最小一式を置く）。
- **Selected Approach**: 2 章。`prerequisites.md` はファイル名を変えずに残し、`setup.md` を足す。
- **Rationale**: 準備の要件は 8 項目あり、台詞部品で語ると 1 章では長すぎる。「読むだけの章」と「手を動かす章」で分かれる。要件の前提（16 章）とも一致する。
- **Trade-offs**: 1 段目までに 3 章を読むことになる。入口の章の一覧で「準備は 2 章」と先に見せて補う。

### Decision: 段と章は同じ名前で対応させる
- **Context**: 要件 9.2・9.4・9.7。設計で決める項目 (3)。
- **Alternatives Considered**:
  1. 章のファイル名の先頭 2 桁（`NN`）だけを見る — 章の `name` は自由。
  2. 辞書と章を同じ名前にする — `dic/NN-name.pasta` ↔ `getting-started/NN-name.md`。
  3. 章の中の印（見出しやコメント）で辞書を指す。
  4. ツールの中の固定の対応表。
- **Selected Approach**: 2。照合は「辞書の名前の拡張子を `.md` に替えた章があるか」を見るだけになる。
- **Rationale**: 対応を引く処理が 1 行で済み、同じ番号の章が 2 つある・番号の無い辞書がある、といった場合分けが要らない。固定の一覧を持たない今の性質（9.7）も保てる。段階表の「ファイル名の規則」（ASCII・`NN-name`）とそろう。
- **Trade-offs**: 辞書の名前を変えると章の名前（＝URL）も変わる。段階表は名前を確定済みなので、頻度は低い。
- **Follow-up**: 13 段目は辞書が無いので、章の名前だけを決める（`13-nar.md`）。

### Decision: 入門の全章の `pasta` ブロックを同じ規則で照合する
- **Context**: 要件 9.10・9.11・8.5。段の章でない章（入口・準備）に `pasta` ブロックを置いた場合の扱いは、要件が明示していない。
- **Alternatives Considered**:
  1. 段の章だけを照合し、入口・準備の章は見ない（規約で「置かない」と書くだけ）。
  2. 入門の全章を同じ規則で見る — 章と同じ名前の辞書が無ければ、その章の `pasta` ブロックはすべて失敗。
- **Selected Approach**: 2。「章の `pasta` ブロックは、その章と同じ名前の辞書の全体か連続した抜き出しである」という 1 つの規則を、入門の全章にかける。辞書の無い章（入口・準備・13 段目）は、結果として `pasta` ブロックを置けない。
- **Rationale**: 1 だと、照合されない作例を入口や準備の章に置く抜け道が残る（8.5 の「作例は辞書と一致」に穴が開く）。2 は場合分けが無く、実装も 1 より短い。入口・準備・13 段目に `pasta` の作例を置く必要は、要件のどこにも無い。
- **Trade-offs**: 9.10 の文面（「段の章にある」）より広くかける。設計ディスカッションで確かめる。

### Decision: 廃止ページは入口の章へ転送する
- **Context**: 要件 11.3。設計で決める項目 (2)。
- **Alternatives Considered**: 入口の章 `index.html`／最小一式の章 `setup.html`／1 段目 `01-boot.html`。
- **Selected Approach**: `"/getting-started/first-ghost.html" = "index.html"`。
- **Rationale**: 旧ページは「最初から最後まで」の 1 枚だった。新しいガイドでそれに当たるのは、進め方と 13 章の一覧を持つ入口の章である。README の深いリンク（`#ゴーストのフォルダ構成`）は転送に頼らず、`setup.html` の見出しへ直接向け直す。

### Decision: 章の数は数字を直す
- **Context**: 要件 10.4。設計で決める項目 (4)。
- **Alternatives Considered**: 3 か所の `47` を `60` に直す／目次から数える。
- **Selected Approach**: 数字を直す。
- **Rationale**: 差分が最小で、検査の強さが変わらない（章が消えたら落ちる）。ロードマップも「章を足す spec が数を直す」前提で書かれている。目次から数える形は、3 つの自己テストで数え方を共有する仕組みが要り、章の消失を見逃す。

### Decision: 章末の確かめ方の共通操作は「SSP を終了して起動し直す」
- **Context**: 要件 3.4。
- **Alternatives Considered**: 起動し直す／開発用パレットの「リロード」／スクリプト入力の `\![reload,shiori]`。
- **Selected Approach**: 全段で「SSP を終了して起動し直す」を共通の手順にする。
- **Rationale**: 開発者用機能を有効にする前（1〜5 段目）でも使え、道具が要らない。段階表の「確かめるための道具」に無い操作を足さずに済む。
- **Trade-offs**: 6 段目以降は「リロード」のほうが速い。案内するかは設計ディスカッションで決める。

### Decision: UKADOC の引用と用語の説明は、表と箇条書きに置く
- **Context**: 要件 5.1・5.2・7.3。
- **Selected Approach**: イベントの出典は「イベント（UKADOC へのリンク）・説明の引用・この段で使う Reference の引用」の表にする。用語の説明は箇条書きにする。今の `prerequisites.md` にある `> **用語**:` の引用ブロックは使わない。
- **Rationale**: 要件 7.3 が台詞以外に許すのは、指示の一文・箇条書き・表・コードブロック・見出しだけである。入門の章では、引用ブロックは台詞部品だけに使う、とすれば見分けがつく。

### Decision: 段の章の見出しの型を固定し、機械検査はしない
- **Context**: 要件 3・8.2。
- **Selected Approach**: 段の章の本体は H2 を 5 つ、この順で持つ（叶えたいこと → 新しく覚える表現 → 辞書ファイルを足す → 起動して確かめる → もっと詳しく）。執筆規約の第 8 節に雛形を載せる。検査は足さない。
- **Rationale**: 要件 10 は既存の検査の追従だけを求めている。内部設計パートのような見出しの検査（`I-sections`）を足す要件は無い。13 枚は 1 つの spec で書くので、雛形と通し読みで足りる。

## Synthesis（設計の統合）
- **一般化**: 「段ごとの照合」（9.1〜9.5）と「抜き出しの照合」（9.10・9.11）は、同じ 1 つの規則の 2 つの向きである。辞書から章へ（辞書の全体と一致するブロックが章にある）と、章から辞書へ（章のどのブロックも辞書の全体か抜き出しである）。全体の一致は抜き出しの特別な場合なので、ブロックの側の判定は関数 1 つ（`isExcerptOf`）で済む。
- **作るか使うか**: 新しいツールは作らない。`listDicFiles`・`extractPastaBlocks`・`normalizeForCompare`・`matchDicFile` はそのまま使う。足すのは `listGuideChapters`・`isExcerptOf` の 2 つと、`runTutorialCheck` の中身の入れ替えだけである。CLI の名前と終了コードは変えない（`manual.yml` を触らずに済む）。
- **単純化**: 段階表（`STAGES.md`）を照合で読まない（`manual.yml` の起動条件に無いため）。章の数は導出しない。見出しの型・地の文の有無は機械検査しない。`verify-content.mjs` のフェンスの数え方は直さず、規約で避ける。

## Risks & Mitigations（設計フェーズで足した分）
- 章のファイルを足した時点で、章の数の自己テスト 3 本が落ちる — 切り替え（`first-ghost.md` の削除・目次・数字・照合の入れ替え）を 1 つのタスクにまとめ、全検査の合否は最後の通しの確認で判定する。
- 地の文の段落を置かない規則（7.1）と、台詞を読み飛ばしても追える規則（7.3）は機械検査されない — 第 8 節のチェックリストと、章ごとのレビューで見る。
- 開発用パレットの項目名・NAR 作成の手順は SSP の版で変わりうる — 章は UKADOC の該当ページへリンクし（12.3）、通しの確認で実機の表示と突き合わせる。
- 16 章の最初の台詞の語が互いに、または他の章の本文と重なると検索の検査が弱まる — 章ごとに固有の 4 文字以上の語で始め、`verify-search.mjs --self-test` で確かめる。
