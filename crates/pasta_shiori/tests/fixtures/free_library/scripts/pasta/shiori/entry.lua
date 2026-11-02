--- @module pasta.shiori.entry
--- テストフィクスチャ: unload を呼ばずに外すテスト（free_library_test.rs）のゴースト（仕様 `ponytail-shiori`）
---
--- SHIORI.load でシステムの DLL（version.dll）を ffi.load し、VM が落ちるまで持ち続ける
--- （VM の片付けでその DLL の FreeLibrary が走る＝Lua の C モジュールを読んだ辞書の形）。
--- GET には ffi.load が通ったかを `Value: ffi=true|false` で返す。

SHIORI = SHIORI or {}

local loaded = false
local version_dll = nil

function SHIORI.load(hinst, load_dir)
    loaded, version_dll = pcall(ffi.load, "version")
    return true
end

function SHIORI.request(req)
    return "SHIORI/3.0 200 OK\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Value: ffi=" .. tostring(loaded) .. "\r\n" ..
        "\r\n"
end

return SHIORI
