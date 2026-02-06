# pasta_lua

Pasta DSL を Lua にトランスパイルし、Lua VM 上で実行するためのクレートです。

## 概要

`pasta_lua` は Pasta DSL の Lua バックエンド実装を提供します。主な機能：

- Pasta DSL → Lua ソースコードへのトランスパイル
- Lua VM ホスティング（mlua ベース）
- モジュール解決とパッケージパス管理
- SHIORI/3.0 プロトコル統合サポート

## アーキテクチャ

```
PastaLoader (起動シーケンス)
    ↓
LuaTranspiler (AST → Lua変換)
    ↓
PastaLuaRuntime (Lua VM ホスト)
    ↓
SHIORI Integration (プロトコル処理)
```

## ディレクトリ構成

`pasta_lua` が想定する標準的なプロジェクト構造：

```
{base_dir}/                          # ベースディレクトリ（SHIORIのload_dir）
├── pasta.toml                       # 設定ファイル（必須）
│
├── dic/                             # Pasta DSL ソースファイル
│   ├── baseware/                    # カテゴリ別サブディレクトリ
│   │   ├── system.pasta
│   │   └── events.pasta
│   └── conversation/
│       ├── greeting.pasta
│       └── talk.pasta
│
├── scripts/                         # 自作 Lua スクリプト
│   ├── pasta/                       # Pasta ランタイムライブラリ
│   │   ├── init.lua                 # PASTA モジュール
│   │   ├── ctx.lua                  # コンテキスト管理
│   │   ├── actor.lua                # アクター管理
│   │   ├── scene.lua                # シーン管理
│   │   └── shiori/
│   │       └── entry.lua            # SHIORI エントリーポイント
│   └── (your_modules)/              # ユーザー定義モジュール
│
├── scriptlibs/                      # 外部 Lua ライブラリ
│   └── (external_libs)/             # サードパーティライブラリ
│
└── profile/                         # ランタイム生成ディレクトリ
    └── pasta/
        ├── save/                    # 永続化データ
        │   └── lua/                 # Lua 保存用モジュール
        ├── cache/                   # キャッシュ
        │   └── lua/                 # トランスパイル済み Lua キャッシュ
        └── logs/
            └── pasta.log            # ログファイル
```

### ディレクトリ詳細

| パス                             | 用途                      | 備考                                     |
| -------------------------------- | ------------------------- | ---------------------------------------- |
| `pasta.toml`                     | 設定ファイル              | 必須。存在しない場合はエラー             |
| `dic/*/*.pasta`                  | Pasta DSL ソース          | デフォルトの検出パターン                 |
| `scripts/`                       | 自作 Lua スクリプト       | package.path に含まれる                  |
| `scripts/pasta/`                 | Pasta ランタイム          | トランスパイル済みコードから呼び出される |
| `scripts/pasta/shiori/entry.lua` | SHIORI エントリーポイント | 存在すれば自動ロード                     |
| `scriptlibs/`                    | 外部ライブラリ            | package.path の最後に追加                |
| `profile/pasta/save/lua/`        | 永続化モジュール          | 最優先で検索される                       |
| `profile/pasta/cache/lua/`       | Lua キャッシュ            | debug_mode 時に出力                      |
| `profile/pasta/logs/`            | ログ出力先                | ローテーション対応                       |

## 設定ファイル（pasta.toml）

