---
name: pasta-lua-coding
description: >-
  pasta.dll Luaランタイム APIリファレンスとコーディング規約。
  ゴーストの scripts/ 配下のカスタムLuaスクリプトや、
  Pasta DSL内のLuaブロック実装を支援する。
  USE FOR: pasta lua, pasta_lua, Lua API, Luaスクリプト, scripts/,
  単語辞書一括投入, WORD.create, イベントハンドラ, REG, RES,
  永続化, @pasta_persistence, save, @pasta_search,
  @pasta_config, @pasta_sakura_script, @enc,
  ACT, SCENE, STORE, GLOBAL, SAVE, lua_test, luacheck,
  pasta lua coding, pasta runtime API.
  DO NOT USE FOR: Pasta DSL文法, .pastaファイル編集,
  pasta_dsl crate, pasta_core crate, Rustクレート実装,
  汎用Luaプログラミング, SHIORIプロトコル実装.
metadata:
  author: ekicyou
  version: "1.0.0"
---

# Pasta Lua Coding Skill

## §1 Purpose & Prerequisites

**目的**: 自然言語の指示からpasta_luaランタイムに準拠したLuaコードを正確に生成するサポートを提供する。

**対象ドメイン**:
- `scripts/` 配下のカスタムLuaスクリプト（`main.lua` がエントリーポイント）
- Pasta DSL内の ` ```lua ``` ` ブロックで記述するシーン関数

**前提条件**: ゴーストプロジェクトが既に存在すること（`pasta.toml`、`dic/`、`scripts/` が揃っている）。

**役割分離**: 姉妹スキル `pasta-ghost-authoring` がPasta DSL文法（`.pasta`ファイルの記述）を担当し、本スキルはその下位層であるLuaランタイム層を担当する。DSLの ` ```lua ``` ` ブロック内のコード記述や、`scripts/` 配下の独自スクリプト開発を支援する。

**scripts/ フォルダ**: ゴーストディレクトリ直下の `scripts/` に独自Luaスクリプトを配置する。`main.lua` がエントリーポイントとしてpastaランタイムに読み込まれ、シーン関数・単語定義・イベントハンドラ等をセットアップする。

### DSL vs Lua 判断基準

| ケース | 推奨 | 理由 |
|--------|------|------|
| 数個の単語定義 | DSL (`＠単語：値1、値2`) | 宣言的で簡潔 |
| 数十〜数百件の単語一括投入 | Lua (`WORD.create_*`) | ループ/外部データ読み込みが必要 |
| 基本的なシーン定義 | DSL (`＊シーン名`) | 可読性が高い |
| 条件分岐を含む複雑なロジック | Lua (シーン関数) | DSLの制御構文は限定的 |
| カスタムSHIORIイベント処理 | Lua (REGテーブル) | DSLではイベントハンドラを直接定義不可 |
| 外部データ（JSON/YAML）の読み込み | Lua (`@json`/`@yaml`) | DSLには外部ファイル操作機能なし |

**自己完結性**: 本スキルは別リポジトリにコピーして単体で機能する。pastaリポジトリ内の他ファイルへの参照に依存しない。

**権威的情報ソース**: `steering/lua-coding.md`（コーディング規約）、`crates/pasta_lua/LUA_API.md`（APIリファレンス）

---

## §2 Quick Reference

### Rust組み込みモジュール

| モジュール | 用途 | require方法 |
|-----------|------|------------|
| `@pasta_search` | シーン・単語検索 | `require` 直接 |
| `@pasta_persistence` | セーブデータ永続化 | `require` 直接 |
| `@pasta_sakura_script` | さくらスクリプト変換 | `require` 直接 |
| `@enc` | UTF-8 ⇔ ANSI変換 | `require` 直接 |
| `@pasta_config` | pasta.toml設定読み取り | `pcall(require, ...)` 保護必須 |

### 内部Luaモジュール（pasta.*名前空間）

