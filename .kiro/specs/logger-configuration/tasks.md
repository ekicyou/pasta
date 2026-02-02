# Implementation Plan

## Task Breakdown

### 1. LoggingConfig構造体の拡張

- [ ] 1.1 (P) LoggingConfigにlevelフィールドとfilterフィールドを追加
  - `level: String`フィールドを追加（デフォルト: "debug"）
  - `filter: Option<String>`フィールドを追加
  - `default_log_level()`ヘルパー関数を実装
  - serdeのdefault属性を設定
  - _Requirements: 1.1, 1.2, 1.3, 5.1_

- [ ] 1.2 (P) LoggingConfig::to_filter_directive()メソッドを実装
  - filter優先、filter未設定時はlevelを返すロジック
  - デフォルト値("debug")へのフォールバック
  - _Requirements: 1.4, 5.2_

- [ ] 1.3 (P) LoggingConfig::default()実装を更新
  - 既存フィールド（file_path, rotation_days）を維持
  - 新規フィールド（level="debug", filter=None）を追加
  - _Requirements: 8.1, 8.2_

- [ ] 1.4 (P) LoggingConfig拡張のユニットテストを作成
  - to_filter_directive()の動作検証（filter優先、level使用、デフォルト）
  - serdeデシリアライズテスト（既存フィールド + 新規フィールド）
  - default()実装の検証
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 5.1, 5.2_

### 2. PastaLuaRuntime::config()アクセサの追加

- [ ] 2.1 (P) PastaLuaRuntime::config()メソッドを実装
  - `pub fn config(&self) -> Option<&PastaConfig>`シグネチャ
  - self.config.as_ref()を返す実装
  - _Requirements: 6.1_

### 3. tracing subscriber初期化ロジックの実装

- [ ] 3.1 init_tracing_with_config()関数を実装
  - LoggingConfigを引数として受け取る
  - PASTA_LOG環境変数の優先処理（EnvFilter::try_from_env）
  - LoggingConfig::to_filter_directive()を使用したフィルタ構築
  - EnvFilter構築失敗時のフォールバック（デフォルト"debug"）
  - tracing_subscriber::registry().with()パターンでの初期化
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 6.2, 6.3, 6.4, 6.5_

- [ ] 3.2 PastaShiori::load()内でinit_tracing_with_config()を呼び出し
  - PastaLoader::load()完了後に実行
  - runtime.config().and_then(|c| c.logging())でLoggingConfigを取得
  - LoggingConfig::default()へのフォールバック
  - init_tracing_with_config()呼び出し
  - 即座にload_dirをINFOログ出力（"Logger initialized for ghost directory"）
  - _Requirements: 6.1, 6.2, 7.1, 7.2, 7.3_

- [ ] 3.3 windows.rs DllMainのinit_tracing()呼び出しを削除
  - DllMain内のinit_tracing()呼び出しを削除（コメント化または削除）
  - _Requirements: 6.1_

- [ ] 3.4 tracing subscriber初期化の統合テスト
  - pasta.toml `[logging].level = "info"`設定時の動作確認
  - pasta.toml `[logging].filter`設定時の動作確認
  - PASTA_LOG環境変数オーバーライドの動作確認
  - 既存pasta.toml（levelなし）での後方互換性確認
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 6.1, 6.2, 6.3, 6.4, 6.5, 8.1, 8.2, 8.3_

### 4. デフォルトログレベルの調整

- [ ] 4.1 (P) shiori.rsのログレベルをTRACEに変更
  - `SHIORI.load function cached` (171行目)
  - `SHIORI.request function cached` (183行目)
  - `SHIORI.unload function cached` (195行目)
  - `SHIORI.load returned true` (238行目)
  - `Processing SHIORI request` (144行目)
  - `SHIORI.request completed` (282行目)
  - debug!マクロをtrace!マクロに変更
  - _Requirements: 3.1_

