# Research & Design Decisions

## Summary
- **Feature**: `lua55-reference-manual-ja-chapter6-full`
- **Discovery Scope**: Extension（既存仕様の部分再作成）
- **Key Findings**:
  - 現在の第6章は約405行（要約版）、原文HTMLは約3,900行（11サブセクション分割で計約4,500行）
  - 既存の翻訳パイプライン（Phase 0-4）は完了済みで、第6章のみ再翻訳すれば良い
  - サブセクション別ファイル（`chapters/en/standard-libraries/*.html`）が11個存在し、これを翻訳ソースとして使用可能

## Research Log

### 現行第6章の分析
- **Context**: 既存の`06-standard-libraries.md`が要約版であり、完全版への再作成が必要
- **Sources Consulted**: 
  - `crates/pasta_lua/doc/lua55-manual/06-standard-libraries.md`（現行版：約405行）
  - `chapters/en/06-standard-libraries.html`（原文：約3,900行）
  - `chapters/en/standard-libraries/*.html`（分割済み：11ファイル）
- **Findings**:
  - 現行版は各関数を表形式で要約しており、詳細説明が欠落
  - 原文には各関数の完全な説明（パラメータ、戻り値、動作、例外）が含まれる
  - パターンマッチング（§6.5.1）、format書式（§6.5）、pack書式（§6.5）の詳細解説が未翻訳
- **Implications**: 11サブセクション各HTMLを個別に翻訳し、1ファイルに統合する必要あり

### 翻訳ソースファイル一覧
- **Context**: 翻訳対象のHTMLファイルを特定
- **Sources Consulted**: `.kiro/specs/completed/lua55-reference-manual-ja/chapters/en/standard-libraries/`
- **Findings**:
  | ファイル名 | セクション | 推定行数 |
  |-----------|-----------|---------|
  | 01-loading-the-libraries-in-c-code.html | §6.1 | 約100行 |
  | 02-basic-functions.html | §6.2 | 約615行 |
  | 03-coroutine-manipulation.html | §6.3 | 約200行 |
  | 04-modules.html | §6.4 | 約400行 |
  | 05-string-manipulation.html | §6.5 | 約819行（最大） |
  | 06-utf-8-support.html | §6.6 | 約150行 |
  | 07-table-manipulation.html | §6.7 | 約250行 |
  | 08-mathematical-functions.html | §6.8 | 約300行 |
  | 09-input-and-output-facilities.html | §6.9 | 約450行 |
  | 10-operating-system-facilities.html | §6.10 | 約350行 |
  | 11-the-debug-library.html | §6.11 | 約400行 |
  | **合計** | **§6.1-6.11** | **約4,000行** |
- **Implications**: 各ファイルを順次翻訳し、最終的に1ファイル（`06-standard-libraries.md`）に統合

### トークン制限分析
- **Context**: AI翻訳の1回あたりの処理可能量を検討
- **Sources Consulted**: 親仕様の設計書（ChapterTranslator設計）
- **Findings**:
  - 1KB HTML ≈ 250 tokens
  - 最大サブセクション（§6.5 文字列操作）は約819行 ≈ 60KB ≈ 15,000 tokens
  - 全11サブセクション合計 ≈ 100-120KB ≈ 25,000-30,000 tokens
  - 1回のAI呼び出しで全体処理可能（100k tokensリミット内）
- **Implications**: サブセクション単位で翻訳し、最後に統合する方式が効率的

### 既存GLOSSARY.mdとの整合性
- **Context**: 既存の用語対応表との一貫性確保
- **Sources Consulted**: `.kiro/specs/completed/lua55-reference-manual-ja/GLOSSARY.md`
- **Findings**:
  - 既存GLOSSARYは約272行、基本型・メタテーブル・GC・エラー処理・コルーチン等をカバー
  - 標準ライブラリ関連用語（`fail`, `pattern`, `capture`等）の追加が必要かもしれない
- **Implications**: 翻訳中に新規用語が出現した場合はGLOSSARYを更新

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 単一ファイル出力 | 11サブセクションを1ファイル（`06-standard-libraries.md`）に統合 | シンプル、既存構造維持 | ファイルサイズ大（推定2,000-3,000行） | **採用** |
| 分割ファイル出力 | `06-standard-libraries/`ディレクトリに11ファイル分割 | 編集・参照が容易 | 既存ディレクトリ構造変更が必要、リンク修正多数 | 今回は見送り |

**選択**: 単一ファイル出力（要件8.1「単一ファイルまたは論理的分割」に適合、5,000行閾値内）

## Design Decisions

### Decision: サブセクション単位翻訳・統合方式

- **Context**: 約3,900行のHTMLを効率的に翻訳する必要がある
- **Alternatives Considered**:
  1. 全体を一括翻訳 — コンテキスト制限でリスクあり
  2. サブセクション単位で翻訳後統合 — 制御しやすい
- **Selected Approach**: サブセクション単位翻訳・統合方式
- **Rationale**: 
  - 既存の分割済みHTMLファイル（11個）を活用
  - 各サブセクションが独立した翻訳単位となり品質管理が容易
  - 統合時にセクション間整合性を確認可能
- **Trade-offs**: 翻訳回数が増加するが、品質は向上
- **Follow-up**: 統合後にリンク・用語整合性を最終チェック

### Decision: 既存GLOSSARYの活用

- **Context**: 用語一貫性を保つための基準
- **Alternatives Considered**:
  1. 新規GLOSSARYを作成
  2. 既存GLOSSARYを拡張
- **Selected Approach**: 既存GLOSSARYを活用し、必要に応じて拡張
- **Rationale**: 他章との用語一貫性を維持
- **Trade-offs**: 新規用語追加時に既存との整合性確認が必要
- **Follow-up**: 翻訳中に`fail`, `pattern`, `capture`等の標準ライブラリ固有用語を追加

## Risks & Mitigations

- **リスク1: ファイルサイズ超過（5,000行超）** — 翻訳後に行数確認し、必要なら分割を検討（要件8.2に基づく）
- **リスク2: パターンマッチング解説の複雑さ** — §6.5.1のパターン構文は特に詳細な翻訳が必要、原文構造を維持
- **リスク3: 用語不一致** — 翻訳後にGLOSSARY全体をクロスチェック

## References

- [Lua 5.5 Reference Manual (英語)](https://www.lua.org/manual/5.5/) — 原文
- [Lua 5.4 日本語マニュアル](https://lua.dokyumento.jp/manual/5.4/) — 用語参考
- [親仕様: lua55-reference-manual-ja](./../completed/lua55-reference-manual-ja/) — 完了済み仕様
