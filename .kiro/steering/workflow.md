# 開発ワークフロー

Kiro仕様駆動開発における作業フローと完了基準。

> **関連ドキュメント**: [CLAUDE.md](../../CLAUDE.md) - AI開発支援ドキュメント（プロジェクト指示、コマンド一覧）

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
6. **Manual Sync Gate（条件付き）**: マニュアル（`book/`）と権威仕様（`doc/spec/`）の整合確認

#### 6. Manual Sync Gate（条件付き）

**ルール本体はこのゲートに置く**（権威）。`kiro-complete` はこのゲートを発火・オーケストレーションするのみで、判定ルールを複製しない。

- **発火条件**: 当該 spec の変更が `doc/spec/` または `book/` に**触れる場合のみ**発火する。
- **スキップ**: 当該 spec の変更が `doc/spec/` にも `book/` にも触れない場合は、このゲートを**スキップ**する（無関係な spec の完了承認を重くしない）。Gate 1〜5 のみで完了可とする。
- **判定**: 発火時は `node book/tools/drift-check.mjs` を実行し、以下が無いことを確認する。
  - **ドリフト**: `book/manual-sources.toml` の記録ハッシュ（版マーカー）と `doc/spec/` の現値ハッシュの不一致（＝参照元が変わったのにマニュアル章が追従していない）。
  - **未マップ / リンク切れ**: `doc/spec/` に存在するがマッピングに無い章・節、マニュアル→`doc/spec/` および外部参照リンクの切れ。
- **中断**: `drift-check.mjs` が**非ゼロ終了**した場合は、**未解決ドリフトとして完了を中断**する。
- **ドリフト解消フロー**: 該当マニュアル章を `doc/spec/` の現状に追従更新したうえで、`book/manual-sources.toml` の版マーカーを現値に更新する（＝レビュー済みであることの明示）。これでゲートを再実行し通過させる。

> 既存 Gate 1〜5 の意味・順序は変更しない。本ゲートは条件付きの**追加**である。

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

リモート同期は **PR（Pull Request）ベース**で行う。これが唯一の手順実体であり、各スキルはこのフローを複製せず本セクションを権威定義として参照する。

> **権威定義**: フィーチャーブランチ／ワークツリーの生成は Claude Code（ハーネス）のワークツリー機能が供給する。スキルは自前でブランチ／ワークツリーを作成せず、「commit → PR 作成 → PR の squash マージ → ブランチ削除」だけを担う。1つの feature = 1つのブランチ = 1つの PR とし、完了時に1回だけ PR を作成して squash マージする。**デフォルトブランチ（main）への直接 push は行わない。**

**フロー（commit → PR 作成 → squash マージ → ブランチ削除）**:

前提として、`{remote}`（`origin` → 単一リモート → none）と `{default-branch}`（`git symbolic-ref` → `main` → `master` → 現ブランチ）を決定的に解決し、`origin`/`main` のハードコードを避ける。

```powershell
$branch = git rev-parse --abbrev-ref HEAD   # 現在の作業ブランチ（ハーネス供給）

# 1. ローカルコミット（§1 のコミット規約に従う）
git add -A; git commit -m "<type>(<scope>): <summary>"   # 既にコミット済みならスキップ

# 2. 現在ブランチを push して PR を作成（base = デフォルトブランチ, head = 現在ブランチ）
gh pr create --base {default-branch} --head $branch --title "<subject>" --body "<body>"

# 3. squash マージ（--squash 固定、--delete-branch でリモートブランチを API 削除）
#    --subject / --body は下記「squash コミットメッセージの生成方針」に従って供給する
gh pr merge --squash --delete-branch --subject "<subject>" --body "<body>"
```

**ブランチ削除のタイミング**: リモートの feature ブランチは `gh pr merge --delete-branch` が **PR マージ成功後に** API で削除する。**マージ成功前にブランチを削除しない**（復旧可能性を確保するため）。ローカルブランチおよびワークツリーは、カレントワークツリーで実行中のため構造的に削除できない。これらの後始末はハーネスのワークツリー teardown に委ねる（`--delete-branch` のローカル削除試行がブロックされて警告を出しても、これは非致命でありマージ成功を覆さない）。

**フォールバック（PR 不可時）**: 現在のブランチがデフォルトブランチである / `{remote}` が不在・オフライン / `gh` 未認証 のいずれかの場合は、**警告を出力して PR 操作・push をスキップ**し、ローカルコミットを保持したまま継続する。**デフォルトブランチへの直接 push は一切行わない。**

**中断条件**: PR の作成またはマージ（API）が失敗した（コンフリクト / mergeable でない / 権限不足等）場合は、**ブランチを削除せず**処理を中断し開発者へ報告する（復旧可能性を確保するため）。マージ成否の判定はマージ API の結果のみに基づき、`--delete-branch` のローカル削除警告とは区別する。

> **release タグ公開のカーブアウト**: `release-workflow` のリリース手順は、コミットの main 反映を **PR のマージコミット方式**（`gh pr merge --merge`、spec 完了の `--squash` とは別系統）で行い、**デフォルトブランチへの直接 push は行わない**。リリースが行う **タグ ref の push**（`git push origin vX.Y.Z`。ブランチ push ではない）のみが本セクションの直接 push 禁止の対象外であり、将来 main のブランチ保護を有効化しても成立する。`.claude/settings.json` はタグ push 許可（`git push origin v*`）を保持し、`git push origin main` の直接 push 許可は撤去する。

#### squash コミットメッセージの生成方針

`gh pr merge --squash` に渡す `--subject` / `--body` は、**分岐点以降のコミット履歴を要約**して作成する（固定文言にしない）。

1. **履歴を収集**: `git log --no-merges --pretty=format:"%h %s%n%b" {default-branch}..HEAD`（= `merge-base..HEAD`）で分岐点以降の全コミットを取得する。
2. **意図を補強**: 対象 spec の `requirements.md` / `design.md` のタイトル・概要も参照し、機能の目的を正確に反映する。
3. **要約してメッセージ化**:
   - **subject**（`--subject`）: `<type>(<scope>): <機能全体を1文で表す要約>`
   - **body**（`--body`）: 主な開発仕様・変更内容を箇条書き（3〜7項目目安）。関連コミットは1項目へ統合し、`fixup` / typo修正 / WIP などの些末な履歴は集約・省略する。
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
