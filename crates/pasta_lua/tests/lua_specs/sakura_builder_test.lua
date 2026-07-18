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
    test("waitトークンを \\w[ms] に変換する", function()
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

        expect(result:find("\\w%[500%]")):toBeTruthy()
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
        expect(result:find("\\w%[100%]")):toBeTruthy()
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
end)
