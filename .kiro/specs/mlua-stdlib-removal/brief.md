# Brief: mlua-stdlib-removal

> **ステータス**: 未着手（2026-10-24 の `/kiro-discovery` で起票。開発者の依頼「mlua-stdlib を外す spec に、OnPastaSetSaveVariable の改めも含めて起票し直して」）。0.4.3（2026-10-24 開発者の決定「次のリリースのタイミングで、mlua 上げちゃいたい」。ウェーブ 1）。要件定義は Opus。作者に見える API を変えるので、リリースノートで知らせる。

## Problem

pasta は、Rust のライブラリ mlua-stdlib（0.1.0）の 6 つのモジュール（`@json`・`@yaml`・`@regex`・`@assertions`・`@testing`・`@env`）を Lua に組み込んでいる。

- **使われていない**: 2026-10-24 のヒアリングで、窓際のぱすたさん（「ぱすた開発」）と emo2（「emo2 開発」）は、どちらも 6 つを 1 つも使っていなかった（ソースを検索して 0 件）。hello-pasta も使っていない。文字列は Lua の標準の `string.match`・`find`・`gsub` で足りている。外部の JSON を読む案は 1 度だけ出たが（気象庁の天気。2026-10-22）、作者が破棄した。
- **mlua の更新を止めている**: crates.io の mlua-stdlib は 0.1.0（`mlua = "^0.11"`）しかない。これがあると、pasta は mlua 0.12 へ上げられない（`mlua-0-12-upgrade`）。
- **既定のシリアライズが JSON になっていた**: pasta 自身が 6 つのうち `@json` だけを使っている。外から変数を書き込むイベント `OnPastaSetSaveVariable`（0.4.2 の `external-control-events`）が、Reference1 を JSON の値として読む。pasta の既定のシリアライズは TOML であるべきで、JSON を選んだのは開発者に確かめずに決めた不当な決定だった（2026-10-24 開発者の指摘）。型を渡すだけなら Reference を 1 つ増やせば済み、JSON は要らなかった（同じ日、開発者「なんで普通にリファレンスを増やさなかったの？」）。
- **TOML を読む部品が無い**: 作者が TOML を読みたいとき（開発者「@toml なら欲しい」）に使える汎用の部品が無い。`@pasta_config` は pasta.toml の値を表にして渡すだけで、任意の TOML の文字列は読めない。

## Current State

2026-10-24（main `06d0b342`）に確かめた。

