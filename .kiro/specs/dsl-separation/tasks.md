# Implementation Plan

## タスク概要

本実装では、pasta_core から DSL パーサーを完全分離し、新規クレート pasta_dsl として抽出する。5つのフェーズに分けて段階的に実行し、各フェーズでテスト検証を行うことでリスクを最小化する。

**総タスク数**: 5メジャータスク、19サブタスク
**要件カバレッジ**: 全6要件（1.1-1.4, 2.1-2.5, 3.1-3.4, 4.1-4.6, 5.1-5.3, 6.1-6.8）
**並列実行**: Phase 5（ドキュメント更新）の8サブタスクが並列実行可能

---

## Phase 1: pasta_dsl クレート作成

- [x] 1. pasta_dsl クレート基盤を構築する
- [x] 1.1 pasta_dsl ディレクトリ構造とCargo.tomlを作成する
  - `crates/pasta_dsl/` ディレクトリを新規作成
  - `Cargo.toml` を作成し、pest 2.8, pest_derive 2.8, thiserror 2 を依存に追加
  - package メタデータ（name, description, publish等）を workspace 設定から継承
  - _Requirements: 1.1, 3.1, 4.1_

- [x] 1.2 pasta_dsl のモジュール構造を定義する
  - `src/lib.rs` を作成し、`pub mod parser;` と `pub mod error;` を定義
  - parser と error の再エクスポート（`pub use parser::*; pub use error::*;`）を設定
  - クレートレベルのドキュメントコメントを追加（独立DSLパーサーとしての説明）
  - _Requirements: 1.1, 1.2_

- [x] 1.3 parser モジュールを pasta_core から移動する
  - `src/parser/` ディレクトリを作成
  - `crates/pasta_core/src/parser/mod.rs` を `crates/pasta_dsl/src/parser/mod.rs` に移動
  - `crates/pasta_core/src/parser/ast.rs` を `crates/pasta_dsl/src/parser/ast.rs` に移動
  - `crates/pasta_core/src/parser/grammar.pest` を `crates/pasta_dsl/src/parser/grammar.pest` に移動
  - parser 内部の `use crate::error::ParseError` 参照を確認（同一クレート内で完結）
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 1.4 ParseError 型を pasta_core から移動する
  - `src/error.rs` を新規作成
  - `crates/pasta_core/src/error.rs` から ParseError, ParseErrorInfo, ParseResult を抽出
  - thiserror derive マクロが正しく動作することを確認
  - SceneTableError, WordTableError は pasta_core に残すことを確認
  - _Requirements: 1.4, 5.1_

- [x] 1.5 テストファイルを pasta_core から移動する
  - `tests/` ディレクトリを作成
  - 4テストファイル（actor_code_block_test.rs, digit_id_var_test.rs, sakura_symbol_tag_test.rs, span_byte_offset_test.rs）を移動
  - 各テストファイルの import パスを `use pasta_core::parser::{...};` から `use pasta_dsl::parser::{...};` に変更
  - テストロジック自体は変更しない（完全に同一）
  - _Requirements: 4.4, 4.5, 4.6_

- [x] 1.6 pasta_dsl の README.md を作成する
  - `README.md` を新規作成
  - 独立DSLパーサーとしての説明を記載（Purpose, Features, Usage）
  - 基本的な使用例（parse_str, parse_file）をコード例で示す
  - 依存関係（pest, pest_derive, thiserror のみ）を明記
  - 外部開発者が pasta_dsl のみを依存に追加できることを強調
  - _Requirements: 4.3, 4.4_

- [x] 1.7 pasta_dsl のビルドとテストを検証する
  - `cargo build -p pasta_dsl` でビルド成功を確認
  - `cargo test -p pasta_dsl` で26テストが成功することを確認
  - ビルド警告がないことを確認
  - _Requirements: 1.2, 4.4_

---

## Phase 2: pasta_core の整理

