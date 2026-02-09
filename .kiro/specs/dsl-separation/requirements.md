# Requirements Document

## Introduction

pasta_coreクレートは現在、DSLパーサー（Pest文法 + AST定義）とレジストリ（シーン/単語テーブル管理）を「言語非依存層」として一体提供している。しかし、pasta DSLの文法定義・AST・パーサーは本質的に**DSL固有の関心事**であり、レジストリ層とは独立して再利用・差し替え可能であるべきである。

本仕様では、pasta_coreからDSL固有部分を新規クレート `pasta_dsl` として抽出し、DSLパーサーを単独で利用・テスト・差し替えできるようにする。これにより、Pasta DSLを活用した別のバックエンド実装や、DSL文法の独立した進化が容易になる。
**移行方針**: 下流クレート（pasta_lua等）は `pasta_dsl` に直接依存するよう変更する。pasta_core はパーサーの再エクスポートを行わず、レジストリ層等のユーティリティに特化する。
## Project Description (Input)
pasta_dslクレートにDSL関係を分離する。coreからDSLを分離してDSLだけを取り出して別のものを実装しやすく。

## Requirements

### Requirement 1: DSLクレート抽出

**Objective:** 開発者として、Pasta DSLのパーサー・AST定義を独立クレートとして利用したい。これにより、Luaバックエンドに依存しない新しいバックエンドを容易に実装できるようにする。

#### Acceptance Criteria

1. The pasta_dsl crate shall DSL文法定義（`grammar.pest`）、AST型定義（`ast.rs`）、パーサーモジュール（`mod.rs`）を独立したクレートとして提供する
2. When `pasta_dsl` をCargo依存に追加した場合, the pasta_dsl crate shall `parse_str()` および `parse_file()` によるDSLパースと `PastaFile`・`FileItem`・`GlobalSceneScope`・`Action` 等のAST型を公開APIとして提供する
3. The pasta_dsl crate shall Pest 2.8によるPEGパーサー生成と `thiserror` によるエラー型のみを必須依存とする（レジストリ関連依存を含まない）
4. The pasta_dsl crate shall パースエラー型（`ParseError`, `ParseErrorInfo`）を独立して定義・公開する

### Requirement 2: pasta_coreの整理と下流クレートの移行

**Objective:** 開発者として、pasta_coreかDSLパーサーを完全に分離し、下流クレートが `pasta_dsl` に直接依存するよう移行したい。pasta_coreはレジストリ等のユーティリティに特化する。

#### Acceptance Criteria

1. The pasta_core crate shall parserモジュールを完全に除去し、`pasta_dsl` への再エクスポートも行わない
2. The pasta_core crate shall レジストリ層（`SceneRegistry`, `WordDefRegistry`, `SceneTable`, `WordTable`, `RandomSelector`）を引き続き直接所有する
3. The pasta_lua crate shall `pasta_dsl` を直接依存に追加し、`pasta_core::parser` への参照を `pasta_dsl::parser` に変更する
4. The pasta_core crate shall `pasta_dsl` への依存を持たない（parserが完全に分離される）
5. When `cargo test --all` を実行した場合, the workspace shall 移行後も全テストが成功する

### Requirement 3: ワークスペース統合

**Objective:** 開発者として、pasta_dslをワークスペースの一員として管理し、一貫したバージョニング・ビルド・テストのワークフローを維持したい。

#### Acceptance Criteria

1. The workspace Cargo.toml shall `crates/pasta_dsl` をワークスペースメンバーとして含む
2. The workspace Cargo.toml shall `pasta_dsl` をワークスペース依存として定義する（`path = "crates/pasta_dsl"`）
3. When `cargo test --all` を実行した場合, the workspace shall pasta_dslを含むすべてのクレートのテストが成功する
4. When `cargo build --workspace` を実行した場合, the workspace shall 依存解決エラーなくビルドが成功する

### Requirement 4: DSLクレートの独立利用性

**Objective:** 外部開発者として、pasta_dslクレートのみを依存に追加して、Pasta DSLのパースを行いたい。pasta_coreやpasta_luaに依存せずにDSL解析が可能であること。

