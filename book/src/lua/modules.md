# 公開モジュール API

さあ、道具箱を開ける時間ですわ。Pasta ランタイムは、ゴースト作りに必要な便利な機能を
いくつものモジュールとして用意しておりますの。検索、永続化、ログ、文字コード変換――
どれもあなたの手間を肩代わりしてくれる、頼もしい子たちですわ。一つずつご紹介いたしましょう。

---

Pasta ランタイムは Rust 側から Lua VM へモジュール群を公開している。`require` で読み込んで使う。
本章では `scripts/` 配下のスクリプトや DSL 内の ` ```lua ``` ` ブロックから利用できる公開モジュールを扱う。

対象方言は **LuaJIT 2.1**（Lua 5.1 系）である。

## モジュール一覧

`@` で始まる名前は Rust 組み込みモジュール（ランタイムが提供）、`pasta.` で始まる名前は
内部 Lua モジュールである。本章は主に前者の Rust 組み込みモジュールを扱う。
内部モジュール（`pasta.word` 等）の実用パターンは [scripts/ の記述パターン](patterns.md) で扱う。

| モジュール | 用途 | require 方法 |
| ---- | ---- | ---- |
| `@pasta_search` | シーン・単語検索 | `require` 直接 |
| `@pasta_persistence` | セーブデータ永続化 | `require` 直接 |
| `@pasta_sakura_script` | さくらスクリプト変換 | `require` 直接 |
| `@enc` | UTF-8 ⇔ ANSI 変換 | `require` 直接 |
| `@pasta_config` | pasta.toml 設定読み取り | **`pcall(require, ...)` 必須** |
| `@pasta_log` | ロギング | `require` 直接 |

加えて mlua-stdlib 統合モジュール（`@json` / `@yaml` / `@regex` / `@assertions` / `@testing`）が
デフォルトで有効である。`@env`（環境変数・パスアクセス）は**セキュリティ上の理由からデフォルト無効**で、
通常のゴーストでは使えない。

---

## @pasta_log — ロギング

最も手軽で、最初に覚えたいモジュール。Lua からのログ出力を Rust 側の tracing 基盤へ橋渡しする。
呼び出し元のファイル名・行番号・関数名が自動でログに付与される。`RuntimeConfig` の設定に関わらず常に利用可能。

`trace` / `debug` / `info` / `warn` / `error` の5レベルを持つ。

```lua
local log = require "@pasta_log"

log.info("ゴースト起動完了")
log.debug("変数の値: " .. tostring(some_var))
log.warn("非推奨 API が使用されています")
log.error("設定ファイルの読み込みに失敗")

-- 文字列以外もそのまま渡せる（table は JSON 変換、nil は空文字列に）
log.info({ player = "ユーザー", score = 100 })  -- {"player":"ユーザー","score":100}
log.trace(nil)                                   -- "" として出力
```

各関数は任意の Lua 値を受け取り、内部で文字列へ変換する。変換に失敗してもエラーや panic は発生せず、
`"<unconvertible value>"` が出力される（ログ呼び出しがゴーストを止めない設計）。

---

## @pasta_persistence — 永続化

セーブデータを JSON（または gzip 圧縮）形式でファイルに保存・読み込みする。

```lua
local persistence = require "@pasta_persistence"

-- 読み込み（ファイル未存在・破損時は空テーブル {} を返す）
local data = persistence.load()
data.play_count = (data.play_count or 0) + 1

-- 保存（成功で true、失敗で nil, error_message）
local ok, err = persistence.save(data)
if not ok then
    log.error("保存失敗: " .. tostring(err))
end
```

`load()` はファイルが無い初回起動時や破損時でも例外を投げず、空テーブルを返す（データ損失防止のため
エラーは握りつぶす）。`save(data)` は循環参照や関数値を含むテーブルでは失敗するので、戻り値を確認する。

実際のゴーストでは、永続化データはシーン関数の `act:init_scene()` が返す `save` テーブル経由で
触ることが多い（[scripts/ の記述パターン](patterns.md) を参照）。

`pasta.toml` の `[persistence]` セクションで動作を設定できる。

```toml
[persistence]
obfuscate = true                              # gzip 圧縮を有効化（拡張子が .dat に）
file_path = "profile/pasta/save/save.json"    # 保存先パス
debug_mode = false                            # デバッグログ出力
```

> **セーブキーの命名規約**: `pasta_` プレフィックスはエンジン予約領域である。ゴースト固有のキーには
> `pasta_` を付けず、`talk_count` や `my_flag` のような任意の名前を使うこと。

---

## @pasta_config — 設定読み取り

`pasta.toml` のカスタムフィールドへ読み取り専用でアクセスする。テスト環境やスタンドアロン実行時には
`pasta.toml` が存在しないことがあるため、**`pcall` で保護して require する**のが必須である。

```lua
local ok, config = pcall(require, "@pasta_config")  -- pcall 必須
if ok then
    print(config.character.name)                    -- トップレベル
    print(config.character.appearance.hair_color)   -- ネストしたアクセス
    local version = config.system and config.system.version or "unknown"  -- nil ガード
end
```

