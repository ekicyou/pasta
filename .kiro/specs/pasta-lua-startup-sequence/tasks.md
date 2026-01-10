# Implementation Plan

## タスク概要

pasta-lua-startup-sequence機能の実装タスクを定義する。全7要件をカバーし、新規loaderモジュールの実装、既存PastaLuaRuntimeの拡張、統合テストを含む。

**総タスク数**: 6メジャータスク、18サブタスク  
**平均タスクサイズ**: 1-3時間/サブタスク  
**要件カバレッジ**: 全7要件（1-7）完全カバー

---

## 実装タスク

### Phase 1: 基盤コンポーネント実装

- [x] 1. loaderモジュール基盤構築
- [x] 1.1 (P) エラー型定義
  - LoaderError型階層をthiserrorで実装（Config, Io, GlobPattern, Parse, Transpile, Runtime）
  - Display traitによる人間可読エラーメッセージ
  - From<T>トレイトで既存エラー型からの変換実装
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 1.2 (P) 設定ファイル解析機能
  - PastaConfig構造体実装（loader + custom_fields）
  - LoaderConfig構造体実装（4フィールド + デフォルト関数）
  - pasta.toml読み込み・デシリアライズ処理
  - ファイル不在時のデフォルト設定生成
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 1.3 (P) ファイル探索機能
  - Discovery内部モジュール実装
  - glob crateによるパターンマッチング（`dic/*/*.pasta`）
  - profile/除外ロジック
  - ディレクトリ存在チェックとエラーハンドリング
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 1.4 (P) LoaderContext定義
  - LoaderContext構造体実装（base_dir, lua_search_paths, custom_fields）
  - PastaConfigからLoaderContextへの変換処理
  - _Requirements: 4.1, 7.1_

### Phase 2: 起動シーケンス実装

- [x] 2. PastaLoader統合API実装
- [x] 2.1 ディレクトリ準備処理
  - `profile/pasta/save/`, `profile/pasta/save/lua/` ディレクトリ自動作成
  - `profile/pasta/cache/lua/` 削除と再作成
  - _Requirements: 2.6, 3.7_

- [x] 2.2 ファイル探索とトランスパイル統合
  - Discovery呼び出しでPastaファイル収集
  - pasta_core::parse_fileによるパース実行
  - LuaTranspiler呼び出しで各ファイルトランスパイル
  - 共有TranspileContextでシーン・単語レジストリ蓄積
  - エラー発生時のファイル名・行番号付きエラー返却
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 6.2, 6.3_

