--- @module pasta.shiori.act
--- SHIORI専用アクションオブジェクトモジュール
---
--- pasta.actを継承し、build()をオーバーライドしてさくらスクリプト文字列を生成する。
--- サーフェス切り替え、待機、改行、クリア等は親クラスから継承。

local ACT = require("pasta.act")
local BUILDER = require("pasta.shiori.sakura_builder")
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

    -- STORE.actor_spotsを読み取り、BUILDER.build()に渡す（persist-spot-position）
    local current_spots = STORE.actor_spots

    -- sakura_builderで変換（新プロパティ名spot_newlinesを使用）
    local script, updated_spots = BUILDER.build(token, {
        spot_newlines = self._spot_newlines
    }, current_spots)

    -- 更新されたスポット状態をSTOREに書き戻し
    if updated_spots then
        STORE.actor_spots = updated_spots
    end

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
    -- 転記対象: year, month, day, hour, min, sec, wday
    -- 転記対象外: unix, ns, yday, ordinal, num_days_from_sunday
    if date.year then self.var.year = date.year end
    if date.month then self.var.month = date.month end
    if date.day then self.var.day = date.day end
    if date.hour then self.var.hour = date.hour end
    if date.min then self.var.min = date.min end
    if date.sec then self.var.sec = date.sec end
    if date.wday then self.var.wday = date.wday end

    -- 日本語変数マッピング（文字列型）
    if date.year then self.var["年"] = string.format("%d年", date.year) end
    if date.month then self.var["月"] = string.format("%d月", date.month) end
    if date.day then self.var["日"] = string.format("%d日", date.day) end
    if date.hour then self.var["時"] = string.format("%d時", date.hour) end
    if date.min then self.var["分"] = string.format("%d分", date.min) end
    if date.sec then self.var["秒"] = string.format("%d秒", date.sec) end

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

return SHIORI_ACT
