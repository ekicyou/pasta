# Implementation Plan: soul-document

## Overview

本タスクリストは、SOUL.mdで定義された「あるべき姿」を証明するRuntime E2Eテスト体系を整備し、Phase 0完了基準を達成するための実装計画です。

**主要な実装範囲**:
- Runtime E2Eテスト実装（シーン辞書、単語辞書、アクター単語辞書）
- pasta_shiori失敗テスト修正（5テスト）
- 共通テストヘルパーのリファクタリング
- ドキュメント整合性の確保

---

## Tasks

- [ ] 1. 共通テストヘルパーモジュールの構築
- [ ] 1.1 (P) E2Eヘルパーモジュールの作成
  - `crates/pasta_lua/tests/common/e2e_helpers.rs`を新規作成
  - `create_runtime_with_finalize()`関数を実装（Lua VM + finalize_scene登録）
  - `transpile()`関数を実装（Pasta→Lua変換）
  - `execute_scene()`関数のスケルトンを追加
  - _Requirements: 7.2_

- [ ] 1.2 (P) 既存テストのリファクタリング
  - `finalize_scene_test.rs`から重複コードを削除
  - `use common::e2e_helpers::*;`でヘルパーを使用
  - テストロジックは変更せず、ヘルパー呼び出しのみ修正
  - リグレッション確認（既存テストが全て成功すること）
  - _Requirements: 7.2_

- [ ] 2. Runtime E2Eテストの実装
- [ ] 2.1 (P) テストフィクスチャの作成
  - `crates/pasta_lua/tests/fixtures/e2e/runtime_e2e_scene.pasta`作成（3つの「挨拶」シーン）
  - `runtime_e2e_word.pasta`作成（単語辞書定義）
  - `runtime_e2e_actor_word.pasta`作成（アクター単語定義）
  - _Requirements: 7.2_

- [ ] 2.2 (P) シーン辞書E2Eテスト
  - `runtime_e2e_test.rs`に`test_scene_prefix_search_and_random_selection()`を実装
  - 前方一致検索からランダム選択までの完全フローを検証
  - 複数回実行で全候補が選択されることを確認（キャッシュ消費検証）
  - _Requirements: 7.2_

- [ ] 2.3 (P) 単語辞書E2Eテスト
  - `test_word_random_selection_and_replacement()`を実装
  - 単語ランダム選択と文字列置換の完全フローを検証
  - キャッシュ消費テスト（3要素×3回で全要素出現）
  - _Requirements: 7.1, 7.2_

- [ ] 2.4 (P) アクター単語スコープE2Eテスト
  - `test_actor_word_scope_resolution()`を実装
  - アクタースコープでの単語解決を検証
  - スコープ優先順位（アクター→シーン→グローバル）の確認
  - _Requirements: 7.2_

- [ ] 2.5 完全フローE2Eテスト
  - `test_complete_flow_pasta_to_output()`を実装
  - Pasta→トランスパイル→Lua実行→出力の完全フローを検証
  - シーン/単語選択の統合動作確認
  - _Requirements: 7.2_

- [ ] 3. 未テスト領域の実装
- [ ] 3.1 (P) コメント行パーステスト
  - `test_comment_line_explicit_parse()`を実装
  - コメント行がASTに含まれないことを確認
  - 複数行コメントと通常行の混在パターンを検証
  - _Requirements: 7.1_

- [ ] 3.2 (P) 属性継承テスト
  - `test_attribute_inheritance()`を実装
  - 親シーンの属性が子シーンに継承されることを確認
  - 属性値のオーバーライドパターンを検証
  - _Requirements: 7.1_

- [ ] 3.3 (P) 変数スコープ完全テスト
  - `test_variable_scope_complete()`を実装
  - ローカル（`＄`）、グローバル（`＄＊`）、システム（`＄＊＊`）の分離を確認
  - 各スコープの独立性を検証
  - _Requirements: 7.1_

- [ ] 3.4 (P) エラーメッセージ詳細性テスト
  - `test_error_message_specificity()`を実装
  - 不正構文に対して行番号・列番号付きエラーが返ることを確認
  - 複数のエラーパターンを検証
  - _Requirements: 7.1_

- [x] 4. pasta_shiori失敗テストの修正
- [x] 4.1 失敗原因の調査
  - `crates/pasta_shiori/tests/shiori_lifecycle_test.rs`の5テストを確認
  - デバッグログを追加し、`shiori.load()`失敗の詳細を特定
  - スタックトレース確認と失敗シーケンスの分析
  - **根本原因**: `pasta_shiori/tests/support/scripts/pasta/`に不完全なLuaモジュールセット
  - _Requirements: 7.3_

- [x] 4.2 SHIORI.DLL初期化修正
  - `pasta_lua/scripts/pasta/`から完全なLuaモジュールセットをコピー
  - `copy_fixture_to_temp()`のコピー順序を修正（サポートファイル→フィクスチャの順）
  - フィクスチャ固有のオーバーライド（main.lua等）が優先されるように
  - _Requirements: 7.3_

- [x] 4.3 失敗テスト修正の検証
  - 5テストすべてが成功することを確認 ✅
  - `test_shiori_load_sets_globals` ✅
  - `test_shiori_request_calls_pasta_scene` ✅
  - `test_shiori_request_increments_counter` ✅
  - `test_shiori_unload_creates_marker` ✅
  - `test_shiori_lifecycle_lua_execution_verified` ✅
  - _Requirements: 7.3_

- [ ] 5. ドキュメント整合性の確認と更新
- [ ] 5.1 TEST_COVERAGE.md更新
  - Section 4の未テスト領域マッピングを更新
  - 新規追加テストと機能の対応表を作成
  - 実装状態カテゴリ（完了/部分完了/未検証/未実装）を更新
  - _Requirements: 7.4_

- [ ] 5.2 SOUL.md整合性確認
  - Phase 0完了基準（Section 5.6）との整合性確認
  - pasta_shiori 100%パス達成の記録
  - 未テスト領域の受容判断結果を反映
  - コアバリュー・設計原則への影響確認（該当なし）
  - _Requirements: 7.5_

- [ ] 5.3 その他ドキュメント確認
  - SPECIFICATION.md: 言語仕様への影響確認（該当なし）
  - GRAMMAR.md: 文法リファレンスへの影響確認（該当なし）
  - クレートREADME: API変更の反映確認（該当なし）
  - steering/*: ステアリング更新確認（該当なし）
  - _Requirements: 7.5_

---

## Task Summary

- **Total**: 5メジャータスク、18サブタスク
- **Requirements Coverage**: 全7要件をカバー
- **Average Task Size**: 1-3時間/サブタスク
- **Parallel Tasks**: 11タスクが並行実行可能

## Quality Validation

✅ 全要件がタスクにマッピング済み  
✅ タスク依存関係が明確  
✅ テストタスクが実装タスクと並行  
✅ ドキュメント整合性タスクが最終確認として配置

---

## Next Steps

1. タスクリストをレビュー
2. 承認後、`/kiro-spec-impl soul-document 1.1`で実装開始
3. 各タスク完了後にコミット・プッシュ
4. 全タスク完了後にPhase 0 DoD達成を確認