| モジュール | 用途 | 主要API |
|-----------|------|--------|
| `pasta.store` | 一元データ管理 | `STORE.actors`, `STORE.scenes`, `STORE.reset()` |
| `pasta.scene` | シーン登録・検索 | `SCENE.create_scene()`, `SCENE.search()` |
| `pasta.word` | 単語ビルダー | `WORD.create_global()`, `WORD.create_local()`, `WORD.create_actor()` |
| `pasta.global` | ユーザー定義関数 | `GLOBAL.関数名 = function(act) ... end` |
| `pasta.save` | 永続化データ | `require("pasta.save")` |
| `pasta.act` | シーン実行コンテキスト | `act:init_scene()`, `act:talk()`, `act:yield()` |
| `pasta.shiori.event.register` | イベントハンドラ登録 | `REG.EventName = function(req) ... end` |
| `pasta.shiori.res` | SHIORIレスポンス | `RES.ok()`, `RES.no_content()` |

### mlua-stdlib統合モジュール

| モジュール | 用途 | デフォルト |
|-----------|------|----------|
| `@json` | JSON encode/decode | ✅ 有効 |
| `@yaml` | YAML encode/decode | ✅ 有効 |
| `@regex` | 正規表現 | ✅ 有効 |
| `@assertions` | アサーション | ✅ 有効 |
| `@testing` | テストフレームワーク | ✅ 有効 |
| `@env` | 環境変数・パスアクセス | ❌ 無効（セキュリティ上） |

### DSL→Luaブリッジ基本形

```lua
-- DSL内 ```lua ブロックから呼ばれるシーン関数の定型
function SCENE.func_name(act)
    local save, var = act:init_scene(SCENE)  -- 必須: save/var を取得
    act:talk(act.さくら.actor, "セリフ")    -- アクター名でトーク
    act:yield()                               -- トークンをyield
end
```

（情報ソース: crates/pasta_lua/LUA_API.md §1）

---

## §3 Coding Conventions

### 3.1 命名規約

| 対象 | 命名スタイル | 例 |
|------|-------------|-----|
| ローカル変数 | snake_case | `local my_var = 1` |
| ローカル関数 | snake_case | `local function do_something()` |
| モジュールテーブル | UPPER_CASE | `local MOD = {}` |
| 定数 | UPPER_CASE | `local MAX_SIZE = 100` |
| プライベートメンバー | `_`プレフィックス | `self._internal = true` |
| クラス実装メタテーブル | `_IMPL`サフィックス | `local WORD_BUILDER_IMPL = {}` |

**禁止パターン**:

```lua
-- ❌ 禁止: PascalCase
local WordBuilder = {}

-- ✅ 推奨: UPPER_CASE + _IMPL
local WORD_BUILDER_IMPL = {}
```

**日本語識別子**: 内部変数・ローカル関数・GLOBAL登録では使用OK。公開API・モジュールテーブル名には使用NG。

```lua
-- ✅ 許可: 内部変数、GLOBAL登録
local function 時報(act) return "正午です" end
GLOBAL.時報 = function(act) return "正午です" end
```

### 3.2 標準モジュール構造

```lua
--- @module pasta.example
local STORE = require("pasta.store")  -- 1. require文は先頭
local MOD = {}                         -- 2. モジュールテーブル（UPPER_CASE）
-- 3. ローカル関数・定数 → 4. 公開関数
return MOD                             -- 5. 末尾で返却
```

モジュールテーブル名はファイル名に対応: `word.lua` → `local WORD = {}`

### 3.3 循環参照回避（STOREパターン）

`pasta.store` は他モジュールをrequireしない。共有データはSTOREに集約し、他モジュールがSTOREをrequireする。

### 3.4 クラス設計パターン

#### MODULE/MODULE_IMPL分離

モジュールテーブル（公開API）とクラス実装メタテーブル（インスタンスメソッド）を分離する。

```lua
local WORD = {}                            -- モジュールテーブル（公開API）
local WORD_BUILDER_IMPL = {}               -- クラス実装メタテーブル
WORD_BUILDER_IMPL.__index = WORD_BUILDER_IMPL