- [x] 2.3 キャッシュファイル保存処理
  - トランスパイル結果を`profile/pasta/cache/lua/`に保存
  - ソースパスからキャッシュファイル名生成（パス区切り文字`\`を`_`に置換）
  - debug_mode設定に基づく保存制御
  - _Requirements: 3.5, 3.6_

- [x] 2.4 統合起動APIメソッド実装
  - PastaLoader::load()実装（5フェーズシーケンス）
  - PastaLoader::load_with_config()実装
  - tracing統合による進捗ログ出力
  - エラーハンドリングとLoaderError返却
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

### Phase 3: ランタイム拡張

- [x] 3. PastaLuaRuntime拡張機能実装
- [x] 3.1 from_loader()コンストラクタ実装
  - TranspileContextとLoaderContextを引数に取るコンストラクタ
  - 内部でpackage.path設定とトランスパイル結果ロード実行
  - _Requirements: 4.1_

- [x] 3.2 package.path設定処理
  - 起動ディレクトリ基準の絶対パス生成（4階層）
  - mlua APIによるpackage.path書き込み
  - 優先順位の正しい設定（save/lua → scripts → cache/lua → scriptlibs）
  - カレントディレクトリ非依存設計
  - _Requirements: 4.2, 4.6_

- [x] 3.3 トランスパイル結果ロード処理（Option B実装）
  - mlua::Lua::load()でメモリ上のバイト列から直接実行
  - モジュール名設定（set_name）
  - キャッシュファイルはデバッグ用途のみ（実行には不使用）
  - エラーハンドリングとLoaderError::RuntimeError返却
  - _Requirements: 4.3, 4.5_

- [x] 3.4 @pasta_configモジュール実装
  - register_config_module()内部メソッド実装
  - toml::Table → mlua::Tableへの変換（toml_to_lua実装）
  - ネスト構造・配列・プリミティブ型の忠実なマッピング
  - 読み取り専用設定の提供
  - カスタムフィールドが空の場合の空テーブル返却
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

### Phase 4: テスト実装

- [x] 4. ユニットテスト実装
- [x] 4.1 (P) PastaConfig デシリアライズテスト
  - 有効なpasta.toml読み込み検証
  - デフォルト値検証
  - カスタムフィールド解析検証
  - 無効なTOML形式のエラーハンドリング検証
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 4.2 (P) Discoveryファイル探索テスト
  - `dic/*/*.pasta`パターンマッチング検証
  - profile/除外検証
  - ディレクトリ不在時のエラー検証
  - 空ディレクトリ時の警告検証
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 4.3 (P) package.path設定テスト
  - 4階層パス生成検証
  - 絶対パス変換検証
  - 優先順位検証
  - _Requirements: 4.2, 4.6_

- [x] 4.4 (P) @pasta_configモジュールテスト
  - TOML → Luaテーブル変換検証
  - ネスト構造・配列・プリミティブ型マッピング検証
  - 空設定時の空テーブル返却検証
  - Lua側からのrequire動作検証
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

### Phase 5: 統合テスト

- [x] 5. 統合テスト実装
- [x] 5.1 全起動シーケンステスト
  - 最小構成（dic/hello.pastaのみ）での起動検証
  - pasta.toml付き構成での起動検証
  - カスタム設定フィールド付き構成での起動検証
  - 複数Pastaファイルの統合トランスパイル検証
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5.2 エラーケース統合テスト
  - ディレクトリ不在エラー
  - パースエラー（ファイル名・行番号情報検証）
  - トランスパイルエラー
  - ランタイム初期化エラー
  - 設定ファイル解析エラー
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 5.3* テストフィクスチャ整備
  - tests/fixtures/loader/minimal/ 構造作成
  - tests/fixtures/loader/with_config/ 構造作成
  - tests/fixtures/loader/with_custom_config/ 構造作成
  - サンプルpasta.toml作成
  - サンプルPastaスクリプト作成
  - _Requirements: 1.1, 2.1, 3.1, 5.1_

### Phase 6: モジュール統合とエクスポート

- [x] 6. lib.rs統合
- [x] 6.1 loaderモジュール公開
  - src/lib.rsにloaderモジュール追加
  - PastaLoader, PastaConfig, LoaderError re-export
  - ドキュメントコメント追加
  - _Requirements: 5.1, 5.5_

---

## 実装完了サマリー

**完了日**: 2025-06-28  
**テスト結果**: 全テスト合格（200+ テスト）  

### 主な成果物
- `crates/pasta_lua/src/loader/` - 新規loaderモジュール（5ファイル）
  - `mod.rs` - PastaLoader統合API
  - `config.rs` - pasta.toml解析
  - `discovery.rs` - ファイル探索
  - `context.rs` - LoaderContext（package.path生成）
  - `error.rs` - LoaderError型階層
- `crates/pasta_lua/tests/loader_integration_test.rs` - 13統合テスト
- `tests/fixtures/loader/` - 3テストシナリオ

### 修正されたバグ
- Windows `canonicalize()` の `\\?\` プレフィックス問題
- Lua `require("pasta")` でのディレクトリモジュール解決（`?/init.lua` パターン追加）
- 非存在ディレクトリのエラーハンドリング

---

## 品質検証

### ✅ 要件カバレッジ
- **Requirement 1** (起動ディレクトリ探索): タスク1.3, 4.2
- **Requirement 2** (設定ファイル解釈): タスク1.2, 2.1, 4.1
- **Requirement 3** (複数ファイルトランスパイル): タスク2.2, 2.3, 5.1
- **Requirement 4** (ランタイム初期化): タスク3.1, 3.2, 3.3, 4.3
- **Requirement 5** (統合起動API): タスク2.4, 5.1, 6.1
- **Requirement 6** (エラーハンドリング): タスク1.1, 2.2, 5.2
- **Requirement 7** (@pasta_configモジュール): タスク1.4, 3.4, 4.4

### ✅ タスク依存関係
- Phase 1: 並列実行可能（各コンポーネント独立）
- Phase 2: Phase 1完了後、順次実行
- Phase 3: Phase 2完了後、順次実行
- Phase 4: 対応実装完了後、並列実行可能
- Phase 5: Phase 3完了後、実装
- Phase 6: 全実装完了後、最終統合

### ✅ テストカバレッジ
- ユニットテスト: 各コンポーネント（1.1-1.4, 3.1-3.4）に対応
- 統合テスト: エンドツーエンド起動シーケンス、エラーケース網羅
- テストフィクスチャ: 3パターン（最小/設定付き/カスタム設定）

---

## 完了

✅ **全18サブタスク完了**  
✅ **全7要件カバー**  
✅ **全200+テスト合格**

この仕様の実装は正常に完了しました。
