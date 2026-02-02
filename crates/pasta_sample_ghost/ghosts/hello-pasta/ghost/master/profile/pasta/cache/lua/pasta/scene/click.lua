local PASTA = require "pasta"

do
    local SCENE = PASTA.create_scene("OnMouseDoubleClick")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("驚き"))
        act.女の子:talk("わっ、びっくりした！")
        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("どうしたの？")
    end
end

do
    local SCENE = PASTA.create_scene("OnMouseDoubleClick")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk("なあに？呼んだ？")
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("こっちに用があるんじゃない？")
    end
end

do
    local SCENE = PASTA.create_scene("OnMouseDoubleClick")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("照れ"))
        act.女の子:talk("え、なに？")
        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("照れてるの？")
    end
end

do
    local SCENE = PASTA.create_scene("OnMouseDoubleClick")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.男の子:talk(act.男の子:word("驚き"))
        act.男の子:talk("うわっ！なに！？")
        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk("反応してくれたね。")
    end
end

do
    local SCENE = PASTA.create_scene("OnMouseDoubleClick")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("怒り"))
        act.女の子:talk("もう、そんなにクリックしないで！")
        act.男の子:talk(act.男の子:word("驚き"))
        act.男の子:talk("お、怒った怒った。")
    end
end

do
    local SCENE = PASTA.create_scene("OnMouseDoubleClick")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk("わ〜い、遊んでくれるの？")
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("まあ、暇だしね。")
    end
end

do
    local SCENE = PASTA.create_scene("OnMouseDoubleClick")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("ふふん、ぼくのことが気になる？")
        act.女の子:talk(act.女の子:word("驚き"))
        act.女の子:talk("えっ？そんなんじゃないよ！")
    end
end
