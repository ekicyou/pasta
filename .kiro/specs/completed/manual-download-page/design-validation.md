# 設計レビュー: manual-download-page

- 対象: `.kiro/specs/completed/manual-download-page/design.md`（requirements.md・research.md・steering と照合）
- 実施日: 2026-11-14
- 方法: kiro-validate-design（非対話）。設計の主張は `book/tools/` の実コード・`.github/workflows/release.yml`・`manual.yml`・`book/src/` の該当章・`book/AUTHORING.md` で確かめた。

## 設計レビューの要約

設計は既存の章と検査の延長（ギャップ分析の候補 A）で、要件 1〜7 の全項目に持ち場と確かめ方が付いている。新しい依存やツールのファイルは作らず、例外の定義を `talk.mjs` に 1 つ置いて 3 つの検査が読む形も、表紙の `cover` の例外と同じ型で、要件に対して大きすぎない。ただし「自己テストへの響きは無い」という主張が実コードと合わず、このままでは CI の自己テストが 2 本落ちて公開が止まる。これは小さい直しで済むので、直す前提で GO とする。

## 確かめた主張（実コードと一致）

- 依存の向き: `talk/talk.mjs` は `../link-check.mjs` を import している（L26）。`link-check.mjs` から `talk.mjs` を読まない設計は正しい。
- `verify-content.mjs`: `D-voice` は `VOICE_DIRS` の先頭 `''`（`book/src` 直下）を走査するので `download.md` に当たる。T 系のループは `ch = rel.slice(SRC.length + 1)` で `download.md` になり、`NO_TALK_CHAPTERS` のキーと合う。`chapterRegions` は区切りが 2 本未満で投げるので、一覧の章では呼ばない分岐が要る（設計どおり）。
- `verify-static.mjs` L715 の `8.4/8.6` は `missingTalk.length === 0`。自己テスト（L453〜）は `grammar/index.html` を壊して `find` で見るので、本検査側の除外の影響を受けない。
- `verify-search.mjs` の `8.2` は `summaryChapters` の全章で `talkQuery`（最初の台詞）を使う。台詞の無い章は失敗になるので、語の差し替えが要る（設計どおり）。「やりたいこと」は今の `book/src` に 0 件。
- `link-check.mjs`: `headingSlug` で `## SHIORI の本体` → `shiori-の本体`、`## 検査の CLI` → `検査の-cli`、`## VS Code を使わずに書くアプリ` → `vs-code-を使わずに書くアプリ` になり、設計のアンカーの表と一致する。`releases/…` の URL は今は照合していない（設計の現状分析どおり）。`readme-manual-url` は `download.html` → `book/src/download.md` に写す。
- `release.yml` の「Upload release-assets」の `path:` は 1 行 1 つの `release/<名前>` が 7 行で、VSIX の行だけ `${{ … }}` を含む（設計の読み取り規則どおり）。
- `manual.yml` の `paths` に `release.yml` は無い（足す提案は妥当）。CI の「Book tools self-tests」は `book/tools` の全 `*-test.mjs` を走らせる。
- 扉の格子は `grid-template-columns: repeat(auto-fit, …)` なので 8 項目でも CSS の変更は要らない。
- `introduction.md` の行番号（説明 L30-32・スキルの節 L41-65・リリースページの文 L76-77・権威の 2 行 L108-109）、5 章の差し替え箇所の行番号、README の L33-48 は、すべて現物と一致する。

## 重大な指摘（≤3）

