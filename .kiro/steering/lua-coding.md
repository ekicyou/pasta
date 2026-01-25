# Luaコーディング規約

pasta_luaクレートにおけるLuaスクリプト開発のコーディング規約を定義する。

---

## 1. 命名規約

### 1.1 基本命名規則

| 対象 | 命名スタイル | 例 |
|------|-------------|-----|
| ローカル変数 | snake_case | `local my_var = 1` |
| ローカル関数 | snake_case | `local function do_something()` |
| モジュールテーブル | UPPER_CASE | `local MOD = {}` |
| 定数 | UPPER_CASE | `local MAX_SIZE = 100` |
| プライベートメンバー | `_`プレフィックス | `self._internal = true` |
| クラス実装メタテーブル | `_IMPL`サフィックス | `local WORD_BUILDER_IMPL = {}` |

### 1.2 禁止パターン

```lua
-- ❌ 禁止: PascalCase
local WordBuilder = {}

-- ✅ 推奨: UPPER_CASE + _IMPL
local WORD_BUILDER_IMPL = {}
```

### 1.3 日本語識別子

日本語識別子は許可する。ただし公開API・モジュールテーブルには使用しない。

```lua
-- ✅ 許可: 内部変数・関数
local function 時報(act)
    return "正午です"
end

-- ✅ 許可: グローバル関数テーブルのエントリ
GLOBAL.時報 = function(act)
    return "正午です"
end
```

---

## 2. モジュール構造規約

### 2.1 標準モジュール構造

```lua
--- @module pasta.example
--- モジュールの説明（1行）
---
--- 詳細な説明（複数行可）

-- 1. require文は先頭に配置
local STORE = require("pasta.store")
local OTHER = require("pasta.other")

-- 2. モジュールテーブル宣言（UPPER_CASE）
local MOD = {}

-- 3. ローカル関数・定数

-- 4. 公開関数

-- 5. 末尾で返却
return MOD
```

### 2.2 モジュール命名

- モジュールテーブル名はファイル名に対応させる
- 例: `word.lua` → `local WORD = {}`（または `MOD`）
- 複数単語: `actor_builder.lua` → `local ACTOR_BUILDER = {}`

### 2.3 循環参照回避パターン

`pasta.store`は他のモジュールをrequireしない。共有データはSTOREに配置し、他モジュールがSTOREをrequireする。

```lua
-- store.lua - 他モジュールをrequireしない
local STORE = {}
STORE.actors = {}
STORE.scenes = {}
return STORE

-- actor.lua - STOREをrequire
local STORE = require("pasta.store")
-- ...
```

---

## 3. クラス設計パターン

### 3.1 MODULE/MODULE_IMPL分離パターン

クラスを持つモジュールは、モジュールテーブルとクラス実装メタテーブルを分離する。

```lua
--- @module pasta.word

local STORE = require("pasta.store")

-- モジュールテーブル（公開API）
local WORD = {}

-- クラス実装メタテーブル（インスタンスメソッド）
--- @class WordBuilder
--- @field _registry table
--- @field _key string
local WORD_BUILDER_IMPL = {}
WORD_BUILDER_IMPL.__index = WORD_BUILDER_IMPL

--- 値を追加
--- @param self WordBuilder
--- @param ... string 可変長引数
--- @return WordBuilder
function WORD_BUILDER_IMPL.entry(self, ...)
    local values = { ... }
    if #values > 0 then
        table.insert(self._registry[self._key], values)
    end
    return self
end

--- ビルダーを作成（ファクトリ関数）
--- @param key string 単語キー
--- @return WordBuilder
function WORD.create_global(key)
    if not STORE.global_words[key] then
        STORE.global_words[key] = {}
    end
    local builder = {
        _registry = STORE.global_words,
        _key = key,
    }
    return setmetatable(builder, WORD_BUILDER_IMPL)
end

return WORD
```

### 3.2 ドット構文 vs コロン構文

| 用途 | 構文 | 例 |
|-----|------|-----|
| **メソッド定義** | ドット構文 + 明示的self | `function IMPL.method(self, arg)` |
| **メソッド呼び出し** | コロン構文（許可） | `obj:method(arg)` |

