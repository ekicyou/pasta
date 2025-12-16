# Research & Design Decisions: pasta-transpiler-actor-variables

---
**Purpose**: Pastaトランスパイラーのアクター変数参照修正に関する設計判断と技術調査の記録

**Usage**:
- 既存コードパターンの分析結果を記録
- 設計判断の根拠を明示
- 実装時の参照資料として使用
---

## Summary
- **Feature**: `pasta-transpiler-actor-variables`
- **Discovery Scope**: Extension（既存トランスパイラーの修正）
- **Key Findings**:
  - Two-Pass アーキテクチャの Pass 1 のみの修正で完結
  - 5箇所の明確な修正ポイント（L276, L353, L355, L375, L390）
  - グローバルラベルモジュールのみに use 文を生成（ローカルラベルは継承）

## Research Log

### Transpiler Architecture Analysis
- **Context**: アクター変数参照修正の影響範囲を特定
- **Sources Consulted**: 
  - `crates/pasta/src/transpiler/mod.rs` (904 lines)
  - `crates/pasta/tests/fixtures/comprehensive_control_flow.transpiled.rn` (参照実装)
  - Gap Analysis Document
- **Findings**:
  - **Two-Pass Strategy**: Pass 1（ラベル生成）+ Pass 2（セレクター生成）の分離アーキテクチャ
  - **Pass 1 Functions**: `transpile_global_label()`, `transpile_local_label()`, `transpile_statement_to_writer()`
  - **Pass 2 Functions**: `transpile_pass2()` - 今回の修正対象外
  - **修正スコープ**: Pass 1 の3関数のみ
- **Implications**: 
  - Pass 2への影響なし（label_selector ロジックは不変）
  - LabelRegistry への影響なし（ID生成ロジックは不変）
  - 既存テストの大部分は影響を受けない

### Module Scope and Import Strategy
- **Context**: use 文をどのスコープに配置するか
- **Sources Consulted**:
  - Rune VM の module scoping ドキュメント
  - `comprehensive_control_flow.transpiled.rn` の構造分析
  - Requirements Namespace Discussion section
- **Findings**:
  - **Rune Module Scope**: モジュール内の関数はモジュールレベルの use 文を継承
  - **Global Label Structure**: `pub mod ラベル名_N { use ...; pub fn __start__(...) {} pub fn ローカル_1(...) {} }`
  - **Local Label Functions**: 同じモジュール内の関数として生成、独自の use 文不要
- **Implications**:
  - グローバルラベル生成時（`transpile_global_label()`）のみ use 文を出力
  - ローカルラベル生成時（`transpile_local_label()`）は use 文不要
  - 3つの use 文: `use pasta::*;`, `use pasta_stdlib::*;`, `use crate::actors::*;`

### Actor Object Structure
- **Context**: ctx.actor に何を格納するか
- **Sources Consulted**:
  - `crates/pasta/tests/fixtures/test-project/main.rn` のアクター定義
  - Rune object literal syntax
  - Gap Analysis の Actor 構造定義
- **Findings**:
  - **Actor Definition**: `pub const さくら = #{ name: "さくら", id: "sakura" };`
  - **Field Access**: Rune の `.` 演算子でフィールドアクセス可能
  - **Actor Function**: `pasta_stdlib::actor_event(name: String)` は文字列を受け取る
- **Implications**:
  - `ctx.actor = さくら;` でオブジェクト全体を代入
  - `yield Actor(ctx.actor.name);` で name フィールドを抽出して渡す
  - Actor 関数は文字列のみ必要（オブジェクト全体は不要）

### Pasta Function Shorthand
- **Context**: `crate::pasta::call` → `call` への短縮可能性
- **Sources Consulted**:
  - Rune use statement documentation
  - 現在の `Statement::Call` / `Statement::Jump` 生成コード
- **Findings**:
  - **Current Implementation**: フルパス `crate::pasta::call()` を使用
  - **Shorthand Enabler**: `use pasta::*;` をモジュールレベルで宣言
  - **Target Functions**: `jump()`, `call()` のみ（Pass 2 で定義）