```toml
[loader]
# Pasta ファイルの検出パターン（glob形式）
# デフォルト: ["dic/*/*.pasta"]
pasta_patterns = ["dic/*/*.pasta"]

# Lua モジュール検索パス（優先度順）
# デフォルト: 下記参照
lua_search_paths = [
    "profile/pasta/save/lua",  # 1. 永続化モジュール（最優先）
    "scripts",                 # 2. 自作スクリプト
    "profile/pasta/cache/lua", # 3. トランスパイル済みキャッシュ
    "scriptlibs",              # 4. 外部ライブラリ
]

# トランスパイル出力ディレクトリ
# デフォルト: "profile/pasta/cache/lua"
transpiled_output_dir = "profile/pasta/cache/lua"

# デバッグモード（トランスパイル結果をファイル保存）
# デフォルト: true
debug_mode = true

[logging]
# ログファイルパス（base_dir からの相対パス）
# デフォルト: "profile/pasta/logs/pasta.log"
file_path = "profile/pasta/logs/pasta.log"

# ログローテーション日数
# デフォルト: 7
rotation_days = 7

[lua]
# 有効にするライブラリの配列（Cargo風記法）
# デフォルト: ["std_all", "assertions", "testing", "regex", "json", "yaml"]
libs = [
    "std_all",      # Lua安全標準ライブラリ (coroutine, table, io, os, string, utf8, math, package)
    "assertions",   # @assertions モジュール
    "testing",      # @testing モジュール
    "regex",        # @regex モジュール
    "json",         # @json モジュール
    "yaml",         # @yaml モジュール
    # "-std_io",    # 減算記法: std_io を除外
    # "env",        # @env モジュール（セキュリティ上デフォルト無効）
    # "std_debug",  # debug ライブラリ（セキュリティ上デフォルト無効）
]

# カスタムフィールド（Lua から @pasta_config で参照可能）
[user]
ghost_name = "MyGhost"
version = "1.0.0"

# アクター定義（STORE.actors に自動初期化）
[actor."さくら"]
spot = 0
default_surface = 0

[actor."うにゅう"]
spot = 1
default_surface = 10
```

### [actor.*] セクション

`[actor.*]` セクションで定義したアクター設定は、Lua ランタイム起動時に `STORE.actors` へ自動的に参照共有されます。

```toml
[actor."さくら"]
spot = 0
default_surface = 0

[actor."うにゅう"]  
spot = 1
default_surface = 10
```

**初期化フロー:**

1. `@pasta_config` モジュール（Rust側）が TOML をパースし `CONFIG.actor` テーブルとして公開
2. `pasta.store` モジュール（Lua）が `STORE.actors = CONFIG.actor` で参照共有
3. `pasta.actor` モジュール（Lua）が各アクターに `ACTOR_IMPL` メタテーブルを設定

**Lua からのアクセス:**

```lua
local STORE = require "pasta.store"
local ACTOR = require "pasta.actor"

-- 直接アクセス
print(STORE.actors["さくら"].spot)  -- 0

-- ACTOR API経由（推奨）
local sakura = ACTOR.get_or_create("さくら")
print(sakura.spot)  -- 0（CONFIG由来プロパティを保持）
```

**特徴:**

- 参照共有のため、`STORE.actors` への変更は `CONFIG.actor` にも反映される
- 動的に追加したアクターと CONFIG 由来アクターは共存可能
- `ACTOR.get_or_create()` は既存アクターを返し、上書きしない

### [lua] セクション詳細

`libs` 配列はCargo風の記法をサポートし、Lua標準ライブラリとmlua-stdlibモジュールを統合制御します。

#### 有効なライブラリ名

**Lua標準ライブラリ（`std_*` プレフィックス）:**

| ライブラリ名     | 説明                                         |
| ---------------- | -------------------------------------------- |
| `std_all`        | 安全な標準ライブラリ全部（debug除く）        |
| `std_all_unsafe` | **全ライブラリ（debug含む、要注意）**        |
| `std_coroutine`  | coroutine ライブラリ                         |
| `std_table`      | table ライブラリ                             |
| `std_io`         | io ライブラリ                                |
| `std_os`         | os ライブラリ                                |
| `std_string`     | string ライブラリ                            |
| `std_utf8`       | utf8 ライブラリ                              |
| `std_math`       | math ライブラリ                              |
| `std_package`    | package ライブラリ（require等）              |
| `std_debug`      | **debug ライブラリ（セキュリティ警告発生）** |

