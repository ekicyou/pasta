# Implementation Tasks: scene-actor-initialization-refactoring

## Summary
トランスパイラーのアクター初期化コード生成ロジックを変更し、出力順序を変え、呼び出し形式を新しいAPI形式に更新し、既存テストを修正する。全5要件をカバーする4つの主要タスク（8つのサブタスク）。

---

## Implementation Tasks

### 1. アクター初期化コード生成ロジックの実装

- [x] 1.1 (P) `generate_local_scene()` メソッドのアクター初期化ブロック出力順序を変更
  - `local args = { ... }` の直後に `act:clear_spot()` と `act:set_spot()` ブロックを挿入
  - `PASTA.create_session()` をアクター初期化ブロックの後に移動（現在は前）
  - counter == 0（`__start__` シーン）かつ actors が非空の場合のみ出力
  - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [x] 1.2 (P) `set_spot()` 出力形式を新しいAPI形式に変更
  - 現在の形式 `act.アクター名:set_spot(位置)` を廃止
  - 新しい形式 `act:set_spot("アクター名", 位置)` に変更（アクター名を文字列リテラル化）
  - 既存の actor.number（位置番号）ロジックは保持
  - _Requirements: 3.1, 3.2, 3.3_

### 2. アクター初期化コード出力の条件制御

- [x] 2.1 (P) `clear_spot()` 呼び出しの条件付き出力を実装
  - アクターが存在する場合（!actors.is_empty()）に限り `act:clear_spot()` を出力
  - `clear_spot()` をアクター初期化ブロックの先頭に配置
  - 条件チェック: counter == 0 && !actors.is_empty()
  - _Requirements: 2.1, 2.2_

- [x] 2.2 (P) ローカルシーン関数のアクター出力制御を確認
  - counter != 0（名前付きローカルシーン）の場合、アクター初期化ブロック全体をスキップ
  - `__start__` シーン（counter == 0）のみが対象であることを確認
  - _Requirements: 4.2_

### 3. 既存機能との互換性確認

- [x] 3.1 (P) 既存テストを新形式に合わせて修正（複数/単数アクター）
  - `test_set_spot_multiple_actors`: 検証文字列を新形式 `act:set_spot("さくら", 0)` に変更
  - `test_set_spot_single_actor`: 同様に新形式に変更
  - 各テストに `clear_spot()` 検証アサーション追加: `assert!(lua_code.contains("act:clear_spot()"))`
  - 出力順序検証: `create_session()` より前に `clear_spot()` と `set_spot()` が来ていることを確認
  - _Requirements: 4.1, 4.2_

- [x] 3.2 (P) 既存テストを新形式に合わせて修正（アクター非存在）
  - `test_set_spot_empty_actors`: アクター非存在時の出力確認（set_spot と clear_spot 両方出力されないこと）
  - `assert!(!lua_code.contains("set_spot"))` と `assert!(!lua_code.contains("act:clear_spot()"))` を検証
  - _Requirements: 2.2, 4.2_

- [x] 3.3 (P) 既存テストを新形式に合わせて修正（明示的番号付き）
  - `test_set_spot_with_explicit_number`: 複数アクター＋明示的位置番号のテストを新形式に更新
  - 検証文字列を新形式に変更、clear_spot 検証追加
  - 位置番号が正しく反映されていることを確認（0, 2（明示的）など）
  - _Requirements: 3.1, 3.2, 3.3, 4.1, 4.2_

### 4. 既存機能への影響確認

- [x] 4.1 (P) 互換性テスト: アクター辞書定義の出力に影響なし
  - グローバルシーン内の `PASTA.create_actor()` 呼び出し出力に変更がないことを確認
  - 既存のグローバルスコープテストが変わらず合格することを検証
  - _Requirements: 5.1_

- [x] 4.2 (P) 互換性テスト: 会話アクション（talk/word）の出力に影響なし
  - シーン内の `act.アクター名:talk()` と `act.アクター名:word()` 出力が変わらないことを確認
  - 既存の動作テストが合格することで検証
  - _Requirements: 5.2_

- [x] 4.3 (P) 互換性テスト: シーン呼び出し（act:call）の出力に影響なし
  - `act:call()` 形式のシーン呼び出し出力に変更がないことを確認
  - 既存の呼び出しテストが合格することで検証
  - _Requirements: 5.3_

---

## Task Execution Notes

- **Parallel Execution**: タスク 1.1, 1.2, 2.1, 2.2, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3 はすべて並列実行可能（依存なし）
- **Implementation Order**: 推奨順序は 1.1 → 1.2 → 2.1 → 2.2 → 3系 → 4系（要件の論理順）
- **Test Validation**: 各テスト修正後は `cargo test -p pasta_lua` で検証
- **Scope**: すべての変更は `code_generator.rs` の `generate_local_scene()` メソッド内に局所化
