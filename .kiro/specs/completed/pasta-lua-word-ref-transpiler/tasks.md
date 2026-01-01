# Implementation Tasks

## 概要

本タスクリストは、pasta_lua トランスパイラーにおける `SetValue::WordRef` 構文のLuaコード生成機能を実装するための作業項目を定義する。

**実装範囲**: `generate_var_set()` メソッド内の `SetValue::WordRef` 分岐を実装し、「＄変数＝＠単語」構文を「`var.変数名 = act:word("単語名")`」形式のLuaコードに変換する。

**推定工数**: 全タスク合計 約3-5時間

---

## Tasks

### Core Implementation

- [x] 1. WordRef代入のコード生成を実装
- [x] 1.1 (P) generate_var_set()メソッドのSetValue::WordRef分岐を実装
  - `SetValue::WordRef { name }` パターンマッチング処理を追加
  - `var_path`変数（既存のVarScope判定結果）を使用してLuaコードを生成
  - ローカル変数: `var.変数名 = act:word("単語名")` 形式で出力
  - グローバル変数: `save.変数名 = act:word("単語名")` 形式で出力
  - 既存の `Action::WordRef` 実装パターン（L399-402）を参照
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 1.2 (P) 既存Expr処理との互換性を検証
  - `SetValue::Expr` 分岐が引き続き正常に動作することを確認
  - 同一ファイル内でExprとWordRefの混在処理が可能であることを確認
  - 既存の統合テスト（sample.pasta）を実行して互換性を検証
  - _Requirements: 2.1, 2.2_

### Testing & Validation

- [x] 2. テストフィクスチャとテストケースを整備
- [x] 2.1 sample.expected.luaを作成
  - sample.luaを元に厳密一致テスト用の期待値ファイルを作成
  - コメントを除去し、トランスパイル結果との完全一致を可能にする
  - `var.場所 = act:word("場所")` 形式の出力を含める
  - _Requirements: 3.1_

- [x] 2.2 統合テストを更新して厳密一致検証を有効化
  - `test_transpile_sample_pasta_line_comparison()` のTODOコメントを解除
  - `sample.expected.lua` との文字列比較による厳密一致検証を実装
  - `assert_eq!` による完全一致アサーションを追加
  - `sample.generated.lua` の保存機能が正常に動作することを確認
  - _Requirements: 3.2, 3.3_

- [x] 2.3* (P) ユニットテストを追加（任意）
  - WordRef Local変数代入の単体テスト
  - WordRef Global変数代入の単体テスト
  - VarScope::Args へのWordRef代入がエラーになることを確認
  - 既存のExpr代入テストが正常に通過することを確認
  - _Requirements: 1.1, 1.2, 1.3, 4.1_
  - _Note: 統合テストで機能カバレッジは確保されるため、MVP後の追加も可_

### Final Validation

- [x] 3. 全体テストと統合検証
- [x] 3.1 全統合テストを実行して合格を確認
  - `cargo test --package pasta_lua` を実行
  - `test_transpile_sample_pasta_line_comparison()` が合格することを確認
  - `test_transpile_reference_code_patterns()` が合格することを確認
  - リグレッションがないことを確認
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 3.2, 3.3, 4.1_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1 | 1.1, 2.3*, 3.1 |
| 1.2 | 1.1, 2.3*, 3.1 |
| 1.3 | 1.1, 2.3*, 3.1 |
| 2.1 | 1.2, 3.1 |
| 2.2 | 1.2, 3.1 |
| 3.1 | 2.1 |
| 3.2 | 2.2, 3.1 |
| 3.3 | 2.2, 3.1 |
| 4.1 | 2.3*, 3.1 |

**Note**: `*`マークは任意タスク（MVP後の対応も可）

---

## Implementation Notes

### Parallel Execution
- タスク 1.1 と 1.2 は並行実行可能（ファイル競合なし、独立検証）
- タスク 2.1 と 2.3* は並行実行可能（異なるファイル操作）
- タスク 2.2 は 2.1 完了後に実行（sample.expected.lua 必要）
- タスク 3.1 は全タスク完了後に実行（最終統合検証）

### Reference Implementation
- `Action::WordRef` パターン（code_generator.rs L399-402）を参照モデルとして使用
- `generate_var_set()` メソッド（code_generator.rs L281-309）を拡張対象とする

### Testing Strategy
- 統合テスト（sample.pasta）で機能カバレッジを確保
- ユニットテスト（タスク2.3*）はMVP後の追加も可能
- 厳密一致テスト（sample.expected.lua）で出力品質を保証
