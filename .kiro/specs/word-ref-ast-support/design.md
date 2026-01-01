# Technical Design: word-ref-ast-support

## Overview

### Objective
Pasta DSL の `＄場所＝＠場所` 構文（VarSet における word_ref 割り当て）をサポートするため、AST 層に SetValue 列挙型を導入し、VarSet.value の型を Expr から SetValue に変更する。

### Scope
- **In Scope**:
  - SetValue 列挙型の定義と VarSet への適用
  - parse_var_set 関数での word_ref 検出と SetValue 構築
  - トランスパイラー層での SetValue 対応（WordRef は無視）
  - 既存テストの更新
- **Out of Scope**:
  - word_ref のセマンティクス実装（別仕様）
  - 他の AST ノードへの SetValue 適用

### Key Components
1. **SetValue 列挙型** - ast.rs に新規追加
2. **VarSet 構造体** - value フィールドの型変更
3. **parse_var_set 関数** - word_ref 検出と SetValue 構築
4. **generate_var_set 関数** - SetValue パターンマッチ対応

---

## Component Details

### Component 1: SetValue 列挙型

**Purpose**: VarSet の値として expr または word_ref のいずれかを表現する型

**Requirements Mapping**:
- REQ-1: SetValue列挙型の導入

**Interface/API**:

```rust
/// VarSet の値を表現する列挙型
/// 
/// grammar.pest の `set = ( expr | word_ref )` に対応し、
/// 代入文の右辺が式か単語参照かを型レベルで区別する
#[derive(Debug, Clone, PartialEq)]
pub enum SetValue {
    /// 式（数値、文字列、変数参照、関数呼び出し、二項演算など）
    Expr(Expr),
    /// 単語参照（`@単語名` 形式）
    WordRef { name: String },
}
```

**Implementation Notes**:
- Expr 列挙型の直後（ast.rs の約680行付近）に配置
- PartialEq は Expr と同様に derive
- Clone は AST ノードの標準実装
- WordRef の name フィールドは `@` プレフィックスを除いた単語名

**Dependencies**:
- なし（新規型）

---

### Component 2: VarSet 構造体の更新

**Purpose**: SetValue を使用するよう value フィールドの型を変更

**Requirements Mapping**:
- REQ-1: SetValue列挙型の導入（適用先として）

**Interface/API**:

```rust
// Before (現在)
pub struct VarSet {
    pub name: String,
    pub scope: VariableScope,
    pub value: Expr,  // ← Expr 型
    pub span: Span,
}

// After (変更後)
pub struct VarSet {
    pub name: String,
    pub scope: VariableScope,
    pub value: SetValue,  // ← SetValue 型に変更
    pub span: Span,
}
```

**Implementation Notes**:
- ast.rs line 514 の `value: Expr` を `value: SetValue` に変更
- SetValue は同ファイル内で定義されるため追加の use 不要
- この変更により、VarSet.value を使用する全箇所でコンパイルエラーが発生（意図的）

**Dependencies**:
- SetValue 列挙型（同ファイル内）

---

### Component 3: parse_var_set 関数の更新

**Purpose**: word_ref ルールを検出し、SetValue::WordRef を構築する

**Requirements Mapping**:
- REQ-2: parse_var_set関数でのSetValue構築
- REQ-3: parse_var_set関数の内部処理更新
- REQ-6: word_ref構文のパース成功確認

**Interface/API**:

```rust
// 戻り値の型は変更なし（VarSet を返す）
// 内部で SetValue を構築

fn parse_var_set(&self, pair: Pair<Rule>) -> Result<VarSet, PastaError> {
    // ... 既存の name, scope 抽出処理 ...
    
    // 内部で expr と word_ref を分離処理
    // word_ref の場合: SetValue::WordRef { name } を構築
    // expr の場合: 既存の二項演算処理後、SetValue::Expr(expr) を構築
}
```

**Implementation Notes**:
- inner.peek() で Rule::word_ref を検出
- word_ref の場合:
  - inner_pairs から word_ref ペアを取得
  - as_str() で `@単語名` を取得し、`@` を除去して name に設定
  - `SetValue::WordRef { name }` を構築
  - 二項演算処理をスキップ（word_ref は単独値）
- expr の場合:
  - 既存の try_parse_expr + 二項演算処理
  - 最終結果を `SetValue::Expr(expr)` でラップ

**Dependencies**:
- SetValue 列挙型
- Rule::word_ref（grammar.pest で定義済み）

**処理フロー**:

```
parse_var_set(pair)
├── name, scope を抽出
├── inner = pair.into_inner()
├── if inner.peek() == Some(Rule::word_ref)
│   ├── word_ref_pair = inner.next()
│   ├── name = word_ref_pair.as_str().trim_start('@')
│   └── value = SetValue::WordRef { name }
├── else (expr の場合)
│   ├── 既存の try_parse_expr 処理
│   ├── 既存の二項演算処理
│   └── value = SetValue::Expr(expr)
└── return VarSet { name, scope, value, span }
```

---

### Component 4: generate_var_set 関数の更新

**Purpose**: SetValue をパターンマッチし、Expr の場合のみコード生成

**Requirements Mapping**:
- REQ-4: トランスパイラー層へのAPI破壊的変更への対応

**Interface/API**:

```rust
fn generate_var_set(&mut self, var_set: &VarSet) -> Result<String, TranspilerError> {
    match &var_set.value {
        SetValue::Expr(expr) => {
            // 既存の generate_expr 呼び出し
            let value_code = self.generate_expr(expr)?;
            // 既存のコード生成処理
        }
        SetValue::WordRef { name: _ } => {
            // 本仕様ではコード生成しない（無視）
            // 将来の仕様で実装
            Ok(String::new())  // または適切なスタブ
        }
    }
}
```

**Implementation Notes**:
- code_generator.rs line 177-227 の generate_var_set 関数を更新
- SetValue::Expr の場合は既存処理をそのまま適用
- SetValue::WordRef の場合:
  - 本仕様では空文字列を返す（コード生成なし）
  - コメントで将来の実装仕様を参照
  - 警告ログの出力は任意

**Dependencies**:
- SetValue 列挙型（use 文の追加が必要）
- 既存の generate_expr 関数

---

## Data Models

### SetValue 列挙型

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SetValue {
    /// 式の値（既存の Expr 型をラップ）
    Expr(Expr),
    /// 単語参照（@単語名）
    WordRef { name: String },
}
```

**フィールド説明**:
- `Expr(Expr)`: 既存の式型をそのままラップ。数値、文字列、変数参照、関数呼び出し、二項演算など
- `WordRef { name }`: 単語参照。name は `@` を除いた単語名（例: `@場所` → `name: "場所"`）

**型の関係**:

```
VarSet
├── name: String
├── scope: VariableScope
├── value: SetValue ──────┬── Expr(Expr)
│                         │   └── Integer | Float | String | ...
│                         └── WordRef { name: String }
└── span: Span
```

---

## Interactions

### パース時のデータフロー

```
1. grammar.pest: set = ( expr | word_ref )
       ↓
2. parse_var_set(pair)
       ├── Rule::word_ref 検出 → SetValue::WordRef { name }
       └── Rule::expr 検出    → try_parse_expr → SetValue::Expr(expr)
       ↓
3. VarSet { name, scope, value: SetValue, span }
       ↓
4. AST に格納
```

### コード生成時のデータフロー

```
1. generate_var_set(var_set: &VarSet)
       ↓
2. match &var_set.value
       ├── SetValue::Expr(expr) → generate_expr(expr) → Rune code
       └── SetValue::WordRef { .. } → "" (無視)
       ↓
3. 生成コードを返却
```

---

## Requirements Traceability

| Requirement ID | Component | Implementation Details |
|----------------|-----------|------------------------|
| REQ-1: SetValue列挙型の導入 | SetValue enum, VarSet struct | ast.rs に SetValue 定義、VarSet.value の型変更 |
| REQ-2: parse_var_set関数でのSetValue構築 | parse_var_set | word_ref → WordRef, expr → Expr(expr) |
| REQ-3: parse_var_set関数の内部処理更新 | parse_var_set | Rule::word_ref の検出ロジック追加 |
| REQ-4: トランスパイラー層への対応 | generate_var_set | SetValue パターンマッチ、WordRef は無視 |
| REQ-5: 既存テストのリグレッション防止 | テストファイル | SetValue::Expr でラップ |
| REQ-6: word_ref構文のパース成功確認 | テストファイル | 新規テストケース追加 |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| VarSet.value 使用箇所の見落とし | Low | Low | コンパイルエラーで検出 |
| word_ref を含むスクリプトの無視 | Medium | Medium | ドキュメント化、将来仕様への参照 |
| parse_var_set の複雑度増加 | Low | Low | 明確な分岐処理、コメント |

---

## Test Strategy

### Unit Tests

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| parse_word_ref_in_var_set | `＄場所＝＠場所` をパース | SetValue::WordRef { name: "場所" } |
| parse_expr_in_var_set | `＄場所＝"東京"` をパース | SetValue::Expr(Expr::String("東京")) |
| generate_var_set_with_expr | SetValue::Expr のコード生成 | 既存と同じ出力 |
| generate_var_set_with_word_ref | SetValue::WordRef のコード生成 | 空文字列 |

### Integration Tests

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| existing_tests_pass | 既存テストスイートの実行 | 全テスト通過 |
| word_ref_syntax_roundtrip | word_ref を含むスクリプトのパース→AST検証 | 正しい AST 構造 |

---

## Migration Guide

### 影響を受けるコード箇所

1. **crates/pasta_core/src/parser/ast.rs**
   - SetValue 列挙型の追加
   - VarSet.value の型変更

2. **crates/pasta_core/src/parser/mod.rs**
   - parse_var_set 関数の更新

3. **crates/pasta_rune/src/transpiler/code_generator.rs**
   - generate_var_set 関数の更新
   - テスト内の VarSet リテラル更新

4. **tests/parser2_integration_test.rs**
   - vs.value のパターンマッチ更新（4箇所）

### マイグレーション手順

```rust
// Before
if let Expr::Integer(n) = vs.value { ... }

// After
if let SetValue::Expr(Expr::Integer(n)) = &vs.value { ... }
```

```rust
// Before
VarSet { name, scope, value: Expr::Integer(10), span }

// After
VarSet { name, scope, value: SetValue::Expr(Expr::Integer(10)), span }
```
