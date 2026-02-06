# Research & Design Decisions

---
**Purpose**: アルファリリース計画の設計判断・調査記録

---

## Summary
- **Feature**: `alpha-release-planning`
- **Discovery Scope**: Simple Addition / Organization Spec
- **Key Findings**:
  - 6子仕様構成（Option C）で適切な粒度と依存関係を実現
  - 既存資産（EVENT/REG/RES）を最大活用し、新規実装を最小化
  - Phase A/B/C の3段階ロードマップで依存関係を明確化

## Research Log

### 子仕様命名規則の調査
- **Context**: alpha シリーズ子仕様の一貫した命名規則が必要
- **Sources Consulted**: 既存 `.kiro/specs/completed/` の命名パターン
- **Findings**:
  - 既存仕様: `shiori-event-module`, `pasta-lua-cache-transpiler` 等（機能ベース命名）
  - 子仕様系列: `alpha<連番2桁>-<機能名>` で識別性・ソート性を確保
- **Implications**: `alpha01-` ～ `alpha06-` の連番で優先度・依存順を暗示

### 仮想イベント発行機構の分離判断
- **Context**: 当初は SHIORI EVENT と同一子仕様だったが、複雑さから分離
- **Sources Consulted**: 開発者フィードバック、`req.date` テーブル仕様
- **Findings**:
  - OnTalk/OnHour は状態管理（前回トーク時刻等）が必要
  - pasta.toml からの設定読み込みが必要
  - OnSecondChange をトリガーとした条件判定ロジック
- **Implications**: alpha02-virtual-event-dispatcher として独立子仕様化

### ビルド・配布戦略の調査
- **Context**: x86/x64 両アーキテクチャ対応、配布形式選定
- **Sources Consulted**: GitHub Actions Windows ランナー仕様、伺かゴースト互換性
- **Findings**:
  - CI: `i686-pc-windows-msvc` + `x86_64-pc-windows-msvc` 両ビルド
  - 配布: x86（32bit）のみ（伺かベースウェア互換性）
  - 形式: ZIP アーカイブ（NARは将来検討）
- **Implications**: alpha05-build-ci と alpha06-release-packaging で責務分離

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks | Notes |
|--------|-------------|-----------|-------|-------|
| Option A | 最小構成（3子仕様） | 最短リリース | 子仕様が肥大化 | 却下 |
| Option B | 機能分割（5-6子仕様） | 責務分離 | 依存管理複雑 | 検討 |
| **Option C** | 推奨構成（6子仕様） | 適切な粒度、明確な依存 | - | **採用** |

## Design Decisions

### Decision: 6子仕様構成（Option C）の採用
- **Context**: アルファリリースに必要な機能を適切な粒度で分割
- **Alternatives Considered**:
  1. Option A（3子仕様）- 各子仕様が肥大化
  2. Option B（5子仕様）- 仮想イベントが SHIORI EVENT に混在
- **Selected Approach**: 6子仕様構成
- **Rationale**:
  - 仮想イベント発行機構の複雑さを分離
  - pasta.shiori.act を独立モジュールとして明確化
  - Phase A/B/C に自然にマッピング
- **Trade-offs**: 子仕様数増加（管理コスト微増）vs 責務明確化（実装効率向上）
- **Follow-up**: 各子仕様の `/kiro-spec-init` 実行で詳細スコープ確定

### Decision: Phase 3段階構成
- **Context**: 子仕様間の依存関係に基づく実行順序
- **Selected Approach**:
  - Phase A: SHIORI基盤（alpha01, alpha02, alpha03）
  - Phase B: サンプルゴースト（alpha04）
  - Phase C: リリース準備（alpha05, alpha06）
- **Rationale**: 依存元を先行、並行可能なCIは独立進行
- **Trade-offs**: Phase間の待機時間 vs 並行作業による効率化

### Decision: シンプルシェル（ピクトグラム風）
- **Context**: サンプルゴーストのシェル（見た目）素材
- **Selected Approach**: 男の子・女の子のピクトグラム風PNG画像を独自作成
- **Rationale**: 
  - 著作権リスク回避
  - 最小限の画像で動作確認可能
  - 姉貴に依頼可能な簡易素材
- **Follow-up**: alpha04-sample-ghost 設計フェーズで素材仕様確定

## Risks & Mitigations
- **Risk**: 子仕様間の依存でブロッキング発生
  - **Mitigation**: alpha05（CI）は独立進行可能、Phase A の3仕様は並行可能
- **Risk**: 仮想イベント発行機構の状態管理複雑化
  - **Mitigation**: 独立子仕様化により集中的に設計・テスト
- **Risk**: シェル素材作成の遅延
  - **Mitigation**: 最低限ピクトグラムで代替、後からアップグレード可能

## References
- [SOUL.md](../../../SOUL.md) - プロジェクトビジョン・設計原則
- [SPECIFICATION.md](../../../SPECIFICATION.md) - Pasta DSL言語仕様
- [Gap Analysis](./gap-analysis.md) - 既存資産棚卸し・実装方針

