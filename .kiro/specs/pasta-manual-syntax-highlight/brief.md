# Brief: pasta-manual-syntax-highlight

## Problem
利用者マニュアル（[https://ekicyou.github.io/pasta/](https://ekicyou.github.io/pasta/)、mdBook 製）の `*.pasta` コードブロックが**無色（シンタックスハイライト無し）**で表示される。文法を学ぶための入門・文法章でコードが読みづらく、VSCode 拡張で得られる色分け体験とのギャップが大きい。読者（ゴースト作者・初学者）の学習効率を損なっている。

## Current State
- マニュアルの pasta ブロックは HTML 上 `<code class="language-pasta">` と正しくタグ付けされている（例: first-ghost に5箇所）。
- しかし mdBook v0.5.3 同梱の **highlight.js は `pasta` 言語定義を持たない**ため、`language-pasta` を未知言語として無色レンダリングする。
- VSCode 拡張は TextMate 文法 **`editors/vscode/syntaxes/pasta.tmLanguage.json`**（261行・`scopeName: source.pasta`・約21パターン・32スコープ）で色分けを実現しており、これが pasta ハイライトの **SSOT（単一の正本）**。
- mdBook 同梱 `book.js` は **全 `<code>` に client 側で `hljs.highlightBlock()` を無条件適用**する（`language-pasta` 除外分岐なし）。highlight.js は `textContent` を読み直して `innerHTML` を書き戻すため、事前注入した色付き span はページ読込時に破壊される（検証で確認済み）。

## Desired Outcome
- マニュアルの `*.pasta` コードブロックが、VSCode 拡張と同等の色分けで表示される（マーカー＊＠％、シーン名、アクター、関数呼出、さくらスクリプト、変数、コメント等が判別可能）。
- light/navy 両テーマで適切に配色される。
- 出力は**純静的 HTML のみ**（サーバー不要・`file://` オフライン閲覧維持・公開サイトに WASM 等のランタイム依存を持ち込まない）。
- pasta 文法の正本は **VSCode の TextMate 文法ただ一つ**に保たれ、二重管理（drift）を生まない。

## Approach
**C: build-time で TextMate 文法を再利用し、highlight.js 互換クラスの span を事前生成する。**

1. **字句解析（build-time Node）**: `book/tools/` 配下に build-time スクリプトを置き、`vscode-textmate`（MIT）＋ `vscode-oniguruma`（MIT・WASM をビルド時のみ読込）で `pasta.tmLanguage.json` をロードし、各 pasta コードブロックをトークナイズする。bigram 索引ビルダ・drift-check と同じ「build-time Node・出力は純静的」パターンに準拠。
2. **スコープ→クラス マッピング**: TextMate スコープ（`comment.line.pasta` / `keyword.control.scene.pasta` / `entity.name.type.actor.pasta` / `string.other.sakura-script.pasta` 等・約32種）を highlight.js 互換クラス（`hljs-comment` / `hljs-keyword` / `hljs-title` 等）へ写像する有限・安定なマッピング層を持つ。これにより mdBook 既存テーマ CSS（light/navy の hljs 配色）をそのまま流用でき、色焼き込み（Shiki 方式）を避けられる。
3. **client 側 再ハイライトの中和（必須）**: mdBook の `book.js` が pasta ブロックを再ハイライトして事前 span を破壊する問題を、`theme/` のカスタム（book.js の差し替え／パッチ、または pasta ブロックを `highlightBlock` 対象から除外するマーカー付与）で防ぐ。**この一手は本アプローチの必須要件**であり、設計フェーズで堅牢な方式を確定する。
4. **HTML 後処理 or preprocessor**: `mdbook build` 後の `book/book/**/*.html` の pasta ブロックを色付き span へ置換する後処理方式を基本線とする（bigram 索引と同じ後処理パターン）。preprocessor 方式との比較は設計で判断。

**なぜ C か**: SSOT 単一（drift ゼロ・VSCode と一致）と light/dark ネイティブ統合を両取りし、確立済みの build-time Node 思想に乗る。検証で B/C 共通の book.js 再ハイライト問題が判明したが、本プロジェクトは既に mdBook 内部へ制御された介入（head.hbs フック・book 出力後処理）を行っており、その延長で対処可能。

## Scope
- **In**:
  - build-time の pasta 字句解析ツール（`vscode-textmate` + `vscode-oniguruma` + `pasta.tmLanguage.json`）
  - TextMate スコープ → highlight.js 互換クラスのマッピング
  - mdBook client 側再ハイライトの中和（pasta ブロックを book.js の highlightBlock から保護）
  - light/navy 両テーマでの配色確認
  - 公開ワークフロー（manual.yml）への組み込み（mdbook build 後・bigram 索引再生成と整合する順序）
  - 検証（pasta ブロックが色分け表示され、`file://` でも保持される／再ハイライトで壊れない）
- **Out**:
  - VSCode 拡張の TextMate 文法そのものの変更・拡張（読み取り再利用のみ）
  - pasta 以外の言語のハイライト改善
  - 公開サイトへのランタイム依存（WASM 等）の持ち込み（build-time に閉じる）
  - エディタ/LSP のハイライト改善（VSCode 拡張・pasta_lsp の領分）

## Boundary Candidates
- **Tokenizer 層**: TextMate 文法ロード＋トークナイズ（build-time Node、`book/tools/` 配下）
- **Mapping 層**: TextMate スコープ → hljs クラスの写像（テーマ CSS 流用の要）
- **mdBook 統合層**: 事前ハイライト HTML の注入＋ book.js 再ハイライト中和（mdBook 内部結合・最も注意を要する seam）
- **公開パイプライン結線**: manual.yml における実行順序（mdbook build → 本ハイライト → bigram 索引 → drift-check の整合）

## Out of Boundary
- `editors/vscode/syntaxes/pasta.tmLanguage.json` の改変（SSOT は読み取り専用で再利用）
- pasta 言語仕様・DSL パーサ（`pasta_dsl`）の変更
- マニュアル本文コンテンツの執筆（pasta-user-manual の領分・完了済み）

## Upstream / Downstream
- **Upstream**:
  - `editors/vscode/syntaxes/pasta.tmLanguage.json`（ハイライト文法の SSOT・読み取り再利用）
  - `pasta-user-manual`（完了）が確立した mdBook 基盤・`book/tools/` build-time パターン・manual.yml 公開パイプライン
- **Downstream**:
  - 将来 pasta 文法が拡張されたら TextMate 文法→本ハイライトが追従（SSOT 単一なので追従は一元的）
  - mdBook のバージョン更新（book.js / highlight.js 同梱物の変更）は本機能の再検証を要する（Revalidation Trigger）

## Existing Spec Touchpoints
- **Extends**: `pasta-user-manual`（完了・アーカイブ）。本仕様はその mdBook サイトへハイライト層を追加する後続強化。pasta-user-manual は再オープンせず、独立した新仕様として扱う。
- **Adjacent**: VSCode 拡張（`editors/vscode/`）— TextMate 文法を共有 SSOT として参照するが、拡張側のコードには手を入れない。`pasta_lsp`（エディタ側ハイライト）とは責務が異なる（本仕様は静的サイト用）。

## Constraints
- **静的・オフライン**: 出力は純静的 HTML/CSS/JS のみ。`file://` 閲覧維持。公開サイトにランタイム WASM/フレームワークを持ち込まない（build-time に閉じる）。pasta-user-manual R1 の制約を継承。
- **drift 回避（SSOT 単一）**: pasta 文法の正本は TextMate 文法ただ一つ。highlight.js 用の第2文法を新設しない（本アプローチ C を選んだ主因）。
- **mdBook 内部結合**: book.js の無条件再ハイライトへの対処が必須。mdBook v0.5.3 の同梱物（ハッシュ付きファイル名・book.js 実装）に依存するため、mdBook 更新時の再検証を Revalidation Trigger として明記する。
- **テーマ**: light（既定）/ navy（dark）両対応。mdBook の既存 hljs テーマ CSS を流用する設計とする。
- **依存方針**: `vscode-textmate` / `vscode-oniguruma`（共に MIT）を build-time devDependency として導入。重量級フレームワークは不可。
