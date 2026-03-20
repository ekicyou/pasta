# ギャップ分析レポート: event-handler-call-equivalence

## 分析サマリー

- **スコープ**: `act:call()` を「名前解決フェーズ」と「実行フェーズ」に分解し、`SCENE.co_exec()` が名前解決フェーズ（`act:find_scene()`）を共有するリファクタリング
- **影響範囲**: `act.lua`（find_scene抽出）+ `scene.lua`（co_execの解決ロジック変更）+ `pasta/shiori/event/` 配下 Lua 3ファイル
- **主要課題**: → **解決済み**: `act:call()` = `act:find_scene()` + 即時実行、`SCENE.co_exec()` = `act:find_scene()` + コルーチン化という2フェーズ分解により、インターフェース差異の問題を解消
- **工数見積り**: **S（1〜3日）** — 変更対象ファイル数が少なく、既存パターンの再構成
- **リスク**: **Medium** — コルーチン管理との統合に注意が必要だが、既存テストカバレッジが厚い（130+テスト）

---

## 1. 現状調査: SCENE.co_exec() 直接呼び出し箇所（全3箇所）

### ① EVENT.no_entry() — [init.lua](../../crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua)
```lua
-- L145: REG未登録イベントのデフォルトハンドラ
function EVENT.no_entry(act)
    return SCENE.co_exec(act.req.id, nil, nil)
end
```
- **問題**: `SCENE.co_exec()` は `SCENE.search()` のみ呼び出し。GLOBAL テーブルを検索しない
- **act:call() との差異**: act:call() の Level 3 (GLOBAL) / Level 4 (act method) / Level 5 (unscoped search) を完全にスキップ

### ② REG.OnBoot デフォルトハンドラ — [boot.lua](../../crates/pasta_lua/pasta_scripts/pasta/shiori/event/boot.lua)
```lua
-- L19: OnBoot のデフォルト実装
REG.OnBoot = function(act)
    return SCENE.co_exec(act.req.id, nil, nil)
end
```
- **問題**: ①と同じ。OnBoot は REG 登録済みハンドラだが、内部で SCENE.co_exec() を直接呼出

### ③ create_scene_thread() — [virtual_dispatcher.lua](../../crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua)
```lua
-- L73-75: 仮想イベント（OnHour, OnTalk）のシーン生成
local function create_scene_thread(event_name, act)
    local SCENE = require("pasta.scene")
    return SCENE.co_exec(event_name, nil, nil)
end
```
- **呼び出し元**: `check_hour()` (L97) および `check_talk()` (L139)
- **問題**: OnHour/OnTalk を `GLOBAL.OnHour` / `GLOBAL.OnTalk` に登録しても検索されない

---

## 2. act:call() の解決パス（リファレンス）

[act.lua](../../crates/pasta_lua/pasta_scripts/pasta/act.lua) `ACT_IMPL.call()` の5段階フォールバック:

| Level | 検索対象 | EVENT dispatch での対応 |
|-------|---------|----------------------|
| L1 | `self.current_scene[key]` (シーンローカル) | ❌ 存在しない |
| L2 | `SCENE.search(key, global_scene_name)` (スコープ付き検索) | ⚠️ SCENE.co_exec() が内部で呼ぶが、コルーチン化される |
| L3 | `GLOBAL[key]` | ❌ **完全に欠落** |
| L4 | `self[key]` (act メソッド) | ❌ 存在しない |
| L5 | `SCENE.search(key, nil)` (スコープなし検索) | ❌ 存在しない |

**注**: EVENT dispatch のコンテキスト（L1 シーンローカルなし、L4 act メソッド意味なし）を考慮すると、実効的に必要なのは L2, L3, L5 のフォールバック。ただし `act:find_scene()` として名前解決ロジックを抽出し共有する設計原則により、Level 選択の判断は不要。

---

## 3. インターフェース差異: act:call() vs EVENT dispatch

### 戻り値の型の不一致

| 項目 | act:call() | EVENT dispatch |
|------|-----------|---------------|
| **戻り値** | 関数の実行結果（即時実行） | `thread` (コルーチン) |
| **act:build()** | 呼び出し元の責任 | `SCENE.co_exec()` 内の wrapped_fn が自動呼び出し |
| **resume_until_valid** | 関係なし | EVENT.fire() が返却コルーチンを resume |
| **STORE.co_scene** | 関係なし | チェイントーク継続管理 |

**設計上の核心課題 → 解決済み**: `act:call()` は `act:find_scene()` + 即時実行、`SCENE.co_exec()` は `act:find_scene()` + コルーチン化とい2フェーズに分解することで、名前解決ロジックを共有しつつ実行方式の差異を吸収する。

### transfer_date_to_var の特殊処理

[virtual_dispatcher.lua L103-106]:
```lua
if act.transfer_date_to_var then
    act:transfer_date_to_var()
end
```
OnHour 発火前に日時情報を act.var に転記する処理。`act:call()` 委譲後もこの前処理は維持が必要。

---

## 4. テストカバレッジ分析

### 既存テストの充実領域
| カテゴリ | テスト数 | カバレッジ |
|---------|---------|-----------|
| act:call() + GLOBAL (L3) | 12 | ✅ 十分 |
| EVENT + GLOBAL 統合（チェイントーク） | 5 | ✅ カバー済み |
| OnHour/OnTalk ディスパッチ | 35+ | ✅ 十分 |
| EVENT.fire() コルーチン管理 | 20+ | ✅ カバー済み |
| EVENT.no_entry フォールバック | 5+ | ⚠️ 基本のみ |

