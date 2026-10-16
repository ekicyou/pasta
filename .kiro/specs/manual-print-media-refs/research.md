# ギャップ分析: manual-print-media-refs

- 実施日: 2026-10-16
- 基準: main `3071114a`（ワークツリー `claude/manual-print-media-refs-40acbf`）
- 確かめ方: ソースと検査ツールを読んだ。あわせて、手元の mdBook v0.5.4（CI は v0.5.3）で `book/` をスクラッチのフォルダへビルドし、`print.html` を見た。mdBook の書き換えの範囲は、別の小さな本（章を 1 段下に置き、生の HTML で `<img>`・`<video>`（`poster` 付き）・`<source>`・`<track>`・`<audio>` を書いたもの）で試した。`book/node_modules` はこのワークツリーに無いので、着色の段と CI の通しは動かしていない。

## 1. 今の状態

### 1.1 実際の出力で確かめたこと

| 確かめたこと | 結果 |
| ------------ | ---- |
| `print.html` の `<video>` の `src` | `media/dev-actions-demo.mp4` のまま（8693 行あたり）。出力の根に `media/` は無い |
| 同じ `<video>` の中の代わりのリンク | `<a href="debug/media/dev-actions-demo.mp4">` に書き換わっている（働く） |
| 動画の実体 | `debug/media/dev-actions-demo.mp4` だけ |
| mdBook が `print.html` で書き換える参照 | `<a href>`・`<img src>`（Markdown の画像を含む） |
| mdBook が書き換えない参照 | `<video>` の `src`・`poster`、`<source>` の `src`、`<track>` の `src`、`<audio>` の `src` |
| `print.html` の中の章の境目 | `<div style="break-before: page; page-break-before: always;"></div>` が 59 個（SUMMARY の章は 60）。章のパスを持つ印は無い。見出しの `id` にも章のパスは付かない |
| mdBook の HTML の正規化 | `controls` が `controls=""` になる（mdBook 0.5 は生の HTML を解析して出し直す） |

ブリーフの「章の境目が残っているか」（議題 1）は、「パスとしては残っていない。改ページの印だけが章の数より 1 つ少なく残る」が答えである。

### 1.2 関係するファイル

| ファイル | 行数 | 関係するところ |
| -------- | ---- | -------------- |
| `book/tools/verify-static.mjs` | 694 | 例外 247〜249 行（`print.html` のときだけ `<video>`・`<source>` の開きタグを消す）。参照の抽出 `extractRefs`（114〜122 行。二重引用符の `href`・`src` だけ。`poster` は拾わない）。`ensureBuilt` 177〜209 行（`mdbook build` → `talk-html` の `transformDir`）。自己テスト `runSelfTest` 383〜528 行（`print.html` の動画を扱う場合は無い）。`classic/` を飛ばす `listFiles` 96〜111 行 |
| `book/tools/talk/talk-html.mjs` | — | 出力を書き換える段のお手本。`transformDir(outDir, { write })` を export し、全部を検査してから書き込む（失敗時は何も書かない）。CLI は `node … book/book`、失敗で `exitCode = 1` |
| `book/tools/talk/talk-html-test.mjs` | — | 書き換えの自己テストのお手本。依存ゼロの `check()` の型、深さ 0・1・2 と `print.html`、2 回実行で不変、失敗時に書き込まない、を確かめている |
| `.github/workflows/manual.yml` | 206 | 台詞の変換 99〜100 行 → 索引 104〜105 行 → リンク検証 110〜111 行 → … → 出力の検査 140〜141 行（`--no-build --self-test`）→ ツール自己テスト 150〜158 行（`book/tools` の `*-test.mjs` を全部）→ 旧版 165〜179 行。先頭の説明 1〜18 行。`paths` 24〜39 行（`README.md`・`SOUL.md` が無い）。古いコメント 113〜117 行（`first-ghost.md`） |
| `book/AUTHORING.md` | — | 「手元での確かめ方」（461 行〜）の「CI と同じ順」の一覧（466〜474 行の 9 行） |
| `book/src/debug/dev-actions.md` | 163 | 19〜21 行の `<video>` と代わりの `<a href>`。直す必要は無い（章のページでは正しい） |
| `book/tools/link-check-test.mjs` | — | 節 I は CI で章のページの見出しの `id` を突き合わせる。`print.html` は読まない（1013 行・1097 行） |
| `.kiro/steering/tech.md` | — | 201 行に manual.yml のパイプラインの並びが書いてある。完了時の同期で 1 段足す |

