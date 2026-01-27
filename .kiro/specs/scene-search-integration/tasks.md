# Implementation Tasks: scene-search-integration

## Task Breakdown

### Phase 1: コア実装

- [x] 1. scene.lua への SCENE.search() 関数実装
- [x] 1.1 (P) @pasta_search モジュールのロードと scene_result_mt メタテーブル定義
  - `local SEARCH = require("@pasta_search")` をモジュールスコープに追加（L9付近）
  - `scene_result_mt` メタテーブルを定義（`__call` メタメソッド付き、L32付近）
  - メタテーブルはモジュールスコープで1回のみ定義
  - _Requirements: 3.1, 3.2, 1.7_

- [x] 1.2 SCENE.search() 関数本体の実装
  - 引数バリデーション（name が非文字列なら nil 返却）
  - `SEARCH:search_scene(name, global_scene_name)` を呼び出して検索実行
  - 検索結果から `SCENE.get(global_name, local_name)` でシーン関数取得
  - メタデータ付きテーブル `{global_name, local_name, func}` を生成
  - `setmetatable()` で scene_result_mt を設定して返却
  - 検索失敗時・シーン未登録時は nil を返却
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 4.1, 4.3_

### Phase 2: テスト実装

- [x] 2. Unit Tests - SCENE.search() の基本動作検証
- [x] 2.1 (P) 正常系テスト（ローカル・グローバル検索）
  - PastaLuaRuntime を使用してテスト環境構築
  - シーン登録後、ローカル検索で取得可能か検証
  - グローバル検索（`global_scene_name = nil`）で取得可能か検証
  - 返却値の global_name, local_name, func フィールドを検証
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 6.1_

- [x] 2.2 (P) エラー系・境界値テスト
  - `name` が nil の場合に nil を返却することを検証
  - `name` が非文字列の場合に nil を返却することを検証
  - `name` が空文字列の場合の動作を検証
  - 検索結果がない場合に nil を返却することを検証
  - シーン関数が未登録の場合に nil を返却することを検証
  - _Requirements: 1.5, 1.6, 4.1, 4.2, 4.3_

- [x] 2.3 __call メタメソッドの動作検証
  - 検索結果テーブルを関数として呼び出し可能か検証
  - `result(act, ...)` が `result.func(act, ...)` に正しく委譲されるか検証
  - メタデータ（global_name, local_name）へのアクセスが可能か検証
  - _Requirements: 1.7, 2.1, 2.2_

- [x] 3. Integration Tests - システム統合検証
- [x] 3.1 PastaLuaRuntime 統合テスト
  - @pasta_search 登録後の SCENE.search() 動作を検証
  - SCENE.register() → SCENE.search() → シーン取得の一連フローを検証
  - 複数シーン登録時のランダム選択動作を検証（@pasta_search の責務）
  - _Requirements: 3.3, 3.4, 2.3, 2.4_

- [x] 3.2 (P) フォールバック検索の統合検証
  - ローカル検索失敗時のグローバル検索フォールバックを検証
  - SearchContext の resolve_scene_id_unified() の動作と整合性確認
  - _Requirements: 1.2, 2.4_

- [x] 3.3 (P) 既存 API との互換性検証
  - SCENE.get(), SCENE.register(), SCENE.create_scene() が引き続き動作することを検証
  - STORE.scenes へのアクセスパターンが変更されていないことを確認
  - _Requirements: 5.1, 5.2, 5.3_

### Phase 3: E2E テストとドキュメント

- [x] 4. E2E Tests - イベントハンドラ統合
- [x] 4.1 (P) イベントハンドラからのシーン呼び出しテスト
  - REG.OnBoot スタイルのイベントハンドラから SCENE.search() を呼び出し
  - イベント名（例: "OnBoot"）に対応するシーンの検索・実行を検証
  - シーンが見つからない場合のデフォルト動作を検証
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 5. ドキュメント整合性の確認と更新
- [x] 5.1 プロジェクトドキュメントの更新
  - TEST_COVERAGE.md - 新規テストファイル（scene_search_test.rs）のマッピング追加
  - crates/pasta_lua/README.md - SCENE.search() API の追加を反映（該当する場合）
  - steering/* - 該当領域のステアリング更新確認（本機能は既存パターン踏襲のため更新不要の可能性大）
  - _Requirements: 全要件_

- [x] 5.2 設計原則との整合性確認
  - SOUL.md - コアバリュー（日本語フレンドリー、UI独立性）との整合性確認
  - SPECIFICATION.md / GRAMMAR.md - 言語仕様への影響確認（本機能はLua実装のため変更なし）
  - _Requirements: 全要件_

## Requirements Coverage Matrix

| Requirement | Tasks |
|-------------|-------|
| 1.1-1.7 | 1.2, 2.1, 2.3 |
| 2.1-2.4 | 1.2, 2.3, 3.1, 3.2 |
| 3.1-3.4 | 1.1, 3.1 |
| 4.1-4.3 | 1.2, 2.2 |
| 5.1-5.3 | 3.3 |
| 6.1-6.3 | 2.1, 4.1 |

## Task Dependencies

```
1.1 (P) → 1.2
       ↓
2.1 (P), 2.2 (P), 2.3
       ↓
3.1, 3.2 (P), 3.3 (P)
       ↓
4.1 (P)
       ↓
5.1, 5.2
```

**Parallel Execution Notes**:
- Task 1.1 は独立して実行可能（モジュールロードとメタテーブル定義のみ）
- Task 2.1, 2.2 は Task 1.2 完了後に並列実行可能（異なるテストケース）
- Task 3.2, 3.3 は Task 3.1 完了後に並列実行可能（異なる統合ポイント）
- Task 4.1 は Task 3.1-3.3 完了後に実行可能
