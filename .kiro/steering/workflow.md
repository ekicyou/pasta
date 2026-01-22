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

---

## 実装完了時アクション

### 1. コミット
```bash
git add -A && git commit -m "<type>(<scope>): <summary>"
```
タイプ: `feat`, `fix`, `refactor`, `docs`, `test`

### 2. リモート同期
```bash
git push origin <branch>
```

### 3. 仕様アーカイブ
```bash
mv .kiro/specs/<spec-name> .kiro/specs/completed/
git add -A && git commit -m "chore(spec): <spec-name>をcompletedへ移動"
git push origin <branch>
```

---

## 回帰責任（Regression-First Fix）

- **同一PRで修正**: 既存テストが落ちたらマージ前に修正
- **原因特定**: 最小再現を特定し根本原因を修正
- **テスト更新**: 挙動変更が正当なら、テストを先に更新し理由を明記

---

## 禁止事項

**MVP禁止**: 以下の表現は完成宣言に使わない
- 「MVP」「部分実装」「スキャフォールドのみ」「とりあえず動く」

**推奨表現**:
- 「全テスト合格」「DoD Gate通過」「追加タスク待ち（未完成）」

---

## ドキュメント保守

### 更新チェックリスト

コード変更時に以下のドキュメント更新を確認：

| 変更種別             | 更新対象ドキュメント                                    |
| -------------------- | ------------------------------------------------------- |
| 公開API変更          | クレートREADME、SPECIFICATION.md                        |
| DSL文法変更          | GRAMMAR.md、steering/grammar.md                         |
| ディレクトリ構造変更 | steering/structure.md、クレートREADME                   |
| 依存関係変更         | steering/tech.md、クレートREADME                        |
| 開発フロー変更       | steering/workflow.md、AGENTS.md                         |
| 新クレート追加       | README.md（ドキュメントマップ）、クレートREADME新規作成 |

### 保守責任

| ドキュメント     | 更新トリガー                         |
| ---------------- | ------------------------------------ |
| README.md        | プロジェクト概要変更、新クレート追加 |
| AGENTS.md        | AI開発支援コンテキスト変更           |
| GRAMMAR.md       | DSL文法変更                          |
| SPECIFICATION.md | 言語仕様変更（権威的）               |
| クレートREADME   | クレートAPI/構造変更                 |
| steering/*       | 対応領域の変更                       |

### 保守ルール

1. **API変更時**: 対応するクレートREADMEの「公開API」セクションを更新
2. **仕様変更時**: まずSPECIFICATION.mdを更新、その後GRAMMAR.mdを同期
3. **PR時確認**: ドキュメント更新漏れがないかDoDチェックリストで確認
