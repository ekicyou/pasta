# Tasks - fix-actor-marker-test-confusion

## Overview
アクターマーカーテストの混同問題を修正するための実装タスク。
設計フェーズは省略（単純なテストロジック修正のため）。

---

## Task 1: テストロジックの修正

### 1.1 ヘルパー関数の追加
- [x] `contains_global_actor_dictionary(content: &str, actor_name: &str) -> bool` 関数を追加
- [x] 行頭パターン検出ロジック: `content.starts_with(pattern) || content.contains(newline_pattern)`

### 1.2 テスト関数の修正
- [x] テスト名を `test_event_files_do_not_contain_global_actor_dictionary` に変更
- [x] 6箇所のアサーションをヘルパー関数を使用した形式に置換
- [x] エラーメッセージを「グローバルアクター辞書定義」に明確化

### 1.3 テスト実行確認
- [x] `cargo test -p pasta_sample_ghost` で全テストpassを確認（28 passed）

### 1.4 integration_test.rs の修正
- [x] 同等のヘルパー関数を追加
- [x] 3箇所のアサーションを修正
- [x] コメントを「グローバルアクター辞書定義」に明確化

---

## Task 2: ドキュメント整合性の確認

### 2.1 ドキュメント更新
- [ ] TEST_COVERAGE.md への反映（必要に応じて）
