# Implementation Plan

## Task Overview

本実装計画は、`ACT_IMPL.call()`の4段階優先順位検索による本格実装と、`SCENE.search()`の第3引数拡張を対象とする。すべてのタスクは既存のactor.luaパターンを流用し、段階的にハンドラー検索機能を構築する。

---

## Tasks

- [x] 1. SCENE.searchシグネチャ拡張
- [x] 1.1 (P) 第3引数attrsの追加とドキュメント更新
  - `SCENE.search(name, global_scene_name, attrs)`シグネチャに変更
  - 第3引数は現時点で未使用だが、将来の属性フィルタリング用に予約
  - LDocコメントを更新して引数説明を追加
  - 既存の2引数呼び出しとの互換性を維持（Lua余剰引数無視仕様）
  - _Requirements: 4.1, 4.2, 4.3_

- [x] 1.2 (P) SCENE.search後方互換性のユニットテスト追加
  - 2引数呼び出しパターンが正常動作することを検証
  - 3引数呼び出しパターン（空テーブル`{}`渡し）が正常動作することを検証
  - _Requirements: 4.3, 5.1, 5.2_

- [x] 2. ACT_IMPL.call本体実装
- [x] 2.1 現在の実装を4段階検索ロジックに置き換え
  - Level 1: `self.current_scene[key]`からハンドラー取得
  - Level 2: `SCENE.search(key, global_scene_name, attrs)`で検索
  - Level 3: `require("pasta.global")[key]`から取得
  - Level 4: `SCENE.search(key, nil, attrs)`でフォールバック
  - 各レベルで最初に見つかった非nilハンドラーを採用
  - `self.current_scene == nil`の場合はLevel 1をスキップ
  - Level 2/4の結果から`.func`フィールドを取得
  - ハンドラー未発見時はnilを返却（サイレント動作）
  - TODOコメントで将来のログ出力拡張ポイントを明示
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 6.1, 6.2_

- [x] 2.2 シグネチャ変更と可変長引数の処理
  - `ACT_IMPL.call(self, global_scene_name, key, attrs, ...)`シグネチャに変更
  - `attrs`パラメータをSCENE.searchに渡す（トランスパイラは常に`{}`を出力）
  - 可変長引数`...`をハンドラーに正しく渡す: `handler(self, ...)`
  - ハンドラーの戻り値をそのまま返却
  - _Requirements: 1.1, 1.4, 3.1, 3.2, 4.1, 4.2_

- [x] 3. ユニットテスト実装
- [x] 3.1 (P) Level 1検索の単体テスト
  - `current_scene[key]`から正しくハンドラーを取得できることを検証
  - `self.current_scene == nil`の場合、Level 2へスキップすることを検証（nil安全性）
  - _Requirements: 2.1_

- [x] 3.2 (P) Level 2検索の単体テスト
  - `SCENE.search(key, global_scene_name, attrs)`経由でハンドラーを取得できることを検証
  - Level 1がnilの場合にLevel 2が呼ばれることを検証
  - _Requirements: 2.2, 4.1_

- [x] 3.3 (P) Level 3検索の単体テスト
  - `pasta.global[key]`から正しくハンドラーを取得できることを検証
  - Level 1, 2がnilの場合にLevel 3が呼ばれることを検証
  - _Requirements: 2.3_

- [x] 3.4 (P) Level 4フォールバック検索の単体テスト
  - `SCENE.search(key, nil, attrs)`でフォールバック検索が動作することを検証
  - Level 1-3がすべてnilの場合にLevel 4が呼ばれることを検証
  - _Requirements: 2.4, 4.2_

- [x] 3.5 (P) 優先順位検証テスト
  - 複数レベルに同じキーが存在する場合、Level 1が優先されることを検証
  - Level 1がnilでLevel 2が存在する場合、Level 2が使われることを検証
  - _Requirements: 2.5_

- [x] 3.6 (P) ハンドラー未発見時の動作テスト
  - すべてのレベルでnilの場合、nilを返却することを検証
  - エラーが発生しないことを検証（サイレント動作）
  - _Requirements: 3.3, 3.4_

- [x] 3.7 (P) ハンドラー実行と戻り値のテスト
  - 見つかったハンドラーが正しく`handler(self, ...)`として呼ばれることを検証
  - 可変長引数が正しくハンドラーに渡されることを検証
  - ハンドラーの戻り値が正しく返却されることを検証
  - _Requirements: 3.1, 3.2_

- [x] 4. 統合テスト実装
- [x] 4.1 トランスパイラ出力との統合テスト
  - トランスパイラが生成する`act:call(SCENE.__global_name__, "key", {}, ...)`形式のコードが正常動作することを検証
  - 既存のtranspiler_integration_test.rsが引き続きパスすることを確認
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 4.2 E2Eシナリオテスト
  - シーン呼び出しチェーンが4段階検索を経て正常に実行されることを検証
  - 実際のPastaスクリプトでの動作確認
  - _Requirements: 2.5, 3.1, 5.3_

---

## Requirements Coverage

| Requirement | Summary                       | Covered by Tasks |
|-------------|-------------------------------|------------------|
| 1.1-1.4     | ACT_IMPL.call シグネチャ定義   | 2.1, 2.2         |
| 2.1-2.5     | 優先順位付き4段階検索          | 2.1, 3.1-3.5     |
| 3.1-3.4     | ハンドラー実行                 | 2.1, 2.2, 3.6, 3.7 |
| 4.1-4.3     | SCENE.searchへの属性渡し       | 1.1, 2.2, 3.2, 3.4 |
| 5.1-5.3     | 既存コードとの互換性           | 1.2, 4.1, 4.2    |
| 6.1-6.2     | ログ出力（将来対応）           | 2.1              |

全6要件がすべてタスクにマッピングされている。

---

## Implementation Notes

### 実装順序の根拠
1. **SCENE.search拡張を先行**: ACT_IMPL.callが依存するため、先に完了させる
2. **本体実装を中核に配置**: 4段階検索ロジックとシグネチャ変更を一括実装
3. **ユニットテストを並列実行可能に**: 各レベルの検索は独立してテスト可能
4. **統合テストで全体検証**: 既存コードとの互換性とE2Eシナリオを最後に確認

### 並列実行可能タスク
- 1.1とタスク3系統（SCENE.search拡張と各種テストは依存関係なし）
- 3.1〜3.7（各レベルの検索テストは独立）

### 推定作業時間
- タスク1: 1-2時間（シグネチャ変更とテスト）
- タスク2: 2-3時間（本体実装）
- タスク3: 3-4時間（7つのユニットテスト）
- タスク4: 1-2時間（統合テスト）
- **合計**: 7-11時間