**mlua-stdlib モジュール:**

| モジュール名 | 説明                        |
| ------------ | --------------------------- |
| `assertions` | @assertions モジュール      |
| `testing`    | @testing モジュール         |
| `env`        | **@env モジュール（警告）** |
| `regex`      | @regex モジュール           |
| `json`       | @json モジュール            |
| `yaml`       | @yaml モジュール            |

#### 減算記法

`-` プレフィックスでライブラリを除外できます：

```toml
[lua]
libs = [
    "std_all",     # 安全な標準ライブラリをすべて有効化
    "-std_io",     # io ライブラリを除外
    "-std_os",     # os ライブラリを除外
    "json",        # json モジュールを有効化
]
```

#### セキュリティ警告

以下のライブラリを有効にすると、ログに警告が出力されます：

- `std_debug` または `std_all_unsafe`: デバッグライブラリはサンドボックス回避に使用される可能性があります
- `env`: ファイルシステムと環境変数へのアクセスを提供します

## Lua モジュール検索パス

`package.path` は以下の形式で設定されます（優先度順）：

```lua
-- 生成されるpackage.path（例）
"/path/to/base/profile/pasta/save/lua/?.lua;"..
"/path/to/base/profile/pasta/save/lua/?/init.lua;"..
"/path/to/base/scripts/?.lua;"..
"/path/to/base/scripts/?/init.lua;"..
"/path/to/base/profile/pasta/cache/lua/?.lua;"..
"/path/to/base/profile/pasta/cache/lua/?/init.lua;"..
"/path/to/base/scriptlibs/?.lua;"..
"/path/to/base/scriptlibs/?/init.lua"
```

### 検索優先順位

1. **`profile/pasta/save/lua/`** - 永続化されたユーザーモジュール（最優先）
2. **`scripts/`** - 自作スクリプト・Pasta ランタイムライブラリ
3. **`profile/pasta/cache/lua/`** - トランスパイル済み Pasta コード
4. **`scriptlibs/`** - 外部 Lua ライブラリ

## 組み込みモジュール

ランタイムに自動登録されるモジュール：

| モジュール名           | 用途                            | デフォルト                 |
| ---------------------- | ------------------------------- | -------------------------- |
| `@pasta_search`        | シーン・単語検索 API            | 常に有効                   |
| `@pasta_config`        | pasta.toml のカスタムフィールド | 常に有効                   |
| `@pasta_sakura_script` | さくらスクリプト変換 API        | 常に有効                   |
| `@enc`                 | エンコーディング変換            | 常に有効                   |
| `@assertions`          | アサーション関数                | 有効                       |
| `@testing`             | テストフレームワーク            | 有効                       |
| `@regex`               | 正規表現サポート                | 有効                       |
| `@json`                | JSON エンコード/デコード        | 有効                       |
| `@yaml`                | YAML エンコード/デコード        | 有効                       |
| `@env`                 | 環境変数アクセス                | **無効**（セキュリティ上） |

### 使用例

```lua
-- Pasta 検索 API
local SEARCH = require "@pasta_search"
local global_name, local_name = SEARCH:search_scene("シーン名", "親シーン")

-- 設定ファイルからカスタム値を取得
local CONFIG = require "@pasta_config"
print(CONFIG.ghost_name)  -- pasta.toml の [user].ghost_name

-- さくらスクリプト変換 API
local SAKURA_SCRIPT = require "@pasta_sakura_script"
local actor = { talk = { script_wait_default = 50 } }
local result = SAKURA_SCRIPT.talk_to_script(actor, "こんにちは。")
-- 結果: "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[100]。"

-- JSON 処理
local JSON = require "@json"
local data = JSON.decode('{"key": "value"}')
```

### API リファレンス

各モジュールの詳細な API 仕様（関数シグネチャ、パラメータ、戻り値、使用例）については [LUA_API.md](./LUA_API.md) を参照してください。

## 使用方法

### 基本的な使用法

