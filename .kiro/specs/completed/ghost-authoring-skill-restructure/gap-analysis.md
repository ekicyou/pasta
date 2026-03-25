# ギャップ分析レポート: ghost-authoring-skill-restructure

## 1. 現状の資産マップ

### 1.1 ターゲットスキル構造

```
.agents/skills/pasta-ghost-authoring/
├── SKILL.md                  (~620行, v1.3.0)
└── references/
    ├── grammar-model.md      (~240行)
    ├── action-line.md        (~220行)
    ├── words.md              (~150行)
    ├── variables.md          (~210行)
    ├── actor-dictionary.md   (~150行)
    ├── call-spec.md          (~150行)
    └── sakura-script.md      (~120行)
```

### 1.2 SKILL.md セクション構成

| セクション | 行範囲 | 行数 | references/ リンク | 内容タイプ | 圧縮可能性 |
|-----------|--------|------|-------------------|-----------|-----------|
| §1 Purpose & Prerequisites | 24-39 | 16 | ❌ | 要約のみ | 不要 |
| §2 Quick Reference | 41-55 | 15 | ✅ grammar-model.md | テーブル | 不要 |
| §3 DSL Syntax (3.1-3.9) | 57-345 | 289 | ✅ 7ファイル | 混合 | 部分的（既に参照構造あり） |
| §4 Project Structure | 347-383 | 37 | ❌ なし | 混合 | **🔴 要改善** |
| §5 Event Mapping | 385-415 | 31 | ❌ | 要約のみ | 不要 |
| §6 Authoring Patterns | 417-620 | 204 | ❌ | 詳細パターン集 | 部分的 |

### 1.3 pasta.toml セクション構造

**2階層アーキテクチャ（エンジンコード検証済み）:**

| 階層 | セクション | 解析方式 |
|------|-----------|---------|
| **エンジン正式解析** | `[loader]`, `[logging]`, `[persistence]`, `[lua]`, `[talk]` | Rust 構造体にデシリアライズ、型検証あり |
| **カスタムフィールド** | `[package]`, `[ghost]`, `[actor."名前"]` | `custom_fields` として Lua に透過的に渡される（`@pasta_config` 経由） |

> **設計上の含意**: カスタムフィールドセクションのキーはエンジン検証なし。Lua 側の慣例的使用法がドキュメントの根拠となる。

### 1.4 pasta.toml ドキュメントカバレッジ

**SKILL.md §4 で記載されている設定項目（6項目）:**

| セクション | キー | ✅/❌ |
|-----------|------|------|
| `[loader]` | `pasta_patterns` | ✅ |
| `[ghost]` | `talk_interval_min` | ✅ |
| `[ghost]` | `talk_interval_max` | ✅ |
| `[actor."名前"]` | `spot` | ✅ (2例) |

**実在するが未記載の設定項目（~24項目）:**

| セクション | 未記載キー |
|-----------|----------|
| `[package]` | `name`, `version`, `edition` |
| `[loader]` | `lua_search_paths`, `transpiled_output_dir`, `debug_mode` |
| `[logging]` | `file_path`, `rotation_days`, `level`, `filter` |
| `[lua]` | `libs` |
| `[ghost]` | `hour_margin`, `spot_newlines` |
| `[talk]` | `script_wait_normal`, `script_wait_period`, `script_wait_comma`, `script_wait_strong`, `script_wait_leader` |
| `[actor."名前"]` | `default_surface`, **`budoux`** |
| `[persistence]` | `data_dir`, `obfuscate`, `file_path`, `debug_mode` |

**カバレッジ: 6/30 ≈ 20%**

---

## 2. 要件ごとのギャップ分析

### Req 1: グローバル変数の永続化メカニズム説明追加

| 項目 | 現状 | ギャップ |
|------|------|---------|
| SKILL.md §3.4 | 「永続的に有効」の1行のみ | **Missing**: SAVE テーブル経由のファイル永続化であることの説明 |
| references/variables.md | スコープ表に「永続的」の1語のみ | **Missing**: JSON ファイル保存の仕組み、保存タイミング |
| クロスリファレンス | pasta-lua-coding への参照なし | **Missing**: `@pasta_persistence` 詳細への誘導 |

**権威ソース（pasta-lua-coding 側で既に文書化済み）:**
- `pasta-lua-coding/SKILL.md` §3 — SAVE キー命名規約
- `pasta-lua-coding/references/internal-modules.md` — SAVE モジュール詳細
- `pasta-lua-coding/references/runtime-api.md` — `@pasta_persistence` API

**戦略**: 重複記述を避け、pasta-ghost-authoring 側には「辞書制作者視点の要約」+「pasta-lua-coding への参照」を追加。

### Req 2: SAVE テーブルのエンジン予約キー記載

