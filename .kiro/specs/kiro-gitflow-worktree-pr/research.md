# Gap Analysis: kiro-gitflow-worktree-pr

> 既存コードベース（kiro スキル群 + steering）への改修。本仕様は「ドキュメント／スキル定義の prose 改訂」と「一度きりの GitHub 設定変更」が実体であり、ランタイムコードの変更を含まない。

## 1. 現状調査（Current State Investigation）

### 対象アセットとレイアウト

| アセット | パス | 役割 | 現状の git 儀式 |
|---|---|---|---|
| 権威ソース | `.kiro/steering/workflow.md` §3「リモート同期（ブランチ戦略）」 | ブランチ戦略の**単一定義元** | main 直 push ＋ 手作業 `merge-base` squash → `--ff-only` → push → A/B 削除。squash メッセージ生成方針もここに定義（L83–139） |
| スキル | `.claude/skills/kiro-start/SKILL.md` | 要件フェーズ開始（branch + init + requirements） | Step 2 で default-branch 上のとき `feat/{feature}` を作成。**push なし**（pull のみ）。Step 5 で branch 作成時のみ commit |
| スキル | `.claude/skills/kiro-tasks/SKILL.md` | tasks 生成 + 中間統合 + impl ブランチ | Step 5 で squash 統合（`$branchA-squash` を merge-base から作成 →`--no-ff` で default を取り込み → default を ff → push → A/B 削除）。Step 6 で `impl/{feature}` をローカル作成 |
| スキル | `.claude/skills/kiro-spec-complete/SKILL.md` | 完了アーカイブ + 最終 push | Step 8 で `squash/$branchA`（= ブランチB）を `origin/main` から作成 →`--ff-only` → `git push origin main` → A/B 削除。繰り返し仕様は Step 2+8+tasks リセットのみ |
| スキル | `.claude/skills/kiro-impl/SKILL.md` | 実装（現ブランチ commit のみ） | git 言及は「破壊的 reset 禁止」のみ。branch/push ロジックなし |

### 抽出した規約・パターン（横展開の必要あり）

- **決定的解決（Step 0）**: `kiro-start` / `kiro-tasks` は固定優先順で `{specs-root}`（`.kiro/specs`）/ `{skill-base}`（`.claude/skills`→`.agents/skills`→`.github/skills`）/ `{remote}`（`origin`優先）/ `{default-branch}`（`symbolic-ref`→`main`→`master`→現ブランチ）を解決。**この移植性パターンは維持必須**（Req 7.1）。`kiro-spec-complete` には Step 0 が**無く**、`origin`/`main` をハードコードしている（L191, L199 等）— ここが移植性の既存ギャップ。
- **軽量オーケストレーション**: `kiro-start` / `kiro-tasks` は「コントローラは git のみ、重い生成はサブエージェント」という分業。git 手順撤去後もこの構造は保てる。
- **フォールバック規約**: `{remote}` = none / pull・push 失敗時は「warn して継続」。ブランチ削除は push 成功後のみ（Req 7.2, 7.4）。
- **禁止事項**: workflow.md §禁止事項に `reset --hard` / `revert` / `checkout --` / `clean -fd` の破壊的操作禁止（Req 7.3）。

### リポジトリ現況（新鮮な証拠 / 2026-06-13 実測）

- `gh auth status`: **ekicyou としてログイン済み**（`gho_…` トークン、keyring）。PR 操作の前提を満たす。
- `gh repo view ekicyou/pasta`: `mergeCommitAllowed: true` / `squashMergeAllowed: true` / `rebaseMergeAllowed: true` → **3方式すべて許可**（brief と一致、squash 限定化は未着手）。
- `deleteBranchOnMerge: false` → **PR マージ時のブランチ自動削除は無効**。Req 2.3（マージ成功後のブランチ削除）の実現手段として、`gh pr merge --squash --delete-branch` か `--enable-delete-branch-on-merge` 設定の二択がある（設計判断）。
- ⚠️ `git remote` URL にトークン平文埋め込み（brief 記載）— **本仕様スコープ外**だが要対応。

## 2. 要件→アセット対応マップ（Requirement-to-Asset Map）

