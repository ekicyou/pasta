---
name: kiro-requirements-discussion
description: 'Kiroワークフローの要件定義フェーズ完了後に、要件＋ギャップ分析を精査してイシューを収集・分類・解決するディスカッションスキル。USE FOR: 要件ディスカッション, requirements discussion, 要件レビュー, ギャップ分析レビュー, 要件精査, 設計判断収集, kiro discussion, kiro review requirements, spec review, 要件確認, 要件修正, 要件フィードバック, discuss requirements, review gap analysis, requirement clarification. DO NOT USE FOR: 要件生成（kiro-spec-requirementsを使用）, ギャップ分析実行（kiro-validate-gapを使用）, 設計ドキュメント生成（kiro-spec-designを使用）, 実装（kiro-spec-implを使用）.'
argument-hint: '<feature-name> — .kiro/specs/ 配下のフィーチャー名'
---

# 要件ディスカッション — Kiro Spec Requirements Discussion

要件定義（requirements.md）およびギャップ分析（gap-analysis.md）の完了後に実行する精査・ディスカッションワークフロー。
修正点・疑問点・不安点をイシューとして収集し、分類に応じて自動修正・設計判断フラグ付け・開発者との1対1ディスカッションを行う。

## 前提条件

- `.kiro/specs/{feature}/requirements.md` が生成済み
- `.kiro/specs/{feature}/gap-analysis.md` が生成済み（推奨、なくても実行可能）
- ステアリングコンテキスト（`.kiro/steering/`）がロード可能

## ワークフロー

### Phase 1: コンテキストロード

1. `.kiro/specs/{feature}/spec.json` を読み、language と現在のフェーズを確認
2. `.kiro/specs/{feature}/requirements.md` を全文読み込み
3. `.kiro/specs/{feature}/gap-analysis.md` を全文読み込み（存在する場合）
4. `.kiro/steering/` 配下をすべて読み込み（product.md, tech.md, structure.md + カスタム）
5. `.kiro/settings/rules/ears-format.md` を読み込み（EARS形式検証用）

### Phase 2: イシュー収集

requirements.md と gap-analysis.md を精査し、以下の観点でイシューを網羅的に収集する：

**収集観点**:
- **矛盾・不整合**: 要件間の矛盾、ギャップ分析との不整合
- **曖昧性**: what/why が不明確な要件、複数の解釈が可能な記述
- **EARS形式違反**: 受入基準のEARS形式不備
- **欠落**: gap-analysis で指摘されているが requirements に未反映の項目
- **過剰**: 要件スコープ外の記述、実装詳細の混入
- **設計判断**: gap-analysis の設計判断項目（design decisions）の要件側への影響
- **テスト困難**: 検証が困難または不可能な受入基準
- **リスク**: gap-analysis で指摘されたリスクの要件側カバレッジ

### Phase 3: イシュー分類

収集したイシューを以下の3カテゴリに分類する：

| カテゴリ | 判定基準 | アクション |
|---------|---------|-----------|
| **A: 自明な修正** | 誤字・EARS形式修正・明らかな不整合など、開発者確認不要 | 即座に修正してコミット |
| **B: 設計判断** | how に関わる判断、アーキテクチャ選択、トレードオフ | 設計フェーズへ先送り（gap-analysis.md の設計判断セクションと統合） |
| **C: 開発者確認** | what/why が曖昧、ドメイン知識依存、優先度判断が必要 | 1議題ずつディスカッション |

### Phase 4: 自明な修正の実行（カテゴリ A）

1. 修正内容を一覧として提示（修正前→修正後）
2. requirements.md を更新
3. gap-analysis.md に影響がある場合はそちらも更新
4. 変更をコミット（コミットメッセージ: `docs({feature}): fix obvious issues in requirements`）

### Phase 5: 設計判断の整理（カテゴリ B）

1. 設計判断項目を一覧として提示
2. gap-analysis.md の設計判断セクションに未記載の項目があれば追記（新規ファイルは作らない）
3. 「これらは設計フェーズ（`/kiro-spec-design`）で解決します」と宣言
4. 変更がある場合はコミット

### Phase 6: 開発者ディスカッション（カテゴリ C）

**進行ルール**:
- **1議題ずつ**進行する（複数を同時に聞かない）
- 各議題について以下を提示：
  1. **議題番号と総数**（例: 「議題 1/3」）
  2. **対象**: どの要件・受入基準に関わるか（ID付き引用）
  3. **問題点**: 何が曖昧・不明確か
  4. **選択肢**: 考えられる解釈や方向性（2-3個）
  5. **推奨**: エージェントとしての推奨案（根拠付き）
- 開発者の回答を受けて：
  1. requirements.md を更新：議論で明らかになった点の記載、不要になった要件の集約・削除も実施（必要に応じて gap-analysis.md も更新）
  2. 変更をコミット（コミットメッセージ: `docs({feature}): resolve discussion #{n} - {topic}`）
  3. **修正内容の要約**を報告し、残り議題の**サマリーレポート**を提示してから次の議題へ

**サマリーレポート形式**（各議題完了後に表示）:
```
## 進捗サマリー
- 完了: {n}/{total} 議題
- 直前の解決: {topic} → {decision}
- 残り: {remaining_topics}
- 次の議題: {next_topic_preview}
```

### Phase 7: 完了報告

すべてのイシューが処理された後：

1. 最終サマリーを表示：
   - カテゴリA（自明修正）: {n}件 修正済み
   - カテゴリB（設計判断）: {n}件 設計フェーズへ
   - カテゴリC（ディスカッション）: {n}件 解決済み
2. 次のステップを案内：
   - 「要件の精査が完了しました。次のコマンドで設計フェーズに進めます: `/kiro-spec-design {feature} [-y]`」

## エラーハンドリング

- **requirements.md 未生成**: 「先に `/kiro-spec-requirements {feature}` を実行してください」と案内して停止
- **gap-analysis.md 未生成**: 警告を出しつつ requirements.md のみで続行（「ギャップ分析なしで精査します。`/kiro-validate-gap {feature}` の実行を推奨します」）
- **イシューなし**: 「精査の結果、修正すべき点は見つかりませんでした。設計フェーズに進めます」と即座に案内

## コミット規約

- prefix: `docs({feature})`
- 自明修正は1コミットにまとめる
- ディスカッション解決は議題ごとに個別コミット
- 設計判断の追記は1コミットにまとめる
