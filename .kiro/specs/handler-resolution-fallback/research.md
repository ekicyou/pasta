# Research & Design Decisions: handler-resolution-fallback

## Summary
- **Feature**: `handler-resolution-fallback`
- **Discovery Scope**: Extension（既存システムの内部リファクタリング + 新規メソッド追加）
- **Key Findings**:
  1. 既存3経路（`find_scene`/`word`/`proxy.word`）のフォールバック順序に不一致がある（GLOBAL位置、SEARCH利用パターン）
  2. `@pasta_search` モジュールのロードパターンが `act.lua`（pcall保護）と `scene.lua`（直接require）で非統一
  3. トランスパイラの `FnScope::Local` → `SCENE.func()` 直接呼び出しは find_handler 経由に変更が必要

---

## Research Log

### 既存フォールバック順序の不一致分析

- **Context**: 3経路（find_scene/word/proxy.word）の検索順序を統一するにあたり、現行差異を精査
- **Sources Consulted**: `act.lua` L237-L350, `actor.lua` L124-L170, `scene.lua` L115-L145
- **Findings**:
  - `find_scene`: scene[key] → SCENE.search(local) → GLOBAL → self[key] → SCENE.search(global) の5段階
  - `word`: scene[name] → GLOBAL → search_word(local) → search_word(global) の4段階
  - `proxy.word`: actor[name] → search_word(actor) → act:word委譲 の3段階+4段階
  - **差異**: `word` は GLOBAL が L2（scene直後）、`find_scene` は L3（ローカル前方一致の後）
- **Implications**: 統一後の順序は「ローカルスコープ完全一致 → ローカル前方一致 → act.XX → グローバル完全一致 → グローバル前方一致」で確定（要件ディスカッションで合意済み）

### SEARCH モジュールのロードパターン

- **Context**: `@pasta_search` の pcall 保護の一貫性を検証
- **Sources Consulted**: `act.lua` L254, `actor.lua` L139, `scene.lua` L125
- **Findings**:
  - `act.lua` / `actor.lua`: `pcall(require, "@pasta_search")` で保護。利用不可時はスキップ
  - `scene.lua`: `require("@pasta_search")` 直接呼び出し。利用不可時は例外
  - `SCENE.search()` は `find_scene` L2/L5 から呼ばれる。SEARCH 未実装時に例外が発生するリスク
- **Implications**: 新設計では `find_act_handler` 内で統一的に pcall パターンを使用。SEARCH の可用性チェックは関数レベルで1回のみ実行

### トランスパイラ FnCall コード生成の変更影響

- **Context**: `FnScope::Local` の出力を `SCENE.func(act, ...)` → `act:expr_fn("func", ...)` / `proxy:expr_fn("func", ...)` に変更
- **Sources Consulted**: `element_gen.rs` L195-L278, L327-L346
- **Findings**:
  - Action::FnCall（アクション行）: 3箇所 — `generate_action()` 内
  - Expr::FnCall（式）: 2箇所 — `generate_expr()` と `generate_expr_to_buffer()`
  - `FnScope::Global` は変更不要（`GLOBAL.func(act, ...)` のまま — フォールバックなしの直接参照）
  - Action::FnCall にはアクター名あり → `proxy:expr_fn()` 形式
  - Expr::FnCall にはアクター名なし → `act:expr_fn()` 形式
- **Implications**: 変更は `FnScope::Local` のみ。3箇所のコード生成パターンすべてを修正。スナップショットテストの更新が必要

### WORD.resolve_value() のポストプロセス互換性

- **Context**: 現行の WORD.resolve_value() と新設計のモード別ポストプロセスの対応関係を確認
- **Sources Consulted**: `word.lua` L110-L121
- **Findings**:
  - 現行: function → 呼び出し / table → 先頭要素 / nil → nil / その他 → tostring
  - word モードポストプロセス: function → `h(act)` / nil → エラーログ+return / その他 → `tostring(h)` — 基本的に互換
  - scene モードポストプロセス: function → コルーチン化 / 非function → エラーログ — 現行 `SCENE.co_exec()` と対応
  - table の先頭要素取得は `SEARCH:search_word` 側で処理（Rust側、ランダム選択付き）されるため `find_handler` レベルでは不要
- **Implications**: word モードのポストプロセスは WORD.resolve_value() とほぼ同等。find_handler が返す値は「テーブルの要素をRust側で選択済みの値」か「function」のいずれか

### SHIORI_ACT_IMPL 継承チェーンと act.XX フォールバック

- **Context**: `self[key]`（actメソッドフォールバック）が SHIORI 継承チェーンでどう動作するか
- **Sources Consulted**: `shiori/act.lua` L101, L161, `act_method_fallback_test.lua`
- **Findings**:
  - SHIORI_ACT_IMPL は ACT_IMPL を `__index` チェーンで継承
  - `self[key]` は SHIORI_ACT_IMPL → ACT_IMPL の順でメソッドを探索
  - `transfer_req_to_var`、`transfer_date_to_var` が唯一の実用ユースケース
  - テスト `act_method_fallback_test.lua` で動作検証済み
