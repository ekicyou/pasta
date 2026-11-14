# ギャップ分析: manual-download-page

> 2026-11-14・`claude/manual-download-page-a75baa`（main `0bada3dd` の上の `9af43abd`）で調べた。入力は `requirements.md`・`brief.md`・steering（`product.md`・`tech.md`・`structure.md`）・`book/AUTHORING.md`・`book/tools/**`・`.github/workflows/manual.yml`・`.github/workflows/release.yml`・`.claude/skills/release/procedure.md`。
> 分析であり決定ではない。決めることは末尾の「設計で決めること」に並べた。

## 分析の要約

- 変更の大半は Markdown（新しい `book/src/download.md`・`SUMMARY.md`・`introduction.md`・5 章・`README.md`）で済む。仕組みの新設は要らない。mdBook は目次の全章を検索の索引に入れるので、要件 1.4 は目次に載せるだけで満たせる。
- 足かせは「台詞の無い章」を許す口がどの検査にも無いこと。手元で台詞の無い `download.md` を目次に足して実際に走らせたところ、**4 つの検査が 5 件落ちた**（`verify-content` の `D-voice`・`T-intro`・`T-outro`、`verify-static` の `8.4/8.6`、`verify-search` の `8.2`）。要件 7.2 が挙げる `verify-content` だけでは足りない。
- 除外の置き場所の主な候補は 3 つ（混成の D は 3 節）。(a) 登録簿 `talk/talk.mjs` に「台詞を置かない章」の 1 つの一覧を置き、3 つのツールがそれを読む。(b) ツールごとに例外を書く。(c) ページに短い掛け合いを置いて検査を変えない（要件 3.6 と brief に反する）。
- リリース手順は `introduction.md` の `| 対象 pasta バージョン | **vX.Y.Z** |` の行を `grep -cF` で **ちょうど 1 か所** と数え、置き換える。行の形と 1 か所であることを残せば手順は変わらない。
- `GENERATION_MAP`（24 章）に、触る章（`introduction.md`・`download.md`・`getting-started/*`・`debug/*`）は 1 つも無い。生成対象章から、消す節へのリンクも無い。スキルの写しの再生成（要件 7.4）は、今の見込みでは要らない。

## 1. 今の状態

### 1.1 目次・扉・トップページ

| 対象 | 今の形 | 要件との差 |
|------|--------|------------|
| `book/src/SUMMARY.md` | `[はじめに](introduction.md)` の次がすぐ `# 入門 / チュートリアル` | 番号の無い `[ダウンロード](download.md)` を、はじめにの次（パートの見出しより前）に足す。mdBook 0.5.4 の前付けの章の扱いのとおりで、仕組みの変更は要らない（試しのビルドで `toc` に入り、`verify-static` の目次の検査も通った） |
| `introduction.md` L14-24 `<nav class="hero-toc">` | 7 項目 | 先頭に `- [ダウンロード <small>Download</small>](download.md)` を足す。`claudia.css` の格子は `repeat(auto-fit, minmax(min(150px, 100%), 1fr))` で、8 項目でも CSS は変えなくてよい。`theme-test.mjs` が見るのはクラスの契約と 420px 以下で 1 列になることだけで、項目の数は見ない。`talk-test.mjs` の扉は自前の写しで、本物の扉には縛られない |
| `introduction.md` L34-39「関連リンク」 | 2 行の表。2 行目のセルに配布物 7 つ | 2 行目を消す。1 行目（GitHub リポジトリと README）は残す（要件 4.4） |
| L41-65「AI にゴーストを書かせるとき」 | スキル 3 つの表・展開の木・版の注意 | ダウンロードのページの `pasta-skills.zip` の説明へ、欠けなく移す（要件 3.5） |
| L67-83「このマニュアルが対象とするバージョン」 | 版の表と LuaJIT 2.1 の説明 | 残す（要件 4.5・4.6） |
| L85-95「安定機能と『将来変更あり』の区別」 | 注記の説明 | 残す（要件 4.6） |
| L97-109「このマニュアルの歩き方」 | 4 つの入口のリストと内部設計パートの注意。**L108-109 に権威の一文**（「利用者向けの権威的な記述は本マニュアル…本マニュアルを正とする」） | 節は消す（要件 4.3）。権威の一文は別の場所へ移す（要件 4.7・OPEN QUESTION 7）。この一文を見る検査は無い |
| L111-119 締めの掛け合いとクレジット | 区切り `---` の後ろ | 残す。`chapterRegions` は最初と最後の `---` で導入・締めを切るので、L28 と L111 の `---` の 2 本は残す |

