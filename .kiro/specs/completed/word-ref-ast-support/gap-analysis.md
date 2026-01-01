# ギャップ分析レポート

## 分析概要

### 対象機能
Pasta DSL変数代入文（VarSet）への単語参照（word_ref: `@単語名`）サポート追加

### 現状
- grammar.pestで`set = ( expr | word_ref )`として文法定義済み
- ASTレイヤーでは`VarSet.value: Expr`として実装
- word_refをExprで表現する設計になっていない（intentional separation）

### 提案する変更
- 新しい`SetValue`列挙型を導入：`Expr(Expr)` | `WordRef { name: String }`
- `VarSet.value`の型を`Expr` → `SetValue`に変更
- parse_var_set関数とトランスパイラーを対応させる

---

## 1. 現在の状態調査

### 1.1 主要ファイル・モジュール

| ファイル | 責務 | 備考 |
|---------|------|------|
| `crates/pasta_core/src/parser/grammar.pest` | PEG文法定義 | `set = ( expr \| word_ref )`既に定義 |
| `crates/pasta_core/src/parser/ast.rs` | AST型定義 | VarSetの定義（line 507+） |
| `crates/pasta_core/src/parser/mod.rs` | パーサー実装 | parse_var_set関数（line 569+） |
| `crates/pasta_rune/src/transpiler/code_generator.rs` | Rune code生成 | generate_var_set関数（line 177+） |
| `tests/parser2_integration_test.rs` | パーサーテスト | VarSetのテスト（line 273, 289他） |

### 1.2 既存の設計パターン

**AST型の特性:**
- 列挙型による値の分離（例：`Expr::Integer`, `Expr::String`, `Expr::VarRef`など）
- `#[derive(Debug, Clone)]`の統一的使用
- `span: Span`による位置情報保持

**パーサー実装の特性:**
- `Pair<Rule>`から直接AST構造を構築
- `try_parse_expr`関数による共通の式パース処理
- `parse_var_set`で`terms`と`operators`を分離して二項式を再構築

**トランスパイラーの特性:**
- `LocalSceneItem::VarSet(var_set)`をパターンマッチで処理
- `generate_var_set`で`var_set.value`に対して`generate_expr`を呼び出し
- スコープ（Local/Global）に基づくコード生成分岐

---

## 2. 要件可能性分析

### 2.1 要件から技術ニーズへの変換

| 要件 | 技術ニーズ | 既存対応 | ギャップ |
|------|---------|---------|---------|
| 要件1: SetValue型追加 | 新しい列挙型定義 | なし | **新規作成** |
| 要件2: VarSet.value型変更 | VarSetの型更新 | Expr型 | **型変更** |
| 要件3: parse_var_set対応 | word_refルール検出 | exprのみ処理 | **Rule::word_ref処理** |
| 要件4: 既存コード対応 | パターンマッチ更新 | 11箇所以上 | **複数ファイル更新** |

### 2.2 ギャップと制約

#### ギャップ1: SetValue型が存在しない
- **現状**: VarSet.valueはExpr型で固定
- **必要**: SetValue列挙型の新規定義
- **影響範囲**: ast.rs（定義）

#### ギャップ2: parse_var_set関数がword_refに非対応
- **現状**: exprのみを`try_parse_expr`で処理
- **必要**: `Rule::word_ref`の検出と`SetValue::WordRef`生成
- **影響範囲**: mod.rs（パーサー実装）

#### ギャップ3: VarSet.valueの使用箇所が複数存在
- **現状**: 以下の箇所で`var_set.value`にアクセス
  - `code_generator.rs` line 192: `self.generate_expr(&var_set.value)`
  - `parser2_integration_test.rs` line 276, 292, 307, 323: `matches!(vs.value, Expr::...)`
  - `code_generator.rs` test line 519: VarSetリテラル作成
- **必要**: SetValue型のパターンマッチに更新
- **影響範囲**: code_generator.rs, 複数テストファイル

#### ギャップ4: generate_expr関数の拡張可能性
- **現状**: `generate_expr(&self, expr: &Expr)`がExpr型に依存
- **必要**: SetValueの場合、word_refをどのようにコード生成するか未定義
- **研究必要**: word_refのRune側実装方法

### 2.3 複雑さの兆候

