# 実装タスク

---

## Task 1: 旧実装ディレクトリの削除 ✅

- [x] 1. `src/parser/` ディレクトリを完全に削除する
  - `rm -rf src/parser/` で旧パーサー実装全体を削除
  - 削除対象: `src/parser/mod.rs`, `src/parser/ast.rs`, `src/parser/pasta.pest`
  - ファイルシステムから完全に削除されたことを確認
  - _Requirements: 1_

- [x] 2. `src/transpiler/` ディレクトリを完全に削除する
  - `rm -rf src/transpiler/` で旧トランスパイラー実装全体を削除
  - 削除対象: `src/transpiler/mod.rs` および関連ファイル
  - ファイルシステムから完全に削除されたことを確認
  - _Requirements: 1_

---

## Task 2: ソースコード層のビルド復旧 ✅

- [x] 3. `src/lib.rs` から旧モジュールのexport宣言を削除する
  - Line 47 の `pub mod parser;` を削除
  - Line 52 の `pub mod transpiler;` を削除
  - Lines 61-64 の `pub use parser::{...13個の型...};` を削除
  - Line 70 の `pub use transpiler::{TranspileContext, Transpiler};` を削除
  - 削除後、`cargo check` でエラーなく成功することを確認
  - _Requirements: 2_

- [x] 4. 他のソースファイルから旧モジュールへの `use` 文と参照コードを削除する
  - `src/cache.rs` (Line 96-233): `#[cfg(test)]` テストモジュール全体を削除
  - `src/runtime/words.rs` (Line 8): `use pasta::transpiler::` で始まる use 文を削除
  - 各ファイルで旧 parser/transpiler への参照を完全に除去
  - _Requirements: 2_

- [x] 5. ソースコード層のビルド確認
  - `cargo check` を実行
  - エラーなく成功することを確認
  - ビルド成功後、修正内容を Git にコミット: `git add -A && git commit -m "refactor(cleanup): 旧parser/transpiler export削除"`
  - _Requirements: 2_

---

## Task 3: テストコード層のビルド復旧 ✅

- [x] 6. 旧実装専用テストファイル 21 個を完全に削除する
  - **カテゴリA** (12ファイル): `tests/pasta_parser_*.rs` の全ファイルを削除
  - **カテゴリB** (7ファイル): `tests/pasta_transpiler_*.rs` の全ファイルを削除
  - **カテゴリC** (3ファイル): `tests/pasta_integration_e2e_simple_test.rs`, `tests/pasta_engine_rune_compile_test.rs`, `tests/pasta_engine_rune_vm_comprehensive_test.rs` を削除
  - 合計 21 ファイルをファイルシステムから完全に削除
  - _Requirements: 3_

- [x] 7. Registry 参照テストのインポートを修正する
  - `tests/pasta_stdlib_call_jump_separation_test.rs` を開く
  - `use pasta::transpiler::{SceneRegistry, WordDefRegistry};` を `use pasta::registry::{SceneRegistry, WordDefRegistry};` に変更
  - テスト内容は変更しない（設計原則検証の価値を保持）
  - _Requirements: 3_

- [x] 8. テストコード層のビルド確認
  - `cargo check --all` を実行
  - エラーなく成功することを確認
  - テストコード修正を Git にコミット: `git add -A && git commit -m "refactor(cleanup): 旧parser/transpiler依存テスト削除とRegistry参照修正"`
  - _Requirements: 3_

---

## Task 4: テスト実行の復旧 ✅

- [x] 9. 残存テストの実行確認
  - `cargo test --all` を実行
  - 全テストが成功することを確認
  - テスト結果を記録（テスト数、実行時間等）
  - _Requirements: 4_

- [x] 10. テスト実行結果のコミット
  - テスト成功を確認した後、Git にコミット: `git add -A && git commit -m "test(cleanup): テスト実行確認完了"`
  - README.md レガシースタック記述削除が必要な場合はここで実施
  - _Requirements: 4_

---

## Task 5: モジュール名正規化準備（parser2 → parser_new） ✅

- [x] 11. `src/parser2/` を `src/parser_new/` に一時的にリネームする
  - `mv src/parser2/ src/parser_new/` を実行
  - ファイルシステムで確認
  - _Requirements: 5_

- [x] 12. `src/transpiler2/` を `src/transpiler_new/` に一時的にリネームする
  - `mv src/transpiler2/ src/transpiler_new/` を実行
  - ファイルシステムで確認
  - _Requirements: 5_

