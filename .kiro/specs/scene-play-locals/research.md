# ギャップ分析: scene-play-locals

- **Feature**: `scene-play-locals`
- **日付**: 2026-11-12（ブランチ `claude/kiro-start-scene-play-locals-3c8452`、base `165d2c46`）
- **Discovery Scope**: Simple Addition（既存のハンドラの拡張）
- **要件**: `requirements.md`（Requirement 1〜7）

## 要約

- **ローカル変数は act ごとの表（`act.var`）で、ハンドラが受け取る act と、シーンを始めるときの act は同じもの**である。ハンドラの中で `act.var[キー] = 値` を入れてからコルーチンを返せば、シーンの最初の行から `＄キー` で読める。`scene.lua`（`co_exec_exact`・`co_exec_registered`）にも、生成するシーンの関数（`code_gen`）にも手を入れなくてよい見込み。brief が In に挙げた「`scene.lua` か、生成するシーンの関数の入口」は、どちらも触らずに済む。
- **brief の例 `{ 名前 = "ぱすた", 回数 = 3 }` は TOML として読めない**。pasta が使う toml 1.1.8（spec 1.1.0）は、引用符の無い日本語のキーを受けない（2026-11-12 にスクラッチで確かめた。`TOML parse error at line 1, column 7`）。`{ "名前" = "ぱすた" }` なら読める。要件は「TOML をそのまま受け、日本語のキーは引用符で包む」で仮置きした（OPEN QUESTION 1）。
- `@toml.decode` は文書（`キー = 値` の並び）しか読まないので、インライン表 1 つは `v = <Reference2>` に包んで読み、`v` だけを取り出す形になる。日付・時刻は `@toml` が文字列に変えるので、読んだ後では文字列と見分けられない（OPEN QUESTION 3）。
- 変数の名前の検査（Requirement 4.2）は、辞書の `＄名前` の決まり（`grammar.pest` の `id`＝XID_START か `_` で始まり XID_CONTINUE が続く、`__…__` を除く。数字だけは `＄０` のシーン引数）に揃える必要がある。Lua のパターンでは XID を正しく判定できないので、正しく揃えるなら Rust 側の小さな関数が要る。ここが設計の主な分かれ目。
- 規模は **S**、リスクは **Low**。触るのは `scene_play.lua`・`failure.lua`（ラベル 1 つ）・必要なら `@pasta_debug` に関数 1 つ、テスト、マニュアル 1 章とその生成物。`pasta_lsp`・`editors/vscode` には触らない。

## 1. 今の状態

### 1.1 受け口: `scene_play.lua`

`crates/pasta_lua/pasta_scripts/pasta/shiori/event/scene_play.lua`（96 行）。

- 門 `allowed(act, event, outcome)`（19〜36 行）: `@pasta_debug.enabled()` と `act.req.security_level ~= "external"`。断るときは警告 1 行で nil（204）。Requirement 5.1・5.2 は、Reference2 を門の後で読めば今のまま満たせる。
- `OnPastaScene`（38〜60 行）: Reference0 を `FAILURE.is_name` で確かめ、Reference1 の `""` を nil にし、`SCENE.co_exec_exact(name, local_name)`。見つかれば `STORE.co_preempt[co] = true` を付けて返す。Requirement 3.3（Reference1 が空ならグローバルだけ）は、今の `if local_name == "" then local_name = nil end` がもう満たしている。Reference1 が無い（nil）ときも `co_exec_exact` はグローバルだけを探す。
- `OnPastaSceneAt`（62〜90 行）: Reference0、Reference1（`^%d+$` かつ 1 以上）、`debug.scene_at(file, n)` → `co_exec_registered`。
- `REG.X = REG.X or function` で、作者のハンドラを上書きしない（Requirement 5.3 は今のまま）。

### 1.2 ローカル変数の置き場と届く範囲

