# pasta.toml 設定仕様書

## 概要

本ドキュメントは `pasta.toml` の既存実装から抽出した設定項目の完全なリファレンスである。
alpha04-sample-ghost の `pasta.toml` 設計時に参照する。

**調査日**: 2026-01-31  
**情報源**: `crates/pasta_lua/src/loader/config.rs`, `scripts/pasta/shiori/event/virtual_dispatcher.lua`, `LUA_API.md`

---

## ファイル構造

```toml
# pasta.toml 構造概要

# === システム予約セクション ===
[loader]         # ローダー設定（Rust側で処理）
[logging]        # ログ設定
[persistence]    # 永続化設定
[lua]            # Luaライブラリ設定

# === ゴースト設定セクション ===
[ghost]          # ゴースト動作設定（Lua側から @pasta_config で参照）

# === カスタムセクション ===
# 任意のセクション/キーを追加可能
# Lua側から @pasta_config で参照可能
```

---

## 1. [loader] セクション

**処理元**: `PastaConfig` 構造体（Rust）  
**参照方法**: Rust側でのみ使用。`@pasta_config` には含まれない（除外される）

### フィールド

| キー | 型 | デフォルト値 | 説明 |
|------|-----|--------------|------|
| `pasta_patterns` | `string[]` | `["dic/*/*.pasta"]` | Pastaファイル検索パターン |
| `lua_search_paths` | `string[]` | （下記参照） | Luaモジュール検索パス（優先順） |
| `transpiled_output_dir` | `string` | `"profile/pasta/cache/lua"` | トランスパイル出力ディレクトリ |
| `debug_mode` | `bool` | `true` | デバッグモード（トランスパイル結果保存） |

### lua_search_paths デフォルト値

```toml
lua_search_paths = [
    "profile/pasta/save/lua",   # ユーザー保存Lua
    "scripts",                   # ゴースト同梱スクリプト
    "profile/pasta/cache/lua",   # トランスパイルキャッシュ
    "scriptlibs",                # 外部ライブラリ
]
```

### 記述例

```toml
[loader]
pasta_patterns = ["dic/*/*.pasta", "custom/*.pasta"]
lua_search_paths = ["scripts", "lib"]
transpiled_output_dir = "cache"
debug_mode = true
```

---

## 2. [logging] セクション

**処理元**: `LoggingConfig` 構造体（Rust）  
**参照方法**: Rust側で `config.logging()` で取得

### フィールド

| キー | 型 | デフォルト値 | 説明 |
|------|-----|--------------|------|
| `file_path` | `string` | `"profile/pasta/logs/pasta.log"` | ログファイルパス |
| `rotation_days` | `usize` | `7` | ログローテーション日数 |

### 記述例

```toml
[logging]
file_path = "profile/pasta/logs/pasta.log"
rotation_days = 14
```

---

## 3. [persistence] セクション

**処理元**: `PersistenceConfig` 構造体（Rust）  
**参照方法**: Rust側で `config.persistence()` で取得

### フィールド

| キー | 型 | デフォルト値 | 説明 |
|------|-----|--------------|------|
| `obfuscate` | `bool` | `false` | gzip圧縮による難読化 |
| `file_path` | `string` | `"profile/pasta/save/save.json"` | 保存ファイルパス |
| `debug_mode` | `bool` | `false` | 永続化デバッグログ |

### 特記事項

- `obfuscate = true` の場合、`.json` 拡張子は自動的に `.dat` に変更される

### 記述例

```toml
[persistence]
obfuscate = true
file_path = "profile/pasta/save/save.dat"
debug_mode = false
```

---

## 4. [lua] セクション

**処理元**: `LuaConfig` 構造体（Rust）  
**参照方法**: Rust側で `config.lua()` で取得

### フィールド

| キー | 型 | デフォルト値 | 説明 |
|------|-----|--------------|------|
| `libs` | `string[]` | （下記参照） | 有効にするLuaライブラリ |

### libs デフォルト値

```toml
libs = ["std_all", "assertions", "testing", "regex", "json", "yaml"]
```

### 有効なライブラリ名

**Lua標準ライブラリ（`std_*` プレフィックス）:**

| ライブラリ名 | 説明 |
|-------------|------|
| `std_all` | 安全な標準ライブラリ全部（debug除く） |
| `std_all_unsafe` | 全ライブラリ（debug含む、要注意） |
| `std_coroutine` | coroutine ライブラリ |
| `std_table` | table ライブラリ |
| `std_io` | io ライブラリ |
| `std_os` | os ライブラリ |
| `std_string` | string ライブラリ |
| `std_utf8` | utf8 ライブラリ |
| `std_math` | math ライブラリ |
| `std_package` | package ライブラリ |
| `std_debug` | debug ライブラリ（セキュリティ注意） |

**mlua-stdlib モジュール:**

| モジュール名 | 説明 |
|-------------|------|
| `assertions` | アサーション機能 |
| `testing` | テストフレームワーク |
| `env` | 環境変数アクセス（セキュリティ上デフォルト無効） |
| `regex` | 正規表現 |
| `json` | JSON操作 |
| `yaml` | YAML操作 |

### 減算記法

`-` プレフィックスで除外可能：

```toml
[lua]
libs = ["std_all", "-std_io", "testing"]  # std_io を除外
```

---

## 5. [package] セクション（省略可能）

**処理元**: Lua側で `@pasta_config` から参照（現在は未使用）  
**参照方法**: `require("@pasta_config").package`  
**用途**: 将来的な pasta_lua 拡張用（ノベルゲームエンジン等）