### 1.3 決まりごと（合わせるもの）

- `book/tools` のツールは Node 標準ライブラリだけで書き、依存を足さない（`talk-html`・`verify-static` とも「依存ゼロ」）。
- 出力を書き換える段は、出力のフォルダを引数で受け、既定は `book/book`。全部を確かめてから書き込み、問題があれば何も書かずに失敗する（作成ツールは問題があれば止める、の原則）。
- 自己テストは `*-test.mjs` の名前にすれば、CI の段の定義を変えずに拾われる。
- 出力の検査は、旧版（`classic/`）を見ない。

## 2. 要件ごとの対応表

| 要件 | 既にあるもの | ギャップ |
| ---- | ------------ | -------- |
| 1.1〜1.4 `print.html` の参照を正しい相対パスへ | 無い | **Missing**: 書き換えの段そのもの |
| 1.5 章のページを変えない | mdBook の出力のまま | 無い（書き換えが `print.html` だけを触れば満たす） |
| 1.6 動画を 1 つだけ置く | 今そうなっている | 無い |
| 2.1〜2.3 対象だけを直す | — | **Missing**。前提 A1 の要素と属性だけを選ぶ処理 |
| 2.4 写し先が決まらない・無いときに失敗 | `talk-html` の「全部確かめてから書く」型 | **Missing / Unknown**: 章の場所の決め方（下の 3 節） |
| 2.5 出力・`print.html` が無いときに失敗 | `talk-html` の `transformDir` が出力フォルダの不在で例外 | `print.html` の不在の扱いは新しく足す |
| 2.6〜2.8 0 件で成功・冪等・件数の表示 | `talk-html` の型 | 新しく足す（型はある） |
| 2.9〜2.10 自己テスト | `talk-html-test.mjs` の型・CI の自動の拾い上げ | 新しく足す |
| 3.1〜3.2 例外を外す | 例外が 247〜249 行にある | **Constraint**: 例外を外すと、書き換えの前の出力では必ず落ちる。CI の段の並びと `ensureBuilt` を同時に直す必要がある |
| 3.3 `poster` の検査 | `extractRefs` は `href`・`src` だけ | **Missing**（前提 A1 を保つ場合だけ）。全ページの参照の抽出に効くので、今の出力で誤検出が出ないことを確かめる（今は `poster` を使うページが無い） |
| 3.4 自分でビルドするときの追従 | `ensureBuilt` が `talk-html` の `transformDir` を取り込んでいる | 同じ形で書き換えを呼ぶ。書き換えのツールは関数を export する必要がある |
| 3.5 自己テスト | `runSelfTest` が出力の写しを壊して確かめる型 | 「`print.html` の動画の `src` を壊すと検出」を足す。例外を外した後は、無傷の写しが合格すること（既存の (a)）が「書き換え済み」の確認にもなる |
| 3.6 `classic/` を見ない | `listFiles` が飛ばす。自己テスト (a2) は `classic/print.html` の `<video src="missing.mp4">` を報告しないことを確かめている | 無い（そのまま保つ） |
| 4.1〜4.3 CI の段 | 台詞の変換の後に段を足す場所がある | 新しく 1 段。旧版の段は旧版のツールだけを使うので触らない |
| 4.4 自己テストの段の順 | 今はビルドの後 | 無い（動かさない） |
| 4.5〜4.7 申し送りの直し | — | `paths` 2 か所、先頭の説明（4〜5 行・10〜17 行）、113〜117 行のコメント |
| 5.1〜5.2 `AUTHORING.md` の一覧 | 9 行の一覧 | 1 行足す（前提 A3） |
| 6.1〜6.3 既存の検査との両立 | — | 通しで確かめる。`node_modules` が要る段（着色）は CI か main 側で確かめる |

