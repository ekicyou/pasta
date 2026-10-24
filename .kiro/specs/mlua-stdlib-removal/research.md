# ギャップ分析: mlua-stdlib-removal

2026-10-24（main `c20ad104` の後の `55da9722`）に、requirements.md と今のコードを突き合わせた。

## 1. 今あるもの

### 撤去する側（mlua-stdlib）

| 場所 | 今の中身 |
| ---- | -------- |
| ルートの `Cargo.toml` 34 行 | `mlua-stdlib = { version = "0.1", features = ["json", "regex", "yaml"] }` |
| `crates/pasta_lua/Cargo.toml` 23 行 | `mlua-stdlib.workspace = true` |
| `Cargo.lock` | `mlua-stdlib` 0.1.0 の依存は `mlua`・`ouroboros`・`owo-colors`・`quick_cache`・`regex`・`serde`・`serde_json`・`serde_yaml`。`regex`・`serde`・`serde_json` は pasta が直に使うので残る。ほかは依存を外した後のビルドで `Cargo.lock` から落ちる（手で消さない） |
| `src/runtime/mod.rs` 214〜232 行 | `should_enable_module` を見て 6 つを `mlua_stdlib::*::register` で登録する。消す行はここだけ |
| `src/runtime/runtime_config.rs` | `default_libs()`（6 つの名前を含む。`lib.rs` 64 行と `loader/mod.rs` 48 行が公開の再エクスポート）、`full()`（`env` ほかを含む）、`libs` の文書、`should_enable_module`（使うのは `mod.rs` の 6 行だけ）、`validate_and_warn` の `env` の警告（390〜396 行）。`to_stdlib` は `std_` で始まらない名前を黙って飛ばす（284 行）ので、名前を消しても `to_stdlib` の働きは変わらない |
| `src/error.rs` 128 行 | `UnknownLibrary` の文が、選べるライブラリに `assertions, testing, env, regex, json, yaml` を並べる。試験 288〜289 行が `regex`・`yaml` を含むことを見ている |
| `pasta_scripts/pasta/shiori/event/external_control.lua` | pasta 自身のスクリプトで 6 つを使うのは、ここの `pcall(require, "@json")` だけ |

### 足す側（`@toml`）

- `toml` クレートは `pasta_lua` の直接の依存。`module_registry.rs` 216 行の `PastaLuaRuntime::toml_to_lua`（私的な関連関数）が、`@pasta_config` のために TOML の値を Lua の値にする。整数は `f64` にし（`*i as f64`）、日付と時刻は `to_string()` の文字列にする。要件 2.3 の「`@pasta_config` と同じ規則」は、この関数を呼べばそのまま満たせる。
- モジュールの登録は `set_loaded_module(lua, name, table)`（`module_registry.rs` 24 行）の型が揃っている。`@pasta_log` は `RuntimeConfig.libs` によらず常に登録される（`mod.rs` 235 行）。
- 公開モジュールの共通の決まり（マニュアル `lua/modules/index.md` の「失敗の返し方」）は、戻り値で失敗を返す関数は `nil, エラーメッセージ`、引数の型が合わない呼び出しは Lua のエラー（`@enc` は例外）。要件 2.4・2.7 はこれに合わせた。
- 「モジュールのメタデータ」（`_VERSION`・`_DESCRIPTION`）を持つのは 4 つのモジュールだけで、持たないもの（`@pasta_search`・`@pasta_config`）もある。`@toml` が持つかは要件にしていない（設計で決める）。

### 改める側（`OnPastaSetSaveVariable`）