- `act.var` は `ACT.new` が作る act ごとの表（`pasta/act.lua` 201〜211 行）。生成するシーンの関数は `local save, var = act:init_scene(...)` で受け取り（`code_gen/scope_gen.rs` 205〜270 行）、`＄x` は `var.x` になる（`code_gen/element_gen.rs` 36〜40 行）。`init_scene` は `self.var` を作り直さずに返すだけ（`act.lua` 221〜233 行）。
- `EVENT.fire` は、ハンドラを呼ぶ前に `create_act(req)` で act を作り、`transfer_req_to_var`（`＄r0`〜`＄r9`・`＄ｒ０`〜・`＄req_id`・`＄req_base_id`）と `transfer_date_to_var`（日時の変数）を入れる（`shiori/event/init.lua` 99〜108 行、`shiori/act.lua` 155〜238 行）。返ったコルーチンは **同じ act** で再開する（`EVENT.drive(result, act, act)`、init.lua 323 行付近）。
- `＞` の呼び出しは `act:call` → `handler(self, ...)`（`act.lua` 725 行〜）で同じ act を渡す。ローカルシーンも同じ。チェイントークの続きは、そのコルーチンが自分の act のまま再開する。
- **結論**: ハンドラの中で、検査を通った値を `act.var` に入れてからコルーチンを返せば、Requirement 1.1〜1.4・1.7 はそのまま満たせる。届く範囲は「シーンの最初の行の前に代入した」ときと同じになる。転記の後に入れるので、Reference2 が `＄r0` や日時の変数に勝つ（1.7）。なお Reference2 の文字列そのものも `＄r2`・`＄ｒ２` に転記される（害は無い）。

### 1.3 `@toml`

- `crates/pasta_lua/src/runtime/module_registry.rs` 165〜190 行。`decode(text)` は `toml::from_str::<toml::Table>`（文書）。読めなければ `nil, message`。文字列でない引数は Lua のエラー。いつも登録される（`runtime/mod.rs` 166 行）。
- 変換（241〜263 行）: 文字列 → 文字列、整数 → `f64`（Lua の数）、浮動小数点数 → 数、真偽 → 真偽、**日付・時刻 → `to_string()` の文字列**、配列・表 → Lua の表。
- スクラッチで toml 1.1.8 を確かめた結果:

| 入力（`v = ` に続けたもの） | 結果 |
| --------------------------- | ---- |
| `{ 名前 = "ぱすた", 回数 = 3 }` | 誤り（column 7） |
| `{ "名前" = "ぱすた" }` | 読める |
| `{ a = 1, }`（末尾のカンマ） | 読める（TOML 1.1） |
| `{` 改行 `a = 1` 改行 `}` | 読める（TOML 1.1） |
| `{ a.b = 1 }` | 読める（`a` が表になる） |
| `{ 1 = 2 }` | 読める（キーは `"1"`） |
| `{ t = 1979-05-27 }` | 読める（Datetime） |
| `{ a = 1 } x` | 誤り |
| `3` | 読める（表でない） |

- `v = ` に包んで読むので、検査は「文書に `v` だけがあり、`v` が表であること」になる。SHIORI の Reference は 1 行なので、改行で別のキーを足される心配はほぼ無いが、`v` 以外のキーがあれば誤りにすれば足りる。

### 1.4 失敗の知らせ方

