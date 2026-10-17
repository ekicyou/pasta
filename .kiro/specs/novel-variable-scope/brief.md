# Brief: novel-variable-scope

2026-10-17 の棚卸で `novel-checkpoint-rollback` から分割（main `b4164100`）。元の brief の「変数の保存の範囲」の議題（要件の 1 番と 8 番）と、そのための文法・コード生成・保存の分をこちらへ移した。0.5.0「areka でノベルゲームが作れる」の道筋に入る。

## Problem

ノベルゲームには、ロードや巻き戻しで戻ってほしい変数（好感度・選んだ道）と、戻ってほしくない変数（クリアした印・見たエンディング）がある。今の pasta の変数で保存されるのは `＄＊x` の 1 種類だけで、全部が 1 つの表に入り、1 つのファイルへ書かれる。どちらの範囲かを書き分ける方法が無い。

一方で、今のゴーストは `＄＊x` を「終了しても消えない記憶」として使っている。入門ガイドもそう教えている。ここを変えると、今のゴーストの保存の形に響き、取り消しにくい。

## Current State

- `＄＊x` は、生成した Lua で `save.x` になる（`crates/pasta_lua/src/code_gen/element_gen.rs` 39・71 行）。文法の規則は、読む `var_ref_global`・書く `var_set_global`・名前を変数で選ぶ `dyn_name_global`（`crates/pasta_dsl/src/parser/grammar.pest` 84・95・107 行）。パーサでは `VarScope::Global`（構文木 `parser/ast/action.rs` 284 行、`parse_elements.rs` 119 行、`parse_action.rs` 111・212・369 行、`partial.rs` 462 行）。
- `save` は `pasta.save` の表 1 つである（`crates/pasta_lua/pasta_scripts/pasta/save.lua`。最初に読み込まれたとき `@pasta_persistence.load()` でファイルから作る）。
- ランタイムを捨てるとき（ゴーストの終了・辞書の再読み込み）に、この表を丸ごと JSON にして `profile/pasta/save/save.json` へ書く（`crates/pasta_lua/src/runtime/lifecycle.rs`）。Lua から `@pasta_persistence.save(表)` で書くこともできる（`crates/pasta_lua/src/runtime/persistence.rs` の 145〜157 行で登録し、書く処理は 202〜231 行）。保存先と難読化は `pasta.toml` の `[persistence]` で変えられる。
- **表の同一性**: 同じ表を、何か所もが握っている。act の `save`（`pasta_scripts/pasta/act.lua` 189 行）、場面の関数の頭の `local save, var = act:init_scene(…)`（`act.lua` 219 行の `return self.save, self.var`、生成は `code_gen/scope_gen.rs` 294 行）、トーク間隔の設定を読む所（`shiori/event/virtual_dispatcher.lua` 46 行）、終了時の保存。表を取り替えると、握っている側が古い表を見続ける。
- **遊びの進み具合でない値も入っている**: エンジンが予約した `＄＊pasta_talk_interval_min`・`＄＊pasta_talk_interval_max`（`virtual_dispatcher.lua` 72〜73 行。マニュアル `book/src/grammar/variables.md` の「予約グローバル変数」）。
- **今の使われ方**: 入門ガイド 10 段目（`book/src/getting-started/10-save.md`）が、`＄＊回数` を「終了しても値が残る変数」として教えている。マニュアルの変数の章（`book/src/grammar/variables.md` の「グローバル変数」48 行・「グローバル変数の保存先」316 行）と、`book/src/lua/modules/pasta-persistence.md` も同じ形で書いている。
- **保存できない値**: 表の中に 1 つでも JSON にできない値（関数・コルーチンなど）があると、終了時の保存がまるごと失敗し、ログに 1 行出るだけで、その回の値はすべて残らない（`lifecycle.rs` 40〜43・59〜61 行）。明示の保存も、失敗を返すだけである（`persistence.rs` 209〜216 行）。
- 変数の名前を表示に戻す所がある: 止まるエラーや値が無いときの表記で、`save.x` を `＄＊x` と書き戻す（`pasta_scripts/pasta/failure.lua` 71・77 行）。辞書の検査も `VarScope::Global` を見る（`crates/pasta_check/src/validate/transpile_rules.rs` 87 行）。
- エディタ: VSCode の着色が `＄＊` を変数として塗る（`editors/vscode/syntaxes/pasta.tmLanguage.json` 185・197 行）。LSP が `＄＊名前` の位置を探す（`crates/pasta_lsp/src/analysis/visit_expr.rs` 47・88・194・247 行）。

## Desired Outcome

