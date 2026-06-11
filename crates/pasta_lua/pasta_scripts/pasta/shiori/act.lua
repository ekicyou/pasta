--- @module pasta.shiori.act
--- SHIORI専用アクションオブジェクトモジュール
---
--- pasta.actを継承し、build()をオーバーライドしてさくらスクリプト文字列を生成する。
--- サーフェス切り替え、待機、改行、クリア等は親クラスから継承。

local ACT = require("pasta.act")
local BUILDER = require("pasta.shiori.sakura_builder")
local CALLBACK = require("pasta.shiori.event.callback")
local CONFIG = require("pasta.config")
local STORE = require("pasta.store")

--- @class ShioriAct : Act SHIORI専用アクションオブジェクト
--- @field _spot_newlines number スポット切り替え時の改行数（デフォルト1.5）
--- @field req ShioriRequest|nil SHIORIリクエストオブジェクト（読み取り専用として扱うこと）
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
--- @param req ShioriRequest|nil SHIORIリクエストオブジェクト（任意）
--- @return ShioriAct アクションオブジェクト
---
--- req は SHIORI リクエストの情報を保持するテーブルです。
--- イベントハンドラ内で `act.req` を通じてリクエスト情報にアクセスできます。
--- **注意**: act.req は読み取り専用として扱ってください。変更は未定義動作となります。
function SHIORI_ACT.new(actors, req)
    local base = ACT.new(actors)
    -- pasta.tomlの[ghost]セクションからspot_newlinesを読み込み（デフォルト1.5）
    base._spot_newlines = CONFIG.get("ghost", "spot_newlines", 1.5)
    -- SHIORIリクエストオブジェクトを設定（任意）
    base.req = req
    return setmetatable(base, SHIORI_ACT_IMPL)
end

--- build()オーバーライド: さくらスクリプト生成
--- 親のbuild()でトークン取得＆リセット後、sakura_builderで変換
--- @param self ShioriAct アクションオブジェクト
--- @return string|nil さくらスクリプト文字列、またはnil（トークン0件時）
function SHIORI_ACT_IMPL.build(self)
    -- 親のbuild()でトークン取得＆リセット
    local token = ACT.IMPL.build(self)

    -- 早期リターン: ACT.IMPL.build()がnilを返した場合はnilを返す (act-build-early-return)
    if token == nil then
        return nil
    end

    -- STORE.actor_spotsを直接BUILDER.build()に渡す（直接変更方式）
    local script = BUILDER.build(token, {
        spot_newlines = self._spot_newlines
    }, STORE.actor_spots)

    return script
end

-- ============================================================================
-- 日時転記機能 (onhour-date-var-transfer)
-- ============================================================================

--- 曜日変換テーブル（wday 0-6 → 日本語/英語曜日名）
local WEEKDAYS_JA = { "日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日" }
local WEEKDAYS_EN = { "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday" }

--- 12時間制変換
--- @param hour number 24時間制の時 (0-23)
--- @return string 午前/午後付き12時間制文字列
local function to_12hour_format(hour)
    if hour == 0 then
        return "深夜0時"
    elseif hour >= 1 and hour <= 11 then
        return string.format("午前%d時", hour)
    elseif hour == 12 then
        return "正午"
    else
        return string.format("午後%d時", hour - 12)
    end
end

--- 英語フィールド（数値型）の転記対象
--- 転記対象外: unix, ns, yday, ordinal, num_days_from_sunday
local DATE_NUM_FIELDS = { "year", "month", "day", "hour", "min", "sec", "wday" }

--- 日本語変数マッピング（文字列型）: { dateフィールド名, 単位サフィックス＝変数名 }
local DATE_JA_FIELDS = {
    { "year",  "年" },
    { "month", "月" },
    { "day",   "日" },
    { "hour",  "時" },
    { "min",   "分" },
    { "sec",   "秒" },
}

--- 日時フィールドを req.date から var へ転記
--- @param self ShioriAct アクションオブジェクト
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.transfer_date_to_var(self)
    -- req または req.date が存在しない場合は何もせず正常終了
    if not self.req or not self.req.date then
        return self
    end

    local date = self.req.date

    -- 英語フィールド（数値型）を転記
    for _, f in ipairs(DATE_NUM_FIELDS) do
        if date[f] then self.var[f] = date[f] end
    end

    -- 日本語変数マッピング（文字列型: 値＋単位サフィックス。例: 年=「2026年」）
    for _, m in ipairs(DATE_JA_FIELDS) do
        local f, suffix = m[1], m[2]
        if date[f] then self.var[suffix] = string.format("%d%s", date[f], suffix) end
    end

    -- 曜日変換
    if date.wday then
        self.var["曜日"] = WEEKDAYS_JA[date.wday + 1]
        self.var.week = WEEKDAYS_EN[date.wday + 1]
    end

    -- 12時間制変換
    if date.hour then
        self.var["時１２"] = to_12hour_format(date.hour)
    end

    return self
end

-- ============================================================================
-- リクエスト転記機能 (req-var-expansion)
-- ============================================================================

--- 全角数字テーブル（0-9 → ０-９）
local FULLWIDTH_DIGITS = { "０", "１", "２", "３", "４", "５", "６", "７", "８", "９" }

