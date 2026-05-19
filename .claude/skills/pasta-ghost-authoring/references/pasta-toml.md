# pasta.toml リファレンス

> Pasta ゴーストプロジェクトの設定ファイル `pasta.toml` 全セクション・全キーのリファレンス。

---

## 概要

`pasta.toml` はゴーストプロジェクトのルートに配置される設定ファイル。
2種類のセクションで構成される:

- **エンジン正式解析セクション** — Rust 構造体で型検証される（`[loader]`、`[logging]`、`[persistence]`、`[lua]`、`[talk]`）
- **カスタムフィールドセクション** — TOML 構造がそのまま Lua に透過される（`[package]`、`[ghost]`、`[actor."名前"]`）

### 最小構成例

```toml
[loader]
pasta_patterns = ["dic/*.pasta"]

[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

---

## [loader]（ファイル読み込み）

辞書ファイルと Lua モジュールの読み込みを制御する。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `pasta_patterns` | `string[]` | `["dic/*/*.pasta"]` | 読み込む `.pasta` ファイルの glob パターン |
| `lua_search_paths` | `string[]` | *(下記参照)* | Lua モジュール検索パス（優先順位順） |
| `transpiled_output_dir` | `string` | `"profile/pasta/cache/lua"` | トランスパイル済み Lua の出力先 |
| `debug_mode` | `bool` | `true` | デバッグモード（トランスパイル出力の保存等） |

### pasta_patterns

`dic/` 配下の `.pasta` ファイルを読み込むのが一般的。

```toml
[loader]
pasta_patterns = ["dic/*.pasta"]
```

### lua_search_paths

Lua の `require()` が検索するパスの一覧。既定値（優先順位順）:

1. `profile/pasta/save/lua` — ユーザー保存スクリプト（最優先）
2. `scripts` — ユーザーカスタムスクリプト
3. `pasta_scripts` — pasta 標準ランタイム
4. `profile/pasta/cache/lua` — トランスパイル済みキャッシュ
5. `scriptlibs` — 追加ライブラリ

---

## [ghost]（ゴースト動作）

ゴーストの動作パラメータを設定する。カスタムフィールドとして Lua に透過される。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `talk_interval_min` | `integer` | `180` | ランダムトーク最小間隔（秒） |
| `talk_interval_max` | `integer` | `300` | ランダムトーク最大間隔（秒） |
| `hour_margin` | `integer` | `30` | OnHour 誤差許容秒数 |
| `spot_newlines` | `number` | `1.5` | スポット切替時の改行量（`\n[半角]` の倍率） |

### talk_interval_min / talk_interval_max

ランダムトーク（OnTalk）の発火間隔を秒単位で指定する。SAVE テーブルの `pasta_talk_interval_min` / `pasta_talk_interval_max` で実行時に上書き可能（詳細は [variables.md](variables.md#永続化とsaveテーブル) を参照）。

```toml
[ghost]
talk_interval_min = 120
talk_interval_max = 240
```

### spot_newlines

アクターのスポット（バルーン）が切り替わるときに挿入される改行量。値 `1.5` は `\n[half]`（1.5行分の改行）に相当する。

```toml
[ghost]
spot_newlines = 2.0
```

---

## [actor."名前"]（アクター設定）

アクターごとの設定。`"名前"` は `descript.txt` の `sakura.name` / `kero.name` と一致させる。
複数アクターを定義可能。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `spot` | `integer` | *(なし)* | バルーン位置（0=sakura 側, 1=kero 側） |
| `budoux` | `integer[]` | *(なし)* | BudouX 自動改行幅 |
| `default_surface` | `integer` | *(なし)* | デフォルトサーフェス ID |

### spot

バルーンの割り当てを制御する。`0` がメイン（sakura 側）、`1` がサブ（kero 側）。

```toml
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

### budoux

BudouX による自動改行の幅を配列形式 `[行1文字幅, 行2以降文字幅]` で指定する。
設定すると、アクターの発話テキストが指定幅で自動的に `\n` 改行される。

