-- sample.lua - sample.pastaからのトランスパイル出力例
-- 目的: pasta-lua-specification の要件定義に基づくトランスパイル結果の参照実装
--
-- トランスパイルルール:
--   - Requirement 0: ローカル変数数制限対策（ACTOR/SCENE変数再利用）
--   - Requirement 0-2: 文字列リテラル形式判定（エスケープ対象の有無で分岐）
--   - Requirement 1a: アクター定義のLua化
--   - Requirement 1b: シーン定義とモジュール構造
--   - Requirement 1c: ローカルシーン関数への変換（__start__, __名前__, モジュール_名前）
--   - Requirement 1d: 変数スコープ管理（var/save/act分離）
--   - Requirement 1f: Runeブロック埋め込み
--   - Requirement 1g: グローバルシーン間遷移

local PASTA = require "pasta.runtime"

-- ####################################################################
-- ＃アクター辞書
-- ####################################################################

-- ％さくら
-- 意図: ACTOR変数を再利用してローカル変数数を抑制（Requirement 0）
do
    local ACTOR = PASTA:create_actor("さくら")
    -- 　＄通常　　　：\s[0]
    -- 意図: \s[0] には [ が含まれるため、n=1 の [=[...]=] 形式を使用（Requirement 0-2）
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
-- 意図: 同一ACTOR変数を再利用（Requirement 0）
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
-- 意図: ファイルレベル属性はAttributeRegistryに記録、Lua出力なし（Requirement 1b）

-- ####################################################################
-- ＃ グローバル単語定義（トップレベル）
-- ####################################################################

-- ＠挨拶：こんにちは、やあ、ハロー
-- 意図: グローバル単語定義はWordDefRegistryに登録、Lua出力なし（Requirement 1b）

-- ####################################################################
-- ＊メイン
-- ####################################################################

-- 意図: SCENE変数を再利用してシーン定義（Requirement 0）
--       "メイン1" はSceneRegistryで一意なモジュール名として解決
do
    local SCENE = PASTA:create_scene("メイン1")

    -- 　＃ ローカル単語定義
    -- 　＠場所：東京、大阪、京都
    -- 　＠天気：晴れ、曇り、雨
    -- 意図: ローカル単語定義はローカルスコープのWordDefRegistryに登録、Lua出力なし（Requirement 1e）

    -- __start__ - グローバルシーンのエントリーポイント
    -- 意図: グローバルシーン（＊メイン）のエントリーポイントは常に __start__（Requirement 1c）
    --       関数シグネチャ: (ctx, ...)
    --       . 構文で宣言、SCENE はセッション初期化の第1引数で明示的に渡す
    function SCENE.__start__(ctx, ...)
        -- 意図: 第1行で引数をテーブル化、第2行でセッション初期化（Requirement 1c）
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　＞グローバル単語呼び出し
        -- 意図: Call文は act:call(モジュール名, ラベル名, 属性フィルター, ...引数) 形式（Requirement 1d）
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
        -- 意図: 引数付きCall文では変数展開後、残りの引数を ... で継承（Requirement 1d）
        act:call("メイン1", "引数付き呼び出し", {}, var.カウンタ, save.グローバル, table.unpack(args))
    end

    -- 　・グローバル単語呼び出し
    -- 意図: 第1階層ローカルシーンは __名前__ 形式（Requirement 1c）
    --       重複対策のカウンタ "1" を末尾に付与
    function SCENE.__グローバル単語呼び出し1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：＠笑顔　＠挨拶！
        -- 意図: 単語参照（＠挨拶）は act.アクター:word("単語名") に変換（Requirement 1e）
        --       アクター属性（＠笑顔）も同様に word() で処理
        --       通常テキストは act.アクター:talk("テキスト") に変換（Requirement 1d）
        act.さくら:word("笑顔")
        act.さくら:word("挨拶")
        act.さくら:talk("！")

        -- 　　　うにゅう：＠通常　やふぅ。
        act.うにゅう:word("通常")
        act.うにゅう:talk("やふぅ。")
    end

    -- 　・ローカル単語呼び出し
    function SCENE.__ローカル単語呼び出し1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：＠通常　＠場所　の天気は？
        -- 意図: 複数の単語参照とテキストの混在を、word() と talk() に分割（Requirement 1d）
        act.さくら:word("通常")
        act.さくら:word("場所")
        act.さくら:talk("の天気は？")

        -- 　　　うにゅう：＠天気　らしいで。
        act.うにゅう:word("天気")
        act.うにゅう:talk("らしいで。")
    end

    -- 　・会話分岐
    -- 意図: 重複シーン名（会話分岐１、会話分岐２）は異なる関数として生成
    --       ランダム選択はシーンセレクター（Requirement 3）が実行時に処理
    function SCENE.__会話分岐1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：ローカル分岐１だよ。
        act.さくら:talk("ローカル分岐１だよ。")

        -- 　　　うにゅう：ちっぽけやね。
        act.うにゅう:talk("ちっぽけやね。")
    end

    -- 　・会話分岐
    function SCENE.__会話分岐2__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：ローカル分岐２だよ。
        act.さくら:talk("ローカル分岐２だよ。")

        -- 　　　うにゅう：もっと飛べる、ワイは飛べるんや！
        act.うにゅう:talk("もっと飛べる、ワイは飛べるんや！")

        -- 　　　さくら　：＠ぐんにょり　なんでだよ。
        -- 意図: アクター属性参照は word() で処理（Requirement 1e）
        act.さくら:word("ぐんにょり")
        act.さくら:talk("なんでだよ。")
    end

    -- 　・変数代入
    function SCENE.__変数代入1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　＄カウンタ＝１０
        -- 意図: ローカル変数（＄変数名）は var.変数名 に代入（Requirement 1d）
        var.カウンタ = 10

        -- 　　　＄＊グローバル＝＠関数（２+１）
        -- 意図: グローバル変数（＄＊変数名）は save.変数名 に代入（Requirement 1d）
        --       関数呼び出し（＠関数）は SCENE.関数(ctx, 引数...) 形式で宣言、呼び出しは : で実行（Requirement 1d）
        save.グローバル = SCENE:関数(ctx, 2 + 1)
    end

    -- 　・引数付き呼び出し
    function SCENE.__引数付き呼び出し1__(ctx, ...)
        local args = { ... }
        local act, save, var = PASTA:create_session(SCENE, ctx)

        -- 　　　さくら　：第１引数は＄０　だよ。
        -- 意図: 引数参照（＄０→args[1]、＄１→args[2]）Pasta DSLの0-baseをLua 1-baseに変換（Requirement 1d）
        --       文字列連結は ".." 演算子、tostring() で型変換（Requirement 1d）
        act.さくら:talk("第１引数は" .. tostring(args[1]) .. "だよ。")

        -- 　　　うにゅう：第２引数は＄１　やね。
        act.うにゅう:talk("第２引数は" .. tostring(args[2]) .. "やね。")
    end

    -- ```lua
    -- function SCENE.関数(ctx, value, ...)
    --     return value * value
    -- end
    -- ```
    -- 意図: Runeブロック（```rune または ```lua）内の関数定義は、
    --       . 構文で SCENE.関数名 としてLua関数定義に変換（Requirement 1f）
    function SCENE.関数(ctx, value, ...)
        return value * value
    end
end


-- ####################################################################
-- ＊会話分岐グローバル
-- ####################################################################

-- 意図: 別のグローバルシーン定義（SCENE変数を再利用）（Requirement 0）
--       グローバルシーン「＊会話分岐グローバル」は一意なモジュール名「会話分岐1」として生成（Requirement 1g）
do
    local SCENE = PASTA:create_scene("会話分岐1")

    -- __start__ - グローバルシーンのエントリーポイント
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
