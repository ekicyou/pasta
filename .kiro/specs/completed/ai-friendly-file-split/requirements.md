# Requirements Document

## Project Description (Input)
全クレートのソースコードを対象に、AIフレンドリーなファイルサイズへの分割リファクタリングを実施する。LLMのコンテキストウィンドウで効率的に処理でき、AIコーディングアシスタントが正確に理解・編集できるファイルサイズを目標とする。

## Introduction

pastaワークスペースの全クレート（pasta_dsl, pasta_core, pasta_lua, pasta_lsp, pasta_shiori, pasta_sample_ghost）を対象に、ソースファイルをAIコーディングアシスタントが効率的に処理できるサイズへ分割するリファクタリング仕様である。

### 現状分析

現在、500行を超えるソースファイルが16個存在し、最大は1,405行（pasta_dsl/src/parser/mod.rs）に達する。LLMのコンテキストウィンドウにおいて、大きなファイルはコード理解精度の低下、編集時のコンテキスト消費の増大、差分検出の困難さを引き起こす。

**500行超のソースファイル一覧（現状）:**

| 行数 | ファイル | クレート | 種別 |
|------|----------|----------|------|
| 1,405 | parser/mod.rs | pasta_dsl | src |
| 1,283 | analysis.rs | pasta_lsp | src |
| 1,187 | shiori.rs | pasta_shiori | src |
| 1,174 | runtime/mod.rs | pasta_lua | src |
| 1,086 | transpiler_integration_test.rs | pasta_lua | test |
| 1,053 | registry/scene_table.rs | pasta_core | src |
| 1,002 | code_generator.rs | pasta_lua | src |
| 933 | shiori_event_test.rs | pasta_lua | test |
| 885 | parser/ast.rs | pasta_dsl | src |
| 850 | loader/config.rs | pasta_lua | src |
| 701 | loader/cache.rs | pasta_lua | src |
| 649 | registry/word_table.rs | pasta_core | src |
| 612 | loader_integration_test.rs | pasta_lua | test |
| 565 | runtime_e2e_test.rs | pasta_lua | test |
| 559 | sakura_script_integration_test.rs | pasta_lua | test |
| 519 | virtual_event_dispatcher_test.rs | pasta_lua | test |

### 根本原因分析（ギャップ分析に基づく）

ギャップ分析（gap-analysis.md）により、ソースファイル肥大化の最大要因が**インラインテスト**であることが判明した。`src/` 配下の10ファイルに合計3,634行（193テスト関数）のインラインテストが存在し、ファイル全体の **11〜67%** を占めている。

テスト外部化による効果を以下に示す：

