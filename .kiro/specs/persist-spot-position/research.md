# Research & Design Decisions: persist-spot-position

## Summary
- **Feature**: persist-spot-position
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. 要件1（CONFIG→STORE.actors パイプライン）と要件2.5（clear_spot/set_spot出力制御）は100%実装済み
  2. `sakura_builder.build()` は `actor_spots` をローカル変数として毎回空で初期化しており、セッション間のスポット状態保持ができない
  3. STOREモジュールは循環参照回避の設計原則に従い「他モジュールをrequireしない」ため、状態フィールド追加は安全

## Research Log

### STORE.actor_spots の配置適性
- **Context**: STORE に新しいフィールド `actor_spots` を追加することの適切性
- **Sources Consulted**: `store.lua` ソースコード、既存フィールド構成
- **Findings**:
  - `STORE` は `actors`, `scenes`, `counters`, `global_words`, `local_words`, `actor_words`, `app_ctx`, `co_scene` を管理
  - `STORE.reset()` で全フィールドを初期化するパターンが確立
  - `STORE.actors` は既に `CONFIG.actor` からの転送パターンを持つ（`store.lua` 末尾の pcall ブロック）
  - `actor_spots` も同じパターンで初期化可能
- **Implications**: STORE にフィールド追加する際は `reset()` への追加も必須

### sakura_builder.build() の純粋関数化パターン
- **Context**: build() のシグネチャ変更方針
- **Sources Consulted**: `sakura_builder.lua` ソースコード、Lua 多値返却パターン
- **Findings**:
  - 現行シグネチャ: `BUILDER.build(grouped_tokens, config) → string`
  - actor_spots の入出力: Lua の多値返却 `return script, updated_actor_spots` で自然に実装可能
  - clear_spot トークンは actor_spots を空テーブルにリセット、spot トークンは actor_spots を更新
  - 入力 actor_spots のコピーを作成して操作することで純粋関数性を保証
- **Implications**: 呼び出し元 (SHIORI_ACT_IMPL.build) が戻り値の第2要素を STORE に書き戻す責務を持つ

### SHIORI_ACT_IMPL.build() の仲介パターン
- **Context**: STORE ↔ sakura_builder 間のデータフロー設計
- **Sources Consulted**: `shiori/act.lua` ソースコード
- **Findings**:
  - 現行: `ACT.IMPL.build(self)` → `BUILDER.build(token, config)` の2段階
  - STORE は `require("pasta.store")` で取得可能（act.lua は既に store を間接参照）
  - `SHIORI_ACT.new()` で STORE.actors を参照している前例あり
- **Implications**: SHIORI_ACT_IMPL.build() 内で STORE を直接 require して actor_spots を読み書きするのが最もシンプル

### サンプルゴーストのアクター名
- **Context**: pasta.toml に追加する [actor] セクションのキー名
- **Sources Consulted**: `actors.pasta`, `talk.pasta`
- **Findings**:
  - メインキャラクター: `女の子`（sakura相当、spot=0）
  - サブキャラクター: `男の子`（kero相当、spot=1）
  - `％女の子、男の子` の形式で使用されている
  - テストフィクスチャでは `さくら` / `うにゅう` を使用（異なるゴースト設定）
- **Implications**: `[actor."女の子"]` と `[actor."男の子"]` を pasta.toml に追加

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| STORE.actor_spots + 純粋build() | STORE に状態保持、build() は入出力のみ | 純粋関数維持、テスト容易、STORE パターン踏襲 | build() シグネチャ変更により既存呼び出しの修正必要 | **採用** |
| build() 内でSTORE直接参照 | build() が STORE を require して状態管理 | シグネチャ変更不要 | 純粋関数性の破壊、テスト困難化 | 不採用 |
| actor オブジェクトに spot 状態保持 | STORE.actors[name].spot を状態として使用 | 既存構造の活用 | CONFIG 初期値と実行時状態の混在、責務不明確 | 不採用 |

## Design Decisions

### Decision: STORE.actor_spots による状態管理
- **Context**: sakura_builder.build() のローカル変数 actor_spots が毎回リセットされるため、セッション間でスポット状態が失われる
- **Alternatives Considered**:
  1. build() 内で STORE を直接参照 — 純粋関数性を破壊
  2. actor オブジェクトの spot フィールドを状態として使用 — CONFIG 値と実行時状態の混在
  3. STORE に actor_spots フィールドを追加し、build() の入出力で受け渡し — **採用**
- **Selected Approach**: STORE.actor_spots を追加、build() は入力として受け取り更新後の値を返す
- **Rationale**: 
  - STORE は全ランタイムデータの一元管理先（設計原則）
  - build() の純粋関数性を維持（テスト容易性）
  - CONFIG → STORE の転送パターンが既に確立（STORE.actors の前例）
- **Trade-offs**: build() のシグネチャ変更が必要だが、呼び出し元は SHIORI_ACT_IMPL.build() のみ
- **Follow-up**: STORE.reset() への actor_spots 初期化追加を忘れないこと

### Decision: actor_spots のコピー戦略
- **Context**: build() に渡された actor_spots を直接変更するかコピーするか
- **Selected Approach**: build() 内でシャローコピーを作成して操作
- **Rationale**: 入力を変更しない純粋関数の原則。clear_spot トークンでテーブルを空にする操作が呼び出し元の参照を破壊しない
- **Trade-offs**: 毎回のコピーコスト（アクター数は通常2-3、無視可能）

## Risks & Mitigations
- **Risk 1**: STORE.reset() に actor_spots 初期化を忘れる → タスクのチェックリストで明示
- **Risk 2**: build() の戻り値変更で既存テストが破綻 → テスト修正をタスクに含める
- **Risk 3**: spot 値の型不整合（number vs string） → spot_to_id() 関数が既に型変換を担当しており、actor_spots には正規化後の値を格納

## References
- [store.lua](../../../crates/pasta_lua/scripts/pasta/store.lua) — STORE モジュール定義
- [sakura_builder.lua](../../../crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua) — ビルダー実装
- [shiori/act.lua](../../../crates/pasta_lua/scripts/pasta/shiori/act.lua) — SHIORI_ACT 実装
- [actor.lua](../../../crates/pasta_lua/scripts/pasta/actor.lua) — アクターモジュール
- [code_generator.rs](../../../crates/pasta_lua/src/code_generator.rs) — Lua コード生成（L292-302）
