-- appearance_test.lua
-- pasta.shiori.appearance 外見状態の走査・観測テスト
-- actor-surface-restore Task 2.1 (Requirements: 1.2, 1.3, 1.4, 1.5, 1.7, 1.8)
-- actor-surface-restore Task 2.2 (Requirements: 2.1, 2.2, 2.3, 2.4, 2.8, 2.9, 2.10, 2.11, 4.4, 5.5)
-- actor-surface-restore Task 4.1 (Requirements: 3.1, 3.4, 3.11, 3.12, 3.13)
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

    test("1.7: 別の observe 呼び出しに分かれた切替後のタグも記録せず、restore で記録を再開する", function()
        local state = APPEARANCE.new()
        -- DSL ではタグ 1 つずつが別の sakura_script トークンになる
        APPEARANCE.observe(state, A, 0, "\\s[1]")
        APPEARANCE.observe(state, A, 0, "\\1")
        APPEARANCE.observe(state, A, 0, "\\s[10]")
        expect(state.actors.A.surface):toBe("1")
        expect(next(state.spots)):toBe(nil)
        -- 次の \p[spot]（restore）で解除され、以後は再び記録される
        APPEARANCE.restore(state, A, 0, {})
        expect(state.detached):toBe(nil)
        APPEARANCE.observe(state, A, 0, "\\s[5]")
        expect(state.actors.A.surface):toBe("5")
        expect(state.spots[0].surface):toBe("5")
    end)

    test("1.7: 同一アクター継続の restore でも切替状態を解除する", function()
        local state = APPEARANCE.new()
        state.owners[0] = "A"
        state.last_spots.A = 0
        APPEARANCE.observe(state, A, 0, "\\1")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("")
        expect(state.detached):toBe(nil)
        APPEARANCE.observe(state, A, 0, "\\s[5]")
        expect(state.actors.A.surface):toBe("5")
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

describe("APPEARANCE.observe - 着せ替え指定の記録", function()
    test("明示 bind をアクターとスポットへ記録し、記録順を保つ (3.1)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,リボン,1]あ\\![bind,腕,時計,0]")
        expect(state.actors.A.binds["帽子"]["リボン"]):toBe(1)
        expect(state.actors.A.binds["腕"]["時計"]):toBe(0)
        expect(state.spots[0].binds["帽子"]["リボン"]):toBe(1)
        expect(state.spots[0].binds["腕"]["時計"]):toBe(0)
        expect(table.concat(state.actors.A.order["帽子"], ",")):toBe("リボン")
        expect(table.concat(state.actors.A.order["腕"], ",")):toBe("時計")
    end)

    test("作者が書いた bind-noevent も記録する (3.1)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind-noevent,帽子,麦わら,0]")
        expect(state.actors.A.binds["帽子"]["麦わら"]):toBe(0)
        expect(state.spots[0].binds["帽子"]["麦わら"]):toBe(0)
    end)

    test("再記録したパーツは記録順の末尾へ移動する (3.1)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら,1]")
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,リボン,1]")
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら,0]")
        expect(table.concat(state.actors.A.order["帽子"], ",")):toBe("リボン,麦わら")
        expect(state.actors.A.binds["帽子"]["麦わら"]):toBe(0)
    end)

    test("数値1ではスポットの同カテゴリ他パーツを不明にし、アクター側は保持する (3.13)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら,1]\\![bind,帽子,花,0]\\![bind,腕,時計,1]")
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,リボン,1]")
        local spot_hat = state.spots[0].binds["帽子"]
        expect(spot_hat["リボン"]):toBe(1)
        expect(spot_hat["麦わら"]):toBe(nil)
        expect(spot_hat["花"]):toBe(nil)
        expect(state.spots[0].binds["腕"]["時計"]):toBe(1)
        expect(state.actors.A.binds["帽子"]["麦わら"]):toBe(1)
        expect(state.actors.A.binds["帽子"]["花"]):toBe(0)
        expect(table.concat(state.actors.A.order["帽子"], ",")):toBe("麦わら,花,リボン")
    end)

    test("数値0ではスポットの同カテゴリ他パーツを変えない", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら,1]\\![bind,帽子,リボン,0]")
        expect(state.spots[0].binds["帽子"]["麦わら"]):toBe(1)
        expect(state.spots[0].binds["帽子"]["リボン"]):toBe(0)
    end)

    test("カテゴリ全脱衣 ,,0 で当該カテゴリを全パーツ脱衣に置き換える (3.11)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら,1]\\![bind,腕,時計,1]")
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,,0]")
        for _, t in ipairs({ state.actors.A.binds["帽子"], state.spots[0].binds["帽子"] }) do
            expect(t[""]):toBe(0)
            expect(t["麦わら"]):toBe(nil)
        end
        expect(#state.actors.A.order["帽子"]):toBe(0)
        expect(state.actors.A.binds["腕"]["時計"]):toBe(1)
        expect(state.spots[0].binds["腕"]["時計"]):toBe(1)
    end)

    test("全脱衣の後の明示パーツはその上に記録し、数値1でも全脱衣は残す (3.11, 3.13)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,,0]\\![bind,帽子,花,0]\\![bind,帽子,リボン,1]")
        local spot_hat = state.spots[0].binds["帽子"]
        expect(spot_hat[""]):toBe(0)
        expect(spot_hat["リボン"]):toBe(1)
        expect(spot_hat["花"]):toBe(nil)
        local actor_hat = state.actors.A.binds["帽子"]
        expect(actor_hat[""]):toBe(0)
        expect(actor_hat["花"]):toBe(0)
        expect(actor_hat["リボン"]):toBe(1)
        expect(table.concat(state.actors.A.order["帽子"], ",")):toBe("花,リボン")
    end)

    --- スポット0の帽子が「全脱衣 → X=1 → Y=1」の状態
    local function stripped_then_two()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,,0]\\![bind,帽子,X,1]\\![bind,帽子,Y,1]")
        return state
    end

    test("全脱衣後に着けていた他パーツは数値1で false（\"\" に落とさない不明）になる (3.13)", function()
        local spot_hat = stripped_then_two().spots[0].binds["帽子"]
        expect(spot_hat["X"]):toBe(false)
        expect(spot_hat["Y"]):toBe(1)
        expect(spot_hat[""]):toBe(0)
    end)

    test("false のパーツのトグルは不明扱いでカテゴリを不明化する (3.12, 3.4)", function()
        local state = stripped_then_two()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,X]")
        expect(state.spots[0].binds["帽子"]):toBe(nil)
        expect(state.actors.A.binds["帽子"]):toBe(false)
    end)

    test("false のパーツは後続の明示 bind で上書きされる (3.1)", function()
        local state = stripped_then_two()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,X,0]")
        local spot_hat = state.spots[0].binds["帽子"]
        expect(spot_hat["X"]):toBe(0)
        expect(spot_hat["Y"]):toBe(1)
        expect(spot_hat[""]):toBe(0)
    end)

    test("false のパーツがあっても全脱衣で丸ごと置き換える (3.11)", function()
        local state = stripped_then_two()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,,0]")
        local spot_hat = state.spots[0].binds["帽子"]
        expect(spot_hat["X"]):toBe(nil)
        expect(spot_hat["Y"]):toBe(nil)
        expect(spot_hat[""]):toBe(0)
    end)

    test("実効値が既知のパーツのトグルは反転値の明示 bind として記録する (3.12)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら,1]")
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら]")
        expect(state.spots[0].binds["帽子"]["麦わら"]):toBe(0)
        expect(state.actors.A.binds["帽子"]["麦わら"]):toBe(0)
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,麦わら,]")
        expect(state.spots[0].binds["帽子"]["麦わら"]):toBe(1)
        expect(state.actors.A.binds["帽子"]["麦わら"]):toBe(1)
    end)

    test("全脱衣の後のトグルは全脱衣の値を実効値として反転する (3.12)", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,,0]\\![bind,帽子,リボン]")
        expect(state.spots[0].binds["帽子"]["リボン"]):toBe(1)
        expect(state.spots[0].binds["帽子"][""]):toBe(0)
        expect(state.actors.A.binds["帽子"]["リボン"]):toBe(1)
        expect(table.concat(state.actors.A.order["帽子"], ",")):toBe("リボン")
    end)

    --- X がスポット0で帽子・腕を、Y がスポット1で帽子を記録済みで、A も帽子・腕を記録済みの状態
    local function dressed()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, { name = "X" }, 0, "\\![bind,帽子,花,1]\\![bind,腕,時計,1]")
        APPEARANCE.observe(state, { name = "Y" }, 1, "\\![bind,帽子,花,0]")
        state.actors.A = { binds = { ["帽子"] = { ["麦わら"] = 1 }, ["腕"] = { ["時計"] = 0 } },
            order = { ["帽子"] = { "麦わら" }, ["腕"] = { "時計" } } }
        return state
    end

    local uncertain = {
        { "実効値が不明なパーツのトグル", "\\![bind,帽子,リボン]" },
        { "カテゴリ単位の着衣 ,,1", "\\![bind,帽子,,1]" },
        { "カテゴリ単位のトグル ,,", "\\![bind,帽子,,]" },
        { "カテゴリ単位のトグル (数値省略)", "\\![bind,帽子]" },
    }
    for _, c in ipairs(uncertain) do
        test(c[1] .. " はスポットのカテゴリを不明、発話アクターのカテゴリを false にする (3.4)", function()
            local state = dressed()
            APPEARANCE.observe(state, A, 0, c[2])
            expect(state.spots[0].binds["帽子"]):toBe(nil)
            expect(state.actors.A.binds["帽子"]):toBe(false)
            expect(state.actors.A.order["帽子"]):toBe(nil)
            -- 他カテゴリ・他アクター・他スポットは変えない
            expect(state.actors.A.binds["腕"]["時計"]):toBe(0)
            expect(state.spots[0].binds["腕"]["時計"]):toBe(1)
            expect(state.actors.X.binds["帽子"]["花"]):toBe(1)
            expect(state.spots[1].binds["帽子"]["花"]):toBe(0)
        end)
    end

    test("不明化したカテゴリは後続の明示 bind で再び記録する (3.4)", function()
        local state = dressed()
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,,1]\\![bind,帽子,リボン,1]")
        expect(state.actors.A.binds["帽子"]["リボン"]):toBe(1)
        expect(state.actors.A.binds["帽子"]["麦わら"]):toBe(nil)
        expect(table.concat(state.actors.A.order["帽子"], ",")):toBe("リボン")
        expect(state.spots[0].binds["帽子"]["リボン"]):toBe(1)
    end)

    for _, arg in ipairs({ '\\![bind,"帽,子",リボン,1]', "\\![bind,帽子,リ\\ボン,1]" }) do
        test("引数に \" や \\ を含む bind は発話スポットの着せ替えのみ全カテゴリ不明にする: " .. arg, function()
            local state = dressed()
            APPEARANCE.observe(state, { name = "X" }, 0, "\\s[3]")
            APPEARANCE.observe(state, A, 0, arg)
            expect(next(state.spots[0].binds)):toBe(nil)
            expect(state.spots[0].surface):toBe("3")
            expect(state.spots[1].binds["帽子"]["花"]):toBe(0)
            expect(state.actors.A.binds["帽子"]["麦わら"]):toBe(1)
            expect(state.actors.A.binds["腕"]["時計"]):toBe(0)
            expect(table.concat(state.actors.A.order["帽子"], ",")):toBe("麦わら")
        end)
    end

    test("アクター名が nil ならスポット側のみ更新する", function()
        local state = APPEARANCE.new()
        APPEARANCE.observe(state, {}, 0, "\\![bind,帽子,リボン,1]\\![bind,腕,,1]")
        expect(next(state.actors)):toBe(nil)
        expect(state.spots[0].binds["帽子"]["リボン"]):toBe(1)
    end)

    test("スコープ切替後の bind は記録しない (1.7)", function()
        local state = dressed()
        APPEARANCE.observe(state, A, 0, "\\![bind,腕,時計,1]\\1\\![bind,帽子,リボン,1]")
        APPEARANCE.observe(state, A, 0, "\\![bind,帽子,花,1]")
        expect(state.actors.A.binds["腕"]["時計"]):toBe(1)
        expect(state.actors.A.binds["帽子"]["リボン"]):toBe(nil)
        expect(state.actors.A.binds["帽子"]["花"]):toBe(nil)
        expect(next(state.spots)):toBe(nil)
    end)

    test("生スクリプト中の bind は記録せず全スポットを不明にする (1.8)", function()
        local state = dressed()
        APPEARANCE.observe(state, nil, nil, "\\![bind,帽子,リボン,1]")
        expect(next(state.spots)):toBe(nil)
        expect(state.actors.A.binds["帽子"]["リボン"]):toBe(nil)
        expect(state.actors.X.binds["帽子"]["リボン"]):toBe(nil)
    end)
