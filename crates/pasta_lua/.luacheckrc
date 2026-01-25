-- .luacheckrc
-- pasta_lua Luaスクリプト用の静的解析設定

-- Lua 5.1互換モード（mlua経由）
std = "lua51"

-- グローバル変数ホワイトリスト
globals = {
    "PASTA",     -- メインAPI (pasta/init.lua)
    "ACTOR",     -- アクターモジュール
    "SCENE",     -- シーンモジュール
    "WORD",      -- 単語モジュール
    "ACT",       -- アクションオブジェクト
    "CTX",       -- 環境コンテキスト
    "STORE",     -- データストア
    "GLOBAL",    -- グローバル関数テーブル
    "PROXY",     -- アクタープロキシ
}

-- 読み取り専用グローバル
read_globals = {
    "require",
    "print",
    "pairs",
    "ipairs",
    "type",
    "tostring",
    "tonumber",
    "setmetatable",
    "getmetatable",
    "table",
    "string",
    "math",
    "coroutine",
    "pcall",
    "xpcall",
    "error",
    "assert",
    "select",
    "unpack",
    "rawget",
    "rawset",
    "next",
}

-- 未使用変数の設定
-- アンダースコアプレフィックスの未使用変数は警告しない
unused_args = false
unused_secondaries = false

-- 未使用変数名のパターン（アンダースコアで始まる変数は無視）
ignore = {
    "21._.*",   -- 未使用変数（アンダースコアプレフィックス）
    "212",      -- 未使用引数
    "213",      -- 未使用ループ変数
}

-- 行長制限
max_line_length = 120

-- 複雑度警告
max_cyclomatic_complexity = 15

-- ファイル固有の設定
files = {
    -- テストファイルは追加のグローバルを許可
    ["tests/**/*.lua"] = {
        globals = {
            "describe",
            "test",
            "it",
            "expect",
            "before_each",
            "after_each",
        },
    },
    ["**/lua_specs/**/*.lua"] = {
        globals = {
            "describe",
            "test",
            "it",
            "expect",
            "before_each",
            "after_each",
        },
    },
}

-- 除外パターン
exclude_files = {
    "scriptlibs/**",   -- 外部ライブラリは除外
    ".luacheckrc",
}

-- コード品質設定
-- 未定義グローバルの参照を許可（require経由で定義される）
allow_defined = true
allow_defined_top = true

-- インライン設定を許可
inline = true
