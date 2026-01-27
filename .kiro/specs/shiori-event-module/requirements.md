# Requirements Document

## Project Description (Input)
# 要件書: pasta.shiori.event モジュール作成

## 目的
SHIORI イベントの振り分けとハンドラ登録の仕組みを提供するモジュールを作成する

## ファイル構成
```
crates/pasta_lua/scripts/pasta/shiori/event/
├── init.lua      -- イベント振り分け（EVENT.fire, EVENT.no_entry）
└── register.lua  -- ハンドラ登録テーブル
```

## 依存モジュール
- `pasta.shiori.res` — レスポンス組み立て（別途作成済み想定）

---

## register.lua

### 目的
イベントハンドラを登録するためのテーブルを提供する

### 要件
```lua
--- @module pasta.shiori.event.register
--- イベントハンドラ登録テーブル
---
--- 使用例:
---   local REG = require("pasta.shiori.event.register")
---   local RES = require("pasta.shiori.res")
---
---   REG.OnBoot = function(req)
---       return RES.ok([[\0\s[0]こんにちは\e]])
---   end

local REG = {}

return REG
```

### ポイント
- 単純な Lua テーブル
- `REG.イベント名 = function(req) ... end` の形式で登録
- 特別な登録関数は不要（Lua らしくテーブルに直接代入）

---

## init.lua

### 目的
イベントの振り分けとデフォルトハンドラを提供する

### 要件
```lua
--- @module pasta.shiori.event
--- イベント振り分けモジュール
---
--- SHIORI リクエストのイベント ID に応じてハンドラを呼び分ける。
--- 未登録イベントはデフォルトハンドラ（no_entry）で処理する。
--- エラー発生時は xpcall でキャッチし、エラーレスポンスに変換する。

local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

local EVENT = {}

--- デフォルトハンドラ（未登録イベント用）
--- @param req table リクエストテーブル
--- @return string SHIORI レスポンス（204 No Content）
function EVENT.no_entry(req)
    return RES.no_content()
end

--- イベント振り分け
--- @param req table リクエストテーブル（req.id にイベント名）
--- @return string SHIORI レスポンス
function EVENT.fire(req)
    local handler = REG[req.id] or EVENT.no_entry
    
    local ok, result = xpcall(function()
        return handler(req)
    end, debug.traceback)
    
    if ok then
        return result
    else
        -- エラー時は 500 Internal Server Error + エラー情報
        return RES.err(result)
    end
end

return EVENT
```

### ポイント
- `REG[req.id]` でハンドラ取得（未登録なら `nil`）
- 未登録イベント → `EVENT.no_entry` → 204 No Content
- `xpcall` でエラーキャッチ → `RES.err()` でエラーレスポンス
- `xpcall` は `EVENT.fire` の外側のみ（内部の末尾再帰を妨げない）

---

## 使用例

### ゴースト開発者がハンドラを登録する場合
```lua
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

REG.OnBoot = function(req)
    return RES.ok([[\0\s[0]こんにちは\e]])
end

REG.OnClose = function(req)
    return RES.ok([[\0\s[0]さようなら\e]])
end

REG.OnMouseDoubleClick = function(req)
    return RES.ok([[\0\s[0]なあに？\e]])
end
```

### pasta.shiori.main から呼び出す場合
```lua
local EVENT = require("pasta.shiori.event")

function SHIORI.request(req)
    return EVENT.fire(req)
end
```

---

## req テーブル構造（参考）
Rust 側から渡されるパース済みリクエスト：
```lua
req = {
    method = "get",
    version = 30,
    id = "OnBoot",           -- イベント名（これで振り分け）
    charset = "UTF-8",
    sender = "SSP",
    reference = {            -- 0始まりの配列
        [0] = "master",
        [1] = "...",
    },
}
```

---

## テスト観点
1. 登録済みイベント → 対応するハンドラが呼ばれる
2. 未登録イベント → `EVENT.no_entry` が呼ばれ、204 が返る
3. ハンドラ内でエラー発生 → 500 + エラー情報が返る（例外が飛ばない）
4. `req.id` が `nil` の場合 → `no_entry` で処理される

---

## 除外事項
- `data` 引数（状態管理）は含めない
- 外部ファイルからの自動登録機構は含めない（手動で `REG.XXX = ...` する）
- シーン検索・実行はこのモジュールでは扱わない

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
