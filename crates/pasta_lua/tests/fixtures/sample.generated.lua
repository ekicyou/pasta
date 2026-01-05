local PASTA = require "pasta"

do
    local ACTOR = PASTA.create_actor("さくら")
    ACTOR.通常 = [=[\s[0]]=]
    ACTOR.照れ = [=[\s[1]]=]
    ACTOR.驚き = [=[\s[2]]=]
    ACTOR.ぐんにょり = [=[\s[3]]=]
    ACTOR.怒り = [=[\s[4]]=]
end

do
    local ACTOR = PASTA.create_actor("うにゅう")
    ACTOR.通常 = [=[\s[10]]=]
    ACTOR.刮目 = [=[\s[11]]=]
end

do
    local SCENE = PASTA.create_scene("メイン1")

    function SCENE.__start__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:set_spot(0)
        act.うにゅう:set_spot(1)

        act:call("メイン1", "グローバル単語呼び出し", {}, table.unpack(args))
        act:call("メイン1", "ローカル単語呼び出し", {}, table.unpack(args))
        act:call("メイン1", "会話分岐", {}, table.unpack(args))
        act:call("メイン1", "変数代入", {}, table.unpack(args))
        act:call("メイン1", "引数付き呼び出し", {}, var.カウンタ, save.グローバル, table.unpack(args))
    end

    function SCENE.__グローバル単語呼び出し_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:word("笑顔")
        act.さくら:word("挨拶")
        act.さくら:talk("！")
        act.うにゅう:word("通常")
        act.うにゅう:talk("やふぅ。")
    end

    function SCENE.__ローカル単語呼び出し_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:word("通常")
        act.さくら:word("場所")
        act.さくら:talk("の天気は？")
        act.うにゅう:word("天気")
        act.うにゅう:talk("らしいで。")
    end

    function SCENE.__会話分岐_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:talk("ローカル分岐１だよ。")
        act.うにゅう:talk("ちっぽけやね。")
    end

    function SCENE.__会話分岐_2__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:talk("ローカル分岐２だよ。")
        act.うにゅう:talk("もっと飛べる、ワイは飛べるんや！")
        act.さくら:word("ぐんにょり")
        act.さくら:talk("なんでだよ。")
    end

    function SCENE.__変数代入_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:word("通常")
        act.さくら:talk("変数を代入。")
        act.うにゅう:talk("中身は内緒や。")
        var.カウンタ = 10
        save.グローバル = SCENE.関数(ctx, 2 + 1)
        var.場所 = act:word("場所")
    end

    function SCENE.__引数付き呼び出し_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:talk("第１引数は")
        act.さくら:talk(tostring(args[1]))
        act.さくら:talk("だよ。")
        act.うにゅう:talk("第２引数は")
        act.うにゅう:talk(tostring(args[2]))
        act.うにゅう:talk("やね。")
    end

    function SCENE.関数(ctx, value, ...)
        return value * value
    end
end

do
    local SCENE = PASTA.create_scene("会話分岐1")

    function SCENE.__start__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA.create_session(SCENE, ctx)

        act.さくら:talk("グローバルの分岐に飛んできた。")
        act.うにゅう:talk("世界取れるで。")
    end

end

