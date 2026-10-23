-- chain-talk-interrupt: 待っている続きを、割り込んだシーンが台詞を出したかで残すか捨てるかのテスト
-- design.md の表「続きの決まり」の各行と、台詞の判定・はっきり捨てる印・予約の確かめの単体の確かめ。
--
-- 検索は @pasta_search の search_scene_exact（と OnTalk 用の search_scene）を差し替えて引く。
-- 組み立ては本物の sakura_builder を使う。台詞の字は、本物の talk_to_script が
-- ウェイトを挟まないように、句読点の無い英字にする。
-- STORE.scenes はスイート間で共有されるため、衝突しない一意プレフィックス "cti_" を使う。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 依存閉包を整合した同一インスタンス集合として一括新規ロードする（shiori_entry_test と同じ）
local RELOAD = {
    "pasta.shiori.entry",
    "pasta.shiori.event",
    "pasta.shiori.event.register",
    "pasta.shiori.event.callback",
    "pasta.shiori.event.boot",
    "pasta.shiori.event.choice_select",
    "pasta.shiori.event.second_change",
    "pasta.shiori.event.mouse",
    "pasta.shiori.event.scene_play",
    "pasta.shiori.event.kick",
    "pasta.shiori.event.external_control",
    "pasta.shiori.event.virtual_dispatcher",
    "pasta.shiori.res",
    "pasta.shiori.stop",
    "pasta.shiori.act",
    "pasta.shiori.sakura_builder",
    "pasta.act",
    "pasta.actor",
    "pasta.scene",
    "pasta.word",
    "pasta.global",
    "pasta.store",
    "pasta.config",
    "pasta.save",
}
for _, name in ipairs(RELOAD) do
    package.loaded[name] = nil
end

local ENTRY = require("pasta.shiori.entry")
local REG = require("pasta.shiori.event.register")
local CALLBACK = require("pasta.shiori.event.callback")
local KICK = require("pasta.shiori.event.kick")
local SCENE = require("pasta.scene")
local STORE = require("pasta.store")
local SHIORI_ACT = require("pasta.shiori.act")
local ACT = require("pasta.act")
local BUILDER = require("pasta.shiori.sakura_builder")

-- ============================================================================
-- 共通の道具
-- ============================================================================

--- package.loaded の複数のキーを差し替えて fn を走らせ、必ず元に戻す
local function with_loaded(stubs, fn)
    local saved = {}
    for name, stub in pairs(stubs) do
        saved[name] = package.loaded[name]
        package.loaded[name] = stub
    end
    local ok, err = pcall(fn)
    for name in pairs(stubs) do
        package.loaded[name] = saved[name]
    end
    if not ok then error(err, 0) end
end

--- [logging] balloon_warning を真にした pasta.config の代役（失敗の表記を必ず出す）
local function make_config()
    local real = package.loaded["pasta.config"]
    return {
        get = function(section, key, default)
            if section == "logging" and key == "balloon_warning" then return true end
            return real.get(section, key, default)
        end,
    }
end

--- 状態を空にし、台詞に使うアクターを置く
local function reset_state()
    STORE.reset()
    CALLBACK.reset()
    STORE.actors = { sakura = { name = "さくら", spot = 0 } }
end

--- 中断のシーン（＊チェイントーク中断）の代役。動いたかを ran に記録する
local chudan = { ran = false }

--- 書いた名前 → 登録名（大域）の表で引く @pasta_search の代役
--- @param names table<string, string>
local function make_search(names)
    return {
        search_scene_exact = function(_, name, _scope)
            if names[name] then return names[name], "__start__" end
            return nil
        end,
        -- 外から起こした OnTalk は act:find_scene の L5（大域の前方一致）で引かれる
        search_scene = function(_, name, scope)
            if scope == nil and names[name] then return names[name], "__start__" end
            return nil
        end,
    }
end

--- 状態を空にして fn を走らせる。opts.chudan が真なら ＊チェイントーク中断 を検索に入れる
--- opts.names: 検索に足す 書いた名前 → 登録名
local function with_env(opts, fn)
    opts = opts or {}
    local names = {}
    for k, v in pairs(opts.names or {}) do names[k] = v end
    with_loaded({
        ["@pasta_search"] = make_search(names),
        ["@pasta_debug"] = {
            enabled = function() return true end,
            scene_at = function() return "cti_play_1", "__start__" end,
        },
        ["pasta.config"] = make_config(),
    }, function()
        reset_state()
        chudan.ran = false
        if opts.chudan then
            SCENE.register("cti_chudan_1", "__start__", function(act)
                chudan.ran = true
                act:talk(STORE.actors.sakura, "CHUDAN")
            end)
            names["チェイントーク中断"] = "cti_chudan_1"
        end
        local ok, err = pcall(fn)
        reset_state()
        if not ok then error(err, 0) end
    end)
