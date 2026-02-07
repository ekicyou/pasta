# Implementation Plan

## Task Format Template

> Use whichever pattern fits the work breakdown:
>
> ### Major task only
> - [ ] {{NUMBER}}. {{TASK_DESCRIPTION}}{{PARALLEL_MARK}}
>   - {{DETAIL_ITEM_1}} *(Include details only when needed. If the task stands alone, omit bullet items.)*
>   - _Requirements: {{REQUIREMENT_IDS}}_
>
> ### Major + Sub-task structure
> - [ ] {{MAJOR_NUMBER}}. {{MAJOR_TASK_SUMMARY}}
> - [ ] {{MAJOR_NUMBER}}.{{SUB_NUMBER}} {{SUB_TASK_DESCRIPTION}}{{SUB_PARALLEL_MARK}}
>   - {{DETAIL_ITEM_1}}
>   - {{DETAIL_ITEM_2}}
>   - _Requirements: {{REQUIREMENT_IDS}}_ *(IDs only; do not add descriptions or parentheses.)*
>
> **Parallel marker**: Append ` (P)` only to tasks that can be executed in parallel. Omit the marker when running in `--sequential` mode.
>
> **Optional test coverage**: When a sub-task is deferrable test work tied to acceptance criteria, mark the checkbox as `- [ ]*` and explain the referenced requirements in the detail bullets.

---

## Implementation Tasks

### Overview
本実装計画は、`transfer_req_to_var()` メソッドの追加とテストスイートの実装を2つのメジャータスクに分割している。設計レビューで確認済みの通り、既存の `transfer_date_to_var()` パターンを完全踏襲するため、実装は1ファイルへの1メソッド追加のみで完結する。

---

- [ ] 1. SHIORI_ACT クラスへの `transfer_req_to_var()` メソッド追加
- [ ] 1.1 Referenceパラメーター転記ロジックの実装
  - `act.req.reference[0]`〜`[9]` のループ処理による転記
  - 全角キー（`ｒ０`〜`ｒ９`）と半角キー（`r0`〜`r9`）の両方に同一値を設定
  - `reference[N]` が `nil` の場合はスキップ（varキー未設定）
  - 0-indexed 配列アクセス（SHIORIプロトコル準拠）の正確な実装
  - _Requirements: 1.1, 1.2, 1.3, 4.2, 4.3_

- [ ] 1.2 イベントメタデータ転記ロジックの実装
  - `act.req.id` → `act.var.req_id` への転記
  - `act.req.base_id` → `act.var.req_base_id` への転記（nil の場合もスキップ）
  - _Requirements: 2.1, 2.2_

- [ ] 1.3 メソッド構造とガード句の実装
  - 関数シグネチャ: `function SHIORI_ACT_IMPL.transfer_req_to_var(self)`
  - 先頭ガード句: `if not self.req then return self end`
  - 戻り値: `return self`（メソッドチェーン対応）
  - LDoc コメント（既存メソッドのフォーマットに準拠）
  - `transfer_date_to_var()` の直後に配置（ファイル内位置）
  - _Requirements: 1.4, 3.1, 3.3, 3.5, 5.3_

- [ ] 2. テストスイートの実装
- [ ] 2.1 (P) 基本転記テストの実装
  - 全角キー（`ｒ０`〜`ｒ９`）への転記検証（reference[0]〜[9] の全10要素）
  - 半角キー（`r0`〜`r9`）への転記検証（同一値の重複設定確認）
  - イベントメタデータ（`req_id`, `req_base_id`）の転記検証
  - _Requirements: 1.1, 1.3, 2.1, 2.2_

- [ ] 2.2 (P) 境界条件・ガード句テストの実装
  - `req = nil` の場合のガード句動作検証（クラッシュしない、`self` を返す）
  - 部分欠落 reference（疎配列）での正常動作確認（例: reference[0], [2] のみ存在）
  - メソッドチェーンの検証（戻り値が `self` であること）
  - _Requirements: 1.2, 1.4, 3.5_

- [ ] 2.3 (P) 統合・副作用テストの実装
  - 未呼出時の var 未設定検証（`transfer_req_to_var()` を呼ばない場合、req 由来キーが存在しない）
  - `transfer_date_to_var()` との共存検証（両メソッド呼び出し後、全22+16=38キーが衝突なく存在）
  - _Requirements: 3.2, 3.4, 5.1, 5.2_

---

## Requirements Coverage Matrix

| Requirement | Tasks Covering |
|-------------|----------------|
| 1.1 | 1.1, 2.1 |
| 1.2 | 1.1, 2.2 |
| 1.3 | 1.1, 2.1 |
| 1.4 | 1.3, 2.2 |
| 2.1 | 1.2, 2.1 |
| 2.2 | 1.2, 2.1 |
| 3.1 | 1.3 |
| 3.2 | 2.3 |
| 3.3 | 1.3 |
| 3.4 | 2.3 |
| 3.5 | 1.3, 2.2 |
| 4.1 | *(DSL変更不要、既存パイプライン)* |
| 4.2 | 1.1 |
| 4.3 | 1.1, 2.1 |
| 4.4 | *(既存動作維持、変更なし)* |
| 5.1 | 2.3 |
| 5.2 | 2.3 |
| 5.3 | 1.3 |

---

## Implementation Notes

### Parallel Execution Strategy
- **Task 1 (実装)**: ボトルネック（1ファイルへの1メソッド追加）
- **Task 2 (テスト)**: サブタスク 2.1, 2.2, 2.3 はすべて並列実行可能
  - 各テストスイートは独立したテストファイル／テストケースとして実装
  - 共有リソース競合なし（lua_test フレームワークのテスト分離）
  - 実装完了（Task 1）後にすべて並列実行可能

### File Impact Analysis
- **変更**: `crates/pasta_lua/scripts/pasta/shiori/act.lua` (1メソッド追加)
- **新規**: `crates/pasta_lua/tests/transfer_req_to_var_test.rs` (テストスイート、lua_test スタイル)
- **影響なし**: パーサー層、DSL文法、既存ランタイムメソッド

### Risk Mitigation
- **既存パターン踏襲**: `transfer_date_to_var()` の実装を完全コピー＆修正することで、構造的リスクを最小化
- **ガード句の徹底**: `req = nil` 時の安全性を最優先
- **キー衝突チェック**: 設計フェーズで検証済み（date 16キー vs req 22キー、衝突なし）

### Definition of Done
- [ ] すべての acceptance criteria を満たすテストが全パス
- [ ] `digit_id_var_test.rs` の既存4テストが引き続き全パス（`＄０` 動作維持）
- [ ] LDoc コメントが既存スタイルに準拠
- [ ] コードが `transfer_date_to_var()` と構造的に一貫している
