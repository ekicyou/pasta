# Brief: mlua-0-12-upgrade

> **ステータス**: 未着手（2026-10-24 の `/kiro-discovery` で起票。`/update` の提案（PR #187）で「起票を勧める」とした 1 本）。版は未定。要件定義は Opus。**今は始められない**（下の Constraints の「先に要るもの」）。

## Problem

pasta の Lua ランタイムは mlua 0.11（LuaJIT 2.1、features `luajit52`・`vendored`・`serialize`）の上にある。mlua は 2026-07-05 に 0.12.0 を出し、2026-10-03 の 0.12.2 が最新である。0.11 系は 0.11.6（2026-01-27）から更新が止まっている。

- 0.11 に留まるほど、上げるときの差が大きくなる。不具合の直しや LuaJIT の取り込み（`luajit-src`・`lua-src` は mlua-sys が版を決めるので、今は `luajit-src 210.6.6` に止まっている）が、0.12 の側にしか入らなくなる。
- 0.12 は Rust 2024 edition で、pasta と揃う。

## Current State

2026-10-24（main `f472fc27`）に確かめた。

- **依存**: ルートの `Cargo.toml` 33 行 `mlua = { version = "0.11", … }`、34 行 `mlua-stdlib = { version = "0.1", features = ["json", "regex", "yaml"] }`。使うのは `pasta_lua` で、`pasta_shiori`・`pasta_check`・`pasta_sample_ghost` も `mlua::` を参照する。
- **使っている場所**: `mlua::` を参照するファイルは 101（`pasta_lua` 92・`pasta_shiori` 7・`pasta_check` 1・`pasta_sample_ghost` 1）。serde の変換（`LuaSerdeExt`・`to_value`・`from_value`）を使うファイルは 4。
- **mlua-stdlib が止めている**: crates.io の mlua-stdlib は 0.1.0（2025-09-24）しかなく、`mlua = "^0.11"` を要る。上流の main は 2026-10-03 に「Upgrade to mlua 0.12.2」を入れたが、版は 0.1.0 のままで、公開されていない。pasta のクレートは crates.io に出すので、git の依存では置き換えられない。
- **tech.md**: 「主要依存関係」に `mlua 0.11`・`mlua-stdlib 0.1` と書いてある。前提の「LuaJIT 2.1 には `coroutine.close`・`<close>` が無い」（roadmap）は、mlua の版に関わらず変わらない。

## Desired Outcome

- pasta が mlua 0.12 系と、それに合う mlua-stdlib の上で動く。
- Lua から見える挙動（serde の変換の結果・エラーの文言・コルーチンの扱い・`require` の探し方）が変わらない。変わるところは、決めたうえでマニュアルに書く。
- x86（`i686-pc-windows-msvc`）と x64 のビルドとテスト、`pasta_lsp` の WASM のビルドが通る。

## Approach

mlua-stdlib の 0.12 対応版が crates.io に出てから始める。始めたら、0.12 の破壊的変更を pasta の使い方に当てて、まとめて直す。直しの量が多いので、`/update` の C 区分（その場で直す）ではなく、spec にした。

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
  - ルートの `Cargo.toml` の mlua・mlua-stdlib の版と、`Cargo.lock`
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

- **Upstream**: mlua-stdlib の 0.12 対応版の公開（上流 mlua-rs/mlua-stdlib）
- **Downstream**: なし（今後の mlua の更新は `/update` で扱う）

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-lua-loader`・`ponytail-lua-debug`・`ponytail-lua-scripts`・`ponytail-lua-transpiler`（`pasta_lua` のソースを広く共有する。同じウェーブに置かない）
  - `ponytail-editor-tooling`・`ponytail-core-registry`（`Cargo.lock` とルートの `Cargo.toml` の席）
  - `shiori4-v3-portable`（`pasta_shiori` とルートの `Cargo.toml`・`Cargo.lock`）

## Constraints

- **先に要るもの**: mlua-stdlib の mlua 0.12 対応版が crates.io に出ること。出るまでは始められない。待たずに進める案（mlua-stdlib を外して json・regex・yaml を自前か別のクレートで置き換える）は、要件で比べる。
- LuaJIT のビルドは、環境変数 `NoDefaultCurrentDirectoryInExePath` を外して行う（残っていると exit 101 で落ちる）。
- pasta.dll は静的 CRT のまま（mlua の `vendored` の LuaJIT も同じビルドに入る）。
- `Cargo.lock` とルートの `Cargo.toml` は、ウェーブの中では 1 本の spec だけが触る。
