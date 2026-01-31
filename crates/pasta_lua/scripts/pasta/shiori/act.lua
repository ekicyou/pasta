--- @module pasta.shiori.act
--- SHIORI専用アクションオブジェクトモジュール
---
--- pasta.actを継承し、さくらスクリプトタグ生成機能を追加する。
--- サーフェス切り替え、待機、改行、クリア等のタグを組み立て、
--- build()で完成したさくらスクリプト文字列を取得する。

local ACT = require("pasta.act")
local CONFIG = require("pasta.config")

--- @class ShioriAct : Act SHIORI専用アクションオブジェクト
--- @field _buffer string[] さくらスクリプト蓄積バッファ
--- @field _current_spot number|nil 現在のスポット番号
--- @field _spot_switch_newlines number スポット切り替え時の改行数（デフォルト1.5）
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
    base._current_spot = nil
    -- pasta.tomlの[ghost]セクションからspot_switch_newlinesを読み込み（デフォルト1.5）
    base._spot_switch_newlines = CONFIG.get("ghost", "spot_switch_newlines", 1.5)
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

--- spotからスポットID番号を決定
--- @param spot any スポット値
--- @return number スポットID番号
local function spot_to_id(spot)
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

--- スポットタグを生成（SSP ukadoc準拠: 常に\p[ID]形式）
--- @param spot_id number スポットID番号
--- @return string スポットタグ
local function spot_to_tag(spot_id)
    return string.format("\\p[%d]", spot_id)
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

    local spot_id = spot_to_id(actor.spot)

    -- スポット切り替え判定
    if self._current_spot ~= spot_id then
        -- スポットタグを追加
        table.insert(self._buffer, spot_to_tag(spot_id))
        -- 既存トークがあれば段落区切り改行を追加（スポットタグの後）
        if self._current_spot ~= nil then
            local percent = math.floor(self._spot_switch_newlines * 100)
            table.insert(self._buffer, string.format("\\n[%d]", percent))
        end
        self._current_spot = spot_id
    end

    -- エスケープ済みテキストを追加
    table.insert(self._buffer, escape_sakura(text))

    -- テキスト後の改行（固定\n）
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

--- バッファを結合してさくらスクリプト文字列を返却（自動リセット付き）
--- @param self ShioriAct アクションオブジェクト
--- @return string さくらスクリプト文字列
function SHIORI_ACT_IMPL.build(self)
    local script = table.concat(self._buffer)
    self:reset() -- 1 yield = 1 build: 自動リセット
    return script .. "\\e"
end

--- さくらスクリプト文字列をyield（コルーチン用）
--- build()を呼び出してyieldし、再開後にリセット済みのselfを返す
--- @param self ShioriAct アクションオブジェクト
--- @return ShioriAct self リセット済みのself（メソッドチェーン用）
function SHIORI_ACT_IMPL.yield(self)
    local script = self:build()
    coroutine.yield(script)
    return self
end

--- バッファとスポット状態をリセット
--- @param self ShioriAct アクションオブジェクト
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.reset(self)
    self._buffer = {}
    self._current_spot = nil
    return self
end

return SHIORI_ACT
