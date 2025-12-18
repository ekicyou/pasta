# Implementation Gap Analysis Report

## Document Information
- **Feature**: pasta-grammar-specification
- **Analysis Date**: 2025-12-18
- **Language**: ja
- **Current State**: brownfield project (GRAMMAR.md exists but is outdated)

---

## 1. Current State Investigation

### 1.1 Key Assets and Architecture

**Existing Documentation**:
- `GRAMMAR.md` (625 行) — 一般向けDSL文法リファレンス（現在、実装との乖離あり）
- `src/parser/pasta.pest` (329 行) — Pest PEG 文法定義（実装の真実の源泉）
- `src/parser/ast.rs` (301 行) — AST型定義（Statement, LabelDef, WordDef等）
- `AGENTS.md` — AI開発支援ドキュメント

**Existing Parser Layer** (`src/parser/`):
- `mod.rs` — Parser API公開インターフェース
- `ast.rs` — AST型定義（完全に定義済み）
- `pasta.pest` — Pest文法定義（30種類以上の production rule）

**Transpiler Layer** (`src/transpiler/`):
- `mod.rs` — Transpiler API
- `label_registry.rs` — ラベル管理・モジュール生成

**Runtime and IR** (`src/runtime/`, `src/ir/`):
- `generator.rs` — Rune VM実行
- `ir.rs` — ScriptEvent IR出力

**Test Fixtures**:
- `tests/fixtures/comprehensive_control_flow.pasta` — 複雑な例
- `tests/fixtures/comprehensive_control_flow.transpiled.rn` — トランスパイル結果参考

### 1.2 Naming and Conventions

**Rust側**:
- スネークケース（snake_case）識別子
- モジュール: `mod.rs`またはファイル名
- テストファイル: `<feature>_test.rs`（単数形、アンダースコア区切り）

**Pasta DSL側**:
- 日本語識別子、全角記号推奨（`＊ラベル`, `＠単語`, `＄変数`）
- マーカー: `＊`(グローバルラベル), `・`(ローカルラベル), `＞`(Call), `？`(Jump)

### 1.3 Integration Surfaces

**Parser → AST**:
- Pest rules → PastaFile struct （top_level_line、global_label、statement等の変換）

**AST → Transpiler**:
- Statement enum（Speech, VarAssign, Call, Jump, RuneBlock等）→ Rune IR

**Transpiler → Runtime**:
- Rune code → ScriptEvent stream (Generator継続で yield)

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs Mapping

| 要件 | 技術的ニーズ | 既存コンポーネント | ギャップ |
|-----|-----------|----------------|--------|
| **Req 1**: Pest文法分析 | pest.pest の全ルール把握 | pasta.pest完全定義済み | Minor: 詳細な説明文追加が必要 |
| **Req 2**: 文法仕様再検討 | あるべき仕様設計、破壊的変更評価 | なし | High: 新規設計が必要 |
| **Req 3**: GRAMMAR.md乖離抽出 | 比較表作成、差分検出 | 部分的（GRAMMAR.md存在）| Major: 体系的な乖離分析なし |
| **Req 4**: テスト修正計画 | リグレッション影響範囲、修正戦略 | テストフレームワーク既存 | High: 破壊的変更対応が大規模 |
| **Req 5**: 実装仕様説明 | pest定義のコメント化 | コメント部分的あり | Medium: コメント完成度向上が必要 |
| **Req 6**: ユーザー向けGRAMMAR.md改訂 | 例文作成、ユースケース記述 | GRAMMAR.mdの骨組みあり | Major: 内容の大幅改訂が必要 |
| **Req 7**: 同期メカニズム | pest定義とドキュメント連携 | なし | High: 新規実装が必要 |
| **Req 8**: 検証プロセス | テスト・クロスリファレンス | テストフレームワーク既存 | Medium: 検証ロジック追加が必要 |

### 2.2 破壊的変更に伴うテスト修正の複雑性

**影響を受けるテストファイル群**（初期推定）:
- `tests/parser_tests.rs` — Parse結果構造変更に伴い、多数のアサーション修正が必要
- `tests/transpile_comprehensive_test.rs` — 文法変更に伴う IR 出力変更
- `tests/comprehensive_control_flow_test.rs` — Call/Jump 構文が変更される場合の影響大
- `tests/sakura_script_tests.rs` — マーカー記号変更の影響
- `tests/fixtures/*.pasta` — テストフィクスチャ全体の見直しが必要
- 統合テスト（`engine_integration_test.rs`, `end_to_end_simple_test.rs` 等）

**リグレッション修正の推定規模**（要件 Req 4 スコープ）:
- ❌ 高リスク：破壊的変更によりテスト60-80%が失敗する可能性
- ⚠️ 修正工数：5-15日（修正内容による）
### 2.3 既存実装の課題と破壊的変更の必要性

