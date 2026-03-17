# ギャップ分析: pasta-skill-grammar-authority

## 1. 現状調査

### 1.1 既存資産の全体像

| 資産 | 行数 | 役割 | 状態 |
|------|------|------|------|
| `.agents/skills/pasta-ghost-authoring/SKILL.md` | 353行 | AI向けDSL文法スキル（メイン） | ✅ 存在、`references/`なし |
| `.agents/skills/pasta-lua-coding/SKILL.md` | 160行 | AI向けLuaランタイムスキル（メイン） | ✅ `references/`付き完了 |
| `.agents/skills/pasta-lua-coding/references/` | 5ファイル/1,485行 | Lua詳細リファレンス | ✅ 完了済み模範 |
| `GRAMMAR.md` | 536行 | 人間向け学習マニュアル | ✅ 存在 |
| `doc/spec/` （01〜12章） | 1,197行 | 権威的仕様書（章別分割） | ✅ 存在、スキル未移植 |
| `.kiro/steering/grammar.md` | 100行 | ステアリング（AI文脈注入） | ✅ 存在 |

### 1.2 姉妹スキル（pasta-lua-coding）の確立済みパターン

`pasta-lua-skill`仕様により完成した構造を模範とする：

```
.agents/skills/pasta-lua-coding/
├── SKILL.md                  # 160行（要約＋Quick Reference テーブル）
└── references/
    ├── coding-conventions.md  # 307行
    ├── internal-modules.md    # 368行
    ├── runtime-api.md         # 372行
    ├── shiori-handlers.md     # 258行
    └── testing-lint.md        # 180行
```

**特徴**:
- SKILL.md: 各§で概要を1段落で説明し、`> 📖 詳細: [references/xxx.md](references/xxx.md)` で導線
- `references/`: 権威文書（`LUA_API.md`, `steering/lua-coding.md`）の完全転記
- 冒頭に転記元を明記

### 1.3 既存SKILL.md（pasta-ghost-authoring）の現行構造

```
§1 Purpose & Prerequisites   （8行）
§2 Quick Reference            （18行 マーカー一覧表）
§3 DSL Syntax                （130行 構文ルール全9サブセクション）
  §3.1 Scenes
  §3.2 Action Lines           ← 🔴 区切りルール不足
  §3.3 Words
  §3.4 Variables
  §3.5 Call Statements
  §3.6 Actor Dictionary
  §3.7 Sakura Script
  §3.8 Lua Code Blocks
  §3.9 Comments & Attributes
§4 Project Structure          （45行）
§5 Event Mapping              （16行）
§6 Authoring Patterns        （110行 辞書制作パターン集）
  §6.1〜§6.10
```

**現行の問題**:
- `references/`ディレクトリが**存在しない**
- §3.2のAction Linesにインライン区切りルールの記載が**ない**（root cause of the bug）
- `doc/spec/06-action-line.md`の「インライン要素の区切り文字」セクション（空白区切り、最長一致、意図しない吸収の例）が未移植
- `doc/spec/02-markers.md`の識別子定義（XID_START/XID_CONTINUE、最長一致切り出し規則）が未移植
- 権威文書→スキルの情報フロー方向が未定義

---

## 2. 要件-資産マッピングとギャップ

### Requirement 1: リファレンス分割アーキテクチャ

| 要素 | 既存資産 | ギャップ |
|------|---------|---------|
| SKILL.md＋references/ 2層構造 | 姉妹スキルに模範あり | **Missing**: pasta-ghost-authoringに`references/`なし |
| doc/spec/章対応の分割ファイル | doc/spec/ 12章すべて存在 | **Missing**: references/への転記ファイルが0 |
| 各§からreferences/への参照パス | 姉妹スキルのパターンあり | **Missing**: 現行SKILL.mdに導線なし |
| 自己完結性 | 現行SKILL.mdは自己完結 | 維持する（references/もフォルダ内に含まれる） |

### Requirement 2: インライン要素の区切りルール明文化

| 要素 | 既存資産 | ギャップ |
|------|---------|---------|
| 空白区切りルール | doc/spec/06-action-line.md §6.3 に完全記載 | **Missing**: SKILL.md §3.2に未記載（**root cause**） |
| 最長一致ルール | doc/spec/06-action-line.md + 02-markers.md 識別子定義 | **Missing**: SKILL.mdに未記載 |
| 正例/誤例ペア | doc/spec/に部分的にあり | **Missing**: ⚠️ペア形式の明示的なピットフォールなし |
| 変数参照の同ルール | doc/spec/06-action-line.md §6.3に記載（全インライン要素共通ルール） | **Missing**: SKILL.md §3.4に空白要件の記載なし |

