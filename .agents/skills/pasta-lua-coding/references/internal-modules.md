# Internal Modules リファレンス

pasta_luaランタイムの `pasta.*` 名前空間で提供される内部Luaモジュールの完全リファレンス。  
STORE（データ一元管理）、ACT（シーン実行コンテキスト）、SCENE（シーン登録・検索）、WORD（単語ビルダー）、GLOBAL（ユーザー定義関数）、SAVE（永続化データ）を網羅する。

---

## STORE パターン

`pasta.store` — 全ランタイムデータの一元管理モジュール。**他モジュールをrequireしない**（循環参照回避の根本原則）。

```lua
local STORE = require("pasta.store")
```

### フィールド一覧

```lua
STORE.actors = {}          -- table<string, Actor>    アクターキャッシュ
STORE.actor_spots = {}     -- table<string, integer>  スポット位置マップ
STORE.scenes = {}          -- table<string, table>    シーンレジストリ
STORE.counters = {}        -- table<string, number>   シーン名カウンタ
STORE.global_words = {}    -- table<string, table>    グローバル単語レジストリ
STORE.local_words = {}     -- table<string, table>    ローカル単語レジストリ
STORE.actor_words = {}     -- table<string, table>    アクター単語レジストリ
STORE.app_ctx = {}         -- table                   汎用コンテキストデータ
STORE.co_scene = nil       -- thread|nil              継続コルーチン（OnTalk等）
```

### reset()

全フィールドを初期化する（テスト・再初期化用）。

```lua
STORE.reset()
```

中断中のコルーチン（`co_scene`）がある場合は `coroutine.close()` で適切にクローズしてからリセットする。

```lua
function STORE.reset()
    if STORE.co_scene then
        if coroutine.status(STORE.co_scene) == "suspended" then
            coroutine.close(STORE.co_scene)
        end
        STORE.co_scene = nil
    end
    STORE.actors = {}
    STORE.actor_spots = {}
    STORE.scenes = {}
    STORE.counters = {}
    STORE.global_words = {}
    STORE.local_words = {}
    STORE.actor_words = {}
    STORE.app_ctx = {}
end
```

### 循環参照回避の原則

STOREは他モジュールをrequireしない。共有データはSTOREに集約し、他モジュールがSTOREをrequireする一方向の依存関係を維持する。

```lua
-- ✅ 正しい: 他モジュール → STORE 方向のrequire
-- store.lua
local STORE = {}
STORE.actors = {}
return STORE

-- actor.lua
local STORE = require("pasta.store")
STORE.actors["さくら"] = { name = "さくら" }
```

**例外**: Rust組み込みモジュール `@pasta_config` のみ `pcall` 経由でrequireする（実行環境の違いに対応）。

```lua
local ok, CONFIG = pcall(require, "@pasta_config")
if ok and type(CONFIG.actor) == "table" then
    STORE.actors = CONFIG.actor
end
```

---

## ACT オブジェクト

シーン関数の引数 `function scene(act)` として渡される実行コンテキスト。トランスパイラ出力のシーン関数やカスタムLuaシーン関数が受け取る。

```lua
--- @class Act
--- @field actors table<string, Actor> 登録アクター
--- @field save table 永続変数（pasta.save）
--- @field app_ctx table アプリケーション実行中の汎用コンテキスト
--- @field var table アクションローカル変数
--- @field token table[] 蓄積トークン
--- @field current_scene SceneTable|nil 現在のシーン
--- @field req ShioriRequest|nil SHIORIリクエスト（ShioriActのみ）
```

### init_scene(scene)

**すべてのシーン関数はこの呼び出しで始まる**（必須定型）。`save`（永続変数）と `var`（アクション内一時変数）を取得する。

```lua
--- @param scene table シーンモジュール（SCENE）
--- @return table save 永続変数（@pasta_persistence管理）
--- @return table var アクション内一時変数
function ACT_IMPL.init_scene(self, scene) end
```

