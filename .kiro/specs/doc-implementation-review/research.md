# Research & Design Decisions: doc-implementation-review

---
**Purpose**: ドキュメントレビュー仕様における設計判断と調査結果の記録

---

## Summary
- **Feature**: doc-implementation-review
- **Discovery Scope**: Extension（既存ドキュメント体系の改善）
- **Key Findings**:
  1. SPECIFICATION.md分割により、AIコンテキストサイズを~7,000トークンから~500-1,000トークン/章に削減可能
  2. steering/grammar.mdとGRAMMAR.mdの役割分離により、AI向け完全性と人間向け読みやすさを両立
  3. 品質管理ドキュメント（TEST_COVERAGE.md等）はLevel 2に統合することでヒエラルキーが明確化

## Research Log

### SPECIFICATION.md分割のコンテキスト最適化

- **Context**: steering/grammar.mdがSPECIFICATION.md全体を参照する場合、コンテキストサイズが過大になる懸念
- **Sources Consulted**: SPECIFICATION.md（1,390行、~28KB、~7,000トークン）の構造分析
- **Findings**:
  - 全13章構成、章ごとの行数は35-330行と幅あり
  - 章2（マーカー定義）が最大で~330行、最も参照頻度が高い
  - 章12-13（未確定事項・参考資料）はAI参照不要
- **Implications**: doc/spec/配下に章別分割し、AIは必要な章のみ読み込む設計が有効

### ドキュメントヒエラルキーの再定義

- **Context**: 品質管理ドキュメントの位置づけが曖昧
- **Sources Consulted**: SOUL.md、AGENTS.md、gap-analysis.md
- **Findings**:
  - 現状Level 2はクレートREADMEのみ定義
  - TEST_COVERAGE.md等はPhase 0 DoD達成の公式証跡
  - AI開発支援に必要な参照情報として重要
- **Implications**: Level 2を「実装層ドキュメント」として再定義し、品質管理ドキュメントを含める

### steering/grammar.mdの役割

- **Context**: GRAMMAR.mdとの重複が発生
- **Sources Consulted**: steering/grammar.md、GRAMMAR.md
- **Findings**:
  - 人間向けドキュメントには読みやすさが優先される
  - AI向けドキュメントには完全性が必要
  - 両者の役割が混在していた
- **Implications**: 明確な役割分離により、重複を解消しつつ両方の価値を保持

## Architecture Pattern Evaluation

| Option                | Description                    | Strengths                        | Risks / Limitations            | Notes    |
| --------------------- | ------------------------------ | -------------------------------- | ------------------------------ | -------- |
| 章別分割（doc/spec/） | SPECIFICATION.mdを章ごとに分割 | コンテキスト最適化、独立管理可能 | ファイル数増加、更新同期コスト | 採用決定 |
| インデックス化のみ    | 行番号参照マップ               | 分割不要                         | 行番号変更時に壊れる           | 却下     |
| 現状維持              | SPECIFICATION.md一括           | 管理シンプル                     | コンテキストサイズ過大         | 却下     |

## Design Decisions

### Decision: SPECIFICATION.md分割配置

- **Context**: AIコンテキストサイズ最適化の必要性
- **Alternatives Considered**:
  1. ルートに統合版を維持しつつdoc/spec/に分割版を作成
  2. doc/spec/に分割して統合版を廃止
- **Selected Approach**: Option 2（doc/spec/に分割、ルート版廃止）
- **Rationale**: ルートをクリーンに保つ、単一ソースの原則
- **Trade-offs**: 人間が全体を見る場合はファイル横断が必要（doc/spec/README.mdで緩和）
- **Follow-up**: 実装時にdoc/spec/README.mdをナビゲーションハブとして整備

### Decision: ドキュメント役割分離

- **Context**: steering/grammar.mdとGRAMMAR.mdの重複
- **Alternatives Considered**:
  1. steering/grammar.md削除、GRAMMAR.md一本化
  2. 役割分離（AI向け vs 人間向け）
- **Selected Approach**: Option 2（役割分離）
- **Rationale**: AI向けには完全性、人間向けには読みやすさという異なる要件
- **Trade-offs**: 二重管理のリスク（ただしSPECIFICATION.mdを正規ソースとすることで緩和）

## Risks & Mitigations

- **分割後の参照整合性** — doc/spec/README.mdに章間リンクを整備
- **Runeブロック例の残存** — steering/grammar.mdをLuaブロック例に更新
- **更新同期の負荷** — 各章は独立してレビュー可能、章間依存は最小

## References

- [SPECIFICATION.md](../../../SPECIFICATION.md) — 現在の言語仕様書（分割対象）
- [gap-analysis.md](gap-analysis.md) — 詳細なGap分析結果
- [SOUL.md](../../../SOUL.md) — プロジェクト憲法（ヒエラルキー定義元）