| ファイル | 現在 | テスト外部化後 | Phase B判定 |
|----------|------|--------------|-------------|
| parser/mod.rs | 1,405行 | ~1,138行 | ✅ 要分割 |
| analysis.rs | 1,283行 | ~1,145行 | ✅ 要分割 |
| shiori.rs | 1,187行 | ~394行 (#[path]分離) | ガイドライン例外 |
| runtime/mod.rs | 1,174行 | ~831行 | ✅ 要分割 |
| scene_table.rs | 1,053行 | ~449行 (#[path]分離) | ✅ 要分割（型分離） |
| code_generator.rs | 1,002行 | ~778行 | ✅ 要分割 |
| ast.rs | 885行 | ~759行 | ✅ 要分割 |
| config.rs | 850行 | ~427行 | ガイドライン例外 |
| cache.rs | 701行 | ~387行 | ガイドライン例外 |
| word_table.rs | 649行 | ~247行 | ❌ **不要** |

テスト外部化だけで **word_table.rs の1ファイルが300行以下** に収まりPhase B不要となる。shiori.rs・cache.rs・config.rs はテスト外部化後300行超だが自然な分割境界が存在しないためガイドライン例外（Req2 AC3）とする。この知見に基づき、実行フェーズを「テスト外部化 → ソース分割 → テスト分割」の3段階で構成する。

## Requirements

### Requirement 1: ファイルサイズ基準とテスト配置ポリシー

**Objective:** AI開発者として、明確なファイルサイズガイドラインとテスト配置ルールが欲しい。LLMコンテキストウィンドウで効率的に処理でき、AIコーディングアシスタントが正確に理解・編集できる最大サイズの基準と、ソースファイル肥大化の根本原因への対処方針とするため。

#### Acceptance Criteria

**サイズ基準**

1. The リファクタリング shall ソースファイル（`src/` 配下のプロダクションコード `.rs` ファイル）の目標上限を **300行** と定める
2. The リファクタリング shall テストファイル（`tests/` 配下の `.rs` ファイル、および `src/` 配下の `#[path]` テスト専用ファイル）の目標上限を **500行** と定める

**テスト配置ポリシー**

3. The リファクタリング shall `src/` 配下のインラインテスト（`#[cfg(test)] mod tests`）を原則として `tests/` ディレクトリに外部化する
4. If テスト外部化に際して `pub(crate)` 可視性昇格やgetter追加など軽微なコード変更で対応可能な場合, the リファクタリング shall 当該変更を実施してテストを外部化する
5. If ファイルのテストがprivateフィールドへの直接アクセスを構造的に必要とし、カプセル化の崩壊なしには外部化できない場合, the リファクタリング shall 当該テストを `#[path]` パターンで `src/` 配下の別ファイルに分離し、例外として記録する

**除外基準**

6. The リファクタリング shall 300行以下のソースファイルをPhase B（ソース分割）の対象外とする
7. If ファイルが自動生成コードやマクロ展開など構造的に分割不能な場合, the リファクタリング shall 当該ファイルを例外として記録し、分割をスキップする
8. The リファクタリング shall pest文法定義ファイル（`.pest`）を分割対象外とする

### Requirement 2: ソースファイルの責務分割

**Objective:** AI開発者として、大きなソースファイルを論理的な単位で分割したい。各ファイルが単一責務を持ち、AIアシスタントがファイル全体を一度に読み込んで正確に理解できるようにするため。

#### Acceptance Criteria

1. The リファクタリング shall テスト外部化（Phase A）完了後のソースファイル行数で分割要否を判定する
2. When ソースファイルがPhase A完了後も300行を超えている場合, the リファクタリング shall 当該ファイルに自然な責務分割境界（型定義、トレイト実装、ヘルパー関数等）が存在するか評価し、境界が存在する場合に複数ファイルへ分割する
3. If ソースファイルが300行を小幅に超えているが自然な分割境界が存在しない場合, the リファクタリング shall 当該ファイルの微超を許容し、例外として記録する
3. The リファクタリング shall 分割後の各ファイルが単一の明確な責務を持つようにする
4. When ファイルを分割する場合, the リファクタリング shall ディレクトリモジュール（`mod.rs` + サブモジュール）パターンで構成し、既存のプロジェクト慣例に従う
5. The リファクタリング shall 分割後のファイル名がsteering（structure.md）のファイル命名規則に従う

※ 分割後のAPI互換性保持（re-export、公開APIの維持）はRequirement 4で規定

### Requirement 3: テストファイルの分割

**Objective:** AI開発者として、大きなテストファイルも適切なサイズに分割したい。テストの発見・理解・メンテナンスをAIアシスタントが効率的に行えるようにするため。

#### Acceptance Criteria

1. When テストファイルが500行を超えている場合, the リファクタリング shall 当該テストファイルをテスト対象の機能単位で複数ファイルに分割する
2. The リファクタリング shall Phase Aで新たに外部化されたテストファイル、および既存の `tests/` 配下テストファイルの双方を分割対象とする
3. The リファクタリング shall `src/` 配下の `#[path]` テスト専用ファイルもテストファイルとして本要件のサイズ基準（500行）を適用する
4. The リファクタリング shall 分割後のテストファイルが `<feature>_test.rs` の命名規則に従う
5. When テストファイルが共通ヘルパーを含む場合, the リファクタリング shall 共通部分を `common/` モジュールに抽出する
6. The リファクタリング shall 分割後のすべてのテストが `cargo test --workspace` で引き続き実行可能であること

### Requirement 4: API互換性の維持

**Objective:** AI開発者として、リファクタリング後もすべての既存コードが動作し続けることを保証したい。分割が純粋な内部構造の変更であり、外部APIに影響しないことを確認するため。

#### Acceptance Criteria

1. The リファクタリング shall 各クレートの `lib.rs` で公開している型・関数・トレイトのシグネチャを変更しない
2. The リファクタリング shall クレート間の依存関係（`Cargo.toml` の `[dependencies]`）を変更しない
3. When 分割によりモジュールパスが変更される場合, the リファクタリング shall re-export により従来のパスからのアクセスを維持する
4. The リファクタリング shall テスト外部化に伴う `pub(crate)` 可視性昇格を「API変更」とはみなさない（`pub(crate)` は同一クレート内限定であり、外部クレートの依存関係・公開シグネチャに影響しないため）
5. The リファクタリング shall `cargo test --workspace` が分割前と同じテスト結果（全テストパス）となること
6. The リファクタリング shall `cargo build --workspace` がワーニングなしで成功すること

### Requirement 5: 実行フェーズと分割順序

**Objective:** AI開発者として、効果的な順序でリファクタリングを進めたい。最もリスクが低く効果が高い作業から段階的に着手し、各フェーズで安定した状態を維持するため。

#### Acceptance Criteria

1. The リファクタリング shall まず全対象ファイルのインラインテストを `tests/` へ外部化する（Phase A: テスト外部化）
2. If ファイルのインラインテストがprivateフィールドへの直接アクセスを構造的に必要とする場合, the リファクタリング shall Phase Aにおいて `#[path]` パターンで `src/` 配下の別ファイルに分離する
3. The リファクタリング shall Phase A完了後に300行を超えるソースファイルを責務単位で分割する（Phase B: ソース分割）
4. The リファクタリング shall `tests/` 配下で500行を超えるテストファイル、および `src/` 配下の500行を超える `#[path]` テストファイルを機能単位で分割する（Phase C: テスト分割）
5. The リファクタリング shall 各フェーズにおいて下位レイヤー（pasta_dsl → pasta_core → pasta_lua → pasta_lsp → pasta_shiori）の順に処理し、上位レイヤーへの影響を最小化する
6. The リファクタリング shall 各フェーズ完了時に `cargo test --workspace` が全テストパスであることを確認し、次のフェーズに進む

### Requirement 6: ドキュメントとステアリングの更新

**Objective:** AI開発者として、分割後のディレクトリ構造とテスト構成がステアリング・ドキュメントに正確に反映されていてほしい。今後のAIアシスタントがプロジェクト構造を正しく把握できるようにするため。

#### Acceptance Criteria

1. When ファイル分割が完了した場合, the リファクタリング shall `.kiro/steering/structure.md` のディレクトリ構造を更新する
2. The リファクタリング shall 新たに作成されたモジュールの責務を structure.md に記載する
3. The リファクタリング shall 各クレートの README.md を分割後の構造に合わせて更新する
4. The リファクタリング shall `TEST_COVERAGE.md` のテストマッピングを分割後のファイル構成に合わせて更新する
5. The リファクタリング shall インラインテスト例外として記録したファイルの一覧と理由を design.md の例外記録に残す
