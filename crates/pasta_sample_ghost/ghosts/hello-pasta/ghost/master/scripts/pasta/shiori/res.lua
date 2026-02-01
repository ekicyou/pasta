--- @module pasta.shiori.res
--- SHIORIレスポンス組み立てモジュール
---
--- SHIORI/3.0プロトコルに準拠したレスポンス文字列を構築する。
--- ステータスコード別の便利関数を提供し、標準ヘッダー（Charset, Sender, SecurityLevel）を自動付与する。

-- 1. require文（なし - 外部依存ゼロ）

-- 2. モジュールテーブル宣言
local RES = {}

-- 3. 定数
local CRLF = "\r\n"
local SPLIT = ": "

-- 4. 環境設定テーブル
--- @class RESEnv 環境設定テーブル
--- @field charset string 文字セット（デフォルト: "UTF-8"）
--- @field sender string 送信者名（デフォルト: "Pasta"）
--- @field security_level string セキュリティレベル（デフォルト: "local"）
RES.env = {
    charset = "UTF-8",
    sender = "Pasta",
    security_level = "local",
}

-- 5. 公開関数

--- @alias HeaderDic table<string, string> ヘッダー辞書

--- 汎用ビルダー関数
--- SHIORI/3.0レスポンス文字列を生成する。
--- 標準ヘッダー3種（Charset, Sender, SecurityLevel）を常に出力し、
--- 追加ヘッダーを辞書から取得して付加する。
---
--- @param code string ステータスコード（例: "200 OK"）
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.build(code, dic)
    dic = dic or {}

    -- Status line
    local rc = "SHIORI/3.0 " .. code .. CRLF

    -- Standard headers (always in this order)
    rc = rc .. "Charset" .. SPLIT .. RES.env.charset .. CRLF
    rc = rc .. "Sender" .. SPLIT .. RES.env.sender .. CRLF
    rc = rc .. "SecurityLevel" .. SPLIT .. RES.env.security_level .. CRLF

    -- Additional headers from dic
    if type(dic) == 'table' then
        for k, v in pairs(dic) do
            rc = rc .. k .. SPLIT .. v .. CRLF
        end
    end

    -- Terminate with empty line
    rc = rc .. CRLF

    return rc
end

--- 200 OK レスポンス
--- Valueヘッダー付きの正常レスポンスを生成する。
---
--- @param value string Value ヘッダーの値
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.ok(value, dic)
    dic = dic or {}
    dic["Value"] = value
    return RES.build("200 OK", dic)
end

--- 204 No Content レスポンス
--- 返すべきデータがない場合の正常終了レスポンスを生成する。
---
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.no_content(dic)
    return RES.build("204 No Content", dic)
end

--- 311 Not Enough レスポンス（TEACH情報不足）
---
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.not_enough(dic)
    return RES.build("311 Not Enough", dic)
end

--- 312 Advice レスポンス（TEACH解釈不能）
---
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.advice(dic)
    return RES.build("312 Advice", dic)
end

--- 400 Bad Request レスポンス
---
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.bad_request(dic)
    return RES.build("400 Bad Request", dic)
end

--- 500 Internal Server Error レスポンス
---
--- @param reason string|nil エラー理由（nilまたは空文字列の場合は"Unknown error"）
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.err(reason, dic)
    dic = dic or {}
    if reason == nil or reason == "" then
        reason = "Unknown error"
    end
    dic["X-Error-Reason"] = reason
    return RES.build("500 Internal Server Error", dic)
end

--- ワーニング付き 204 No Content レスポンス
--- 警告情報を通知しつつ正常終了を示す。
---
--- @param reason string ワーニング理由
--- @param dic HeaderDic|nil 追加ヘッダー辞書
--- @return string SHIORI/3.0レスポンス文字列
function RES.warn(reason, dic)
    dic = dic or {}
    dic["X-Warn-Reason"] = reason
    return RES.no_content(dic)
end

-- 6. 末尾で返却
return RES
