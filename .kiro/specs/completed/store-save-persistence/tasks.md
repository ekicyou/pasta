# Implementation Tasks

## 1. Rust側永続化API実装

- [x] 1.1 (P) PersistenceConfig構造体の実装
  - `loader/config.rs`にPersistenceConfig構造体を追加
  - obfuscate、file_pathフィールドを定義し、serde Deserializeを実装
  - デフォルト値の提供（obfuscate: false, file_path: "profile/pasta/save/save.json"）
  - PastaConfigに`persistence: Option<PersistenceConfig>`フィールドを追加
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 1.2 (P) PersistenceError型の定義
  - エラー型を定義（IoError, SerdeError, LuaSerdeError, DirectoryCreationError, LuaAccessError）
  - エラーメッセージとFrom実装を提供
  - エラーレベルに応じた分類（DEBUG, WARN, ERROR）を実装
  - _Requirements: 6.1, 6.3_

- [x] 1.3 persistence.rsモジュールの実装
  - `runtime/persistence.rs`を作成し基本構造を実装
  - PersistenceState構造体（file_path, obfuscate, debug_mode）を定義
  - register関数を実装し、@pasta_persistenceモジュールをLua側に登録
  - load_impl、save_impl関数の骨格を実装（upvalueとしてPersistenceStateを保持）
  - _Requirements: 1.1_

- [x] 1.4 ファイルI/Oと形式自動判別の実装
  - load_from_file関数でJSON/gzip形式の自動判別を実装（gzipマジックヘッダー検出）
  - save_table_to_file関数でアトミック書き込み（一時ファイル→リネーム）を実装
  - ディレクトリ自動作成機能を実装（fs::create_dir_all）
  - gzip圧縮/解凍機能を実装（flate2::GzEncoder/GzDecoder、圧縮レベルdefault）
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 6.2, 6.4_

- [x] 1.5 Lua VMとのデータ変換
  - LuaSerdeExtを使用してLuaテーブル↔serde_json::Value変換を実装
  - load_impl: serde_json::Value → Luaテーブル変換
  - save_impl: Luaテーブル → serde_json::Value変換
  - ネストしたテーブル、文字列、数値、ブール値、配列のサポート
  - _Requirements: 1.2, 1.3, 4.6_

- [x] 1.6 エラーハンドリングとロギング
  - ファイル不存在時は空テーブル返却、DEBUGログ出力
  - ファイル破損時は空テーブル返却、WARNログ出力
  - 保存失敗時はエラーログ出力、処理継続（パニック禁止）
  - リネーム失敗時は元ファイル保持、一時ファイル削除
  - _Requirements: 1.4, 1.5, 6.1, 6.3, 6.5_

## 2. PastaLuaRuntimeの拡張

- [x] 2.1 Runtime構造体への設定保持
  - PastaLuaRuntime構造体に`config: Option<PastaConfig>`フィールドを追加
  - from_loader_with_scene_dic関数でPastaConfigをLoaderContextから受け取り保持
  - _Requirements: 3.5_

- [x] 2.2 @pasta_persistenceモジュール登録
  - register_persistence_module関数を実装
  - from_loader_with_scene_dic内のPhase 5で登録（@enc登録後、main.lua読み込み前）
  - PersistenceConfigを取得し、persistence::register関数を呼び出し
  - _Requirements: 1.1_

- [x] 2.3 Drop実装による自動保存
  - PastaLuaRuntimeにDropトレイトを実装
  - save_persistence_data関数を実装（require("pasta.ctx").saveを取得）
  - Luaテーブルをserde_json::Valueに変換し、persistence::save_table_to_fileを呼び出し
  - エラー時はERRORログのみ出力、パニックしない
  - _Requirements: 3.1, 3.2, 3.4_

- [x] 2.4 明示的保存APIのサポート
  - @pasta_persistence.save(data)がLuaから呼び出し可能であることを確認
  - 定期保存、特定イベント後の保存シナリオをサポート
  - _Requirements: 3.3_

## 3. Luaスクリプト層の実装

- [x] 3.1 (P) pasta.saveモジュールの作成
  - `scripts/pasta/save.lua`を作成
  - require("@pasta_persistence").load()を呼び出し、結果を返す
  - シンプルな構造（追加メソッドなし、通常のテーブル操作）
  - _Requirements: 2.1, 2.2_

- [x] 3.2 ctx.luaの修正
  - `ctx.save = require("pasta.save")`を追加
  - STORE.saveへの参照を削除
  - _Requirements: 2.3, 2.4_

- [x] 3.3 (P) store.luaの修正
  - STORE.saveフィールドを完全削除
  - reset関数からSTORE.save初期化を削除
  - 破壊的変更として明記（後方互換性なし）
  - _Requirements: 2.5_

## 4. テストの実装

- [x] 4.1 (P) persistence.rsユニットテスト
  - JSON形式のロード/セーブテスト
  - gzip形式のロード/セーブテスト
  - ファイル不存在時の空テーブル返却テスト
  - ファイル破損時の空テーブル返却テスト
  - 形式自動判別テスト（gzipマジックヘッダー検出）
  - アトミック書き込みテスト（一時ファイル→リネーム）
  - ネストしたテーブルのシリアライズテスト
  - _Requirements: 4.1, 4.2, 4.3, 4.6, 6.1, 6.4, 7.1_

- [x] 4.2 (P) 統合テスト
  - `tests/persistence_integration_test.rs`を作成
  - ランタイム経由のロード/セーブラウンドトリップテスト
  - Drop時自動保存テスト
  - pasta.toml設定ファイルからの読み込みテスト
  - _Requirements: 3.1, 5.1, 7.1_

- [x]* 4.3 (P) Lua仕様テスト
  - `tests/lua_specs/persistence_spec.lua`を作成
  - ファイル不存在時の空テーブル返却テスト
  - データ保存・読み込みラウンドトリップテスト
  - ネストしたテーブルのテスト
  - _Requirements: 1.2, 1.3, 1.4_

## 5. 設定とドキュメント

- [x] 5.1 (P) Cargo.tomlへの依存追加
  - pasta_lua/Cargo.tomlにflate2 1.1依存を追加
  - serde_json既存依存を確認
  - mlua serialization feature有効化を確認
  - _Requirements: 4.5_

- [x] 5.2 デバッグモード実装
  - PersistenceConfigにdebug_modeフィールドを追加
  - debug_mode有効時に保存・読み込みログを出力
  - Luaテーブル↔Rust変換エラーの詳細レポート
  - _Requirements: 7.2, 7.3_

## 6. システム統合と検証

- [x] 6.1 モジュール初期化順序の検証
  - from_loader_with_scene_dic実装でPhase 5登録を確認
  - @pasta_persistenceがscene_dic.lua読み込み前に利用可能であることを検証
  - 既存の@pasta_config、@encパターンとの整合性確認
  - _Requirements: 1.1_

- [x] 6.2 エンドツーエンド統合テスト
  - 起動→データロード→変更→Drop保存→再起動→データ復元の完全フローテスト
  - obfuscate有効/無効の両モードでテスト
  - エラーシナリオ（破損ファイル、ディスク容量不足シミュレーション）のテスト
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 3.1, 3.2, 4.1, 4.2, 6.1, 6.2, 6.3, 6.4, 6.5_