function WORD_BUILDER_IMPL.entry(self, ...)  -- ドット構文 + 明示的self
    table.insert(self._registry[self._key], { ... })
    return self
end

function WORD.create_global(key)            -- ファクトリ関数
    STORE.global_words[key] = STORE.global_words[key] or {}
    return setmetatable({ _registry = STORE.global_words, _key = key }, WORD_BUILDER_IMPL)
end
return WORD
```

| 用途 | 構文 | 例 |
|-----|------|-----|
| メソッド**定義** | ドット構文 + 明示的self | `function IMPL.method(self, arg)` |
| メソッド**呼び出し** | コロン構文（許可） | `obj:method(arg)` |

- **コンストラクタ**: `setmetatable(obj, MODULE_IMPL)` でIMPLを設定
- **シングルトン**: `require` キャッシング機構を活用（`pasta.store` が代表例）
- **継承**: `setmetatable(CHILD_IMPL, { __index = PARENT.IMPL })` + `MODULE.IMPL` で公開
- **禁止**: `MODULE.instance()` 手動管理、コロン構文でのメソッド定義（`function IMPL:method()`）

### 3.5 EmmyLua型注釈

- ファイル先頭に `@module` を配置、クラス定義直前に `@class` + `@field`
- 公開関数に `@param` / `@return` を付与。可変長引数は `@param ...`（`@vararg` は禁止）
- 戻り値nilの場合は `@return Type|nil` で明記

### 3.6 エラーハンドリング

- **ガードクローズ**: 関数先頭で前提条件検証 → 早期リターン
- **pcall**: 外部関数・リスクのある操作に使用
- **nilチェック**: 明示的な条件確認
- **禁止**: サイレントnil返却（`return data[key]` のように暗黙的にnilを返す）

（情報ソース: steering/lua-coding.md §1-§5）

---

## §4 Runtime API

**require使い分け**: `@pasta_search`, `@pasta_persistence`, `@pasta_sakura_script`, `@enc` → `require` 直接。`@pasta_config` → `pcall(require, ...)` 保護必須（テスト環境で存在しない可能性）。

### 4.1 @pasta_search

シーン・単語の検索機能。フォールバック戦略（ローカル → グローバル）を使用。

```lua
local SEARCH = require "@pasta_search"
```

- `search_scene(name, global?)` → `global_name, local_name | nil` — 前方一致。`global` 指定時ローカル→グローバルへフォールバック
- `search_word(name, global?)` → `string | nil` — 同じフォールバック戦略
- `set_scene_selector(...)` / `set_word_selector(...)` — テスト用決定論的選択。引数なしでランダムに戻す

```lua
SEARCH:set_scene_selector(0, 0, 0)  -- 常に最初の候補
SEARCH:set_word_selector(0, 1, 0)   -- 1番目、2番目、1番目の順
SEARCH:set_scene_selector()          -- ランダムに戻す
```

### 4.2 @pasta_persistence

セーブデータの永続化。

```lua
local persistence = require "@pasta_persistence"
```

**load() / save(data)**

```lua
persistence.load() -> table                        -- 初回は空テーブル
persistence.save(data) -> true, nil | nil, error_message
```

pasta.toml `[persistence]` 設定:

| オプション | 型 | デフォルト | 説明 |
|-----------|---|----------|------|
| `obfuscate` | bool | `false` | gzip圧縮有効化 |
| `file_path` | string | `"profile/pasta/save/save.json"` | 保存先パス |

```lua
local data = persistence.load()
data.count = (data.count or 0) + 1
local ok, err = persistence.save(data)
if not ok then print("保存失敗:", err) end
```

### 4.3 @pasta_config

`pasta.toml` のカスタムフィールドへの読み取り専用アクセス。`[loader]` セクションは除外。

```lua
local ok, config = pcall(require, "@pasta_config")  -- pcall必須
if ok then
    print(config.character.name)  -- "まゆら"
