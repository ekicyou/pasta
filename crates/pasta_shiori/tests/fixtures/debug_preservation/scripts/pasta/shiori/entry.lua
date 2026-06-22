--- @module pasta.shiori.entry
--- テストフィクスチャ: 作者デバッグ保全テスト用エントリーポイント（task 6.1）。
---
--- 実プロダクションの EVENT.fire パイプラインを使用する。`OnDebugScene` は
--- ブレークポイントを張る決定的な行（`debug_local` 代入行）を持ち、停止中に
--- ローカル変数 `debug_local` を inspect できる。VM はアクタースレッドに pin
--- されるため、この scene 実行中のラインフック発火＝アクタースレッド発火である
--- （R10.1/R10.2）。

local EVENT = require("pasta.shiori.event")
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

SHIORI = SHIORI or {}

local function error_handler(err)
    if type(err) == "string" then
        return err:match("^[^\n]+")
    end
    return nil
end

function SHIORI.load(hinst, load_dir)
    return true
end

function SHIORI.request(req)
    local ok, result = xpcall(function()
        return EVENT.fire(req)
    end, error_handler)

    if ok then
        return result
    else
        return RES.err(result)
    end
end

function SHIORI.unload()
end

-- ============================================================================
-- デバッグ保全テスト用 scene ハンドラ。
-- ============================================================================

--- ブレークポイント対象の scene。`debug_local` 代入行（下記の固定行）に BP を張り、
--- 停止フレームでローカル `debug_local` を inspect する。VM がアクタースレッド上で
--- 実行されるため、ここでフックが発火する＝フックがアクタースレッドで発火する証跡。
REG.OnDebugScene = function(act)
    return coroutine.create(function(act)
        local debug_local = 4242
        local marker = debug_local + 1     -- BREAKPOINT 行: ここで `debug_local` は
                                            -- 確定済みの named local（=4242）として inspect 可能。
        act:raw_script("debug=" .. tostring(debug_local) .. ",m=" .. tostring(marker) .. "\\e")
        coroutine.yield(act:build())
    end)
end

return SHIORI
