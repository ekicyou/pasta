# Coding Conventions リファレンス

pasta_luaクレートにおけるLuaスクリプト開発の命名規約、モジュール構造、クラス設計パターン、EmmyLua型注釈、エラーハンドリングの完全リファレンス。

---

## 命名規約

### 基本命名規則

| 対象 | 命名スタイル | 例 |
|------|-------------|-----|
| ローカル変数 | snake_case | `local my_var = 1` |
| ローカル関数 | snake_case | `local function do_something()` |
| モジュールテーブル | UPPER_CASE | `local MOD = {}` |
| 定数 | UPPER_CASE | `local MAX_SIZE = 100` |
| プライベートメンバー | `_`プレフィックス | `self._internal = true` |
| クラス実装メタテーブル | `_IMPL`サフィックス | `local WORD_BUILDER_IMPL = {}` |

### 禁止パターン

```lua
-- ❌ 禁止: PascalCase
local WordBuilder = {}

-- ✅ 推奨: UPPER_CASE + _IMPL
local WORD_BUILDER_IMPL = {}
```

### 日本語識別子

日本語識別子は内部変数・ローカル関数・GLOBAL登録では使用OK。公開API・モジュールテーブル名には使用NG。

```lua
-- ✅ 許可: 内部変数・関数
local function 時報(act)
    return "正午です"
end

-- ✅ 許可: グローバル関数テーブルのエントリ
GLOBAL.時報 = function(act)
    return "正午です"
end

-- ❌ 禁止: モジュールテーブル名や公開API名に日本語
local 単語 = {}  -- NG
```

---

## モジュール構造

### 標準モジュール構造

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

### モジュール命名

- モジュールテーブル名はファイル名に対応させる
- `word.lua` → `local WORD = {}`（または `MOD`）
- 複数単語: `actor_builder.lua` → `local ACTOR_BUILDER = {}`

### 循環参照回避（STOREパターン）

`pasta.store` は他のモジュールをrequireしない。共有データはSTOREに配置し、他モジュールがSTOREをrequireする一方向依存を維持。

```lua
-- store.lua - 他モジュールをrequireしない
local STORE = {}
STORE.actors = {}
STORE.scenes = {}
return STORE

-- actor.lua - STOREをrequire
local STORE = require("pasta.store")
STORE.actors["さくら"] = { name = "さくら" }
```

---

## クラス設計パターン

### MODULE/MODULE_IMPL分離

モジュールテーブル（公開API）とクラス実装メタテーブル（インスタンスメソッド）を分離する。

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

### ドット構文 vs コロン構文

| 用途 | 構文 | 例 |
|-----|------|-----|
| メソッド**定義** | ドット構文 + 明示的self | `function IMPL.method(self, arg)` |
| メソッド**呼び出し** | コロン構文（許可） | `obj:method(arg)` |

```lua
-- ✅ 推奨: メソッド定義はドット構文
function WORD_BUILDER_IMPL.entry(self, ...)
    -- ...
end

-- ✅ 許可: 呼び出しはコロン構文
builder:entry("value1", "value2")
```

### コンストラクタパターン

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

### シングルトンパターン

Luaの `require` キャッシング機構を活用する。`pasta.store` が代表例。

```lua
-- store.lua
local STORE = {}
STORE.data = {}
return STORE

-- 使用側: 常に同じインスタンスを取得
local STORE = require("pasta.store")
```

### 継承パターン

サブクラスは親のIMPLを継承チェーンで参照する。親モジュールは `IMPL` フィールドで公開する。

```lua
-- 親モジュール: IMPL公開
local ACT = {}
local ACT_IMPL = {}
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
    local base = ACT.new(actors)  -- 親コンストラクタ
    base._extra = "value"
    return setmetatable(base, CHILD_ACT_IMPL)
end

return CHILD_ACT
```

### 禁止パターン

```lua
-- ❌ 禁止: MODULE.instance() 手動管理
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

## EmmyLua型注釈

### @module / @class / @field

ファイル先頭に `@module` を配置、クラス定義直前に `@class` + `@field`。

```lua
--- @module pasta.actor
--- アクターモジュール
---
--- アクターオブジェクトの管理とプロキシ生成を担当する。
```

```lua
--- @class ClassName
--- @field fieldName type フィールドの説明
--- @field optionalField type|nil オプショナルフィールド
local CLASS_IMPL = {}
```

### @param / @return

全公開関数に `@param` と `@return` を付与する。

```lua
--- 関数の説明
--- @param arg1 type 引数1の説明
--- @param arg2 type|nil オプショナル引数
--- @return ReturnType 戻り値の説明
function MODULE.func(arg1, arg2)
    -- ...
end
```

**可変長引数**: `@vararg` は使用せず `@param ...` を使用する。

```lua
-- ✅ 推奨
--- @param ... string 可変長引数
function IMPL.entry(self, ...)

-- ❌ 禁止
--- @vararg string
function IMPL.entry(self, ...)
```

**戻り値nil許容**:

```lua
--- @return Actor|nil アクター、または見つからない場合nil
function MODULE.find(name)
    return STORE.actors[name]
end
```

---

## エラーハンドリング

### ガードクローズ

関数の先頭で前提条件を検証し、早期リターンする。

```lua
function process(data)
    -- ガードクローズ
    if not data then return nil end
    if type(data) ~= "table" then return nil end

    -- メイン処理
    return transform(data)
end
```

### pcall

外部関数やリスクのある操作に使用する。

```lua
local ok, result = pcall(function()
    return risky_operation()
end)
if not ok then
    return nil, result  -- エラーメッセージを返す
end
return result
```

### nilチェック

明示的な条件確認を行う。

```lua
--- @return string|nil
function get_value(key)
    if not key or key == "" then
        return nil
    end
    return data[key]
end
```

### 禁止パターン

```lua
-- ❌ 禁止: サイレントnil返却（エラー条件が不明確）
function get_data(key)
    return data[key]  -- keyがnilの場合の動作が不明確
end

-- ✅ 推奨: 明示的なチェック
function get_data(key)
    if not key then return nil end
    return data[key]
end
```

---

## チェックリスト

コードレビュー・AI生成コード確認用。

### 命名
- [ ] ローカル変数・関数はsnake_case
- [ ] モジュールテーブルはUPPER_CASE
- [ ] クラス実装メタテーブルは `_IMPL` サフィックス
- [ ] PascalCaseを使用していない

### 構造
- [ ] require文はファイル先頭
- [ ] モジュールテーブルはファイル末尾で返却
- [ ] 循環参照がない（STOREパターン使用）

### クラス
- [ ] MODULE/MODULE_IMPL分離
- [ ] メソッド定義はドット構文 + 明示的self
- [ ] setmetatableパターン使用
- [ ] 継承が必要な場合は `MODULE.IMPL = MODULE_IMPL` で公開

### 型注釈
- [ ] `@module` がファイル先頭にある
- [ ] 公開関数に `@param` / `@return` がある
- [ ] `@vararg` ではなく `@param ...` を使用

### エラー処理
- [ ] nilチェックが適切
- [ ] ガードクローズパターン使用
- [ ] サイレントnil返却がない

### Rustネイティブモジュール
- [ ] `@pasta_config` 等オプショナルモジュールはpcall経由
- [ ] 常に利用可能なモジュール（`@pasta_search` 等）はrequire直接可