| 項目 | 現状 | ギャップ |
|------|------|---------|
| `pasta_` プレフィックス規約 | pasta-ghost-authoring に記載なし | **Missing**: 命名規約と衝突警告 |
| `pasta_talk_interval_min/max` | pasta-ghost-authoring に記載なし | **Missing**: 用途・既定値・3段フォールバック |
| 権威ソース | `virtual_dispatcher.lua` で実装済み | 実装と整合する記述が必要 |

**3段フォールバック（実装の地上真実）:**
1. SAVE テーブル (`pasta_talk_interval_min`) — 最優先
2. pasta.toml `[ghost].talk_interval_min` — 次優先
3. ハードコード既定値 (180秒) — 最終フォールバック

**確定事実（C-1 調査で判明）**: `＄＊XXX` DSL グローバル変数は Lua トランスパイル時に `save.XXX` に変換される。つまり SAVE テーブルキーと DSL グローバル変数は**同一のもの**。辞書制作者は `.pasta` ファイル内で `＄＊pasta_talk_interval_min = 60` と書くことで直接エンジン予約キーを変更できる。これは辞書制作者向けドキュメント（variables.md）に記載すべき内容であることが確定。

### Req 3: pasta.toml リファレンス新設

| 項目 | 現状 | ギャップ |
|------|------|---------|
| references/pasta-toml.md | **存在しない** | **新設が必要** |
| 全セクション網羅 | §4 にミニマルな例のみ (3/9セクション) | 6セクション追加 |
| BudouX 設定 | どこにも記載なし | **Missing**: `[actor."名前"].budoux = [幅1, 幅2]` |

**地上真実（hello-pasta/ghost/master/pasta.toml）:**

```toml
[package]       # 省略可
[loader]        # 必須
[logging]       # 省略可
[ghost]         # 省略可
[talk]          # 省略可
[actor."名前"]  # 省略可（複数定義可）
[persistence]   # 省略可
```

**BudouX の設定経路:**
```
pasta.toml [actor."名前"] → @pasta_config.actor["名前"] → STORE.actors["名前"] → actor テーブル
→ actor.budoux フィールドを SakuraScript ランタイムが読み取り → apply_budoux_if_configured()
```

- 型: 配列 `[usize]`（1要素: 全行同一幅, 2要素: 1行目幅 + 2行目以降幅）
- サンプルゴーストには未設定（利用可能だが未使用）
- テストコードでは `{ budoux = {6} }` や `{ budoux = {10, 12} }` が使用されている

### Req 4: SKILL.md のセクション構造最適化（大規模整理）

| 項目 | 現状 | ギャップ |
|------|------|------|
| §4 pasta.toml | インラインで詳細記述 | references/pasta-toml.md への委譲が必要 |
| §6 Authoring Patterns | 204行のパターン集 | **分離対象**: references/authoring-patterns.md へ移動 |
| セクション番号体系 | §1-§6 存在 | 維持必須（外部参照の破壊防止） |
| SKILL.md 行数 | ~620行 | **目標: 350行以内**（-43%） |

**C-2 クローズ**: Option B（大規模リファクタ）を選択。§6 パターン集を `references/authoring-patterns.md` へ分離する。

### Req 5: metadata.version バンプ

| 項目 | 現状 | ギャップ |
|------|------|---------|
| 現行バージョン | `1.3.0` | `1.4.0` へバンプが必要（新規リファレンス追加 = マイナー） |

---

## 3. 実装アプローチ評価

### Option A: 最小変更（Extend Existing）

**変更対象:**
1. `SKILL.md` §3.4 — 「永続的に有効」の表現修正 + クロスリファレンス追加
2. `SKILL.md` §4 — pasta.toml 記述を圧縮して references/ 参照に
3. `references/variables.md` — SAVE 永続化メカニズム + エンジン予約キーの追記
4. `references/pasta-toml.md` — **新設**（全セクション・全キーの体系的リファレンス）
5. `SKILL.md` YAML ヘッダー — version バンプ

**トレードオフ:**
- ✅ 変更ファイル数最小（3ファイル編集 + 1ファイル新設）
- ✅ 既存構造を尊重、リンク破壊リスクなし
- ✅ 既に §3 で確立された「要約 + 📖 リンク」パターンに倣う
- ❌ SKILL.md 全体の行数は大きく減少しない

### Option B: 大規模リファクタ（Restructure SKILL.md）— **✅ 選択済み（C-2 クローズ）**

**変更内容（Option A + 追加）:**
1. `SKILL.md` §3.4 — 「永続的に有効」の表現修正 + クロスリファレンス追加
2. `SKILL.md` §4 — pasta.toml 記述を圧縮して references/ 参照に
3. `SKILL.md` §6 — **パターン集（204行）を `references/authoring-patterns.md` へ分離**
4. `references/variables.md` — SAVE 永続化メカニズム + エンジン予約キーの追記
5. `references/pasta-toml.md` — **新設**（全セクション・全キーの体系的リファレンス）
6. `references/authoring-patterns.md` — **新設**（§6 分離）
7. `SKILL.md` YAML ヘッダー — version バンプ

