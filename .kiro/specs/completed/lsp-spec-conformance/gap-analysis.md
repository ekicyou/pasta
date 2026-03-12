# ギャップ分析: lsp-spec-conformance

> **作成日**: 2026-03-12
> **対象仕様**: `lsp-spec-conformance` (requirements.md v1)
> **分析手法**: gap-analysis.md フレームワーク準拠

---

## 1. 現状調査

### 1.1 関連資産マップ

| カテゴリ                   | ファイル                                        | 内容                                                  | 状態             |
| -------------------------- | ----------------------------------------------- | ----------------------------------------------------- | ---------------- |
| LSP セマンティックトークン | `pasta_lsp/src/analysis/token_types.rs`         | 15 トークンタイプ定義 + RawToken 構造体               | ✅ 安定           |
| LSP AST ビジター           | `pasta_lsp/src/analysis/visitors.rs`            | `visit_local_scene_item` 内に CueCommand マッチアーム | ⚠️ 最低限対応のみ |
| LSP テキストユーティリティ | `pasta_lsp/src/analysis/text_utils.rs`          | `get_line_text`, `line_byte_offset`                   | ✅ 安定           |
| LSP 解析エンジン           | `pasta_lsp/src/analysis/mod.rs`                 | `AnalysisEngine::analyze`                             | ✅ 安定           |
| DSL Cue AST 型             | `pasta_dsl/src/parser/ast/cue.rs`               | `CueCommandNode`, `ScopedName`, `CueArgToken`         | ✅ 完成           |
| DSL シーン AST             | `pasta_dsl/src/parser/ast/scene.rs`             | `LocalSceneItem::CueCommand` バリアント               | ✅ 完成           |
| DSL パーサー               | `pasta_dsl/src/parser/parse_scene.rs`           | `parse_cue_cmd_line` 実装                             | ✅ 完成           |
| VSCode TextMate 文法       | `editors/vscode/syntaxes/pasta.tmLanguage.json` | 8 パターン（cue 未対応）                              | ⚠️ ギャップ       |
| VSCode package.json        | `editors/vscode/package.json`                   | 11 カスタムセマンティックトークンタイプ               | ⚠️ cue 未対応     |
| LSP 統合テスト             | `pasta_lsp/tests/`                              | 10 テストファイル（79 テスト）                        | ⚠️ cue テストなし |

### 1.2 既存パターン分析

#### トークン生成パターン（visitors.rs）

pasta_lsp は 2 つのトークン生成パターンを使用している：

**パターン A: Span ベース（シンプル）**
- `visit_call_scene`, `visit_attr`, `visit_keywords`, `visit_code_block`
- AST ノードの `span` をそのまま `add_token_from_span` にわたして 1 トークン生成
- **前提**: ノード全体が 1 つのトークンタイプで充足する場合

**パターン B: テキストスキャン（細粒度）**
- `visit_var_set`, `visit_action_line`
- AST ノードの `span` からソーステキストを取得し、カーソルで走査
- マーカー・名前・演算子・値をそれぞれ個別トークンとして生成
- **前提**: 1 行内に複数のトークンタイプが混在する場合

**現在の CueCommand 対応**:
```rust
LocalSceneItem::CueCommand(cue) => {
    if cue.span.is_valid() {
        Self::add_token_from_span(&cue.span, source, token_type::OPERATOR, 0, tokens);
    }
}
```
→ パターン A の最小実装。行全体を OPERATOR として出力しており、細粒度トークン化はゼロ。

#### AST の Span 保持状況

| AST フィールド                             | Span                             | 備考                         |
| ------------------------------------------ | -------------------------------- | ---------------------------- |
| `CueCommandNode.span`                      | ✅ 行全体（pad ~ or_comment_eol） | `cue_cmd_line` の pest Span  |
| `CueCommandNode.command`                   | ❌ String のみ                    | Span なし                    |
| `CueCommandNode.scope` → `ScopedName.span` | ✅ `cue_cmd_scope` の Span        | `@name` / `@actor:name` 全体 |
| `CueCommandNode.args` → `CueArgToken`      | ❌ 値のみ                         | Span なし                    |

**重要な制約**: `command` と `args` に個別 Span がないため、テキストスキャン方式（パターン B）が必須。ただし `visit_var_set` で確立済みのパターンなので前例あり。

### 1.3 テストパターン

- テストファイルは `crates/pasta_lsp/tests/` 直下にフラット配置
- `AnalysisEngine::analyze(source)` を直接呼び出し、`result.tokens` / `result.diagnostics` を検証
- `token_type::*` 定数でトークンタイプを比較
- 全角/半角同値テストは `fullwidth_halfwidth_test.rs` に独立ファイルとして存在

---

## 2. 要件充足可能性分析

### 要件-資産マッピング