end
```

### 4.4 @pasta_sakura_script

テキストにウェイトタグを挿入してさくらスクリプトに変換。

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"
SAKURA_SCRIPT.talk_to_script(actor, talk) -> string
```

| パラメータ | 型 | 説明 |
|-----------|---|------|
| `actor` | table | `talk` サブテーブルにウェイト設定を持つ |
| `talk` | string | 変換対象のセリフテキスト |

**actor.talk テーブルの主要フィールド**:

| フィールド | デフォルト | 説明 |
|-----------|----------|------|
| `script_wait_default` | 50 | デフォルトウェイト（ms） |
| `script_wait_period` / `_comma` / `_newline` / `_exclamation` | 100/75/100/100 | 句読点・改行・感嘆符ウェイト |
| `chars_period` / `chars_comma` | `"。．.｡"` / `"、，,､"` | 句点・読点文字セット |
| `chars_exclamation` | `"！？!?‼⁉❗❓"` | 感嘆符文字セット |
| `chars_no_wait` / `chars_half_wait` | `"…ー〜～"` / `"っッ"` | ウェイトなし・半分ウェイト文字 |
| `chars_newline` | `"\n"` | 改行として認識する文字 |

```lua
local actor = { talk = { script_wait_default = 80 } }
local result = SAKURA_SCRIPT.talk_to_script(actor, "こんにちは。")
```

### 4.5 @enc

UTF-8 ⇔ ANSI（システムロケール）変換。主にWindows環境のファイルパス処理用。

```lua
local enc = require "@enc"
enc.to_ansi(utf8_str) -> ansi_string, nil | nil, error_message
enc.to_utf8(ansi_str) -> utf8_string, nil | nil, error_message
```

```lua
local ansi_path, err = enc.to_ansi("C:/ユーザー/設定.txt")
if ansi_path then
    local file = io.open(ansi_path, "r")
end
```

### 4.6 mlua-stdlib モジュール

```lua
local json = require "@json"           -- json.encode(t) / json.decode(s)
local yaml = require "@yaml"           -- yaml.encode(t) / yaml.decode(s)
local regex = require "@regex"          -- regex.new(pattern):find_all(s)
-- @assertions / @testing — §7で詳述。@env — デフォルト無効（セキュリティ上）
```

（情報ソース: crates/pasta_lua/LUA_API.md §2-§6, §8）

---

## §5 Internal Modules

### 5.1 STORE パターン

`pasta.store` — 全ランタイムデータの一元管理。他モジュールをrequireしない（循環参照回避）。

主要フィールド: `actors`, `actor_spots`, `scenes`, `counters`, `global_words`, `local_words`, `actor_words`, `app_ctx`, `co_scene`

- `STORE.reset()` — 全フィールドを初期化（テスト・再初期化用）

### 5.2 ACT オブジェクト

シーン関数の引数 `function scene(act)` として渡される実行コンテキスト。

#### `act:init_scene(SCENE)` — 必須定型

**すべてのシーン関数はこの呼び出しで始まる**。`save`（永続変数）と `var`（アクション内一時変数）を取得する。

```lua
function SCENE.my_scene(act)
    local save, var = act:init_scene(SCENE)  -- 必須
    save.count = (save.count or 0) + 1       -- 永続変数
    var.temp = "一時データ"                   -- アクション内のみ有効
end
```

#### 主要メソッド

| メソッド | シグネチャ | 説明 |
|---------|----------|------|
| `init_scene` | `(scene) → save, var` | 必須初期化 |
| `talk` | `(actor, text) → self` | セリフ出力 |
| `raw_script` | `(text) → self` | さくらスクリプト直接出力 |
| `surface` / `wait` / `newline` / `clear` | `(id)` / `(ms)` / `(n)` / `()` → self | 表示制御 |
| `set_spot` / `clear_spot` | `(name, number)` / `()` → nil | スポット操作 |
| `word` | `(name) → string\|nil` | 4段階単語検索 |
| `call` | `(global, key, attrs, ...) → any` | シーン/関数呼び出し |
| `yield` | `() → self` | build()してコルーチンyield |

