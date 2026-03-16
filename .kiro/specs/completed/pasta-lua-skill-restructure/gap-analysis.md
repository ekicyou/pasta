# Gap Analysis: pasta-lua-skill-restructure

## 1. 現状調査

### 1.1 既存アセット

| アセット | パス | 行数 | 役割 |
|---------|------|------|------|
| SKILL.md（本体） | `.agents/skills/pasta-lua-coding/SKILL.md` | 588 | AIエージェント向けスキル定義（要約版） |
| LUA_API.md | `crates/pasta_lua/LUA_API.md` | 1160 | 権威的APIリファレンス（Runtime API詳細） |
| lua-coding.md | `.kiro/steering/lua-coding.md` | 695 | 権威的コーディング規約（steering） |
| references/ | 未作成 | 0 | 対象ディレクトリ |

### 1.2 SKILL.md セクション別行数

| セクション | 行数 | 情報ソース | 要約率 |
|-----------|------|-----------|-------|
| YAML frontmatter | ~20行 | 独自 | — |
| §1 Purpose & Prerequisites | ~30行 | 独自（新規追加層） | — |
| §2 Quick Reference | ~50行 | LUA_API.md §1 (33行) | 拡張 |
| §3 Coding Conventions | ~95行 | lua-coding.md §1-§5 (374行) | 25% |
| §4 Runtime API | ~120行 | LUA_API.md §2-§6,§8 (590行) | 20% |
| §5 Internal Modules | ~115行 | lua-coding.md §6 (164行) + LUA_API.md §7 (91行) | 45% |
| §6 SHIORI Handlers | ~90行 | LUA_API.md §9 (322行) | 28% |
| §7 Testing & Lint | ~60行 | lua-coding.md §7 (117行) | 51% |

### 1.3 先行パターン

`.agents/skills/` 配下の既存スキル（`karpathy-guidelines`, `pasta-ghost-authoring`, `pasta-lua-coding`）はいずれも `SKILL.md` 単体構成。`references/` サブディレクトリを持つスキルは**存在しない**。本仕様が初の `references/` パターン導入事例となる。

### 1.4 重要な制約

- **自己完結性**: SKILL.mdヘッダーに「本スキルは別リポジトリにコピーして単体で機能する」と明記されている。references/を導入した場合、コピー時にreferences/も含める必要がある
- **情報ソースの二重管理**: LUA_API.md / lua-coding.md がソース・オブ・トゥルース。references/ファイルはそこからの派生物であり、同期コストが発生する
- **VS Code Copilot Skills仕様**: `SKILL.md` のYAML frontmatterのみが自動検出される。references/内のファイルはスキル本体からの明示的read_file指示が必要

---

## 2. 要件別ギャップマップ

### Requirement 1: SKILL.md行数制約（500行未満）

| 項目 | 状態 | ギャップ |
|------|------|---------|
| 現在の行数 | 588行 | **88行の削減が必要** |
| YAML frontmatter | 維持 | ギャップなし |
| §1 Purpose (~30行) | 維持 | ギャップなし |
| §2 Quick Reference (~50行) | 維持 | ギャップなし |
| §3 Conventions (~95行) | 要約化必要 | → 15-20行に圧縮（-75行）|
| §4 Runtime API (~120行) | 要約化必要 | → 15-20行に圧縮（-100行）|
| §5 Internal Modules (~115行) | 要約化必要 | → 15-20行に圧縮（-95行）|
| §6 SHIORI Handlers (~90行) | 要約化必要 | → 15-20行に圧縮（-70行）|
| §7 Testing (~60行) | 要約化必要 | → 10-15行に圧縮（-45行）|
| References索引セクション | 未作成 | → 新規追加（+15-20行）|

**想定結果**: ~20 (front) + ~30 (§1) + ~50 (§2) + ~20×5 (§3-§7要約) + ~20 (References索引) = **~220-280行** → 500行制約を大幅にクリア

### Requirement 2: referencesディレクトリ構成

| 項目 | 状態 | ギャップ |
|------|------|---------|
| ディレクトリ作成 | 未作成 | **新規作成** |
| ドメイン別ファイル | 未作成 | **5ファイル新規作成** |
| 自己完結性 | — | 各ファイルに前提知識セクション追加が必要 |

### Requirement 3: リファレンスドメイン分割

| ファイル | 情報ソース | 想定行数 | ギャップ |
|---------|-----------|---------|---------|
| runtime-api.md | LUA_API.md §2-§6,§8 (590行) | 250-350行 | **新規作成**。ソースから大幅な内容取り込み |
| internal-modules.md | lua-coding.md §6 (164行) + LUA_API.md §7 (91行) | 200-300行 | **新規作成**。ACTメソッド表の拡充が主 |
| shiori-handlers.md | LUA_API.md §9 (322行) | 200-300行 | **新規作成**。イベント一覧・仮想ディスパッチャの拡充 |
| coding-conventions.md | lua-coding.md §1-§5 (374行) | 250-350行 | **新規作成**。パターン例・禁止例の拡充 |
| testing-lint.md | lua-coding.md §7 (117行) | 100-150行 | **新規作成**。テスト構成例の追加 |

### Requirement 4: リファレンス内容のリッチ化

