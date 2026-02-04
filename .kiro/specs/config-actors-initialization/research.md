# Research & Design Decisions

## Summary
- **Feature**: `config-actors-initialization`
- **Discovery Scope**: Extension（既存システムへの初期化ロジック追加）
- **Key Findings**:
  - 既存 `STORE.reset()` メソッドが全フィールド初期化ロジックを持っており、再利用可能
  - `@pasta_config` モジュールは Lua VM 起動直後に登録済み（`runtime/mod.rs:538`）
  - `ACTOR_IMPL` メタテーブルパターンは `actor.lua` で確立済み、拡張のみ必要

## Research Log

### pasta.store モジュール初期化パターン
- **Context**: STORE.actors の初期化方式の検討
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/store.lua`
- **Findings**:
  - `STORE.reset()` メソッドが Line 56-72 に存在し、全フィールドをリセット
  - 現状 `STORE.actors = {}` で静的初期化（Line 23）
  - `reset()` 内に CONFIG 連携ロジックを追加することで初期化・リセット両方をカバー
- **Implications**: 初期化コードの重複を避けるため、モジュール末尾で `reset()` を呼び出す設計が最適

### pasta.actor モジュールメタテーブル設定
- **Context**: CONFIG 由来アクターへの ACTOR_IMPL 設定タイミング
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/actor.lua`
- **Findings**:
  - `ACTOR_IMPL` は Line 17-18 で定義済み
  - `ACTOR.get_or_create()` が Line 72-80 で新規アクターにメタテーブル設定
  - モジュール初期化時に `STORE.actors` を走査してメタテーブルを設定するロジック追加が必要
- **Implications**: モジュール末尾（`return ACTOR` の前）に初期化ループを追加

### モジュール依存関係
- **Context**: 循環参照回避の確認
- **Sources Consulted**: `store.lua` コメント（Line 5-6）
- **Findings**:
  - store.lua: 「他モジュールを require しない」制約あり → `@pasta_config` のみ例外追加
  - actor.lua: 既に `pasta.store` を require しているため依存追加なし
- **Implications**: store.lua の依存関係ポリシーを更新（`@pasta_config` は Rust 組み込みモジュールのため例外扱い）

## Architecture Pattern Evaluation

| Option       | Description                   | Strengths        | Risks / Limitations | Notes    |
| ------------ | ----------------------------- | ---------------- | ------------------- | -------- |
| 参照共有     | STORE.actors = CONFIG.actor   | シンプル、効率的 | CONFIG 変更反映     | **採用** |
| ディープコピー | 各オブジェクトをコピー        | 独立性           | コード量増加        | 不採用   |

## Design Decisions

### Decision: 参照共有方式
- **Context**: STORE.actors と CONFIG.actor の関係
- **Alternatives Considered**:
  1. ディープコピー — 各アクターオブジェクトをコピー
  2. 参照共有 — `STORE.actors = CONFIG.actor` で直接代入
- **Selected Approach**: 参照共有
- **Rationale**: 
  - メモリ効率が良い
  - コード量最小化（コピーループ不要）
  - ランタイム変更が CONFIG にも反映される副作用は許容（SHIORI イベントループ内で状態変更は想定内）
- **Trade-offs**: 
  - ✅ シンプル、高効率
  - ⚠️ CONFIG 変更の副作用（意図的に許容）
- **Follow-up**: 副作用を要件 4.1 に明記済み

### Decision: メタテーブル設定責任の分離
- **Context**: STORE.actors のアクターに ACTOR_IMPL を設定する責任者
- **Alternatives Considered**:
  1. store.lua で設定 — store から actor を require
  2. actor.lua で設定 — 既存の依存関係を維持
- **Selected Approach**: actor.lua で設定
- **Rationale**: 
  - store.lua の「他モジュールを require しない」制約を尊重
  - actor.lua は既に store を require しており、自然な拡張
  - ACTOR_IMPL の所有権は actor.lua にある
- **Trade-offs**:
  - ✅ 既存アーキテクチャ遵守
  - ✅ 単一責任原則（ACTOR_IMPL は actor.lua が管理）
- **Follow-up**: なし

## Risks & Mitigations
- **Risk 1**: CONFIG.actor がテーブル以外の型の場合 — 型チェックで空テーブルにフォールバック（要件 2.3）
- **Risk 2**: STORE.actors[name] がテーブル以外の場合 — スキップ処理（要件 2.6）
- **Risk 3**: 循環参照 — store.lua は `@pasta_config`（Rust モジュール）のみ追加、Lua モジュール依存なし

## References
- [store.lua](../../crates/pasta_lua/scripts/pasta/store.lua) — 現行実装
- [actor.lua](../../crates/pasta_lua/scripts/pasta/actor.lua) — ACTOR_IMPL パターン
- [runtime/mod.rs](../../crates/pasta_lua/src/runtime/mod.rs) — モジュール登録順序