## 3. 実装の選択肢

### 3.1 章の場所の決め方（要件 2.4・前提 A4。設計の主な決定）

| 案 | 中身 | 長所 | 短所 |
| -- | ---- | ---- | ---- |
| 3.1-a 末尾のパスで探す | 参照（`media/dev-actions-demo.mp4`）と同じ末尾を持つファイルを出力から探し（`classic/` は除く）、1 つに決まるときだけ直す。brief の推奨 | 単純。`print.html` の構造に頼らない。今の 1 件は確実に決まる | 別の章のフォルダに同じ名前のファイルがあると、正しい書き方でも必ず失敗する。たまたま出力の根に同じパスのファイルがあると「もう正しい」と見なして別のファイルを指したままにする恐れ |
| 3.1-b 改ページと章の並びを対応させる | `print.html` を改ページの印で区切り、i 番目の区切りを SUMMARY の i 番目の章に対応させ、その章のフォルダから解決する | 同じ名前のファイルが別の章にあっても正しく直る。属する章が分かる | mdBook の改ページの出し方に頼る（区切りの数が章の数 − 1 と合わなければ失敗にする必要）。下書きの章・区切り線など SUMMARY の書き方による例外の確認が要る（Research Needed） |
| 3.1-c 章のページと突き合わせる | `print.html` の中の対象の要素と同じ要素（同じ属性の並び）を持つ章のページを出力から探し、そのページのフォルダから解決する | `print.html` の構造に頼らず、属する章が分かる | mdBook が属性を正規化する（`controls=""` など）ので、比べ方を決める必要。同じ要素を 2 つの章に書くと決まらない |
| 3.1-d 同じ区切りの中の書き換え済みのリンクから推す | 同じ `<video>` の中の `<a href>`（mdBook が書き換えた）から章のフォルダを推す | 今の書き方なら確実 | 代わりのリンクを書かない動画では使えない。書き方の約束を執筆者に課す |

**2026-10-16 の要件ディスカッションの決定**: 同じ名前のファイルが別の章にあっても、要素を書いた章のファイルへ正しく直す（前提 A4）。よって 3.1-a を単独で使う形は採らない。3.1-b・3.1-c（またはその組み合わせ）から設計で選ぶ。対象の範囲は前提 A1 のとおり `poster`・`<track>`・`<audio>` まで含める。

組み合わせ（3.1-b で決め、3.1-a で写し先の実在を確かめる、など）もある。どれでも「決まらなければ失敗」は満たせる。要件ディスカッションで、同じ名前のファイルが別の章にあるときに直すべきか（失敗でよいか）を決めると、案が絞れる。

### 3.2 段の形（前提 A3）

- **案 A（既存の段に組み込む）**: 台詞の変換（`talk-html.mjs`）の中で `print.html` の参照も直す。CI と `AUTHORING.md` の一覧は変わらない。台詞の変換の責務が増え、ブリーフの「着色・台詞の変換・索引の各ツールは境界の外」と食い違う。
- **案 B（新しい小さなツール）**: `book/tools/` に 1 本（例: `print-media/print-media-refs.mjs` と `print-media-refs-test.mjs`、または `book/tools/` 直下）。CI に 1 段、`AUTHORING.md` の一覧に 1 行、`verify-static.mjs` の `ensureBuilt` に 1 呼び出しを足す。ブリーフの推奨。`AUTHORING.md` は申し送りで空いた。
- **案 C（出力の検査の中で直す）**: 検査が書き換えも行う。検査が出力を変えることになり、`--no-build` の意味（既存の出力をそのまま検査）と食い違う。採らない方がよい。

### 3.3 次点の直し方（前提 A5）

動画を出力の根の `media/` にも写す案は、HTML を触らずに済むが、約 3.1 MB を二重に公開し、要件 1.6 と食い違う。写し先の名前がぶつかったときの扱いも別に要る。brief どおり書き換えを推す。