消すもの（「関連リンク」の配布物のセル・「AI にゴーストを書かせるとき」・「このマニュアルの歩き方」）を見出しのアンカーで指すリンクは、`book/src`・ルートと各クレートの README・`doc/SOUL.md`・3 スキルのどこにも無い（grep で確かめた）。要件 4.8 の付け替えは、今は 0 件である。

### 1.2 入手の説明がある場所（今）

| 場所 | 今の書き方 | 要件 |
|------|------------|------|
| `getting-started/prerequisites.md` L35-42 | 掛け合い（L37「リリースページの `hello-pasta.nar` をお取りなさいまし」）＋箇条書き 4 行。`hello-pasta.nar` と `pasta.dll.zip` の固定リンク、過去の版のリリースページ | 5.1。箇条書きのリンクをダウンロードのページへ。L37 の台詞の「リリースページ」をどうするかは決める（下の 6） |
| `getting-started/setup.md` L148 | 手順 1「`hello-pasta.nar`（最新版）をダウンロードする」に固定リンク | 5.1。手順の流れは残す |
| `debug/recipe.md` L15-24「入手と置き方」 | 固定リンク 2 つ・Windows の種類の見分け方・対応する Windows・中身の表・置き方 | 5.2。リンクだけを寄せ、見分け方などは残す。要件 3.3 の「見分け方の節」への行き先はこの節（`debug/recipe.md#入手と置き方`） |
| `debug/validate.md` L18-38「使い方」 | 固定リンク・展開・中身・呼び方・`cargo install` | 5.3 |
| `debug/vscode-setup.md` L25-39 | Marketplace の手順（URL は無く、拡張機能ビューで検索）・VSIX の名前の形 `pasta-vscode-<version>.vsix`（リンクは無い） | 5.4。VSIX の入手先にダウンロードのページへのリンクを添える |
| `README.md` L33-48 | マニュアルのパートの表（5 行）＋配布物 6 行の一覧と Marketplace へのリンク | 要件 6 |

ほかに入手の説明を持つが、要件の範囲の外にあるもの（触るかは決める。下の 6）:

- `crates/pasta_check/README.md` L11-13（`pasta_check.zip` の固定リンク。crates.io にも出る）
- `.claude/skills/pasta-check/SKILL.md` L14（同じ。`pasta-skills.zip` で配るスキル。OPEN QUESTION 6）

### 1.3 配布物の名前とリンク

`release.yml` L207-217 の添付は次の 7 つ。

| 配布物 | 添付の名前 | 固定のリンク |
|--------|------------|--------------|
| SHIORI | `pasta.dll.zip` | `https://github.com/ekicyou/pasta/releases/latest/download/pasta.dll.zip` |
| サンプルゴースト | `hello-pasta.nar` | `…/latest/download/hello-pasta.nar` |
| 検査の CLI | `pasta_check.zip` | `…/latest/download/pasta_check.zip` |
| スキル | `pasta-skills.zip` | `…/latest/download/pasta-skills.zip` |
| pasta-recipe | `pasta-recipe.x64.zip`・`pasta-recipe.arm64.zip` | `…/latest/download/pasta-recipe.x64.zip` ほか |
| VSCode 拡張 | `pasta-vscode-{版}.vsix` | **無い**（版が名前に入る）。最新のリリースのページ `https://github.com/ekicyou/pasta/releases/latest` か一覧 `…/releases` で案内する（要件 2.5） |

Marketplace のページは `https://marketplace.visualstudio.com/items?itemName=ekicyou.pasta-vscode`（README L48 で使っている。拡張の識別子は `release.yml` L424 の `ekicyou.pasta-vscode`）。

`link-check.mjs` が照合する GitHub の URL は `blob`・`tree` だけで、`releases/…` と Marketplace の URL はネットにつながないので確かめない。リンクの綴りの誤りは検査で止まらない（人のレビューか、設計で検査を足すかを決める）。

### 1.4 リリース手順とトップページ

`.claude/skills/release/procedure.md`:

- L196: `grep -cF '| 対象 pasta バージョン | **v{V_OLD}** |' book/src/introduction.md`（数が 1 であることを見る）
- L204: その行を `**v{V}**` に置き換える（1 か所）
- L251: `git diff --numstat` が `1 1 book/src/introduction.md` であること
- L244・L254: `verify-content.mjs` が通ること

`verify-content.mjs` の `F-version`（L415）は、行頭から行末までが `| 対象 pasta バージョン | **v…** |` の行を正規表現で取る。表の 1 列目の文言・太字・区切りの空白を変えると、リリース手順と F の両方が外れる。要件 4.5 のとおり、行を手で触らなければ手順は変わらない。ダウンロードのページに版の番号を書くと、リリースのたびに書き換えが要る（手順に無い）。固定のリンクで済ませ、ページに版の番号を書かない形にすれば、手順を変えずに済む。

### 1.5 検査がダウンロードのページに求めるもの（試しの実行の結果）

台詞も `---` も無い試しの `download.md` を作り、目次に足して全部を走らせた（終わった後で元に戻し、作業ツリーはきれいなまま）。

| 検査 | 結果 | 理由 |
|------|------|------|
| `verify-content.mjs` `D-voice:book/src/download.md` | **FAIL** | `VOICE_DIRS` に `''`（`book/src` 直下）があり、全章の散文に令嬢の口調を求める（L322・L337-348） |
| `verify-content.mjs` `T-intro:download.md`・`T-outro:download.md` | **FAIL** | `chapterRegions`（`gen-skill-refs.mjs` L121）が `---` の 2 本を要り、無いと章構造の不備になる。`---` を足しても、`checkRegion` が導入・締めに両方の話し手を求める（`missing-speaker`） |
| `verify-content.mjs` `T-syntax:download.md` | PASS | 台詞が無ければ記法の不正も無い |
| `verify-static.mjs` `8.4/8.6: 全章に両方の話し手の台詞部品がある` | **FAIL** | `analyzeTheme` が目次の全章の HTML に `class="talk talk-{id} "` を求める（L343） |
| `verify-search.mjs` `8.2: download.md の最初の台詞の語で章がヒットする` | **FAIL** | 目次の全章で最初の台詞の語を取り、無い章を失敗にする（L236-241） |
| `link-check.mjs` | PASS | 章の実在と見出しを見るだけ |
| `gen-skill-refs.mjs --check`・`tutorial-check.mjs` | PASS | 対象外 |
| `talk/talk.mjs --stats` と `talk-test.mjs` J-9 | 影響なし | `--stats` は構造の無い章を数えないだけ。J-9 は目次の章の数と行の数が等しいことを見る（どちらも 1 増える） |
| `verify-static.mjs` の目次・全章の生成・テーマ資材・メニュー id | PASS | 前付けの章として普通に出る |

今の状態（新しいページの無い状態）では、上の検査はどれも通る（`verify-content` 394 件すべて PASS・`link-check`・`gen-skill-refs --check`・`tutorial-check`・`verify-static --no-build`（27 件）・`verify-search --no-build`（131 件）がどれも OK）。試しの状態では `verify-content` が 398 件（ページの分の 4 件が増え、そのうち 3 件が FAIL）になった。`book/node_modules` はこのワークツリーにあり、着色の段も走った。ツールの自己テスト（`*-test.mjs`）は走らせていない。

除外するとき、自己テストへの響き:

- `verify-static` の自己テストは、目次の全章を `pages` にして `grammar/index.html` を壊す。除外をページの一覧の側で行えば、自己テストの期待は変わらない。
- `verify-search` の自己テストは自前の写しを使い、目次に縛られない。
- `verify-content` には自己テストが無い。`verify-scripts-test.mjs` は子プロセスで走らせて exit 0・多数の検査・`RESULT: OK` を見るだけのスモークなので、除外を入れても期待は変わらない。

### 1.6 見出しのアンカー（要件 3.7）

mdBook 0.5.4 と `link-check.mjs` の `headingSlug` は、記号を落とす。ファイル名をそのまま見出しにすると次のアンカーになる（試しのビルドの `id` と `headingSlug` の結果が一致した）。

| 見出しの例 | アンカー |
|------------|----------|
| `` ## `pasta.dll.zip` `` | `#pastadllzip` |
| `## hello-pasta.nar` | `#hello-pastanar` |
| `` ## `pasta-skills.zip` `` | `#pasta-skillszip` |
| `## pasta-recipe（x64・arm64）` | `#pasta-recipex64arm64` |
| `## VS Code の拡張（VSIX）` | `#vs-code-の拡張vsix` |

