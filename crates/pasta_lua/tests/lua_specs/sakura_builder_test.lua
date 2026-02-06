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

-- ============================================================================
-- Requirement 6: sakura_builderモジュール（グループ化形式）
-- ============================================================================

describe("SAKURA_BUILDER - talk token", function()
    test("talkトークンがtalk_to_scriptで変換される", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")

        local result = BUILDER.build({}, {})

        expect(result):toBe("\\e")
    end)
end)

describe("SAKURA_BUILDER - 複合シナリオ", function()
    test("複数トークンを正しく連結する", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        expect(result:find("\\n%[150%]")):toBeTruthy() -- spot変更時の段落改行
        expect(result:find("\\p%[1%]")):toBeTruthy()
        expect(result:find("Hi")):toBeTruthy()
        expect(result:find("\\n")):toBeTruthy()
        expect(result:find("\\c")):toBeTruthy()
        expect(result:sub(-2)):toBe("\\e")
    end)
end)

describe("SAKURA_BUILDER - talk_to_script変換", function()
    test("テキストがtalk_to_scriptで変換される", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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

        -- spot変更時に段落改行が出力される
        expect(result:find("\\n%[150%]")):toBeTruthy()
    end)

    test("同じactorの連続actorトークンではスポットタグを出力しない", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
-- actor-spot-refactoring: 統合シナリオ (Task 4.4)
-- ============================================================================

describe("SAKURA_BUILDER - 統合シナリオ（グループ化トークン構造）", function()
    test("set_spot()なしでのtalk() → デフォルトspot(0)使用を確認", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        expect(result:find("\\n%[150%]\\p%[1%]Kero speaks")):toBeTruthy()
    end)

    test("clear_spot() → talk() → デフォルトspot(0)への復帰を確認", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
    test("入力actor_spotsテーブルが変更されないことを確認", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

        local input_spots = { ["さくら"] = 0, ["うにゅう"] = 1 }
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

        -- build呼び出し前のスナップショット
        local original_sakura = input_spots["さくら"]
        local original_kero = input_spots["うにゅう"]

        local _, _ = BUILDER.build(tokens, {}, input_spots)

        -- 入力テーブルが変更されていないことを確認
        expect(input_spots["さくら"]):toBe(original_sakura)
        expect(input_spots["うにゅう"]):toBe(original_kero)
    end)

    test("nil入力時に空テーブルとして扱われることを確認", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local result, updated_spots = BUILDER.build(tokens, {}, nil)

        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(type(updated_spots)):toBe("table")
    end)

    test("第2戻り値としてactor_spotsテーブルが返されることを確認", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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

        local result, updated_spots = BUILDER.build(tokens, {})

        expect(type(result)):toBe("string")
        expect(type(updated_spots)):toBe("table")
        expect(updated_spots["さくら"]):toBe(0)
    end)

    test("後方互換性: actor_spots省略時も正常動作", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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
        local result, updated_spots = BUILDER.build(tokens, {})

        expect(result:find("Hello")):toBeTruthy()
        expect(type(updated_spots)):toBe("table")
    end)
end)

describe("SAKURA_BUILDER - persist-spot-position: clear_spotトークン処理 (Task 5.4)", function()
    test("clear_spotトークンで入力のactor_spotsがリセットされる", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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

        local result, updated_spots = BUILDER.build(tokens, {}, input_spots)

        -- clear_spot後はデフォルトspot(0)を使用
        expect(result:find("\\p%[0%]")):toBeTruthy()
        -- updated_spotsからエントリがクリアされている
        expect(updated_spots["さくら"]):toBe(nil)
        expect(updated_spots["うにゅう"]):toBe(nil)
    end)

    test("入力テーブルがclear_spotで変更されないことを確認", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

        local input_spots = { ["さくら"] = 5 }
        local tokens = {
            { type = "clear_spot" },
        }

        local _, _ = BUILDER.build(tokens, {}, input_spots)

        -- 入力テーブルは変更されない（純粋関数性）
        expect(input_spots["さくら"]):toBe(5)
    end)
end)

describe("SAKURA_BUILDER - persist-spot-position: spotトークン処理 (Task 5.5)", function()
    test("spotトークンで入力のactor_spotsが正しく更新される", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

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

        local _, updated_spots = BUILDER.build(tokens, {}, input_spots)

        expect(updated_spots["さくら"]):toBe(0)
        expect(updated_spots["うにゅう"]):toBe(1)
    end)

    test("入力actor_spotsの値を引き継いでスポットタグが出力される", function()
        local BUILDER = require("pasta.shiori.sakura_builder")
        local actors = create_mock_actors()

        -- 前回のスポット状態を入力として渡す
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
        }

        local result, updated_spots = BUILDER.build(tokens, { spot_newlines = 1.5 }, input_spots)

        -- 前回のスポット値が引き継がれている
        expect(result:find("\\p%[0%]")):toBeTruthy()
        expect(result:find("\\p%[1%]")):toBeTruthy()
        expect(result:find("\\n%[150%]")):toBeTruthy() -- spot変更時の段落改行

        -- updated_spotsも前回の値を保持
        expect(updated_spots["さくら"]):toBe(0)
        expect(updated_spots["うにゅう"]):toBe(1)
    end)
end)