- 要素が1つの場合: 全行に同じ幅を適用（例: `budoux = [10]`）
- 要素が2つの場合: 1行目と2行目以降で異なる幅を適用（例: `budoux = [10, 12]`）

BudouX は日本語の自然な分かち書き位置で改行するため、単語の途中では改行されない。

```toml
[actor."女の子"]
spot = 0
budoux = [10, 12]   # 1行目≤10文字、2行目以降≤12文字

[actor."男の子"]
spot = 1
budoux = [10]        # 全行≤10文字
```

### default_surface

アクターの初期サーフェス ID。設定すると、シーン開始時にこのサーフェスが自動適用される。

```toml
[actor."女の子"]
spot = 0
default_surface = 0

[actor."男の子"]
spot = 1
default_surface = 10
```

---

## [talk]（トーク表示制御）

さくらスクリプト生成時のウェイト挿入と禁則処理を制御する。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `script_wait_normal` | `integer` | `50` | 通常文字ウェイト（ms） |
| `script_wait_period` | `integer` | `1000` | 句点ウェイト（ms） |
| `script_wait_comma` | `integer` | `500` | 読点ウェイト（ms） |
| `script_wait_strong` | `integer` | `500` | 強調記号ウェイト（ms） |
| `script_wait_leader` | `integer` | `200` | リーダーウェイト（ms） |
| `chars_period` | `string` | `"｡。．."` | 句点として扱う文字 |
| `chars_comma` | `string` | `"、，,"` | 読点として扱う文字 |
| `chars_strong` | `string` | `"？！!?"` | 強調記号として扱う文字 |
| `chars_leader` | `string` | `"･・‥…"` | リーダーとして扱う文字 |
| `chars_line_start_prohibited` | `string` | *(行頭禁則文字列)* | 行頭に来てはいけない文字 |
| `chars_line_end_prohibited` | `string` | *(行末禁則文字列)* | 行末に来てはいけない文字 |

**使用例**: 表示速度を速くする場合:

```toml
[talk]
script_wait_normal = 30
script_wait_period = 600
script_wait_comma = 300
```

---

## [persistence]（永続化）

SAVE テーブルの保存設定を制御する。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `obfuscate` | `bool` | `false` | gzip 圧縮による難読化 |
| `file_path` | `string` | `"profile/pasta/save/save.json"` | 保存ファイルパス |
| `debug_mode` | `bool` | `false` | デバッグモード（保存内容のログ出力等） |

```toml
[persistence]
obfuscate = true
file_path = "profile/pasta/save/save.json"
```

---

## [logging]（ログ出力）

ログファイル出力の設定を制御する。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `file_path` | `string` | `"profile/pasta/logs/pasta.log"` | ログファイルパス |
| `rotation_days` | `integer` | `7` | ログローテーション日数 |
| `level` | `string` | `"info"` | デフォルトログレベル（`error`/`warn`/`info`/`debug`/`trace`） |
| `filter` | `string` | *(なし)* | EnvFilter ディレクティブ（設定時は `level` より優先） |

```toml
[logging]
level = "debug"
filter = "debug,pasta_shiori=info"
```

---

## [package]（パッケージ情報）★ 上級者向け

パッケージメタデータ。伺かゴーストでは `install.txt` / `readme.txt` で管理できるため省略可能。将来の汎用用途（ノベルゲーム、ツール等）では必須となる可能性がある。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `name` | `string` | *(必須)* | パッケージ名 |
| `version` | `string` | *(必須)* | セマンティックバージョン |
| `edition` | `string` | *(必須)* | エディション（例: `"2024"`） |

```toml
[package]
name = "hello-pasta"
version = "1.0.0"
edition = "2024"
```

---

## [lua]（Lua ライブラリ）★ 上級者向け

Lua ランタイムにロードするライブラリを選択する。詳細は `pasta-lua-coding` スキルを参照。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `libs` | `string[]` | `["std_all","assertions","testing","regex","json","yaml"]` | ロードする Lua ライブラリ |

```toml
[lua]
libs = ["std_all", "json", "yaml"]
```
