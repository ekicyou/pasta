# Brief: pasta-user-manual

## Problem
pasta ゴーストの作者（利用者）が、Pasta DSL 文法・Lua ランタイム API・ゴースト制作の手順を学ぶための、
読みやすく検索可能な「利用者マニュアル」が存在しない。現状、知識は以下に分散している:
- `doc/spec/*.md`（権威的仕様。実装判断向けで利用者向けの読み物ではない）
- `GRAMMAR.md`（利用者向けクイックリファレンス）
- スキル `pasta-lua-coding` / `pasta-ghost-authoring`（AI向け参照、人間が通読しづらい）

利用者は複数の場所を横断する必要があり、入門者がゼロからゴーストを作る導線がない。

## Current State
- 文法仕様 Markdown（`doc/spec/` 全12章）と `GRAMMAR.md` は整備済み = マニュアル原稿の母体は既に存在
- Lua API/コーディング規約は `.kiro/steering/tech.md` とスキル `pasta-lua-coding/SKILL.md` に存在
- Lua 5.5 言語リファレンス日本語版は別リポ [ekicyou/lua55-manual-ja](https://github.com/ekicyou/lua55-manual-ja) へ独立済み
- 静的サイト生成基盤は未導入（package.json / conf.py / book.toml いずれも root に無し）
- 環境実測: `mdbook v0.5.3` が cargo bin に**導入済み**、`node/npm`・`python3.13/pip` も利用可能

## Desired Outcome
- Pasta DSL 文法・Lua API/コーディング・入門チュートリアルを 1 つの検索可能な HTML サイトで通読できる
- サーバー不要の**純粋な静的 HTML+JS** として配布でき、GitHub Pages 等にそのまま公開できる
- 既存の `doc/spec/` Markdown 資産を可能な限り再利用し、二重管理を避ける

## Approach
**mdBook をバックボーンに採用。**
- 入力は Markdown のため `doc/spec/` 資産をほぼそのまま流用できる
- `mdbook build` が静的 HTML+JS（クライアント側 elasticlunr 全文検索・`.nojekyll` 同梱）を出力 → サーバー不要で公開可能（実機検証済み）
- Rust プロジェクトと同一ツールチェーンで追加エコシステム依存ゼロ（mdbook は既に導入済み）

**却下した代替案:**
- Sphinx（Python 製）— 利用可能だが reST/MyST 設定が重く、Rust プロジェクトに異種の Python 依存を持ち込む。Markdown 資産にはオーバースペック
- VitePress / Docusaurus（Node 製）— モダンだが `node_modules` ツリーを抱え、2 つ目のエコシステムを持ち込む

## Scope
- **In**:
  - mdBook プロジェクト構成（`book.toml`、`SUMMARY.md`、章構成）
  - Pasta DSL 文法マニュアル（`doc/spec/` + `GRAMMAR.md` を利用者向けに再編・流用）
  - Lua API / コーディング規約マニュアル（`pasta-lua-coding` 相当の内容を人間可読化）
  - 入門 / チュートリアル（ゼロからはじめての pasta 辞書を作る手順、新規書き下ろし）
  - 静的ビルド成果物の生成と公開導線（GitHub Pages 想定）
  - 本文の執筆・トーン設計（「Claudia 令嬢」ボイスによる執筆。下記「執筆方針」参照）
- **Out**:
  - Lua 5.5 言語リファレンス本体の取り込み（別リポへの参照リンクのみ）
  - 仕様書（`doc/spec/`）を権威的ソースから置き換えること（マニュアルは派生・利用者向け、仕様書は権威を維持）
  - 多言語化（i18n）。初版は日本語のみ
  - 自動 API ドキュメント生成（手書き運用）

## Boundary Candidates
- サイト基盤・ビルド構成（book.toml / SUMMARY / CI / 公開）
- 文法マニュアルコンテンツ（DSL）
- Lua API/コーディングマニュアルコンテンツ
- 入門チュートリアルコンテンツ
- 既存 `doc/spec/` との同期方針（流用 or 参照 or コピー）

## Out of Boundary
- 仕様書 `doc/spec/` の内容変更・再設計（マニュアルは消費側であり、仕様の権威を侵さない）
- Lua 言語自体のリファレンス整備（別リポの責務）
- ゴースト実行基盤（areka 側）の機能

## Upstream / Downstream
- **Upstream**: `doc/spec/`（文法権威ソース）、`GRAMMAR.md`、スキル `pasta-lua-coding` / `pasta-ghost-authoring`、`SOUL.md`
- **Downstream**: ゴースト作者向け公開ドキュメント、将来的な多言語化・APIリファレンス自動生成の土台

## Existing Spec Touchpoints
- **Extends**: なし（新規境界）
- **Adjacent**: 完了済み `documentation-consolidation` / `lua-api-documentation` / `lua55-reference-manual-ja`（コンテンツの母体だが本 spec はサイト化・利用者導線が責務で重複しない）

## 執筆方針（ボイス・トーン）

マニュアル本文は、案内役キャラクター「**Claudia 令嬢**」のボイスで執筆する。
ただし全編を口調で押し通すのではなく、文章の役割ごとに文体を使い分ける。

| パート | 文体 | 目的 |
| ------ | ---- | ---- |
| 導入・語りかけ・章の繋ぎ・コラム・励まし | Claudia 令嬢の軽妙なキャラ口調（「おほほ」系お嬢様＋熱い魂） | 読者を引き込み、学習のモチベーションを保つ |
| 仕様・構文・手順・API などの**説明本体** | 普通の文体（淡々と正確・読みやすさ最優先） | 誤読を防ぎ、リファレンスとして信頼できること |

**運用ルール:**
- キャラ口調は「味付け」であり、技術的正確さを犠牲にしない。説明が口調で読みにくくなるなら普通の文体を優先する
- コードブロック・表・構文定義・コマンド例の**内部**にはキャラ口調を持ち込まない
- 各章は「Claudia の軽い導入 → 普通の文体での本体解説 → ひとことの締め」を基本リズムとする
- トーンの一貫性を保つため、語り口のサンプル（ボイスガイド）を最初に 1 つ確立し、以降はそれに準拠する

## Constraints
- 出力は**サーバー不要の静的 HTML+JS のみ**で完結すること（ユーザー必須要件）
- 既存 `doc/spec/` Markdown 資産を再利用し、二重管理を最小化する
- 日本語コンテンツ（UTF-8）を正しく表示・検索できること
- 追加の重量級依存（Python/Node エコシステム）を持ち込まない（mdBook = Rust ツールチェーン内）
- 本文は「Claudia 令嬢」ボイスで執筆する（軽妙トークはキャラ口調、説明本体は普通の文体。上記「執筆方針」参照）