end)

--- 他アクター X がスポット 0 で発話し、スポット 0 のサーフェスが surface の状態を作る
local function spot0_by_x(surface)
    local state = APPEARANCE.new()
    state.owners[0] = "X"
    state.last_spots.X = 0
    if surface then
        state.spots[0] = { surface = surface, binds = {} }
    end
    return state
end

--- A の既知サーフェスを記録する（前回発話もスポット 0。スポットは変えない）
local function known_a(state, surface)
    state.actors.A = { surface = surface, binds = {} }
    state.last_spots.A = 0
    return state
end

describe("APPEARANCE.restore - サーフェス比較", function()
    test("一致なら空文字列", function()
        local state = known_a(spot0_by_x("5"), "5")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("")
        expect(state.spots[0].surface):toBe("5")
    end)

    test("不一致なら \\s[既知] を返しスポットを更新する", function()
        local state = known_a(spot0_by_x("10"), "5")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("\\s[5]")
        expect(state.spots[0].surface):toBe("5")
        expect(state.actors.A.surface):toBe("5")
    end)

    test("スポットが不明なら \\s[既知] を返しスポットを更新する (4.4)", function()
        local state = known_a(spot0_by_x(nil), "smile")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("\\s[smile]")
        expect(state.spots[0].surface):toBe("smile")
    end)

    test("既知なし・既定なしなら空文字列でスポットを変えない (2.3, 2.9)", function()
        local state = spot0_by_x("10")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("")
        expect(state.spots[0].surface):toBe("10")
        local unknown = spot0_by_x(nil)
        expect(APPEARANCE.restore(unknown, A, 0, {})):toBe("")
        expect(unknown.spots[0]):toBe(nil)
    end)

    test("既知が無ければ既定サーフェスへフォールバックする (2.8)", function()
        local B = { name = "B", surface = 0 }
        local state = spot0_by_x("10")
        expect(APPEARANCE.restore(state, B, 0, {})):toBe("\\s[0]")
        expect(state.spots[0].surface):toBe("0")
        local C = { name = "C", surface = "smile" }
        expect(APPEARANCE.restore(spot0_by_x(nil), C, 0, {})):toBe("\\s[smile]")
    end)

    test("既定より記録した既知サーフェスを優先する", function()
        local B = { name = "B", surface = 0 }
        local state = spot0_by_x("10")
        state.actors.B = { surface = "3", binds = {} }
        expect(APPEARANCE.restore(state, B, 0, {})):toBe("\\s[3]")
    end)

    test("既定サーフェスが表示中と一致なら空文字列", function()
        local B = { name = "B", surface = 10 }
        expect(APPEARANCE.restore(spot0_by_x("10"), B, 0, {})):toBe("")
    end)

    test("想定外の型・空文字列の既定サーフェスは無視する", function()
        for _, v in ipairs({ {}, true, "" }) do
            local B = { name = "B", surface = v }
            local state = spot0_by_x("10")
            expect(APPEARANCE.restore(state, B, 0, {})):toBe("")
            expect(state.spots[0].surface):toBe("10")
        end
    end)

    test("アクター名が nil なら復旧しない", function()
        local state = spot0_by_x("10")
        expect(APPEARANCE.restore(state, { surface = 0 }, 0, {})):toBe("")
        expect(state.spots[0].surface):toBe("10")
    end)
end)