```lua
function SCENE.my_scene(act)
    local save, var = act:init_scene(SCENE)  -- 必須
    save.count = (save.count or 0) + 1       -- セッション間永続
    var.temp = "一時データ"                   -- アクション内のみ有効
end
```

### トーク系メソッド

#### talk(actor, text)

アクターのセリフを出力する。テキストは `@pasta_sakura_script` でウェイトタグ付きさくらスクリプトに自動変換される。

```lua
--- @param actor Actor|ActorProxy アクターオブジェクト
--- @param text string セリフテキスト
--- @return Act self メソッドチェーン用
function ACT_IMPL.talk(self, actor, text) end
```

```lua
act:talk(act.さくら.actor, "こんにちは")
```

#### raw_script(text)

さくらスクリプトを直接出力する（ウェイト変換を経由しない）。

```lua
--- @param text string さくらスクリプト文字列
--- @return Act self
function ACT_IMPL.raw_script(self, text) end
```

```lua
act:raw_script("\\h\\s[5]\\e")
```

### 表示制御メソッド

| メソッド | シグネチャ | 説明 |
|---------|----------|------|
| `surface` | `(id: integer) → self` | サーフェス変更 |
| `wait` | `(ms: integer) → self` | ウェイト挿入（ミリ秒） |
| `newline` | `(n?: integer) → self` | 改行挿入（デフォルト1） |
| `clear` | `() → self` | 表示クリア |

```lua
act:surface(5):wait(500):talk(actor, "驚いた！"):newline()
```

### スポット操作

#### set_spot(name, number)

スポット（アクター表示位置）を設定する。

```lua
--- @param name string スポット名
--- @param number integer スポット番号
--- @return nil
function ACT_IMPL.set_spot(self, name, number) end
```

#### clear_spot()

スポットをクリアする。

```lua
--- @return nil
function ACT_IMPL.clear_spot(self) end
```

### 検索・呼び出し

#### word(name)

4段階単語検索を実行する。

```lua
--- @param name string 単語キー
--- @return string|nil 単語の値、または見つからない場合nil
function ACT_IMPL.word(self, name) end
```

検索順序: アクター単語 → ローカル単語 → グローバル単語 → `@pasta_search`

#### find_scene(key, scope?, attrs?)

シーン名前解決（5段階フォールバック検索）。ハンドラー関数を検索して返す（実行しない）。

```lua
--- @param key string 検索キー
--- @param scope? string グローバルシーンスコープ
--- @param attrs? table 属性テーブル
--- @return function|nil 見つかったハンドラ関数
function ACT_IMPL.find_scene(self, key, scope, attrs) end
```

**検索順序**:
1. **L1**: `current_scene[key]` — シーンローカル検索
2. **L2**: `SCENE.search(key, scope)` — スコープ付き前方一致検索
3. **L3**: `GLOBAL[key]` — GLOBALテーブル
4. **L4**: `self[key]` — actメソッドフォールバック（type=="function" のみ）
5. **L5**: `SCENE.search(key, nil)` — スコープなし全体検索

> `act:call()` と `SCENE.co_exec()` の両方がこのメソッドを使用して名前解決を行う。

#### call(global, key, attrs, ...)

シーンまたは関数を呼び出す。内部で `find_scene()` を使用して名前解決し、見つかったハンドラを即時実行する。

```lua
--- @param global string グローバルシーン名
--- @param key string|nil ローカルキー（nilの場合は警告ログ出力＋即時リターン）
--- @param attrs table|nil 属性テーブル
--- @param ... any 追加引数
--- @return any 呼び出し結果
function ACT_IMPL.call(self, global, key, attrs, ...) end
```

> **nilガード**: `key == nil` の場合（未定義変数参照等）、`find_scene` は呼ばれず `log.warn` を出力して `nil` を返す。動的コール `＞expr` で式評価結果が nil になった場合に適用される。

