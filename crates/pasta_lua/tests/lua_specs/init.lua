-- lua_specs エントリーポイント
-- このファイルから各テストスイートを require して実行する
--
-- 新しいテストを追加する場合：
--   1. lua_specs/ に *_spec.lua ファイルを作成
--   2. 下記の specs テーブルにモジュール名を追加

local specs = {
    "transpiler_spec",
    -- 将来のテストスイートをここに追加
    -- "code_generator_spec",
    -- "context_spec",
}

print("=====================================")
print("Running Lua test suites...")
print("=====================================")

local passed = 0
local failed = 0

for _, spec_name in ipairs(specs) do
    print(string.format("\n[SUITE] %s", spec_name))
    local ok, err = pcall(function()
        require(spec_name)
    end)
    if ok then
        passed = passed + 1
        print(string.format("  ✅ %s loaded successfully", spec_name))
    else
        failed = failed + 1
        print(string.format("  ❌ %s failed: %s", spec_name, tostring(err)))
    end
end

print("\n=====================================")
print(string.format("Test suites: %d passed, %d failed", passed, failed))
print("=====================================")

if failed > 0 then
    error(string.format("%d test suite(s) failed", failed))
end