`[loader]` セクション**以外**のすべてのセクション・フィールドが公開される（`[loader]` は内部設定のため
除外）。値は読み取り専用で、TOML の型・ネスト構造がそのまま保持される。設定ファイル未存在時は空テーブルになる。

---

## @pasta_search — シーン・単語検索

シーンと単語を前方一致で検索する。フォールバック戦略（ローカル → グローバル）を備える。

> **利用可能タイミング**: `require("pasta").finalize_scene()` 呼び出し後にのみ使用できる。
> 通常はトランスパイル済みコードの読み込み後にランタイムが自動で呼ぶため、シーン関数内では使用可能。

```lua
local SEARCH = require "@pasta_search"

-- シーン検索（成功で global_name, local_name の2値、失敗で nil）
local global, local_name = SEARCH:search_scene("メイン")
if global then
    print(global, local_name)  -- 例: "メイン_1", "__start__"
end

-- ローカル優先検索（第2引数に親グローバルシーン名を指定）
local g, l = SEARCH:search_scene("選択肢", "メイン_1")

-- 単語検索（成功で値の文字列、失敗で nil）
local word = SEARCH:search_word("挨拶")
```

このモジュールはテスト時の決定論的制御 API（`set_scene_selector` / `set_word_selector`）も持つ。
詳細は本マニュアルの範囲外だが、複数候補からの選択順序を整数インデックス（0 始まり）で固定できる。

実際のシーン関数では、`@pasta_search` を直接呼ぶより、`act:word(name)` や ACT オブジェクトの
検索メソッド経由で使うことが多い（内部で `@pasta_search` を利用する）。

---

## @pasta_sakura_script — さくらスクリプト変換

セリフテキストにウェイトタグ（`\_w[ms]`）を自動挿入し、自然な会話テンポのさくらスクリプトへ変換する。
既存のさくらスクリプトタグ（`\s[5]` 等）はそのまま保持される。

```lua
local SAKURA_SCRIPT = require "@pasta_sakura_script"

local actor = { talk = {} }  -- talk サブテーブルにウェイト設定を持つ
local result = SAKURA_SCRIPT.talk_to_script(actor, "こんにちは。")
-- 結果: "こ\_w[50]ん\_w[50]に\_w[50]ち\_w[50]は\_w[100]。"
```

ウェイト値は `actor.talk` テーブルのフィールド（`script_wait_default` = 50ms、`script_wait_period` = 100ms 等）で
調整できる。`pasta.toml` の `[talk]` セクションでデフォルト値を設定でき、actor 側の設定が優先される。

budoux 日本語分割モデルを使った自動改行 `break_lines(text, widths)` も提供する。actor に `budoux`
フィールドが設定されていれば `talk_to_script` が自動で後段適用するため、通常は直接呼ぶ必要はない。

なお、シーン関数で `act:talk(actor, text)` を使うと、このモジュールが内部的に呼ばれてテキストが
自動変換される。多くの場合、`@pasta_sakura_script` を直接 require する必要はない。

---

## @enc — 文字コード変換

UTF-8 と ANSI（システムロケール、Windows では CP932 / Shift_JIS）間の文字コード変換。
主に Windows 環境でのファイルパス処理に使う。`(result, error)` の2値を返す。

```lua
local enc = require "@enc"

-- UTF-8 → ANSI
local ansi_path, err = enc.to_ansi("C:/ユーザー/設定.txt")
if ansi_path then
    local file = io.open(ansi_path, "r")
    -- ...
else
    log.error("変換エラー: " .. tostring(err))
end

-- ANSI → UTF-8
local utf8_path = enc.to_utf8(some_windows_api_result())
```

> **プラットフォーム依存**: 変換結果は OS のロケールに依存する。Windows 以外では挙動が異なる可能性がある。

---

## mlua-stdlib 統合モジュール

汎用ユーティリティとして次のモジュールがデフォルトで有効である。

```lua
-- JSON
local json = require "@json"
local str = json.encode({ name = "test", value = 42 })
local obj = json.decode('{"name": "test"}')

-- YAML
local yaml = require "@yaml"
local y = yaml.encode({ name = "test" })

-- 正規表現
local regex = require "@regex"
local pattern = regex.new("\\d+")
local matches = pattern:find_all("abc123def456")
```

`@assertions` と `@testing` はテスト記述用に有効化されている。`@env`（環境変数・ファイルパスアクセス）は
**デフォルト無効**で、有効化には Rust 側の `RuntimeConfig` 設定が必要なため、通常のゴーストからは使えない。

---

道具の名前と使い方を、ざっと頭に入れていただけたかしら。フンッ、最初は覚えきれなくても結構ですわ。
必要になったらこの章へ戻ってくればよいのですから。次は、これらの道具を実際の `scripts/` で
どう組み合わせるか――生きた書き方を見ていきましょう。さあ、熱く参りますわよ！
