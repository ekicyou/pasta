# Implementation Plan

## タスク概要

本実装計画は、SHIORIライフサイクル全フェーズ（load/request/unload）においてLuaコード実行を検証するE2Eテストを段階的に構築する。既存テストパターンを活用しつつ、観測可能な副作用を持つフィクスチャと検証ロジックを実装する。

---

## Task 1: テスト基盤整備

### 1.1 (P) pasta_lua標準ライブラリの自己完結化
  - pasta_luaクレートの`scripts/`および`scriptlibs/`を`crates/pasta_shiori/tests/support/`にコピー
  - `tests/support/scripts/pasta/shiori/`配下に最小セットを配置
  - `tests/support/scriptlibs/`配下に必要なライブラリを配置
  - _Requirements: 5_

### 1.2 (P) common/mod.rsヘルパー関数実装
  - `copy_fixture_to_temp(fixture_name)`関数を実装
  - `tests/fixtures/`配下のフィクスチャをTempDirにコピー
  - `tests/support/`配下のscripts/scriptlibsもコピー
  - `copy_dir_recursive()`および`copy_support_dirs()`ヘルパー関数を実装
  - _Requirements: 5_

---

## Task 2: テストフィクスチャ作成

### 2.1 (P) lifecycle.pastaシーン定義
  - `tests/fixtures/shiori_lifecycle/dic/test/lifecycle.pasta`を作成
  - `＊テスト挨拶`シーンを定義（出力: 「ライフサイクルテスト成功！」）
  - Pasta DSL文法に準拠
  - _Requirements: 4_

### 2.2 (P) pasta.toml設定ファイル作成
  - `tests/fixtures/shiori_lifecycle/pasta.toml`を作成
  - `[loader] debug_mode = true`を設定
  - _Requirements: 5_

### 2.3 main.lua観測可能SHIORI関数実装
  - `tests/fixtures/shiori_lifecycle/scripts/pasta/shiori/main.lua`を作成（support/からコピーして編集）
  - SHIORI.load(): `loaded_hinst`、`load_dir`グローバル変数を設定、unloadマーカーパスを記録
  - SHIORI.request(): `request_count`をインクリメント、`@pasta_search`で「テスト挨拶」シーンを検索・呼び出し、シーン出力をSHIORI応答に埋め込み
  - SHIORI.unload(): `unload_called.marker`ファイルを作成
  - シーン未検出時は500エラーを返す（テスト失敗を明確化）
  - _Requirements: 1, 2, 3, 4_

---

## Task 3: インテグレーションテスト実装

### 3.1 test_shiori_load_sets_globalsテスト
  - PastaShiori::load()を実行
  - Luaグローバル変数`SHIORI.loaded_hinst`および`SHIORI.load_dir`を検証
  - hinst値が正しく設定されていることを確認
  - load_dir値が正しく設定されていることを確認
  - _Requirements: 1_

### 3.2 test_shiori_request_increments_counterテスト
  - SHIORI.request()を複数回呼び出し
  - Luaグローバル変数`SHIORI.request_count`を検証
  - 呼び出し回数とカウント値が一致することを確認
  - _Requirements: 2_

### 3.3 test_shiori_request_calls_pasta_sceneテスト
  - SHIORI.request()を実行
  - 応答に「ライフサイクルテスト成功！」が含まれることを検証
  - Pasta DSLシーンがトランスパイルされ、Luaから呼び出されたことを確認
  - _Requirements: 4_

### 3.4 test_shiori_unload_creates_markerテスト
  - PastaShioriをドロップ
  - `unload_called.marker`ファイルの存在を検証
  - SHIORI.unloadが実行されたことを確認
  - _Requirements: 3_

### 3.5 test_shiori_lifecycle_lua_execution_verified統合テスト
  - 全フェーズ（load/request/unload）をE2E実行
  - Phase 1（load）: hinst/load_dirグローバル変数を検証
  - Phase 2（request）: シーン出力検証、request_count検証
  - Phase 3（unload）: ファイルマーカー検証
  - すべてのRequirementが統合的に動作することを確認
  - _Requirements: 1, 2, 3, 4, 5_

---

## Task 4: エラーケーステスト（オプション）

### 4.1* (P) シーン未検出時のエラーハンドリングテスト
  - lifecycle.pastaから「テスト挨拶」シーンを削除
  - SHIORI.request()が500エラーを返すことを検証
  - エラーメッセージに「Scene 'テスト挨拶' not found」が含まれることを確認
  - _Requirements: 4.3_

### 4.2* (P) SHIORI.unload未定義時の動作テスト
  - main.luaからSHIORI.unload関数を削除
  - ドロップがエラーなく完了することを検証
  - _Requirements: 3.3_

---

## Task 5: CI/CD統合準備

### 5.1 (P) テスト実行確認
  - `cargo test --package pasta_shiori --test shiori_lifecycle_test`を実行
  - すべてのテストが成功することを確認
  - テスト実行時間を記録（ベースライン確立）
  - _Requirements: 1, 2, 3, 4, 5_

---

## Requirements Coverage Matrix

| Requirement | Tasks |
|-------------|-------|
| 1 (SHIORI.load実行確認) | 2.3, 3.1, 3.5 |
| 2 (SHIORI.request実行確認) | 2.3, 3.2, 3.5 |
| 3 (SHIORI.unload実行確認) | 2.3, 3.4, 3.5, 4.2* |
| 4 (Pasta DSL読み込み確認) | 2.1, 2.3, 3.3, 3.5, 4.1* |
| 5 (テストフィクスチャ整備) | 1.1, 1.2, 2.2, 3.5, 5.1 |

すべての要件が網羅されています。
