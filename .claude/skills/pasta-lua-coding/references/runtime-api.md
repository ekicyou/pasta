# Runtime API リファレンス

pasta_luaクレートがRust側からLua VMに公開しているモジュール群の完全APIリファレンス。  
本ドキュメントは `scripts/` 配下のカスタムLuaスクリプトや `pasta_scripts/` 配下のランタイムスクリプト、および Pasta DSL内の ` ```lua ``` ` ブロックで使用するランタイムモジュールを網羅する。

---

## @pasta_search

シーンと単語の検索機能を提供するモジュール。Rust側の `SearchContext`（Radix Trie）にバインドする。

```lua
local SEARCH = require "@pasta_search"
```

> **利用可能タイミング**: `require("pasta").finalize_scene()` 呼び出し後。それ以前の使用はエラーになる。

### search_scene(name, global_scene_name?)

シーンを前方一致で検索する。フォールバック戦略（ローカル → グローバル）を使用。

```lua
SEARCH:search_scene(name, global_scene_name?) -> global_name, local_name | nil
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `name` | string | ✅ | 検索するシーン名（前方一致） |
| `global_scene_name` | string | ❌ | 親シーン名（指定時はローカル検索を優先） |

**戻り値**:
- **成功時**: `global_name, local_name` の2値
  - `global_name`: グローバルシーン名（例: `"メイン_1"`）
  - `local_name`: ローカルシーン名（例: `"__選択肢_1__"` または `"__start__"`）
- **失敗時**: `nil`

**フォールバック検索戦略**:
1. `global_scene_name` 指定時 → ローカルシーン（指定グローバルシーン内）を検索 → 見つからなければグローバルシーンにフォールバック
2. `global_scene_name` 省略時 → グローバルシーンのみ検索

```lua
-- グローバル検索のみ
local global, local_name = SEARCH:search_scene("メイン")
if global then
    print("Found:", global, local_name)  -- "メイン_1", "__start__"
end

-- ローカル優先検索（フォールバック付き）
local g, l = SEARCH:search_scene("選択肢", "メイン_1")
if g then
    print("Found local scene:", g, l)  -- "メイン_1", "__選択肢_1__"
end

-- 見つからない場合
local result = SEARCH:search_scene("存在しないシーン")
if not result then
    print("Scene not found")
end
```

### search_word(name, global_scene_name?)

単語を検索する。フォールバック戦略（ローカル → グローバル）を使用。

```lua
SEARCH:search_word(name, global_scene_name?) -> string | nil
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `name` | string | ✅ | 検索する単語キー |
| `global_scene_name` | string | ❌ | 親シーン名（指定時はローカル単語を優先） |

**戻り値**:
- **成功時**: 単語の値（文字列）
- **失敗時**: `nil`

```lua
-- グローバル単語を検索
local word = SEARCH:search_word("挨拶")
if word then print(word) end  -- "こんにちは"

-- ローカル単語を優先検索
local local_word = SEARCH:search_word("返事", "メイン_1")
```

### set_scene_selector(...) / set_word_selector(...)

テスト用にランダム選択を決定論的に制御する。

```lua
SEARCH:set_scene_selector(n1, n2, ...)  -- シーケンス設定
SEARCH:set_scene_selector()             -- デフォルト（ランダム）に戻す

SEARCH:set_word_selector(n1, n2, ...)   -- シーケンス設定
SEARCH:set_word_selector()              -- デフォルトに戻す
```

| パラメータ | 型 | 説明 |
|-----------|---|------|
| `n1, n2, ...` | integer | 選択インデックスのシーケンス（0始まり） |

複数の候補がある場合（重複シーン、複数値の単語など）、通常はランダム選択される。テストでは決定論的な動作が必要なため、選択順序を事前指定できる。

```lua
SEARCH:set_scene_selector(0, 0, 0)  -- 常に最初の候補を選択
SEARCH:set_word_selector(0, 1, 0)   -- 1番目、2番目、1番目の順

-- テスト後にリセット必須
SEARCH:set_scene_selector()
SEARCH:set_word_selector()
```