- `external_control.lua`（145 行）の `read_json` と、`OnPastaSetSaveVariable` の本体（110〜145 行）が書き換えの範囲。門（`refused`）・`bad_reference0`・`fail`・`quoted`・`is_name` はそのまま使える。
- 失敗の種類 `pasta/failure.lua` 12〜25 行は 12 種。`NOT_JSON = "JSON として読めない値"` と `NOT_ASSIGNABLE = "代入できない値"` を、今は `OnPastaSetSaveVariable` だけが使う。新しい形では表（配列・オブジェクト）が届かないので、`NOT_ASSIGNABLE` の今の使い道（「表」）は無くなる。`NOT_NUMBER = "数にできない値"` は `act.lua` 594 行・`scene_play.lua` 96 行が使っている。
- 数にする先例は 2 つある。`act.lua` 579〜589 行は `tonumber` の範囲（「変更前の暗黙変換と同じ範囲」）、`scene_play.lua` 92 行は `^%d+$` に絞る（全角・前後の空白・小数・符号を数にしない）。
- `failure.value_of`・`FAILURE.kind_of`（値なし・空文字列）で、要件 4.5 の表記の中身はそろう。

### 試験

| 場所 | 中身 | この spec での扱い |
| ---- | ---- | ------------------ |
| `tests/runtime/stdlib_modules_test.rs`（310 行） | `@assertions`・`@testing`・`@env` の試験と、`RuntimeConfig::minimal`・`full` の試験 | 消す。`minimal`・`full` の試験のうち 6 つに関わらない部分を残すかは設計で見る |
| `tests/runtime/stdlib_regex_test.rs`（278 行） | `@regex` の試験 | 消す |
| `tests/runtime/main.rs` 26〜27 行 | 上の 2 つの `mod` | 外す |
| `tests/runtime/unit_test.rs` | 149 行の `test_to_stdlib_ignores_mlua_stdlib_modules` だけでなく、`default_libs` の中身（18〜20 行）・`should_enable_module` の試験など、6 つの名前に触れる行が約 29 ある | 書き直すか消す |
| `tests/shiori/res_test.rs` 49 行 | `@assertions` を 1 か所で使う | 素の `assert` か `return` の値で確かめる形に書き直す |
| `tests/lua_unittest_runner.rs` 68 行 | 単体テストの土台に本物の `@json` を登録する | 外す |
| `tests/lua_specs/external_control_event_test.lua` | 冒頭の「土台の `@json`」の試験（7〜11 行）と、`OnPastaSetSaveVariable` の JSON の試験（318〜355 行ほか） | 新しい形の試験に書き直す |
| `tests/lua_specs/failure_test.lua` 344〜352 行 | 種類が 12 種であること、`NOT_JSON` の文言 | 置き換えた種類の文言に直す。数は 12 のまま |
| `tests/external_control_events_test.rs` 210〜233 行 | Rust の通しの試験で `&["会員番号", "\"01\""]` を送り、`"01"` が入ることを見る | **brief に無い**。新しい形（`&["会員番号", "01"]`）に直す |
| `tests/loader/config_test.rs` 396〜436 行 | 撤去済みの `[lua] libs = ["std_all", "env"]` で `@env` が使えないこと | 触らない（ウェーブの約束）。モジュールが無くなっても `pcall(require, "@env")` は `false` のままで通る |
| `tests/runtime/debug_integration_test.rs` 595 行 | `RuntimeConfig::full().debug.enabled` を見るだけ | `full()` を残せば触らない |

### 文書

- マニュアル: brief の 10 頁のとおり。加えて次を確かめた。
  - `lua/index.md` 30 行は `modules/mlua-stdlib.md#lua-標準ライブラリ` へリンクしている。章を置き換えるとき、Lua の標準ライブラリの節の置き場と、このリンクの行き先を一緒に決める（要件 5.3）。
  - `grammar/variables.md` 50・402 行と `internals/shiori.md` 103 行は `OnPastaSetSaveVariable` の節へのリンクだけで、JSON に触れていない。節の見出し（`OnPastaSetSaveVariable（グローバル変数に代入）`）を変えなければ、直さなくてよい。
  - `shiori-events.md` の `OnPastaSetSaveVariable` の節は 819〜895 行。既定ハンドラーの表の行は 131 行。
  - mlua-stdlib の章は、ほかの章と同じく冒頭と末尾にクローディアたちの会話がある。置き換える `@toml` の章も、会話の検査（`talk/talk-test.mjs`）を通す形にする。