**既存実装の課題**:
1. **pest定義のコメント不足** — production rules が簡潔だが、目的や制約の説明不足
2. **GRAMMAR.mdの不完全性**:
   - Rune ブロック記述が曖昧（実験的機能扱い）
   - ブロックコメント記法が存在するが pest定義に反映されていない
   - 一部削除された機能（「条件分岐」の if/elif/else）が記述されている
   - 同期マーカー（`@同時発言開始`等）の実装状況不明

3. **実装と乖離の例**:
   - `// ブロックコメント` は GRAMMAR.md に記載（`/* */` 記法）だが、pest定義には無い
   - 「ローカル関数」の`@@サブ処理`記法が GRAMMAR.md に存在するが、pest には`・`（local_label_marker）のみ
   - `@呼び出したいラベル`と`＠＊＊サブ処理` の記法が曖昧
   - **ローカルラベル記法の曖昧性**: `・` vs `ー` vs `＊＊` の混用

**破壊的変更が必要な箇所**（推定、設計段階での確定が必要）:
- ローカルラベル記法の統一（例：`・` に統一、`ー` `＊＊` は廃止）
- ブロックコメント（`/* */`）の正式サポート or 削除の決定
- 関数呼び出し記法の統一（`@ラベル` vs `＠＊ラベル` vs `＠＊＊ラベル` の明確化）
- Rune ブロック実装ステータスに基づくドキュメント整備

4. **Pest定義の複雑性**:
   - sakura_command パターンが5種類のバリエーションを持つ（括弧、数字、パターンマッチング）
   - Unicode 識別子対応（Hiragana, Katakana, CJK, Hangul等）がサポートされている
   - Full-width/half-width マーカー二重対応（`＠` と `@`等）

---

## 3. Implementation Approach Options

### Option A: 既存コンポーネント拡張（コメント充実化中心）

**対象コンポーネント**:
- `src/parser/pasta.pest` に詳細なコメント追加
- GRAMMAR.md 既存セクション改訂
- テスト例の拡充

**実装範囲**:
1. pest.pest 全 production rule に説明コメント追加（目的、制約、バリエーション）
2. GRAMMAR.md セクションごとの正確性確認と修正
3. `tests/fixtures/` に乖離箇所を実証するテストケース追加

**互換性**:
- ✅ 既存API変更なし、backward compatible
- ✅ 既存テストへの影響なし
- ✅ パーサー動作変更なし

**複雑性**:
- GRAMMAR.md は全体改訂が必要（骨組みは OK）
- pest 定義理解度が高い必要性

**トレードオフ**:
- ✅ 実装期間短い（既存構造活用）
- ✅ リスク低（追加のみ、削除なし）
- ❌ GRAMMAR.md のユーザー向け改訳は不完全な可能性
- ❌ 同期メカニズム（Req 5）は別途実装が必要

---

### Option B: 新規ドキュメント生成（テンプレート+自動化）

**新規ファイル**:
- `.kiro/docs/pest-grammar-reference.md` — 開発者向け技術リファレンス
- `.kiro/docs/grammar-revision-checklist.md` — 改訂チェックリスト
- `src/parser/GRAMMAR_REFERENCE.rs` — Rustdoc コメント（API側）

**実装範囲**:
1. pest.pest を分析し、production rule → markdown ドキュメント自動生成
2. GRAMMAR.md を新規テンプレートから再生成
3. 乖離検出ツール・スクリプトを作成

**互換性**:
- ✅ 既存ドキュメント保持（GRAMMAR.md は新版で置き換え）
- ✅ 新規ファイル追加のみ

**複雑性**:
- 高い：pest → markdown 変換ロジック実装
- pest AST パースが必要

**トレードオフ**:
- ✅ 将来の同期メカニズムへ基盤提供
- ✅ 自動化により保守負荷低下
- ❌ 初期実装期間長い
- ❌ pest → markdown 変換の精度確保が課題

---

### Option C: ハイブリッド（段階的改訂 + 自動化基盤構築）

**Phase 1**: 即座改訂（Option A の軽量版）
- pest.pest にコメント追加
- GRAMMAR.md 高優先度セクション改訂（ラベル、発言、制御）
- 明確な乖離箇所を修正

**Phase 2**: 自動化基盤構築
- `.kiro/tools/grammar-validator.rs` — pest ↔ GRAMMAR.md 検証ツール
- 同期メカニズム実装（Req 5）
- テスト拡充

**Phase 3**: 完全改訂
- 検証ツールの出力に基づき全セクション改訂
- ユーザー向け例文・ユースケース追加
- 最終検証

**実装範囲**:
段階的なため、各フェーズで明確な成果物

