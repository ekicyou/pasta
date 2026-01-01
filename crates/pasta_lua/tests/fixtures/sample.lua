-- sample.lua - sample.pastaからのトランスパイル出力例
-- 目的: pasta-lua-specification の要件定義に基づくトランスパイル結果の参照実装
--
-- 設計その他:
-- - do...end: Lua をスコープ分割してローカル変数枠を管理
-- - var, save, act: 3個テーブルで変数管理（ローカル変数数制限対策）
-- - act:call(): グローバルシーン呼び出し形式

local PASTA = require "pasta.runtime"

-- ####################################################################
-- ＃アクター辞書
-- ####################################################################

-- ％さくら
-- 意図: ACTOR変数を再利用してローカル変数数を抑制（Requirement 1）
do
    local ACTOR = PASTA:create_actor("さくら")
    -- 　＄通常　　　：\s[0]
    -- 意図: \s[0] には [ が含まれるため、n=1 の [=[...]=] 形式を使用（Requirement 2）
    --       危険パターン判定: "]" が含まれるため n=0 ([[...]]) は不可
    ACTOR.通常 = [=[\s[0]]=]
    -- 　＄照れ　　　：\s[1]
    ACTOR.照れ = [=[\s[1]]=]
    -- 　＄驚き　　　：\s[2]
    ACTOR.驚き = [=[\s[2]]=]
    -- 　＄ぐんにょり：\s[3]
    ACTOR.ぐんにょり = [=[\s[3]]=]
    -- 　＄怒り　　　：\s[4]
    ACTOR.怒り = [=[\s[4]]=]
end

-- ％うにゅう
-- 意図: 同一ACTOR変数を再利用（Requirement 1）
do
    local ACTOR = PASTA:create_actor("うにゅう")
    -- 　＄通常　：\s[10]
    ACTOR.通常 = [=[\s[10]]=]
    -- 　＄刮目　：\s[11]
    ACTOR.刮目 = [=[\s[11]]=]
end

-- ####################################################################
-- ＃ファイルレベル属性
-- ####################################################################

-- ＆天気：晴れ
-- 意図: ファイルレベル属性はAttributeRegistryに記録、Lua出力なし（Requirement 3b）

-- ####################################################################
-- ＃ グローバル単語定義（トップレベル）
-- ####################################################################

-- ＠挨拶：こんにちは、やあ、ハロー
-- 意図: グローバル単語定義はWordDefRegistryに登録、Lua出力なし（Requirement 3e）

-- ####################################################################
-- ＊メイン
-- ####################################################################

-- 意図: SCENE変数を再利用してシーン定義（Requirement 1）
--       "メイン1" はSceneRegistryで一意なモジュール名として解決
do
    local SCENE = PASTA:create_scene("メイン1")

    -- 　＃ ローカル単語定義
    -- 　＠場所：東京、大阪、京都
    -- 　＠天気：晴れ、曇り、雨
    -- 意図: ローカル単語定義はローカルスコープのWordDefRegistryに登録、Lua出力なし（Requirement 3e）

    -- __start__ - グローバルシーンのエントリーポイント
    -- 意図: グローバルシーン（＊メイン）のエントリーポイントは常に __start__（Requirement 3c）
    --       関数シグネチャ: (ctx, ...)
    --       . 構文で宣言、SCENE はセッション初期化の第1引数で明示的に渡す
    function SCENE.__start__(ctx, ...)
        -- 意図: 第1行で引数をテーブル化、第2行でセッション初期化（Requirement 3c）
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　＞グローバル単語呼び出し
        -- 意図: Call文は act:call(モジュール名, ラベル名, 属性フィルター, ...引数) 形式（Requirement 3d）
        --       第3引数の {} は属性フィルター用の空テーブル（将来拡張用に予約）
        --       table.unpack(args) で受け取った引数を継承
        act:call("メイン1", "グローバル単語呼び出し", {}, table.unpack(args))

        -- 　　　＞ローカル単語呼び出し
        act:call("メイン1", "ローカル単語呼び出し", {}, table.unpack(args))

        -- 　　　＞会話分岐
        act:call("メイン1", "会話分岐", {}, table.unpack(args))

        -- 　　　＞変数代入
        act:call("メイン1", "変数代入", {}, table.unpack(args))

        -- 　　　＞引数付き呼び出し（＄カウンタ、＄＊グローバル）
        -- 意図: 引数付きCall文では変数展開後、残りの引数を ... で継承（Requirement 3d）
        act:call("メイン1", "引数付き呼び出し", {}, var.カウンタ, save.グローバル, table.unpack(args))
    end

    -- 　・グローバル単語呼び出し
    -- 意図: 第1階層ローカルシーンは __ローカルシーン名_N__ 形式で常にカウンター付与（Requirement 3c）
    --       N=1,2,3... で各ローカルシーン定義順に採番。Rune実装と同一パターン
    function SCENE.__グローバル単語呼び出し_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：＠笑顔　＠挨拶！
        -- 意図: ＠XXX 参照は word("XXX") に展開（Requirement 3e）
        --       通常テキストは talk() に展開（Requirement 3d）
        act.さくら:word("笑顔")
        act.さくら:word("挨拶")
        act.さくら:talk("！")

        -- 　　　うにゅう：＠通常　やふぅ。
        act.うにゅう:word("通常")
        act.うにゅう:talk("やふぅ。")
    end

    -- 　・ローカル単語呼び出し
    function SCENE.__ローカル単語呼び出し_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：＠通常　＠場所　の天気は？
        -- 意図: ＠XXX 参照と通常テキストの混在を word() と talk() に分割（Requirement 3e, 3d）
        act.さくら:word("通常")
        act.さくら:word("場所")
        act.さくら:talk("の天気は？")

        -- 　　　うにゅう：＠天気　らしいで。
        act.うにゅう:word("天気")
        act.うにゅう:talk("らしいで。")
    end

    -- 　・会話分岐
    -- 意図: ローカルシーン定義順に採番。最初の「会話分岐」は_1（Requirement 3c、Rune実装と同一）
    function SCENE.__会話分岐_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：ローカル分岐１だよ。
        act.さくら:talk("ローカル分岐１だよ。")

        -- 　　　うにゅう：ちっぽけやね。
        act.うにゅう:talk("ちっぽけやね。")
    end

    -- 　・会話分岐
    -- 意図: ローカルシーン定義順に採番。重複「会話分岐」は_2
    function SCENE.__会話分岐_2__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：ローカル分岐２だよ。
        act.さくら:talk("ローカル分岐２だよ。")

        -- 　　　うにゅう：もっと飛べる、ワイは飛べるんや！
        act.うにゅう:talk("もっと飛べる、ワイは飛べるんや！")

        -- 　　　さくら　：＠ぐんにょり　なんでだよ。
        -- 意図: ＠XXX 参照は word() に展開（Requirement 3e）
        act.さくら:word("ぐんにょり")
        act.さくら:talk("なんでだよ。")
    end

    -- 　・変数代入
    function SCENE.__変数代入_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：＠通常　変数を代入。
        act.さくら:word("通常")
        act.さくら:talk("変数を代入。")

        -- 　　　うにゅう：中身は内緒や。
        act.うにゅう:talk("中身は内緒や。")

        -- 　　　＄カウンタ＝１０
        -- 意図: ローカル変数（＄変数名）は var.変数名 に代入（Requirement 3d）
        var.カウンタ = 10

        -- 　　　＄＊グローバル＝＠関数（２+１）
        -- 意図: グローバル変数（＄＊変数名）は save.変数名 に代入（Requirement 3d）
        --       関数呼び出し（＠関数）は SCENE.関数(ctx, 引数...) 形式で宣言、呼び出しは : で実行（Requirement 3d）
        save.グローバル = SCENE:関数(ctx, 2 + 1)

        -- 　　　＄場所＝＠場所
        -- 意図: WordRef代入（＄変数＝＠単語）は var.変数名 = act:word("単語名") 形式で出力（Requirement 3d）
        var.場所 = act:word("場所")
    end

    -- 　・引数付き呼び出し
    function SCENE.__引数付き呼び出し_1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：第１引数は＄０　だよ。
        -- 意図: 引数参照（＄０→args[1]、＄１→args[2]）Pasta DSLの0-baseをLua 1-baseに変換（Requirement 3d）
        --       各アクション要素は個別のtalk/tostring呼び出しに展開（Requirement 3d）
        act.さくら:talk("第１引数は")
        act.さくら:talk(tostring(args[1]))
        act.さくら:talk("だよ。")

        -- 　　　うにゅう：第２引数は＄１　やね。
        act.うにゅう:talk("第２引数は")
        act.うにゅう:talk(tostring(args[2]))
        act.うにゅう:talk("やね。")
    end

    -- ```
    -- function SCENE.関数(ctx, value, ...)
    --     return value * value
    -- end
    -- ```
    -- 意図: コードブロック（``` で識別、言語識別子は無視）内のコードをそのまま出力（Requirement 3f）
    function SCENE.関数(ctx, value, ...)
        return value * value
    end
end


-- ####################################################################
-- ＊会話分岐
-- ####################################################################

-- 意図: 別のグローバルシーン定義（SCENE変数を再利用）（Requirement 1）
--       「＊メイン」にローカルシーン「会話分岐」が存在するため、
--       グローバルシーン「＊会話分岐」は一意なモジュール名「会話分岐1」として生成（Requirement 3b）
do
    local SCENE = PASTA:create_scene("会話分岐1")

    function SCENE.__start__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　さくら　：グローバルの分岐に飛んできた。
        act.さくら:talk("グローバルの分岐に飛んできた。")

        -- 　　うにゅう：世界取れるで。
        act.うにゅう:talk("世界取れるで。")
    end
end

-- eof
