# Workflow - 開発ワークフロー

Kiro仕様駆動開発における作業フローと完了時アクション。

---

## 実装完了時のアクション

仕様の実装が完了し、フェーズが `implementation-complete` に移行した際の必須アクション：

### 1. コミット
```bash
git add -A
git commit -m "<type>(<scope>): <summary>

<body>

Spec: <spec-name>"
```

**コミットタイプ**:
- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメント
- `test`: テスト追加・修正

### 2. リモート同期
```bash
git push origin <branch>
```

### 3. ロードマップ更新
ROADMAPが存在する場合（メタ仕様配下の仕様など）：
- Progress Summary を更新
- Phase 列を `implementation-complete` に更新
- 📍 参照: `focus.md` のROADMAP更新タイミング

### 4. 完了確認
- スペックファイルが `.kiro/specs/completed/` に移動済み
- `spec.json` の `phase` が `implementation-complete`
- 全テストがパス
- ロードマップ更新済み（該当する場合）

## 仕様フェーズフロー

```
requirements → design → tasks → implementation → implementation-complete
```

各フェーズ移行時に進捗を確認し、完了時は上記アクションを実行。

---
_Document patterns, not every workflow variation_