```lua
-- ✅ 推奨: メソッド定義はドット構文
function WORD_BUILDER_IMPL.entry(self, ...)
    -- ...
end

-- ✅ 許可: 呼び出しはコロン構文
builder:entry("value1", "value2")
```

### 3.3 コンストラクタパターン

```lua
--- @class Instance
local MODULE_IMPL = {}
MODULE_IMPL.__index = MODULE_IMPL

--- @param args Args 引数
--- @return Instance
function MODULE.new(args)
    local obj = {
        field1 = args.field1,
        field2 = args.field2,
    }
    return setmetatable(obj, MODULE_IMPL)
end
```

### 3.4 シングルトンパターン

Luaの`require`キャッシング機構を活用する。

```lua
-- store.lua
local STORE = {}
STORE.data = {}
return STORE

-- 使用側: 常に同じインスタンスを取得
local STORE = require("pasta.store")
```

### 3.5 禁止パターン

```lua
-- ❌ 禁止: MODULE.instance() パターン
function MODULE.instance()
    if not _instance then
        _instance = MODULE.new()
    end
    return _instance
end

-- ❌ 禁止: コロン構文でのメソッド定義
function IMPL:method(arg)
    -- ...
end
```

---

## 4. EmmyLua型アノテーション規約

### 4.1 モジュールアノテーション

ファイル先頭に`@module`を配置する。

```lua
--- @module pasta.actor
--- アクターモジュール
---
--- アクターオブジェクトの管理とプロキシ生成を担当する。
```

### 4.2 クラスアノテーション

```lua
--- @class ClassName
--- @field fieldName type フィールドの説明
--- @field optionalField type|nil オプショナルフィールド
local CLASS_IMPL = {}
```

### 4.3 関数アノテーション

全公開関数に`@param`と`@return`を付与する。

```lua
--- 関数の説明
--- @param arg1 type 引数1の説明
--- @param arg2 type|nil オプショナル引数
--- @return ReturnType 戻り値の説明
function MODULE.func(arg1, arg2)
    -- ...
end
```

### 4.4 可変長引数

`@vararg`は使用せず、`@param ...`を使用する。

```lua
-- ✅ 推奨
--- @param ... string 可変長引数
function IMPL.entry(self, ...)

-- ❌ 禁止
--- @vararg string
function IMPL.entry(self, ...)
```

### 4.5 戻り値nil許容

```lua
--- @return Actor|nil アクター、または見つからない場合nil
function MODULE.find(name)
    return STORE.actors[name]
end
```

---

## 5. エラーハンドリング規約

### 5.1 nilチェックパターン

```lua
--- @return string|nil
function get_value(key)
    if not key or key == "" then
        return nil
    end
    return data[key]
end
```

### 5.2 ガードクローズパターン

関数の先頭で前提条件を検証し、早期リターンする。

```lua
function process(data)
    -- ガードクローズ
    if not data then
        return nil
    end
    if type(data) ~= "table" then
        return nil
    end
    
    -- メイン処理
    return transform(data)
end
```

### 5.3 pcall使用パターン

外部関数やリスクのある操作にはpcallを使用する。

```lua
local ok, result = pcall(function()
    return risky_operation()
end)
if not ok then
    return nil, result  -- エラーメッセージを返す
end
return result
```

### 5.4 禁止パターン

```lua
-- ❌ 禁止: サイレントnil返却（エラー条件が不明確）
function get_data(key)
    return data[key]  -- keyがnilの場合の動作が不明確
end

-- ✅ 推奨: 明示的なチェック
function get_data(key)
    if not key then
        return nil
    end
    return data[key]
end
```

---

## 6. Pasta固有ランタイム規約

### 6.1 PASTAモジュールAPI

`pasta/init.lua`が公開APIを提供する。

```lua
local PASTA = require("pasta")

-- アクター作成
local actor = PASTA.create_actor("さくら")

-- シーン登録
local scene = PASTA.create_scene("scene_name")

-- 単語定義
PASTA.create_word("キーワード")
    :entry("値1", "値2")
    :entry("値3")
```

### 6.2 CTXオブジェクト

セッション管理とコルーチン制御を担当する。

```lua
--- @class CTX
--- @field save table 永続変数
--- @field actors table<string, Actor> 登録アクター

-- 作成
local ctx = CTX.new(save_data, actors)

-- アクション開始
local act = ctx:start_action()

-- コルーチンでシーン実行
local co = ctx:co_action(scene_func, args)
```

