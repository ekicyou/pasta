--- dic/ に置いた作者の .lua（debug-mcp-runtime task 7.3）。止まった場所が dic\道具.lua:N: と出る例
local GLOBAL = require("pasta.global")

function GLOBAL.道具(act)
    local tool = nil
    return tool.name
end
