---
name: kiro-design-discussion
description: 'Kiroワークフローの設計フェーズ完了後に、設計＋設計分析レポートを精査してイシューを収集・分類・解決するディスカッションスキル。USE FOR: 設計ディスカッション, design discussion, 設計レビュー, 設計分析レビュー, 設計精査, kiro design discussion, kiro review design, design review discussion, 設計確認, 設計修正, 設計フィードバック, discuss design, review design validation, design clarification. DO NOT USE FOR: 設計生成（kiro-spec-designを使用）, 設計バリデーション実行（kiro-validate-designを使用）, 要件ディスカッション（kiro-requirements-discussionを使用）, タスク生成（kiro-spec-tasksを使用）, 実装（kiro-spec-implを使用）.'
argument-hint: '<feature-name> — .kiro/specs/ 配下のフィーチャー名'
---

# 設計ディスカッション — Kiro Spec Design Discussion

設計（design.md）および設計分析レポートの完了後に実行する精査・ディスカッションワークフロー。
修正点・疑問点・不安点をイシューとして収集し、分類に応じて自動修正・開発者との1対1ディスカッションを行う。

## 前提条件

- `.kiro/specs/{feature}/design.md` が生成済み
- `.kiro/specs/{feature}/requirements.md` が生成済み（要件との整合確認用）
- ステアリングコンテキスト（`.kiro/steering/`）がロード可能

## ワークフロー

### Phase 1: コンテキストロード

1. `.kiro/specs/{feature}/spec.json` を読み、language と現在のフェーズを確認
2. `.kiro/specs/{feature}/design.md` を全文読み込み
3. `.kiro/specs/{feature}/requirements.md` を全文読み込み（要件との整合確認用）
4. `.kiro/specs/{feature}/gap-analysis.md` を全文読み込み（存在する場合、設計判断の解決確認用）
5. `.kiro/specs/{feature}/research.md` を全文読み込み（存在する場合、調査結果の反映確認用）
6. `.kiro/steering/` 配下をすべて読み込み（product.md, tech.md, structure.md + カスタム）

### Phase 2: イシュー収集

design.md を中心に、関連ドキュメントとの整合性も含めて精査し、以下の観点でイシューを網羅的に収集する：

**収集観点**:
- **矛盾・不整合**: 設計内の矛盾、要件との不整合、gap-analysis の設計判断との齟齬
- **曖昧性**: what/why/how が不明確な設計、複数の解釈が可能な記述
- **要件カバレッジ**: requirements.md の要件・受入基準が設計で未対応
- **インターフェース不備**: コンポーネント間の契約・型・引数が不明確
- **過剰**: 要件スコープ外の設計、YAGNI 違反
- **テスト困難**: 設計上テストが困難な構造
- **リスク**: 未解決のリスク、gap-analysis で指摘されたリスクの設計側カバレッジ不足
- **設計判断未解決**: gap-analysis の設計判断項目が design.md で未解決のまま残っている

### Phase 3: イシュー分類

収集したイシューを以下の2カテゴリに分類する：

| カテゴリ | 判定基準 | アクション |
|---------|---------|-----------|
| **A: 自明な修正** | 誤字・明らかな不整合・記述漏れなど、開発者確認不要 | 即座に修正してコミット |
| **B: 開発者確認** | what/why/how が曖昧、ドメイン知識依存、トレードオフ判断が必要 | 1議題ずつディスカッション |

### Phase 4: 自明な修正の実行（カテゴリ A）

1. 修正内容を一覧として提示（修正前→修正後）
2. design.md を更新
3. requirements.md に影響がある場合はそちらも更新
4. 変更をコミット（コミットメッセージ: `docs({feature}): fix obvious issues in design`）

### Phase 5: 開発者ディスカッション（カテゴリ B）

**進行ルール**:
- **1議題ずつ**進行する（複数を同時に聞かない）
- 各議題について以下を提示：
  1. **議題番号と総数**（例: 「議題 1/3」）
  2. **対象**: どの設計セクション・コンポーネントに関わるか（引用付き）
  3. **問題点**: 何が曖昧・不明確か
  4. **選択肢**: 考えられる設計方針や代替案（2-3個）
  5. **推奨**: エージェントとしての推奨案（根拠付き）
- 開発者の回答を受けて：
  1. design.md を更新：議論で明らかになった点の記載、不要になった要件の集約・削除も実施（必要に応じて requirements.md も更新）
  2. 変更をコミット（コミットメッセージ: `docs({feature}): resolve design discussion #{n} - {topic}`）
  3. **修正内容の要約**を報告し、残り議題の**サマリーレポート**を提示してから次の議題へ

**サマリーレポート形式**（各議題完了後に表示）:
```
## 進捗サマリー
- 完了: {n}/{total} 議題
- 直前の解決: {topic} → {decision}
- 残り: {remaining_topics}
- 次の議題: {next_topic_preview}
```

### Phase 6: 完了報告

すべてのイシューが処理された後：

1. 最終サマリーを表示：
   - カテゴリA（自明修正）: {n}件 修正済み
   - カテゴリB（ディスカッション）: {n}件 解決済み
2. 次のステップを案内：
   - 「設計の精査が完了しました。次のコマンドでタスク分解に進めます: `/kiro-spec-tasks {feature} [-y]`」

## エラーハンドリング

- **design.md 未生成**: 「先に `/kiro-spec-design {feature}` を実行してください」と案内して停止
- **requirements.md 未生成**: 警告を出しつつ design.md のみで続行
- **イシューなし**: 「精査の結果、修正すべき点は見つかりませんでした。タスク分解に進めます」と即座に案内

## コミット規約

- prefix: `docs({feature})`
- 自明修正は1コミットにまとめる
- ディスカッション解決は議題ごとに個別コミット
