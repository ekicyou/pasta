# 開発ワークフロー

Kiro仕様駆動開発における作業フローと完了基準。

> **関連ドキュメント**: [AGENTS.md](../../AGENTS.md) / [CLAUDE.md](../../CLAUDE.md) - AI開発支援ドキュメント（プロジェクト指示、コマンド一覧）

---

## 仕様フェーズ

```
requirements → design → tasks → implementation → implementation-complete
```

### コマンド
| コマンド                            | 用途       |
| ----------------------------------- | ---------- |
| `/kiro-spec-init "description"`     | 仕様初期化 |
| `/kiro-spec-requirements {feature}` | 要件定義   |
| `/kiro-spec-design {feature} [-y]`  | 設計生成   |
| `/kiro-spec-tasks {feature} [-y]`   | タスク分解 |
| `/kiro-spec-impl {feature} [tasks]` | 実装       |
| `/kiro-spec-status {feature}`       | 進捗確認   |

---

## 完了基準（DoD）

すべて同時に満たすこと：

1. **Spec Gate**: 全フェーズ承認済み
2. **Test Gate**: `cargo test --all` 成功
3. **Doc Gate**: 仕様差分を反映
4. **Steering Gate**: 既存ステアリングと整合
5. **Soul Gate**: [SOUL.md](../../SOUL.md) との整合性確認（タスク生成時に自動追加）

---

## 実装完了時アクション

### 1. コミット
```powershell
git add -A; git commit -m "<type>(<scope>): <summary>"
```
タイプ: `feat`, `fix`, `refactor`, `docs`, `test`

### 2. スキルドキュメント更新検討

実装内容が以下に該当する場合、対応するスキルの SKILL.md および references/ を読み込み、実装との乖離がないか検証・更新する。

| 変更領域 | 対象スキル | 確認ポイント |
|----------|-----------|-------------|
| DSL文法・マーカー・シーン構造 | `pasta-ghost-authoring` | references/ の文法記述、§2マーカー一覧表、パターン集の整合性 |
| Lua API・ランタイム動作・モジュール追加 | `pasta-lua-coding` | references/ のAPI記述、モジュール一覧、コーディング規約の整合性 |
| 両方に影響する変更 | 両スキル | 役割分離（DSL層 vs Lua層）の境界が正しいか |

**手順**:
1. SKILL.md を読み込み、実装で変更・追加した機能が反映されているか確認
2. references/ 配下の該当ファイルを読み込み、記述の正確性を検証
3. 乖離があれば更新し、SKILL.md の metadata.version をバンプ
4. 更新があった場合はコミット:
   ```powershell
   git add .agents/skills/; git commit -m "docs(skill): <スキル名> を実装に同期"
   ```

**スキップ条件**: テストのみの変更、ドキュメントのみの変更、スキル対象外クレート（pasta_lsp等）のみの変更は対象外。

### 3. リモート同期（ブランチ戦略）

現在のブランチに応じて同期方法を分岐する。

> **注（直接 push）**: 完了フローは main へ**直接 push** する（PR は経由しない）。これは本プロジェクトの意図された運用であり、auto-mode で実行する場合は main への push 権限が必要。main 以外のブランチでは squash ブランチ経由で ff マージ後に main へ直接 push する。

**main ブランチの場合**: そのまま push する。
```powershell
git push origin main
```

**main 以外のブランチ（= ブランチA）の場合**: ブランチAの「mainからの分岐点以降の差分」を1コミットに集約した squash ブランチ（`squash/<A>` = ブランチB）を作り、main へ fast-forward マージしてから push する。マージ・同期がすべて成功したら、ブランチ A/B をローカル・リモート両方から削除する。

```powershell
$branchA = git rev-parse --abbrev-ref HEAD
$branchB = "squash/$branchA"

# 1. リモート最新を取得（push reject 予防）
git fetch origin

# 2. origin/main を起点に squash ブランチBを作成し、Aの全差分を1コミットへ集約
git switch -c $branchB origin/main
git merge --squash $branchA
#   コンフリクト時: 内容を精査して解決する（spec完了文脈ではA側の変更が原則正）。
#   解決できたら継続。判断不能・意味的に危険な場合は中断し開発者へ報告。

#   squash コミットメッセージは、分岐点以降の履歴を要約して生成する（下記「方針」参照）
git log --no-merges --pretty=format:"%h %s%n%b" origin/main..$branchA
git commit -F <生成した要約メッセージ>   # 履歴要約から作成（-m 複数指定でも可）

# 3. main を squash ブランチへ fast-forward（Bはmain先端+1コミットなので構造上必ずff可能）
git switch main
git merge --ff-only $branchB

# 4. push（reject時はリモートmainが先行＝特殊ケース。中断して報告）
git push origin main

# 5. すべて成功したらブランチ A/B をローカル削除
git branch -D $branchA
git branch -D $branchB

# 6. リモートに存在すれば削除
if (git ls-remote --heads origin $branchA) { git push origin --delete $branchA }
if (git ls-remote --heads origin $branchB) { git push origin --delete $branchB }
```