### フィールド

| キー | 型 | デフォルト値 | 説明 |
|------|-----|--------------|------|
| `name` | `string` | なし | パッケージ名 |
| `version` | `string` | なし | バージョン（セマンティックバージョニング推奨） |
| `authors` | `string[]` | なし | 作者リスト |

### 特記事項

- **省略可能**: 伺かゴーストでは install.txt/readme.txt でメタデータを管理するため、このセクションは不要
- **将来的な拡張**: pasta_lua をノベルゲームエンジン等に転用する場合に備えた定義
- **現在の利用**: PastaConfig では読み込まれるが、システム側では使用されない

### 記述例

```toml
[package]
name = "hello-pasta"
version = "0.1.0"
authors = ["どっとステーション駅長"]
```

---

## 6. [ghost] セクション

**処理元**: Lua側で `@pasta_config` から参照  
**参照方法**: `require("@pasta_config").ghost` または `pasta.config.get("ghost", key, default)`

### 既定義フィールド

| キー | 型 | デフォルト値 | 説明 | 使用箇所 |
|------|-----|--------------|------|----------|
| `spot_switch_newlines` | `number` | `1.5` | スポット切替時の改行数 | `pasta.shiori.act` |
| `talk_interval_min` | `number` | `180` | ランダムトーク最小間隔（秒） | `virtual_dispatcher` |
| `talk_interval_max` | `number` | `300` | ランダムトーク最大間隔（秒） | `virtual_dispatcher` |
| `hour_margin` | `number` | `30` | 時報前トークスキップマージン（秒） | `virtual_dispatcher` |

### 記述例

```toml
[ghost]
spot_switch_newlines = 2.0
talk_interval_min = 120
talk_interval_max = 240
hour_margin = 45
```

---

## 7. カスタムセクション

`[loader]` 以外の任意のセクション/キーは `@pasta_config` から参照可能。

### 例: トップレベルキー

```toml
ghost_name = "TestGhost"
version = "1.0.0"

[loader]
debug_mode = true
```

Lua側での参照:

```lua
local config = require("@pasta_config")
print(config.ghost_name)  -- "TestGhost"
print(config.version)     -- "1.0.0"
```

### 例: カスタムセクション

```toml
[user_data]
key1 = "value1"
key2 = 42
nested = { inner = "data" }
```

Lua側での参照:

```lua
local config = require("@pasta_config")
print(config.user_data.key1)          -- "value1"
print(config.user_data.nested.inner)  -- "data"
```

---

## 8. hello-pasta 向け pasta.toml 設計案

### 案A: 最小構成

```toml
[loader]
debug_mode = true
```

### 案B: 推奨構成（Cargo.toml風メタデータ付き） ✅ 採用

```toml
# === パッケージメタデータ ===
# Cargo.toml の [package] セクションを参考にした構造
[package]
name = "hello-pasta"
version = "0.1.0"
authors = ["どっとステーション駅長"]

# === ローダー設定 ===
[loader]
debug_mode = true

# === ゴースト動作設定 ===
[ghost]
# スポット切替時の改行（1.5 = 150ms相当の改行）
spot_switch_newlines = 1.5

# ランダムトーク間隔（秒）- テスト用に短縮
talk_interval_min = 60   # 1分
talk_interval_max = 120  # 2分

# 時報前のトークスキップマージン（秒）
hour_margin = 30
```

### 案C: フル構成（全セクション明示）

```toml
# === パッケージメタデータ ===
[package]
name = "hello-pasta"
version = "0.1.0"
authors = ["どっとステーション駅長"]

# === ローダー設定 ===
[loader]
pasta_patterns = ["dic/*/*.pasta"]
lua_search_paths = ["scripts", "profile/pasta/cache/lua"]
transpiled_output_dir = "profile/pasta/cache/lua"
debug_mode = true

# === ログ設定 ===
[logging]
file_path = "profile/pasta/logs/pasta.log"
rotation_days = 7

# === 永続化設定 ===
[persistence]
obfuscate = false
file_path = "profile/pasta/save/save.json"

# === Luaライブラリ設定 ===
[lua]
libs = ["std_all", "assertions", "testing", "regex", "json", "yaml"]

# === ゴースト動作設定 ===
[ghost]
spot_switch_newlines = 1.5
talk_interval_min = 180
talk_interval_max = 300
hour_margin = 30
```

---

## 8. 設計上の注意点

### [package] セクションについて

現在の実装では `[package]` セクションは**定義されていない**。
これは `custom_fields` として `@pasta_config` から参照可能だが、Rust側での特別な処理はない。

サンプルゴーストで `[package]` を導入する場合は、以下を検討：

1. **案A**: 単なるメタデータとして `custom_fields` に含める（現状のまま）
2. **案B**: `PastaConfig` に `PackageConfig` を追加し、正式にサポート

### [ghost] セクションの設計思想

- Lua側から `pasta.config.get("ghost", key, default)` で安全にアクセス
- デフォルト値はLua側で定義（Rust側は関知しない）
- 未定義キーはLua側でデフォルト値にフォールバック

---

## 参考リンク

- [loader/config.rs](../../../crates/pasta_lua/src/loader/config.rs) - Rust実装
- [pasta/config.lua](../../../crates/pasta_lua/scripts/pasta/config.lua) - Lua側設定アクセス
- [virtual_dispatcher.lua](../../../crates/pasta_lua/scripts/pasta/shiori/event/virtual_dispatcher.lua) - 設定使用例
- [LUA_API.md](../../../crates/pasta_lua/LUA_API.md) - API仕様
