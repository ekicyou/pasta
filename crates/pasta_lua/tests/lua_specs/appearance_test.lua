-- appearance_test.lua
-- pasta.shiori.appearance 外見状態の走査・観測テスト
-- actor-surface-restore Task 2.1 (Requirements: 1.2, 1.3, 1.4, 1.5, 1.7, 1.8)
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

local APPEARANCE = require("pasta.shiori.appearance")

local A = { name = "A" }

--- スポット 0 / 1 のサーフェスが既知の状態を作る
local function known_spots()
    local state = APPEARANCE.new()
    APPEARANCE.observe(state, { name = "X" }, 0, "\\s[7]")
    APPEARANCE.observe(state, { name = "Y" }, 1, "\\s[8]")
    return state
end

describe("APPEARANCE.new", function()
    test("4 つの空表を持つ新しい状態を返す", function()
        local s1 = APPEARANCE.new()
        local s2 = APPEARANCE.new()
        expect(s1 == s2):toBe(false)
        for _, key in ipairs({ "actors", "spots", "owners", "last_spots" }) do
            expect(type(s1[key])):toBe("table")
            expect(next(s1[key])):toBe(nil)
        end
    end)
end)

describe("APPEARANCE.observe - サーフェス変更", function()
    test("\\s[ID] をアクターとスポットへ文字列で記録する", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\s[5]こんにちは")
        expect(state.actors.A.surface):toBe("5")
        expect(state.spots[0].surface):toBe("5")
    end)

    test("\\sN を数字 1 文字の ID として記録する", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 2, "\\s3やあ")
        expect(state.actors.A.surface):toBe("3")
        expect(state.spots[2].surface):toBe("3")
    end)

    test("エイリアス・-1 を解釈せずそのまま記録する", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\s[smile]")
        expect(state.actors.A.surface):toBe("smile")
        APPEARANCE.observe(state, A, 0, "\\s[-1]")
        expect(state.actors.A.surface):toBe("-1")
        expect(state.spots[0].surface):toBe("-1")
    end)

    test("複数あるときは最後のサーフェス変更を採用する", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\s[0]あ\\_w[100]\\s[10]い\\s4う")
        expect(state.actors.A.surface):toBe("4")
        expect(state.spots[0].surface):toBe("4")
    end)

    test("\\\\ に続く s は検出しない", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\\\s[5]")
        expect(state.actors.A):toBe(nil)
        expect(state.spots[0]):toBe(nil)
    end)

    test("\\\\\\s[5] (エスケープ後の本物のタグ) は検出する", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\\\\\s[5]")
        expect(state.actors.A.surface):toBe("5")
    end)

    test("他タグ引数内の入れ子タグは検出しない", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![raise,OnX,\\s[0]]あ\\_a[\\s[3]]い")
        expect(state.actors.A):toBe(nil)
        expect(state.spots[0]):toBe(nil)
    end)

    test("サーフェス以外のタグ (\\n, \\_w, \\![set,...]) では記録しない", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "あ\\n\\_w[300]\\![set,balloontimeout,0]\\w9")
        expect(state.actors.A):toBe(nil)
        expect(state.spots[0]):toBe(nil)
    end)

    test("バックスラッシュを含まない文字列では状態を変えない", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "こんにちは s[5]")
        expect(next(state.actors)):toBe(nil)
        expect(next(state.spots)):toBe(nil)
    end)

    test("空 ID の \\s[] は記録しない", function()
        local state = known_spots()
        APPEARANCE.observe(state, A, 0, "\\s[]あ")
        expect(state.actors.A):toBe(nil)
        expect(state.spots[0].surface):toBe("7")
        APPEARANCE.observe(state, nil, nil, "\\s[]")
        expect(state.spots[0].surface):toBe("7")
    end)

    local malformed = { "あ\\", "\\s[", "\\s[5", "\\s[5あ\\", "\\p[", "\\![bind,a,b,1" }
    for _, text in ipairs(malformed) do
        test("不正な入力 " .. text .. " で例外を出さず状態を変えない", function()
            local state = known_spots()
            APPEARANCE.observe(state, A, 0, text)
            APPEARANCE.observe(state, nil, nil, text)
            expect(state.actors.A):toBe(nil)
            expect(state.spots[0].surface):toBe("7")
            expect(state.spots[1].surface):toBe("8")
        end)
    end

    test("アクター名が nil ならスポット側のみ更新する", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, {}, 0, "\\s[5]")
        expect(next(state.actors)):toBe(nil)
        expect(state.spots[0].surface):toBe("5")
    end)

    test("他スポット・他アクターの状態は変えない", function()
        local state = known_spots()
        APPEARANCE.observe(state, A, 0, "\\s[2]")
        expect(state.spots[1].surface):toBe("8")
        expect(state.actors.X.surface):toBe("7")
        expect(state.spots[0].surface):toBe("2")
    end)
