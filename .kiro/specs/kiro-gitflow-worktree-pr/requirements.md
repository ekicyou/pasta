# Requirements Document

## Introduction

kiro ワークフローの `kiro-start` / `kiro-tasks` / `kiro-spec-complete` の3スキルと、その権威ソースである `.kiro/steering/workflow.md` §3「リモート同期（ブランチ戦略）」は、「ローカルでブランチを切り、`merge-base` から squash ブランチを作って fast-forward マージし、デフォルトブランチ（main）へ直接 push する」という独自の git 統合儀式を前提にしている。一方、現行の Claude Code ハーネスは「ワークツリーに隔離して作業し、最後に Pull Request（PR）でマージする」運用を前提としており、両者の前提が衝突している。手作業の squash-ff-push ロジックは複雑で保守コストが高く、main への直接 push はハーネスの「push は明示要求時のみ／デフォルトブランチ上ではまずブランチを切る」方針とも噛み合わない。

本仕様は、権威ソースである `workflow.md` を PR ベースのブランチ戦略へ全面改訂したうえで、3スキルから手作業の squash-ff-push 儀式と main 直接 push を撤去する。ワークツリー管理はハーネス／セッション管理へ委譲し、スキルは「commit → PR 作成 → PR の squash マージ → ブランチ削除」だけを担う。1つの feature = 1つのブランチ = 1つの PR とし、`kiro-tasks` の中間 main 統合と `impl/{feature}` 別ブランチ生成を撤去して planning と impl を同一ブランチへ積み、`kiro-spec-complete` で1回だけ PR → squash マージする。あわせて GitHub リポジトリ設定を squash 限定へ強制し、付け忘れや手動マージ事故を構造的に防止する。スキルの移植性（specs root / skill base / remote / default branch の決定的解決）と `{remote}` 不在時のフォールバック方針は現行同様に維持する。

## Boundary Context

- **In scope（含む）**:
  - `workflow.md` §3「リモート同期（ブランチ戦略）」と「直接 push」注記・squash メッセージ生成方針を PR ベースへ全面改訂（権威ソースを最初に確定）。
  - `kiro-spec-complete` の手作業 squash-ff-push（現 Step 8）を PR ベース（PR 作成 + squash マージ + ブランチ削除）へ置換。繰り返し仕様（release-workflow 等）の同期分岐も PR ベース化。
  - `kiro-tasks` の中間 main 統合（現 Step 5）と `impl/{feature}` 生成（現 Step 6）の撤去、および関連記述（Output/Constraints/Safety）の整合。
  - `kiro-start` のブランチ生成方針をワークツリー委譲モデルへ整合（デフォルトブランチ直作業の扱い、push しない方針の明記）。
  - `kiro-impl` への互換性注記（単一 feature ブランチ上で動作する旨）。
  - GitHub リポジトリのマージ方式を squash 限定へ変更する一度きりの設定作業のタスク化。
- **Out of scope（含まない）**:
  - kiro スキル群以外（CI/CD、他リポジトリ）の git 運用。
  - `git remote` URL に埋め込まれたトークンの是正（別件として推奨するが本仕様では扱わない）。
  - PR テンプレート整備や CODEOWNERS 等のレビュー体制構築（将来の別仕様候補）。
  - DSL / Lua / pasta 本体の機能変更。
- **Adjacent expectations（隣接系への期待）**:
  - ワークツリー隔離そのものはハーネス／セッション管理が担う（本仕様はスキル側の整合のみを担い、ハーネス機能の実装・改変は行わない）。**フィーチャーごとの非デフォルト作業ブランチは Claude Code（ハーネス）のワークツリー機能が供給する前提**とし、kiro スキルは自前でブランチ／ワークツリーを作成しない。
  - PR 作成・マージには `gh` CLI と PR 作成権限が利用可能であることを前提とする。
  - 繰り返し仕様 `release-workflow` の同期分岐が PR ベース化の影響を受ける。

## Requirements

### Requirement 1: 権威ソース（workflow.md）のPRベース・ブランチ戦略への改訂

**Objective:** As a kiro ワークフローのメンテナ, I want `workflow.md` のリモート同期セクションを PR ベースのブランチ戦略として単一の権威定義に書き換えたい, so that 3スキルがルールを複製せず一貫した PR 運用に従える。

#### Acceptance Criteria

1. The ワークフロー権威ソース shall リモート同期セクションを「commit → PR 作成 → PR の squash マージ → ブランチ削除」という PR ベースのフローとして定義する。
2. The ワークフロー権威ソース shall デフォルトブランチ（main）への直接 push を指示する記述（「直接 push」注記を含む）を含まない。
3. The ワークフロー権威ソース shall 手作業の `merge-base` squash・fast-forward マージ・squash ブランチ生成の手順を含まない。
4. While 各スキルがブランチ戦略を参照するとき, the ワークフロー権威ソース shall ルールの単一の定義元として機能し、各スキルはこれを複製せず参照する。
5. Where squash マージ時のコミットメッセージ生成方針が必要な場合, the ワークフロー権威ソース shall PR の squash マージ文脈に整合した生成方針を定義する。

### Requirement 2: kiro-spec-complete のPRベース完了フロー

**Objective:** As a 開発者, I want `kiro-spec-complete` が手作業 squash-ff-push の代わりに PR を作成して squash マージで完了させてほしい, so that main の履歴がクリーンに保たれ、ハーネスの運用方針と整合する。

