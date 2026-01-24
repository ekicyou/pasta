# Gap Analysis: soul-document

## 分析日時
2026-01-24

## 1. 現状調査（Current State Investigation）

### 1.1 既存ドキュメント資産

| ドキュメント | 状態 | ソウルドキュメント適性 |
|-------------|------|----------------------|
| [README.md](../../../README.md) | ✅ 存在 | アーキテクチャ概要あり、ビジョン不足 |
| [SPECIFICATION.md](../../../SPECIFICATION.md) | ✅ 存在（1,278行） | 技術仕様書、ソウル要素なし |
| [GRAMMAR.md](../../../GRAMMAR.md) | ✅ 存在（634行） | 文法リファレンス、学習用 |
| [AGENTS.md](../../../AGENTS.md) | ✅ 存在 | AI開発支援、ワークフロー中心 |
| [.kiro/steering/product.md](../../../.kiro/steering/product.md) | ✅ 存在 | **ビジョン・コアバリュー記載あり** |
| [.kiro/steering/tech.md](../../../.kiro/steering/tech.md) | ✅ 存在 | 技術スタック、設計哲学あり |
| [.kiro/steering/grammar.md](../../../.kiro/steering/grammar.md) | ✅ 存在 | DSL文法サマリー |

**発見**: `product.md` に「ビジョン」「コアバリュー」セクションが既に存在。ただし断片的で、テスト可能な形式になっていない。

### 1.2 既存テスト資産

#### テストファイル一覧（16ファイル）

| クレート | テストファイル | テスト数 | カテゴリ |
|---------|--------------|---------|---------|
| pasta_core | `actor_code_block_test.rs` | 3+ | Parser |
| pasta_core | `span_byte_offset_test.rs` | 不明 | Parser |
| pasta_lua | `transpiler_integration_test.rs` | 24 | Transpiler |
| pasta_lua | `search_module_test.rs` | 10+ | Registry |
| pasta_lua | `japanese_identifier_test.rs` | 2 | コアバリュー（UNICODE） |
| pasta_lua | `loader_integration_test.rs` | 不明 | Loader |
| pasta_lua | `finalize_scene_test.rs` | 不明 | Transpiler |
| pasta_lua | `actor_word_dictionary_test.rs` | 不明 | Transpiler |
| pasta_lua | `stdlib_modules_test.rs` | 不明 | Stdlib |
| pasta_lua | `stdlib_regex_test.rs` | 14 | Stdlib |
| pasta_lua | `ucid_test.rs` | 3 | コアバリュー（UNICODE） |
| pasta_lua | `pasta_lua_encoding_test.rs` | 不明 | Encoding |
| pasta_lua | `fallback_search_integration_test.rs` | 不明 | 前方一致検索 |
| pasta_lua | `lua_unittest_runner.rs` | Luaテスト実行 | メタテスト |
| pasta_shiori | `shiori_lifecycle_test.rs` | 5（**全失敗**） | SHIORI統合 |
| pasta_shiori | `lua_request_test.rs` | 18+ | SHIORI統合 |

#### テスト実行結果サマリー

```
pasta_core: 全テストパス
pasta_lua: 全テストパス（約80件）
pasta_shiori: 5件失敗（shiori_lifecycle_test）
```

### 1.3 テストフィクスチャ資産

| ディレクトリ | ファイル数 | 用途 |
|-------------|-----------|------|
| `tests/fixtures/parser2/` | 3 | 基本文法テスト |
| `tests/fixtures/transpiler2/` | 9 | トランスパイル検証 |
| `tests/fixtures/shiori/` | 不明 | SHIORI統合テスト |
| `crates/pasta_lua/tests/fixtures/` | 5 | Luaサンプル |
| `crates/pasta_lua/tests/lua_specs/` | 4 | Lua仕様テスト |

---

## 2. 要件実現可能性分析（Requirements Feasibility）

### Requirement 1: ソウルドキュメント体系の定義