- **Implications**:
  - `use pasta::*;` 追加により短縮形が有効化
  - L375, L390 で `crate::pasta::` プレフィックス削除
  - コード可読性向上（`for a in call(...)` の方が簡潔）

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **Option A: Direct Modification** | `transpiler/mod.rs` の5箇所を直接修正 | 明確な修正ポイント、既存パターン踏襲 | なし | ✅ 推奨（Gap Analysis の結論） |
| Option B: Refactor with Context | `TranspileContext` にアクター収集機能を追加 | 将来の拡張性 | 過剰設計、現時点で不要 | 却下 |
| Option C: AST Preprocessing | AST に変換前処理を追加 | 分離された責務 | アーキテクチャ変更が大きい | 却下（スコープ外） |

**選択**: Option A - 既存アーキテクチャ内での最小限の修正

## Design Decisions

### Decision: ローカルラベル関数は use 文不要

- **Context**: `transpile_local_label()` で use 文を生成すべきか
- **Alternatives Considered**:
  1. **Option A**: use 文不要（モジュールレベルを継承）
  2. **Option B**: ローカル関数内で use 文を重複生成
- **Selected Approach**: Option A - use 文不要
- **Rationale**: 
  - Rune のスコープルール: モジュール内関数はモジュールレベルの use を継承
  - 参照実装（`comprehensive_control_flow.transpiled.rn`）でも use 文はモジュールレベルのみ
  - コード重複を避ける
- **Trade-offs**: 
  - ✅ 簡潔な出力
  - ✅ 既存パターンとの一貫性
  - ❌ なし
- **Follow-up**: テスト実行時に Rune VM コンパイルが成功することを確認

### Decision: Actor 関数には name フィールドのみ渡す

- **Context**: `yield Actor(...)` に何を渡すか
- **Alternatives Considered**:
  1. **Option A**: `Actor(ctx.actor.name)` - name フィールドのみ
  2. **Option B**: `Actor(ctx.actor)` - オブジェクト全体
- **Selected Approach**: Option A - name フィールドのみ
- **Rationale**:
  - `pasta_stdlib::actor_event(name: String)` の型シグネチャが文字列を要求
  - Rust 側の ScriptEvent::ChangeSpeaker も name: String フィールドのみ
  - オブジェクト全体を渡すと型エラーが発生
- **Trade-offs**:
  - ✅ 型安全性
  - ✅ 既存 ScriptEvent 定義との互換性
  - ❌ なし（将来的に拡張が必要な場合は ScriptEvent を変更）
- **Follow-up**: Rune VM 実行時のイベント生成を検証

### Decision: ワイルドカードインポート使用

- **Context**: `use crate::actors::*;` vs 個別インポート
- **Alternatives Considered**:
  1. **Option A**: ワイルドカードインポート（`use crate::actors::*;`）
  2. **Option B**: 使用されるアクターのみ個別インポート（`use crate::actors::{さくら, うにゅう};`）
- **Selected Approach**: Option A - ワイルドカードインポート
- **Rationale**:
  - 実装の簡素化: AST からアクター使用を収集・ソートする処理が不要
  - スクリプトの柔軟性: Rune ブロック内で動的にアクターを参照可能
  - 既存パターンとの一貫性: `pasta_stdlib::*` も同様にワイルドカード使用
- **Trade-offs**:
  - ✅ 実装が簡潔
  - ✅ 将来のアクター追加に対応
  - ❌ 名前空間の汚染（軽微、actors モジュールは限定的）
- **Follow-up**: なし（標準的なパターン）

## Risk Assessment

### Low Risk Items
- ✅ 明確な修正箇所（5箇所のみ）
- ✅ 既存アーキテクチャを変更しない
- ✅ 既存テストスイートが包括的
- ✅ 参照実装が存在（comprehensive_control_flow.rn）

### Mitigation Strategies
1. **テスト再実行**: `cargo test --all-targets` で全テスト通過を確認
2. **Rune VM 検証**: トランスパイル出力を Rune VM でコンパイル
3. **出力比較**: 参照実装と生成コードの構造比較

## Follow-up Items
1. **実装後**: `test-project/main.rn` のアクター定義を `pub mod actors { ... }` に移動
2. **実装後**: `comprehensive_control_flow.transpiled.rn` の再生成
3. **検証**: 全テストが pass することを確認
