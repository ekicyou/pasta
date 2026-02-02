local PASTA = require "pasta"

PASTA.create_word("雑談"):entry("何か用？", "暇だなあ...", "ねえねえ", "聞いてる？", "うーん", "眠くなってきた...")
do
    local SCENE = PASTA.create_scene("OnTalk")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("通常"))
        act.女の子:talk(act.女の子:word("雑談"))
    end
end

do
    local SCENE = PASTA.create_scene("OnTalk")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk("Pasta DSL、使ってみてね！")
        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("Lua 側も触ってみなよ。")
    end
end

do
    local SCENE = PASTA.create_scene("OnTalk")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("考え"))
        act.女の子:talk("今日は何しようかな...")
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("宿題やったの？")
    end
end

do
    local SCENE = PASTA.create_scene("OnTalk")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("通常"))
        act.女の子:talk("ねえ、今日の天気どう思う？")
        act.男の子:talk(act.男の子:word("考え"))
        act.男の子:talk("さあ、外見てないからわかんないや。")
    end
end

do
    local SCENE = PASTA.create_scene("OnTalk")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk("一緒にお話しよう！")
        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("しょうがないなあ。")
    end
end

do
    local SCENE = PASTA.create_scene("OnTalk")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("眠い"))
        act.女の子:talk("ふわあ...ちょっと眠いかも。")
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("寝てていいよ、ぼくが見てるから。")
    end
end

do
    local SCENE = PASTA.create_scene("OnHour")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("笑顔"))
        act.女の子:talk(tostring(var.時１２))
        act.女の子:talk("だよ！時報だよ～。")
        act.男の子:talk(act.男の子:word("元気"))
        act.男の子:talk("もう　")
        act.男の子:talk(tostring(var.時１２))
        act.男の子:talk("か、早いね。")
    end
end

do
    local SCENE = PASTA.create_scene("OnHour")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("通常"))
        act.女の子:talk("今　")
        act.女の子:talk(tostring(var.時１２))
        act.女の子:talk("だって。")
        act.男の子:talk(act.男の子:word("通常"))
        act.男の子:talk("へえ、そうなんだ。")
    end
end

do
    local SCENE = PASTA.create_scene("OnHour")

    function SCENE.__start__(act, ...)
        local args = { ... }
        local save, var = act:init_scene(SCENE)

        act.女の子:talk(act.女の子:word("考え"))
        act.女の子:talk(tostring(var.時１２))
        act.女の子:talk("...時間が経つのって不思議だね。")
        act.男の子:talk(act.男の子:word("考え"))
        act.男の子:talk("哲学的だね。")
    end
end