- `pasta/failure.lua`: `FAILURE.reply(act, { label, subject, detail, warning })` が、表記（`balloon_warning` に従う）とログの警告を出して `act:build()` を返す。エスケープ（`\`・`%`・改行）は `FAILURE.escape`（Requirement 4.7・4.8 は今の出口を使えば満たせる）。
- ラベル（`FAILURE.LABEL`）: `BAD_NAME`「名前にならない値」・`NOT_ASSIGNABLE`「代入できない値」（`OnPastaSetSaveVariable` が使う）はある。「TOML として読めない」に当たるラベルは無い（新設が要る。要件では「表にできない値」で仮置き）。
- イベント固有のラベル（`代入できない値`・`型の名前にならない値`）は `book/src/reference/runtime-failures.md` には載っておらず、`shiori-events.md` の各節の誤りの表にだけある。前例に従えば、新しいラベルも `shiori-events.md` だけでよい。

### 1.5 変数の名前の決まり

- `crates/pasta_dsl/src/parser/grammar.pest`: `id = !(reserved_id) ~ (XID_START | "_") ~ XID_CONTINUE*`、`reserved_id = "__" ~ (XID_CONTINUE で "_" 以外)* ~ "__"`、`var_id = id | digit_id`、`digit = ASCII_DIGIT | '０'..'９'`。数字だけの `＄０` はシーン引数（`args[n]`）で、`var` ではない。
- `pasta_lua` は `pasta_dsl` に依存している（`Cargo.toml` 17 行）。`pasta_dsl::parser::PastaParser2` は公開。
- 前例: `OnPastaSetSaveVariable` は名前を区切らず、検査もしない（`external_control.lua` 128 行）。この spec は brief で「変数の名前にならない」を誤りにすると決めているので、前例とは違う。

### 1.6 テスト

- Lua の仕様のテスト: `crates/pasta_lua/tests/lua_specs/scene_play_event_test.lua`（432 行。228 行〜が既定のハンドラの門・引数の誤り・見つからない・再生・`EVENT.fire` を通す）。差し替えの act と `call(event, refs, opts)` の手助けがある。
- **ギャップ**: Lua の仕様のテストを走らせる `tests/lua_unittest_runner.rs` は `@toml` を登録していない（`@pasta_persistence`・`@pasta_search`・`@pasta_sakura_script`・`@pasta_log` だけ）。`register_toml_module` は `pub(crate)`。Lua の仕様のテストで Reference2 を読むには、ランナーに `@toml` を足す（公開の入口が要る）か、テストで差し替えるか、Rust の結合テスト（`tests/scene_play_event_test.rs`。本物のランタイムで辞書を読み込んで `request(...)` する）で確かめる。
- Rust の結合テスト `tests/scene_play_event_test.rs`（593 行）は、本物のランタイムで「名前で鳴らす・位置で鳴らす・無効なら 204・見つからない」を確かめている。値が `＄名前` で読めること（Requirement 1.2・1.3）を端から端まで確かめるにはここが合う。

### 1.7 マニュアルと生成物

- `book/src/lua/shiori-events.md`: `OnPastaScene`（668 行〜）・`OnPastaSceneAt`（703 行〜）・シーンを再生するイベントの共通の決まり（730 行〜）・MCP からの呼び方（796 行〜）。
- `book/tools/gen-skill-refs.mjs` が `lua/shiori-events.md` → `.claude/skills/pasta-lua-coding/references/shiori-events.md` を作る。CI（`manual.yml`）が `--check` で照合する。
- `book/src/internals/debug.md` 108 行が、ハンドラの中身（名前 → `co_exec_exact`、位置 → `scene_at` → `co_exec_registered`）を書いている。Reference2 の 1 文を足すのが自然。
- `book/src/debug/dev-actions.md` は VS Code の使い方。Reference2 は VS Code から送らないので触らない（並走の `vscode-scene-outline-play` がここを触る）。

## 2. 要件ごとの対応表

| 要件 | 今ある資産 | ギャップ |
| ---- | ---------- | -------- |
| 1.1〜1.4 値を入れて始める・届く範囲 | `act.var`、同じ act での再開、`act:call` | 無し（ハンドラで入れるだけ） |
| 1.5 書いたとおりの名前 | `var.名前` は揃えない | 無し |
| 1.6 空の表 | — | 無し |
| 1.7 転記された変数に勝つ | 転記はハンドラの前 | 無し（仮置き。OPEN QUESTION 4） |
| 2.1〜2.3 文字列・数・真偽 | `@toml` の変換 | 無し |
| 2.4 日付・時刻 | `@toml` が文字列にする | **Constraint**: 読んだ後では文字列と見分けられない（受けるなら無し、断るなら `@toml` か読み方に手が要る） |
| 2.5 配列・表は断る | Lua の `type(v) == "table"` | 無し |
| 3.1〜3.2 無い・空なら今のまま | 今のハンドラ | 無し（Reference2 が nil・`""` なら今の道を通す） |
| 3.3 Reference1 が空ならグローバル | 今の `""` → nil | 無し |
| 3.4 Reference1 を Reference2 と読まない | 今のハンドラ | 無し |
| 4.1 TOML として読めない | `@toml.decode` の `nil, message` | **Missing**: 包んで読む小さな関数、ラベル 1 つ（新設） |
| 4.2 変数の名前にならない | `grammar.pest` の `id` | **Missing**: 名前の検査。Lua だけでは XID を正しく判定できない |
| 4.3 扱えない型 | `NOT_ASSIGNABLE` | 無し（ラベルは今のものを使える） |
| 4.4 誤りなら 1 つも入れない | — | 検査を全部済ませてから入れる順にするだけ |
| 4.5 1 件だけ | — | **Constraint**: `pairs` の順は決まらない。決まった 1 件にするならキーを並べ替える |
| 4.6 確かめる順 | 今の Reference0 → Reference1 → 検索 | Reference2 の検査を検索の前に挟む |
| 4.7〜4.8 表記の設定・エスケープ | `FAILURE.reply`・`FAILURE.escape` | 無し |
| 5.1〜5.3 門・上書きしない | `allowed`・`REG.X or` | 無し（門の後で読む） |
| 6.1〜6.4 マニュアル・生成物 | `shiori-events.md`、`gen-skill-refs.mjs` | 書き足すだけ |
| 7.1 LuaJIT 2.1 | — | 無し（`goto` や 5.2 以降の機能を使わなければよい） |
| 7.2 今のテスト | 両方のテスト | 無し |
| 7.3 自動のテスト | `scene_play_event_test.lua`・`.rs` | **Missing**: Lua の仕様のテストのランナーに `@toml` が無い |

## 3. 実装の道の候補

### 案 A: `scene_play.lua` だけで済ませる（名前の検査を Lua で近似）

- 2 つのハンドラの、シーンを引く直前に、共通の小さな関数 `read_locals(act, event)` を挟む。`@toml.decode("v = " .. ref2)` → `v` だけ・表であること → キーごとに名前と型を確かめる → 全部通ったら `act.var` へ入れる。
- 名前の検査は Lua のパターンで近似する: 空でない、ASCII の英数字と `_` 以外の ASCII を含まない、ASCII の数字で始まらない、数字だけ（半角・全角）でない、`^__.*__$` でない。ASCII 以外の文字は全部通す。
- ✅ 触るファイルが最も少ない（Lua 1 本＋ラベル 1 つ）。Rust に触らない。
- ❌ `"名前、"`（読点）や `"★"` のような、辞書では `＄` の後に書けない名前を通してしまう。Requirement 4.2 の「辞書で書ける名前」と食い違う。

### 案 B: 名前の検査だけ Rust の DSL の文法に任せる（推奨候補）

- 案 A と同じく `scene_play.lua` で読み、名前の検査だけを Rust の小さな関数（たとえば `@pasta_debug.is_local_var_name(s)`）にする。中身は `pasta_dsl` の `PastaParser2` で `id` の規則に全体が当たるかを見る（`digit_id` は断る）。
- ✅ 辞書の決まりと 1 か所で揃う（文法を変えれば検査も変わる）。`@pasta_debug` はデバッグの門の内側でしか使わないので、置き場が合う。
- ❌ Rust の関数 1 つとそのテストが増える。`@pasta_debug` を差し替えている Lua の仕様のテストの手助け（`make_debug`）に、その関数を足す必要がある。
- 補足: 文法の `id` 規則を外から呼べるか（`Rule` が公開か、`id` が `@{}` の原子規則として単独で呼べるか）は設計で確かめる。呼べなければ `unicode-ident` 相当の判定を `pasta_dsl` に置くことになる（依存は増やさず、pest の XID と同じ表が要る）。

### 案 C: シーンの入口（`scene.lua` か `code_gen`）で受け取る

- `co_exec_exact`・`co_exec_registered` に `locals` の引数を足し、`SCENE.wrap` の中で `act.var` へ入れる。または生成するシーンの関数の入口で受け取る。
- ✅ 「シーンを始めるときに入れる」を名前どおりに表せる。
- ❌ 同じ act で再開するので、ハンドラで入れるのと結果が変わらない。`scene.lua` は `cue-command-runtime`・`novel-*` が触る（brief の Adjacent）ので、重なりが増える。`code_gen` まで行くと生成物とその照合テストが大きく揺れる。**採る理由が無い**。

## 4. 規模とリスク

- **規模: S**（1〜3 日）。ハンドラに 30〜50 行、ラベル 1 つ、（案 B なら）Rust の関数 1 つ、テスト、マニュアル 1 章。
- **リスク: Low**。今の仕組み（同じ act・`@toml`・`FAILURE.reply`）を使うだけで、Reference2 が無いときの道は変わらない。

## 5. 設計へ持ち越すこと

1. **名前の検査の置き場**（案 A の近似か、案 B の文法そのものか）。要件 4.2 を文字どおり満たすのは案 B。
2. **`@toml` の読み方**: `v = ` に包んで、`v` 以外のキーがあれば誤りにする。読めなかった理由（toml の誤りの文は複数行でキャレット付き）をログの警告にどこまで載せるか（1 行目だけにするか）。
3. **決まった 1 件**（4.5）: 複数の誤りがあるとき、キーを並べ替えて最初の 1 件にするか。テストを決まった形にするなら並べ替える。
4. **Lua の仕様のテストで `@toml` を使えるようにするか**: ランナーに登録する（`register_toml_module` を公開する入口が要る）か、Rust の結合テストだけで端から端まで確かめるか。
5. **ログの警告の文言**（OPEN QUESTION 5 の決着の後）。今の形 `<イベント>: bad argument: ReferenceN='…'` に揃えるのが自然。
6. 大きい整数（2^53 を超える）は `f64` になって丸まる。`@pasta_config` と同じ変換なので、マニュアルに書くかだけ決める。
7. TOML の基本文字列では `\` がエスケープになるので、Windows のパスなどはリテラル文字列（`'C:\foo'`）で書くよう、マニュアルの例で示すか。

## 6. 触る見込みのファイル

- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/scene_play.lua`（本体）
- `crates/pasta_lua/pasta_scripts/pasta/failure.lua`（ラベル 1 つ）
- 案 B のとき: `crates/pasta_lua/src/runtime/module_registry.rs`（`@pasta_debug` に関数 1 つ）と、必要なら `crates/pasta_dsl/src/parser/`（名前の判定の公開）
- `crates/pasta_lua/tests/lua_specs/scene_play_event_test.lua`、`crates/pasta_lua/tests/scene_play_event_test.rs`、必要なら `crates/pasta_lua/tests/lua_unittest_runner.rs`
- `book/src/lua/shiori-events.md`、`book/src/internals/debug.md`（1 文）
- `.claude/skills/pasta-lua-coding/references/shiori-events.md`（`node book/tools/gen-skill-refs.mjs` で作り直すだけ。手では書かない）
- 触らない: `crates/pasta_lua/pasta_scripts/pasta/scene.lua`、`crates/pasta_lua/src/code_gen/`、`crates/pasta_lsp/`、`editors/vscode/`、`book/src/debug/dev-actions.md`

