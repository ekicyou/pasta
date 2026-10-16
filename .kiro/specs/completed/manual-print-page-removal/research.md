# Research & Design Decisions

## Summary
- **Feature**: `manual-print-page-removal`
- **Discovery Scope**: Extension（既存のマニュアルのビルドと検査の取り下げと調整。軽い調査）
- **Key Findings**:
  - mdBook v0.5.4 は `[output.html.print] enable = false` で、`print.html` と章のページの印刷のボタン（`<a href="../print.html" title="Print this book">`）を作らない。
  - 同じ設定で、章のページの `<link rel="stylesheet" href="../css/print-*.css" media="print">` と `css/print-*.css` そのものも出力から消える。章の印刷で目次・メニュー・前後の矢印を隠していた規則が無くなるので、要件 1.3 を保つには新版のテーマ（`claudia.css` の `@media print`）に同じ働きの規則を持たせる必要がある。
  - mdBook の設定は環境変数 `MDBOOK_OUTPUT__HTML__PRINT__ENABLE=false` で上書きでき、旧版の固定のコミットの `book.toml` を変えずに `print.html` とボタンを止められる（手元の v0.5.4 で確かめた）。

## Research Log

### mdBook の `[output.html.print] enable = false` の出力
- **Context**: brief の案 A。何が消えるかを実物で確かめる。
- **Sources Consulted**: 手元の mdBook v0.5.4 で `book/` を 2 通り（既定・環境変数で印刷を止める）に生成し、`grammar/index.html` と出力の一覧を比べた（2026-10-16）。
- **Findings**:
  - 既定: 出力に `print.html`・`css/print-9e4910d8.css`。章のページに印刷のボタンの `<a href="../print.html" ...>` と、`media="print"` の `print-*.css` の `<link>`。
  - 印刷を止めた: `print.html`・`css/print-*.css` が無い。章のページの中に `print` の文字列が 1 つも無い（ボタンも `<link>` も無い）。
  - mdBook の `print.css` の中身は、`#mdbook-sidebar`・`#mdbook-menu-bar`・`.nav-chapters`・`.mobile-nav-chapters` を隠す、`#mdbook-page-wrapper.page-wrapper` の左の余白と `transform` を戻す、`#mdbook-content` の幅の上限と余白を外す、`.page` のスクロールを戻す、見出し・`pre`・`code` の途中で改ページしない、`pre`・`code` を折り返す、など。
- **Implications**: 章の印刷を今までどおりにするには、これらの働きをテーマが受け持つ。テーマはすでに `@media print`（`claudia.css` の 7 節）を持ち、`theme-test.mjs` が P 系の検査でその中身を確かめているので、同じ場所と同じ検査の形に足す。

### 旧版の生成で印刷を止める方法
- **Context**: 要件 2。旧版は固定のコミット `CLASSIC_REF`（6d4e868d）の `book/` をその版のツールで作る（`manual-claudia-theme` 要件 11）。
- **Sources Consulted**: `git show 6d4e868d:book/book.toml`（`[output.html.print]` の節は無い・Claudia テーマ以前の素の mdBook）。手元の v0.5.4 での環境変数の上書きの確かめ。
- **Findings**: `MDBOOK_OUTPUT__HTML__PRINT__ENABLE=false mdbook build <dir> -d <out>` で、旧版の原稿を変えずに `print.html` とボタンが作られない。旧版の章のページからも `print-*.css` が消えるので、旧版の章を刷ると目次とメニューも刷られる。
- **Implications**: 旧版は固定のコミットのまま触らないので、印刷用の規則は足せない。要件 2.3 で「章の印刷の見た目は保証しない」と決めた。CI の mdBook は v0.5.3。環境変数の上書きは mdBook の昔からの機能だが、PR の CI の旧版の段で「`print.html` が無いこと」を確かめるので、効かなければ CI が止まって分かる。

### `print.html` を前提にしているもの
- **Context**: 取り下げる範囲の確定。
- **Sources Consulted**: `grep -rn print` を `book/tools`・`book/theme`・`book/AUTHORING.md`・`.github/workflows/manual.yml`・`.kiro/steering`・`book/src` に対して実行（2026-10-16 の main 相当の作業ツリー）。
- **Findings**:
  - 取り下げ: `book/tools/print-media-refs.mjs`・`print-media-refs-test.mjs`、`manual.yml` の段「Fix print page media references」と先頭の説明 11・14・18 行、`AUTHORING.md` 472 行、`verify-static.mjs` の import（31 行）・`ensureBuilt` の呼び出し（210〜216 行）・説明（8・21・256〜257・303 行）・自己テスト (a2) の `classic/print.html`・(d) の `print.html`・(e)、本検査の `pages`（696〜713 行）、steering `tech.md` 201 行・`structure.md` 232 行。
  - 説明だけ直す: `book/book.toml` 15 行・`book/theme/claudia.css` 3 行と 7 節の `.content` の注（mdBook の `print.css` に触れている）・`book/theme/claudia.js` 3 行・`book/tools/talk/talk-html.mjs` 4 行。
  - 直さない: `talk-html-test.mjs` の (D)（自前のフィクスチャ）、`link-check-test.mjs` 1013・1097 行（`print.html` を読まないことの確かめ）、`theme-test.mjs` の `@media print` の検査（章の印刷の検査であり `print.html` とは無関係）。
  - `book/src` に `print.html` と印刷のボタンの案内は無い。
