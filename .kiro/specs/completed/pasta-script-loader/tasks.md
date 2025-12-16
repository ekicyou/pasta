# Implementation Tasks - pasta-script-loader

## Task Overview

全9要件をカバーする実装タスク。DirectoryLoaderによるファイル収集、PastaEngineへの統合、エラーハンドリング、テストフィクスチャ整備、統合テストを含む。

---

## Tasks

- [ ] 1. エラー型定義の拡張
- [ ] 1.1 (P) ディレクトリローダー用エラー型の追加
  - `NotAbsolutePath`, `DirectoryNotFound`, `NotADirectory`, `PermissionDenied`エラーvariantを追加
  - `DicDirectoryNotFound`, `MainRuneNotFound`エラーvariantを追加
  - `ParseError`構造体を定義（file, line, column, messageフィールド）
  - `MultipleParseErrors` variantを追加（`errors: Vec<ParseError>`フィールド）
  - `From<&PastaError> for Option<ParseError>`変換実装を追加
  - 各エラーに対して`std::error::Error`トレイト実装を確認
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 7.9_

- [ ] 2. ディレクトリローダーの実装
- [ ] 2.1 (P) LoadedFiles構造体の定義
  - `script_root: PathBuf`, `pasta_files: Vec<PathBuf>`, `main_rune: PathBuf`フィールドを持つ構造体を定義
  - 不変データ構造として実装（構築後は変更不可）
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 2.2 (P) DirectoryLoaderのディレクトリ検証機能
  - `validate_directory(path: &Path)`メソッドを実装
  - 絶対パスチェック（`path.is_absolute()`）、相対パスは`NotAbsolutePath`エラー
  - ディレクトリ存在チェック、不在時は`DirectoryNotFound`エラー
  - ディレクトリ種別チェック（`metadata().is_dir()`）、ファイルの場合は`NotADirectory`エラー
  - 読み取り権限検証（`read_dir()`試行）、失敗時は`PermissionDenied`エラー
  - fail-fast戦略：各検証で即座にエラーを返却
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [ ] 2.3 dic/ディレクトリとmain.runeの検証
  - `check_main_rune(script_root: &Path)`メソッドを実装し、`main.rune`の存在を確認
  - `main.rune`不在時は`MainRuneNotFound`エラーを返却
  - `dic/`サブディレクトリの存在を確認、不在時は`DicDirectoryNotFound`エラーを返却
  - _Requirements: 2.3, 2.10, 2.11_

- [ ] 2.4 .pastaファイル収集機能
  - `collect_pasta_files(dic_path: &Path)`メソッドを実装
  - `dic/**/*.pasta`パターンで再帰的にファイルを検索（globクレート使用）
  - `.pasta`拡張子を大文字小文字区別なく認識（`.PASTA`も含む）
  - `_`で始まるファイルをスキップ
  - 隠しファイル（`.`で始まる）を自動除外
  - ファイル探索順序は保証せず（ファイルシステム依存）
  - 収集したファイルパスを`Vec<PathBuf>`として返却
  - _Requirements: 2.1, 2.2, 2.5, 2.6, 2.7, 2.8_

- [ ] 2.5 DirectoryLoader::load()の統合
  - `load(script_root: &Path) -> Result<LoadedFiles>`公開メソッドを実装
  - Step 1: `validate_directory()`でディレクトリ検証
  - Step 2: `check_main_rune()`と`dic/`検証
  - Step 3: `collect_pasta_files()`でファイル収集
  - Step 4: `LoadedFiles`構造体を構築して返却
  - `.pasta`ファイルが0件の場合は警告ログを出力（tracingクレート使用）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 2.1, 2.2, 2.3, 2.10, 2.11, 2.12_

- [ ] 3. エラーログライターの実装
- [ ] 3.1 (P) ErrorLogWriter構造体の実装
  - `pub(crate) struct ErrorLogWriter`を定義
  - `log(script_root: &Path, errors: &[ParseError])`メソッドを実装
  - 各エラーをtracing::info!()で出力（フォーマット: "パースエラー: {}:{} - {}", file, line, message）
  - エラー件数をサマリーログで出力（"合計 N 件のパースエラー (script_root: ...)"）
  - ファイル出力機能は削除（tracingクレートのsubscriberがログ先を制御）
  - _Requirements: 3.5, 3.6_

- [ ] 4. PastaEngineのディレクトリ初期化機能
- [ ] 4.1 merge_asts()ヘルパー関数の実装
  - `merge_asts(asts: Vec<Ast>) -> Ast`関数を実装
  - 全ASTの`labels`フィールドを1つの`Vec`に統合
  - 統合後のラベルリストを持つ`Ast`構造体を返却
  - 全ファイル間でラベル連番がユニークになることを保証
  - _Requirements: 3.1, 3.2, 3.3, 4.1, 4.2_