- [x] 13. 中間リネーム後のビルド確認
  - `cargo check` を実行
  - エラーなく成功することを確認（まだ use 文の修正は不要）
  - _Requirements: 5_

---

## Task 6: モジュール名正規化完了（parser_new → parser） ✅

- [x] 14. 全ソースコードとテストコードの `use` 文を修正する
  - `use pasta::parser2::` を `use pasta::parser_new::` に修正 (grep で検索)
  - `use pasta::transpiler2::` を `use pasta::transpiler_new::` に修正 (grep で検索)
  - `src/lib.rs` のモジュール宣言 `pub mod parser2;` を `pub mod parser_new;` に修正
  - `src/lib.rs` のモジュール宣言 `pub mod transpiler2;` を `pub mod transpiler_new;` に修正
  - すべてのファイルが修正されたことを確認
  - _Requirements: 5_

- [x] 15. 中間リネーム後のビルド確認
  - `cargo check` を実行
  - エラーなく成功することを確認
  - _Requirements: 5_

- [x] 16. `src/parser_new/` を `src/parser/` に最終リネームする
  - `mv src/parser_new/ src/parser/` を実行
  - ファイルシステムで確認
  - _Requirements: 5_

- [x] 17. `src/transpiler_new/` を `src/transpiler/` に最終リネームする
  - `mv src/transpiler_new/ src/transpiler/` を実行
  - ファイルシステムで確認
  - _Requirements: 5_

- [x] 18. 全ソースコードとテストコードの `use` 文を最終更新する
  - `use pasta::parser_new::` を `use pasta::parser::` に修正 (grep で検索)
  - `use pasta::transpiler_new::` を `use pasta::transpiler::` に修正 (grep で検索)
  - `src/lib.rs` のモジュール宣言 `pub mod parser_new;` を `pub mod parser;` に修正
  - `src/lib.rs` のモジュール宣言 `pub mod transpiler_new;` を `pub mod transpiler;` に修正
  - 全ファイルが修正されたことを確認
  - _Requirements: 5_

---

## Task 7: モジュール名正規化後のビルド・テスト復旧 ✅

- [x] 19. モジュール正規化後のビルド確認
  - `cargo check` を実行
  - エラーなく成功することを確認
  - _Requirements: 6_

- [x] 20. モジュール正規化後のテスト実行確認
  - `cargo test --all` を実行
  - 全テストが成功することを確認
  - テスト結果を記録
  - _Requirements: 6_

- [x] 21. モジュール正規化のコミット
  - ビルド・テスト成功を確認した後、Git にコミット: `git add -A && git commit -m "refactor(cleanup): parser2→parser, transpiler2→transpiler へのモジュール正規化完了"`
  - _Requirements: 6_

---

## Task 8: 未使用テストフィクスチャの削除 ✅

- [x] 22. `tests/` ディレクトリ配下の `*.rs` 以外のファイルをリストアップする
  - `find tests/ -type f ! -name "*.rs"` を実行
  - 結果をメモ（テストフィクスチャのファイル一覧）
  - _Requirements: 7_

- [x] 23. 残存テストコードから参照されていないファイルを特定する
  - grep で各ファイルが残存テストから参照されているか確認
  - 参照されていないファイルをリストアップ
  - _Requirements: 7_

- [x] 24. 参照されていないファイルのみを削除する
  - 参照なしファイルを削除（`rm` コマンド）
  - 削除が安全だったか確認
  - _Requirements: 7_

- [x] 25. テストフィクスチャ削除後のテスト実行確認
  - `cargo test --all` を実行
  - 全テストが成功することを確認
  - _Requirements: 7_

---

## Task 9: 最終検証とコミット ✅

- [x] 26. 最終ビルド確認
  - `cargo check --all` を実行
  - エラーなく成功することを確認
  - _Requirements: 8_

- [x] 27. 最終テスト実行確認
  - `cargo test --all` を実行
  - 全テストが成功することを確認
  - テスト結果をレポート（テスト数、実行時間、パス・失敗状況等）
  - _Requirements: 8_

- [x] 28. 最終状態の確認
  - `src/parser/`, `src/transpiler/` が旧実装（削除済み）ではなく、 parser2/transpiler2 からのリネーム版であることを確認
  - Registry モジュール (`src/registry/`) が無変更であることを確認
  - すべてのファイルが正規化されたことを確認
  - _Requirements: 8_

- [x] 29. 最終コミット
  - 全変更を Git にコミット: `git add -A && git commit -m "docs(cleanup): legacy-parser-transpiler-cleanup 完了 - 旧実装削除、モジュール正規化、テスト修正完了"`
  - _Requirements: 8_

