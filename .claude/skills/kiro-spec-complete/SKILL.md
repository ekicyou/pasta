---
name: kiro-spec-complete
description: 'Kiro仕様駆動開発のSpec完了ワークフローを実行する。DoDゲート検証→コミット→completedフォルダ移動→spec.json更新→参照パス修正→ロードマップ更新→スキルドキュメント同期→最終コミット→pushまでを中断なく完遂する。Use when: 実装完了を承認する, 承認してください, 完了を承認, spec承認, approve implementation, kiro承認完了。DO NOT USE when: 実装が完了したのみ（承認の明示がない場合）、タスクが終わっただけ'
argument-hint: <feature-name>
---

# Kiro Spec 完了ワークフロー

## 発動条件（必須）

> **⚠️ このスキルは開発者の明示的「承認」がある場合にのみ発動する。**

### ✅ 発動する（承認の明示がある）
- 「実装完了を**承認**します」「**承認**してください」
- 「このspecを**承認**する」「**approve**」「kiro **承認**完了」

### ❌ 発動しない（承認の明示がない）
- 「実装が完了した」「タスクが全部終わった」
- 「spec完了」「アーカイブしてほしい」などの曖昧な表現のみ
- AIが自律的に「完了したと判断」した場合

### 承認が不明瞭なとき
発動を迷う場合は **発動しない**。必要なら開発者に確認する:
> 「実装完了の承認をいただけますか？承認いただいた場合、完了ワークフローを実行します。」

---

## いつ使うか
- 開発者が上記「発動する」に該当する承認を**明示的に**宣言したとき
- tasks.md の全タスクが `[x]` 完了している状態で使用
- 設計文書リフレッシュが完了した後

## 権威的ソース

> **このスキルは `.kiro/steering/workflow.md` に従う。**
> 完了基準（DoD）、コミット規約、ドキュメント更新チェックリスト、スキルドキュメント更新判定はすべて workflow.md を参照すること。
> このスキルは workflow.md の「実装完了時アクション」を**自動オーケストレーション**するものであり、ルールの複製は行わない。

## 哲学
- **中断せず一連で完遂する** — 全ステップを止めずに実行
- **VSCodeの変更ファイル確定挙動を回避** — spec.json編集は移動後に行う
- **ステアリングが正** — DoD・コミット規約・禁止事項はすべて workflow.md に従う
- **繰り返し仕様は移動しない** — `release-workflow` 等は常に `.kiro/specs/` 直下に留まる

## 前提条件
- `.kiro/specs/{feature}/tasks.md` の全タスクが完了
- 設計文書と最終実装の整合確認済み

## 例外: 繰り返し仕様

`release-workflow` のような繰り返し実行型仕様は `completed/` に**移動しない**。

判定基準:
- spec.json や requirements.md に「繰り返し」「repeatable」「定期実行」等の記述がある
- `/kiro-spec-impl` のたびにタスクがリセットされる設計

繰り返し仕様の場合:
1. ステップ1（DoD検証）とステップ2（コミット）のみ実行
2. ステップ3〜5（移動・パス更新・ロードマップ）をスキップ（`completed/` へは移動しない）
3. ステップ8（リモート同期＝PR ベース）を実行
4. tasks.md のチェックボックスをリセット（全 `[x]` → `[ ]`）

---

## 手順

### ステップ0: 決定的解決（portable context）

リモート操作で用いる `{remote}` と `{default-branch}` を、**固定優先順序で1回だけ決定的に解決**する。各値はちょうど1つの結果（または明示的なスキップ）に収束させ、推測しない。解決した値は以降のステップ（特にステップ8）で `origin`/`main` のハードコードの代わりに再利用する。この優先順序は `kiro-start` の「Step 0: Resolve portable context」と整合している。

1. **デフォルトリモート（`{remote}`）**: `git remote` を実行し、以下の固定ルールを適用する。
   - `origin` が存在する → `{remote}` = `origin`。
   - そうでなく、リモートがちょうど1つだけ存在する → `{remote}` = そのリモート。
   - それ以外（リモートなし、または `origin` を含まない複数リモート）→ `{remote}` = none。リモート操作はすべてスキップ扱いとする（一度だけ警告する）。

   ```powershell
   $remotes = git remote
   if ($remotes -contains "origin") { $remote = "origin" }
   elseif (@($remotes).Count -eq 1) { $remote = $remotes }
   else { $remote = $null }  # none: リモート操作はスキップ
   ```

