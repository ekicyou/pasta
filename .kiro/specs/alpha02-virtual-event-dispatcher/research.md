# ディスカバリーノート

## 1. 概要

### 機能タイプ

**Extension（拡張）** - 既存の SHIORI イベントシステム (`pasta.shiori.event`) に仮想イベント発行機構を追加する。

### 発見の焦点

- 既存 EVENT モジュールとの統合ポイント確認
- `@pasta_config` を使用した設定読み込みパターン
- `ctx.save` 永続化テーブルへのアクセスパターン
- boot.lua パターンに従ったデフォルトハンドラ登録

---

## 2. 統合ポイント分析

### 2.1 EVENT モジュール (`pasta.shiori.event.init`)

**ファイル**: `crates/pasta_lua/scripts/pasta/shiori/event/init.lua`

```lua
-- 既存の fire API
function EVENT.fire(req)
    local handler = REG[req.id] or EVENT.no_entry
    -- xpcall でハンドラ実行、エラーは RES.err で返却
end
```

**統合ポイント**:
- `REG.OnSecondChange` にデフォルトハンドラを登録
- `no_entry` パターンに従い、シーン関数呼び出しを実装
- `RES.ok()`, `RES.no_content()`, `RES.err()` を使用

### 2.2 REG テーブル (`pasta.shiori.event.register`)

**ファイル**: `crates/pasta_lua/scripts/pasta/shiori/event/register.lua`

```lua
-- 既存のハンドラ登録パターン
REG.EventName = function(req)
    -- 処理
    return RES.ok(script) or RES.no_content()
end
```

**統合ポイント**:
- `REG.OnSecondChange` を second_change.lua で登録
- boot.lua と同じパターン（require 時に自動登録）

### 2.3 boot.lua デフォルトハンドラパターン

**ファイル**: `crates/pasta_lua/scripts/pasta/shiori/event/boot.lua`

```lua
local REG = require "pasta.shiori.event.register"
local RES = require "pasta.shiori.res"

REG.OnBoot = function(req)
    return RES.no_content()
end

return REG
```

**パターン**:
1. REG と RES を require
2. `REG.EventName = function(req)` で登録
3. `return REG` で終了
4. init.lua で `require("pasta.shiori.event.boot")` により自動ロード

---

## 3. 設定アクセスパターン

### 3.1 @pasta_config モジュール

**使用方法**:

```lua
local config = require "@pasta_config"

-- 安全なアクセス（nilガード必須）
local ghost = config.ghost or {}
local talk_interval_min = ghost.talk_interval_min or 180
```

**特性**:
- 読み取り専用
- `[loader]` セクション除外
- TOML 型保持（数値、文字列、真偽値、配列、テーブル）
- 存在しない場合は空テーブル

### 3.2 pasta.toml 設定例

```toml
[ghost]
talk_interval_min = 180
talk_interval_max = 300
hour_margin = 30
```

---

## 4. 永続化パターン

### 4.1 ctx.save テーブル

**ファイル**: `crates/pasta_lua/scripts/pasta/ctx.lua`

```lua
-- ctx.save は @pasta_persistence により自動永続化
ctx.save.virtual_event = ctx.save.virtual_event or {
    last_talk_time = 0,
    is_talking = false,
}
```

**注意**:
- モジュールローカル変数（`next_hour_unix`）は永続化対象外
- 起動時にモジュールローカル変数は `0` に初期化

---

## 5. 時刻処理パターン

### 5.1 req.date テーブル構造

Rust 側 (`lua_request.rs`) で生成:

```lua
req.date = {
    unix = 1234567890,  -- Unix timestamp（秒）
    year = 2024,
    month = 12,
    day = 15,
    hour = 14,
    min = 30,
    sec = 45,
    wday = 0,  -- 0=日曜 〜 6=土曜
}
```

### 5.2 次の正時計算

```lua
-- 現在時刻から次の正時を計算
local function calculate_next_hour_unix(current_unix)
    -- 現在の時を切り上げて次の00分00秒
    local seconds_into_hour = current_unix % 3600
    return current_unix - seconds_into_hour + 3600
end
```

---

## 6. ランダム処理パターン

### 6.1 math.random 使用

```lua
-- Lua 5.x 標準ライブラリ
local interval = math.random(config.talk_interval_min, config.talk_interval_max)
```

**注意**:
- Lua 5.x では起動時に自動シード設定
- 追加の `math.randomseed()` 呼び出しは不要

---

## 7. シーン関数呼び出しパターン

### 7.1 SCENE.search 使用

**既存実装** (`pasta.shiori.event.init` の `no_entry`):

```lua
local SCENE = require("pasta.scene")
local scene_result = SCENE.search(req.id, nil, nil)

if scene_result then
    local ok, err = pcall(function()
        return scene_result()
    end)
    if not ok then
        return RES.err(err_msg)
    end
end
```

**virtual_dispatcher での使用**:
- `SCENE.search("OnTalk")` / `SCENE.search("OnHour")` でグローバルシーン検索
- `pcall` でラップしてエラーハンドリング
- alpha01 では戻り値無視、204 返却

---

## 8. 設計決定

### D1: モジュール構造

**決定**: 2ファイル構成
- `virtual_dispatcher.lua` - メイン判定ロジック
- `second_change.lua` - OnSecondChange デフォルトハンドラ

**根拠**: boot.lua パターンに従い、ハンドラ登録を分離

### D2: 設定キャッシュ

**決定**: モジュールローカル変数でキャッシュ

```lua
local cached_config = nil  -- 遅延初期化

local function get_config()
    if cached_config then return cached_config end
    -- 初回のみ読み込み
    cached_config = { ... }
    return cached_config
end
```

**根拠**: 毎秒呼び出されるため、設定読み込みオーバーヘッドを最小化

### D3: OnHour 判定ロジック

**決定**: Unix タイムスタンプ比較

```lua
-- モジュールローカル変数（永続化不要）
local next_hour_unix = 0

-- 判定ロジック
if next_hour_unix == 0 then
    -- 初回起動: 次の正時を計算して設定、イベント発行スキップ
    next_hour_unix = calculate_next_hour_unix(req.date.unix)
elseif req.date.unix >= next_hour_unix then
    -- 正時超過: イベント発行、次の正時を更新
    next_hour_unix = calculate_next_hour_unix(req.date.unix)
    -- OnHour 処理
end
```

**根拠**: 分単位判定（min==0）より堅牢、秒単位の誤差許容

### D4: 状態テーブル初期化

**決定**: virtual_dispatcher 内で初期化

```lua
local function ensure_state()
    if not ctx.save.virtual_event then
        ctx.save.virtual_event = {
            last_talk_time = 0,
            is_talking = false,
        }
    end
end
```

**根拠**: ctx モジュールへの依存を最小限に

---

## 9. 互換性確認

### 9.1 既存テスト

- `shiori_event_test.rs` - EVENT.fire 基本動作
- `shiori_res_test.rs` - RES モジュール

### 9.2 影響範囲

- `init.lua` - `require("pasta.shiori.event.second_change")` 追加
- 他ファイルへの変更なし

---

## 10. リスクと軽減策

| リスク | 軽減策 |
|--------|--------|
| 毎秒呼び出しによるパフォーマンス影響 | 設定キャッシュ、早期リターン |
| req.date 不在時のクラッシュ | nil チェック、204 返却 |
| 時刻ずれによる OnHour 重複発行 | Unix タイムスタンプ比較で厳密判定 |
| is_talking 状態の不整合 | alpha03 で act モジュール統合時に解決予定 |
