# Technical Design Document

## 1. Overview

### Feature Summary

OnSecondChange イベントをトリガーとして、OnTalk（ランダムトーク）と OnHour（時報）仮想イベントを条件判定・発行する機構を実装する。

### Background

伺かゴーストにおいて、ユーザー操作がなくてもキャラクターが自発的に話しかける「ランダムトーク」と、正時に挨拶する「時報」は基本機能である。本仕様では、ベースウェアから毎秒通知される OnSecondChange イベントを起点として、これらの仮想イベントを発行する判定ロジックを Lua モジュールとして実装する。

### Scope

**In Scope:**
- OnTalk/OnHour 発行条件判定ロジック
- モジュール内部状態管理（next_hour_unix, next_talk_time）
- 設定読み込み（pasta.toml [ghost]セクション）
- OnSecondChange デフォルトハンドラ
- シーン関数呼び出し（SCENE.search）

**Out of Scope:**
- さくらスクリプト組み立て（alpha03）
- 実際のトーク内容生成（alpha04）

### Requirements Traceability

| Requirement ID | Design Section |
|---------------|---------------|
| REQ-1 | 3.1 OnTalk Flow, 4.1 virtual_dispatcher |
| REQ-2 | 3.2 OnHour Flow, 4.1 virtual_dispatcher |
| REQ-3 | 5.1 State Model |
| REQ-4 | 5.2 Time Model |
| REQ-5 | 5.3 Config Model, 4.1 virtual_dispatcher |
| REQ-6 | 4.2 second_change |
| REQ-7 | 6. Testing Strategy |
| REQ-8 | 7. Documentation |

---

## 2. Architecture

### 2.1 System Context

```
┌─────────────────────────────────────────────────────────┐
│                    ベースウェア (SSP)                     │
│                                                         │
│  OnSecondChange 毎秒通知                                 │
└───────────────────────────┬─────────────────────────────┘
                            │ SHIORI/3.0 Request
                            ▼
┌─────────────────────────────────────────────────────────┐
│                    pasta_shiori (Rust)                   │
│                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │ lua_request │───▶│   req.date  │───▶│  Lua Runtime │ │
│  │   parser    │    │   生成      │    │             │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
└───────────────────────────┬─────────────────────────────┘
                            │ EVENT.fire(req)
                            ▼
┌─────────────────────────────────────────────────────────┐
│                  pasta.shiori.event (Lua)               │
│                                                         │
│  ┌─────────────┐    ┌─────────────────────────────────┐ │
│  │    REG      │    │ second_change (デフォルトハンドラ)│ │
│  │ [OnSecond   │───▶│    virtual_dispatcher.dispatch  │ │
│  │  Change]    │    └─────────────┬───────────────────┘ │
│  └─────────────┘                  │                     │
│                                   ▼                     │
│  ┌─────────────────────────────────────────────────────┐│
│  │             virtual_dispatcher                      ││
│  │  ┌─────────┐    ┌─────────┐    ┌─────────┐         ││
│  │  │ OnHour  │───▶│ OnTalk  │───▶│  終了   │         ││
│  │  │  判定   │    │  判定   │    │ (204)   │         ││
│  │  └────┬────┘    └────┬────┘    └─────────┘         ││
│  │       │              │                              ││
│  │       ▼              ▼                              ││
│  │  SCENE.search   SCENE.search                        ││
│  │  ("OnHour")     ("OnTalk")                          ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

### 2.2 Module Structure

```
crates/pasta_lua/scripts/pasta/shiori/event/
├── init.lua           # EVENT モジュール（既存）
├── register.lua       # REG ハンドラテーブル（既存）
├── boot.lua           # OnBoot デフォルトハンドラ（既存）
├── second_change.lua  # OnSecondChange デフォルトハンドラ【新規】
└── virtual_dispatcher.lua  # 仮想イベント判定・発行【新規】
```

### 2.3 Dependency Graph

```
second_change.lua
    │
    ├──▶ pasta.shiori.event.register (REG)
    ├──▶ pasta.shiori.res (RES)
    └──▶ pasta.shiori.event.virtual_dispatcher
             │
             ├──▶ pasta.shiori.event.register (REG)
             ├──▶ pasta.shiori.res (RES)
             ├──▶ pasta.scene (SCENE)
             └──▶ @pasta_config (設定)