2. **デフォルトブランチ（`{default-branch}`）**: 以下の固定優先順序で決定的に解決する。
   - `{remote}` が解決済みなら、`git symbolic-ref --quiet --short refs/remotes/{remote}/HEAD` を読み、先頭の `"{remote}/"` プレフィックスを除去した名前。
   - それが空で、ローカルに `main` ブランチが存在する → `{default-branch}` = `main`。
   - そうでなく、ローカルに `master` ブランチが存在する → `{default-branch}` = `master`。
   - それ以外 → `{default-branch}` = 現在のブランチ。

   ```powershell
   $defaultBranch = $null
   if ($remote) {
     $defaultBranch = git symbolic-ref --quiet --short "refs/remotes/$remote/HEAD"
     if ($defaultBranch) { $defaultBranch = $defaultBranch -replace "^$remote/", "" }
   }
   if (-not $defaultBranch) {
     if (git show-ref --verify --quiet "refs/heads/main") { $defaultBranch = "main" }
     elseif (git show-ref --verify --quiet "refs/heads/master") { $defaultBranch = "master" }
     else { $defaultBranch = git branch --show-current }
   }
   ```

   `{default-branch}` は1つの具体的なブランチ名として確定し、以降で再評価しない。

> **以降のステップは、`origin`/`main` のハードコードではなく、ここで解決した `{remote}` / `{default-branch}` を用いる前提とする。** `{remote}` が none の場合、リモート同期（ステップ8）は安全にスキップし警告する。

### ステップ1: DoD（完了基準）ゲート検証

`.kiro/steering/workflow.md` の「完了基準（DoD）」セクションを読み込み、**workflow.md が定義する全ゲートを順に検証**する。判定ルールの本体は workflow.md（権威）にあり、このスキルは発火・オーケストレーションのみを行いルールを複製しない。

1. **workflow.md を読み込む**（未読の場合）
2. **workflow.md の DoD に列挙された全ゲート**（現状: Spec / Test / Doc / Steering / Soul、および条件付きの **Manual Sync Gate**）を順に検証する。ゲートの構成は workflow.md を正とし、ここで個数や内訳を固定しない。
3. **Test Gate**:
   - まず `session_store_sql` でセッション記録を確認し、直近のターンで `cargo test` が実行され全テスト成功していたか判定する
     ```sql
     SELECT t.content FROM turns t
     JOIN sessions s ON t.session_id = s.id
     WHERE s.id = (SELECT id FROM sessions ORDER BY start_time DESC LIMIT 1)
       AND t.content LIKE '%cargo test%'
       AND t.content LIKE '%test result%'
     ORDER BY t.created_at DESC LIMIT 3
     ```
   - **スキップ可**: セッション記録に `test result: ok` が確認でき、その後にテスト対象コードの変更がない場合、Test Gate をスキップする。スキップ時は完了チェックリストに「(セッション記録により省略)」と注記する
   - **スキップ不可**: セッション記録が見つからない、テスト結果が不明瞭、またはテスト後にコード変更がある場合は実行する:
   ```powershell
   cargo test --all 2>&1 | Select-String "test result:|FAILED|error\["
   ```
4. **Manual Sync Gate（条件付き）**: 判定ルールの本体は workflow.md「完了基準（DoD）> 6. Manual Sync Gate（条件付き）」にある（権威）。ここではその発火だけを行う。
   - **発火条件**: 当該 spec の変更が `doc/spec/` または `book/` に**触れる場合のみ**、`drift-check` を実行する。
     ```powershell
     node book/tools/drift-check.mjs
     ```
   - **中断**: `drift-check.mjs` が**非ゼロ終了**（未解決ドリフト / 未マップ / リンク切れ）した場合は、未解決ドリフトとして**ワークフローを中断し開発者に報告**する（下記5の「いずれかのゲート失敗時は中断」と整合）。ドリフト解消フローは workflow.md を参照。
   - **スキップ**: 当該 spec の変更が `doc/spec/` にも `book/` にも触れない場合は、このゲートを**スキップ**する（Gate 1〜5 のみで完了可）。スキップ時は完了チェックリストに「(無関係変更によりスキップ)」と注記する。
