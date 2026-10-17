--- 作者の .lua（日本語のファイル名）。止まった場所が scripts\天気.lua:N: と出る例
local M = {}

function M.予報()
    local forecast = nil
    return forecast.today
end

return M
