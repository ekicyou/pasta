-- sakura_builder module tests
-- Tests for pasta.shiori.sakura_builder module - grouped token to sakura script conversion
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- ============================================================================
-- ヘルパー: モックアクター作成
-- ============================================================================

local function create_mock_actors()
    return {
        sakura = { name = "さくら", spot = 0 },
        kero = { name = "うにゅう", spot = 1 },
    }
end

-- ヘルパー: テストごとの共通セットアップ（BUILDER と actors を返す）
-- リロード規約（先行スイートの package.loaded 操作）を保存するため、
-- require はファイル先頭ではなくテスト実行時に行う
local function setup()
    return require("pasta.shiori.sakura_builder"), create_mock_actors()
end

-- ============================================================================
-- Requirement 6: sakura_builderモジュール（グループ化形式）
-- ============================================================================

describe("SAKURA_BUILDER - talk token", function()
    test("talkトークンがtalk_to_scriptで変換される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- デフォルト設定（script_wait_normal=50）では
        -- effective_wait = 50 - 50 = 0 なのでウェイトタグは挿入されない
        -- 文字列がそのまま含まれることを確認
        expect(result:find("Hello")):toBeTruthy()
    end)

    test("句点にはウェイトタグが挿入される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "あ。" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- 句点（。）にはデフォルトでウェイトタグが挿入される
        -- script_wait_period=1000 → effective=950
        expect(result:find("\\_w%[950%]")):toBeTruthy()
    end)

    test("読点にはウェイトタグが挿入される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "あ、" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- 読点（、）にはデフォルトでウェイトタグが挿入される
        -- script_wait_comma=500 → effective=450
        expect(result:find("\\_w%[450%]")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - surface token", function()
    test("surfaceトークンを \\s[id] に変換する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",    actor = actors.sakura, text = "" },
                    { type = "surface", id = 5 },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\s%[5%]")):toBeTruthy()
    end)

    test("文字列IDをサポートする", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",    actor = actors.sakura, text = "" },
                    { type = "surface", id = "smile" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\s%[smile%]")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - wait token", function()
    test("waitトークンを \\_w[ms] に変換する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "" },
                    { type = "wait", ms = 500 },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\_w%[500%]")):toBeTruthy()
    end)

    -- `\w時間` は 1〜9 の 50ms 単位専用であり、角括弧書式を取らない（UKADOC）
    test("角括弧付き \\w[ms] を出力しない", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "" },
                    { type = "wait", ms = 500 },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- `\_w[` の直前が `_` でない `\w[` が存在しないこと
        expect(result:find("[^_]w%[500%]")):toBeFalsy()
    end)
end)

describe("SAKURA_BUILDER - newline token", function()
    test("newlineトークンを \\n に変換する（n=1）", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",    actor = actors.sakura, text = "" },
                    { type = "newline", n = 1 },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\n")):toBeTruthy()
    end)

    test("複数改行を連続出力する（n=3）", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",    actor = actors.sakura, text = "" },
                    { type = "newline", n = 3 },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\n\\n\\n")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - clear token", function()
    test("clearトークンを \\c に変換する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "" },
                    { type = "clear" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\c")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - raw_script token", function()
    test("raw_scriptトークンをそのまま出力する（エスケープなし）", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",       actor = actors.sakura,      text = "" },
                    { type = "raw_script", text = "\\![open,calendar]" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[open,calendar%]")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - yield token", function()
    test("yieldトークンは無視される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "surface", id = 5 },
                    { type = "yield" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\s%[5%]")):toBeTruthy()
        -- yieldは出力されない
    end)
end)

describe("SAKURA_BUILDER - \\e終端", function()
    test("出力末尾に \\e を付与する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:sub(-2)):toBe("\\e")
    end)

    test("空トークン配列でも \\e を付与する", function()
        local BUILDER = setup()

        local result = BUILDER.build({}, {})

        expect(result):toBe("\\e")
    end)
end)

describe("SAKURA_BUILDER - 複合シナリオ", function()
    test("複数トークンを正しく連結する", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",    actor = actors.sakura, text = "Hello" },
                    { type = "surface", id = 5 },
                    { type = "wait",    ms = 100 },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk",    actor = actors.kero, text = "Hi" },
                    { type = "newline", n = 1 },
                    { type = "clear" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
        expect(result:find("\\s%[5%]")):toBeTruthy()
        expect(result:find("\\_w%[100%]")):toBeTruthy()
        -- 意図した変更(R3.1): A(spot0)→B(spot1) は各spot初テキストのため段落区切り改行を出さない
        expect(result:find("\\n%[150%]")):toBeFalsy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
        expect(result:find("Hi")):toBeTruthy()
        expect(result:find("\\n")):toBeTruthy()
        expect(result:find("\\c")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)
end)

describe("SAKURA_BUILDER - talk_to_script変換", function()
    test("テキストがtalk_to_scriptで変換される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "50 off sale" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- デフォルト設定ではウェイトタグは挿入されない
        -- 文字列がそのまま含まれることを確認
        expect(result:find("50 off sale")):toBeTruthy()
    end)
end)

-- ============================================================================
-- actor-spot-refactoring: spotトークン処理 (Task 2.1)
-- ============================================================================

describe("SAKURA_BUILDER - spotトークン処理", function()
    test("spotトークン処理でactor_spots[actor.name]が正しく更新される", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- spot設定後のtalk → \p[0]が出力される
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
    end)

    test("複数actorのspot独立管理を確認", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hi Sakura" },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Hi Kero" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- 両方のスポットタグが出力される
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
    end)
end)

-- ============================================================================
-- actor-spot-refactoring: clear_spotトークン処理 (Task 2.2)
-- ============================================================================

describe("SAKURA_BUILDER - clear_spotトークン処理", function()
    test("clear_spotトークン処理でactor_spots={}にリセットされる", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot",      actor = actors.sakura, spot = 5 },
            { type = "clear_spot" },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Reset" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- clear_spot後はデフォルトspot(0)を使用
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Reset")):toBeTruthy()
    end)

    test("clear_spotトークン処理でlast_actor=nilにリセットされる", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Before" },
                }
            },
            { type = "clear_spot" },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "After" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- clear_spot後に同じactorでもスポットタグが再出力される
        -- \p[0]が2回出現するはず
        local count = 0
        for _ in result:gmatch("\\p%[0%]") do
            count = count + 1
        end
        expect(count):toBe(2)
    end)
end)

-- ============================================================================
-- actor-spot-refactoring: actorトークンのactor切り替え検出 (Task 2.3)
-- ============================================================================

describe("SAKURA_BUILDER - actorトークンのactor切り替え検出", function()
    test("actor_spots未設定時にデフォルト値0を使用する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- デフォルトspot(0)が使用される
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
    end)

    test("last_actor != token.actor時に\\p[spot]を出力する", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "First" },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Second" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- actor切り替え時にスポットタグが出力される
        expect(result:find("\\p%[0%].*First")):toBeTruthy()
        expect(result:find("\\p%[1%].*Second")):toBeTruthy()
    end)

    test("spot変更時に\\n[N]を出力する（Nはconfig.spot_newlines * 100）", function()
        local BUILDER, actors = setup()

        -- 意図した変更(R2.7): 完全遅延方式では A→B の単純切替では改行ゼロ。
        -- N 算出（spot_newlines * 100）を検証するため A→B→A 往復にし、
        -- 戻り手番(A2)の直前で \n[150] が1回だけ出ることを確認する。
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "First" },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Second" },
                }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Third" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- has-text 済みの spot0 へ戻る手番の直前でのみ段落改行が出力される
        expect(result:find("\\p%[0%]\\n%[150%]Third")):toBeTruthy()
        -- A1・B1 の前には改行が出ない（改行は戻り手番の1回だけ）
        local count = 0
        for _ in result:gmatch("\\n%[150%]") do count = count + 1 end
        expect(count):toBe(1)
    end)

    test("同じactorの連続actorトークンではスポットタグを出力しない", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "First" },
                }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Second" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- スポットタグは1回のみ
        local count = 0
        for _ in result:gmatch("\\p%[0%]") do
            count = count + 1
        end
        expect(count):toBe(1)
        expect(result:find("FirstSecond")):toBeTruthy()
    end)
end)

-- ============================================================================
-- sakura-script-newline: 直前が全さくらスクリプトなら段落区切り改行を抑制
-- 判定基準:「前回の段落区切り改行以降に一般文字列（talk）が1文字でも出力されたか」
-- ============================================================================

describe("SAKURA_BUILDER - sakura-script-newline: 改行キャンセル", function()
    -- ヘルパー: 出力中の \n[150] 出現回数を数える
    local function count_break(result)
        local count = 0
        for _ in result:gmatch("\\n%[150%]") do
            count = count + 1
        end
        return count
    end

    test("直前の会話が全てさくらスクリプト（surfaceのみ）なら改行を出力しない", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    -- 一般文字列なし（surfaceのみ）
                    { type = "surface", id = 5 },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Hello" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- 直前（さくら）が一般文字列を出していないので段落改行はキャンセル
        expect(count_break(result)):toBe(0)
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
        expect(result:find("Hello")):toBeTruthy()
    end)

    test("直前の会話が全てさくらスクリプト（sakura_scriptのみ）でも改行を出力しない", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    -- 作者記述のインラインさくらスクリプト（一般文字列ではない）
                    { type = "sakura_script", actor = actors.sakura, text = "\\w9" },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Hello" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        expect(count_break(result)):toBe(0)
    end)

    test("空文字列のtalkは一般文字列としてカウントしない", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    -- 空talk + surface（可視文字は1文字も出ない）
                    { type = "talk",    actor = actors.sakura, text = "" },
                    { type = "surface", id = 3 },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Hi" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        expect(count_break(result)):toBe(0)
    end)

    test("A→B の単純切替では各spot初テキストのため改行を出さない（意図した変更 R3.1）", function()
        local BUILDER, actors = setup()

        -- 意図した変更(R3.1): 先出し版は離脱側(A)末尾に \n[150] を出していたが、
        -- 完全遅延方式では A(spot0)・B(spot1) とも各 spot の初テキストであり、
        -- 離脱側末尾のゴミ改行は排除される（改行ゼロ）。
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "First" },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Second" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        expect(count_break(result)):toBe(0)
    end)

    test("先頭のサーフェス設定手番でも、可視発話が各spot初テキストなら改行を出さない（意図した変更 R3.1/R5.2）", function()
        local BUILDER, actors = setup()

        -- \0\s[5] → \1\s[15] → \0「A」→ \1「B」
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "surface", id = 5 } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "surface", id = 15 } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- 意図した変更(R3.1/R5.2): A(spot0)・B(spot1) とも各 spot の初テキストのため
        -- 段落区切り改行は出ない。先頭 surface 手番はもちろん、可視発話 A→B の間にも出ない。
        expect(count_break(result)):toBe(0)
        expect(result:find("A\\p%[1%]B")):toBeTruthy()
    end)

    test("さくらスクリプトのみの手番を挟んでも、戻り先が各spot初テキストなら改行を出さない（意図した変更 R2.3/R3.1）", function()
        local BUILDER, actors = setup()

        -- \0「こんにちは」→ \1\s[10] → \0\s[11] → \1「またね」
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "こんにちは" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "surface", id = 10 } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "surface", id = 11 } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "またね" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- 意図した変更(R2.3/R3.1): 「こんにちは」は spot0 初テキスト、「またね」は spot1 初テキスト。
        -- spot0 へ戻る surface のみの手番で pending がセットされるが、次の切替(→spot1)で破棄・再評価され、
        -- 切替先 spot1 は has-text 偽のため改行は出ない。往復のいずれの手番でも段落改行はゼロ。
        expect(count_break(result)):toBe(0)
        -- 「こんにちは」の直後に段落区切り改行は出ない
        expect(result:find("こんにちは\\n%[150%]")):toBeFalsy()
    end)

    test("clear_spotで改行許可フラグがリセットされる", function()
        local BUILDER, actors = setup()

        -- 「A」出力後 clear_spot → その後 surfaceのみ手番 → 別actor切替 で改行しない。
        -- clear_spot 後に spot を再設定し、切替先スポットが異なる（0→1）状況を作ることで、
        -- 改行が出ない理由が「スポット同値」ではなく「フラグのリセット」であることを分離する。
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A" } }
            },
            { type = "clear_spot" },
            -- clear_spot でマップが空になるため spot を再設定
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "surface", id = 7 } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- clear_spot でフラグがリセットされ、直前(さくらのsurfaceのみ)は一般文字列なし。
        -- スポットは 0→1 で異なるが、フラグ false のため \1「B」への切替で改行は出ない。
        expect(count_break(result)):toBe(0)
    end)
end)

-- ============================================================================
-- sakura-script-newline: 完全遅延（Task 3.1）
-- 会話終端・往復・スポット共有シナリオの完全遅延挙動を固定するテスト。
-- 要件 2.6, 2.7, 2.8, 3.1, 6.6, 7.3
-- ============================================================================

