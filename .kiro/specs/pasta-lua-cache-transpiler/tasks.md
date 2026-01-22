# Implementation Plan

## 1. キャッシュバージョン管理機能

- [ ] 1.1 (P) キャッシュバージョンファイルの読み書き機能を実装
  - `.cache_version` ファイルのパス定義と読み込みロジック
  - `env!("CARGO_PKG_VERSION")` でバージョン取得
  - バージョン不一致時の全クリア処理（`remove_dir_all`）
  - 新バージョンでの再初期化
  - _Requirements: 1, 2_

- [ ] 1.2 (P) バージョンチェックのユニットテスト
  - バージョン一致時の保持動作テスト
  - バージョン不一致時のクリア動作テスト
  - .cache_version ファイルが存在しない場合の初期化テスト
  - _Requirements: 1, 2_

## 2. CacheManager コンポーネント実装

- [ ] 2.1 CacheManager 構造体の実装
  - `base_dir`, `cache_dir` フィールド定義
  - `new()` コンストラクタ
  - `prepare_cache_dir()` でバージョンチェックとディレクトリ準備
  - _Requirements: 1, 2, 7_

- [ ] 2.2 (P) ファイル変更検出機能の実装
  - `needs_transpile()` メソッド: タイムスタンプ比較ロジック
  - `std::fs::metadata()` によるファイルメタデータ取得
  - ソースとキャッシュの modified time 比較
  - キャッシュ不在時の判定
  - _Requirements: 1_

- [ ] 2.3 (P) パス変換機能の実装
  - `source_to_module_name()`: Pasta パス → モジュール名変換
  - `source_to_cache_path()`: Pasta パス → キャッシュファイルパス変換
  - ディレクトリ階層の再現（`dic/subdir/` → `pasta/scene/subdir/`）
  - 日本語ファイル名のサポート
  - ハイフン → アンダースコア変換
  - _Requirements: 2, 4_

- [ ] 2.4 (P) キャッシュファイル保存機能の実装
  - `save_cache()` メソッド: Lua コードをファイルに書き込み
  - ディレクトリ自動作成（`create_dir_all`）
  - UTF-8 エンコーディングで保存
  - モジュール名を返却
  - _Requirements: 2_

- [ ] 2.5 scene_dic.lua 生成機能の実装
  - `generate_scene_dic()` メソッド
  - 全モジュール名のリストから require 文列挙
  - アルファベット順ソート
  - ヘッダーコメント追加（自動生成マーカー、タイムスタンプ）
  - 末尾に `require("pasta").finalize_scene()` 呼び出し
  - 孤立キャッシュの警告ログ出力
  - _Requirements: 3_

## 3. エラー型の拡張

- [ ] 3.1 (P) LoaderError に新規バリアント追加
  - `CacheDirectoryError`: ディレクトリ操作失敗
  - `MetadataError`: メタデータ取得失敗
  - `CacheWriteError`: キャッシュ書き込み失敗
  - `SceneDicGenerationError`: scene_dic.lua 生成失敗
  - `PartialTranspileError`: 部分的トランスパイル失敗（成功/失敗カウント含む）
  - `TranspileFailure` 構造体定義
  - _Requirements: 6_

- [ ] 3.2 (P) RuntimeError に mlua::Error 統合
  - `LuaError` バリアントを追加
  - `#[from]` 属性で自動変換
  - _Requirements: 5, 6_

## 4. PastaLoader の統合

- [ ] 4.1 PastaLoader への CacheManager 統合
  - `load_with_config()` で CacheManager インスタンス化
  - Phase 2: `prepare_cache_dir()` 呼び出し（バージョンチェック含む）
  - Phase 4: 各ファイルに対して `needs_transpile()` で判定
  - トランスパイル対象のみ処理、スキップ数カウント
  - Phase 5: `save_cache()` でキャッシュ保存、`generate_scene_dic()` 実行
  - _Requirements: 1, 2, 3, 5, 7_

