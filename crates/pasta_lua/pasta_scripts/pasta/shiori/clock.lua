--- @module pasta.shiori.clock
--- 仮想時刻のずれを持つ時計（VirtualClock）
---
--- SSP の仮想時刻と OS の時計のずれ（整数秒）を持ち、時報の判定と日時変数のための
--- 「今」の時刻と日付の表を返す。今の決め方は X-Pasta-Time → SSP の仮想時刻 → OS の時計の順。
---   - observe: 毎分・毎時の合図が見込みの時機から外れたら読み取りを予約し、
---     OnPastaClock の返事（Reference0〜5 = 年・月・日・時・分・秒）からずれを決める
---   - take_probe: 予約があれば読み取りのスクリプトを 1 回だけ返す
---   - now / date: ずれを足した時刻と日付の表（ずれ 0 なら req.date をそのまま返す）
---   - 時刻の飛び: OnSecondChange ごとに前回の「今」との差を見て、設定し直しなどの飛びを
---     通し番号（epoch）と飛びの合計に数える。steady は飛びを差し引いた「なめらかな時計」
---
--- 循環参照回避: act.lua と event/init.lua の両方から読まれるため、
--- pasta.shiori.event.*・pasta.shiori.act・pasta.store を require しない。
--- 状態は保存しない（load から unload までの寿命）。

local log = require("@pasta_log")

--- 毎分の合図の許容幅（秒）。now % 60 がこの値以下か 60 - この値以上なら見込みどおり（調整つまみ）
local MINUTE_TOLERANCE = 2
--- 毎時の合図の遅れの許容（秒）。トークできるまで待つ遅れを許す
local HOUR_LATE_TOLERANCE = 60
--- 毎時の合図の早まりの許容（秒）
local HOUR_EARLY_TOLERANCE = 5
--- この秒数以下のずれは 0（仮想時刻なし）とみなす
local OFFSET_DEADBAND = 2
--- 秒の合図の間で「今」がこの秒数より戻ったら時刻の飛びとする（調整つまみ）
local JUMP_BACK = 2
--- 秒の合図の間で「今」がこの秒数より進んだら時刻の飛びとする（調整つまみ）
local JUMP_FORWARD = 90

--- 仮想の日時を読む SSP のスクリプト
local PROBE_SCRIPT = "\\![get,property,OnPastaClock,system.year,system.month,system.day,"
    .. "system.hour,system.minute,system.second]\\e"

---@class PastaDate                  -- act.req.date と同じ形（lua_request.rs の lua_date_from）
---@field unix integer
---@field year integer
---@field month integer               -- 1〜12
---@field day integer
---@field hour integer                -- 0〜23
---@field min integer
---@field sec integer
---@field ns integer
---@field yday integer                -- 1 月 1 日が 1
---@field ordinal integer             -- yday と同じ
---@field wday integer                -- 日曜が 0
---@field num_days_from_sunday integer

---@class ClockRequest                -- act.req のうち clock が読む所
---@field id string
---@field date PastaDate|nil
---@field dic table<string, string>|nil
---@field reference table<integer, string>|nil
---@field security_level string|nil

---@class VirtualClock
local M = {}

-- Module-local state
local _offset = 0
local _probe_armed = false
local _prev_now = nil      -- 前回の OnSecondChange の「今」
local _epoch = 0           -- 時刻の飛びの通し番号
local _jump_total = 0      -- 飛びの合計（秒。ふだんの 1 秒分を除く）
local _resumed = false     -- OnSysResume の後で、次の 1 回の飛びを数えない
local _suspended = false   -- OnSysSuspend の後で、次の 1 回の飛びを数えない（飛ばない秒では消えない）

--- X-Pasta-Time（テストの時刻の上書き）があるリクエストか
---@param req ClockRequest
---@return boolean
local function has_pasta_time(req)
    return req.dic ~= nil and req.dic["X-Pasta-Time"] ~= nil
end

--- 返事の 6 つの値から仮想の Unix 時刻を作る。読めなければ nil
---@param ref table<integer, string>|nil
---@return integer|nil
local function virtual_unix(ref)
    if ref == nil then return nil end
    local v = {}
    for i = 0, 5 do
        local n = tonumber(ref[i])
        if n == nil or n ~= math.floor(n) then return nil end
        v[i] = n
    end
    local ok, t = pcall(os.time, {
        year = v[0], month = v[1], day = v[2], hour = v[3], min = v[4], sec = v[5],
    })
    if not ok or type(t) ~= "number" then return nil end
    return t
end

--- ずれを入れ替える（変わったときだけデバッグログを出す）
---@param offset integer
local function set_offset(offset)
    if math.abs(offset) <= OFFSET_DEADBAND then offset = 0 end
    if offset ~= _offset then
        log.debug(string.format("[clock] 仮想時刻のずれ: %d → %d 秒", _offset, offset))
    end
    _offset = offset
end

--- OnPastaClock の返事を受け取る
---@param req ClockRequest
local function receive_reply(req)
    if has_pasta_time(req) then return end
    local t = virtual_unix(req.reference)
    if t == nil or req.date == nil or req.date.unix == nil then
        local ref = req.reference or {}
        log.warn(string.format(
            "[clock] 仮想時刻の値が読めないため OS の時計を使う: %s,%s,%s,%s,%s,%s",
            tostring(ref[0]), tostring(ref[1]), tostring(ref[2]),
            tostring(ref[3]), tostring(ref[4]), tostring(ref[5])))
        set_offset(0)
        return
    end
    set_offset(t - req.date.unix)