#### Acceptance Criteria

1. The pasta_dsl crate shall レジストリ（`fast_radix_trie`, `rand`）への依存を持たない
2. The pasta_dsl crate shall Luaランタイム（`mlua`）への依存を持たない
3. When `pasta_dsl` のみを依存に追加したプロジェクトでビルドした場合, the project shall pasta_core・pasta_lua なしでコンパイルが成功する
4. The pasta_dsl crate shall 単体で `cargo test -p pasta_dsl` が成功する独自のテストスイートを持つ
5. When DSLパーサーのみをテスト対象とする既存テストファイル（`actor_code_block_test.rs`, `digit_id_var_test.rs`, `sakura_symbol_tag_test.rs`, `span_byte_offset_test.rs`）が存在する場合, the pasta_dsl crate shall これらを `crates/pasta_dsl/tests/` に移動し、import パスを `pasta_dsl::parser` に変更した上でテストが成功する
6. When DSLテストがpasta_dslに移動された後, the pasta_core crate shall 移動元のテストファイルを保持しない（重複テストを排除する）

### Requirement 5: エラー型の分離

**Objective:** 開発者として、DSLパースエラーとレジストリエラーが明確に分離され、それぞれのクレートが独立してエラー型を管理できるようにしたい。

#### Acceptance Criteria

1. The pasta_dsl crate shall `ParseError` および `ParseErrorInfo` をDSLクレート内で定義・公開する
2. The pasta_core crate shall レジストリエラー（`SceneTableError`, `WordTableError`）を自身で定義し、ParseErrorの再エクスポートは行わない
3. The pasta_lua crate shall パースエラーを `pasta_dsl` から直接参照する（`pasta_core::ParseError` への参照を `pasta_dsl::ParseError` に変更）

### Requirement 6: ドキュメント・ステアリングのモジュール構成図更新

**Objective:** 開発者として、pasta_dslクレート追加後もすべてのドキュメントが実際のモジュール構成と一致していることを保証し、AI・人間ともに正確なコンテキストで作業できるようにしたい。

#### Acceptance Criteria

以下のドキュメントに含まれるモジュール構成図・クレート一覧・ディレクトリツリーを、pasta_dslクレートの追加を反映して更新する：

1. When pasta_dslクレートが追加された場合, the [README.md](../../../README.md) shall レイヤー構成図とディレクトリツリーにpasta_dslを含む
2. When pasta_dslクレートが追加された場合, the [SOUL.md](../../../SOUL.md) shall ドキュメントヒエラルキー（Level 2: Implementation Layer）のクレートREADMEリストにpasta_dslを含む
3. When pasta_dslクレートが追加された場合, the [.kiro/steering/tech.md](../../../.kiro/steering/tech.md) shall ワークスペースレイヤー構成図とクレート責務テーブルにpasta_dslを含む
4. When pasta_dslクレートが追加された場合, the [.kiro/steering/structure.md](../../../.kiro/steering/structure.md) shall ディレクトリ構造ツリー、ワークスペース構成図、レイヤー分離原則にpasta_dslを含む
5. When pasta_dslクレートが追加された場合, the [crates/pasta_core/README.md](../../../crates/pasta_core/README.md) shall アーキテクチャ図とディレクトリ構成をpasta_dsl依存に更新する
6. When pasta_dslクレートが追加された場合, the [crates/pasta_lua/README.md](../../../crates/pasta_lua/README.md) shall 依存関係セクションに `pasta_dsl` への直接依存を反映する
7. When pasta_dslクレートが追加された場合, the [crates/pasta_shiori/README.md](../../../crates/pasta_shiori/README.md) shall 依存関係テーブルにおけるpasta_dslの位置付けを反映する（pasta_core経由の間接依存ではない）
8. When DSLテストがpasta_dslに移動された場合, the [TEST_COVERAGE.md](../../../TEST_COVERAGE.md) shall テストカバレッジサマリーのクレート一覧にpasta_dslを追加し、pasta_coreのテスト数を更新する
