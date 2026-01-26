# Research & Design Decisions

## Summary
- **Feature**: `store-save-table`
- **Discovery Scope**: Extension（既存システムへの拡張）
- **Key Findings**:
  - `pasta.store`は循環参照回避のため他モジュールをrequireしないパターンを維持している
  - `pasta.ctx`は既に`pasta.act`をrequireしており、依存追加は許容される
  - 現在の`CTX.new`は`save`引数をオプションとして受け取り、nilの場合は空テーブルを生成している

## Research Log

### モジュール依存関係の分析
- **Context**: 循環参照回避パターンの維持可能性を確認
- **Sources Consulted**: `.kiro/steering/lua-coding.md`, `pasta/store.lua`, `pasta/ctx.lua`
- **Findings**:
  - `store.lua`: 他モジュールを一切requireしない（ゼロ依存）
  - `ctx.lua`: `pasta.act`のみrequire（1依存）
  - actorsは`STORE.actors`ではなくローカルで管理されている
- **Implications**: `ctx.lua`への`pasta.store`追加は依存方向を維持し、循環参照を生じない

### 既存パターンの確認
- **Context**: LuaDocアノテーションとモジュール構造のパターン確認
- **Sources Consulted**: `store.lua`, `ctx.lua`, `.kiro/steering/lua-coding.md`
- **Findings**:
  - フィールドは`@field`アノテーションと`@type`アノテーション両方で文書化
  - モジュールテーブルはUPPER_CASE（STORE, CTX）
  - 初期化は空テーブル`{}`で行う
- **Implications**: 既存パターンに完全準拠した実装が可能

## Architecture Pattern Evaluation

| Option             | Description                            | Strengths                          | Risks / Limitations            | Notes                    |
| ------------------ | -------------------------------------- | ---------------------------------- | ------------------------------ | ------------------------ |
| STORE.save直接参照 | CTX.newでSTORE.saveを直接代入          | シンプル、参照共有により一貫性維持 | なし                           | 採用：最小変更で要件充足 |
| コピー方式         | STORE.saveを浅いコピーでctx.saveに渡す | 独立性                             | 変更が反映されない、要件不一致 | 不採用                   |

## Design Decisions

### Decision: STORE.saveへの直接参照
- **Context**: `ctx.save`が`STORE.save`と同一テーブルを参照する必要がある
- **Alternatives Considered**:
  1. 浅いコピー — 独立したテーブルを持つが変更が反映されない
  2. 直接参照 — 同一テーブルを共有し変更を即時反映
- **Selected Approach**: 直接参照（`ctx.save = STORE.save`）
- **Rationale**: 要件が「ctx.save = STORE.save」と明確に指定しており、永続変数の一元管理という目的に合致
- **Trade-offs**: シンプル性を優先、分離性は不要
- **Follow-up**: テストで参照同一性を確認

## Risks & Mitigations
- 循環参照のリスク — store.luaは依存を追加しないため発生しない
- インターフェース変更リスク — 未リリースプロジェクトのため影響なし

## References
- [Luaコーディング規約](../../steering/lua-coding.md) — 命名規約、モジュール構造
