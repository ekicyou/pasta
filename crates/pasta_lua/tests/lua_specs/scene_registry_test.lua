-- pasta.scene（SCENE レジストリ API）単体テスト
-- review-improvement-loop cell 3.46 (G1): SCENE モジュール直接テストの欠落を補完する
-- （既存の act_find_scene_test 等は ACT 経由のフォールバック検索のみで、
--   レジストリ操作・カウンタ・search 型ガード・co_exec は未カバーだった）
--
-- STORE.scenes / STORE.counters はスイート間で共有されるため、
-- 衝突しない一意プレフィックス "spec346_" を使用する。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 先行スイートが pasta.store / pasta.scene を個別リロードしている場合、
-- ロード済みモジュール間で STORE 参照が分裂していることがある。
-- 整合した同一インスタンス集合を保証するため一括で新規ロードする（既存スイートと同一規約）。
package.loaded["pasta.store"] = nil
package.loaded["pasta.word"] = nil
package.loaded["pasta.scene"] = nil

local SCENE = require("pasta.scene")
local STORE = require("pasta.store")

describe("SCENE.get_or_increment_counter", function()
    test("初回は 1、同一ベース名の2回目は 2 を返す", function()
        expect(SCENE.get_or_increment_counter("spec346_counter_a")):toBe(1)
        expect(SCENE.get_or_increment_counter("spec346_counter_a")):toBe(2)
    end)

    test("ベース名ごとにカウンタは独立している", function()
        expect(SCENE.get_or_increment_counter("spec346_counter_b")):toBe(1)
        expect(SCENE.get_or_increment_counter("spec346_counter_c")):toBe(1)
        expect(SCENE.get_or_increment_counter("spec346_counter_b")):toBe(2)
    end)
end)

describe("SCENE.register / get / get_global_table", function()
    test("register がグローバルテーブルを生成しシーン関数を格納する", function()
        local fn = function() return "spec346" end
        SCENE.register("spec346_reg", "本編", fn)

        expect(SCENE.get("spec346_reg", "本編")):toBe(fn)
        local tbl = SCENE.get_global_table("spec346_reg")
        expect(tbl.__global_name__):toBe("spec346_reg")
    end)

    test("同一グローバル名への再 register はテーブルを再生成せず追記する", function()
        local fn1 = function() end
        local fn2 = function() end
        SCENE.register("spec346_reg2", "場面1", fn1)
        local tbl_before = SCENE.get_global_table("spec346_reg2")
        SCENE.register("spec346_reg2", "場面2", fn2)

        expect(SCENE.get_global_table("spec346_reg2")):toBe(tbl_before)
        expect(SCENE.get("spec346_reg2", "場面1")):toBe(fn1)
        expect(SCENE.get("spec346_reg2", "場面2")):toBe(fn2)
    end)

    test("未登録グローバル名は get_global_table / get とも nil を返す", function()
        expect(SCENE.get_global_table("spec346_unknown")):toBeNil()
        expect(SCENE.get("spec346_unknown", "any")):toBeNil()
        -- 登録済みグローバル・未登録ローカルも nil
        SCENE.register("spec346_reg3", "存在", function() end)
        expect(SCENE.get("spec346_reg3", "不在")):toBeNil()
    end)

    test("create_global_table は同一名で同一テーブルを返す（冪等）", function()
        local t1 = SCENE.create_global_table("spec346_cgt")
        local t2 = SCENE.create_global_table("spec346_cgt")
        expect(t1):toBe(t2)
        expect(t1.__global_name__):toBe("spec346_cgt")
    end)
end)

