# 要件定義フェーズ: TODO リスト＆ディスカッション

## 現況
- ✅ Requirements.md: 要件8つ生成・承認待ち
- ✅ Gap Analysis (2025-12-19): Layer-wise 戦略確定
- ✅ Design.md: Phase 0-3 完全計画
- ✅ Test Hierarchy Plan: テスト層別化・リネーム計画
- ✅ Grammar Specification.md: さくら・Jump改訂済み

設計フェーズ完了後、要件定義フェーズの最終確認を行います。

---

## TODO リスト

### ✅ 自明な修正（即座に実施）

- [ ] **TODO-1**: Sakura コマンド簡素化の程度確認
  - **現状**: design.md では「案 B: 完全簡素化」を推奨、が requirements.md は依然「詳細5パターン」前提のままの可能性
  - **確認点**: grammar-specification.md 11章・7章で、実装仕様はどこまで決定されたか
  - **影響**: pest修正スコープ・テスト修正範囲に直結

- [ ] **TODO-2**: Jump 削除の承認確認
  - **現状**: design.md・gap-analysis では Jump 廃止を方針化。但し requirements 段階で「確定か仮定か」を明記すべき
  - **確認点**: Jump 廃止は「必須」か「オプション」か（DSL後方互換性の判断）
  - **影響**: テスト・トランスパイラ修正規模が大きく変動

- [ ] **TODO-3**: Sakura 全角許容の判断
  - **現状**: gap-analysis で「全角 `＼` `［］` 削除」を推奨。但し requirements.md 上「推奨 vs 必須」の明確化が不足
  - **確認点**: 「全角は非推奨・削除対象」か、「全角も許容するが仕様書では半角例を中心」か
  - **影響**: pest修正の最小性・テスト修正規模

- [ ] **TODO-4**: GRAMMAR.md 改訂スコープの確認
  - **現状**: design.md では「7章・11章・制御フロー」の改訂を指示。但し現在の grammar-specification.md で「すでに改訂済みか未着手か」を確認必須
  - **確認点**: grammar-specification.md は最終版か、未改訂の部分があるか
  - **影響**: 設計→実装への引き継ぎタスク明確化

- [ ] **TODO-5**: テスト層別化の命名規則確認
  - **現状**: test-hierarchy-plan.md で「pasta_parser_*_test.rs」等を規則化
  - **確認点**: この命名規則が Cargo.toml・CI設定で問題ないか。また、tests/common への依存リセット必要か
  - **影響**: Phase 0（テスト層別化）の実行可否

- [ ] **TODO-6**: Design における Sakura 「案 A vs 案 B」の正式決定
  - **現状**: design.md で「案 B 推奨」とのみ記載。但し Acceptance Criteria が無いため、決定ベースが不透明
  - **確認点**: 「案 B（完全簡素化）」が設計フェーズの最終決定か、それとも実装フェーズで改めて判断対象か
  - **影響**: Phase 1 の pest修正タスク詳細化

---

### 開発者確認が必要な項目（議題別ディスカッション）

#### ✅ 議題 1: Sakura コマンド簡素化の程度（クローズ）

**決定**: **案 B（完全簡素化）を採用**

**内容**:
- 詳細5パターン廃止 → 「ASCIIトークン + 任意の非ネスト `[...]`」の単純形へ
- **ただしブラケット内の `\]` エスケープは必須対応**
- 「Sakura は非解釈・字句のみ」の仕様に準拠
- 未知トークンはそのまま通す（将来の拡張性確保）

**修正対象**:
- design.md: 「案 B 推奨」→「案 B 決定」へ明記
- Phase 1.1 の pest修正タスク: `sakura_command` を簡素化ルール（`\]` 対応）へ変更
- test-hierarchy-plan.md: 全角テスト削除・詳細パターンテスト削除を正式化

**影響**:
- pest修正: 最小化（Pattern 1-5 削除）
- テスト削除: 最大化（詳細パターン検証ケース全削除）
- Grammar.md: 簡潔な説明で充分

---

#### 議題 2: Jump 削除の最終承認（TODO-2）

**背景**:
- Design.md・Gap-Analysis で Jump 廃止方針は確定
- 但し requirements.md に「Jump 廃止は必須」vs「Jump 廃止は選択肢」の区別がない

**質問**:
1. Jump 廃止は**破壊的変更として必須**か、それとも**オプション**か？
2. Jump を廃止しない場合の代替案（Call への統一の見合わせ）は検討対象か？

**決定ポイント**:
- **決定 A**: Jump 廃止は必須 → Phase 1-3 で Jump 削除を進行
- **決定 B**: Jump 維持 → design.md・gap-analysis を修正し、Jump は継続サポート

---

#### 議題 3: 全角文字の扱い（TODO-3）

**背景**:
- gap-analysis で「全角 `＼` `［］` は削除対象」
- 但し requirements.md では「半角推奨」と「全角削除」の区別が不透明

**質問**:
1. 全角文字は**完全削除**（pest で全角入力を reject）するか、**許容するが非推奨**（grammar では半角例のみ）か？
2. 既存スクリプトで全角を使用している場合の移行戦略は？

**決定ポイント**:
- **決定 A**: 全角完全削除 → pest修正・テスト削除最大
- **決定 B**: 全角許容・非推奨 → pest修正なし、grammar examples の半角統一のみ

---

#### 議題 4: Grammar.md 改訂の現状確認（TODO-4）

**背景**:
- grammar-specification.md は先日「さくら・Jump改訂」済み
- Design では「7章・11章・制御フロー」改訂指示があるが、現在の完成度は？

**質問**:
1. grammar-specification.md のどのセクションが「仕様として確定」し、どこが「まだ改訂待ち」か？
2. 実装フェーズで改訂対象のセクションは？

**決定ポイント**:
- 現状確認のみ（修正不要な可能性）
- 必要なら「改訂セクション」をタスクとして design.md に追加

---

#### 議題 5: Test Hierarchy Plan の Cargo 互換性確認（TODO-5）

**背景**:
- test-hierarchy-plan.md で「pasta_parser_*_test.rs」等の命名規則提案
- 但し Cargo.toml の tests/ ディレクトリ設定・CI との相互作用未確認

**質問**:
1. テストファイルリネーム後、`cargo test` が正しく動作するか（Cargo は自動検出するか、明示指定が必要か）？
2. tests/common への相互参照は影響を受けるか？

**決定ポイント**:
- 現状確認 → 必要ならリネーム計画を修正

---

## 進捗管理

### クローズ予定
1. 議題 1-5 を 1 つずつディスカッション
2. 各議題のクローズごとに requirements.md / design.md / grammar-specification.md を更新
3. 更新後、git commit で進捗記録
4. 全議題クローズ後、設計→実装フェーズへ移行

### スケジュール
- 各議題: 1-2 時間見積
- 全クローズ: 本日中（可能なら）

---

## 参考資料
- requirements.md
- gap-analysis-2025-12-19.md
- design.md
- test-hierarchy-plan.md
- grammar-specification.md

