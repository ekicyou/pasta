# Research & Design Decisions

---
**Purpose**: ドキュメント統合仕様のディスカバリー結果と設計決定の記録。
---

## Summary
- **Feature**: `documentation-consolidation`
- **Discovery Scope**: Simple Addition（シンプルなドキュメント作業）
- **Key Findings**:
  1. pasta_lua/README.md が優秀なテンプレートとして活用可能（276行、包括的）
  2. ステアリングファイル（5ファイル）は堅牢で責務分離が明確
  3. gap-analysis.md で全技術的不明点が解消済み、追加調査不要

## Research Log

### ドキュメント現状調査
- **Context**: 散在するドキュメントの統合計画を立てるための現状把握
- **Sources Consulted**: 全 `.md` ファイルの grep 調査、gap-analysis.md
- **Findings**:
  - ルートレベル: 4ファイル（README, AGENTS, GRAMMAR, SPECIFICATION）
  - ステアリング: 5ファイル（product, tech, structure, grammar, workflow）
  - クレート別README: 1/3存在（pasta_lua のみ）
  - 孤立ドキュメント: 内部ドキュメント除き数件
- **Implications**: Phase 1-4 の段階的実装アプローチが妥当

### クロスリファレンス分析
- **Context**: ドキュメント間のリンク状況を把握
- **Sources Consulted**: grep による参照解析
- **Findings**:
  - GRAMMAR.md → SPECIFICATION.md: ✅ トップに権威的参照あり
  - README.md → GRAMMAR.md/SPECIFICATION.md: ❌ 欠落
  - AGENTS.md → steering/*: ❌ 暗黙的参照のみ
  - pasta_lua/README.md → root README: ❌ バックリンク欠落
- **Implications**: Phase 1 で相互参照網を構築することで情報アクセス改善

### テンプレートパターン抽出
- **Context**: 新規クレートREADME作成のためのパターン抽出
- **Sources Consulted**: pasta_lua/README.md（276行）
- **Findings**:
  - セクション構成: 概要 → アーキテクチャ → ディレクトリ構成 → 設定 → 使用方法 → 関連クレート
  - 表形式の情報整理
  - コードブロックによる設定例・使用例
  - 組み込みモジュール一覧表
- **Implications**: pasta_core/README.md, pasta_shiori/README.md はこのパターンを踏襲

## Architecture Pattern Evaluation

| Option       | Description                | Strengths                        | Risks / Limitations            | Notes                    |
| ------------ | -------------------------- | -------------------------------- | ------------------------------ | ------------------------ |
| Option A     | 既存ドキュメント改善中心   | 最小限の変更、既存参照への影響小 | クレートREADME欠落解消不可     | Req 3, 5, 6 の一部に適用 |
| Option B     | 新規ドキュメント作成中心   | 責務分離明確、網羅率100%         | ファイル数増加                 | Req 4, 7 に適用          |
| **Option C** | **ハイブリッドアプローチ** | **バランス、段階的改善**         | **全フェーズ完了まで期間長い** | **推奨: 全要求に適用**   |

## Design Decisions

### Decision: オンボーディングパス配置
- **Context**: 新規開発者向けの学習パス（ユーザー/開発者/AI向け）の配置場所
- **Alternatives Considered**:
  1. (A) README.md 内にセクション追加
  2. (B) 独立ファイル `docs/ONBOARDING.md` 作成
- **Selected Approach**: **(A) README.md 内にセクション追加**
- **Rationale**: ファイル数増加を抑制、エントリーポイント集約、README.md は自然なエントリーポイント
- **Trade-offs**: README.md がやや長くなるが、許容範囲内
- **Follow-up**: README.md のサイズを監視、必要に応じて将来分離

### Decision: 保守ガイドライン配置
- **Context**: ドキュメント更新チェックリストや保守ルールの配置場所
- **Alternatives Considered**:
  1. (A) workflow.md に「ドキュメント保守」セクション追加
  2. (B) 新規 `docs/MAINTENANCE.md` 作成
- **Selected Approach**: **(A) workflow.md 拡充**
- **Rationale**: 既存のワークフロードキュメントに統合、AI参照容易性向上
- **Trade-offs**: workflow.md がやや長くなるが、責務の一貫性維持
- **Follow-up**: なし

### Decision: GRAMMAR.md 対応方針
- **Context**: GRAMMAR.md と SPECIFICATION.md の重複への対応
- **Alternatives Considered**:
  1. (A) 軽微な削減（重複部分のみ削除）
  2. (B) 大幅な削減（クイックリファレンスに縮小）
  3. (C) 現状維持（整理と乖離解消のみ）
- **Selected Approach**: **(C) 現状維持**
- **Rationale**: 仕様変更の可能性が高く、現時点では大幅な構造変更はリスク
- **Trade-offs**: 重複は残るが、仕様安定後に改めて整理可能
- **Follow-up**: 仕様が安定した段階で削減を再検討

### Decision: 実装フェーズ順序
- **Context**: 開発者の優先順位要望に基づくフェーズ順序決定
- **Alternatives Considered**:
  1. 技術的依存関係順（Phase 1: クレートREADME → Phase 4: 相互参照）
  2. 優先順位D→C→B→A順（Phase 1: 相互参照 → Phase 4: クレートREADME）
- **Selected Approach**: **優先順位D→C→B→A順**
- **Rationale**: 開発者が相互参照（D）を最優先と指定、クレートREADME（A）は最後
- **Trade-offs**: 技術的には相互参照が最後の方が自然だが、開発者要望を尊重
- **Follow-up**: なし

## Risks & Mitigations
- **スコープクリープリスク (Medium)** — 各フェーズのOut of Scopeを厳守、フェーズ毎にレビュー
- **技術的リスク (Low)** — マークダウン編集のみ、コード変更なし
- **品質リスク (Low)** — pasta_lua/README.md をテンプレート活用で品質確保

## References
- [pasta_lua/README.md](../../crates/pasta_lua/README.md) — クレートREADMEテンプレート
- [gap-analysis.md](./gap-analysis.md) — 詳細なギャップ分析結果
- [.kiro/steering/workflow.md](../../.kiro/steering/workflow.md) — 開発ワークフロー規約
