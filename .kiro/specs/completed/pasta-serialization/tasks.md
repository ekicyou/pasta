# Implementation Plan

## Phase 1: Core Extension (エンジン基盤拡張)

- [ ] 1. PastaEngineに永続化パス管理機能を追加
- [ ] 1.1 (P) エンジン構造体に永続化パスフィールドを追加
  - `PastaEngine`に`persistence_path: Option<PathBuf>`フィールドを追加
  - `new_with_persistence`コンストラクタを実装（絶対パス・相対パス対応）
  - `with_persistence_and_random_selector`コンストラクタを実装
  - パス検証ロジック（存在確認、`is_dir()`、`canonicalize`）を実装
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 4.1, 4.2_

- [ ] 1.2 (P) 実行コンテキスト構築機能を実装
  - `build_execution_context`メソッドを実装（`HashMap<String, String>`生成）
  - `persistence_path`フィールドをコンテキストに設定（パスありは絶対パス文字列、なしは空文字列）
  - `rune::to_value`でHashMapをRune `Value`型に変換
  - `execute_label_with_filters`でコンテキストをVM実行時に渡すよう変更（`vm.execute(hash, (context,))`）
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] 1.3 (P) エラー型を拡張
  - `PastaError`に`PersistenceDirectoryNotFound`バリアント追加
  - `PastaError`に`InvalidPersistencePath`バリアント追加
  - エラーメッセージに構造化フィールド（`path`）を含める
  - _Requirements: 1.4, 7.1_

- [ ] 1.4 (P) トランスパイラのラベル関数シグネチャを変更
  - `transpiler/mod.rs`のラベル関数生成ロジックを修正（`pub fn label_name(ctx)`形式）
  - 既存サンプル・テストへの影響がないことを確認（Runeは未使用引数を許容）
  - _Requirements: 2.5_

## Phase 2: Runtime Support (stdlib拡張とテスト基盤)

- [ ] 2. persistence stdlib moduleを実装
- [ ] 2.1 (P) TOML関数を実装
  - `Cargo.toml`に`toml = "0.8"`依存を追加
  - `crates/pasta/src/stdlib/persistence.rs`を作成
  - `toml_to_string(data: rune::Value) -> Result<String, String>`を実装
  - `toml_from_string(toml_str: &str) -> Result<rune::Value, String>`を実装
  - Rune `Value`型と`toml::Value`型の相互変換ロジックを実装
  - _Requirements: 5.1, 5.4_

- [ ] 2.2 (P) ファイルI/O関数を実装
  - `read_text_file(path: &str) -> Result<String, String>`を実装
  - `write_text_file(path: &str, content: &str) -> Result<(), String>`を実装
  - エラーハンドリング（ファイル存在確認、権限エラー等）を実装
  - _Requirements: 5.2_

- [ ] 2.3 エンジン初期化時にpersistence moduleを登録
  - `create_persistence_module()`関数を実装
  - `PastaEngine`のコンテキスト初期化で`context.install(persistence::create_persistence_module()?)`を呼び出し
  - 4関数がRuneスクリプトから呼び出し可能であることを確認
  - _Requirements: 2.6_

- [ ] 3. テストインフラを構築
- [ ] 3.1 (P) テストフィクスチャを準備
  - `tests/fixtures/persistence/`ディレクトリを作成
  - サンプルTOMLファイル（`sample_save.toml`, `sample_config.toml`）を作成
  - _Requirements: 3.5_

- [ ] 3.2 (P) ユニットテストを実装
  - `Cargo.toml`の`[dev-dependencies]`に`tempfile = "0.3"`を追加
  - 絶対パス指定テスト（`test_new_with_persistence_absolute_path`）
  - 相対パス指定テスト（`test_new_with_persistence_relative_path`）
  - パスなし初期化テスト（`test_new_without_persistence`）
  - 無効パスエラーテスト（`test_invalid_persistence_path`）
  - コンテキスト生成テスト（パスあり/なし）
  - _Requirements: 6.1, 6.2, 6.7_

- [ ] 3.3 統合テストを実装
  - `tests/persistence_test.rs`を作成
  - Runeスクリプトから`ctx["persistence_path"]`アクセステスト
  - TOML保存・読み込みテスト（`toml_to_string`, `toml_from_string`）
  - tempfile一時ディレクトリ自動削除テスト
  - 複数エンジンインスタンス独立性テスト（異なる永続化パス）
  - トランスパイラ生成コード確認テスト（`pub fn label_name(ctx)`シグネチャ）
  - テスト時のフィクスチャコピーロジック実装
  - _Requirements: 2.4, 3.1, 3.2, 3.3, 3.4, 6.3, 6.4, 6.5, 6.6_

## Phase 3: Logging & Documentation (ロギングとドキュメント)

- [ ] 4. 構造化ロギングを実装
- [ ] 4.1 (P) tracingマクロを統合
  - エンジン初期化成功時に`info!`ログを出力（`path`フィールド）
  - 永続化ディレクトリ不在時に`error!`ログを出力（`path`, `error`フィールド）
  - 永続化パスなし初期化時に`debug!`ログを出力
  - コンテキスト構築時に`debug!`ログを出力（オプション、高頻度なら省略）
  - すべてのログに構造化フィールドを含める
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 5. Rune開発者向けドキュメントを作成
- [ ] 5.1 (P) 永続化実装ガイドを作成
  - `doc/rune-persistence-guide.md`を作成
  - 永続化パスの取得方法（`ctx["persistence_path"]`）を説明
  - TOML保存・読み込みのサンプルコードを提供
  - ファイルI/O関数（`read_text_file`, `write_text_file`）の使用例を提供
  - 永続化パスなし時の処理例を提供
  - _Requirements: 2.6, 5.1, 5.2, 5.4, 5.5_

- [ ] 5.2 (P) セキュリティベストプラクティスを文書化
  - パストラバーサル攻撃の脅威を説明
  - 固定ファイル名の使用を推奨
  - ホワイトリスト検証のサンプルを提供
  - サニタイズ処理のサンプルを提供（`../`, `/`, `\`除去）
  - エラーハンドリングのベストプラクティス（try-catchパターン）を説明
  - _Requirements: 5.3, 7.6_

- [ ] 5.3 (P) API docコメントを追加
  - `PastaEngine::new_with_persistence`のdocコメント作成
  - `PastaEngine::with_persistence_and_random_selector`のdocコメント作成
  - stdlib persistence関数のdocコメント作成
  - _Requirements: 2.6_

## Requirements Coverage

全7カテゴリ40要件をカバー:

- **Requirement 1** (1.1-1.5): エンジン初期化時の永続化パス指定 → タスク1.1, 1.2, 1.3
- **Requirement 2** (2.1-2.6): 永続化パスのRuneスクリプトへの提供 → タスク1.2, 1.4, 2.3, 5.1, 5.3
- **Requirement 3** (3.1-3.5): テスト用永続化ディレクトリの管理 → タスク3.1, 3.3
- **Requirement 4** (4.1-4.5): エンジン内部での永続化パス管理 → タスク1.1
- **Requirement 5** (5.1-5.5): Runeスクリプトでの永続化実装ガイダンス → タスク2.1, 2.2, 5.1, 5.2
- **Requirement 6** (6.1-6.7): テストカバレッジ → タスク3.2, 3.3
- **Requirement 7** (7.1-7.6): エラーハンドリングとロギング → タスク1.3, 4.1, 5.2
