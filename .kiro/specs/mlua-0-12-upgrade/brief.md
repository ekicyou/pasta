# Brief: mlua-0-12-upgrade

> **ステータス**: 未着手（2026-10-24 の `/kiro-discovery` で起票。`/update` の提案（PR #187）で「起票を勧める」とした 1 本）。0.4.3 のウェーブ 2（2026-10-24 開発者の決定「次のリリースのタイミングで、mlua 上げちゃいたい」。1 本だけで走らせる）。要件定義は Opus。`mlua-stdlib-removal` の後に始める（下の Constraints の「先に要るもの」。2026-10-24 に、上流の公開を待つ道から改めた）。

## Problem

pasta の Lua ランタイムは mlua 0.11（LuaJIT 2.1、features `luajit52`・`vendored`・`serialize`）の上にある。mlua は 2026-07-05 に 0.12.0 を出し、2026-10-03 の 0.12.2 が最新である。0.11 系は 0.11.6（2026-01-27）から更新が止まっている。

- 0.11 に留まるほど、上げるときの差が大きくなる。不具合の直しや LuaJIT の取り込み（`luajit-src`・`lua-src` は mlua-sys が版を決めるので、今は `luajit-src 210.6.6` に止まっている）が、0.12 の側にしか入らなくなる。
- 0.12 は Rust 2024 edition で、pasta と揃う。

## Current State

2026-10-24（main `f472fc27`）に確かめた。

- **依存**: ルートの `Cargo.toml` 33 行 `mlua = { version = "0.11", … }`、34 行 `mlua-stdlib = { version = "0.1", features = ["json", "regex", "yaml"] }`。使うのは `pasta_lua` で、`pasta_shiori`・`pasta_check`・`pasta_sample_ghost` も `mlua::` を参照する。
- **使っている場所**: `mlua::` を参照するファイルは 101（`pasta_lua` 92・`pasta_shiori` 7・`pasta_check` 1・`pasta_sample_ghost` 1）。serde の変換（`LuaSerdeExt`・`to_value`・`from_value`）を使うファイルは 4。
- **mlua-stdlib が止めている**: crates.io の mlua-stdlib は 0.1.0（2025-09-24）しかなく、`mlua = "^0.11"` を要る。上流の main は 2026-10-03 に「Upgrade to mlua 0.12.2」を入れたが、版は 0.1.0 のままで、公開されていない。pasta のクレートは crates.io に出すので、git の依存では置き換えられない。
  - 2026-10-24 のヒアリングで、どのゴーストも mlua-stdlib の 6 つのモジュールを使っていないと分かった。開発者の決定で、mlua-stdlib を外す spec（`mlua-stdlib-removal`）を先に入れる。この spec は mlua-stdlib を扱わない。
- **tech.md**: 「主要依存関係」に `mlua 0.11`・`mlua-stdlib 0.1` と書いてある。前提の「LuaJIT 2.1 には `coroutine.close`・`<close>` が無い」（roadmap）は、mlua の版に関わらず変わらない。

## Desired Outcome

- pasta が mlua 0.12 系の上で動く。
- Lua から見える挙動（serde の変換の結果・エラーの文言・コルーチンの扱い・`require` の探し方）が変わらない。変わるところは、決めたうえでマニュアルに書く。
- x86（`i686-pc-windows-msvc`）と x64 のビルドとテスト、`pasta_lsp` の WASM のビルドが通る。

## Approach

`mlua-stdlib-removal` で mlua-stdlib を外してから始める。0.12 の破壊的変更を pasta の使い方に当てて、まとめて直す。直しの量が多いので、`/update` の C 区分（その場で直す）ではなく、spec にした。

0.12 の主な破壊的変更（変更履歴より）:

- クレート直下の再エクスポートの再編。よく使うもの以外は、新しいサブモジュール（`chunk`・`debug`・`error`・`function`・`table`・`string`・`state`・`thread`・`userdata`）へ移った。`use mlua::…` の行の多くが変わる見込み
- userdata の型に `MaybeSync` が要る
- serde: 配列のメタテーブルを持つ表は、いつも配列として出す（`detect_mixed_tables` を使っていても）
- `Error::ToLuaConversionError` を削除
- `BorrowedStr`・`BorrowedBytes` の寿命の引数を削除
- `Function::wrap`／`wrap_mut`／`wrap_async` の形の変更、`AnyUserData::type_name` が `LuaString` を返す
- GC の口の作り直し（`gc_inc`／`gc_gen` → `gc_set_mode`）

## Scope

- **In**:
  - ルートの `Cargo.toml` の mlua の版と、`Cargo.lock`
  - 4 つのクレートの mlua の使い方の直し（再エクスポートの移動・userdata・serde・エラーの型）
  - serde の変換の挙動が変わるかを確かめる特性化テスト（直す前に足す）
  - `tech.md` の「主要依存関係」、マニュアルの内部設計で mlua の版に触れている所
- **Out**:
  - mlua の features を変えること（`luajit52`・`vendored`・`serialize` のまま。Luau や Lua 5.4 へは移らない）
  - 0.12 で増えた機能（`#[derive(UserData)]`・スレッドのコールバックなど）を使うための作り直し

## Boundary Candidates

- 依存の版と、コンパイルを通すための機械的な直し
- serde・エラー・コルーチンの挙動の差（Lua から見える所）

## Out of Boundary

- Lua ランタイムの設計の変更（`ponytail-lua-*` の各 spec が持つ）
- LuaJIT そのものの版を mlua と別に選ぶこと

## Upstream / Downstream

