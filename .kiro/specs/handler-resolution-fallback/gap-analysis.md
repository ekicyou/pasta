# Gap Analysis: handler-resolution-fallback

## 1. 現状調査

### 1.1 対象アセット一覧

| ファイル | レイヤー | 責務 |
|----------|----------|------|
| `pasta_scripts/pasta/act.lua` | Lua ランタイム | `ACT_IMPL.word()`（4段階）、`ACT_IMPL.find_scene()`（5段階）、`ACT_IMPL.call()` |
| `pasta_scripts/pasta/actor.lua` | Lua ランタイム | `PROXY_IMPL.word()`（3段階+委譲）、`PROXY_IMPL.talk()`、`PROXY_IMPL.sakura_script()` |
| `pasta_scripts/pasta/scene.lua` | Lua ランタイム | `SCENE.search()`、`SCENE.co_exec()` |
| `pasta_scripts/pasta/word.lua` | Lua ランタイム | `WORD.resolve_value()`、ビルダーパターン |
| `pasta_scripts/pasta/global.lua` | Lua ランタイム | `GLOBAL` テーブル（関数格納） |
| `src/code_gen/element_gen.rs` | Rust トランスパイラ | `Action::FnCall` / `Expr::FnCall` コード生成 |
| `crates/pasta_dsl/src/parser/ast/action.rs` | Rust AST | `FnScope::Local` / `FnScope::Global` 定義 |

### 1.2 既存フォールバック経路の比較

#### `ACT_IMPL.find_scene(key, global_scene_name, attrs)` — 5段階

| Level | 検索対象 | 完全一致/前方一致 |
|-------|----------|-------------------|
| L1 | `current_scene[key]` | 完全一致 |
| L2 | `SCENE.search(key, global_scene_name)` | 前方一致（Rust） |
| L3 | `GLOBAL[key]` | 完全一致 |
| L4 | `self[key]`（actメソッド） | 完全一致 |
| L5 | `SCENE.search(key, nil)` | 前方一致（Rust、スコープなし） |

#### `ACT_IMPL.word(name)` — 4段階

| Level | 検索対象 | 完全一致/前方一致 |
|-------|----------|-------------------|
| L1 | `current_scene[name]` | 完全一致 |
| L2 | `GLOBAL[name]` | 完全一致 |
| L3 | `SEARCH:search_word(name, scene_name)` | 前方一致（Rust） |
| L4 | `SEARCH:search_word(name, nil)` | 前方一致（Rust） |

#### `PROXY_IMPL.word(name)` — 3段階+委譲

| Level | 検索対象 | 完全一致/前方一致 |
|-------|----------|-------------------|
| L1 | `actor[name]` | 完全一致 |
| L2 | `SEARCH:search_word(name, __actor_xxx__)` | 前方一致（Rust） |
| L3 | `act:word(name)`に委譲 | 上記4段階 |

### 1.3 コンベンション・パターン

- **ログ**: `@pasta_log` モジュール（`log.warn()`, `log.error()`）。`act.lua` で既にインポート済み
- **SEARCH API**: `require("@pasta_search")` を `pcall` で遅延ロード（初期化順序問題の回避）
- **WORD.resolve_value(value, act)**: function→呼び出し / table→先頭要素 / その他→tostring
- **テスト**: `tests/lua_specs/act_find_scene_test.lua`（5段階テスト）、`tests/lua_specs/actor_word_test.lua`
- **メタテーブル公開**: `ACT.IMPL = ACT_IMPL` で外部テストからアクセス可能

### 1.4 トランスパイラの現行コード生成

#### `Action::FnCall`（アクション行中、アクター付き）
```
さくら：＠計算（1、2）
```
**現在の出力**:
```lua
act.さくら:talk(tostring(SCENE.計算(act, 1, 2)))
```

#### `Expr::FnCall`（式コンテキスト）
```
＄結果＝＠計算（1、2）
```
**現在の出力**:
```lua
var.結果 = SCENE.計算(act, 1, 2)
```

**要件が求める変更後の出力**:
- アクター付き: `act.さくら:talk(tostring(proxy:expr_fn("計算", 1, 2)))` ← ★要検討
- 式コンテキスト: `var.結果 = act:expr_fn("計算", 1, 2)`

---

## 2. 要件フィージビリティ分析

### 2.1 要件 ↔ アセット マッピング

