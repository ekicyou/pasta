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
3. **`number` で受ける数の書き方**。LuaJIT の `tonumber` は、前後の空白、`0x10`、`1e3` を数にし、`inf`・`nan` も数にしうる。`inf`・`nan` がセーブに入ると、`toml-serialization` の TOML の書き出しで扱いが要る。先例は 2 つある（`act.lua` の `tonumber` の範囲と、`scene_play.lua` の `^%d+$`）。推奨は、10 進の整数と小数、先頭の `-` だけを受け（`^-?%d+$`・`^-?%d+%.%d+$` の形）、空白・全角・16 進・指数・`inf`・`nan` は断る。
4. **失敗の種類の割り当て**。種類の数 12 を保つ案は 2 つ。
   - 案 1: `NOT_JSON` を「知らない型」（例「型の名前にならない値」）に置き換え、値が型に合わない（`number`・`boolean`）と Reference1 が無いときは `NOT_ASSIGNABLE`（代入できない値）を使う。
   - 案 2: `number` の失敗に既存の `NOT_NUMBER`（数にできない値）を使い、`boolean` と Reference1 が無いときは `NOT_ASSIGNABLE`、知らない型は `NOT_JSON` を置き換えた種類にする。
   - どちらも `failure_test.lua` の数は 12 のまま。`recommend-menu-scenes` は種類を足さない約束なので、重ならない。
5. **`toml::de::Error` の文の形**。`Display` は複数行（位置と、問題の行の抜き出し）になる。失敗の表記やログに載せるときに 1 行にするかは、`@toml` を使う作者に任せる（`decode` は文をそのまま返す）のが簡単。
6. **大きな整数**。`toml_to_lua` は整数を `f64` にするので、2^53 を超える整数は丸まる。`@pasta_config` と同じ規則（要件 2.3）なので、そのままにしてマニュアルに一言書くかを設計で決める。
