--- @module pasta.config
--- 設定モジュール
---
--- @pasta_config（pasta.toml）から設定値を取得するためのラッパーモジュール。
--- セクションとキーを指定して設定値を取得し、存在しない場合はデフォルト値を返す。
---
--- @usage
--- local config = require("pasta.config")
--- local value = config.get("ghost", "spot_switch_newlines", 1.5)

-- @pasta_configはRust側で登録されるモジュール
-- テスト環境など利用不可の場合は空テーブルを使用
local ok, pasta_config = pcall(require, "@pasta_config")
if not ok then
    pasta_config = {}
end

--- @class PastaConfig
local PASTA_CONFIG = {}

--- 設定値を取得
---
--- @param section string セクション名（必須）
--- @param key string キー名
--- @param default any デフォルト値（設定が存在しない場合に返す）
--- @return any 設定値またはデフォルト値
function PASTA_CONFIG.get(section, key, default)
    if section == nil then
        error("section is required")
    end

    local section_table = pasta_config[section]
    if section_table == nil then
        return default
    end

    local value = section_table[key]
    if value == nil then
        return default
    end

    return value
end

return PASTA_CONFIG