#### Acceptance Criteria

1. When kiro-spec-complete が完了フローのリモート同期段階に到達したとき, the kiro-spec-complete スキル shall 現在の feature ブランチから PR を作成し、squash 方式でマージする。
2. The kiro-spec-complete スキル shall デフォルトブランチへの直接 push および手作業 squash-ff-push 儀式を実行しない。
3. When PR の squash マージが成功したとき, the kiro-spec-complete スキル shall 当該 feature ブランチをローカルおよびリモートから削除する。
4. If PR の作成またはマージが失敗した（コンフリクト／マージ不可／権限不足等）とき, then the kiro-spec-complete スキル shall ブランチを削除せず処理を中断し、開発者へ報告する。
5. Where 対象が繰り返し仕様（release-workflow 等）の場合, the kiro-spec-complete スキル shall `completed/` への移動をスキップしつつ、リモート同期を PR ベースで実行する。

### Requirement 3: kiro-tasks の簡素化（中間統合とimplブランチの撤去）

**Objective:** As a 開発者, I want `kiro-tasks` が tasks 生成とコミットまでで完結し、中間の main 統合や別 impl ブランチを作らないでほしい, so that 1つの feature = 1つのブランチ = 1つの PR を維持できる。

#### Acceptance Criteria

1. The kiro-tasks スキル shall tasks.md の生成とコミットまでで処理を完結する。
2. The kiro-tasks スキル shall `merge-base` からの squash 統合（現 Step 5 のデフォルトブランチへの中間統合）を実行しない。
3. The kiro-tasks スキル shall `impl/{feature}` 別ブランチ（現 Step 6）を生成しない。
4. The kiro-tasks スキル shall planning（requirements/design/tasks）と impl を、ハーネスが供給する単一の作業ブランチ上で継続させ、新規ブランチを切らない。
5. When 中間統合・impl ブランチ生成を撤去したとき, the kiro-tasks スキル shall Output / Constraints / Safety の記述を新フローと矛盾しないよう整合させる。

### Requirement 4: kiro-start のハーネスワークツリー委譲モデルへの整合

**Objective:** As a 開発者, I want `kiro-start` がフィーチャーブランチ生成を Claude Code（ハーネス）のワークツリー機能へ委譲し、自前でブランチを切らず push もしないでほしい, so that ハーネスのワークツリー前提と衝突しない。

#### Acceptance Criteria

1. The kiro-start スキル shall フィーチャーブランチおよびワークツリーの作成を Claude Code（ハーネス）のワークツリー機能へ委譲し、自前でブランチ／ワークツリーを作成しない。
2. The kiro-start スキル shall フィーチャーブランチ（`feat/{feature}`）の自動生成ロジックを撤去する。
3. The kiro-start スキル shall spec 初期化フェーズで push を実行しない。
4. While 現在のブランチがデフォルトブランチのとき, the kiro-start スキル shall フィーチャーブランチを作成せず、ハーネスが供給する非デフォルトの作業ブランチ（ワークツリー）上での実行を促す旨を警告して継続する。

### Requirement 5: kiro-impl の互換性維持

**Objective:** As a 開発者, I want `kiro-impl` が単一 feature ブランチ上での commit のみで従来通り動作することを保証してほしい, so that 改修後も実装フェーズが破綻しない。

#### Acceptance Criteria

1. The kiro-impl スキル shall 現在のブランチへの commit のみを行い、ブランチ生成や push を新たに導入しない。
2. Where 単一 feature ブランチ運用を明示することが有用な場合, the kiro-impl スキル shall その旨の注記のみを追加し、機能の振る舞いは変更しない。

### Requirement 6: GitHubリポジトリのsquash限定マージ強制

**Objective:** As a メンテナ, I want GitHub リポジトリのマージ方式を squash 限定へ設定したい, so that `--squash` の付け忘れや手動マージ事故を構造的に防止できる。

#### Acceptance Criteria

1. The GitHub リポジトリ設定タスク shall リポジトリのマージ方式について squash マージのみを有効化し、merge commit と rebase マージを無効化する。
2. The GitHub リポジトリ設定タスク shall 一度きりの設定作業として独立したタスクに分離される。
3. Where 追加の強制が望ましい場合, the GitHub リポジトリ設定タスク shall squash コミットメッセージ既定形や branch ruleset（PR 必須 / linear history）の併用を任意の選択肢として提示する。

### Requirement 7: 移植性とフォールバックの維持

**Objective:** As a メンテナ, I want 改修後も kiro スキル群の移植性とオフライン時のフォールバックが維持されてほしい, so that 他リポジトリでも安全に動作する。

#### Acceptance Criteria

1. The kiro スキル群 shall specs root / skill base / remote / default branch の決定的解決（deterministic resolution）ロジックを維持する。
2. If `{remote}` が不在またはオフラインのとき, then the kiro スキル群 shall 警告して処理を継続するフォールバックを維持し、PR 関連操作を安全にスキップする。
3. The kiro スキル群 shall 破壊的 git 操作（`reset --hard` / `revert` 等）の禁止方針と整合する。
4. The kiro スキル群 shall ブランチ削除を PR マージ成功後に限定し、マージ成功前にブランチを削除しない原則を維持する。
