# Gap Analysis: pasta-lua-skill

## 1. 現状調査

### 1.1 ドメイン関連アセット

| アセット | パス | 用途 |
|---------|------|------|
| Luaコーディング規約 | `.kiro/steering/lua-coding.md` (~650行) | 命名規約・モジュール構造・PASTA固有パターン |
| Lua APIリファレンス | `crates/pasta_lua/LUA_API.md` (~1200行) | Rust組み込みモジュール・SHIORI EVENT仕様 |
| 姉妹スキル (DSL層) | `.agents/skills/pasta-ghost-authoring/SKILL.md` (~378行) | 構造テンプレート・YAML Frontmatter形式の先例 |
| WORDモジュール | `crates/pasta_lua/scripts/pasta/word.lua` | ビルダーパターンAPI実装 |
| ACTモジュール | `crates/pasta_lua/scripts/pasta/shiori/act.lua` | シーン関数引数・トークン蓄積API |
| SCENEモジュール | `crates/pasta_lua/scripts/pasta/scene.lua` | シーン登録・検索・co_exec |
| GLOBALモジュール | `crates/pasta_lua/scripts/pasta/global.lua` | ユーザー定義関数テーブル |
| SAVEモジュール | `crates/pasta_lua/scripts/pasta/save.lua` | JSON永続化 |
| STOREモジュール | `crates/pasta_lua/scripts/pasta/store.lua` | 一元データ管理 |
| SHIORIイベント登録 | `crates/pasta_lua/scripts/pasta/shiori/event/register.lua` | REGテーブル登録API |
| 仮想ディスパッチャ | `crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua` | OnTalk/OnHour自動発行 |
| RESモジュール | `crates/pasta_lua/scripts/pasta/shiori/res.lua` | レスポンス生成関数 |
| サンプルゴースト辞書 | `crates/pasta_sample_ghost/dist-src/ghost/master/dic/*.pasta` (4ファイル) | 実践的なDSL使用例 |
| サンプルmain.lua | `crates/pasta_lua/scripts/main.lua` | ユーザースクリプトエントリーポイント |

### 1.2 既存の規約・パターン

- **SKILL.md構造規約**: 姉妹スキル `pasta-ghost-authoring` が確立済み
  - YAML Frontmatter: `name`, `description` (USE FOR / DO NOT USE FOR), `metadata` (author, version)
  - セクション構成: §1 Purpose → §2 Quick Reference → §3-§N ドメイン知識 → 最終§ パターン集
  - 合計行数: ~378行（本スキルも同程度～やや多い想定）

- **モジュール構造**: `MODULE` / `MODULE_IMPL` 分離パターン（lua-coding.md 準拠）
  - require → モジュールテーブル → ローカル関数 → 公開関数 → return
  - ドット構文定義（明示的self）、コロン構文呼び出し

- **ファイルI/Oパターン**: CSV/TSVネイティブ対応なし
  - 標準 `io.open()` + `string.gmatch()` で手動パース
  - `@json` / `@yaml` モジュールで構造化データ対応
  - `@enc` モジュールで UTF-8↔ANSI パス変換（Windows対応）

### 1.3 統合サーフェス

- **姉妹スキルとの境界**: DSL層（Pasta文法）はghost-authoring側、Lua層は本スキル側
  - 接点: Luaブロック（`` ```lua ``` ``）がDSL内に埋め込まれるケース
  - 単語定義の使い分け: DSLの `＠単語名：値1、値2` vs Luaの `WORD.create_*(key):entry(...)`

- **ランタイム依存**: スキル内容は `crates/pasta_lua/` の実装に依存するが、スキル自体はpastaリポジトリ外にコピーして使用するため、参照ではなく転記が必要

---

## 2. 要件実現可能性分析

### Requirement-to-Asset Map