- `book/tools/gen-skill-refs.mjs` 59 行と `gen-skill-refs-test.mjs` 98 行は章のファイル名を持つ。数（24）は置き換えなら変わらない。`--check` は、生成の印を 1 行目に持つ対応表の外のファイルを「孤立」として落とす（`checkAll` の `orphans`）。古い `references/mlua-stdlib.md` は生成の印を持つので、消さなければ `--check` が落ちる。
- スキル `pasta-lua-coding`: `SKILL.md` の 53・76・114〜123・158・160 行。references の `mlua-stdlib.md`（消す）・`modules-index.md`・`pasta-log.md`・`shiori-events.md`（作り直す）。`testing-lint.md`（手書き）は 6 つに触れていない。`pasta-ghost-authoring/references/variables.md` は `grammar/variables.md` の写しで、あちらを直さなければ変わらない。
- `crates/pasta_lua/README.md` 56・85 行、`tech.md` 34 行。
- 過去のリリースノート `doc/release-notes/v0.4.2.md` は「JSON の値を代入」と書いているが、出した版の記録なので直さない。
- 第三者のライセンスの一覧 `THIRD_PARTY_LICENSES.txt` は git で追っておらず、リリースのときに作り直されるので、要件 1.2 は手を入れずに満たされる（`about.toml` も直さなくてよい見込み）。

## 2. 要件と資産の対応

| 要件 | 使える資産 | ギャップ |
| ---- | ---------- | -------- |
| 1.1〜1.3 6 つを無くす | `mod.rs` の 6 行、依存の 2 行 | Constraint: `Cargo.lock` の席はこの spec が持つ（同じウェーブで唯一） |
| 1.4 標準ライブラリを変えない | `to_stdlib`・`parse_std_lib` | なし（触らない） |
| 1.5 名前を選べるライブラリとして扱わない | `to_stdlib` は `std_` 以外を飛ばす | Unknown: 6 つの名前（と、ほかの `std_` でない名前）を、黙って飛ばすか誤りにするか（下の Research Needed 1） |
| 1.6 誤りの文 | `error.rs` 128 行 | 文から 6 つを外し、試験 288〜289 行を直す |
| 1.7 `env` の警告 | `validate_and_warn` | `env` の分岐を消す。`should_enable_module` は使う所が無くなる |
| 2.1 既定で有効 | `set_loaded_module`、`@pasta_log` の常に登録する型 | Missing: `@toml` の登録。`libs` で切れるようにするかは Research Needed 2 |
| 2.2〜2.3 decode と値の対応 | `toml_to_lua` | Missing: 文字列を `toml::Table` に読む口（`str::parse::<toml::Table>` か `toml::from_str`） |
| 2.4 失敗は `nil, エラーメッセージ` | `toml::de::Error` の `Display`（行と列と理由） | Missing |
| 2.5 `encode` を持たない | — | なし（作らない） |
| 2.7 文字列でない引数は Lua のエラー | mlua の引数の型の変換（`String` を受ける関数なら自然に Lua のエラー） | なし |
| 3.1〜3.3 文字列 | `act.req.reference[1]` | Missing: `read_json` の置き換え |
| 3.4 number | `tonumber`・`^%d+$` の先例 | Unknown: 受ける数の書き方（Research Needed 3） |
| 3.5 boolean | — | Missing（`"true"`・`"false"` の完全一致） |
| 3.6 delete | 今の `null` の消し方（`save[name] = nil`） | 書き換えのみ |
| 3.7 型の名前の見分け | — | Missing |
| 4.1〜4.5 失敗 | `fail`・`FAILURE.report`・`value_of`・`kind_of` | Constraint: 失敗の種類を増やさない。`NOT_JSON` の置き換えの名前と、どの失敗にどの種類を当てるか（Research Needed 4） |
| 4.6〜4.8 出方・既存の断り | 今のまま | なし |
| 5.1〜5.3 章の置き換え | mlua-stdlib の章（135 行）・`SUMMARY.md` | Constraint: 章の数 63 を変えない。Lua の標準ライブラリの節の置き場 |
| 5.4 イベントの節 | `shiori-events.md` 819〜895 行・131 行 | Constraint: 131 行の表は `recommend-menu-scenes` と隣の行 |
| 5.6 スキル | `gen-skill-refs.mjs` | Constraint: `.claude/skills/**` の編集は分類器が Self-Modification として止めることがある。止まったら開発者の許可を求める |
| 5.8 検査 | `verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`・`link-check.mjs`・`gen-skill-refs --check` | なし（直した後に全部回す） |
| 6.4〜6.5 試験 | 上の表 | brief に無かった `tests/external_control_events_test.rs` を含める |

