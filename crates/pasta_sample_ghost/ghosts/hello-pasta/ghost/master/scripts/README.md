# scripts/ - pasta_lua Lua Scripts Layer

自作Luaコード・スクリプト実装層

## 構成

- **root**: メインスクリプト（hello.lua, init.lua等）
- **helpers/**: ヘルパー関数ユーティリティ
- **examples/**: 使用例・サンプルスクリプト

## package.path設定

```lua
package.path = "crates/pasta_lua/scripts/?.lua;" .. package.path
package.path = "crates/pasta_lua/scripts/?/init.lua;" .. package.path
```

## Lua require

```lua
local helpers = require("helpers.string_utils")
local transpiler = require("transpiler")
```