| 項目 | 評価 | 理由 |
|------|------|------|
| CRUD/単純ロジック | ✅ | VarSet型変更は構造的な拡張 |
| 既存パターン活用 | ✅ | 列挙型のバリアント追加パターンを踏襲 |
| マルチファイル影響 | ⚠️ | 4ファイル以上の更新が必要 |
| 外部統合 | ❌ | パーサー層内に閉じている |
| パフォーマンス | ❌ | 影響なし |

---

## 3. 実装アプローチのオプション

### オプションA: VarSet構造体の型変更（推奨）

**概要:**
- SetValue列挙型を新規定義
- VarSet.valueをExpr → SetValueに変更
- パーサーとトランスパイラーを一括更新

**修正対象ファイル:**

| ファイル | 変更内容 | 複雑度 |
|---------|---------|--------|
| ast.rs | SetValue型定義追加 | 低 |
| mod.rs | parse_var_set関数更新（Rule::word_ref対応） | 中 |
| code_generator.rs | generate_var_set/generate_expr更新 | 中 |
| parser2_integration_test.rs | パターンマッチ更新（4箇所） | 低 |
| code_generator test | VarSetリテラル作成更新 | 低 |

**互換性評価:**
- **破壊的変更**: VarSet.valueの型が変更されるため、既存のパターンマッチは**コンパイルエラー**になる
- **スコープ**: パーサー層内に閉じており、外部APIには影響なし
- **テスト**: 既存テストは対応パターンマッチで修正可能

**複雑度と保守性:**
- SetValue型は単純な列挙型で認知負荷は低い
- パターンマッチの修正は機械的で自動化可能
- 型安全性が向上（word_refをExprに混在させない）

**トレードオフ:**
- ✅ 設計意図が型レベルで正確に表現される
- ✅ word_ref固有の処理を明示的にできる
- ✅ 既存テストがコンパイルエラーで検出されるため漏れが少ない
- ❌ VarSetを使用する全箇所の更新が必須
- ❌ 複数ファイルへの変更が必要

---

### オプションB: Exprにword_refバリアントを追加

**概要:**
- `Expr::WordRef { name: String }`を新規追加
- VarSet.valueの型は変わらない（Expr）
- parse_var_setのみ対応すれば十分

**修正対象ファイル:**

| ファイル | 変更内容 | 複雑度 |
|---------|---------|--------|
| ast.rs | Expr列挙型にWordRef追加 | 低 |
| mod.rs | parse_var_set関数更新（Rule::word_ref対応） | 低 |
| code_generator.rs | generate_expr関数にwordref対応追加 | 低 |

**互換性評価:**
- **破壊的変更**: なし。Exprに新しいバリアントが追加されるだけ
- **既存パターンマッチ**: 網羅性チェックがある場合、コンパイラ警告（野生パターン未対応）が出る可能性
- **スコープ**: Exprはアクション行など複数の箇所で使用されているため、設計の一貫性に疑問

**複雑度と保守性:**
- 修正箇所が最小限で迅速に対応可能
- ただし、grammar.pestの`( expr | word_ref )`という分離の意図が曖昧になる

**トレードオフ:**
- ✅ 最小限の変更で実装可能
- ✅ 既存テストに手を加える必要が少ない
- ❌ grammar.pestの分離設計の意図がASTに反映されない
- ❌ word_refのセマンティクスがExprに混在するため、将来的な保守性が低い
- ❌ ユーザーの指摘「exprと混同しないように分離した」という方針に反する

---

### オプションC: 専用フィールド追加（ハイブリッド）

**概要:**
```rust
pub struct VarSet {
    pub name: String,
    pub scope: VarScope,
    pub value: Option<Expr>,
    pub word_ref: Option<String>,
    pub span: Span,
}
```

**修正対象ファイル:**
- ast.rs（フィールド追加）
- mod.rs（word_refチェックと設定）
- code_generator.rs（条件分岐）

**互換性評価:**
- **破壊的変更**: Option型ラッピングため、既存の`var_set.value`アクセスはコンパイルエラー
- **曖昧性**: `value`と`word_ref`が同時に存在できる設計は不正な状態を許容

**複雑度と保守性:**
- 条件分岐が複雑化（両方Noneの場合を考慮）
- 型安全性が低い

**トレードオフ:**
- ❌ Option型の条件分岐で複雑性増加
- ❌ 不正な状態（両方Someまたは両方None）を許容
- ❌ 結局、既存コードの更新が必要

---

## 4. 実装複雑度とリスク評価