describe("SAKURA_BUILDER - sakura-script-newline: 完全遅延（Task 3.1）", function()
    -- ヘルパー: 指定パターンの出現回数を数える
    local function count(result, pattern)
        local c = 0
        for _ in result:gmatch(pattern) do c = c + 1 end
        return c
    end

    test("A→B 終了で段落区切り改行がゼロ（異なるスポット 0/1）", function()
        local BUILDER, actors = setup()

        -- 意図した挙動(R2.6/R3.1): A(spot0)→B(spot1) で会話が終わる場合、
        -- 各 spot は初テキストであり、離脱側末尾のゴミ改行も出さない完全遅延方式では
        -- 出力全体に段落区切り改行 \n[ が1つも含まれない。
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- \n[ は1つも含まれない（末尾の未フラッシュ pending も破棄される）
        expect(result:find("\\n%[")):toBeFalsy()
        expect(result:find("\\p%[0%]A1")):toBeTruthy()
        expect(result:find("\\p%[1%]B1")):toBeTruthy()
    end)

    test("A→B→A 往復（異なるスポット）で戻り手番のみ改行", function()
        local BUILDER, actors = setup()

        -- 意図した挙動(R2.7/R2.8): A1・B1 は各 spot 初テキストのため改行なし。
        -- A2 は has-text 済みの spot0 へ戻るため、その直前でのみ \n[150] が1回出る。
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A2" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- 戻り手番 A2 の直前でのみ段落改行（\p[0]\n[150]A2）
        expect(result:find("\\p%[0%]\\n%[150%]A2")):toBeTruthy()
        -- A1・B1 の直前には改行が出ない
        expect(result:find("\\p%[0%]A1")):toBeTruthy()
        expect(result:find("\\p%[1%]B1")):toBeTruthy()
        -- \n[150] の総数は1
        expect(count(result, "\\n%[150%]")):toBe(1)
    end)

    test("同一スポット共有の2アクター交代（spot 0 共有）でも切替直後に改行", function()
        local BUILDER, actors = setup()

        -- 意図した挙動(R6.6/R3.1): 両アクターとも spot 0。A1 で spot0 に has-text が付き、
        -- B へ切り替えると同一 spot0 を解決し pending が真になるため、B1 直前に \n[150] が出る。
        -- 先出し版の「同一スポット抑制」は完全遅延方式では適用されない。
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 0 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- B1 直前に \n[150] が出る（\p[0]\n[150]B1）
        expect(result:find("\\p%[0%]\\n%[150%]B1")):toBeTruthy()
        -- 切替のたびにタグ出力されるため \p[0] は2回
        expect(count(result, "\\p%[0%]")):toBe(2)
        -- A1 の直前には改行が出ない
        expect(result:find("\\p%[0%]A1")):toBeTruthy()
        -- \n[150] の総数は1（戻り手番の1回のみ）
        expect(count(result, "\\n%[150%]")):toBe(1)
    end)

    test("N 算出: spot_newlines 1.5→\\n[150] / 2.0→\\n[200]（往復シナリオ）", function()
        local BUILDER, actors = setup()

        local function make_tokens()
            return {
                { type = "spot", actor = actors.sakura, spot = 0 },
                { type = "spot", actor = actors.kero,   spot = 1 },
                {
                    type = "actor",
                    actor = actors.sakura,
                    tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
                },
                {
                    type = "actor",
                    actor = actors.kero,
                    tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
                },
                {
                    type = "actor",
                    actor = actors.sakura,
                    tokens = { { type = "talk", actor = actors.sakura, text = "A2" } }
                },
            }
        end

        -- spot_newlines = 1.5 → \n[150]
        local r15 = BUILDER.build(make_tokens(), { spot_newlines = 1.5 })
        expect(r15:find("\\p%[0%]\\n%[150%]A2")):toBeTruthy()
        expect(count(r15, "\\n%[150%]")):toBe(1)
        expect(r15:find("\\n%[200%]")):toBeFalsy()

        -- spot_newlines = 2.0 → \n[200]
        local r20 = BUILDER.build(make_tokens(), { spot_newlines = 2.0 })
        expect(r20:find("\\p%[0%]\\n%[200%]A2")):toBeTruthy()
        expect(count(r20, "\\n%[200%]")):toBe(1)
        expect(r20:find("\\n%[150%]")):toBeFalsy()
    end)
end)

-- ============================================================================
-- sakura-script-newline: 保留破棄・フラッシュ位置・クリア系（Task 3.2）
-- 完全遅延方式(S0–S7)の pending 破棄 / フラッシュ位置 / clear系リセット挙動を
-- 固定するテスト。要件 1.3, 2.2, 2.3, 3.3, 4.4, 4.6, 7.3
-- ============================================================================

describe("SAKURA_BUILDER - sakura-script-newline: 保留破棄・フラッシュ・クリア（Task 3.2）", function()
    -- ヘルパー: 指定パターンの出現回数を数える
    local function count(result, pattern)
        local c = 0
        for _ in result:gmatch(pattern) do c = c + 1 end
        return c
    end

    -- 1. pending 破棄→再評価 (R2.3)
    test("has-text済みスポットへ surface のみで戻り次の切替が起きると保留が破棄され切替先で再評価される", function()
        local BUILDER, actors = setup()

        -- A1(spot0 text) → kero surface(spot1) → sakura surface(spot0 復帰=pending真) →
        -- kero talk C(spot1 復帰。spot0 の pending は破棄され spot1 の has-text で再評価=偽) →
        -- sakura talk A2(spot0 復帰。spot0 の has-text 真で再評価=真→改行)
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "surface", id = 6 } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "surface", id = 5 } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "C" } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A2" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- spot0 復帰(surface のみ)で立った pending は次の切替(→spot1)で破棄され、
        -- spot1 は has-text 偽のため C の直前に改行は出ない。
        expect(result:find("\\p%[1%]C")):toBeTruthy()
        -- spot0 の has-text は persist しており、A2 の直前で再評価→改行が1回だけ出る。
        expect(result:find("\\p%[0%]\\n%[150%]A2")):toBeTruthy()
        expect(count(result, "\\n%[150%]")):toBe(1)
    end)

    -- 2. テキストなし終端の保留破棄 (R3.3)
    test("has-text済みスポットへ surface/wait のみで戻り終端すると保留改行を出さず \\e で終わる", function()
        local BUILDER, actors = setup()

        -- A1(spot0) → B1(spot1) → sakura surface+wait のみで spot0 復帰(pending真)して終端
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "surface", id = 5 },
                    { type = "wait",    ms = 100 },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- 末尾に段落区切り改行 \n[ が出力されず、\e で終わる（未フラッシュ pending は破棄）
        expect(result:sub(-2)):toBe("\\e")
        expect(result:find("\\n%[")):toBeFalsy()
    end)

    -- 3. フラッシュ位置 (R1.3)
    test("戻り手番の先頭に surface を挟むと \\n[N] は surface の後・talk の直前に出る", function()
        local BUILDER, actors = setup()

        -- A1(spot0) → B1(spot1) → sakura surface+talk C で spot0 復帰(pending真)
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "surface", id = 5 },
                    { type = "talk",    actor = actors.sakura, text = "C" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- \n[150] は \p 直後ではなく surface \s[5] の後・talk C の直前に出る
        expect(result:find("\\p%[0%]\\s%[5%]\\n%[150%]C")):toBeTruthy()
        -- \p 直後にフラッシュされていないこと（\p[0]\n[150]\s ではない）
        expect(result:find("\\p%[0%]\\n%[150%]\\s")):toBeFalsy()
        expect(count(result, "\\n%[150%]")):toBe(1)
    end)

    -- 4. 同一アクター継続グループでのフラッシュ (R2.2)
    test("切替グループが surface のみでも同一アクター後続グループの talk 直前でフラッシュされる", function()
        local BUILDER, actors = setup()

        -- A1(spot0) → B1(spot1) → sakura surface のみ(spot0 復帰=pending真) →
        -- sakura talk C(同一アクター継続=切替なし。pending 生存。C 直前でフラッシュ)
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "surface", id = 5 } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "C" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- surface の後、同一アクター後続グループの talk C 直前で \n[150] がフラッシュされる。
        -- 同一アクター継続は切替なし(S2)のため pending は破棄されず生存する。
        expect(result:find("\\s%[5%]\\n%[150%]C")):toBeTruthy()
        expect(count(result, "\\n%[150%]")):toBe(1)
        -- surface グループと talk グループの間で切替は起きないため \p[0] は2回のみ
        expect(count(result, "\\p%[0%]")):toBe(2)
    end)

    -- 5. clear_spot リセット (R4.4)
    test("has-text済み状態で clear_spot 後は以後の切替で保留が発生しない", function()
        local BUILDER, actors = setup()

        -- A1(spot0)/B1(spot1) で has-text 蓄積 → clear_spot → spot 再設定 →
        -- A2(spot0 復帰)/B2(spot1 復帰) とも has-text リセット済みのため改行ゼロ
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
            { type = "clear_spot" },
            -- clear_spot でマップが空になるため spot を再設定
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A2" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B2" } }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        -- clear_spot が has-text をリセットするため、A2(spot0 へ「戻る」)でも改行は出ない。
        -- リセットされない実装なら A2 で \n[150] が出て count が 1 以上になり落ちる。
        expect(count(result, "\\n%[150%]")):toBe(0)
        expect(result:find("\\p%[0%]A2")):toBeTruthy()
        expect(result:find("\\p%[1%]B2")):toBeTruthy()
    end)

    -- 6. \c で保留破棄＋has-text リセット (R4.6)
    test("\\c は保留改行を破棄し当該スポットの has-text を偽へリセットする", function()
        local BUILDER, actors = setup()

        -- (A) 破棄: A1(spot0)/B1(spot1) → sakura {clear, talk A2} で spot0 復帰(pending真)。
        --     \c が pending を破棄するため A2 直前に改行は出ない（\p[0]\cA2）。
        local tokens_a = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "clear" },
                    { type = "talk", actor = actors.sakura, text = "A2" },
                }
            },
        }
        local result_a = BUILDER.build(tokens_a, { spot_newlines = 1.5 })

        -- \c 直後の A2 の前に段落区切り改行が出ない（保留は破棄）
        expect(result_a:find("\\p%[0%]\\cA2")):toBeTruthy()
        expect(count(result_a, "\\n%[150%]")):toBe(0)

        -- (B) has-text リセット: A1(spot0)/B1(spot1) → sakura {clear}(spot0 has-text→偽) →
        --     kero surface のみ(spot1 復帰) → sakura talk A2(spot0 再訪)。
        --     \c が has-text をリセットしたため、他スポットを経て再訪しても改行は出ない。
        local tokens_b = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "clear" } }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = { { type = "surface", id = 6 } }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = { { type = "talk", actor = actors.sakura, text = "A2" } }
            },
        }
        local result_b = BUILDER.build(tokens_b, { spot_newlines = 1.5 })

        -- \c で spot0 の has-text が偽へリセットされたため、再訪した A2 の直前に改行は出ない。
        -- リセットされない実装なら A2 で \n[150] が出て落ちる。
        expect(count(result_b, "\\n%[150%]")):toBe(0)
        expect(result_b:find("\\p%[0%]A2")):toBeTruthy()
        expect(result_b:find("\\c")):toBeTruthy()
    end)
end)

-- ============================================================================
-- actor-spot-refactoring: 統合シナリオ (Task 4.4)
-- ============================================================================

describe("SAKURA_BUILDER - 統合シナリオ（グループ化トークン構造）", function()
    test("set_spot()なしでのtalk() → デフォルトspot(0)使用を確認", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "No spot set" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("No spot set")):toBeTruthy()
    end)

    test("set_spot() → talk() → spot切り替えとスポットタグ出力を確認", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Sakura speaks" },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Kero speaks" },
                }
            },
        }
        local result = BUILDER.build(tokens, { spot_newlines = 1.5 })

        expect(result:find("\\p%[0%]Sakura speaks")):toBeTruthy()
        -- 意図した変更(R3.1): A(spot0)→B(spot1) は各spot初テキストのため \n[150] を出さない
        expect(result:find("\\p%[1%]Kero speaks")):toBeTruthy()
        expect(result:find("\\n%[150%]")):toBeFalsy()
    end)

    test("clear_spot() → talk() → デフォルトspot(0)への復帰を確認", function()
        local BUILDER, actors = setup()

        local tokens = {
            { type = "spot",      actor = actors.sakura, spot = 5 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "At spot 5" },
                }
            },
            { type = "clear_spot" },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Back to default" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\p%[5%]")):toBeTruthy()
        expect(result:find("\\p%[0%]")):toBeTruthy() -- clear後はデフォルト
    end)
end)

-- ============================================================================
-- persist-spot-position: build()シグネチャ拡張テスト (Task 5.3, 5.4, 5.5)
-- ============================================================================

describe("SAKURA_BUILDER - persist-spot-position: 純粋関数性テスト (Task 5.3)", function()
    test("入力テーブルがspotトークンで直接変更されることを確認", function()
        local BUILDER, actors = setup()

        local input_spots = {}
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }

        BUILDER.build(tokens, {}, input_spots)

        -- 入力テーブルが直接変更されている
        expect(input_spots["さくら"]):toBe(0)
        expect(input_spots["うにゅう"]):toBe(1)
    end)

    test("nil入力時に空テーブルとして扱われることを確認", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }

        -- actor_spots = nil で呼び出し
        local result = BUILDER.build(tokens, {}, nil)

        expect(result:find("\\p%[0%]")):toBeTruthy()
    end)

    test("spotトークンでinput_spotsが直接変更される", function()
        local BUILDER, actors = setup()

        local input_spots = {}
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }

        local result = BUILDER.build(tokens, {}, input_spots)

        expect(type(result)):toBe("string")
        expect(input_spots["さくら"]):toBe(0)
    end)

    test("後方互換性: actor_spots省略時も正常動作", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hello" },
                }
            },
        }

        -- 第3引数を省略しても動作
        local result = BUILDER.build(tokens, {})

        expect(result:find("Hello")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - persist-spot-position: clear_spotトークン処理 (Task 5.4)", function()
    test("clear_spotトークンで入力のactor_spotsがリセットされる", function()
        local BUILDER, actors = setup()

        local input_spots = { ["さくら"] = 5, ["うにゅう"] = 3 }
        local tokens = {
            { type = "clear_spot" },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Reset" },
                }
            },
        }

        local result = BUILDER.build(tokens, {}, input_spots)

        -- clear_spot後はデフォルトspot(0)を使用
        expect(result:find("\\p%[0%]")):toBeTruthy()
        -- 入力テーブルのエントリが直接クリアされている
        expect(input_spots["さくら"]):toBe(nil)
        expect(input_spots["うにゅう"]):toBe(nil)
    end)

    test("直接変更: clear_spotで入力テーブルのエントリがクリアされる", function()
        local BUILDER = setup()

        local input_spots = { ["さくら"] = 5, ["うにゅう"] = 3 }
        local tokens = {
            { type = "clear_spot" },
        }

        BUILDER.build(tokens, {}, input_spots)

        -- 入力テーブルが直接変更される（直接変更方式）
        expect(input_spots["さくら"]):toBe(nil)
        expect(input_spots["うにゅう"]):toBe(nil)
    end)
end)

describe("SAKURA_BUILDER - persist-spot-position: spotトークン処理 (Task 5.5)", function()
    test("spotトークンで入力のactor_spotsが正しく更新される", function()
        local BUILDER, actors = setup()

        local input_spots = { ["さくら"] = 0 }
        local tokens = {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Hi" },
                }
            },
        }

        BUILDER.build(tokens, {}, input_spots)

        expect(input_spots["さくら"]):toBe(0)
        expect(input_spots["うにゅう"]):toBe(1)
    end)

    test("入力actor_spotsの値を引き継いでスポットタグが出力される", function()
        local BUILDER, actors = setup()

        -- 前回のスポット状態を入力として渡す
        -- 意図した変更(R2.7): 完全遅延方式では A→B の単純切替は改行ゼロになるため、
        -- 引き継いだ spot 値による段落改行の観測を維持するには A→B→A 往復が必要。
        local input_spots = { ["さくら"] = 0, ["うにゅう"] = 1 }
        local tokens = {
            -- spotトークンなし（前回の値を引き継ぐ）
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Still here" },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk", actor = actors.kero, text = "Me too" },
                }
            },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk", actor = actors.sakura, text = "Again" },
                }
            },
        }

        local result = BUILDER.build(tokens, { spot_newlines = 1.5 }, input_spots)

        -- 前回のスポット値が引き継がれている
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
        -- has-text 済みの spot0 へ戻る手番の直前でのみ段落改行が出力される
        expect(result:find("\\p%[0%]\\n%[150%]Again")):toBeTruthy()

        -- 入力テーブルも前回の値を保持（直接変更だが値は変わらない）
        expect(input_spots["さくら"]):toBe(0)
        expect(input_spots["うにゅう"]):toBe(1)
    end)
end)

-- ============================================================================
-- act-sakura-script-method: sakura_scriptトークン処理 (Task 4.3)
-- ============================================================================

describe("SAKURA_BUILDER - sakura_scriptトークン処理", function()
    test("sakura_scriptトークンがtalk_to_scriptで変換される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "sakura_script", actor = actors.sakura, text = "\\n" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- さくらスクリプトタグがそのまま出力に含まれる
        expect(result:find("\\n")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("talk + sakura_script混合がそれぞれ処理される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",          actor = actors.sakura, text = "Hello" },
                    { type = "sakura_script", actor = actors.sakura, text = "\\w9" },
                    { type = "talk",          actor = actors.sakura, text = "World" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("Hello")):toBeTruthy()
        expect(result:find("\\w9")):toBeTruthy()
        expect(result:find("World")):toBeTruthy()
    end)

    test("raw_scriptトークンの既存動作が変化していない", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "raw_script", text = "\\![open,notepad]" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[open,notepad%]")):toBeTruthy()
    end)
end)

-- ============================================================================
-- choice-definition-dsl: choice/choice_timeoutトークン処理 (Task 4.1)
-- ============================================================================

describe("SAKURA_BUILDER - choiceトークン処理", function()
    test("choiceトークンを \\![*]\\q[display,target] に変換する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice", display = "こんにちは", target = "挨拶" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[%*%]\\q%[こんにちは,挨拶%]")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("displayテキスト内の ] をエスケープする", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice", display = "yes]no", target = "分岐" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[%*%]\\q%[yes\\]no,分岐%]")):toBeTruthy()
    end)

    test("displayテキスト内の , をエスケープする", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice", display = "A,B", target = "分岐" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[%*%]\\q%[A\\,B,分岐%]")):toBeTruthy()
    end)

    test("displayテキスト内の \\ をエスケープする", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice", display = "a\\b", target = "分岐" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[%*%]\\q%[a\\\\b,分岐%]")):toBeTruthy()
    end)

    test("displayテキスト内の複合デリミタをエスケープする（], ,, \\）", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice", display = "yes]or,no\\maybe", target = "挨拶" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        -- \ → \\, ] → \], , → \,
        expect(result:find("\\!%[%*%]\\q%[yes\\]or\\,no\\\\maybe,挨拶%]")):toBeTruthy()
    end)

    test("targetテキスト内のデリミタもエスケープされる", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice", display = "はい", target = "a],b\\c" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[%*%]\\q%[はい,a\\]\\,b\\\\c%]")):toBeTruthy()
    end)