- **組み込み**: `crates/pasta_lua/src/runtime/mod.rs` 214〜232 行が、`RuntimeConfig::should_enable_module` を見て 6 つを登録する。既定は `runtime_config.rs` 16〜25 行の `default_libs`（`std_all`・`assertions`・`testing`・`regex`・`json`・`yaml`。`env` は既定で無効で、有効にすると警告を出す。390 行）。pasta.toml の `[lua] libs` で入り切りする口は撤去済みで、`RuntimeConfig` は Rust の側だけの設定である。
- **依存**: ルートの `Cargo.toml` 34 行、`crates/pasta_lua/Cargo.toml`。
- **`OnPastaSetSaveVariable`**: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/external_control.lua`（145 行）。Reference0 が変数の名前、Reference1 が JSON の値 1 つ（`60`・`"01"`・`true`・`null`）。`null` で消す。配列とオブジェクトは断る。`pcall(require, "@json")` で読み、無ければ失敗の表記を出す。失敗の種類は `failure.lua` 22〜23 行の `NOT_JSON = "JSON として読めない値"`・`NOT_ASSIGNABLE = "代入できない値"`。文字列の `01` を MCP から送るには `"\"01\""` と二重に包む必要がある。
- **TOML**: `toml` クレートは `pasta_lua` の直接の依存。`module_registry.rs` 216 行の `toml_to_lua` が、TOML の値を Lua の値にする（`@pasta_config` が使う）。Lua の値を TOML にする関数は無い。
- **テスト**: `tests/lua_unittest_runner.rs` 68 行が `@json` を登録する（`external_control_event_test.lua` のため）。`tests/runtime/unit_test.rs` 149 行 `test_to_stdlib_ignores_mlua_stdlib_modules`、`src/error.rs` 288〜289 行、`tests/loader/config_test.rs` 410 行（`libs = ["std_all", "env"]`）、`failure_test.lua`（種類の数）。
- **文書**: マニュアル 10 頁（`lua/modules/mlua-stdlib.md` の章・`lua/modules/index.md`・`lua/index.md`・`lua/shiori-events.md`・`lua/patterns.md`・`lua/dsl-vs-lua.md`・`lua/modules/pasta-log.md`・`internals/loader.md`・`internals/execution-model.md`・`SUMMARY.md`）。スキル `pasta-lua-coding` の `SKILL.md` と references 4 つ（references は `gen-skill-refs.mjs` が作る）。`tech.md` 34 行。

## Desired Outcome

- pasta が mlua-stdlib に依存しない。6 つのモジュールは無くなる。
- `OnPastaSetSaveVariable` は JSON を使わず、Reference で型を受け取る。MCP から引用符を二重に包まずに文字列を送れる。
- 作者が `@toml` で TOML の文字列を Lua の表に読める。
- マニュアルとスキルが今の姿を書いている。

## Approach

mlua-stdlib を丸ごと外す。pasta 自身が要る分（TOML を読む）だけを、既にある部品（`toml` クレートと `toml_to_lua`）で `@toml` として持つ。`OnPastaSetSaveVariable` は、型を Reference2 で受け取る形に改める。

`OnPastaSetSaveVariable` の新しい形（2026-10-24 開発者の決定。型の名前は英語）:

| Reference | 中身 |
|-----------|------|
| 0 | 変数の名前（今のまま） |
| 1 | 値。届いた文字列をそのまま使う（引用符は要らない） |
| 2 | 型。`number`・`string`・`boolean`・`delete` のどれか。省いたら `string` |

```
raise_event(event = "OnPastaSetSaveVariable", references = ["pasta_talk_interval_min", "60", "number"])
raise_event(event = "OnPastaSetSaveVariable", references = ["会員番号", "01"])
raise_event(event = "OnPastaSetSaveVariable", references = ["会員番号", "", "delete"])
```

`number` で数にならない値、`boolean` で `true`・`false` 以外、知らない型の名前は、今の失敗の表記と警告で断り、前の値を残す。

## Scope

- **In**:
  - mlua-stdlib の撤去（依存・登録・`RuntimeConfig` のモジュール名・`env` の警告・テスト）
  - `@toml`（`decode`：TOML の文字列 → Lua の表。読めなければ `nil, エラーメッセージ`。既定で有効）
  - `OnPastaSetSaveVariable` を Reference2 の型で受ける形に改める（`external_control.lua`・失敗の種類の名前（`NOT_JSON` の置き換え）・`failure_test.lua` の数・`external_control_event_test.lua`・`lua_unittest_runner.rs` の `@json` の登録を外す）
  - マニュアル（mlua-stdlib の章を `@toml` の章に置き換える・`OnPastaSetSaveVariable` の節と失敗の表記の表・6 つに触れている頁）、`gen-skill-refs.mjs` での references の作り直し、`pasta-lua-coding` の `SKILL.md`、`tech.md`
- **Out**:
  - mlua 0.12 への更新（`mlua-0-12-upgrade`。この spec の後）
  - JSON・YAML・正規表現の代わりの部品（作者の要望が出たら起票する）
  - `OnPastaCallFunction`（JSON を使っていない）
  - セーブファイル・`@pasta_log` の表の出力・ソースマップのサイドカーの JSON（`toml-serialization`）

## Boundary Candidates

- 依存の撤去と、作者に見える部品の入れ替え（6 つ → `@toml`）
- 外からのイベントの値の受け方（`OnPastaSetSaveVariable`）

## Out of Boundary

- mlua の版（`mlua-0-12-upgrade`）
- DAP・LSP の JSON（プロトコルが決めている）

## Upstream / Downstream

- **Upstream**: `external-control-events`（完了。0.4.2 で出した `OnPastaSetSaveVariable` の形を変える）
- **Downstream**: `mlua-0-12-upgrade`（この spec で mlua-stdlib が無くなれば、上流の公開を待たずに始められる）

## Existing Spec Touchpoints

- **Extends**: `external-control-events`（完了。`OnPastaSetSaveVariable` の値の受け方を改める）
- **Adjacent**:
  - `ponytail-lua-loader`・`ponytail-lua-scripts`（`pasta_lua` の `runtime/`・`pasta_scripts/` を共有しうる。同じウェーブに置かない）
  - `ponytail-editor-tooling`・`ponytail-core-registry`（`Cargo.lock` とルートの `Cargo.toml` の席）
  - `recommend-menu-scenes`（`lua/shiori-events.md` を共有する。節は別）

## Constraints

- 作者に見える API を消す変更（6 つのモジュール）と、0.4.2 で出した `OnPastaSetSaveVariable` の形を変える変更を含む。リリースノートで知らせる。
- pasta から JSON を追い出す。作者の要望で JSON が求められるまで、シリアライズは TOML（2026-10-24 開発者の方針）。DAP・LSP の JSON はプロトコルが決めているので残る。
- `pasta-lua-coding` の `SKILL.md` の編集は、分類器が Self-Modification として止める。止まったら開発者の許可を求める。
- マニュアルの章を消す・足すときは、章の数を決め打ちする 3 つの検査（`verify-scripts-test.mjs`・`talk/talk-test.mjs`・`gen-skill-refs-test.mjs`）を合わせる。章の数は、mlua-stdlib の章を `@toml` の章に置き換えるなら変わらない。
- `Cargo.lock` とルートの `Cargo.toml` は、ウェーブの中では 1 本の spec だけが触る。

## 開発者の判断（要件の前）

- すべて決まった（2026-10-24 開発者の決定）。型の名前は英語（`number`・`string`・`boolean`・`delete`）。`@toml` は `decode` だけで、`encode` は持たない（要る作者が出てから）。
- セーブファイルは決まった（2026-10-24 開発者の方針「pasta から json は一旦追い出して。ユーザー要望で json が要求されるまではシリアライズは toml」）。TOML にする。セーブファイル・ログの表・サイドカーは、別の spec `toml-serialization` が持つ。

## 2026-10-24 棚卸の再測定（main c20ad104）

- **前提の変化**: 起票（main `06d0b342`）の後に入ったのは文書だけで、Current State の行の位置は合っている。依存の更新（PR #187）は mlua-stdlib に触れていない（ルートの `Cargo.toml` 34 行のまま）。
- **触るファイル**
  - 依存: ルートの `Cargo.toml` 34 行、`crates/pasta_lua/Cargo.toml` 23 行、`Cargo.lock`（mlua-stdlib と、それだけが引く `serde_yaml`・`ouroboros`・`owo-colors`・`quick_cache` など）。`regex` クレートは `sakura_script/tokenizer.rs`・`line_breaker.rs`・`runtime/mod.rs` が直に使うので残る。
  - Rust: 登録 `runtime/mod.rs`（286。214〜232 行）、モジュール名と警告 `runtime/runtime_config.rs`（564）、誤りの文 `src/error.rs`（291。試験 288〜289 行）、`@toml` の置き場 `runtime/module_registry.rs`（239。`toml_to_lua` 216 行）。
  - Lua: `pasta_scripts/pasta/shiori/event/external_control.lua`（145）、失敗の種類 `pasta/failure.lua`（22 行。置き換えなら種類の数 12 は変わらない）。
  - 試験: `tests/lua_unittest_runner.rs` 68 行、`tests/runtime/unit_test.rs` 149 行、`tests/lua_specs/external_control_event_test.lua`・`failure_test.lua`（351 行）。Current State に無いもの: `tests/runtime/stdlib_modules_test.rs`（310）・`stdlib_regex_test.rs`（278）を消し、`tests/runtime/main.rs` 26〜27 行を外す。`tests/shiori/res_test.rs` 49 行が `@assertions` を使う。
  - 文書: マニュアル 10 頁（数は合っている）、`crates/pasta_lua/README.md` 56・85 行、マニュアルの章からスキルの写しを作る道具 `book/tools/gen-skill-refs.mjs` 59 行とその試験 `gen-skill-refs-test.mjs` 98 行（章のファイル名を持つ）、スキル `pasta-lua-coding/SKILL.md`（53・76・114〜118・158・160 行）と写し 4 つ（`references/mlua-stdlib.md` は消す）、`tech.md` 34 行。新しいファイルは `@toml` の章と、その試験。1,000 行に近いものは無い。
- **規模**: 9〜11 タスク。
- **先に要るもの**: 働きの上では無い。0.4.3 の中のファイルの重なり:
  - `toml-serialization`: `tests/loader/config_test.rs`（`@env` の試験 396〜436 行は、モジュールが消えても通るので手を付けない）と `runtime/module_registry.rs`（あちらが触らない約束）。その 2 つを守れば重ならない。マニュアル `pasta-log.md`（19 行）・`internals/execution-model.md`（128 行）・`tech.md` は節が別。
  - `recommend-menu-scenes`: ソースは重ならない。マニュアル `lua/shiori-events.md` の既定ハンドラの表（119〜131 行）は同じ表の隣の行（こちらは `OnPastaSetSaveVariable` の行、あちらは行を足して数を直す）で、後から入る側が合わせる。あちらが失敗の種類を足すと `failure.lua`・`failure_test.lua` で重なる。
  - `ponytail-check-release`: 重ならない。ウェーブ 1 で `Cargo.lock` を触るのはこの spec だけ。
- **種別**: 基盤（mlua 0.12 を止めている依存を外す。開発者の依頼。作者に見える部品の入れ替えと、0.4.2 で出したイベントの形の変更を含む）。
- **要件定義のモデル**: Opus（公開の Lua の部品を変えるが、型の名前・`decode` だけ・既定で有効まで開発者が決めた）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - Current State の試験と文書の一覧に、上の `stdlib_modules_test.rs`・`stdlib_regex_test.rs`・`tests/runtime/main.rs`・`res_test.rs`・`pasta_lua/README.md`・`gen-skill-refs.mjs`・`gen-skill-refs-test.mjs` が無い。
  - 章を置き換えるときは、`SUMMARY.md`・上の 2 つの道具・古い写しの `references/mlua-stdlib.md` を同じ変更で直す（`gen-skill-refs --check` が消えた章の写しを見るかを確かめる）。
  - 「リリースノートで知らせる」は、コミットの件名の `!`（詳しいリリースノートの「動きが変わる変更」）と申し送りで行う。見どころのファイルは `release` スキルが書く。
