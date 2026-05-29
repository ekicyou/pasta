-- act_init_scene_global_record_test.lua
-- init_scene でグローバルシーン名を STORE.last_global_scene に記録するテスト
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("ACT_IMPL.init_scene - グローバルシーン名記録", function()
    local ACT, STORE

    local function setup()
        -- モジュールキャッシュをクリアして新鮮な状態にする
        package.loaded["pasta.act"] = nil
        package.loaded["pasta.store"] = nil
        STORE = require("pasta.store")
        ACT = require("pasta.act")
        STORE.last_global_scene = nil
    end

    test("グローバルシーン実行時に last_global_scene にシーン名が記録される", function()
        setup()
        local act = ACT.new({})
        local scene = { __global_name__ = "TestGlobalScene" }

        act:init_scene(scene)

        expect(STORE.last_global_scene):toBe("TestGlobalScene")
    end)

    test("ローカルシーン実行時に last_global_scene は変更されない", function()
        setup()
        STORE.last_global_scene = "PreviousScene"
        local act = ACT.new({})
        local scene = {} -- __global_name__ なし（ローカルシーン）

        act:init_scene(scene)

        expect(STORE.last_global_scene):toBe("PreviousScene")
    end)

    test("__global_name__ が nil のシーンでは last_global_scene は保持される", function()
        setup()
        STORE.last_global_scene = "KeptScene"
        local act = ACT.new({})
        local scene = { __global_name__ = nil }

        act:init_scene(scene)

        expect(STORE.last_global_scene):toBe("KeptScene")
    end)

    test("連続してグローバルシーンを実行すると last_global_scene が更新される", function()
        setup()
        local act = ACT.new({})

        act:init_scene({ __global_name__ = "FirstScene" })
        expect(STORE.last_global_scene):toBe("FirstScene")

        act:init_scene({ __global_name__ = "SecondScene" })
        expect(STORE.last_global_scene):toBe("SecondScene")
    end)

    test("グローバル→ローカルの順で実行すると last_global_scene はグローバルシーン名のまま", function()
        setup()
        local act = ACT.new({})

        act:init_scene({ __global_name__ = "GlobalOne" })
        expect(STORE.last_global_scene):toBe("GlobalOne")

        act:init_scene({}) -- ローカルシーン
        expect(STORE.last_global_scene):toBe("GlobalOne")
    end)
end)
