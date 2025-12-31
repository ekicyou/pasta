# Lua Testing Library

Lua用の軽量テストライブラリ。[Lua language server](https://github.com/LuaLS/lua-language-server) に対応しています。

## 特徴

- **依存ゼロ**: Pure Luaのみで動作
- **シンプルなAPI**: `describe`, `test`, `expect` の3つの関数
- **直感的な記述**: RSpec/Jest風のテスト記述
- **否定テスト**: `.not_` で期待値を反転
- **カラー出力**: テスト結果を見やすく表示

---

## 使い方

### 基本的なテスト

```lua
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("算術演算のテスト", function()
    test("1 + 2 は 3", function()
        expect(1 + 2):toBe(3)
    end)

    test("1 + 2 は 4 ではない", function()
        expect(1 + 2).not_:toBe(4)
    end)
end)
```

### テーブルの比較

```lua
describe("テーブル比較", function()
    test("同じテーブル参照", function()
        local tbl = {}
        expect(tbl):toBe(tbl)
    end)

    test("異なるテーブルインスタンス", function()
        expect({}).not_:toBe({})
    end)
end)
```

### 実行結果

```
算術演算のテスト (2/2)✔
  1 + 2 は 3✔
  1 + 2 は 4 ではない✔
テーブル比較 (2/2)✔
  同じテーブル参照✔
  異なるテーブルインスタンス✔
All tests passed.
```

---

## API リファレンス

### `describe(グループ名, 関数)`

テストをグループ化します。ネストも可能です。

```lua
describe("外側のグループ", function()
    describe("内側のグループ", function()
        test("テストケース", function()
            expect(true):toBe(true)
        end)
    end)
end)
```

### `test(テスト名, 関数)`

個別のテストケースを登録します。

```lua
test("説明文", function()
    -- テストコード
end)
```

### `expect(値)`

アサーション（検証）を行います。

**利用可能なマッチャー**:
- `:toBe(期待値)` - 厳密な等価比較（`==`）
- `:toBeTruthy()` - 真偽値が true
- `:toBeFalsy()` - 真偽値が false
- `.not_` - 期待値を反転（例: `expect(1).not_:toBe(2)`）

詳細は `expect.lua` を参照してください。

---

## 設定

### カラー出力を無効化

環境変数 `NO_COLOR` を設定すると、カラー出力が無効になります。

```bash
export NO_COLOR=1
lua your_test.lua
```

---

## テスト実行方法

### Lua スクリプトとして実行

```bash
# Lua インタープリタで直接実行
lua crates/pasta_lua/tests/lua_specs/transpiler_spec.lua
```

### VSCode デバッガで実行

1. F5 キーを押す
2. デバッグ設定「Lua (lua_specs tests)」を選択
3. ブレークポイントを設定してステップ実行可能

---

## 関連リンク

- [GitHub リポジトリ](https://github.com/Tsukina-7mochi/lua-testing-library)
- [Lua Language Server](https://github.com/LuaLS/lua-language-server)

---

## ライセンス

MIT License

