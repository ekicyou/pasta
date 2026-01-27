--- @module pasta.shiori.entry
--- SHIORI/3.0 プロトコル エントリーポイント
---
--- グローバル SHIORI テーブルを定義し、load/request/unload 関数を提供します。
--- これらの関数は PastaShiori (Rust) から呼び出され、SHIORI プロトコルイベントを処理します。
---
--- イベント処理は EVENT モジュールに委譲され、リクエストIDに基づいて
--- 登録されたハンドラにディスパッチされます。

local EVENT = require "pasta.shiori.event"

-- グローバル SHIORI テーブルを初期化（既存の場合は維持）
SHIORI = SHIORI or {}

--- SHIORI load イベントを処理
--- ベースウェアによって SHIORI DLL がロードされた時に呼び出されます。
---
--- @param hinst integer DLL インスタンスハンドル
--- @param load_dir string ロードディレクトリパス (ghost/master/)
--- @return boolean success 常に true を返却（将来の初期化処理の拡張ポイント）
function SHIORI.load(hinst, load_dir)
    -- 将来の拡張ポイント:
    -- - 設定ファイルの読み込み
    -- - セーブデータの復元
    -- - リソースの初期化
    return true
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
    return EVENT.fire(req)
end

--- SHIORI unload イベントを処理
--- ベースウェアによって SHIORI DLL がアンロードされる時に呼び出されます。
---
--- DLL が解放される前にクリーンアップ処理を実行します。
function SHIORI.unload()
    -- 将来の拡張ポイント:
    -- - セーブデータの永続化
    -- - リソースの解放
    -- - ロギングの終了処理
end

return SHIORI
