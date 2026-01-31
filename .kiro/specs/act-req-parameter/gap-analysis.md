# Gap Analysis: act-req-parameter

## Executive Summary

本分析は、`pasta.shiori.act` モジュールに `req` フィールドを追加し、イベントディスパッチで `act` オブジェクトを生成・引き渡す機能の実装ギャップを調査した。

### 主要発見事項
- ✅ **既存基盤は十分**: `pasta.shiori.act` と `pasta.shiori.event` の拡張で実現可能
- ✅ **シンプルな変更**: 2ファイルの軽微な修正で要件を満たせる
- ⚠️ **後方互換性**: ハンドラシグネチャ変更に注意が必要（`function(req)` → `function(req, act)`）
- ⚠️ **シーン関数呼び出し**: 現状 `scene_result()` を `scene_result(act)` に変更する必要あり

### 推奨アプローチ
**Option A（既存コンポーネント拡張）** を推奨。新規ファイル不要、影響範囲が限定的。

---

## 1. Current State Investigation

### 1.1 対象ファイル・モジュール

| ファイル | 責務 | 変更の必要性 |
|----------|------|--------------|
| `crates/pasta_lua/scripts/pasta/shiori/act.lua` | SHIORI専用actオブジェクト | 🔧 `new()` シグネチャ変更、`req` フィールド追加 |
| `crates/pasta_lua/scripts/pasta/shiori/event/init.lua` | イベント振り分け | 🔧 act生成・引き渡しロジック追加 |
| `crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua` | 仮想イベント発行 | 🔧 `execute_scene()` に act 引き渡し |
| `crates/pasta_lua/scripts/pasta/store.lua` | データストア | ✅ 変更不要（actors 取得元として利用） |
| `crates/pasta_lua/scripts/pasta/act.lua` | 親クラス Act | ✅ 変更不要 |

### 1.2 既存のコード構造

#### SHIORI_ACT.new() 現状
```lua
function SHIORI_ACT.new(actors)
    local base = ACT.new(actors)
    base._buffer = {}
    base._current_spot = nil
    base._spot_switch_newlines = CONFIG.get("ghost", "spot_switch_newlines", 1.5)
    return setmetatable(base, SHIORI_ACT_IMPL)
end
```
**Gap**: `req` パラメータなし、`base.req` フィールドなし

#### EVENT.fire() / EVENT.no_entry() 現状
```lua
function EVENT.fire(req)
    local handler = REG[req.id] or EVENT.no_entry
    -- ...
    return handler(req)  -- act を渡していない
end

function EVENT.no_entry(req)
    local scene_result = SCENE.search(req.id, nil, nil)
    if scene_result then
        pcall(function()
            return scene_result()  -- act を渡していない
        end)
    end
    return RES.no_content()
end
```
**Gap**: act オブジェクト未生成、ハンドラ/シーン関数に act 未引き渡し

#### virtual_dispatcher.execute_scene() 現状
```lua
local function execute_scene(event_name)
    local scene_fn = SCENE.search(event_name, nil, nil)
    if not scene_fn then return nil end
    local ok, result = pcall(scene_fn)  -- act を渡していない
    return result
end
```
**Gap**: act オブジェクト未引き渡し

### 1.3 パターン・規約

| 規約 | 現状 | 対応 |
|------|------|------|
| モジュール構造 | `1. require文 → 2. モジュール宣言 → 3. 公開関数 → 4. 返却` | 維持 |
| エラーハンドリング | `pcall`/`xpcall` + `RES.err()` | 維持 |
| テスタビリティ | 依存性注入パターン（actors引数） | 維持（STOREを直接参照しない） |
| 後方互換性 | 既存ハンドラシグネチャ `function(req)` | **要対応** |

---

## 2. Requirements Feasibility Analysis

### 2.1 要件と既存資産のマッピング

| 要件 | 既存資産 | ギャップ |
|------|----------|----------|
| REQ-1: act.req フィールド追加 | `SHIORI_ACT.new(actors)` | Missing: req パラメータ、base.req フィールド |
| REQ-2: イベントディスパッチでact生成 | `EVENT.fire()`, `EVENT.no_entry()` | Missing: act 生成・引き渡しロジック |
| REQ-3: STORE.actors からアクター取得 | `pasta.store.actors` | ✅ 利用可能 |
| REQ-4: 後方互換性維持 | 既存ハンドラ `function(req)` | Constraint: 新旧シグネチャ両対応必要 |

### 2.2 技術的制約

1. **循環参照回避**: `pasta.shiori.act` から `pasta.store` への require は問題なし
2. **遅延ロード**: `pasta.scene` は既に遅延ロードパターン使用済み（継続）
3. **Lua互換性**: Lua 5.4 の可変長引数 `...` で後方互換対応可能

### 2.3 複雑性シグナル

- **Simple**: 既存パターンへの軽微な拡張
- **No new dependencies**: 新規モジュール不要
- **Well-defined interfaces**: act/req のインターフェースは既に確立

---

## 3. Implementation Approach Options

### Option A: 既存コンポーネント拡張 ✅推奨

**適用理由**: 変更が2ファイルに限定、既存パターンに自然適合

#### 変更内容

