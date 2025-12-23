# Implementation Gap Analysis

## 分析概要

**スコープ**: pasta2.pestを権威的文法として採用した新パーサー層（parser2）の構築

**主要な課題**:
- pasta2.pestとpasta.pestの間に文法規則・トークン定義の重大な差分が存在
- 既存のAST型定義がpasta.pestに最適化されており、pasta2.pest文法との不整合が多数
- 12個のparserテストファイルが既存parser実装に依存しており、並行運用時の競合リスク
- transpiler層が現在のAST型（Statement, SceneDef等）に強く依存している

**推奨アプローチ**: Option B（新規コンポーネント作成）+ 段階的移行戦略

---

## 1. Current State Investigation

### 既存アセット

#### 主要ファイル構成
```
src/parser/
  ├── mod.rs          (978行, PastaParser + parse_file/parse_str)
  ├── ast.rs          (297行, AST型定義 15種類)
  ├── pasta.pest      (338行, 現行文法)
  └── pasta2.pest     (202行, 新文法 ★移行対象)

tests/
  └── pasta_parser_*.rs  (12ファイル, 既存parser依存)
```

#### アーキテクチャ統合ポイント

| 依存元 | 使用API | 影響範囲 |
|--------|---------|----------|
| `lib.rs` | `pub mod parser;` + 13個の再公開型 | 全クレート公開API |
| `cache.rs` | `use crate::parser::parse_str;` | ParseCache実装 |
| `transpiler/mod.rs` | 9個のAST型を直接インポート | 2パストランスパイラ全体 |
| 12個のテストファイル | `pasta::parser::parse_str` | テストスイート全体 |

#### 命名・レイヤー規約

| 規約 | 実装パターン |
|------|-------------|
| モジュール構成 | `mod.rs` (エントリー) + 機能別ファイル |
| Pest統合 | `#[derive(Parser)]` + `#[grammar = "path"]` |
| エラー型 | `Result<T, PastaError>`, pest errorは`PastaError::PestError`でラップ |
| 公開API | `pub use ast::*;` でAST型を再公開 |

### 既存の設計パターン

#### Pestパーサー統合パターン
```rust
#[derive(Parser)]
#[grammar = "parser/pasta.pest"]
pub struct PastaParser;

pub fn parse_str(source: &str, filename: &str) -> Result<PastaFile, PastaError> {
    let mut pairs = PastaParser::parse(Rule::file, source)
        .map_err(|e| PastaError::PestError(format!("Parse error in {}: {}", filename, e)))?;
    // ... AST構築ロジック
}
```

#### AST型階層（15種類）
- コンテナ型: `PastaFile`, `SceneDef`, `WordDef`
- ステートメント型: `Statement` (4 variants: Speech, Call, VarAssign, RuneBlock)
- 式型: `Expr` (5 variants: Literal, VarRef, FuncCall, BinaryOp, Paren)
- 列挙型: `SceneScope`, `VarScope`, `FunctionScope`, `JumpTarget`, `BinOp`, `Literal`

---

## 2. Requirements Feasibility Analysis

### pasta2.pest文法の主要差分

#### 新規導入概念

| 文法要素 | pasta2.pest | pasta.pest | 実装ギャップ |
|---------|-------------|------------|--------------|
| **スコープ構造** | `file_scope`, `global_scene_scope`, `local_scene_scope` | top-levelのみ | ✅ Missing: 階層的スコープ解析 |
| **属性行** | `file_attr_line`, `global_scene_attr_line` | `attribute_content` | ⚠️ Extend: ファイルレベル属性サポート |
| **継続行** | `continue_action_line` | `continuation_line` | ⚠️ Extend: `:` で始まる継続構文 |
| **コードブロック** | `code_block` (バッククォート3連 + 言語ID) | `rune_block` | ✅ Missing: 言語識別子付きコードフェンス |
| **マーカー統一** | `marker` + `_sep` (例: `kv_marker`, `comma_sep`) | 個別定義 | ⚠️ Refactor: トークン定義の抽象化 |

#### トークン規則の変更

