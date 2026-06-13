# Brief: kiro-gitflow-worktree-pr

## Problem
kiro ワークフローの `kiro-start` / `kiro-tasks` / `kiro-spec-complete` の3スキルと、その権威ソースである `.kiro/steering/workflow.md` は、いずれも「ローカルでブランチを切り、`merge-base` から squash ブランチを作って ff マージし、main へ**直接 push** する」という独自の git 統合儀式を前提に書かれている。

現在の Claude Code（ハーネス）は「**ワークツリーに隔離して作業し、最後に PR でマージする**」運用を前提とした設計になっており、両者の前提が衝突している。手作業の squash-ff-push ロジックは複雑で保守コストが高く、main への直 push はハーネスの「push は明示要求時のみ／デフォルトブランチ上ではまずブランチを切る」という方針とも噛み合わない。

## Current State
- **kiro-start**: デフォルトブランチ上のとき `feat/{feature}` を作成し、spec.json + requirements.md を commit。push なし。
- **kiro-tasks**: tasks.md を生成・commit 後、（非デフォルトブランチなら）Step 5 で「squash ブランチ B を merge-base から作成 → main を B へマージ → main を B に ff → push → A/B 削除」を実行し、さらに Step 6 で別ブランチ `impl/{feature}` を作成。＝**feature ライフサイクル中に main へ2回統合**する設計。
- **kiro-spec-complete**: DoD ゲート → commit → completed/ へ移動 → spec.json 更新 → ロードマップ/ドキュメント同期 → 最終 commit の後、Step 8 で `kiro-tasks` と同種の手作業 squash-ff-push を実行して main へ直接反映。
- **workflow.md §3「リモート同期（ブランチ戦略）」**: 上記儀式の権威定義。「完了フローは main へ**直接 push** する（PR は経由しない）」と明記されている。
- **kiro-impl**: ブランチ/ push ロジックは持たず、現在のブランチに commit するのみ（git 言及は「破壊的 reset 禁止」制約だけ）。

### リポジトリ現況（調査済み）
- `origin` → `github.com/ekicyou/pasta`、デフォルトブランチ `main`、`gh` CLI 利用可（認証済み）。
- マージ方式: **merge commit / rebase / squash の3方式すべて許可**（squash 限定ではない）。
- ⚠️ `git remote` の URL に `github_pat_...` トークンが平文埋め込み（別途是正推奨。本 spec の対象外だが要対応）。

## Desired Outcome
- 3スキル＋ workflow.md から手作業の squash-ff-push 儀式と main 直 push を撤去する。
- 作業はハーネス/セッション管理のワークツリーで隔離され、スキルは「commit → PR 作成 → `gh pr merge --squash`」だけを担う。
- 1つの feature = 1つのブランチ = 1つの PR となり、PR 単位で squash マージされ、main の履歴がクリーンに保たれる。
- GitHub リポジトリ設定で squash 限定を強制し、`--squash` 付け忘れや手動マージ事故を構造的に防止する。

## Approach
ディスカッションで確定した4つの方針：

1. **ワークツリー管理 = ハーネス/セッションに委譲**: スキルは `git worktree` を自前で切らない。ブランチ管理ロジックを撤去し、commit + PR だけを行う。「現行 Claude Code の前提に合わせる」最も忠実な形。
2. **PR 粒度 = feature 全体で1PR**: planning（requirements/design/tasks）と impl を**同一ブランチに積み**、`kiro-spec-complete` で1回だけ PR → squash マージ。`kiro-tasks` の中間 main 統合と `impl/{feature}` 別ブランチ生成は撤去。
3. **マージ実行 = スキルが自動**: `kiro-spec-complete` が `gh pr create` → `gh pr merge --squash`（ブランチ削除込み）で完結。現行の自律性を維持しつつ、機構を PR 経由へ置換。
4. **GitHub 設定 = squash 限定に強制**: `gh repo edit ekicyou/pasta --enable-squash-merge --enable-merge-commit=false --enable-rebase-merge=false`（一度きり）。任意で squash コミットメッセージ既定形と branch ruleset（PR 必須 / linear history）も併用可。

## Scope
- **In**:
  - `kiro-spec-complete` Step 8 の手作業 squash-ff-push を PR ベース（`gh pr create` + `gh pr merge --squash` + ブランチ削除）へ置換。繰り返し仕様（release-workflow 等）の同期分岐も PR ベースへ更新。
  - `kiro-tasks` の Step 5（squash 統合）と Step 6（`impl/{feature}` 生成）を撤去。tasks 生成 + commit までに簡素化。Output/Constraints/Safety 記述も整合。
  - `kiro-start` のブランチ生成方針をワークツリー委譲モデルへ整合（main 直作業の扱い、push しない方針の明記）。
  - `.kiro/steering/workflow.md` §3「リモート同期（ブランチ戦略）」と「直 push」注記・squash メッセージ生成方針を PR ベースへ全面改訂（権威ソースを最初に確定）。
  - GitHub リポジトリのマージ方式を squash 限定に変更する作業（`gh repo edit`、一度きり。タスク化）。
  - `kiro-impl` は現状ブランチ commit のみで互換。必要なら「単一 feature ブランチ上で動く」旨の注記のみ。
- **Out**:
  - kiro スキル群以外（CI/CD、他リポジトリ）の git 運用。
  - `git remote` URL のトークン是正（別件として推奨するが本 spec では扱わない）。
  - PR テンプレート整備や CODEOWNERS 等のレビュー体制構築（将来の別 spec 候補）。
  - DSL / Lua / pasta 本体の機能変更。

## Boundary Candidates
- 権威ソース層: `workflow.md`（ブランチ戦略の単一定義）— ここを先に確定し、3スキルはこれに従う。
- スキル・オーケストレーション層: `kiro-start` / `kiro-tasks` / `kiro-spec-complete` の git 手順記述。
- インフラ設定層: GitHub リポジトリのマージ方式設定（`gh repo edit`、コードではなく構成）。

## Out of Boundary
- ハーネス側のワークツリー機能そのものの実装・改変（委譲先であり、本 spec はスキル側の整合のみ）。
- main への push 権限や認証情報（PAT）の管理ポリシー。

## Upstream / Downstream
- **Upstream**: Claude Code ハーネスのワークツリー隔離・PR 運用前提。`gh` CLI と GitHub リポジトリ設定。
- **Downstream**: `kiro-impl`（単一 feature ブランチ上で動作）、将来のレビュー体制 spec（PR テンプレート / branch ruleset 強化）。

## Existing Spec Touchpoints
- **Extends**: なし（新規の単一スコープ改修）。
- **Adjacent**: `release-workflow`（繰り返し仕様。Step 8 の同期分岐が PR ベース化の影響を受ける）。`review-improvement-loop`（レビュー運用と将来的に関連しうる）。

## Constraints
- workflow.md は「権威ソース」。3スキルはルールを複製せず参照する設計思想を維持する。
- `gh` CLI と PR 作成権限が前提。`{remote}` 不在/オフライン時のフォールバック（warn して継続）方針は現行同様に維持。
- 破壊的 git 操作の禁止（workflow.md §禁止事項）と整合させる。push 成功前にブランチを削除しない原則を PR マージ後の削除に読み替える。
- スキルの移植性（specs root / skill base / remote / default branch の決定的解決）を損なわない。