## 4. 規模とリスク

- **規模: S**（1〜2 日）。お手本（`talk-html`・`talk-html-test`）がそのまま使え、触るファイルは 5 つ（新しいツールとテスト、`verify-static.mjs`、`manual.yml`、`AUTHORING.md`）。ブリーフの見積もり 2〜3 タスクと合う。
- **リスク: 低**。触るのは `print.html` だけで、章のページ・リンク検証の節 I に当たらない。主なリスクは、3.1 の決め方が mdBook の出力の形に頼る場合に、mdBook の版の上げで黙って壊れること。決まらなければ失敗する作りにすれば、黙っては壊れない。

## 5. 設計への申し送り

- **推す形**: 案 B（新しい小さなツール）。関数を export して `verify-static.mjs` の `ensureBuilt` から呼ぶ（`talk-html` と同じ形）。書き込みは全部を確かめてから行う。
- **決めること**:
  1. 章の場所の決め方（3.1）。同じ名前のファイルが別の章にあるときの扱い。
  2. 対象の要素と属性（前提 A1）。`poster` を含めるなら、`verify-static.mjs` の参照の抽出にも `poster` を足す。
  3. 対象が 0 件のときの表示と終了コード（前提 A2）。
  4. `print.html` が無いときを失敗にするか（要件 2.5。mdBook は `[output.html.print] enable = false` で `print.html` を出さない。今の `book.toml` は既定のまま）。
- **Research Needed**:
  - 改ページの印の数が、SUMMARY の書き方（部の見出し・区切り線・下書きの章）でどう変わるか（3.1-b を採る場合）。
  - CI の mdBook v0.5.3 と手元の v0.5.4 で、`print.html` のメディア要素の扱いが同じか（v0.5.3 の出力は CI で確かめる）。
- **完了時の同期**: `.kiro/steering/tech.md` 201 行のパイプラインの並びに、新しい段を足す。

---

# 設計の調べ（2026-10-16・kiro-spec-design）

## Summary

- **Feature**: `manual-print-media-refs`
- **Discovery Scope**: Extension（既存のビルド後の段の並びに、小さなツールを 1 本足す）。light discovery で行った
- **Key Findings**:
  - `print.html` の `<main>` の中は、改ページの `<div>` で 60 個に区切れ、その i 番目は目次（`toc.html`）の i 番目の章のリンクと 1 対 1 に対応する（手元の v0.5.4 の出力で 60 章すべてを確かめた）
  - mdBook v0.5.3 の `print.rs` は v0.5.4 と同じ（違いはコメントの 1 語だけ）。改ページの印は「章と章の間に 1 つ」、書き換えるのは `a`・`img` の `href`・`src`・`xlink:href` だけ。CI と手元で出力の形は変わらない
  - 見出しの `id` は `print.html` で重ならないように付け直される（`アクター辞書` → `アクター辞書-1`）。要素の `id` 属性も同じく付け直されうるので、章のページとの突き合わせは、タグの全体ではなく、直す対象の属性の値だけで比べる

## Research Log

