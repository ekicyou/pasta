# ギャップ分析: mlua-0-12-upgrade

2026-10-24（main `98653c88`）に、requirements.md と今のコードを突き合わせた。推測を減らすため、ワークスペースをスクラッチへ写し（`git archive HEAD`）、ルートの `Cargo.toml` の mlua だけを `0.12` に変えて `cargo update -p mlua` → `cargo check`・`cargo clippy`・`cargo test` を x64 と x86 で実際に走らせた（作業ツリーと `Cargo.lock` は触っていない）。

## 0. 要点

- **mlua 0.12.2 への直しは、思っていたよりずっと小さい。** x64 では、`mlua::Debug` の参照を `mlua::debug::Debug` に変える 6 ファイルの `use` の直しだけでワークスペース全体がビルドでき、**テスト 3,046 本がすべて通った**（失敗 0・無視 9。lua_test のランナー・デバッガの試験・FFI の試験・SHIORI の結合の試験を含む）。
- 残るのは clippy の `-D warnings` で誤りになる `mlua::String`（0.12 で非推奨。`mlua::LuaString` へ）の 14 か所（6 ファイル）だけ。
- brief が「破壊的変更が当たる所」に挙げた所は、実際には当たらない（下の 2 章）。`MaybeSync` は `send` の features を使わないので自動で満たされ、`enc.rs` 74・117 行と `compare_test.rs` 40 行は `Value::type_name`（`&str` のまま）で、変わった `AnyUserData::type_name` ではない。
- **大きな発見: 今の main は x86 でビルドできない。** `toml-serialization`（`e221ebf3`）が入れたセーブの整数の変換が、x86 の Lua の整数（`mlua::Integer` = 32 ビットの `i32`）を考えていない。mlua の版に関わらず起きる（mlua 0.11 の今の作業ツリーでも同じ誤りを確かめた）。CI の build-x86 は `e221ebf3`・`1fb7ed38`・`98653c88` の 3 回続けて赤（x64・WASM・luacheck・cargo-deny は緑）。Requirement 7 と OPEN QUESTION 1・2 はこれによる。
- `Cargo.lock` の差は mlua・mlua-sys（0.10.0 → 0.13.0）・`luajit-src`（`210.6.6+707c12b` → `210.7.4+c6ffc14`）・`lua-src`（550.0.0 → 551.0.2）の 4 つだけ。ほかの依存は動かない（Requirement 1.4 を満たす）。

## 1. 今あるもの

### 依存

| 場所 | 今の中身 |
| ---- | -------- |
| ルートの `Cargo.toml` 33 行 | `mlua = { version = "0.11", features = ["luajit52", "vendored", "serialize"] }`（mlua-stdlib はもう無い） |
| `crates/pasta_lua/Cargo.toml` 22 行 | `mlua.workspace = true`。mlua に直に依存するのは `pasta_lua` だけ |
| `crates/pasta_lua/src/lib.rs` 69 行 | `pub use mlua;`。`pasta_shiori`・`pasta_check`（dev-dependency の試験）・`pasta_sample_ghost`（試験）は `pasta_lua::mlua` を通して使う。`pasta_lua` は crates.io に出しているので、mlua の版は `pasta_lua` の公開の口の一部（OPEN QUESTION 5） |
| `Cargo.lock` | mlua 0.11.6・mlua-sys 0.10.0・`luajit-src 210.6.6+707c12b`・`lua-src 550.0.0` |
| `about.hbs` 14〜70 行 | LuaJIT の著作権の文を固定で書いている（依存解析に出ないため）。`luajit-src` 210.6.6 と 210.7.4 の `luajit2/COPYRIGHT` は同じ文であることを確かめた（Requirement 8.3 は確かめるだけで済む） |
| `.cargo/config.toml` | x86・x64 とも `+crt-static`。mlua-sys 0.13 の `vendored` もこの設定でビルドできた（Requirement 1.5） |

### mlua を参照するファイル

`git grep -E 'mlua::|use mlua'` で 103（`pasta_lua` 94・`pasta_shiori` 7・`pasta_check` 1・`pasta_sample_ghost` 1）。brief の数と同じ。ただし、0.12 で直しが要るのはこのうち 12 ファイルだけ（下）。

### 0.12 で直しが要る所（実測）

