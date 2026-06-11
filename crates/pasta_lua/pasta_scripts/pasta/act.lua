--- @module pasta.act
--- アクションオブジェクトモジュール
---
--- トランスパイラー出力のシーン関数から第1引数として受け取るオブジェクト。
--- トークン蓄積、アクタープロキシ動的生成、シーン制御を担当する。

local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local GLOBAL = require("pasta.global")
local STORE = require("pasta.store")
local log = require "@pasta_log"

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
        elseif t == "talk" or t == "sakura_script" then
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
        elseif t == "raw_script" then
            -- raw_script: ハイブリッド分類
            -- アクターグループ存在時はグループ内に追加、不在時はresultに直接追加
            if current_actor_token then
                table.insert(current_actor_token.tokens, token)
            else
                table.insert(result, token)
            end
        else
            -- アクター行動トークン（surface, wait, newline, clear）
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
        app_ctx = STORE.app_ctx,
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
    if scene.__global_name__ then
        STORE.last_global_scene = scene.__global_name__
    end
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

--- sakura_scriptトークン蓄積
--- @param self Act アクションオブジェクト
--- @param actor Actor アクターオブジェクト
--- @param text string さくらスクリプトタグ文字列
--- @return Act self メソッドチェーン用
function ACT_IMPL.sakura_script(self, actor, text)
    table.insert(self.token, { type = "sakura_script", actor = actor, text = text })
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

--- 選択肢トークン蓄積（構造化データのみ、さくらスクリプト非依存）
--- @param self Act アクションオブジェクト
--- @param target string 選択肢のジャンプ先
--- @param display string|nil 表示テキスト（nilの場合targetを使用）
--- @return Act self メソッドチェーン用
function ACT_IMPL.choice(self, target, display)
    table.insert(self.token, { type = "choice", target = target, display = display or target })
    return self
end

--- 選択肢タイムアウトトークン蓄積（構造化データのみ、さくらスクリプト非依存）
--- @param self Act アクションオブジェクト
--- @param seconds number|nil タイムアウト秒数（nilの場合は引数なし）
--- @return Act self メソッドチェーン用
function ACT_IMPL.choice_timeout(self, seconds)
    table.insert(self.token, { type = "choice_timeout", seconds = seconds })
    return self
end

--- 辞書前方一致検索（find_act_handler の L2/L5 共通処理・モード別ディスパッチ）
---
--- word モードは SEARCH:search_word、scene/expr モードは SCENE.search を使用する。
--- SCENE.search はモジュールテーブル経由で呼び出す（テストでの差し替えを許容）。
---
--- @param SEARCH table @pasta_search モジュール
--- @param mode string "word" | "scene" | "expr"
--- @param key string 検索キー
--- @param scene_name string|nil ローカル検索時のシーン名（nil でグローバル検索）
--- @return any|nil 見つかったハンドラー、またはnil
local function search_dictionary(SEARCH, mode, key, scene_name)
    if mode == "word" then
        local result = SEARCH:search_word(key, scene_name)
        if result ~= nil then return result end
    else
        local result = SCENE.search(key, scene_name)
        if result then return result.func end
    end
    return nil
end

--- ハンドラーフォールバック検索コア（6段階）
---
--- 検索レベル:
--- L1: current_scene[key] 完全一致（全モード共通）
--- L2: ローカル辞書前方一致（word: search_word(key, scene_name)、scene/expr: SCENE.search(key, scene_name)）
--- L3: ACT_IMPL[key] function型のみ（act.XX フォールバック。全モード共通）
--- L4: GLOBAL[key] 完全一致（全モード共通）
--- L5: グローバル辞書前方一致（word: search_word(key, nil)、scene/expr: SCENE.search(key, nil)）
--- word モードの @pasta_search 未利用時はL2・L5 word 前方一致をスキップ
--- scene/expr モードは SCENE.search を直接呼び出す（@pasta_search 可用性チェックは package.loaded 参照）
---
--- @param self Act アクションオブジェクト
--- @param mode string "word" | "scene" | "expr"
--- @param key string 検索キー
--- @return any|nil 見つかったハンドラー（関数または値）、またはnil
function ACT_IMPL.find_act_handler(self, mode, key)
    -- @pasta_search 可用性チェック（オプショナルモジュールのため呼び出し毎に pcall で確認。
    -- require はロード済みキャッシュを返すため低コスト）
    local ok, SEARCH = pcall(require, "@pasta_search")
    if not ok then SEARCH = nil end

    -- L1: current_scene[key] 完全一致
    if self.current_scene and self.current_scene[key] ~= nil then
        return self.current_scene[key]
    end

    -- L2: ローカル辞書前方一致（@pasta_search 利用可能かつ scene_name あり時のみ）
    local scene_name = self.current_scene and self.current_scene.__global_name__
    if SEARCH and scene_name then
        local result = search_dictionary(SEARCH, mode, key, scene_name)
        if result ~= nil then return result end
    end

    -- L3: self[key] function型のみ（act.XX / SHIORI_ACT_IMPL 継承チェーンを含む）
    local method = self[key]
    if type(method) == "function" then
        return method
    end

    -- L4: GLOBAL[key] 完全一致（全モード共通）
    if GLOBAL[key] ~= nil then
        return GLOBAL[key]
    end

    -- L5: グローバル辞書前方一致（@pasta_search 利用可能時のみ・key をグローバル検索）
    if SEARCH then
        local result = search_dictionary(SEARCH, mode, key, nil)
        if result ~= nil then return result end
    end

    return nil
