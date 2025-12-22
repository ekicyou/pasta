# Implementation Plan

## タスク一覧

### Major Task 1: SPECIFICATION.md の式（Expression）セクション更新

- [x] 1.1 1.3節「式の制約（設計方針）」を「式（Expression）のサポート」に改名し、式の正式採用を記載
  - SPECIFICATION.md line 25-50 の「式の制約」セクション全体を置換
  - セクション名を「### 1.3 式（Expression）のサポート」に変更
  - 「式を記述できません」という説明を削除
  - 式の構文をテーブル形式（式、項、二項演算、演算子）で説明
  - pasta2.pest の `expr`, `term`, `bin`, `bin_op` 規則を参照
  - 算術・比較・論理演算の使用例を追加（`＄count＝10 + 5`, `＄result＝＄a * ＄b`, `＠func（＄x + 1）`）
  - _Requirements: 1.1, 2.3, 7.1_

---

### Major Task 2: SPECIFICATION.md の変数セクション（ローカル・グローバル）更新

- [x] 2.1 9.1節「変数型」を拡張し、ローカル変数とグローバル変数の有効範囲を正確に記載
  - SPECIFICATION.md line 979-995 の「グローバル変数」「ローカル変数」セクションを更新
  - グローバル変数スコープを「ファイル全体」→「永続的」に変更
  - ローカル変数スコープを「親ローカルシーン内」→「一連のシーンが終わるまで」に変更
  - 各セクションに var_ref / var_set の pasta2.pest 規則を記載
  - 使用例を追加（ローカル変数、グローバル変数の代入と参照）
  - _Requirements: 2.4, 5.2, 7.2_

---

### Major Task 3: SPECIFICATION.md の Call セクション（グローバルシーン参照削除）更新

- [x] 3.1 4.1節「Call ターゲットの形式」を更新し、グローバルシーン参照パターンを削除
  - SPECIFICATION.md line 593-630 の「Call の詳細仕様」セクションを確認
  - パターン1（グローバルシーン参照 `＊会話` 形式）を削除
  - パターン2（ローカルシーン参照）を「シーン参照」に改名
  - セマンティクスを「アクションスコープから参照できるローカル・グローバルシーン全てから選択」に更新
  - パターン3（動的ターゲット）は保持
  - グローバル Call（`>*id`）のサポート中止を明記
  - _Requirements: 2.11, 5.3_

---

### Major Task 4: GRAMMAR.md の式（Expression）セクション更新

- [x] 4.1 「式の制約（重要）」セクション全体を「式（Expression）のサポート」に置換
  - GRAMMAR.md line 238-260 の「式の制約」セクションを削除
  - 「パスタスクリプトでは式を記述できます」に変更
  - 基本構文セクションを追加（算術式、変数含む式、関数引数での式の例）
  - 対応演算子テーブルを追加（算術、比較、論理）
  - SPECIFICATION.md 1.3節への参照を記載
  - _Requirements: 3.6, 7.6_

---

### Major Task 5: GRAMMAR.md の変数スコープ説明更新

- [x] 5.1 変数スコープテーブル（line 221-225）を更新し、有効範囲を正確に記載
  - ローカル変数の有効範囲を「親シーン内」→「一連のシーンが終わるまで」に変更
  - グローバル変数の有効範囲を「ファイル全体」→「永続的」に変更
  - _Requirements: 3.4, 5.2_

- [x] 5.2 変数参照セクション（line 228-235）を確認し、有効範囲説明が反映されているか検証
  - 既存の変数参照説明は保持（`＄変数名` の展開説明）
  - 必要に応じて ローカル・グローバル区別の説明を追加
  - _Requirements: 3.4_

---

### Major Task 6: GRAMMAR.md のスコープ明示参照構文削除

- [x] 6.1 (P) Call ターゲットの形式テーブル（line 278-297）を更新し、グローバルシーン行を削除
  - 現在のテーブルから「グローバルシーン `＞＊シーン名`」行を削除
  - ローカルシーン行を「シーン参照」に改名
  - 説明を「アクションスコープから参照できるローカル・グローバルシーン全てから選択」に更新
  - 動的ターゲット行は保持
  - _Requirements: 3.2, 5.3_

- [x] 6.2 (P) GRAMMAR.md 内で `@*id` または `>*id` の記載がないか確認
  - grep検索で `@\*`, `>\*` を検索
  - 削除対象の構文説明が残存していないか確認
  - 保持対象（`@単語`, `>シーン`）は保持確認
  - _Requirements: 3.5, 6.4_

---

### Major Task 7: comprehensive_control_flow2.pasta の修正

- [x] 7.1 line 63 の `@＊天気` を `@天気` に修正
  - ファイル: `tests/fixtures/comprehensive_control_flow2.pasta`
  - 対象行: 「さくら　：＠場所　の天気は＠＊天気　だね。」
  - 修正: `@＊天気` → `@天気` に変更（スコープ兼用参照に統一）
  - _Requirements: 4.7, 6.7_

