# Requirements Document

## Project Description (Input)
現在のクレート構成を、pastaの単独クレートから、pasta_parser/pasta_runeの２クレート構成にしてほしい。ワークグループを導入し、/crates/*配下をクレート置き場にしてください。

## Introduction
本仕様では、現在の単独クレート構成（`pasta`）を、Cargoワークスペースを活用した複数クレート構成に移行します。具体的には、パーサー層（`pasta_parser`）とランタイム層（`pasta_rune`）を分離し、`/crates/`ディレクトリ配下に配置することで、モジュール間の責任境界を明確化し、保守性・テスト容易性を向上させます。

## Requirements

### Requirement 1: Cargoワークスペースの導入
**Objective:** As a 開発者, I want Cargoワークスペース構成を導入する, so that 複数クレートを統一的に管理し、依存関係を明確化できる

#### Acceptance Criteria
1. When ルートディレクトリにCargo.tomlが配置される, the Cargoワークスペース shall `[workspace]`セクションで`members = ["crates/*"]`を定義する
2. The Cargoワークスペース shall 各メンバークレートのビルド・テストを`cargo build --workspace`および`cargo test --workspace`で実行可能にする
3. When 既存の`Cargo.toml`の`[workspace]`セクションが空である, the 移行プロセス shall 旧セクションを削除しワークスペース定義に置き換える
4. The Cargoワークスペース shall `resolver = "2"`を指定し、dependency resolutionの一貫性を保証する

### Requirement 2: `pasta_core`クレートの分離
**Objective:** As a 開発者, I want パーサー層とレジストリ層を独立クレート化する, so that DSL解析ロジックと共有型定義を他の層から分離し、再利用可能にする

#### Acceptance Criteria
1. When `/crates/pasta_core/`ディレクトリが作成される, the pasta_core shall 独自のCargo.tomlを持ち、`name = "pasta_core"`を定義する
2. The pasta_core shall 以下のモジュールを含む:
   - `parser/ast.rs` (AST型定義)
   - `parser/mod.rs` (パーサーAPI)
   - `parser/grammar.pest` (Pest文法定義)
   - `registry/mod.rs` (レジストリAPI)
   - `registry/scene_registry.rs` (シーン管理)
   - `registry/word_registry.rs` (単語辞書)
   - `error.rs` (パース関連エラー型: ParseError等)
3. When pasta_coreがビルドされる, the pasta_core shall `pest`、`pest_derive`、および`thiserror`への依存関係を持つ
4. The pasta_core shall `lib.rs`で以下をモジュール単位で公開する:
   - `pub mod parser;` (AST型、parse関数を含む)
   - `pub mod registry;` (Registry型を含む)
   - `pub mod error;` (ParseError等パース関連エラー型を含む)
5. When 既存の`src/parser/`および`src/registry/`から移行される, the pasta_core shall すべての関連テストを`tests/`配下に移動し実行可能にする

### Requirement 3: `pasta_rune`クレートの分離
**Objective:** As a 開発者, I want ランタイム・トランスパイラ層を独立クレート化する, so that 実行エンジンとコア層の疎結合を実現する

#### Acceptance Criteria
1. When `/crates/pasta_rune/`ディレクトリが作成される, the pasta_rune shall 独自のCargo.tomlを持ち、`name = "pasta_rune"`を定義する
2. The pasta_rune shall 以下のモジュールを含む:
   - `transpiler/` (code_generator, context)
   - `runtime/` (generator, variables, scene, words, random)
   - `stdlib/` (persistence)
   - `engine.rs`, `cache.rs`, `loader.rs`, `error.rs` (Rune実行時エラー型: PastaError等), `ir/`
3. When pasta_runeがビルドされる, the pasta_rune shall `pasta_core`, `rune`, `thiserror`, `glob`, `tracing`, `rand`, `futures`, `toml`, `fast_radix_trie`への依存関係を持つ
4. The pasta_rune shall `pasta_core`のAST型、Registry型、および相互参照を`use pasta_core::{parser, registry, error}`でインポートする
5. When 既存の統合テストが移行される, the pasta_rune shall `tests/`配下のすべてのテストファイルを実行可能にする

### Requirement 4: ルートCargo.tomlの構成変更
**Objective:** As a 開発者, I want ルートCargo.tomlをワークスペース管理専用にする, so that 依存関係の重複を排除し、バージョン管理を統一する

#### Acceptance Criteria
1. When ルートCargo.tomlが更新される, the ルートCargo.toml shall `[workspace]`セクションで`members`および`resolver`を定義する
2. The ルートCargo.toml shall `[workspace.dependencies]`セクションで共通依存関係（rune, thiserror, pest等）のバージョンを一元管理する
3. When メンバークレートがルートで定義された依存関係を使用する, the メンバークレート shall `依存関係名.workspace = true`記法で参照する
4. The ルートCargo.toml shall `[package]`セクションを削除し、ワークスペース専用構成にする

### Requirement 5: ディレクトリ構造の移行
**Objective:** As a 開発者, I want 既存のsrc/構造を新クレート構成に移行する, so that モジュール配置が明確化され、ビルドエラーなく動作する

#### Acceptance Criteria
1. When `src/parser/`および`src/registry/`が移行される, the 移行プロセス shall `/crates/pasta_core/src/`へファイルを移動する
2. When `src/`の残りモジュールが移行される, the 移行プロセス shall `/crates/pasta_rune/src/`へファイルを移動する
3. While 移行作業中である, the 移行プロセス shall ルートディレクトリの`src/`を空にする
4. When 移行が完了する, the プロジェクトルート shall `src/`ディレクトリを削除する
5. The 移行プロセス shall `tests/fixtures/`をワークスペースレベル`/tests/fixtures/`に移動する（両クレート共有）
6. When `examples/`ディレクトリが存在する, the 移行プロセス shall `/crates/pasta_rune/examples/`へ移動する（言語層依存サンプル）

### Requirement 6: テストの継続性保証
**Objective:** As a 開発者, I want 既存のすべてのテストが新構成で実行可能である, so that リグレッションなく移行を完了できる

#### Acceptance Criteria
1. When `cargo test --workspace`が実行される, the テストスイート shall すべての既存テストを実行し、成功する
2. When パーサー・レジストリテストが実行される, the pasta_core shall 独自の`tests/`配下のテストで検証される
3. When 統合テストが実行される, the pasta_rune shall `tests/pasta_*_test.rs`形式のすべてのテストファイルを実行する
4. If 既存テストがインポートパスの変更により失敗する, then the 移行プロセス shall インポート文を`use pasta_core::{parser, registry, error}`および`use pasta_rune::`に修正する
5. The テストスイート shall ワークスペースレベル`/tests/common/mod.rs`でフィクスチャパス解決等の共通ユーティリティを提供する
6. When テストがワークスペースレベル`/tests/fixtures/`からフィクスチャを読み込む, the テストコード shall `tests/common/`のユーティリティ関数を使用してパス解決する

### Requirement 7: ビルド互換性の維持
**Objective:** As a 開発者, I want 既存のビルドコマンドが新構成で動作する, so that CI/CDパイプラインへの影響を最小化する

#### Acceptance Criteria
1. When `cargo build`が実行される, the Cargoワークスペース shall すべてのメンバークレートをビルドする
2. When `cargo build --release`が実行される, the Cargoワークスペース shall リリースビルドを成功させる
3. The Cargoワークスペース shall ビルド成果物を`target/`ディレクトリに統一的に配置する
4. When 依存関係が解決される, the Cargoワークスペース shall バージョンコンフリクトなく依存関係グラフを構築する

### Requirement 8: ドキュメントの更新
**Objective:** As a 開発者, I want プロジェクト文書が新構成を反映する, so that 新規貢献者が正確な情報を得られる

#### Acceptance Criteria
1. When `.kiro/steering/structure.md`が更新される, the structure.md shall ワークスペース構成および`/crates/`配下の構造を記述する
2. When `README.md`にビルド手順が記載されている場合, the README.md shall `cargo build --workspace`および`cargo test --workspace`コマンドを明記する
3. The ドキュメント shall パッケージ間の依存関係図を含む（pasta_rune → pasta_parser）
4. When `.kiro/steering/tech.md`が参照される, the tech.md shall ワークスペース構成をアーキテクチャ原則に追加する

### Requirement 9: 公開API境界の明確化
**Objective:** As a 開発者, I want クレート間のAPI境界が明確である, so that 不要な内部実装の公開を防ぐ

#### Acceptance Criteria
1. The pasta_core shall `lib.rs`で以下をモジュール単位で公開する:
   - `pub mod parser;` (parse関数、AST型を含む)
   - `pub mod registry;` (Registry型を含む)
   - `pub mod error;` (ParseError等パース関連エラー型を含む)
2. The pasta_rune shall `lib.rs`でコアへの間接公開と言語層APIを公開する:
   - `pub use pasta_core as core;` (コアモジュールへの間接公開、`core`という名前で)
   - `pub use engine::PastaEngine;`
   - `pub use ir::{ScriptEvent, ContentPart, ...};`
   - `pub use error::PastaError;` (Rune実行時エラー)
3. When 内部モジュールが他クレートから参照不要である, the 内部モジュール shall `pub(crate)`可視性を使用する
4. The pasta_core shall Pest文法ファイル（grammar.pest）をクレート内部に保持し、外部公開しない
5. When 外部ユーザーがpasta_coreの型にアクセスする場合, the ユーザー shall `use pasta_rune::core::{parser, registry}`経由でアクセス可能である

