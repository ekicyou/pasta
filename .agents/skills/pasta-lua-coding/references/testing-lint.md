# テスト・Lint リファレンス

pasta_luaクレートにおけるLuaスクリプトのテストフレームワーク（lua_test）、テストファイル規約、決定論的テスト手法、luacheck設定の完全リファレンス。

---

## lua_test フレームワーク

### 基本構文

```lua
local describe = require("lua_test").describe
local test = require("lua_test").test
local expect = require("lua_test").expect
```

### describe / test

```lua
describe("モジュール名", function()
    test("テストケースの説明", function()
        -- テスト本体
        expect(actual_value).to_equal(expected_value)
    end)
end)
```

ネストも可能:

```lua
describe("WORD", function()
    describe("create_global", function()
        test("新しいキーを作成できる", function()
            -- ...
        end)
    end)
end)
```

### expect マッチャー一覧

| マッチャー | 説明 | 例 |
|-----------|------|-----|
| `to_equal(v)` | 等値比較 | `expect(1 + 1).to_equal(2)` |
| `to_be_truthy()` | truthy判定 | `expect(value).to_be_truthy()` |
| `to_be_falsy()` | falsy判定 | `expect(nil).to_be_falsy()` |
| `to_be_nil()` | nil判定 | `expect(result).to_be_nil()` |
| `to_be_type(t)` | 型チェック | `expect(obj).to_be_type("table")` |
| `to_contain(v)` | テーブル内包含 | `expect(list).to_contain("item")` |
| `to_throw()` | エラー発出確認 | `expect(fn).to_throw()` |

### テスト内でのpcall

テスト本体が例外を投げる場合はpcallでラップし、結果をexpectで検証する:

```lua
test("不正な引数でエラーを返す", function()
    local ok, err = pcall(function()
        MODULE.func(nil)
    end)
    expect(ok).to_be_falsy()
    expect(err).to_contain("invalid argument")
end)
```

---

## テストファイル規約

### ファイル命名

| 対象ソースファイル | テストファイル名 |
|-------------------|----------------|
| `pasta/word.lua` | `tests/word_test.lua` |
| `pasta/act.lua` | `tests/act_test.lua` |
| `pasta/store.lua` | `tests/store_test.lua` |

テストファイルは `tests/` ディレクトリに配置し、`_test.lua` サフィックスを付ける。

### init.lua 登録

テストファイルは `tests/init.lua` に登録する。登録しないとテストランナーが認識しない。

```lua
-- tests/init.lua
require("tests.word_test")
require("tests.act_test")
require("tests.store_test")
```

### テストファイル構造

```lua
-- tests/word_test.lua
local describe = require("lua_test").describe
local test = require("lua_test").test
local expect = require("lua_test").expect

-- テスト対象のモジュールを読み込み
local WORD = require("pasta.word")
local STORE = require("pasta.store")

describe("WORD", function()
    -- セットアップ: 各テストの前提状態を構築
    -- ※ lua_testにはbeforeEach/afterEachはない
    -- テストごとにSTOREをリセットする

    test("create_globalで新規キーが作成される", function()
        STORE.global_words = {}  -- リセット
        WORD.create_global("fruit"):entry("apple", "banana")
        expect(STORE.global_words["fruit"]).to_be_truthy()
    end)
end)
```

---

## 決定論的テスト

ランダム選択を含むモジュール（シーン選択・単語選択）のテストでは、セレクターを固定する。

### set_scene_selector

`@pasta_search` モジュールのシーン選択ロジックを固定する。

```lua
local pasta_search = require("@pasta_search")

-- 常に最初のシーンを選択
pasta_search.set_scene_selector(function(scenes)
    return scenes[1]
end)
```

### set_word_selector

`@pasta_search` モジュールの単語選択ロジックを固定する。

```lua
local pasta_search = require("@pasta_search")

-- 常に最初の候補を選択
pasta_search.set_word_selector(function(candidates)
    return candidates[1]
end)
```

### リセット

テスト後は `nil` を渡してデフォルト動作に戻す:

```lua
-- テスト終了後にリセット
pasta_search.set_scene_selector(nil)
pasta_search.set_word_selector(nil)
```

### 決定論的テストの例

```lua
describe("トーク選択", function()
    test("特定のシーンが選択される", function()
        local pasta_search = require("@pasta_search")

        -- 常に2番目のシーンを選択するよう固定
        pasta_search.set_scene_selector(function(scenes)
            return scenes[2]
        end)

        local result = SCENE.search("talk")
        expect(result).to_be_truthy()
        -- 特定のシーンが返されることを検証

        -- リセット
        pasta_search.set_scene_selector(nil)
    end)
end)
```

> 📖 `@pasta_search` モジュールの完全なAPI仕様は [runtime-api.md](runtime-api.md) を参照。

---

## luacheck

### .luacheckrc 設定

プロジェクトルートの `.luacheckrc` で設定する。

```lua
-- .luacheckrc
std = "lua51"

-- pasta.dll が提供するグローバル関数
globals = {
    "REG",      -- SHIORIハンドラ登録
    "RES",      -- SHIORIレスポンス生成
}

-- pasta.dll が提供する読み取り専用グローバル
read_globals = {
    "STORE",    -- 共有データストア
    "ACT",      -- アクションオブジェクト
    "SCENE",    -- シーンモジュール
    "WORD",     -- 単語モジュール
    "GLOBAL",   -- グローバル永続化
    "SAVE",     -- セーブデータ永続化
}

-- テストディレクトリ固有設定
files["tests/**"] = {
    globals = {
        "describe",
        "test",
        "expect",
    },
}
```

### 実行コマンド

```bash
# 全ファイルチェック
luacheck scripts/

# 特定ファイルのみ
luacheck scripts/pasta/word.lua

# エラー詳細表示
luacheck scripts/ --codes
```

### Rustテスト統合

`cargo test` で lua_test テストを実行する:

```bash
# Luaテストを含む全テスト実行
cargo test -p pasta_lua

# Luaテストのみ（テスト名フィルタ）
cargo test -p pasta_lua lua_test
```

lua_test フレームワークはRust側の `#[test]` 関数から呼び出される。テスト結果はRustのテストランナーを通じて報告される。