安定はしているが読みにくい。`link-check.mjs` は `<a id="…">` の明示のアンカーも照合する（L10-15）ので、見出しに読みやすい id を添える手もある。ほかの章と README がこのアンカーを指すので、名前は設計で 1 度決めて変えない。

### 1.7 執筆規約（`book/AUTHORING.md`）

- 冒頭のコメント（L4）と第 1 節「鉄則」（L45-49）が、全章を「掛け合いの導入 → 本体 → 掛け合いの締め」とする。例外として書いてあるのは入門ガイド（本体も台詞）と表紙（扉の中にも台詞）だけ。
- 第 7 節「置ける場所」（L412-421）の表に、台詞を置かない章の行は無い。要件 7.3 はここに書き足す。
- 第 7 節「検査と表情の集計」（L431-441）の表は、`D-voice`・`T-intro`・`T-outro`・`verify-static`・`verify-search` を並べている。除外を入れるなら、この表の説明も合わせる。
- 第 5 節のチェックリストと第 8 節「章を足す・名前を変えるとき」（L704-）は入門ガイドの章の決まりで、`download.md` は対象外。入門ガイドの章のリンクを書き換えても、第 8 節の `C-prose`（台詞以外の段落は指示の一文だけ）を守ればよい。箇条書きの中のリンクの差し替えは `C-prose` に当たらない。

## 2. 要件と資産の対応

| 要件 | 使える資産 | ギャップ |
|------|------------|----------|
| 1.1-1.3 目次と扉 | `SUMMARY.md`・`hero-toc`・`claudia.css` | Markdown の追記だけ。仕組みの不足なし |
| 1.4 検索 | mdBook の索引・`bigram-index` | 不足なし。ただし `verify-search` 8.2 が台詞の語で章を探すので、この章の確かめ方を別に決める（**制約**） |
| 2.1-2.4 表と固定リンク | 1.3 の名前と URL | 不足なし |
| 2.5 VSIX | `releases/latest` | 固定のリンクが無い（**制約**。OPEN QUESTION 3） |
| 3.1-3.4 ファイルごとの説明 | 各章の既存の文（中身・置き方） | 文の写しと要約。どこまで重ねるかは決める（要件 5 との持ち場の線引き） |
| 3.5 スキルの節の移設 | `introduction.md` L41-65 | 移すだけ |
| 3.6 台詞を置かない | — | 検査の 4 か所が全章に台詞を求める（**不足**。1.5） |
| 3.7 安定したアンカー | `headingSlug`・`<a id>` の照合 | 名前の決めだけ（1.6） |
| 4.1-4.6 トップページ | `introduction.md` の今の節 | 削るだけ。`F-*` の 3 つとリリースの行は残る |
| 4.7 権威の一文 | `introduction.md` L108-109 | 消す節の中にあるので、置き場所を決める（**未定**。OPEN QUESTION 7） |
| 4.8 アンカーの付け替え | `link-check.mjs` | 今は 0 件（1.1） |
| 5.1-5.5 各章 | 1.2 の各章 | リンクの差し替え。`prerequisites.md` の台詞と箇条書きの持ち方は決める |
| 6.1-6.3 README | `README.md` L33-48・`link-check` の公開 URL の照合 | 不足なし。`https://ekicyou.github.io/pasta/download.html` は `link-check` が章と見出しへ写して照合する |
| 7.1 CI 全部 | `manual.yml` の全段 | 1.5 の 5 件を除けば通る見込み |
| 7.2 除外 | — | **不足**。対象は `verify-content` だけでなく `verify-static`・`verify-search` も |
| 7.3 規約 | `AUTHORING.md` 第 7 節 | 書き足し |
| 7.4 スキルの写し | `GENERATION_MAP` | 対象の章が無く、今は再生成不要 |
| 7.5 アンカーの不備で止める | `link-check.mjs` | 既にある |

## 3. 実装の進め方の候補

### 候補 A: 既存の章と検査の延長（台詞を置かない章の一覧を 1 つ足す）

