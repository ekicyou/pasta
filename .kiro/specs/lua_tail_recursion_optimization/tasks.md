````markdown
# Implementation Tasks

## Overview
Lua 末尾再帰最適化機能の実装タスク。末尾呼び出し検出と条件付き return 生成ロジックを `code_generator.rs` に追加し、既存テストの互換性を維持しながら、Lua TCO を有効化する。

---

## Task Breakdown

- [x] 1. 末尾呼び出し判定ロジックの実装
- [x] 1.1 (P) `is_callable_item` ヘルパー関数を実装
  - `LocalSceneItem` enum に対する `matches!` マクロベースの判定関数
  - コメントで将来の拡張方法（`FnCall` など）を明示
  - テスト可能な単純な型判定ロジック
  - _Requirements: 1, 5_

- [x] 1.2 `generate_local_scene_items` メソッドを拡張
  - `rposition` で最後の呼び出し可能項目インデックスを計算
  - ループ内で現在インデックスと比較し `is_tail_call` フラグを判定
  - 末尾 `CallScene` に対してのみ `is_tail_call = true` を渡す
  - 既存の `last_actor` 追跡ロジックは変更なし
  - _Requirements: 1, 3_

- [x] 2. return 文の条件付き生成
- [x] 2.1 `generate_call_scene` メソッドを拡張
  - メソッドシグネチャに `is_tail_call: bool` パラメータを追加
  - `is_tail_call = true` の場合、`return act:call(...)` を出力
  - `is_tail_call = false` の場合、従来通り `act:call(...)` を出力
  - `Result<(), TranspileError>` エラーハンドリング維持
  - _Requirements: 2, 3_

- [x] 2.2 (P) `generate_local_scene_items` 内の `generate_call_scene` 呼び出しを更新
  - メソッド呼び出しに `is_tail_call` フラグを渡す
  - 末尾判定結果を正確に反映
  - _Requirements: 2_

- [x] 3. 既存テストの互換性確認と新規テストケース追加
- [x] 3.1 既存テスト実行とリグレッション確認
  - `cargo test --package pasta_lua` で既存テスト全件実行
  - 末尾以外の `CallScene` が `return` なしで出力されることを確認
  - 既存の動作ログと出力順序に変更がないことを確認
  - リグレッションが 0 件であることを検証
  - _Requirements: 3_

- [x] 3.2 単一呼び出しテストケース追加
  - `test_single_call_scene_gets_return` テストを実装
  - 単一の `act:call()` のみを含むシーン関数から `return act:call(...)` が生成されることを検証
  - _Requirements: 4_

- [x] 3.3 (P) 複数呼び出しテストケース追加
  - `test_multiple_call_scenes_only_last_gets_return` テストを実装
  - 複数の `act:call()` を含むシーン関数から、最後の呼び出しのみに `return` が付与されることを検証
  - _Requirements: 4_

- [x] 3.4 (P) 末尾以外のシーンテストケース追加
  - `test_call_scene_followed_by_action_no_return` テストを実装
  - `act:call()` の後に `ActionLine` が続く場合、`return` が生成されないことを検証
  - _Requirements: 4_

- [x] 3.5 (P) 呼び出しなしシーンテストケース追加
  - `test_no_call_scene_no_return` テストを実装
  - シーン関数に `act:call()` が含まれない場合、`return` が生成されないことを検証
  - _Requirements: 4_

- [x] 4. 統合テストとE2E検証
- [x] 4.1 テストフィクスチャの作成と統合テスト追加
  - `tests/fixtures/tail_call_optimization.pasta` テストフィクスチャを作成
  - 4パターン（単一呼び出し、複数呼び出し、末尾以外、呼び出しなし）を含むシーンを定義
  - 各パターンの期待される Lua 出力（`return` 付与の有無）を検証する統合テストを追加
  - `cargo test --all` で全テスト PASS を確認
  - _Requirements: 3, 4_

- [x] 4.2 コード品質確認と最適化
  - ドキュメント・コメント（将来拡張方法）が正確に記載されていることを確認
  - `cargo clippy` で警告がないことを確認
  - 末尾判定ロジックのパフォーマンスを確認（`rposition` の妥当性）
  - _Requirements: 1, 5_

````