end

--- find_act_handler への thin wrapper（ACT_IMPL.find_handler）
--- @param self Act アクションオブジェクト
--- @param mode string "word" | "scene" | "expr"
--- @param key string 検索キー
--- @return any|nil
function ACT_IMPL.find_handler(self, mode, key)
    return self:find_act_handler(mode, key)
end

--- 単語取得（find_handler + word ポストプロセス）
--- 検索順序は find_act_handler の6段階フォールバック（word モード）
--- ポストプロセス: handler=nil → warn+nil、function → h(self)、その他 → tostring(h)
--- @param self Act アクションオブジェクト
--- @param name string 単語名
--- @return string|nil 見つかった単語、またはnil
function ACT_IMPL.word(self, name)
    if not name or name == "" then
        return nil
    end

    local handler = self:find_handler("word", name)
    if handler == nil then
        log.warn(string.format("act:word - handler not found: key='%s', mode='word', via=act",
            tostring(name)))
        return nil
    end
    if type(handler) == "function" then
        return handler(self)
    end
    return tostring(handler)
end

--- expr関数呼び出し（find_handler + expr ポストプロセス）
--- find_handler("expr", key) でハンドラーを取得してポストプロセスを実行する。
--- ポストプロセス: function → h(self, ...) 可変引数を伝搬、非function → warn+nil
--- @param self Act アクションオブジェクト
--- @param key string 関数名
--- @param ... any 可変引数（ハンドラーに伝搬）
--- @return any ハンドラーの戻り値、またはnil
function ACT_IMPL.expr_fn(self, key, ...)
    local handler = self:find_handler("expr", key)
    if type(handler) == "function" then
        return handler(self, ...)
    end
    log.warn(string.format("act:expr_fn - handler not found: key='%s', mode='expr', via=act",
        tostring(key)))
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

--- シーン名前解決（find_handler への thin wrapper）
---
--- キーに対応するハンドラー関数を検索して返す（実行しない）。
--- find_handler("scene", key) に委譲する。
--- コルーチン化は呼び出し元 SCENE.co_exec の責務。
---
--- @param self Act アクションオブジェクト
--- @param key string 検索キー（シーン名/関数名）
--- @param global_scene_name string|nil 互換性のため残す（未使用）
--- @param attrs table|nil 属性テーブル（互換性のため残す・未使用）
--- @return function|nil 見つかったハンドラ関数、またはnil
function ACT_IMPL.find_scene(self, key, global_scene_name, attrs)
    return self:find_handler("scene", key)
end

--- シーン呼び出し（find_handler 委譲 + scene ポストプロセス）
---
--- トランスパイラ出力から呼び出され、キーに対応するハンドラーを検索して実行する。
--- find_handler("scene", key) でハンドラーを取得し、function なら直接呼ぶ。
---
--- @param self Act アクションオブジェクト
--- @param global_scene_name string|nil グローバルシーン名（互換性のため残す・未使用）
--- @param key string 検索キー
--- @param attrs table|nil 属性テーブル（互換性のため残す・未使用）
--- @param ... any 可変長引数（ハンドラーに渡す）
--- @return any ハンドラーの戻り値、またはnil
function ACT_IMPL.call(self, global_scene_name, key, attrs, ...)
    -- nil ガード: 式評価結果が nil の場合（未定義変数等）
    if key == nil then
        log.warn("act:call - nil key (undefined variable?), skipping scene search")
        return nil
    end

    local handler = self:find_handler("scene", key)

    -- scene ポストプロセス
    if type(handler) == "function" then
        return handler(self, ...)
    end

    log.warn(string.format("act:call - handler not found: key='%s', mode='scene', via=act",
        tostring(key)))
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