end

--- イベントを SHIORI.request で送る
--- @param id string イベント名
--- @param reference table|nil Reference（0 始まり）
--- @return string
local function request(id, reference)
    return ENTRY.request({
        id = id, method = "get", version = 30, reference = reference or {},
        security_level = "local",
    })
end

--- 応答のヘッダ行の値を返す（無ければ nil）
local function header_value(res, name)
    local prefix = name .. ": "
    for line in res:gmatch("(.-)\r\n") do
        if line:sub(1, #prefix) == prefix then
            return line:sub(#prefix + 1)
        end
    end
    return nil
end

--- 応答が字句 part を含むか
local function has(res, part)
    return res:find(part, 1, true) ~= nil
end

--- 台詞を積む
local function talk(act, text)
    act:talk(STORE.actors.sakura, text)
end

--- 登録名 global のシーンを返すハンドラを REG[event] に置き、シーン関数を登録する
local function register_scene(event, global, fn)
    SCENE.register(global, "__start__", fn)
    REG[event] = function(_act)
        return SCENE.co_exec_registered(global, "__start__")
    end
end

--- 次の自然なおしゃべりの代わりに、待っている続きをそのまま返すハンドラ
REG.OnCtiContinue = function(_act)
    return STORE.co_scene
end

--- 前半（ZENHAN）を出して区切り、後半（KOUHAN）を待たせる続きを作る
--- @return thread 待っている続き
local function start_chain()
    register_scene("OnCtiChain", "cti_chain_1", function(act)
        talk(act, "ZENHAN")
        act:yield()
        talk(act, "KOUHAN")
    end)
    local res = request("OnCtiChain")
    expect(has(res, "ZENHAN")):toBe(true)
    expect(type(STORE.co_scene)):toBe("thread")
    return STORE.co_scene
end

--- 次の自然なおしゃべりで後半が出て、続きが無くなることを確かめる
local function expect_second_half()
    local res = request("OnCtiContinue")
    expect(has(res, "KOUHAN")):toBe(true)
    expect(STORE.co_scene):toBeNil()
end

--- シーン fn を割り込ませ、その応答を返す
local function interrupt(fn)
    register_scene("OnCtiInterrupt", "cti_int_1", fn)
    return request("OnCtiInterrupt")
end

-- ============================================================================
-- 単体の確かめ: 台詞の判定（1.1）・印の表（1.2）・予約の確かめ（1.3）
-- ============================================================================

describe("BUILDER.speaks - 台詞の判定", function()
    --- act に積んだトークンをまとめ、台詞の判定を返す
    local function speaks(fill)
        reset_state()
        local act = SHIORI_ACT.new(STORE.actors, { id = "OnCtiSpeaks" })
        fill(act)
        local groups = ACT.IMPL.build(act) or {}
        reset_state()
        return BUILDER.speaks(groups)
    end

    test("字のある台詞は真", function()
        expect(speaks(function(act) talk(act, "HELLO") end)):toBe(true)
    end)

    test("選択肢だけでも真", function()
        expect(speaks(function(act) act:choice("cti_target", "CHOOSE") end)):toBe(true)
    end)

    test("表情の単語（字が \\s[5] だけの台詞）だけなら偽", function()
        expect(speaks(function(act) talk(act, "\\s[5]") end)):toBe(false)
    end)

    test("\\![bind,…] のさくらスクリプトだけなら偽", function()
        expect(speaks(function(act)
            act:sakura_script(STORE.actors.sakura, "\\![bind,閉じる,赤,1]")
        end)):toBe(false)
    end)

    test("空の台詞・空の列は偽", function()
        expect(speaks(function(act) talk(act, "") end)):toBe(false)
        expect(BUILDER.speaks({})):toBe(false)
    end)
end)

describe("STORE.co_preempt - はっきり捨てる印の表", function()
    test("読み込むと印の表があり、キーが弱い参照になっている", function()
        expect(type(STORE.co_preempt)):toBe("table")
        expect(getmetatable(STORE.co_preempt).__mode):toBe("k")
    end)

    test("リセットの後は空の新しい表になる", function()
        local old = STORE.co_preempt
        old[coroutine.create(function() end)] = true
        STORE.reset()
        expect(STORE.co_preempt ~= old):toBe(true)
        expect(next(STORE.co_preempt)):toBeNil()
        expect(getmetatable(STORE.co_preempt).__mode):toBe("k")
    end)
end)

describe("CALLBACK.has_staged - 予約の確かめ", function()
    test("予約を置いた直後は真、消費した後は偽", function()
        CALLBACK.reset()
        expect(CALLBACK.has_staged()):toBe(false)
        CALLBACK.stage_pending("OnPastaCallBack1", os.time() + 60, nil)
        expect(CALLBACK.has_staged()):toBe(true)
        CALLBACK.consume_staged(coroutine.create(function() end), {})
        expect(CALLBACK.has_staged()):toBe(false)
        CALLBACK.reset()
    end)

    test("予約を捨てた後は偽", function()
        CALLBACK.reset()
        CALLBACK.stage_pending("OnPastaCallBack1", os.time() + 60, nil)
        CALLBACK.discard_staged()
        expect(CALLBACK.has_staged()):toBe(false)
        CALLBACK.reset()
    end)
end)

-- ============================================================================
-- 続きの決まり: 台詞を出さない割り込みは続きを残す（1.1・1.2・1.3・7.1）
-- 中断のシーンを検索に入れておき、台詞が無ければ動かないことも併せて確かめる（3.3）
-- ============================================================================

describe("続きの決まり - 台詞を出さない割り込みは続きを残す", function()
    test("何も出さないシーンの後も続きが残り、次の自然なおしゃべりで後半が出る", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            local res = interrupt(function() end)
            expect(has(res, "204 No Content")):toBe(true)
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
            expect_second_half()
        end)
    end)

    test("表情の単語だけのシーンでも続きが残る", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            local res = interrupt(function(act) talk(act, "\\s[5]") end)
            expect(has(res, "\\s[5]")):toBe(true)
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
            expect_second_half()
        end)
    end)

    test("\\![bind,…] だけのシーンでも続きが残る", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            local res = interrupt(function(act)
                act:sakura_script(STORE.actors.sakura, "\\![bind,閉じる,赤,1]")
            end)
            expect(has(res, "\\![bind,閉じる,赤,1]")):toBe(true)
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
            expect_second_half()
        end)
    end)

    test("失敗の表記だけのシーンでも続きが残る", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            local res = interrupt(function(act) act:call(nil, nil) end)
            expect(has(header_value(res, "Value") or "", "【")):toBe(true)
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
            expect_second_half()
        end)
    end)

    test("台詞を出さずに区切ったシーンは前の続きを残し、自分の残りを捨てる", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            local res = interrupt(function(act)
                talk(act, "\\s[5]")
                act:yield()
                talk(act, "INTREST")
            end)
            expect(has(res, "\\s[5]")):toBe(true)
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
            local next_res = request("OnCtiContinue")
            expect(has(next_res, "KOUHAN")):toBe(true)
            expect(has(next_res, "INTREST")):toBe(false)
            expect(STORE.co_scene):toBeNil()
        end)
    end)