| 受け入れ基準 | 既存資産 | ギャップ |
|-------------|---------|---------|
| ビジョン・ミッション定義 | `product.md` に断片あり | 体系的整理が必要 |
| コアバリュー明文化 | `product.md` に表形式あり | テスト可能形式に変換必要 |
| 行指向文法原則 | `SPECIFICATION.md` にあり | 参照リンク設定のみ |
| 前方一致設計思想 | `grammar.md` にあり | 設計思想として再構成必要 |
| UI独立性原則 | `tech.md` に記載あり | 設計原則として明示化必要 |

**ギャップ**: 既存資産を再構成・統合する作業が主。新規作成は少ない。

### Requirement 2: コア機能テスト群

| テスト対象 | 既存テスト | 状態 |
|-----------|----------|------|
| グローバルシーン（＊） | `transpiler_integration_test.rs` | ✅ 完了 |
| ローカルシーン（・） | `transpiler_integration_test.rs` | ✅ 完了 |
| アクション行 | `transpiler_integration_test.rs` | ✅ 完了 |
| 変数スコープ | `transpiler2/variable_scope.pasta` | 🔶 フィクスチャあり、テスト要確認 |
| Call文（＞） | `comprehensive_control_flow.pasta` | 🔶 フィクスチャあり |
| 単語定義・参照（＠） | `actor_word_dictionary_test.rs` | ✅ 完了 |
| 属性定義（＆） | `transpiler2/attribute_inheritance.pasta` | 🔶 フィクスチャあり |
| コメント行（＃） | 不明 | ⚠️ 要調査 |
| 前方一致検索 | `search_module_test.rs`, `fallback_search_integration_test.rs` | ✅ 完了 |
| ランダム選択 | `search_module_test.rs` | 🔶 部分的 |
| Luaトランスパイル | `transpiler_integration_test.rs`（24件） | ✅ 完了 |

**ギャップ**: 既存テストをソウルドキュメント機能にマッピングする作業が必要。

### Requirement 3: 機能マッピングレポート

| 必要機能 | 既存資産 | ギャップ |
|---------|---------|---------|
| テスト⇔機能対応表 | なし | **新規作成必要** |
| 実装状態カテゴリ表示 | なし | **新規作成必要** |
| Phase別進捗サマリー | `product.md` に手動記載 | 自動化検討 |
| 自動マッピング更新 | なし | **Research Needed** |

**ギャップ**: レポート機能は完全に新規開発が必要。

### Requirement 4: 既存テスト整理・分類

| 必要作業 | 既存状態 | ギャップ |
|---------|---------|---------|
| カテゴリ分類 | ファイル名から推測可能 | ドキュメント化必要 |
| 機能明示ドキュメント | なし | **新規作成必要** |
| 複数機能テスト区別 | なし | 手動分析必要 |
| 未カバー機能リスト | なし | 分析結果から生成 |

### Requirement 5: 整合性維持ワークフロー

| 必要機能 | 既存資産 | ギャップ |
|---------|---------|---------|
| 新機能→テスト要件生成 | なし | **Research Needed** |
| チェックリスト | なし | **新規作成必要** |
| 削除アラート | なし | **Research Needed** |
| 定期レビュー手順 | `workflow.md` に基盤あり | 拡張必要 |

### Requirement 6: Phase 0課題連携

| 必要機能 | 既存資産 | ギャップ |
|---------|---------|---------|
| Phase 0課題の「あるべき姿」 | なし | **新規作成必要** |
| リグレッションテスト | 一部あり | 明示的タグ付け必要 |
| 現状⇔目標差分 | `product.md` に手動記載 | 構造化必要 |
| 過去仕様の知見反映 | `.kiro/specs/completed/` に31件 | 抽出・統合必要 |

---

## 3. 実装アプローチオプション

### Option A: 既存ドキュメント拡張

**概要**: `product.md`, `tech.md` を拡張してソウルドキュメント化

**拡張対象**:
- `product.md` → ソウルドキュメント本体に昇格
- テストマッピングは別ファイル `TEST_COVERAGE.md` として追加
- 既存steering構造を維持

**トレードオフ**:
- ✅ 既存構造を活用、移行コスト最小
- ✅ steeringとの整合性維持
- ❌ ドキュメント肥大化リスク
- ❌ ソウルドキュメントとしての独立性不足

### Option B: 新規ソウルドキュメント作成

**概要**: `SOUL.md` を新規作成し、テストカバレッジレポートを別途構築