フィールド: `actors`, `save`, `app_ctx`, `var`, `token`, `current_scene`, `req`（ShioriActのみ、§6参照）

（情報ソース: steering/lua-coding.md §6.3, §6.5）

### 5.3 SCENE モジュール

`pasta.scene` — シーンの登録・検索・実行。

- `SCENE.create_scene(base_name, local_name?, scene_func?)` — グローバルシーン作成。カウンタ自動採番
- `SCENE.search(name, global_scene_name?, attrs?)` — シーン検索（前方一致）
- `SCENE.co_exec(name, global_scene_name?, attrs?)` — コルーチン実行

**DSL→Luaブリッジ**: `function SCENE.func(act) ... end` パターン（§2 基本形参照）

### 5.4 WORD モジュール

`pasta.word` — ビルダーパターンによる単語定義。

```lua
local WORD = require("pasta.word")
```

| ファクトリ関数 | 引数 | スコープ |
|--------------|------|---------|
| `WORD.create_global(key)` | 単語キー | グローバル |
| `WORD.create_local(scene_name, key)` | シーン名, 単語キー | ローカル |
| `WORD.create_actor(actor_name, key)` | アクター名, 単語キー | アクター |

- `PASTA.create_word(key)` — `WORD.create_global` のエイリアス（`pasta/init.lua` 経由）
- `builder:entry(...)` — 値追加（メソッドチェーン可能）

**大量投入の使用例**:

```lua
local WORD = require("pasta.word")
local foods = { "ラーメン", "カレー", "寿司", "焼肉", "パスタ" }
local builder = WORD.create_global("好きな食べ物")
for _, food in ipairs(foods) do
    builder:entry(food)
end
```

### 5.5 GLOBAL モジュール

`pasta.global` — ユーザー定義グローバル関数テーブル。

```lua
local GLOBAL = require("pasta.global")
GLOBAL.時報 = function(act)
    return os.date("%H") .. "時です"
end
```

DSLから `＠時報()` で呼び出し可能。

### 5.6 SAVE モジュール

`pasta.save` — `@pasta_persistence` 経由の永続化データ。

- **ACT経由**（推奨）: `local save, var = act:init_scene(SCENE)` で `save` を取得
- **直接require**: `local save = require("pasta.save")`

### 5.7 finalize_scene

```lua
require("pasta").finalize_scene()
```

`scene_dic.lua` 末尾で自動呼び出し。Lua側のシーン・単語レジストリから `@pasta_search` モジュールを構築する。この呼び出し以降、`@pasta_search` が使用可能になる。

（情報ソース: steering/lua-coding.md §6.1-§6.6, crates/pasta_lua/LUA_API.md §7）

---

## §6 SHIORI Handlers

### 6.1 REG テーブル登録

`pasta.shiori.event.register` にイベントハンドラを登録する。

```lua
local REG = require("pasta.shiori.event.register")
local RES = require("pasta.shiori.res")

REG.OnBoot = function(req)
    local shell_name = req.reference[0]
    return RES.ok("\\h\\s[0]起動しました。\\e")
end
```

**req パラメータ**: SHIORIリクエスト情報
- `req.id` — イベント名（`"OnBoot"` 等）
- `req.reference[N]` — Referenceヘッダ（0始まりインデックス）
- `req.date` — 日時情報

### 6.2 RES レスポンス生成

```lua
local RES = require("pasta.shiori.res")
```

| 関数 | ステータス | 説明 |
|------|----------|------|
| `RES.ok(value)` | 200 OK | 成功レスポンス + Value |
| `RES.ok_with(headers)` | 200 OK | 成功レスポンス + 複数ヘッダ |
| `RES.no_content()` | 204 No Content | 空レスポンス |
| `RES.err(message)` | 500 Internal Server Error | エラーレスポンス |

```lua
return RES.ok("\\h\\s[0]こんにちは\\e")   -- 成功
return RES.no_content()                     -- 空応答
return RES.err("設定ファイルが見つかりません") -- エラー
```

