--- @module pasta.act
--- アクションオブジェクトモジュール
---
--- トランスパイラー出力のシーン関数から第1引数として受け取るオブジェクト。
--- トークン蓄積、アクタープロキシ動的生成、シーン制御を担当する。

local ACTOR = require("pasta.actor")
local SCENE = require("pasta.scene")
local GLOBAL = require("pasta.global")
local STORE = require("pasta.store")
local WORD = require("pasta.word")
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
        if t == "spot" then
            -- spot はグループを閉じない
            table.insert(result, token)
        elseif t == "clear_spot" then
            -- clear_spot は現在のグループを閉じる（後の発言・表示制御は新しいグループに入る）
            table.insert(result, token)
            current_actor_token = nil
            current_actor = nil
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
            -- アクター行動トークン（surface, wait, newline, clear, choice, choice_timeout）
            -- 現在のアクターグループ内に追加
            -- 現在のグループが無い場合（出力の先頭や clear_spot の後で、発言より先に積まれた場合）は
            -- アクター nil（アクター未指定）のグループを開いて入れ、切替タグなしで積んだ位置に出す
            -- yield 直後の act:surface(…) など、1 回の出力の先頭に積むと起きる
            if not current_actor_token then
                current_actor_token = { type = "actor", actor = nil, tokens = {} }
                table.insert(result, current_actor_token)
                current_actor = nil
            end
            table.insert(current_actor_token.tokens, token)
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
--- nil は空文字扱い（トークンを積まない）。nil 以外は tostring する。
--- @param text any 発話テキスト
--- @param var_name string|nil 変数参照由来のとき変数パス（nil 時の警告用）
--- @return Act self メソッドチェーン用
function ACT_IMPL.talk(self, actor, text, var_name)
    if text == nil then
        if var_name then
            log.warn(string.format("act:talk - undefined variable: '%s'", var_name))
        end
        return self
    end
    table.insert(self.token, { type = "talk", actor = actor, text = tostring(text) })
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
    -- scope: 選択肢を出した時点の実行中のグローバルシーン名（シーンの外なら nil）
    local scope = self.current_scene and self.current_scene.__global_name__
    table.insert(self.token, { type = "choice", target = target, display = display or target, scope = scope })
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

--- ハンドラーフォールバック検索コア（5段階）
---
--- 検索レベル:
--- L1: current_scene[key] 完全一致（全モード共通）
--- L2: ローカル辞書前方一致（word: search_word(key, scene_name)、scene/expr: SCENE.search(key, scene_name)）
--- L3: self[key] function型のみ（ACT_IMPL・SHIORI_ACT_IMPL の継承チェーン。全モード共通）
--- L4: GLOBAL[key] 完全一致（全モード共通）
--- L5: グローバル辞書前方一致（word: search_word(key, nil)、scene/expr: SCENE.search(key, nil)）
--- @pasta_search は呼び出し毎に pcall(require) で取得し、取得できないときは全モードで L2・L5 をスキップする
--- （scene/expr モードの SCENE.search も内部で @pasta_search を使う）
---
--- skip_methods（動的参照用）: 継承したメソッドに届かせない。L1 は rawget で表自身のフィールドだけを引き
--- （SCENE_TABLE_IMPL の create_word 等に一致させない）、L3 は探さない。L2・L4・L5 は同じ。
---
--- @param self Act アクションオブジェクト
--- @param mode string "word" | "scene" | "expr"
--- @param key string 検索キー
--- @param skip_methods boolean|nil true で継承したメソッドに届かせない。nil/false は既存どおり
--- @return any|nil 見つかったハンドラー（関数または値）、またはnil
function ACT_IMPL.find_act_handler(self, mode, key, skip_methods)
    -- @pasta_search 可用性チェック（オプショナルモジュールのため呼び出し毎に pcall で確認。
    -- require はロード済みキャッシュを返すため低コスト）
    local ok, SEARCH = pcall(require, "@pasta_search")
    if not ok then SEARCH = nil end

    -- L1: current_scene[key] 完全一致（skip_methods 時は表自身のフィールドだけ）
    local scene = self.current_scene
    if scene then
        local value
        if skip_methods then value = rawget(scene, key) else value = scene[key] end
        if value ~= nil then return value end
    end

    -- L2: ローカル辞書前方一致（@pasta_search 利用可能かつ scene_name あり時のみ）
    local scene_name = self.current_scene and self.current_scene.__global_name__
    if SEARCH and scene_name then
        local result = search_dictionary(SEARCH, mode, key, scene_name)
        if result ~= nil then return result end
    end

    -- L3: self[key] function型のみ（act.XX / SHIORI_ACT_IMPL 継承チェーンを含む。skip_methods 時は飛ばす）
    if not skip_methods then
        local method = self[key]
        if type(method) == "function" then
            return method
        end
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
--- @param skip_methods boolean|nil find_act_handler へそのまま渡す
--- @return any|nil
function ACT_IMPL.find_handler(self, mode, key, skip_methods)
    return self:find_act_handler(mode, key, skip_methods)
