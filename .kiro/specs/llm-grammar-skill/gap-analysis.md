# ギャップ分析レポート: llm-grammar-skill

## 分析サマリー

- **スコープ**: LLMがPasta DSLでゴーストを作成するためのVS Code Copilot Skill（SKILL.md形式）の新規作成
- **性質**: 純粋なドキュメント成果物（コード変更なし）。既存の豊富なドキュメント群から情報を集約・再構成する作業
- **既存アセット充実度**: 高い。`steering/grammar.md`、`doc/spec/`（12章）、`GRAMMAR.md`、サンプルゴースト（`pasta_sample_ghost`）がすべて揃っている
- **主要課題**: 情報の「集約と最適化」。LLMコンテキストウィンドウに収まるサイズで、コード生成に十分な情報密度を実現する必要がある
- **推定労力**: S（1〜3日）/ リスク: Low

---

## 1. 現状調査（Current State Investigation）

### 1.1 ドメイン関連アセットの所在

| アセット | パス | 状態 | 要件との関連 |
|----------|------|------|-------------|
| AI向け文法参照 | `steering/grammar.md` | ✅ 存在 | Req 2（文法リファレンス）の主要ソース |
| 権威的仕様書 | `doc/spec/01〜12` | ✅ 存在（12章） | Req 2, 6（整合性の権威的ソース） |
| 人間向け学習資料 | `GRAMMAR.md` | ✅ 存在（1000行超） | Req 2（補足参照）、Req 6（役割分離） |
| サンプルゴースト | `pasta_sample_ghost/dist-src/ghost/master/` | ✅ 存在 | Req 3, 4（テンプレート・パターン集の実例） |
| ゴースト設定 | `pasta_sample_ghost/.../pasta.toml` | ✅ 存在 | Req 3（設定テンプレート） |
| ゴースト記述 | `pasta_sample_ghost/.../descript.txt` | ✅ 存在 | Req 3（descript.txtテンプレート） |
| SHIORIエントリ | `pasta_lua/scripts/pasta/shiori/entry.lua` | ✅ 存在 | Req 5（イベントマッピング） |
| 仮想イベント | `pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua` | ✅ 存在 | Req 5（OnTalk/OnHour機構） |
| シーン検索 | `pasta_lua/scripts/pasta/scene.lua` | ✅ 存在 | Req 5（シーン関数フォールバック） |
| SHIORI README | `pasta_shiori/README.md` | ✅ 存在 | Req 5（SHIORIプロトコル概要） |
| VS Code Copilot Skill | `.agents/skills/` | ❌ 未存在 | Req 1（ディレクトリ新規作成が必要） |

### 1.2 スキルファイル形式の規約

ユーザーのグローバル設定（`~/.agents/skills/`）に多数のスキル実例が存在。標準構造は：

```
.agents/skills/<skill-name>/SKILL.md
```

SKILL.mdの構成要素：
- **YAML Frontmatter**: `name`, `description`（USE FOR / DO NOT USE FOR を含む長文）, `license`, `metadata`
- **本文**: When to Use / Quick Reference / Workflow / Error Handling / Constraints
- **description内のトリガーフレーズ**: Copilotがスキル呼び出しを決定するためのキーワード群

### 1.3 辞書ファイル（サンプルゴースト）の実態

`hello-pasta` ゴーストの `dic/` 配下に4ファイル：

| ファイル | 役割 | 行数 | 使用パターン |
|----------|------|------|-------------|
| `actors.pasta` | アクター辞書定義 | 〜30行 | `％`マーカー + `＠表情：\s[ID]` |
| `boot.pasta` | 起動/終了イベント | 〜30行 | `＊OnBoot`, `＊OnClose`, 重複シーン |
| `click.pasta` | マウス反応 | 〜50行 | `＊OnMouseDoubleClick` × 7パターン |
| `talk.pasta` | ランダムトーク/時報 | 〜50行 | `＊OnTalk`, `＊OnHour`, `＠単語定義` |

---

## 2. 要件フィージビリティ分析

### 要件→アセット対応マップ

| 要件 | 必要な技術要素 | 既存アセット | ギャップ |
|------|---------------|-------------|---------|
| **Req 1**: スキルファイル構造 | YAML frontmatter, SKILL.md形式 | グローバルスキル実例多数 | **Missing**: `.agents/skills/` ディレクトリ自体が未存在。新規作成 |
| **Req 2**: 文法リファレンス | マーカー一覧、構文説明 | `steering/grammar.md`（完全網羅） | ギャップなし。情報の再構成・圧縮のみ |
| **Req 3**: プロジェクト構造テンプレート | ディレクトリ構成、設定ファイル | `pasta_sample_ghost` 実物 | ギャップなし。既存実例からテンプレート化 |
| **Req 4**: パターン集 | イベントハンドラ、ランダムトーク例 | `hello-pasta` の `dic/*.pasta` | ギャップなし。実例をパターンとして抽出 |
| **Req 5**: SHIORIイベントマッピング | イベントディスパッチ機構 | `entry.lua`, `virtual_dispatcher.lua`, `scene.lua` | **Constraint**: Luaランタイム内部の詳細はスキルに含めすぎない。ゴースト作者視点での説明に留める |
| **Req 6**: ドキュメント整合性 | 権威的ソースとの一致 | `doc/spec/` 12章、`steering/grammar.md` | ギャップなし。参照元が明確 |

