--- @module pasta.co
--- コルーチンユーティリティモジュール
---
--- 標準 coroutine モジュールの安全な拡張を提供する。
--- エラーハンドリングを組み込んだラップ関数などを含む。

--- @class Co コルーチンユーティリティ
local CO = {}

--- coroutine.wrap の安全版（状態タグを第1戻り値として返す）
---
--- 標準の coroutine.wrap はエラーを伝搬するが、この関数は状態を
--- 戻り値として返す。
---   - エラー時: nil, error_message
---   - yield時: "yield", ...（yieldの引数）
---   - return時: "return", ...（returnの引数）
---   - dead時（終了後の再呼び出し）: nil, "dead"
---
--- 使用例:
--- ```lua
--- local CO = require("pasta.co")
--- local wrapped = CO.safe_wrap(function(x)
---     coroutine.yield("yielded", x)
---     return "done", x * 2
--- end)
---
--- local tag, a, b = wrapped(5)
--- -- tag = "yield", a = "yielded", b = 5
---
--- tag, a, b = wrapped()
--- -- tag = "return", a = "done", b = 10
---
--- tag, a = wrapped()
--- -- tag = nil, a = "dead"
--- ```
---
--- @param func function コルーチン関数
--- @return function ラップされた関数（tag, ...を返す）
function CO.safe_wrap(func)
    local co = coroutine.create(func)

    return function(...)
        -- dead状態の場合は即座に nil, "dead" を返す
        if coroutine.status(co) == "dead" then
            return nil, "dead"
        end

        local results = { coroutine.resume(co, ...) }
        local ok = results[1]

        if not ok then
            -- エラーの場合: nil, error_message
            return nil, results[2]
        else
            -- 成功の場合: 状態に応じてタグを設定
            local status = coroutine.status(co)
            table.remove(results, 1) -- ok フラグを削除

            if status == "suspended" then
                -- yield した場合: "yield", ...
                return "yield", table.unpack(results)
            else
                -- return した場合（dead）: "return", ...
                return "return", table.unpack(results)
            end
        end
    end
end

return CO
