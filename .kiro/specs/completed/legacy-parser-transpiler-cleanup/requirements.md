# Requirements Document

## 承認情報

**承認日時**: 2025-12-24  
**承認状態**: ✅ **承認済み**

---

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

## Requirements

### Requirement 1: 旧実装ディレクトリの削除
**目的:** 開発者として、パーサー２・トランスパイラー２への移行が完了したため、旧実装コードを削除し、コードベースを整理したい

#### Acceptance Criteria
1. When 旧実装削除を実施する場合、Pastaプロジェクトは `src/parser/` ディレクトリを完全に削除する
2. When 旧実装削除を実施する場合、Pastaプロジェクトは `src/transpiler/` ディレクトリを完全に削除する
3. When ディレクトリ削除後、Pastaプロジェクトは削除された旧モジュールへの全参照を特定する

---

### Requirement 2: ソースコード層のビルド復旧
**目的:** 開発者として、旧実装削除後にソースコードのビルドエラーを修正し、ライブラリとして正常にコンパイル可能な状態を維持したい

#### Acceptance Criteria
1. When 旧parser/transpiler参照を削除する場合、Pastaプロジェクトは `src/lib.rs` から旧モジュールのexport宣言を削除する
2. When 旧parser/transpiler参照を削除する場合、Pastaプロジェクトは他のソースファイル（`src/engine.rs`, `src/runtime/`, etc.）から旧モジュールへの `use` 文と参照コードを削除する
3. When ソースコード修正を完了する場合、Pastaプロジェクトは `cargo check` コマンドでエラーなくコンパイルを成功させる

---

### Requirement 3: テストコード層のビルド復旧
**目的:** 開発者として、旧実装削除後にテストコードを整理し、残存テストがコンパイル可能な状態を維持したい

#### Acceptance Criteria
1. When 旧parser/transpiler依存テストを削除する場合、Pastaプロジェクトは `tests/` 配下の旧実装専用テストファイル（21個）を完全に削除する
2. When Registry参照テストを修正する場合、Pastaプロジェクトは `tests/pasta_stdlib_call_jump_separation_test.rs` の `use pasta::transpiler::` を `use pasta::registry::` に変更する
3. When テストコード修正を完了する場合、Pastaプロジェクトは `cargo check --all` コマンドでエラーなくコンパイルを成功させる
4. When ビルド修正を完了する場合、Pastaプロジェクトは修正内容を Git にコミットする

**削除対象テストファイル（21個）**:
- カテゴリA: `tests/pasta_parser_*.rs` (12ファイル)
- カテゴリB: `tests/pasta_transpiler_*.rs` (7ファイル)  
- カテゴリC: `tests/pasta_integration_e2e_simple_test.rs`, `tests/pasta_engine_rune_compile_test.rs`, `tests/pasta_engine_rune_vm_comprehensive_test.rs`

**修正して残すテストファイル（1個）**:
- `tests/pasta_stdlib_call_jump_separation_test.rs` - Registry参照に変更

**注記**: parser2/transpiler2 用のテストは別仕様で追加予定

---

### Requirement 4: テスト実行の復旧
**目的:** 開発者として、テストファイル削除後も残存テストが正常に実行されることを確認したい

#### Acceptance Criteria
1. When テストファイル削除後、Pastaプロジェクトは残存するparser2/transpiler2ベースのテストが正常に実行されることを確認する
2. When テスト修正を完了する場合、Pastaプロジェクトは `cargo test --all` コマンドで全テストを成功させる
3. When テスト修正を完了する場合、Pastaプロジェクトは修正内容を Git にコミットする

---

### Requirement 5: モジュール名の正規化（parser2 → parser）
**目的:** 開発者として、旧実装削除後に parser2/transpiler2 の「２」を外し、正式名称としてモジュールを統一したい

#### Acceptance Criteria
1. When モジュール名を正規化する場合、Pastaプロジェクトは `src/parser2/` ディレクトリを `src/parser_new/` に一時的にリネームする
2. When モジュール名を正規化する場合、Pastaプロジェクトは `src/transpiler2/` ディレクトリを `src/transpiler_new/` に一時的にリネームする
3. When リネーム後、Pastaプロジェクトは `src/parser_new/` を `src/parser/` に最終リネームする
4. When リネーム後、Pastaプロジェクトは `src/transpiler_new/` を `src/transpiler/` に最終リネームする
5. When ディレクトリリネーム後、Pastaプロジェクトは全ソースコードとテストコードの `use` 文とモジュール宣言を新しい名前に修正する

---

### Requirement 6: モジュール名正規化後のビルド復旧
**目的:** 開発者として、モジュール名変更後にビルドとテストが成功することを確認し、安定した状態を維持したい

#### Acceptance Criteria
1. When モジュール名変更後、Pastaプロジェクトは `cargo check` コマンドでエラーなくコンパイルを成功させる
2. When モジュール名変更後、Pastaプロジェクトは `cargo test --all` コマンドで全テストを成功させる
3. When ビルド・テスト成功後、Pastaプロジェクトは修正内容を Git にコミットする

---

### Requirement 7: 未使用テストフィクスチャの削除
**目的:** 開発者として、旧実装専用のテストフィクスチャや補助ファイルを削除し、テストディレクトリを整理したい

#### Acceptance Criteria
1. When 未使用ファイルを検索する場合、Pastaプロジェクトは `tests/` ディレクトリ配下の `*.rs` 以外のファイルをリストアップする
2. When 未使用ファイルを判定する場合、Pastaプロジェクトは残存テストコードから参照されていないファイルを特定する
3. When 未使用ファイルを削除する場合、Pastaプロジェクトは参照されていないファイルのみを削除する
4. When ファイル削除後、Pastaプロジェクトは `cargo test --all` コマンドで全テストを成功させる

---

### Requirement 8: 最終検証とコミット
**目的:** 開発者として、全作業完了後にビルドとテストが正常に動作することを最終確認し、クリーンな状態でコミットしたい

#### Acceptance Criteria
1. When 最終検証を実施する場合、Pastaプロジェクトは `cargo check --all` コマンドでエラーなくコンパイルを成功させる
2. When 最終検証を実施する場合、Pastaプロジェクトは `cargo test --all` コマンドで全テストを成功させる
3. When 最終検証成功後、Pastaプロジェクトは全変更を Git にコミットする
4. The Pastaプロジェクトは、旧実装関連のコードとファイルが完全に削除され、parser/transpiler が正規化された状態を維持する