## 3. 進め方の案

### 案 A: 今の部品を広げる（推奨に近い）

- `@toml` を `module_registry.rs` に `register_toml_module` として足し、`toml_to_lua` を呼ぶ。`mod.rs` の 6 行を、その呼び出し 1 行に置き換える。
- `external_control.lua` の `read_json` を、型の名前で分ける小さな関数に置き換える。新しいファイルは作らない。
- `runtime_config.rs` は、6 つの名前と `should_enable_module`・`env` の警告を消す。`default_libs()` は `["std_all"]` になり、`new()` と `minimal()` が同じ中身になる。
- 良い点: 新しいファイルが Lua の試験とマニュアルの章だけで済む。既にある型（`set_loaded_module`・`fail`）に乗る。
- 気になる点: `default_libs`・`full`・`minimal` が公開の API なので、`pasta_lua` を Rust から使う人には中身が変わる（壊れはしない）。

### 案 B: `@toml` を別のファイルに分ける

- `runtime/toml_module.rs` のような新しいファイルに `decode` を置き、`toml_to_lua` をそちらへ移すか、共有にする。
- 良い点: `module_registry.rs`（239 行）が膨らまない。
- 気になる点: `toml_to_lua` を動かすと `@pasta_config` に触る。`module_registry.rs` は同じウェーブで `toml-serialization` が触らない約束なので、こちらが動かしてもぶつからないが、差分が大きくなる。ponytail の考えでは、関数 1 つと登録 1 つのために新しいファイルは要らない。

### 案 C: 組み合わせ

- Rust の側は案 A、`RuntimeConfig` は名前の扱いだけを変えて `should_enable_module` を将来の部品のために残す。
- 気になる点: 使う所の無い関数を残すことになる（ponytail の「後のためのコード」）。

## 4. 規模と危うさ

- **規模: M（3〜7 日）**。Rust の撤去と `@toml` は小さいが、マニュアル 10 頁・スキル・references の作り直し・試験 9 ファイルの書き直しが重い。brief の 9〜11 タスクは妥当。
- **危うさ: Low〜Medium**。技術はどれも手元にある（`toml`・`toml_to_lua`・失敗の仕組み）。危うさは、作者に見える API を消すことと、0.4.2 で出したイベントの形を互換無しに変えることで、知らせ方（コミットの件名の `!`）で扱う。スキルの編集が分類器に止められうるのが、進み方の上の危うさ。

## 5. 設計への申し送り

### 推奨

- 案 A。`@toml` は `@pasta_log` と同じく、`RuntimeConfig.libs` によらず常に登録する（要件 2.1「設定なしで」。`libs` に名前を足すと、無くした「モジュール名で入り切りする」口をまた作ることになる）。
- `OnPastaSetSaveVariable` は、`delete` を先に見て Reference1 を見ない → 型の名前を確かめる → Reference1 の有無 → 型ごとの読み、の順にすると、要件 3.6・4.3・4.4 の優先がはっきりする。
- マニュアルは、`lua/modules/mlua-stdlib.md` を `lua/modules/toml.md`（名前は設計で決める）に置き換え、Lua の標準ライブラリの節は、公開モジュールの一覧の頁（`lua/modules/index.md` の共通事項）へ移すのが、章の中身に合う。`lua/index.md` 30 行のリンクを一緒に直す。
- コミットは、撤去と `@toml`、`OnPastaSetSaveVariable`、文書を分け、作者に見える変更の 2 つに `!` を付ける。

