# Implementation Plan

- [ ] 1. 権威ソース（workflow.md §3）の PR ベース改訂
- [x] 1.1 workflow.md §3「リモート同期（ブランチ戦略）」を PR ベースのブランチ戦略へ全面改訂
  - 「commit → gh pr create → gh pr merge --squash --delete-branch → ブランチ削除」を唯一の手順実体として定義
  - 「直接 push」注記、merge-base squash、--ff-only、squash/<A> 生成手順を撤去
  - squash コミットメッセージ生成方針を PR squash 文脈（--subject/--body、merge-base..HEAD 履歴＋spec タイトル要約）へ移植
  - release タグ公開カーブアウト（git push origin main --tags は §3 の禁止対象外）を明記（DD5）
  - 破壊的 git 操作禁止と整合、ブランチ削除は PR マージ成功後に限定
  - DoD（§6 Manual Sync Gate 含む）の意味・順序は不変に保つ
  - 完了状態: §3 に PR フロー定義が存在し「直接 push」記述が無い
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 7.3, 7.4_
  - _Boundary: workflow.md §3_

- [ ] 2. kiro-complete の PR 化と決定的解決
- [x] 2.1 kiro-spec-complete に Step 0（決定的解決）を導入
  - {remote}（origin→単一→none）と {default-branch}（symbolic-ref→main→master→現ブランチ）を固定優先順序で解決
  - origin/main のハードコードを撤去
  - 完了状態: kiro-spec-complete のリモート操作が origin/main ハードコードに依存しない
  - _Requirements: 7.1_
  - _Boundary: kiro-complete_
  - _Depends: 1.1_
- [x] 2.2 kiro-spec-complete の Step 8 を PR ベース完了フローへ置換
  - gh pr create --base {default-branch} --head {current} → gh pr merge --squash --delete-branch --subject --body
  - マージ成否（API 結果）とローカル後始末警告を分離（--delete-branch のローカル削除警告は非致命で継続）
  - 直接 push・手作業 squash-ff-push を撤去
  - PR 作成/マージ（API）失敗時はブランチを残し中断・報告
  - 繰り返し仕様（release-workflow 等）は completed/ 移動スキップ＋PR ベース同期
  - default ブランチ上/PR 不可（remote none/未認証）時は警告して push スキップ、ローカルコミット保持
  - 完了チェックリスト・エラー回避節を PR ベースへ整合
  - 完了状態: kiro-spec-complete が gh pr create/merge --squash で main 反映、直接 push 経路が無い
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 7.2, 7.4_
  - _Boundary: kiro-complete_
  - _Depends: 1.1, 2.1_

- [ ] 3. kiro-complete へのリネームと参照更新
- [ ] 3.1 kiro-spec-complete スキルを kiro-complete へリネーム
  - ディレクトリ .claude/skills/kiro-spec-complete/ → .claude/skills/kiro-complete/
  - frontmatter name: kiro-spec-complete → kiro-complete
  - 後方互換エイリアスを設けない、完了ワークフローの振る舞いは不変
  - 完了状態: .claude/skills/kiro-complete/ が存在し kiro-spec-complete/ が不在、/kiro-complete で起動
  - _Requirements: 8.1, 8.2, 8.4, 8.5_
  - _Boundary: kiro-complete_
  - _Depends: 2.2_
- [ ] 3.2 kiro-spec-complete への全参照を kiro-complete へ更新
  - book/tools/verify-drift-gate.mjs L239 のパス要素を kiro-complete へ更新
  - workflow.md・関連スキルの kiro-spec-complete 参照を更新（CLAUDE.md は名指し参照なしを確認）
  - 完了状態: grep kiro-spec-complete が運用ドキュメント/verify-drift-gate.mjs にヒットしない
  - _Requirements: 8.3_
  - _Boundary: verify-drift-gate.mjs, workflow.md, 関連スキル_
  - _Depends: 3.1_

- [ ] 4. kiro-tasks スキルの撤去と参照整理
- [ ] 4.1 kiro-tasks スキルディレクトリの撤去と参照整理
  - .claude/skills/kiro-tasks/ を削除
  - workflow.md・関連スキルの kiro-tasks 参照を撤去後運用へ整合（タスク生成は /kiro-spec-tasks -y 直接実行）
  - squash 統合（旧 Step 5）・impl/{feature} 生成（旧 Step 6）を他スキルへ再導入しない
  - 完了状態: .claude/skills/kiro-tasks/ が不在、grep kiro-tasks が運用ドキュメントにヒットしない
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Boundary: kiro-tasks, workflow.md_
  - _Depends: 1.1_