```

---

## 3. System Flows

### 3.1 OnTalk 発行フロー

```
┌───────────────┐
│ OnSecondChange│
│   受信        │
└───────┬───────┘
        │
        ▼
┌───────────────┐     ┌───────────────┐
│ req.date      │ No  │ return        │
│ 存在チェック  │────▶│ RES.no_content│
└───────┬───────┘     └───────────────┘
        │ Yes
        ▼
┌───────────────┐     ┌───────────────┐
│ OnHour 判定   │ Yes │ OnHour 処理   │
│ 優先実行      │────▶│ (フロー 3.2)  │
└───────┬───────┘     └───────────────┘
        │ No
        ▼
┌───────────────┐     ┌───────────────┐
│ req.status    │ Yes │ return nil    │
│ == "talking" │────▶│ (スキップ)    │
└───────┬───────┘     └───────────────┘
        │ No
        ▼
┌───────────────┐     ┌───────────────┐
│ next_talk_time│ No  │ return nil    │
│ == 0 (初回)   │────▶│ (初期化のみ)  │
└───────┬───────┘     └───────────────┘
        │ Yes
        ▼
┌───────────────┐     ┌───────────────┐
│ 時報マージン  │ No  │ return nil    │
│ チェック      │────▶│ (スキップ)    │
└───────┬───────┘     └───────────────┘
        │ Yes (余裕あり)
        ▼
┌───────────────┐
│ SCENE.search  │
│ ("OnTalk")    │
└───────┬───────┘
        │
        ▼
┌───────────────┐     ┌───────────────┐
│ シーン        │ No  │ return nil    │
│ 存在チェック  │────▶│ (スキップ)    │
└───────┬───────┘     └───────────────┘
        │ Yes
        ▼
┌───────────────┐
│ next_talk_time│
│ = 次回予定    │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ シーン関数    │
│ 実行 (pcall)  │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ return result │
│ or nil        │
└───────────────┘
```

### 3.2 OnHour 発行フロー

```
┌───────────────┐
│ OnHour 判定   │
│ 開始          │
└───────┬───────┘
        │
        ▼
┌───────────────┐     ┌───────────────┐
│ next_hour_unix│ Yes │ 次の正時計算  │
│ == 0          │────▶│ 設定のみ      │────▶ return nil
└───────┬───────┘     └───────────────┘      (初回はスキップ)
        │ No
        ▼
┌───────────────┐     ┌───────────────┐
│ req.date.unix │ No  │ return nil    │
│ >= next_hour  │────▶│ (未到達)      │
└───────┬───────┘     └───────────────┘
        │ Yes
        ▼
┌───────────────┐     ┌───────────────┐
│ req.status    │ Yes │ return nil    │
│ == "talking" │────▶│ (スキップ)    │
└───────┬───────┘     └───────────────┘
        │ No
        ▼
┌───────────────┐
│ next_hour_unix│
│ = 次の正時    │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ SCENE.search  │
│ ("OnHour")    │
└───────┬───────┘
        │
        ▼
┌───────────────┐     ┌───────────────┐
│ シーン        │ No  │ return nil    │
│ 存在チェック  │────▶│ (スキップ)    │
└───────┬───────┘     └───────────────┘
        │ Yes
        ▼
┌───────────────┐
│ シーン関数    │
│ 実行 (pcall)  │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ return result │
│ ("fired")     │
└───────────────┘
```

### 3.3 初期化フロー

```
┌───────────────┐
│ モジュール    │
│ ロード時      │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ 設定読み込み  │
│ @pasta_config │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ キャッシュ    │
│ 保存          │
└───────┬───────┘
        │
        ▼
┌───────────────┐
│ next_hour_unix│
│ = 0 (初期値)  │
└───────────────┘
```

---

## 4. Components

### 4.1 virtual_dispatcher モジュール

**ファイル**: `crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua`

#### 4.1.1 モジュール構造

```lua
---@module pasta.shiori.event.virtual_dispatcher
--- 仮想イベント（OnTalk/OnHour）の条件判定・発行モジュール
---
--- OnSecondChange をトリガーとして、以下の仮想イベントを発行:
--- - OnHour: 正時に発行（優先）
--- - OnTalk: 一定時間経過後にランダム発行
---
--- 使用例:
--- ```lua
--- local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
--- local result = dispatcher.dispatch(req)
--- ```

