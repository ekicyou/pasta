# Gap Analysis: onhour-date-var-transfer

## Executive Summary

OnHour仮想イベント発火時に `req.date.XXX` フィールドを `act.var.XXX` へ転記する機能のギャップ分析結果。

**スコープ**: Lua レイヤーのみ（virtual_dispatcher.lua と関連モジュール）
**複雑度**: S（1-3日）
**リスク**: Low - 既存パターンの軽微な拡張

### 主要な発見事項

1. **act が scene_fn に渡されていない問題（実装ミス確認済み）**: 現在の `execute_scene()` は `pcall(scene_fn)` で act を渡していない。これは act-req-parameter 仕様の実装ミスであり、本仕様で修正する
2. **転記タイミング**: OnHour 発火判定後、シーン実行前に `act.var` への転記を行う必要がある
3. **変数名マッピング**: pasta DSL の `＄時` → `act.var.hour` 変換は別仕様として切り出し（本仕様スコープ外）

---

## 1. Current State Investigation

### 1.1 Domain-Related Assets

| Asset | Path | Role |
|-------|------|------|
| virtual_dispatcher.lua | `crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua` | 仮想イベント判定・発行 |
| act.lua | `crates/pasta_lua/scripts/pasta/act.lua` | Act オブジェクト定義、`var` テーブル保持 |
| shiori/act.lua | `crates/pasta_lua/scripts/pasta/shiori/act.lua` | SHIORI_ACT、`req` フィールド保持 |
| second_change.lua | `crates/pasta_lua/scripts/pasta/shiori/event/second_change.lua` | OnSecondChange ハンドラ |
| code_generator.rs | `crates/pasta_lua/src/code_generator.rs` | 変数参照 Lua コード生成 |

### 1.2 Existing Architecture Patterns

**仮想イベントディスパッチフロー**:
```
OnSecondChange → second_change.lua → dispatcher.dispatch(act)
                                         ↓
                                   check_hour(act) / check_talk(act)
                                         ↓
                                   execute_scene(event_name)  ← act が渡されていない
                                         ↓
                                   SCENE.search() → scene_fn
                                         ↓
                                   pcall(scene_fn)  ← act なしで呼び出し
```

**変数管理**:
- `act.var` = アクションローカル変数（シーン実行中のみ有効）
- `save` = 永続変数（グローバル）
- トランスパイラーは `＄変数名` → `var.変数名` を生成

### 1.3 Integration Surfaces

| Surface | Detail |
|---------|--------|
| execute_scene() | シーン実行関数 - act 引き渡しが必要 |
| check_hour(act) | OnHour 判定 - ここで転記処理を追加 |
| act.var テーブル | 転記先 - 既存構造を利用 |
| req.date フィールド | 転記元 - 既存構造を利用 |

---

## 2. Requirements Feasibility Analysis

### 2.1 Requirement-to-Asset Map

| Requirement | Existing Asset | Gap |
|-------------|---------------|-----|
| REQ-1: OnHour発火時に日時変数設定 | check_hour(act) | **Missing**: 転記ロジック |
| REQ-2: 転記フィールド定義 | req.date, act.var | OK - 既存構造利用可能 |
| REQ-3: execute_scene への act 引き渡し | execute_scene() | **Bug Fix**: act 引数追加 |
| REQ-4: 既存動作との互換性 | virtual_dispatcher | OK - 軽微な拡張 |

### 2.2 Technical Needs

1. **転記ロジック追加** (check_hour 内):
   ```lua
   -- OnHour 発火確定後、シーン実行前
   local function transfer_date_to_var(act)
       if act.req and act.req.date then
           for key, value in pairs(act.req.date) do
               act.var[key] = value
           end
       end
   end
   ```

2. **execute_scene への act 引き渡し**:
   - 現在: `execute_scene(event_name)` → `pcall(scene_fn)`
   - 必要: `execute_scene(event_name, act)` → `pcall(scene_fn, act)`
   - **注意**: act-req-parameter 仕様との整合性確認が必要

3. **日本語変数名マッピング** (Research Needed):
   - `＄時` → `act.var.hour` または `act.var.時` か
   - トランスパイラー層 vs ランタイム層での対応検討

### 2.3 Constraints & Unknowns

| Item | Type | Detail |
|------|------|--------|
| act → scene_fn | Constraint | 現在 execute_scene() は act を渡していない。修正が必要。 |
| 変数名マッピング | Research Needed | 日本語変数名(`時`)と英語フィールド名(`hour`)の対応方式 |
| OnTalk との差異 | Design Decision | OnTalk でも同様の転記を行うか |

---

## 3. Implementation Approach Options

### Option A: execute_scene 拡張（推奨）

