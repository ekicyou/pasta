# pasta.toml リファレンス

> Pasta ゴーストプロジェクトの設定ファイル `pasta.toml` 全セクション・全キーのリファレンス。
> 本リファレンスは **プロファイルモデル** に基づき、各セクション・各フィールドを3分類へ一意に整理する。

---

## 概要

`pasta.toml` はゴーストプロジェクトのルートに配置される設定ファイル。
ロード後、Rust 側ローダで **単一の補完ステップ** を通り、省略された項目は **SHIORI デフォルト（SSOT）** から補完される。明示した値は上書きされない。補完後の値は `@pasta_config` 経由で Lua からも同一の値として参照できる。

> `pasta.toml` ファイル自体は必須（不在は許容されない）。最小化の対象はあくまで「記述量」であり、ファイルの存在ではない。

### プロファイルと3分類

デフォルト表は用途ごとに **SHIORI プロファイル**（伺かゴースト動作）と **将来エンジンプロファイル**（ノベルゲーム／ツール等）の2系統を概念として持つ。本リファレンスが確定するのは SHIORI プロファイルのデフォルト値のみ。

各セクション・各フィールドは、次の **3分類のいずれか1つ** に一意分類される（重複しない）:

1. **SHIORI デフォルト有（省略可）** — 省略すると SSOT のデフォルト値が自動補完される。最小構成では書かなくてよい。
2. **必須（デフォルト不能）** — ゴースト固有でデフォルト化できないため、作者が必ず書く必要がある。現状は `[actor]`（1つ以上）のみ。
3. **エンジンプロファイル専用** — SHIORI 用途では適用・記述が不要。現状は `[package]` のみ。

---

## 3分類表

`pasta.toml` の全セクション・代表フィールドを3分類へマッピングする。「SHIORI デフォルト」列の値は **SSOT 由来**（Rust `crates/pasta_lua/src/loader/config.rs` の `Default` 実装・`default_*()` 関数）。