### print.html の章の区切りと目次の対応
- **Context**: 前提 A4（要素を書いた章の場所から直す）を満たすには、`print.html` の中の各要素がどの章のものかを知る必要がある
- **Sources Consulted**: 手元の mdBook v0.5.4 で `book/` をスクラッチへビルドした出力（`print.html`・`toc.html`・章のページ）。mdBook の `crates/mdbook-html/src/html/print.rs`（v0.5.3 のタグ）と、v0.5.3...v0.5.4 の差分（GitHub API）
- **Findings**:
  - `render_print_page` は、章の木を順に直列化し、2 章目からは前に `<div style="break-before: page; page-break-before: always;"></div>` を 1 つ足す。区切りは章の数 − 1 個（今は 59 個・60 章）
  - `toc.html` の `<a href="….html">` は 60 本で、SUMMARY の順（下書きの章・区切り線・部の見出しはリンクを持たない）。`print.html` の区切りで分けた i 番目の部分の最初の `<h1 id>` は、`toc.html` の i 番目の章のページの最初の `<h1 id>` と、`id` の付け直し 1 件（`grammar/actor-dictionary.html`）を除いて一致した
  - 章のページの `<main>` と、`print.html` の対応する部分とで、直す対象（`video` の `src`・`poster`、`audio`・`source`・`track` の `src`）の値の並びを比べると、60 章すべてで一致した（今は `debug/dev-actions.html` の 1 件だけ）
  - mdBook は、章に `<h1>` が無いとき `print.html` の側にだけ `<h1>` を足し、`id` 属性を重ならないように付け直す。メディア要素の `src`・`poster` は変えない
  - mdBook 0.5 は生の HTML を解析して出し直すので、属性の値は二重引用符で出る（`controls=""`）
  - v0.5.3 の `print.rs` と v0.5.4 の差分はコメントの「an string」→「a string」だけ。`toc.html` は v0.5.3 の CI でも出ている（出力の検査の「目次 (toc) が SUMMARY 全章へのリンクを含む」が CI で合格している）
- **Implications**: 区切りと目次の並びで章を決め（案 3.1-b）、章のページとの値の並びの一致で確かめる（案 3.1-c の軽い形）組み合わせが、同じ名前のファイルが別の章にあっても正しく直せ、mdBook の形が変わったら黙らずに止まる

### 既存の段の型
- **Context**: 書き換えの段は `talk-html.mjs` と同じ型にする（requirements の前提 A3）
- **Sources Consulted**: `book/tools/talk/talk-html.mjs`・`talk-html-test.mjs`・`verify-static.mjs`・`.github/workflows/manual.yml`・`book/AUTHORING.md`
- **Findings**: `transformDir(outDir, { write })` を export し、全ファイルを確かめてから書く。失敗は `Error` を投げ、CLI が `process.exitCode = 1`。自己テストは依存ゼロの `check()` と一時フォルダのフィクスチャ。`verify-static.mjs` は `talk-html` の関数を取り込み、`ensureBuilt` の中で呼ぶ
- **Implications**: 同じ形の関数 1 つ（`fixPrintMediaRefs`）と CLI を持つ 1 ファイルにする。下位のフォルダは作らない（`link-check.mjs`・`tutorial-check.mjs` と同じく `book/tools/` 直下）

### 出力の検査の影響
- **Findings**: 今の出力に `poster=`・`data-src=` などは 1 つも無い（スクラッチの出力で確かめた）。`extractRefs` に `poster` を足しても、今の出力で検査の結果は変わらない。例外（247〜249 行）を外すと、書き換えの前の出力では `print.html` の `media/dev-actions-demo.mp4` で必ず落ちる
- **Implications**: CI・`ensureBuilt`・`AUTHORING.md` の一覧の 3 か所に同時に書き換えを入れる

## Architecture Pattern Evaluation

| 案 | 中身 | 強み | 弱み | 判断 |
| -- | ---- | ---- | ---- | ---- |
| 3.1-b だけ | 区切りの数と目次の章の数が合えば、i 番目の部分を i 番目の章とする | 単純 | 数が偶然合ったまま並びがずれると、同じ名前の別の章のファイルを黙って指す（出力の検査も気づかない） | 単独では採らない |
| 3.1-c だけ | 対象の値が同じ章のページを探す | 構造に頼らない | 同じ名前の動画を 2 つの章に書くと決まらない（要件 2.10 の場合を落とす） | 採らない |
| 3.1-b ＋ 値の並びの照合 | 3.1-b で決め、その章のページの `<main>` の対象の値の並びと一致することを確かめる | 同じ名前でも正しく直る。並びのずれ・mdBook の形の変化で止まる。2 回目の実行も見分けられる | 章のページを 1 枚ずつ読む（60 枚・一瞬） | **採る** |

## Design Decisions