end)

-- ============================================================================
-- call-execution-correctness 2.4: 選択肢トークンの scope を \q の第 3 引数に出す
-- ============================================================================

describe("SAKURA_BUILDER - choiceトークンの scope（\\q 第 3 引数）", function()
    local function build_choice(choice)
        local BUILDER, actors = setup()
        choice.type = "choice"
        return BUILDER.build({
            { type = "actor", actor = actors.sakura, tokens = { choice } },
        }, {})
    end

    test("scope があれば \\q[display,target,scope] を出す", function()
        local result = build_choice({ display = "はい", target = "分岐", scope = "メイン" })
        expect(result:find("\\![*]\\q[はい,分岐,メイン]", 1, true)):toBeTruthy()
    end)

    test("scope が無ければ現行の \\q[display,target] を出す", function()
        local result = build_choice({ display = "はい", target = "分岐" })
        expect(result:find("\\![*]\\q[はい,分岐]", 1, true)):toBeTruthy()
    end)

    test("target が On で始まるときは scope を付けない", function()
        local result = build_choice({ display = "はい", target = "OnTest", scope = "メイン" })
        expect(result:find("\\![*]\\q[はい,OnTest]", 1, true)):toBeTruthy()
    end)

    test("target が script: で始まるときは scope を付けない", function()
        local result = build_choice({ display = "はい", target = "script:\\e", scope = "メイン" })
        expect(result:find("\\![*]\\q[はい,script:\\\\e]", 1, true)):toBeTruthy()
    end)

    test("scope のデリミタも display・target と同じくエスケープする", function()
        local result = build_choice({ display = "はい", target = "分岐", scope = "a],b\\c" })
        expect(result:find("\\![*]\\q[はい,分岐,a\\]\\,b\\\\c]", 1, true)):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - choice_timeoutトークン処理", function()
    test("choice_timeoutトークンを \\![set,choicetimeout,<ms>] に変換する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice_timeout", seconds = 5 },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[set,choicetimeout,5000%]")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)

    test("choice_timeout seconds=nil の場合は 0 を出力する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice_timeout", seconds = nil },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[set,choicetimeout,0%]")):toBeTruthy()
    end)

    test("choice_timeout 小数秒をミリ秒に変換し floor する", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "choice_timeout", seconds = 2.5 },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("\\!%[set,choicetimeout,2500%]")):toBeTruthy()
    end)
end)

describe("SAKURA_BUILDER - choice統合シナリオ", function()
    test("talk + choice_timeout + choice の複合トークンが正しく連結される", function()
        local BUILDER, actors = setup()

        local tokens = {
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",           actor = actors.sakura, text = "選んでね" },
                    { type = "choice_timeout", seconds = 10 },
                    { type = "choice",         display = "はい",  target = "yes_route" },
                    { type = "choice",         display = "いいえ", target = "no_route" },
                }
            },
        }
        local result = BUILDER.build(tokens, {})

        expect(result:find("選んでね")):toBeTruthy()
        expect(result:find("\\!%[set,choicetimeout,10000%]")):toBeTruthy()
        expect(result:find("\\!%[%*%]\\q%[はい,yes_route%]")):toBeTruthy()
        expect(result:find("\\!%[%*%]\\q%[いいえ,no_route%]")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)
end)

-- ============================================================================
-- sakura-builder-string-buffer: バッファ化検証 (Task 3.1)
-- 設計 Testing Strategy「Integration Tests」項2-3 / 要件 2.4, 3.4, 3.5, 4.1, 4.2
-- 既に Task 2.1 で実装済みの挙動（バッファ化・buffer_factory 注入）を
-- 実行パスで検証する追加テスト。
-- ============================================================================

describe("SAKURA_BUILDER - string-buffer: 空入力 (Task 3.1)", function()
    -- 要件 3.4: 空の grouped_tokens に対しては \e のみを返す
    test("空の grouped_tokens で build() が \\e のみを返す", function()
        local BUILDER = setup()

        local result = BUILDER.build({}, {})

        expect(result):toBe("\\e")
    end)

    test("空入力はネイティブ/フォールバック両経路で \\e のみを返す", function()
        local BUILDER = setup()
        local buf = require("pasta.buf")

        local native = BUILDER.build({}, {})
        local fallback = BUILDER.build({}, { buffer_factory = buf.new_fallback })

        expect(native):toBe("\\e")
        expect(fallback):toBe("\\e")
        -- 両経路でバイト一致
        expect(fallback):toBe(native)
    end)
end)