- **Implications**: 道具の削除と `verify-static.mjs` の直しは同じ変更に入れないと、import の失敗か検査の数の不一致で CI が落ちる。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 設定で止める（採用） | `book.toml` の `[output.html.print] enable = false`、旧版は環境変数で同じ設定 | mdBook の機能だけ。書き換えの段が要らない | 章の印刷用の CSS も消える → テーマに持たせる | brief の案 A |
| ボタンだけ隠す | テーマでボタンを隠し `print.html` は残す | 章の印刷用の CSS が残る | URL を知る人が開くと固まる。書き換えの段を持ち続ける | brief で不採用 |
| 生成後に消す | `print.html` を消し、章のボタンを書き換える | 印刷用の CSS が残る | 新しい書き換えの道具が要る。旧版は新版のツールを使わない決まりに反する | 不採用 |

## Design Decisions

### Decision: 章の印刷用の規則をテーマに持たせる
- **Context**: 印刷を止めると mdBook の `print.css` が章のページから消え、要件 1.3 が崩れる。
- **Alternatives Considered**:
  1. mdBook の `print.css` を `book/theme/` に写して `additional-css` に足す
  2. `claudia.css` の `@media print` に、同じ働きの規則を自分の言葉で書く
- **Selected Approach**: 2。テーマの 7 節（印刷の規則）に、目次・メニュー・前後の矢印を隠す、本文の枠の余白と幅の上限を外す、見出し・`pre`・`code` の途中で改ページしない、`pre`・`code` を折り返す、の規則を足す。`theme-test.mjs` に P-7（隠す 4 つの要素）と P-8（本文の枠を紙幅に広げる）を足す。
- **Rationale**: テーマの印刷の規則はもともと 7 節に集まっており、`theme-test` が中身を検査する形ができている。ファイルを増やさない。mdBook の CSS（MPL-2.0）を丸写ししない。
- **Trade-offs**: mdBook の版が上がって要素の id が変わると規則が効かなくなる → Revalidation Triggers に書く。リンクの色（`#4183c4`）は本文では `general.css` の `.content a:link`（`var(--links)`）が勝っていたので写さない。
- **Follow-up**: 実装の後に、新版の章のページの印刷のプレビューで、目次・メニュー・矢印が出ないことを目で確かめる。

### Decision: 古い URL は 404 のまま
- **Context**: `/pasta/print.html` を開く人への扱い。
- **Selected Approach**: 転送しない。転送は mdBook の `[output.html.redirect]` が転送用の `print.html` を出力に作るので、要件 1.1・4.3 と両立しない。
- **Trade-offs**: ブックマークしていた人は 404 のページ（mdBook の目次と検索がある）を見る。

### Decision: 「無いこと」の検査は `verify-static.mjs` の 1 関数
- **Context**: 要件 4.3・4.4。
- **Selected Approach**: `analyzePrint(root, pages)` を足し、出力の根の `print.html` の有無と、章のページの中の `print.html` へのリンクを返す。`analyze()` の壊れた参照の検査でもボタンは「行き先の無いリンク」として見つかるが、設定が戻って `print.html` も戻った場合は壊れた参照にならないので、専用の検査を置く。
- **Trade-offs**: 関数を 1 つ足す。旧版（`classic/`）は根の `print.html` だけを見ることで自然に外れる。

## Synthesis
- **Generalization**: 新版と旧版の「印刷用のページを作らない」は同じ mdBook の設定 1 つで、置き場所（`book.toml` か生成のときの環境変数か）だけが違う。
- **Build vs Adopt**: mdBook の設定と環境変数の上書きを使う。新しい道具は作らない。
- **Simplification**: 転送・生成後の書き換え・旧版用の印刷の CSS は持たない。`print-media-refs` の道具と段を丸ごと消す。

## Risks & Mitigations
- CI の mdBook v0.5.3 で環境変数の上書きが効かない — 旧版の段が「`print.html` が無いこと」を確かめて止まる。そのときは旧版の段で生成の後に `print.html` を消す形に切り替える（設計の見直し）。
- テーマの印刷の規則が mdBook の要素の id とずれる — `theme-test` の P-7・P-8 が規則の有無を、実装の後の目視が効き目を確かめる。mdBook の版を上げるときに見直す。

## References
- `.kiro/specs/completed/manual-print-media-refs/design.md` — Revalidation Triggers に「`enable = false` にしたとき本ツールが失敗する」
- `.kiro/specs/completed/manual-claudia-theme/` — 旧版の生成（要件 11）とテーマの印刷の規則（要件 3.10）

## Consulted Skills
- なし（マニュアルのビルドと CI の調整。DSL・Lua・ゴーストの知識は使わない）