### Requirement 3: 権威文書のAI向け再構成

| doc/spec/章 | 行数 | SKILL.md §3対応 | ギャップ |
|-------------|------|-----------------|--------|
| 06-action-line.md | 111行 | §3.2（4行要約） | **Missing**: インライン判定ルール、行継続構文、改行セマンティクス |
| 10-words.md | 50行 | §3.3（10行要約） | **Missing**: 動的単語参照、スコープ解決アルゴリズム |
| 09-variables.md | 62行 | §3.4（7行要約） | **Missing**: 式サポート、代入制約、許可される値の型表 |
| 04-call-spec.md | 92行 | §3.5（7行要約） | **Missing**: 2段階検索、フィルター、動的ターゲット詳細 |
| 11-actor-dictionary.md | 86行 | §3.6（12行要約） | **Missing**: シーンスコープ内指定、フォールバック解決ルール |
| 07-sakura-script.md | 47行 | §3.7（7行要約） | **Missing**: コマンド字句構造、文字クラス |
| 01-grammar-model.md | 54行 | なし | **Missing**: 行指向文法モデル、式サポート概要 |
| 02-markers.md | 300行 | §2（18行要約） | **Missing**: 識別子定義、演算子、リテラル |
| 03-block-structure.md | 149行 | §3.8の一部 | **Missing**: ブロック構造の完全定義 |
| 05-literals.md | 29行 | 記載なし | **Missing**: 型変換ルール |
| 08-attributes.md | 46行 | §3.9の一部 | **Missing**: 配置ルール詳細 |
| 12-future.md | 104行 | — | **対象外**: 未確定事項。コード生成には不要 |

### Requirement 4: 重複排除と構造強化

| 要素 | 既存資産 | ギャップ |
|------|---------|---------|
| §2 マーカー表→references/リンク | §2にマーカー表あり | **Missing**: references/リンクなし |
| §3→references/導線 | なし | **Missing**: 姉妹スキルのパターンに合わせて追加 |
| GRAMMAR.mdとの重複排除 | GRAMMAR.md 536行 とSKILL.md §3で内容重複 | **Constraint**: GRAMMAR.mdは人間向け/SKILL.mdはAI向け。同一情報の冗長な重複を避け、権威方向を明確に |
| §6 Authoring Patterns維持 | §6.1〜§6.10（110行） | 維持（スキル固有知識） |

### Requirement 5: 危険パターンとピットフォール集

| 要素 | 既存資産 | ギャップ |
|------|---------|---------|
| ❌/✅対比形式パターン集 | なし | **Missing**: 新規作成が必要 |
| インライン区切り忘れパターン | doc/spec/06-action-line.md に「意図しない吸収の例」あり | **Missing**: スキル形式への変換が必要 |
| 行継続でマーカー使用の誤り | doc/spec/06-action-line.md §6.4 | **Missing**: ピットフォールとして未記載 |
| 属性配置の誤り | doc/spec/08-attributes.md | **Missing**: ピットフォールとして未記載 |

### Requirement 6: 役割分離の明確化

| 要素 | 既存資産 | ギャップ |
|------|---------|---------|
| 三者の役割定義 | steering/grammar.md に部分的記載 | **Missing**: SKILL.md §1に未記載 |
| 情報更新フロー | なし | **Missing**: 新規定義が必要 |
| GRAMMAR.md参照の排除 | 現行SKILL.mdにGRAMMARへの参照なし | ✅ 既に準拠 |

---

## 3. 実装アプローチオプション

### Option A: 拡張アプローチ（既存SKILL.mdを拡張＋references/新設）

**概要**: 現行SKILL.md（353行）を維持しつつ、§3各サブセクションにインライン区切りルール等を追加。`references/`を新設して権威文書を転記。

- **SKILL.mdの変更箇所**:
  - §1: 役割分離の明記を追加（5〜10行増）
  - §2: マーカー表にreferences/リンク列を追加
  - §3.2: インライン区切りルール＋ピットフォールを追加（20〜30行増）
  - §3.x各所: "📖 詳細: references/xxx.md" 導線を追加（各2行×8箇所）
- **references/新設**: doc/spec/の主要章を転記した6〜8ファイル
- **推定変更量**: SKILL.md +60〜80行（353→410〜430行）、references/ 新規600〜800行

**トレードオフ**:
- ✅ 既存のSKILL.md構造（§1〜§6）を完全に維持。LLMの既存学習に影響しない
- ✅ 姉妹スキルのパターンと完全一致
- ✅ 段階的に実装可能（SKILL.md修正→references/作成）
- ❌ SKILL.mdがやや肥大化する可能性（430行程度まで）