- 新しいページ 1 枚と、Markdown の書き換え（`SUMMARY.md`・`introduction.md`・5 章・`README.md`・`AUTHORING.md`）。
- 検査は、登録簿 `book/tools/talk/talk.mjs`（`SPEAKERS` を持ち、3 つのツールがすでに import している）に「台詞を置かない章」の一覧（今は `download.md` の 1 つ）を置き、次の 4 か所がそれを読む。
  - `verify-content.mjs` `D-voice`: 一覧の章を外す（あるいは逆に「口調が無い」ことを求める）
  - `verify-content.mjs` `T-intro`・`T-outro`: 一覧の章は導入・締めを求めず、代わりに「台詞が 1 つも無い」ことを求める（`scanTalk(md).blocks.length === 0`）。`T-syntax` は今のまま全章に掛ける
  - `verify-static.mjs` `8.4/8.6`: `analyzeTheme` に渡す `pages` から一覧の章を外す
  - `verify-search.mjs` `8.2`: 一覧の章は台詞の語の代わりに H1（または表の語）で章がヒットすることを確かめる（要件 1.4 の確かめにもなる）
- ✅ 例外の定義が 1 か所。ほかの章への要求は変わらない（要件 7.2 の「ほかの章は今のまま」）
- ✅ 「台詞が無いこと」まで検査で守れる（要件 3.6 を機械で確かめられる）
- ❌ 3 つのツールに数行ずつ手が入る。一覧が将来ほかの章の逃げ道に使われ得る（規約で「ダウンロードのページだけ」と書いて抑える）

### 候補 B: ツールごとに個別の例外を書く

- 各ツールに `'download.md'` の名指しの条件を直書きする。
- ✅ 1 つ 1 つの差分は候補 A より小さい
- ❌ 同じ例外が 4 か所に散り、将来どれかを直し忘れる。規約（`AUTHORING.md`）とも 1 対 1 で結びにくい

### 候補 C: ページに短い掛け合いを置き、検査を変えない

- 導入と締めに 1〜2 行ずつの掛け合いを置き、`---` で本体を挟む。
- ✅ ツールの変更が 0。規約の「全章が掛け合いで始まり、掛け合いで終わる」が崩れない
- ❌ 要件 3.6 と brief の決定（「Claudia の掛け合いは入れない。簡潔さを優先」）に反する。要件の側を直すことになる

### 候補 D（混成）: 候補 A に、表紙の `cover` と同じ型の「台詞を置かない章」の扱いを `checkRegion` の側に持たせる

- `checkRegion` の `opts` に台詞を置かない章の扱いを足し、`T-intro`・`T-outro` の判定を 1 か所に寄せる。`verify-static`・`verify-search` は候補 A と同じ。
- ✅ 表紙の例外（`cover`）と並ぶ形で読みやすい
- ❌ `talk.mjs` の判定関数の意味が広がる。`talk-test.mjs` に自己テストを足すことになる

## 4. 手間と危険

- **手間: S（1〜3 日）**。中身は Markdown の書き換えとページ 1 枚。検査の除外は 3 つのツールに数行ずつ。仕組みの新設は無い。
- **危険: 低**。どれも既にある型（表紙の `cover` の例外・目次から章を数える関数・登録簿）の延長。読む人に届く公開物はマニュアルと README だけで、リリースの手順と配布物には触れない。気をつけるのは、(1) リリースの行の形を変えないこと、(2) 固定のリンクの綴り（検査で止まらない）、(3) 消す節の権威の一文を落とさないこと。

## 5. 設計へ持ち越す調べもの

- 台詞を置かない章の `verify-search` の確かめ方（H1「ダウンロード」の 2 文字は bigram で 1 語。ほかのページにも多く出るので、ヒットに章が含まれるかで見る。表の固有の語を使う手もある）。
- 見出しのアンカーを読みやすくするなら、`<a id>` の明示のアンカーを mdBook 0.5.4 の出力がそのまま残すか（`link-check-test.mjs` 節 I の型で確かめる）。
- `https://github.com/ekicyou/pasta/releases/latest`（VSIX の案内先）がリリースの Assets を開いた状態で見えるか。

## 6. 設計で決めること（要件ディスカッションの議題の材料）

