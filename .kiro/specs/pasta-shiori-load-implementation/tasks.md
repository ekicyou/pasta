# Implementation Plan

## Task Overview

実装タスクは以下の順序で進めます：

1. **基盤構築**: ロギング基盤（GlobalLoggerRegistry, PastaLogger）
2. **pasta_lua拡張**: LoggingConfig, PastaLuaRuntime拡張, PastaLoader統合
3. **pasta_shiori実装**: load()実装、Span設定、エラーハンドリング
4. **統合テスト**: E2Eテスト、複数インスタンステスト

---

## Implementation Tasks

### Phase 1: ロギング基盤構築

- [ ] 1. GlobalLoggerRegistryの実装
- [ ] 1.1 (P) シングルトンレジストリ構造を実装
  - Arc<Mutex<HashMap<PathBuf, Arc<PastaLogger>>>>でロガー管理
  - instance()メソッドでOnceLockパターン
  - register()/unregister()メソッド実装
  - _Requirements: 7.7_

- [ ] 1.2 (P) MakeWriter実装でSpan識別ログ振り分け
  - Span::current()からload_dirフィールド取得
  - HashMap検索で対応PastaLoggerを探索
  - 該当なしはio::sink()にフォールバック
  - _Requirements: 7.7_

- [ ] 1.3 (P) GlobalLoggerRegistryのユニットテスト
  - register/unregisterの動作確認
  - Span設定時のロガー振り分け確認
  - 該当なしログの破棄確認
  - _Requirements: 7.7_

- [ ] 2. PastaLogger実装
- [ ] 2.1 (P) 構造体定義とnew()メソッド
  - log_path, rotation_days, _guardフィールド
  - RollingFileAppender（daily, max_log_files）生成
  - non_blocking::WorkerGuard確保
  - _Requirements: 7.1, 7.3, 7.4_

- [ ] 2.2 (P) ログディレクトリ自動作成とパス検証
  - profile/pasta/logs/配下のみ許可
  - パストラバーサル防止（正規化と検証）
  - ディレクトリ作成権限エラー時のフォールバック
  - _Requirements: 7.6, 7.8_

- [ ] 2.3 (P) PastaLoggerのユニットテスト
  - ディレクトリ自動作成確認
  - max_log_filesローテーション動作確認
  - パス検証とセキュリティチェック
  - _Requirements: 7.3, 7.4, 7.6, 7.8_

### Phase 2: pasta_lua層の拡張

- [ ] 3. LoggingConfig実装
- [ ] 3.1 (P) 構造体定義とデシリアライズ
  - file_path, rotation_daysフィールド
  - デフォルト値関数（default_file_path, default_rotation_days）
  - serde Deserialize実装
  - _Requirements: 7.2, 7.3, 7.4_

- [ ] 3.2 (P) PastaConfig::logging()メソッド追加
  - custom_fieldsから[logging]セクション取得
  - Option<LoggingConfig>を返却
  - デシリアライズエラー時はNone
  - _Requirements: 7.2_

- [ ] 3.3 (P) LoggingConfigのユニットテスト
  - [logging]セクション読み込み確認
  - デフォルト値適用確認
  - セクション不存在時のNone確認
  - _Requirements: 7.2, 7.3, 7.4_

- [ ] 4. PastaLuaRuntime拡張
- [ ] 4.1 loggerフィールド追加とDrop実装
  - logger: Option<PastaLogger>フィールド追加
  - Drop時のログフラッシュ確認
  - ライフサイクル一致の保証
  - _Requirements: 1.1, 1.2, 7.7_

- [ ] 4.2 PastaLuaRuntime拡張のユニットテスト
  - loggerフィールドの保持確認
  - Drop時のWorkerGuardフラッシュ確認
  - _Requirements: 1.2, 7.7_

- [ ] 5. PastaLoader::load()統合
- [ ] 5.1 ロガー初期化ロジック追加
  - PastaConfig::logging()で[logging]セクション取得
  - LoggingConfig存在時にPastaLogger::new()呼び出し
  - 生成したloggerをPastaLuaRuntimeに格納
  - _Requirements: 7.1, 7.5_

- [ ] 5.2 PastaLoader::load()の統合テスト
  - [logging]セクションあり時のロガー生成確認
  - [logging]セクションなし時のlogger=None確認
  - 既存13テストの無修正動作確認
  - _Requirements: 7.1, 7.5_

### Phase 3: pasta_shiori層の実装

- [ ] 6. MyError拡張
- [ ] 6.1 (P) From<LoaderError>実装
  - 各LoaderErrorバリアントをMyError::Load(String)に変換
  - format!()でエラー詳細メッセージ生成
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 6.2 (P) MyError変換のユニットテスト
  - 各LoaderErrorバリアントの変換確認
  - エラーメッセージ内容確認
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 7. RawShiori::new()でSubscriber初期化
- [ ] 7.1 グローバルSubscriber初期化ロジック
  - OnceLockパターンでset_global_default()を1回のみ実行
  - GlobalLoggerRegistry::instance()をMakeWriterに設定
  - 初期化失敗時のフォールバック（ログ無効化、panic回避）
  - _Requirements: 7.1_

- [ ] 7.2 Subscriber初期化の統合テスト
  - 1回目のRawShiori::new()成功確認
  - 2回目以降のno-op動作確認
  - 初期化失敗時のエラーハンドリング確認
  - _Requirements: 7.1_