| 要件 | 既存アセット | ギャップ |
|------|-------------|----------|
| Req 1: `find_handler()` | `ACT_IMPL.find_scene()`, `ACT_IMPL.word()`, `PROXY_IMPL.word()` | **構造変更**: 3関数のロジックを統一関数に集約。シグネチャ変更（mode引数追加） |
| Req 2: フォールバック戦略 | 各関数に分散して存在 | **ギャップ**: 現行 `find_scene` のL4（actメソッドフォールバック）が要件にない。L2/L5の二重SCENE.search が要件の「ローカル→グローバル」に整理される |
| Req 3: ポストプロセス | `WORD.resolve_value()` が部分的に存在 | **ギャップ**: シーンのコルーチン化は `SCENE.co_exec()` に存在するが `find_handler` 外。expr用ポストプロセスは新規 |
| Req 4: `expr_fn` | なし | **Missing**: 完全新規。act.luaとactor.lua両方に追加 |
| Req 5: トランスパイラ変更 | `element_gen.rs` `Action::FnCall` / `Expr::FnCall` | **構造変更**: `SCENE.func(act, ...)` → `act:expr_fn("func", ...)` / `proxy:expr_fn("func", ...)` |
| Req 6: リファクタリング | `ACT_IMPL.word()`, `PROXY_IMPL.word()`, `ACT_IMPL.find_scene()` | **構造変更**: 内部ロジック全面書き換え |
| Req 7: エラーログ | `@pasta_log` 既存 | **Minor**: ログ呼び出し追加のみ |

### 2.2 フォールバック順序の差異分析（重要）

#### `find_scene` 現行 vs 要件

| 段階 | 現行 `find_scene` | 要件 `find_handler("scene")` | 差異 |
|------|-------------------|------|------|
| 1 | `current_scene[key]` | `scene.XX` 完全一致 | **同等** |
| 2 | `SCENE.search(key, scope)` | ローカルシーン辞書（前方一致） | **同等**（`SCENE.search` = Rust前方一致） |
| 3 | `GLOBAL[key]` | `GLOBAL.XX` 完全一致 | **同等** |
| 4 | `self[key]` actメソッド | — | **⚠削除**: actメソッドフォールバックが要件にない |
| 5 | `SCENE.search(key, nil)` | グローバルシーン辞書（前方一致） | **同等** |

**設計判断ポイント**: L4（actメソッドフォールバック）の削除が既存動作に影響する可能性あり。`act:call()` 経由でactメソッドが呼ばれるケースがあるか要調査。

#### `word` 現行 vs 要件

| 段階 | 現行 `ACT_IMPL.word` | 要件 `find_handler("word")` | 差異 |
|------|----------------------|----------|------|
| 1 | `current_scene[name]` | `scene.XX` 完全一致 | **同等** |
| 2 | `GLOBAL[name]` | ローカル単語辞書（前方一致） | **⚠順序変更**: 現行はGLOBALがL2、要件ではローカル辞書がL2 |
| 3 | `SEARCH:search_word(name, scene)` | `GLOBAL.XX` 完全一致 | **⚠順序変更**: 現行はローカル辞書がL3 |
| 4 | `SEARCH:search_word(name, nil)` | グローバル単語辞書（前方一致） | **同等** |

**設計判断ポイント（確定）**: GLOBAL.XX完全一致を辞書より先に解決する順序を維持する。`scene.XX` → `GLOBAL.XX` → ローカル辞書 → グローバル辞書 の順序で統一する（現行 `ACT_IMPL.word` のL2順序と同等を維持しつつ、`find_scene` とも共通した「完全一致優先」ルールとして明文化する）。

### 2.3 トランスパイラ変更の影響範囲

#### actor付き `Action::FnCall`（`FnScope::Local`）

**問題**: 現在のコード生成で `actor` 変数は `generate_action()` のパラメータとして渡される。要件では `proxy:expr_fn()` に変更するが、アクタープロキシは `act.{actor}` で取得できるため、以下のように変更可能:

```lua
-- 現行: act.さくら:talk(tostring(SCENE.計算(act, 1, 2)))
-- 変更後: act.さくら:talk(tostring(act.さくら:expr_fn("計算", 1, 2)))
```

#### 式コンテキスト `Expr::FnCall`（`FnScope::Local`）

**問題**: `Expr::FnCall` にはactor情報がない（式はアクター行とは独立）。要件では `act:expr_fn()` に変更:

```lua
-- 現行: var.結果 = SCENE.計算(act, 1, 2)
-- 変更後: var.結果 = act:expr_fn("計算", 1, 2)
```

#### `FnScope::Global` の扱い

**要確認**: `＠＊func()` は `GLOBAL.func(act, ...)` に変換される。これは`find_handler`のグローバルレベル検索と実質同じだが、フォールバックなしの直接参照。要件では `FnScope::Global` の扱いが明示されていない。

### 2.4 複雑さシグナル

- **ワークフロー型**: 複数の既存関数のリファクタリング + 新規関数追加 + トランスパイラ変更
- **リグレッションリスク中**: フォールバック順序変更が既存動作に影響する可能性
- **テスト影響大**: `act_find_scene_test.lua` の5段階テストは全面書き換え、スナップショットテスト更新

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張（段階的リファクタリング）

**概要**: `act.lua` と `actor.lua` に `find_handler()` / `find_act_handler()` / `find_actor_handler()` を追加し、既存の `word()` / `find_scene()` を段階的に書き換え。

