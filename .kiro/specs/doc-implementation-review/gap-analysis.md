# Gap Analysis: doc-implementation-review

**分析日**: 2026-02-05  
**分析者**: GitHub Copilot (Claude Opus 4.5)

---

## 1. 現状調査

### 1.1 対象ドキュメント棚卸し

| カテゴリ     | ドキュメント                 | 存在 | 最終更新日（推定） |
| ------------ | ---------------------------- | ---- | ------------------ |
| **Level 0**  | README.md                    | ✅    | 最新               |
| **Level 1**  | SOUL.md                      | ✅    | 2026-01-25         |
| **Level 1**  | GRAMMAR.md                   | ✅    | 最新               |
| **Level 1**  | AGENTS.md                    | ✅    | 最新               |
| **品質管理** | TEST_COVERAGE.md             | ✅    | 更新必要           |
| **品質管理** | OPTIMIZATION.md              | ✅    | 2026-01-25         |
| **品質管理** | SCENE_TABLE_REVIEW.md        | ✅    | 2026-01-25         |
| **クレート** | pasta_core/README.md         | ✅    | 最新               |
| **クレート** | pasta_lua/README.md          | ✅    | 最新               |
| **クレート** | pasta_lua/LUA_API.md         | ✅    | 最新               |
| **クレート** | pasta_shiori/README.md       | ✅    | 最新               |
| **クレート** | pasta_sample_ghost/README.md | ✅    | 最新               |

### 1.2 実装状況サマリー

| 項目                | ドキュメント記載 | 実測値 | 状態     |
| ------------------- | ---------------- | ------ | -------- |
| 総テスト数          | 700+             | 736    | ⚠️ 要更新 |
| pasta_core テスト   | 91 → 94          | 94     | ✅ 整合   |
| pasta_lua テスト    | 464 → 257        | 226+α  | ⚠️ 要確認 |
| pasta_shiori テスト | 28               | 12+    | ⚠️ 要確認 |
| 完了仕様数          | 26件 → 31件      | 48件   | ❌ 要更新 |
| Phase 0状態         | 完了             | 完了   | ✅ 整合   |
| Rust edition        | 2024             | 2024   | ✅ 整合   |
| mlua version        | 0.11             | 0.11   | ✅ 整合   |

---

## 2. 要求との対応分析

### 2.1 Requirement 1: ドキュメント実装整合性評価

#### 検出された乖離

| ドキュメント         | 箇所               | 乖離内容                      | 重要度 |
| -------------------- | ------------------ | ----------------------------- | ------ |
| **TEST_COVERAGE.md** | テスト数           | 「700+テスト」→実測736テスト  | ⚠️ 軽微 |
| **TEST_COVERAGE.md** | クレート別テスト数 | pasta_lua 464→実測約230程度   | ❌ 重大 |
| **SOUL.md**          | Phase 0状態        | 「完了」記載、31件→実際は48件 | ⚠️ 軽微 |
| **product.md**       | 完了仕様数         | 「26件」記載→実際は48件       | ❌ 重大 |
| **structure.md**     | pasta_rune参照     | Runeは廃止済み、Luaのみ       | ❌ 重大 |
| **tech.md**          | pasta_rune記載     | Runeバックエンドは廃止済み    | ❌ 重大 |

### 2.2 Requirement 2: 重複・冗長性検出

#### 検出された重複

| 重複箇所         | ドキュメントA | ドキュメントB       | 推奨アクション                      |
| ---------------- | ------------- | ------------------- | ----------------------------------- |
| マーカー一覧表   | GRAMMAR.md    | steering/grammar.md | grammar.md → GRAMMAR.mdへ参照リンク |
| アーキテクチャ図 | README.md     | structure.md        | 統一して相互参照                    |
| Phase進捗        | SOUL.md       | product.md          | product.md を正規ソースに           |
| DoD定義          | SOUL.md       | workflow.md         | workflow.md を正規ソースに          |
| ディレクトリ構造 | README.md     | structure.md        | structure.md を正規ソースに         |

### 2.3 Requirement 3: 欠落ドキュメント特定

#### 検出された欠落

| 機能/領域                       | 現状                 | 推奨アクション                   |
| ------------------------------- | -------------------- | -------------------------------- |
| **CHANGELOG.md**                | 存在しない           | 追加推奨（リリース履歴用）       |
| **CONTRIBUTING.md**             | 存在しない           | 追加推奨（コントリビュータ向け） |
| **pasta.toml 完全リファレンス** | LUA_API.mdに一部記載 | 独立ドキュメント化検討           |
| **Luaモジュール一覧（Lua側）**  | 部分的               | LUA_API.mdに追記                 |
| **エラーコード一覧**            | 存在しない           | 将来的に追加                     |