- [ ] 5. kiro-start のハーネスワークツリー委譲整合
- [ ] 5.1 (P) kiro-start の feat ブランチ生成撤去と default 上 STOP
  - feat/{feature} 自動生成ロジック（Step 2）を撤去、自前のブランチ/ワークツリー作成をしない
  - 非デフォルト（ハーネス作業ブランチ）上では spec 初期化を実行し現在ブランチへ commit、push しない
  - デフォルトブランチ上実行時は STOP し、ワークツリーでの再実行を促す
  - frontmatter description・Constraints・Output・Safety を整合
  - 完了状態: kiro-start が feat 生成せず、default 上で STOP、非 default 上で commit・no-push
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 7.2_
  - _Boundary: kiro-start_
  - _Depends: 1.1_

- [ ] 6. kiro-impl 互換注記
- [ ] 6.1 (P) kiro-impl に単一 feature ブランチ互換注記を追加
  - 「ハーネス供給の単一作業ブランチ上で動作、commit のみ、ブランチ生成・push を導入しない」旨の注記
  - tasks.md 未コミット時は git add -A で取り込む旨（振る舞い不変）
  - 完了状態: kiro-impl/SKILL.md に注記が存在し、振る舞いは変更されていない
  - _Requirements: 5.1, 5.2, 3.5, 3.6_
  - _Boundary: kiro-impl_

- [ ] 7. GitHub リポジトリの squash 限定設定（一度きり）
- [ ] 7.1 (P) gh repo edit でマージ方式を squash 限定化
  - gh repo edit ekicyou/pasta --enable-squash-merge --enable-merge-commit=false --enable-rebase-merge=false --delete-branch-on-merge
  - 任意強制（squash メッセージ既定形、branch ruleset: PR 必須/linear history）は選択肢として提示し既定では有効化しない
  - 完了状態: gh repo view が squashMergeAllowed のみ true、deleteBranchOnMerge true を返す
  - _Requirements: 6.1, 6.2, 6.3_
  - _Boundary: GitHub 設定タスク_

- [ ] 8. 検証（静的整合・ツール・dry-run）
- [ ] 8.1 静的整合チェックと verify-drift-gate.mjs 通過
  - grep: 旧名（kiro-tasks / kiro-spec-complete）が運用ドキュメントに残らない、workflow.md §3 に「直接 push」「merge-base squash」「--ff-only」手順が無く PR フロー定義が存在
  - node book/tools/verify-drift-gate.mjs が成功（kiro-complete/SKILL.md パス解決、workflow.md DoD 結線アサート）
  - 完了状態: grep がクリーン、verify-drift-gate.mjs が exit 0
  - _Requirements: 1.1, 1.2, 3.1, 8.1, 8.3_
  - _Depends: 3.2, 4.1_
- [ ] 8.2 PR フロー dry-run と GitHub 設定確認
  - 使い捨てブランチで gh pr create → gh pr merge --squash --delete-branch を実行し、squash 1 コミットで main 反映・リモートブランチ削除を確認
  - {remote} を一時的に none とした環境で kiro-complete が警告継続・push スキップすることを確認
  - gh repo view --json で squash のみ true・deleteBranchOnMerge true を確認
  - 完了状態: dry-run で squash マージ成立・リモートブランチ削除・フォールバック動作を観測
  - _Requirements: 2.1, 2.3, 2.6, 6.1, 7.2_
  - _Depends: 2.2, 7.1_

- [ ] 9. ドキュメント整合性の確認と更新（必須・最終）
- [ ] 9.1 ドキュメント整合性の確認と更新
  - SOUL.md: コアバリュー影響なし（git ワークフロー変更、N/A 確認）
  - doc/spec/・GRAMMAR.md・TEST_COVERAGE.md・クレートREADME: DSL/Lua/API 非変更（N/A）
  - steering/*: workflow.md 改訂済み（本体）。structure.md 等に kiro-tasks/kiro-spec-complete 言及がないことを確認
  - CLAUDE.md: Minimal Workflow に kiro-tasks 撤去・kiro-complete リネームの影響がないか確認、必要なら最小追記
  - pasta-ghost-authoring / pasta-lua-coding: DSL/Lua 非変更（N/A）
  - 完了状態: 全ドキュメントの整合確認完了、該当箇所更新済み
  - _Requirements: 3.3, 8.3_

## Implementation Notes

- 2.2: kiro-spec-complete の frontmatter `description` 末尾が旧表現「…最終コミット→push まで」のまま。Task 3.1（frontmatter 編集）で PR squash マージ文脈へ更新すること。
