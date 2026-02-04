# Implementation Plan: refine-talk-conversion

## Overview

本実装は、`sakura_builder.lua`のトーク変換処理を洗練化し、単純エスケープ処理（`escape_sakura`）を`@pasta_sakura_script.talk_to_script`に置き換える。変更範囲は明確で限定的であり、4箇所の変更（require追加、関数削除、呼び出し置換、テスト更新）で完了する。

## Implementation Tasks

- [ ] 1. sakura_builder.luaの実装変更
- [ ] 1.1 (P) @pasta_sakura_scriptモジュールの読み込み追加
  - ファイル先頭（local BUILDER = {}の後）に`local SAKURA_SCRIPT = require "@pasta_sakura_script"`を追加
  - SAKURA_SCRIPT変数にモジュール参照を格納
  - _Requirements: 2.1, 2.2_

- [ ] 1.2 (P) escape_sakura関数の削除
  - 行12-17の`escape_sakura`ローカル関数定義を削除
  - escape_sakuraへの参照が残っていないことを確認
  - _Requirements: 4.1, 4.2_

- [ ] 1.3 talk_to_script呼び出しへの置き換え
  - 行96の`escape_sakura(inner.text)`を`SAKURA_SCRIPT.talk_to_script(actor, inner.text)`に置換
  - actorオブジェクト（行78で取得済み）を第1引数として渡す
  - actorがnilの場合も適切に処理されることを確認（talk_to_scriptがデフォルト値にフォールバック）
  - _Requirements: 1.1, 1.2, 1.3, 3.1, 3.2, 3.3_

- [ ] 2. テスト期待値の更新
- [ ] 2.1 sakura_builder_test.luaの期待値更新
  - 24テストケースの期待値をウェイト挿入形式に更新
  - デフォルトウェイト値を適用した変換パターン：
    - `"こんにちは\\e"` → `"こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]\\e"`（通常文字：50ms）
    - `"こんにちは。\\e"` → `"こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]。\_w[1000]\\e"`（句点：1000ms）
    - `"こんにちは、\\e"` → `"こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[50]、\_w[500]\\e"`（読点：500ms）
    - `"あ！\\e"` → `"あ\_w[50]！\_w[500]\\e"`（強調文字：500ms）
  - escape_sakuraベースの期待値を完全に削除
  - `\e`終端タグが維持されることを確認
  - _Requirements: 6.1, 6.2_

- [ ] 2.2* 関連インテグレーションテストの期待値確認
  - さくらスクリプト出力を検証する他のインテグレーションテストを実行
  - 失敗したテストの期待値をウェイト挿入形式に修正
  - 既存さくらスクリプトタグ（`\s[ID]`, `\w[ms]`等）が保護されることを確認
  - _Requirements: 6.3, 5.1, 5.2, 5.3_

- [ ] 3. 動作検証と統合確認
- [ ] 3.1 (P) ビルドとユニットテスト実行
  - `cargo test -p pasta_lua`を実行
  - すべてのテストがパスすることを確認
  - さくらスクリプト出力の正確性を検証
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 3.2 (P) ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認（変更なし想定）
  - SPECIFICATION.md - 言語仕様の更新確認（変更なし想定）
  - TEST_COVERAGE.md - 新規テストのマッピング追加（該当なし想定）
  - crates/pasta_lua/README.md - API変更の反映確認（変更なし想定）
  - steering/* - 該当領域のステアリング更新確認（変更なし想定）
  - _Requirements: すべて（DoD準拠）_

## Task Summary

- **合計**: 3メジャータスク、7サブタスク
- **並列実行可能**: タスク1.1, 1.2, 3.1, 3.2（4タスク）
- **オプショナル**: タスク2.2（インテグレーションテスト期待値確認）
- **推定工数**: 各サブタスク1-2時間、合計8-14時間

## Requirements Coverage

| Requirement | Covered by Tasks |
|-------------|------------------|
| 1.1, 1.2, 1.3 | 1.3 |
| 2.1, 2.2 | 1.1 |
| 3.1, 3.2, 3.3 | 1.3 |
| 4.1, 4.2 | 1.2 |
| 5.1, 5.2, 5.3 | 2.2, 3.1 |
| 6.1, 6.2 | 2.1 |
| 6.3 | 2.2 |

全要件カバレッジ: ✅ 6要件すべて対応

## Implementation Notes

- タスク1.1と1.2は並列実行可能（異なる行の編集）
- タスク1.3は1.1の完了後に実行（SAKURA_SCRIPT変数の利用）
- タスク2.1は1.3の完了後に実行（実装変更を受けた期待値更新）
- タスク2.2はオプショナル（MVPには不要だが、完全性のため推奨）
- タスク3.1と3.2は並列実行可能（独立した検証作業）