### 複雑度評価（オプションA推奨）

| 項目 | 評価 | 根拠 |
|------|------|------|
| **総合難度** | **M (3-7日)** | 4ファイル+テスト複数更新、既知パターン活用 |
| AST型定義 | S | 列挙型定義は単純 |
| パーサー実装 | M | Rule::word_refの検出と値抽出が必要 |
| トランスパイラー | M | SetValueのパターンマッチ、word_refコード生成が未定義 |
| 既存コード更新 | S | 機械的なパターンマッチ修正 |
| テスト更新 | S | 既存テストの修正 |

### リスク評価

| リスク項目 | レベル | 根拠 |
|----------|--------|------|
| **総合リスク** | **Medium** | 既知技術、明確スコープ、既存パターン活用 |
| 技術的未知 | 低 | Rustの列挙型/パターンマッチは確立済み |
| 統合複雑度 | 低 | パーサー層に閉じている |
| パフォーマンス | 低 | 型変更のみで実行時影響なし |
| **word_refコード生成** | **High** | Rune側の実装方法が未定義（別仕様） |

---

## 5. 要件-アセット対応マップ

| 要件 | 必要なアセット | 現状 | ギャップ | 優先度 |
|------|--------------|------|---------|--------|
| 1. SetValue型定義 | ast.rs | なし | **Missing** | High |
| 2. VarSet.value型変更 | ast.rs | Expr型 | **Type Change** | High |
| 3. Rule::word_ref検出 | parser mod.rs | exprのみ | **Missing Logic** | High |
| 4. word_refコード生成 | code_generator.rs | ExprOnly | **Unknown** | Medium |
| 5. パターンマッチ更新 | 複数ファイル | Expr依存 | **Constraint** | High |

---

## 6. 推奨事項（必須要件を踏まえた修正）

### 必須要件の追加
1. **コンパイルエラー**: 全てのモジュールで `cargo check --all` が成功
2. **リグレッション**: 既存テストが全て合格（`cargo test --all` で0失敗）
3. **設計意図**: grammar.pestの分離をAST上で反映

### 修正された推奨アプローチ: **オプションA（SetValue列挙型導入）**

**選定理由:**
必須要件を踏まえ、以下の点でオプションAが最適：
- ✅ grammar.pestの`( expr | word_ref )`分離の意図を**型安全に反映**
- ✅ expr と word_ref を構造的に分離し、設計の一貫性を保証
- ✅ 破壊的変更を最小限に（パターンマッチ追加のみ）
- ✅ 将来のセマンティクス実装に対応しやすい
- ✅ VarSet.value に SetValue 型を使用することで、expr/word_ref の違いを明示的に表現

### 実装計画

| アクション | 変更内容 | 影響 |
|----------|---------|------|
| ast.rs | SetValue列挙型定義 + VarSet.value型変更 | 必須 |
| mod.rs | parse_var_set内でRule::word_refを検出し、SetValue構築 | 必須 |
| code_generator.rs | SetValueのパターンマッチ（WordRef は無視実装） | 必須 |
| 既存テスト | VarSet.value の型アクセスを SetValue に合わせて修正 | 必須 |

**修正対象ファイル数:** 4ファイル以上
**破壊的変更:** VarSet.value 型の変更（コンパイルエラー発生）
**必須対応:** トランスパイラー層のパターンマッチ更新
**テスト修正必要:** あり（type mismatch エラー対応）

---

## 7. 次のステップ

**設計フェーズ（`/kiro-spec-design word-ref-ast-support`）:**
- SetValue列挙型の具体的な実装詳細（位置、モジュール構成）
- parse_var_set関数内でのSetValue構築ロジック
- word_refのコード生成戦略（トランスパイラー層での無視実装）
- VarSet.value型変更に伴う既存コード修正範囲の詳細確認
- AST層と文法層の整合性に関するドキュメント化

**実装フェーズ（`/kiro-spec-impl word-ref-ast-support`）:**
- ast.rsへのSetValue列挙型定義
- VarSet構造体のvalue フィールド型変更（Expr → SetValue）
- mod.rs parse_var_set内でのRule::word_ref検出と SetValue構築
- code_generator.rsのSetValue パターンマッチ対応（WordRef は無視実装）
- 既存テストの型修正（VarSet.value → SetValue）
- 新しいテスト追加（word_refパース動作確認）
- `cargo check --all` と `cargo test --all` による検証