| 種類 | ファイル | 中身 |
| ---- | -------- | ---- |
| コンパイルの誤り（E0432） | `src/debug/hook.rs` 61 行、`src/debug/session/stop_loop.rs` 9 行、`src/debug/session/mod.rs` 97 行、`src/debug/hook_tests.rs` 9 行、`src/debug/inspect_test_support.rs` 25 行、`tests/chunk_name_validation_test.rs` 57 行 | `mlua::Debug` がクレート直下から消え、`mlua::debug::Debug` へ移った。`HookTriggers`・`VmState` は直下に残る |
| 非推奨の警告（clippy `-D warnings` で誤り） | `src/runtime/enc.rs`（24 行の `String as LuaString` の 1 か所で 7 件）、`src/runtime/ffi_load/mod.rs` 3、`src/runtime/module_registry.rs` 1、`src/runtime/searcher.rs` 3、`tests/runtime/encoding_test.rs` 4、`tests/runtime/literal_fixes_test.rs` 2 | `mlua::String` は `mlua::LuaString` の非推奨の別名になった |
| 説明だけ | `src/debug/types.rs` 217 行（文書のコメントの `mlua::Debug`）、`src/debug/inspect.rs` 23・66 行と `src/debug/inspect_stack_coroutine_tests.rs` 159 行（「mlua 0.11.6 のソースで確かめた」「mlua 0.11 / mlua-sys 0.10 の FFI」） | Requirement 8.2 の対象。0.12.2 のソースで確かめ直して書き改める |

`pasta_shiori`・`pasta_check`・`pasta_sample_ghost` は、どれも直しが要らなかった。

### brief の見立てと実際

| brief の見立て | 実際 |
| -------------- | ---- |
| `use mlua::…` の行の多くが変わる | 変わるのは `Debug` を使う 6 ファイルだけ。`Table`・`Function`・`Value`・`Error`・`Lua`・`LuaOptions`・`StdLib`・`Thread`・`AnyUserData`・`LightUserData`・`ErrorContext`・`HookTriggers`・`VmState`・`ffi` などはクレート直下に残る（`mlua-0.12.2/src/lib.rs` 103〜153 行） |
| `search/context.rs` 357 行の `impl UserData` に `MaybeSync` が要る | 要らない。`MaybeSync` は `send` の features のときだけ `Sync` を求め、それ以外は全部の型に付く |
| `runtime/enc.rs` 74・117 行、`tests/ffi/compare_test.rs` 40 行の型の名前 | `Value::type_name()`（`&'static str` のまま）で、変わったのは `AnyUserData::type_name`。当たらない |
| `module_registry.rs` の `register_toml_module` | そのままビルドでき、`@toml` の試験も通った |
| `persistence.rs` 362 行・`log.rs` 123 行の `Table::pairs` | 0.12 でも `lua_next` で回す（`table.rs` 687 行）。`__pairs` は呼ばない。試験も通った |
| `persistence.rs` の `array_metatable` | 0.12 でも `LuaSerdeExt::array_metatable`（`serde/mod.rs` 192 行）。配列の印の試験 2 本も通った |
| GC の口・`ToLuaConversionError`・`BorrowedStr`・`Function::wrap` | pasta のどこにも無い（brief のとおり） |

### Lua から見える挙動に関わる 0.12 の変更（変更履歴と差分）

- `Error` の `Display`（`runtime error: `・`syntax error: ` の接頭辞、コールバックの誤りの積み重ね）は、`ToLuaConversionError` が消えたほかは同じ（0.11.6 と 0.12.2 の `error.rs` の差分は書式だけ）。止まるエラーのバルーンの 1 行の形（`pasta_shiori/src/error.rs` の `single_line`）は変わらない。
- `StdLib` の定数は `JIT` のビットの位置が変わった（`1 << 11` → `1 << 12`。Luau 用の `INTEGER` が入ったため）。pasta は名前（`std_jit` などの 13 の名前 → `StdLib::JIT`）で引くので影響しない。
- 0.12.2 の直し: コールバックと userdata の後始末の中の panic と誤りの扱い、参照のスタックが尽きたときに誤りを返す、スレッドを使い回すときのグローバルなフックの戻し。0.12.1: フックから `yield` した後のコルーチンのスタック。0.12.0: `Lua::current_thread` が暗黙の非同期のスレッドを根の持ち主に解く（pasta は非同期を使わない）。デバッガは「行のフックの中で `current_thread()` が走っているコルーチンを指す」ことに頼る（`inspect.rs` 10〜40 行）が、その試験（`inspect_stack_coroutine_tests.rs`・`wiring_*`・`session_step_controller_tests.rs`）は 0.12.2 で全部通った。
- serde の「配列の印のある表はいつも配列で出す」は、`to_value` を使っていないので当たらない。
- LuaJIT は `707c12b` → `c6ffc14` に進む。pasta の試験（Lua の単体テスト・FFI・コルーチン）は全部通ったが、LuaJIT 側の変更の中身は見ていない（Research Needed）。

