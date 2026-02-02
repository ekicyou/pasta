local PASTA = require "pasta"

PASTA.create_word("起動挨拶"):entry("おはよう！今日もよろしくね！", "やっほー", "また会えたね！", "起動完了！準備OKだよ。")
PASTA.create_word("終了挨拶"):entry("またね～！", "お疲れ様！", "ばいばーい！")
do
    local SCENE = PASTA.create_scene("OnBoot")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk(act.女の子:word("起動挨拶"))
        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("へえ、また来たんだ。")
    end
end

do
    local SCENE = PASTA.create_scene("OnBoot")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("通常"))
        act.女の子:talk("起動したよ～。")
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("さあ、始めようか。")
    end
end

do
    local SCENE = PASTA.create_scene("OnFirstBoot")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk("初めまして！")
        act:sakura_script([[\n]])
        act.女の子:talk("わたしは女の子、よろしくね。")
        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("ぼくは男の子。ちゃんと使ってよね。")
    end
end

do
    local SCENE = PASTA.create_scene("OnClose")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("通常"))
        act.女の子:talk(act.女の子:word("終了挨拶"))
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("また呼んでよね。")
    end
end

do
    local SCENE = PASTA.create_scene("OnClose")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("眠い"))
        act.女の子:talk("おやすみなさい...")
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("じゃあね。")
    end
end
