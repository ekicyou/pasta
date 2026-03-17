# Research & Design Decisions

## Summary
- **Feature**: `pasta-skill-grammar-authority`
- **Discovery Scope**: Extension（既存スキルの再構成、コード変更なし）
- **Key Findings**:
  - 姉妹スキル `pasta-lua-coding` のSKILL.md＋references/パターンがそのまま適用可能
  - doc/spec/ 11章（1,026行、12-future.md除く）が転記元として完備
  - references/ は7ファイル構成が最適（SKILL.md §3サブセクションとの対応関係が明確）

## Research Log

### 姉妹スキルのアーキテクチャ分析
- **Context**: pasta-lua-coding が模範パターン。同じ2層構造を適用可能か検証
- **Sources Consulted**: `.agents/skills/pasta-lua-coding/SKILL.md`（160行）、`references/`（5ファイル/1,485行）
- **Findings**:
  - SKILL.md: 各§で1段落の要約 → `> 📖 詳細: [references/xxx.md]` 導線
  - references/: 冒頭に転記元を明記、AI向けに再構成
  - §2 Quick Reference にテーブル形式で概要を集約
  - §1 に役割分離と自己完結性を明記
- **Implications**: pasta-ghost-authoring にも同一パターンを適用。ただし §3（130行）は現行要約レベルを維持し、姉妹スキルほど圧縮しない（§6 パターン集がスキル固有知識であり、SKILL.md本体に残す必要があるため）

### doc/spec/ 章別サイズと references/ ファイルマッピング
- **Context**: references/ のファイル粒度を決定するため、doc/spec/ 各章のサイズと内容の関連性を調査
- **Findings**:

| doc/spec/章 | 行数 | 主要内容 | references/対応ファイル |
|---|---|---|---|
| 01-grammar-model.md | 54行 | 行指向文法、式サポート | grammar-model.md（統合） |
| 02-markers.md | 300行 | マーカー定義、識別子、演算子、リテラル | grammar-model.md（基礎部分） + action-line.md（識別子定義） |
| 03-block-structure.md | 149行 | ブロック構造、行種別表 | grammar-model.md（統合） |
| 04-call-spec.md | 92行 | Call仕様、スコープ解決 | call-spec.md（単独） |
| 05-literals.md | 29行 | リテラル型、変換ルール | grammar-model.md（統合） |
| 06-action-line.md | 111行 | アクション行、インライン要素、区切りルール | action-line.md（単独＋02識別子定義） |
| 07-sakura-script.md | 47行 | さくらスクリプトタグ、透過ルール | sakura-script.md（単独） |
| 08-attributes.md | 46行 | 属性配置ルール | grammar-model.md（統合） |
| 09-variables.md | 62行 | 変数スコープ、代入構文 | variables.md（単独） |
| 10-words.md | 50行 | 単語定義、スコープ解決 | words.md（単独） |
| 11-actor-dictionary.md | 86行 | アクター辞書、フォールバック | actor-dictionary.md（単独） |
| **合計** | **1,026行** | | **7ファイル** |

- **Implications**: 02-markers.md（300行）は内容が多岐にわたるため分割が必要。識別子定義（最長一致ルール）は action-line.md に統合（インライン区切りと密接に関連）。残りの基礎要素（改行、空白、コロン、インデント、演算子、リテラル、ブロック構造）は grammar-model.md に統合

### §3.2 へのインライン区切りルール追加量の見積もり
- **Context**: root cause fix として §3.2 に直接追加するルールの量を見積もり
- **Findings**:
  - 追加内容: 「インライン要素の区切り文字」サブセクション（空白区切り、最長一致、＠＠エスケープ）
  - 「⚠️ よくある間違い」セクション（4パターンの❌/✅対比）
  - 推定追加量: 40〜50行
  - SKILL.md 全体: 353行 → 約400行
- **Implications**: 姉妹スキルの160行と比較すると多いが、§6パターン集（110行）がスキル固有のため妥当

## Design Decisions

### Decision: references/ ファイル構成（7ファイル）

- **Context**: doc/spec/ 11章 → references/ へのマッピング粒度を決定する必要がある
- **Alternatives Considered**:
  1. 11ファイル（1:1 マッピング）— 粒度が細かすぎ、小ファイル（29行等）が非効率
  2. 3〜4ファイル（大分類統合）— 1ファイルが大きすぎ、LLMのコンテキスト圧迫
  3. 7ファイル（機能単位グループ化）— §3サブセクションとの対応が明確
- **Selected Approach**: 7ファイル構成
- **Rationale**:
  - SKILL.md §3 の9サブセクションと自然に対応（§3.8 Lua/§3.9 Comments は grammar-model.md に統合）
  - 各ファイルが80〜200行の適切なサイズに収まる
  - LLMが必要なファイルだけ `read_file` でロード可能
- **Trade-offs**: 02-markers.md の分割により情報源の追跡がやや複雑になるが、各ファイル冒頭に転記元を明記することで軽減
- **Follow-up**: 実装時に各ファイルの冒頭コメントを統一フォーマットで記載

### Decision: grammar-model.md への統合対象

- **Context**: 01/02/03/05/08 の5章は単独ファイルにするには小さく、内容が基礎構造に関連
- **Alternatives Considered**:
  1. 5ファイル個別 — 29行ファイル等が非効率
  2. 1ファイル統合 — 内容が雑多になる懸念
- **Selected Approach**: grammar-model.md として統合。ただし02-markers.mdの識別子定義（最長一致ルール）はaction-line.mdに配置
- **Rationale**:
  - 01（行指向モデル）+ 03（ブロック構造）+ 05（リテラル）+ 08（属性） は「文法の基盤」として一貫性がある
  - 02のマーカー定義・演算子・空白定義も基盤に含めることで、grammar-model.md が「文法の基礎知識」として機能
  - 識別子定義は action-line.md に移動（インライン区切りルールと不可分なため）
- **Trade-offs**: grammar-model.md が最大（推定150〜200行）になるが、AI向け再構成で圧縮可能

### Decision: SKILL.md §3.2 へのピットフォール配置

- **Context**: Req5 AC1 は §3.2 に「⚠️ よくある間違い」を新設と指定
- **Alternatives Considered**:
  1. §7 として独立セクション新設
  2. references/ に専用ファイル（pitfalls.md）
  3. §3.2 内にインライン配置
- **Selected Approach**: §3.2 内にインライン配置（Req5 AC1 に準拠）
- **Rationale**: root cause がインライン区切りルールの欠落であり、§3.2 Action Lines に直接配置することでLLMが必ず参照する位置に置ける
- **Trade-offs**: §3.2 の行数が増えるが、4パターン程度なら20行以内に収まる

### Decision: SKILL.md §3 の圧縮レベル維持

- **Context**: 姉妹スキルでは各§を1段落に圧縮しているが、ghost-authoring のコンテキストが異なる
- **Selected Approach**: 現行のサブセクション要約レベルを維持（圧縮しない）
- **Rationale**:
  - §3 の各サブセクションは5〜12行で既にコンパクト
  - §6 パターン集（110行）が SKILL.md 固有で移動不可のため、§3 を圧縮しても全体行数の削減効果が小さい
  - Lua スキルの §3 相当は Quick Reference テーブルに集約できたが、DSL文法は構文ルールが多く同じ手法が適用困難
- **Trade-offs**: SKILL.md が約400行になるが、リファレンスなし353行からの増分は最小限（+50行程度）
