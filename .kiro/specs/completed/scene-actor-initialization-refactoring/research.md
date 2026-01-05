# Research & Design Decisions

## Summary
- **Feature**: `scene-actor-initialization-refactoring`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - 変更対象は `code_generator.rs` の `generate_local_scene()` メソッド内部のみ
  - インターフェース変更なし、呼び出し元コード不変
  - 既存テストは検証文字列の更新のみ必要

## Research Log

### 既存コード構造の調査
- **Context**: アクター初期化コードの現在の出力位置と形式を特定
- **Sources Consulted**: 
  - [`pasta_lua/src/code_generator.rs`](c:\home\maz\git\pasta\crates\pasta_lua\src\code_generator.rs) L237-280
  - [`pasta_core/src/parser/ast.rs`](c:\home\maz\git\pasta\crates\pasta_core\src\parser\ast.rs) L314 (`SceneActorItem`)
- **Findings**:
  - 現在: `args` → `create_session()` → `set_spot()` × N の順序
  - `set_spot()` は `counter == 0` かつ `!actors.is_empty()` の場合のみ出力
  - 形式: `act.アクター名:set_spot(位置番号)`
- **Implications**: 出力順序の変更と形式変更は同一メソッド内で完結

### テスト修正範囲の調査
- **Context**: 既存テストへの影響範囲を特定
- **Sources Consulted**:
  - [`transpiler_integration_test.rs`](c:\home\maz\git\pasta\crates\pasta_lua\tests\transpiler_integration_test.rs) L849-958
- **Findings**:
  - 4つの関連テスト: `test_set_spot_multiple_actors`, `test_set_spot_single_actor`, `test_set_spot_empty_actors`, `test_set_spot_with_explicit_number`
  - 検証文字列 `"act.さくら:set_spot(0)"` → `"act:set_spot(\"さくら\", 0)"` への変更
  - 順序検証の追加は不要（実装後エラーで対応）
- **Implications**: テスト修正は実装後、エラー出力に応じて対応

## Architecture Pattern Evaluation

| Option              | Description                          | Strengths                          | Risks / Limitations | Notes    |
| ------------------- | ------------------------------------ | ---------------------------------- | ------------------- | -------- |
| **Option A (選択)** | 既存メソッド内で出力順序・形式を変更 | 最小限の変更、インターフェース不変 | テスト更新が必須    | 推奨     |
| Option B            | アクター初期化を独立メソッドに抽出   | 責務分離が明確                     | ファイル複雑化      | 過剰設計 |

## Design Decisions

### Decision: 既存メソッド内で出力順序・形式を変更
- **Context**: アクター初期化コードの出力位置と形式を変更する必要がある
- **Alternatives Considered**:
  1. Option A: `generate_local_scene()` メソッド内で変更（内部実装のみ）
  2. Option B: 新メソッド `generate_actor_initialization()` を追加
- **Selected Approach**: Option A
- **Rationale**: 
  - 変更範囲が最小（1メソッド内の約15行）
  - 既存パターンとの整合性が高い
  - テスト修正も検証文字列の変更のみで対応可能
- **Trade-offs**: 
  - ✅ 実装が直線的で理解しやすい
  - ✅ インターフェース変更なし（呼び出し元不変）
  - ❌ if/else分岐が若干増加
- **Follow-up**: 実装後のテストエラーを確認し、検証文字列を更新

## Risks & Mitigations
- **テスト失敗**: 実装後に既存テストが失敗 → 検証文字列を新形式に更新
- **出力順序の不整合**: 順序検証不足 → 目視確認 + フィクスチャー比較で対応
- **Luaランタイム未実装**: `act:clear_spot()` / `act:set_spot()` 未実装 → 別仕様で対応（本仕様はトランスパイラーのみ）

## References
- [gap-analysis.md](gap-analysis.md) - 実装ギャップ分析
- [requirements.md](requirements.md) - 要件定義
