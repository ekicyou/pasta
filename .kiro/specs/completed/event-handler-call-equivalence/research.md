# Research & Design Decisions: event-handler-call-equivalence

## Summary
- **Feature**: `event-handler-call-equivalence`
- **Discovery Scope**: Extension（既存システムの統合リファクタリング）
- **Key Findings**:
  1. SCENE.co_exec() の全3呼び出し箇所が `act` オブジェクトを保持しており、`act:find_scene()` への委譲が容易
  2. `act:find_scene()` のパラメータ順序は `(key, global_scene_name?, attrs?)` が最適（ユーザー空間でのフォールバックパターン `act:find_scene("時報1300") or act:find_scene("時報")` を自然に記述可能）
  3. コルーチン管理は完全に無変更で維持可能（find_scene は関数を返すだけで実行しない）

## Research Log

### SCENE.co_exec() 呼び出し箇所の act 可用性

- **Context**: SCENE.co_exec() に `act` パラメータを追加できるか？全呼び出し元で `act` が利用可能かを調査
- **Sources Consulted**: `event/init.lua` L111-120, `event/boot.lua` L14-19, `event/virtual_dispatcher.lua` L51-62
- **Findings**:
  - `EVENT.no_entry(act)` — `act` を引数として受け取り → 利用可能
  - `REG.OnBoot(act)` — `act` を引数として受け取り → 利用可能
  - `create_scene_thread(event_name, act)` — `act` を第2引数として受け取り → 利用可能
  - 全3箇所とも `act` を保持しているが、現在は `SCENE.co_exec()` に渡していない
- **Implications**: SCENE.co_exec() のシグネチャに `act` を追加するのは安全。呼び出し元の変更は全て内部コード。

### act:find_scene() のパラメータ設計

- **Context**: `act:call(global_scene_name, key, attrs, ...)` と `act:find_scene()` のシグネチャ整合性
- **Findings**:
  - `act:call()` のパラメータ順: `(self, global_scene_name, key, attrs, ...)`
  - DSLトランスパイラが生成する call 呼び出しは `act:call("シーン名", "ローカル名", nil)` の形式
  - しかし `find_scene` の主目的は「キーで検索」であり、ユーザー空間での利用パターンは:
    ```lua
    act:find_scene("時報1300") or act:find_scene("時報")
    ```
  - よって `find_scene(self, key, global_scene_name?, attrs?)` が自然
  - `call` 内部では `self:find_scene(key, global_scene_name, attrs)` とパラメータを並べ替えて呼び出す
- **Implications**: `call()` と `find_scene()` でパラメータ順序が異なるが、それぞれの用途に最適化されている。`call` は DSL トランスパイラが生成するため順序変更不可、`find_scene` はユーザー向けAPI。

### EVENT dispatch での5段階フォールバックの実効性

- **Context**: EVENT dispatch コンテキストで5段階すべてが意味を持つか？
- **Findings**:
  - **L1** (current_scene): `create_act()` で生成された直後の `act` は `current_scene = nil` → 常にスキップ（害なし）
  - **L2** (スコープ付き検索): `global_scene_name = nil` → スコープなし検索と同等
  - **L3** (GLOBAL): **本仕様の主要修正点**。現在欠落しており、これにより `GLOBAL.OnHour` が EVENT dispatch で検索されない
  - **L4** (act メソッド): 新規 `act` に意味のあるメソッドなし → 常にスキップ（害なし）
  - **L5** (スコープなし検索): L2 と同等（`global_scene_name` が両方 nil のため）
- **Implications**: L1/L4 は EVENT dispatch では無操作だが、`find_scene` をそのまま共有することでコードパスの1本化を達成。論理的に不要なレベルをスキップする最適化は行わない（正確さ＞性能）。

### コルーチン管理への影響

- **Context**: find_scene 抽出により、コルーチン生成・管理フローに影響はないか？
- **Findings**:
  - `SCENE.co_exec()` のコルーチン生成:
    1. `act:find_scene()` で関数を取得（同期的）
    2. `coroutine.create(wrapped_fn)` でコルーチン化
    3. `wrapped_fn` 内で `fn(act, ...)` + `act:build()`
  - `EVENT.fire()` のコルーチン消費:
    1. `resume_until_valid(co, act)` で初回 resume（`act` を引数渡し）
    2. nil yield → 再 resume ループ
    3. `set_co_scene(co)` で STORE.co_scene 管理
  - チェイントーク（`STORE.co_scene`）:
    - `check_talk()` L135 で `STORE.co_scene` をそのまま返す → 新規検索なし → 影響なし
  - **全コルーチン操作は find_scene の外側** → 影響ゼロ
- **Implications**: コルーチン管理コードは一切変更不要。find_scene は「関数オブジェクトを返すだけ」。

### transfer_date_to_var の前処理タイミング

