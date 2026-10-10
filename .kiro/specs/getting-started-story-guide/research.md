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