1. **台詞を置かない章の除外の方式**（OPEN QUESTION 1）: 候補 A（登録簿に一覧を 1 つ）・B（ツールごとに直書き）・C（掛け合いを置いて検査を変えない）・D（`checkRegion` に寄せる）。あわせて、要件 7.2 の対象が `verify-content` だけでなく `verify-static` の `8.4/8.6` と `verify-search` の `8.2` にも及ぶことを要件に書き足すか。
2. **除外した章に「台詞が無いこと」を求めるか**: 外すだけにするか、要件 3.6 を機械で守るために「台詞が 1 つも無い」「口調が無い」を求める検査に置き換えるか。
3. **要件 1.4（検索）の確かめ方**: `verify-search` の `8.2` の代わりに、H1 や表の語でこの章がヒットすることを確かめるか、mdBook の既定に任せて確かめないか。
4. **VSIX の案内先**（OPEN QUESTION 3）: 最新のリリースのページ `releases/latest`、一覧 `releases`、または Marketplace だけ（VSIX の手動の導入は `vscode-setup.md` に任せる）。
5. **各配布物の見出しとアンカー**（要件 3.7）: ファイル名の見出し（`#pastadllzip` のような読みにくいアンカー）か、目的の見出し（「サンプルゴースト（hello-pasta.nar）」）か、見出しに `<a id="pasta-dll-zip">` を添えるか。
6. **ダウンロードのページと各章の説明の重なり**（要件 3.1 と 5.2-5.4）: ページの短い説明に、中身の表・展開の手順・SmartScreen の注意まで写すか、1〜2 行にとどめて各章へ送るか。写すほど、配布物が変わったときの直す場所が増える。
7. **権威の一文の置き場所**（OPEN QUESTION 7）: トップページの冒頭の説明（2〜3 行）に入れるか、「将来変更あり」の節の後ろに 1 行で残すか。要件 4.1 の「2〜3 行の説明」に収まるか。
8. **「このマニュアルの歩き方」にだけある情報**: 「内部設計パートはゴーストを作るだけなら読まなくてよい」の一文と、外部リンク集への入口。扉の案内の項目の英字の添え書きで足りるとみて消すか、どこかへ残すか。
9. **入門ガイドの台詞の扱い**（OPEN QUESTION 5）: `prerequisites.md` L37 の Claudia の台詞「リリースページの `hello-pasta.nar` をお取りなさいまし」を、ダウンロードのページを指す言い方に直すか、台詞は残して箇条書きのリンクだけを差し替えるか。箇条書きの「過去の版はリリースページ」の一文も、ダウンロードのページへ寄せるか。
10. **README の形**（OPEN QUESTION 2）: 配布物の一覧を消してリンク 2 つにするか（要件 6.2 の仮定）、GitHub で最初に読まれることを重く見て、一覧を短く残すか。README のパートの表のダウンロードの行の位置（先頭か）。
11. **範囲の外の入手の説明**（OPEN QUESTION 6 の周り）: `crates/pasta_check/README.md` と `.claude/skills/pasta-check/SKILL.md` の `pasta_check.zip` の説明を、このページを指すように直すか、今のまま置くか（スキルは `pasta-skills.zip` で配るので、配った先でもマニュアルの URL で読める。`.claude/skills/` の書き換えは分類器が止めることがある）。
12. **固定のリンクの綴りを検査で守るか**: `releases/latest/download/<名前>` の名前を `release.yml` の添付の一覧と照らす検査を足すか、人のレビューに任せるか（今の `link-check` はネットにつながず、この URL を見ない）。足すと、配布物を増やす spec（`pasta-dll-multiarch` など）がこのページを直し忘れたときに止まる。
13. **AUTHORING の書き足しの範囲**（要件 7.3）: 第 7 節「置ける場所」の表への 1 行に加えて、冒頭のコメント・第 1 節「鉄則」・第 7 節「検査と表情の集計」の表の説明も合わせるか。→ 要件ディスカッションで、規約の全章向けの記述も例外と合わせると要件 7.3 に書き足した（自明な修正）。
14. **トップページに残す「関連リンク」の形**（要件 4.4・OPEN QUESTION 8）: 配布物のセルを消すと 1 行になる表をそのまま残すか、一文にするか。要件はリンクを残すことだけを求める。

> 要件ディスカッションでの振り分け: 1 の方式（A・B・D）、5 の見出しとアンカーの名前、3 の確かめ方の細部、14 は設計フェーズ（`/kiro-design`）で決める。1 で外した検査の代わりに何を確かめるか、2・4・6〜12 は要件ディスカッションの議案にした。

## 次の段

要件ディスカッション（`/kiro-requirements-discussion manual-download-page`）で上の議題を決め、そのあと `/kiro-design manual-download-page` で設計へ進む。