### yield()

蓄積トークンを `build()` してコルーチン `yield` する。トーク区切りや一連のスクリプト出力の区切りに使用。

```lua
--- @return Act self
function ACT_IMPL.yield(self) end
```

```lua
function SCENE.my_scene(act)
    local save, var = act:init_scene(SCENE)
    act:talk(act.さくら.actor, "最初のセリフ")
    act:yield()  -- ここでトークン送出
    act:talk(act.さくら.actor, "次のセリフ")
    act:yield()  -- 2回目の送出
end
```

### PROXYパターン

アクターへのプロキシオブジェクト。ACTへの逆参照を持ち、3段階単語検索を実装する。

```lua
--- @class ActorProxy
--- @field actor Actor
--- @field act Act

-- トランスパイラー出力で使用
act.さくら:talk("こんにちは")
local word = act.さくら:word("名前")  -- 3段階: actor→actor辞書→act:word()
```

---

## SCENE モジュール

`pasta.scene` — シーンの登録・検索・コルーチン実行。

```lua
local SCENE = require("pasta.scene")
```

### create_scene(base_name, local_name?, scene_func?)

グローバルシーンを作成する。同名シーンは自動カウンタで採番される。

```lua
--- @param base_name string 基本シーン名
--- @param local_name? string ローカルシーン名
--- @param scene_func? function シーン関数
--- @return table シーンテーブル
function SCENE.create_scene(base_name, local_name, scene_func) end
```

```lua
local scene = SCENE.create_scene("メイン")
-- → グローバル名 "メイン_1" が作成される（カウンタ自動採番）
```

### search(name, global_scene_name?, attrs?)

シーンを検索する（前方一致）。内部で `@pasta_search` を使用。

```lua
--- @param name string シーン名
--- @param global_scene_name? string 親シーン名
--- @param attrs? table 属性テーブル
--- @return function|nil シーン関数
function SCENE.search(name, global_scene_name, attrs) end
```

### co_exec(act, name, global_scene_name?, attrs?)

`act:find_scene()` を使用してシーンを検索し、コルーチンとして実行する。

```lua
--- @param act Act アクションオブジェクト（find_scene による名前解決に使用）
--- @param name string シーン名
--- @param global_scene_name? string 親シーン名
--- @param attrs? table 属性テーブル
--- @return thread|nil シーンコルーチン、またはnil
function SCENE.co_exec(act, name, global_scene_name, attrs) end
```

> **Breaking Change**: 第1引数に `act` が追加された。イベントディスパッチ層からの呼び出しは `SCENE.co_exec(act, event_name, nil, nil)` とすること。

### DSL→Luaブリッジ

Pasta DSLの ` ```lua ``` ` ブロックで定義されたシーン関数は `function SCENE.func_name(act)` パターンに変換される。

```lua
function SCENE.func_name(act)
    local save, var = act:init_scene(SCENE)  -- 必須: save/var を取得
    act:talk(act.さくら.actor, "セリフ")    -- アクター名でトーク
    act:yield()                               -- トークンをyield
end
```

---

## WORD モジュール

`pasta.word` — ビルダーパターンによる単語定義。

```lua
local WORD = require("pasta.word")
```

### ファクトリ関数

| 関数 | 引数 | スコープ |
|------|------|---------|
| `WORD.create_global(key)` | 単語キー | グローバル |
| `WORD.create_local(scene_name, key)` | シーン名, 単語キー | ローカル |
| `WORD.create_actor(actor_name, key)` | アクター名, 単語キー | アクター |

**エイリアス**: `PASTA.create_word(key)` — `WORD.create_global` のエイリアス（`pasta/init.lua` 経由）

### ビルダーパターン

#### entry(...)

値を追加する。メソッドチェーン可能。

```lua
--- @param self WordBuilder
--- @param ... string 可変長引数（値）
--- @return WordBuilder self
function WORD_BUILDER_IMPL.entry(self, ...) end
```