end

--- 単語取得（find_handler + word ポストプロセス）
--- 検索順序は find_act_handler の5段階フォールバック（L1〜L5、word モード）
--- var_path があるとき（動的参照）は name を WORD.dynamic_key でキーにし、継承したメソッドに届かせずに検索する
--- ポストプロセス: handler=nil → warn+nil、function → h(self)、その他 → tostring(h)
--- @param self Act アクションオブジェクト
--- @param name any 単語名。var_path があるときは参照変数の値
--- @param var_path string|nil 動的参照の変数パス。nil なら既存の挙動（空キーは警告なしで nil）
--- @return string|nil 見つかった単語、またはnil
function ACT_IMPL.word(self, name, var_path)
    local skip_methods = nil
    if var_path ~= nil then
        name = WORD.dynamic_key(name, var_path, "act:word")
        if name == nil then return nil end
        skip_methods = true
    elseif not name or name == "" then
        return nil
    end

    local handler = self:find_handler("word", name, skip_methods)
    if handler == nil then
        log.warn(string.format("act:word - handler not found: key='%s', mode='word', via=act",
            tostring(name)))
        return nil
    end
    if type(handler) == "function" then
        local scene = self.current_scene
        return self:restore_scene(scene, handler(self))
    end
    return tostring(handler)
end

--- expr ポストプロセス（expr_fn・expr_fn_var 共通）: function → h(self, ...)、非function → warn+nil
--- @param self Act アクションオブジェクト
--- @param key string 関数名
--- @param skip_methods boolean|nil find_handler へ渡す
--- @param ... any 可変引数（ハンドラーに伝搬）
--- @return any ハンドラーの戻り値、またはnil
local function call_expr(self, key, skip_methods, ...)
    local handler = self:find_handler("expr", key, skip_methods)
    if type(handler) == "function" then
        local scene = self.current_scene
        return self:restore_scene(scene, handler(self, ...))
    end
    log.warn(string.format("act:expr_fn - handler not found: key='%s', mode='expr', via=act",
        tostring(key)))
    return nil
end

--- expr関数呼び出し（find_handler + expr ポストプロセス）
--- find_handler("expr", key) でハンドラーを取得してポストプロセスを実行する。
--- ポストプロセス: function → h(self, ...) 可変引数を伝搬、非function → warn+nil
--- @param self Act アクションオブジェクト
--- @param key string 関数名
--- @param ... any 可変引数（ハンドラーに伝搬）
--- @return any ハンドラーの戻り値、またはnil
function ACT_IMPL.expr_fn(self, key, ...)
    return call_expr(self, key, nil, ...)
end