### Research Needed

1. **`RuntimeConfig` に 6 つの名前が並んだとき**。今の `to_stdlib` は `std_` で始まらない名前を黙って飛ばす。6 つの名前を誤りにすると、Rust から `pasta_lua` を使う人が気づけるが、`ConfigError` の新しい場合を作ることになる。推奨は、今のまま黙って飛ばす（`std_` でない知らない名前も今そうしている）。
2. **`@toml` を `libs` で切れるようにするか**。推奨は、切れない（上の推奨）。
3. **`number` で受ける数の書き方**。LuaJIT の `tonumber` は、前後の空白、`0x10`、`1e3` を数にし、`inf`・`nan` も数にしうる。`inf`・`nan` がセーブに入ると、`toml-serialization` の TOML の書き出しで扱いが要る。先例は 2 つある（`act.lua` の `tonumber` の範囲と、`scene_play.lua` の `^%d+$`）。**決定（2026-10-24 要件の話し合い）**: Lua の `tonumber` で数にし、`inf`・`nan`（結果が無限大か非数）だけを断る。実行中に届いた値を数にする所（`ACT.num`・`clock.lua`・`mouse.lua`・`callback.lua`・`scene_play.lua`）はどれも Lua の `tonumber` で、Rust の変換は書いた文を読むとき（DSL の数・SHIORI の見出し・pasta.toml）だけという今の線に合わせる。Rust の変換を Lua に公開する案は、5 か所と DSL の算術の動きを変えるので採らない（そろえたくなったら別の spec）。`lj_strscan.c` で、前後の空白・`inf`/`infinity`/`nan`・`0x` を受けることを確かめた。
4. **失敗の種類の割り当て**。**決定（2026-10-24）**: `number` の失敗は既存の `NOT_NUMBER`（数にできない値）、`boolean` の失敗と Reference1 が無いときは `NOT_ASSIGNABLE`（代入できない値）、知らない型の名前は `NOT_JSON` を置き換えた「型の名前にならない値」。`failure_test.lua` の数は 12 のまま。`recommend-menu-scenes` は種類を足さない約束なので、重ならない。
   - 空の Reference2 は型を省いたのと同じ（文字列）。空の Reference1 は空の文字列を代入し、無い Reference1 は断る（同じ日の決定）。
5. **`toml::de::Error` の文の形**。`Display` は複数行（位置と、問題の行の抜き出し）になる。失敗の表記やログに載せるときに 1 行にするかは、`@toml` を使う作者に任せる（`decode` は文をそのまま返す）のが簡単。
6. **大きな整数**。`toml_to_lua` は整数を `f64` にするので、2^53 を超える整数は丸まる。`@pasta_config` と同じ規則（要件 2.3）なので、そのままにしてマニュアルに一言書くかを設計で決める。

## 6. 設計の調べ（2026-10-24・`kiro-spec-design`）

### 要約

- **種別**: 既存の仕組みの拡張（Extension）。軽い調べ（`design-discovery-light`）で足りる。新しい外部の依存は無く、`toml` クレート（Cargo.lock では 1.1.8）・`toml_to_lua`・`set_loaded_module`・失敗の仕組み（`FAILURE.report`・`kind_of`）がすべて手元にある。
- **使ったスキルと手引き**: `ponytail`（いちばん小さい完全な変更・後のためのコードを作らない）、`pasta-lua-coding`（`SKILL.md` の 53・76・114〜123・158・160・174 行を確かめた。公開モジュールの規範の正はマニュアルの章）、`kiro-spec-design` の `design-principles`・`design-discovery-light`・`design-synthesis`・`design-review-gate`。
- **大事な発見**:
  - ギャップ分析の一覧に無い、消すテストの名前を持つ文書が 2 つある。`.kiro/steering/structure.md` 155〜156 行（`stdlib_modules_test.rs`・`stdlib_regex_test.rs` の木）と、ルートの `TEST_COVERAGE.md` 90〜91 行。
  - スキル `pasta-lua-coding` の `SKILL.md` 174 行も「グローバル変数に JSON の値を代入する」と書いている（ギャップ分析の行の一覧に無い）。
  - リリースノートの「動きが変わる変更」は、main のコミットの件名の `!` から作る（`.github/scripts/release/release-notes.ps1` 104〜109 行）。PR は squash でマージするので、main に残る件名は PR の題名 1 つだけになる。ギャップ分析の「コミットを分けて 2 つに `!`」は main に残らないので、PR の題名に `!` を付け、2 つの変更を両方書く。
  - 移す「Lua 標準ライブラリ」の節は「ゴーストから変える手段は無い」と書いているが、`[loader] ffi = true` で `ffi` を開ける（`reference/pasta-toml.md` 249 行）。移すときに、この 1 点を今の姿に直す。

