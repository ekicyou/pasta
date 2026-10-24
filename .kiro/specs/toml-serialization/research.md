# ギャップ分析: toml-serialization

> 2026-10-24・ブランチ `claude/kiro-start-toml-serialization-cb6b21`（main `55da9722` の上）。要件は `requirements.md`（Requirement 1〜8、前提 A1〜A9）。依存の版は `Cargo.lock` の値で確かめた: mlua 0.11.6、toml 1.1.8（spec 1.1.0）、toml_writer 1.1.3、toml_parser 1.1.5、serde_json 1.0.154。ライブラリの動きは、レジストリのソース（`c:\rust\cargo\registry\src\index.crates.io-…\`）を読んで確かめた。cargo test は走らせていない。

## 1. 分析のまとめ

- **触るファイルは brief のとおりで、新しいファイルは無くて済む**。セーブ（`runtime/persistence.rs` 568 行・`runtime/lifecycle.rs` 63 行）、既定の保存先（`loader/config/sections.rs` 356 行の 60〜115 行）、ログの表（`runtime/log.rs` 241 行の 108〜129 行）、サイドカー（`debug/source_map/sidecar.rs` 145 行）。`toml` は既に `pasta_lua` の依存で、既定の機能（`parse`・`display`・`serde`）が入っている。`Cargo.toml`・`Cargo.lock` は触らずに済む。
- **今の mlua の値の変換（serde）に任せると、要件の線引きにならない**。既定の設定では、配列の並びと名前のキーの混ざった表は**名前のキーを黙って捨てる**（今の JSON のセーブでも起きている）。`nil` の穴・数のキー・関数は、TOML へ変える段で表全体の誤りになり、キーごとに外せない。要件の「そのキーだけ外して警告」（Requirement 4）には、pasta 自身の Lua → TOML の変換（深さの上限・循環の守り・キーの道筋つき）が要る。
- **ログの 1 行は、そのままでは守れない**。toml の値の書き出しは、改行を含む文字列を複数行の文字列（`"""…"""`）で書く。Requirement 6.2 のために、文字列を 1 行のエスケープで書く手当てが要る。
- **退避（`.broken`）には、終了時の保存まで届く「書かない」印が要る**。今の終了時の保存（`lifecycle.rs`）は、`load()` と別の道で `save_to_file` を直接呼ぶ。Windows の `fs::rename` は既にある退避ファイルを置き換えるので、Requirement 5.4 には名前の重なりの確かめが要る。
- 推奨は **Option A（今のファイルを広げる）**。規模は M、危うさは Medium（保存の形式は後戻りしにくい。ただし決まり事は開発者が決めてある）。

## 2. 今の状態（コードで確かめたこと）

### 2.1 セーブ

- `persistence.rs`
  - `register`（105〜160 行）: `effective_file_path()` を検査（絶対パス・`..`・ルート・ドライブ → `RuntimeError`）し、`load`・`save` の 2 つの関数を、`PersistenceState`（`file_path`・`obfuscate`・`debug_mode`。`Clone` で 2 つの閉包に別々に入る）つきで登録する。
  - `load_impl`（166〜196 行）: `load_from_file` の結果を `lua.to_value(&serde_json::Value)` で Lua にする。表でなければ警告して空の表。`NotFound` は警告して空の表。そのほかの誤り（壊れた JSON・展開できない gzip・PermissionDenied など）も警告して空の表で、**元のファイルはそのまま残るが、終了時の保存がその空の表で上書きする**（ponytail-lua-loader の Must fix 1）。
  - `save_impl`（202〜232 行）: `lua.from_value::<serde_json::Value>`（既定の設定）で変換し、失敗したら `(nil, "Failed to convert Lua value: …")`。1 つでも変換できない値があると、**ファイル全体を書かない**。
  - `load_from_file`（244〜262 行）: 空のファイル → 空のオブジェクト。先頭 `1f 8b` → gzip を展開して JSON、そうでなければ JSON。
  - `save_to_file`（276〜318 行）: 親フォルダを作り、`obfuscate` なら gzip した JSON（整形なし）、そうでなければ整形した JSON。`path.with_extension("tmp")` に書いて `sync_all`、`fs::rename` で置き換え、失敗したら一時ファイルを消す。
  - `DESCRIPTION = "Persistent data storage (JSON/gzip)"`（38 行）。マニュアル `pasta-persistence.md` 17 行と、スキルの references に写しがある。
  - `PersistenceError::JsonError(#[from] serde_json::Error)`（50〜52 行）。
  - `pub mod persistence`（`runtime/mod.rs` 31 行）で、`save_to_file`・`load_from_file`・`PersistenceError` は **crates.io に出している `pasta_lua` の公開の Rust API** である。型が `serde_json::Value` から変わると、公開の API が変わる（コミットに `!` を付ける理由が 1 つ増える）。
- `lifecycle.rs`: `Drop` → `save_persistence_data`。`PastaConfig` から保存先を計算し直し、`require("pasta.save")` の表を `from_value::<serde_json::Value>` で変換して `save_to_file`。誤りは `tracing::error!(…, "Failed to save persistence data on drop")`。`debug_mode` なら `"Saved persistence data on drop"`。
  - この 2 つの文言は、`pasta_shiori/src/shiori_lifecycle_tests.rs`（563 行あたり。保存先の親を通常のファイルにして保存を失敗させる）と `pasta_shiori/tests/ffi_logging_test.rs`（192 行）が、ログの順番を確かめるために探している。**文言を変えない**か、テストも直す。
- `pasta_scripts/pasta/save.lua`: 最初の `require` で `persistence.load()` を 1 回呼ぶだけ。Lua 側は形式を知らない。`scriptlibs/lua_test/mocks.lua` の `@pasta_persistence` の偽物も形式に触れない。

### 2.2 保存先の設定

- `sections.rs` 66〜115 行: `PersistenceConfig { obfuscate, file_path, debug_mode }`（`deny_unknown_fields` は無い。見本のゴーストの `data_dir` は黙って無視される）。既定 `"profile/pasta/save/save.json"`（96 行）。
- `effective_file_path`（103〜113 行）: `obfuscate` かつ末尾 `.json` → **`self.file_path.replace(".json", ".dat")`**。これは末尾だけでなく、**途中に `.json` を含むフォルダ名も替える**（例: `my.json.d/save.json` → `my.dat.d/save.dat`）。要件 3.3 で直す。`.dat` 以外 → 末尾に `.dat`。
- テスト: `tests/loader/config_test.rs` 325〜391 行（既定値と `effective_file_path` の 3 通り）、`tests/loader/config_sections_test.rs` 155〜168 行（`data.bin` → `data.bin.dat`）、`tests/runtime/persistence_integration_test.rs` 60〜84 行（同じ 3 通り）。

### 2.3 ログの表

- `log.rs` `table_to_string`（101〜130 行）: 要素数 > 1000 → `<table: N elements>`。`table_depth_exceeds`（明示のスタックで反復。キーの表もたどる。循環は深さが増え続けて止まる）で 10 段を超えたら `tostring`。そのあと `from_value_with::<serde_json::Value>(…, deny_recursive_tables(true))` → `serde_json::to_string`（1 行）。失敗したら `tostring`。
- テスト: `tests/log/module_test.rs` の `test_log_array_table_as_json`（`[1,2,3]` を探す。TOML では `[1, 2, 3]` になる）、`test_log_table_as_json`・`test_log_nested_table_json`（キーの名前を探すだけ）、`test_log_circular_reference_no_error`、`test_log_large_table_abbreviated`。

### 2.4 サイドカー

- `sidecar.rs`: `SidecarFile { version: u32, pasta_file: String, pairs: Vec<[u32; 2]> }`（`Serialize`/`Deserialize` の derive）。`write_sidecar` は `serde_json::to_vec` → `<lua_path>.map`、`read_sidecar` は `serde_json::from_slice`。呼ぶのは `loader/source_map_build.rs` 150 行（書く）だけで、読むのはテストだけ（`internals/debug.md` 349 行も同じことを書いている）。VSCode 拡張・DAP はサイドカーを読まない（`editors/` に参照なし）。
- テスト: `debug/source_map_sidecar_tests.rs`（49・155・196・221 行で `serde_json` を直接使う）、`tests/loader_source_map_build_test.rs`（`read_sidecar` 経由の往復だけ。形式に依らない）。
- `SidecarFile` はそのまま toml で書ける（根がキーのある表で、`pairs = [[1, 2], [3, 4]]` は TOML 1.0 以降の入れ子の配列）。空の `pairs = []` も書けて読める。

### 2.5 文書

| 場所 | 今の記述 |
| ---- | ---- |
| `book/src/lua/modules/pasta-persistence.md` | 11・17・26・57・82・89・111・118・123 行（JSON・`save.json`・`.json`→`.dat`・変換できない値で保存全体が失敗） |
| `book/src/reference/pasta-toml.md` | 167 行（全体の例）、713〜722 行（`[persistence]` の節） |
| `book/src/grammar/variables.md` | 318〜320 行 |
| `book/src/lua/modules/pasta-log.md` | 104・110 行 |
| `book/src/internals/logging-encoding.md` | 196〜201 行 |
| `book/src/internals/execution-model.md` | 440・441・458 行 |
| `book/src/internals/debug.md` | 349 行（449 行の `serde_json::Value` は DAP なので残す） |
| `book/src/debug/source-level.md` | 146 行 |
| `.kiro/steering/tech.md` | 37・38 行（`toml` は「設定ファイル管理」、`serde_json` は「シリアライゼーション」） |
| スキルの references（生成物） | `book/tools/gen-skill-refs.mjs` 47・50・54・58 行が、`variables.md`・`pasta-toml.md`（→ `pasta-ghost-authoring`）、`pasta-persistence.md`・`pasta-log.md`（→ `pasta-lua-coding`）を写す |
| `.claude/skills/pasta-ghost-authoring/SKILL.md` 175 行 | 「JSON ファイルに保存」（生成物でない。手で直す。分類器が止めたら開発者の許可を求める） |

マニュアルの記述の誤りも 1 つ見つけた: `pasta-persistence.md` 82 行は「文字列・数値以外のキー（真偽値など）を持つ表」で保存が失敗すると書くが、実際は**配列でない表の数のキー（`{[5] = "x"}`）でも失敗する**（2.6 の表）。書き直しで消える。

### 2.6 Lua の値と、今の変換・TOML の変換（ライブラリのソースで確かめた）

mlua 0.11.6 の値の変換の既定（`serde/de.rs` 79〜90 行の `Options::new()`）: `deny_unsupported_types = true`、`deny_recursive_tables = true`、`sort_keys = false`、`encode_empty_tables_as_array = false`、`detect_mixed_tables = false`。表を配列とみなすかは `Table::encode_as_array`（`table.rs` 798〜818 行）で、`detect_mixed_tables = false` なら **`raw_len() > 0` か配列の印のメタテーブルがあれば配列**。配列として読むときは `sequence_values` を 1 から順にたどり（`table.rs` 1219〜1236 行。長さの内側の `nil` は `nil` として出し、長さの外は `nil` で止まる）、**ほかのキーは見ない**。LuaJIT の数は `n.to_bits() == (i as f64).to_bits()` のとき整数（`state/raw.rs` 805〜813 行。`1.0` は整数 `1`、`-0.0` は浮動小数点数）。

toml 1.1.8 の `toml::Value` の読み取り（`value.rs` 420〜517 行）は `visit_unit`・`visit_none`・`visit_bytes` を持たず、表のキーは `String` だけを受ける。書き出しは `serialize_none` で `unsupported None value`、キーが文字列でないと `key_not_string`。文書（`toml::to_string`）の根は表でなければならない。`toml::Table` は既定（`preserve_order` 無し）で `BTreeMap` なので、キーは並べ替えられて毎回同じ順になる。

| Lua の値 | 今（JSON・mlua の既定） | mlua の既定で `toml::Value` へ | 要件（Requirement 4） |
| ---- | ---- | ---- | ---- |
| `{a = 1, b = "x"}` | `{"a":1,"b":"x"}` | 表 | 書ける |
| `{1, 2, 3}` | `[1,2,3]` | 配列 | 書ける |
| `{}` | `{}`（空の表） | 空の表 | 書ける（空の表） |
| 読み込んだ空の配列 | `[]`（印のメタテーブルで配列のまま） | 同じ | 書ける（空の配列のまま） |
| `{1, nil, 3}` | `[1,null,3]`（読み戻すと穴） | `nil` で誤り（表全体） | 書けない（そのキーを外す） |
| `{1, 2, x = 3}` | **`[1,2]`（`x` を黙って捨てる）** | `[1, 2]`（同じく捨てる） | 書けない（そのキーを外す） |
| `{[5] = "x"}` | **誤り（保存全体が失敗）** | 誤り（キーが文字列でない） | 書けない |
| `{[true] = 1}` | 誤り | 誤り | 書けない |
| 関数・コルーチン・ユーザーデータ | 誤り（`unsupported value type`） | 誤り | 書けない |
| 循環 | 誤り（`recursive table detected`） | 誤り | 書けない |
| 10 万段の入れ子 | **Rust の再帰でスタックが溢れうる**（保存の道に深さの関門が無い） | 同じ | 上限を超えたら書けない（落ちない） |
| `0/0`・`1/0` | `null`（読み戻すと `nil`。値が変わる） | `nan`・`inf` | 書ける（TOML は `nan`・`inf` を持つ） |
| `1.0`・`1.5` | `1`・`1.5` | 整数 `1`・浮動小数点数 `1.5` | 同じ |
| UTF-8 でない文字列 | 誤り（`visit_bytes` を受けない） | 誤り | 書けない |
| `save[1] = "x"`（根が配列に見える） | **根が `["x"]` になり、読み戻すと「表でない」で空の表** | 文書の根が配列で誤り | 根は表のまま、そのキーだけ外す（4.10） |

ここから分かること:

- 今の JSON のセーブにも、黙って値を失う道が 2 つある（混ざった表の名前のキー、`nan`・`inf`）。要件はこれを「外して警告」にそろえる。
- 「そのキーだけ外す」は、mlua の既定の変換を 1 回呼ぶ形では作れない。`detect_mixed_tables(true)` にすると混ざった表は表として読まれ、数のキーで誤りになるので「書けない」は分かるが、**どのキーかは誤りの文からしか分からず、外したあとで続けられない**。
- 根のキーごとに `from_value` を呼ぶ形（Option C）なら、根のキー単位では外せるが、`flags.secret` のような深いキーの単位では外せない（前提 A1）。深さの関門も別に要る。

### 2.7 読み込み（TOML → Lua）

- `toml_to_lua`（`module_registry.rs` 216〜238 行）は `PastaLuaRuntime` の非公開の関数で、`persistence.rs` からは呼べない。今のウェーブの約束で `module_registry.rs` に触らない（`mlua-stdlib-removal` が `@toml` を足す）。
- mlua の `lua.to_value(&toml::Value)` を使うと、TOML の日時は toml_datetime の決まりで `{ ["$__toml_private_datetime"] = "…" }` という表になる（Requirement 1.12 に合わない）。配列には印のメタテーブルが付く（空の配列が配列のまま戻る。Requirement 4.1 の「空の配列のまま」に合う。今の JSON の読み込みも同じ）。
- どちらにしても、`persistence.rs` の中に、日時を文字列にする小さな TOML → Lua の変換を持つのが素直（`toml_to_lua` の写しに近い形。配列の印を付けるかは設計で決める）。
- BOM: toml_parser 1.1.5 の字句解析（`lexer/mod.rs` 35 行）が先頭の UTF-8 BOM を飛ばす。Requirement 1.11 はそのまま満たせる。
- 空のファイル: TOML では空の文書は空の表として正しい。今の「空なら空」の特別扱いは要らなくなる（あっても害は無い）。

### 2.8 ログの 1 行

- `impl Display for toml::Value`（`value.rs` 383〜392 行）は `ValueSerializer` で値の書き方を出す。キーのある表は `{ a = 1, b = 2 }`（`ser/value/map.rs` 165〜182・210〜228 行。空なら `{}`）、配列は `[1, 2, 3]`。
- ただし文字列は toml_writer の `TomlStringBuilder::as_default`（`string.rs` 35〜47 行）が選び、**改行を含む文字列は複数行の文字列（`"""`）になる**。Requirement 6.2 を満たすには、ログの道だけ文字列を 1 行のエスケープで書く必要がある。toml_writer は `pasta_lua` の直接の依存でないので（`Cargo.toml` に足せない）、`log.rs` の中で表を歩いて書くか、文字列だけ自前でエスケープして組み立てる。
- 今の `[1,2,3]`（空白なし）は `[1, 2, 3]` になる。テスト `test_log_array_table_as_json` を直す。

### 2.9 退避（`.broken`）

- Rust の `std::fs::rename` は、Windows では置き換えを許して動く（行き先が既にあれば上書きする）。Unix でも同じ。**既にある `save.toml.broken` は、そのままでは黙って上書きされる**。Requirement 5.4 には、行き先の有無を確かめて別の名前（番号・日時など）を選ぶ手当てが要る。
- PermissionDenied（OneDrive・ウイルス対策ソフトのロック）では、読みも名前の変更も失敗しうる。Requirement 5.5 は「そのランタイムの間は書かない」。
- 今の終了時の保存は `load()` の状態を知らない（`lifecycle.rs` が設定から保存先を計算し直して `save_to_file` を直接呼ぶ）。「書かない」の印を、`save(data)` の閉包と終了時の保存の両方から見える所に置く必要がある（設計で決める。候補は 4 章）。
- `pasta_shiori` のテスト（2.1）は、保存先 `blocker/save.json` の親が通常のファイルである形で保存の失敗を作る。Windows では読み込みが `NotFound` 扱いになる見込み、Unix では `NotADirectory` で「読めない」→ 退避も失敗 → 書かない、になりうる。どちらの道でも終了時に `"Failed to save persistence data on drop"` のエラーのログが出るようにすれば、テストは通る（Requirement 5.5 の「書かずにエラーのログを出す」と合う）。

## 3. 要件と資産の対応

| 要件 | 今ある資産 | ギャップ |
| ---- | ---- | ---- |
| 1.1〜1.4 TOML で書いて読む・gzip | `save_to_file`・`load_from_file`（gzip の判別と原子的な書き込み） | **Missing**: TOML の書き出し・読み込み。JSON の道を消す |
| 1.5〜1.6 空・無い | 同上 | 空の TOML は空の表（そのまま） |
| 1.7・1.8 往復・同じ並び | — | **Missing**: Lua ↔ TOML の変換（2.6・2.7）。並びは `toml::Table` の並べ替えで満たせる |
| 1.9・1.10 書き込みの決まり・API | `save_to_file`、`register` | 保つ。Rust の公開 API の型は変わる |
| 1.11 BOM | toml_parser | 満たしている |
| 1.12 日時 → 文字列 | `toml_to_lua` は呼べない | **Missing**: `persistence.rs` の中の TOML → Lua |
| 2.1〜2.3 引き継がない | 既定の名前の変更で自然に満たす | 2.3 は Requirement 5 に乗る |
| 2.4・2.5 文書・申し送り | — | 文案を作る（設計・完了の時） |
| 3.1〜3.3 既定・拡張子 | `effective_file_path` | **Constraint**: `.replace` を末尾だけに直す |
| 3.4 `.json` の名前 | — | 特別扱いを消すだけ |
| 3.5・3.6 検査・キー | `register` の検査 | 保つ |
| 4.1〜4.10 書けない値 | `log.rs` の `table_depth_exceeds`（深さの守りの型） | **Missing**: キーの道筋つき・キーごとに外す変換・深さと循環の守り |
| 5.1〜5.7 退避 | — | **Missing**: rename・名前の重なり・「書かない」印と終了時の保存の連携 |
| 6.1〜6.7 ログ | `table_to_string` の 2 つの関門と `tostring` の戻し | **Missing**: TOML の 1 行の書き出し（改行のエスケープ）。関門は保つ |
| 7.1〜7.6 サイドカー | `SidecarFile` の derive | `serde_json` を `toml` に替えるだけ |
| 8.1〜8.8 文書 | 2.5 の表 | 書き直し。references は `gen-skill-refs.mjs` で作り直す |

## 4. 作り方の候補

### Option A: 今のファイルを広げる（推奨）

- `persistence.rs` に、Lua → TOML の変換（キーの道筋・理由つきの「外したキー」の一覧を返す。明示のスタックか、深さの上限つきの再帰）と、TOML → Lua の変換（日時 → 文字列）を置く。`save_impl`・`load_impl`・`save_to_file`・`load_from_file` を TOML にする。退避と「書かない」印もここ。
- `log.rs` は、同じ変換を「1 つでも書けなければ全体を失敗」の形で呼び（`pub(crate)`）、1 行の書き出しは `log.rs` の中で持つ。今の 2 つの関門はそのまま前に置く。
- `lifecycle.rs` は、同じ変換と「書かない」印を使う（`@pasta_persistence` の `save` を Lua から呼ぶ形に寄せるか、印を `PastaLuaRuntime` に持たせるかは設計）。
- `sections.rs` は既定値と `effective_file_path`。`sidecar.rs` は `serde_json` を `toml` に替える。
- 長所: 新しいファイルが無く、`runtime/mod.rs`（このウェーブでは `mlua-stdlib-removal` の持ち場）に `mod` の行を足さずに済む。持ち場の約束をそのまま守れる。
- 短所: `persistence.rs` が 568 行から 800 行前後に増える見込み（中の `#[cfg(test)]` が約 250 行）。テストを `persistence_tests.rs` へ `#[path]` で出す（structure.md の src 内テストの配置の決まり）と、1,000 行から遠く保てる。

### Option B: 変換を新しいファイルに分ける

- `runtime/toml_value.rs`（仮の名前）に Lua ↔ TOML の変換を置き、セーブとログから使う。
- 長所: 変換を単独で試しやすい。後の `novel-save-slots`・`novel-checkpoint-rollback` も同じ変換を使える。
- 短所: `runtime/mod.rs` に `mod` の行が要り、このウェーブの `mlua-stdlib-removal` の持ち場に入る（約束の外）。後から入る側が合わせる 1 行で済むが、ウェーブの約束を変えることになる。

### Option C: mlua の値の変換に任せ、根のキーごとに呼ぶ

- 根の `save` テーブルのキーごとに、深さの関門を通してから `from_value_with::<toml::Value>`（`detect_mixed_tables(true)`）を呼び、失敗したキーだけ外す。
- 長所: いちばん短い。
- 短所: 外す単位が根のキーだけになる（A1 に合わない）。警告にキーの道筋を出せない（誤りの文に頼る）。混ざった表が「数のキー」の誤りとして報告され、理由の言葉が分かりにくい。日時の読み込み（1.12）は別に要る。ログの改行（6.2）も別に要る。

## 5. 規模と危うさ

- **規模: M（3〜7 日）**。変換 1 つ（キーの道筋・理由・深さ・循環）、退避（名前の重なり・終了時の連携）、ログの 1 行の書き出し、サイドカーの置き換え、テストの書き直し（2.1〜2.4 の各テスト。`serde_json::json!` を使う材料を TOML に替える）、マニュアル 8 頁と references。brief の 12〜14 タスクと合う。
- **危うさ: Medium**。保存の形式は後戻りしにくく、`novel-*` の 3 本が前提にする。ただし、TOML にする・引き継がない・キーごとに外す・退避する、は開発者が決めてある。残る未知は、退避の終了時の連携と、ログの 1 行の書き出しの細部（どちらも既知の道具で解ける）。

## 6. 設計への申し送り

### 推奨と、設計で決めること

- Option A。変換は `persistence.rs` に置き、ログから `pub(crate)` で使う。
- 「書かない」印の持ち方と、終了時の保存の道（`lifecycle.rs` が Lua の `save` を呼ぶ形に寄せるか、印を共有するか）。`"Failed to save persistence data on drop"`・`"Saved persistence data on drop"` の文言は、`pasta_shiori` のテストが探すので保つ。
- 退避ファイルの名前が重なったときの名前の付け方（`save.toml.broken.1` など）。
- 入れ子の上限の段数（前提 A2 は 32 段）。
- サイドカーの `version` を 1 のままにするか 2 に上げるか（中身の項目は同じ。読むのはテストだけ）。
- `PersistenceError` の誤りの種類（`JsonError` を TOML の読み書きの誤りに替える）。公開の API なので、コミットの件名の `!` と合わせて示す。
- `DESCRIPTION` の文字列（例: `"Persistent data storage (TOML/gzip)"`）。マニュアルと references に写す。
- 根のキーの扱い（4.10）と、配列の要素の表の中のキーの扱い（4.4）を、変換の 1 つの決まりとして書く。

### Research Needed

- toml の文書の書き出し（`toml::to_string` と `to_string_pretty`）が、日本語のキー（`"会員番号"`）・入れ子の表・表の配列（`[[inventory]]`）・配列の中の表をどう書くかを、試しの出力で確かめる（読み戻して同じになることは確か。見た目と、`[a.b]` の見出しの出方の確かめ）。
- 手で書いた深い入れ子の `save.toml`（`[[[[…]]]]` を 10 万段など）を読むときに、toml の読み込みと Lua への変換がスタックを溢れさせないか。pasta が書くファイルは上限の内側なので、起きるのは手で書いたときだけ。必要なら読んだ後に深さを確かめるか、読む前に大きさで断る。
- ログの 1 行の書き出しで、`toml::Value` の `Display` を使いつつ文字列だけ 1 行にする方法（値を歩いて自前で組む方が短いかもしれない）。
- 整数の判定（`n.to_bits() == (n as i64 as f64).to_bits()`）は、`2^63` ちょうどの数で `as i64` が `i64::MAX` に飽和し、`as f64` で `2^63` に戻るため、整数 `9223372036854775807` として書かれて値が変わりうる。pasta 自身の変換では、`i64` の範囲（`-2^63 <= n < 2^63`）を先に確かめてから整数にする（要件 1.7 の往復を守るため。2026-10-24 要件ディスカッションで追記）。
- 同じ書けないキーは、`save(data)` のたびと終了時の保存のたびに警告が出る（要件 4.5 は 1 回の保存の中で 1 キー 1 件）。ログが増えすぎないかは設計で見て、必要なら 1 ランタイムに 1 回に絞るかを決める。
- 整数でない大きな数（2^63 を超える整数の見た目の数）は、mlua が浮動小数点数として渡すので TOML の浮動小数点数になる。手で書いた 2^53 を超える整数は LuaJIT で丸まる。マニュアルに一言書くかを決める。

### 同じウェーブの約束（roadmap「持ち場と、同じウェーブの約束」）

- `runtime/module_registry.rs` に触らない（`toml_to_lua` は呼ばず、写しに近い変換を `persistence.rs` に置く）。
- `Cargo.toml`・`Cargo.lock` に触らない（toml_writer を直接の依存に足さない）。
- `tests/loader/config_test.rs` は本仕様が直す（`mlua-stdlib-removal` は触らない約束）。
- マニュアルの `pasta-log.md`・`execution-model.md`、`tech.md` は `mlua-stdlib-removal` と節が別。`reference/pasta-toml.md` は `recommend-menu-scenes` と節が別。後から入る側が合わせる。
- `runtime/mod.rs` は `mlua-stdlib-removal` の持ち場。Option A なら触らない。

### 範囲の外として確かめて閉じたもの

- `pasta_check` はセーブも `[persistence]` も読まない（`crates/pasta_check/src` に参照なし。`release.rs` 371 行の `var/save.dat` は配布物の検査の材料の名前で、形式に関係しない）。
- VSCode 拡張・LSP はセーブもサイドカーも読まない。
- `pasta_shiori/src/shiori_lifecycle_tests.rs` 552 行の `blocker/save.json` は、保存を失敗させるための名前で、拡張子に意味は無い（brief のとおり）。2.9 の文言の条件だけ守る。
- 見本のゴーストの `[persistence] data_dir` は読まれないが、動きに影響しない（範囲の外）。

## 7. OPEN QUESTIONS（2026-10-24 要件ディスカッションで、全部を推奨案で確定）

9 の `version` は設計で決める（推奨は 1 のまま）。

1. **外す粒度（A1）**: いちばん近い文字列のキーで外し、配列の要素が書けなければ配列ごと外す。— 推奨: このまま。失う値が最も少なく、警告にキーの道筋を出せる。
2. **入れ子の上限（A2）**: セーブは 32 段（ログは 10 段のまま）。— 推奨: 32 段。ノベルの保存の入れ子に余裕を残しつつ、異常な深さを止められる。
3. **`file_path` の `.json`（A3）**: 書いたとおりのパスで TOML を読み書きし、そこに古い JSON があれば退避する。— 推奨: このまま。拡張子から意図を推し量らない決まり（「想定外の書き方に忖度しない」）に合い、元のファイルも `.broken` に残る。
4. **`obfuscate` の名前（A4）**: 既定は今と同じ `save.dat`。古い `save.dat` は `save.dat.broken` に退避される。— 推奨: このまま。名前の決まりが TOML の場合と対になり、古い中身も消えない（使っているゴーストは居ない）。
5. **退避ファイルの重なり（A5）**: 既にある `.broken` は上書きせず、別の名前にする。— 推奨: このまま（名前の付け方は設計）。2 度目の破損で 1 度目の退避を失わないため。
6. **退避できないとき（A6）**: そのランタイムの間は保存先へ書かず、`save()` は失敗を返し、終了時はエラーのログ。— 推奨: このまま。元のセーブを空で上書きしない、という決まりの最後の守り。
7. **`save(data)` の戻り値（A7）**: キーを外しても `true, nil` を返し、外したことはログで知らせる。— 推奨: このまま。戻り値の形を変えず、今の呼び出し側を壊さない。
8. **ログの表の戻し方（A8）**: 書けない値を含む表は、表全体を `tostring()` にする。— 推奨: このまま。今の「変換できなければ tostring」と同じで、一部を黙って省いた出力を出さない。
9. **サイドカーの名前と版（A9）**: `<lua_path>.map` のまま、`version` は設計で決める。— 推奨: 名前は変えない、`version` は 1 のまま。中身の項目が同じで、読むのはテストだけ。
