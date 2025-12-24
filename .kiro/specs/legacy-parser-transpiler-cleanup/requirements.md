# Requirements Document

## Project Description (Input)
ランタイム層からの参照がパーサー２、トランスパイラー２を呼び出すようになった。旧実装のパーサー、トランスパイラーについて、以下の手順で削除せよ。

１．パーサー、トランスパイラーのディレクトリを丸ごと
２．`cargo check`が通るまでソースコードを修正。
　　多分参照コードの削除だけでよいはずだが、注意して修正。
３．`cargo check --all`が通るまでソースコードを修正。
　　多分テストコードのコードの削除だけでよいはず。
４．全コミット。
５．`cargo test --all`が通るまでテスト修正。
６．全コミット。
７．パーサー２、トランスパイラー２のモジュール名から「２」を外す。
８．ビルドが通るまで修正
９．テストが通るまで修正
１０．全コミット。
１１．テストディレクトリに残っている、*.rs以外のファイルについて、残存テストから
　　　参照されていないものを削除。
１２．全テストが通ることを確認。
１３．全コミット

---

## 概要

本仕様は、pastaプロジェクトにおける旧parser/transpiler実装の完全削除と、parser2/transpiler2モジュールの正規化（名称から「2」を除去）を管理する。ランタイム層が新実装（parser2/transpiler2）への移行を完了したため、レガシーコードを安全に削除し、コードベースを整理する。

---

## Requirements

### Requirement 1: レガシーディレクトリの削除
**Objective:** As a プロジェクトメンテナ, I want 旧parser/transpilerディレクトリを完全削除する, so that コードベースが新実装のみを含む状態になる

#### Acceptance Criteria
1. When レガシーディレクトリ削除を実行する, the Pasta Build System shall `src/parser/` ディレクトリを完全に削除する
2. When レガシーディレクトリ削除を実行する, the Pasta Build System shall `src/transpiler/` ディレクトリを完全に削除する
3. The Pasta Build System shall `src/parser2/` と `src/transpiler2/` ディレクトリを保持する

### Requirement 2: ソースコードコンパイル成功
**Objective:** As a 開発者, I want ソースコード内の旧実装参照を削除する, so that `cargo check` が成功する

#### Acceptance Criteria
1. When レガシーディレクトリ削除後にビルドを実行する, the Pasta Build System shall すべての `src/` 配下ファイルでコンパイルエラーが0件である
2. If `src/lib.rs` や他のモジュールが旧parser/transpilerを参照している, then the Developer shall 該当参照コードを削除する
3. When `cargo check` を実行する, the Cargo Build Tool shall 終了コード0を返す

### Requirement 3: 全ターゲットコンパイル成功
**Objective:** As a 開発者, I want テストコード内の旧実装参照を削除する, so that `cargo check --all` が成功する

#### Acceptance Criteria
1. When 全ターゲットビルドを実行する, the Pasta Build System shall `tests/` 配下のすべてのテストファイルでコンパイルエラーが0件である
2. If テストファイルが旧parser/transpilerをインポートしている, then the Developer shall 該当インポート文を削除する
3. When `cargo check --all` を実行する, the Cargo Build Tool shall 終了コード0を返す
4. When コンパイル成功を確認する, the Developer shall 変更内容をGitコミットする

### Requirement 4: 全テスト成功（旧実装削除後）
**Objective:** As a QA担当者, I want 旧実装削除後も全テストが成功する, so that 機能退行がないことを確認できる

#### Acceptance Criteria
1. When `cargo test --all` を実行する, the Pasta Test Framework shall すべてのテストが成功する（失敗0件）
2. If テストが旧実装固有の機能をテストしている, then the Developer shall 該当テストを削除または新実装向けに修正する
3. When テスト修正が完了する, the Developer shall 変更内容をGitコミットする

### Requirement 5: モジュール名正規化（「2」除去）
**Objective:** As a 開発者, I want parser2/transpiler2の名称から「2」を除去する, so that モジュール名が正規化される

#### Acceptance Criteria
1. When モジュール名変更を実行する, the Developer shall `src/parser2/` を `src/parser/` にリネームする
2. When モジュール名変更を実行する, the Developer shall `src/transpiler2/` を `src/transpiler/` にリネームする
3. When モジュール名変更を実行する, the Developer shall すべての参照コード（`use`, `mod`, `extern`文）を新名称に更新する
4. The Pasta Build System shall モジュールパス変更後に `src/lib.rs` のエクスポート宣言が正しく機能する

### Requirement 6: ビルド成功（正規化後）
**Objective:** As a 開発者, I want モジュール名変更後もビルドが成功する, so that リネームが正しく反映されたことを確認できる

#### Acceptance Criteria
1. When `cargo check` を実行する, the Cargo Build Tool shall 終了コード0を返す
2. When `cargo check --all` を実行する, the Cargo Build Tool shall 終了コード0を返す
3. If ビルドエラーが発生する, then the Developer shall モジュールパス参照を修正する

### Requirement 7: テスト成功（正規化後）
**Objective:** As a QA担当者, I want モジュール名変更後も全テストが成功する, so that リネームによる機能退行がないことを確認できる

#### Acceptance Criteria
1. When `cargo test --all` を実行する, the Pasta Test Framework shall すべてのテストが成功する（失敗0件）
2. If テストがモジュールパスを直接参照している, then the Developer shall テストコードを新名称に更新する
3. When テスト成功を確認する, the Developer shall 変更内容をGitコミットする

### Requirement 8: 未使用テストファイルの削除
**Objective:** As a プロジェクトメンテナ, I want testsディレクトリ内の未使用非Rustファイルを削除する, so that テストfixturesが整理される

#### Acceptance Criteria
1. When 未使用ファイル検出を実行する, the Developer shall `tests/` 配下の `*.rs` 以外のファイルを列挙する
2. When 各非Rustファイルを検証する, the Developer shall 現存テストから参照されていないファイルを特定する
3. If ファイルが未参照である, then the Developer shall 該当ファイルを削除する
4. The Pasta Build System shall 削除後も `cargo test --all` が成功する

### Requirement 9: 最終検証とコミット
**Objective:** As a プロジェクトメンテナ, I want 全変更後に完全な検証を実施する, so that クリーンアップが完全であることを確認できる

#### Acceptance Criteria
1. When 最終検証を実行する, the Cargo Build Tool shall `cargo check --all` が成功する
2. When 最終検証を実行する, the Pasta Test Framework shall `cargo test --all` が成功する（失敗0件）
3. When 全検証が成功する, the Developer shall すべての変更をGitコミットする
4. The Git Repository shall レガシーコード削除、モジュール正規化、テストクリーンアップの各段階でコミット履歴を持つ