| 項目 | pasta2.pest | pasta.pest | 影響 |
|------|-------------|------------|------|
| ID定義 | `id1`/`idn` (XID系) + reserved pattern | `label_name` (XID系) + full-width digits | ⚠️ Constraint: reserved ID (`__name__`) 検証が必要 |
| 文字列リテラル | `string_fenced` (PUSH/POP, 4階層括弧) | 基本的な文字列のみ | ✅ Missing: 入れ子括弧対応 (`「「text」」`) |
| 空白定義 | `space_chars` (14種類のUnicode空白) | `WHITESPACE` (7種類) | ⚠️ Extend: より多くのUnicode空白サポート |
| 式構文 | `expr` → `term` + `bin*` (左結合) | 基本的な式のみ | ⚠️ Extend: 演算子優先順位が明示的 |

#### AST型定義への影響

**完全に新規必要な型**:
- `FileScope` (file-level attributes + words)
- `GlobalSceneScope` (init lines + local scenes)
- `LocalSceneScope` (statements + code blocks)
- `CodeBlock` (language_id + content)
- `KeyValue` / `KeyWords` (汎用的なkey:value構造)

**拡張が必要な既存型**:
- `PastaFile`: `file_scope` フィールド追加
- `SceneDef`: `global_scene_scope` 構造対応
- `Statement`: `ContinueAction` variant追加
- `Literal`: 4階層文字列リテラル対応

### 技術的要求と制約

#### データモデル

| 要求 | pasta2.pest由来 | 既存AST対応 | ギャップ |
|------|----------------|------------|---------|
| ファイルレベルスコープ | `file_scope` | ❌ なし | ✅ Missing |
| 階層的シーンスコープ | `global_scene_scope` → `local_scene_scope` | ⚠️ 平坦なstatements | ⚠️ Partial |
| 予約ID検証 | `reserved_id` rule | ❌ なし | ✅ Missing |
| 入れ子文字列 | `slfence_ja1`～`ja4` (PUSH/POP) | ❌ なし | ✅ Missing |

#### 非機能要件

| 項目 | 既存実装 | pasta2.pest要求 | 評価 |
|------|---------|----------------|------|
| Unicode空白 | 7種類 | 14種類 (`\u{2000}`～`\u{205F}`) | ⚠️ Medium complexity |
| パースエラー品質 | `PastaError::PestError` + filename | 同等 | ✅ 既存パターン再利用可 |
| パフォーマンス | Pestによる最適化 | 同等 | ✅ Low risk |

### 実装上の複雑性シグナル

- **高度なアルゴリズム**: スコープ階層の完全解析 (file → global → local の3層構造)
- **外部統合**: Pest 2.8のPUSH/POPスタック機能（4階層入れ子文字列必須）
- **検証ロジック**: reserved ID pattern (`__.*__`)の拒否（Pest negative lookahead必須）
- **Unicode完全対応**: 14種類の空白文字すべてのサポート

### 実装必須項目（Research完了後に実装）

**Implementation Required**:
1. **Pest PUSH/POPスタック**: 4階層文字列リテラル (`「「「「text」」」」`) の完全実装（Pestドキュメント研究 + 動作検証）
2. **スコープ解析戦略**: `file_scope`内の`global_scene_scope`反復の完全実装（再帰的パース）
3. **継続行の完全統合**: `continue_action_line` (`:` 開始) の意味論を確定し実装
4. **reserved ID検証**: Pest negative lookaheadまたはRust側検証の決定と実装

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌ **非推奨**

**対象**: `src/parser/pasta.pest` を pasta2.pest規則で上書き + `ast.rs` を拡張

#### 互換性評価
- ❌ **破壊的変更**: `file_scope`, `global_scene_scope` 等の導入により、既存ASTの構造が根本的に変化
- ❌ **テスト依存**: 12個のparserテストが現在のAST型を前提としており、全面書き換えが必要
- ❌ **transpiler依存**: 9個のAST型を直接使用しており、変更の影響範囲が全レイヤーに波及

#### 複雑性と保守性
- ❌ **認知負荷**: pasta.pestとpasta2.pestの文法規則が異なるため、段階的移行が不可能
- ❌ **単一責任原則違反**: 1つのparserモジュールが2つの文法仕様をサポートすることになる
- ❌ **ロールバック不可**: 一度変更すると、既存機能が動作しなくなる

#### Trade-offs
- ✅ ファイル数は少ない（新規作成不要）
- ❌ **全レイヤーのリグレッションリスク（Critical）**
- ❌ 既存テストスイートの全面書き換え必須
- ❌ transpilerの同時変更が必須（2パスロジックの破壊）

**結論**: 要件との乖離が大きすぎるため不採用

---

### Option B: Create New Components ✅ **強く推奨**

**対象**: `src/parser2/` ディレクトリに完全に独立したモジュールを作成

