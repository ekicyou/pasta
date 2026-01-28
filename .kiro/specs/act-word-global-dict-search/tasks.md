# 実装タスク

## タスク一覧

- [x] 1. Rust側検索APIのフォールバック廃止
- [x] 1.1 (P) WordTable フォールバックロジック削除
  - `collect_word_candidates()` の Step 2 グローバルフォールバック分岐を削除
  - `module_name` が空の場合はグローバルのみ検索、非空の場合はローカルのみ検索
  - 結果0件時もフォールバックしない
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 1.2 (P) SceneTable フォールバックロジック削除
  - `collect_scene_candidates()` の Step 2 グローバルフォールバック分岐を削除
  - `module_name` が空の場合はグローバルのみ検索、非空の場合はローカルのみ検索
  - WordTable と同様のパターンで修正
  - _Requirements: 1.5, 1.6, 1.7, 1.8_

- [x] 1.3 (P) WordTable 既存テスト修正
  - `test_collect_word_candidates_fallback_to_global` を削除
  - `test_collect_word_candidates_global_only` を新規追加（`module_name=""` でグローバル検索）
  - `test_collect_word_candidates_local_takes_priority` を `test_collect_word_candidates_local_only` にリネーム
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 1.4 (P) SceneTable 既存テスト修正
  - `test_collect_scene_candidates_fallback_to_global` を削除
  - `test_collect_scene_candidates_global_only` を新規追加
  - `test_collect_scene_candidates_fallback_local_takes_priority` を `test_collect_scene_candidates_local_only` にリネーム
  - _Requirements: 1.5, 1.6, 1.7, 1.8_

- [x] 2. pasta.word モジュールへ値解決ロジック追加
- [x] 2.1 WORD.resolve_value() 関数実装
  - `nil` の場合は `nil` を返す
  - 関数値の場合は `value(act)` を実行して戻り値を返す
  - 配列値の場合は最初の要素 `value[1]` を返す
  - その他の値は `tostring(value)` で文字列化
  - pasta.word モジュールからエクスポート
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [x] 3. ACT_IMPL.word 実装（act.lua）
- [x] 3.1 シーン・GLOBAL完全一致検索実装
  - シーンテーブル `self.current_scene[name]` で完全一致検索
  - 見つかった場合は `WORD.resolve_value(value, self)` で値解決
  - GLOBAL テーブル `GLOBAL[name]` で完全一致検索
  - 見つかった場合は `WORD.resolve_value(value, self)` で値解決
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 3.2 SEARCH API 呼び出しによる前方一致検索
  - シーンローカル辞書検索: `SEARCH:search_word(name, scene_name)`
  - グローバル辞書検索: `SEARCH:search_word(name, nil)`
  - 全て見つからない場合は `nil` を返す
  - _Requirements: 2.5, 2.6, 2.7_

- [x] 4. PROXY_IMPL.word 修正（actor.lua）
- [x] 4.1 アクター完全一致検索実装
  - アクターテーブル `self.actor[name]` で完全一致検索
  - 見つかった場合は `WORD.resolve_value(value, self.act)` で値解決
  - _Requirements: 3.1, 3.2_

- [x] 4.2 SEARCH API 呼び出しによるアクター辞書検索
  - アクター辞書検索: `SEARCH:search_word(name, "__actor_" .. actor.name .. "__")`
  - 見つかった場合は `WORD.resolve_value(value, self.act)` で値解決
  - 見つからない場合は `act:word(name)` に委譲
  - _Requirements: 3.3, 3.4, 3.5_

- [x] 4.3 Lua検索ロジック削除
  - `search_prefix_lua()` 関数を削除
  - `resolve_value()` ローカル関数を削除（pasta.word に移動済み）
  - `math.random` による候補選択ロジックを削除
  - `WORD.get_actor_words()` / `WORD.get_local_words()` / `WORD.get_global_words()` 呼び出しを削除
  - _Requirements: 3.6, 3.7, 3.8, 3.9_

- [x] 5. finalize.rs アクター単語辞書収集実装
- [x] 5.1 collect_words() 修正
  - `all_words.actor` からアクター単語辞書を収集
  - `WordCollectionEntry` に `actor_name` フィールドを追加（必要に応じて）
  - global, local と同様のパターンで処理
  - _Requirements: 4.1_

- [x] 5.2 build_word_registry() 修正
  - 収集したアクター単語を `register_actor()` で登録
  - アクター名をキーとして適切に処理
  - _Requirements: 4.2_

- [x] 6. 統合テスト・後方互換性検証
- [x] 6.1 統合テスト実装
  - ACT_IMPL.word の4レベルフォールバック動作検証
  - PROXY_IMPL.word の3レベルフォールバック動作検証
  - アクター辞書検索の動作確認
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 6.2 後方互換性テスト実行
  - 全既存テストスイート実行（`cargo test --all`）
  - リグレッション確認
  - `act:word()` および `proxy:word()` の動作が従来と同等であることを確認
  - _Requirements: 6.1, 6.2, 6.3_

## タスク実行順序

**フェーズ1: Rust側修正（並列実行可能）**
- 1.1, 1.2 - WordTable / SceneTable のフォールバック削除
- 1.3, 1.4 - 既存テスト修正

**フェーズ2: Lua側基盤準備**
- 2.1 - WORD.resolve_value() 実装

**フェーズ3: Lua側実装（並列実行可能）**
- 3.1, 3.2 - ACT_IMPL.word 実装
- 4.1, 4.2, 4.3 - PROXY_IMPL.word 修正

**フェーズ4: アクター辞書収集**
- 5.1, 5.2 - finalize.rs 修正

**フェーズ5: 統合検証**
- 6.1 - 統合テスト
- 6.2 - 後方互換性確認

## 完了基準（DoD）

1. **Spec Gate**: 全要件カバー確認
2. **Test Gate**: `cargo test --all` 成功
3. **Doc Gate**: CHANGELOG 更新
4. **Steering Gate**: tech.md, workflow.md との整合性確認
5. **Soul Gate**: SOUL.md の設計原則との整合性確認
