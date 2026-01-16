--- Test main.lua for integration tests
--- Captures hinst and load_dir for verification

-- Global SHIORI table
SHIORI = SHIORI or {}

-- Storage for test verification
SHIORI._test = {
    hinst = nil,
    load_dir = nil,
    request_count = 0,
    last_request = nil,
}

--- Handle SHIORI load event.
--- @param hinst integer DLL handle
--- @param load_dir string Load directory path
--- @return boolean success
function SHIORI.load(hinst, load_dir)
    SHIORI._test.hinst = hinst
    SHIORI._test.load_dir = load_dir
    return true
end

--- Handle SHIORI/3.0 request.
--- @param request_text string Raw SHIORI request text
--- @return string SHIORI response
function SHIORI.request(request_text)
    SHIORI._test.request_count = SHIORI._test.request_count + 1
    SHIORI._test.last_request = request_text

    return "SHIORI/3.0 204 No Content\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Sender: Pasta\r\n" ..
        "X-Request-Count: " .. SHIORI._test.request_count .. "\r\n" ..
        "\r\n"
end

return SHIORI