**互換性**:
- ✅ 各フェーズが独立した改善を提供
- ✅ 既存機能変更なし

**複雑性**:
- 中程度（各フェーズは独立）
- 段階ごとに知見が蓄積

**トレードオフ**:
- ✅ リスク最小化
- ✅ 段階的な品質向上
- ✅ 並行作業可能（Phase 1 後、Phase 2-3 可能）
- ❌ 全体完成期間は長い

---

## 4. Requirement-to-Asset Gap Map

| Requirement | Existing Assets | Gap Type | Details |
|-------------|-----------------|----------|---------|
| Req 1: Pest 分析 | pasta.pest 完全定義 | Minor | コメント詳細化のみ必要 |
| Req 2: 乖離抽出 | GRAMMAR.md, pasta.pest | Major | 比較分析なし、自動化ツール無し |
| Req 3: 仕様説明 | pasta.pest + AST | Medium | コメント不足、特に複雑ルール |
| Req 4: ユーザー文書 | GRAMMAR.md 骨組み | Major | 内容改訂、例文追加必要 |
| Req 5: 同期機構 | テストフレームワーク | High | 新規実装必須 |
| Req 6: 検証プロセス | 統合テスト基盤 | Medium | 検証ロジック追加必要 |

---

## 5. Implementation Complexity & Risk Assessment

### Effort Estimate

| 要件 | 難易度 | 工数 | 根拠 |
|-----|-------|------|------|
| Req 1: Pest 分析 | S | 1-2日 | pest 分析のみ（既存構造活用） |
| Req 2: 文法仕様再検討 | M-L | 3-7日 | 設計検討、破壊的変更評価 |
| Req 3: GRAMMAR.md乖離抽出 | M | 2-4日 | 比較分析、乖離リスト作成 |
| **Req 4: テスト修正計画** | **L** | **5-15日** | **破壊的変更に伴うリグレッション対応（大規模）** |
| Req 5: 仕様説明作成 | M | 2-4日 | pest コメント化 or 開発者リファレンス |
| Req 6: ユーザー文書改訂 | L | 5-10日 | 内容改訂 + 例文作成 + ユースケース |
| Req 7: 同期メカニズム | M-L | 3-6日 | 設計・実装 |
| Req 8: 検証プロセス | M | 2-4日 | 検証ツール + テスト追加 |
| **総計** | **M-L** | **25-52日** | **テスト修正が支配的 (5-15日 = 20-29%)** |

### Risk Assessment

| リスク要素 | リスク度 | 影響 | 対策 |
|----------|--------|------|-----|
| **テスト修正の規模** | **High** | **プロジェクト期間延伸、ビルド失敗** | **Req 4 の優先度化、修正計画の段階化、自動修正スクリプト検討** |
| pest 定義理解度不足 | Medium | 誤ったコメント追加 | pest公式ドキュメント参照、既存テストで検証 |
| 破壊的変更の不確定性 | High | 計画外のテスト修正 | 設計段階での詳細な破壊的変更リスト化 |
| GRAMMAR.md 改訳の完全性 | High | ユーザー混乱 | 複数レビュー、テスト例拡充 |
| 同期メカニズム設計 | Medium | 将来保守負荷 | 設計段階での詳細検討、設計文書化 |
| 自動化ツールの精度 | Medium | 偽陽性/偽陰性 | ツール開発後の複数ケースでの検証 |

---

## 6. Recommendations for Design Phase

### 推奨アプローチ: **Option C（ハイブリッド段階的改訂）+ テスト優先化**

**理由**:
- Phase 0（一次設計再構築）の現況において、リスク最小化と品質確保が重要
- テスト修正（Req 4）が全体工数の25-30%を占める重大要因
- 破壊的変更を早期に特定し、テスト修正計画を並行で進めることが必須
- 各フェーズで明確な検証ポイント設置可能

### Key Design Decisions Required

1. **Req 2（文法仕様再検討）の実施順序**:
   - **推奨**: Req 1 と並行して即座実施
   - 設計段階で「どの記法が廃止されるか」を明確化し、テスト修正計画を立案
   - 破壊的変更の「確定リスト」を作成し、テスト影響範囲を把握

2. **Req 4（テスト修正計画）の優先度化**:
   - **推奨**: 設計段階で詳細な修正計画を作成
   - テストファイルごとの「修正必須」「修正方法」「修正順序」を決定
   - 修正工数が大きいため、早期投資が全体リスク削減につながる
   - テスト修正の段階化:
     - Phase 1: Parser テスト修正（基盤）
     - Phase 2: Transpiler テスト修正（中間層）
     - Phase 3: Engine / 統合テスト修正（最上位層）

