--[[
SHIORI RESPONSE
    SHIORI/3.0 200 OK[CRLF]
    Charset: Shift_JIS[CRLF]
    Sender: サンプル[CRLF]
    SecurityLevel: local[CRLF]
    Value: \0\s[通常]おはこんばんちは。[CRLF]
    [CRLF]
]]

local insert_wait  = require "shiori.insert_wait" 

local CRLF = "\r\n"
local SPLIT = ": "
local env ={
    char_set=       "UTF-8",
    sender=         "emo",
    security_level= "local",
    CRLF=           CRLF,
    SPLIT=          SPLIT,
    time_ok=        0,
}

local function join(...)
    local rc = ""
    local args = {...}
    for k, v in pairs(args) do
        rc = rc .. v .. CRLF
    end
    return rc .. CRLF
end

--Charset
local function set_char_set(value)
    env.char_set = value
end

--Sender
local function set_sender(value)
    env.sender = value
end

--SecurityLevel
local function set_security_level(value)
    env.security_level = value
end

--SHIORI RESPONSE BUILD
local function build(code, dic)
    local rc = "SHIORI/3.0 " .. code .. CRLF
    rc = rc .. "Charset"        .. SPLIT .. env.char_set        .. CRLF
    --rc = rc .. "Sender"         .. SPLIT .. env.sender          .. CRLF
    --rc = rc .. "SecurityLevel"  .. SPLIT .. env.security_level  .. CRLF

    if type(dic) == 'table' then
        for k, v in pairs(dic) do
            rc = rc .. k .. SPLIT .. v .. CRLF
        end
    end

    return rc .. CRLF
end

-- 200 OK           正常に終了し、会話がある
--                  env.time_ok に最終送信時刻を保存
local function ok(value, dic)
    env.time_ok = os.time() 
    if not dic then dic={}
    end
    dic["Value"] = value
    return build("200 OK", dic)
end

-- 204 No Content   正常に終了したが、返すべきデータがない
local function no_content(dic)
    return build("204 No Content", dic)
end

-- 311 Not Enough   TEACH リクエストを受けたが、情報が足りない
local function not_enough(dic)
    return build("311 Not Enough", dic)
end

-- 312 Advice       TEACH リクエスト内の最も新しいヘッダが解釈不能
local function advice(dic)
    return build("312 Advice", dic)
end

-- 400 Bad Request  リクエスト不備
local function bad_request(dic)
    return build("400 Bad Request", dic)
end

-- 500 Internal Server Error    サーバ内でエラーが発生した
local function err(resion ,dic)
    if not dic then dic={}
    end
    dic["X-Error-Resion"] = resion
    return build("500 Internal Server Error", dic)
end

-- 204 No Content ワーニングあり
local function warn(resion ,dic)
    if not dic then dic={}
    end
    dic["X-Warn-Resion"] = resion
    return no_content(dic)
end


-- talk             ウェイト処理を追加し、200 OK終了する。
--                  env.time_ok に最終送信時刻を保存
local function talk(script, dic)
    local value = insert_wait.insert_wait(script)
    return ok(value, dic)
end


local pub = {
    join                = join,
    env                 = env,
    set_char_set        = set_char_set,
    set_sender          = set_sender,
    set_security_level  = set_security_level,
    build               = build,
    ok                  = ok,
    no_content          = no_content,
    not_enough          = not_enough,
    advice              = advice,
    bad_request         = bad_request,
    err                 = err,
    warn                = warn,
    talk                = talk,
}

return pub