--- リクエストフィールドを req から var へ転記
--- reference[0]〜[9] を全角キー（ｒ０〜ｒ９）と半角キー（r0〜r9）に転記し、
--- req.id / req.base_id を req_id / req_base_id に転記する。
--- @param self ShioriAct アクションオブジェクト
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.transfer_req_to_var(self)
    -- req が存在しない場合は何もせず正常終了
    if not self.req then
        return self
    end

    -- Reference パラメーター転記（0-indexed, 0〜9）
    local ref = self.req.reference
    if ref then
        for i = 0, 9 do
            local val = ref[i]
            if val ~= nil then
                -- 全角キー: ｒ＋全角数字（例: ｒ０）
                self.var["ｒ" .. FULLWIDTH_DIGITS[i + 1]] = val
                -- 半角キー: r＋半角数字（例: r0）
                self.var["r" .. i] = val
            end
        end
    end

    -- イベントメタデータ転記
    if self.req.id then
        self.var.req_id = self.req.id
    end
    if self.req.base_id then
        self.var.req_base_id = self.req.base_id
    end

    return self
end

-- ============================================================================
-- SSPプロパティ書き込み (property-write-helpers feature)
-- ============================================================================

--- SSPタグ引数をエスケープする（SHIORI専用）
--- Step 1: \, %, ] をバックスラッシュでエスケープ
--- Step 2: , または " を含む場合は "" で全体をクォートし、内部の " を "" に二重化
--- @param s string エスケープ対象の文字列
--- @return string エスケープ済み文字列
local function escape_tag_arg(s)
    -- Step 1: 文字エスケープ（この順番で適用）
    s = s:gsub("\\", "\\\\")
    s = s:gsub("%%", "\\%%")
    s = s:gsub("]", "\\]")
    -- Step 2: クォーティング（, または " を含む場合）
    if s:find('[,"]') then
        s = s:gsub('"', '""')
        s = '"' .. s .. '"'
    end
    return s
end

--- SSPプロパティ書き込みタグ（\![set,property,name,value]）を蓄積
--- @param self ShioriAct アクションオブジェクト
--- @param name string プロパティ名（nil・空文字列は不可）
--- @param value any プロパティ値（nil の場合は "" として扱う）
--- @return ShioriAct self メソッドチェーン用
function SHIORI_ACT_IMPL.set_property(self, name, value)
    if name == nil or name == "" then
        error("set_property: name must not be nil or empty")
    end
    if value == nil then
        value = ""
    else
        value = tostring(value)
    end
    local escaped_name = escape_tag_arg(name)
    local escaped_value = escape_tag_arg(value)
    local tag = "\\![set,property," .. escaped_name .. "," .. escaped_value .. "]"
    table.insert(self.token, { type = "raw_script", text = tag })
    return self
end

--- get_property の第1引数を names 配列へ正規化し、各名前を検証する
--- @param name_or_names string|string[] プロパティ名または名前の配列
--- @return string[] names 正規化済み配列
--- @return number n 名前数
local function normalize_property_names(name_or_names)
    local names
    if type(name_or_names) == "string" then
        names = { name_or_names }
    elseif type(name_or_names) == "table" then
        names = name_or_names
    else
        error("get_property: first argument must be a property name (string) or array of names (table)")
    end
    local n = #names
    if n == 0 then error("get_property: at least one property name required") end
    for i = 1, n do
        local name = names[i]
        if name == nil or name == "" then
            error("get_property: name must not be nil or empty")
        end
    end
    return names, n
end

--- SSPプロパティ読み取りタグ（\![get,property,{id},{names...}]）を蓄積し、yield で値を受け取る
--- @param self ShioriAct アクションオブジェクト
--- @param name_or_names string|string[] プロパティ名または名前の配列
--- @param timeout number|nil タイムアウト秒数（デフォルト 5）
--- @param timeout_message string|nil タイムアウト時のエラーメッセージ
--- @return any ... プロパティ値（多値返却）
function SHIORI_ACT_IMPL.get_property(self, name_or_names, timeout, timeout_message)
    -- 引数正規化＋バリデーション
    local names, n = normalize_property_names(name_or_names)
    local co, is_main = coroutine.running()
    if is_main or co == nil then
        error("get_property: must be called inside a scene coroutine")
    end

    -- デフォルト適用
    timeout = timeout or 5
    if timeout_message == nil then
        timeout_message = "callback timeout: get_property"
    end

    -- イベント ID 生成 + ステージング
    local event_id = CALLBACK.next_event_id()
    CALLBACK.stage_pending(event_id, os.time() + timeout, timeout_message)

    -- トークンバッファ退避
    local saved_tokens = self.token
    self.token = {}

    -- get タグのみを新バッファに登録
    local parts = { "\\![get,property," .. event_id }
    for i = 1, n do
        parts[#parts + 1] = escape_tag_arg(names[i])
    end
    local tag = table.concat(parts, ",") .. "]"
    table.insert(self.token, { type = "raw_script", text = tag })

    -- yield（get タグのみのスクリプトを送信）
    local refs, reason = coroutine.yield(self:build())

    -- トークンバッファ復元（成功・失敗いずれの経路でも）
    self.token = saved_tokens

    if reason then
        error(reason)
    end
    -- refs == nil（タイムアウト静黙経路）は空テーブル扱い → 全要素 nil の n 値返却（等価簡素化）
    refs = refs or {}
    local out = {}
    for i = 1, n do
        local v = refs[i]
        out[i] = (v == nil or v == "") and nil or v
    end
    return table.unpack(out, 1, n)
end

return SHIORI_ACT