### 調べたこと

#### `@toml` の置き場と読み方

- **きっかけ**: 要件 2.1〜2.7。
- **見たもの**: `runtime/mod.rs` 214〜235 行（6 つの登録と、`@pasta_log` を常に登録する行）、`runtime/module_registry.rs`（`set_loaded_module` 24 行・`register_log_module`・`toml_to_lua` 216 行）、`loader/config/mod.rs` 71 行（`toml::from_str::<toml::Table>` で文字列を読む先例）。
- **分かったこと**:
  - `with_config_and_source_map` は、`PastaLuaRuntime::new`・`with_config`・`from_loader` のどれからも通る共通の道で、`main.lua` を読む前に走る。`@pasta_log` の隣に 1 行足せば、要件 2.1 の時機がそろう。
  - `toml_to_lua` は `PastaLuaRuntime` の私的な関連関数で、`module_registry.rs` の中から `Self::toml_to_lua` で呼べる。見える範囲を変えずに済む。
  - mlua 0.11 の `String`（Rust の `String` と `mlua::String`）は、引数の数を文字列へ変える（`coerce_string`）。`toml.decode(42)` を Lua のエラーにする（要件 2.7）には、引数を `Value` で受けて、文字列でなければエラーにする。
  - `toml::de::Error` の `Display` は複数行（位置・問題の行・理由）になる。
- **影響**: 新しい Rust のファイルは要らない。`module_registry.rs` に関数 1 つ、`mod.rs` は 18 行を消して 1 行を足す。

#### `RuntimeConfig` に残るもの

- **見たもの**: `runtime_config.rs` 全体、`should_enable_module` と `default_libs`・`minimal`・`full` の呼び出し元（ワークスペースを検索）。
- **分かったこと**:
  - `should_enable_module` を呼ぶのは `mod.rs` の 6 行と `tests/runtime/unit_test.rs` だけ。`pasta_shiori`・`pasta_check` は呼ばない。
  - `RuntimeConfig::minimal()` はテスト 4 ファイル（`log/integration_test.rs`・`debug_integration_test.rs`・`debug_integration_zero_cost_test.rs`・`encoding_test.rs`）が使う。`full()` は `debug_integration_test.rs` 595 行が `debug.enabled` を見るだけ。
  - `to_stdlib` は `std_` で始まらない名前を黙って飛ばす。`from_libs` に 6 つの名前を渡す呼び出しは、テストのほかに無い（`README.md` 85〜95 行の例だけ）。
- **影響**: `should_enable_module` は消す。`new()` と `minimal()` は同じ中身（`["std_all"]`）になるが、どちらも公開の API で呼び出し元があるので残す。

#### 文書の行き先

- **見たもの**: `book/src/SUMMARY.md` 48 行、`gen-skill-refs.mjs` 59 行・`gen-skill-refs-test.mjs` 98 行、`outName`（`lua/modules/toml.md` → `references/toml.md`）、`lua/index.md` 30 行、`lua/modules/index.md` の共通事項、ほかの章のファイル名の付け方（`pasta-search.md`・`enc.md`。`@` を外したモジュールの名前）。
- **分かったこと**: 章のファイル名は `lua/modules/toml.md`、写しは `references/toml.md` になる。章の数は 63 のまま。リンクの行き先の見出し `### Lua 標準ライブラリ` を `lua/modules/index.md` へ移せば、アンカーは `#lua-標準ライブラリ` のままで、`lua/index.md` はファイル名だけを直せばよい。