| 要件   | AC                              | 既存資産                                           | ギャップ                     | タグ           |
| ------ | ------------------------------- | -------------------------------------------------- | ---------------------------- | -------------- |
| R1.1   | マーカー + コマンド名トークン   | `CueCommandNode.span` + テキストスキャン           | テキストスキャン実装が必要   | **Missing**    |
| R1.2   | スコープ個別トークン            | `ScopedName.span` あり                             | テキストスキャン実装が必要   | **Missing**    |
| R1.3   | 引数個別トークン                | `CueArgToken` 値あり、Span なし                    | テキストスキャン実装が必要   | **Missing**    |
| R1.4   | 文字列リテラル引数              | `CueArgToken::StringLiteral`                       | トークンタイプマッピングのみ | **Missing**    |
| R1.5   | 数値リテラル引数                | `CueArgToken::Integer/Float`                       | トークンタイプマッピングのみ | **Missing**    |
| R1.6   | @参照引数                       | `CueArgToken::AtRef`                               | トークンタイプマッピングのみ | **Missing**    |
| R1.7   | 全角/半角同値                   | 既存パターン確立済み                               | テストのみ                   | **Missing**    |
| R2.1   | マーカー用トークンタイプ        | `TOKEN_TYPES` 配列末尾に追加可能                   | 定義のみ                     | **Missing**    |
| R2.2   | コマンド名用トークンタイプ      | 同上                                               | 定義のみ                     | **Missing**    |
| R2.3   | SemanticTokensLegend 登録       | `semantic_tokens_legend()` が `TOKEN_TYPES` を返却 | 配列追加で自動反映           | **Missing**    |
| R2.4   | VSCode package.json 設定        | `semanticTokenTypes` 配列に追加                    | 追加のみ                     | **Missing**    |
| R3.1   | TextMate キューコマンドパターン | `pasta.tmLanguage.json`                            | パターン新規追加             | **Missing**    |
| R3.2   | マーカースコープ                | 既存の `keyword.other.marker.pasta` パターンあり   | 再利用可能                   | **Missing**    |
| R3.3   | コマンド名スコープ              | なし                                               | 新規定義                     | **Missing**    |
| R3.4   | @scope スコープ                 | `inline-word-ref` パターン参考可能                 | 新規定義                     | **Missing**    |
| R3.5   | 引数スコープ                    | なし                                               | 新規定義                     | **Missing**    |
| R4.1-4 | テスト                          | テストインフラ確立済み                             | テストコード新規作成         | **Missing**    |
| R4.5   | リグレッションなし              | 79 テスト通過中                                    | 確認のみ                     | 既存           |
| R5.1   | インデックス不変                | 既存 15 エントリ                                   | 末尾追加で保証               | 既存           |
| R5.2   | 末尾追加                        | `TOKEN_TYPES` 配列                                 | 追加のみ                     | **Constraint** |
| R5.3   | 既存パターン不変                | TextMate 既存 8 パターン                           | 不変                         | 既存           |
| R5.4   | 優先度衝突回避                  | patterns 配列順序                                  | 適切な位置に挿入             | **Constraint** |

### 複雑性シグナル

- **メイン実装**: テキストスキャン方式のビジター実装（`visit_var_set` の前例あり → パターン確立済み）
- **外部依存**: なし（tower-lsp, pasta_dsl のみ、いずれも既存）
- **アルゴリズム的複雑性**: 低（カーソルベースの文字列走査のみ）
- **統合ポイント**: `TOKEN_TYPES` 配列、`package.json`、TextMate 文法 — いずれも追記型

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張（推奨）

**概要**: 既存の `visitors.rs` 内に `visit_cue_command` メソッドを追加し、テキストスキャン方式で細分化トークンを生成する。

**変更対象**:

| ファイル                | 変更内容                                                                                                                   |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `token_types.rs`        | `TOKEN_TYPES` 末尾に 1-2 タイプ追加、`token_type` mod にインデックス定数追加                                               |
| `visitors.rs`           | `visit_local_scene_item` の CueCommand アームを `visit_cue_command` メソッド呼び出しに変更。メソッド新設（推定 60-100 行） |
| `pasta.tmLanguage.json` | `cue-command` パターン追加（patterns 配列の `call` と `actor` の間に挿入）                                                 |
| `package.json`          | `semanticTokenTypes` と `semanticTokenScopes` に cue 用エントリ追加                                                        |
| テスト                  | `cue_command_token_test.rs` 新設                                                                                           |
| `README.md`             | トークンタイプ表に cue 行を追加                                                                                            |

**トレードオフ**:
- ✅ 既存のテキストスキャンパターン（`visit_var_set`）を踏襲 — 一貫性が高い
- ✅ ファイル数の増加が最小限（テストファイル 1 つのみ新設）
- ✅ visitors.rs は「guideline exception」として既に 750 行超を許容している
- ❌ visitors.rs がさらに膨張する（推定 +80 行で 830 行前後）

### Option B: 新コンポーネント分離

**概要**: `analysis/cue_visitors.rs` を新設し、カ ーコマンド固有のトークン化ロジックを分離する。