- **Context**: OnHour 発火前の `act:transfer_date_to_var()` は維持されるか？
- **Sources Consulted**: `virtual_dispatcher.lua` L96-99
- **Findings**:
  - `check_hour()` 内で `create_scene_thread()` の**前**に呼ばれる
  - `create_scene_thread()` が `SCENE.co_exec()` を呼ぶ（find_scene 経由に変更後も同じ位置）
  - 呼び出し順序は不変: `transfer_date_to_var()` → `create_scene_thread()` → `act:find_scene()` → `coroutine.create()`
- **Implications**: 前処理タイミングに影響なし。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: find_scene 抽出（採用） | ACT_IMPL.call() から名前解決を find_scene() として抽出し、SCENE.co_exec() が共有 | コードパス1本化、コルーチン問題消滅、既存IF不変 | SCENE.co_exec のシグネチャ変更（内部のみ） | ギャップ分析 Option A と同一、2フェーズ分解設計 |
| B: co_exec 内で act:call() | EVENT.no_entry で act:call() を呼び結果をコルーチン化 | 変更箇所少 | 即時実行⇔コルーチンの橋渡し問題が残る | 不採用 |
| C: act:call_co() 新設 | コルーチン返却版の新メソッド | 明示的 | 2パス問題（call と call_co で解決ロジック複製の温床） | 不採用 |

## Design Decisions

### Decision: `act:find_scene()` のパラメータ順序

- **Context**: `act:call(global_scene_name, key, attrs, ...)` とのパラメータ順序整合性
- **Alternatives Considered**:
  1. `find_scene(self, global_scene_name, key, attrs)` — call() と同一順序
  2. `find_scene(self, key, global_scene_name?, attrs?)` — キー優先順序
- **Selected Approach**: Option 2 — `find_scene(self, key, global_scene_name?, attrs?)`
- **Rationale**:
  - ユーザー空間での利用パターン `act:find_scene("時報1300")` が自然
  - `global_scene_name` と `attrs` は省略可能なオプショナル引数として後置
  - `call()` のパラメータ順序はDSLトランスパイラが生成するため変更不可
- **Trade-offs**: `call()` と `find_scene()` でパラメータ順序が異なるが、用途が明確に異なるため混乱リスクは低い
- **Follow-up**: スキルファイル（pasta-lua-coding）に find_scene の API ドキュメントを追加

### Decision: SCENE.co_exec() シグネチャ変更

- **Context**: SCENE.co_exec() が act:find_scene() を呼ぶために act パラメータが必要
- **Alternatives Considered**:
  1. SCENE.co_exec に act を追加: `SCENE.co_exec(act, name, global_scene_name, attrs)`
  2. act なしで GLOBAL 検索のみを SCENE.co_exec 内にハードコード
- **Selected Approach**: Option 1 — act を第1引数に追加
- **Rationale**:
  - 全3呼び出し箇所で act が利用可能（調査済み）
  - 名前解決ロジックの1本化原則に完全適合
  - Option 2 は GLOBAL 検索のみの部分的解決であり、5段階フォールバック全体の共有にならない
- **Trade-offs**: シグネチャ変更により全呼び出し元の修正が必要（3箇所のみ、全て内部コード）

## Risks & Mitigations

- **コルーチン管理の破壊** — find_scene は関数を返すだけで実行しないため影響なし。既存 chaintalk テスト (5件) + EVENT テスト (20+件) で検証
- **後方互換性の喪失** — act:call() の外部 IF は不変。既存テスト 130+ がリグレッション検出
- **L1/L4 の不要な検索オーバーヘッド（EVENT dispatch）** — nil チェックのみで実質的なコスト 0。コードパス共有の利益が上回る
- **SCENE.co_exec シグネチャ変更の影響範囲漏れ** — grep 調査で全3箇所を特定済み。テスト用モック `_set_scene_executor` は co_exec を経由しないため影響なし

## References

- [gap-analysis.md](gap-analysis.md) — 実装ギャップ分析（3バイパス箇所、テストギャップ、Option A-D 比較）
- [act.lua](../../crates/pasta_lua/pasta_scripts/pasta/act.lua) — ACT_IMPL.call() 5段階フォールバック (L336+)
- [scene.lua](../../crates/pasta_lua/pasta_scripts/pasta/scene.lua) — SCENE.co_exec() (L192+), SCENE.search() (L150+)
- [event/init.lua](../../crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua) — EVENT.fire() (L152+), EVENT.no_entry() (L142+)
- [event/boot.lua](../../crates/pasta_lua/pasta_scripts/pasta/shiori/event/boot.lua) — REG.OnBoot デフォルトハンドラ
- [event/virtual_dispatcher.lua](../../crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua) — create_scene_thread() (L68+)
- [global.lua](../../crates/pasta_lua/pasta_scripts/pasta/global.lua) — GLOBAL テーブル