### テストギャップ（要追加）
| ギャップ | 重要度 |
|---------|-------|
| EVENT dispatch → GLOBAL フォールバック（OnHour/OnTalk） | **Critical** — 本仕様の主要目的 |
| `act:call()` が EVENT.no_entry から呼ばれることの検証 | **Critical** — リファクタリング後の正当性 |
| `＊OnHour` DSL ラベル + `GLOBAL.OnHour` 共存時の優先順位 | **High** — Req 2 AC4 |
| `.pasta` フィクスチャによる仮想イベントの E2E テスト | **Medium** — DSL → EVENT 全経路 |

---

## 5. 実装アプローチ選択肢

### Option A: act:call() を2フェーズに分解（find_scene + execute）— 採用決定

**概要**: `act:call()` から名前解決ロジックを `act:find_scene()` として抽出する。`act:call()` は `act:find_scene()` + 即時実行、`SCENE.co_exec()` は `act:find_scene()` + コルーチン化という構成になる。

**変更対象**:
- `act.lua`: `ACT_IMPL.find_scene()` 新規抽出（既存 call() の5段フォールバック部分）、`ACT_IMPL.call()` を find_scene + execute に再構成
- `scene.lua`: `SCENE.co_exec()` が `SCENE.search()` の代わりに `act:find_scene()` を使用
- `event/init.lua`, `event/virtual_dispatcher.lua`, `event/boot.lua`: 変更不要（SCENE.co_exec 経由で自動的に恩恵を受ける）

**トレードオフ**:
- ✅ 名前解決コードパスの完全な1本化
- ✅ `act:call()` の既存インターフェースを変更しない（内部分解のみ）
- ✅ EVENT dispatch 側の3ファイルは変更不要（SCENE.co_exec が内部で find_scene を使うだけ）
- ✅ コルーチン化の橋渡し問題が消滅（find_scene は関数を返すだけで実行しない）

### Option B（不採用）: EVENT.no_entry 内で act:call() を呼び、結果をコルーチン化

**概要**: `EVENT.no_entry(act)` と `create_scene_thread()` で `act:call()` を呼び出し、その結果をコルーチンでラップ。

**トレードオフ**:
- ✅ 変更箇所が少ない
- ❌ `act:call()` の即時実行とコルーチン化の橋渡し問題が残る
- ❌ yield/チェイントークとの統合が複雑

### Option C（不採用）: act:call() にコルーチン対応の新メソッドを追加

**概要**: `act:call_co()` のようなコルーチン返却版を追加。

**トレードオフ**:
- ❌ call と call_co の2パスが生まれる
- ❌ 解決ロジックは共有されるが、実行方法が分岐

### Option D（不採用）: act:call() 自体をコルーチン対応にリファクタリング

**概要**: `act:call()` がオプションでコルーチンを返すモードを持つ。

**トレードオフ**:
- ❌ 既存の act:call() 呼び出し元すべてに影響
- ❌ 影響範囲が大きすぎる

---

## 6. 推奨事項

### 推奨アプローチ: **Option A**（act:call() を2フェーズに分解）

**理由**:
1. 「名前解決コードパスは1つだけ」の原則に完全適合
2. コルーチン化の橋渡し問題が消滅（find_scene は関数を返すだけで実行しない）
3. 既存の act:call() の外部インターフェースを変更しない（内部分解のみ）
4. EVENT dispatch 側の3ファイルは変更不要（SCENE.co_exec が内部で find_scene を使う）
5. 既存テスト（130+）への影響が最小

### 設計フェーズへの持ち越し事項

1. **~~act:call() 即時実行 → コルーチン化の橋渡し方法~~** — ✅ 2フェーズ分解により解決済み。find_scene は関数を返すだけなのでコルーチン化の問題は発生しない
2. **`act:find_scene()` のコンテキスト設定** — EVENT dispatch では `global_scene_name` / `key` の振り分けをどうするか
3. **transfer_date_to_var の呼び出しタイミング** — find_scene 前に実行する現行の前処理フローの維持
4. **チェイントーク (yield) との統合** — SCENE.co_exec() が find_scene 結果をコルーチン化するので、既存 yield 動作は維持されるはず（要検証）
5. **~~REG.OnBoot のデフォルト実装~~** — ✅ 2フェーズ分解により解決済み。REG.OnBoot は SCENE.co_exec() を使うが、その co_exec が `act:find_scene()` を使うためGLOBAL含む全解決空間が自動的にカバーされる

---

## 7. リスク評価

| リスク項目 | レベル | 緩和策 |
|-----------|-------|-------|
| コルーチン管理の破壊 | Medium | 既存の chaintalk テスト (5件) で検証可能 |
| 後方互換性の喪失 | Low | 既存テスト 130+ がリグレッション検出 |
| パフォーマンス影響 | Low | act:call() の追加オーバーヘッドは無視可能 |
| transfer_date_to_var の喪失 | Low | 前処理として維持すればよい |
| REG ハンドラ内部の動作変更 | Low | REG 登録済みハンドラは最優先で変更なし |

**総合リスク: Medium** — コルーチン統合の設計に注意が必要だが、テストカバレッジが厚く安全ネットは十分。
