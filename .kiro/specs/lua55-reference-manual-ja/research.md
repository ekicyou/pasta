# Research & Design Decisions

## Summary
- **Feature**: `lua55-reference-manual-ja`
- **Discovery Scope**: Extension（既存pasta_luaへのドキュメント追加）
- **Key Findings**:
  1. Lua 5.5マニュアルHTMLは9章構成で各章30-80KBに分割可能
  2. Lua 5.4日本語版の用語を参考にすることで翻訳一貫性を確保可能
  3. AI章単位処理でコンテキスト制限を回避し効率的な翻訳が可能

## Research Log

### HTML構造分析（Lua 5.5マニュアル）
- **Context**: 効率的な章単位分割のため、原文HTMLの構造を調査
- **Sources Consulted**: reference-lua55-en.html（369KB）の直接解析
- **Findings**:
  - 全セクションが`<h1>` (Lua 5.5 Reference Manual)、`<h2>` (章)、`<h3>` (節) で構造化
  - 各章は`<h2>` タグで明確に区切られている
  - アンカーリンクは `#<section-id>` 形式（例: `#2.1`, `#6.4.1`）
  - コードブロックは `<pre>` タグ内に配置
  - 関数シグネチャは `<pre>` 内で特別な書式なし
- **Implications**: 正規表現またはHTMLパーサーで章単位分割が可能

### Lua 5.4日本語版との比較分析
- **Context**: 翻訳用語の参考基準を確立
- **Sources Consulted**: reference-lua54-ja.html（430KB）、lua.dokyumento.jp
- **Findings**:
  - 基本型の訳語: nil（そのまま）、boolean（そのまま）、number（そのまま）、string（文字列）
  - 技術概念の訳語: metatable（メタテーブル）、coroutine（コルーチン）、garbage collection（ガベージコレクション）
  - API名は常に原文維持: `lua_pushstring`, `luaL_checknumber` など
  - 訳語の揺れ: "スタック" vs "stack"、"レジストリ" vs "registry"
- **Implications**: GLOSSARY.mdで訳語選択を明示し、全章で統一適用

### Lua 5.5の新機能（Lua 5.4との差分）
- **Context**: 翻訳時にLua 5.5固有の新機能を識別
- **Sources Consulted**: reference-lua55-en.html 第8章「Incompatibilities」
- **Findings**:
  - **新キーワード**: `global`（グローバル変数宣言）
  - **新API**: `lua_pushexternalstring`, `lua_numbertocstring`, `lua_closethread`
  - **構文変更**: `global namelist ['=' explist]` 構文の追加
  - **型アノテーション**: `global x: integer = 10` 形式（現在は無視）
- **Implications**: Lua 5.4参照時に5.5固有要素は新規翻訳が必要

### 章構成とサイズ見積もり
- **Context**: Phase 0の分割作業設計
- **Findings**:
  | 章 | 内容 | 推定サイズ | 複雑度 |
  |----|------|-----------|-------|
  | 1 | イントロダクション | 2-3KB | 低 |
  | 2 | 基本概念 | 30-40KB | 中 |
  | 3 | 言語仕様 | 40-50KB | 高 |
  | 4 | C API | 80-100KB | 高 |
  | 5 | 補助ライブラリ | 30-40KB | 中 |
  | 6 | 標準ライブラリ | 80-100KB | 高（分割候補） |
  | 7 | スタンドアロン | 3-5KB | 低 |
  | 8 | 非互換性 | 5-10KB | 低 |
  | 9 | 完全構文 | 5-10KB | 低 |
  | Index | 索引 | 10-15KB | 中 |
- **Implications**: 4章と6章は章内分割も検討（1ファイル50KB以下目標）

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 単一ファイル | 全章を1ファイルに集約 | シンプル、リンク不要 | サイズ過大、ナビゲーション困難 | 非採用 |
| 章単位分割 | 1章1ファイル（大章は2-3分割） | 適切なサイズ、並列作業可 | リンク整合性管理が必要 | **採用** |
| 節単位分割 | 各節を個別ファイル | 最小単位、高い柔軟性 | ファイル数過多、管理困難 | 非採用 |

