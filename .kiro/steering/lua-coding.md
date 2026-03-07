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

### 3.5 継承パターン

サブクラスは親のIMPLを継承チェーンで参照する。親モジュールは`IMPL`フィールドで公開する。

```lua
-- 親モジュール: IMPL公開
local ACT = {}
local ACT_IMPL = {}
-- ...
ACT.IMPL = ACT_IMPL  -- 継承のために公開
return ACT

-- 子モジュール: 継承チェーン設定
local ACT = require("pasta.act")
local CHILD_ACT = {}
local CHILD_ACT_IMPL = {}

-- 継承チェーン: CHILD_ACT_IMPL → ACT.IMPL
setmetatable(CHILD_ACT_IMPL, { __index = ACT.IMPL })

-- __index オーバーライド: rawget で自クラスを先に検索
function CHILD_ACT_IMPL.__index(self, key)
    local method = rawget(CHILD_ACT_IMPL, key)
    if method then return method end
    return ACT.IMPL.__index(self, key)  -- 親の__indexに委譲
end

CHILD_ACT.IMPL = CHILD_ACT_IMPL  -- 更なる継承のために公開

function CHILD_ACT.new(actors)
    local base = ACT.new(actors)  -- 親コンストラクタを呼ぶ
    -- 子クラス固有フィールドを追加
    base._extra = "value"
    return setmetatable(base, CHILD_ACT_IMPL)
end

return CHILD_ACT
```

### 3.6 禁止パターン

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

-- アクター作成（または取得）
local actor = PASTA.create_actor("さくら")

-- シーン登録
local scene = PASTA.create_scene("scene_name")

-- 単語定義
PASTA.create_word("キーワード")
    :entry("値1", "値2")
    :entry("値3")

-- シーン辞書最終化（Rustから上書きされるスタブ）
PASTA.finalize_scene()
```

### 6.2 saveモジュールと永続化

`pasta.save`は`@pasta_persistence`経由でセッション間永続データをロードする。
ACTオブジェクトが`save`フィールドとして参照する。

```lua
-- pasta/save.lua（内部実装）
local persistence = require("@pasta_persistence")
local save = persistence.load()
return save

-- 使用側: ACT経由でアクセス
function scene(act)
    local save, var = act:init_scene(SCENE)
    -- save: セッション間永続（@pasta_persistence管理）
    -- var:  アクション内一時変数
    save.count = (save.count or 0) + 1
end
```

### 6.3 ACTオブジェクト

トランスパイラー出力のシーン関数が受け取るオブジェクト。

```lua
--- @class Act
--- @field actors table<string, Actor> 登録アクター
--- @field save table 永続変数（pasta.save）
--- @field app_ctx table アプリケーション実行中の汎用コンテキスト
--- @field var table アクションローカル変数
--- @field token table[] 蓄積トークン
--- @field current_scene SceneTable|nil 現在のシーン
--- @field req ShioriRequest|nil SHIORIリクエスト（ShioriActのみ）

-- シーン関数内での使用
function scene(act)
    local save, var = act:init_scene(SCENE)
    act:talk(actor, "こんにちは")
    act:yield()  -- トークンをコルーチンyield
end

-- ACTメソッド一覧
-- act:init_scene(scene) → save, var
-- act:talk(actor, text) → self
-- act:raw_script(text) → self
-- act:surface(id) → self
-- act:wait(ms) → self
-- act:newline(n) → self
-- act:clear() → self
-- act:set_spot(name, number) → nil
-- act:clear_spot() → nil
-- act:word(name) → string|nil   （4段階検索）
-- act:call(global, key, attrs, ...) → any
-- act:build() → table[]|nil     （グループ化トークン）
-- act:yield() → self            （build()してコルーチンyield）
```

### 6.4 PROXYパターン

アクターへのプロキシオブジェクト。ACTへの逆参照を持ち、3段階単語検索を実装。

```lua
--- @class ActorProxy
--- @field actor Actor
--- @field act Act