> 📖 テストでの活用パターンは [testing-lint.md](testing-lint.md#決定論的テスト) を参照。

---

## @pasta_persistence

セーブデータの永続化機能。JSON/gzip形式でファイルに保存・読み込みする。

```lua
local persistence = require "@pasta_persistence"
```

**モジュールメタデータ**: `_VERSION = "0.1.0"`, `_DESCRIPTION = "Persistent data storage (JSON/gzip)"`

### load()

永続化ファイルからデータを読み込む。

```lua
persistence.load() -> table
```

**戻り値**: 保存されていたデータ（Luaテーブル）。ファイル未存在時・読み込みエラー時は空テーブル `{}`。

**特記事項**:
- ファイル未存在（初回起動等）→ 空テーブル返却
- ファイル破損 → 空テーブル返却（エラーログ出力、データ損失防止のためエラーは握りつぶす）
- gzip圧縮ファイル（`.dat`）も自動検出・展開

```lua
local data = persistence.load()
data.play_count = data.play_count or 0
data.player_name = data.player_name or "Guest"
```

### save(data)

データを永続化ファイルに保存する。

```lua
persistence.save(data) -> true, nil | nil, error_message
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `data` | table | ✅ | 保存するデータ（Luaテーブル） |

**エラー条件**:
- Lua値のJSON変換失敗（循環参照、関数値など）
- ファイル書き込み失敗（権限不足、ディスク容量不足など）

```lua
local data = persistence.load()
data.play_count = (data.play_count or 0) + 1
data.last_played = os.date()

local ok, err = persistence.save(data)
if not ok then
    print("保存失敗:", err)
end
```

### pasta.toml設定

`[persistence]` セクションで動作をカスタマイズ。

```toml
[persistence]
obfuscate = true                              # gzip圧縮有効化（難読化）
file_path = "profile/pasta/save/save.json"    # 保存先パス
debug_mode = false                            # デバッグログ出力
```

| オプション | 型 | デフォルト | 説明 |
|-----------|---|----------|------|
| `obfuscate` | bool | `false` | gzip圧縮有効化（拡張子が `.dat` に変更） |
| `file_path` | string | `"profile/pasta/save/save.json"` | 保存先パス |
| `debug_mode` | bool | `false` | デバッグログ出力有効化 |

---

## @pasta_config

`pasta.toml` のカスタムフィールドに読み取り専用でアクセスするモジュール。

```lua
local ok, config = pcall(require, "@pasta_config")  -- pcall必須
```

> **⚠️ pcall保護が必須**: テスト環境やスタンドアロン実行時には `pasta.toml` が存在しない場合がある。`pcall` なしで `require` するとエラーで停止する。

### 公開されるフィールド

`pasta.toml` 内の `[loader]` セクション**以外**のすべてのセクション・フィールドが公開される。

```toml
# pasta.toml例
[loader]
# ❌ loader セクションは @pasta_config には含まれない
pasta_patterns = ["dic/*/*.pasta"]

[character]
name = "まゆら"
age = 17

[character.appearance]
hair_color = "黒"
eye_color = "茶"

[system]
debug = true
version = "1.0.0"
```

### アクセス例

```lua
local ok, config = pcall(require, "@pasta_config")
if ok then
    -- トップレベルアクセス
    print(config.character.name)  -- "まゆら"
    print(config.character.age)   -- 17

    -- ネストしたアクセス
    print(config.character.appearance.hair_color)  -- "黒"

    -- 安全なアクセス（nilガード）
    local version = config.system and config.system.version or "unknown"
end
```

### 注意事項
- **読み取り専用**: 値の変更はできない
- **TOML構造の保持**: ネストしたテーブル構造がそのまま維持される
- **型保持**: TOML型がそのまま維持（数値、文字列、真偽値、配列、テーブル）
- **`[loader]` 除外**: 内部設定のため公開されない
- **設定ファイル未存在時**: 空テーブルになる

---

## @pasta_sakura_script

テキストにウェイトタグ（`\_w[ms]`）を自動挿入してさくらスクリプトに変換する。セリフの自然な会話テンポを演出する。

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"
```

### talk_to_script(actor, talk)

```lua
SAKURA_SCRIPT.talk_to_script(actor, talk) -> string
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `actor` | table | ✅ | actorオブジェクト。`talk` サブテーブルにウェイト設定を持つ |
| `talk` | string | ✅ | 変換対象のセリフテキスト |

### actor.talkテーブルの全フィールド

| フィールド | 型 | デフォルト | 説明 |
|-----------|---|----------|------|
| `script_wait_default` | integer | 50 | デフォルトウェイト（ms） |
| `script_wait_period` | integer | 100 | 句点「。」のウェイト（ms） |
| `script_wait_comma` | integer | 75 | 読点「、」のウェイト（ms） |
| `script_wait_newline` | integer | 100 | 改行のウェイト（ms） |
| `script_wait_exclamation` | integer | 100 | 感嘆符・疑問符のウェイト（ms） |
| `chars_period` | string | `"。．.｡"` | 句点として認識する文字 |
| `chars_comma` | string | `"、，,､"` | 読点として認識する文字 |
| `chars_exclamation` | string | `"！？!?‼⁉❗❓"` | 感嘆符・疑問符として認識する文字 |
| `chars_no_wait` | string | `"…ー〜～"` | ウェイトを挿入しない文字 |
| `chars_half_wait` | string | `"っッ"` | 半分のウェイトを挿入する文字 |
| `chars_newline` | string | `"\n"` | 改行として認識する文字 |

### 動作仕様

1. **さくらスクリプトタグの保護**: 既存タグ（`\s[5]`, `\_w[100]` 等）はそのまま保持
2. **文字種別判定**: 各文字を句点/読点/感嘆符/改行/ノーウェイト/ハーフウェイト/通常に分類
3. **句読点の累積**: 連続する句読点は累積し、最後にまとめてウェイトを挿入
4. **ノーウェイト文字**: 「…」「ー」などはウェイトなしで出力
5. **ハーフウェイト文字**: 「っ」「ッ」は半分のウェイト（切り捨て）

### 使用例

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"

-- デフォルト設定
local actor = { talk = {} }
local result = SAKURA_SCRIPT.talk_to_script(actor, "こんにちは。")
-- 結果: "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[100]。"

-- カスタム設定
local actor2 = { talk = { script_wait_default = 80, script_wait_period = 200 } }
local result2 = SAKURA_SCRIPT.talk_to_script(actor2, "やあ。")
-- 結果: "や\_w[80]あ\_w[200]。"

-- さくらスクリプトタグはそのまま保持
local actor3 = { talk = { script_wait_default = 50 } }
local result3 = SAKURA_SCRIPT.talk_to_script(actor3, "こんにちは\\s[5]元気？")
-- 結果: "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\\s[5]元\_w[50]気\_w[100]？"

-- 連続する句読点（累積）
local result4 = SAKURA_SCRIPT.talk_to_script(actor, "え！？")
-- 結果: "え\_w[200]！？"  -- 100 + 100 = 200

-- ハーフウェイト文字
local result5 = SAKURA_SCRIPT.talk_to_script(actor, "あっ")
-- 結果: "あ\_w[50]っ\_w[25]"  -- 50 / 2 = 25
```

### pasta.toml での設定

`[talk]` セクションでデフォルト値を設定できる。actor側の設定が優先される（`PastaConfig.talk()` 経由でマージ）。

```toml
[talk]
script_wait_default = 50
script_wait_period = 100
script_wait_comma = 75
```

### break_lines(text, widths)

budoux 日本語分割モデルを用いて、テキストの自然な区切り位置にさくらスクリプト改行タグ（`\n`）を挿入する。
さくらスクリプトタグ（`\_w[ms]` 等）は幅計算から除外されつつ元の位置関係を保持して出力される。

```lua
SAKURA_SCRIPT.break_lines(text, widths) -> string
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `text` | string \| nil | ✅ | 処理対象テキスト。`nil` の場合は空文字列を返す |
| `widths` | table \| nil | ✅ | 行ごとの幅閾値（CJK文字幅）の配列。空テーブル・`nil` の場合は入力をそのまま返す |

**widths の仕様**:
- `widths[1]` = 1行目の幅上限
- `widths[2]` = 2行目の幅上限
- 配列末尾の値が3行目以降に繰り返し適用される（例: `{10, 12}` → 3行目以降は幅12）
- 例: `{10, 12}` なら1行目≤10、2行目以降≤12

**通常 `talk_to_script` と組み合わせて使う必要はない**: `actor.budoux` フィールドが設定されていれば `talk_to_script` が自動的に `break_lines` を後段処理として呼び出す。`break_lines` を直接呼ぶのはカスタムパイプラインを構築する場合のみ。

### pasta.toml での budoux 設定

アクターへ `budoux` フィールドを追加することで、`talk_to_script` が自動的に改行を挿入するようになる。

```toml
[actor."女の子"]
spot = 0
budoux = [10, 12]
# 1行目≤10文字幅、2行目以降≤12文字幅で自動改行
```

### 使用例（直接呼び出し）

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"

-- 直接呼び出し（カスタムパイプライン用）
local result = SAKURA_SCRIPT.break_lines("今日はいい天気ですね", {6})
-- 結果例: "今日は\nいい天気ですね"（budoux分割位置に依存）

-- さくらスクリプトタグは幅計算から除外され出力に保持される
local with_tags = SAKURA_SCRIPT.break_lines(
    "こ\\_w[50]れ\\_w[50]は\\_w[50]テ\\_w[50]ス\\_w[50]ト",
    {6}
)
-- 結果例: "こ\\_w[50]れ\\_w[50]は\\_w[50]\nテ\\_w[50]ス\\_w[50]ト"

-- nil / 空テーブルは安全
local r1 = SAKURA_SCRIPT.break_lines(nil, {10})   -- ""
local r2 = SAKURA_SCRIPT.break_lines("テスト", {}) -- "テスト"（変更なし）
local r3 = SAKURA_SCRIPT.break_lines("テスト", nil) -- "テスト"（変更なし）

-- talk_to_script 経由の自動適用（pasta.toml に budoux = [10, 12] が設定済み）
local actor = CONFIG.actor["女の子"]  -- actor.budoux = {10, 12} が含まれる
local script = SAKURA_SCRIPT.talk_to_script(actor, "今日はいい天気ですね")
-- ウェイト挿入後に自動的に break_lines が適用される
```

---

## @enc

UTF-8 ⇔ ANSI（システムロケール）間の文字コード変換。主にWindows環境でのファイルパス処理に使用。

```lua
local enc = require "@enc"
```

**モジュールメタデータ**: `_VERSION = "0.1.0"`, `_DESCRIPTION = "Encoding conversion (UTF-8 <-> ANSI)"`

> **⚠️ プラットフォーム依存性**: Windows環境ではShift_JIS（CP932）変換、Unix環境では処理が異なる可能性がある。クロスプラットフォームのコードでは変換結果の違いに注意。

### to_ansi(utf8_str)

UTF-8文字列をANSIエンコーディングに変換する。

```lua
enc.to_ansi(utf8_str) -> ansi_string, nil | nil, error_message
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `utf8_str` | string | ✅ | 変換元のUTF-8文字列 |

**エラーケース**:
- 入力が有効なUTF-8でない場合
- ANSI変換に失敗した場合（表現できない文字を含む）

### to_utf8(ansi_str)

ANSIエンコーディングのバイト列をUTF-8文字列に変換する。

```lua
enc.to_utf8(ansi_str) -> utf8_string, nil | nil, error_message
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `ansi_str` | string | ✅ | 変換元のANSIバイト列 |

### 戻り値パターン

`@enc` モジュールは `(result, error)` タプルパターンを採用:

```lua
local ansi_path, err = enc.to_ansi("C:/ユーザー/設定.txt")
if ansi_path then
    local file = io.open(ansi_path, "r")
    if file then
        local content = file:read("*a")
        file:close()
    end
else
    print("変換エラー:", err)
end

-- ANSI → UTF-8
local utf8_path, err = enc.to_utf8(some_windows_api_result())
if utf8_path then
    print("ファイルパス:", utf8_path)
end
```

---

## @pasta_log

Luaからのログ出力をRust tracingインフラにブリッジするモジュール。呼び出し元のLuaファイル名・行番号・関数名を自動キャプチャして構造化ログイベントとして出力する。

```lua
local log = require "@pasta_log"
```

**モジュールメタデータ**: `_VERSION = "0.1.0"`, `_DESCRIPTION = "Lua logging bridge to Rust tracing"`

> **常時利用可能**: `RuntimeConfig.libs` の設定に関わらず、常にロードされる。

### trace(value)

TRACEレベルでログを出力する。

```lua
log.trace(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `value` | any | ❌ | ログメッセージ（省略時・nil時は空文字列） |

### debug(value)

DEBUGレベルでログを出力する。

```lua
log.debug(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `value` | any | ❌ | ログメッセージ |

### info(value)

INFOレベルでログを出力する。

```lua
log.info(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `value` | any | ❌ | ログメッセージ |

### warn(value)

WARNレベルでログを出力する。

```lua
log.warn(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `value` | any | ❌ | ログメッセージ |

### error(value)

ERRORレベルでログを出力する。

```lua
log.error(value) -> nil
```

| パラメータ | 型 | 必須 | 説明 |
|-----------|---|------|------|
| `value` | any | ❌ | ログメッセージ |

### 値の変換規則

各関数は任意のLua値を受け取り、以下の優先順で文字列に変換する：

| Lua型 | 変換方法 |
|-------|---------|
| `string` | そのまま出力 |
| `integer`, `number`, `boolean` | `tostring()` 相当 |
| `table` | JSON変換（要素数≤1000、ネスト深さ≤10の場合）。制限超過時は `tostring()` フォールバック |
| `nil`（または引数なし） | 空文字列 `""` |
| `function`, `userdata`, `thread` 等 | `tostring()` フォールバック |

変換失敗時は `"<unconvertible value>"` を出力する。**エラーやpanicは発生しない**。

### 構造化ログフィールド

各ログイベントには以下の構造化フィールドが自動付与される：

| フィールド | 説明 | 例 |
|-----------|------|-----|
| `lua_source` | 呼び出し元ソースファイル | `"@main.lua"`, `"@scripts/touch_detect.lua"` |
| `lua_line` | 呼び出し元行番号 | `42` |
| `lua_fn` | 呼び出し元関数名 | `"on_boot"`, `""` (トップレベル時) |

### 使用例

```lua
local log = require "@pasta_log"

-- 基本的なログ出力
log.info("ゴースト起動完了")
log.debug("変数の値を確認: " .. tostring(some_var))
log.warn("非推奨APIが使用されています")
log.error("設定ファイルの読み込みに失敗")

-- 任意の型をそのままログ出力可能
log.info({player = "ユーザー", score = 100})  -- JSON: {"player":"ユーザー","score":100}
log.debug(42)                                  -- "42"
log.trace(nil)                                 -- "" (空文字列)
log.warn(true)                                 -- "true"

-- Lua callstack情報は自動キャプチャされる
-- 出力例: INFO pasta_lua::runtime::log: ゴースト起動完了 lua_source="@main.lua" lua_line=15 lua_fn="on_boot"
```

---

## mlua-stdlib 統合モジュール

pasta_luaは [mlua-stdlib](https://github.com/khvzak/mlua-stdlib) を統合しており、追加のユーティリティモジュールが利用可能。

### デフォルトで有効なモジュール

#### @json

JSON エンコード/デコード。

```lua
local json = require "@json"

local str = json.encode({name = "test", value = 42})
local obj = json.decode('{"name": "test"}')
```

#### @yaml

YAML エンコード/デコード。

```lua
local yaml = require "@yaml"

local str = yaml.encode({name = "test"})
local obj = yaml.decode("name: test")
```

#### @regex

正規表現サポート。

```lua
local regex = require "@regex"

local pattern = regex.new("\\d+")
local matches = pattern:find_all("abc123def456")
```

#### @assertions

アサーション関数群。

```lua
local assert = require "@assertions"

assert.equal(1, 1)
assert.not_equal(1, 2)
assert.truthy(true)
assert.falsy(nil)
```

#### @testing

テストフレームワーク。

```lua
local testing = require "@testing"

testing.describe("My Feature", function()
    testing.it("should work", function()
        -- テストコード
    end)
end)
```

### デフォルトで無効なモジュール

#### @env

環境変数とファイルシステムパスへのアクセス。**セキュリティ上の理由からデフォルト無効**。

有効化にはRust側の `RuntimeConfig` 設定が必要。`libs` 配列に `"env"` エントリを追加します:

```rust
use pasta_lua::RuntimeConfig;

// デフォルト構成に @env を追加（libs 配列で制御）
let config = RuntimeConfig::from_libs(vec![
    "std_all".into(),
    "assertions".into(),
    "testing".into(),
    "env".into(),
    "regex".into(),
    "json".into(),
    "yaml".into(),
]);

// または、すべて有効化（@env を含む）
let config = RuntimeConfig::full();
```

```lua
-- @env が有効な場合のみ
local env = require "@env"
local home = env.var("HOME")
```

### RuntimeConfig によるモジュール制御

Rust側では `libs` 配列で各モジュールの有効/無効を制御します。
プリセットのコンストラクタを使うか、`from_libs` でカスタム配列を渡します:

```rust
use pasta_lua::RuntimeConfig;

RuntimeConfig::new()      // デフォルト（std_all + assertions/testing/regex/json/yaml、@env は無効）
RuntimeConfig::full()     // すべて有効（std_all_unsafe + @env を含む）
RuntimeConfig::minimal()  // std_all のみ（mlua-stdlib モジュールなし）

// カスタム設定: libs 配列のエントリで有効化、`-` プレフィックスで無効化
let config = RuntimeConfig::from_libs(vec![
    "std_all".into(),    // 安全な Lua 標準ライブラリすべて
    "assertions".into(), // @assertions を有効化
    "regex".into(),      // @regex を有効化
    "json".into(),       // @json を有効化
    "-std_debug".into(), // std_debug を除外
]);
```

> モジュールの有効/無効は `libs` 配列のエントリ（例: `"env"` で有効化、`"-env"` で無効化）で表現します。`enable_*` のような boolean フィールドは存在しません。

---

## 関連リファレンス

- [internal-modules.md](internal-modules.md) — ACTオブジェクトの`act:word()`が内部的に`@pasta_search`を利用する
- [shiori-handlers.md](shiori-handlers.md) — SHIORIハンドラ内でのランタイムAPI活用パターン