**1. `pasta/shiori/act.lua`**
```lua
--- 新規ShioriActを作成
--- @param actors table<string, Actor> 登録アクター
--- @param req table|nil SHIORIリクエストテーブル（オプション）
--- @return ShioriAct アクションオブジェクト
function SHIORI_ACT.new(actors, req)
    local base = ACT.new(actors)
    base._buffer = {}
    base._current_spot = nil
    base._spot_switch_newlines = CONFIG.get("ghost", "spot_switch_newlines", 1.5)
    base.req = req  -- 追加: req フィールド
    return setmetatable(base, SHIORI_ACT_IMPL)
end
```

**2. `pasta/shiori/event/init.lua`**
```lua
local STORE = require("pasta.store")  -- 追加
local SHIORI_ACT = require("pasta.shiori.act")  -- 追加

--- act オブジェクトを生成
--- @param req table リクエストテーブル
--- @return ShioriAct act オブジェクト
local function create_act(req)
    return SHIORI_ACT.new(STORE.actors, req)
end

function EVENT.fire(req)
    local handler = REG[req.id] or EVENT.no_entry
    local act = create_act(req)  -- 追加

    local ok, result = xpcall(function()
        return handler(req, act)  -- act を第2引数に追加
    end, ...)
    -- ...
end

function EVENT.no_entry(req, act)  -- act 引数追加
    local SCENE = require("pasta.scene")
    local scene_result = SCENE.search(req.id, nil, nil)

    if scene_result then
        local ok, err = pcall(function()
            return scene_result(act)  -- act を渡す
        end)
        -- ...
    end
    return RES.no_content()
end
```

**3. `pasta/shiori/event/virtual_dispatcher.lua`**
```lua
local function execute_scene(event_name, act)  -- act 引数追加
    local scene_fn = SCENE.search(event_name, nil, nil)
    if not scene_fn then return nil end
    local ok, result = pcall(scene_fn, act)  -- act を渡す
    return result
end
```

#### トレードオフ
- ✅ 最小限のファイル変更（2-3ファイル）
- ✅ 既存パターンと整合
- ✅ テスト既存資産を大部分再利用可能
- ❌ 既存ハンドラの第2引数（act）は無視される（無害）

---

### Option B: ファクトリ関数の新規作成

**適用理由**: act 生成ロジックを独立モジュールに分離

#### 変更内容
新規ファイル: `pasta/shiori/event/act_factory.lua`
```lua
local STORE = require("pasta.store")
local SHIORI_ACT = require("pasta.shiori.act")

local M = {}

function M.create(req)
    return SHIORI_ACT.new(STORE.actors, req)
end

return M
```

#### トレードオフ
- ✅ 単一責任原則に準拠
- ✅ テスト時にファクトリをモック可能
- ❌ 新規ファイル追加
- ❌ 現段階では過剰な抽象化

---

### Option C: ハイブリッドアプローチ

**適用理由**: 段階的導入が必要な場合

#### Phase 1: act.req のみ追加
- `SHIORI_ACT.new(actors, req)` を実装
- 既存ハンドラは変更なし

#### Phase 2: イベントディスパッチ統合
- `EVENT.fire()` で act 生成・引き渡し
- 新規ハンドラシグネチャ `function(req, act)` 対応

#### トレードオフ
- ✅ 段階的リスク軽減
- ❌ 複数リリースサイクル必要
- ❌ 本機能では不要な複雑さ

---

## 4. Effort & Risk Assessment

### Effort: **S (1-3 days)**

**理由**:
- 既存パターンへの軽微な拡張
- 影響ファイル数: 2-3ファイル
- 新規依存関係なし
- テストパターン確立済み

### Risk: **Low**

**理由**:
- 既存アーキテクチャへの自然な適合
- 後方互換性は Lua の可変長引数で対応可能
- 既存テストを拡張するだけで検証可能
- ロールバック容易（パラメータ追加のみ）

---

## 5. Recommendations for Design Phase

### 推奨アプローチ
**Option A（既存コンポーネント拡張）** を採用

### キー決定事項
1. **シーン関数シグネチャ（仕様）**: すべてのシーン関数は `act` を第1引数に取る（`function __start__(act, ...)`）。非準拠は古いかバグ
2. **後方互換性戦略**: 既存ハンドラ `function(req)` は第2引数を無視する形で互換維持（Lua仕様）
3. **act 生成タイミング**: `EVENT.fire()` の冒頭で1回のみ生成
4. **STORE.actors 取得**: `EVENT.fire()` 内で直接参照（テスタビリティは SHIORI_ACT.new() の引数注入で確保）

### 追加調査項目
- **なし**: 既存実装の調査で十分な情報を取得済み

### テスト戦略
1. `shiori_act_test.lua` に `SHIORI_ACT.new(actors, req)` テストを追加
2. `shiori_event_test.rs` に act 引き渡しテストを追加
3. 既存テストの回帰確認

---

## 6. Summary

| 項目 | 評価 |
|------|------|
| 実装可能性 | ✅ 高い |
| 推奨アプローチ | Option A（既存拡張） |
| Effort | S (1-3 days) |
| Risk | Low |
| 新規ファイル | なし |
| 影響ファイル | 2-3ファイル |