- [x] 2. pasta_core から parser 関連を完全除去する
- [x] 2.1 pasta_core の lib.rs から parser を除去する
  - `pub mod parser;` 行を削除
  - `pub use parser::*;` を削除
  - ParseError, ParseErrorInfo, ParseResult の再エクスポートを削除
  - registry 関連の再エクスポートのみを維持
  - _Requirements: 2.1, 2.2_

- [x] 2.2 pasta_core の error.rs から ParseError を除去する
  - ParseError, ParseErrorInfo, ParseResult の定義を削除
  - SceneTableError, SceneTableResult, WordTableError, WordTableResult のみを残す
  - thiserror derive マクロの使用を確認
  - _Requirements: 2.2, 5.2_

- [x] 2.3 pasta_core から parser ディレクトリとテストファイルを削除する
  - `src/parser/` ディレクトリ全体を削除（mod.rs, ast.rs, grammar.pest）
  - `tests/` から4テストファイルを削除（actor_code_block_test.rs, digit_id_var_test.rs, sakura_symbol_tag_test.rs, span_byte_offset_test.rs）
  - 削除後にディレクトリ構造を確認
  - _Requirements: 2.1, 4.6_

- [x] 2.4 pasta_core の Cargo.toml から不要な依存を削除する
  - pest 2.8 を dependencies から削除
  - pest_derive 2.8 を dependencies から削除
  - fast_radix_trie, rand, thiserror, tracing のみを維持
  - _Requirements: 2.4_

- [x] 2.5 pasta_core のビルドとテストを検証する
  - `cargo build -p pasta_core` でビルド成功を確認
  - `cargo test -p pasta_core` で104テストが成功することを確認（26テスト移動後）
  - ビルド警告がないことを確認
  - _Requirements: 2.2, 2.5_

---

## Phase 3: pasta_lua の移行

- [x] 3. pasta_lua を pasta_dsl に直接依存させる
- [x] 3.1 pasta_lua の Cargo.toml に pasta_dsl 依存を追加する
  - `[dependencies]` セクションに `pasta_dsl.workspace = true` を追加
  - pasta_core への依存を維持（registry 層のため）
  - 依存順序を確認（pasta_dsl, pasta_core, mlua 等）
  - _Requirements: 2.3_

- [x] 3.2 pasta_lua の import パスを pasta_dsl に変更する
  - `use pasta_core::parser::{...};` を `use pasta_dsl::parser::{...};` に変更
  - `pasta_core::ParseError` を `pasta_dsl::ParseError` に変更
  - 対象ファイル: code_generator.rs, context.rs, transpiler.rs, runtime/mod.rs 等
  - grep で `pasta_core::parser` への参照が残っていないことを確認
  - _Requirements: 2.3, 5.3_

- [x] 3.3 pasta_lua のビルドとテストを検証する
  - `cargo build -p pasta_lua` でビルド成功を確認
  - `cargo test -p pasta_lua` で既存の全テストが成功することを確認
  - import パス変更漏れがないことを確認
  - _Requirements: 2.3, 2.5_

---

## Phase 4: ワークスペース統合とテスト検証

- [x] 4. ワークスペース全体で pasta_dsl を統合する
- [x] 4.1 Cargo.toml のワークスペース依存に pasta_dsl を追加する
  - `[workspace.dependencies]` セクションに `pasta_dsl = { path = "crates/pasta_dsl", version = "0.1.3" }` を追加
  - members に `"crates/*"` が含まれていることを確認（pasta_dsl が自動的に含まれる）
  - _Requirements: 3.1, 3.2_

- [x] 4.2 ワークスペース全体のビルドを検証する
  - `cargo build --workspace` で全クレートがビルド成功することを確認
  - 依存解決エラーがないことを確認
  - バージョン指定が他のクレートと一致していることを確認（0.1.3）
  - _Requirements: 3.3, 3.4_