--- 動的関数呼び出し（＠＄名前（…））
--- value を WORD.dynamic_key で関数名にし、継承したメソッドに届かせずに expr_fn と同じ検索・ポストプロセスを行う
--- @param self Act アクションオブジェクト
--- @param value any 参照変数の値
--- @param var_path string 参照変数の Lua パス（警告用）
--- @param ... any 可変引数（ハンドラーに伝搬）
--- @return any ハンドラーの戻り値、またはnil
function ACT_IMPL.expr_fn_var(self, value, var_path, ...)
    local key = WORD.dynamic_key(value, var_path, "act:expr_fn")
    if key == nil then return nil end
    return call_expr(self, key, true, ...)
end

--- アクター名からプロキシを得る（アクション行の生成コードが呼ぶ）
--- 登録済み（self.actors[name]）なら act.名前 と同じプロキシを返す。名前を文字列で受けるため
--- act のメンバー名と同名のアクターでも __index を通らない。
--- 未登録なら名前だけのその場限りのアクター（{ name = name }・メタテーブルなし）のプロキシを返す。
--- 直前の話者（self.token を末尾から見て最初の talk／sakura_script の actor）が同じ未登録名ならそれを再利用し、
--- そうでなければ警告を 1 行出して目印の talk トークンを積む。登録状態・立ち位置・act のフィールドには書かない
--- @param self Act アクションオブジェクト
--- @param name string アクター名
--- @return ActorProxy プロキシ（常に非 nil）
function ACT_IMPL.actor_proxy(self, name)
    local actor = self.actors[name]
    if actor then
        return ACTOR.create_proxy(actor, self)
    end
    for i = #self.token, 1, -1 do
        local t = self.token[i]
        if t.type == "talk" or t.type == "sakura_script" then
            if t.actor and t.actor.name == name then
                return ACTOR.create_proxy(t.actor, self)
            end
            break
        end
    end
    actor = { name = name }
    log.warn(string.format("act:actor_proxy - unregistered actor: name='%s'", tostring(name)))
    table.insert(self.token, { type = "talk", actor = actor, text = "【未登録アクター：" .. tostring(name) .. "】" })
    return ACTOR.create_proxy(actor, self)
end

--- グローバル関数呼び出し（＠＊名前（…））
--- GLOBAL[name] が関数なら act を第1引数にして呼ぶ（中のエラーはそのまま伝わる）。
--- 無い・関数でないときは警告して nil（act:expr_fn と同じ warn レベル）
--- @param self Act アクションオブジェクト
--- @param name string 関数名
--- @param ... any 関数に渡す引数
--- @return any ... 関数の戻り値すべて、または nil
function ACT_IMPL.global_fn(self, name, ...)
    local f = GLOBAL[name]
    if type(f) == "function" then
        local scene = self.current_scene
        return self:restore_scene(scene, f(self, ...))
    end
    log.warn(string.format("act:global_fn - function not found: key='%s'", tostring(name)))
    return nil
end

--- act:arith のネイティブ演算（メタメソッドに届く前に数値化済みの値だけを渡す）
local ARITH_OPS = {
    ["+"] = function(a, b) return a + b end,
    ["-"] = function(a, b) return a - b end,
    ["*"] = function(a, b) return a * b end,
    ["/"] = function(a, b) return a / b end,
    ["%"] = function(a, b) return a % b end,
}

--- 動的コールの「呼ばない」印（act:call_key が返し、act:call が検索せずに nil を返す）。外へ公開しない
local SKIP_CALL = {}

--- act:arith の警告に出す値の表記。table 等は tostring しない（__tostring を呼ばないため）
--- @param v any
--- @return string
local function arith_value_text(v)
    local t = type(v)
    if t == "nil" then return "nil" end
    if t == "string" then return string.format("'%s' (string)", v) end
    if t == "boolean" then return string.format("%s (boolean)", tostring(v)) end
    return string.format("(%s)", t)
end