5. **いずれかのゲートが失敗した場合**: ワークフローを中断し、開発者に報告

### ステップ2: 未コミットファイルのコミット

実装中の変更をすべてコミットする。コミットメッセージ形式は workflow.md の規約に従う。

```powershell
git add -A
git commit -m "<type>({feature-name}): 実装完了

- 変更の要約（箇条書き）"
```

### ステップ3: completedフォルダへの移動

specディレクトリをcompleted配下へ移動する。

```powershell
New-Item -ItemType Directory -Path ".kiro/specs/completed" -Force | Out-Null
Move-Item ".kiro/specs/{feature-name}" ".kiro/specs/completed/"
```

**重要**: この時点ではspec.jsonを**編集しない**。VSCodeが編集中のファイルを追跡しており、移動前に編集すると移動操作と競合してファイルが元の場所に復活する。

### ステップ4: spec.jsonのステータス更新

**移動完了後に** spec.json を更新する。以下のフィールドを変更:

```json
{
  "phase": "completed",
  "completed_at": "YYYY-MM-DDTHH:MM:SSZ"
}
```

> **注意**: `"status"` フィールドは使用しない。`"phase": "completed"` のみで完了を表す。

### ステップ5: 参照パスの更新

他のspecファイルや親仕様がこのspecを参照している場合、パスを更新する。

1. **参照箇所の検索**:
```powershell
Get-ChildItem ".kiro/specs" -Filter "*.md" -Recurse |
  Where-Object { $_.FullName -notlike "*completed*" } |
  Select-String -Pattern "{feature-name}" |
  Select-Object -ExpandProperty Path | Sort-Object -Unique
```

2. **パスの一括置換**: `.kiro/specs/{feature-name}/` → `.kiro/specs/completed/{feature-name}/`

3. **親仕様への完了マーク**: 親仕様のdesign.mdに完了ステータス（✅）を反映する（該当する場合）

### ステップ6: 追加更新チェック

workflow.md の「実装完了時アクション」セクションおよび「ドキュメント保守」セクションに従い、以下を実施する。

#### 6-1. ロードマップ更新

`.kiro/steering/roadmap.md` を確認し、完了したSpecが「Specs (dependency order)」に記載されているか判定する。

**スコープ判定（優先順位）**:
1. `requirements.md` に明示的なロードマップ項目との紐付け記述がある場合
2. 開発者が直接指示した場合
3. ロードマップの Specs 一覧にこの feature-name が含まれる場合
4. 判断に迷う場合は開発者に確認

**スコープ内の場合**:
- 対応する `- [ ] {feature-name}` を `- [x] {feature-name}` に更新

**スコープ外の場合**: スキップ

#### 6-2. スキルドキュメント更新

workflow.md の「スキルドキュメント更新検討」セクションに従い、変更領域に応じた対象スキル（`pasta-ghost-authoring` / `pasta-lua-coding`）との整合性を確認・更新する。スキップ条件も workflow.md に準拠。

#### 6-3. ステアリング・ドキュメント更新

workflow.md の「ドキュメント保守 > 更新チェックリスト」に従い、該当するドキュメントを更新する。

### ステップ7: 完了最終コミット

移動・ステータス更新・参照パス修正・追加更新をコミットする。

```powershell
git add -A
git commit -m "chore({feature-name}): spec完了・アーカイブ"
```

### ステップ8: リモート同期（PR ベース）

> **権威的ソース**: workflow.md「実装完了時アクション > 3. リモート同期（ブランチ戦略）」に従う。リモート同期は **PR（Pull Request）ベース**であり、これが唯一の手順実体である。このスキルはフローを複製せず、本セクションを参照したうえで以下を実行する。

