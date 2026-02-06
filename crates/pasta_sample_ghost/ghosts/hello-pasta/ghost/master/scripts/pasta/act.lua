--- @module pasta.act
--- アクションオブジェクトモジュール
---
--- トランスパイラー出力のシーン関数から第1引数として受け取るオブジェクト。
--- トークン蓄積、アクタープロキシ動的生成、シーン制御を担当する。

local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local GLOBAL = require("pasta.global")

-- ============================================================================
-- グループ化ローカル関数（actor-talk-grouping feature）
-- ============================================================================

--- トークン配列をアクター切り替え境界でグループ化
--- @param tokens table[] フラットなトークン配列
--- @return table[] グループ化されたトークン配列
local function group_by_actor(tokens)
    if not tokens or #tokens == 0 then
        return {}
    end

    local result = {}
    local current_actor_token = nil -- 現在の type="actor" トークン
    local current_actor = nil       -- 現在のアクター（nilは未設定）

    for _, token in ipairs(tokens) do
        local t = token.type

        -- アクター属性設定トークン: 独立して出力
        if t == "spot" or t == "clear_spot" then
            table.insert(result, token)
        elseif t == "talk" then
            local talk_actor = token.actor
            -- アクター変更検出（最初のtalkまたはアクター変更時）
            if current_actor_token == nil or talk_actor ~= current_actor then
                -- 新しい type="actor" トークンを開始
                current_actor_token = {
                    type = "actor",
                    actor = talk_actor,
                    tokens = {}
                }
                table.insert(result, current_actor_token)
                current_actor = talk_actor
            end
            table.insert(current_actor_token.tokens, token)
        else
            -- アクター行動トークン（surface, wait, newline, clear, raw_script）
            -- 現在のアクターグループ内に追加
            if current_actor_token then
                table.insert(current_actor_token.tokens, token)
            end
            -- 注: current_actor_tokenがnilの場合（talkより先にアクター行動が来た場合）は無視
            -- 現在の設計ではこの状況は発生しない
        end
    end

    return result
end

--- グループ化トークン内の連続talkトークンを統合
--- @param grouped table[] グループ化されたトークン配列
--- @return table[] 統合済みトークン配列
local function merge_consecutive_talks(grouped)
    local result = {}

    for _, token in ipairs(grouped) do
        if token.type == "actor" then
            -- type="actor" トークン内のtalkを統合
            local merged_tokens = {}
            local pending_talk = nil

            for _, inner in ipairs(token.tokens) do
                if inner.type == "talk" then
                    if pending_talk then
                        -- 連続talk: テキスト結合
                        pending_talk.text = pending_talk.text .. inner.text
                    else
                        -- 新規talk開始
                        pending_talk = {
                            type = "talk",
                            actor = inner.actor,
                            text = inner.text
                        }
                    end
                else
                    -- 非talkトークン: pending_talkをフラッシュ
                    if pending_talk then
                        table.insert(merged_tokens, pending_talk)
                        pending_talk = nil
                    end
                    table.insert(merged_tokens, inner)
                end
            end

            -- 最後のpending_talkをフラッシュ
            if pending_talk then
                table.insert(merged_tokens, pending_talk)
            end

            table.insert(result, {
                type = "actor",
                actor = token.actor,
                tokens = merged_tokens
            })
        else
            -- spot, clear_spot はそのまま出力
            table.insert(result, token)
        end
    end

    return result
end

-- ============================================================================
-- Actクラス定義
-- ============================================================================

--- @class Act アクションオブジェクト
--- @field actors table<string, Actor> 登録アクター（名前→アクター）
--- @field save table 永続変数テーブル
--- @field app_ctx table アプリケーション実行中の汎用コンテキストデータ
--- @field var table アクションローカル変数
--- @field token table[] 構築中のスクリプトトークン
--- @field current_scene SceneTable|nil 現在のシーンテーブル
local ACT = {}