end)

-- ============================================================================
-- 続きの決まり: 台詞を出した割り込みは続きを捨てる（2.1・2.2・7.1）
-- 中断のシーンが無いとき（中断の台詞があるときは 3.3 のテスト）
-- ============================================================================

describe("続きの決まり - 台詞を出した割り込みは続きを捨てる", function()
    test("台詞を出すシーンは続きを捨て、応答は今どおり反応になる", function()
        with_env({}, function()
            start_chain()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(has(res, "REACTION")):toBe(true)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("選択肢だけを出すシーンも続きを捨てる", function()
        with_env({}, function()
            start_chain()
            interrupt(function(act) act:choice("cti_target", "CHOOSE") end)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("台詞を出して区切ったシーンは、その残りが新しい続きになる", function()
        with_env({}, function()
            local old = start_chain()
            interrupt(function(act)
                talk(act, "NEWFIRST")
                act:yield()
                talk(act, "NEWREST")
            end)
            expect(type(STORE.co_scene)):toBe("thread")
            expect(STORE.co_scene ~= old):toBe(true)
            local res = request("OnCtiContinue")
            expect(has(res, "NEWREST")):toBe(true)
            expect(has(res, "KOUHAN")):toBe(false)
        end)
    end)
end)

-- ============================================================================
-- 続きの決まり: 再開・続きなし（1.4・1.5）
-- ============================================================================

describe("続きの決まり - 再開と続きなし", function()
    test("再開した続きは台詞が無くても今どおり結果で置き換える（最後まで動けば無くなる）", function()
        with_env({ chudan = true }, function()
            register_scene("OnCtiChain", "cti_chain_1", function(act)
                talk(act, "ZENHAN")
                act:yield()
                talk(act, "\\s[5]")
            end)
            request("OnCtiChain")
            expect(type(STORE.co_scene)):toBe("thread")
            local res = request("OnCtiContinue")
            expect(has(res, "\\s[5]")):toBe(true)
            expect(STORE.co_scene):toBeNil()
            expect(chudan.ran):toBe(false)
        end)
    end)

    test("続きが無ければ、台詞を出さずに区切ったシーンの残りが続きになる", function()
        with_env({ chudan = true }, function()
            local res = interrupt(function(act)
                talk(act, "\\s[5]")
                act:yield()
                talk(act, "LATER")
            end)
            expect(has(res, "\\s[5]")):toBe(true)
            expect(type(STORE.co_scene)):toBe("thread")
            expect(chudan.ran):toBe(false)
            expect(has(request("OnCtiContinue"), "LATER")):toBe(true)
        end)
    end)
end)

-- ============================================================================
-- 続きの決まり: 開発者が指名した再生ははっきり捨てる（4.1〜4.3・7.1）
-- 中断のシーンを検索に入れ、台詞を出しても中断の台詞が付かないことを確かめる
-- ============================================================================

describe("続きの決まり - キック・シーン再生ははっきり捨てる", function()
    --- 保留のキックを KICK.try_dispatch で引き当てたコルーチンを返すハンドラ
    local function register_kick(fn)
        SCENE.register("cti_kick_1", "__start__", fn)
        REG.OnCtiKick = function(act)
            STORE.kick_pending = "cti_kick_1"
            return KICK.try_dispatch(act)
        end
    end

    test("キックは台詞が無くても続きを捨てる", function()
        with_env({ chudan = true }, function()
            start_chain()
            register_kick(function(act) talk(act, "\\s[5]") end)
            request("OnCtiKick")
            expect(STORE.co_scene):toBeNil()
            expect(chudan.ran):toBe(false)
        end)
    end)

    test("キックが台詞を出しても中断の台詞を付けず、区切れば残りが続きになる", function()
        with_env({ chudan = true }, function()
            local old = start_chain()
            register_kick(function(act)
                talk(act, "KICKED")
                act:yield()
                talk(act, "KICKREST")
            end)
            local res = request("OnCtiKick")
            expect(has(res, "KICKED")):toBe(true)
            expect(has(res, "CHUDAN")):toBe(false)
            expect(chudan.ran):toBe(false)
            expect(STORE.co_scene ~= old):toBe(true)
            expect(has(request("OnCtiContinue"), "KICKREST")):toBe(true)
        end)
    end)

    test("OnPastaScene は台詞が無くても続きを捨て、台詞を出しても中断の台詞を付けない", function()
        with_env({ chudan = true, names = { cti_play = "cti_play_1" } }, function()
            start_chain()
            SCENE.register("cti_play_1", "__start__", function(act) talk(act, "\\s[5]") end)
            request("OnPastaScene", { [0] = "cti_play" })
            expect(STORE.co_scene):toBeNil()

            start_chain()
            SCENE.register("cti_play_1", "__start__", function(act) talk(act, "PLAYED") end)
            local res = request("OnPastaScene", { [0] = "cti_play" })
            expect(has(res, "PLAYED")):toBe(true)
            expect(has(res, "CHUDAN")):toBe(false)
            expect(STORE.co_scene):toBeNil()
            expect(chudan.ran):toBe(false)
        end)
    end)

    test("OnPastaSceneAt は台詞が無くても続きを捨て、台詞を出しても中断の台詞を付けない", function()
        with_env({ chudan = true }, function()
            start_chain()
            SCENE.register("cti_play_1", "__start__", function(act) talk(act, "\\s[5]") end)
            request("OnPastaSceneAt", { [0] = "dic/a.pasta", [1] = "1" })
            expect(STORE.co_scene):toBeNil()

            start_chain()
            SCENE.register("cti_play_1", "__start__", function(act) talk(act, "PLAYEDAT") end)
            local res = request("OnPastaSceneAt", { [0] = "dic/a.pasta", [1] = "1" })
            expect(has(res, "PLAYEDAT")):toBe(true)
            expect(has(res, "CHUDAN")):toBe(false)
            expect(STORE.co_scene):toBeNil()
            expect(chudan.ran):toBe(false)
        end)
    end)
end)

-- ============================================================================
-- 続きの決まり: エラーで止まったシーンは捨てる（5.1・5.2・5.3・7.1）
-- ============================================================================

describe("続きの決まり - エラーで止まったシーンは捨てる", function()
    test("台詞を積んでいても続きを捨て、表記に中断の台詞が混ざらない", function()
        with_env({ chudan = true }, function()
            start_chain()
            local res = interrupt(function(act)
                talk(act, "BEFOREERR")
                error("cti boom")
            end)
            expect(has(header_value(res, "X-Error-Reason") or "", "cti boom")):toBe(true)
            expect(has(res, "CHUDAN")):toBe(false)
            expect(chudan.ran):toBe(false)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("再開した続き自身がエラーで止まれば、その続きを捨てる", function()
        with_env({ chudan = true }, function()
            register_scene("OnCtiChain", "cti_chain_1", function(act)
                talk(act, "ZENHAN")
                act:yield()
                error("cti boom")
            end)
            request("OnCtiChain")
            expect(type(STORE.co_scene)):toBe("thread")
            local res = request("OnCtiContinue")
            expect(has(header_value(res, "X-Error-Reason") or "", "cti boom")):toBe(true)
            expect(STORE.co_scene):toBeNil()
            expect(chudan.ran):toBe(false)
        end)
    end)
end)

-- ============================================================================
-- 続きの決まり: コールバック待ち（2.5・2.6）
-- ============================================================================

describe("続きの決まり - コールバック待ち", function()
    --- 待っているコールバックのイベント名を 1 つ返す
    local function waiting_event()
        return (next(CALLBACK.pending))
    end

    test("台詞を出さずに待ちに入ったら続きを残し、戻った回に台詞を出せば捨てる", function()
        with_env({}, function()
            local co = start_chain()
            interrupt(function(act)
                act:get_property("name")
                talk(act, "RETURNED")
            end)
            expect(STORE.co_scene):toBe(co)
            expect(STORE.co_callback):toBeNil()
            local event = waiting_event()
            expect(event):toBe("OnPastaCallBack1")

            local res = request(event, { [0] = "v1" })
            expect(has(res, "RETURNED")):toBe(true)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("戻った回にも台詞を出さなければ続きは残り、後半が出る", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            interrupt(function(act)
                act:get_property("name")
                talk(act, "\\s[5]")
            end)
            expect(STORE.co_scene):toBe(co)
            request(waiting_event(), { [0] = "v1" })
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
            expect_second_half()
        end)
    end)
end)

-- ============================================================================
-- 続きの決まり: 素のコルーチン（1.7）
-- ============================================================================

describe("続きの決まり - 素のコルーチン", function()
    test("空でない文字列を出せば続きを捨てる", function()
        with_env({}, function()
            start_chain()
            REG.OnCtiRaw = function(_act)
                return coroutine.create(function() return "RAW\\e" end)
            end
            local res = request("OnCtiRaw")
            expect(has(res, "RAW")):toBe(true)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("何も出さない・空の文字列なら続きを残す", function()
        with_env({}, function()
            local co = start_chain()
            REG.OnCtiRaw = function(_act)
                return coroutine.create(function() end)
            end
            request("OnCtiRaw")
            expect(STORE.co_scene):toBe(co)
            REG.OnCtiRaw = function(_act)
                return coroutine.create(function() return "" end)
            end
            request("OnCtiRaw")
            expect(STORE.co_scene):toBe(co)
            expect_second_half()
        end)
    end)
end)

-- ============================================================================
-- 続きの決まり: 外から起こした OnTalk（6.1・1.6）
-- REG.OnTalk は無いので、シーンの検索（act:find_scene）で ＊OnTalk が引かれる
-- ============================================================================

describe("続きの決まり - 外から起こした OnTalk", function()
    test("台詞を出さない OnTalk は続きを残し、台詞を出す OnTalk は捨てる", function()
        with_env({ names = { OnTalk = "cti_ontalk_1" } }, function()
            local co = start_chain()
            SCENE.register("cti_ontalk_1", "__start__", function(act) talk(act, "\\s[5]") end)
            local res = request("OnTalk")
            expect(has(res, "\\s[5]")):toBe(true)
            expect(STORE.co_scene):toBe(co)

            SCENE.register("cti_ontalk_1", "__start__", function(act) talk(act, "NEWTALK") end)
            res = request("OnTalk")
            expect(has(res, "NEWTALK")):toBe(true)
            expect(STORE.co_scene):toBeNil()
        end)
    end)
end)

describe("続きの決まり - 外から呼んだ OnPastaCallFunction", function()
    test("台詞を出さない呼び先は続きを残し、台詞を出す呼び先は捨てて中断の台詞を付ける", function()
        with_env({ chudan = true, names = { cti_call = "cti_call_1" } }, function()
            local co = start_chain()
            SCENE.register("cti_call_1", "__start__", function(act) talk(act, "\\s[5]") end)
            request("OnPastaCallFunction", { [0] = "cti_call" })
            expect(chudan.ran):toBe(false)
            expect(STORE.co_scene):toBe(co)

            SCENE.register("cti_call_1", "__start__", function(act) talk(act, "CALLED") end)
            local res = request("OnPastaCallFunction", { [0] = "cti_call" })
            expect(chudan.ran):toBe(true)
            expect(has(res, "CHUDAN")):toBe(true)
            expect(has(res, "CALLED")):toBe(false)
            expect(STORE.co_scene):toBeNil()
        end)
    end)
end)

-- ============================================================================
-- 中断の台詞（3.1〜3.10・3.12・2.4・7.1）
-- 中断のシーンの中身を変えるときは、同じ登録名 cti_chudan_1 に登録し直す
-- ============================================================================

--- 中断のシーンの中身を fn に差し替える（動いたかは ran に記録する）
local function set_chudan(fn)
    SCENE.register("cti_chudan_1", "__start__", function(act)
        chudan.ran = true
        fn(act)
    end)
end

describe("中断の台詞 - 中断があるとき", function()
    test("応答は中断の台詞だけの 1 本で、反応は出ず、続きは残らない", function()
        with_env({ chudan = true }, function()
            start_chain()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(chudan.ran):toBe(true)
            expect(has(res, "CHUDAN")):toBe(true)
            expect(has(res, "REACTION")):toBe(false)
            expect(has(res, "KOUHAN")):toBe(false)
            -- 1 本のさくらスクリプト（\e で終わる 1 本だけ）
            local value = header_value(res, "Value") or ""
            local _, ends = value:gsub("\\e", "")
            expect(ends):toBe(1)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("割り込んだシーンの表情は応答に含まれず、次の応答にも持ち越されない", function()
        with_env({ chudan = true }, function()
            start_chain()
            local res = interrupt(function(act)
                -- 表情の単語はアクターの台詞として \s[5] を積む
                talk(act, "\\s[5]")
                talk(act, "REACTION")
            end)
            expect(has(res, "CHUDAN")):toBe(true)
            expect(has(res, "\\s[5]")):toBe(false)
            -- 外見の状態（次の応答の復旧の元）に、出さなかった \s[5] が記録されていない
            local rec = STORE.appearance.actors["さくら"]
            expect(tostring(rec and rec.surface)):toBe("nil")
            local spot0 = STORE.appearance.spots[0]
            expect(tostring(spot0 and spot0.surface)):toBe("nil")
        end)
    end)

    test("割り込んだシーン自身が区切っても、その残りは続きにならない", function()
        with_env({ chudan = true }, function()
            start_chain()
            local res = interrupt(function(act)
                talk(act, "NEWFIRST")
                act:yield()
                talk(act, "NEWREST")
            end)
            expect(has(res, "CHUDAN")):toBe(true)
            expect(has(res, "NEWFIRST")):toBe(false)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("中断が区切ると、頭の空の区切りを飛ばし最初に出力のある区切りまでだけが付き、続きは残らない", function()
        with_env({ chudan = true }, function()
            set_chudan(function(act)
                act:yield()
                talk(act, "CHUDANA")
                act:yield()
                talk(act, "CHUDANB")
            end)
            start_chain()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(has(res, "CHUDANA")):toBe(true)
            expect(has(res, "CHUDANB")):toBe(false)
            expect(has(res, "REACTION")):toBe(false)
            expect(STORE.co_scene):toBeNil()
        end)
    end)
end)

describe("中断の台詞 - 今と同じ応答になるとき", function()
    test("中断が無ければ反応が出て、続きは捨てられる", function()
        with_env({}, function()
            start_chain()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(has(res, "REACTION")):toBe(true)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("中断の出力が空なら反応が出る", function()
        with_env({ chudan = true }, function()
            set_chudan(function() end)
            start_chain()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(chudan.ran):toBe(true)
            expect(has(res, "REACTION")):toBe(true)
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("中断の出力が空で割り込んだシーンが区切れば、その残りが続きになる", function()
        with_env({ chudan = true }, function()
            set_chudan(function() end)
            local old = start_chain()
            local res = interrupt(function(act)
                talk(act, "NEWFIRST")
                act:yield()
                talk(act, "NEWREST")
            end)
            expect(has(res, "NEWFIRST")):toBe(true)
            expect(type(STORE.co_scene)):toBe("thread")
            expect(STORE.co_scene ~= old):toBe(true)
            expect(has(request("OnCtiContinue"), "NEWREST")):toBe(true)
        end)
    end)

    test("＊チェイントーク中断ごっこ だけなら中断の台詞は付かない", function()
        with_env({ names = { ["チェイントーク中断ごっこ"] = "cti_gokko_1" } }, function()
            local gokko = false
            SCENE.register("cti_gokko_1", "__start__", function(act)
                gokko = true
                talk(act, "GOKKO")
            end)
            start_chain()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(has(res, "REACTION")):toBe(true)
            expect(has(res, "GOKKO")):toBe(false)
            expect(gokko):toBe(false)
        end)
    end)

    test("中断が待ちに入ると中断の台詞は出ず反応が出て、待ちも予約も残らない", function()
        with_env({ chudan = true }, function()
            set_chudan(function(act)
                act:get_property("name")
                talk(act, "CHUDAN")
            end)
            start_chain()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(chudan.ran):toBe(true)
            expect(has(res, "REACTION")):toBe(true)
            expect(has(res, "CHUDAN")):toBe(false)
            expect(has(res, "\\![get,property")):toBe(false)
            expect(next(CALLBACK.pending)):toBeNil()
            expect(CALLBACK.has_staged()):toBe(false)
            expect(STORE.co_callback):toBeNil()
            expect(STORE.co_scene):toBeNil()
        end)
    end)

    test("続きが無ければ、台詞を出すシーンでも中断を実行しない", function()
        with_env({ chudan = true }, function()
            local res = interrupt(function(act) talk(act, "REACTION") end)
            expect(has(res, "REACTION")):toBe(true)
            expect(chudan.ran):toBe(false)
        end)
    end)
end)

describe("中断の台詞 - 中断がエラーで止まったとき", function()
    test("止まった場所が ＊チェイントーク中断 の止まるエラーの応答になり、続きは捨てられる", function()
        with_env({ chudan = true }, function()
            set_chudan(function(act)
                act:init_scene({ __global_name__ = "cti_chudan_1" }, "チェイントーク中断")
                error("cti chudan boom")
            end)
            start_chain()
            local res = interrupt(function(act)
                act:init_scene({ __global_name__ = "cti_int_1" }, "割り込み")
                talk(act, "REACTION")
            end)
            expect(has(header_value(res, "X-Error-Reason") or "", "cti chudan boom")):toBe(true)
            expect(has(res, "＊チェイントーク中断")):toBe(true)
            expect(has(res, "＊割り込み")):toBe(false)
            expect(has(res, "REACTION")):toBe(false)
            expect(STORE.co_scene):toBeNil()
        end)
    end)
end)

-- ============================================================================
-- 再生しなかった指名の再生は続きに触れない（4.4・4.5）
-- ============================================================================

describe("続きの決まり - 再生しなかった指名の再生", function()
    test("OnPastaScene が見つけられなければ続きはそのまま", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            request("OnPastaScene", { [0] = "cti_missing" })
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
        end)
    end)

    test("キックのシーンが引き当てられなければ続きはそのまま", function()
        with_env({ chudan = true }, function()
            local co = start_chain()
            REG.OnCtiKickMiss = function(act)
                STORE.kick_pending = "cti_kick_missing"
                return KICK.try_dispatch(act)
            end
            request("OnCtiKickMiss")
            expect(STORE.co_scene):toBe(co)
            expect(chudan.ran):toBe(false)
        end)
    end)
end)