end)

describe("APPEARANCE.observe - スコープ切替タグ", function()
    local scope_tags = { "\\0", "\\1", "\\h", "\\u", "\\p[2]", "\\p3" }
    for _, tag in ipairs(scope_tags) do
        test(tag .. " で全スポットを不明化し、以降の \\s を記録しない", function()
            local state = known_spots()
            APPEARANCE.observe(state, A, 0, "\\s[1]あ" .. tag .. "\\s[10]い")
            expect(next(state.spots)):toBe(nil)
            expect(state.actors.A.surface):toBe("1")
            expect(state.actors.X.surface):toBe("7")
        end)
    end

    test("直前発話アクター・前回発話スポットは消さない", function()
        local state = APPEARANCE.new()
        state.owners[0] = "A"
        state.last_spots.A = 0
        APPEARANCE.observe(state, A, 0, "\\1\\s[10]")
        expect(state.owners[0]):toBe("A")
        expect(state.last_spots.A):toBe(0)
    end)

    test("state テーブル自体の同一性を保つ", function()
        local state = known_spots()
        local actors = state.actors
        APPEARANCE.observe(state, A, 0, "\\1")
        expect(state.actors == actors):toBe(true)
        expect(type(state.spots)):toBe("table")
    end)

    test("\\_u や \\pX など切替でないタグでは不明化しない", function()
        local state = known_spots()
        APPEARANCE.observe(state, A, 0, "\\_u[0x3042]\\_q\\![open,inputbox]")
        expect(state.spots[0].surface):toBe("7")
        expect(state.spots[1].surface):toBe("8")
    end)
end)

describe("APPEARANCE.observe - アクター未指定の生スクリプト", function()
    local cases = {
        { "サーフェス変更", "\\s[3]" },
        { "\\sN", "\\s3" },
        { "bind", "\\![bind,頭,帽子,1]" },
        { "bind-noevent", "\\![bind-noevent,頭,帽子,0]" },
        { "スコープ切替", "\\1" },
        { "スポット指定", "\\p[1]" },
    }
    for _, c in ipairs(cases) do
        test(c[1] .. " を含むと全スポットを不明化しアクター状態は変えない", function()
            local state = known_spots()
            state.owners[0] = "X"
            state.last_spots.X = 0
            APPEARANCE.observe(state, nil, nil, "あ" .. c[2] .. "い")
            expect(next(state.spots)):toBe(nil)
            expect(state.actors.X.surface):toBe("7")
            expect(state.actors.Y.surface):toBe("8")
            expect(state.owners[0]):toBe("X")
            expect(state.last_spots.X):toBe(0)
        end)
    end

    test("該当タグを含まない生スクリプト (\\![set,...]) では不明化しない", function()
        local state = known_spots()
        APPEARANCE.observe(state, nil, nil, "\\![set,property,a,b]\\_w[100]\\n")
        expect(state.spots[0].surface):toBe("7")
        expect(state.spots[1].surface):toBe("8")
    end)

    test("\\\\s や入れ子の \\s では不明化しない", function()
        local state = known_spots()
        APPEARANCE.observe(state, nil, nil, "\\\\s[1]\\![raise,OnX,\\s[0]]")
        expect(state.spots[0].surface):toBe("7")
    end)
end)