describe("SAKURA_BUILDER - string-buffer: フォールバック実走バイト一致 (Task 3.1)", function()
    -- 要件 2.4 / 3.5 / 4.2:
    -- (i) 既定（native buf）と (ii) buffer_factory=buf.new_fallback の出力がバイト一致すること。
    -- spot 状態を使うため、両呼び出しで独立した actor_spots テーブルを渡し副作用を分離する。

    -- 代表的な grouped_tokens を生成するファクトリ。
    -- 呼び出しごとに新しいテーブルを返し、2回の build 間で入力の共有・汚染を避ける。
    local function make_complex_tokens(actors)
        return {
            { type = "spot", actor = actors.sakura, spot = 0 },
            { type = "spot", actor = actors.kero,   spot = 1 },
            {
                type = "actor",
                actor = actors.sakura,
                tokens = {
                    { type = "talk",    actor = actors.sakura, text = "こんにちは。今日は、いい天気。" },
                    { type = "surface", id = 5 },
                    { type = "wait",    ms = 100 },
                    { type = "newline", n = 2 },
                }
            },
            {
                type = "actor",
                actor = actors.kero,
                tokens = {
                    { type = "talk",          actor = actors.kero, text = "Hi there" },
                    { type = "sakura_script", actor = actors.kero, text = "\\w9" },
                    { type = "clear" },
                    { type = "choice_timeout", seconds = 5 },
                    { type = "choice",        display = "はい]", target = "yes,route" },
                    { type = "raw_script",    text = "\\![open,calendar]" },
                }
            },
        }
    end

    test("複合入力（spot/talk/surface/wait/newline/choice 等）でバイト一致する", function()
        local BUILDER, actors = setup()
        local buf = require("pasta.buf")

        local config = { spot_newlines = 1.5 }

        -- (i) 既定（native buf）。独立した actor_spots を渡す
        local native_spots = {}
        local native = BUILDER.build(make_complex_tokens(actors), config, native_spots)

        -- (ii) フォールバック注入。同じ config・独立した actor_spots
        local fb_spots = {}
        local fb_config = { spot_newlines = 1.5, buffer_factory = buf.new_fallback }
        local fallback = BUILDER.build(make_complex_tokens(actors), fb_config, fb_spots)

        -- 完全一致（バイト一致）
        expect(fallback):toBe(native)
        -- vacuous でないこと（実際に内容を持つ出力である）の確認
        expect(#native > 2):toBeTruthy()
        expect(native:sub(-2)):toBe("\\e")
        -- 副作用（actor_spots 更新）も両経路で一致
        expect(fb_spots["さくら"]):toBe(native_spots["さくら"])
        expect(fb_spots["うにゅう"]):toBe(native_spots["うにゅう"])
    end)

    test("spot 変更・改行を含む入力でバイト一致する", function()
        local BUILDER, actors = setup()
        local buf = require("pasta.buf")

        local function make_tokens()
            return {
                { type = "spot", actor = actors.sakura, spot = 0 },
                { type = "spot", actor = actors.kero,   spot = 2 },
                {
                    type = "actor",
                    actor = actors.sakura,
                    tokens = {
                        { type = "talk", actor = actors.sakura, text = "おはよう。" },
                    }
                },
                {
                    type = "actor",
                    actor = actors.kero,
                    tokens = {
                        { type = "talk", actor = actors.kero, text = "おはよ、ございます。" },
                        { type = "newline", n = 3 },
                    }
                },
                { type = "clear_spot" },
                {
                    type = "actor",
                    actor = actors.sakura,
                    tokens = {
                        { type = "talk", actor = actors.sakura, text = "またね。" },
                    }
                },
            }
        end

        local config = { spot_newlines = 2.0 }

        local native = BUILDER.build(make_tokens(), config, {})
        local fallback = BUILDER.build(make_tokens(), { spot_newlines = 2.0, buffer_factory = buf.new_fallback }, {})

        expect(fallback):toBe(native)
        -- 意図した変更(R3.1/R4.4): 完全遅延方式では A(spot0)→B(spot2)→clear_spot→A(spot0) の
        -- いずれの手番も has-text 偽（clear_spot でリセット）のため \n[200] は出ない。
        -- このケースの主眼は native/fallback のバイト一致であり、vacuous でないことを別途確認する。
        -- （\n[200] を含む往復ケースは別タスク Task 3.3 が担当する）
        expect(native:find("\\n%[200%]")):toBeFalsy()
        expect(#native > 2):toBeTruthy()
        expect(native:find("おはよう")):toBeTruthy()
        expect(native:sub(-2)):toBe("\\e")
    end)

    test("choice/raw_script/sakura_script を含む単一actor入力でバイト一致する", function()
        local BUILDER, actors = setup()
        local buf = require("pasta.buf")

        local function make_tokens()
            return {
                {
                    type = "actor",
                    actor = actors.sakura,
                    tokens = {
                        { type = "talk",          actor = actors.sakura, text = "選んでね" },
                        { type = "choice_timeout", seconds = 10 },
                        { type = "choice",        display = "A,B]C\\D", target = "route]1" },
                        { type = "sakura_script", actor = actors.sakura, text = "\\_w[200]" },
                        { type = "raw_script",    text = "\\![open,notepad]" },
                    }
                },
            }
        end

        local native = BUILDER.build(make_tokens(), {}, {})
        local fallback = BUILDER.build(make_tokens(), { buffer_factory = buf.new_fallback }, {})

        expect(fallback):toBe(native)
        -- 出力が実際にエスケープ済みchoiceを含む（vacuousでない）
        expect(native:find("\\!%[%*%]\\q%[A\\,B\\]C\\\\D,route\\]1%]")):toBeTruthy()
        expect(native:sub(-2)):toBe("\\e")
    end)

    -- Task 3.3 (要件 6.5): \n[200] を実際に出す A→B→A 往復シナリオで
    -- native/fallback 両経路のバイト一致を検証する。上の clear_spot 経路ケースは
    -- has-text リセットのため \n[200] が出ず vacuous になり得るので、
    -- \n[200] カバレッジをこの往復ケースが担う。
    test("往復シナリオ（A→B→A, \\n[200] 実出力）でバイト一致する", function()
        local BUILDER, actors = setup()
        local buf = require("pasta.buf")

        -- spot sakura=0, spot kero=1。戻り手番 A2 の直前で \n[200] がフラッシュされる。
        local function make_tokens()
            return {
                { type = "spot", actor = actors.sakura, spot = 0 },
                { type = "spot", actor = actors.kero,   spot = 1 },
                {
                    type = "actor",
                    actor = actors.sakura,
                    tokens = { { type = "talk", actor = actors.sakura, text = "A1" } }
                },
                {
                    type = "actor",
                    actor = actors.kero,
                    tokens = { { type = "talk", actor = actors.kero, text = "B1" } }
                },
                {
                    type = "actor",
                    actor = actors.sakura,
                    -- 戻り手番。has-text 済みの spot0 へ戻るため直前に \n[200] が出る
                    tokens = { { type = "talk", actor = actors.sakura, text = "A2" } }
                },
            }
        end

        local config = { spot_newlines = 2.0 }

        local native = BUILDER.build(make_tokens(), config, {})
        local fallback = BUILDER.build(make_tokens(), { spot_newlines = 2.0, buffer_factory = buf.new_fallback }, {})

        -- native/fallback のバイト一致
        expect(fallback):toBe(native)
        -- vacuous でないこと: \n[200] が実際に出力に含まれる
        expect(native:find("\\n%[200%]")):toBeTruthy()
        expect(native:sub(-2)):toBe("\\e")
    end)
end)

-- ============================================================================
-- actor-surface-restore: 現行出力の特性化（Task 1.1）
-- 外見復旧フック追加前の出力を完全一致で固定する。以下の構成はいずれも
-- 機能導入後もバイト等価でなければならない（要件 5.1, 5.2, 5.3, 5.4）。
-- 複数ビルドに跨る会話は同一 actor_spots を連続する BUILDER.build に渡す。
-- 一般文字列は句読点を含めずウェイトタグが挿入されない決定的な出力にする。
-- ============================================================================

describe("SAKURA_BUILDER - actor-surface-restore: 現行出力の特性化（Task 1.1）", function()
    -- ヘルパー: actor グループトークン
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function script(actor, text)
        return { type = "sakura_script", actor = actor, text = text }
    end

    local function case_dedicated(appearance)
        local BUILDER, actors = setup()
        local sakura, kero = actors.sakura, actors.kero
        local config = { spot_newlines = 1.5 }
        -- 明示的な専用スポット割り当て（共有なし）
        local actor_spots = { ["さくら"] = 0, ["うにゅう"] = 1 }

        local r1 = BUILDER.build({
            group(sakura, { { type = "surface", id = 5 }, talk(sakura, "A1") }),
            group(kero, { script(kero, "\\s[10]"), talk(kero, "B1") }),
            group(sakura, { talk(sakura, "A2") }),
        }, config, actor_spots, appearance)
        expect(r1):toBe("\\p[0]\\s[5]A1\\p[1]\\s[10]B1\\p[0]\\n[150]A2\\e")

        -- スコープ切替タグ（\1）以降のサーフェス変更と、アクター未指定の生さくらスクリプト
        local r2 = BUILDER.build({
            group(sakura, { script(sakura, "\\1\\s[11]"), talk(sakura, "A3") }),
            { type = "raw_script", text = "\\0\\s[3]" },
            group(kero, { talk(kero, "B2") }),
            group(sakura, { talk(sakura, "A4") }),
        }, config, actor_spots, appearance)
        expect(r2):toBe("\\p[0]\\1\\s[11]A3\\0\\s[3]\\p[1]B2\\p[0]\\n[150]A4\\e")

        local r3 = BUILDER.build({
            group(kero, { talk(kero, "B3") }),
            group(sakura, { { type = "surface", id = "smile" }, talk(sakura, "A5"), script(sakura, "\\s6") }),
        }, config, actor_spots, appearance)
        expect(r3):toBe("\\p[1]B3\\p[0]\\s[smile]A5\\s6\\e")

        -- スポット位置マップは変化しない
        expect(actor_spots["さくら"]):toBe(0)
        expect(actor_spots["うにゅう"]):toBe(1)
    end

    local function case_no_surface(appearance)
        local BUILDER, actors = setup()
        local sakura, kero = actors.sakura, actors.kero
        local config = { spot_newlines = 1.5 }
        local actor_spots = {}

        local r1 = BUILDER.build({
            group(sakura, { talk(sakura, "A1") }),
            group(kero, { talk(kero, "B1"), { type = "wait", ms = 100 } }),
            group(sakura, { talk(sakura, "A2"), { type = "newline", n = 1 } }),
        }, config, actor_spots, appearance)
        expect(r1):toBe("\\p[0]A1\\p[0]\\n[150]B1\\_w[100]\\p[0]\\n[150]A2\\n\\e")

        local r2 = BUILDER.build({
            group(kero, { talk(kero, "B2") }),
            group(sakura, { talk(sakura, "A3") }),
        }, config, actor_spots, appearance)
        expect(r2):toBe("\\p[0]B2\\p[0]\\n[150]A3\\e")
    end

    local function case_clear_spot(appearance)
        local BUILDER, actors = setup()
        local sakura, kero = actors.sakura, actors.kero
        local config = { spot_newlines = 1.5 }
        local actor_spots = {}

        local r1 = BUILDER.build({
            { type = "spot", actor = sakura, spot = 0 },
            { type = "spot", actor = kero,   spot = 1 },
            group(sakura, { { type = "surface", id = 5 }, talk(sakura, "A1") }),
            group(kero, { script(kero, "\\s[10]"), talk(kero, "B1") }),
            { type = "clear_spot" },
            { type = "spot", actor = sakura, spot = 0 },
            { type = "spot", actor = kero,   spot = 1 },
            group(kero, { talk(kero, "B2") }),
            group(sakura, { talk(sakura, "A2") }),
        }, config, actor_spots, appearance)
        expect(r1):toBe("\\p[0]\\s[5]A1\\p[1]\\s[10]B1\\p[1]B2\\p[0]A2\\e")

        local r2 = BUILDER.build({
            group(sakura, { talk(sakura, "A3") }),
            group(kero, { talk(kero, "B3") }),
        }, config, actor_spots, appearance)
        expect(r2):toBe("\\p[0]A3\\p[1]B3\\e")
    end

    local function case_same_actor(appearance)
        local BUILDER, actors = setup()
        local sakura = actors.sakura
        local config = { spot_newlines = 1.5 }
        local actor_spots = { ["さくら"] = 0 }

        local r1 = BUILDER.build({
            group(sakura, { { type = "surface", id = 5 }, talk(sakura, "A1") }),
            group(sakura, { script(sakura, "\\s[6]"), talk(sakura, "A2") }),
        }, config, actor_spots, appearance)
        expect(r1):toBe("\\p[0]\\s[5]A1\\s[6]A2\\e")

        local r2 = BUILDER.build({
            group(sakura, { talk(sakura, "A3") }),
            group(sakura, { talk(sakura, "A4") }),
        }, config, actor_spots, appearance)
        expect(r2):toBe("\\p[0]A3A4\\e")
    end

    -- 特性化ケースを、第4引数なし（現行呼び出し）と共有外見状態（Task 3.1）の両方で同じ期待値に照合する
    test("5.1: 明示専用スポット・既定未設定の複数アクター（サーフェス・スコープ切替・生スクリプト）3ビルド", function() case_dedicated(nil) end)
    test("5.1: 明示専用スポット・既定未設定の複数アクター（サーフェス・スコープ切替・生スクリプト）3ビルド（共有外見状態）", function()
        case_dedicated(require("pasta.shiori.appearance").new())
    end)
    test("5.3: サーフェス・着せ替えなし会話（スポット0フォールバック共有）2ビルド", function() case_no_surface(nil) end)
    test("5.3: サーフェス・着せ替えなし会話（スポット0フォールバック共有）2ビルド（共有外見状態）", function()
        case_no_surface(require("pasta.shiori.appearance").new())
    end)
    test("5.1/5.4: clear_spot を含む会話（クリア後に専用スポットを再割り当て）2ビルド", function() case_clear_spot(nil) end)
    test("5.1/5.4: clear_spot を含む会話（クリア後に専用スポットを再割り当て）2ビルド（共有外見状態）", function()
        case_clear_spot(require("pasta.shiori.appearance").new())
    end)
    test("5.2: 同一アクター連続発話（サーフェス変更を含む）2ビルド", function() case_same_actor(nil) end)
    test("5.2: 同一アクター連続発話（サーフェス変更を含む）2ビルド（共有外見状態）", function()
        case_same_actor(require("pasta.shiori.appearance").new())
    end)
end)

describe("SAKURA_BUILDER - actor-surface-restore: サーフェス復旧の統合（Task 3.1）", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function script(actor, text)
        return { type = "sakura_script", actor = actor, text = text }
    end
    local config = { spot_newlines = 1.5 }

    test("2.6: 同一スポットで A(\\s[0]A1)→B(\\s[10]B1)→A(A2) は戻りの \\p[0] 直後に \\s[0] を復旧する", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "\\s[0]A1") }),
            group(b, { talk(b, "\\s[10]B1") }),
            group(a, { talk(a, "A2") }),
        }, config, { ["さくら"] = 0, ["うにゅう"] = 0 })
        expect(result):toBe("\\p[0]\\s[0]A1\\p[0]\\n[150]\\s[10]B1\\p[0]\\s[0]\\n[150]A2\\e")
    end)

    test("1.1: 構造化サーフェス指示も観測され、交代時に復旧される", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { { type = "surface", id = 5 }, talk(a, "A1") }),
            group(b, { { type = "surface", id = "smile" }, talk(b, "B1") }),
            group(a, { talk(a, "A2") }),
            group(b, { talk(b, "B2") }),
        }, config, { ["さくら"] = 0, ["うにゅう"] = 0 })
        expect(result):toBe(
            "\\p[0]\\s[5]A1\\p[0]\\s[smile]\\n[150]B1\\p[0]\\s[5]\\n[150]A2\\p[0]\\s[smile]\\n[150]B2\\e")
    end)

    test("2.5: 復旧タグだけが出た切替でも段落区切り改行の有無は復旧なしの場合と一致する", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local function run(sa, sb)
            return BUILDER.build({
                group(a, { talk(a, sa .. "A1") }),
                group(b, { talk(b, sb .. "B1") }),
                group(a, { { type = "wait", ms = 100 } }), -- 復旧タグのみ（一般文字列なし）
                group(b, { talk(b, "B2") }),
                group(a, { talk(a, "A2") }),
            }, config, {})
        end
        local with_surface = run("\\s[0]", "\\s[10]")
        local without_surface = run("", "")
        expect(with_surface):toBe(
            "\\p[0]\\s[0]A1\\p[0]\\n[150]\\s[10]B1\\p[0]\\s[0]\\_w[100]"
            .. "\\p[0]\\s[10]\\n[150]B2\\p[0]\\s[0]\\n[150]A2\\e")
        expect((with_surface:gsub("\\s%[%d+%]", ""))):toBe(without_surface)
    end)

    test("4.5: clear_spot 後も外見状態が保持され、スポット0共有の交代で復旧する", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "\\s[0]A1") }),
            group(b, { talk(b, "\\s[10]B1") }),
            { type = "clear_spot" },
            group(a, { talk(a, "A2") }),
        }, config, {})
        expect(result):toBe("\\p[0]\\s[0]A1\\p[0]\\n[150]\\s[10]B1\\p[0]\\s[0]A2\\e")
    end)

    test("5.2: 同一アクターの連続グループでは復旧しない（生スクリプトでの不明化後も）", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "\\s[0]A1") }),
            group(b, { talk(b, "\\s[10]B1") }),
            group(a, { talk(a, "A2") }),
            group(a, { talk(a, "A3") }),
            { type = "raw_script", text = "\\s[7]" },
            group(a, { talk(a, "A4") }),
        }, config, {})
        expect(result):toBe("\\p[0]\\s[0]A1\\p[0]\\n[150]\\s[10]B1\\p[0]\\s[0]\\n[150]A2A3\\s[7]A4\\e")
    end)

    test("5.5: spot トークンで移動したアクターは移動先スポットで復旧する", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            { type = "spot", actor = a, spot = 0 },
            { type = "spot", actor = b, spot = 1 },
            group(a, { talk(a, "\\s[0]A1") }),
            group(b, { talk(b, "\\s[10]B1") }),
            group(a, { talk(a, "A2") }),
            { type = "spot", actor = b, spot = 0 },
            group(b, { talk(b, "B2") }),
        }, config, {})
        expect(result):toBe("\\p[0]\\s[0]A1\\p[1]\\s[10]B1\\p[0]\\n[150]A2\\p[0]\\s[10]\\n[150]B2\\e")
    end)

    test("5.6: スポット未設定アクターのスポット0フォールバック共有で復旧する", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            { type = "spot", actor = a, spot = 0 },
            group(a, { talk(a, "\\s[0]A1") }),
            group(b, { talk(b, "\\s[10]B1") }), -- うにゅうは未設定 → spot 0
            group(a, { talk(a, "A2") }),
        }, config, {})
        expect(result):toBe("\\p[0]\\s[0]A1\\p[0]\\n[150]\\s[10]B1\\p[0]\\s[0]\\n[150]A2\\e")
    end)

    test("2.11/5.1: 専用スポットのアクターがスコープ切替で変えた表示は次のビルドで巻き戻さない", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local APPEARANCE = require("pasta.shiori.appearance")
        local appearance = APPEARANCE.new()
        local actor_spots = { ["さくら"] = 0, ["うにゅう"] = 1 }
        local r1 = BUILDER.build({
            group(a, { talk(a, "\\s[0]A1\\1\\s[10]X\\0\\s[5]Y") }),
            group(b, { talk(b, "B1") }),
        }, config, actor_spots, appearance)
        expect(r1):toBe("\\p[0]\\s[0]A1\\1\\s[10]X\\0\\s[5]Y\\p[1]B1\\e")
        local r2 = BUILDER.build({
            group(a, { talk(a, "A2") }),
            group(b, { talk(b, "B2") }),
        }, config, actor_spots, appearance)
        expect(r2):toBe("\\p[0]A2\\p[1]B2\\e")
    end)

    test("2.11/5.1: タグごとに別トークンでも、スコープ切替で変えた表示を次のビルドで巻き戻さない", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local appearance = require("pasta.shiori.appearance").new()
        local actor_spots = { ["さくら"] = 0, ["うにゅう"] = 1 }
        local r1 = BUILDER.build({
            group(a, {
                script(a, "\\s[0]"), talk(a, "A1"), script(a, "\\1"), script(a, "\\s[10]"), talk(a, "X"),
                script(a, "\\0"), script(a, "\\s[5]"), talk(a, "Y"),
            }),
            group(b, { talk(b, "B1") }),
        }, config, actor_spots, appearance)
        expect(r1):toBe("\\p[0]\\s[0]A1\\1\\s[10]X\\0\\s[5]Y\\p[1]B1\\e")
        local r2 = BUILDER.build({
            group(a, { talk(a, "A2") }),
            group(b, { talk(b, "B2") }),
        }, config, actor_spots, appearance)
        expect(r2):toBe("\\p[0]A2\\p[1]B2\\e")
    end)

    test("2.7: 同一の外見状態を渡した後続ビルドで、B の \\p[0] 直後に \\s[10] を復旧する", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local appearance = require("pasta.shiori.appearance").new()
        local actor_spots = { ["さくら"] = 0, ["うにゅう"] = 0 }
        local r1 = BUILDER.build({
            group(b, { talk(b, "\\s[10]B1") }),
            group(a, { talk(a, "\\s[3]A1") }),
        }, config, actor_spots, appearance)
        expect(r1):toBe("\\p[0]\\s[10]B1\\p[0]\\n[150]\\s[3]A1\\e")
        local r2 = BUILDER.build({
            group(b, { talk(b, "B2") }),
        }, config, actor_spots, appearance)
        expect(r2):toBe("\\p[0]\\s[10]B2\\e")
    end)

    test("1.7: タグごとに別トークンの発話でも、スコープ切替後のサーフェス変更をアクターへ記録しない", function()
        local BUILDER, actors = setup()
        local a, k = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { script(a, "\\s[0]"), talk(a, "A1"), script(a, "\\1"), script(a, "\\s[10]"), talk(a, "X") }),
            group(k, { talk(k, "B1") }),
            group(a, { talk(a, "A2") }),
        }, config, { ["さくら"] = 0, ["うにゅう"] = 0 })
        expect(result):toBe("\\p[0]\\s[0]A1\\1\\s[10]X\\p[0]\\n[150]B1\\p[0]\\s[0]\\n[150]A2\\e")
    end)
end)

describe("SAKURA_BUILDER - actor-surface-restore: 着せ替え復旧の統合（Task 4.3）", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function script(actor, text)
        return { type = "sakura_script", actor = actor, text = text }
    end
    local config = { spot_newlines = 1.5 }

    test("3.2/5.6: スポット0フォールバック共有の交代で、サーフェス復旧の後に着せ替え復旧が \\p[0] 直後へ出る", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { script(a, "\\s[0]"), script(a, "\\![bind,帽子,麦わら,1]"), talk(a, "A1") }),
            group(b, { script(b, "\\s[10]"), script(b, "\\![bind,帽子,リボン,1]"), talk(b, "B1") }),
            group(a, { talk(a, "A2") }),
        }, config, {})
        expect(result):toBe(
            "\\p[0]\\s[0]\\![bind,帽子,麦わら,1]A1"
            .. "\\p[0]\\s[10]\\![bind,帽子,リボン,1]\\n[150]B1"
            .. "\\p[0]\\s[0]\\![bind-noevent,帽子,麦わら,1]\\n[150]A2\\e")
    end)

    test("3.6: 着せ替え復旧タグだけが出た切替でも段落区切り改行の配置は着せ替えなしの場合と一致する", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local function run(ba, bb)
            local ta = ba and { script(a, ba) } or {}
            ta[#ta + 1] = talk(a, "A1")
            local tb = bb and { script(b, bb) } or {}
            tb[#tb + 1] = talk(b, "B1")
            return BUILDER.build({
                group(a, ta),
                group(b, tb),
                group(a, { { type = "wait", ms = 100 } }), -- 着せ替え復旧タグのみ（一般文字列なし）
                group(b, { talk(b, "B2") }),
                group(a, { talk(a, "A2") }),
            }, config, {})
        end
        local with_bind = run("\\![bind,帽子,麦わら,1]", "\\![bind,帽子,リボン,1]")
        local without_bind = run(nil, nil)
        expect(with_bind):toBe(
            "\\p[0]\\![bind,帽子,麦わら,1]A1\\p[0]\\![bind,帽子,リボン,1]\\n[150]B1"
            .. "\\p[0]\\![bind-noevent,帽子,麦わら,1]\\_w[100]"
            .. "\\p[0]\\![bind-noevent,帽子,リボン,1]\\n[150]B2"
            .. "\\p[0]\\![bind-noevent,帽子,麦わら,1]\\n[150]A2\\e")
        expect((with_bind:gsub("\\!%[bind[^%]]*%]", ""))):toBe(without_bind)
    end)

    test("5.1: 明示専用スポット・既定未設定で bind を含む会話は復旧タグを出さず、フックなしの出力と一致する（3ビルド）", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local APPEARANCE = require("pasta.shiori.appearance")
        local function run(appearance)
            local actor_spots = { ["さくら"] = 0, ["うにゅう"] = 1 }
            local out = {}
            out[1] = BUILDER.build({
                group(a, { script(a, "\\![bind,帽子,麦わら,1]"), talk(a, "A1") }),
                group(b, { script(b, "\\![bind,服,制服,1]"), talk(b, "B1") }),
                group(a, { talk(a, "A2") }),
            }, config, actor_spots, appearance)
            -- スコープ切替タグ以降の bind と、アクター未指定の生さくらスクリプトの bind（全スポット不明化）
            out[2] = BUILDER.build({
                group(a, { script(a, "\\1"), script(a, "\\![bind,帽子,リボン,1]"), talk(a, "A3") }),
                { type = "raw_script", text = "\\0\\![bind,帽子,,0]" },
                group(b, { talk(b, "B2") }),
                group(a, { talk(a, "A4") }),
            }, config, actor_spots, appearance)
            -- トグル bind（数値省略）
            out[3] = BUILDER.build({
                group(b, { talk(b, "B3") }),
                group(a, { script(a, "\\![bind,帽子,麦わら]"), talk(a, "A5") }),
            }, config, actor_spots, appearance)
            return out
        end
        local hooked = run(APPEARANCE.new())
        expect(hooked[1]):toBe("\\p[0]\\![bind,帽子,麦わら,1]A1\\p[1]\\![bind,服,制服,1]B1\\p[0]\\n[150]A2\\e")
        expect(hooked[2]):toBe("\\p[0]\\1\\![bind,帽子,リボン,1]A3\\0\\![bind,帽子,,0]\\p[1]B2\\p[0]\\n[150]A4\\e")
        expect(hooked[3]):toBe("\\p[1]B3\\p[0]\\![bind,帽子,麦わら]A5\\e")

        -- フックなし（restore が常に空文字列）の出力とバイト等価
        local real_restore = APPEARANCE.restore
        local stub_calls = 0
        APPEARANCE.restore = function()
            stub_calls = stub_calls + 1
            return ""
        end
        local ok, unhooked = pcall(run, APPEARANCE.new())
        APPEARANCE.restore = real_restore
        assert(ok, unhooked)
        expect(stub_calls > 0):toBe(true) -- スタブがビルダーから実際に呼ばれたこと
        for i = 1, 3 do
            expect(hooked[i]):toBe(unhooked[i])
        end
    end)
end)

-- ============================================================================
-- paragraph-break-tag-only-talk: 字のある talk の段落区切りの特性化（Task 1.2）
-- 本仕様で変えない部分（Requirement 3.1・3.3・4.3）を現行コードで固定する。
-- 戻り時の改行 1 つ・\c・clear_spot・終端での破棄・改行幅は sakura-script-newline の Task 3.1/3.2 の
-- 「完全遅延」「保留破棄・フラッシュ・クリア」の各テストが押さえている。
-- ============================================================================

describe("SAKURA_BUILDER - paragraph-break-tag-only-talk: タグを含む字のある talk（Task 1.2）", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end

    test("3.1/4.3: 保留中の改行は talk の先頭のタグより前に 1 つだけ出る", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "A1") }),
            group(b, { talk(b, "\\s[10]B1") }),
            group(a, { talk(a, "\\s[5]A2") }),
            group(b, { talk(b, "B2\\s[11]B3") }),
        }, { spot_newlines = 1.5 }, { ["さくら"] = 0, ["うにゅう"] = 1 })
        expect(result):toBe("\\p[0]A1\\p[1]\\s[10]B1\\p[0]\\n[150]\\s[5]A2\\p[1]\\n[150]B2\\s[11]B3\\e")
    end)
