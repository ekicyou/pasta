# Implementation Plan

## Task Format Template

Use whichever pattern fits the work breakdown:

### Major task only
- [ ] {{NUMBER}}. {{TASK_DESCRIPTION}}{{PARALLEL_MARK}}
  - {{DETAIL_ITEM_1}} *(Include details only when needed. If the task stands alone, omit bullet items.)*
  - _Requirements: {{REQUIREMENT_IDS}}_

### Major + Sub-task structure
- [ ] {{MAJOR_NUMBER}}. {{MAJOR_TASK_SUMMARY}}
- [ ] {{MAJOR_NUMBER}}.{{SUB_NUMBER}} {{SUB_TASK_DESCRIPTION}}{{SUB_PARALLEL_MARK}}
  - {{DETAIL_ITEM_1}}
  - {{DETAIL_ITEM_2}}
  - _Requirements: {{REQUIREMENT_IDS}}_ *(IDs only; do not add descriptions or parentheses.)*

> **Parallel marker**: Append ` (P)` only to tasks that can be executed in parallel. Omit the marker when running in `--sequential` mode.
>
> **Optional test coverage**: When a sub-task is deferrable test work tied to acceptance criteria, mark the checkbox as `- [ ]*` and explain the referenced requirements in the detail bullets.

---

## Tasks

- [ ] 1. (P) resume_until_validローカル関数の実装
- [ ] 1.1 (P) コルーチンのnil yieldスキップループを実装
  - `coroutine.resume(co, ...)`を呼び出し、ok、valueを取得
  - ok=falseの場合は即座にok=false、エラーメッセージを返す
  - ok=trueかつvalue≠nilの場合はループ終了、ok=true、valueを返す
  - ok=trueかつvalue=nilかつstatus=deadの場合はループ終了、ok=true、nilを返す
  - ok=trueかつvalue=nilかつstatus≠deadの場合は引数なしで再度resumeしループ継続
  - _Requirements: 1.1, 1.2_

- [ ] 1.2 (P) resume_until_validのLuaDoc注釈を記述
  - 関数の目的（nil yieldスキップ）を説明
  - @param co thread、@param ... any（初回resume引数）を記述
  - @return boolean ok、@return any valueを2行に分けて記述
  - _Requirements: 1.4_

- [ ] 2. EVENT.fireへのresume_until_valid統合
- [ ] 2.1 thread分岐でresume_until_validを呼び出し
  - 既存の`coroutine.resume(result, act)`をresume_until_validに置き換え
  - ok=falseの場合、set_co_scene(result)してerror(yielded_value)で伝搬
  - ok=trueの場合、set_co_scene(result)してRES.ok(yielded_value)を返す
  - _Requirements: 2.1, 2.4, 3.1, 3.2_

- [ ] 2.2 (P) 既存ハンドラタイプ（文字列/nil）の動作確認
  - 文字列戻り値のハンドラがRES.ok(result)で処理されることを確認
  - nil戻り値がRES.no_content()に変換されることを確認
  - _Requirements: 2.2, 2.3_

- [ ] 3. (P) テストケースの実装
- [ ] 3.1 (P) resume_until_valid単体テスト
  - nil yieldしてsuspendedのコルーチンが再resumeされることを確認
  - 有効値を返したらループ終了を確認
  - dead状態でnilを返したらループ終了を確認（空シーン）
  - エラー発生時にok=false、エラーメッセージを返すことを確認
  - テスト配置: `event_coroutine_test.lua`にdescribe("resume_until_valid")ブロック追加
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 3.2 (P) EVENT.fire統合テスト
  - nil yieldするthread型ハンドラがresume_until_validで処理されることを確認
  - 有効値を返すthread型ハンドラが正常動作することを確認
  - エラー発生時にset_co_sceneでcloseされることを確認
  - テスト配置: `event_coroutine_test.lua`の既存ブロックを拡張
  - _Requirements: 2.1, 2.4, 3.1, 3.2_

- [ ] 3.3 (P) 後方互換性テスト
  - 既存の文字列ハンドラが正常動作することを確認
  - 既存のnil戻り値が204 No Contentに変換されることを確認
  - テスト配置: `event_coroutine_test.lua`の既存ブロックを拡張
  - _Requirements: 2.2, 2.3_

- [ ]* 3.4 受け入れ基準カバレッジ検証（オプション）
  - 1.1: nil + suspended → 再resume
  - 1.2: 有効値またはdead → ループ終了
  - 1.3: エラー伝搬とclose
  - 1.4: ローカル関数実装
  - 2.1-2.4: EVENT.fire統合、後方互換、suspended保存
  - 3.1-3.3: エラー時close、エラー伝搬、dead時close
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3_

- [ ] 4. ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（今回影響なし）
  - SPECIFICATION.md - 言語仕様の更新不要（ランタイム層の防御処理）
  - GRAMMAR.md - 文法リファレンスの同期不要
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - クレートREADME - API変更なし（ローカル関数追加のみ）
  - steering/* - workflow.mdのDoD確認
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3_

---

## Summary

- **合計**: 4メジャータスク、11サブタスク
- **要件カバレッジ**: 全11受け入れ基準（1.1-1.4, 2.1-2.4, 3.1-3.3）
- **並列実行可能**: タスク1、タスク3の各サブタスク（ファイル競合なし）
- **推定工数**: 1-3時間/サブタスク、合計11-33時間
