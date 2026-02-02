local PASTA = require "pasta"

do
    local ACTOR = PASTA.create_actor("女の子")
    ACTOR:create_word("笑顔"):entry([=[\s[0]]=])
    ACTOR:create_word("通常"):entry([=[\s[1]]=])
    ACTOR:create_word("照れ"):entry([=[\s[2]]=])
    ACTOR:create_word("驚き"):entry([=[\s[3]]=])
    ACTOR:create_word("泣き"):entry([=[\s[4]]=])
    ACTOR:create_word("困惑"):entry([=[\s[5]]=])
    ACTOR:create_word("キラキラ"):entry([=[\s[6]]=])
    ACTOR:create_word("眠い"):entry([=[\s[7]]=])
    ACTOR:create_word("怒り"):entry([=[\s[8]]=])
end

do
    local ACTOR = PASTA.create_actor("男の子")
    ACTOR:create_word("笑顔"):entry([=[\s[10]]=])
    ACTOR:create_word("通常"):entry([=[\s[11]]=])
    ACTOR:create_word("照れ"):entry([=[\s[12]]=])
    ACTOR:create_word("驚き"):entry([=[\s[13]]=])
    ACTOR:create_word("泣き"):entry([=[\s[14]]=])
    ACTOR:create_word("困惑"):entry([=[\s[15]]=])
    ACTOR:create_word("キラキラ"):entry([=[\s[16]]=])
    ACTOR:create_word("眠い"):entry([=[\s[17]]=])
    ACTOR:create_word("怒り"):entry([=[\s[18]]=])
end
