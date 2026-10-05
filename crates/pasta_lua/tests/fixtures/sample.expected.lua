local PASTA = require "pasta"
local GLOBAL = require "pasta.global"

do
    local ACTOR = PASTA.create_actor("さくら")
    ACTOR:create_word("通常"):entry([=[\s[0]]=], [=[\s[100]]=])
    ACTOR:create_word("照れ"):entry([=[\s[1]]=])
    ACTOR:create_word("驚き"):entry([=[\s[2]]=])
    ACTOR:create_word("ぐんにょり"):entry([=[\s[3]]=])
    ACTOR:create_word("怒り"):entry([=[\s[4]]=])
end

do
    local ACTOR = PASTA.create_actor("うにゅう")
    ACTOR:create_word("通常"):entry([=[\s[10]]=])
    ACTOR:create_word("刮目"):entry([=[\s[11]]=])
end

PASTA.create_word("挨拶"):entry("こんにちは", "やあ", "ハロー")
do
    local SCENE = PASTA.create_scene("メイン")

    SCENE:create_word("場所"):entry("東京", "大阪", "京都")
    SCENE:create_word("天気"):entry("晴れ", "曇り", "雨")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)
        act:clear_spot()
        act:set_spot("さくら", 0)
        act:set_spot("うにゅう", 1)

        act:call_restore(SCENE.__global_name__, "グローバル単語呼び出し", {}, table.unpack(args))
        act:call_restore(SCENE.__global_name__, "ローカル単語呼び出し", {}, table.unpack(args))
        act:call_restore(SCENE.__global_name__, "会話分岐", {}, table.unpack(args))
        act:call_restore(SCENE.__global_name__, "変数代入", {}, table.unpack(args))
        act:call_restore(SCENE.__global_name__, "引数付き呼び出し", {}, var.カウンタ, save.グローバル, table.unpack(args))
        act:call_restore(SCENE.__global_name__, "グローバル関数呼び出し", {}, table.unpack(args))
        return act:call(SCENE.__global_name__, "共有プロパティ操作", {}, table.unpack(args))
    end

    function SCENE.グローバル単語呼び出し_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("笑顔"))
        act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("挨拶"))
        act:actor_proxy("さくら"):talk("！")
        act:actor_proxy("うにゅう"):talk(act:actor_proxy("うにゅう"):word("通常"))
        act:actor_proxy("うにゅう"):talk("やふぅ。")
    end

    function SCENE.ローカル単語呼び出し_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("通常"))
        act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("場所"))
        act:actor_proxy("さくら"):talk("の天気は？")
        act:actor_proxy("うにゅう"):talk(act:actor_proxy("うにゅう"):word("天気"))
        act:actor_proxy("うにゅう"):talk("らしいで。")
    end

    function SCENE.会話分岐_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk("ローカル分岐１だよ。")
        act:actor_proxy("うにゅう"):talk("ちっぽけやね。")
    end

    function SCENE.会話分岐_2(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk("ローカル分岐２だよ。")
        act:actor_proxy("うにゅう"):talk("もっと飛べる、ワイは飛べるんや！")
        act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("ぐんにょり"))
        act:actor_proxy("さくら"):talk("なんでだよ。")
    end

    function SCENE.変数代入_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("通常"))
        act:actor_proxy("さくら"):talk("変数を代入。")
        act:actor_proxy("うにゅう"):talk("中身は内緒や。")
        var.カウンタ = 10
        save.グローバル = act:expr_fn("関数", act:arith("+", 2, 1))
        var.場所 = act:word("場所")
    end

    function SCENE.引数付き呼び出し_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk("第１引数は")
        act:actor_proxy("さくら"):talk(args[1], "args[1]")
        act:actor_proxy("さくら"):talk("だよ。")
        act:actor_proxy("うにゅう"):talk("第２引数は")
        act:actor_proxy("うにゅう"):talk(args[2], "args[2]")
        act:actor_proxy("うにゅう"):talk("やね。")
    end

    function SCENE.グローバル関数呼び出し_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk((act:global_fn("グローバル関数", "グローバル")))
        act:actor_proxy("さくら"):talk("　を呼んだよ。")
        var.結果 = act:global_fn("グローバル関数", "代入テスト")
        act:global_fn("グローバル関数", "式文テスト")
        act:expr_fn("関数", 42)
        act:actor_proxy("うにゅう"):talk("ローカルもいけるで")
        act:actor_proxy("うにゅう"):talk((act:actor_proxy("うにゅう"):expr_fn("関数", 1)))
    end

    function SCENE.共有プロパティ操作_1(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:set_property("system.name", "テストゴースト")
        var.ゴースト名 = act:get_property("currentghost.name")
        save.幅 = act:get_property("currentghost.balloon.scope(0).validwidth.initial")
        act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("通常"))
        act:actor_proxy("さくら"):talk("名前は")
        act:actor_proxy("さくら"):talk(tostring(act:get_property("currentghost.name")))
        act:actor_proxy("さくら"):talk("です。")
        act:actor_proxy("うにゅう"):talk("幅は")
        act:actor_proxy("うにゅう"):talk(var.ゴースト名, "var.ゴースト名")
        act:actor_proxy("うにゅう"):talk("だって。")
    end
end

do
    local SCENE = PASTA.create_scene("会話分岐")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act:actor_proxy("さくら"):talk("グローバルの分岐に飛んできた。")
        act:actor_proxy("うにゅう"):talk("世界取れるで。")
    end
end
