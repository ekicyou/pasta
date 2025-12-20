# 開発ワークフロー

Kiro仕様駆動開発における作業フローと完了基準。

---

## 仕様フェーズ

```
requirements → design → tasks → implementation → implementation-complete
```

### コマンド
| コマンド | 用途 |
|---------|------|
| `/kiro-spec-init "description"` | 仕様初期化 |
| `/kiro-spec-requirements {feature}` | 要件定義 |
| `/kiro-spec-design {feature} [-y]` | 設計生成 |
| `/kiro-spec-tasks {feature} [-y]` | タスク分解 |
| `/kiro-spec-impl {feature} [tasks]` | 実装 |
| `/kiro-spec-status {feature}` | 進捗確認 |

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
git add -A
git commit -m "<type>(<scope>): <summary>

Spec: <spec-name>"
```

**コミットタイプ**: `feat`, `fix`, `refactor`, `docs`, `test`

### 2. リモート同期
```bash
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
