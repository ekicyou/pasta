# Lua Unit Tests

Busted テストフレームワークを使用したLuaユニットテスト

## 構成

- `*_spec.lua`: テストファイル（Busted 命名規約）
  - transpiler_spec.lua
  - code_generator_spec.lua
  - context_spec.lua

## 実行方法

```bash
# ローカル開発（luarocks 導入時）
busted --verbose

# 特定ファイル実行
busted tests/lua_specs/transpiler_spec.lua
```

## テスト構造

```lua
describe("Module Name", function()
  it("should do something", function()
    assert.truthy(result)
  end)
end)
```
