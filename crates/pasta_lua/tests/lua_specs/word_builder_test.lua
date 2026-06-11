-- pasta.word（WordBuilder / resolve_value）単体テスト
-- review-improvement-loop cell 3.46 (G1): 既存 actor_word_test が触れていない
-- ビルダー境界（空 entry・複数 entry・チェーン）・create_local・エイリアス・
-- resolve_value の全分岐を固定する
--
-- STORE.*_words はスイート間で共有されるため一意プレフィックス "spec346_" を使用する。
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- 先行スイートのモジュール個別リロードによる STORE 参照分裂を避けるため、
-- 整合した同一インスタンス集合を一括で新規ロードする（既存スイートと同一規約）。
package.loaded["pasta.store"] = nil
package.loaded["pasta.word"] = nil

local WORD = require("pasta.word")
local STORE = require("pasta.store")

describe("WORD - WordBuilder.entry", function()
    test("ビルダー生成時にキーが空配列で初期化される", function()
        WORD.create_global("spec346_init_only")
        local entries = STORE.global_words["spec346_init_only"]
        expect(type(entries)):toBe("table")
        expect(#entries):toBe(0)
    end)

    test("引数なしの entry() は何も登録しない", function()
        WORD.create_global("spec346_empty_entry"):entry()
        expect(#STORE.global_words["spec346_empty_entry"]):toBe(0)
    end)

    test("entry の可変長引数が 1 つの値リストとして登録される", function()
        WORD.create_global("spec346_multi"):entry("りんご", "ばなな")
        local entries = STORE.global_words["spec346_multi"]
        expect(#entries):toBe(1)
        expect(entries[1][1]):toBe("りんご")
        expect(entries[1][2]):toBe("ばなな")
    end)

    test("複数回の entry は値リストを順に追加する（チェーン可）", function()
        local builder = WORD.create_global("spec346_chain")
        local ret = builder:entry("一"):entry("二", "三")
        expect(ret):toBe(builder)

        local entries = STORE.global_words["spec346_chain"]
        expect(#entries):toBe(2)
        expect(entries[1][1]):toBe("一")
        expect(entries[2][1]):toBe("二")
        expect(entries[2][2]):toBe("三")
    end)

    test("同一キーへの再ビルダー生成は既存エントリを保持する", function()
        WORD.create_global("spec346_append"):entry("既存")
        WORD.create_global("spec346_append"):entry("追加")
        local entries = STORE.global_words["spec346_append"]
        expect(#entries):toBe(2)
        expect(entries[1][1]):toBe("既存")
        expect(entries[2][1]):toBe("追加")
    end)
end)

describe("WORD - create_local / create_word エイリアス", function()
    test("create_local はシーン名スコープへ登録し、シーン間で独立する", function()
        WORD.create_local("spec346_sceneA", "天気"):entry("晴れ")
        WORD.create_local("spec346_sceneB", "天気"):entry("雨")

        expect(STORE.local_words["spec346_sceneA"]["天気"][1][1]):toBe("晴れ")
        expect(STORE.local_words["spec346_sceneB"]["天気"][1][1]):toBe("雨")
    end)

    test("get_local_words は登録済みシーンの辞書を返す", function()
        WORD.create_local("spec346_sceneC", "場所"):entry("東京")
        expect(WORD.get_local_words("spec346_sceneC")):toBe(STORE.local_words["spec346_sceneC"])
    end)

    test("create_word は create_global の別名としてグローバル辞書へ登録する", function()
        WORD.create_word("spec346_alias"):entry("値")
        expect(STORE.global_words["spec346_alias"][1][1]):toBe("値")
    end)
end)

describe("WORD.get_all_words", function()
    test("global / local / actor の 3 レジストリを STORE と同一参照で返す", function()
        local all = WORD.get_all_words()
        expect(all.global):toBe(STORE.global_words)
        expect(all["local"]):toBe(STORE.local_words)
        expect(all.actor):toBe(STORE.actor_words)
    end)
end)

describe("WORD.resolve_value", function()
    test("nil は nil を返す", function()
        expect(WORD.resolve_value(nil, nil)):toBeNil()
    end)

    test("関数は act を引数に実行し戻り値を返す", function()
        local fake_act = { marker = "spec346" }
        local received = nil
        local value = function(act)
            received = act
            return "関数結果"
        end
        expect(WORD.resolve_value(value, fake_act)):toBe("関数結果")
        expect(received):toBe(fake_act)
    end)

    test("非空テーブルは最初の要素を返す", function()
        expect(WORD.resolve_value({ "先頭", "次点" }, nil)):toBe("先頭")
    end)

    test("空テーブルは nil を返す", function()
        expect(WORD.resolve_value({}, nil)):toBeNil()
    end)

    test("数値は tostring で文字列化される", function()
        expect(WORD.resolve_value(42, nil)):toBe("42")
    end)

    test("false は tostring で 'false' になる（nil 扱いされない）", function()
        expect(WORD.resolve_value(false, nil)):toBe("false")
    end)
end)
