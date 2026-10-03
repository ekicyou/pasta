--- @class CT
--- @field private _cleanups fun()[] 登録されたクリーンアップ関数のリスト
--- @field private _cancelled boolean キャンセルフラグ
--- スコープから外れたときに登録関数を実行します。
local IMPL = {}

--- クリーンアップ関数を登録します。
--- @param fn fun() クリーンアップ関数
function IMPL.defer(self, fn)
    table.insert(self._cleanups, fn)
    return self
end

--- スコープのクリーンアップをキャンセルします。
function IMPL.cancel(self)
    self._cancelled = true
    self._cleanups = {}
end

--- スコープから外れたときに登録関数を実行します。
function IMPL.__close(self, err)
    if self._cancelled then return end
    for i = #self._cleanups, 1, -1 do
        pcall(self._cleanups[i], err)
    end
    self._cleanups = {}
end

--- キャンセルトークンオブジェクトを生成します。
--- __close は Lua 5.4 の <close> 構文向けのメタメソッドです。LuaJIT 2.1 ランタイムには
--- <close> 構文が無いため、__close は自動では呼ばれません（ランタイムはこのモジュールを使っていません）。
--- @return CT CTオブジェクト
local function new()
    local obj = {
        _cleanups = {},
        _cancelled = false,
    }
    setmetatable(obj, IMPL)
    return obj
end

return new