- **Upstream**: `mlua-stdlib-removal`（mlua-stdlib を外す）
- **Downstream**: なし（今後の mlua の更新は `/update` で扱う）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-lua-loader`・`ponytail-lua-debug`・`ponytail-lua-scripts`・`ponytail-lua-transpiler`（`pasta_lua` のソースを広く共有する。2026-10-24 に 0.4.4 へ移ったので、この spec の後に入る）
  - `toml-serialization`（0.4.3 の先頭。`persistence.rs`・`log.rs` の mlua の serde の使い方が、この spec の直しと重なる。先に入ったものに合わせる）
  - `ponytail-editor-tooling`・`ponytail-core-registry`（`Cargo.lock` とルートの `Cargo.toml` の席）
  - `shiori4-v3-portable`（`pasta_shiori` とルートの `Cargo.toml`・`Cargo.lock`）

## Constraints

- **先に要るもの**: `mlua-stdlib-removal`。mlua-stdlib が残っていると、0.12 へ上げられない（上流の公開は待たない。2026-10-24 開発者の決定）。
- LuaJIT のビルドは、環境変数 `NoDefaultCurrentDirectoryInExePath` を外して行う（残っていると exit 101 で落ちる）。
- pasta.dll は静的 CRT のまま（mlua の `vendored` の LuaJIT も同じビルドに入る）。
- `Cargo.lock` とルートの `Cargo.toml` は、ウェーブの中では 1 本の spec だけが触る。

## 2026-10-24 棚卸の再測定（main c20ad104）

- **前提の変化**: 起票（main `f472fc27`）の後に入ったのは文書だけ。`mlua::` を参照するファイルは 101 のまま（`pasta_lua` 92・`pasta_shiori` 7・`pasta_check` 1・`pasta_sample_ghost` 1）。mlua の値の変換（`LuaSerdeExt`）を使うファイルは 4 でなく 3（`runtime/persistence.rs`・`runtime/log.rs`・`runtime/lifecycle.rs`）で、3 つとも `toml-serialization` が書き直す。
- **触るファイル**: ルートの `Cargo.toml` 33 行と `Cargo.lock`、上の 101 ファイル（`pasta_shiori` は `src/error.rs`・`lua_request.rs`・`shiori.rs` と試験 4、`pasta_check` は `tests/validate_parity_test.rs`、見本のゴーストは `tests/self_deploy_integration_test.rs`）。破壊的変更が当たる所: 利用者の値の型 `pasta_lua/src/search/context.rs` 357 行（`impl UserData`。`MaybeSync`）、型の名前 `runtime/enc.rs` 74・117 行と `tests/ffi/compare_test.rs` 40 行。GC の口・`ToLuaConversionError`・`BorrowedStr`・`Function::wrap` は pasta のどこにも無い。`tech.md` 7・33 行。1,000 行に近いものは無い。
- **規模**: 12〜16 タスク（L のまま）。
- **先に要るもの**
  - 働きの上では `mlua-stdlib-removal`。
  - ファイルの重なりでは、ウェーブ 1 の 3 本が入った後: `toml-serialization`（上の 3 ファイル）、`mlua-stdlib-removal`（`runtime/mod.rs`・`runtime_config.rs`・`module_registry.rs`・`tests/runtime/unit_test.rs`・`tests/lua_unittest_runner.rs`・`Cargo.toml`・`Cargo.lock`）、`ponytail-check-release`（`self_deploy_integration_test.rs`）。`recommend-menu-scenes` とは重ならない。
  - 後の `shiori4-v3-portable` とは `Cargo.toml`・`Cargo.lock`・`pasta_shiori/src/shiori.rs` で重なる。
- **種別**: 基盤（依存の版上げ。`/update` から）。
- **要件定義のモデル**: Opus（土台の決定は無く、直しの量が多いだけ）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - Current State の「serde の変換を使うファイルは 4」は 3。
  - Upstream に `toml-serialization` を、ファイルの順として足す（今は Adjacent にだけある）。値の変換の特性化の試験は、`toml-serialization` が入った後のセーブとログの表に対して書く。あちらが手書きの変換にすれば、この spec の値の変換の直しは減る。

## 申し送り（ponytail-check-release より、2026-10-24）

- `crates/pasta_sample_ghost/tests/self_deploy_integration_test.rs` の形が変わった。ファイルの中の `neutralize_debug_env`・`copy_tree_skip_profile`・`fresh_ghost_copy`・`fixture_master_dir` は無くなり、`mod common;` の `common::master_copy()`（`profile/` と `pasta.dll` を除いて master を一時フォルダへ写す）と、`common/mod.rs` の `#[ctor]`（`PASTA_DEBUG`・`PASTA_DEBUG_PORT` を外す）を使う。mlua の直しでこのファイルを触るときは、今の形の上で直す。
- `common/mod.rs` の公開の口はこの 2 つだけ（`touch_shortcuts_test.rs`・`tutorial_stages_test.rs` も使う）。参照: `.kiro/specs/completed/ponytail-check-release/design.md` の「SampleGhostTests」。

## 申し送り（mlua-stdlib-removal より・2026-10-24）

- mlua-stdlib は依存から外れた（ルートと `pasta_lua` の `Cargo.toml`・`Cargo.lock`）。上の「依存」「mlua-stdlib が止めている」「tech.md」の記述は古い。
- `RuntimeConfig` の公開の中身が変わった: `default_libs()`・`new()`・`minimal()` は `["std_all"]`、`full()` は `["std_all_unsafe"]`、`should_enable_module` は無い。`validate_and_warn` の `env` の警告も無い。`UnknownLibrary` の文は 13 の `std_` の名前だけ。
- `runtime/mod.rs` の初期化は `@pasta_search` → `@pasta_log` → `@toml`（`module_registry.rs` の `register_toml_module`。`toml_to_lua` を使う）。mlua 0.12 で `Value`・`Function` の型が変わるなら、この関数も直す範囲に入る。
- 参照先: `.kiro/specs/completed/mlua-stdlib-removal/design.md` の RuntimeConfig・TomlModule・RuntimeInit。