-- 1. require文
local RES = require("pasta.shiori.res")

-- 2. モジュールローカル変数
local next_hour_unix = 0      -- 次の正時タイムスタンプ
local next_talk_time = 0      -- 次回トーク発行予定時刻
local cached_config = nil     -- 設定キャッシュ

-- 3. モジュールテーブル宣言
---@class VirtualDispatcher
local M = {}

-- 4. 内部関数

--- 設定を読み込み・キャッシュ
---@return table 設定テーブル
local function get_config()
    if cached_config then return cached_config end
    
    local ok, config = pcall(require, "@pasta_config")
    if not ok then config = {} end
    
    local ghost = config.ghost or {}
    cached_config = {
        talk_interval_min = ghost.talk_interval_min or 180,
        talk_interval_max = ghost.talk_interval_max or 300,
        hour_margin = ghost.hour_margin or 30,
    }
    return cached_config
end

--- 次の正時タイムスタンプを計算
---@param current_unix number 現在のUnixタイムスタンプ
---@return number 次の正時のUnixタイムスタンプ
local function calculate_next_hour_unix(current_unix)
    local seconds_into_hour = current_unix % 3600
    return current_unix - seconds_into_hour + 3600
end

--- 次回トーク時刻を計算
---@param current_unix number 現在のUnixタイムスタンプ
---@return number 次回トーク予定時刻
local function calculate_next_talk_time(current_unix)
    local cfg = get_config()
    local interval = math.random(cfg.talk_interval_min, cfg.talk_interval_max)
    return current_unix + interval
end

--- シーン関数を実行
---@param event_name string イベント名 ("OnTalk" or "OnHour")
---@return string|nil 実行結果（エラー時は nil）
local function execute_scene(event_name)
    local SCENE = require("pasta.scene")
    local scene_fn = SCENE.search(event_name, nil, nil)
    
    if not scene_fn then
        return nil
    end
    
    local ok, result = pcall(scene_fn)
    if not ok then
        -- エラーログ出力（既存EVENT.no_entry()パターンに準拠）
        -- 将来的にはtracing/loggingモジュールに切り替え予定
        local err_msg = result
        if type(result) == "string" then
            err_msg = result:match("^[^\n]+") or result
        end
        print("[virtual_dispatcher] Scene execution error (" .. event_name .. "): " .. tostring(err_msg))
        return nil
    end
    
    return result
end

-- 5. 公開関数

--- OnHour 判定・発行
---@param req table リクエストテーブル
---@return string|nil "fired" (発行成功), nil (発行なし)
function M.check_hour(req)
    local current_unix = req.date.unix
    
    -- 初回起動: 次の正時を計算して設定、発行スキップ
    if next_hour_unix == 0 then
        next_hour_unix = calculate_next_hour_unix(current_unix)
        return nil
    end
    
    -- 正時未到達
    if current_unix < next_hour_unix then
        return nil
    end
    
    -- トーク中はスキップ（SSPからの状態情報を使用）
    if req.status == "talking" then
        return nil
    end
    
    -- 次の正時を更新
    next_hour_unix = calculate_next_hour_unix(current_unix)
    
    -- シーン実行
    local result = execute_scene("OnHour")
    
    return result and "fired" or nil
end

--- OnTalk 判定・発行
---@param req table リクエストテーブル
---@return string|nil "fired" (発行成功), nil (発行なし)
function M.check_talk(req)
    local current_unix = req.date.unix
    local cfg = get_config()
    
    -- トーク中はスキップ（SSPからの状態情報を使用）
    if req.status == "talking" then
        return nil
    end
    
    -- 初回または次回トーク時刻未設定
    if next_talk_time == 0 then
        next_talk_time = calculate_next_talk_time(current_unix)
        return nil
    end
    
    -- 次回トーク時刻未到達
    if current_unix < next_talk_time then
        return nil
    end
    
    -- 時報マージンチェック（正時が近い場合はスキップ）
    local time_to_hour = next_hour_unix - current_unix
    if time_to_hour > 0 and time_to_hour < cfg.hour_margin then
        return nil
    end
    
    -- シーン実行
    local result = execute_scene("OnTalk")
    
    -- 次回トーク時刻を再計算（発行成否に関わらず）
    next_talk_time = calculate_next_talk_time(current_unix)
    
    return result and "fired" or nil