- 変数に、保存の範囲が 2 つある。**セーブごと**（ロードと巻き戻しで戻る）と、**全セーブ共通**（戻らない）。作者は、変数ごとにどちらかを書き分けられる。
- 今のゴーストの `＄＊x` は、辞書を書き直さなくても、今までどおり終了しても値が残る。今の `save.json` も、そのまま読める。
- ノベル向けの設定でないとき、出力と保存の挙動を変えない。
- 後の spec が、2 つの範囲を別々に扱える口がある。巻き戻し（`novel-checkpoint-rollback`）は、セーブごとの分だけを控え、表を取り替えずに中身を入れ替える。セーブのスロット（`novel-save-slots`）は、セーブごとの分をスロットへ書く。
- 保存できない値を入れたとき、どうなるかが決まっていて、作者に分かる。1 つの値のせいで、黙って全部が残らない、ということが起きない。

## Approach

迷いどころの例（今の書き方だけで書いた。もう一方の書き方は未定）:

```pasta
＊OnBoot
    ＄＊起動回数＝＄＊起動回数＋１
＊第一章
    ＄＊好感度＝＄＊好感度＋１
    ＄＊クリアした＝１
```

今のゴーストの `＄＊起動回数` は、消えない記憶である。ノベルの `＄＊好感度` は、ロードで戻ってほしい。`＄＊クリアした` は、ロードしても消えてほしくない。3 つとも同じ `＄＊` で書いてあるが、望む動きは 2 通りある。

要件定義で決めること。どれも互いに絡み、推奨案はまだ無い。

1. **書き方**（最初の議題）: 今の `＄＊x` を、セーブごとにするか、全セーブ共通にするか。もう一方をどう書くか（新しい印を足す・名前の決まりで分ける・設定ファイルに名前を並べる、など）。文法を変えるなら、エディタの着色・LSP・辞書の検査・表記の書き戻しまで揃える。
2. **今のゴーストとの両立**: ノベル向けの設定でないときは、2 つを区別しないでよいか。今の `save.json` の形を変えるなら、古いファイルをどう読むか。
3. **どちらに入るか決まっている物**: エンジンが予約した `pasta_` で始まる名前（トーク間隔などの設定）。既読（`novel-talk-flow`。全セーブ共通）を同じ仕組みに置くか。
4. **表の形**: 1 つの表の中で 2 つに分けるか、表を 2 つにするか。2 つにするなら、場面の関数に渡す表が増え、`act.lua` の `init_scene` と `scope_gen.rs` に触る。Lua から `require("pasta.save")` で表を読んでいる今のスクリプトを、壊さないこと。
5. **保存できない値**: 変数へ入れられる型を絞るか。保存のときに、その値だけを飛ばして残りを書くか。作者にどう知らせるか（`pasta/failure.lua` の表記に乗せるか、ログだけか）。
6. **保存の時機**: 今の「終了時と明示の呼び出し」を、2 つの範囲でも同じにするか。全セーブ共通の分を書いたらすぐ保存するか（`novel-save-slots` の議題）と、どこで線を引くか。

## Scope

- **In**: 2 つの範囲の書き方（文法・パーサ・コード生成）。今の `＄＊x` と `save.json` との両立。2 つの範囲を分けて持ち、中身を入れ替えられる口（`save.lua`）。保存できない値の扱い。今の 1 つのファイルへの保存（終了時・明示）を 2 つの範囲に合わせること。予約の `pasta_` の名前の置き場所。エディタの着色・LSP・辞書の検査・表記の書き戻し。マニュアル（変数の章・保存の Lua の口・入門 10 段目）、スキル references の再生成、テスト。
- **Out**: 区切りごとの控えと巻き戻し（`novel-checkpoint-rollback`。この spec の口を使う）。スロットへの読み書きと、全セーブ共通の分を書く時機の作り込み（`novel-save-slots`）。既読のための変わらない行 ID（`novel-talk-flow`）。

## Boundary Candidates

- 書き方（`grammar.pest`・パーサ・構文木・`partial.rs`）。
- コード生成（`element_gen.rs`。表を 2 つにするなら `scope_gen.rs` と `act.lua` の `init_scene` も）。
- 保存の表と永続化（`save.lua`・`persistence.rs`・`lifecycle.rs`。予約の名前を動かすなら `virtual_dispatcher.lua` 46 行）。
- エディタと検査と表記（`pasta.tmLanguage.json`・`visit_expr.rs`・`transpile_rules.rs`・`failure.lua`）。
- マニュアル。

## Out of Boundary

- 控えを取る・中身を入れ替える時機（`novel-checkpoint-rollback`）。
- スロットのファイルの名前・形・一覧（`novel-save-slots`）。
- CG 鑑賞・回想・音楽室。全セーブ共通の変数があれば作者が組めるので、専用の仕組みは作らない（2026-10-10 の決定）。

## Upstream / Downstream