describe("SCENE - シーンテーブルの create_word メタメソッド", function()
    test("scene_table:create_word がグローバル名スコープのローカル単語を登録する", function()
        SCENE.register("spec346_word_scene", "__start__", function() end)
        local tbl = SCENE.get_global_table("spec346_word_scene")

        tbl:create_word("挨拶"):entry("おはよう", "こんにちは")

        local words = STORE.local_words["spec346_word_scene"]
        expect(words):toBeTruthy()
        expect(#words["挨拶"]):toBe(1)
        expect(words["挨拶"][1][1]):toBe("おはよう")
        expect(words["挨拶"][1][2]):toBe("こんにちは")
    end)
end)

describe("SCENE - アクセサ", function()
    test("get_start は __start__ シーン関数を返す（未登録は nil）", function()
        local start_fn = function() end
        SCENE.register("spec346_start", "__start__", start_fn)
        expect(SCENE.get_start("spec346_start")):toBe(start_fn)
        expect(SCENE.get_start("spec346_no_start")):toBeNil()
    end)

    test("get_all_scenes は STORE.scenes と同一テーブルを返す", function()
        expect(SCENE.get_all_scenes()):toBe(STORE.scenes)
    end)
end)

describe("SCENE.create_scene - カウンタ採番", function()
    test("ベース名にカウンタを付与した一意なグローバル名で登録する", function()
        local fn1 = function() end
        local tbl1 = SCENE.create_scene("spec346_cs", "__start__", fn1)
        expect(tbl1.__global_name__):toBe("spec346_cs_1")
        expect(SCENE.get("spec346_cs_1", "__start__")):toBe(fn1)

        local fn2 = function() end
        local tbl2 = SCENE.create_scene("spec346_cs", "__start__", fn2)
        expect(tbl2.__global_name__):toBe("spec346_cs_2")
        expect(SCENE.get("spec346_cs_2", "__start__")):toBe(fn2)
    end)

    test("local_name / scene_func 省略時は登録なしで空のグローバルテーブルを返す", function()
        local tbl = SCENE.create_scene("spec346_cs_empty")
        expect(tbl.__global_name__):toBe("spec346_cs_empty_1")
        expect(SCENE.get_start("spec346_cs_empty_1")):toBeNil()
    end)
end)

describe("SCENE.search", function()
    test("name が文字列でない場合は @pasta_search を呼ばず nil を返す（型ガード）", function()
        expect(SCENE.search(nil)):toBeNil()
        expect(SCENE.search(123)):toBeNil()
        expect(SCENE.search({})):toBeNil()
    end)

    test("検索ヒット時は __call 可能な結果オブジェクトを返す", function()
        local saved_search = package.loaded["@pasta_search"]
        local ok, err = pcall(function()
            local fn = function(arg) return "called:" .. tostring(arg) end
            SCENE.register("spec346_hit", "本編", fn)
            package.loaded["@pasta_search"] = {
                search_scene = function(_self, name, scope)
                    if name == "spec346_hit" then
                        return "spec346_hit", "本編"
                    end
                    return nil
                end,
            }

            local result = SCENE.search("spec346_hit")
            expect(result):toBeTruthy()
            expect(result.global_name):toBe("spec346_hit")
            expect(result.local_name):toBe("本編")
            expect(result.func):toBe(fn)
            -- __call メタメソッドで直接呼び出し可能
            expect(result("X")):toBe("called:X")
        end)
        package.loaded["@pasta_search"] = saved_search
        if not ok then error(err, 0) end
    end)

    test("search_scene が nil を返したとき nil を返す", function()
        local saved_search = package.loaded["@pasta_search"]
        local ok, err = pcall(function()
            package.loaded["@pasta_search"] = {
                search_scene = function() return nil end,
            }
            expect(SCENE.search("spec346_miss")):toBeNil()
        end)
        package.loaded["@pasta_search"] = saved_search
        if not ok then error(err, 0) end
    end)

    test("Rust 側ヒットでも Lua 側にシーン関数が未登録なら nil を返す", function()
        local saved_search = package.loaded["@pasta_search"]
        local ok, err = pcall(function()
            package.loaded["@pasta_search"] = {
                search_scene = function() return "spec346_rust_only", "本編" end,
            }
            -- "spec346_rust_only" は SCENE.register していない
            expect(SCENE.search("anything")):toBeNil()
        end)
        package.loaded["@pasta_search"] = saved_search
        if not ok then error(err, 0) end
    end)
end)

describe("SCENE.co_exec", function()
    --- find_scene / build を備えた最小 act スタブを生成
    local function make_act(find_result, build_result)
        return {
            find_scene = function(_self, _name, _scope, _attrs) return find_result end,
            build = function(_self) return build_result end,
        }
    end

    test("find_scene が nil を返すとき nil を返す", function()
        local act = make_act(nil, nil)
        expect(SCENE.co_exec(act, "missing")):toBeNil()
    end)

    test("find_scene が非関数を返すとき nil を返す", function()
        local act = make_act("not a function", nil)
        expect(SCENE.co_exec(act, "bad")):toBeNil()
    end)

    test("コルーチンを返し、resume でシーン実行後に act:build() 結果を返す", function()
        local executed_with = nil
        local scene_fn = function(act_arg) executed_with = act_arg end
        local act = make_act(scene_fn, "BUILT_RESULT")

        local co = SCENE.co_exec(act, "spec346_co")
        expect(type(co)):toBe("thread")

        local ok, result = coroutine.resume(co, act)
        expect(ok):toBe(true)
        expect(result):toBe("BUILT_RESULT")
        expect(executed_with):toBe(act)
        expect(coroutine.status(co)):toBe("dead")
    end)

    test("act:build() が nil のときコルーチンは値なしで終了する", function()
        local act = make_act(function() end, nil)
        local co = SCENE.co_exec(act, "spec346_co_nil")

        local ok, result = coroutine.resume(co, act)
        expect(ok):toBe(true)
        expect(result):toBeNil()
    end)
end)

describe("SCENE.co_exec_exact", function()
    --- @pasta_search を差し替えて fn を動かし、元に戻す
    local function with_search(search, fn)
        local saved_search = package.loaded["@pasta_search"]
        package.loaded["@pasta_search"] = search
        local ok, err = pcall(fn)
        package.loaded["@pasta_search"] = saved_search
        if not ok then error(err, 0) end
    end

    local search = {
        search_scene_exact = function(_self, name, scope)
            assert(scope == nil)
            if name == "spec346_exact" then return "spec346_exact_1", "__start__" end
            if name == "spec346_exact_unreg" then return "spec346_exact_unreg_1", "__start__" end
            return nil
        end,
        search_local_scene_exact = function(_self, name, local_name)
            if name ~= "spec346_exact" then return nil, "global" end
            if local_name == "子" then return "spec346_exact_1", "子_1" end
            if local_name == "未登録" then return "spec346_exact_1", "未登録_1" end
            return nil, "local"
        end,
    }

    test("包んだコルーチンは最初の再開の act と引数で関数を呼び、最後に組み立ての結果を返す", function()
        local got = {}
        SCENE.register("spec346_exact_1", "__start__", function(a, x, y)
            got = { a = a, x = x, y = y }
        end)
        with_search(search, function()
            local co, miss = SCENE.co_exec_exact("spec346_exact")
            expect(type(co)):toBe("thread")
            expect(miss):toBeNil()
            local act = { build = function() return "BUILT" end }
            local ok, result = coroutine.resume(co, act, "x1", "y1")
            expect(ok):toBe(true)
            expect(result):toBe("BUILT")
            expect(got.a):toBe(act)
            expect(got.x):toBe("x1")
            expect(got.y):toBe("y1")
            expect(coroutine.status(co)):toBe("dead")
        end)
    end)

    test("ローカルを渡すと、その大域の中のローカルを包む", function()
        local called = false
        SCENE.register("spec346_exact_1", "子_1", function() called = true end)
        with_search(search, function()
            local co = SCENE.co_exec_exact("spec346_exact", "子")
            coroutine.resume(co, { build = function() return nil end })
            expect(called):toBe(true)
        end)
    end)

    test("見つからなかった側を 2 つ目の値で返す", function()
        with_search(search, function()
            local co, miss = SCENE.co_exec_exact("spec346_none")
            expect(co):toBeNil()
            expect(miss):toBe("global")
            co, miss = SCENE.co_exec_exact("spec346_exact_unreg")
            expect(co):toBeNil()
            expect(miss):toBe("global")
            co, miss = SCENE.co_exec_exact("spec346_none", "子")
            expect(co):toBeNil()
            expect(miss):toBe("global")
            co, miss = SCENE.co_exec_exact("spec346_exact", "無い")
            expect(co):toBeNil()
            expect(miss):toBe("local")
            co, miss = SCENE.co_exec_exact("spec346_exact", "未登録")
            expect(co):toBeNil()
            expect(miss):toBe("local")
        end)
    end)

    test("get_exact は関数を実行せずに返し、見つからなかった側は co_exec_exact と同じ", function()
        local called = false
        local fn = function() called = true end
        SCENE.register("spec346_exact_1", "子_1", fn)
        with_search(search, function()
            expect(SCENE.get_exact("spec346_exact", "子")):toBe(fn)
            expect(called):toBe(false)
            local got, miss = SCENE.get_exact("spec346_none")
            expect(got):toBeNil()
            expect(miss):toBe("global")
            got, miss = SCENE.get_exact("spec346_exact", "無い")
            expect(got):toBeNil()
            expect(miss):toBe("local")
        end)
    end)

    test("co_exec_registered は関数でない値に nil を返す", function()
        SCENE.register("spec346_exact_1", "値", "not a function")
        expect(SCENE.co_exec_registered("spec346_exact_1", "値")):toBeNil()
        expect(SCENE.co_exec_registered("spec346_exact_none", "__start__")):toBeNil()
    end)
end)

describe("SCENE.wrap", function()
    test("作るだけで関数は動かさない。再開で act と引数を渡し、関数の戻り値でなく組み立ての結果を返す", function()
        local got
        local co = SCENE.wrap(function(a, x)
            got = { a = a, x = x }
            return "IGNORED"
        end)
        expect(type(co)):toBe("thread")
        expect(got):toBeNil()
        local act = { build = function() return "BUILT" end }
        local ok, result = coroutine.resume(co, act, 7)
        expect(ok):toBe(true)
        expect(result):toBe("BUILT")
        expect(got.a):toBe(act)
        expect(got.x):toBe(7)
    end)

    test("関数が区切ると、続きの再開の後に組み立てる", function()
        local built = 0
        local act = { build = function() built = built + 1; return "B" .. built end }
        local co = SCENE.wrap(function() coroutine.yield("Y") end)
        local _, first = coroutine.resume(co, act)
        expect(first):toBe("Y")
        expect(built):toBe(0)
        local _, last = coroutine.resume(co)
        expect(last):toBe("B1")
        expect(coroutine.status(co)):toBe("dead")
    end)
end)

describe("SCENE.register / create_scene - 全体のシーンの表", function()
    test("register と create_scene は create_global_table と同じ表を使う", function()
        local tbl = SCENE.create_global_table("spec346_shared_1")
        SCENE.register("spec346_shared_1", "子", function() end)
        expect(SCENE.get_global_table("spec346_shared_1")):toBe(tbl)
        expect(tbl.__global_name__):toBe("spec346_shared_1")
        local created = SCENE.create_scene("spec346_shared2")
        expect(created):toBe(SCENE.get_global_table("spec346_shared2_1"))
        expect(created.__global_name__):toBe("spec346_shared2_1")
    end)
end)