**変更対象**:

| ファイル                   | 変更内容                                                           |
| -------------------------- | ------------------------------------------------------------------ |
| `token_types.rs`           | Option A と同一                                                    |
| `analysis/cue_visitors.rs` | 新設。`visit_cue_command` および補助メソッド                       |
| `analysis/mod.rs`          | `mod cue_visitors;` 追加                                           |
| `visitors.rs`              | CueCommand アームから `cue_visitors::visit_cue_command` を呼び出し |
| `pasta.tmLanguage.json`    | Option A と同一                                                    |
| `package.json`             | Option A と同一                                                    |
| テスト                     | Option A と同一                                                    |
| `README.md`                | Option A と同一                                                    |

**トレードオフ**:
- ✅ visitors.rs の膨張を抑制
- ✅ cue 固有ロジックの独立テスト容易性
- ❌ ファイル間の結合が増える（`AnalysisEngine` の `pub(super)` メソッドへのアクセス）
- ❌ 既存の visitors.rs のパターン（全ビジターが 1 ファイル）からの逸脱
- ❌ 80 行程度の追加では分離の恩恵が薄い

### Option C: ハイブリッド（Span 追加 + ビジター拡張）

**概要**: まず `pasta_dsl` の `CueCommandNode` に `command` / `args` 個別の Span を追加した上で、LSP 側で Span ベーストークン化（パターン A）を使用する。

**変更対象**:

| ファイル                              | 変更内容                                                               |
| ------------------------------------- | ---------------------------------------------------------------------- |
| `pasta_dsl/src/parser/ast/cue.rs`     | `command_span: Span`, `CueArgToken` に `Span` フィールド追加           |
| `pasta_dsl/src/parser/parse_scene.rs` | `parse_cue_cmd_line` / `parse_cue_cmd_args` で個別 Span を取得して設定 |
| `token_types.rs`                      | Option A と同一                                                        |
| `visitors.rs`                         | Span ベースの `visit_cue_command`（テキストスキャン不要 → シンプル）   |
| `pasta.tmLanguage.json`               | Option A と同一                                                        |
| `package.json`                        | Option A と同一                                                        |
| テスト                                | Option A と同一 + pasta_dsl 側の Span テスト追加                       |

**トレードオフ**:
- ✅ LSP ビジターがシンプル (Span ベースで `add_token_from_span` 呼び出しのみ)
- ✅ 将来の利用者（dola 等）にも Span 情報が利用可能
- ❌ pasta_dsl クレートへの変更が必要（スコープ拡大）
- ❌ 既存の `CueArgToken` enum に破壊的変更（各バリアントに Span 追加）
- ❌ parse_scene.rs の `parse_cue_cmd_args` が複雑化
- ❌ 本仕様の目的（LSP 追従）を超えたスコープ

---

## 4. 工数・リスク評価

| 項目       | 評価            | 根拠                                                                           |
| ---------- | --------------- | ------------------------------------------------------------------------------ |
| **工数**   | **S（1-3 日）** | 既存パターン踏襲、追記型変更、外部依存なし                                     |
| **リスク** | **Low**         | 確立されたパターン（テキストスキャン方式）、既存テストでリグレッション検出可能 |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（既存コンポーネント拡張）

**理由**:
1. 80 行程度の追加では分離のメリットが薄い（Option B 不採用）
2. pasta_dsl への変更はスコープ外（Option C 不採用）
3. `visit_var_set` のテキストスキャンパターンが前例として存在し、一貫性が高い

### 設計フェーズで解決すべき判断事項

| ID  | 項目                              | 選択肢                                             | 推奨                                                                         |
| --- | --------------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------- |
| D1  | マーカーのトークンタイプ          | 新規 `cueMarker` vs 既存再利用                     | 新規 `cueMarker` の方がテーマ制御しやすい（R2.1）                            |
| D2  | コマンド名のトークンタイプ        | `function` 再利用 vs 新規 `cueCommand`             | 新規 `cueCommand` が意図明確（R2.2）。ただし `function` 再利用でも十分       |
| D3  | テキストスキャンの実装詳細        | `CueCommandNode.span` の行テキストからカーソル走査 | `visit_var_set` パターン踏襲が自然                                           |
| D4  | TextMate パターン位置             | `call` の後、`actor` の前                          | `!` / `！` は `action-line` の `\S+?` にマッチしないため衝突なし             |
| D5  | ScopedName の actor:name 分割粒度 | `@actor:name` 全体で1トークン vs actor/name 分割   | ScopedName.span が全体を持つため1トークンが自然                              |
| D6  | 括弧・カンマ記号のトークン化      | OPERATOR トークン生成 vs TextMate に委譲           | TextMate 委譲がシンプル。Oniguruma の全角スペース制約（`[\s\u3000]+`）に留意 |

### Research Needed（設計フェーズで調査）

- なし（すべて既存パターンで解決可能）
