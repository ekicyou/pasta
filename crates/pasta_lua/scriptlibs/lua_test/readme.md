# Lua Testing Library

A tiny testing library for Lua. Supports [Lua language server](https://github.com/LuaLS/lua-language-server)

## Usage

```lua
local describe = require("src.test").describe
local test = require("src.test").test
local expect = require("src.test").expect

describe("toBe", function()
    test("1 + 2 to be 3", function()
        expect(1 + 2):toBe(3)
    end)

    test("1 + 2 not to be 4", function()
        expect(1 + 2).not_:toBe(4)
    end)

    test("Same table", function()
        local tbl = {}
        expect(tbl):toBe(tbl)
    end)

    test("{} not to be {}", function()
        expect({}).not_:toBe({})
    end)
end)
```

```
toBe (4/4)✔
  1 + 2 to be 3✔
  1 + 2 not to be 4✔
  Same table✔
  {} not to be {}✔
toBe (4/4)✔
All tests passed.
```

## Configuration

- Set `NO_COLOR` in environment variable to disable colored output.

## Functions

### describe

Makes groups of tests.

### test

Register test.

### expect

Makes expectation. See `src/expect.lua`.