### 2.4 Requirement 4: ドキュメントヒエラルキー整合性

#### SOUL.md定義 vs 実際

| ヒエラルキー定義 | ドキュメント           | 存在 | 課題       |
| ---------------- | ---------------------- | ---- | ---------- |
| Level 0          | README.md              | ✅    | -          |
| Level 1          | SOUL.md                | ✅    | -          |
| Level 1          | SPECIFICATION.md       | ✅    | 対象外     |
| Level 1          | GRAMMAR.md             | ✅    | -          |
| Level 1          | AGENTS.md              | ✅    | -          |
| Level 2          | pasta_core/README.md   | ✅    | -          |
| Level 2          | pasta_lua/README.md    | ✅    | -          |
| Level 2          | pasta_shiori/README.md | ✅    | -          |
| Level 3          | .kiro/steering/*       | ✅    | 一部陳腐化 |

**未登録ドキュメント**:
- TEST_COVERAGE.md（品質管理）
- OPTIMIZATION.md（品質管理）
- SCENE_TABLE_REVIEW.md（品質管理）
- pasta_lua/LUA_API.md（Level 2）
- pasta_sample_ghost/README.md（Level 2）

### 2.5 Requirement 5: クレートREADME整合性

#### pasta_core/README.md

| 項目             | 記載内容 | 実装   | 状態   |
| ---------------- | -------- | ------ | ------ |
| parse_str()      | ✅        | ✅ 存在 | ✅ 整合 |
| parse_file()     | ✅        | ✅ 存在 | ✅ 整合 |
| SceneRegistry    | ✅        | ✅ 存在 | ✅ 整合 |
| WordDefRegistry  | ✅        | ✅ 存在 | ✅ 整合 |
| SceneTable       | ✅        | ✅ 存在 | ✅ 整合 |
| RandomSelector   | ✅        | ✅ 存在 | ✅ 整合 |
| ディレクトリ構造 | ✅        | ✅ 一致 | ✅ 整合 |

#### pasta_lua/README.md

| 項目                | 記載内容 | 実装           | 状態   |
| ------------------- | -------- | -------------- | ------ |
| ディレクトリ構成    | ✅        | ✅ 一致         | ✅ 整合 |
| pasta.toml設定      | ✅        | ✅ 動作確認済み | ✅ 整合 |
| [actor.*]セクション | ✅        | ✅ 動作確認済み | ✅ 整合 |
| [lua] libs配列      | ✅        | ✅ 実装済み     | ✅ 整合 |

#### pasta_lua/LUA_API.md

| モジュール           | 記載 | 実装                     | 状態   |
| -------------------- | ---- | ------------------------ | ------ |
| @pasta_search        | ✅    | ✅ search/mod.rs          | ✅ 整合 |
| @pasta_persistence   | ✅    | ✅ runtime/persistence.rs | ✅ 整合 |
| @enc                 | ✅    | ✅ encoding/mod.rs        | ✅ 整合 |
| @pasta_config        | ✅    | ✅ runtime/mod.rs         | ✅ 整合 |
| @pasta_sakura_script | ✅    | ✅ sakura_script/mod.rs   | ✅ 整合 |
| @assertions          | ✅    | ✅ mlua-stdlib            | ✅ 整合 |
| @testing             | ✅    | ✅ mlua-stdlib            | ✅ 整合 |
| @regex               | ✅    | ✅ mlua-stdlib            | ✅ 整合 |
| @json                | ✅    | ✅ mlua-stdlib            | ✅ 整合 |
| @yaml                | ✅    | ✅ mlua-stdlib            | ✅ 整合 |

#### pasta_shiori/README.md

| 項目                | 記載内容 | 実装         | 状態   |
| ------------------- | -------- | ------------ | ------ |
| Shiori trait        | ✅        | ✅ shiori.rs  | ✅ 整合 |
| load/request/unload | ✅        | ✅ 実装済み   | ✅ 整合 |
| Windows DLL         | ✅        | ✅ windows.rs | ✅ 整合 |

### 2.6 Requirement 6: 品質管理ドキュメント整合性

#### TEST_COVERAGE.md

| 項目        | 記載       | 実測  | 状態         |
| ----------- | ---------- | ----- | ------------ |
| 総テスト数  | 700+       | 736   | ⚠️ 軽微な乖離 |
| pasta_core  | 94         | 94    | ✅ 整合       |
| pasta_lua   | 464        | 約230 | ❌ 要確認     |
| Phase 0 DoD | 完了       | 完了  | ✅ 整合       |
| 最終更新日  | 2025-06-24 | -     | ❌ 古い日付   |

#### OPTIMIZATION.md

| 項目                 | 記載 | 実装                         | 状態   |
| -------------------- | ---- | ---------------------------- | ------ |
| TCO最適化            | ✅    | ✅ code_generator.rs L320-460 | ✅ 整合 |
| アクター最適化       | ✅    | ✅ 実装済み                   | ✅ 整合 |
| 文字列リテラル最適化 | ✅    | ✅ string_literalizer.rs      | ✅ 整合 |

#### SCENE_TABLE_REVIEW.md

| 項目               | 記載 | 実装             | 状態   |
| ------------------ | ---- | ---------------- | ------ |
| RadixMap使用       | ✅    | ✅ scene_table.rs | ✅ 整合 |
| prefix_index       | ✅    | ✅ 実装済み       | ✅ 整合 |
| MockRandomSelector | ✅    | ✅ random.rs      | ✅ 整合 |

### 2.7 Requirement 7: Steeringドキュメント整合性

#### product.md

| 項目        | 記載   | 実際   | 状態     |
| ----------- | ------ | ------ | -------- |
| Phase 0完了 | ✅      | ✅      | ✅ 整合   |
| 完了仕様数  | 26件   | 48件   | ❌ 要更新 |
| Phase 2状態 | 進行中 | 進行中 | ✅ 整合   |

#### tech.md

| 項目               | 記載 | 実際       | 状態       |
| ------------------ | ---- | ---------- | ---------- |
| Rust 2024 edition  | ✅    | ✅          | ✅ 整合     |
| mlua 0.11          | ✅    | ✅          | ✅ 整合     |
| **pasta_rune記載** | ✅    | ❌ 廃止済み | ❌ 削除必要 |
| **Rune 0.14記載**  | ✅    | ❌ 使用せず | ❌ 削除必要 |

#### structure.md

| 項目               | 記載 | 実際         | 状態       |
| ------------------ | ---- | ------------ | ---------- |
| pasta_core構造     | ✅    | ✅ 一致       | ✅ 整合     |
| pasta_lua構造      | ✅    | ✅ 一致       | ✅ 整合     |
| **pasta_rune記載** | ✅    | ❌ 存在しない | ❌ 削除必要 |
| テスト配置規則     | ✅    | ✅ 一致       | ✅ 整合     |

---

## 3. 実装アプローチ評価

### Option A: 最小限の修正（推奨）

**対象**: 明確な乖離のみを修正

- **修正対象**:
  - TEST_COVERAGE.md: テスト数更新、日付更新
  - product.md: 完了仕様数を48件に更新
  - tech.md: pasta_rune関連記述を削除
  - structure.md: pasta_rune関連記述を削除

**Trade-offs**:
- ✅ 工数最小（S: 1-3日）
- ✅ リスク低（既存構造維持）
- ❌ 重複問題は残存

### Option B: 構造的リファクタリング

**対象**: 重複解消 + 欠落補完

- **修正対象**:
  - Option Aの全項目
  - 重複箇所の参照リンク化
  - SOUL.mdヒエラルキーに品質管理ドキュメント追加
  - CHANGELOG.md新規作成

**Trade-offs**:
- ✅ 保守性向上
- ✅ Single Source of Truth確立
- ❌ 工数中（M: 3-7日）
- ❌ 既存参照リンクの更新必要

### Option C: 完全再編成

**対象**: ドキュメント体系の再設計

- **修正対象**:
  - Option A/Bの全項目
  - ドキュメントヒエラルキー再定義
  - steering/*.mdの統廃合
  - 自動生成ドキュメント検討

**Trade-offs**:
- ✅ 長期的な保守性最大化
- ❌ 工数大（L: 1-2週）
- ❌ 既存ワークフローへの影響大

---

## 4. 工数・リスク評価

| 領域                     | 工数 | リスク | 根拠             |
| ------------------------ | ---- | ------ | ---------------- |
| TEST_COVERAGE.md更新     | S    | Low    | 数値置換のみ     |
| tech.md/structure.md更新 | S    | Low    | 削除のみ         |
| product.md更新           | S    | Low    | 数値更新         |
| 重複解消（参照リンク化） | M    | Medium | 既存参照への影響 |
| ヒエラルキー更新         | S    | Low    | SOUL.md追記のみ  |
| CHANGELOG.md新規作成     | M    | Low    | 新規ファイル     |

**総合評価**:
- **推奨アプローチ**: Option A + 一部Option B（重複解消）
- **総工数**: M（3-7日）
- **総リスク**: Low-Medium

---

## 5. 推奨アクションプラン

### High Priority（即時対応）

| #   | アクション | 対象             | 詳細                                |
| --- | ---------- | ---------------- | ----------------------------------- |
| 1   | **更新**   | tech.md          | pasta_rune/Rune関連記述を完全削除   |
| 2   | **更新**   | structure.md     | pasta_rune関連記述を完全削除        |
| 3   | **更新**   | product.md       | 完了仕様数を48件に更新              |
| 4   | **更新**   | TEST_COVERAGE.md | テスト数736、日付を2026-02-05に更新 |

### Medium Priority（今Phase中）

| #   | アクション | 対象                | 詳細                                               |
| --- | ---------- | ------------------- | -------------------------------------------------- |
| 5   | **更新**   | SOUL.md             | ドキュメントヒエラルキーに品質管理ドキュメント追加 |
| 6   | **統合**   | steering/grammar.md | GRAMMAR.mdへの参照リンクに置換                     |
| 7   | **追加**   | README.md           | ドキュメントマップにTEST_COVERAGE.md等追加         |

### Low Priority（将来検討）

| #   | アクション | 対象            | 詳細                              |
| --- | ---------- | --------------- | --------------------------------- |
| 8   | **追加**   | CHANGELOG.md    | リリース履歴管理用（Phase 4以降） |
| 9   | **追加**   | CONTRIBUTING.md | OSS公開時に作成                   |

---

## 6. 設計フェーズへの引継ぎ事項

### 確認済み事項

- ✅ クレートREADMEは全て実装と整合
- ✅ LUA_API.mdのモジュール記載は正確
- ✅ OPTIMIZATION.mdの最適化記載は実装と一致
- ✅ SCENE_TABLE_REVIEW.mdの設計記載は正確

### 要調査事項（Research Needed）

- [ ] pasta_luaテスト数の正確な内訳確認（464→230の差分原因）
- [ ] Lua側モジュール（pasta.*）の完全なAPI一覧

### 設計判断が必要な事項

- steering/*.md の粒度と責務範囲の再定義
- ドキュメント自動生成（rustdoc連携等）の検討

---

## 7. 結論

本Gap分析により、以下の知見を得ました：

1. **クレートドキュメント**: 全て実装と整合しており、品質が高い
2. **Steeringドキュメント**: pasta_rune廃止後の更新漏れが主要課題
3. **品質管理ドキュメント**: テスト数・完了仕様数の数値更新が必要
4. **重複問題**: GRAMMAR.md ⇔ steering/grammar.md が最も顕著

**推奨**: Option A（最小限修正）を基本とし、重複解消のみOption Bを適用する「A+α」アプローチが最適です。

---

## 8. 議題ディスカッション記録

### 議題1: steering/grammar.md と GRAMMAR.md の役割分離

**決定日**: 2026-02-05  
**ステータス**: ✅ 方針決定

**方針**:
- `GRAMMAR.md`: 人間向けマニュアル（読みやすさ優先、完全性不要）
- `steering/grammar.md`: AI向け完全参照（SPECIFICATION.md準拠、完全性が必要）

**Implementation時の作業内容**:
1. steering/grammar.mdの冒頭に以下を追記:
   ```markdown
   ## このドキュメントの役割
   
   **対象読者**: AI（GitHub Copilot等）
   **目的**: Pasta DSL実装時の完全な参照情報提供
   **権威的仕様**: SPECIFICATION.md（全11章、1,210行）
   
   人間向けの読みやすいマニュアルは `/GRAMMAR.md` を参照してください。
   ```

2. Runeブロック例をLuaブロック例に変更:
   ```pasta
   ＊計算
   ```lua
   function calculate(ctx)
       local result = 10 + 20
       return result
   end
   ```
   ```

3. SPECIFICATION.mdへの参照を各セクションに追加

**理由**:
- AI向けには完全性が求められる（SPECIFICATION.md準拠）
- 人間向けには読みやすさが優先（ロジカル完全性は不要）
- 役割の違いを明確化することで、両者の価値を保持