--- 被演算子を数値にする。number はそのまま、string は tonumber（変更前の暗黙変換と同じ範囲）、
--- それ以外は nil。数値にできないときは警告する（値も説明も nil なら内側の失敗の伝播として黙る）
--- @param op string 演算子
--- @param v any 被演算子
--- @param desc string|nil 被演算子の説明（変数パス・関数名）
--- @return number|nil
local function arith_operand(op, v, desc)
    local t = type(v)
    local n = (t == "number" and v) or (t == "string" and tonumber(v)) or nil
    if n ~= nil or (v == nil and desc == nil) then
        return n
    end
    local value = arith_value_text(v)
    local operand = desc and string.format("operand='%s', ", desc) or ""
    log.warn(string.format("act:arith - operand is not a number: op='%s', %svalue=%s", op, operand, value))
    return nil
end

--- 数値の二項演算（算術式の生成コードが呼ぶ）
--- 両方が数値にできれば Lua のネイティブ演算の結果、どちらかが数値にできなければ nil。
--- act の状態は読み書きしない
--- @param self Act アクションオブジェクト（未使用）
--- @param op string "+" | "-" | "*" | "/" | "%"
--- @param lhs any 左の被演算子
--- @param rhs any 右の被演算子
--- @param lhs_desc string|nil 左の説明（警告用）
--- @param rhs_desc string|nil 右の説明（警告用）
--- @return number|nil 演算結果
function ACT_IMPL.arith(self, op, lhs, rhs, lhs_desc, rhs_desc) -- luacheck: ignore 212/self
    local f = ARITH_OPS[op]
    if not f then
        log.warn(string.format("act:arith - unknown operator: op='%s'", tostring(op)))
        return nil
    end
    local a = arith_operand(op, lhs, lhs_desc)
    local b = arith_operand(op, rhs, rhs_desc)
    if a == nil or b == nil then
        return nil
    end
    return f(a, b)
end

--- 被演算子を文字列にする。string はそのまま、number は tostring（アクション行の表示と同じ表記）、
--- それ以外は nil。文字列にできないときは警告する（値も説明も nil なら内側の失敗の伝播として黙る）
--- @param v any 被演算子
--- @param desc string|nil 被演算子の説明（変数パス・関数名）
--- @return string|nil
local function concat_operand(v, desc)
    local t = type(v)
    if t == "string" then return v end
    if t == "number" then return tostring(v) end
    if v ~= nil or desc ~= nil then
        local operand = desc and string.format("operand='%s', ", desc) or ""
        log.warn(string.format("act:concat - operand is not a string or number: op='&', %svalue=%s",
            operand, arith_value_text(v)))
    end
    return nil
end

--- 文字列の二項連結（連結式の生成コードが呼ぶ）
--- 両方が文字列か数値なら区切りなしでつないだ文字列、どちらかがそうでなければ nil。
--- メタメソッドは呼ばない。act の状態は読み書きしない
--- @param self Act アクションオブジェクト（未使用）
--- @param lhs any 左の被演算子
--- @param rhs any 右の被演算子
--- @param lhs_desc string|nil 左の説明（警告用）
--- @param rhs_desc string|nil 右の説明（警告用）
--- @return string|nil 連結結果
function ACT_IMPL.concat(self, lhs, rhs, lhs_desc, rhs_desc) -- luacheck: ignore 212/self
    local a = concat_operand(lhs, lhs_desc)
    local b = concat_operand(rhs, rhs_desc)
    if a == nil or b == nil then
        return nil
    end
    return a .. b
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
--- @param key string|table 検索キー、または act:call_key が返した「呼ばない」印（検索せず nil）
--- @param attrs table|nil 属性テーブル（互換性のため残す・未使用）
--- @param ... any 可変長引数（ハンドラーに渡す）
--- @return any ハンドラーの戻り値、またはnil
function ACT_IMPL.call(self, global_scene_name, key, attrs, ...)
    -- nil ガード: 式評価結果が nil の場合（未定義変数等）
    if key == nil then
        log.warn("act:call - nil key (undefined variable?), skipping scene search")
        return nil
    end
    -- 「呼ばない」印: call_key が警告と失敗表記を出し済み
    if key == SKIP_CALL then
        return nil
    end

    local handler = self:find_handler("scene", key)

    -- scene ポストプロセス
    if type(handler) == "function" then
        return handler(self, ...)
    end

    local warning = string.format("act:call - handler not found: key='%s', mode='scene', via=act",
        tostring(key))
    return self:failure(string.format("Call失敗：「%s」が見つからない", tostring(key)), warning)