> **前提**: 本ステップは `origin`/`main` をハードコードせず、ステップ0で解決した `{remote}` / `{default-branch}` を用いる。フィーチャーブランチ／ワークツリーは Claude Code（ハーネス）が供給しており、このスキルは自前でブランチ／ワークツリーを作成・削除しない。1つの feature = 1つのブランチ = 1つの PR とし、完了時に1回だけ PR を作成して squash マージする。**`{default-branch}` への直接 push は一切行わない。**

確認不要。現在のブランチを判定し、以下を中断なく実行する。

```powershell
$branch = git rev-parse --abbrev-ref HEAD   # 現在の作業ブランチ（ハーネス供給）
```

#### PR 可否判定

以下を**すべて満たす**ときのみ PR を作成・マージする（PR 可）:

1. 現在ブランチが `{default-branch}` 以外（非デフォルトブランチ）。
2. ステップ0で解決した `{remote}` が none でない（リモートあり）。
3. `gh` が認証済み（`gh auth status` が成功）。

いずれかが欠ける場合は **PR 不可** とし、下記「フォールバック（PR 不可時）」へ進む。

#### PR 可: PR 作成 → squash マージ → リモートブランチ削除

```powershell
# 1. 現在ブランチを push して PR を作成（base = {default-branch}, head = 現在ブランチ）
gh pr create --base {default-branch} --head $branch --title "<subject>" --body "<body>"

# 2. squash マージ（--squash 固定、--delete-branch でリモートブランチを API 削除）
#    --subject / --body は下記「squash メッセージ生成」に従って供給する
gh pr merge --squash --delete-branch --subject "<subject>" --body "<body>"
```

- **マージ成否はマージ API の結果のみで判定する。** `gh pr merge` の成否がマージ成否であり、それ以外の警告でマージ成功を覆さない。
- **リモートブランチ削除**: `gh pr merge --delete-branch` が **PR マージ成功後に** API でリモート feature ブランチを削除する。
- **ローカル後始末警告は非致命**: `--delete-branch` のローカル削除試行は、カレントワークツリーでブランチがチェックアウト中のため**ブロックされ警告を出す**ことがある。これは**非致命**でありマージ成功（API 結果）を覆さない。リモートブランチは API により削除済みである。
- **ローカルブランチ／ワークツリーの後始末はハーネスへ委譲**: このスキルは自分のワークツリー／カレントブランチを削除しない（構造的に不可）。ローカルブランチ・ワークツリーの teardown はハーネスがセッション/タスク境界で実施する。

**squash メッセージ生成**（`gh pr merge --squash` の `--subject` / `--body`）:
- 固定文言にせず、**分岐点以降のコミット履歴を要約**して作成する。
- 手順:
  1. `git log --no-merges --pretty=format:"%h %s%n%b" {default-branch}..HEAD`（= `merge-base..HEAD`）で分岐点以降の全コミットを取得
  2. 対象 spec の `requirements.md` / `design.md` のタイトル・概要も参照し意図を補強
  3. 以下の形へ再構成:
     - **subject**（`--subject`）: `<type>({feature-name}): <機能全体を1文で表す要約>`
     - **body**（`--body`）: 主な開発仕様・変更内容を箇条書き（3〜7項目目安）。関連コミットは統合し、`fixup`/typo/WIP 等の些末な履歴は集約・省略。個々のコミット羅列ではなく「何を・なぜ作ったか」の開発単位で再構成する。

#### フォールバック（PR 不可時）

現在ブランチが `{default-branch}` である / `{remote}` が none（リモートなし・オフライン）/ `gh` 未認証 のいずれかの場合:

- **警告を出力**し、PR 作成・push を**スキップ**する。
- ローカルコミットは**そのまま保持**して継続する。
- **`{default-branch}` への直接 push は一切行わない。**

#### 中断条件

PR の**作成またはマージ（API）が失敗**した場合（コンフリクト / mergeable でない / 権限不足等）は、**ブランチを削除せず**処理を中断し開発者へ報告する（復旧可能性を確保するため）。中断するのは**マージ API が失敗したとき**のみであり、`--delete-branch` のローカル削除警告（非致命）とは区別する。

---

## 完了チェックリスト

