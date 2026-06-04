# Requirements Document

## Project Description (Input)

pasta ゴーストの作者（利用者）向けの利用者マニュアルを、**mdBook をバックボーンとしたサーバー不要の静的 HTML+JS サイト**として構築する。

- **誰の問題か**: pasta ゴーストを制作する利用者。Pasta DSL 文法・Lua ランタイム API・ゴースト制作手順を学びたいが、読みやすく検索可能なマニュアルが存在しない。
- **現状**: 知識が `doc/spec/*.md`（実装判断向けの権威的仕様）・`GRAMMAR.md`（クイックリファレンス）・スキル `pasta-lua-coding` / `pasta-ghost-authoring`（AI 向け参照）に分散し、利用者は複数の場所を横断する必要がある。入門者がゼロからゴーストを作る導線もない。静的サイト生成基盤は未導入。
- **どう変えるか**: Pasta DSL 文法・Lua API/コーディング・入門チュートリアルを 1 つの検索可能な静的 HTML サイトに統合する。既存 `doc/spec/` Markdown 資産を流用し、`mdbook build` でサーバー不要・クライアント側全文検索つきの成果物を生成、GitHub Pages 等にそのまま公開できるようにする。

**含める範囲（In）**: mdBook プロジェクト構成 ／ Pasta DSL 文法マニュアル ／ Lua API・コーディングマニュアル ／ 入門・チュートリアル ／ 静的ビルド成果物の生成と公開導線 ／ 本文の執筆・トーン設計。

**含めない範囲（Out）**: Lua 5.5 言語リファレンス本体の取り込み（別リポ [ekicyou/lua55-manual-ja](https://github.com/ekicyou/lua55-manual-ja) への参照リンクのみ） ／ `doc/spec/` の権威的ソースの置き換え ／ 多言語化（i18n、初版は日本語のみ） ／ 自動 API ドキュメント生成。

**主要制約**: 出力はサーバー不要の静的 HTML+JS のみで完結すること ／ 既存 `doc/spec/` 資産を再利用し二重管理を最小化 ／ 日本語（UTF-8）の表示・検索が正しく動作すること ／ Python・Node の重量級エコシステム依存を持ち込まない（mdBook = Rust ツールチェーン内）。

**執筆方針（ボイス・トーン）**: 本文は案内役キャラクター「Claudia 令嬢」のボイスで執筆する。導入・語りかけ・コラム・締めはキャラ口調、仕様・構文・手順・API の説明本体は普通の文体（正確さ・読みやすさ最優先）で使い分ける。キャラ口調は味付けであり技術的正確さを犠牲にしない。

> 詳細な背景・境界・上流下流・代替案の却下理由は [brief.md](./brief.md)（discovery 成果物）を参照。

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->
