--- @module pasta.shiori.event
--- イベント振り分けモジュール
---
--- SHIORI リクエストのイベント ID に応じてハンドラを呼び分ける（鎖 → REG の値 → シーンの順）。
--- 鎖も REG の値も無いイベントはデフォルトハンドラ（no_entry）で処理する。
--- no_entry ではシーン関数フォールバックを試み、見つからなければ 204 を返す。
--- エラーは呼び出し元（SHIORI.request）の xpcall でキャッチされる。
---
--- ハンドラシグネチャ:
---   function(act: ShioriAct) -> string|thread|nil
---
--- 戻り値の規則（EVENT.fire）:
---   - "SHIORI/" で始まる文字列: RES で作った応答全体として包まずにそのまま返す
---   - それ以外の文字列: Value の 200（空文字列なら 204）
---   - nil: 204
---   - thread: EVENT.drive で再開し、出力を接頭辞にかかわらず常に Value の 200 にする（出力が無ければ 204）
---
--- 再開の手順は EVENT.drive に一本化している。通常のイベント（EVENT.fire）・
--- コールバックの再開（CALLBACK.try_route）・タイムアウト掃引（CALLBACK.sweep）の
--- 3 つの呼び出し元が同じ手順（予約の消費・継続の更新）でシーンを再開する。
---
--- act オブジェクト経由で SHIORI リクエスト情報にアクセス:
---   - act.req.id: イベント名（例: "OnBoot", "OnClose"）
---   - act.req.method: "get" | "notify"
---   - act.req.version: 30（SHIORI/3.0）
---   - act.req.charset: 文字セット（例: "UTF-8"）
---   - act.req.sender: 送信者名（例: "SSP"）
---   - act.req.reference: 参照テーブル（reference[0], reference[1], ...）
---   - act.req.dic: 全ヘッダー辞書
---   - act.req.date: 日付情報（date.unix, date.hour, date.min, date.sec, etc.）
---
--- 注意: act.req は読み取り専用として扱うこと。変更は未定義動作となる。
---
--- Rust側統合パターン（pasta.shiori.entry。実際は xpcall で保護して呼ぶ）:
--- ```lua
--- local EVENT = require("pasta.shiori.event")
---
--- function SHIORI.request(req)
---     return EVENT.fire(req)
--- end
--- ```
---
--- 使用例（ハンドラ登録）:
--- ```lua
--- local REG = require("pasta.shiori.event.register")
---
--- -- 返した文字列は Value の 200 になる（SHIORI/ で始まる文字列は応答全体としてそのまま返る）
--- REG.OnBoot = function(act)
---     act.sakura:talk("こんにちは")
---     return act:build()
--- end
--- ```
---
--- ハンドラの引き方（鎖 → 表の値 → シーン）:
--- まず REG:register で重ねたハンドラ（鎖）を新しく登録した順に呼ぶ。nil・false は見送って
--- 次へ回り、真の値を返したらそこで止めて、その戻り値を上の規則で応答にする。
--- 鎖の写しは呼ぶ前に 1 回だけ取るので、途中の登録・外しは次のイベントから効く。
--- 全部見送れば（鎖が空でも）表の生の値 rawget(REG, req.id) を呼ぶ（戻り値は真偽で見ない）。
--- 鎖のハンドラも表の値も、呼ぶたびに新しい act を受け取る。
---
--- シーン関数フォールバック:
--- 鎖が全部見送り、REG にハンドラも未登録の場合、SCENE.co_exec(act, req.id) でシーンを検索し、
--- 見つかればシーンコルーチンを EVENT.drive で再開してさくらスクリプトを 200 OK で返す。
--- 見つからなければ 204 No Content を返す。
---
--- テスト用reqテーブル:
--- ```lua
--- local test_req = {
---     id = "OnTest",
---     method = "get",
---     version = 30,
--- }
--- ```

-- 1. require文
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")
local SHIORI_ACT = require("pasta.shiori.act")
local STORE = require("pasta.store")
local CALLBACK = require("pasta.shiori.event.callback")
local STOP = require("pasta.shiori.stop")
local CLOCK = require("pasta.shiori.clock")

-- 1.5. デフォルトイベントハンドラをロード
require("pasta.shiori.event.boot")
require("pasta.shiori.event.choice_select")
require("pasta.shiori.event.second_change")
require("pasta.shiori.event.mouse")
require("pasta.shiori.event.scene_play")
require("pasta.shiori.event.external_control")

-- 2. モジュールテーブル宣言
--- @class EVENT
local EVENT = {}

-- 3. 内部関数

--- actオブジェクトを作成
--- @param req table SHIORIリクエストテーブル
--- @return ShioriAct actオブジェクト
local function create_act(req)
    return SHIORI_ACT.new(STORE.actors, req)
end

