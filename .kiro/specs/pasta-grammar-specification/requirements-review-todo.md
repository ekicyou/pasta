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

#### ✅ 議題 2: Jump 削除の最終承認（クローズ）

**決定**: **Jump 廃止は必須**（破壊的変更として）

**内容**:
- Jump 文（`？`）は廃止
- Call 文（`＞`）へ統一
- セマンティクス上の区別なし（Call と同一動作）
- MVP 達成前の段階において、破壊的変更を積極的に適用

**修正対象**:
- requirements.md: 「Jump 廃止は必須」を明記（オプションではない）
- design.md: Phase 1-3 タスク（Jump 削除）を確定タスクに昇格
- gap-analysis: Jump 廃止を必須方針に統一

**影響**:
- Parser 層: jump_marker, jump_content ルール削除
- Transpiler 層: Statement::Jump, pasta::jump() 削除
- Tests: Jump 依存テスト全削除、`？` → `＞` 置換
- GRAMMAR.md: Jump 記述削除、Call のみ記載

---

#### ✅ 議題 3: 全角文字の扱い（クローズ）

**決定**: **全角完全削除（Case A）**

**内容**:
- Sakura エスケープに関して、全角定義は存在しない
- ハーフ幅 `\` のみの定義で確定
- pest 定義から全角文字（`＼` `［］`）を削除
- 既存全角スクリプトは半角への移行が必須

**修正対象**:
- design.md: Phase 1.1 pest修正ルール内で「全角文字削除」を明記
- src/parser/pasta.pest: 全角 `＼` `［］` ルール削除
- tests: 全角テストケース削除・置換
- grammar-specification.md 11.16: ハーフ幅定義のみ確定

**影響**:
- pest修正: sakura_escape, bracket_open/close の全角ルール削除
- テスト: 全角パターンテスト全削除

---

#### ✅ 議題 4: Grammar.md 改訂の現状確認（クローズ）

**確認結果**: grammar-specification.md は **既に Call のみの正しい状態**

**詳細**:
- 4 章（Call 詳細仕様）: ✅ Call のみ定義、Jump 記述なし
- 7 章（Sakura）: ✅ 字句仕様確定、全角削除反映
- 11 章（未確定事項）: ✅ すべて確定
- Jump 廃止: ✅ 既に反映済み（Call への統一）

**修正不要**: grammar-specification.md は Jump 廃止決定に既に準拠している

**Phase 3 タスク**: GRAMMAR.md（ユーザー向けドキュメント）の改訂は設計通り実施

**確認結果**: grammar-specification.md は **既に Call のみの正しい状態**

**詳細**:
- 4 章（Call 詳細仕様）: ✅ Call のみ定義、Jump 記述なし
- 7 章（Sakura）: ✅ 字句仕様確定、全角削除反映
- 11 章（未確定事項）: ✅ すべて確定
- Jump 廃止: ✅ 既に反映済み（Call への統一）

**修正不要**: grammar-specification.md は Jump 廃止決定に既に準拠している

**Phase 3 タスク**: GRAMMAR.md（ユーザー向けドキュメント）の改訂は設計通り実施

---

#### ✅ 議題 5: テスト層別化の Cargo 互換性確認（クローズ）

**確認結果**: Cargo 互換性 **問題なし** ✅

**理由**:
- Cargo は `tests/**/*.rs` すべてをテスト対象として自動検出 ✅
  - 命名規則変更（`*_test.rs` → `pasta_parser_*_test.rs` 等）は検出を阻害しない
- tests/common への相互参照は命名に影響なし ✅
- CI（GitHub Actions）も `cargo test` コマンド実行なので互換性あり ✅

**結論**: Phase 0 テスト層別化を安全に実行可能

---

#### ✅ 議題 6: 全決定事項の最終確認（クローズ）

**確認内容**: Issue 1-5 の決定内容が正しく文書に反映されているか

**検証結果**:

| 文書 | 反映内容 | 状態 |
|------|--------|------|
| requirements.md | Issue 2（Jump 廃止）を「必須」と記載 | ✅ 反映済み |
| design.md | Issues 1-3 の決定（Sakura 案B、Jump 廃止、全角削除）を明記 | ✅ 反映済み |
| grammar-specification.md | Call のみ定義、Jump 記述なし | ✅ 反映済み |
| test-hierarchy-plan.md | テスト層別化命名規則を確定 | ✅ 反映済み |

**矛盾・漏れ**: なし

**最終判定**: **GO** → Phase 0 実行開始可能

---

## 進捗管理

### 現在のクローズ状況
- ✅ Issue 1（Sakura 簡素化）: 決定済み（決定: 案 B + \] 必須） — コミット: ce3f232
- ✅ Issue 2（Jump 削除）: 決定済み（決定: Jump 廃止必須） — コミット: 451299b
- ✅ Issue 3（全角文字）: 決定済み（決定: Case A 完全削除） — コミット: f0499c9
- ✅ Issue 4（Grammar.md 改訂）: 確認完了（修正不要、既に準拠） — コミット: 65940c1
- ✅ Issue 5（Cargo 互換性）: 確認完了（Cargo 互換性あり、GO） — 自動判定
- ✅ Issue 6（最終確認）: 確認完了（全文書整合性 OK、GO） — 自動判定

### 要件定義フェーズ: 完了 ✅

**全決定事項整理**:

1. **Sakura スクリプト**: 半角のみ、字句解釈なし、`\]` エスケープ必須
2. **Jump マーカー**: 廃止、Call (`＞`) へ統一
3. **全角文字**: 完全削除、ハーフ幅のみ対応
4. **Grammar 仕様**: 既に決定に準拠（修正不要）
5. **テスト層別化**: Cargo 互換性確認（実行可能）

### 次のステップ
- ✅ すべての文書更新完了
- ✅ コミット記録完了
- 👉 **Phase 0 実行開始: テスト層別化・グリーン確認**


---

## 参考資料
- requirements.md
- gap-analysis-2025-12-19.md
- design.md
- test-hierarchy-plan.md
- grammar-specification.md

