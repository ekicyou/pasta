--- @module pasta.shiori.act
--- SHIORI専用アクションオブジェクトモジュール
---
--- pasta.actを継承し、さくらスクリプトタグ生成機能を追加する。
--- サーフェス切り替え、待機、改行、クリア等のタグを組み立て、
--- build()で完成したさくらスクリプト文字列を取得する。

local ACT = require("pasta.act")

--- @class ShioriAct : Act SHIORI専用アクションオブジェクト
--- @field _buffer string[] さくらスクリプト蓄積バッファ
--- @field _current_scope number|nil 現在のスコープ番号
local SHIORI_ACT = {}

--- SHIORI_ACT実装メタテーブル
local SHIORI_ACT_IMPL = {}

-- 継承チェーン設定: SHIORI_ACT_IMPL → ACT.IMPL
setmetatable(SHIORI_ACT_IMPL, { __index = ACT.IMPL })

-- __index メソッドを定義（メソッド検索 + アクタープロキシ生成）
function SHIORI_ACT_IMPL.__index(self, key)
    -- 1. SHIORI_ACT_IMPLメソッドを検索
    local method = rawget(SHIORI_ACT_IMPL, key)
    if method then return method end

    -- 2. ACT.IMPLにフォールバック（アクタープロキシ生成を含む）
    return ACT.IMPL.__index(self, key)
end

--- 継承用に実装メタテーブルを公開
SHIORI_ACT.IMPL = SHIORI_ACT_IMPL

--- 新規ShioriActを作成
--- @param actors table<string, Actor> 登録アクター
--- @return ShioriAct アクションオブジェクト
function SHIORI_ACT.new(actors)
    local base = ACT.new(actors)
    base._buffer = {}
    base._current_scope = nil
    return setmetatable(base, SHIORI_ACT_IMPL)
end

--- さくらスクリプト用エスケープ処理
--- @param text string 入力テキスト
--- @return string エスケープ済みテキスト
local function escape_sakura(text)
    if not text then return "" end
    local escaped = text:gsub("\\", "\\\\")
    escaped = escaped:gsub("%%", "%%%%")
    return escaped
end

--- spotからスコープ番号を決定
--- @param spot any スポット値
--- @return number スコープ番号
local function spot_to_scope(spot)
    if spot == "sakura" or spot == 0 then
        return 0
    elseif spot == "kero" or spot == 1 then
        return 1
    elseif type(spot) == "number" then
        return spot
    elseif type(spot) == "string" then
        -- "char2" → 2, "char10" → 10
        local n = spot:match("^char(%d+)$")
        if n then
            return tonumber(n)
        end
    end
    return 0 -- デフォルトはsakura
end

--- スコープタグを生成
--- @param scope number スコープ番号
--- @return string スコープタグ
local function scope_to_tag(scope)
    if scope == 0 then
        return "\\0"
    elseif scope == 1 then
        return "\\1"
    else
        return string.format("\\p[%d]", scope)
    end
end

--- talkメソッド（オーバーライド）
--- スコープ切り替えとテキスト追加を行う
--- @param self ShioriAct アクションオブジェクト
--- @param actor Actor アクターオブジェクト
--- @param text string 発話テキスト
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.talk(self, actor, text)
    if not actor then
        error("actor is required")
    end

    local scope = spot_to_scope(actor.spot)

    -- スコープ切り替え判定
    if self._current_scope ~= scope then
        -- 既存トークがあれば改行を追加
        if self._current_scope ~= nil then
            table.insert(self._buffer, "\\n")
        end
        -- スコープタグを追加
        table.insert(self._buffer, scope_to_tag(scope))
        self._current_scope = scope
    end

    -- エスケープ済みテキストを追加
    table.insert(self._buffer, escape_sakura(text))

    -- テキスト後の改行
    table.insert(self._buffer, "\\n")

    -- 親クラスのtalk()を呼び出し（tokenバッファ用）
    ACT.IMPL.talk(self, actor, text)

    return self
end

--- サーフェス変更タグを追加
--- @param self ShioriAct アクションオブジェクト
--- @param id number|string サーフェスID
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.surface(self, id)
    table.insert(self._buffer, string.format("\\s[%s]", tostring(id)))
    return self
end

--- 待機タグを追加
--- @param self ShioriAct アクションオブジェクト
--- @param ms number 待機時間（ミリ秒）
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.wait(self, ms)
    ms = math.max(0, math.floor(ms or 0))
    table.insert(self._buffer, string.format("\\w[%d]", ms))
    return self
end

--- 改行タグを追加
--- @param self ShioriAct アクションオブジェクト
--- @param n number|nil 改行回数（デフォルト1）
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.newline(self, n)
    n = n or 1
    if n >= 1 then
        for _ = 1, n do
            table.insert(self._buffer, "\\n")
        end
    end
    return self
end

--- クリアタグを追加
--- @param self ShioriAct アクションオブジェクト
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.clear(self)
    table.insert(self._buffer, "\\c")
    return self
end

--- バッファを結合してさくらスクリプト文字列を返却
--- @param self ShioriAct アクションオブジェクト
--- @return string さくらスクリプト文字列
function SHIORI_ACT_IMPL.build(self)
    local script = table.concat(self._buffer)
    return script .. "\\e"
end

--- バッファとスコープをリセット
--- @param self ShioriAct アクションオブジェクト
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.reset(self)
    self._buffer = {}
    self._current_scope = nil
    return self
end

return SHIORI_ACT