**概要**: execute_scene() に act を渡し、シーン実行前に転記処理を実行

**変更ファイル**:
1. `virtual_dispatcher.lua`
   - `execute_scene(event_name)` → `execute_scene(event_name, act)`
   - `check_hour(act)` 内で転記処理追加
   - `pcall(scene_fn)` → `pcall(scene_fn, act)`
   - テスト用 `scene_executor(event_name)` → `scene_executor(event_name, act)`

**Trade-offs**:
- ✅ 最小限の変更
- ✅ 既存パターン（act 引き渡し）に準拠
- ✅ テスト容易
- ❌ OnTalk への影響を考慮必要

### Option B: 転記専用ヘルパー関数

**概要**: 転記処理を独立したヘルパー関数として実装

**変更ファイル**:
1. `virtual_dispatcher.lua` - ヘルパー関数追加
2. 将来的に他のイベントでも再利用可能

**Trade-offs**:
- ✅ 再利用性が高い
- ✅ テスト単体が容易
- ❌ ファイル追加の可能性

### Option C: Hybrid (A + 変数名マッピング)

**概要**: Option A + 日本語変数名マッピング対応

**追加対応**:
- ランタイム層で `hour` → `時` エイリアス設定
- または トランスパイラー層で `＄時` → `var.hour` 変換ルール

**Trade-offs**:
- ✅ 完全な日本語対応
- ❌ 設計判断が必要（ランタイム vs トランスパイラー）
- ❌ スコープ拡大リスク

---

## 4. Implementation Complexity & Risk

### Effort: S (1-3 days)

**理由**:
- 既存パターンの軽微な拡張
- 変更ファイル数: 1-2ファイル
- テストパターンは既存を流用可能

### Risk: Low

**理由**:
- 既存アーキテクチャに準拠
- 後方互換性維持可能
- 影響範囲が限定的（virtual_dispatcher のみ）

**中リスク要因解決済み**:
- execute_scene への act 引き渡し変更が act-req-parameter 仕様と競合する可能性
  - → **確認済み**: act-req-parameter の実装ミスであることを開発者が確認
  - → 本仕様で修正をスコープに含めることで承認済み

---

## 5. Recommendations for Design Phase

### 5.1 Preferred Approach

**Option A: execute_scene 拡張** を推奨

理由:
1. 最小限の変更で要件を満たす
2. 既存のテストパターンを流用可能
3. 将来的な拡張（OnTalk対応等）も容易

### 5.2 Key Design Decisions

1. **転記タイミング**: シーン実行直前（execute_scene 内）vs check_hour 内
2. **転記フィールド範囲**: 
   - 必須: `unix`, `hour`, `minute`, `second`
   - 任意: `year`, `month`, `day`, `weekday`
3. **変数名マッピング方針**:
   - Phase 1: 英語フィールド名のみ（`act.var.hour`）
   - Phase 2（将来）: 日本語エイリアス対応

### 5.3 Research Items to Carry Forward

| Item | Priority | Detail |
|------|----------|--------|
| ~~act-req-parameter 実装状態確認~~ | ~~P0~~ | **完了**: 実装ミス確認済み、本仕様で修正 |
| ~~日本語変数名マッピング設計~~ | ~~P1~~ | **スコープ外**: 別仕様として切り出し |
| OnTalk 転記対応 | P2 | OnTalk でも同様の転記を行うか検討（議題 #2） |

---

## 6. Appendix: Code Snippets

### 現在の execute_scene 実装

```lua
local function execute_scene(event_name)
    if scene_executor then
        return scene_executor(event_name)
    end

    local SCENE = require("pasta.scene")
    local scene_fn = SCENE.search(event_name, nil, nil)

    if not scene_fn then
        return nil
    end

    local ok, result = pcall(scene_fn)  -- act が渡されていない
    -- ...
end
```

### 提案する実装（概要）

```lua
--- 日時フィールドを act.var に転記
local function transfer_date_to_var(act)
    if not act or not act.req or not act.req.date then
        return
    end
    for key, value in pairs(act.req.date) do
        act.var[key] = value
    end
end

local function execute_scene(event_name, act)
    if scene_executor then
        return scene_executor(event_name, act)
    end

    local SCENE = require("pasta.scene")
    local scene_fn = SCENE.search(event_name, nil, nil)

    if not scene_fn then
        return nil
    end

    local ok, result = pcall(scene_fn, act)  -- act を渡す
    -- ...
end

function M.check_hour(act)
    -- ... 既存の判定ロジック ...

    -- OnHour 発火確定後
    transfer_date_to_var(act)  -- 転記処理

    local result = execute_scene("OnHour", act)
    return result and "fired" or nil
end
```