## 7. 設計の決定

### 決定: `@toml` は `RuntimeConfig.libs` によらず常に登録する（申し送り 1・Research Needed 2）

- **きっかけ**: 要件 2.1「設定なしで」。
- **案**: 1. 常に登録する（`@pasta_log` と同じ）／2. `libs` に `toml` の名前を足して切れるようにする。
- **推奨・採用**: 1。
- **理由**: 2 は、この spec で無くす「モジュールの名前で入り切りする口」をまた作る。切りたい人がいない。
- **得失**: Rust から `pasta_lua` を使う人も `@toml` を外せない。読むだけの部品で、ファイルにも環境にも触れないので害が無い。

### 決定: `RuntimeConfig.libs` の 6 つの名前は黙って飛ばす（申し送り 2・Research Needed 1）

- **案**: 1. 今の `to_stdlib` のまま、`std_` で始まらない名前を黙って飛ばす／2. `std_` で始まらない名前を `ConfigError::UnknownLibrary` にする。
- **推奨・採用**: 1。
- **理由**: 要件 1.5 は「どの名前を並べても入らない」で、1 で満たせる。2 は `ConfigError` の新しい場合と試験を足し、今 `std_` でない名前を並べている呼び出し（`README.md` の例）を壊す。
- **得失**: Rust から 6 つの名前を並べた人は、誤りでなく黙って無視される。`pasta_lua` を Rust から使う人は今のところ pasta 自身だけである。

### 決定: マニュアルの章は `lua/modules/toml.md`、Lua 標準ライブラリの節は公開モジュールの一覧の頁へ（申し送り 3）

- **推奨・採用**: `lua/modules/mlua-stdlib.md` を消し、`lua/modules/toml.md`（題は `@toml`）を作る。`### Lua 標準ライブラリ` の節は `lua/modules/index.md` の「共通事項」の終わりへ移し、`lua/index.md` 30 行のリンクを `modules/index.md#lua-標準ライブラリ` に直す。
- **理由**: ほかの章と同じ名付け（`@` を外したモジュールの名前）。Lua 標準ライブラリはどのモジュールにも共通の前提で、一覧の頁の共通事項に合う。
- **ついでに直すこと**: 移す節の「ゴーストから変える手段は無い」を、「`ffi` だけは `pasta.toml` の `[loader] ffi = true` で開ける。ほかは変えられない」に直す（`reference/pasta-toml.md` の `ffi` へリンク）。今の文は、FFI を開く口を足した `lua-cmodule-robustness`（PR #150）の後に古くなっている。

### 決定: `@toml` はモジュールのメタデータ（`_VERSION`・`_DESCRIPTION`）を持たない（申し送り 4）

- **推奨・採用**: 持たない。
- **理由**: 要件 2.5「`decode` だけを持つ」。`@pasta_config`・`@pasta_search` も持たない。版の文字列を足しても使う人がいない。`lua/modules/index.md` のメタデータの節（4 つのモジュールを並べる）は変えない。

### 決定: 読めないときのエラーメッセージは `toml` クレートの文をそのまま返す（申し送り 5）

- **推奨・採用**: `toml::de::Error` の `Display` をそのまま 2 つ目の戻り値にする。1 行にしない。
- **理由**: 位置と問題の行を含む文が、作者がいちばん直しやすい。失敗の表記に載せるときは、`FAILURE.escape` が CR・LF を空白にする。章に「複数行になることがある」と書く。

### 決定: 2^53 を超える整数は `@pasta_config` と同じく丸まる。章に 1 行書く（申し送り 6）