end)

-- ============================================================================
-- paragraph-break-tag-only-talk: 字の無いトークンを段落区切りの状態から外す（Task 5.1）
-- 既定設定（script_wait_normal=50 → 実効ウェイト 0）で句読点の無いテキストを使い、ウェイトを入れない。
-- スポットは さくら=0・うにゅう=1 を明示する（スポット共有の復旧タグを出さない）。
-- ============================================================================

describe("SAKURA_BUILDER - paragraph-break-tag-only-talk: 字の無いトークン（Task 5.1）", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function script(actor, text)
        return { type = "sakura_script", actor = actor, text = text }
    end
    local config = { spot_newlines = 1.5 }
    local function spots()
        return { ["さくら"] = 0, ["うにゅう"] = 1 }
    end

    -- 申し送りの最小例（6.1）
    test("4.1: タグだけの talk の後のアクターの初回の手番に改行を出さない（申し送り a）", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(b, { talk(b, "\\s[10]\\1\\![move,-353,,,0,base,base]") }),
            group(a, { talk(a, "A1") }),
            group(b, { talk(b, "\\s[11]B1") }),
        }, config, spots())
        expect(result):toBe("\\p[1]\\s[10]\\1\\![move,-353,,,0,base,base]\\p[0]A1\\p[1]\\s[11]B1\\e")
    end)

    test("4.2: タグだけの talk の手番に改行を出さず、後の字のある手番には出す（申し送り b）", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "A1") }),
            group(b, { talk(b, "B1") }),
            group(a, { talk(a, "\\s[1000]\\![bind,腕,組み,1]") }),
            group(b, { talk(b, "B2") }),
            group(a, { talk(a, "A2") }),
        }, config, spots())
        expect(result):toBe(
            "\\p[0]A1\\p[1]B1\\p[0]\\s[1000]\\![bind,腕,組み,1]\\p[1]\\n[150]B2\\p[0]\\n[150]A2\\e")
    end)

    -- 字の境界（6.2）。共通の並び: さくら A1 → うにゅう X → さくら A2 → うにゅう B2。
    -- X が字を出すなら B2 の前に \n[150] が出る。X の出力は X の文字列そのまま（既定設定でウェイト 0）。
    local boundary_cases = {
        { "talk", "\\s[1000]", false, "1.2" },
        { "talk", "\\s[1000]\\![bind,腕,組み,1]", false, "1.2" },
        { "talk", "\\_w[500]", false, "1.2" },
        { "talk", "\\n", false, "1.2" },
        { "talk", "\\n[150]", false, "1.2" },
        { "talk", "\\1\\![move,-353,,,0,base,base]", false, "1.2" },
        { "talk", "\\\\", true, "1.3" },
        { "talk", "\\%", true, "1.3" },
        { "talk", "\\_u[0x3042]", true, "1.4" },
        { "talk", "\\_m[0x41]", true, "1.4" },
        { "talk", "\\&[amp]", true, "1.4" },
        { "talk", "\\s[0] ", true, "1.5" },
        { "talk", "\\s[0]　", true, "1.5" },
        { "talk", "\\s[0]\t", true, "1.5" },
        { "talk", "\\q[はい,OnYes]", false, "1.6" },
        { "talk", "\\q[a\\]b,OnX]", false, "1.6/7.6" },
        { "talk", "\\あ", true, "1.7" },
        { "talk", "\\nHello", true, "1.1/7.4" },
        { "talk", "\\w9OK", true, "1.1/7.4" },
        { "talk", "\\w0", true, "1.1/7.4" },
        { "talk", "\\_?\\s[1]\\_?", true, "1.10" },
        { "talk", "\\_?\\_?", false, "1.10" },
        { "talk", "\\_?", false, "1.10" },
        { "sakura_script", "\\_u[0x3042]", true, "1.9" },
        { "sakura_script", "\\&[amp]", true, "1.9" },
        { "sakura_script", "\\s[1]\\_u[0x3042]", true, "1.9" },
        { "sakura_script", "\\_?abc\\_?", true, "1.10" },
        { "sakura_script", "\\s[5]", false, "1.9/3.4" },
    }
    for _, c in ipairs(boundary_cases) do
        local kind, text, has_char, req = c[1], c[2], c[3], c[4]
        test(string.format("%s: %s %q は%s", req, kind, text, has_char and "字あり" or "字なし"), function()
            local BUILDER, actors = setup()
            local a, b = actors.sakura, actors.kero
            local x = kind == "talk" and talk(b, text) or script(b, text)
            local result = BUILDER.build({
                group(a, { talk(a, "A1") }),
                group(b, { x }),
                group(a, { talk(a, "A2") }),
                group(b, { talk(b, "B2") }),
            }, config, spots())
            expect(result):toBe("\\p[0]A1\\p[1]" .. text .. "\\p[0]\\n[150]A2\\p[1]"
                .. (has_char and "\\n[150]" or "") .. "B2\\e")
        end)
    end

    test("1.9: 文字を表示する sakura_script の直前に保留中の改行を出す", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "A1") }),
            group(b, { talk(b, "B1") }),
            group(a, { script(a, "\\_u[0x3042]") }),
        }, config, spots())
        expect(result):toBe("\\p[0]A1\\p[1]B1\\p[0]\\n[150]\\_u[0x3042]\\e")
    end)

    -- 保留中の改行（6.3）
    test("2.4: 字の無い talk と wait の後の字のある talk の直前に改行を出す", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "A1") }),
            group(b, { talk(b, "B1") }),
            group(a, { talk(a, "\\s[5]"), { type = "wait", ms = 100 }, talk(a, "A2") }),
        }, config, spots())
        expect(result):toBe("\\p[0]A1\\p[1]B1\\p[0]\\s[5]\\_w[100]\\n[150]A2\\e")
    end)

    test("2.5: 字の無い talk だけで終端すると改行を出さない", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "A1") }),
            group(b, { talk(b, "B1") }),
            group(a, { talk(a, "\\s[5]") }),
        }, config, spots())
        expect(result):toBe("\\p[0]A1\\p[1]B1\\p[0]\\s[5]\\e")
    end)

    test("2.5/2.2: 字の無い talk だけで離脱し、戻った字のある手番の前に改行を 1 つだけ出す", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "A1") }),
            group(b, { talk(b, "B1") }),
            group(a, { talk(a, "\\s[5]") }),
            group(b, { talk(b, "B2") }),
            group(a, { talk(a, "A2") }),
        }, config, spots())
        expect(result):toBe("\\p[0]A1\\p[1]B1\\p[0]\\s[5]\\p[1]\\n[150]B2\\p[0]\\n[150]A2\\e")
    end)
end)

-- ============================================================================
-- choice-line-layout: 選択肢の行の配置
-- 選択肢の前と、選択肢の後の台詞の前に出す行の区切りを完全一致で固定する。
-- 既定設定（script_wait_normal=50 → 実効ウェイト 0）で句読点の無いテキストを使い、ウェイトを入れない。
-- スポットは さくら=0・うにゅう=1 を明示する（スポット共有の復旧タグを出さない）。
-- ============================================================================