- [ ] 4.2 (P) shiori.rsのログレベルをINFOに変更
  - `SHIORI.unload called successfully` (311行目)
  - debug!マクロをinfo!マクロに変更
  - _Requirements: 3.3_

- [ ] 4.3 (P) persistence.rsのログレベルをWARNに変更
  - `Persistence file not found` (170行目)
  - debug!マクロをwarn!マクロに変更
  - _Requirements: 3.2_

### 5. 200 OKレスポンスログの追加

- [ ] 5.1 (P) call_lua_request()に200 OKログを追加
  - レスポンス取得後、response.starts_with("SHIORI/3.0 200 OK")で判定
  - 200 OK時にリクエスト文字列をDEBUGログ出力
  - 200 OK時にレスポンス文字列をDEBUGログ出力
  - 文字列長制限なし（フル出力）
  - _Requirements: 4.1, 4.2, 4.3_

### 6. E2Eテストとドキュメント整合性確認

- [ ] 6.1* E2Eテストの作成
  - サンプルゴーストでのログ出力確認
  - ログレベル変更（TRACE/INFO/WARN）の反映確認
  - 200 OKレスポンス時のDEBUGログ出力確認
  - subscriber初期化直後のロードディレクトリINFOログ出力確認
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 8.1, 8.2, 8.3_

- [ ] 6.2 ドキュメント整合性の確認と更新
  - SOUL.md - コアバリュー・設計原則との整合性確認
  - SPECIFICATION.md - 言語仕様の更新確認（該当なし）
  - GRAMMAR.md - 文法リファレンスの同期確認（該当なし）
  - TEST_COVERAGE.md - 新規テストのマッピング追加
  - クレートREADME - API変更の反映（pasta_lua/README.md, pasta_shiori/README.md）
  - steering/* - 該当領域のステアリング更新確認（tech.md）

## Task Summary

- **合計**: 6メジャータスク、16サブタスク
- **並列実行可能**: 8タスク（(P)マーク付き）
- **オプショナル**: 1タスク（E2Eテスト - 受け入れ基準準拠、MVP後に実施可）
- **推定工数**: 各サブタスク1-3時間、総計24-48時間

## Requirements Coverage

全8要件（Requirement 1-8）がタスクにマッピング済み：

- **Requirement 1** (LoggingConfig拡張): Task 1.1, 1.2, 1.3, 1.4
- **Requirement 2** (EnvFilterフィルタリング): Task 3.1, 3.4
- **Requirement 3** (ログレベル調整): Task 4.1, 4.2, 4.3
- **Requirement 4** (200 OKログ): Task 5.1
- **Requirement 5** (pasta.toml設定スキーマ): Task 1.1, 1.2
- **Requirement 6** (tracing subscriber遅延初期化): Task 2.1, 3.1, 3.2, 3.3, 3.4
- **Requirement 7** (ロードディレクトリ記録): Task 3.2
- **Requirement 8** (後方互換性): Task 1.3, 3.4

## Implementation Notes

### 並列実行可能タスク

以下のタスクグループは並列実行可能（境界が独立、リソース競合なし）：

**Group A** (pasta_lua/loader): Task 1.1, 1.2, 1.3, 1.4
**Group B** (pasta_lua/runtime): Task 2.1
**Group C** (ログレベル調整): Task 4.1, 4.2, 4.3, 5.1

**Group D** (統合): Task 3.1, 3.2, 3.3 は順次実行（3.1→3.2→3.3の順）

### 推奨実装順序

1. **Phase 1** (並列): Task 1.1-1.4, 2.1 - 基盤コンポーネント
2. **Phase 2** (順次): Task 3.1→3.2→3.3 - 統合ロジック
3. **Phase 3** (並列): Task 4.1-4.3, 5.1 - ログレベル調整
4. **Phase 4** (順次): Task 3.4, 6.1, 6.2 - テスト・検証・ドキュメント