3. **Req 5（同期メカニズム）の実装形式**:
   - Option A: pest.pest コメント行 → 特定マーカー（例: `// ref: GRAMMAR.md#section-1`）
   - Option B: 別ファイル（`.kiro/tools/grammar-index.toml`）で pest ↔ GRAMMAR 対応記録
   - **推奨**: Option B（疎結合、拡張性向上、テスト修正計画にも流用可）

4. **GRAMMAR.md 改訳の優先順位**:
   - 高優先: ラベル定義、発言文、制御構文（Call/Jump）— テスト修正対応の進行に合わせて更新
   - 中優先: 変数、属性、単語定義
   - 低優先: Rune ブロック、同期セクション、イベントハンドリング（Phase 0 での曖昧性あり）

### Research Items for Design

1. **破壊的変更の詳細リスト化**（Req 2 スコープ）:
   - 廃止予定の記法（`ー`, `＊＊` など）
   - 統一予定の記法（例：ローカルラベルを `・` に統一）
   - 新規追加予定の機能

2. **Rune ブロック機能の実装状況確認**:
   - 現在の pasta.pest では `rune_block_content` を定義中だが、GRAMMAR.md 記載は「将来実装予定」
   - 実装ステータスと仕様の正確なマッピングが必要

3. **同期・イベント機能**:
   - `@同時発言開始`、`OnClick` 等のマーカーが GRAMMAR.md に記載されているが、pest 定義にない
   - 実装計画を確認し、ドキュメント対応範囲を決定

4. **ローカルラベル記法の統一戦略**:
   - GRAMMAR.md: `・`, `ー`, `＊＊` 混在記述
   - pest.pest: `local_label_marker = { "・" | "-" }`
   - 正規記法を統一・明確化し、廃止予定記法を文書化

---

## 7. Implementation Strategy

### Phase 1: 仕様決定と破壊的変更リスト化 (1-2 日)
- **Duration**: 1-2 日
- **目的**: 破壊的変更を確定し、テスト修正計画の基盤を整える
- **Deliverables**:
  - 破壊的変更の確定リスト（廃止記法、統一方針、新規機能等）
  - テスト影響範囲の初期推定（修正テストファイル、修正ケース数）
  - テスト修正計画書（段階、優先順位、工数見積）

### Phase 2: Pest分析 + 文法仕様説明 (2-4 日)
- **Duration**: 2-4 日
- **Deliverables**:
  - pasta.pest コメント化 (50% 完成)
  - 開発者向け技術リファレンス草案
  - 乖離箇所リスト（`.kiro/specs/pasta-grammar-specification/gap-findings.md`）

### Phase 3: Parser テスト修正 (3-7 日)
- **Duration**: 3-7 日 （破壊的変更の大きさに依存）
- **Deliverables**:
  - `tests/parser_tests.rs` 修正・合格
  - `tests/fixtures/*.pasta` 修正
  - Parser レイヤーテスト 100% 合格

### Phase 4: Transpiler + Engine テスト修正 (2-5 日)
- **Duration**: 2-5 日
- **Deliverables**:
  - `tests/transpile_comprehensive_test.rs` 修正
  - `tests/engine_integration_test.rs` 修正
  - 統合テスト 100% 合格

### Phase 5: ドキュメント完成 (5-10 日)
- **Duration**: 5-10 日
- **Deliverables**:
  - pasta.pest 100% コメント完成
  - GRAMMAR.md 全セクション改訂 + 例文拡充
  - 同期メカニズム実装

---

## 8. Unknowns and Open Questions

- ⚠️ 破壊的変更の具体的な範囲は？ (テスト修正工数を大きく左右)
- ⚠️ ローカルラベル記法の最終決定は？ (単一形式に統一すると、既存スクリプトが影響)
- ⚠️ Rune ブロック機能は「実験的」か「完全実装」か？ (Req 6 ドキュメント範囲に影響)
- ⚠️ 同期・イベント機能の実装予定時期は？ (ドキュメント対応範囲の確定に影響)
- ⚠️ テスト修正の自動化は可能か？ (Python/Rust スクリプトで対応箇所を自動検出)

---

## Conclusion

**Current Gap Summary**:
- ✅ Pest 定義は完全に存在（foundation 強固）
- ❌ GRAMMAR.md は部分的に陳旧化（高優先度改訂必要）
- ⚠️ 実装 ↔ ドキュメント同期メカニズムなし（今後の保守課題）

**Recommended Path**:
**Option C (ハイブリッド段階的改訂)** により、リスク最小化を図りながら段階的に品質向上。Phase 1 で即座改訳、Phase 2 で自動化基盤構築、Phase 3 で完全改訂。

**Next Step**: 設計フェーズで Phase 1-3 の詳細タスク分解、Rune ブロック等の未確定項目の技術調査を実施。
