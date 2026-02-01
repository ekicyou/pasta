--- @module pasta.co
--- コルーチンユーティリティモジュール
---
--- 標準 coroutine モジュールの安全な拡張を提供する。
--- エラーハンドリングを組み込んだラップ関数などを含む。

--- @class Co コルーチンユーティリティ
local CO = {}

--- coroutine.wrap の安全版（エラーを第1戻り値として返す）
---
--- 標準の coroutine.wrap はエラーを伝搬するが、この関数はエラーを
--- 戻り値として返す。成功時は err=nil、失敗時は err=エラーメッセージ。
---
--- 使用例:
--- ```lua
--- local CO = require("pasta.co")
--- local wrapped = CO.safe_wrap(function(x, y)
---     if x == 0 then
---         error("ゼロ除算エラー")
---     end
---     return y / x, "成功"
--- end)
---
--- local err, result, msg = wrapped(2, 10)
--- if err then
---     print("エラー:", err)
--- else
---     print("結果:", result, msg)  -- 結果: 5  成功
--- end
--- ```
---
--- @param func function コルーチン関数
--- @return function ラップされた関数（err, ...を返す）
function CO.safe_wrap(func)
    local co = coroutine.create(func)

    return function(...)
        local results = { coroutine.resume(co, ...) }
        local ok = results[1]

        if not ok then
            -- エラーの場合: err = エラーメッセージ
            return results[2]
        else
            -- 成功の場合: err = nil, 後続は結果
            table.remove(results, 1) -- ok フラグを削除
            return nil, table.unpack(results)
        end
    end
end

return CO
