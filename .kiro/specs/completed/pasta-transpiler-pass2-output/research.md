# Research & Design Decisions

---
**Feature**: pasta-transpiler-pass2-output  
**Discovery Scope**: Extension  
**Key Findings**:
- `transpile_pass2()` 関数（170-217行目）が唯一の修正対象
- `LabelRegistry` は完全な情報を保持（ID、fn_path）、データ構造変更不要
- Rune構文は参照フィクスチャで動作確認済み
---

## Summary

本機能は既存トランスパイラーの単一関数修正であり、Light Discoveryプロセスを適用。`transpile_pass2()` 関数の出力内容のみ変更し、データ構造・Pass 1・テストフレームワークは不変。

## Research Log

### Extension Point Analysis
- **Context**: Pass 2コード生成の修正箇所を特定
- **Sources Consulted**: 
  - `crates/pasta/src/transpiler/mod.rs` (163-219行目)
  - `gap-analysis.md` セクション1.2
- **Findings**:
  - `transpile_pass2()` 関数が `pub mod pasta` を生成（170-217行目）
  - `writeln!(writer, ...)` パターンで行単位出力
  - エラーハンドリングは `.map_err(|e| PastaError::io_error(e.to_string()))?` で統一
- **Implications**: 既存パターンを踏襲、単一関数内で完結

### LabelRegistry Data Structure
- **Context**: Pass 2で必要なデータの確認
- **Sources Consulted**: 
  - `crates/pasta/src/transpiler/label_registry.rs` (1-100行目)
  - `requirements.md` Req 5 AC3
- **Findings**:
  - `LabelInfo` 構造体に必要な全情報が含まれる:
    - `id: i64` - ラベルID
    - `fn_path: String` - `"crate::会話_1::__start__"` 形式
    - `attributes: HashMap<String, String>` - P1機能用（現在未使用）
  - `registry.all_labels()` イテレーターで全ラベルを取得可能
- **Implications**: データ構造変更不要、既存APIのみで実装可能

### Rune Syntax Verification
- **Context**: 関数ポインタ返却の構文確認
- **Sources Consulted**:
  - `.kiro/specs/pasta-transpiler-pass2-output/reference_comparison.rn` (112-127行目)
  - `crates/pasta/tests/fixtures/comprehensive_control_flow.transpiled.rn` (105-131行目)
- **Findings**:
  - 関数ポインタ構文: `1 => crate::メイン_1::__start__,` （呼び出しではなく名前のみ）
  - エラーケースのクロージャ: `_ => |_ctx, _args| { yield pasta_stdlib::Error(...); },`
  - 両方とも参照フィクスチャに記載済み
- **Implications**: 構文検証不要、実装に直接使用可能

### Testing Infrastructure
- **Context**: テスト検証パターンの変更範囲
- **Sources Consulted**:
  - `crates/pasta/tests/two_pass_transpiler_test.rs` (1-148行目)
  - `gap-analysis.md` セクション1.4
- **Findings**:
  - 4つのテストケースが存在:
    - `test_two_pass_transpiler_to_vec()`
    - `test_two_pass_transpiler_to_string()`
    - `test_transpile_to_string_helper()`
    - `test_multiple_files_simulation()`
  - 現在の検証: `assert!(output.contains("for a in crate::会話_1::__start__(ctx, args)"))`
  - 変更後の検証: `__pasta_trans2__` モジュール存在、`label_selector()` 呼び出し確認
- **Implications**: 検証パターンのみ変更、テスト構造は不変

## Design Decisions

### Decision: 単一関数内での完全な実装
- **Context**: `transpile_pass2()` 関数をヘルパー関数に分割するか、単一関数で実装するか
- **Alternatives Considered**:
  1. Option A（単一関数）: `transpile_pass2()` 内で全てのコード生成を実施
  2. Option B（分割）: `generate_pasta_trans2_module()` と `generate_pasta_module()` を別関数に
- **Selected Approach**: Option A（単一関数）
- **Rationale**: 
  - 生成コードは約70行と許容範囲内
  - 既存パターン（`transpile_pass1()` も単一関数）を踏襲
  - 将来的に複雑化した場合のみリファクタリングを検討（YAGNI原則）
- **Trade-offs**: 
  - 利点: 最小限の変更、既存構造維持、リスク最小化
  - 欠点: 関数の長さが70行程度に増加
- **Follow-up**: P1機能（属性フィルタ）追加時に再評価

### Decision: エラーハンドリングパターンの継続
- **Context**: 新規生成コードのエラーハンドリング方法
- **Alternatives Considered**:
  1. 既存の `.map_err(|e| PastaError::io_error(e.to_string()))?` パターンを継続
  2. 新しいエラータイプ（`CodeGenerationError` など）を導入
- **Selected Approach**: 既存パターンを継続
- **Rationale**: 
  - コード生成エラーは全て I/O エラーとして扱う既存設計に準拠
  - 新規エラータイプは不要（I/O 失敗以外のケースがない）
- **Trade-offs**: 
  - 利点: 既存パターンとの一貫性、変更最小化
  - 欠点: なし（現状で十分）

## Risks & Mitigations
- **Risk 1**: Rune構文エラー → 参照フィクスチャで動作確認済み、リスク低
- **Risk 2**: テスト失敗 → 段階的にテスト更新、出力を目視確認
- **Risk 3**: 後方互換性 → Pass 1不変、LabelRegistry不変、影響なし

## References
- [Gap Analysis](./gap-analysis.md) - 実装アプローチ評価（Option A推奨）
- [Requirements Document](./requirements.md) - 5要件定義（EARS形式）
- [Reference Fixture](./reference_comparison.rn) - 正誤比較用実装例
- [Transpiler Source](../../crates/pasta/src/transpiler/mod.rs) - 現在の実装（170-217行目）
