# scriptlibs/ - External Lua Libraries

外部Luaライブラリ専用ディレクトリ

## 用途

- テストフレームワーク
- その他の外部ライブラリ

## package.path設定

```lua
package.path = "crates/pasta_lua/scriptlibs/?/init.lua;" .. package.path
```

## Lua require

```lua
local busted = require("busted")
-- other external libraries
```