| Req | 触れるアセット | 変更種別 | ギャップ種別 |
|---|---|---|---|
| **1** workflow.md PR 化 | `workflow.md` §3 全面改訂（L83–139） | 既存 prose 置換 | **Constraint**: 権威ソース。3スキルが参照する単一定義元なので**最初に確定**する必要 |
| **2** spec-complete PR 化 | `kiro-spec-complete` Step 8（L184–250）/ 繰り返し分岐（L49–62）/ チェックリスト（L267–270）/ エラー回避（L292–310） | Step 8 を `gh pr create`+`gh pr merge --squash` へ置換 | **Constraint**: Step 0 不在（`origin`/`main` ハードコード）。移植性のため決定的解決の導入も要検討。**Unknown**: ブランチ削除手段（`--delete-branch` vs repo 設定） |
| **3** kiro-tasks 撤去 | `kiro-tasks/` スキルディレクトリ全体を削除（ディスカッション #3 で「簡素化」→「撤去」へ変更）。CLAUDE.md・`workflow.md`・関連スキルの `kiro-tasks` 参照を grep して整理 | スキル削除 + 参照整理 | **Constraint**: タスク生成は `/kiro-spec-tasks -y` 直接実行へ移行。tasks.md commit は kiro-impl / kiro-spec-complete が拾う。**Research Needed**: `kiro-tasks` 参照箇所の網羅（grep `kiro-tasks`） |
| **4** kiro-start 整合 | `kiro-start` Step 2（L65–87）/ frontmatter description（L3）/ Constraints（L144） | branch 方針の注記更新 | **Unknown（要設計判断）**: ワークツリー委譲下で `feat/{feature}` を**今後も作るのか**。1 feature=1 branch の起点をどこが用意するか |
| **5** kiro-impl 注記 | `kiro-impl/SKILL.md` | 注記追加のみ | **Low**: 機能変更なし |
| **6** GitHub squash 限定 | リポジトリ設定（コードでなく構成） | `gh repo edit` タスク化 | **Missing**: 設定作業そのものは未実施。タスク化が必要 |
| **7** 移植性・フォールバック | 全スキル横断 | 既存規約の保全 | **Constraint**: 改訂で壊さないことの検証が必要。特に `kiro-spec-complete` への Step 0 導入是非 |

### 主要なギャップ要約

- **Missing（新規作成）**: GitHub リポジトリの squash 限定設定作業（Req 6）。これだけが「新しい行為」で、他はすべて既存 prose の改訂。
- **Constraint（既存制約）**: workflow.md が権威ソース＝**改訂順序に依存**（workflow.md → 3スキルの順でないと不整合が残る）。`kiro-spec-complete` の Step 0 不在という既存の移植性ギャップ。
- **Unknown / Research Needed**:
  - ~~**U1**: ワークツリー委譲下で `kiro-start` が `feat/{feature}` ブランチを引き続き作るのか~~ → **解決済み（要件ディスカッション #1）**: 完全ハーネス委譲。kiro スキルはブランチ／ワークツリーを自前で作らず、Claude Code のワークツリー機能が非デフォルト作業ブランチを供給する前提。`kiro-start` の `feat/{feature}` 自動生成は撤去（Req 4.1/4.2 更新）。default ブランチ上で走らせた場合は警告して継続。
  - **U2**: PR マージ後のブランチ削除を `gh pr merge --delete-branch` で行うか、リポジトリ `deleteBranchOnMerge` 設定で行うか（Req 2.3 / Req 6 の連動）。
  - **U3**: `kiro-spec-complete` に決定的解決（Step 0）を新規導入して移植性を `kiro-tasks` と揃えるか、最小改訂に留めるか（Req 7.1 のスコープ）。
  - ~~**U4**: 繰り返し仕様（release-workflow）が default-branch 上で走る場合の PR フロー~~ → **解決済み（要件ディスカッション #2）**: `kiro-start` は default ブランチ上なら**中断（STOP）**。`kiro-spec-complete` は PR 動作可能なら警告して継続し PR squash マージ、default 上／PR 不可時は**警告して push をスキップ**（main 直 push は一切しない、Req 2.6 追加）。繰り返し仕様も例外にせず PR 化（completed/ 移動はしない）。

## 3. 実装アプローチ選択肢（Options）