--- STORE.co_sceneを統一管理するローカル関数
--- LuaJIT 2.1 には coroutine.close が無いため、以下の close 分岐は実行されず、
--- 破棄は参照を外すだけになる（破棄したコルーチンは GC が回収する）。
--- @param co thread|nil コルーチンまたはnil
--- @param keep boolean 真なら待っている続きをそのまま残し、co の残りは捨てる
local function set_co_scene(co, keep)
    -- 1. 引数検証（suspended以外はclose）
    if co and coroutine.status(co) ~= "suspended" then
        if coroutine.close then
            coroutine.close(co)
        end
        co = nil
    end

    -- 待っている続きを残す: co は待ちの表が持つか、どこにも置かずに捨てる
    if keep then
        if STORE.co_callback and co == STORE.co_callback then
            STORE.co_callback = nil
        end
        return
    end

    -- NEW: コールバック登録済みコルーチン検出
    if STORE.co_callback and co == STORE.co_callback then
        -- コールバック待ちコルーチンは CALLBACK.pending で管理される
        -- co_scene には登録しない、旧 co_scene と同一なら close もしない
        if STORE.co_scene and STORE.co_scene ~= co then
            -- 別の旧コルーチンがある場合は通常通り close
            if coroutine.close then coroutine.close(STORE.co_scene) end
        end
        STORE.co_scene = nil
        STORE.co_callback = nil
        return
    end

    -- 2. 同一オブジェクトチェック
    if STORE.co_scene == co then
        return
    end

    -- 3. 旧コルーチンをclose（存在すれば無条件）
    if STORE.co_scene then
        if coroutine.close then
            coroutine.close(STORE.co_scene)
        end
    end

    -- 4. 上書き（coはsuspendedまたはnil確定）
    STORE.co_scene = co
end

---@alias DriveKind "resume"|"none"|"preempt"|"interrupt"

--- 再開の前に、このコルーチンの割り込みの種類を決める（判定の順はこのとおり）
--- @param co thread
--- @return DriveKind
local function drive_kind(co)
    if STORE.co_scene == nil then
        return "none"
    elseif co == STORE.co_scene then
        return "resume"
    elseif STORE.co_preempt[co] then
        return "preempt"
    end
    return "interrupt"
end

--- nil yieldをスキップして有効値またはdead状態まで繰り返しresumeする
--- @param co thread コルーチン
--- @param ... any 初回resume引数（通常はact）
--- @return boolean ok 処理成功フラグ（エラー時false）
--- @return any value 有効値またはエラーメッセージ
local function resume_until_valid(co, ...)
    -- 初回resumeは引数を渡す
    local ok, value = coroutine.resume(co, ...)

    -- ループ: nil yieldをスキップ
    while true do
        -- エラー時は即座に返す
        if not ok then
            return ok, value
        end

        -- 有効値（nil以外）の場合はループ終了
        if value ~= nil then
            return ok, value
        end

        -- dead状態のnilは有効値として扱う（空シーン）
        if coroutine.status(co) == "dead" then
            return ok, value
        end

        -- nil + suspended: 引数なしで再度resume
        ok, value = coroutine.resume(co)
    end
end

--- 中断のシーンを動かして、まとめたトークン列を返す。
--- 見つからない・出力が無い・待ちに入ったなら nil を返す。
--- 中断のシーンがエラーで止まったら、そのエラーを投げ直す（割り込んだシーンのコルーチンが止まる）
--- @param act table 割り込んだシーンの act（act.req を中断の act に渡す）
--- @return table[]|nil
local function run_interrupt_scene(act)
    -- 遅延ロードで循環参照回避
    local SCENE = require("pasta.scene")
    local ico = SCENE.co_exec_exact("チェイントーク中断")
    if not ico then
        return nil
    end
    local iact = SHIORI_ACT.new(STORE.actors, act.req)
    -- 文字列にせず、まとめたトークン列で受け取る（割り込んだ側の組み立てが差し替えて 1 回で文字列にする）
    iact.build = require("pasta.act").IMPL.build
    -- 最初に出力のある区切りまで。残りのコルーチンは参照を外して捨てる
    local ok, groups = resume_until_valid(ico, iact)
    -- 中断の中の get_property の予約で、割り込んだシーンが待ちに登録されないようにする
    local waited = CALLBACK.has_staged()
    CALLBACK.discard_staged()
    if not ok then
        -- 止まるエラーの表記をふつうの組み立てで作るため、インスタンスの build を外す
        iact.build = nil
        -- 投げ直す前に結び直す: STOP.hold が止まった場所を中断のシーンとして控えるため
        require("pasta.failure").bind(iact)
        error(groups, 0)
    end
    if waited then
        return nil
    end
    return groups
end

-- 4. 公開関数

--- デフォルトハンドラ（未登録イベント用）
--- シーン関数をイベント名で検索し、見つかった場合はthreadを返す。
--- 見つからない場合はnilを返す（EVENT.fireでRES.no_content()に変換される）。
--- @param act ShioriAct actオブジェクト
--- @return thread|nil シーンコルーチン、またはnil
function EVENT.no_entry(act)
    -- シーン関数をイベント名で検索（遅延ロードで循環参照回避）
    local SCENE = require("pasta.scene")
    return SCENE.co_exec(act, act.req.id, nil, nil)
end