end

--- 動的コールの式の値を検索キーにする（生成コードが act:call / act:call_restore のキーに渡す）
---
--- 空でない文字列（"nil" を含む）はそのまま、数値は tostring。それ以外（nil・空文字列・真偽値・表など）は
--- 警告（var_path があれば WORD.dynamic_key と同じ文言、なければ被演算子の表記。値 nil で説明もなければ
--- 内側の演算が警告済みとして黙る）と失敗表記を出し、「呼ばない」印を返す。
--- @param self Act アクションオブジェクト
--- @param value any 動的コールの式の値（生値）
--- @param var_path string|nil 式が変数参照 1 つのときの Lua パス（"var.x" / "save.x" / "args[1]"）
--- @param desc string|nil 式が関数呼び出し 1 つのときの表記（"@名前()" / "@*名前()" / "@$パス()"）
--- @return string|table 検索キー、または「呼ばない」印
function ACT_IMPL.call_key(self, value, var_path, desc)
    local t = type(value)
    if t == "number" then return tostring(value) end
    if t == "string" and value ~= "" then return value end
    local warning
    if var_path then
        WORD.dynamic_key(value, var_path, "act:call") -- 警告だけ出す（キーにできないことは判定済み）
    elseif value ~= nil or desc ~= nil then
        local operand = desc and string.format("operand='%s', ", desc) or ""
        warning = string.format("act:call - key is not a string or number: %svalue=%s",
            operand, arith_value_text(value))
    end
    local subject = var_path or desc
    subject = subject and (subject .. " が") or "値が"
    self:failure("Call失敗：" .. subject .. (value == "" and "空文字列" or (" " .. t)), warning)
    return SKIP_CALL
end

--- 実行中のシーンを scene に戻し、残りの引数をそのまま返す
--- @param self Act アクションオブジェクト
--- @param scene SceneTable|nil 戻す先（呼び出し前の act.current_scene）
--- @param ... any そのまま返す値
--- @return any ...
function ACT_IMPL.restore_scene(self, scene, ...)
    self.current_scene = scene
    return ...
end

--- act:call と同じ引数で呼び、戻った後に実行中のシーンを呼び出し前のものへ戻す
---
--- DSL の途中の Call 行が使う（末尾の Call は act:call のまま）。Lua からも呼べる。
--- 呼ばれた側がシーン・Lua の関数・見つからないのいずれでも戻す。STORE.last_global_scene は触らない。
--- @param self Act アクションオブジェクト
--- @param global_scene_name string|nil 未使用（act:call と同じ）
--- @param key string|table 検索キー、または act:call_key が返した「呼ばない」印
--- @param attrs table|nil act:call へそのまま渡す
--- @param ... any 呼ばれた側へ渡す引数
--- @return any ... 呼ばれた側の戻り値
function ACT_IMPL.call_restore(self, global_scene_name, key, attrs, ...)
    local scene = self.current_scene
    return self:restore_scene(scene, self:call(global_scene_name, key, attrs, ...))
end

--- 失敗表記の唯一の出口: 警告があればログへ出し、【text】をアクター無しの talk トークンとして積む
---
--- アクター nil の talk は切替タグを出さずに積んだ位置（直前に話したアクターのバルーン、
--- または出力の先頭なら現在のスコープ）へ文字を出す。
--- @param self Act アクションオブジェクト
--- @param text string 失敗表記の中身（【】はこの関数が付ける）
--- @param warning string|nil ログに出す警告文。nil なら出さない
--- @return nil
function ACT_IMPL.failure(self, text, warning)
    if warning then
        log.warn(warning)
    end
    table.insert(self.token, { type = "talk", actor = nil, text = "【" .. text .. "】" })
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