**変更ファイル**:
1. `pasta_scripts/pasta/act.lua` — `find_handler`, `find_act_handler`, `expr_fn` 追加、`word()` / `find_scene()` 書き換え
2. `pasta_scripts/pasta/actor.lua` — `find_handler`, `find_actor_handler`, `expr_fn` 追加、`word()` 書き換え
3. `src/code_gen/element_gen.rs` — `Action::FnCall` / `Expr::FnCall` コード生成変更

**トレードオフ**:
- ✅ ファイル追加なし。既存構造を維持
- ✅ `act.lua` / `actor.lua` の責務に自然に収まる
- ❌ `act.lua` がさらに肥大化（現在約450行）
- ❌ 段階的移行中にテストが壊れるリスク

### Option B: ハンドラー解決モジュール分離

**概要**: `pasta_scripts/pasta/handler.lua` を新設し、`find_handler` / `find_act_handler` / `find_actor_handler` のコアロジックを集約。`act.lua` / `actor.lua` は薄いラッパーとして委譲。

**変更ファイル**:
1. `pasta_scripts/pasta/handler.lua` — **新規**: コア検索ロジック、ポストプロセス
2. `pasta_scripts/pasta/act.lua` — `word()` / `find_scene()` / `expr_fn` を `handler.lua` に委譲
3. `pasta_scripts/pasta/actor.lua` — `word()` / `expr_fn` を `handler.lua` に委譲
4. `src/code_gen/element_gen.rs` — 同上

**トレードオフ**:
- ✅ 関心の分離が明確（検索ロジック vs トークン蓄積）
- ✅ テスト容易性向上（handler.lua単体テスト可能）
- ❌ ファイル追加（handler.lua）
- ❌ `act.lua` / `actor.lua` への循環参照に注意が必要

### Option C: ハイブリッド（推奨）

**概要**: Phase 1 で `act.lua` に `find_act_handler` を追加、`actor.lua` に `find_actor_handler` を追加（要件の宣言通り）。`find_handler` は各IMPLのメソッドとして実装。将来的な分離は不要な限り行わない。

**変更ファイル**:
1. `pasta_scripts/pasta/act.lua` — `find_act_handler`, `find_handler`, `expr_fn` 追加、`word()` / `find_scene()` リファクタリング
2. `pasta_scripts/pasta/actor.lua` — `find_actor_handler`, `find_handler`, `expr_fn` 追加、`word()` リファクタリング
3. `src/code_gen/element_gen.rs` — `Action::FnCall(Local)` / `Expr::FnCall(Local)` コード生成変更

**フェーズ分割**:
- **Phase 1**: `find_act_handler` / `find_actor_handler` 実装、`expr_fn` 新設
- **Phase 2**: 既存 `word()` / `find_scene()` を `find_handler` ベースにリファクタリング
- **Phase 3**: トランスパイラ変更 + スナップショット更新

**トレードオフ**:
- ✅ 要件の関数宣言にそのまま対応
- ✅ 段階的に進められ、各Phase終了時にテスト確認可能
- ✅ 過剰な抽象化を避けられる
- ❌ Phaseの境界管理が必要

---

## 4. 工数・リスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **M（3–7日）** | Lua側リファクタリング + 新規メソッド + Rustトランスパイラ変更 + スナップショット更新。パターンは既存踏襲 |
| **リスク** | **Medium** | フォールバック順序変更によるセマンティクス変更あり。既存テスト全パスが定量的確認基準 |

### リスク詳細

| リスク | 影響度 | 軽減策 |
|--------|--------|--------|
| フォールバック順序変更（word L2/L3入替） | 中 | 既存テスト全パス確認 + サンプルゴースト動作確認 |
| `find_scene` L4（actメソッド）削除 | 中 | actメソッド利用箇所のgrep調査。利用なしなら安全 |
| トランスパイラ変更によるスナップショット大量更新 | 低 | `cargo test -p pasta_lua` で全スナップショット更新 |
| `FnScope::Global` の扱い未定義 | 低 | 設計フェーズで明確化 |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option C（ハイブリッド）** — 要件宣言に忠実かつ段階的実装可能

### 設計フェーズで決定すべき事項

1. **`find_scene` L4（actメソッドフォールバック）の扱い**: 削除してよいか、既存利用箇所の調査結果を踏まえて判断
2. **`word` フォールバック順序変更の影響**: GLOBAL L2→L3 への変更がサンプルゴーストに影響するか確認
3. **`FnScope::Global` の扱い**: `＠＊func()` は `GLOBAL.func(act, ...)` のまま維持するか、`act:expr_fn` 経由にするか
4. **アクター付き `Action::FnCall` の生成形式**: `act.さくら:talk(tostring(act.さくら:expr_fn(...)))` vs `act.さくら:talk(tostring(proxy:expr_fn(...)))` — 変数 `proxy` がスコープ内に存在するか
5. **`SEARCH` API の `pcall` パターン**: 統一関数内で1回だけ初期化に変更するか