### ギャップ・制約の詳細

#### Missing: `.agents/skills/` ディレクトリ
- プロジェクトルートに `.agents/` ディレクトリが存在しない
- 新規作成が必要（`.agents/skills/pasta-ghost-authoring/SKILL.md`）
- `.gitignore` への影響確認が必要（通常 `.agents/` は追跡対象）

#### Constraint: スキルファイルサイズの最適化
- LLMのコンテキストウィンドウに収まるサイズである必要がある
- `steering/grammar.md` は約200行、`GRAMMAR.md` は1000行超
- スキルは「コード生成に必要十分な情報」に絞る必要がある
- 冗長な説明より、コードテンプレートとパターン例を優先すべき

#### Constraint: 役割分離の明確化
- `steering/grammar.md`: AI向け完全参照（開発時の実装判断用）
- `GRAMMAR.md`: 人間向け学習資料
- 新スキル: LLMによるゴーストコード生成に特化
- 重複を最小化しつつ、スキル単体で自己完結する必要がある

---

## 3. 実装アプローチの選択肢

### Option A: 単一ファイル（SKILL.md のみ）

**概要**: 文法リファレンス・テンプレート・パターン集をすべて1つのSKILL.mdに統合

**構成**:
```
.agents/skills/pasta-ghost-authoring/
└── SKILL.md    # 全情報を含むスキルファイル
```

**トレードオフ**:
- ✅ 最もシンプルな構成、ファイル管理が容易
- ✅ VS Code Copilot Skillの標準的なパターンに合致
- ✅ LLMへの注入時に単一ファイルで完結
- ❌ ファイルが大きくなる可能性（推定300〜500行）
- ❌ 文法仕様の変更時に1ファイルを更新する必要

### Option B: SKILL.md + 補助ファイル分割

**概要**: SKILL.mdにメタデータ・概要・ワークフローを記載し、詳細な文法リファレンスやテンプレートを別ファイルに分割

**構成**:
```
.agents/skills/pasta-ghost-authoring/
├── SKILL.md              # メタデータ、概要、ワークフロー
├── grammar-reference.md  # 文法リファレンス
├── templates.md          # テンプレート集
└── patterns.md           # パターン集
```

**トレードオフ**:
- ✅ 各ファイルが管理しやすいサイズ
- ✅ 文法変更時の影響範囲が限定的
- ❌ VS Code Copilot Skillの標準パターンから逸脱（SKILLは通常単一ファイル）
- ❌ 補助ファイルがLLMコンテキストに自動注入されるかは不明確
- ❌ 複数ファイルの整合性管理が必要

### Option C: SKILL.md + 既存ドキュメント参照 — ❌ 除外

> **除外理由**: スキルは別リポジトリにコピーして使用するため、pastaリポジトリ内のドキュメントへの参照に依存できない（Req 1 AC 5）。自己完結制約と矛盾するため、本オプションは候補から除外する。

**概要**: SKILL.mdにメタデータと要約を記載し、詳細は既存の `steering/grammar.md` や `doc/spec/` を参照させる

**構成**:
```
.agents/skills/pasta-ghost-authoring/
└── SKILL.md    # メタデータ + 要約 + 既存ドキュメントへの参照指示
```

**トレードオフ**:
- ✅ 情報の重複を最小化
- ✅ 既存ドキュメント更新時に自動的に最新情報を反映
- ❌ LLMがスキル呼び出し後に追加のファイル読み込みを行う必要がある
- ❌ スキル単体で自己完結しない（LLMの追加tool call依存）
- ❌ コンテキストウィンドウ効率が悪い（複数ファイル読み込み）

---

## 4. 推奨事項

### 推奨アプローチ: Option A（単一ファイル）

**理由**:
1. VS Code Copilot Skillの標準パターンに最も合致する
2. LLMのコンテキストに1回の注入で完結し、コード生成効率が最も高い
3. `steering/grammar.md`（約200行）と `hello-pasta` サンプル（約160行合計）の情報量から、300〜500行の単一ファイルに収まると推定
4. 文法変更は低頻度（Phase 0完了済み）のため、1ファイル更新の負荷は低い

### 設計フェーズへの持ち越し事項

| 項目 | 種別 | 詳細 |
|------|------|------|
| SKILL.mdの情報密度設計 | 設計判断 | `steering/grammar.md` のどの情報を含め、何を省略するかの具体的な取捨選択 |
| トリガーフレーズの設計 | 設計判断 | USE FOR / DO NOT USE FOR のキーワード選定（日本語/英語混在） |
| Luaコードブロック説明の深さ | 設計判断 | ゴースト作者が通常使用しない高度機能をどこまで含めるか |
| `.gitignore` への追加要否 | 確認 | `.agents/` ディレクトリがgit追跡対象かどうか |

---

## 5. 実装複雑度・リスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **労力** | **S**（1〜3日） | コード変更なし、既存ドキュメントからの情報集約・再構成のみ |
| **リスク** | **Low** | 確立されたパターン（SKILL.md形式）に従う、既存コードベースへの影響ゼロ、すべてのソース情報が揃っている |
