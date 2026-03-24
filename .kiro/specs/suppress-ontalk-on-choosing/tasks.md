# Implementation Plan

## Tasks: suppress-ontalk-on-choosing

---

- [ ] 1. has_status ヘルパー関数を virtual_dispatcher.lua に追加する
  - `calculate_next_talk_time()` の直後・`create_scene_thread()` の直前に、モジュールローカル関数 `has_status(status, keyword)` を追加する
  - `string.find` の第 4 引数 `true`（プレーンテキストモード）を使い、パターン文字が誤解釈されないようにする
  - `status` が `nil` の場合は `false` を返す nil-safe 実装にする
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 2. check_talk の talking ガードを has_status に置き換え、choosing ガードを追加する
  - 既存の `act.req.status == "talking"` による完全一致検査を `has_status(..., "talking")` に置き換える（CSV 複合値への対応）
  - talking ガードの直後に `has_status(..., "choosing")` による choosing ガードを追加する
  - ガード節を関数冒頭に配置したまま維持し、タイマー初期化・到達チェックより前に実行されることを確認する（タイマー非消費の保証）
  - _Requirements: 1.1, 1.2, 3.4_

- [ ] 3. check_hour の talking ガードを has_status に置き換え、choosing ガードを追加する
  - 既存の `act.req.status == "talking"` による完全一致検査を `has_status(..., "talking")` に置き換える（CSV 複合値への対応）
  - talking ガードの直後に `has_status(..., "choosing")` による choosing ガードを追加する
  - ガード節を「正時到達チェック後・`next_hour_unix` 更新前」の位置に維持し、正時タイムスタンプが消費されないことを確認する
  - _Requirements: 2.1, 2.2, 3.4_

- [ ] 4. Lua BDD テストで choosing 抑制を検証する
- [ ] 4.1 (P) choosing 単独 / CSV での OnTalk スキップと CSV talking を検証する
  - `check_talk` に対して `status = "choosing"` のテストを追加（T1）
  - `check_talk` に対して `status = "talking,choosing,balloon(0=2)"` の CSV テストを追加（T3）
  - `check_talk` に対して `status = "talking,balloon(0=0)"` の CSV talking テストを追加（T5）
  - 既存の `status = "idle"` をベースとした「スキップしない」逆テスト（T7）でリグレッションがないことを確認する
  - _Requirements: 4.1, 4.3, 4.4_

- [ ] 4.2 (P) choosing 単独 / CSV での OnHour スキップを検証する
  - `check_hour` に対して `status = "choosing"` のテストを追加（T2）
  - `check_hour` に対して `status = "talking,choosing,balloon(0=2)"` の CSV テストを追加（T4）
  - `check_hour` に対して `status = "talking,balloon(0=0)"` の CSV talking テストを追加（T6）
  - _Requirements: 4.2, 4.3, 4.4_

- [ ] 4.3 (P) タイマー非消費を検証する
  - `_get_internal_state()` で `next_talk_time` を before/after 比較し、`status = "choosing"` で呼んだ後もタイマーが更新されていないことを確認する（T8）
  - `_get_internal_state()` で `next_hour_unix` を before/after 比較し、`status = "choosing"` で呼んだ後もタイムスタンプが更新されていないことを確認する（T9）
  - _Requirements: 1.2, 2.2_

- [ ] 5. Rust 統合テストで choosing 抑制を検証する
  - `virtual_event_config_test.rs` で `status = "choosing"` を渡したとき `check_hour` / `check_talk` が共に `nil` を返すことを確認する
  - `status = "talking,choosing,balloon(0=2)"` の CSV 複合値でも同様にスキップされることを確認する
  - _Requirements: 4.1, 4.2, 4.3_
