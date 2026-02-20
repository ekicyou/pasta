# Requirements Document

## Project Description (Input)
全クレートのソースコードを対象に、AIフレンドリーなファイルサイズへの分割リファクタリングを実施する。LLMのコンテキストウィンドウで効率的に処理でき、AIコーディングアシスタントが正確に理解・編集できるファイルサイズを目標とする。

## Introduction

pastaワークスペースの全クレート（pasta_dsl, pasta_core, pasta_lua, pasta_lsp, pasta_shiori, pasta_sample_ghost）を対象に、ソースファイルをAIコーディングアシスタントが効率的に処理できるサイズへ分割するリファクタリング仕様である。

### 現状分析

現在、500行を超えるソースファイルが16個存在し、最大は1,301行（pasta_dsl/src/parser/mod.rs）に達する。LLMのコンテキストウィンドウにおいて、大きなファイルはコード理解精度の低下、編集時のコンテキスト消費の増大、差分検出の困難さを引き起こす。

**500行超のソースファイル一覧（現状）:**

| 行数 | ファイル | クレート |
|------|----------|----------|
| 1,301 | parser/mod.rs | pasta_dsl |
| 1,186 | analysis.rs | pasta_lsp |
| 1,086 | transpiler_integration_test.rs | pasta_lua (test) |
| 1,026 | runtime/mod.rs | pasta_lua |
| 1,000 | shiori.rs | pasta_shiori |
| 933 | shiori_event_test.rs | pasta_lua (test) |
| 923 | registry/scene_table.rs | pasta_core |
| 892 | code_generator.rs | pasta_lua |
| 800 | parser/ast.rs | pasta_dsl |
| 763 | loader/config.rs | pasta_lua |
| 612 | loader_integration_test.rs | pasta_lua (test) |
| 582 | loader/cache.rs | pasta_lua |
| 565 | runtime_e2e_test.rs | pasta_lua (test) |
| 559 | sakura_script_integration_test.rs | pasta_lua (test) |
| 557 | registry/word_table.rs | pasta_core |
| 519 | virtual_event_dispatcher_test.rs | pasta_lua (test) |

## Requirements

### Requirement 1: ファイルサイズ基準の定義

**Objective:** AI開発者として、明確なファイルサイズガイドラインが欲しい。LLMコンテキストウィンドウで効率的に処理でき、AIコーディングアシスタントが正確に理解・編集できる最大サイズの基準とするため。

#### Acceptance Criteria

1. The リファクタリング shall ソースファイル（`src/` 配下の `.rs` ファイル）の目標上限を **300行** と定める
2. The リファクタリング shall テストファイル（`tests/` 配下の `.rs` ファイル）の目標上限を **500行** と定める
3. The リファクタリング shall 300行以下のソースファイルは分割対象外とする
4. If ファイルが自動生成コードやマクロ展開など構造的に分割不能な場合, the リファクタリング shall 当該ファイルを例外として記録し、分割をスキップする
5. The リファクタリング shall pest文法定義ファイル（`.pest`）を分割対象外とする

### Requirement 2: ソースファイルの分割

**Objective:** AI開発者として、大きなソースファイルを論理的な単位で分割したい。各ファイルが単一責務を持ち、AIアシスタントがファイル全体を一度に読み込んで正確に理解できるようにするため。

#### Acceptance Criteria

1. When ソースファイルが300行を超えている場合, the リファクタリング shall 当該ファイルを論理的な責務単位（型定義、トレイト実装、ヘルパー関数等）で複数ファイルに分割する
2. The リファクタリング shall 分割後の各ファイルが単一の明確な責務を持つようにする
3. The リファクタリング shall 分割後のモジュール構造が既存のクレート公開API（`pub use`）を維持する
4. When ファイルを分割する場合, the リファクタリング shall 適切な `mod.rs` または親モジュールからの `pub use` re-export により、外部から見たAPIの互換性を保持する
5. The リファクタリング shall 分割後のファイル名がsteering（structure.md）のファイル命名規則に従う

### Requirement 3: テストファイルの分割

**Objective:** AI開発者として、大きなテストファイルも適切なサイズに分割したい。テストの発見・理解・メンテナンスをAIアシスタントが効率的に行えるようにするため。

#### Acceptance Criteria

1. When テストファイルが500行を超えている場合, the リファクタリング shall 当該テストファイルをテスト対象の機能単位で複数ファイルに分割する
2. The リファクタリング shall 分割後のテストファイルが `<feature>_test.rs` の命名規則に従う
3. When テストファイルが共通ヘルパーを含む場合, the リファクタリング shall 共通部分を `common/` モジュールに抽出する
4. The リファクタリング shall 分割後のすべてのテストが `cargo test --workspace` で引き続き実行可能であること

### Requirement 4: API互換性の維持

**Objective:** AI開発者として、リファクタリング後もすべての既存コードが動作し続けることを保証したい。分割が純粋な内部構造の変更であり、外部APIに影響しないことを確認するため。

#### Acceptance Criteria

1. The リファクタリング shall 各クレートの `lib.rs` で公開している型・関数・トレイトのシグネチャを変更しない
2. The リファクタリング shall クレート間の依存関係（`Cargo.toml` の `[dependencies]`）を変更しない
3. When 分割によりモジュールパスが変更される場合, the リファクタリング shall re-export により従来のパスからのアクセスを維持する
4. The リファクタリング shall `cargo test --workspace` が分割前と同じテスト結果（全テストパス）となること
5. The リファクタリング shall `cargo build --workspace` がワーニングなしで成功すること
6. The リファクタリング shall テスト外部化に伴う `pub(crate)` 可視性昇格を「API変更」とはみなさない（`pub(crate)` は同一クレート内限定であり、外部クレートの依存関係・公開シグネチャに影響しないため）

### Requirement 5: 分割戦略の優先順位

**Objective:** AI開発者として、効果的な順序でリファクタリングを進めたい。最も効果が高いファイルから着手し、段階的にプロジェクト全体の品質を向上させるため。

#### Acceptance Criteria

1. The リファクタリング shall 1,000行超のファイル（Critical: 5ファイル）を最優先で分割する
2. The リファクタリング shall 次に500〜999行のファイルを分割する
3. The リファクタリング shall ソースファイル（`src/`）をテストファイル（`tests/`）より先に分割する
4. The リファクタリング shall 下位レイヤー（pasta_dsl → pasta_core → pasta_lua）の順に分割し、上位レイヤーへの影響を最小化する

### Requirement 6: structure.mdステアリングの更新

**Objective:** AI開発者として、分割後のディレクトリ構造がステアリングに正確に反映されていてほしい。今後のAIアシスタントがプロジェクト構造を正しく把握できるようにするため。

#### Acceptance Criteria

1. When ファイル分割が完了した場合, the リファクタリング shall `.kiro/steering/structure.md` のディレクトリ構造を更新する
2. The リファクタリング shall 新たに作成されたモジュールの責務を structure.md に記載する
3. The リファクタリング shall 各クレートの README.md を分割後の構造に合わせて更新する