- **推奨・採用**: `toml_to_lua` をそのまま使い、`@toml` の章の値の対応の表の下に「2^53（9007199254740992）を超える整数は、いちばん近い数に丸まる（`@pasta_config` と同じ）」と書く。
- **理由**: 要件 2.3 が `@pasta_config` と同じ規則を求める。LuaJIT の数は倍精度の浮動小数点数だけなので、丸めない方法が無い。

### 決定: `OnPastaSetSaveVariable` の確かめの順

- **採用**: 門（`SecurityLevel`）→ Reference0 → Reference2 の型の名前（無い・空は `string`。知らない名前は「型の名前にならない値」）→ `delete` なら Reference1 を見ずに消す → Reference1 が無ければ断る（`number` は「数にできない値」、ほかは「代入できない値」）→ 型ごとの読み。
- **理由**: 型の名前を先に確かめれば、`delete` も型の 1 つとして同じ分かれ道で扱え、要件 3.6・4.3・4.4 の優先がはっきりする。知らない型の名前と無い Reference1 が同時に来たときは、型の名前の誤りを出す（要件 4.4 は型が無い・空・`string` のときだけ）。

### 決定: 「型の名前にならない値」の鍵の名前は `BAD_TYPE`

- **採用**: `FAILURE.LABEL.NOT_JSON` を消し、同じ席に `BAD_TYPE = "型の名前にならない値"` を置く。
- **理由**: 文言の「〜にならない値」は既存の `BAD_NAME = "名前にならない値"` と同じ形で、鍵の名前も合わせる。数は 12 のまま。

### 決定: `!` は PR の題名に付ける

- **採用**: squash マージの件名（PR の題名）に `!` を付け、6 つのモジュールを外すことと `OnPastaSetSaveVariable` の形を変えることの 2 つを書く。途中のコミットにも付けてよいが、main に残るのは題名だけである。見どころのファイル（`doc/release-notes/v0.4.3.md`）は `release` スキルが書くので、この spec は `kiro-complete` の申し送りで 2 点を渡す。

## 8. まとめ直し（design-synthesis）

- **一般化**: `number`・`string`・`boolean`・`delete` の 4 つの読み方は、「型の名前 → Reference1 の読み方」の小さな対応表 1 つにまとまる。表を外へ公開したり、型を足す口を作ったりはしない（要る型が出てから）。
- **作るか借りるか**: TOML を読むのは `toml` クレート（借りる）、値の変換は既存の `toml_to_lua`（借りる）、数の読み方は Lua の `tonumber`（借りる）。新しく書くのは、`decode` の Lua 関数 1 つと、型で分ける Lua の関数 1 つだけ。JSON・YAML・正規表現の代わりは作らない（Out of scope）。
- **削ったもの**: `should_enable_module`（使う所が無くなる）、`env` の警告、`read_json`、`@json` の有無の確かめ（「JSON の部品が無い」の失敗）、配列・オブジェクトの失敗。新しい Rust のファイル・新しい `ConfigError`・`@toml` の切り替え・`encode`・メタデータは作らない。

## 9. 危うさと手当て

- スキル `pasta-lua-coding` の `SKILL.md` と `references/mlua-stdlib.md` の削除は、分類器が Self-Modification として止めることがある → 止まったら、入った分と残りを示して開発者の許可を求める（作業を止めて待つ）。`references/` の作り直しは `gen-skill-refs.mjs` が書くが、古い写しの削除は手で行う。
- cargo-deny の `unused-allowed-license` は既定で警告（失敗にならない）。依存が減って使われなくなる許可のライセンスがあっても、`deny.toml`・`about.toml` は触らない。
- `recommend-menu-scenes` と `lua/shiori-events.md` の既定ハンドラーの表を共有する → この spec は `OnPastaSetSaveVariable` の行の文だけを直す。行の数・表の前の「9 つ」には触らない。後から入る側が合わせる。
- `toml-serialization` と `tech.md`・マニュアルの頁を共有する → この spec は `tech.md` の mlua-stdlib の依存の行と、組み込みモジュールの表の `@env` の行（と `@toml` の行を足す）だけを直す。