### Option B: 再構成アプローチ（SKILL.mdを圧縮＋references/に詳細移動）

**概要**: 姉妹スキルのパターンにより忠実に、SKILL.mdを160〜200行に圧縮し、§3の詳細をすべてreferences/に移動。

- **SKILL.md**: §3を段落要約（各サブセクション1〜2行）に圧縮。Quick Referenceとパターン集は維持
- **references/**: doc/spec/章対応＋ピットフォール集で8〜10ファイル

**トレードオフ**:
- ✅ SKILL.md が最もコンパクト（LLMコンテキスト圧迫最小）
- ✅ 姉妹スキルとの構造的一貫性が最高
- ❌ §3の要約が過度に圧縮されると、LLMがreferences/を毎回read_fileする必要がある
- ❌ 現行のSKILL.mdから大幅な書き直しが必要

### Option C: ハイブリッドアプローチ（推奨）

**概要**: SKILL.mdの§3は現行の要約レベルを維持しつつ、**ピットフォール**と**インライン区切りルール**のみSKILL.md本体に追加。残りの詳細をreferences/に転記。

- **SKILL.md**:
  - §1: 役割分離追加
  - §2: references/リンク追加
  - §3.2: ⚠️ピットフォール＋区切りルール追加（**root cause fix**として最重要）
  - §3.x: 各サブセクションに📖導線のみ追加
  - §6: 維持（変更なし）
- **references/**: 6〜7ファイル（doc/spec/の主要章を機能単位でグループ化）

**推奨references/構成**:

| ファイル | 転記元 | 推定行数 |
|---------|--------|---------|
| `action-line.md` | doc/spec/06-action-line.md + 02-markers.md（識別子） | 150〜180行 |
| `words.md` | doc/spec/10-words.md | 60〜80行 |
| `variables.md` | doc/spec/09-variables.md | 70〜90行 |
| `call-spec.md` | doc/spec/04-call-spec.md | 100〜120行 |
| `actor-dictionary.md` | doc/spec/11-actor-dictionary.md | 90〜110行 |
| `sakura-script.md` | doc/spec/07-sakura-script.md | 50〜60行 |
| `grammar-model.md` | doc/spec/01-grammar-model.md + 02-markers.md（基本） + 03-block-structure.md + 05-literals.md + 08-attributes.md | 150〜200行 |

**トレードオフ**:
- ✅ root cause（インライン区切り）がSKILL.md本体で解決される
- ✅ §6パターン集がそのまま維持される
- ✅ 段階的実装に最適（まず§3.2修正→references/作成）
- ✅ SKILL.md の行数増は最小限（+40〜50行）
- ❌ 完全なreferences/は7ファイル/670〜840行の新規作成が必要

---

## 4. 複雑性とリスク

### 工数見積もり

**Effort: M（3〜7日相当）**
- 理由: 主に文書の転記・再構成作業。コード変更なし。権威文書が完備されており、転記元は明確。ただし7ファイル/700行超の新規作成と整合性確認が必要。

### リスク評価

**Risk: Low**
- 理由: 既存コードへの変更なし。姉妹スキルの確立済みパターンを踏襲。権威文書（doc/spec/）が完備しており、情報不足のリスクなし。テストとの整合性確認不要（ドキュメントのみの変更）。

### 懸念事項

1. **references/の行数膨張**: doc/spec/全章を忠実に転記すると合計1,197行。スキルフォルダの自己完結性は保たれるが、ファイルサイズに注意
2. **doc/spec/との同期維持**: doc/spec/が更新された場合、references/への反映が必要。情報更新フローを§1で明文化することで軽減

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: Option C（ハイブリッド）

**理由**:
1. root causeであるインライン区切りルールがSKILL.md本体に直接記載され、LLMが必ず参照する
2. 姉妹スキルのパターンとの一貫性を維持しつつ、§6パターン集（スキル固有知識）を保全
3. 段階的実装が可能（Phase 1: SKILL.md修正、Phase 2: references/作成）

### 設計フェーズで決定すべき事項

1. **references/のファイル粒度**: doc/spec/章と1:1にするか、機能単位でグループ化するか
2. **SKILL.md §3の圧縮度**: 現行のサブセクション要約レベルを維持するか、さらに圧縮するか
3. **ピットフォール集の配置**: SKILL.md §3.2内にインラインで記載するか、専用セクション（§7相当）を新設するか
4. **doc/spec/12-future.md（将来拡張）の扱い**: references/に含めるか除外するか