**中断条件**: コンフリクト解決不能 / `--ff-only` マージ失敗 / push reject のいずれかが発生した場合は、**ブランチ A/B を削除せず**処理を中断し開発者へ報告する（復旧可能性を確保するため）。

#### squash コミットメッセージの生成方針

ブランチBのコミットメッセージは、**分岐点以降のコミット履歴を要約**して作成する（固定文言にしない）。

1. **履歴を収集**: `git log --no-merges --pretty=format:"%h %s%n%b" origin/main..$branchA` で分岐点以降の全コミットを取得する。
2. **意図を補強**: 対象 spec の `requirements.md` / `design.md` のタイトル・概要も参照し、機能の目的を正確に反映する。
3. **要約してメッセージ化**:
   - **subject**: `<type>(<scope>): <機能全体を1文で表す要約>`
   - **body**: 主な開発仕様・変更内容を箇条書き（3〜7項目目安）。関連コミットは1項目へ統合し、`fixup` / typo修正 / WIP などの些末な履歴は集約・省略する。
   - 個々のコミットを羅列するのではなく、**「何を・なぜ作ったか」の開発単位**で再構成する。

### 4. 仕様アーカイブ

**重要**: spec.json更新は仕様移動の**直後**に実行（移動前に更新するとVSCode仕様でファイルが復活する場合がある）

**例外: 繰り返し仕様**: `release-workflow` のような繰り返し実行型仕様は `completed/` に移動しない。`/kiro-spec-impl` 実行のたびにタスクがリセットされ、常に `.kiro/specs/` 直下に留まる。

```powershell
# 1. 仕様ディレクトリを移動
Move-Item .kiro/specs/<spec-name> .kiro/specs/completed/

# 2. spec.jsonのphaseを"completed"に更新
# （エディタまたはjqコマンドで .kiro/specs/completed/<spec-name>/spec.json を編集）

# 3. コミット（プッシュは §3 リモート同期（ブランチ戦略）に従う）
git add -A; git commit -m "chore(spec): <spec-name>をcompletedへ移動"
```

---

## タスク生成ルール

### 必須タスク（自動追加）

**`/kiro-spec-tasks` 実行時、以下のタスクを常に生成リストに含めること**：

#### 最終タスク: ドキュメント整合性確認

すべての実装タスクの後に、以下の最終タスクを**必ず追加**する：

```markdown
**Task: ドキュメント整合性の確認と更新**

実装完了後、以下のドキュメントとの整合性を確認・更新：

1. [ ] SOUL.md - コアバリュー・設計原則との整合性確認
2. [ ] doc/spec/ - 言語仕様の更新（該当する場合）
3. [ ] GRAMMAR.md - 文法リファレンスの同期（該当する場合）
4. [ ] TEST_COVERAGE.md - 新規テストのマッピング追加
5. [ ] クレートREADME - API変更の反映（該当する場合）
6. [ ] steering/* - 該当領域のステアリング更新
7. [ ] .agents/skills/pasta-ghost-authoring/ - DSL文法変更時にスキル同期（該当する場合）
8. [ ] .agents/skills/pasta-lua-coding/ - Lua API変更時にスキル同期（該当する場合）

特に、以下の場合は**SOUL.md更新が必須**：
- コアバリュー（日本語フレンドリー、UNICODE識別子、yield型、宣言的フロー）に影響
- 設計原則（行指向文法、前方一致、UI独立性）に影響
- Phase 0完了基準（DoD）の進捗に影響
```

### タスク生成時の注意事項

- 実装タスクは具体的かつテスト可能な粒度に分割
- 各タスクにDoD（完了条件）を明記
- 最終タスク「ドキュメント整合性確認」は**削除・省略禁止**
- タスク順序は依存関係を考慮（テストファースト推奨）

---

## 回帰責任（Regression-First Fix）