---

# 設計フェーズの調査と決定（2026-11-12、`/kiro-spec-design -y`）

## Summary

- **Feature**: `scene-play-locals`
- **Discovery Scope**: Extension（既存のハンドラの拡張。軽い調査 `design-discovery-light` で足りる。外部の新しい依存は無い）
- **Key Findings**:
  - 文法の `id` 規則は、`pasta_dsl` の外から単独で呼べる。`pasta_dsl::parser::{PastaParser2, Rule}` は公開で、`crates/pasta_dsl/tests/choice_line_test.rs` が `PastaParser2::parse(Rule::choice_line, …)` を外から呼んでいる。`id` は `@{}` の原子規則なので、当たった範囲（`as_str()`）が入力の全体と同じ長さかで「全体が名前か」を判定できる（pest の `parse` は入力の先頭だけ当たっても `Ok` を返すため、長さの比較が要る）。
  - `id` 規則だけで、要件 4.2 の例は全部はじける。数字（半角・全角）は XID_START でないので先頭に来られず、`__名前__` は `reserved_id` の否定先読みで、空白・`-`・`.`・`、`・`★` は全体に当たらないことで落ちる。`digit_id` を別に断る必要は無い。
  - toml 1.1.8 の誤りの文は 1 行目が `TOML parse error at line 1, column N`、次に入力の抜き出しとキャレット、最後の行が理由（``invalid unquoted key, expected letters, numbers, `-`, `_` ``・``unexpected key or value, expected newline, `#` ``・`duplicate key`・``unclosed inline table, expected `}` ``）。`v = ` に包むので列は 4 ずれる。1 行の警告には最後の空でない行（理由）だけを載せるのが読みやすい。