| 要件 | 情報ソース | 状態 | メモ |
|------|-----------|------|------|
| **Req 1: スキルファイル構造** | pasta-ghost-authoring SKILL.md | ✅ 十分 | 姉妹スキルの構造をテンプレートとして再利用可能 |
| **Req 2: コーディング規約** | lua-coding.md | ✅ 十分 | ~650行の規約から必要部分を抽出・転記。整合性制約含む |
| **Req 3: ランタイムAPI** | LUA_API.md | ✅ 十分 | ~1200行のリファレンスから必要APIを転記。整合性制約含む |
| **Req 4: 内部Luaモジュール** | scripts/pasta/*.lua ソース | ✅ 十分 | 全モジュールのソースコードが利用可能。DSL→Luaブリッジ規約含む |
| **Req 5: SHIORIハンドラ** | register.lua, virtual_dispatcher.lua, LUA_API.md | ✅ 十分 | REG/RES/仮想ディスパッチャの実装とドキュメント両方あり |
| **Req 6: テスト・Lint** | lua-coding.md テストセクション | ✅ 十分 | lua_testフレームワーク仕様が記載済み |

### ギャップ・制約の特定

| ID | 種別 | 内容 |
|----|------|------|
| **Gap①** | Constraint | スキルは自己完結的であるべき（Req 1.5）。LUA_API.md (~1200行) を全文転記するとスキルが膎大になる。必要APIのみの抽出・要約が必要 |
| **Gap②** | Constraint | 姉妹スキルは~378行。本スキルはカバー範囲が広いため、行数管理が課題になる可能性がある |

### 複雑性シグナル

- **主な作業**: 既存ドキュメントからの情報抽出・体系化・転記（CRUDではなくドキュメントエンジニアリング）
- **外部統合**: なし（スキルファイルはMarkdownドキュメントのみ）

---

## 3. 実装アプローチオプション

### Option A: 単一 SKILL.md ファイル（推奨）

**検討条件**: 姉妹スキル `pasta-ghost-authoring` と同じ構造。成果物が1ファイルのみ。

- **構成**:
  - `.agents/skills/pasta-lua-coding/SKILL.md` に全情報を集約
  - YAML Frontmatter + 6〜7セクション構成
  - 想定行数: 400〜600行

- **互換性**:
  - 姉妹スキルと同一形式のため、VS Code Copilot Skill 機構で即座に利用可能
  - 別リポジトリへのコピー運用も同一手順

- **保守性**:
  - 1ファイルで完結するため、更新・レビューが容易
  - API変更時の追従箇所が1ファイルに集中

**トレードオフ**:
- ✅ 姉妹スキルとの一貫性が高い
- ✅ 運用手順が確立済み（コピー＆ペースト）
- ✅ VS Code Copilot の skill 機構で自動ロードされる
- ❌ 行数が多くなるとコンテキストウィンドウ圧迫の可能性
- ❌ 全セクションが常にロードされる（部分ロード不可）

### Option B: SKILL.md + 補助ファイル分割

**検討条件**: 情報量が単一ファイルに収まらない場合。

- **構成**:
  - `.agents/skills/pasta-lua-coding/SKILL.md` — メイン（概要・クイックリファレンス・トリガー）
  - `.agents/skills/pasta-lua-coding/API_REFERENCE.md` — APIリファレンス詳細
  - `.agents/skills/pasta-lua-coding/PATTERNS.md` — パターン集（外部データ投入含む）

- **統合**:
  - SKILL.md から補助ファイルを `read_file` で参照する指示を記載
  - VS Code Copilot が SKILL.md をトリガーした後、LLMが必要に応じて補助ファイルを読み込む

**トレードオフ**:
- ✅ 各ファイルが短く保てる
- ✅ 必要な部分のみの読み込みが可能
- ❌ VS Code Copilot Skill 機構は SKILL.md のみを自動ロードする（補助ファイルは手動参照が必要）
- ❌ 姉妹スキルとの構造的一貫性が崩れる
- ❌ コピー運用時にファイル漏れのリスク

### Option C: ハイブリッド（段階的拡張）

**検討条件**: 初回は Option A で開始し、行数が膨大になった場合に Option B へ移行。

- **フェーズ1**: 単一 SKILL.md で全要件をカバー（~500行目標）
- **フェーズ2**: 行数が600行を超えた場合、詳細APIリファレンスを補助ファイルに分離

**トレードオフ**:
- ✅ 初期は最小構成で迅速に完成
- ✅ 実際の行数を見てから分割判断できる
- ❌ 分割時にリネーム・再構成が発生

---

## 4. 実装複雑性・リスク

### 工数

**S（1〜3日）**

- 理由: 成果物はMarkdownドキュメント1ファイル（コード変更なし）。既存ドキュメント（lua-coding.md, LUA_API.md, モジュールソース）からpasta_lua固有APIの情報を抽出・転記・体系化する作業が中心。姉妹スキルの構造テンプレートが確立済み。

### リスク

**Low**

- 理由: 確立済みのパターン（姉妹スキル）に従う。成果物はドキュメントのみでコードを壊すリスクがない。情報ソースがすべて特定済み。唯一の不確実性は行数管理（Gap②）だが、Option C で段階的に対処可能。

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**Option A（単一 SKILL.md）** を推奨。姉妹スキル `llm-grammar-skill` でも単一ファイル方式で成功した先例がある。

### 設計フェーズでの重点事項

1. **セクション構成の確定**: 6要件を何セクションに集約するか。姉妹スキルの6セクション構成を参考に、6〜7セクションの目次を設計する
2. **情報量の圧縮戦略**: LUA_API.md (~1200行) と lua-coding.md (~650行) から、スキルに転記する情報の取捨選択基準を定義する（Gap①対応）
3. **DSL vs Lua 判断基準の設計**: Req 1.7 の「いつLuaを使うべきか」ガイドラインを設計する

### 持ち越しリサーチ項目

- なし（必要な情報ソースはすべて特定・確認済み）