#### 新規作成の根拠
1. **明確な責任分離**: parser（pasta.pest）とparser2（pasta2.pest）は異なる文法仕様を実装
2. **既存の複雑性**: 現在のparserモジュールは既に978行（mod.rs）+ 297行（ast.rs）で十分に複雑
3. **独立したライフサイクル**: parser2は将来的にparserを置き換えるが、移行期間中は並存が必要

#### 統合ポイント

**lib.rsへの追加**:
```rust
pub mod parser;   // 既存
pub mod parser2;  // 新規追加

// parser2の公開API
pub use parser2::{
    parse_file as parse_file_v2,
    parse_str as parse_str_v2,
    PastaFile as PastaFileV2,
    // ... その他のAST型
};
```

**依存関係の設計**:
- parser2 → error (PastaError再利用)
- parser2 → ir (ScriptEvent型は共通)
- parser2 ✗ parser (完全独立)
- parser2 ✗ transpiler (新transpilerは将来実装)

#### 責任境界

| コンポーネント | 責任 | インターフェース |
|---------------|------|------------------|
| `parser2::mod` | pasta2.pest文法のパース | `parse_file_v2`, `parse_str_v2` |
| `parser2::ast` | pasta2.pest由来のAST型定義 | `PastaFileV2`, `FileScope`, `GlobalSceneScope` 等 |
| `parser2::grammar.pest` | 権威的文法定義（pasta2.pestから移動） | Pest規則 |

#### Trade-offs
- ✅ **完全な責任分離**（pasta.pest vs pasta2.pest）
- ✅ **既存機能のゼロリスク**（parserモジュール無変更）
- ✅ **段階的移行可能**（テストをparser2に徐々に移行）
- ✅ **将来のリファクタリング容易**（parser削除時に`mv parser2 parser`）
- ❌ ファイル数増加（+3ファイル: mod.rs, ast.rs, grammar.pest）
- ❌ 初期段階でtranspiler統合なし（既存transpilerはparser依存のまま）

**結論**: 要件「既存parserを削除せず並存」を完全に満たす唯一の選択肢

---

### Option C: Hybrid Approach ⚠️ **条件付き検討**

**戦略**: Option B（parser2新規作成）+ 段階的な機能統合

#### 段階的実装計画

**Phase 1: parser2完全実装（本仕様のスコープ）**
- ✅ `src/parser2/` ディレクトリ作成
- ✅ pasta2.pest → grammar.pest 移動（git mv）
- ✅ **完全なAST型定義**（FileScope, GlobalSceneScope, LocalSceneScope, CodeBlock, 4階層StringLiteral等）
- ✅ Pest PUSH/POPスタック研究完了 + 実装
- ✅ reserved ID検証の実装
- ✅ `parse_str`, `parse_file` 完全実装
- ✅ **包括的テストスイート**（全文法規則カバレッジ）

**Phase 2: transpiler2実装（将来仕様）**
- ⚠️ transpiler2の設計・実装（parser2::AST対応）
- ⚠️ 既存テストのparser2への段階的移行
- ⚠️ parser + transpiler の統合テスト

**Phase 3: 移行完了（将来仕様）**
- ⚠️ 全テストがparser2で合格
- ⚠️ `mv parser parser_legacy`
- ⚠️ `mv parser2 parser`
- ⚠️ lib.rsのエクスポート統合

#### リスク軽減策
- **Feature Flag**: `Cargo.toml` で `parser2` feature を optional に（初期段階）
- **並行テスト**: CI/CDで `parser` と `parser2` の両方をテスト
- **ドキュメント**: migration.mdに移行手順を明記

#### Trade-offs
- ✅ **段階的リスク分散**（Phase 1でリスクを最小化）
- ✅ **柔軟な計画変更**（各Phaseで評価・再計画可能）
- ❌ **計画の複雑性**（3フェーズの調整が必要）
- ❌ **一貫性リスク**（Phase間でAST設計が変わる可能性）

**結論**: Option Bをベースとして、Phase 1を本仕様で完遂し、Phase 2/3は別仕様で管理する戦略を推奨

---

## 4. Requirement-to-Asset Map

