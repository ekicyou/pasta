# Research & Design Decisions

## Summary
- **Feature**: `soul-scene-architecture`
- **Discovery Scope**: Simple Addition（ドキュメント追加のみ、コード変更なし）
- **Key Findings**:
  - Luaバックエンドが確定済み（Runeは廃止）
  - `sample.pasta`と`sample.generated.lua`が最重要参照ファイル
  - 映画撮影メタファー（シーン＝台本、アクター＝役者、アクション＝演技、act＝カメラ）が設計哲学の核心

## Research Log

### Lua実行パターンの調査
- **Context**: シーン関数がLuaコードとしてどのように実行されるかを理解する必要がある
- **Sources Consulted**: 
  - `crates/pasta_lua/tests/fixtures/sample.pasta`
  - `crates/pasta_lua/tests/fixtures/sample.generated.lua`
- **Findings**:
  - 各シーンは`function SCENE.__シーン名__(act, ...)`シグネチャを持つLua関数に変換される
  - `act:init_scene(SCENE)`でシーンコンテキストを初期化し、`save`と`var`を取得
  - `act.アクター:talk()`で発話をactに記録（撮影）
  - `act:call()`でシーン呼び出し、末尾の`return act:call()`で末尾呼び出し最適化
  - `act:word()`でシーン・アクタースコープの単語を展開
- **Implications**: ドキュメントはこの実行パターンを明確に説明する必要がある

### 既存SOUL.mdの構造分析
- **Context**: 新章の挿入位置と既存章との整合性を確認
- **Findings**:
  - 「4. 辞書アーキテクチャ」は静的データ構造を説明
  - 「5. Phase 0」は開発状況を説明
  - 新章は「動的実行モデル」として4と5の間に挿入するのが自然
  - 既存の5～8章は6～9章に繰り下げ
- **Implications**: 章番号の更新が必要だが、内容の変更は不要

## Architecture Pattern Evaluation

| オプション | 説明 | 強み | リスク/制限 | 備考 |
|-----------|------|------|------------|------|
| 独立章として挿入 | 「5. シーン実行アーキテクチャ」として独立章を追加 | 既存章との論理的分離、明確な責務 | 章番号繰り下げが必要 | **採用** |
| サブセクションとして追加 | 「4.5」として辞書章の一部に | 章番号変更不要 | 静的構造と動的実行の混在 | 却下 |

## Design Decisions

### Decision: 実装言語の明示
- **Context**: AIアシスタントへの明確性のため、Luaを明示するか抽象化するか
- **Alternatives Considered**:
  1. 「スクリプトVM」と抽象化
  2. 「Lua」と明示
- **Selected Approach**: Luaと明示する
- **Rationale**: AI clarity、実際の実装との一致、将来の変更時はドキュメント更新で対応
- **Trade-offs**: 特定技術への言及だが、実態に即している

### Decision: 4要素メタファーの採用
- **Context**: 映画撮影メタファーの要素数（3 or 4）
- **Selected Approach**: 4要素（シーン＝台本、アクター＝役者、アクション＝演技、act＝カメラ）
- **Rationale**: `act`オブジェクトが実行時の中心的役割を担うため、明示的に含める
- **Evidence**: `sample.generated.lua`での`act.さくら:talk()`、`act:call()`パターン

## Risks & Mitigations
- **リスク1**: 章番号繰り下げによる外部参照の破壊 → **対策**: 影響範囲の確認（SOUL.mdへの直接章番号参照は少ない）
- **リスク2**: 技術詳細の過剰記載 → **対策**: SOUL.mdは哲学・アーキテクチャレベル、SPECIFICATION.mdへ技術詳細を委譲

## References
- [sample.pasta](../../crates/pasta_lua/tests/fixtures/sample.pasta) — DSL入力サンプル
- [sample.generated.lua](../../crates/pasta_lua/tests/fixtures/sample.generated.lua) — Lua出力サンプル
- [SPECIFICATION.md](../../../SPECIFICATION.md) — 権威的技術仕様
- [SOUL.md](../../../SOUL.md) — 改訂対象ドキュメント