### Option A: 権威ソース先行・各スキル最小改訂（Extend）
**When**: 既存の参照関係（workflow.md＝権威、スキル＝参照）を最大限活かす。
- workflow.md §3 を PR ベースへ全面改訂し、3スキルは「workflow.md に従う」記述に寄せて手順実体を削る。
- `kiro-tasks` は Step 5/6 を削除、`kiro-spec-complete` は Step 8 を PR 手順へ差し替え（最小）、`kiro-start` は注記のみ。
- **Trade-offs**: ✅ 変更量最小・参照設計思想に忠実 ✅ 権威ソース単一性を強化 ❌ `kiro-spec-complete` の `origin`/`main` ハードコードと Step 0 不在は残置（移植性ギャップ温存）❌ U1/U4 の設計判断は別途必要。

### Option B: 権威ソース改訂 + 全スキルの決定的解決統一（Hybrid）
**When**: この機にスキル間の移植性を揃える。
- Option A に加え、`kiro-spec-complete` へ Step 0（決定的解決）を新規導入し、`{remote}`/`{default-branch}` をハードコードから脱却。PR 作成も `{remote}` ベースに。
- **Trade-offs**: ✅ 3スキルの移植性が一貫（Req 7.1 を積極的に満たす）✅ PR フローが `{default-branch}` 非依存で堅牢 ❌ `kiro-spec-complete` の変更量増（Step 0 追加 + 既存 PowerShell 手順の刷新）❌ 既存挙動への回帰リスクがやや上がる。

### Option C: PR ヘルパーの共通リファレンス化（New）
**When**: PR 作成→squash マージ→ブランチ削除の手順を、3スキルで重複させず1箇所に集約したい。
- workflow.md §3 に「PR ベース統合の標準手順」を**唯一の実体**として置き、スキルは番号付き手順を持たず参照だけにする（`kiro-spec-complete` のチェックリストも参照へ）。
- 必要なら `references/` に PR 手順スニペットを切り出し（ただし本リポジトリのスキルは現状 `references/` 不使用）。
- **Trade-offs**: ✅ DRY 徹底・将来のレビュー体制 spec（PR テンプレ等）拡張に強い ✅ 重複ドリフト防止 ❌ スキル単体の自己完結性が下がる（読み手が workflow.md を往復）❌ 既存スキルの自己完結スタイルから逸脱。

## 4. 工数・リスク（Effort & Risk）

| 項目 | 評価 | 根拠（1行） |
|---|---|---|
| **Effort** | **S（1–3日）** | 実体は markdown 4ファイルの prose 改訂 + 一度きりの `gh repo edit`。ランタイムコード・テスト変更なし |
| **Risk** | **Medium** | 個々の編集は低リスクだが、(a) 権威ソースとスキルの**整合**を崩すと運用が壊れる、(b) U1/U4 の設計判断未定、(c) git フロー誤記は実行時に初めて顕在化し検証が手動依存。回帰検出が prose レビュー頼り |

## 5. 設計フェーズへの推奨（Recommendations）

### 推奨アプローチ
**Option B（権威ソース改訂 + `kiro-spec-complete` への決定的解決統一）を軸に、Option C の「workflow.md を唯一の手順実体に」要素を部分採用**するのが、Req 1（単一定義元）・Req 7（移植性）を同時に満たし保守コストを下げる。改訂順序は厳守：**① workflow.md §3 → ② kiro-spec-complete（PR 化）→ ③ kiro-spec-complete → kiro-complete へリネーム＋全参照更新（ディスカッション #4）→ ④ kiro-tasks 撤去＋参照整理 → ⑤ kiro-start（feat ブランチ生成撤去・default 上 STOP）→ ⑥ kiro-impl 注記 → ⑦ GitHub 設定タスク**。

> **リネーム注記（#4）**: `kiro-spec-complete` → `kiro-complete`。ライフサイクル入口（`kiro-start`/`kiro-impl`/`kiro-complete`）の命名統一が狙い。後方互換エイリアスなし＝クリーン置換。参照は `grep -r kiro-spec-complete` で網羅（CLAUDE.md・workflow.md・sibling skills・registry 説明）。**Research Needed**: 参照箇所の完全列挙。PR 化（②）とリネーム（③）は同一スキルを触るため、PR 化を先に完了させてからディレクトリ名変更を行うと差分が読みやすい。