### 6.3 ACTオブジェクト

トランスパイラー出力のシーン関数が受け取るオブジェクト。

```lua
--- @class Act
--- @field ctx CTX 環境オブジェクト
--- @field var table アクションローカル変数
--- @field token table[] 蓄積トークン
--- @field current_scene table|nil 現在のシーン

-- シーン関数内での使用
function scene(act)
    local save, var = act:init_scene(SCENE)
    act:talk(actor, "こんにちは")
    act:yield()
end
```

### 6.4 PROXYパターン

アクターへのプロキシオブジェクト。ACTへの逆参照を持つ。

```lua
--- @class ActorProxy
--- @field actor Actor
--- @field act Act

-- 使用例（トランスパイラー出力）
act.さくら:talk("こんにちは")
local word = act.さくら:word("名前")
```

### 6.5 STOREパターン

全ランタイムデータを一元管理する。循環参照を回避するため、他モジュールをrequireしない。

```lua
-- store.lua
local STORE = {}
STORE.actors = {}
STORE.scenes = {}
STORE.global_words = {}
STORE.local_words = {}
STORE.actor_words = {}
return STORE
```

---

## 7. テスト・Lint規約

### 7.1 テストフレームワーク

`lua_test`フレームワークを使用する（BDD風）。

```lua
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("モジュール名", function()
    describe("関数名", function()
        test("期待される動作", function()
            expect(result):toBe(expected)
        end)
    end)
end)
```

### 7.2 テストファイル命名

テストファイルは`*_test.lua`パターンを使用する。

```
crates/pasta_lua/tests/lua_specs/
├── actor_word_test.lua
├── transpiler_test.lua
└── init.lua
```

### 7.3 テスト構造テンプレート

```lua
--- @module tests.example_test
--- Exampleモジュールのテスト

local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

-- テスト対象
local Example = require("pasta.example")

describe("Example", function()
    describe("new", function()
        test("デフォルト値で作成できる", function()
            local instance = Example.new()
            expect(instance):not_:toBe(nil)
        end)
    end)
    
    describe("method", function()
        test("正常系: 期待値を返す", function()
            local instance = Example.new()
            expect(instance:method()):toBe("expected")
        end)
        
        test("異常系: nilを返す", function()
            local instance = Example.new()
            expect(instance:method(nil)):toBe(nil)
        end)
    end)
end)
```

### 7.4 luacheck設定

`crates/pasta_lua/.luacheckrc`にプロジェクト設定を配置する。

```lua
-- グローバル変数ホワイトリスト
globals = {
    "PASTA", "ACTOR", "SCENE", "WORD",
    "ACT", "CTX", "STORE", "GLOBAL",
}

-- UTF-8（日本語識別子）許可
allow_defined = true

-- 未使用変数警告（アンダースコアプレフィックス除外）
unused_args = false

-- 行長制限
max_line_length = 120
```

### 7.5 luacheck実行

```bash
# プロジェクトルートから
cd crates/pasta_lua
lua scriptlibs/luacheck/bin/luacheck.lua scripts/ --config .luacheckrc

# または簡易エイリアス（推奨）
# Makefileやスクリプトで設定
```

---

## 8. チェックリスト

コードレビュー・AI生成コード確認用のチェックリスト。

### 命名
- [ ] ローカル変数・関数はsnake_case
- [ ] モジュールテーブルはUPPER_CASE
- [ ] クラス実装メタテーブルは`_IMPL`サフィックス
- [ ] PascalCaseを使用していない

### 構造
- [ ] require文はファイル先頭
- [ ] モジュールテーブルはファイル末尾で返却
- [ ] 循環参照がない（STOREパターン使用）

### クラス
- [ ] MODULE/MODULE_IMPL分離
- [ ] メソッド定義はドット構文 + 明示的self
- [ ] setmetatableパターン使用

### 型注釈
- [ ] `@module`がファイル先頭にある
- [ ] 公開関数に`@param`/`@return`がある
- [ ] `@vararg`ではなく`@param ...`を使用

### エラー処理
- [ ] nilチェックが適切
- [ ] ガードクローズパターン使用
- [ ] サイレントnil返却がない
