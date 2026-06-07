--- @module pasta.buf
--- 文字列連結用バッファの生成とバックエンド選択モジュール
---
--- LuaJIT String Buffer Library（string.buffer）が利用可能ならそれを採用し、
--- 利用不可な環境では最小実装フォールバックを提供する。さくらスクリプト等の
--- 組み立てで中間テーブルと GC 負荷を避けるための再利用可能な葉ユーティリティ。
---
--- @usage
--- local buf = require("pasta.buf")
--- local b = buf.new()
--- b:put("Hello"):put(" "):put("World")
--- local s = b:tostring()  -- "Hello World"（非破壊）

--- @class Buffer
--- @field put fun(self: Buffer, s: string): Buffer   -- 文字列を追記（追記順保持・self返却）
--- @field tostring fun(self: Buffer): string         -- 蓄積済み全片を追記順に連結（非破壊）

--- @class pasta.buf
--- @field new fun(): Buffer                 -- バックエンド object を生成して返す
--- @field backend string                    -- "luajit" | "fallback"（採用検証用）
--- @field new_fallback fun(): Buffer         -- 最小実装を明示生成（テスト用）
local M = {}

-- ----------------------------------------------------------------------------
-- 最小実装フォールバック（FallbackBuffer）
-- LuaJIT string.buffer の利用メソッド（put / tostring）と同一シグネチャを提供する。
-- 内部配列 _parts へ追記し、tostring は table.concat で追記順連結（非破壊）。
-- ----------------------------------------------------------------------------

--- @class FallbackBuffer
local FallbackBuffer = {}
FallbackBuffer.__index = FallbackBuffer

--- 文字列を追記する（追記順を保持・チェーン可）
--- @param s string 追記する文字列
--- @return FallbackBuffer self
function FallbackBuffer:put(s)
    local parts = self._parts
    parts[#parts + 1] = s
    return self
end

--- 蓄積済みの全片を追記順に連結して返す（非破壊）
--- @return string
function FallbackBuffer:tostring()
    return table.concat(self._parts)
end

--- 最小実装バッファを明示生成する（テスト seam・フォールバック実走用）
--- @return FallbackBuffer
function M.new_fallback()
    return setmetatable({ _parts = {} }, FallbackBuffer)
end

-- ----------------------------------------------------------------------------
-- バックエンド選択（ロード時に1度だけ評価し、例外を送出しない）
-- ネイティブ採用時は object をラップせず生成関数を直束縛する（性能維持）。
-- ----------------------------------------------------------------------------

local ok, sb = pcall(require, "string.buffer")
if ok and sb and type(sb.new) == "function" then
    M.new = sb.new
    M.backend = "luajit"
else
    M.new = M.new_fallback
    M.backend = "fallback"
end

return M