| Requirement | 既存アセット | ギャップ | 推奨実装 |
|-------------|--------------|----------|----------|
| **Req 1: pasta2.pest保全** | `src/parser/pasta2.pest` | ✅ Missing: git mv 実行のみ | Option B: `git mv` → `src/parser2/grammar.pest` |
| **Req 2: parser2モジュール作成** | ❌ 存在しない | ✅ Missing: ディレクトリ・ファイル全体 | Option B: 新規作成 |
| **Req 3: AST型定義** | `parser::ast` (pasta.pest用) | ⚠️ Constraint: 既存型と不整合 | Option B: `parser2::ast` 新規定義 |
| **Req 4: Pest統合** | `#[derive(Parser)]` パターン | ✅ Reusable: 既存パターン適用可 | Option B: 同パターンで実装 |
| **Req 5: レガシー共存** | `src/parser/` | ✅ No change: 完全保全 | Option B: 無変更 |
| **Req 6: モジュール構成** | `parser/mod.rs`, `ast.rs`, `pasta.pest` | ✅ Missing: parser2版 | Option B: 同構成で新規 |
| **Req 7: エラー統合** | `PastaError::PestError` | ✅ Reusable: そのまま使用可 | Option B: error.rs再利用 |
| **Req 8: 初期テスト** | 12個のparser tests | ⚠️ Constraint: parser依存 | Option B: 新規test作成 |
| **Req 9: ドキュメント** | parser mod doc | ✅ Missing: parser2版 | Option B: 新規doc作成 |
| **Req 10: 履歴追跡** | git history | ✅ No issue: `git mv` で保全 | Option B: `git mv` 使用 |

---

## 5. Implementation Complexity & Risk

### 工数見積もり: **L (Large: 1-2 weeks)**

**内訳**:
1. **Pest PUSH/POPスタック研究** (1日): ドキュメント調査 + 動作検証
2. **ディレクトリ・ファイル作成** (0.5日): `parser2/mod.rs`, `ast.rs`, git mv
3. **完全なAST型定義** (3日): 20種類以上の型（FileScope, GlobalSceneScope, LocalSceneScope, CodeBlock, 4階層StringLiteral, reserved ID対応等）
4. **Pestパーサー統合** (2日): `#[derive(Parser)]`, Rule定義、PUSH/POP実装
5. **parse_str/parse_file完全実装** (3日): Pest結果から完全なAST構築ロジック（スコープ階層解析含む）
6. **包括的テストスイート** (2日): 全文法規則カバレッジ、fixtures作成
7. **ドキュメント・lib.rs統合** (0.5日): mod doc, 公開API追加

**根拠**:
- ⚠️ pasta2.pest文法の完全実装には高度な学習コストあり
- ⚠️ PUSH/POPスタック機構は新規技術要素
- ⚠️ スコープ階層解析は複雑なロジック
- ✅ 既存パターン（parser実装）の部分的再適用が可能

### リスク評価: **High**

| リスク要因 | レベル | 軽減策 |
|-----------|--------|--------|
| **pasta2.pest文法の完全実装** | High | 実装前にPest動作を完全検証、曖昧な箇所は明確化してから着手 |
| **Pest PUSH/POPスタック** | High | 専用の研究フェーズを設け、サンプルコードで動作確認後に実装 |
| **スコープ階層解析の複雑性** | Medium | 段階的実装（file → global → local の順）とユニットテスト |
| **reserved ID検証** | Medium | Pest negative lookahead vs Rust検証の両方をプロトタイプし、品質の高い方を採用 |
| **テストカバレッジ不足** | Medium | 全文法規則に対するテストケースマトリクスを事前作成 |
| **transpiler統合** | Low | parser2は独立、transpiler統合は将来仕様（本仕様外） |
| **既存parserへの影響** | Low | 完全独立なのでゼロリスク |

**根拠**:
- ❌ pasta2.pest文法の新規要素（PUSH/POP、スコープ階層）が高リスク
- ❌ 完全実装要求により妥協の余地なし
- ✅ 独立モジュールなので影響範囲が限定的
- ⚠️ 工数超過リスクあり（L → XL の可能性）

---

## 6. Recommendations for Design Phase

### 推奨アプローチ: **Option B (Create New Components)**

**理由**:
1. 要件「レガシーparser保全」を完全に満たす唯一の選択肢
2. pasta2.pest文法とpasta.pest文法の差分が大きく、段階的移行が不可能
3. 将来のリファクタリング（parser削除）が容易

### 重要な設計判断事項

