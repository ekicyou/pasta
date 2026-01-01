# Research & Design Decisions

## Summary

- **Feature**: pasta-lua-word-ref-transpiler
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. 既存の `Action::WordRef` 実装パターン（L399-402）を踏襲可能
  2. 単語名はXID識別子のためエスケープ処理不要
  3. テスト期待値（`sample.lua` L180）は既に定義済み

## Research Log

### 既存実装パターンの調査

- **Context**: SetValue::WordRef のコード生成パターンを決定するため
- **Sources Consulted**: 
  - `crates/pasta_lua/src/code_generator.rs` L399-402
  - `crates/pasta_core/src/parser/grammar.pest` L157
- **Findings**:
  - `Action::WordRef` は `act.{actor}:word("{word_name}")` 形式で出力
  - エスケープ処理なしで直接ダブルクォートに包んでいる
  - 既存パターンと完全に並行した実装が可能
- **Implications**: 新規パターンの設計は不要、既存パターンを踏襲

### 単語名の文字制約調査

- **Context**: エスケープ処理の必要性を判断するため
- **Sources Consulted**:
  - `crates/pasta_core/src/parser/grammar.pest` L14-22
- **Findings**:
  - `word_ref = { word_marker ~ id ~ s}`
  - `id` は `XID_START + XID_CONTINUE*`（Unicode Extended Identifier）
  - Rust/JavaScript識別子と同等の制約
  - 特殊文字（`"`、`\`等）は文法レベルで除外済み
- **Implications**: エスケープ処理は不要、直接出力で安全

### テスト期待値の確認

- **Context**: 実装後のテスト合格を確認するため
- **Sources Consulted**:
  - `crates/pasta_lua/tests/fixtures/sample.pasta` L53
  - `crates/pasta_lua/tests/fixtures/sample.lua` L180
- **Findings**:
  - `sample.pasta`: `＄場所＝＠場所` が含まれている
  - `sample.lua`: `var.場所 = act:word("場所")` が期待値として定義済み
  - Global変数WordRef代入のテストケースは存在しないが、Local変数で検証可能
- **Implications**: 実装後、既存テストが自動的に合格する

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存メソッド拡張 | generate_var_set()に分岐追加 | 最小変更、既存パターン踏襲 | なし | **採用** |
| B: 新規メソッド作成 | generate_word_ref_set()を新規作成 | 責務分離 | 過剰設計、既存パターンと不整合 | 不採用 |

## Design Decisions

### Decision: 既存メソッド内への分岐追加

- **Context**: SetValue::WordRef のコード生成を実装する場所の決定
- **Alternatives Considered**:
  1. 既存の `generate_var_set()` 内の match 式に分岐追加
  2. 新規メソッド `generate_word_ref_assignment()` を作成
- **Selected Approach**: Option 1 - 既存メソッド内への分岐追加
- **Rationale**: 
  - SetValue は Expr と WordRef の2バリアントのみ
  - 既存の match 式で自然に処理可能
  - 新規メソッドは過剰設計
- **Trade-offs**: 
  - メリット: 最小変更、既存パターン維持
  - デメリット: なし
- **Follow-up**: なし

### Decision: エスケープ処理なし

- **Context**: 単語名の出力時にエスケープが必要か判断
- **Alternatives Considered**:
  1. エスケープなし（直接出力）
  2. `name.escape_default()` でエスケープ
- **Selected Approach**: Option 1 - エスケープなし
- **Rationale**:
  - 単語名はXID識別子のみ（文法制約）
  - 既存の `Action::WordRef` も同様にエスケープなし
  - 一貫性と簡潔性を優先
- **Trade-offs**:
  - メリット: シンプル、既存パターンと一致
  - デメリット: なし（文法レベルで安全性担保）
- **Follow-up**: なし

## Risks & Mitigations

- **リスクなし**: 
  - 既存パターンの単純な適用
  - テスト期待値が定義済み
  - 影響範囲が限定的（単一メソッド内の分岐）

## References

- [pasta_lua/src/code_generator.rs](../../crates/pasta_lua/src/code_generator.rs) — 実装対象ファイル
- [pasta_core/src/parser/grammar.pest](../../crates/pasta_core/src/parser/grammar.pest) — 文法定義（XID識別子制約）
- [pasta_lua/tests/fixtures/sample.lua](../../crates/pasta_lua/tests/fixtures/sample.lua) — テスト期待値
- [gap-analysis.md](./gap-analysis.md) — 詳細なギャップ分析

