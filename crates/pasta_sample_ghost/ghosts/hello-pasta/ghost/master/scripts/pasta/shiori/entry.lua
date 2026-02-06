--- @module pasta.shiori.entry
--- SHIORI/3.0 プロトコル エントリーポイント
---
--- グローバル SHIORI テーブルを定義し、load/request/unload 関数を提供します。
--- これらの関数は PastaShiori (Rust) から呼び出され、SHIORI プロトコルイベントを処理します。
---
--- イベント処理は EVENT モジュールに委譲され、リクエストIDに基づいて
--- 登録されたハンドラにディスパッチされます。

local EVENT = require "pasta.shiori.event"
local RES = require "pasta.shiori.res"

-- グローバルシーン関数の登録
local GLOBAL = require "pasta.global"

function GLOBAL.close_ghost(act, ms)
    if type(ms) == "number" and ms >= 1 then
        act:wait(ms)
    end
    act:raw_script([=[\-]=])
end

GLOBAL.ゴースト終了 = GLOBAL.close_ghost

-- グローバル SHIORI テーブルを初期化（既存の場合は維持）
SHIORI = SHIORI or {}

--- xpcall用エラーハンドラ
--- @param err any エラーオブジェクト
--- @return string|nil エラーメッセージの最初の行
local function error_handler(err)
    if type(err) == "string" then
        return err:match("^[^\n]+")
    end
    return nil
end

--- SHIORI load イベントを処理
--- ベースウェアによって SHIORI DLL がロードされた時に呼び出されます。
---
--- @param hinst integer DLL インスタンスハンドル
--- @param load_dir string ロードディレクトリパス (ghost/master/)
--- @return boolean success 常に true を返却（将来の初期化処理の拡張ポイント）
function SHIORI.load(hinst, load_dir)
    local ok, result = xpcall(function()
        -- 将来の拡張ポイント:
        -- - 設定ファイルの読み込み
        -- - セーブデータの復元
        -- - リソースの初期化
        return true
    end, error_handler)

    if ok then
        return result
    else
        print("[SHIORI.load] Error: " .. tostring(result))
        return false
    end
end

--- SHIORI/3.0 リクエストを処理
--- ベースウェアからの各 SHIORI リクエストに対して呼び出されます。
---
--- イベントベースのディスパッチのため EVENT.fire() に委譲します。
--- req テーブルは Rust 側でパース済みで、以下のフィールドを含みます:
---
--- @param req table パース済み SHIORI リクエストテーブル
--- @field req.id string イベントID（例: "OnBoot", "OnSecondChange"）
--- @field req.method string HTTP風メソッド（"GET", "NOTIFY"）
--- @field req.version string プロトコルバージョン（例: "3.0"）
--- @field req.charset string 文字エンコーディング（例: "UTF-8"）
--- @field req.sender string 送信者識別子
--- @field req.reference table Reference 値の配列（0始まりインデックス）
--- @field req.dic table カスタムヘッダーの辞書
--- @return string SHIORI/3.0 形式のレスポンス
function SHIORI.request(req)
    local ok, result = xpcall(function()
        return EVENT.fire(req)
    end, error_handler)

    if ok then
        return result
    else
        return RES.err(result)
    end
end

--- SHIORI unload イベントを処理
--- ベースウェアによって SHIORI DLL がアンロードされる時に呼び出されます。
---
--- DLL が解放される前にクリーンアップ処理を実行します。
function SHIORI.unload()
    local ok, err = xpcall(function()
        -- 将来の拡張ポイント:
        -- - セーブデータの永続化
        -- - リソースの解放
        -- - ロギングの終了処理
    end, error_handler)

    if not ok then
        print("[SHIORI.unload] Error: " .. tostring(err))
    end
end

return SHIORI
