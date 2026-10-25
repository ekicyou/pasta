--- @module pasta.shiori.entry
--- テストフィクスチャ: 版 3 の入口と芯の型の後始末を試すゴースト（仕様 `shiori4-v3-portable`）
---
--- ID ごとの働き:
---   OnProbeCount     … Lua に届いた要求の数（この要求を含む）を Value で返す
---   OnProbeBadStatus … 読めない開始行の応答を返す
---   OnProbeSpin      … 1 秒ほど回り続けてから 204 を返す（ack の待ちの打ち切りを起こす）
---   OnProbeSlowUnload … 次の SHIORI.unload を 0.3 秒ほど回らせる（unload が終わりを待つかを確かめる）

SHIORI = SHIORI or {}

-- Lua に届いた要求の数（GET・NOTIFY とも数える）
local request_count = 0

-- 立てると SHIORI.unload が 0.3 秒ほど回る
local slow_unload = false

function SHIORI.load(hinst, load_dir)
    return true
end

function SHIORI.request(req)
    request_count = request_count + 1
    local id = req.id

    if id == "OnProbeCount" then
        return "SHIORI/3.0 200 OK\r\n" ..
            "Charset: UTF-8\r\n" ..
            "Value: " .. tostring(request_count) .. "\r\n" ..
            "\r\n"
    elseif id == "OnProbeBadStatus" then
        return "SHIORI/3.0 2x0 OK\r\n" ..
            "Charset: UTF-8\r\n" ..
            "Value: broken\r\n" ..
            "\r\n"
    elseif id == "OnProbeSlowUnload" then
        slow_unload = true
    elseif id == "OnProbeSpin" then
        local deadline = os.clock() + 1.0
        while os.clock() < deadline do
        end
    end

    return "SHIORI/3.0 204 No Content\r\n" ..
        "Charset: UTF-8\r\n" ..
        "\r\n"
end

function SHIORI.unload()
    if slow_unload then
        local deadline = os.clock() + 0.3
        while os.clock() < deadline do
        end
    end
end

return SHIORI
