--- REG のハンドラ（シーンの外で止まる例）
local REG = require("pasta.shiori.event.register")

--- 途中の台詞を積んでから止まる
REG.OnHandlerFail = function(act)
    act:raw_script("ハンドラの途中")
    error("handler boom")
end

--- On で始まらない ID で止まる
REG.plain_fail = function(act)
    act:raw_script("ハンドラの途中")
    error("plain boom")
end

return {}
