--- @class CTX 環境オブジェクト
--- @field spot_actors table 登場アクター
--- @field var table セッション変数（セッションが終わると消える）
--- @field save table 永続変数（セッションが終わっても残る）
--- @field sakura_script string 現在構築中のスクリプト
local IMPL = {}

local MOD = {}

--- 環境オブジェクトの新規作成
--- @return CTX 環境オブジェクト
function MOD.new()
    local ctx = {}
    ctx.spot_actors = {}
    ctx.var = {}
    ctx.save = {}
    ctx.sakura_script = ""
    setmetatable(ctx, IMPL)
    return ctx
end

return MOD