**新規作成**:
- `/SOUL.md` - ソウルドキュメント本体
- `/TEST_COVERAGE.md` - テストカバレッジマッピング
- `.kiro/steering/soul.md` - ソウルドキュメントサマリー（steering用）

**トレードオフ**:
- ✅ 明確な責務分離
- ✅ ソウルドキュメントの独立性
- ❌ ドキュメント重複リスク
- ❌ 既存 `product.md` との整合性維持コスト

### Option C: ハイブリッドアプローチ（推奨）

**概要**: 既存資産を再構成し、新規レポート機能を追加

**フェーズ1（ドキュメント整理）**:
- `product.md` を拡張してソウルドキュメント要素を強化
- 既存の断片を体系的に再構成

**フェーズ2（テストマッピング）**:
- `TEST_COVERAGE.md` を新規作成
- 既存テストの機能分類を実施

**フェーズ3（整合性維持）**:
- `workflow.md` にソウルドキュメント保守手順を追加
- チェックリストを定義

**トレードオフ**:
- ✅ 既存資産活用 + 新規価値追加
- ✅ 段階的導入が可能
- ❌ 計画的なフェーズ管理が必要
- ❌ 一時的な不整合期間が発生

---

## 4. 複雑性・リスク評価

### 工数見積もり

| 要件 | 工数 | 根拠 |
|-----|------|------|
| Req 1: ソウルドキュメント体系 | **S**（1-3日） | 既存資産の再構成が主 |
| Req 2: コア機能テスト群 | **M**（3-7日） | 既存テストのマッピング + 一部新規 |
| Req 3: マッピングレポート | **M**（3-7日） | 新規作成、フォーマット設計含む |
| Req 4: テスト整理・分類 | **S**（1-3日） | 分析とドキュメント化 |
| Req 5: 整合性維持ワークフロー | **M**（3-7日） | 手順設計 + 自動化検討 |
| Req 6: Phase 0連携 | **S**（1-3日） | 既存Phase 0課題との紐付け |

**総合工数**: **M〜L**（7-14日）

### リスク評価

| リスク | レベル | 軽減策 |
|-------|-------|-------|
| ドキュメント重複 | Medium | Option C採用で責務明確化 |
| テストマッピング漏れ | Low | 段階的レビューで検証 |
| 既存テスト失敗（pasta_shiori） | Medium | 本仕様スコープ外として分離 |
| Phase 0との競合 | Low | Phase 0が指針として本仕様を参照する構造 |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option C: ハイブリッドアプローチ**

### 重要決定事項
1. ソウルドキュメントの配置場所：`product.md` 拡張 vs 新規 `SOUL.md`
2. テストカバレッジレポートのフォーマット：Markdown表 vs 自動生成
3. 整合性維持の自動化レベル：手動チェックリスト vs CI統合

### Research Needed（設計フェーズで調査）
- [ ] 既存テスト数の正確なカウント
- [ ] テスト⇔機能の詳細マッピング
- [ ] 自動マッピング更新の実現可能性
- [ ] Phase 0課題の詳細リスト抽出

### 次のステップ
1. 本ギャップ分析をレビュー
2. `/kiro-spec-design soul-document` で設計フェーズへ進む

---

## 付録: 既存テスト⇔コア機能 初期マッピング

| コア機能 | 対応テストファイル | 状態 |
|---------|-------------------|------|
| 行指向文法 | `parser/` 内部テスト | ✅ |
| グローバルシーン | `transpiler_integration_test.rs` | ✅ |
| ローカルシーン | `transpiler_integration_test.rs` | ✅ |
| 日本語識別子 | `japanese_identifier_test.rs`, `ucid_test.rs` | ✅ |
| 前方一致検索 | `search_module_test.rs`, `fallback_search_integration_test.rs` | ✅ |
| 単語定義・参照 | `actor_word_dictionary_test.rs` | ✅ |
| Luaトランスパイル | `transpiler_integration_test.rs` | ✅ |
| SHIORI統合 | `shiori_lifecycle_test.rs` | ❌（失敗中） |
| 変数スコープ | フィクスチャあり | 🔶 |
| 属性継承 | フィクスチャあり | 🔶 |