## Design Decisions

### Decision: 章単位分割方式
- **Context**: 369KB HTMLを効率的にAI処理するための分割単位
- **Alternatives Considered**:
  1. 単一ファイル — 全章を1ファイル（サイズ過大）
  2. 章単位分割 — 各章を個別ファイル（バランス良好）
  3. 節単位分割 — 各節を個別ファイル（過度な細分化）
- **Selected Approach**: 章単位分割（大章は内部サブセクションで分割）
- **Rationale**: 
  - 1ファイル20-50KBはAI処理に最適なサイズ
  - 章ごとのコンテキストが保持される
  - 並列翻訳作業が可能
- **Trade-offs**: 章間リンクの管理が必要
- **Follow-up**: 4章（C API）と6章（標準ライブラリ）は章内分割を検討

### Decision: AI翻訳フロー設計
- **Context**: 人間レビュー不可の制約下で高品質翻訳を実現
- **Alternatives Considered**:
  1. 一括翻訳+人手レビュー — 開発者レビュー不可で却下
  2. AI一括翻訳のみ — 品質担保不足で却下
  3. AI多段階品質改善 — 複数回のAIレビューで品質向上
- **Selected Approach**: Phase 0-4の5段階AI処理
- **Rationale**:
  - Phase 0で分割することでコンテキスト制限回避
  - Phase 1で初版翻訳と用語対応表作成
  - Phase 2-4で段階的な品質向上
- **Trade-offs**: 工数増加（32-60h）だが品質担保
- **Follow-up**: 各Phaseの具体的なプロンプト設計

### Decision: 用語対応表フォーマット
- **Context**: 200-300語の技術用語を一貫して管理
- **Selected Approach**: GLOSSARY.md（表形式）
- **Format**:
  ```markdown
  | English | 日本語 | 備考 |
  |---------|-------|------|
  | metatable | メタテーブル | Lua 5.4準拠 |
  | garbage collection | ガベージコレクション | GCと略記しない |
  ```
- **Rationale**: Markdown表形式で検索・編集が容易
- **Follow-up**: Phase 1で初版作成、Phase 2-4で更新

### Decision: ナビゲーション設計
- **Context**: 章間リンクとセクションアンカーの命名規則
- **Selected Approach**:
  - ファイル間リンク: `[§2.1 値と型](./02-basic-concepts.md#21--値と型)`
  - セクションアンカー: 見出しテキストをハイフン区切り小文字化
  - パンくずリスト: 各ファイル冒頭に配置
- **Rationale**: GitHub/VS Code互換のMarkdownリンク形式
- **Follow-up**: AI検証スクリプトでリンク整合性チェック

## Risks & Mitigations
- **Risk 1**: AI翻訳の技術用語誤訳 — Mitigation: GLOSSARY.mdで統一、Phase 2で用語チェック
- **Risk 2**: 章間リンク切れ — Mitigation: Phase 3でリンク検証スクリプト実行
- **Risk 3**: Lua 5.5新機能の訳語不統一 — Mitigation: 新機能リストを事前作成し優先翻訳
- **Risk 4**: コンテキスト制限による翻訳品質低下 — Mitigation: Phase 0で章単位分割

## References
- [Lua 5.5 Reference Manual](https://www.lua.org/manual/5.5/) — 翻訳ソース（英語原文）
- [Lua 5.4 日本語マニュアル](https://lua.dokyumento.jp/manual/5.4/) — 用語参考文献
- [Lua License (MIT)](https://www.lua.org/license.html) — ライセンス準拠要件
- [pasta_lua README](../../crates/pasta_lua/README.md) — 配置先クレートの既存ドキュメント