-- 使用例（トランスパイラー出力）
act.さくら:talk("こんにちは")
local word = act.さくら:word("名前")  -- 3段階検索: actor→actor辞書→act:word()
```

### 6.5 STOREパターン

全ランタイムデータを一元管理する。循環参照を回避するため、他モジュールをrequireしない。

```lua
-- store.lua
local STORE = {}
STORE.actors = {}          -- table<string, Actor>    アクターキャッシュ
STORE.actor_spots = {}     -- table<string, integer>  スポット位置マップ
STORE.scenes = {}          -- table<string, table>    シーンレジストリ
STORE.counters = {}        -- table<string, number>   シーン名カウンタ
STORE.global_words = {}    -- table<string, table>    グローバル単語レジストリ
STORE.local_words = {}     -- table<string, table>    ローカル単語レジストリ
STORE.actor_words = {}     -- table<string, table>    アクター単語レジストリ
STORE.app_ctx = {}         -- table                   汎用コンテキストデータ
STORE.co_scene = nil       -- thread|nil              継続コルーチン（OnTalk等）

-- 全データリセット（テスト・再初期化用）
function STORE.reset()
    if STORE.co_scene then
        if coroutine.status(STORE.co_scene) == "suspended" then
            coroutine.close(STORE.co_scene)
        end
        STORE.co_scene = nil
    end
    STORE.actors = {}
    -- 他フィールドも同様にリセット
end

return STORE
```

**例外**: Rust組み込みモジュール`@pasta_config`のみpcall経由でrequireする（実行環境の違いに対応）。

```lua
local ok, CONFIG = pcall(require, "@pasta_config")
if ok and type(CONFIG.actor) == "table" then
    STORE.actors = CONFIG.actor
end
```

### 6.6 Rustネイティブモジュールパターン

`@pasta_*`プレフィックスのモジュールはRust側で提供されるネイティブモジュール。

| モジュール | アクセス方法 | 用途 |
|-----------|------------|------|
| `@pasta_search` | `require` / `pcall(require, ...)` | シーン・単語検索（Radix Trie） |
| `@pasta_persistence` | `require` | セッション永続化データ |
| `@pasta_config` | `pcall(require, ...)` | pasta.toml設定読み込み |
| `@pasta_sakura_script` | `require` | さくらスクリプト処理 |

```lua
-- 常に利用可能なモジュール: require直接
local SEARCH = require("@pasta_search")
local result = SEARCH:search_scene(name, global_scene_name)

-- オプショナルモジュール: pcallで保護（テスト環境等でも動作）
local ok, SEARCH = pcall(require, "@pasta_search")
if ok and SEARCH then
    local word = SEARCH:search_word(name, scene_name)
end

-- 設定モジュール: 常にpcall（単体テスト環境では存在しない可能性）
local ok, CONFIG = pcall(require, "@pasta_config")
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

テストファイルは`*_test.lua`または`*_spec.lua`パターンを使用する。

```
crates/pasta_lua/tests/lua_specs/
├── actor_word_test.lua        # _test.lua: 機能単位テスト
├── transpiler_test.lua
├── persistence_spec.lua       # _spec.lua: 仕様ベーステスト
├── virtual_dispatcher_spec.lua
└── init.lua                   # エントリーポイント（テストスイート登録）
```

**init.luaパターン**: テストスイートは`specs`テーブルに登録してpcallで実行。

```lua
-- tests/lua_specs/init.lua
local specs = {
    "module_test",
    "feature_spec",
    -- 追加はここに
}
for _, spec_name in ipairs(specs) do
    local ok, err = pcall(function() require(spec_name) end)
    if not ok then error(spec_name .. " failed: " .. tostring(err)) end
end
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
- [ ] 継承が必要な場合は`MODULE.IMPL = MODULE_IMPL`で公開

### 型注釈
- [ ] `@module`がファイル先頭にある
- [ ] 公開関数に`@param`/`@return`がある
- [ ] `@vararg`ではなく`@param ...`を使用

### エラー処理
- [ ] nilチェックが適切
- [ ] ガードクローズパターン使用
- [ ] サイレントnil返却がない

### Rustネイティブモジュール
- [ ] `@pasta_config`等オプショナルモジュールはpcall経由
- [ ] 常に利用可能なモジュール（`@pasta_search`等）はrequire直接可