end

--- 仮想イベントディスパッチ（メインエントリポイント）
---@param req table リクエストテーブル
---@return string|nil シーン実行結果 or nil
function M.dispatch(req)
    -- req.date 存在チェック
    if not req.date then
        return nil
    end
    
    -- OnHour 判定（優先）
    local hour_result = M.check_hour(req)
    if hour_result then
        return hour_result
    end
    
    -- OnTalk 判定
    local talk_result = M.check_talk(req)
    return talk_result
end

-- 6. テスト用関数（内部状態リセット）

--- テスト用: 内部状態をリセット
function M._reset()
    next_hour_unix = 0
    next_talk_time = 0
    cached_config = nil
end

--- テスト用: 内部状態を取得
---@return table 内部状態
function M._get_internal_state()
    return {
        next_hour_unix = next_hour_unix,
        next_talk_time = next_talk_time,
        cached_config = cached_config,
    }
end

-- 7. 末尾で返却
return M
```

#### 4.1.2 API 仕様

| 関数 | 引数 | 戻り値 | 説明 |
|------|------|--------|------|
| `dispatch(req)` | req: table | string\|nil | メインエントリ。OnHour/OnTalk を判定・発行 |
| `check_hour(req)` | req | "fired"\|nil | OnHour 判定 |
| `check_talk(req)` | req | "fired"\|nil | OnTalk 判定 |
| `_reset()` | なし | なし | テスト用内部状態リセット |
| `_get_internal_state()` | なし | table | テスト用内部状態取得 |

### 4.2 second_change モジュール

**ファイル**: `crates/pasta_lua/scripts/pasta/shiori/event/second_change.lua`

```lua
---@module pasta.shiori.event.second_change
--- OnSecondChange デフォルトハンドラ
---
--- 仮想イベントディスパッチャを呼び出し、結果に応じてレスポンスを返す。
--- ゴースト開発者は REG.OnSecondChange を上書きしてカスタムハンドラを設定可能。

local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")
local dispatcher = require("pasta.shiori.event.virtual_dispatcher")

---OnSecondChange デフォルトハンドラ
---@param req table SHIORI リクエストテーブル
---@return string SHIORI レスポンス
REG.OnSecondChange = function(req)
    local result = dispatcher.dispatch(req)
    
    if result then
        -- alpha01: シーン実行成功でも 204 を返す
        -- alpha03: さくらスクリプト変換・200 OK を返す
        return RES.no_content()
    end
    
    return RES.no_content()
end