| 項目 | 状態 | ギャップ |
|------|------|---------|
| 完全なAPIシグネチャ | SKILL.mdでは簡易版 | LUA_API.mdから全パラメータ・戻り値を移行 |
| 実用例 | 基本例のみ | 情報ソースの追加例 + エッジケース例 |
| 相互リファレンスリンク | なし | ファイル間リンク設計が必要 |
| エッジケース・注意事項 | 一部記載 | 各APIの制約・プラットフォーム依存性を明記 |
| 情報ソーストレーサビリティ | `（情報ソース: ...）` で記載 | フッターに統一形式で記載 |

### Requirement 5: SKILL.md本体の構成

| 項目 | 状態 | ギャップ |
|------|------|---------|
| §1-§2維持 | 該当内容あり | ギャップなし |
| §3-§7要約化 | 詳細版のみ | 各3-5行の要約 + リファレンスリンクに書き換え |
| References索引 | なし | 新規セクション追加 |
| フォールバック性 | 現在は単体で完結 | 要約版でdegradedでも使用可能な設計が必要 |

---

## 3. 実装アプローチ

### Option A: 純粋分離（Extract & Link）

**概要**: SKILL.mdの§3-§7を要約に置き換え、詳細をそのままreferences/に移動。情報ソース（LUA_API.md, lua-coding.md）からの追加取り込みは最小限。

**変更対象**:
- `SKILL.md`: §3-§7を3-5行要約+リンクに書き換え、References索引追加
- `references/`: 5ファイル新規作成（現SKILL.mdの§3-§7内容をそのまま移動）

**トレードオフ**:
- ✅ 最小工数。既存内容の移動のみ
- ✅ 情報ソースとの乖離リスクなし（現状維持）
- ❌ Requirement 4（リッチ化）を満たさない
- ❌ references/ファイルが薄い（各60-120行程度）

### Option B: リッチ化分離（Extract, Enrich & Link）

**概要**: SKILL.mdの§3-§7を要約に置き換え、references/には情報ソース（LUA_API.md, lua-coding.md）から詳細を取り込んでリッチ化した内容を配置。

**変更対象**:
- `SKILL.md`: §3-§7を3-5行要約+リンクに書き換え、References索引追加
- `references/`: 5ファイル新規作成（SKILL.md現行内容 + 情報ソースからの詳細取り込み）

**トレードオフ**:
- ✅ 全要件を満たす
- ✅ references/が実用的なリファレンスとして機能
- ✅ 情報ソースへの直接参照が不要になる（スキルの自己完結性向上）
- ❌ 情報ソースとの二重管理コスト（同期が必要）
- ❌ 工数がOption Aの2-3倍

### Option C: ハイブリッド（段階的リッチ化）

**概要**: Phase 1でSKILL.mdの分離（Option A）を実施し、Phase 2で段階的にリッチ化。

**変更対象**:
- Phase 1: Option Aと同一
- Phase 2: references/ファイルに情報ソースから詳細を追加

**トレードオフ**:
- ✅ 早期にSKILL.md 500行制約をクリア
- ✅ リッチ化を段階的に検証可能
- ❌ Phase 2まで Requirement 4 未達
- ❌ 2回のレビューが必要

---

## 4. 複雑性・リスク評価

### 工数見積

| アプローチ | 工数 | 根拠 |
|-----------|------|------|
| Option A | **S**（1-2日） | 既存内容の移動・要約化のみ |
| Option B | **M**（3-5日） | 情報ソース精読 + コンテンツ統合 + 相互リンク設計 |
| Option C | **S + S**（各1-2日） | 2フェーズに分割 |

### リスク評価

| リスク | 水準 | 説明 |
|--------|------|------|
| 情報ソースとの同期 | **中** | LUA_API.md / lua-coding.md が更新された場合、references/の更新を忘れるリスク。トレーサビリティフッターで軽減可能 |
| スキル自己完結性の低下 | **低** | references/をSKILL.mdと同梱する運用を明文化すれば問題なし |
| 要約の情報欠落 | **低** | §1-§2を維持し、§3-§7の要約にキーワードを含めれば、AIが適切にリファレンスを選択できる |
| 先例なきパターン | **低** | references/パターンはVS Code Skills仕様に反しない。read_fileで参照可能 |

---

## 5. 推奨事項

### 推奨アプローチ: **Option B（リッチ化分離）**

**根拠**:
- Requirement 4（リッチ化）がプロジェクトの明示的目標。Option Aでは目標未達
- 情報ソース（LUA_API.md 1160行 + lua-coding.md 695行 = 1855行）に十分な素材がある
- 一度のパスで完了する方が、段階的アプローチより整合性が高い

### 設計フェーズへの引き継ぎ事項

1. **references/ファイル間の相互リンク形式**: 相対パスリンク（`[ACTオブジェクト](internal-modules.md#act-オブジェクト)`）を標準化
2. **情報ソーストレーサビリティ形式**: 各ファイル末尾のフッター書式を設計で確定
3. **SKILL.md要約セクションのテンプレート**: §3-§7各セクションの要約テンプレートを設計で確定
4. **自己完結性の明文化**: SKILL.mdヘッダーにreferences/の同梱要件を追記

---

## 情報ソース

- `.agents/skills/pasta-lua-coding/SKILL.md`（588行）
- `crates/pasta_lua/LUA_API.md`（1160行）
- `.kiro/steering/lua-coding.md`（695行）
- `.kiro/specs/pasta-lua-skill-restructure/requirements.md`