- **Upstream**: なし。areka は変数を見ないので、`novel-areka-contract` の名前も要らない。`novel-talk-flow` も待たない。
- **Downstream**: `novel-checkpoint-rollback`（控えに入れるのはセーブごとの分だけ）、`novel-save-slots`（セーブごとの分をスロットへ、全セーブ共通の分をスロットの外へ）、`hello-novel-sample`（フラグと、クリアした印）、`manual-novel-guide`。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `call-attribute-filter`（条件の値に変数を書く。`＞告白＆好感度＝＄＊好感度`）、`.kiro/specs/completed/store-save-table/`・`.kiro/specs/completed/store-save-persistence/`（`save` の表と終了時の保存の元の設計）、`.kiro/specs/completed/talk-frequency-persistence/`（予約の `pasta_` の名前）、`.kiro/specs/completed/pasta-config-restructure/`（`[persistence]` の設定）、`.kiro/specs/completed/getting-started-story-guide/`（入門 10 段目）。

## Constraints

- 今のゴーストの `＄＊x` を、書き直しなしで今までどおり動かす。今の `save.json` を読めなくしない。
- 全セーブ共通の変数は、巻き戻さない。
- 表を取り替えない。中身を入れ替える。
- 文法を変えるなら、マニュアルが文法の唯一の権威である。マニュアル・エディタ・LSP・辞書の検査を同じ spec で揃える。
- 書き込みは、今と同じく、一時ファイルへ書いてから名前を替える。

## 測定（2026-10-17 の棚卸、main b4164100）

- **触るファイル**: 文法 `crates/pasta_dsl/src/parser/grammar.pest`（272 行。84・95・107 行）、パーサ `parse_elements.rs`（200）・`parse_action.rs`（456）・構文木 `parser/ast/action.rs`（`VarScope` 284 行）・`partial.rs`（477）、コード生成 `crates/pasta_lua/src/code_gen/element_gen.rs`（564。39・71 行）、`pasta_scripts/pasta/save.lua`（13）、`crates/pasta_lua/src/runtime/persistence.rs`（568）・`lifecycle.rs`（63）、表記の書き戻し `pasta/failure.lua`（233。71・77 行）、辞書の検査 `crates/pasta_check/src/validate/transpile_rules.rs`（188）、LSP `crates/pasta_lsp/src/analysis/visit_expr.rs`（483）、VSCode の着色 `editors/vscode/syntaxes/pasta.tmLanguage.json`（272）、マニュアル `book/src/grammar/variables.md`（421）・`book/src/lua/modules/pasta-persistence.md`（136）・`book/src/getting-started/10-save.md`（122）と生成スキル、テスト。表を 2 つにするなら `act.lua`（829。189・219 行）と `code_gen/scope_gen.rs`（445。294 行）、予約の名前を動かすなら `shiori/event/virtual_dispatcher.lua`（288。46 行）も。1,000 行に近いものは無い。
- **規模**: 8〜10 タスク（書き方の文法とパーサ 1〜2、コード生成 1、2 つの範囲の表と入れ替えの口 1〜2、永続化と保存できない値 1〜2、エディタ・検査・表記 1、マニュアル 1〜2、テスト 1）。
- **先に要るもの**: なし。`novel-talk-flow` を待たないので、`novel-checkpoint-rollback` より先に、もっと早く始められる。ファイルの重なり: 文法とパーサは `scene-attribute-store`・`cue-command-runtime`・`call-attribute-filter`・`scene-anchor-link`、`element_gen.rs` は `scene-attribute-store`・`call-attribute-filter`・`scene-anchor-link`、VSCode の着色と LSP は `call-attribute-filter`・`scene-anchor-link`、`save.lua`・`persistence.rs`・`lifecycle.rs` は `novel-save-slots`（後に置く）。表を 2 つにするなら、`act.lua`（1 つのウェーブに 1 つの spec だけ）と `scope_gen.rs`（`scene-attribute-store`・`cue-command-runtime`）とも重なる。`novel-checkpoint-rollback` とは、予約の名前を動かす場合の `virtual_dispatcher.lua` だけが重なりうる（こちらは 46 行、あちらは 288 行。順番が前後なのでぶつからない）。
- **種別**: 機能（保存の範囲を書き分ける方法が無い）。開発者の 0.5.0 の決定（巻き戻しとセーブ）から。
- **要件定義のモデル**: Fable。今のゴーストの `＄＊x` が頼っている保存の形に触り、取り消しにくい。書き方・どちらを今の `＄＊` にするか・両立・何を全セーブ共通にするか・保存できない値が互いに絡み、推奨案がまだ無い。
- **議題**: 「Approach」の 1〜6（書き方、今のゴーストとの両立、どちらに入るか決まっている物、表の形、保存できない値、保存の時機）。1 番が最初で、ほかはその答えに引きずられる。