- [ ] 4.2 from_directory()の基本フロー実装
  - `pub fn from_directory(path: impl AsRef<Path>) -> Result<Self>`メソッドを実装
  - Step 1: `DirectoryLoader::load()`でファイル収集
  - Step 2: 全.pastaファイルをパース、エラーは収集（fail-fastしない）
  - Step 3: パースエラーがある場合、`ErrorLogWriter::log()`で出力し`MultipleParseErrors`を返却
  - Step 4: `merge_asts()`で全ASTを統合
  - Step 5: `Transpiler::transpile()`でRuneソースへ変換
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [ ] 4.3 Rune統合とコンパイル
  - Step 6: `rune::Sources`を作成し、トランスパイル結果を"entry"として追加
  - Step 7: `main.rune`を`Sources`に追加（`Source::from_path()`使用）
  - Step 8: `rune::prepare().with_context().with_diagnostics().build()`でコンパイル
  - Runeコンパイルエラー発生時は`PastaError::RuneCompileError`を返却（fail-fast）
  - _Requirements: 3.9, 3.10, 3.11, 3.12_

- [ ] 4.4 LabelTableとエンジン構築
  - Step 9: `LabelTable::from_labels()`でラベルテーブルを構築
  - Step 10: `unit`, `runtime`, `label_table`フィールドを持つ`PastaEngine`を返却
  - `DefaultRandomSelector`を使用（カスタムセレクタはwith_selector版で対応）
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

- [ ] 4.5 (P) from_directory_with_selector()の実装
  - `pub fn from_directory_with_selector(path: impl AsRef<Path>, selector: Box<dyn RandomSelector>) -> Result<Self>`を実装
  - `from_directory()`と同じフローだが、Step 9でカスタム`selector`を使用
  - _Requirements: 9.2_

- [ ] 4.6 (P) ラベル列挙APIの実装
  - `pub fn list_labels(&self) -> Vec<String>`メソッドを実装（全ラベル列挙）
  - `pub fn list_global_labels(&self) -> Vec<String>`メソッドを実装（グローバルラベルのみ）
  - `LabelTable`内のラベル情報から名前を抽出
  - _Requirements: 9.4, 9.5_

- [ ] 5. lib.rsへの統合
- [ ] 5.1 (P) loader.rsモジュールの追加とre-export
  - `crates/pasta/src/loader.rs`ファイルを作成
  - `lib.rs`に`mod loader;`宣言を追加
  - `pub use loader::{DirectoryLoader, LoadedFiles};`でpublic API公開
  - `pub(crate) use loader::ErrorLogWriter;`でクレート内部可視化
  - `pub use error::ParseError;`を追加（新規public API）
  - _Requirements: 9.1_

- [ ] 6. テストフィクスチャの整備
- [ ] 6.1 (P) テストプロジェクト基本構造の作成
  - `crates/pasta/tests/fixtures/test-project/`ディレクトリを作成
  - `main.rune`を作成（最小限のRune実装、空の`main()`関数）
  - `dic/`サブディレクトリを作成
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 6.2 (P) 基本スクリプトファイルの作成
  - `dic/greetings.pasta`を作成（基本会話スクリプト、同名ラベル×3を含む）
  - `dic/sakura_script.pasta`を作成（さくらスクリプトサンプル）
  - `dic/variables.pasta`を作成（変数操作スクリプト）
  - 各ファイルは有効なPasta DSL構文を持つ
  - _Requirements: 5.4, 5.5, 5.6_

- [ ] 6.3 (P) サブディレクトリとフィルタリングテスト用ファイル
  - `dic/special/`サブディレクトリを作成
  - `dic/special/holiday.pasta`を作成（サブディレクトリ走査テスト用）
  - `dic/_ignored.pasta`を作成（`_`プレフィックステスト用、読み込まれないことを検証）
  - _Requirements: 5.7, 5.8_

- [ ] 7. 統合テストの実装
- [ ] 7.1 正常系テスト：初期化とラベル列挙
  - `tests/directory_loader_test.rs`ファイルを作成
  - `test_from_directory_success()`テストケースを実装
  - テストフィクスチャから`PastaEngine::from_directory()`で初期化
  - 初期化成功を確認
  - `list_global_labels()`で全グローバルラベルを列挙し、期待されるラベルが存在することを確認
  - _Requirements: 6.1, 6.2_