```rust
use pasta_lua::PastaLoader;

// ベースディレクトリから起動
let runtime = PastaLoader::load("path/to/ghost/master/")?;

// Lua コードを実行
let result = runtime.exec("return 1 + 1")?;
```

### カスタム設定での起動

```rust
use pasta_lua::{PastaLoader, RuntimeConfig};

// デフォルト設定（std_all + assertions, testing, regex, json, yaml）
let config = RuntimeConfig::new();
let runtime = PastaLoader::load_with_config("path/to/base", config)?;

// 全機能有効（std_all_unsafe + @env モジュール含む）
let config = RuntimeConfig::full();
let runtime = PastaLoader::load_with_config("path/to/base", config)?;

// 最小構成（std_all のみ、mlua-stdlib モジュールなし）
let config = RuntimeConfig::minimal();
let runtime = PastaLoader::load_with_config("path/to/base", config)?;

// カスタム構成
let config = RuntimeConfig::from_libs(vec![
    "std_all".into(),
    "regex".into(),
    "-std_io".into(),  // io を除外
]);
let runtime = PastaLoader::load_with_config("path/to/base", config)?;
```

### トランスパイラー単独使用

```rust
use pasta_lua::{LuaTranspiler, PastaLuaRuntime};

let transpiler = LuaTranspiler::default();
let mut output = Vec::new();

// Pasta AST をトランスパイル
let context = transpiler.transpile(&pasta_file, &mut output)?;
let lua_code = String::from_utf8(output)?;

// ランタイムを作成して実行
let runtime = PastaLuaRuntime::new(context)?;
runtime.exec(&lua_code)?;
```

## SHIORI 統合

SHIORI/3.0 プロトコルとの統合には `scripts/pasta/shiori/entry.lua` を配置します：

```lua
-- scripts/pasta/shiori/entry.lua
local EVENT = require "pasta.shiori.event"

SHIORI = SHIORI or {}

function SHIORI.load(hinst, load_dir)
    -- 初期化処理
    return true
end

function SHIORI.request(req)
    -- req テーブル経由でイベント振り分け
    return EVENT.fire(req)
end

function SHIORI.unload()
    -- クリーンアップ処理
end

return SHIORI
```

### pasta.shiori.res モジュール

SHIORI/3.0 レスポンス文字列を構築するためのユーティリティモジュールです。

```lua
local RES = require "pasta.shiori.res"

-- 200 OK レスポンス
return RES.ok("Hello!")

-- 204 No Content（処理完了、返却値なし）
return RES.no_content()

-- カスタムヘッダー付きレスポンス
return RES.no_content({ ["X-Custom"] = "value" })

-- エラーレスポンス
return RES.err("Something went wrong")

-- ワーニング付きレスポンス（204 + X-Warn-Reason）
return RES.warn("Deprecated feature used")

-- TEACH イベント用レスポンス
return RES.not_enough()  -- 311 Not Enough
return RES.advice()      -- 312 Advice
```

**利用可能な関数**:

| 関数                    | ステータス                | 説明                                |
| ----------------------- | ------------------------- | ----------------------------------- |
| `RES.ok(value, dic)`    | 200 OK                    | Value ヘッダー付き成功レスポンス    |
| `RES.no_content(dic)`   | 204 No Content            | 値なし成功レスポンス                |
| `RES.not_enough(dic)`   | 311 Not Enough            | TEACH イベント（情報不足）          |
| `RES.advice(dic)`       | 312 Advice                | TEACH イベント（アドバイス）        |
| `RES.bad_request(dic)`  | 400 Bad Request           | クライアントエラー                  |
| `RES.err(reason, dic)`  | 500 Internal Server Error | サーバーエラー（X-Error-Reason 付） |
| `RES.warn(reason, dic)` | 204 No Content            | 警告付きレスポンス（X-Warn-Reason） |
| `RES.build(code, dic)`  | 任意                      | 汎用ビルダー（上記の基盤）          |