```lua
WORD.create_global("好きな食べ物")
    :entry("ラーメン", "カレー")
    :entry("寿司")
    :entry("焼肉", "パスタ")
```

### 大量投入の使用例

```lua
local WORD = require("pasta.word")

-- ループによる一括投入
local foods = { "ラーメン", "カレー", "寿司", "焼肉", "パスタ" }
local builder = WORD.create_global("好きな食べ物")
for _, food in ipairs(foods) do
    builder:entry(food)
end

-- ローカル単語の投入
WORD.create_local("メイン_1", "返事")
    :entry("はい", "ええ")
    :entry("そうね")

-- アクター単語の投入
WORD.create_actor("さくら", "一人称")
    :entry("わたし")
    :entry("あたし")
```

---

## GLOBAL モジュール

`pasta.global` — ユーザー定義グローバル関数テーブル。

```lua
local GLOBAL = require("pasta.global")
```

関数を登録すると、DSLから `＠関数名()` で呼び出し可能になる。

```lua
GLOBAL.時報 = function(act)
    return os.date("%H") .. "時です"
end

-- DSLから呼び出し: ＠時報()
```

---

## SAVE モジュール

`pasta.save` — `@pasta_persistence` 経由の永続化データ。

### ACT経由のアクセス（推奨）

```lua
function SCENE.my_scene(act)
    local save, var = act:init_scene(SCENE)
    -- save: セッション間永続（@pasta_persistence管理）
    -- var:  アクション内一時変数
    save.count = (save.count or 0) + 1
end
```

### 直接require

```lua
local save = require("pasta.save")
```

内部実装:

```lua
-- pasta/save.lua
local persistence = require("@pasta_persistence")
local save = persistence.load()
return save
```

---

## finalize_scene

```lua
require("pasta").finalize_scene()
```

Lua側のシーン・単語レジストリから `@pasta_search` モジュールを構築する内部関数。

### 目的

トランスパイル済みLuaコードが読み込まれた後に呼び出され、以下を実行:
1. `pasta.scene` レジストリから全シーン情報を収集
2. `pasta.word` レジストリから全単語定義を収集
3. `SceneRegistry` と `WordDefRegistry` を構築
4. `SearchContext` を作成し `@pasta_search` モジュールとして登録

### 呼び出しタイミング

通常 `scene_dic.lua` の末尾で自動呼び出し:

```lua
-- scene_dic.lua (自動生成)
require("scene.main")
require("scene.sub")
require("pasta").finalize_scene()  -- 最後に呼び出し
```

### 処理フロー

```
pasta.scene レジストリ → collect_scenes() → SceneRegistry ─┐
                                                             ├→ SearchContext (@pasta_search)
pasta.word レジストリ  → collect_words()  → WordDefRegistry ┘
```

### シーン収集データ構造

```lua
{
  ["グローバルシーン名"] = {
    __global_name__ = "グローバルシーン名",
    __start__ = function() ... end,
    __ローカルシーン名__ = function() ... end,
  },
}
```

### 単語収集データ構造

```lua
{
  global = {
    ["キー"] = {{"値1", "値2"}, {"値3"}},
  },
  ["local"] = {
    ["シーン名"] = {
      ["キー"] = {{"ローカル値"}},
    },
  },
}
```

### 上級者向け情報

- この関数は `scene_dic.lua` 読み込み前にRust側で登録される
- Lua側のスタブ実装をRustバインディングで上書きする形式
- 呼び出し後に `@pasta_search` が使用可能になる

---

## 関連リファレンス

- [runtime-api.md](runtime-api.md) — `@pasta_search` の完全APIシグネチャ、`@pasta_persistence` の設定詳細
- [shiori-handlers.md](shiori-handlers.md) — ACTオブジェクトの `req` フィールド（ShioriAct）の詳細、イベント一覧