return REG
```

### 4.3 init.lua 変更

**変更点**: second_change.lua のロード追加

```lua
-- 1.5. デフォルトイベントハンドラをロード
require("pasta.shiori.event.boot")
require("pasta.shiori.event.second_change")  -- 追加
```

---

## 5. Data Models

### 5.1 Module Local State

```lua
-- モジュールローカル変数（セッション中のみ有効）
local next_hour_unix = 0      -- 次の正時タイムスタンプ
local next_talk_time = 0      -- 次回トーク予定時刻
local cached_config = nil     -- 設定キャッシュ
```

| 変数 | 型 | 初期値 | 永続化 | 説明 |
|-----|-----|--------|--------|------|
| `next_hour_unix` | number | 0 | ✗ | 次の正時タイムスタンプ（セッション中有効） |
| `next_talk_time` | number | 0 | ✗ | 次回トーク予定時刻（セッション中有効） |
| `cached_config` | table\|nil | nil | ✗ | 設定キャッシュ |

### 5.2 Time Model (req.date)

**Rust側実装確認済み**: `pasta_shiori/src/lua_request.rs::parse_request()` L54で`lua_date()`により生成。全てのSHIORI REQUESTに自動付与される。

```lua
---@class ReqDate
---@field unix number Unix timestamp（秒）
---@field year number 年
---@field month number 月（1-12）
---@field day number 日（1-31）
---@field hour number 時（0-23）
---@field min number 分（0-59）
---@field sec number 秒（0-59）
---@field wday number 曜日（0=日曜 〜 6=土曜）
req.date = {
    unix = 1234567890,
    year = 2024,
    month = 12,
    day = 15,
    hour = 14,
    min = 0,
    sec = 0,
    wday = 0,
}
```

### 5.3 Config Model (pasta.toml [ghost])

```toml
[ghost]
talk_interval_min = 180   # トーク最小間隔（秒）
talk_interval_max = 300   # トーク最大間隔（秒）
hour_margin = 30          # 時報前マージン（秒）
```

| 設定項目 | 型 | デフォルト | 説明 |
|---------|-----|----------|------|
| `talk_interval_min` | number | 180 | トーク最小間隔（秒） |
| `talk_interval_max` | number | 300 | トーク最大間隔（秒） |
| `hour_margin` | number | 30 | 時報前マージン（秒） |

---

## 6. Testing Strategy

### 6.1 テストファイル

**ファイル**: `crates/pasta_lua/tests/virtual_event_dispatcher_test.rs`

### 6.2 テストケース

| テストケース | 検証内容 |
|-------------|---------|
| `test_dispatch_without_req_date` | req.date 不在時に nil 返却 |
| `test_onhour_first_run_skip` | 初回起動時は OnHour 発行スキップ |
| `test_onhour_fires_at_hour` | 正時超過時に OnHour 発行 |
| `test_ontalk_interval_check` | interval 経過前はスキップ |
| `test_ontalk_fires_after_interval` | interval 経過後に発行 |
| `test_ontalk_hour_margin_skip` | 時報前マージン内はスキップ |
| `test_onhour_priority_over_ontalk` | OnHour が OnTalk より優先 |
| `test_config_default_values` | 設定未定義時のデフォルト値 |
| `test_module_state_reset` | モジュール再読み込み時の状態リセット |
| `test_skip_when_talking` | req.status=="talking"時はスキップ |

### 6.3 Lua ユニットテスト

**ファイル**: `crates/pasta_lua/tests/lua_specs/virtual_dispatcher_spec.lua`

```lua
describe("virtual_dispatcher", function()
    local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
    
    before_each(function()
        dispatcher._reset()
    end)
    
    describe("dispatch", function()
        it("returns nil when req.date is missing", function()
            local result = dispatcher.dispatch({})
            expect(result).to_be_nil()
        end)
    end)
    
    describe("check_hour", function()
        it("initializes next_hour_unix on first run", function()
            -- ...
        end)
    end)
end)
```

---

## 7. Documentation

### 7.1 LUA_API.md 追記

```markdown
## X. pasta.shiori.event.virtual_dispatcher モジュール

OnSecondChange をトリガーとして仮想イベント（OnTalk/OnHour）を発行する。

### 公開 API

| 関数 | 説明 |
|------|------|
| `dispatch(req)` | 仮想イベント判定・発行 |

### 発行条件

**OnHour（優先）**:
- 現在時刻が次の正時を超過
- 非トーク中

**OnTalk**:
- 前回トークから設定間隔経過
- 非トーク中
- 時報前マージン外

### 設定 (pasta.toml)

```toml
[ghost]
talk_interval_min = 180
talk_interval_max = 300
hour_margin = 30
```
```

---

## 8. Implementation Notes

### 8.1 ファイル作成順序

1. `virtual_dispatcher.lua` - メインロジック
2. `second_change.lua` - デフォルトハンドラ
3. `init.lua` 修正 - require 追加
4. テストファイル作成

### 8.2 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `init.lua` | `require("pasta.shiori.event.second_change")` 追加 |

### 8.3 alpha03 への引き継ぎ事項

- シーン実行結果のさくらスクリプト変換
- `dispatch()` の戻り値を 200 OK レスポンスに変換

---

## Appendix

### A. 次の正時計算式

```lua
-- current_unix = 1702648800 (2023-12-15 14:00:00 UTC)
local seconds_into_hour = current_unix % 3600  -- 0
local next_hour = current_unix - seconds_into_hour + 3600  -- 1702652400 (15:00:00)

-- current_unix = 1702650123 (2023-12-15 14:22:03 UTC)
local seconds_into_hour = 1702650123 % 3600  -- 1323
local next_hour = 1702650123 - 1323 + 3600  -- 1702652400 (15:00:00)
```

### B. ランダム間隔計算

```lua
-- min=180, max=300 の場合
local interval = math.random(180, 300)  -- 180〜300 の整数
local next_talk = current_unix + interval
```