## Research Log

### 文法の `id` 規則を外から呼べるか

- **Context**: 要件 4.2 の「辞書で `＄名前` と書ける名前」を、辞書の決まりと 1 か所で揃えたい（ギャップ分析の案 B）。
- **Sources Consulted**: `crates/pasta_dsl/src/parser/grammar.pest` 16〜22 行、`crates/pasta_dsl/src/parser/mod.rs` 45〜62 行、`crates/pasta_dsl/tests/choice_line_test.rs` 1〜20 行、`crates/pasta_lua/Cargo.toml`（`pest` に直接は依存しない）。
- **Findings**:
  - `#[derive(Parser)] pub struct PastaParser2` が `Rule` を同じモジュールに作り、`pub use` 無しで `pasta_dsl::parser::Rule` として外から見える。
  - `pasta_lua` から直接呼ぶには `pest::Parser` のトレイトが要り、`pasta_lua` の `Cargo.toml` に `pest` を足すことになる。`pasta_dsl` に `pub fn` を 1 つ置けば、`pasta_lua` の依存は変わらない。
- **Implications**: 判定は `pasta_dsl::parser::is_id(s: &str) -> bool` に置き、`@pasta_debug` から呼ぶ。

### toml の誤りの文とインライン表の読み方

- **Context**: ギャップ分析の持ち越し 2（`v = ` に包む読み方と、ログに載せる理由の量）。
- **Sources Consulted**: スクラッチ（toml = "=1.1.8"、`toml::from_str::<toml::Table>`）で 10 通りを確かめた。
- **Findings**:

| 入力（`v = ` に続けたもの） | 結果 |
| --------------------------- | ---- |
| `{ 名前 = "ぱすた" }` | 誤り。理由の行は ``invalid unquoted key, expected letters, numbers, `-`, `_` `` |
| `{ a = 1 } x` | 誤り。``unexpected key or value, expected newline, `#` `` |
| `{ a = 1, a = 2 }` | 誤り。`duplicate key`（同じキーを 2 度書くと読めない） |
| `{ a = 1` | 誤り。``unclosed inline table, expected `}` `` |
| `3` | 読める（`v` が表でない） |
| `{ a = [1] }` | 読める（`a` が配列） |
| `{ "a" = 'C:\foo' }` | 読める（リテラル文字列。`C:\foo` のまま） |
| `{ "a" = "C:\foo" }` | 読める（基本文字列。`\f` が改ページ文字になり `C:<FF>oo`） |
| `{ a = 9007199254740993 }` | 読める（整数。`@toml` が `f64` にすると 9007199254740992 に丸まる） |
| `{ a = 1 }` 改行 `b = 2` | 読める（文書に `v` と `b` の 2 つ） |

- **Implications**:
  - 「インライン表 1 つ」は、文書のキーが `v` だけで、`v` が表であることで確かめる。`b = 2` のように別のキーが増えた文書も表にできない値の誤りにする。
  - 基本文字列の `\` はエスケープとして読まれ、`\f` のように誤りにならずに別の文字になることがある。マニュアルの例では `\` を含む値をリテラル文字列（`'…'`）で書く。
  - 理由の行は `[^\r\n]+` の最後の 1 つを取れば得られる。

### 届く範囲（ギャップ分析 1.2 の確かめ直し）

- `EVENT.fire` は `create_act(req)` の中で `transfer_req_to_var`・`transfer_date_to_var` を済ませてからハンドラを呼ぶ（`shiori/event/init.lua` 99〜108 行）。返したコルーチンは同じ act で再開する（323 行）。ハンドラで `act.var` に入れれば、転記の後で、シーンの最初の行の前になる。
- 生成するコードの `＄x` は `var.x`（`code_gen/element_gen.rs` 36〜40 行）。LuaJIT は 0x80 以上のバイトを名前に使えるので、`act.var["名前"]` と `var.名前` は同じ項目になる。
- `scene.lua` は `act.var` を作り直さない（179・192 行で読むだけ）。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
| ------ | ----------- | --------- | ------------------- | ----- |
| A: Lua で近似 | 名前の検査を Lua のパターンで近似 | Rust に触らない | `"名前、"`・`"★"` を通す。要件 4.2 と食い違う | 不採用 |
| B: 文法の `id` に任せる | `pasta_dsl::parser::is_id` を `@pasta_debug.is_local_var_name` から呼ぶ | 辞書の決まりと 1 か所で揃う。依存は増えない | Rust の関数 2 つ（pasta_dsl に 1・登録に 1） | **採用** |
| C: シーンの入口で受ける | `scene.lua`・`code_gen` に `locals` を通す | 名前どおりの形 | 結果はハンドラで入れるのと同じ。他 spec と重なる | 不採用（ギャップ分析のとおり） |

## Design Decisions

### Decision: 名前の検査は `pasta_dsl` の `id` 規則に任せる

- **Context**: 要件 4.2。
- **Alternatives Considered**: 1. Lua の近似（案 A） 2. `pasta_lua` に `pest` を足して `Rule::id` を直接呼ぶ 3. `pasta_dsl` に `pub fn is_id` を置く
- **Selected Approach**: 3。`@pasta_debug.is_local_var_name(s)` がそれを呼ぶ。
- **Rationale**: 文法の持ち主の中に置けば、文法を変えたときに検査も一緒に変わる。`pasta_lua` の依存は変わらない。`@pasta_debug` はデバッグの門の内側でしか使わない内部モジュールなので、置き場が合う。
- **Trade-offs**: 公開の関数が `pasta_dsl` に 1 つ増える（足すだけで、壊す変更ではない）。
- **Follow-up**: `pasta_dsl` の単体テストで、要件 4.2 の例と、通るべき名前（`名前`・`_x`・`x1`・`r0`・`ｒ０`）を確かめる。

### Decision: 読み取りと検査は `scene_play.lua` の局所関数 1 つ

- **Context**: 2 つのハンドラが同じ Reference2 を同じ規則で読む。
- **Selected Approach**: `read_locals(act, event, debug)` を `scene_play.lua` に置き、2 つのハンドラが、シーンを引く直前に呼ぶ。値を `act.var` に入れるのは、コルーチンが得られた後だけ。
- **Rationale**: 新しいモジュールを作るほどの量ではない（30 行ほど）。`scene.lua`・`code_gen` に触らない。
- **Trade-offs**: 無し。

### Decision: 誤りが複数あるときはキーを並べ替えて最初の 1 件

- **Context**: 要件 4.5。`pairs` の順は決まらない。
- **Selected Approach**: キーを `table.sort`（バイト順）で並べ、キーごとに「名前 → 型」の順で確かめ、最初の誤りを知らせる。
- **Rationale**: 送り直したときに同じ誤りが出る。テストが決まった形になる。

### Decision: ログの警告は今の `bad argument` の形に揃える

- **Selected Approach**:
  - 表にできない値: `<イベント名>: bad argument: Reference2='<届いた文字列>' (<理由>)`。理由は toml の誤りの文の最後の空でない行、表でないときと別のキーがあるときは `not an inline table`。
  - 名前にならない値: `<イベント名>: bad argument: Reference2 key='<キー>' (not a local variable name)`
  - 代入できない値: `<イベント名>: bad argument: Reference2 key='<キー>' (table is not assignable)`
- **Rationale**: 今の `Reference0=''`・`Reference1='４２'` と同じく、引数の名前と引用符で包んだ値を先に置く。toml の位置の行は `v = ` の分だけ列がずれて誤解を招くので載せない。

### Decision: 自動のテストは Rust の結合テストと `pasta_dsl` の単体テスト

- **Context**: 要件 7.3。Lua の仕様のテストのランナーは `@toml` を登録していない（`register_toml_module` は `pub(crate)`）。
- **Alternatives Considered**: 1. ランナーに `@toml` を足す（`pasta_lua` の公開の入口を 1 つ増やす） 2. Lua の仕様のテストで `@toml` を差し替える（本物の toml を通らない） 3. 本物のランタイムを使う結合テスト `tests/scene_play_event_test.rs` に足す
- **Selected Approach**: 3 と、`pasta_dsl` の単体テスト。
- **Rationale**: 本物の `@toml`・本物の `@pasta_debug`・生成したシーンの `＄名前` を一度に通る。`pasta_lua` の公開の口を増やさない。警告の文言は、`package.loaded["@pasta_log"]` を集める表に差し替えて確かめる（`FAILURE`・門は呼んだ時点で `require` する）。

## Synthesis

- **Generalization**: 2 つのハンドラの Reference2 は同じ問題なので、読み取りを 1 つの局所関数にまとめる。`OnPastaCallFunction` など他のイベントへ広げる口は作らない（要件に無い）。
- **Build vs. Adopt**: TOML の読み取りは既存の `@toml.decode`、名前の判定は既存の文法の `id` 規則、失敗の出口は既存の `FAILURE.reply`。新しく作るのは、`id` 規則を文字列の全体に当てる 1 行の関数と、その登録だけ。
- **Simplification**: `scene.lua`・`code_gen`・Lua の仕様のテストのランナーに触らない。ラベルの新設は「表にできない値」1 つだけ。配列と表は Lua では見分けないので、どちらも `（表）`。

## 参照したスキル

- `kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）
- ponytail（最小の完全な変更）: 新しいモジュール・依存・抽象を足さない判断に使った。

## Risks & Mitigations

- `.claude/skills/pasta-lua-coding/references/shiori-events.md` の作り直しは、`.claude/skills` への書き込みとして分類器に拒まれることがある — 生成は `node book/tools/gen-skill-refs.mjs` だけで行い、拒まれたらそこで止めて開発者に許可を求める。
- 結合テストのフィクスチャ（`FIXTURE`）に足すシーンが、今のテストの行番号（`LINE_*`）を動かす — 末尾にだけ足す。
- `＄end` のような Lua の予約語の名前は `id` 規則を通るが、辞書の側でも同じ（`var.end`）で、この spec の外 — 触らない。