- [x] 7.2 末尾の Rune ブロック（line 65-67）に有効な関数定義例を追加
  - ファイル: `tests/fixtures/comprehensive_control_flow2.pasta`
  - 現在: ｀｀｀rune と ｀｀｀ の間が空
  - 修正: 以下の関数定義を追加
    ```rune
    fn is_even(n) => n % 2 == 0
    fn greet(name) => "Hello, " + name
    ```
  - _Requirements: 4.10, 6.7_

---

### Major Task 8: ドキュメント修正の整合性検証

- [x] 8.1 (P) 修正後の SPECIFICATION.md が pasta2.pest と一致することを目視確認
  - 式セクションの expr, term, bin, bin_op 説明が pasta2.pest 定義と一致確認
  - 変数セクションの有効範囲説明（永続的 / 一連のシーンが終わるまで）の正確性確認
  - Call セクションでグローバルシーン参照パターンが削除されているか確認
  - 廃止構文（Jump文、全角\\）の使用例がないか確認
  - _Requirements: 1.1, 2.1, 5.1, 6.1_

- [x] 8.2 (P) 修正後の GRAMMAR.md が pasta2.pest と一致することを目視確認
  - 式セクションの例が pasta2.pest expr 規則と一致確認
  - 変数スコープテーブルの表現が SPECIFICATION.md と一貫性確認
  - Call ターゲットテーブルでグローバルシーン行が削除されているか確認
  - スコープ明示参照構文（@*id, >*id）の説明がないか確認
  - _Requirements: 1.1, 3.1, 5.1, 6.4_

---

### Major Task 9: comprehensive_control_flow2.pasta のパース可能性検証

- [x] 9.1 修正後の comprehensive_control_flow2.pasta が pasta2.pest でエラーなくパース可能か確認
  - ターミナルで `cargo test` 実行
  - comprehensive_control_flow2.pasta 関連テストが全てパス確認
  - 新規エラーが導入されていないことを確認
  - _Requirements: 4.1, 4.6, 4.10_

---

### Major Task 10: SPECIFICATION.md 内の廃止構文確認・削除

- [x] 10.1 (P) SPECIFICATION.md で廃止構文（Jump文、全角\\、全角[]）の説明が残存していないか確認
  - grep検索で `？`, `全角\\`, `全角[]` を検索
  - Jump文（？マーカー）関連の説明があれば「廃止」と明記
  - さくらスクリプトセクションで全角\\ の使用例がないか確認
  - さくらスクリプト引数括弧で全角[] の説明がないか確認
  - _Requirements: 6.1, 6.2, 6.3_

---

### Major Task 11: GRAMMAR.md 内の廃止構文確認・削除

- [x] 11.1 (P) GRAMMAR.md で廃止構文（Jump文、全角\\、全角[]）が記載されていないか確認
  - grep検索で `？`, `全角\\`, `全角[]` を検索
  - Jump文を「使用可能な構文」として記載していないか確認
  - さくらスクリプトマーカーで全角\\ が「有効」として記載されていないか確認
  - さくらスクリプト引数括弧で全角[] が「有効」として記載されていないか確認
  - _Requirements: 6.4, 6.5, 6.6_

---

## Task Summary

- **Total**: 11 major tasks, 16 sub-tasks
- **Requirements Coverage**: All 7 requirements mapped
  - Requirement 1 (権威性確立): 1.1, 3.1, 8.1, 8.2
  - Requirement 2 (SPECIFICATION.md整合性): 2.1, 3.1, 10.1
  - Requirement 3 (GRAMMAR.md整合性): 4.1, 5.1, 5.2, 6.1, 6.2, 11.1
  - Requirement 4 (comprehensive_control_flow2.pasta パース可能性): 7.1, 7.2, 9.1
  - Requirement 5 (ドキュメント間一貫性): 5.1, 5.2, 6.1, 6.2, 8.1, 8.2
  - Requirement 6 (廃止構文削除): 10.1, 11.1
  - Requirement 7 (新規構文反映): 1.1, 2.1, 4.1, 5.1

- **Task Types**:
  - Implementation: 7.1, 7.2 (修正作業)
  - Verification: 8.1, 8.2, 9.1, 10.1, 11.1 (検証・確認)
  - Average scope: 1-2 hours per sub-task

- **Parallel Opportunities** (P markers):
  - 6.1, 6.2: GRAMMAR.md Call セクション・スコープ参照削除（独立）
  - 8.1, 8.2: SPECIFICATION.md・GRAMMAR.md 検証（独立）
  - 10.1, 11.1: 廃止構文確認（独立）

---

## Next Steps

1. **Review & Approval**: Review tasks and confirm readiness
2. **Implementation**: Execute tasks 1-7 (修正作業)
3. **Verification**: Execute tasks 8-11 (検証・確認)
4. **Validation**: `cargo test` を実行して既存テストが継続パス確認