--- ACT実装メタテーブル
local ACT_IMPL = {}

--- __indexメタメソッド: メソッド検索とアクタープロキシ動的生成
--- @param self Act アクションオブジェクト
--- @param key string キー名
--- @return any メソッドまたはプロキシ
function ACT_IMPL.__index(self, key)
    -- 1. ACT_IMPLメソッドを検索
    local method = ACT_IMPL[key]
    if method then return method end

    -- 2. アクター名としてプロキシ生成
    local actor = self.actors[key]
    if actor then
        return ACTOR.create_proxy(actor, self)
    end

    return nil
end

--- 新規Actを作成
--- @param actors table<string, Actor> 登録アクター
--- @return Act アクションオブジェクト
function ACT.new(actors)
    local obj = {
        actors = actors or {},
        save = require("pasta.save"),
        app_ctx = require("pasta.store").app_ctx,
        var = {},
        token = {},
        current_scene = nil,
    }
    return setmetatable(obj, ACT_IMPL)
end

--- シーン初期化（トランスパイラー出力から呼び出し）
--- @param self Act アクションオブジェクト
--- @param scene SceneTable SCENEテーブル
--- @return table save 永続変数テーブル
--- @return table var アクションローカル変数テーブル
function ACT_IMPL.init_scene(self, scene)
    self.current_scene = scene
    return self.save, self.var
end

--- talkトークン蓄積（状態レス化: actorトークン/spot_switch生成を削除）
--- @param self Act アクションオブジェクト
--- @param actor Actor アクターオブジェクト
--- @param text string 発話テキスト
--- @return Act self メソッドチェーン用
function ACT_IMPL.talk(self, actor, text)
    table.insert(self.token, { type = "talk", actor = actor, text = text })
    return self
end

--- raw_scriptトークン蓄積
--- @param self Act アクションオブジェクト
--- @param text string 生スクリプト文字列
--- @return Act self メソッドチェーン用
function ACT_IMPL.raw_script(self, text)
    table.insert(self.token, { type = "raw_script", text = text })
    return self
end

--- surfaceトークン蓄積
--- @param self Act アクションオブジェクト
--- @param id number|string サーフェスID
--- @return Act self メソッドチェーン用
function ACT_IMPL.surface(self, id)
    table.insert(self.token, { type = "surface", id = id })
    return self
end

--- waitトークン蓄積
--- @param self Act アクションオブジェクト
--- @param ms number 待機時間（ミリ秒）
--- @return Act self メソッドチェーン用
function ACT_IMPL.wait(self, ms)
    ms = math.max(0, math.floor(ms or 0))
    table.insert(self.token, { type = "wait", ms = ms })
    return self
end

--- newlineトークン蓄積
--- @param self Act アクションオブジェクト
--- @param n number|nil 改行回数（デフォルト1）
--- @return Act self メソッドチェーン用
function ACT_IMPL.newline(self, n)
    table.insert(self.token, { type = "newline", n = n or 1 })
    return self
end

--- clearトークン蓄積
--- @param self Act アクションオブジェクト
--- @return Act self メソッドチェーン用
function ACT_IMPL.clear(self)
    table.insert(self.token, { type = "clear" })
    return self
end

