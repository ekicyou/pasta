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
│   │       └── main.lua             # SHIORI エントリーポイント
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

| パス                            | 用途                      | 備考                                     |
| ------------------------------- | ------------------------- | ---------------------------------------- |
| `pasta.toml`                    | 設定ファイル              | 必須。存在しない場合はエラー             |
| `dic/*/*.pasta`                 | Pasta DSL ソース          | デフォルトの検出パターン                 |
| `scripts/`                      | 自作 Lua スクリプト       | package.path に含まれる                  |
| `scripts/pasta/`                | Pasta ランタイム          | トランスパイル済みコードから呼び出される |
| `scripts/pasta/shiori/main.lua` | SHIORI エントリーポイント | 存在すれば自動ロード                     |
| `scriptlibs/`                   | 外部ライブラリ            | package.path の最後に追加                |
| `profile/pasta/save/lua/`       | 永続化モジュール          | 最優先で検索される                       |
| `profile/pasta/cache/lua/`      | Lua キャッシュ            | debug_mode 時に出力                      |
| `profile/pasta/logs/`           | ログ出力先                | ローテーション対応                       |

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

# カスタムフィールド（Lua から @pasta_config で参照可能）
[user]
ghost_name = "MyGhost"
version = "1.0.0"
```

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

| モジュール名    | 用途                            | デフォルト                 |
| --------------- | ------------------------------- | -------------------------- |
| `@pasta_search` | シーン・単語検索 API            | 常に有効                   |
| `@pasta_config` | pasta.toml のカスタムフィールド | 常に有効                   |
| `@enc`          | エンコーディング変換            | 常に有効                   |
| `@assertions`   | アサーション関数                | 有効                       |
| `@testing`      | テストフレームワーク            | 有効                       |
| `@regex`        | 正規表現サポート                | 有効                       |
| `@json`         | JSON エンコード/デコード        | 有効                       |
| `@yaml`         | YAML エンコード/デコード        | 有効                       |
| `@env`          | 環境変数アクセス                | **無効**（セキュリティ上） |

### 使用例

```lua
-- Pasta 検索 API
local SEARCH = require "@pasta_search"
local global_name, local_name = SEARCH:search_scene("シーン名", "親シーン")

-- 設定ファイルからカスタム値を取得
local CONFIG = require "@pasta_config"
print(CONFIG.ghost_name)  -- pasta.toml の [user].ghost_name

-- JSON 処理
local JSON = require "@json"
local data = JSON.decode('{"key": "value"}')
```

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

// 全機能有効（@env モジュール含む）
let config = RuntimeConfig::full();
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

SHIORI/3.0 プロトコルとの統合には `scripts/pasta/shiori/main.lua` を配置します：

```lua
-- scripts/pasta/shiori/main.lua
SHIORI = SHIORI or {}

function SHIORI.load(hinst, load_dir)
    -- 初期化処理
    return true
end

function SHIORI.request(request_text)
    -- リクエスト処理
    return "SHIORI/3.0 200 OK\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Value: Hello!\r\n" ..
        "\r\n"
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

| 関数                   | ステータス                   | 説明                               |
| ---------------------- | ---------------------------- | ---------------------------------- |
| `RES.ok(value, dic)`   | 200 OK                       | Value ヘッダー付き成功レスポンス   |
| `RES.no_content(dic)`  | 204 No Content               | 値なし成功レスポンス               |
| `RES.not_enough(dic)`  | 311 Not Enough               | TEACH イベント（情報不足）         |
| `RES.advice(dic)`      | 312 Advice                   | TEACH イベント（アドバイス）       |
| `RES.bad_request(dic)` | 400 Bad Request              | クライアントエラー                 |
| `RES.err(reason, dic)` | 500 Internal Server Error    | サーバーエラー（X-Error-Reason 付） |
| `RES.warn(reason, dic)`| 204 No Content               | 警告付きレスポンス（X-Warn-Reason） |
| `RES.build(code, dic)` | 任意                         | 汎用ビルダー（上記の基盤）         |

**環境設定**:

```lua
RES.env.charset = "UTF-8"       -- デフォルト
RES.env.sender = "Pasta"        -- デフォルト
RES.env.security_level = "local" -- デフォルト
```

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