🔴 **重大な指摘 1**: 台詞を置かない章を目次に足すと、CI の自己テスト 2 本が落ちる（設計はこれを見落としている）
**懸念**: 設計の NoTalkExemption の Implementation Notes は「自己テストへの響きは無い（…`verify-scripts-test` は件数を厳密に見ない）」と書くが、実コードは次の 2 か所で件数・構造を厳密に見ている。
- `book/tools/verify-scripts-test.mjs` L82-85: `T-intro`・`T-outro` の実行件数が `summaryChapters().length`（目次の全章数）と一致することを求める。設計どおり一覧の章で `T-intro`・`T-outro` を呼ばないと、件数が 1 つ減って落ちる。
- `book/tools/gen-skill-refs-test.mjs` L596-612（K-10 の後のループ）: `book/src` の全章（`SUMMARY.md` を除く）で `chapterRegions` が投げず、本文の範囲が `legacyBodyRange` と一致することを求める。`download.md` は `---` を置かない設計なので `bad-structure` で投げ、「K download.md の本文の範囲が既存の判定と一致」が落ちる。
（`talk/talk-test.mjs` の J-9 は `--stats` が構造の無い章を `try/catch` で飛ばしつつ行は出すので、影響しない。）
**影響**: CI の「Book tools self-tests」が失敗し、`needs: build` で公開が止まる。要件 7.1（すべての検査を通る）を満たせない。
**提案**: File Structure Plan と NoTalkExemption に次の 2 ファイルを足す。(1) `verify-scripts-test.mjs`: `T-intro`・`T-outro` の期待件数を「目次の章 − `NO_TALK_CHAPTERS` の章」にし、`T-notalk` を「`NO_TALK_CHAPTERS` の件数」だけ実行していることを確かめる行を足す（`T-syntax` は全章のまま）。(2) `gen-skill-refs-test.mjs`: 全章のループから `NO_TALK_CHAPTERS` の章を外す（`talk.mjs` から import。`gen-skill-refs.mjs` は既に `talk.mjs` を読んでいるので依存の向きは変わらない）。`download.md` に空の導入・締めの `---` を 2 本置いて逃げる案は、3.6 の「区切りを置かない」と、空の導入で `T-intro` が別に落ちることから採らない。あわせて、設計の「Validation」の記述を実態に合わせて直す。
**トレーサビリティ**: 7.1, 7.2（7.6 の `T-notalk` の件数の確かめを含む）
**根拠**: design.md「NoTalkExemption」の Implementation Notes（Validation の行）・「File Structure Plan / Modified Files」・「Testing Strategy」

## 軽微な指摘（議題にしなくてよいもの）

- `release-asset` の名前の取り出し `[\w.-]*` は、裸の URL の直後に半角のピリオドが来ると（例 `…/pasta_check.zip.`）ピリオドまで名前に含めて誤検出する。今の `book/src` の固定リンクは `<…>`・リンク記法・全角の句読点で終わるので実害は無いが、`checkReadmeManualLinks` と同じく末尾の `[.,;:!?]+` を落とす 1 行を入れておくと安全。実装時の判断でよい。
- `verify-search` の `8.2` の検索は elasticlunr の OR（bigram）なので、「やりたいこと」が固有でなくても章はヒットする。設計も「固有であることは検査しない」と明記しており、要件 1.4・7.7（ヒットすること）には足りる。

## 設計の良い点

- 例外を `NO_TALK_CHAPTERS` の 1 か所に置き、外した要求ごとに代わりの要求（`T-notalk`・固有の語での検索）を対にして課している。`T-syntax` を外さないことで「台詞が 1 つでも置かれたら止まる」（7.6）を二重に守り、逃げ道の乱用は規約（AUTHORING 第 7 節）で縛っている。要件 7.2・7.6・7.7 の意図をそのまま機械に落としている。
- 見出し（アンカー）を配布物のファイル名でなく役目の日本語にし、名前と見出しを分けて持つ。`pasta-dll-multiarch` などで配布物の名前が変わってもほかの章のリンクが壊れず、古い名前の固定リンクは `release-asset` が止める。`manual.yml` の `paths` に `release.yml` を足して、名前だけを変えた変更でもマニュアルの検査が走るようにしている。変更は Markdown と既存ツールの分岐に収まり、過剰な仕組みは見当たらない。

## 最終判定

**判定: GO**（重大な指摘 1 を設計に反映することが条件）

**理由**: 構造の食い違いや要件の取りこぼしは無く、要件の全項目に実装と確かめ方の道筋がある。唯一の重大な指摘は自己テスト 2 ファイルの期待値の追従で、設計の方式を変えずに File Structure Plan へ 2 ファイルを足すだけで直る。

**次の手順**:
1. 設計ディスカッションで重大な指摘 1 を確定し、design.md の File Structure Plan・NoTalkExemption・Testing Strategy に `verify-scripts-test.mjs` と `gen-skill-refs-test.mjs` の変更を書き足す。
2. あわせて design.md の Open Questions 1〜6 を確定する。
3. その後 `/kiro-spec-tasks manual-download-page` でタスクを作る。