| セクション / キー | 分類 | SHIORI デフォルト |
|------------------|------|------------------|
| `[actor."名前"]`（1つ以上） | **必須（デフォルト不能）** | *(なし — 作者が必ず記述)* |
| `[actor]` › `spot` | 必須（デフォルト不能） | *(なし — ゴースト固有)* |
| `[actor]` › `budoux` / `default_surface` | SHIORI デフォルト有（省略可） | *(なし＝未設定)* |
| `[loader]` | SHIORI デフォルト有（省略可） | *(下記キー参照)* |
| `[loader]` › `pasta_patterns` | SHIORI デフォルト有 | `["dic/**/*.pasta"]` |
| `[loader]` › `lua_search_paths` | SHIORI デフォルト有 | *(下記参照)* |
| `[loader]` › `transpiled_output_dir` | SHIORI デフォルト有 | `"profile/pasta/cache/lua"` |
| `[loader]` › `debug_mode` | SHIORI デフォルト有 | `true` |
| `[ghost]` | SHIORI デフォルト有（省略可） | *(下記キー参照)* |
| `[ghost]` › `talk_interval_min` | SHIORI デフォルト有 | `180` |
| `[ghost]` › `talk_interval_max` | SHIORI デフォルト有 | `300` |
| `[ghost]` › `hour_margin` | SHIORI デフォルト有 | `30` |
| `[ghost]` › `spot_newlines` | SHIORI デフォルト有 | `1.5` |
| `[talk]` | SHIORI デフォルト有（省略可） | *(下記キー参照)* |
| `[persistence]` | SHIORI デフォルト有（省略可） | *(下記キー参照)* |
| `[logging]` | SHIORI デフォルト有（省略可） | *(下記キー参照)* |
| `[lua]` | SHIORI デフォルト有（省略可） | `["std_all","assertions","testing","regex","json","yaml"]` |
| `[debug]` | SHIORI デフォルト有（省略可） | `enabled=false` / `port=9276` |
| `[package]` | **エンジンプロファイル専用** | *(SHIORI では不要 — [予約注記](#package予約注記) 参照)* |

> 分類は一意。同一のセクション・フィールドが複数分類に重複して属することはない。

---

## 最小テンプレート

SHIORI として起動するために **必須なのは `[actor]` のみ**。他の全セクションは省略でき、SSOT デフォルトが自動補完される。`[package]`・`[loader]` を含む必要は **ない**。

```toml
# 最小構成: 必須の [actor] のみ。他は SHIORI デフォルトで補完される。
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

- `"名前"` は `descript.txt` の `sakura.name` / `kero.name` と一致させる。
- `spot` はゴースト固有でデフォルト化できないため、各アクターで必ず指定する（`0`=sakura 側 / `1`=kero 側）。
- 慣例的な dic 配置（`dic/**/*.pasta`）の辞書は、`[loader]` を書かなくても `pasta_patterns` の SHIORI デフォルトで読み込まれる。

> `[actor]` を1つも書かない場合でも起動は停止しないが、SHIORI として正しく動作しない可能性があるため、軽量な警告ログが1回出力される。

---

## フルリファレンステンプレート

全セクション・全フィールドを **分類・SHIORI デフォルト注記付き** で網羅したテンプレート。値はすべて SSOT 由来のデフォルトを示す（必要な項目だけ抜き出して使う想定）。

```toml
# ============================================================
# pasta.toml フルリファレンステンプレート
# 各行の注記 = 分類 / SHIORI デフォルト値（SSOT 由来）
# 「省略可」セクションは丸ごと削除しても SSOT デフォルトで補完される。
# ============================================================

# --- 必須（デフォルト不能）: 最小構成で唯一必須 ---
[actor."女の子"]
spot = 0                # 必須（デフォルト不能）: 0=sakura側 / 1=kero側
budoux = [10, 12]       # 省略可（SHIORI デフォルト有）: 未設定=自動改行なし
default_surface = 0     # 省略可（SHIORI デフォルト有）: 未設定=指定なし

[actor."男の子"]
spot = 1                # 必須（デフォルト不能）

# --- 省略可（SHIORI デフォルト有）: ファイル読み込み ---
[loader]
pasta_patterns = ["dic/**/*.pasta"]            # 既定 ["dic/**/*.pasta"]
lua_search_paths = [                            # 既定（優先順位順）:
  "profile/pasta/save/lua",                     #   ユーザー保存スクリプト
  "scripts",                                    #   ユーザーカスタムスクリプト
  "profile/pasta/pasta_scripts",                #   pasta 標準ランタイム
  "profile/pasta/cache/lua",                    #   トランスパイル済みキャッシュ
  "scriptlibs",                                 #   追加ライブラリ
]
transpiled_output_dir = "profile/pasta/cache/lua"  # 既定 "profile/pasta/cache/lua"
debug_mode = true                                  # 既定 true

# --- 省略可（SHIORI デフォルト有）: ゴースト動作 ---
[ghost]
talk_interval_min = 180   # 既定 180（秒）
talk_interval_max = 300   # 既定 300（秒）
hour_margin = 30          # 既定 30（秒）
spot_newlines = 1.5       # 既定 1.5

# --- 省略可（SHIORI デフォルト有）: トーク表示制御 ---
[talk]
script_wait_normal = 50     # 既定 50（ms）
script_wait_period = 1000   # 既定 1000（ms）
script_wait_comma = 500     # 既定 500（ms）
script_wait_strong = 500    # 既定 500（ms）
script_wait_leader = 200    # 既定 200（ms）
chars_period = "｡。．."      # 既定 "｡。．."
chars_comma = "、，,"        # 既定 "、，,"
chars_strong = "？！!?"      # 既定 "？！!?"
chars_leader = "･・‥…"       # 既定 "･・‥…"
# chars_line_start_prohibited / chars_line_end_prohibited も既定値あり（行頭・行末禁則）

# --- 省略可（SHIORI デフォルト有）: 永続化 ---
[persistence]
obfuscate = false                                   # 既定 false
file_path = "profile/pasta/save/save.json"          # 既定 "profile/pasta/save/save.json"
debug_mode = false                                  # 既定 false

# --- 省略可（SHIORI デフォルト有）: ログ出力 ---
[logging]
file_path = "profile/pasta/logs/pasta.log"   # 既定 "profile/pasta/logs/pasta.log"
rotation_days = 7                            # 既定 7
level = "info"                               # 既定 "info"
# filter = "debug,pasta_shiori=info"         # 未設定（設定時は level より優先）

# --- 省略可（SHIORI デフォルト有）: Lua ライブラリ ---
[lua]
libs = ["std_all", "assertions", "testing", "regex", "json", "yaml"]  # 既定

# --- 省略可（SHIORI デフォルト有）: デバッグバックエンド ---
[debug]
enabled = false             # 既定 false
port = 9276                 # 既定 9276
# present_as = "lua"        # 未設定（既定 .pasta）
source_map_sidecar = false  # 既定 false

# --- エンジンプロファイル専用: SHIORI 用途では記述不要（[package] 予約注記 参照） ---
# [package] は SHIORI では適用されない。記述しても無視される。
```

---

## [package] 予約注記
<a id="package予約注記"></a>

`[package]`（`name` / `version` / `edition`）は **エンジンプロファイル専用** に分類される。

- **SHIORI 用途では不要**: 伺かゴーストでは `install.txt` / `readme.txt` 等でメタデータを管理できるため、`[package]` を記述する必要はない。最小テンプレート・サンプルゴーストにも含めない。
- **記述しても無視される**: 既存ゴーストが `[package]` を含んでいても、エラーや警告を出さず従来どおり起動する（完全後方互換）。
- **将来仕様へ予約**: エンジンプロファイル（ノベルゲーム／ツール等の将来用途）における `[package]` のデフォルト値の確定・実装は **将来仕様へ予約** されており、本仕様では確定しない。

| キー | 型 | 説明 |
|------|-----|------|
| `name` | `string` | パッケージ名（エンジンプロファイル専用） |
| `version` | `string` | セマンティックバージョン（エンジンプロファイル専用） |
| `edition` | `string` | エディション（例: `"2024"`、エンジンプロファイル専用） |

---

## 各セクション詳細

ここから先は、各セクション・各キーのフィールド詳細リファレンス（型・デフォルト・用途）。

### [loader]（ファイル読み込み）

辞書ファイルと Lua モジュールの読み込みを制御する。**分類: SHIORI デフォルト有（省略可）**。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `pasta_patterns` | `string[]` | `["dic/**/*.pasta"]` | 読み込む `.pasta` ファイルの glob パターン |
| `lua_search_paths` | `string[]` | *(下記参照)* | Lua モジュール検索パス（優先順位順） |
| `transpiled_output_dir` | `string` | `"profile/pasta/cache/lua"` | トランスパイル済み Lua の出力先 |
| `debug_mode` | `bool` | `true` | デバッグモード（トランスパイル出力の保存等） |

#### pasta_patterns

`dic/` 配下の `.pasta` ファイルを再帰的に読み込む。既定の `["dic/**/*.pasta"]` は dic 直下・一階層・多階層をすべて網羅するため、慣例的な dic 配置であれば `[loader]` を省略してそのまま起動できる。

```toml
[loader]
pasta_patterns = ["dic/**/*.pasta"]
```

#### lua_search_paths

Lua の `require()` が検索するパスの一覧。既定値（優先順位順）:

1. `profile/pasta/save/lua` — ユーザー保存スクリプト（最優先）
2. `scripts` — ユーザーカスタムスクリプト
3. `profile/pasta/pasta_scripts` — pasta 標準ランタイム
4. `profile/pasta/cache/lua` — トランスパイル済みキャッシュ
5. `scriptlibs` — 追加ライブラリ

---

### [ghost]（ゴースト動作）

ゴーストの動作パラメータを設定する。カスタムフィールドとして Lua（`@pasta_config`）に透過される。**分類: SHIORI デフォルト有（省略可）**。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `talk_interval_min` | `integer` | `180` | ランダムトーク最小間隔（秒） |
| `talk_interval_max` | `integer` | `300` | ランダムトーク最大間隔（秒） |
| `hour_margin` | `integer` | `30` | OnHour 誤差許容秒数 |
| `spot_newlines` | `number` | `1.5` | スポット切替時の改行量（`\n[半角]` の倍率） |

#### talk_interval_min / talk_interval_max

ランダムトーク（OnTalk）の発火間隔を秒単位で指定する。SAVE テーブルの `pasta_talk_interval_min` / `pasta_talk_interval_max` で実行時に上書き可能（詳細は [variables.md](variables.md#永続化とsaveテーブル) を参照）。

```toml
[ghost]
talk_interval_min = 120
talk_interval_max = 240
```

#### spot_newlines

アクターのスポット（バルーン）が切り替わるときに挿入される改行量。値 `1.5` は `\n[half]`（1.5行分の改行）に相当する。

```toml
[ghost]
spot_newlines = 2.0
```

---

### [actor."名前"]（アクター設定）

アクターごとの設定。`"名前"` は `descript.txt` の `sakura.name` / `kero.name` と一致させる。複数アクターを定義可能。

**分類: 必須（デフォルト不能）** — 少なくとも1つの `[actor]` 定義が SHIORI 起動に必須。`spot` はゴースト固有でデフォルト化できない。`budoux` / `default_surface` はアクター内の省略可フィールド（未設定が既定）。

| キー | 型 | 既定値 | 分類 | 説明 |
|------|-----|--------|------|------|
| `spot` | `integer` | *(なし)* | 必須（デフォルト不能） | バルーン位置（0=sakura 側, 1=kero 側） |
| `budoux` | `integer[]` | *(なし)* | 省略可 | BudouX 自動改行幅 |
| `default_surface` | `integer` | *(なし)* | 省略可 | デフォルトサーフェス ID |

#### spot

バルーンの割り当てを制御する。`0` がメイン（sakura 側）、`1` がサブ（kero 側）。アクターごとに必ず指定する。

```toml
[actor."女の子"]
spot = 0

[actor."男の子"]
spot = 1
```

#### budoux

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

#### default_surface

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

### [talk]（トーク表示制御）

さくらスクリプト生成時のウェイト挿入と禁則処理を制御する。**分類: SHIORI デフォルト有（省略可）**。

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

### [persistence]（永続化）

SAVE テーブルの保存設定を制御する。**分類: SHIORI デフォルト有（省略可）**。

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

### [logging]（ログ出力）

ログファイル出力の設定を制御する。**分類: SHIORI デフォルト有（省略可）**。

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

### [lua]（Lua ライブラリ）★ 上級者向け

Lua ランタイムにロードするライブラリを選択する。**分類: SHIORI デフォルト有（省略可）**。詳細は `pasta-lua-coding` スキルを参照。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `libs` | `string[]` | `["std_all","assertions","testing","regex","json","yaml"]` | ロードする Lua ライブラリ |

```toml
[lua]
libs = ["std_all", "json", "yaml"]
```

---

### [debug]（デバッグバックエンド）★ 上級者向け

pasta_lua に組み込まれた DAP デバッグバックエンドを制御する。**分類: SHIORI デフォルト有（省略可）**。省略時はデバッグ OFF（本番経路はゼロコスト）。

| キー | 型 | 既定値 | 説明 |
|------|-----|--------|------|
| `enabled` | `bool` | `false` | デバッグバックエンドの有効化 |
| `port` | `integer` | `9276` | DAP リスナーがバインドする TCP ポート |
| `present_as` | `string` | *(なし＝`.pasta`)* | ソース表示モード（`"pasta"` / `"lua"`） |
| `source_map_sidecar` | `bool` | `false` | `.lua.map` サイドカーの追加出力 |

```toml
[debug]
enabled = true
port = 9276
present_as = "lua"
```

---

### [package]（パッケージ情報）★ エンジンプロファイル専用

**SHIORI 用途では記述不要**。分類・予約の詳細は [\[package\] 予約注記](#package予約注記) を参照。記述しても無視され、従来どおり起動する。
