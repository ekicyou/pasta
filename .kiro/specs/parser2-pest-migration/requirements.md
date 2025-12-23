# Requirements Document

## Project Description (Input)
pasta2.pestに基づいた実装を行う。pasta2.pestを憲法とし、新たなパーサー層を構築せよ。現在の実装（mod parser）はまだ削除せず、置き換えの準備として（mod parser2）を作る方向で進めよ。pasta2.pestは「parser2」ディレクトリに移動・リネームしてよいが、内容の書き換えは行わないこと。

## Introduction
本仕様は、既存の`src/parser/pasta.pest`ではなく`pasta2.pest`を権威的文法定義として採用し、新たなパーサー層を構築するための要件を定義します。レガシーコード（`mod parser`）を保持しつつ、並行して新実装（`mod parser2`）を作成し、段階的な移行を可能にします。

## Requirements

### Requirement 1: pasta2.pest文法の保全
**Objective:** 開発者として、pasta2.pestファイルを移動してもその内容を保全したい。これにより、文法定義の権威性を維持できる。

#### Acceptance Criteria
1. When pasta2.pestを`src/parser2/grammar.pest`に移動する場合、the Parser2 migration process shall preserve the exact file contents without modification
2. When parser2ディレクトリ構造が作成される場合、the Parser2 module shall treat grammar.pest as the single source of truth for syntax rules
3. The Parser2 implementation shall reject any manual edits to grammar.pest that deviate from the original pasta2.pest specification

### Requirement 2: 新しいパーサーモジュール（parser2）の作成
**Objective:** 開発者として、既存parserとは独立した新しいparser2モジュールを作成したい。これにより、段階的移行とリグレッションリスク軽減を実現できる。

#### Acceptance Criteria
1. The Pasta project shall create a new module `src/parser2/` with independent namespace
2. The Parser2 module shall expose public API functions with the same naming as legacy parser: `parse_file`, `parse_str` (namespaced via module path `pasta::parser2::parse_str`)
3. When lib.rsがpublic APIを公開する場合、the Pasta crate shall export `parser2` module as public (`pub mod parser2;`) for usage via `pasta::parser2::*`
4. The Parser2 module shall not share AST type definitions with the legacy parser module to ensure complete independence

### Requirement 3: pasta2.pest文法に基づくAST型定義
**Objective:** 開発者として、pasta2.pest文法規則を正確に反映したAST型を定義したい。これにより、文法と実装の一貫性を保証できる。

#### Acceptance Criteria
1. When grammar.pestでRuleが定義されている場合、the Parser2 AST module shall define corresponding Rust structs for all terminal and non-terminal rules
2. The Parser2 AST types shall support Unicode identifiers (XID_START, XID_CONTINUE) as defined in grammar.pest
3. The Parser2 AST types shall distinguish between global_marker (`＊` or `*`) and local_marker (`・` or `-`) scene definitions
4. The Parser2 AST types shall represent full-width and half-width marker pairs (e.g., `＠`/`@`, `＄`/`$`, `＞`/`>`) as equivalent token types

### Requirement 4: Pest parser生成の統合
**Objective:** 開発者として、grammar.pestからPest parserを生成し、Rustコードに統合したい。これにより、型安全なパース処理を実現できる。

#### Acceptance Criteria
1. The Parser2 module shall use `#[grammar = "parser2/grammar.pest"]` directive for pest_derive (relative to src/ directory)
2. The Parser2 module shall generate a `PastaParser2` struct using `#[derive(Parser)]` macro
3. When parse errorsが発生する場合、the Parser2 shall return `PastaError::PestError` with file location and error context
4. The Parser2 shall successfully parse valid Pasta scripts using `PastaParser2::parse(Rule::file, source)`

### Requirement 5: レガシーparserとの共存
**Objective:** 開発者として、既存のmod parserを削除せずに稼働させたい。これにより、新旧パーサーの比較テストとリスク管理を可能にする。

#### Acceptance Criteria
1. The Pasta project shall maintain both `src/parser/` and `src/parser2/` modules simultaneously
2. When lib.rsがインポートを宣言する場合、the Pasta crate shall provide distinct import paths: `pasta::parser` and `pasta::parser2`
3. The existing test suite shall continue to use `pasta::parser` without modification
4. The Parser2 module shall not cause compilation errors or runtime conflicts with the legacy parser module

### Requirement 6: parser2モジュールの基本構成
**Objective:** 開発者として、parser2モジュールを標準的なRustモジュール構成で実装したい。これにより、保守性と拡張性を確保できる。

#### Acceptance Criteria
1. The Parser2 module shall define a `mod.rs` file as the module entry point
2. The Parser2 module shall define an `ast.rs` file for AST type definitions
3. The Parser2 module shall define a `grammar.pest` file as the Pest grammar specification
4. When `mod.rs`がpublic APIを公開する場合、the Parser2 module shall re-export AST types using `pub use ast::*`

### Requirement 7: エラーハンドリング統合
**Objective:** 開発者として、parser2のエラーを既存のPastaError型で扱いたい。これにより、統一的なエラー処理を維持できる。

#### Acceptance Criteria
1. The Parser2 module shall return `Result<T, PastaError>` for all parsing operations
2. When Pest parse errorsが発生する場合、the Parser2 shall wrap them in `PastaError::PestError` variant
3. When IO errorsが発生する場合、the Parser2 shall wrap them in `PastaError::IoError` variant using `From` trait
4. The Parser2 error messages shall include filename and source location context

### Requirement 8: 初期テストカバレッジ
**Objective:** 開発者として、parser2の基本動作を検証するテストを用意したい。これにより、実装の正確性を初期段階で確保できる。

#### Acceptance Criteria
1. The Pasta project shall create a new test file `tests/pasta_parser2_basic_test.rs`
2. When parser2がテストされる場合、the test suite shall verify successful parsing of minimal valid Pasta script
3. When invalid syntaxがテストされる場合、the test suite shall verify that parser2 returns `PastaError::PestError`
4. The Parser2 tests shall use fixtures from `tests/fixtures/` directory for integration testing

### Requirement 9: ドキュメント整備
**Objective:** 開発者として、parser2モジュールの目的と使用方法を文書化したい。これにより、将来の開発者が意図を理解できる。

#### Acceptance Criteria
1. The Parser2 `mod.rs` shall include a module-level doc comment (`//!`) explaining the migration purpose
2. The Parser2 doc comment shall reference grammar.pest as the authoritative specification
3. The Parser2 public API functions shall include doc comments with usage examples
4. When README.mdが更新される場合、the Pasta project shall document the parallel parser architecture

### Requirement 10: ファイル移動の追跡可能性
**Objective:** 開発者として、pasta2.pestの移動を履歴から追跡したい。これにより、文法定義の変更履歴を保全できる。

#### Acceptance Criteria
1. When pasta2.pestを移動する場合、the migration process shall use `git mv` command to preserve file history
2. The git commit message shall follow conventional commits format: `refactor(parser2): Move pasta2.pest to parser2/grammar.pest`
3. The commit message shall explicitly state "no content changes" to clarify the operation
4. When grammar.pestが作成される場合、the file shall retain all original line-by-line content from pasta2.pest
