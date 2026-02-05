# 開発ワークフロー

Kiro仕様駆動開発における作業フローと完了基準。

> **関連ドキュメント**: [AGENTS.md](../../AGENTS.md) - AI開発支援ドキュメント（ワークフロー概要、コマンド一覧）

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

### 2. リモート同期
```powershell
git push origin <branch>
```

### 3. 仕様アーカイブ

**重要**: spec.json更新は仕様移動の**直後**に実行（移動前に更新するとVSCode仕様でファイルが復活する場合がある）

```powershell
# 1. 仕様ディレクトリを移動
Move-Item .kiro/specs/<spec-name> .kiro/specs/completed/

# 2. spec.jsonのphaseを"completed"に更新
# （エディタまたはjqコマンドで .kiro/specs/completed/<spec-name>/spec.json を編集）

# 3. コミット＆プッシュ
git add -A; git commit -m "chore(spec): <spec-name>をcompletedへ移動"
git push origin <branch>
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
| 開発フロー変更       | steering/workflow.md、AGENTS.md                         |
| 新クレート追加       | README.md（ドキュメントマップ）、クレートREADME新規作成 |
| テストカバレッジ変更 | TEST_COVERAGE.md                                        |

### 保守責任

| ドキュメント     | 更新トリガー                             |
| ---------------- | ---------------------------------------- |
| **SOUL.md**      | **コアバリュー・設計原則変更（最優先）** |
| README.md        | プロジェクト概要変更、新クレート追加     |
| AGENTS.md        | AI開発支援コンテキスト変更               |
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