- **Implications**: 新設計で act.XX は `type(self[key]) == "function"` チェック付きで維持。メタテーブル経由の継承チェーンは変更不要

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存拡張 | act.lua/actor.lua に find_handler 追加 | ファイル追加なし、既存構造維持 | act.lua 肥大化 | — |
| B: モジュール分離 | handler.lua 新設、コア検索ロジック集約 | 関心分離明確 | 循環参照リスク、ファイル追加 | — |
| **C: ハイブリッド（採用）** | 要件宣言通り act/actor に各メソッド追加 | 要件と直接対応、段階的移行可能 | Phase分割の管理 | gap-analysis Option C |

---

## Design Decisions

### Decision: find_handler のコア関数配置

- **Context**: `find_act_handler` / `find_actor_handler` をどこに配置するか
- **Alternatives Considered**:
  1. 別モジュール `handler.lua` に集約
  2. 各 IMPL テーブルのメソッドとして配置
- **Selected Approach**: 各 IMPL テーブルのメソッドとして配置（Option C）
- **Rationale**: 要件の関数宣言（`ACT_IMPL.find_act_handler`、`PROXY_IMPL.find_actor_handler`）にそのまま対応。メタテーブル継承チェーン（SHIORI_ACT_IMPL → ACT_IMPL）を活用した `self[key]` フォールバックは、メソッドが self のプロトタイプチェーン上にある必要がある
- **Trade-offs**: act.lua の行数増加（~60-80行）。ただしリファクタリングで既存コードが削減されるため純増は小さい
- **Follow-up**: act.lua が 500行超になる場合、将来的な分離を検討

### Decision: FnScope::Global の扱い

- **Context**: `＠＊func()` による明示的グローバル直接参照のコード生成を変更するか
- **Alternatives Considered**:
  1. `GLOBAL.func(act, ...)` のまま（フォールバックなし直接参照を維持）
  2. `act:expr_fn()` に統合し、find_handler のグローバルレベルで解決
- **Selected Approach**: `GLOBAL.func(act, ...)` のまま維持
- **Rationale**: `＠＊func()` はゴースト制作者が「GLOBALテーブルのこの関数を確実に呼ぶ」という明示的意図。フォールバック検索を介在させると「意図した関数ではない関数が返る」リスクが生まれる。直接参照は予測可能性を担保する
- **Trade-offs**: word モードの `GLOBAL.name` 完全一致とは別系統として残る
- **Follow-up**: なし

### Decision: SEARCH モジュールの pcall 統一

- **Context**: `@pasta_search` のロード方式が act.lua（pcall）と scene.lua（直接require）で非統一
- **Alternatives Considered**:
  1. 全箇所を pcall に統一
  2. 全箇所を直接 require に統一（SEARCH を必須依存とする）
  3. 現状維持（混在）
- **Selected Approach**: find_act_handler 内で pcall を1回実行。結果を変数に保持して以降のレベルで再利用
- **Rationale**: SEARCH は Rust 側バインディング。初期化順序の問題で利用不可になるケースがあるため、pcall 保護を維持。ただし関数レベルで1回のみ実行してオーバーヘッドを最小化
- **Trade-offs**: SEARCH 未実装時に前方一致レベルがすべてスキップされる。ただしこれは現行動作と同等
- **Follow-up**: scene.lua の直接 require は本フィーチャーのスコープ外だが、将来的に find_handler 経由に統一される

---

## Risks & Mitigations

- **フォールバック順序変更によるリグレッション** — 既存テスト（950+件）の全パス確認。特に word の GLOBAL 位置変更（L2→L4相当）は既存ゴーストの動作に影響する可能性。段階的移行（find_handler 追加 → 既存関数リファクタリング → トランスパイラ変更）でリスクを分散
- **スナップショットテスト大量更新** — トランスパイラ変更により `SCENE.func(act, ...)` → `act:expr_fn("func", ...)` の出力変更。`insta` のスナップショット一括更新で対応
- **SHIORI_ACT_IMPL 継承チェーンの破壊** — `self[key]` フォールバック位置の変更（L4→ローカル後・グローバル前）。テスト `act_method_fallback_test.lua` で継承動作を明示的に検証

---

## References

- [act.lua](../../crates/pasta_lua/pasta_scripts/pasta/act.lua) — ACT_IMPL.find_scene(), ACT_IMPL.word(), ACT_IMPL.call()
- [actor.lua](../../crates/pasta_lua/pasta_scripts/pasta/actor.lua) — PROXY_IMPL.word()
- [scene.lua](../../crates/pasta_lua/pasta_scripts/pasta/scene.lua) — SCENE.search(), SCENE.co_exec()
- [word.lua](../../crates/pasta_lua/pasta_scripts/pasta/word.lua) — WORD.resolve_value()
- [global.lua](../../crates/pasta_lua/pasta_scripts/pasta/global.lua) — GLOBAL テーブル
- [element_gen.rs](../../crates/pasta_lua/src/code_gen/element_gen.rs) — FnCall コード生成
- [shiori/act.lua](../../crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua) — SHIORI_ACT_IMPL 継承
- [gap-analysis.md](gap-analysis.md) — 要件フィージビリティ分析