- [ ] 8. PastaShiori::load()実装
- [ ] 8.1 load_dir存在確認とバリデーション
  - PathBuf変換と存在チェック
  - 不存在時はDirectoryNotFoundエラー
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 8.2 Span設定とPastaLoader::load()呼び出し
  - info_span!("shiori_load", load_dir = %path)でSpan作成
  - Span::enter()で全配下ログにload_dir付与
  - PastaLoader::load(load_dir)呼び出し
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 8.3 GlobalLoggerRegistryへのロガー登録
  - PastaLuaRuntime内のloggerをArc化
  - GlobalLoggerRegistry::register(load_dir, logger)呼び出し
  - load_dir存在時のみ登録（loggerがNoneなら登録スキップ）
  - _Requirements: 7.7_

- [ ] 8.4 runtimeフィールド保持とhinst保存
  - runtime: Option<PastaLuaRuntime>にSome設定
  - hinst: isizeに保存
  - load_dir: Option<PathBuf>に保存
  - _Requirements: 1.2, 5.1, 5.2, 6.2_

- [ ] 8.5 エラーハンドリングとログ出力
  - LoaderError → MyError変換（From実装経由）
  - エラー時はerror!()でログ出力
  - falseを返却（SHIORI規約）
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 8.6 複数回load()時の既存ランタイム解放
  - 既存runtimeがSomeの場合はDrop
  - 新しいruntimeで上書き
  - _Requirements: 1.3_

- [ ] 9. PastaShiori::request()実装
- [ ] 9.1 Span設定（debug_span）
  - debug_span!("shiori_request", load_dir = %path)でSpan作成
  - 高頻度ログのため、debugレベル
  - _Requirements: 6.1_

- [ ] 9.2 runtime初期化確認
  - runtimeがNoneの場合はNotInitializedエラー
  - _Requirements: 6.1_

- [ ] 10. PastaShiori::unload()実装（将来対応）
- [ ] 10.1 GlobalLoggerRegistry::unregister()呼び出し
  - load_dirで登録解除
  - runtime=Noneに設定（Dropでランタイム解放）
  - _Requirements: 6.3_

### Phase 4: 統合テストと検証

- [ ] 11. PastaShiori::load()統合テスト
- [ ] 11.1 成功パステスト
  - 有効なload_dirでload()成功確認
  - runtimeがSomeになることを確認
  - trueが返ることを確認
  - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [ ] 11.2 エラーパステスト
  - load_dir不存在時のfalse返却確認
  - pasta.toml不存在時のConfigNotFoundエラー確認
  - エラーログ出力確認
  - _Requirements: 2.3, 3.1, 3.2, 3.3, 4.1, 4.2_

- [ ] 11.3 複数回load()テスト
  - 既存ランタイム解放確認
  - 新しいランタイム生成確認
  - _Requirements: 1.3_

- [ ] 12. E2Eテスト：SHIORI load → request → unloadサイクル
- [ ] 12.1 完全ライフサイクルテスト
  - load()成功 → request()実行可能 → unload()でクリーンアップ
  - 各フェーズでログ出力確認
  - _Requirements: 1.1, 1.2, 1.3, 6.1, 6.2, 6.3_

- [ ] 13. 複数インスタンス同時ロードテスト
- [ ] 13.1 独立ログファイル出力確認
  - ghost1とghost2を同時ロード
  - 各インスタンスが独立したログファイルに出力
  - Span識別による振り分け動作確認
  - _Requirements: 7.7_

- [ ] 13.2 ログローテーション動作確認
  - rotation_days設定に基づく古いログ削除確認
  - max_log_files動作確認
  - _Requirements: 7.4_

- [ ]*14. 受入基準検証（オプション）
- [ ]*14.1 要件1（Runtime管理）の全受入基準確認
  - 1.1: PastaLoader::load()使用確認
  - 1.2: PastaLuaRuntimeインスタンス保持確認
  - 1.3: 複数回load()時の既存ランタイム破棄確認
  - _Requirements: 1.1, 1.2, 1.3_

- [ ]*14.2 要件7（ロギング）の全受入基準確認
  - 7.1～7.8: 各受入基準の動作確認
  - ログファイルパス、ローテーション、複数インスタンス独立性
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1, 1.2, 1.3 | 4.1, 8.2, 8.4, 8.6, 11.1, 11.3, 12.1, 14.1 |
| 2.1, 2.2, 2.3 | 8.1, 11.1, 11.2 |
| 3.1, 3.2, 3.3, 3.4 | 6.1, 6.2, 8.5, 11.2 |
| 4.1, 4.2 | 11.2 |
| 5.1, 5.2 | 8.4 |
| 6.1, 6.2, 6.3 | 9.1, 9.2, 10.1, 12.1, 14.1 |
| 7.1 | 2.1, 5.1, 5.2, 7.1, 7.2, 14.2 |
| 7.2 | 3.1, 3.2, 3.3, 14.2 |
| 7.3, 7.4 | 2.1, 2.3, 3.1, 3.3, 13.2, 14.2 |
| 7.5 | 5.1, 5.2, 14.2 |
| 7.6 | 2.2, 2.3, 14.2 |
| 7.7 | 1.1, 1.2, 1.3, 4.1, 4.2, 8.3, 13.1, 14.2 |
| 7.8 | 2.2, 2.3, 14.2 |

**全7要件（26受入基準）がタスクにマッピング済み**

---

## Notes

- `(P)` マーカーは並列実行可能なタスクを示します
- `*` マーカーはMVP後の検証タスク（任意）を示します
- Phase 1-2は並列実行可能（pasta_shiori層とpasta_lua層の独立性）
- Phase 3はPhase 2完了後に実行（PastaLoader統合が前提）
- Phase 4は全実装完了後に実行
