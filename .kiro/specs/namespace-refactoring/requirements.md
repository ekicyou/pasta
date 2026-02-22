# Requirements Document

## Introduction

pasta ワークスペース全体のクレート・テストにおける名前空間（モジュール・ディレクトリ構造）を見直し、一貫性と可読性を向上させるリファクタリング仕様。特に `pasta_lua` クレートのテストファイルが 58 ファイル（.rs 36 + .lua 22）にまで肥大化しており、フラットな `tests/` 配置が保守性を損なっている。`pasta_lua` を重点対象に統一的な名前空間方針を策定し、適用する。

**スコープ外**: pasta_lsp（テスト10本、現時点で実害なし。将来テスト増加時に別途対応）。

## Project Description (Input)

全クレートおよびテストの名前空間整理。
特にテストにおいてファイル数が増えすぎている個所があり、適切な名前空間に仕分けるべき。その他、全体機能について一度適切な名前空間を検討し、名前空間リファクタリングを行う。

## Requirements

### Requirement 1: テストディレクトリのサブモジュール化方針策定

**Objective:** 開発者として、テストファイルが肥大化したクレートに対して機能別サブディレクトリ分割の方針を持ちたい。これにより、テスト追加時の配置先が自明になり、一覧性と保守性が向上する。

#### Acceptance Criteria

1. The namespace-refactoring shall テスト数が 10 ファイルを超えるクレートについて、機能ドメイン別のサブディレクトリ構造を定義する
2. The namespace-refactoring shall Rust 統合テストのサブディレクトリ化方式（例: `tests/<category>/main.rs` + サブモジュール）を採用し、`cargo test` で全テストが検出される構成を維持する
3. The namespace-refactoring shall 既存テストの全件が、リファクタリング後も `cargo test --all` で同一結果（全パス）となることを保証する

### Requirement 2: pasta_lua テストの名前空間整理

**Objective:** 開発者として、36 本の .rs テストファイルが `tests/` 直下にフラットに並ぶ `pasta_lua` クレートのテストを機能ドメイン別に整理したい。これにより、関連テストの発見性と一覧性が向上する。

#### Acceptance Criteria

1. The namespace-refactoring shall `pasta_lua/tests/` 配下に以下の機能ドメインに対応するサブディレクトリを作成する: transpiler系、runtime系、loader系、sakura_script系、shiori系、search系、log系、その他
2. The namespace-refactoring shall 各 .rs テストファイルを対応する機能ドメインのサブディレクトリへ移動する
3. The namespace-refactoring shall `tests/common/` および `tests/fixtures/` の共有リソースは現在の位置を維持し、移動後のテストから正しく参照できる
4. The namespace-refactoring shall `tests/lua_specs/` および `tests/snapshots/` は既にサブディレクトリ化されているため、位置を維持する
5. If テスト移動により `mod.rs` の追加や `#[path]` アトリビュートが必要になる場合, the namespace-refactoring shall Rust の統合テストのディレクトリ慣例（各ファイルが独立クレートとなるフラット構成 vs サブモジュール構成）に基づき、最適な方式で構成する

### Requirement 3: src 内テストファイルの配置方針統一

**Objective:** 開発者として、`src/` 内に配置されたテストファイル（`pasta_core/src/registry/scene_table_tests.rs` および `pasta_shiori/src/shiori_tests.rs`）の取り扱い方針を統一したい。一貫したプロジェクト規約によりテスト配置の迷いを排除する。

**ギャップ分析結果**: 両ファイルとも private フィールド（`labels`, `prefix_index`, `cache` 等）への直接アクセスが構造的に必要であり、`tests/` への外部化は技術的に不適切。`#[cfg(test)] #[path]` パターンを正式方針として採用する。

#### Acceptance Criteria

1. The namespace-refactoring shall `src/` 内テストファイルについて、`#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;` パターンを正式な配置方針として採用する（private フィールドへの直接アクセスが必要なテストに限定）
2. The namespace-refactoring shall 公開 API のみをテストする統合テストは従来通り `tests/` に外部化する方針を維持する
3. The namespace-refactoring shall 選択された方針を `steering/structure.md` に明記し、今後の開発で一貫性を保つ

### Requirement 4: ソースモジュールの名前空間レビュー

**Objective:** 開発者として、各クレートの `src/` モジュール構造が責務に対して適切な粒度と階層を持つことを確認したい。特に `pasta_lua`（30 ファイル）は責務の分離と名前空間の整合性を検証する。

#### Acceptance Criteria

1. The namespace-refactoring shall 各クレートの `src/` モジュールについて、現在の名前空間が責務に対して適切かレビューする
2. If モジュールが所属するディレクトリの責務と合致しない場合, the namespace-refactoring shall 適切なディレクトリへ移動し、`mod.rs` の re-export を更新する
3. The namespace-refactoring shall `string_literalizer.rs` や `normalize.rs` のような `pasta_lua/src/` 直下のユーティリティファイルが適切な名前空間に属しているか検証する

### Requirement 5: テストファイル命名規則の徹底

**Objective:** 開発者として、全クレートのテストファイル名が一貫した命名規則に従っていることを確認したい。`steering/tech.md` で定義済みの `<feature>_test.rs` パターンが全体で遵守されているか検証する。

#### Acceptance Criteria

1. The namespace-refactoring shall 全テストファイルが `<feature>_test.rs` パターンに準拠していることを検証する
2. If 命名規則に従わないファイル（例: `lua_unittest_runner.rs`）が存在する場合, the namespace-refactoring shall 規則に準拠したファイル名にリネームする、または例外として承認された理由を文書化する

### Requirement 6: steering/structure.md の更新

**Objective:** 開発者として、リファクタリング後のディレクトリ構造を `steering/structure.md` に正確に反映したい。ステアリングが常に実体と一致していることが、AI支援開発の品質基盤となる。

#### Acceptance Criteria

1. When 名前空間リファクタリングが完了した時, the namespace-refactoring shall `steering/structure.md` のディレクトリツリーを実際の構造と同期する
2. The namespace-refactoring shall テストディレクトリのサブモジュール化方針をステアリングに明記する
3. The namespace-refactoring shall `src/` 内テストファイルの取り扱い方針をステアリングに明記する