describe("APPEARANCE.restore - 先頭タグ列", function()
    local suppressed = {
        { "surface トークン", { { type = "surface", id = 3 } } },
        { "sakura_script の \\s[ID]", { { type = "sakura_script", text = "\\s[3]" } } },
        { "sakura_script の \\sN", { { type = "sakura_script", text = "\\_w[100]\\s3" } } },
        { "タグのみのトークンの後", {
            { type = "wait", ms = 100 }, { type = "newline", n = 1 }, { type = "clear" },
            { type = "talk", text = "" }, { type = "yield" }, { type = "surface", id = 3 },
        } },
        { "無関係なタグの後", { { type = "sakura_script", text = "\\![set,a,b]\\_w[1]\\s[3]あ" } } },
        { "talk 先頭の \\s[ID]（単語参照の展開）", { { type = "talk", text = "\\s[3]あ" } } },
        { "talk 先頭タグ列の \\sN", { { type = "talk", text = "\\_w[100]\\s3あ" } } },
    }
    for _, c in ipairs(suppressed) do
        test(c[1] .. " があれば復旧しない (2.4)", function()
            local state = known_a(spot0_by_x("10"), "5")
            expect(APPEARANCE.restore(state, A, 0, c[2])):toBe("")
            expect(state.spots[0].surface):toBe("10")
        end)
    end

    local not_suppressed = {
        { "一般文字列 talk の後", { { type = "talk", text = "あ" }, { type = "surface", id = 3 } } },
        { "sakura_script 内の一般文字の後", { { type = "sakura_script", text = "あ\\s[3]" } } },
        { "エスケープ \\\\ の後", { { type = "sakura_script", text = "\\\\\\s[3]" } } },
        { "スコープ切替タグの後", { { type = "sakura_script", text = "\\1\\s[3]" } } },
        { "raw_script の後", { { type = "raw_script", text = "" }, { type = "surface", id = 3 } } },
        { "入れ子の \\s のみ", { { type = "sakura_script", text = "\\![raise,OnX,\\s[3]]" } } },
        { "talk 内の一般文字の後", { { type = "talk", text = "あ\\s[3]" } } },
        { "talk 内のスコープ切替タグの後", { { type = "talk", text = "\\1\\s[3]あ" } } },
    }
    for _, c in ipairs(not_suppressed) do
        test(c[1] .. " のサーフェス変更では抑止しない (2.10)", function()
            local state = known_a(spot0_by_x("10"), "5")
            expect(APPEARANCE.restore(state, A, 0, c[2])):toBe("\\s[5]")
            expect(state.spots[0].surface):toBe("5")
        end)
    end

    test("一般文字列後の変更は復旧後の観測で既知・表示中を最後の値にする (2.10)", function()
        local state = known_a(spot0_by_x("10"), "5")
        expect(APPEARANCE.restore(state, A, 0, { { type = "talk", text = "あ" } })):toBe("\\s[5]")
        APPEARANCE.observe(state, A, 0, "あ\\s[3]い\\s[4]")
        expect(state.actors.A.surface):toBe("4")
        expect(state.spots[0].surface):toBe("4")
    end)

    test("トークン列を変更しない", function()
        local tokens = { { type = "sakura_script", text = "\\s[3]" } }
        APPEARANCE.restore(known_a(spot0_by_x("10"), "5"), A, 0, tokens)
        expect(#tokens):toBe(1)
        expect(tokens[1].text):toBe("\\s[3]")
    end)
end)

describe("APPEARANCE.restore - 継続と交代", function()
    test("同一アクターの継続ならスポット不明でも復旧せず状態を変えない (2.11)", function()
        local state = APPEARANCE.new()
        state.actors.A = { surface = "5", binds = {} }
        state.owners[0] = "A"
        state.last_spots.A = 0
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("")
        expect(state.spots[0]):toBe(nil)
        expect(state.owners[0]):toBe("A")
        expect(state.last_spots.A):toBe(0)
    end)

    test("同一アクターの継続なら既定サーフェスでも復旧しない (2.11)", function()
        local B = { name = "B", surface = 0 }
        local state = APPEARANCE.new()
        state.owners[0] = "B"
        state.last_spots.B = 0
        state.spots[0] = { surface = "10", binds = {} }
        expect(APPEARANCE.restore(state, B, 0, {})):toBe("")
        expect(state.spots[0].surface):toBe("10")
    end)

    test("直前発話アクター・前回発話スポットを更新する", function()
        local state = known_a(spot0_by_x("10"), "5")
        state.last_spots.A = 1
        APPEARANCE.restore(state, A, 0, {})
        expect(state.owners[0]):toBe("A")
        expect(state.last_spots.A):toBe(0)
    end)

    test("初回の発話は既定サーフェスがあれば不明スポットへ復旧し、次は継続になる", function()
        local B = { name = "B", surface = 0 }
        local state = APPEARANCE.new()
        expect(APPEARANCE.restore(state, B, 0, {})):toBe("\\s[0]")
        expect(state.owners[0]):toBe("B")
        expect(state.last_spots.B):toBe(0)
        state.spots = {}
        expect(APPEARANCE.restore(state, B, 0, {})):toBe("")
    end)

    test("A がスポット0→1→0 と戻るとスポット0で既知サーフェスを復旧する (5.5)", function()
        local state = APPEARANCE.new()
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("")
        APPEARANCE.observe(state, A, 0, "\\s[0]あ")
        -- 移動先スポット1（不明）へも外見が追従する
        expect(APPEARANCE.restore(state, A, 1, {})):toBe("\\s[0]")
        APPEARANCE.observe(state, A, 1, "\\s[5]い")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("\\s[5]")
        expect(state.spots[0].surface):toBe("5")
        expect(state.owners[0]):toBe("A")
        expect(state.last_spots.A):toBe(0)
    end)

    test("A→B→A の交代で A の既知サーフェスを復旧する", function()
        local B = { name = "B" }
        local state = APPEARANCE.new()
        APPEARANCE.restore(state, A, 0, {})
        APPEARANCE.observe(state, A, 0, "\\s[0]A1")
        expect(APPEARANCE.restore(state, B, 0, {})):toBe("")
        APPEARANCE.observe(state, B, 0, "\\s[10]B1")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("\\s[0]")
        expect(state.spots[0].surface):toBe("0")
    end)

    test("不明化の後も別アクターとの交代なら復旧する", function()
        local state = known_a(spot0_by_x("10"), "5")
        APPEARANCE.observe(state, nil, nil, "\\s[1]")
        expect(APPEARANCE.restore(state, A, 0, {})):toBe("\\s[5]")
    end)
end)