- **同一PRで修正**: 既存テストが落ちたらマージ前に修正
- **原因特定**: 最小再現を特定し根本原因を修正
- **テスト更新**: 挙動変更が正当なら、テストを先に更新し理由を明記

---

## 禁止事項

### MVP禁止

以下の表現は完成宣言に使わない：
- 「MVP」「部分実装」「スキャフォールドのみ」「とりあえず動く」

**推奨表現**:
- 「全テスト合格」「DoD Gate通過」「追加タスク待ち（未完成）」

### 危険な Git 操作の禁止

**❌ 絶対禁止**：複数の変更を巻き込む可能性のある破壊的 Git 操作

| 禁止コマンド             | 理由                                   | 代替手段                                       |
| ------------------------ | -------------------------------------- | ---------------------------------------------- |
| `git revert <commit>`    | 他セッションの未コミット作業を巻き込む | `git show <commit>` で差分確認後、手動で逆変更 |
| `git reset --hard`       | 未コミット変更を完全消去               | `git status` で確認後、必要なら `git stash`    |
| `git checkout -- <file>` | ファイル単位の強制破棄                 | `git diff <file>` で確認後、エディタで手動修正 |
| `git clean -fd`          | 未追跡ファイルの一括削除               | `git clean -fdn` で確認後、個別削除            |

**✅ 安全な修正手順**：

1. **状況確認**：
   ```powershell
   git status              # 未コミット変更を確認
   git diff                # 差分を確認
   ```

2. **変更の取り消し（ファイル単位）**：
   ```powershell
   # エディタで手動修正（推奨）
   # または git restore で個別復元
   git restore <file>      # 慎重に使用
   ```

3. **コミット単位の修正**：
   ```powershell
   # revert の代わりに逆コミットを手動作成
   git show <commit>       # 差分確認
   # エディタで逆変更を適用
   git add <files>
   git commit -m "revert: <変更内容の説明>"
   ```

**複数セッション作業時の原則**：

- **コミット前に必ず `git status` 確認**
- **未コミット変更がある場合、破壊的操作は厳禁**
- **疑問があれば開発者に確認**
- **AI エージェント間での作業共有を前提とした慎重な Git 操作**

---

## ドキュメント保守

### 更新チェックリスト

コード変更時に以下のドキュメント更新を確認：

| 変更種別             | 更新対象ドキュメント                                    |
| -------------------- | ------------------------------------------------------- |
| コアバリュー影響     | **SOUL.md（最優先）**、doc/spec/                        |
| 公開API変更          | クレートREADME、doc/spec/                               |
| DSL文法変更          | GRAMMAR.md、steering/grammar.md、SOUL.md（設計原則）    |
| ディレクトリ構造変更 | steering/structure.md、クレートREADME                   |
| 依存関係変更         | steering/tech.md、クレートREADME                        |
| 開発フロー変更       | steering/workflow.md、CLAUDE.md                         |
| DSL文法変更          | .agents/skills/pasta-ghost-authoring/ (SKILL.md + references/) |
| Lua API変更          | .agents/skills/pasta-lua-coding/ (SKILL.md + references/)     |
| 新クレート追加       | README.md（ドキュメントマップ）、クレートREADME新規作成 |
| テストカバレッジ変更 | TEST_COVERAGE.md                                        |

### 保守責任

| ドキュメント     | 更新トリガー                             |
| ---------------- | ---------------------------------------- |
| **SOUL.md**      | **コアバリュー・設計原則変更（最優先）** |
| README.md        | プロジェクト概要変更、新クレート追加     |
| CLAUDE.md        | AI開発支援プロジェクト指示変更           |
| GRAMMAR.md       | DSL文法変更                              |
| doc/spec/        | 言語仕様変更（権威的）                   |
| TEST_COVERAGE.md | テスト追加・削除・機能変更               |
| クレートREADME   | クレートAPI/構造変更                     |
| steering/*       | 対応領域の変更                           |

### 保守ルール

1. **コアバリュー変更時**: まずSOUL.mdを更新、その後doc/spec/・GRAMMAR.mdを同期
2. **API変更時**: 対応するクレートREADMEの「公開API」セクションを更新
3. **仕様変更時**: まずdoc/spec/を更新、その後GRAMMAR.mdを同期
4. **テスト追加時**: TEST_COVERAGE.mdのマッピングを更新
5. **PR時確認**: ドキュメント更新漏れがないかDoDチェックリストで確認