### x86 の不具合（mlua の版に関わらない）

`mlua-sys` は `lua_Integer` を、32 ビットの機械では `i32`、64 ビットでは `i64` にする（0.10.0 も 0.13.0 も同じ。`src/lua51/lua.rs` 72・74 行）。`toml-serialization` の 3 か所がこれを `i64` と決めて書いている。

| 場所 | 中身 |
| ---- | ---- |
| `runtime/persistence.rs` 382 行 | `Value::Integer(i) => toml::Value::Integer(*i)`（x86 では `i32` を `i64` に入れられない） |
| `runtime/persistence.rs` 430 行 | 配列かどうかを決めるキーの数え上げ `max_int.max(*i)`（`max_int` は `i64`） |
| `runtime/persistence.rs` 554 行 | `toml::Value::Integer(i) => Value::Integer(*i)`（`i64` を `i32` に入れられない。32 ビットを超える整数の行き先が要る） |
| `runtime/persistence_tests.rs` 602・606 行 | `Value::Integer(i64::MAX)`・`Value::Integer(i64::MIN)` の照合（x86 では型が合わない） |

スクラッチで仮の直し（`i64::from(*i)`、読みは `mlua::Integer::try_from(*i)` が失敗したら `Value::Number(*i as f64)`、`i64` の端の試験を 64 ビットだけにする）を当てると、x86 の clippy は通ったが、x64 の clippy が `useless_conversion`（`i64::from` が x64 では同じ型）で落ちた。両方の機械で警告を出さない書き方は設計で決める（`#[cfg(target_pointer_width)]`・`as` と `allow`・`TryFrom` を使うなど）。

この仮の直しの上で x86 の全部の試験を走らせた結果は、末尾の「x86 の試験の結果」に書く。

- mlua は LuaJIT の数を `num_traits::cast` で `mlua::Integer` に収まるかを確かめてから `Value::Integer` にする（`state/raw.rs` 922〜929 行）。x86 では 32 ビットを超える整数の値は `Value::Number` で来るので、セーブには `3000000000.0` のような浮動小数として書かれる（x64 では整数）。読み戻した Lua の値は同じ（OPEN QUESTION 2）。
- `@pasta_log` の表の書き出し（`log.rs`）も同じ分け方に乗るので、x86 では 32 ビットを超える整数が浮動小数の形で出るはず（未確認・Research Needed）。
- 2^53 を超える整数が丸まることは、`toml-serialization` の確定した設計判断 6 でマニュアル（`pasta-persistence.md` 137 行・`toml.md` 56 行）に書いてある。x86 も同じ扱いでよい（要件 7.4 の前提）。

## 2. 要件と今あるものの対応