### 6.3 主要SHIORIイベント

| イベント | reference[0] | reference[1] | 備考 |
|---------|-------------|-------------|------|
| OnFirstBoot | バニッシュ復帰フラグ ("0"/"1") | — | 初回起動 |
| OnBoot | シェル名 | — | 通常起動 |
| OnClose | 終了理由 ("user"等) | — | 終了 |
| OnGhostChanged | 切替先ゴースト名 | 切替元ゴースト名 | ゴースト切替 |
| OnMouseDoubleClick | スコープ ("0"/"1") | — | ref[4]=当たり判定ID |
| OnSecondChange | 現在秒 (0-59) | 累積秒 | 毎秒 |
| OnMinuteChange | 現在分 (0-59) | 現在時 (0-23) | 毎分 |

```lua
REG.OnFirstBoot = function(req)
    local is_vanish = req.reference[0] == "1"
    if is_vanish then
        return RES.ok("\\h\\s[0]帰ってきたわ。\\e")
    end
    return RES.ok("\\h\\s[0]はじめまして。\\e")
end
```

### 6.4 シーン関数フォールバック

REGテーブルにハンドラが登録されていない場合、`SCENE.search` でグローバルシーンを検索する。

- DSLで `＊OnBoot` と定義したシーンは、OnBootイベント発火時にREG未登録であれば自動的に呼び出される
- シーンも見つからない場合は `204 No Content` を返す

### 6.5 仮想ディスパッチャ

`pasta.shiori.event.virtual_dispatcher` — OnSecondChangeをトリガーとしてOnTalk/OnHourを自動発行。

- **OnHour**: 正時超過時にOnHourシーンを検索・実行
- **OnTalk**: 設定間隔でランダムトーク発行

pasta.toml `[ghost]` セクション設定:

| 設定 | デフォルト | 説明 |
|------|----------|------|
| `talk_interval_min` | 180 | 最小トーク間隔（秒） |
| `talk_interval_max` | 300 | 最大トーク間隔（秒） |
| `hour_margin` | 30 | 時報前スキップマージン（秒） |

（情報ソース: crates/pasta_lua/LUA_API.md §9）

---

## §7 Testing & Lint

### 7.1 lua_test フレームワーク

BDD風テストフレームワーク。`describe` / `test` / `expect` を使用。マッチャー: `toBe`, `not_:toBe` 等。

```lua
local describe = require("lua_test.test").describe
local test = require("lua_test.test").test
local expect = require("lua_test.test").expect

describe("モジュール名", function()
    test("期待される動作", function()
        expect(Module.func()):toBe("expected")
    end)
end)
```

### 7.2 テストファイル規約

命名: `*_test.lua` / `*_spec.lua`。init.lua で `specs` テーブルに登録しpcallで実行。

```lua
local specs = { "module_test", "feature_spec" }
for _, name in ipairs(specs) do
    local ok, err = pcall(function() require(name) end)
    if not ok then error(name .. " failed: " .. tostring(err)) end
end
```

### 7.3 決定論的テスト

`set_scene_selector` / `set_word_selector` でランダム選択を固定してからテスト。

```lua
local SEARCH = require "@pasta_search"
SEARCH:set_scene_selector(0, 0, 0)  -- 常に最初の候補
SEARCH:set_word_selector(0, 1, 0)   -- 指定順序で選択
-- テスト実行...
SEARCH:set_scene_selector()          -- テスト後にリセット
SEARCH:set_word_selector()
```

### 7.4 luacheck 設定

`.luacheckrc` 設定例:

```lua
globals = {
    "PASTA", "ACTOR", "SCENE", "WORD",
    "ACT", "CTX", "STORE", "GLOBAL",
}
allow_defined = true      -- UTF-8/日本語識別子許可
unused_args = false
max_line_length = 120
```

実行:

```bash
lua scriptlibs/luacheck/bin/luacheck.lua scripts/ --config .luacheckrc
```

（情報ソース: steering/lua-coding.md §7）
