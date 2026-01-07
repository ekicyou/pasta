# Lua Unit Tests

lua_test テストフレームワークを使用したLuaユニットテスト

## 構成

- `init.lua`: エントリーポイント（各specをrequireして実行）
- `*_spec.lua`: テストファイル
  - transpiler_spec.lua

## テストの追加方法

1. `lua_specs/` に `*_spec.lua` ファイルを作成
2. `init.lua` の `specs` テーブルにモジュール名を追加

```lua
local specs = {
    "transpiler_spec",
    "your_new_spec",  -- 追加
}
```

## 実行方法

```bash
# Rust テストランナー経由（推奨）
cargo test --test lua_unittest_runner run_lua_unit_tests -- --nocapture

# pasta_lua の全テスト
cargo test -p pasta_lua
```

## テスト構造

```lua
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("Module Name", function()
    test("should do something", function()
        expect(result):toBeTruthy()
    end)
end)
```