#### 優先度A（本仕様で決定・実装必須）
1. **AST型の命名**: `PastaFileV2` vs `PastaFile2` vs 別名？ → 設計フェーズで決定
2. **公開API命名**: 既に決定 - `parser2::parse_str` (namespace経由)
3. **スコープ構造のAST表現**: `FileScope { attrs, words, scenes }`の詳細設計 → 設計フェーズで詳細化
4. **PUSH/POPスタック実装方法**: Pestドキュメント研究 + 実装パターン確立 → 設計/実装フェーズで完了必須
5. **reserved ID検証方法**: Pest negative lookahead vs Rust validation → 設計フェーズで決定
6. **継続行の統合**: `continue_action_line`と既存`continuation_line`の関係 → 設計フェーズで明確化

#### 優先度B（将来仕様で対応）
7. **transpiler2設計**: parser2::AST → Runeコード変換 → 別仕様
8. **テスト移行戦略**: 12個の既存testをparser2に移行する順序 → 別仕様

### 研究項目（Design Phaseで実施完了必須）

**Research Item 1: Pest PUSH/POPスタック（必須）**
- **目的**: 4階層文字列リテラル (`「「「「text」」」」`) の完全実装方法確認
- **アクション**: Pestドキュメント + サンプルコード実装 + 動作検証完了
- **成果物**: 実装可能な確定コード + テストケース

**Research Item 2: スコープ階層パース戦略（必須）**
- **目的**: `file_scope` → `global_scene_scope` → `local_scene_scope` の完全解析方法
- **アクション**: Pestの再帰的規則処理の完全理解 + アルゴリズム確定
- **成果物**: AST構築アルゴリズムの詳細設計 + 擬似コード

**Research Item 3: reserved ID検証ロジック（必須）**
- **目的**: `__name__` パターンの拒否方法（Pest段階 vs AST構築段階）
- **アクション**: Pest negative lookahead vs Rust validationの両方を実装・比較
- **成果物**: 実装方針の確定（エラーメッセージ品質・パフォーマンス評価済み）

**Research Item 4: 継続行の意味論（必須）**
- **目的**: `continue_action_line` (`:` 開始) の正確な動作仕様
- **アクション**: pasta2.pestの意図を明確化、AST表現を決定
- **成果物**: 継続行のAST型定義 + サンプルコード

### 次フェーズへの引き継ぎ事項

**Design Phaseで詳細化すべき項目**:
- [ ] parser2::ASTの全型定義（20種類以上予想：FileScope, GlobalSceneScope, LocalSceneScope, CodeBlock, 4階層StringLiteral, ReservedIdValidation等）
- [ ] `parse_str`のエラーハンドリング詳細（全エラーケースの網羅）
- [ ] `src/parser2/mod.rs`の構造（関数分割方針、1000行超え想定）
- [ ] 包括的テストケースマトリクス（全文法規則カバレッジ）
- [ ] 4つのResearch Itemの完了と実装方針確定

**Implementation Phaseで実施すべき項目**:
- [ ] `git mv src/parser/pasta2.pest src/parser2/grammar.pest`
- [ ] parser2::ast型定義（完全版）
- [ ] Pestパーサー統合（PUSH/POPスタック含む）
- [ ] スコープ階層解析ロジック実装
- [ ] reserved ID検証実装
- [ ] 包括的テストスイート作成
- [ ] 全テスト合格確認

---

## 7. Conclusion

**ギャップ分析の結論**:

pasta2.pest文法は既存pasta.pest文法との間に、スコープ構造・トークン定義・文字列リテラル等の重大な差分が存在します。既存のparser実装を拡張する方法（Option A）はリグレッションリスクが極めて高く、要件「レガシーparser保全」と矛盾するため不採用とします。

**推奨実装戦略**: Option B（新規parser2モジュール作成）+ **完全実装アプローチ**

この戦略により、以下を達成します：
- ✅ 既存parserの完全保全（ゼロリスク）
- ✅ pasta2.pest文法の**完全実装**（202行すべて）
- ✅ PUSH/POPスタック研究完了後の確実な実装
- ✅ 包括的テストスイート（全文法規則カバレッジ）
- ✅ 将来のリファクタリング容易性

**工数**: **L (1-2週間)**（完全実装要求により増加）、**リスク**: **High**（PUSH/POPスタック、スコープ階層、妥協なし）

次のDesign Phaseでは、4つのResearch Itemを完了し、AST型の完全設計・スコープ解析アルゴリズム・PUSH/POPスタック実装パターンを確定させます。MVP禁止原則に従い、pasta2.pestの全機能を実装完了してからのみ完成とします。