--- シーンコルーチンを出力が得られるか終わるまで再開し、予約の消費と継続の更新までを行う
--- 失敗時（エラーで終わったとき）は消費されていない予約を捨て、継続を空にする。
--- 応答文字列は作らず、エラーを投げるかどうかも呼び出し元が決める。
--- @param co thread 再開するシーンコルーチン（suspended、CALLBACK.pending から外し済み）
--- @param act table このコルーチンに紐づく act（待機を登録するときに使う）
--- @param ... any 最初の resume に渡す引数
--- @return boolean ok false ならシーンがエラーで終わった
--- @return any value ok=true: 出力（nil は出力の無いまま終了）／ok=false: エラーの値
function EVENT.drive(co, act, ...)
    local FAILURE = require("pasta.failure")
    -- 1. 種類は再開の前に決める（再開の中で続きが変わる前の状態で判断するため）
    local kind = drive_kind(co)
    -- 2. 印を整える（新しいシーンはまだ act が記録されていないので、最初の再開に渡す act に掛ける）
    local target = FAILURE.act_of(co) or act
    if target then
        target._spoke = nil
        target._interrupted = nil
        if kind == "interrupt" then
            target._on_speech = run_interrupt_scene
        end
    end
    -- 3. nil yieldをスキップ
    local ok, value = resume_until_valid(co, ...)
    -- 4. 欄の関数は成否にかかわらずすぐ外す（後で STOP.response が build しても中断を動かさないため）
    local scene_act = FAILURE.act_of(co) or act
    if target then target._on_speech = nil end
    if scene_act then scene_act._on_speech = nil end
    if not ok then
        -- 止まった場所の表記のため、失敗したコルーチンとエラーの値を控える
        STOP.hold(co, value)
        -- 予約の後、中断の前にエラーで終わったシーンの予約を残さない
        CALLBACK.discard_staged()
        -- 5. dead のコルーチンを渡し継続を空にする
        set_co_scene(co, false)
        return false, value
    end
    -- 6. ステージング消費（待ちに入ったかを set_co_scene が STORE.co_callback で見るので先に行う）
    CALLBACK.consume_staged(co, act)
    -- 7. 台詞の印が無い素のコルーチンは、出力が空でないかで決める
    local spoke = scene_act and scene_act._spoke
    if spoke == nil then
        spoke = value ~= nil and value ~= ""
    end
    -- 8. 状態保存（set_co_scene内部でstatus判断）
    if scene_act and scene_act._interrupted then
        -- 中断の台詞が出たので、古い続きも co の残りも捨てる
        set_co_scene(nil, false)
    else
        -- 台詞の無い割り込みだけが待っている続きを残す
        set_co_scene(co, kind == "interrupt" and not spoke)
    end
    return true, value
end

--- イベント振り分け
--- 待機中のコールバックに一致すれば CALLBACK.try_route の応答を返す。
--- 次に CLOCK.observe に合図と返事を見せ、返事（OnPastaClock）を受け取った回は 204 を返す。
--- それ以外は鎖 → REG の値 → シーンの順にハンドラを実行し、戻り値をモジュール冒頭の規則で応答にする（thread は EVENT.drive で再開）
--- @param req table リクエストテーブル（req.id にイベント名）
--- @return string SHIORI レスポンス
function EVENT.fire(req)
    -- (新規) コールバックルーティング
    local cb_response = CALLBACK.try_route(req)
    if cb_response then
        return cb_response
    end

    -- 合図と返事を時計に見せる。返事（OnPastaClock）を受け取った回は REG へ進まない
    if CLOCK.observe(req) then
        return RES.no_content()
    end

    -- 鎖（REG:register）を新しい順に呼び、真の値を返したものの act と戻り値で応答する
    -- ハンドラごとに新しい act を作り、止まったときの表記のため呼ぶたびに控え直す
    -- エラーは SHIORI.request の xpcall でキャッチされる
    local act, result
    for _, fn in ipairs(getmetatable(REG).handlers(req.id)) do
        act = create_act(req)
        STOP.hold_handler(act)
        result = fn(act)
        if result then
            break
        end
    end

    -- 全部見送れば底（REG[id] の生の値、無ければシーンを探す既定）を呼ぶ。底の戻り値は真偽で見ない
    if not result then
        act = create_act(req)
        STOP.hold_handler(act)
        result = (rawget(REG, req.id) or EVENT.no_entry)(act)
    end

    -- 型判定
    if type(result) == "thread" then
        local ok, yielded_value = EVENT.drive(result, act, act)
        if not ok then
            error(yielded_value)
        end
        -- シーンの出力は接頭辞にかかわらず常に Value にする
        return RES.ok(yielded_value)
    elseif type(result) == "string" then
        -- SHIORI/ で始まる文字列は RES で作った応答全体として包まずに返す
        if result:sub(1, 7) == "SHIORI/" then
            return result
        end
        return RES.ok(result)
    else
        -- nil
        return RES.no_content()
    end
end

--- テスト用: resume_until_validを公開
--- @param co thread コルーチン
--- @param ... any 初回resume引数
--- @return boolean ok
--- @return any value
function EVENT._resume_until_valid(co, ...)
    return resume_until_valid(co, ...)
end

-- 5. 末尾で返却
return EVENT
