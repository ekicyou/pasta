local PASTA = require "pasta.dic"
local ACTOR, SCENE

-- ####################################################################
-- ＃アクター辞書
-- ####################################################################

--％さくら
ACTOR = PASTA:create_actor("さくら")
--　＄通常　：\s[0]
--　＄照れ　：\s[1]
--　＄驚き　：\s[2]
--　＄悲しみ：\s[3]
--　＄怒り　：\s[4]
--　＄試案　：\s[5]
--　＄笑顔　：\s[6]
--　＄目とじ：\s[7]
--　＄あきれ：\s[8]
--　＄にやそ：\s[9]
ACTOR.通常 = [==[\s[0]]==]
ACTOR.照れ = [==[\s[1]]==]
ACTOR.驚き = [==[\s[2]]==]
ACTOR.悲しみ = [==[\s[3]]==]
ACTOR.怒り = [==[\s[4]]==]
ACTOR.試案 = [==[\s[5]]==]
ACTOR.笑顔 = [==[\s[6]]==]
ACTOR.目とじ = [==[\s[7]]==]
ACTOR.あきれ = [==[\s[8]]==]
ACTOR.にやそ = [==[\s[9]]==]

--
--％うにゅう
ACTOR = PASTA:create_actor("うにゅう")
--　＄通常　：\s[10]
--　＄刮目　：\s[11]
ACTOR.通常 = [==[\s[10]]==]
ACTOR.刮目 = [==[\s[11]]==]


-- ####################################################################
-- ＃ファイルレベル属性
-- ####################################################################

--＆天気：晴れ
--⇒属性記録のためのテーブルに保存、Lua出力無し

--＃ グローバル単語定義（トップレベル）
--＠挨拶：こんにちは、やあ、ハロー
--⇒単語辞書はWordDefRegistryに登録されるため、Lua出力無し

--＊メイン
SCENE = PASTA:create_scene("メイン1")

--⇒SceneRegistryに登録されるため、Lua出力無し
--⇒ここで、グローバルシーンのモジュール名を「メイン1」と解決した

--　＃ ローカル単語定義
--　＠場所：東京、大阪、京都
--　＠天気：晴れ、曇り、雨
--⇒単語辞書はWordDefRegistryに登録されるため、Lua出力無し

-- ここから__start__関数
function SCENE.__start__(scene, ctx, ...)
    local args = { ... }
    local act, save, var = PASTA:create_session(scene, ctx)

    --＃ 変数代入
    --＄カウンタ＝１０
    var.カウンタ = 10
    --＄＊グローバル＝＠関数（２＋１）
    save.グローバル = scene.関数(2 + 1)

    --＃ 最初のローカルスコープ（__start__関数）
    --さくら　：＠挨拶！
    act.さくら:word("挨拶")
    act.さくら:talk("！")
    --うにゅう：やっほー。
    act.うにゅう:talk("やっほー。")

    --＃ Call文（引数なし）３つ目の引数は属性フィルターとして予約
    --＞自己紹介
    act:call("メイン1", "自己紹介", {}, table.unpack(args))

    --＃ Call文（引数あり）- 戻り後に変数を使用
    --＄カウンタ＝１
    var.カウンタ = 1
    --＞カウント表示（＄カウンタ）
    act:call("メイン1", "カウント表示", {}, var.カウンタ, ...)

    --＃ Call文で別のラベルへ遷移
    --＞会話分岐
    act:call("メイン1", "会話分岐", {})
end

--　-自己紹介
function SCENE.__自己紹介1__(scene, ctx, ...)
    local args = { ... }
    local act, save, var = PASTA:create_session(scene, ctx)

    --＃ Callラベル1: 自己紹介
    --さくら　：私はさくらです。
    act.さくら:talk("私はさくらです。")
    --うにゅう：僕はうにゅうだよ。
    act.うにゅう:talk("僕はうにゅうだよ。")

    --＃ ネストされたCall
    --＞趣味紹介
end

---趣味紹介
function SCENE.メイン1_趣味紹介1(scene, ctx, ...)
    local args = { ... }
    local act, save, var = PASTA:create_session(scene, ctx)

    --＃ Callラベル2: 趣味紹介（ネスト先）
    --さくら　：私の趣味は読書です。
    act.さくら:talk("私の趣味は読書です。")
    --うにゅう：僕はゲームが好き。
    act.うにゅう:talk("僕はゲームが好き。")
end

---カウント表示　＄値
function SCENE.メイン1_カウント表示1(scene, ctx, ...)
    local args = { ... }
    local act, save, var = PASTA:create_session(scene, ctx)

    --＃ Callラベル3: 引数を受け取る
    --さくら　：カウンターは＄０　です。
    act.さくら:talk("カウンターは" .. tostring(args[1]) .. "です。")
end

--・会話分岐
--＃ Callラベル1: 天気の話
--さくら　：＠場所の天気　は＠天気　だね。
--うにゅう：そうだね。

--＃ Call文で別ラベルへ遷移
--＞別の話題

--・別の話題
--＃ Callラベル2: 別の話
--さくら　：ところで、明日は何する？
--うにゅう：まだ決めてないなぁ。

--```lua
--function SCENE.関数(ctx, value, ...)
--    return value * value
--end
--```

function SCENE.関数(ctx, value, ...)
    return value * value
end