- [x] 4.3 ワークスペース全体のテストを検証する
  - `cargo test --all` で全テストが成功することを確認
  - pasta_dsl: 26テスト、pasta_core: 104テスト、pasta_lua: 既存テスト全て
  - リグレッションがないことを確認
  - _Requirements: 2.5, 3.4, 4.4_

---

## Phase 5: ドキュメント更新

- [x] 5. プロジェクト全体のドキュメントを pasta_dsl 追加に対応させる
- [x] 5.1 (P) README.md のアーキテクチャ図を更新する
  - レイヤー構成図に pasta_dsl を追加（Language-independent Layer）
  - ディレクトリツリーに `crates/pasta_dsl/` を追加
  - pasta_core の説明を「レジストリ層とユーティリティ」に更新
  - _Requirements: 6.1_

- [x] 5.2 (P) SOUL.md のドキュメントヒエラルキーを更新する
  - Level 2（Implementation Layer）のクレートREADMEリストに `crates/pasta_dsl/README.md` を追加
  - アルファベット順に挿入（pasta_core の後）
  - _Requirements: 6.2_

- [x] 5.3 (P) .kiro/steering/tech.md のワークスペース構成を更新する
  - ワークスペースレイヤー構成図に pasta_dsl を追加
  - クレート責務テーブルに pasta_dsl 行を追加（Layer: Parser, 責務: DSL→AST変換）
  - pasta_core の責務を「Registry: シーン/単語テーブル管理」に更新
  - _Requirements: 6.3_

- [x] 5.4 (P) .kiro/steering/structure.md のディレクトリ構造を更新する
  - ディレクトリ構造ツリーに `crates/pasta_dsl/` を追加
  - ワークスペース構成図に pasta_dsl を追加
  - レイヤー分離原則に pasta_dsl の位置付けを反映
  - _Requirements: 6.4_

- [x] 5.5 (P) crates/pasta_core/README.md のアーキテクチャ図を更新する
  - アーキテクチャ図から parser モジュールを削除
  - registry モジュールのみを記載
  - pasta_dsl への依存がないことを明示
  - _Requirements: 6.5_

- [x] 5.6 (P) crates/pasta_lua/README.md の依存関係を更新する
  - 依存関係セクションに pasta_dsl への直接依存を追加
  - pasta_core への依存理由を「registry 層のみ」に明記
  - アーキテクチャ図に pasta_dsl と pasta_core の両方を記載
  - _Requirements: 6.6_

- [x] 5.7 (P) crates/pasta_shiori/README.md の依存関係を更新する
  - 依存関係テーブルに pasta_dsl の位置付けを反映
  - pasta_shiori は pasta_core::parser を使用していないことを確認
  - 間接依存の説明を更新（pasta_lua 経由で pasta_dsl に依存）
  - _Requirements: 6.7_

- [x] 5.8 (P) TEST_COVERAGE.md のテストカバレッジを更新する
  - テストカバレッジサマリーのクレート一覧に pasta_dsl を追加（26テスト）
  - pasta_core のテスト数を更新（130→104テスト）
  - 合計テスト数を再計算
  - _Requirements: 6.8_

---

## 完了基準

全タスク完了後、以下を確認：

- [x] `cargo test --all` が成功する
- [x] 全6要件が実装されている
- [x] 8ドキュメントファイルが更新されている
- [x] pasta_dsl が独立して利用可能である（`cargo build -p pasta_dsl` 成功）
- [x] pasta_core から parser が完全に除去されている
- [x] pasta_lua が pasta_dsl に直接依存している

**Definition of Done (DoD)**:
1. **Test Gate**: `cargo test --all` 成功
2. **Doc Gate**: 仕様差分を8ドキュメントに反映済み
3. **Steering Gate**: structure.md, tech.md が最新のアーキテクチャと一致
4. **Soul Gate**: SOUL.md のドキュメントヒエラルキーが更新済み
