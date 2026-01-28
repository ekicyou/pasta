local PASTA = require "pasta"

do
    local ACTOR = PASTA.create_actor("さくら")
    ACTOR:create_word("通常"):entry([=[\s[0]]=])
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

        act:call(SCENE.__global_name__, "グローバル単語呼び出し", {}, table.unpack(args))
        act:call(SCENE.__global_name__, "ローカル単語呼び出し", {}, table.unpack(args))
        act:call(SCENE.__global_name__, "会話分岐", {}, table.unpack(args))
        act:call(SCENE.__global_name__, "変数代入", {}, table.unpack(args))
        return act:call(SCENE.__global_name__, "引数付き呼び出し", {}, var.カウンタ, save.グローバル, table.unpack(args))
    end

    function SCENE.__グローバル単語呼び出し_1__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk(act.さくら:word("笑顔"))
        act.さくら:talk(act.さくら:word("挨拶"))
        act.さくら:talk("！")
        act.うにゅう:talk(act.うにゅう:word("通常"))
        act.うにゅう:talk("やふぅ。")
    end

    function SCENE.__ローカル単語呼び出し_1__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk(act.さくら:word("通常"))
        act.さくら:talk(act.さくら:word("場所"))
        act.さくら:talk("の天気は？")
        act.うにゅう:talk(act.うにゅう:word("天気"))
        act.うにゅう:talk("らしいで。")
    end

    function SCENE.__会話分岐_1__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk("ローカル分岐１だよ。")
        act.うにゅう:talk("ちっぽけやね。")
    end

    function SCENE.__会話分岐_2__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk("ローカル分岐２だよ。")
        act.うにゅう:talk("もっと飛べる、ワイは飛べるんや！")
        act.さくら:talk(act.さくら:word("ぐんにょり"))
        act.さくら:talk("なんでだよ。")
    end

    function SCENE.__変数代入_1__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk(act.さくら:word("通常"))
        act.さくら:talk("変数を代入。")
        act.うにゅう:talk("中身は内緒や。")
        var.カウンタ = 10
        save.グローバル = SCENE.関数(act, 2 + 1)
        var.場所 = act:word("場所")
    end

    function SCENE.__引数付き呼び出し_1__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk("第１引数は")
        act.さくら:talk(tostring(args[1]))
        act.さくら:talk("だよ。")
        act.うにゅう:talk("第２引数は")
        act.うにゅう:talk(tostring(args[2]))
        act.うにゅう:talk("やね。")
    end

    function SCENE.関数(act, value, ...)
        return value * value
    end
end

do
    local SCENE = PASTA.create_scene("会話分岐")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.さくら:talk("グローバルの分岐に飛んできた。")
        act.うにゅう:talk("世界取れるで。")
    end
end