--- 単語検索（アクター非依存、4レベル検索）
--- 検索順序:
--- 1. シーンテーブル完全一致 (current_scene[name])
--- 2. GLOBAL完全一致 (GLOBAL[name])
--- 3. シーンローカル辞書前方一致 (SEARCH:search_word(name, scene_name))
--- 4. グローバル辞書前方一致 (SEARCH:search_word(name, nil))
--- @param self Act アクションオブジェクト
--- @param name string 単語名
--- @return string|nil 見つかった単語、またはnil
function ACT_IMPL.word(self, name)
    if not name or name == "" then
        return nil
    end

    local WORD = require("pasta.word")

    -- 1. シーンテーブル完全一致
    if self.current_scene and self.current_scene[name] ~= nil then
        local value = self.current_scene[name]
        return WORD.resolve_value(value, self)
    end

    -- 2. GLOBAL完全一致
    if GLOBAL[name] ~= nil then
        local value = GLOBAL[name]
        return WORD.resolve_value(value, self)
    end

    -- 3 & 4. SEARCH API を使用した前方一致検索（利用可能な場合のみ）
    local ok, SEARCH = pcall(require, "@pasta_search")
    if ok and SEARCH then
        -- 3. シーンローカル辞書（前方一致）
        local scene_name = self.current_scene and self.current_scene.__global_name__
        if scene_name then
            local result = SEARCH:search_word(name, scene_name)
            if result then
                return result -- SEARCH APIは既に文字列を返す
            end
        end

        -- 4. グローバル辞書（前方一致）
        local result = SEARCH:search_word(name, nil)
        if result then
            return result -- SEARCH APIは既に文字列を返す
        end
    end

    return nil
end

--- トークン取得とリセット（グループ化・統合済み）
--- @param self Act アクションオブジェクト
--- @return table[]|nil グループ化されたトークン配列、またはnil（トークン0件時）
function ACT_IMPL.build(self)
    local tokens = self.token
    self.token = {}

    -- 早期リターン: トークン0件時はnilを返す (act-build-early-return)
    if #tokens == 0 then
        return nil
    end

    -- Phase 1: アクター切り替え境界でグループ化
    local grouped = group_by_actor(tokens)

    -- Phase 2: 連続talkを統合
    local merged = merge_consecutive_talks(grouped)

    return merged
end

--- build()結果をyield
--- @param self Act アクションオブジェクト
--- @return Act self メソッドチェーン用
function ACT_IMPL.yield(self)
    local result = self:build()
    coroutine.yield(result)
    return self
end

--- シーン呼び出し（4段階検索）
---
--- トランスパイラ出力から呼び出され、キーに対応するハンドラーを検索して実行する。
--- 4段階の優先順位に従い、最初に見つかった有効な関数を実行する。
---
--- @param self Act アクションオブジェクト
--- @param global_scene_name string|nil グローバルシーン名
--- @param key string 検索キー
--- @param attrs table|nil 属性テーブル（将来拡張用、現在は未使用）
--- @param ... any 可変長引数（ハンドラーに渡す）
--- @return any ハンドラーの戻り値、またはnil
function ACT_IMPL.call(self, global_scene_name, key, attrs, ...)
    local handler = nil

    -- Level 1: シーンローカル検索
    if self.current_scene then
        handler = self.current_scene[key]
    end

    -- Level 2: グローバルシーン名スコープ検索
    if not handler then
        local result = SCENE.search(key, global_scene_name, attrs)
        if result then
            handler = result.func
        end
    end

    -- Level 3: グローバル関数モジュール
    if not handler then
        handler = GLOBAL[key]
    end

    -- Level 4: スコープなし全体検索（フォールバック）
    if not handler then
        local result = SCENE.search(key, nil, attrs)
        if result then
            handler = result.func
        end
    end

    -- ハンドラー実行
    if handler then
        return handler(self, ...)
    end

    -- TODO: ハンドラー未発見時のログ出力（将来実装）
    return nil
end

--- スポット設定トークン生成（状態レス化）
--- @param self Act アクションオブジェクト
--- @param name string アクター名
--- @param number integer 位置
--- @return nil
function ACT_IMPL.set_spot(self, name, number)
    local actor = self.actors[name]
    if actor then
        table.insert(self.token, { type = "spot", actor = actor, spot = number })
    end
end

--- 全スポットクリアトークン生成（状態レス化）
--- @param self Act アクションオブジェクト
--- @return nil
function ACT_IMPL.clear_spot(self)
    table.insert(self.token, { type = "clear_spot" })
end

--- 継承用に実装メタテーブルを公開
ACT.IMPL = ACT_IMPL

return ACT