- [ ] 7.2 (P) 同名ラベルのランダム選択テスト
  - `test_multiple_labels_random_selection()`テストケースを実装
  - 同名ラベルを持つ複数ファイルからエンジンを初期化
  - `execute_label()`で同名ラベルを複数回実行
  - いずれかのラベル定義が選択されることを確認（実行結果の変化を検証）
  - _Requirements: 6.3_

- [ ] 7.3 (P) ローカルラベルスコープテスト
  - `test_local_label_scope_isolation()`テストケースを実装
  - 同名の親グローバルラベルを持つ複数ファイルを用意
  - 各親ラベル内にローカルラベルを定義
  - ローカルラベル実行時、親ラベルスコープが正しく隔離されていることを確認
  - 親ラベルAのローカルラベルから親ラベルBのローカルラベルへのアクセスが禁止されることを確認
  - _Requirements: 6.4, 4.5, 4.6, 4.7_

- [ ] 7.4 (P) ファイルフィルタリングテスト
  - `test_ignored_files_skipped()`テストケースを実装
  - `_ignored.pasta`を含むディレクトリから初期化
  - `_ignored.pasta`内のラベルが登録されていないことを確認（`list_labels()`で検証）
  - _Requirements: 6.5_

- [ ] 7.5 エラーケーステスト：ディレクトリ不在
  - `test_directory_not_found_error()`テストケースを実装
  - 存在しないパスを指定して`from_directory()`を呼び出し
  - `PastaError::DirectoryNotFound`が返されることを確認
  - _Requirements: 6.6, 7.1_

- [ ] 7.6 (P) エラーケーステスト：dic/ディレクトリ不在
  - `test_dic_directory_not_found_error()`テストケースを実装
  - `dic/`ディレクトリが存在しないパスを指定
  - `PastaError::DicDirectoryNotFound`が返されることを確認
  - _Requirements: 6.6, 7.2_

- [ ] 7.7 (P) エラーケーステスト：main.rune不在
  - `test_main_rune_not_found_error()`テストケースを実装
  - `main.rune`が存在しないパスを指定
  - `PastaError::MainRuneNotFound`が返されることを確認
  - _Requirements: 6.7, 7.3_

- [ ] 7.8 (P) Runeモジュールインポートテスト
  - `test_rune_module_import()`テストケースを実装
  - `main.rune`から`mod`文で他のRuneモジュールを参照するスクリプトを用意
  - エンジン初期化とラベル実行が成功することを確認
  - インポートされたモジュールの関数が正しく動作することを確認
  - _Requirements: 6.8, 3.10, 3.11_

- [ ] 8. パフォーマンス最適化
- [ ] 8.1 (P) パースキャッシュの活用確認
  - 既存の`PARSE_CACHE`グローバル変数がディレクトリローダーでも使用されることを確認
  - `parse_file()`呼び出しがキャッシュを自動的に利用することを検証
  - _Requirements: 8.1, 8.2_

- [ ] 8.2 (P) キャッシュヒット/ミスログの追加
  - デバッグビルド時にキャッシュヒット/ミス情報をログ出力する機能を追加
  - `tracing::debug!()`でキャッシュ操作をログ記録
  - _Requirements: 8.5_

- [ ] 9. ドキュメント整備
- [ ] 9.1 (P) APIドキュメントコメントの追加
  - `DirectoryLoader::load()`にdocコメントを追加（使用例、エラー条件を記載）
  - `PastaEngine::from_directory()`にdocコメントを追加（初期化手順、要件を記載）
  - `PastaEngine::from_directory_with_selector()`にdocコメントを追加
  - 各エラー型variantにdocコメントを追加
  - _Requirements: 9.1, 9.2, 9.3_

---

## Requirements Coverage Summary

| Requirement | Covered by Tasks |
|-------------|------------------|
| Req 1 (1.1-1.6) | 2.2, 2.5, 4.2 |
| Req 2 (2.1-2.13) | 2.1, 2.3, 2.4, 2.5 |
| Req 3 (3.1-3.12) | 3.1, 4.1, 4.2, 4.3, 7.8 |
| Req 4 (4.1-4.7) | 4.1, 4.4, 7.3 |
| Req 5 (5.1-5.8) | 6.1, 6.2, 6.3 |
| Req 6 (6.1-6.8) | 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8 |
| Req 7 (7.1-7.10) | 1.1, 7.5, 7.6, 7.7 |
| Req 8 (8.1-8.5) | 8.1, 8.2 |
| Req 9 (9.1-9.5) | 4.5, 4.6, 5.1, 9.1 |

**Total**: 9 major tasks, 29 sub-tasks
**Parallel-capable tasks**: 17 sub-tasks marked with (P)
**Average task size**: 1-2 hours per sub-task