end

--- 秒の合図ごとに前回の「今」との差を見て、時刻の飛びを数える（X-Pasta-Time 付きは呼ばない）
---@param now integer
local function watch_jump(now)
    if _prev_now ~= nil then
        local d = now - _prev_now
        if d < -JUMP_BACK or d > JUMP_FORWARD then
            if _resumed or _suspended then
                -- スリープ明けは飛びにしない。SSP は OnSysSuspend を復帰してから送り、
                -- OnSysResume は復帰後の最初の秒より数秒遅れて送る（2026-10-18 実機）
                _suspended = false
            else
                _epoch = _epoch + 1
                _jump_total = _jump_total + d - 1
                log.debug(string.format("[clock] 時刻の飛び: %d 秒", d))
            end
        elseif _resumed then
            -- 復帰した後の飛ばない秒: スリープしなかった（取りやめた）場合も印を残さない
            _suspended = false
        end
    end
    _resumed = false
    _prev_now = now
end

--- 合図と返事を受ける。EVENT.fire が REG より前に、すべてのリクエストで呼ぶ。
--- 返事（OnPastaClock）を受け取ったときだけ true を返す（呼び元は 204 を返し、REG へ進まない）。
---@param req ClockRequest
---@return boolean consumed
function M.observe(req)
    if req.id == "OnPastaClock" then
        if req.security_level == "external" then return false end
        receive_reply(req)
        return true
    end
    if req.id == "OnSysResume" then _resumed = true end
    if req.id == "OnSysSuspend" then _suspended = true end
    if has_pasta_time(req) then return false end
    local now = M.now(req)
    if now == nil then return false end
    if req.id == "OnSecondChange" then
        watch_jump(now)
    elseif req.id == "OnMinuteChange" then
        local s = now % 60
        if s > MINUTE_TOLERANCE and s < 60 - MINUTE_TOLERANCE then _probe_armed = true end
    elseif req.id == "OnHourTimeSignal" then
        local s = now % 3600
        if s > HOUR_LATE_TOLERANCE and s < 3600 - HOUR_EARLY_TOLERANCE then _probe_armed = true end
    end
    return false
end

--- 予約があれば消して、読み取りのスクリプトを返す。予約が無ければ nil。
--- 呼び元（OnSecondChange の既定ハンドラ）が、ブロック対象の Status でないことを確かめてから呼ぶ。
---@param req ClockRequest
---@return string|nil script
function M.take_probe(req)
    if not _probe_armed or has_pasta_time(req) then return nil end
    _probe_armed = false
    return PROBE_SCRIPT
end

--- 時報の判定に使う「今」の Unix 時刻。req.date（か、その unix）が無ければ nil。
---@param req ClockRequest
---@return integer|nil
function M.now(req)
    if req.date == nil or req.date.unix == nil then return nil end
    if has_pasta_time(req) then return req.date.unix end
    return req.date.unix + _offset
end

--- 日時変数に使う「今」の日付の表。req.date が無ければ nil。unix の無い表はずれを足せないので、そのまま返す。
--- 仮想の日付は新しい表で返す（req.date を書き換えない）。
---@param req ClockRequest
---@return PastaDate|nil
function M.date(req)
    if req.date == nil then return nil end
    if _offset == 0 or has_pasta_time(req) or req.date.unix == nil then return req.date end
    local unix = req.date.unix + _offset
    local t = os.date("*t", unix)
    local wday = t.wday - 1
    return {
        unix = unix,
        year = t.year,
        month = t.month,
        day = t.day,
        hour = t.hour,
        min = t.min,
        sec = t.sec,
        ns = req.date.ns,
        yday = t.yday,
        ordinal = t.yday,
        wday = wday,
        num_days_from_sunday = wday,
    }
end

--- 飛びを差し引いた「なめらかな時計」。間隔・待ち時間・撫でを数える。req.date（か、その unix）が無ければ nil。
--- X-Pasta-Time があれば req.date.unix をそのまま返す。
---@param req ClockRequest
---@return integer|nil
function M.steady(req)
    local now = M.now(req)
    if now == nil or has_pasta_time(req) then return now end
    return now - _jump_total
end

--- 飛びの通し番号。ディスパッチャが「前回見た値から変わったか」で決め直す。
---@return integer
function M.epoch()
    return _epoch
end

--- 今のずれ（整数秒。0 は仮想時刻なし。SSP が OS の時計を書き換えるときは常に 0）。決め直しは epoch() で見る。
---@return integer
function M.offset()
    return _offset
end

--- テスト用: 状態を初期値に戻す
function M._reset()
    _offset = 0
    _probe_armed = false
    _prev_now = nil
    _epoch = 0
    _jump_total = 0
    _resumed = false
    _suspended = false
end

--- テスト用: 状態を読む
---@return { offset: integer, probe_armed: boolean, prev_now: integer|nil,
---   epoch: integer, jump_total: integer, resumed: boolean }
function M._get_state()
    return {
        offset = _offset,
        probe_armed = _probe_armed,
        prev_now = _prev_now,
        epoch = _epoch,
        jump_total = _jump_total,
        resumed = _resumed,
        suspended = _suspended,
    }
end

return M
