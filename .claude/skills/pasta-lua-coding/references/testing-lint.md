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

`@pasta_search` の `set_scene_selector` / `set_word_selector` に整数を 1 個以上渡すと、シャッフルをやめ、渡した整数（候補の並びの 0 始まりの位置）の順に候補を返すようになる。`(0)` なら候補の並びの順、`(1)` なら 2 番目の候補から返す。整数の意味・引数の扱い・設定がランタイム全体に効くことは [pasta-search.md](pasta-search.md#set_scene_selector--set_word_selector) を、候補の並びの規則は [pasta-search.md](pasta-search.md#候補の選び方) を正とする（本節は手順の例のみ）。

### 固定する

```lua
local SEARCH = require("@pasta_search")

SEARCH:set_scene_selector(0)  -- シーン検索を決定論的にする
SEARCH:set_word_selector(0)   -- 単語検索を決定論的にする
```

### リセット

テスト後は**引数なし**で呼び出してデフォルト（シャッフル）動作に戻す。設定はランタイム全体の検索に効くため、戻し忘れると後続のテストに影響する。

```lua
-- テスト終了後にリセット（引数なし = デフォルトに戻す）
SEARCH:set_scene_selector()
SEARCH:set_word_selector()
```

### 決定論的テストの例

```lua
describe("トーク選択", function()
    test("同じ順でシーンが選択される", function()
        local SEARCH = require("@pasta_search")

        -- シャッフルをやめて決まった順にする
        SEARCH:set_scene_selector(0)

        local result = SCENE.search("talk")
        expect(result).to_be_truthy()
        -- 期待するシーンが返されることを検証（順序の規則は pasta-search.md）

        -- リセット（引数なしでデフォルトに復帰）
        SEARCH:set_scene_selector()
        SEARCH:set_word_selector()
    end)
end)
```

> 📖 `@pasta_search` モジュールの完全なAPI仕様は [pasta-search.md](pasta-search.md#set_scene_selector--set_word_selector) を参照。

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
luacheck pasta_scripts/

# 特定ファイルのみ
luacheck pasta_scripts/pasta/word.lua

# エラー詳細表示
luacheck pasta_scripts/ --codes
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

---

## モックライブラリ (lua_test.mocks)

`lua_test.mocks` モジュール（`require("lua_test.mocks")`）は、Rustバックエンドモジュール5つのデフォルトスタブを一括で `package.loaded` に注入するライブラリ。テストごとの手動 `package.loaded` 設定ボイラープレートを排除する。

### API

| 関数 | 説明 |
|------|------|
| `mocks.install(opts?)` | 全5モジュールのスタブを `package.loaded` に登録。`opts` でモジュール単位のカスタムスタブを上書き可能 |
| `mocks.reset()` | 全5モジュールの `package.loaded` エントリを `nil` に戻す |

### 基本利用パターン

```lua
local mocks = require("lua_test.mocks")

describe("イベントハンドラ", function()
    test("OnBootが正常に動作する", function()
        mocks.install()   -- 全5モジュールを一括スタブ化

        local event = require("pasta.shiori.event")
        -- ... テストロジック ...

        mocks.reset()     -- クリーンアップ
    end)
end)
```

### カスタムスタブの指定

`opts` テーブルで特定モジュールのみ置き換え可能。指定しないモジュールはデフォルトスタブが使われる。

```lua
mocks.install({
    persistence = {
        load = function() return { talk_count = 5 } end,
        save = function(_data) return true end,
    },
    log = {
        trace = function() end,
        debug = function() end,
        info  = function(msg) print("[INFO] " .. msg) end,
        warn  = function() end,
        error = function() end,
    },
})
```

### 対象モジュールとデフォルトスタブ

| キー名 | モジュール名 | デフォルトスタブの動作 |
|--------|-------------|----------------------|
| `persistence` | `@pasta_persistence` | `load` → `{}`、`save` → `true` |
| `search` | `@pasta_search` | メタテーブルキャッチオール（任意メソッド呼び出しで `nil` を返す関数） |
| `sakura_script` | `@pasta_sakura_script` | `talk_to_script` → テキスト返却、`break_lines` → テキスト返却 |
| `config` | `@pasta_config` | 空テーブル `{}` |
| `log` | `@pasta_log` | `trace`/`debug`/`info`/`warn`/`error` → noop |

---

## 関連リファレンス

- [pasta-search.md](pasta-search.md#set_scene_selector--set_word_selector) — `@pasta_search` の `set_scene_selector` / `set_word_selector` 完全APIシグネチャ