**環境設定**:

```lua
RES.env.charset = "UTF-8"       -- デフォルト
RES.env.sender = "Pasta"        -- デフォルト
RES.env.security_level = "local" -- デフォルト
```

### pasta.shiori.sakura_builder モジュール

トークン配列からさくらスクリプト文字列を生成するための純粋関数モジュールです。`pasta.shiori.act` の `build()` メソッド内部で使用されます。

#### グループ化トークン形式（推奨）

`ACT_IMPL.build()` は **グループ化されたトークン配列** を返します（actor-talk-grouping 機能）:

```lua
local BUILDER = require "pasta.shiori.sakura_builder"

-- グループ化形式: type="actor" がトークングループを保持
local grouped_tokens = {
    { type = "spot", actor = sakura, spot = 0 },
    { type = "actor", actor = sakura, tokens = {
        { type = "talk", actor = sakura, text = "こんにちは！" },
        { type = "surface", id = 5 },
        { type = "wait", ms = 1000 },
    }},
}
local config = { spot_newlines = 0.5 }
local script = BUILDER.build(grouped_tokens, config)
-- 結果: "\p[0]こんにちは！\s[5]\w[1000]\e"
```

#### レガシーフラット形式（後方互換）

```lua
-- フラット形式: 個別トークンの配列（後方互換のため引き続きサポート）
local tokens = {
    { type = "actor", actor = { spot = 0 } },
    { type = "talk", text = "こんにちは！" },
    { type = "surface", id = 5 },
    { type = "wait", ms = 1000 },
}
local script = BUILDER.build(tokens, config)
```

**トークンタイプ**:

| タイプ          | フィールド        | 出力例                               |
| --------------- | ----------------- | ------------------------------------ |
| `talk`          | `text`            | エスケープ済みテキスト               |
| `actor`         | `actor`, `tokens` | グループ内トークンを順次処理         |
| `actor`(legacy) | `actor.spot`      | `\p[n]` (スポットタグ)               |
| `spot`          | `actor`, `spot`   | 内部状態更新（出力なし）             |
| `clear_spot`    | -                 | 内部状態リセット（出力なし）         |
| `spot_switch`   | -                 | `\n[percent]` (段落区切り、レガシー) |
| `surface`       | `id`              | `\s[id]`                             |
| `wait`          | `ms`              | `\w[ms]`                             |
| `newline`       | `n`               | `\n` × n回                           |
| `clear`         | -                 | `\c`                                 |
| `raw_script`    | `text`            | そのまま出力                         |
| `yield`         | -                 | 無視（出力対象外）                   |

## ファイル検出パターン

デフォルトの `dic/*/*.pasta` パターンでは：

- ✅ `dic/baseware/system.pasta` - 検出される
- ✅ `dic/talk/greeting.pasta` - 検出される
- ❌ `dic/root.pasta` - 検出されない（直下は対象外）
- ❌ `profile/pasta/cache/lua/cached.pasta` - 除外される

カスタムパターン例：

```toml
[loader]
# 再帰的に全ての .pasta ファイルを検出
pasta_patterns = ["dic/**/*.pasta"]

# 複数パターンの指定
pasta_patterns = ["dic/*/*.pasta", "extra/*.pasta"]
```

## モジュール名の生成

Pasta ファイルのパスからモジュール名が自動生成されます：

| ソースパス                  | モジュール名          |
| --------------------------- | --------------------- |
| `dic/baseware/system.pasta` | `dic_baseware_system` |
| `dic/talk/greeting.pasta`   | `dic_talk_greeting`   |

## 関連クレート

- [`pasta_core`](../pasta_core/) - パーサー、AST、レジストリ
- [`pasta_shiori`](../pasta_shiori/) - SHIORI DLL ラッパー
- [プロジェクト概要](../../README.md) - pasta プロジェクト全体

## ライセンス

プロジェクトルートの LICENSE ファイルを参照してください。