- [ ] 4.2 prepare_directories の削除処理廃止
  - `remove_dir_all` 呼び出しを削除
  - ディレクトリ存在確認と作成のみ実施
  - _Requirements: 1_

- [ ] 4.3 debug_mode 時の統計ログ出力
  - トランスパイル対象数、スキップ数、失敗数のログ
  - `tracing::info!` でサマリー出力
  - _Requirements: 5_

## 5. finalize_scene() スタブ実装

- [ ] 5.1 (P) pasta stdlib への finalize_scene() 追加
  - `src/stdlib/mod.rs` に `finalize_scene()` 関数定義
  - 警告ログのみ出力（`tracing::warn!`）
  - `Ok(())` を返却
  - _Requirements: 3, 5_

- [ ] 5.2 (P) pasta モジュールへの finalize_scene 登録
  - `src/runtime/mod.rs` の `register_pasta_module()` に登録
  - Lua から `require("pasta").finalize_scene()` で呼び出し可能にする
  - mlua::Error への変換
  - _Requirements: 3, 5_

## 6. PastaLuaRuntime の scene_dic ロード機能

- [ ] 6.1 scene_dic.lua ロード機能の実装
  - `load_scene_dic()` メソッド追加
  - `self.lua.load("require('pasta.scene_dic')").exec()` 実行
  - mlua::Error → RuntimeError::LuaError 自動変換
  - _Requirements: 5_

- [ ] 6.2 PastaLoader Phase 6 での scene_dic ロード統合
  - ランタイム初期化後に `load_scene_dic()` 呼び出し
  - エラー時の適切なエラーメッセージ
  - _Requirements: 5_

## 7. 統合テスト

- [ ] 7.1 増分トランスパイル動作の統合テスト
  - 初回起動: 全ファイルトランスパイル
  - 2回目起動（変更なし）: 全スキップ
  - ファイル変更後: 該当ファイルのみ再トランスパイル
  - キャッシュファイルの内容検証
  - _Requirements: 1, 2_

- [ ] 7.2 scene_dic.lua 生成と自動ロードの統合テスト
  - scene_dic.lua の生成内容検証（require 文リスト）
  - `finalize_scene()` 呼び出しの存在確認
  - PastaLuaRuntime での scene_dic ロード成功確認
  - 警告ログの出力確認
  - _Requirements: 3, 5_

- [ ] 7.3 エラーハンドリングの統合テスト
  - パースエラー発生時の部分失敗テスト
  - エラーサマリーの内容検証
  - 失敗ファイルのキャッシュ非更新確認
  - _Requirements: 6_

- [ ] 7.4 キャッシュバージョン管理の統合テスト
  - バージョン変更時の全クリア動作確認
  - .cache_version ファイルの内容検証
  - 旧バージョンキャッシュの削除確認
  - _Requirements: 1, 2_

- [ ] 7.5 パス解決とカスタム設定の統合テスト
  - pasta.toml の設定読み込み確認
  - カスタム `transpiled_output_dir` でのキャッシュ出力
  - デフォルト値のフォールバック動作
  - 日本語ファイル名の処理確認
  - _Requirements: 4, 7_

## 8. Loader テストの修正

- [ ] 8.1 test_cache_cleared_on_load の修正
  - テスト名を `test_cache_incremental_update` に変更
  - 古いキャッシュが保持されることを検証
  - テスト冒頭で手動 `remove_dir_all` 追加（clean state 確保）
  - _Requirements: 1_

## 9. ユニットテスト

- [ ] 9.1* CacheManager ユニットテスト
  - `needs_transpile()` の各ケーステスト（AC 1.1-1.5）
  - `source_to_module_name()` のパス変換テスト（AC 4.1-4.6）
  - `source_to_cache_path()` のパス変換テスト（AC 2.1-2.4）
  - `generate_scene_dic()` の出力フォーマットテスト（AC 3.1-3.8）
  - _Requirements: 1, 2, 3, 4_