describe("SAKURA_BUILDER - choice-line-layout: 選択肢の行の配置", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function spots()
        return { ["さくら"] = 0, ["うにゅう"] = 1 }
    end

    -- 本体を直す前に置いた特性化（台詞の中の \q は talk として通り、選択肢の区切りの対象にならない）
    test("3.4: 台詞の中に手で書いた \\q[…]・\\__q[…] だけの出力は変わらない", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local result = BUILDER.build({
            group(a, {
                talk(a, "どれ"),
                talk(a, "\\q[はい,OnYes]"),
                talk(a, "\\q[いいえ,OnNo]"),
                talk(a, "つづき"),
            }),
        }, {}, spots())
        expect(result):toBe("\\p[0]どれ\\q[はい,OnYes]\\q[いいえ,OnNo]つづき\\e")

        -- 1 つの台詞に改行つきで書く形と、\__q の形
        local inline = BUILDER.build({
            group(a, { talk(a, "どれ\\n\\q[はい,OnYes]\\n\\__q[OnNo]いいえ\\__q") }),
        }, {}, spots())
        expect(inline):toBe("\\p[0]どれ\\n\\q[はい,OnYes]\\n\\__q[OnNo]いいえ\\__q\\e")
    end)

    -- 選択肢のトークンと、その出力（タグの中身は変えない。3.1）
    local function choice(display)
        return { type = "choice", display = display, target = display .. "先" }
    end
    local Q1 = "\\![*]\\q[甲,甲先]"
    local Q2 = "\\![*]\\q[乙,乙先]"

    test("2.1/2.2/2.9/2.11: 字 → 選択肢 → 選択肢で \\n[150] と \\n が選択肢の直前に 1 つずつ出る", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local result = BUILDER.build({
            group(a, { talk(a, "どれ"), choice("甲"), choice("乙") }),
        }, {}, spots())
        expect(result):toBe("\\p[0]どれ\\n[150]" .. Q1 .. "\\n" .. Q2 .. "\\e")
    end)

    test("2.3: 吹き出しの最初の選択肢は区切らない（選択肢だけ・clear の直後・別の吹き出しにだけ字）", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero

        local only = BUILDER.build({ group(a, { choice("甲") }) }, {}, spots())
        expect(only):toBe("\\p[0]" .. Q1 .. "\\e")

        local cleared = BUILDER.build({
            group(a, { talk(a, "どれ"), { type = "clear" }, choice("甲") }),
        }, {}, spots())
        expect(cleared):toBe("\\p[0]どれ\\c" .. Q1 .. "\\e")

        local other = BUILDER.build({
            group(b, { talk(b, "はい") }),
            group(a, { choice("甲") }),
        }, {}, spots())
        expect(other):toBe("\\p[1]はい\\p[0]" .. Q1 .. "\\e")
    end)

    test("2.4: 表情・ウェイト・choice_timeout・raw_script が挟まっても区切りの数と幅は変わらない", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local function noise()
            return { type = "surface", id = 5 }, { type = "wait", ms = 100 },
                { type = "choice_timeout", seconds = 10 }, { type = "raw_script", text = "\\![open,notepad]" }
        end
        local NOISE = "\\s[5]\\_w[100]\\![set,choicetimeout,10000]\\![open,notepad]"

        local tokens = { talk(a, "どれ") }
        for _, t in ipairs({ noise() }) do tokens[#tokens + 1] = t end
        tokens[#tokens + 1] = choice("甲")
        for _, t in ipairs({ noise() }) do tokens[#tokens + 1] = t end
        tokens[#tokens + 1] = choice("乙")

        local result = BUILDER.build({ group(a, tokens) }, {}, spots())
        expect(result):toBe("\\p[0]どれ" .. NOISE .. "\\n[150]" .. Q1 .. NOISE .. "\\n" .. Q2 .. "\\e")
    end)

    test("2.5/2.11: 選択肢の後の字の直前に \\n が 1 つ出て、続く字の前には出ない", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local result = BUILDER.build({
            group(a, { talk(a, "選んで"), choice("甲"), talk(a, "つづき"), talk(a, "おわり") }),
        }, {}, spots())
        expect(result):toBe("\\p[0]選んで\\n[150]" .. Q1 .. "\\nつづきおわり\\e")
    end)

    test("2.8: 字の後の選択肢の前の幅は choice_newlines で決まり、spot_newlines と互いに影響しない", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero

        local wide = BUILDER.build({
            group(a, { talk(a, "どれ"), choice("甲"), choice("乙") }),
        }, { choice_newlines = 2.0 }, spots())
        expect(wide):toBe("\\p[0]どれ\\n[200]" .. Q1 .. "\\n" .. Q2 .. "\\e")

        -- 換算は spot_newlines と同じ（100 倍して切り捨て）
        local floored = BUILDER.build({
            group(a, { talk(a, "どれ"), choice("甲") }),
        }, { choice_newlines = 1.239 }, spots())
        expect(floored):toBe("\\p[0]どれ\\n[123]" .. Q1 .. "\\e")

        -- 話し手が戻った吹き出しで、選択肢の前（choice_newlines）と字の前（spot_newlines）が並ぶ並び
        local function returning()
            return {
                group(a, { talk(a, "やあ") }),
                group(b, { talk(b, "はい") }),
                group(a, { choice("甲"), talk(a, "どう") }),
            }
        end
        local spot_only = BUILDER.build(returning(), { spot_newlines = 3.0 }, spots())
        expect(spot_only):toBe("\\p[0]やあ\\p[1]はい\\p[0]\\n[150]" .. Q1 .. "\\n[300]どう\\e")
        local choice_only = BUILDER.build(returning(), { choice_newlines = 3.0 }, spots())
        expect(choice_only):toBe("\\p[0]やあ\\p[1]はい\\p[0]\\n[300]" .. Q1 .. "\\n[150]どう\\e")
    end)

    test("2.7: 台詞の末尾の \\n と newline のトークンは、区切りと並んで両方出る", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local result = BUILDER.build({
            group(a, { talk(a, "どれ\\n"), choice("甲"), { type = "newline", n = 1 }, choice("乙") }),
        }, {}, spots())
        expect(result):toBe("\\p[0]どれ\\n\\n[150]" .. Q1 .. "\\n\\n" .. Q2 .. "\\e")
    end)

    test("2.5/2.11: 話し手の切り替えと重なっても区切りは 1 か所に 1 つだけ出る", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero

        -- 字・選択肢 → 別の話し手 → 戻った字（字の前は spot_newlines の区切りだけ）
        local back_to_text = BUILDER.build({
            group(a, { talk(a, "どれ"), choice("甲") }),
            group(b, { talk(b, "はやく") }),
            group(a, { talk(a, "うーん") }),
        }, {}, spots())
        expect(back_to_text):toBe("\\p[0]どれ\\n[150]" .. Q1 .. "\\p[1]はやく\\p[0]\\n[150]うーん\\e")

        -- 字 → 別の話し手 → 戻った選択肢・字（選択肢は保留に触らず、保留は次の字の直前で出る）
        local back_to_choice = BUILDER.build({
            group(a, { talk(a, "やあ") }),
            group(b, { talk(b, "はい") }),
            group(a, { choice("甲"), talk(a, "どう") }),
        }, {}, spots())
        expect(back_to_choice):toBe("\\p[0]やあ\\p[1]はい\\p[0]\\n[150]" .. Q1 .. "\\n[150]どう\\e")

        -- 選択肢だけ → 別の話し手 → 戻った字（字が無かった吹き出しなので保留は立たず、\n が出る）
        local choice_only = BUILDER.build({
            group(a, { choice("甲") }),
            group(b, { talk(b, "はやく") }),
            group(a, { talk(a, "うーん") }),
        }, {}, spots())
        expect(choice_only):toBe("\\p[0]" .. Q1 .. "\\p[1]はやく\\p[0]\\nうーん\\e")
    end)

    test("2.2/2.5: 話し手なしのグループの選択肢はスポット 0 の吹き出しに出たものとして数える", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero

        local two = BUILDER.build({ group(nil, { choice("甲"), choice("乙") }) }, {}, spots())
        expect(two):toBe(Q1 .. "\\n" .. Q2 .. "\\e")

        local then_spot0 = BUILDER.build({
            group(nil, { choice("甲") }),
            group(a, { talk(a, "つづき") }),
        }, {}, spots())
        expect(then_spot0):toBe(Q1 .. "\\p[0]\\nつづき\\e")

        -- スポット 1 は別の吹き出しなので区切らない
        local then_spot1 = BUILDER.build({
            group(nil, { choice("甲") }),
            group(b, { talk(b, "つづき") }),
        }, {}, spots())
        expect(then_spot1):toBe(Q1 .. "\\p[1]つづき\\e")
    end)

    test("2.10: choice_newlines に数値でない値を渡した結果は spot_newlines に渡したときと同じ", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        -- 幅つきの改行を 1 つだけ出す並びを、設定ごとに 1 つずつ用意する
        local function build_with(key, value)
            local tokens
            if key == "choice_newlines" then
                tokens = { group(a, { talk(a, "どれ"), choice("甲") }) }
            else
                tokens = { group(a, { talk(a, "A1") }), group(b, { talk(b, "B1") }), group(a, { talk(a, "A2") }) }
            end
            local ok, result = pcall(BUILDER.build, tokens, { [key] = value }, spots())
            return ok, ok and result:match("\\n%[%d+%]") or nil
        end

        local expected = {
            { "abc", false, nil },       -- 計算できない文字列は Lua のエラー
            { true, false, nil },
            { "2", true, "\\n[200]" },   -- 数値に読める文字列は Lua が数値として計算する
            { false, true, "\\n[150]" }, -- 偽は既定の 1.5
        }
        for _, e in ipairs(expected) do
            local value, want_ok, want_tag = e[1], e[2], e[3]
            local spot_ok, spot_tag = build_with("spot_newlines", value)
            local choice_ok, choice_tag = build_with("choice_newlines", value)
            expect(spot_ok):toBe(want_ok)
            expect(spot_tag):toBe(want_tag)
            expect(choice_ok):toBe(spot_ok)
            expect(choice_tag):toBe(spot_tag)
        end
    end)

    test("選択肢を含む並びで、既定のバッファと buf.new_fallback の出力が一致する", function()
        local BUILDER, actors = setup()
        local buf = require("pasta.buf")
        local a, b = actors.sakura, actors.kero
        local function make_tokens()
            return {
                group(a, { talk(a, "どれ"), choice("甲"), choice("乙"), talk(a, "つづき") }),
                group(b, { talk(b, "はやく"), choice("甲") }),
                group(a, { talk(a, "うーん") }),
            }
        end

        local native = BUILDER.build(make_tokens(), {}, spots())
        local fallback = BUILDER.build(make_tokens(), { buffer_factory = buf.new_fallback }, spots())

        expect(native):toBe("\\p[0]どれ\\n[150]" .. Q1 .. "\\n" .. Q2 .. "\\nつづき"
            .. "\\p[1]はやく\\n[150]" .. Q1 .. "\\p[0]\\n[150]うーん\\e")
        expect(fallback):toBe(native)
    end)

    -- ------------------------------------------------------------------------
    -- Task 2.3: 台詞に書いた \c を吹き出しの消去として数える（7.1〜7.4）
    -- DSL の台詞の行に書いた \c は、テキストが \c だけの sakura_script になる。
    -- ------------------------------------------------------------------------
    local function script(actor, text)
        return { type = "sakura_script", actor = actor, text = text }
    end

    test("2.3/7.1: 字 → 台詞に書いた \\c → 選択肢 → 選択肢で、最初の選択肢は区切らず 2 つ目の前に \\n が出る", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local result = BUILDER.build({
            group(a, { talk(a, "まえおき"), script(a, "\\c"), choice("甲"), choice("乙") }),
        }, {}, spots())
        expect(result):toBe("\\p[0]まえおき\\c" .. Q1 .. "\\n" .. Q2 .. "\\e")
    end)

    -- 既存の「\c は保留改行を破棄し…has-text を偽へリセットする」の並びを、clear の代わりに \c で書く
    test("7.2: 台詞に書いた \\c は保留を捨て、その吹き出しの字の記録を消す（\\n[150] が出ない）", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero

        -- 話し手が戻った直後に \c と台詞（保留を捨てる）
        local discard = BUILDER.build({
            group(a, { talk(a, "やあ") }),
            group(b, { talk(b, "はい") }),
            group(a, { script(a, "\\c"), talk(a, "つぎ") }),
        }, {}, spots())
        expect(discard):toBe("\\p[0]やあ\\p[1]はい\\p[0]\\cつぎ\\e")

        -- 戻った吹き出しを \c で消し、字を表示しない別の話し手を挟んで、もう一度戻る（字の記録を消す）
        local revisit = BUILDER.build({
            group(a, { talk(a, "やあ") }),
            group(b, { talk(b, "はい") }),
            group(a, { script(a, "\\c") }),
            group(b, { { type = "surface", id = 6 } }),
            group(a, { talk(a, "つぎ") }),
        }, {}, spots())
        expect(revisit):toBe("\\p[0]やあ\\p[1]はい\\p[0]\\c\\p[1]\\s[6]\\p[0]つぎ\\e")

        -- 字の後に \c で消した吹き出しへ、別の話し手の後で戻る
        local reset = BUILDER.build({
            group(a, { talk(a, "やあ"), script(a, "\\c") }),
            group(b, { talk(b, "はい") }),
            group(a, { talk(a, "つぎ") }),
        }, {}, spots())
        expect(reset):toBe("\\p[0]やあ\\c\\p[1]はい\\p[0]つぎ\\e")
    end)

    test("7.3: 引数つきの \\c・\\x・\\C・エスケープ・囲みの中・raw_script の \\c は状態を変えない", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        -- { 型, テキスト, そのトークンが字を出すか（字の判定は今のまま） }
        local cases = {
            { "sakura_script", "\\c[char,1]", false },
            { "sakura_script", "\\c[line,1]", false },
            { "sakura_script", "\\x", false },
            { "sakura_script", "\\C", false },
            { "sakura_script", "\\\\c", false },      -- エスケープ `\\` と c（sakura_script はタグだけを字に数える）
            { "sakura_script", "\\_?\\c\\_?", true }, -- 囲みの中（中身のある囲みは字）
            { "talk", "\\c[char,1]", false },
            { "talk", "\\\\c", true },
            { "raw_script", "\\c", false },
        }
        for _, case in ipairs(cases) do
            local text, shows_text = case[2], case[3]
            local function x()
                return { type = case[1], actor = a, text = text }
            end

            -- 選択肢の配置: 字の後のままなので、選択肢の前に \n[150] が出る
            local before_choice = BUILDER.build({
                group(a, { talk(a, "どれ"), x(), choice("甲") }),
            }, {}, spots())
            expect(before_choice):toBe("\\p[0]どれ" .. text .. "\\n[150]" .. Q1 .. "\\e")

            -- 話し手の切り替えの区切り: 保留は捨てられず、次の字の直前に \n[150] が出る
            local returned = BUILDER.build({
                group(a, { talk(a, "やあ") }),
                group(b, { talk(b, "はい") }),
                group(a, { x(), talk(a, "つぎ") }),
            }, {}, spots())
            local tail = shows_text and ("\\n[150]" .. text .. "つぎ") or (text .. "\\n[150]つぎ")
            expect(returned):toBe("\\p[0]やあ\\p[1]はい\\p[0]" .. tail .. "\\e")
        end

        -- グループの外の raw_script も数えない
        local top = BUILDER.build({
            group(a, { talk(a, "どれ") }),
            { type = "raw_script", text = "\\c" },
            group(a, { choice("甲") }),
        }, {}, spots())
        expect(top):toBe("\\p[0]どれ\\c\\n[150]" .. Q1 .. "\\e")
    end)

    test("7.4: talk の「まえ \\c あと」は、まえに今までどおりの区切りを当て、あとを消した吹き出しの最初の字に数える", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero

        -- 保留があれば「まえ」の前（トークンの前）に \n[150] が出る。出力する文字列は切らない
        local pending = BUILDER.build({
            group(a, { talk(a, "やあ") }),
            group(b, { talk(b, "はい") }),
            group(a, { talk(a, "まえ\\cあと") }),
        }, {}, spots())
        expect(pending):toBe("\\p[0]やあ\\p[1]はい\\p[0]\\n[150]まえ\\cあと\\e")

        -- 別の話し手の後で戻ると、最後の \c より後ろに字があるときだけ \n[150] が出る
        local function returning(text)
            return BUILDER.build({
                group(a, { talk(a, text) }),
                group(b, { talk(b, "はい") }),
                group(a, { talk(a, "つぎ") }),
            }, {}, spots())
        end
        expect(returning("まえ\\cあと")):toBe("\\p[0]まえ\\cあと\\p[1]はい\\p[0]\\n[150]つぎ\\e")
        expect(returning("まえ\\c")):toBe("\\p[0]まえ\\c\\p[1]はい\\p[0]つぎ\\e")
        expect(returning("まえ\\cなか\\c")):toBe("\\p[0]まえ\\cなか\\c\\p[1]はい\\p[0]つぎ\\e")
        expect(returning("\\c\\cあと")):toBe("\\p[0]\\c\\cあと\\p[1]はい\\p[0]\\n[150]つぎ\\e")

        -- 選択肢の後: 「まえ」の前に \n が出る。「あと」は字なので、次の選択肢の前は \n[150]
        local after_choice = BUILDER.build({
            group(a, { choice("甲"), talk(a, "まえ\\cあと"), choice("乙") }),
        }, {}, spots())
        expect(after_choice):toBe("\\p[0]" .. Q1 .. "\\nまえ\\cあと\\n[150]" .. Q2 .. "\\e")

        -- \c より前に字が無ければ、選択肢の後でも区切りを出さない
        local no_lead = BUILDER.build({
            group(a, { choice("甲"), talk(a, "\\cあと"), choice("乙") }),
        }, {}, spots())
        expect(no_lead):toBe("\\p[0]" .. Q1 .. "\\cあと\\n[150]" .. Q2 .. "\\e")

        -- \c より後ろに字が無ければ、次の選択肢は消した吹き出しの最初の選択肢
        local no_trail = BUILDER.build({
            group(a, { talk(a, "まえ\\c"), choice("甲") }),
        }, {}, spots())
        expect(no_trail):toBe("\\p[0]まえ\\c" .. Q1 .. "\\e")
    end)
end)

-- ============================================================================
-- 選択肢の後の 2 か所の区切り（vertical-balloon-lines 5.1〜5.5・5.7）
-- ============================================================================

describe("SAKURA_BUILDER - vertical-balloon-lines: after_choice_newlines", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function choice(display)
        return { type = "choice", display = display, target = display .. "先" }
    end
    local function spots()
        return { ["さくら"] = 0, ["うにゅう"] = 1 }
    end
    local Q1 = "\\![*]\\q[甲,甲先]"
    local Q2 = "\\![*]\\q[乙,乙先]"

    -- 字 → 選択肢 → 選択肢 → 字
    local function sequence(a)
        return { group(a, { talk(a, "どれ"), choice("甲"), choice("乙"), talk(a, "つづき") }) }
    end

    test("5.1/5.2: 2.0 で選択肢の後の選択肢の前と台詞の前が \\n[200]", function()
        local BUILDER, actors = setup()
        local result = BUILDER.build(sequence(actors.sakura), { after_choice_newlines = 2.0 }, spots())
        expect(result):toBe("\\p[0]どれ\\n[150]" .. Q1 .. "\\n[200]" .. Q2 .. "\\n[200]つづき\\e")
    end)

    test("5.3: 書かれていない・1.0 のときは今と同じ \\n", function()
        local BUILDER, actors = setup()
        local expected = "\\p[0]どれ\\n[150]" .. Q1 .. "\\n" .. Q2 .. "\\nつづき\\e"
        expect(BUILDER.build(sequence(actors.sakura), {}, spots())):toBe(expected)
        expect(BUILDER.build(sequence(actors.sakura), { after_choice_newlines = 1.0 }, spots())):toBe(expected)
    end)

    test("5.5: 換算は choice_newlines と同じ（100 倍して切り捨て）", function()
        local BUILDER, actors = setup()
        local result = BUILDER.build(sequence(actors.sakura), { after_choice_newlines = 1.239 }, spots())
        expect(result):toBe("\\p[0]どれ\\n[150]" .. Q1 .. "\\n[123]" .. Q2 .. "\\n[123]つづき\\e")
    end)

    test("5.5: 数値でない値を渡した結果は choice_newlines に渡したときと同じ", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local function build_with(config)
            return pcall(BUILDER.build, sequence(a), config, spots())
        end
        for _, value in ipairs({ "2.0", true, {} }) do
            local after_ok, after_res = build_with({ after_choice_newlines = value })
            local choice_ok, choice_res = build_with({ choice_newlines = value })
            expect(after_ok):toBe(choice_ok)
            expect(after_ok and after_res:gsub("%[%d+%]", "") or true)
                :toBe(choice_ok and choice_res:gsub("%[%d+%]", "") or true)
        end
    end)

    test("5.4: 話し手が戻った吹き出しでは spot_newlines だけで、after_choice_newlines を重ねない", function()
        local BUILDER, actors = setup()
        local a, b = actors.sakura, actors.kero
        local result = BUILDER.build({
            group(a, { talk(a, "どれ"), choice("甲") }),
            group(b, { talk(b, "はい") }),
            group(a, { talk(a, "どう") }),
        }, { after_choice_newlines = 2.0, spot_newlines = 3.0 }, spots())
        expect(result):toBe("\\p[0]どれ\\n[150]" .. Q1 .. "\\p[1]はい\\p[0]\\n[300]どう\\e")
    end)

    test("5.7: 字の後の選択肢の前（choice_newlines）は after_choice_newlines で変わらない", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local result = BUILDER.build({
            group(a, { talk(a, "どれ"), choice("甲") }),
        }, { after_choice_newlines = 2.0, choice_newlines = 3.0 }, spots())
        expect(result):toBe("\\p[0]どれ\\n[300]" .. Q1 .. "\\e")
    end)
end)

-- ============================================================================
-- 立ち位置の警告を未登録アクターで重ねない（failure-output-unification 5.4・6.3）
-- ============================================================================

describe("SAKURA_BUILDER - 未登録アクターの actor_spots fallback", function()
    --- 警告を記録するログを差し込んで sakura_builder を読み直し、body を実行する（後で元に戻す）
    local function with_warns(body)
        local saved_log = package.loaded["@pasta_log"]
        local saved_builder = package.loaded["pasta.shiori.sakura_builder"]
        local warns = {}
        local noop = function() end
        package.loaded["@pasta_log"] = {
            trace = noop, debug = noop, info = noop, error = noop,
            warn = function(msg) table.insert(warns, msg) end,
        }
        package.loaded["pasta.shiori.sakura_builder"] = nil
        local ok, err = pcall(body, require("pasta.shiori.sakura_builder"), warns)
        package.loaded["@pasta_log"] = saved_log
        package.loaded["pasta.shiori.sakura_builder"] = saved_builder
        if not ok then error(err, 0) end
    end

    local function group(actor)
        return {
            type = "actor",
            actor = actor,
            tokens = { { type = "talk", actor = actor, text = "A" } },
        }
    end

    test("未登録の印のアクターは立ち位置 0 で、警告を出さない", function()
        with_warns(function(BUILDER, warns)
            local FAILURE = require("pasta.failure")
            local adhoc = { name = "sb_ぱすたー" }
            FAILURE.mark_unregistered_actor(adhoc)
            expect(BUILDER.build({ group(adhoc) }, {}, {})):toBe("\\p[0]A\\e")
            expect(#warns):toBe(0)
        end)
    end)

    test("印の無いアクターは立ち位置が無ければ従来どおり 0 で、警告を 1 行出す", function()
        with_warns(function(BUILDER, warns)
            local actor = { name = "sb_さくら" }
            expect(BUILDER.build({ group(actor) }, {}, {})):toBe("\\p[0]A\\e")
            expect(#warns):toBe(1)
            expect(warns[1]):toBe("actor_spots fallback: 'sb_さくら' -> spot=0")
        end)
    end)
end)

-- ============================================================================
-- 吹き出しごとの列の持ち越し（vertical-balloon-lines 1.5・2.4・4.1〜4.5・4.8）
-- 本物の @pasta_sakura_script を使う。終わりの列の状態はテスト用の columns_out で覗く
-- ============================================================================

describe("SAKURA_BUILDER - vertical-balloon-lines: 列の持ち越し", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function choice(display)
        return { type = "choice", display = display, target = display .. "先" }
    end
    --- budoux の並びを持つアクター 2 人（名前は spots() のスポットに合わせる）
    local function actors_with(budoux, extra)
        local a = { name = "さくら", spot = 0, budoux = budoux }
        local b = { name = "うにゅう", spot = 1, budoux = budoux }
        for k, v in pairs(extra or {}) do
            a[k] = v
            b[k] = v
        end
        return a, b
    end
    local function spots()
        return { ["さくら"] = 0, ["うにゅう"] = 1 }
    end
    --- 組み立てて、出力と終わりの列の状態の表を返す
    local function build(tokens, config)
        local BUILDER = setup()
        config = config or {}
        config.columns_out = {}
        local out = BUILDER.build(tokens, config, spots())
        return out, config.columns_out
    end
    local function expect_column(col, line, width, jump)
        expect(col ~= nil):toBe(true)
        expect(col.line):toBe(line)
        expect(col.width):toBe(width)
        expect(col.jump):toBe(jump or false)
    end

    test("4.1: 表情・ウェイト・choice_timeout を挟んでも続きの字数から数える", function()
        local a = actors_with({ 4 })
        local out, cols = build({
            group(a, {
                talk(a, "あい"),
                { type = "surface", id = 1 },
                { type = "wait", ms = 100 },
                { type = "choice_timeout", seconds = 1 },
                talk(a, "う"),
            }),
        })
        expect(out):toBe("\\p[0]あい\\s[1]\\_w[100]\\![set,choicetimeout,1000]\\nう\\e")
        expect_column(cols[0], 2, 2)
    end)

    test("2.4: act:sakura_script のテキストの中のタグも数える", function()
        local a = actors_with({ 4 })
        local out, cols = build({
            group(a, {
                talk(a, "あい"),
                { type = "sakura_script", actor = a, text = "\\n" },
                talk(a, "う"),
            }),
        })
        expect(out):toBe("\\p[0]あい\\nう\\e")
        expect_column(cols[0], 2, 2)
    end)

    test("4.2: 話し手が戻った吹き出しは次の列（2 列目の幅）から", function()
        local a, b = actors_with({ 4, 20 })
        local out, cols = build({
            group(a, { talk(a, "あい") }),
            group(b, { talk(b, "か") }),
            group(a, { talk(a, "うえおか") }),
        })
        expect(out):toBe("\\p[0]あい\\p[1]か\\p[0]\\n[150]うえおか\\e")
        expect_column(cols[0], 2, 8)
        expect_column(cols[1], 1, 2)
    end)

    test("4.2: 選択肢の前後の区切りの後は次の列から", function()
        local a = actors_with({ 4, 20 })
        local out, cols = build({
            group(a, { talk(a, "あい"), choice("甲"), talk(a, "うえおか") }),
        })
        expect(out):toBe("\\p[0]あい\\n[150]\\![*]\\q[甲,甲先]\\nうえおか\\e")
        expect_column(cols[0], 3, 8)
    end)

    test("4.2: newline トークンは n 列進める", function()
        local a = actors_with({ 4, 20 })
        local out, cols = build({
            group(a, { talk(a, "あい"), { type = "newline", n = 2 }, talk(a, "うえおか") }),
        })
        expect(out):toBe("\\p[0]あい\\n\\nうえおか\\e")
        expect_column(cols[0], 3, 8)
    end)

    test("4.3: clear の後は 1 列目の幅から", function()
        local a = actors_with({ 20, 4 })
        local out, cols = build({
            group(a, {
                talk(a, "あい"),
                { type = "newline", n = 1 },
                { type = "clear" },
                talk(a, "うえおか"),
            }),
        })
        expect(out):toBe("\\p[0]あい\\n\\cうえおか\\e")
        expect_column(cols[0], 1, 8)
    end)

    test("4.3: clear_spot は全スポットの表を捨て、次は 1 列目から", function()
        local a, b = actors_with({ 20, 4 })
        local out, cols = build({
            group(a, { talk(a, "あい"), { type = "newline", n = 1 } }),
            group(b, { talk(b, "か"), { type = "newline", n = 1 } }),
            { type = "clear_spot" },
            group(a, { talk(a, "うえおか") }),
        })
        expect(out):toBe("\\p[0]あい\\n\\p[1]か\\n\\p[0]うえおか\\e")
        expect_column(cols[0], 1, 8)
        expect(cols[1]):toBe(nil)
    end)

    test("4.4: スポット 0 と 1 を行き来しても、それぞれの続きから数える", function()
        local a, b = actors_with({ 20 })
        local _, cols = build({
            group(a, { talk(a, "あい") }),
            group(b, { talk(b, "かきくけ"), { type = "newline", n = 1 }, talk(b, "こ") }),
            group(a, { talk(a, "う") }),
            group(b, { talk(b, "さし") }),
        })
        expect_column(cols[0], 2, 2)
        expect_column(cols[1], 3, 4)
    end)

    test("4.5: 2 回目の出力は 1 列目から（表は出力ごとに作って捨てる）", function()
        local a, b = actors_with({ 4 })
        local function tokens()
            return {
                group(a, { talk(a, "あい") }),
                group(b, { talk(b, "か") }),
                group(a, { talk(a, "う") }),
            }
        end
        local first, cols1 = build(tokens())
        local second, cols2 = build(tokens())
        expect(second):toBe(first)
        expect_column(cols1[0], 2, 2)
        expect_column(cols2[0], 2, 2)
    end)

    test("4.5: columns_out を渡さなくても出力は同じ", function()
        local BUILDER = setup()
        local a = actors_with({ 4 })
        local tokens = { group(a, { talk(a, "あい"), { type = "surface", id = 1 }, talk(a, "う") }) }
        expect(BUILDER.build(tokens, {}, spots())):toBe("\\p[0]あい\\s[1]\\nう\\e")
    end)

    test("4.8: 選択肢の表示の文字は数えない", function()
        local a = actors_with({ 4, 20 })
        local _, cols = build({
            group(a, { talk(a, "あい"), choice("ながいながいせんたくし") }),
        })
        expect_column(cols[0], 2, 0)
    end)

    test("4.1: raw_script は列の状態に触らない", function()
        local a = actors_with({ 4 })
        local out, cols = build({
            group(a, { talk(a, "あい"), { type = "raw_script", text = "\\n\\c" }, talk(a, "う") }),
        })
        expect(out):toBe("\\p[0]あい\\n\\c\\nう\\e")
        expect_column(cols[0], 2, 2)
    end)

    test("1.5: budoux_newlines は pasta が出す区切りの幅を変えない", function()
        local a, b = actors_with({ 4 }, { budoux_newlines = 1.8 })
        local out = build({
            group(a, { talk(a, "あい"), { type = "wait", ms = 100 }, talk(a, "う"), choice("甲"), choice("乙") }),
            group(b, { talk(b, "か") }),
            group(a, { talk(a, "え") }),
        })
        expect(out):toBe("\\p[0]あい\\_w[100]\\n[180]う\\n[150]\\![*]\\q[甲,甲先]\\n\\![*]\\q[乙,乙先]"
            .. "\\p[1]か\\p[0]\\n[150]え\\e")
    end)

    test("_internal: 次の列・最初の列の動き", function()
        local BUILDER = setup()
        local col = { line = 2, width = 7, jump = true }
        BUILDER._internal.next_column(col, 2)
        expect_column(col, 4, 0, true)
        BUILDER._internal.first_column(col)
        expect_column(col, 1, 0, false)
    end)
end)

-- ============================================================================
-- 単独の `\` の直後の pasta の区切り（vertical-balloon-lines 1.2・決定 9）
-- ============================================================================

describe("SAKURA_BUILDER - vertical-balloon-lines: 単独の \\ の後の区切り", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local CHOICE = { type = "choice", display = "甲", target = "甲先" }
    local Q = "\\![*]\\q[甲,甲先]"
    local function spots()
        return { ["さくら"] = 0, ["うにゅう"] = 1 }
    end

    test("字の後の選択肢の前の区切りは、末尾の単独の \\ をエスケープにしてから出す", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({ group(a, { talk(a, "天気\\"), CHOICE }) }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\n[150]" .. Q .. "\\e")
    end)

    test("エスケープの \\\\ で終わる台詞の後は、区切りを今のまま出す", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({ group(a, { talk(a, "天気\\\\"), CHOICE }) }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\n[150]" .. Q .. "\\e")
    end)

    test("前の呼び出しの単独の \\ の後に、持ち越しで入る台詞の頭の改行も守る", function()
        local BUILDER = setup()
        local a = { name = "さくら", spot = 0, budoux = { 6 } }
        local out = BUILDER.build({ group(a, { talk(a, "天気\\"), talk(a, "今日") }) }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\n今日\\e")
    end)

    test("作者が書いたタグが台詞の頭にあるときは、前の単独の \\ に触らない", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({
            group(a, { talk(a, "天気\\"), { type = "sakura_script", actor = a, text = "\\s[1]" } }),
        }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\s[1]\\e")
    end)

    test("act:newline の改行も、末尾の単独の \\ をエスケープにしてから出す", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({ group(a, { talk(a, "天気\\"), { type = "newline", n = 2 } }) }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\n\\n\\e")
    end)

    test("act:clear の \\c も、末尾の単独の \\ をエスケープにしてから出す", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({ group(a, { talk(a, "天気\\"), { type = "clear" } }) }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\c\\e")
    end)

    test("守りが続いても、エスケープは 1 つだけ足す", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({
            group(a, { talk(a, "天気\\"), { type = "newline", n = 1 }, { type = "clear" } }),
        }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\n\\c\\e")
        -- エスケープした後の末尾は単独の \ ではない（続けて守っても足さない）
        local b = BUILDER._internal.tracking_buffer(require("pasta.buf").new())
        b:put("天気\\")
        b:put("\\n")
        b:put("\\c")
        expect(b:tostring()):toBe("天気\\\\\\n\\c")
        -- 作者が書いた片（put_raw）は守らない
        local r = BUILDER._internal.tracking_buffer(require("pasta.buf").new())
        r:put("天気\\")
        r:put_raw("\\s[1]")
        expect(r:tostring()):toBe("天気\\\\s[1]")
    end)

    test("単独の \\ で始まる台詞の頭に持ち越しの改行が入るときも、前の単独の \\ を守る", function()
        local BUILDER = setup()
        local a = { name = "さくら", spot = 0, budoux = { 6 } }
        local out = BUILDER.build({ group(a, { talk(a, "天気\\"), talk(a, "\\今日") }) }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\n\\今日\\e")
    end)

    test("終わりの \\e も、末尾の単独の \\ をエスケープにしてから出す", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({ group(a, { talk(a, "天気\\") }) }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\e")
    end)

    test("話し手の切り替えの \\p[…] も、末尾の単独の \\ をエスケープにしてから出す", function()
        local BUILDER, actors = setup()
        local a, k = actors.sakura, actors.kero
        local out = BUILDER.build({
            group(a, { talk(a, "天気\\") }),
            group(k, { talk(k, "やあ") }),
        }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\\\p[1]やあ\\e")
    end)

    test("pasta が出す表情・ウェイト・制限時間のタグも、末尾の単独の \\ をエスケープにしてから出す", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local cases = {
            { { type = "surface", id = 5 }, "\\s[5]" },
            { { type = "wait", ms = 300 }, "\\_w[300]" },
            { { type = "choice_timeout", seconds = 5 }, "\\![set,choicetimeout,5000]" },
        }
        for _, c in ipairs(cases) do
            local out = BUILDER.build({ group(a, { talk(a, "天気\\"), c[1] }) }, {}, spots())
            expect(out):toBe("\\p[0]天気\\\\" .. c[2] .. "\\e")
        end
    end)

    test("作者が書いた raw_script は、前の単独の \\ に触らない", function()
        local BUILDER, actors = setup()
        local a = actors.sakura
        local out = BUILDER.build({
            group(a, { talk(a, "天気\\"), { type = "raw_script", text = "\\s[1]" } }),
        }, {}, spots())
        expect(out):toBe("\\p[0]天気\\\\s[1]\\e")
    end)
end)

-- ============================================================================
-- 作例の照合・Rust との同値・区切りの不変条件（vertical-balloon-lines 7.1・7.3・決定 7・決定 8）
-- 本物の @pasta_sakura_script を使う（lua_unittest_runner が登録する）
-- ============================================================================

describe("SAKURA_BUILDER - vertical-balloon-lines: 作例・同値・不変条件", function()
    local function group(actor, inner_tokens)
        return { type = "actor", actor = actor, tokens = inner_tokens }
    end
    local function talk(actor, text)
        return { type = "talk", actor = actor, text = text }
    end
    local function choice(display)
        return { type = "choice", display = display, target = display .. "先" }
    end
    local function spots()
        return { ["さくら"] = 0, ["うにゅう"] = 1, ["ぱすた"] = 0 }
    end
    local function copy(col)
        return { line = col.line, width = col.width, jump = col.jump }
    end
    local function strip_waits(s)
        return (s:gsub("\\_w%[%d+%]", ""))
    end

    test("7.3: Introduction の作例を組み立てると、作例の文字列になる（ウェイトを除く）", function()
        local BUILDER = setup()
        local pasta = { name = "ぱすた", spot = 0, budoux = { 40 }, budoux_newlines = 1.8 }
        -- 1 行目の末尾の ＠左 が返す \_l[-295,0] と 2 行目は、act が 1 つの talk につなぐ
        local text = "長い長い、記憶の糸。生まれてから、続いてる一本の長い記憶。"
            .. "\\_l[-295,0]"
            .. "そう、きっと、一本道。いつか来る終わりの日まで。"
        local out = BUILDER.build({ group(pasta, { talk(pasta, text) }) }, { spot_newlines = 1.8 }, spots())
        expect(strip_waits(out)):toBe("\\p[0]長い長い、記憶の糸。生まれてから、\\n[180]続いてる一本の長い記憶。"
            .. "\\_l[-295,0]そう、きっと、一本道。いつか来る終わりの\\n[180]日まで。\\e")
    end)

    test("決定 7: next_column・first_column と talk_to_script(\\n・\\c) の書き換えが同じ", function()
        local BUILDER = setup()
        local SAKURA_SCRIPT = require("@pasta_sakura_script")
        local actors = {
            false, -- nil の代わり（表の中に nil を置けないため）
            { budoux = { 4 } },
            { budoux = { 4, 8 }, budoux_jump = { 6 } },
        }
        local starts = {
            { line = 1, width = 0, jump = false },
            { line = 2, width = 7, jump = true },
            { line = 5, width = 13, jump = false },
            { line = 3, width = 0, jump = true },
        }
        for _, a in ipairs(actors) do
            local actor = a or nil
            for _, start in ipairs(starts) do
                local cases = {
                    { "\\n", function(c) BUILDER._internal.next_column(c, 1) end },
                    { "\\n[180]", function(c) BUILDER._internal.next_column(c, 1) end },
                    { "\\n\\n", function(c) BUILDER._internal.next_column(c, 2) end },
                    { "\\c", function(c) BUILDER._internal.first_column(c) end },
                }
                for _, case in ipairs(cases) do
                    local by_lua, by_rust = copy(start), copy(start)
                    case[2](by_lua)
                    SAKURA_SCRIPT.talk_to_script(actor, case[1], by_rust)
                    expect(by_rust.line):toBe(by_lua.line)
                    expect(by_rust.width):toBe(by_lua.width)
                    expect(by_rust.jump):toBe(by_lua.jump)
                end
            end
        end
    end)

    -- 出力の文字列を、スポットごとに列の動きの規則で数え直す（組み立てを呼ばない独立の数え方）。
    -- \p[n]・\0・\1 で吹き出しを選び、\n・\n[…] で次、引数の無い \c で最初、
    -- \_l[…] で飛ぶ（has_jump のときだけ。無ければ列は動かない）。\\ はエスケープとして飛ばす。
    -- CLEAR_SPOT の印（テストが clear_spot の直後に置く raw_script）で全スポットを捨てる。
    -- ponytail: 選択肢の表示の中の \n などは見分けない。テストの入力に置かない
    local CLEAR_SPOT = "\\![test,clear_spot]"
    local function recount(out, has_jump)
        local cols, spot, i = {}, 0, 1
        local function at(s)
            cols[s] = cols[s] or { line = 1, jump = false }
            return cols[s]
        end
        at(0)
        while i <= #out do
            local rest = out:sub(i)
            local n
            if rest:sub(1, #CLEAR_SPOT) == CLEAR_SPOT then
                cols, n = {}, #CLEAR_SPOT
            elseif rest:sub(1, 2) == "\\\\" then
                n = 2
            elseif rest:match("^\\p%[%d+%]") then
                local m = rest:match("^\\p%[(%d+)%]")
                spot, n = tonumber(m), #m + 4
                at(spot)
            elseif rest:match("^\\[01]") then
                spot, n = tonumber(rest:sub(2, 2)), 2
                at(spot)
            elseif rest:match("^\\n%[[^%]]*%]") then
                at(spot).line, n = at(spot).line + 1, #rest:match("^\\n%[[^%]]*%]")
            elseif rest:sub(1, 2) == "\\n" then
                at(spot).line, n = at(spot).line + 1, 2
            elseif rest:sub(1, 2) == "\\c" and rest:sub(3, 3) ~= "[" then
                at(spot).line, at(spot).jump, n = 1, false, 2
            elseif rest:match("^\\_l%[[^%]]*%]") then
                if has_jump then
                    at(spot).line, at(spot).jump = 1, true
                end
                n = #rest:match("^\\_l%[[^%]]*%]")
            else
                n = 1
            end
            i = i + n
        end
        return cols
    end

    local function expect_invariant(tokens, config, has_jump)
        local BUILDER = setup()
        config = config or {}
        config.columns_out = {}
        local out = BUILDER.build(tokens, config, spots())
        local recounted = recount(out, has_jump)
        for spot, col in pairs(config.columns_out) do
            expect(recounted[spot] ~= nil):toBe(true)
            expect(recounted[spot].line):toBe(col.line)
            expect(recounted[spot].jump):toBe(col.jump)
        end
        -- 組み立てが表を作らなかったスポットは、数え直しでも 1 列目のまま
        for spot, col in pairs(recounted) do
            if config.columns_out[spot] == nil then
                expect(col.line):toBe(1)
                expect(col.jump):toBe(false)
            end
        end
        return out
    end

    test("決定 8: 話し手の切り替え・選択肢の前後・表情を挟む台詞", function()
        local a = { name = "さくら", spot = 0, budoux = { 4, 8 } }
        local b = { name = "うにゅう", spot = 1, budoux = { 6 } }
        expect_invariant({
            group(a, { talk(a, "あいうえおかきく"), { type = "surface", id = 3 }, talk(a, "けこ") }),
            group(b, { talk(b, "さしすせそたちつ"), choice("甲"), choice("乙"), talk(b, "てと") }),
            group(a, { talk(a, "なにぬ"), choice("丙"), talk(a, "ねの") }),
            group(b, { talk(b, "はひ") }),
        }, { spot_newlines = 1.5, after_choice_newlines = 2.0 })
    end)

    test("決定 8: newline・clear・台詞の中の \\n と \\c・sakura_script", function()
        local a = { name = "さくら", spot = 0, budoux = { 4, 20 } }
        local b = { name = "うにゅう", spot = 1, budoux = { 4 } }
        expect_invariant({
            group(a, {
                talk(a, "あいうえおか"),
                { type = "newline", n = 2 },
                talk(a, "きく\\nけこさしすせ"),
                { type = "sakura_script", actor = a, text = "\\n[half]" },
                talk(a, "そた"),
            }),
            group(b, { talk(b, "まみむめもや"), { type = "clear" }, talk(b, "ゆよ\\cらり") }),
            group(a, { { type = "wait", ms = 50 }, talk(a, "るれ") }),
        })
    end)

    test("決定 8: clear_spot の後は全スポットを捨てて数え直す", function()
        local a = { name = "さくら", spot = 0, budoux = { 4 } }
        local b = { name = "うにゅう", spot = 1, budoux = { 4 } }
        expect_invariant({
            group(a, { talk(a, "あいうえおか"), { type = "newline", n = 1 } }),
            group(b, { talk(b, "かきくけこ") }),
            { type = "clear_spot" },
            { type = "raw_script", text = CLEAR_SPOT },
            group(b, { talk(b, "さしすせそ") }),
            group(a, { talk(a, "たち") }),
            group(b, { talk(b, "つ") }),
        })
    end)

    test("決定 8: budoux_jump のある \\_l で飛び、\\c で戻る", function()
        local a = { name = "さくら", spot = 0, budoux = { 4 }, budoux_jump = { 6, 4 } }
        local b = { name = "うにゅう", spot = 1, budoux = { 4 }, budoux_jump = { 6, 4 } }
        expect_invariant({
            group(a, {
                talk(a, "あいうえおか\\_l[-295,0]きくけこさしすせ"),
                { type = "clear" },
                talk(a, "こさ\\_l[-295,0]しす"),
                { type = "clear" },
            }),
            group(b, { talk(b, "たちつて\\_l[-295,0]となにぬ\\cねの\\_l[-295,0]はひ") }),
            group(b, { talk(b, "ふへ") }),
        }, nil, true)
    end)
end)

-- ============================================================================
-- actor-display-style: 装いの出し入れの組み立て（Task 3.1）
-- ============================================================================

describe("SAKURA_BUILDER - actor-display-style: 装いの出し入れ（Task 3.1）", function()
    -- 句読点のウェイトを入れない（実効ウェイト＝値 - 50 = 0）
    local function actor(name, font)
        return {
            name = name, font = font,
            script_wait_period = 50, script_wait_comma = 50,
            script_wait_strong = 50, script_wait_leader = 50,
        }
    end
    local function group(a, inner_tokens)
        return { type = "actor", actor = a, tokens = inner_tokens }
    end
    local function talk(a, text)
        return { type = "talk", actor = a, text = text }
    end
    local function choice(display)
        return { type = "choice", display = display, target = display .. "先" }
    end
    local OJI = "\\f[color,90,110,140]"
    local GAL = "\\f[color,255,0,128]"
    local Q1 = "\\![*]\\q[甲,甲先]"
    local function cast()
        return actor("おじさん", OJI .. "\\f[bold,1]"), actor("ギャル", GAL), actor("ぱすた", nil)
    end
    local function shared_spot()
        return { ["おじさん"] = 0, ["ギャル"] = 0, ["ぱすた"] = 0, ["さくら"] = 1 }
    end

    test("2.5: 要件の例とバイト列で一致する", function()
        local BUILDER = setup()
        local oji, pasta = actor("おじさん", OJI), actor("ぱすた", nil)
        local result = BUILDER.build({
            group(oji, { talk(oji, "やあ、相変わらず薄着だね。寒くないの？") }),
            group(pasta, { talk(pasta, "やあ、相変わらずエッチですね。") }),
        }, { spot_newlines = 1.8 }, { ["おじさん"] = 0, ["ぱすた"] = 0 })
        expect(result):toBe("\\p[0]\\f[color,90,110,140]やあ、相変わらず薄着だね。寒くないの？"
            .. "\\p[0]\\f[default]\\n[180]やあ、相変わらずエッチですね。\\e")
    end)

    test("2.6: 装いのあるアクターどうしの切り替えは戻してから付ける（bold が残らない）", function()
        local BUILDER = setup()
        local oji, gal = cast()
        local result = BUILDER.build({
            group(oji, { talk(oji, "A") }),
            group(gal, { talk(gal, "B") }),
        }, {}, shared_spot())
        expect(result):toBe("\\p[0]" .. OJI .. "\\f[bold,1]A\\p[0]\\f[default]" .. GAL .. "\\n[150]B\\e")
    end)

    test("2.1/2.2: 別スポットへ移るときは戻さず、戻ってきたスポットで戻してから付け直す", function()
        local BUILDER = setup()
        local oji = actor("おじさん", OJI)
        local sakura = actor("さくら", nil)
        local result = BUILDER.build({
            group(oji, { talk(oji, "A") }),
            group(sakura, { talk(sakura, "B") }),
            group(oji, { talk(oji, "C") }),
        }, {}, shared_spot())
        expect(result):toBe("\\p[0]" .. OJI .. "A\\p[1]B\\p[0]\\f[default]" .. OJI .. "\\n[150]C\\e")
    end)

    test("2.4/4.1: 話し続ける間と台詞に書いた \\f の後に装いを差し込まない", function()
        local BUILDER = setup()
        local oji = actor("おじさん", OJI)
        local result = BUILDER.build({
            group(oji, { talk(oji, "A\\f[bold,1]B"), talk(oji, "C") }),
            group(oji, { talk(oji, "D") }),
        }, {}, shared_spot())
        expect(result):toBe("\\p[0]" .. OJI .. "A\\f[bold,1]BCD\\e")
    end)

    test("2.9: 未登録アクターは装いの無いアクターとして \\p[0] に出る", function()
        local BUILDER = setup()
        local FAILURE = require("pasta.failure")
        local oji = actor("おじさん", OJI)
        local adhoc = { name = "sb_装い未登録" }
        FAILURE.mark_unregistered_actor(adhoc)
        local result = BUILDER.build({
            group(oji, { talk(oji, "A") }),
            group(adhoc, { talk(adhoc, "X") }),
        }, {}, { ["おじさん"] = 0 })
        expect(result):toBe("\\p[0]" .. OJI .. "A\\p[0]\\f[default]\\n[150]X\\e")
    end)

    test("2.7: 装いのタグは改行の判定・選択肢の前の区切りに影響しない", function()
        local BUILDER = setup()
        local function run(font)
            local oji = actor("おじさん", font)
            local sakura = actor("さくら", nil)
            return BUILDER.build({
                group(oji, { choice("甲") }),
                group(sakura, { talk(sakura, "B") }),
                group(oji, { talk(oji, "C"), choice("甲") }),
            }, {}, shared_spot())
        end
        local styled = run(OJI)
        expect(styled):toBe("\\p[0]" .. OJI .. Q1 .. "\\p[1]B\\p[0]\\f[default]" .. OJI
            .. "\\nC\\n[150]" .. Q1 .. "\\e")
        expect((styled:gsub("\\f%[[^%]]*%]", ""))):toBe(run(nil))
    end)

    test("3.3: 選択肢の後に装いを出し直さない", function()
        local BUILDER = setup()
        local oji = actor("おじさん", OJI)
        local result = BUILDER.build({
            group(oji, { talk(oji, "A"), choice("甲"), talk(oji, "B") }),
        }, {}, shared_spot())
        expect(result):toBe("\\p[0]" .. OJI .. "A\\n[150]" .. Q1 .. "\\nB\\e")
    end)

    test("3.2: 台詞の中の \\c の後に装いを出し直す（内側のトークンは変換後に通す）", function()
        local BUILDER = setup()
        local oji = actor("おじさん", OJI)
        local result = BUILDER.build({
            group(oji, { talk(oji, "A\\cB"), { type = "clear" }, talk(oji, "C") }),
        }, {}, shared_spot())
        expect(result):toBe("\\p[0]" .. OJI .. "A\\c" .. OJI .. "B\\c" .. OJI .. "C\\e")
    end)

    test("6.1〜6.4/2.1〜2.3/2.6: 共有スポットの会話 ぱすた→おじさん→ぱすた→ギャル→おじさん（Task 3.2）", function()
        local BUILDER = setup()
        local function run(oji_font, gal_font)
            -- シェルのあるぱすた（surface = 0・装い無し）と、シェルの無いおじさん・ギャル
            local oji, gal, pasta = actor("おじさん", oji_font), actor("ギャル", gal_font), actor("ぱすた", nil)
            pasta.surface = 0
            return BUILDER.build({
                group(pasta, { talk(pasta, "P1") }),
                group(oji, { talk(oji, "O1") }),
                group(pasta, { talk(pasta, "P2") }),
                group(gal, { talk(gal, "G1") }),
                group(oji, { talk(oji, "O2") }),
            }, {}, shared_spot())
        end
        local OJI_FONT = OJI .. "\\f[bold,1]"
        local result = run(OJI_FONT, GAL)
        expect(result):toBe("\\p[0]\\s[0]P1"
            .. "\\p[0]\\f[default]" .. OJI_FONT .. "\\n[150]O1"
            .. "\\p[0]\\f[default]\\n[150]P2"
            .. "\\p[0]\\f[default]" .. GAL .. "\\n[150]G1"
            .. "\\p[0]\\f[default]" .. OJI_FONT .. "\\n[150]O2\\e")
        -- 6.2/6.3: 立ち絵の復旧タグは最初のぱすたの 1 回だけ
        local _, surfaces = result:gsub("\\s%[", "")
        expect(surfaces):toBe(1)
        -- 装いを抜くと、装いの無いゴーストの出力と同じ（6.1 の改行・6.2/6.3 の復旧は装いに左右されない）
        expect((result:gsub("\\f%[[^%]]*%]", ""))):toBe(run(nil, nil))
    end)

    test("最上位の生のさくらスクリプトは通さない", function()
        local BUILDER = setup()
        local oji = actor("おじさん", OJI)
        local result = BUILDER.build({
            group(oji, { talk(oji, "A") }),
            { type = "raw_script", text = "\\c" },
        }, {}, shared_spot())
        expect(result):toBe("\\p[0]" .. OJI .. "A\\c\\e")
    end)
end)