```
- [ ] DoD 全ゲート通過（workflow.md 準拠: Spec/Test/Doc/Steering/Soul）
- [ ] cargo test --all 成功（またはセッション記録により省略）
- [ ] Manual Sync Gate: 条件付き発火（doc/spec/ または book/ に触れる spec のみ drift-check 実行・非ゼロで中断／無関係変更はスキップ）
- [ ] 未コミットファイルをコミット済み（ステップ2）
- [ ] completedフォルダへ移動済み（ステップ3）※繰り返し仕様はスキップ
- [ ] spec.json の phase を "completed" に更新済み（ステップ4）※繰り返し仕様はスキップ
- [ ] 参照パス更新済み（ステップ5）※繰り返し仕様はスキップ
- [ ] ロードマップ更新済み（スコープ内の場合）
- [ ] スキルドキュメント同期済み（該当する場合）
- [ ] 完了コミット済み（ステップ7）
- [ ] ステップ0で `{remote}` / `{default-branch}` を決定的解決済み
- [ ] リモート同期完了（ステップ8、PR ベース。解決した `{remote}`/`{default-branch}` を使用）
      - PR 可（非デフォルトブランチ かつ `{remote}` あり かつ `gh` 認証あり）: `gh pr create --base {default-branch} --head <current>` → `gh pr merge --squash --delete-branch --subject … --body …`（メッセージは `merge-base..HEAD` 履歴を要約）。マージ成否は API 結果のみで判定し、`--delete-branch` のローカル削除警告は非致命として継続。リモートブランチは API 削除、ローカルブランチ／ワークツリーはハーネス teardown へ委譲
      - PR 不可（`{default-branch}` 上 / `{remote}` none / `gh` 未認証）: 警告して PR・push スキップ、ローカルコミット保持（`{default-branch}` への直接 push なし）
      - PR 作成／マージ（API）失敗: ブランチを残し中断・報告
```

---

## エラー回避

### VSCode変更確定問題
- **症状**: 移動したファイルが元の場所に復活する
- **対策**: spec.jsonは必ずステップ4（移動後）で編集。移動前に編集しない

### 参照パス更新漏れ
- **症状**: 後続specが旧パスで参照しファイルが見つからない
- **対策**: ステップ5で `Select-String` による網羅的検索を実施

### コミット漏れ
- **症状**: pushしたが変更が反映されていない
- **対策**: 各コミット前に `git status --short` で確認

### テスト失敗時
- **症状**: `cargo test --all` が失敗
- **対策**: ワークフローを中断し開発者に報告。テスト修正後に再実行

### リモート同期関連（ステップ8 PR ベース）

#### PR 作成失敗
- **症状**: `gh pr create` が失敗（既存 PR との衝突 / push 権限不足 / ネットワーク等）
- **対策**: **ブランチを削除せず**中断して開発者へ報告。既存 PR がある場合はその PR を確認して再マージを検討

#### マージ不可（mergeable でない / API 失敗）
- **症状**: `gh pr merge --squash` が失敗（コンフリクトで mergeable=false / 必須チェック未通過 / 権限不足等）
- **対策**: **ブランチを削除せず**中断して開発者へ報告。コンフリクトは GitHub 上または別途解決のうえ再実行。中断判定は**マージ API の結果のみ**で行い、`--delete-branch` のローカル削除警告とは混同しない

#### gh 未認証 / リモートなし（PR 不可）
- **症状**: `gh auth status` が失敗、または `{remote}` が none
- **対策**: 警告を出力し PR・push をスキップ。ローカルコミットは保持して継続する。**`{default-branch}` への直接 push は行わない**

#### `{default-branch}` 上で承認された
- **症状**: 現在ブランチが `{default-branch}`（PR の head に使えない）
- **対策**: 警告を出力し PR・push をスキップ、ローカルコミット保持。通常はハーネス供給の非デフォルトブランチ上で完了する想定

#### `--delete-branch` のローカル削除警告
- **症状**: `gh pr merge --delete-branch` がローカルブランチ削除を試みてブロックされ警告を出す（カレントワークツリーでチェックアウト中のため）
- **対策**: **非致命として無視し継続**。リモートブランチは API で削除済み。ローカルブランチ／ワークツリーの後始末はハーネスのワークツリー teardown に委ねる（このスキルは自分のワークツリーを削除しない／できない）