**トレードオフ:**
- ✅ SKILL.md を大幅圧縮（620行 → 350行以内， -43%）
- ✅ 要件定義の肨化目標を完全に満たす
- ⚠️ references/ ファイル数が増加（7→8→新設 2ファイル）
- ⚠️ §6 内の外部参照リンクは分離後に更新必要

### Option C: ハイブリッド（推奨）

**Option A をベースに、以下を追加:**
- SAVE 予約キー情報を `references/variables.md` に記載するか、新設の `references/pasta-toml.md` 内 `[persistence]` セクションに記載するかは**設計判断**

**トレードオフ:**
- ✅ 要件を過不足なく満たす
- ✅ 構造的一貫性を維持
- ✅ 将来のリファクタへの基盤（pasta-toml.md が参照の起点）

---

## 4. 設計判断が必要な項目

### DJ-1: SAVE エンジン予約キーの記載場所 — **✅ 確定: (a)**

**確定根拠**: `＄＊グローバル変数` = `save.キー名`（同一）であることが確認された（C-1 クローズ）。DSL 辞書制作者が `＄＊pasta_talk_interval_min = 60` と書けばエンジン動作を変更できるため、variables.md（変数スコープの文書）に記載するのが自然かつ唯一の適切な場所。

**確定**: `references/variables.md` に「永続化と SAVE テーブル」セクションを追加。`pasta-toml.md` への記載は不要（重複を避ける）。

### DJ-2: BudouX の記載カテゴリ

**事実**: BudouX は pasta.toml の `[actor."名前"]` セクション内のキーとして設定可能。しかしサンプルゴーストには未設定。

**選択肢:**
- **(a)** `references/pasta-toml.md` の `[actor."名前"]` セクションに記載（Req 3-AC4 の要件通り）
- **(b)** 追加で `references/sakura-script.md` にも言及を追加

**推奨**: **(a)** のみ — BudouX はさくらスクリプト処理の一部だが、設定は actor テーブル経由であり pasta-toml.md が適切。

### DJ-3: `[lua]` と `[package]` の記載深度

**事実**: `[lua].libs` と `[package]` は辞書制作者には直接関係しない（エンジン開発者向け）。

**選択肢:**
- **(a)** 全キーを網羅的に記載（完全リファレンスとして）
- **(b)** 「上級者向け」として簡潔に記載し、詳細は pasta-lua-coding に委譲
- **(c)** 辞書制作者向けスキルとして省略

**推奨**: **(b)** — 網羅性と対象読者のバランス。

### DJ-4: pasta-toml.md の2階層構造の表現 (新規)

**事実**: pasta.toml セクションは2階層に分かれる。
- **エンジン正式解析** (5種): `[loader]`, `[logging]`, `[persistence]`, `[lua]`, `[talk]` — Rust 構造体にデシリアライズ、型検証あり
- **カスタムフィールド** (3種): `[package]`, `[ghost]`, `[actor."名前"]` — `custom_fields` として Lua に透過的に渡される

**選択肢:**
- **(a)** 2階層を明示的に分離して記載（「エンジン設定」と「ゴースト設定」等）
- **(b)** フラットに全セクションを列挙し、備考欄で解析方式の違いを注記
- **(c)** 辞書制作者には区別不要。全セクションをフラットに記載し、内部構造には触れない

**推奨**: **(b)** — 辞書制作者にとって実用上の差異は小さいが、「カスタムフィールドは自由にキーを追加できる」という情報は有用。

---

## 5. 実装複雑度とリスク

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **M（1週間程度）** | §6分離作業が追加（Option B 選択） |
| **リスク** | **Low-Medium** | §6 内の外部参照確認と新ファイル設計が必要 |

---

## 6. 設計フェーズに持ち越す項目

1. **DJ-2〜DJ-4** の設計判断の確定
2. `references/pasta-toml.md` の具体的なセクション構成と記述粒度
3. `references/variables.md` への追記内容の具体的なテキスト
4. `references/authoring-patterns.md` の構成と内容
5. SKILL.md §4・§6 圧縮後の最終構成

---

## 7. 推奨

**Option B（大規模リファクタ）を採用**（C-2 クローズ済み）。

変更対象ファイル:
1. `SKILL.md` — §3.4 + §4 + §6圧縮 + YAML ヘッダー
2. `references/variables.md` — SAVE 永続化セクション追記
3. `references/pasta-toml.md` — **新設**（全セクション・全キーリファレンス）
4. `references/authoring-patterns.md` — **新設**（§6 分離）