| 要件 | 今あるもの | 差 |
| ---- | ---------- | -- |
| 1.1〜1.3 依存の版・features・crates.io | ルートの `Cargo.toml` 33 行 | **Missing**: 版を `0.12`（下限は OPEN QUESTION 4）に変えるだけ。features はそのまま通る |
| 1.4 他の依存を動かさない | `cargo update -p mlua` で 4 つだけ動く | 満たせる（`cargo update` を全体にかけない） |
| 1.5 静的 CRT | `.cargo/config.toml` | 満たせる（x86・x64 ともビルドできた） |
| 1.6 1 つの版 | `pub use mlua;` | 満たせる |
| 2.1 x86・x64 のビルド | x64 は 6 ファイルの直しで通る。x86 は今の main で通らない | **Missing**: x86 の整数の 3 か所（Requirement 7） |
| 2.2 テスト | x64 は 3,046 本すべて通った | **Constraint**: x86 は往復の試験 2 本が落ちる（末尾）。期待の書き方は OPEN QUESTION 2 しだい |
| 2.3 clippy | 非推奨の `mlua::String` の 14 か所 | **Missing**: 機械的な置き換え。x86 の直しも x64 の clippy を通す形が要る |
| 2.4 WASM | `pasta_lsp`・`pasta_check` のライブラリは mlua に依存しない（`cargo tree -i mlua` で出ない） | 満たせる（`Cargo.lock` の差だけ）。走らせて確かめるのは実装のとき |
| 2.5 lua_test | `tests/lua_unittest_runner.rs` | 満たせる（x64 で通った） |
| 3.1 SHIORI の応答 | `pasta_shiori` の結合の試験・`pasta_sample_ghost` の試験・`tests/shiori/` | 既存の試験で押さえている（x64 で通った） |
| 3.2 誤りの文言 | `runtime_error_balloon_e2e_test.rs`・`pasta_shiori/src/error.rs` の `single_line` の試験・`transpiler/failure_output_test.rs`・`searcher.rs` の `syntax error: ` の外し方 | 既存で押さえている。mlua の `Display` も変わっていない |
| 3.3・3.4 require | `tests/runtime/module_searcher_test.rs`・`searcher.rs` の 4 要素の検査 | 既存で押さえている |
| 3.5 コルーチン | `lua_specs/event_coroutine_test.lua` ほか・`store.lua`・`chain_talk_interrupt_test.rs` | 既存で押さえている。`coroutine.status` の値そのものを照合する試験の網羅は設計で確かめる（**Unknown**） |
| 3.6 配列の印 | `persistence_tests.rs` の `test_codec_array_marked_empty_table_is_empty_array`・`test_codec_read_value_types` | 既存で押さえている |
| 3.7 `__pairs` を呼ばない | `persistence.rs` 359 行の説明だけ | **Missing**: `__pairs` を持つ表を保存して、`__pairs` が呼ばれないことを確かめる試験が無い（特性化の候補） |
| 3.8 `@toml`・`@enc`・`ffi.load` | `tests/runtime/encoding_test.rs`・`tests/ffi/`・`@toml` の試験 | 既存で押さえている |
| 3.9 13 の `std_` の名前 | `runtime_config.rs` の `to_stdlib` と試験 | 既存で押さえている（`JIT` のビットの移動は名前で引くので効かない） |
| 4.* 挙動の差の扱い | — | x64 では差が見つからなかった。x86 の整数の書き方の差（OPEN QUESTION 2）は mlua でなく main の不具合から来る |
| 5.* 特性化の先行 | — | 足す候補は 3.7（`__pairs`）と、3.5 の `coroutine.status` の照合（足りなければ）。x86 は今の main でビルドできないので、x86 の特性化は Requirement 7 の直しの後になる |
| 6.* デバッガ | `debug/` の試験（`hook_tests`・`inspect_stack_coroutine_tests`・`wiring_*`・`session_step_controller_tests`・`wiring_pasta_step_e2e`） | 既存で押さえている（x64 で通った）。説明のコメントの「0.11.6 で確かめた」は改める |
| 7.* x86 の整数 | — | **Missing**（上の 1 章） |
| 8.1 `tech.md` | 7 行・33 行 | **Missing**: 「mlua 0.11」を「mlua 0.12」へ |
| 8.2 マニュアル・ソースの説明 | `book/src/internals/` に mlua の版を書いた所は無い（`mlua::Lua`・`mlua::ffi`・`Lua::unsafe_new_with` などの名前は 0.12 にもある）。ソースのコメント 3 か所 | **Missing**: ソースのコメントだけ |
| 8.3 LuaJIT の著作権の文 | `about.hbs` | 確かめた。同じ文 |
| 8.4 roadmap | `roadmap.md` 137 行 | **Missing**: 入った後に書く |
| 8.5 リリースノート | — | **Missing**: 0.4.3 のリリースのときに書く |

## 3. 進め方の案

### 案 A: 1 本の流れで直す（推奨の候補）

特性化の試験（`__pairs`・足りなければ `coroutine.status`）を 0.11 の上で足す → x86 の整数の直し（0.11 の上で x86 のビルドと試験を通す）→ 版を上げて `Debug` の `use` と `LuaString` を直す → 説明・`tech.md` を直す。

- ✅ 要件 5.2（0.11 の上で特性化を通してから上げる）を x86 でも守れる
- ✅ 直しが小さいので、タスクは 6〜8 本で済む（brief の 11〜15 より少ない）
- ❌ x86 の main の赤は、この spec が入るまで続く

### 案 B: x86 の直しを先に別の PR で main へ入れ、この spec は版上げだけにする

- ✅ main の CI がすぐ緑に戻る（`shiori4-v3-portable` など、後の spec のギャップ分析が赤の上で始まらない）
- ✅ この spec の差が mlua だけになり、見直しやすい
- ❌ 小さくても x86 の整数の書き方（OPEN QUESTION 2）を先に決める必要がある
- ❌ 記憶の方針「起票に安易に逃がさない」とぶつかるように見えるが、別 spec を起こすのではなく同じ 0.4.3 の中の追いかけの直し（「main 赤は追いかけ修正」）なので、方針の内側に収まる

