# Requirements Document

## Project Description (Input)
luaコード内で「act:wait(ms)」ってコードがあるが、この関数は存在しないのではないか？ACT:raw_scriptなら存在している。コードの深掘り調査を行い、実際に存在しないことが確認できれば実装を用意せよ。

## 調査結果

### 状況

`crates/pasta_shiori/tests/support/scripts/pasta/act.lua` はテスト用スタブファイルである。  
本実装 (`crates/pasta_lua/pasta_scripts/pasta/act.lua`) に存在するメソッドのうち、以下がスタブに未実装：

| メソッド                     | 本実装 (pasta_lua) | テストスタブ (pasta_shiori) |
| ---------------------------- | ------------------ | --------------------------- |
| `wait(ms)`                   | ✅ L217             | ❌ 未実装                    |
| `surface(id)`                | ✅ L208             | ❌ 未実装                    |
| `newline(n)`                 | ✅ L227             | ❌ 未実装                    |
| `clear()`                    | ✅ L235             | ❌ 未実装                    |
| `sakura_script(actor, text)` | ✅ L190             | ❌ 未実装                    |

### 使用箇所

`act:wait(ms)` は以下で呼び出される：
- `pasta_lua/pasta_scripts/pasta/shiori/entry.lua` L18: `GLOBAL.close_ghost` 内
- `pasta_lua/tests/lua_specs/act_test.lua`: 複数のテスト
- `pasta_lua/tests/lua_specs/act_grouping_test.lua`
- `pasta_lua/tests/lua_specs/shiori_act_test.lua`

### ACT:wait の仕様（act_test.lua より）

- `{ type = "wait", ms = ms }` トークンを `self.token` に追加
- 負の値は `0` に変換
- 小数点以下を切り捨て (`math.floor`)
- `nil` は `0` として扱う
- メソッドチェーン用に `self` を返す

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
