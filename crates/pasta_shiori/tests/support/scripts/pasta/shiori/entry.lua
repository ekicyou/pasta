--- @module pasta.shiori.entry
--- SHIORI/3.0 Protocol Entry Point (Minimal Test Fixture)
---
--- This module defines the global SHIORI table with load, request, and unload functions.
--- These are called by PastaShiori (Rust) to handle SHIORI protocol events.

-- Global SHIORI table for protocol handling
SHIORI = SHIORI or {}

--- Handle SHIORI load event.
--- Called when the SHIORI DLL is loaded by the baseware.
---
--- @param hinst integer DLL handle
--- @param load_dir string Load directory path
--- @return boolean success Always returns true in minimal implementation
function SHIORI.load(hinst, load_dir)
    -- Minimal implementation: always succeed
    return true
end

--- Handle SHIORI/3.0 request.
--- Called for each SHIORI request from the baseware.
---
--- @param req table Parsed SHIORI request table with fields: id, method, version, charset, sender, reference, dic
--- @return string SHIORI response (204 No Content in minimal implementation)
function SHIORI.request(req)
    -- Minimal implementation: return 204 No Content
    return "SHIORI/3.0 204 No Content\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Sender: Pasta\r\n" ..
        "\r\n"
end

--- Handle SHIORI unload event.
--- Called when the SHIORI DLL is unloaded by the baseware.
function SHIORI.unload()
    -- Minimal implementation: do nothing
end

return SHIORI