### 設計で決すべき判断（Carry-forward）
- **U1**: `kiro-start` は `feat/{feature}` を作り続けるか／ワークツリー前提に委ねるか。→ 「1 feature=1 branch=1 PR」の branch 起点を誰が用意するかを設計で確定（Req 3.4 / Req 4 と直結）。
- **U2**: ブランチ削除手段（`gh pr merge --squash --delete-branch` 推奨 vs `deleteBranchOnMerge` 設定）。現状 repo 設定は `false`。Req 6 で設定変更するなら両者の役割分担を明記。
- **U3**: `kiro-spec-complete` への Step 0 導入範囲（フル決定的解決 vs `{remote}`/`{default-branch}` だけ解決）。
- **U4**: 繰り返し仕様＋default-branch 上での PR フロー定義（default 上では PR を作らず警告して継続、等のフォールバック）。
- **squash メッセージ生成方針**（Req 1.5）: `gh pr merge --squash` のコミットメッセージをどう供給するか（`--subject`/`--body` 明示 vs PR 本文流用）。
- **検証戦略**: ランタイムテストが無いため、設計で「PR フローの dry-run / 手動受け入れ手順」を Research Needed として明記推奨。

### 持ち越す調査項目（Research Needed）
- `gh pr create` / `gh pr merge --squash` の最小オプションセットと、`{remote}` 不在時のスキップ挙動の標準化。
- branch ruleset（PR 必須 / linear history）併用の要否（Req 6.3、任意）。

---

## 設計合成（Synthesis）— 2026-06-13

### 設計判断の確定（DD1–DD5、design.md と同期）
- **DD1 (U2)**: ブランチ削除＝`gh pr merge --squash --delete-branch`（リモート）＋ローカル後始末はハーネス委譲。多重防御で repo `--delete-branch-on-merge` も有効化。
- **DD2 (U3)**: `kiro-complete` に Step 0（決定的解決）を新規導入。`origin`/`main` ハードコード撤去。kiro-tasks 撤去で kiro-complete が主要 git-ops スキルになるため移植性が必須化。
- **DD3 (Req 1.5)**: squash メッセージは `gh pr merge --subject/--body` で供給。本文は `merge-base..HEAD` 履歴＋ spec タイトル要約。方針は workflow.md、実行は kiro-complete。
- **DD4 (検証)**: 静的整合チェック＋`verify-drift-gate.mjs`＋使い捨てブランチ dry-run の3層。ランタイムテストは非該当。
- **DD5 (境界)**: workflow.md §3 を「spec 完了ブランチ統合」に限定し、release タグ公開（`git push origin main --tags`）を禁止対象外と明記（カーブアウト）。

### 参照インベントリ（grep 確定。Req 3.3 / 8.3）
| ファイル | 参照 | 対応 |
|---|---|---|
| `.kiro/steering/workflow.md` | §3 全体（直接 push・squash 儀式・メッセージ方針） | **改訂**（権威） |
| `.claude/skills/kiro-tasks/SKILL.md` | スキル本体 | **削除** |
| `.claude/skills/kiro-spec-complete/SKILL.md` | スキル本体・name | **リネーム＋PR 化** → `kiro-complete/` |
| `book/tools/verify-drift-gate.mjs` L239 | `.claude/skills/kiro-spec-complete/SKILL.md` パスをハードコード | **`kiro-complete` へ更新**（更新しないとテスト失敗） |
| `CLAUDE.md` | `/kiro-tasks`・`/kiro-spec-complete` の名指し参照 | **なし**（grep 確認済み。新フロー注記は任意） |
| `.claude/settings.json` L4 | `Bash(git push origin main:*)` 許可 | **保持**（release タグ push に必要。Out of Boundary） |
| `.kiro/specs/release-workflow/{design,tasks}.md` | `git push origin main --tags`「workflow.md 準拠」 | **保持**（DD5 カーブアウトで引用を有効化。本 spec では編集しない） |
| `.kiro/specs/review-improvement-loop/*`, `.kiro/specs/completed/**` | 記述的言及 | **対象外**（運用依存なし。Out of Boundary） |
