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

## タスク一覧

### 1. SHIORI_ACT への日時転記メソッド追加

- [x] 1.1 (P) transfer_date_to_var() メソッドの実装
  - SHIORI_ACT_IMPL に transfer_date_to_var() メソッドを追加
  - req.date の全フィールド（year, month, day, hour, min, sec, wday）を var へ転記
  - unix, ns, yday, ordinal, num_days_from_sunday は転記対象外
  - req または req.date が存在しない場合は何もせず正常終了
  - メソッドチェーン用に self を返す
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 1.2 (P) 日本語変数マッピングの実装
  - var.年 ← "{year}年" 形式の文字列
  - var.月 ← "{month}月" 形式の文字列
  - var.日 ← "{day}日" 形式の文字列
  - var.時 ← "{hour}時" 形式の文字列（24時間制）
  - var.分 ← "{min}分" 形式の文字列
  - var.秒 ← "{sec}秒" 形式の文字列
  - _Requirements: 1.2, 1.4_

- [x] 1.3 (P) 曜日変換の実装
  - var.曜日 ← wday (0-6) を日本語曜日文言に変換
  - var.week ← wday (0-6) を英語曜日名に変換
  - 変換テーブル: 0="日曜日"/"Sunday", 1="月曜日"/"Monday", ..., 6="土曜日"/"Saturday"
  - _Requirements: 1.2, 1.4_

- [x] 1.4 (P) 12時間制変換の実装
  - var.時１２ ← hour (0-23) を午前/午後付き文字列に変換
  - 0時 → "深夜0時"
  - 1-11時 → "午前{hour}時"
  - 12時 → "正午"
  - 13-23時 → "午後{hour-12}時"
  - _Requirements: 1.2, 1.4_

### 2. virtual_dispatcher への転記呼び出し追加

- [x] 2.1 check_hour() 内での transfer_date_to_var() 呼び出し
  - OnHour 発火確定後（execute_scene 呼び出し前）に act:transfer_date_to_var() を呼び出し
  - 転記処理の失敗はログ出力のみ、シーン実行は継続
  - check_talk() では呼び出さない（OnHour のみ）
  - _Requirements: 2.1, 4.1, 4.2_

### 3. execute_scene への act 引き渡し修正

- [x] 3.1 execute_scene シグネチャの変更
  - execute_scene(event_name) → execute_scene(event_name, act) に変更
  - pcall(scene_fn, act) で act を渡すよう修正
  - scene_executor が設定されている場合は scene_executor(event_name, act) で act を渡す
  - _Requirements: 3.1, 3.2_

- [x] 3.2 check_hour および check_talk からの呼び出し修正
  - execute_scene("OnHour", act) に変更
  - execute_scene("OnTalk", act) に変更
  - _Requirements: 3.3_

### 4. テストの実装

- [x] 4.1 (P) transfer_date_to_var() の単体テスト
  - 正常系: 全フィールド転記確認（英語・数値型）
  - req 不在時の安全終了
  - req.date 不在時の安全終了
  - 日本語変数マッピング確認（年月日時分秒）
  - 曜日変換確認（wday 0-6 全パターン）
  - 12時間制変換確認（hour 0, 1, 11, 12, 13, 23 のケース）
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 4.2 (P) check_hour() の統合テスト
  - OnHour 発火時に transfer_date_to_var() が呼び出されることを確認
  - act.var に日時変数が設定されることを確認
  - _Requirements: 2.1_

- [x] 4.3 (P) execute_scene() の統合テスト
  - act がシーン関数に渡されることを確認
  - テスト用 scene_executor に act が渡されることを確認
  - _Requirements: 3.1, 3.2_

- [x] 4.4 (P) check_talk() の統合テスト
  - OnTalk では transfer_date_to_var() が呼び出されないことを確認
  - execute_scene("OnTalk", act) で act が渡されることを確認
  - _Requirements: 4.1, 3.3_

- [ ]* 4.5 E2E テスト（オプション）
  - OnSecondChange → OnHour → シーン関数での日時変数アクセス
  - シーン関数内で act.var.時, act.var.曜日 等にアクセス可能なことを確認
  - Acceptance Criteria 2.2 のユーザーシナリオ検証
  - _Requirements: 2.2, 4.3_

### 5. ドキュメント整合性の確認と更新

- [x] 5.1 ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（日本語フレンドリー原則への適合確認）
  - TEST_COVERAGE.md - 新規テストのマッピング追加（4.1-4.5）
  - crates/pasta_lua/README.md - SHIORI_ACT API の更新（transfer_date_to_var メソッドの追加）
  - .kiro/steering/lua-coding.md - 日本語変数マッピングパターンの追加検討
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3_