### Decision: 章の決め方
- **Context**: 要件 2.4・2.10、前提 A4
- **Alternatives Considered**: 上の表
- **Selected Approach**: `print.html` の `<main>` を改ページの印で分け、`toc.html` の章のリンクの並びと対応させる。数が違えば失敗。各部分の対象の値の並びが、対応する章のページの `<main>` の並び（そのままの値か、直した後の値）と一致しなければ失敗
- **Rationale**: 出力のフォルダだけを入力にでき（`talk-html` と同じ引数）、mdBook の出力の形の変化は数か並びの食い違いとして必ず表に出る
- **Trade-offs**: `toc.html` の形（`<a href="….html">`）に頼る。代わりに SUMMARY.md を読むと、ツールが `book/src` の場所も知る必要があり、フィクスチャも増える
- **Follow-up**: 章の並びの出どころ（`toc.html` か SUMMARY.md か）は設計ディスカッションで確かめる（design.md の未決事項 1）

### Decision: 2 回目の実行の扱い
- **Selected Approach**: 部分の値が、章のページの値そのもの（未修正）か、それを直した値（修正済み）なら一致とみなす。修正済みなら変えない
- **Rationale**: 要件 2.7 の冪等を、別の印を足さずに満たす。根の章では両者が同じなので区別は要らない

### Decision: 写し先の実在の確かめ
- **Selected Approach**: 直す・直さないにかかわらず、外部 URL・見出しだけ・根からの絶対パスを除く対象すべてで、直した後のパスのファイルが出力に在ることを確かめる（`classic/` は対象外にしない。写し先が `classic/` を指すことは無い）
- **Rationale**: 要件 2.4。出力の根から外へ出るパスも「無い」とする

### Decision: 0 件のとき
- **Selected Approach**: `print.html` に対象が 1 つも無ければ、`toc.html` や章のページを読まずに 0 件で成功する（前提 A2）。`print.html` そのものが無いのは失敗（要件 2.5）

### Synthesis
- **Generalization**: 対象の要素と属性を 1 つの表（`video: src・poster` ほか）にまとめ、書き換えと照合の両方がこの表だけを見る。要素を足すときは表に 1 行足すだけ
- **Build vs. Adopt**: HTML の構文解析器（parse5 など）は採らない。`book/tools` は依存ゼロの決まりがあり、mdBook 0.5 が属性を二重引用符に正規化して出すので、タグと属性の正規表現で足りる（`talk-html`・`verify-static` と同じ判断）
- **Simplification**: 純関数と出力フォルダの関数を分けない（`talk-html` は分けているが、本ツールは複数のファイルを突き合わせるので、1 ファイルだけの純関数に意味が薄い）。export は 1 つ。設定・オプションは `write` だけ

## Risks & Mitigations
- mdBook の版の上げで `print.html` の区切りの形・`toc.html` の形が変わる — 数か並びの食い違いで失敗し、CI が公開前に止まる（黙って壊れない）
- 章の本文に改ページの `<div>` と同じ文字列を生の HTML で書く — 区切りの数が合わなくなって失敗する（黙って壊れない）
- 属性の値に `>` を含む（正規表現がタグの終わりを誤る） — 今の書き方には無い。照合が食い違えば失敗する
- `extractRefs` の `\bposter` が `data-poster=` にも当たる — 既存の `data-src` と同じ性質。今の出力には無い

## References
- mdBook `crates/mdbook-html/src/html/print.rs`（v0.5.3）— 改ページの印と書き換えの範囲
- mdBook compare v0.5.3...v0.5.4 — `print.rs` はコメントだけの違い

## 使ったスキル・指針（設計のフェーズ）
- `kiro-spec-design`（rules: design-principles・design-discovery-light・design-synthesis・design-review-gate）
- `ponytail`（作り込みを避ける。依存を足さない・ファイルを増やさない）

## 設計の決定（2026-10-16・設計ディスカッション）

- 章の並びは、出力の `toc.html` の章のリンクから取る（推奨案で確定）。入力が出力のフォルダだけで済み、`print.html` と同じビルドの並びになるため