### 案 C: 機械的な直し（`Debug`・`LuaString`）と挙動の確かめを 2 つの段に分ける

`Cargo.toml` を上げて機械的に直す段と、特性化・x86・説明の段に分ける。

- ✅ revert しやすい（「1 抽出＝1 検証＝1 コミット」）
- ❌ 要件 5.2 の「上げる前に特性化」と順が逆になる。直しが 12 ファイルと小さいので、分ける得は少ない

## 4. 手間と危うさ

- **手間: S（1〜3 日）**。x64 の直しは 12 ファイルの機械的な置き換えで、試験はすでに全部通る。x86 の整数の 3 か所と試験の期待の書き分け、特性化の 1〜2 本、説明の直しが加わる。brief の L（11〜15 タスク）は、実測の後では大きすぎる。
- **危うさ: 低〜中**。x64 は実測で差が無い。中にするのは、x86 の整数の扱い（セーブの書き方の差。作者のセーブファイルが x86 と x64 で違って見える）と、LuaJIT の版が進むことで起きうる、試験で押さえていない所の差。

## 5. 設計へ持ち越すもの

- **決めること**
  - x86 の整数の直しの形（`#[cfg(target_pointer_width)]` で分ける・`TryFrom` で揃える・`allow(clippy::useless_conversion)` を使う、のどれか）。両方の機械で clippy を通す
  - 32 ビットを超える整数の値を、x86 で浮動小数のまま書くか、整数の値を持つ浮動小数を整数として書くか（OPEN QUESTION 2）。後者にすると x64 の既存の振る舞い（`1.0` の扱いなど）にも触れるので、範囲に注意
  - x86 で読む 32 ビットを超える整数の行き先（`Value::Number(i as f64)`。2^53 を超えれば丸まる）
  - `persistence_tests.rs` の `i64` の端の試験の、x86 での期待
  - mlua の版の書き方（`"0.12"` か `"0.12.2"`。OPEN QUESTION 4）と `serialize` の扱い（OPEN QUESTION 3）
- **Research Needed**
  - LuaJIT `707c12b` → `c6ffc14` の間の変更（`luajit-src` の `luajit_relver.txt` と上流の履歴）。Lua から見える変更があればマニュアルに書く
  - x86 での `@pasta_log` の表の整数の出方
  - `coroutine.status` の値を照合する既存の試験の網羅（足りなければ特性化に足す）
  - mlua 0.12.2 の `global_hook_proc`・`callback_error_ext`・`current_thread` を読み直し、`inspect.rs` の説明を書き改める
- **確かめ方の注意**
  - `cargo` を走らせる前に `NoDefaultCurrentDirectoryInExePath` を外す
  - スクラッチのような長いパスの `CARGO_TARGET_DIR` では、`icu_normalizer_data` のビルドスクリプトのリンクが `LNK1104` で落ちた。短いパスの target で走らせる
  - `pasta_lua` の試験は `sample.generated.lua` を改行コードだけ書き換えることがある。`git checkout --` で戻す

## x86 の試験の結果

mlua 0.12.2 と 1 章の仮の直し（x86 の整数の 3 か所、`test_codec_read_value_types` を 64 ビットだけにする、`LuaString` の置き換え）の上で、`cargo test --all --target i686-pc-windows-msvc --no-fail-fast` を走らせた。

- 通った 3,043・落ちた 2・無視 9（x64 の 3,046 との差の 1 本は、64 ビットだけにした `test_codec_read_value_types`）。
- 落ちた 2 本は、どちらも表に入った `i64::MIN` の往復。
  - `runtime::persistence::tests::test_codec_round_trip_values`（`persistence_tests.rs` 681 行、キー `imin`）
  - `runtime::persistence::tests::test_codec_round_trip_i64_extremes_as_values`（同 703 行）
  - どちらも、期待 `Integer(-9223372036854775808)` に対して `Float(-9.223372036854776e18)` が戻った。x86 では Lua に渡った `-2^63` が `mlua::Integer`（`i32`）に収まらないので `Value::Number` になり、浮動小数として書かれる（OPEN QUESTION 2 の差そのもの）。
- ほかは、デバッガ・FFI・Shift_JIS の変換・SHIORI の結合・lua_test を含めて全部通った。mlua 0.12 から来る x86 だけの差は見つからなかった。
- x86 の clippy（`-D warnings`）は、この仮の直しで通った。x64 の clippy は `i64::from` の `useless_conversion` 2 件で落ちる（1 章）。
