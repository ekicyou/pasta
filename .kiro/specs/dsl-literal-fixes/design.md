# Design Document: dsl-literal-fixes

## Overview

**Purpose**: Pasta DSL の文字列リテラルと単語定義の値にある 2 つの不具合（U24・U25）を直し、U06（引用なしの単語値の `＃`）は現行挙動を仕様として確定する。ゴースト作者が書いた引用文字列が、生成 Lua の構文エラーでランタイム全体を起動不能にすることを無くす。

**Users**: ゴースト作者（`.pasta` 辞書を書く人）と、pasta の開発者（再発をテストで検出する人）。

**Impact**: 文法規則 1 つ（`string_contents`）、パーサの分岐 2 箇所（`parse_attr`・`parse_key_words`）、Lua 文字列出力の関数 1 つ（`StringLiteralizer::literalize_with_span`）を変える。新しい部品・抽象・ソースファイルは足さない。足すファイルはテスト 2 つだけである。改行を含む引用文字列は、これまでパースを通っていたものがパースエラーになる（非互換。ただし現行でもランタイムを壊しており、動く辞書には影響しない）。

### Goals
- 改行をまたぐ引用文字列をパースエラーにし、値に改行を入れない（1.1〜1.5）。
- 別々の行の `""` を、それぞれ空文字列にする（2.1〜2.3）。
- 単語値・属性値の `「」`・`""` を空文字列にする（3.1〜3.5）。
- 引用なしの単語値の `#`・`＃` が値の一部である現行挙動を、テストとマニュアルで固定する（4.1〜4.3）。
- 改行を含む文字列値が Lua 出力に渡っても、構文エラーにせず同じ値で読み戻せる（5.1〜5.3）。
- マニュアル 3 章とスキル references を新しい挙動に合わせる（6.1〜6.6）。

### Non-Goals
- 同じ行の中で閉じていない引用（`＠w：「abc`）を引用なしの値として読む現行挙動の変更。
- 同じ行の `""…""` を空文字列 2 つとして読む先読み（要件ディスカッションで却下済み）。
- アクション行・継続行の `＃`、`\\` エスケープ、さくらスクリプトの解析（`sakura_body` の改行またぎを含む）。
- ハイライト文法（`pasta.tmLanguage.json`）の変更。単語定義の行・属性行の行末コメントの色分け。
- コード生成の各アーム（`element_gen.rs`・`scope_gen.rs`）とランタイムの Lua（`act.lua`・`actor.lua`・`word.lua`）の変更。

## Boundary Commitments

### This Spec Owns
- 文法規則 `string_contents` の定義（引用文字列の中身に改行を含めない）。
- `parse_attr`・`parse_key_words` での `Rule::string_blank` の値（空文字列）。
- `StringLiteralizer::literalize_with_span` の出力規則のうち、改行（CR・LF）を含む値の形式。
- 上の 3 点を固定する自動テスト（パーサ・出力・ランタイムの 3 層）。
- マニュアル `book/src/grammar/` の `literals.md`・`words.md`・`block-structure.md` の該当節と、そこから再生成されるスキル references。

### Out of Boundary
- `crates/pasta_lua/src/code_gen/`（`element_gen.rs`・`scope_gen.rs`）。同じ Wave の `dsl-codegen-runtime-safety` が持つ。
- `crates/pasta_lua/pasta_scripts/`（`act.lua`・`actor.lua`・`word.lua`）と `pasta_core` の単語テーブル。空の候補は現行のまま「見つかった」扱いになる見込みで、この仕様はテストで確かめるだけにする。
- `editors/vscode/syntaxes/pasta.tmLanguage.json`。全規則が 1 行の `match` で、引用文字列は行をまたいで色分けされない（7.1・7.2 は現状で満たす）。
- `grammar.pest` の他の規則（`word_nofenced`・`attr_string`・`sakura_body`・`string_blank`・囲みの規則）。
- 属性値の型解釈（下流の `scene-attribute-store` が持つ）。

### Allowed Dependencies
- pest の既存機能（`PEEK`・スタック）。新しいクレートは足さない。
- テストは既存の入口だけを使う: `pasta_dsl::parser::parse_str`、`crates/pasta_lua/tests/common/e2e_helpers.rs` の `transpile`・`create_runtime_with_finalize`・`create_runtime_with_search`、`mlua`。
- マニュアルの検査は既存のツールだけを使う: `book/tools/gen-skill-refs.mjs`、`book/tools/highlight/*-test.mjs`、`link-check.mjs`、`verify-content.mjs`。
- 依存の向きは現行どおり `pasta_dsl` → `pasta_lua`（トランスパイラ → ランタイム）。逆向きの参照を足さない。

### Revalidation Triggers
- `string_contents`・`string_literal`・`string_blank`・`strfence` の定義を変えるとき（この仕様のパーサテストを再確認する）。
- 引用文字列を書ける場所を文法に足すとき（1.4 の文脈一覧とテストに足す）。
- `StringLiteralizer` の出力規則を変えるとき（改行を含まない値の出力が変わると既存スナップショット全体に波及する）。
- 属性値の空文字列の表現（`AttrValue::String("")`）を変えるとき（`scene-attribute-store` が前提にする）。
- 単語検索の「見つかった」判定（`result ~= nil`）を変えるとき（3.4・3.5 のテストを再確認する）。

## Architecture

### Existing Architecture Analysis

処理の流れは `.pasta` → パーサ（`pasta_dsl`、pest 文法）→ AST → トランスパイラ（`pasta_lua`）→ 生成 Lua → `pasta.scene_dic` が各モジュールを `require` → ランタイム、である。`scene_dic` は `require` を並べるだけなので、1 モジュールの構文エラーでロード全体が止まる。

修正点は、この流れの中で既に 1 箇所に集約されている。

| 層 | 集約点 | 経由する呼び出し元 |
| -- | ------ | ------------------ |
| 文法 | `string_contents`（`grammar.pest:123`） | `string_fenced` 経由で式・`key_literal`・単語値・属性値・キューコマンド引数。`choice_label` は直接使う |
| パーサ | `parse_key_words`（`parse_elements.rs`） | グローバル単語（`parser/mod.rs`）・ローカル単語・アクター単語（`parse_scene.rs`）のすべて |
| パーサ | `parse_attr`（`parse_elements.rs`） | 属性行・シーン宣言行への付記 |
| 出力 | `StringLiteralizer::literalize_with_span` | `scope_gen.rs`・`element_gen.rs` のすべての文字列出力 |

式の `「」`（`parse_action.rs` の `Expr::BlankString`）とキューコマンド引数の `「」`（`parse_scene.rs` の `String::new()`）は、既に空文字列になっている。単語値と属性値だけが外れている。

### Architecture Pattern & Boundary Map

**Architecture Integration**:
- 採る方式: 既存の集約点 3 箇所の局所修正（A1 + B1 + C1）。新しい部品は足さない。
- 責任の分け方: 文法は「改行入りの引用を作らない」、パーサは「空の引用を空文字列にする」、出力は「改行入りの値が来ても壊れない」。出力側の修正は文法の修正に依存しない多層防御で、単独で成り立つ。
- 守る既存の形: キューコマンド引数の `Rule::string_blank => String::new()`、`attr_string`・`word_nofenced` の改行除外の書き方、`StringLiteralizer` の「形式を 1 関数で決める」構造。
- ステアリングとの整合: 文法の権威はマニュアル。この変更でマニュアルと実装を一致させる。

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| パーサ | pest（既存） | `string_contents` の否定先読みに `"\r"`・`"\n"` を足す | 新しい依存なし |
| トランスパイラ | Rust（既存） | 改行を含む値を `"…"` 形式でエスケープして出す | Lua の標準エスケープ（`\\`・`\"`・`\r`・`\n`）だけを使う |
| ランタイム | mlua / LuaJIT（既存） | 変更なし。テストでロードと読み戻しを確かめる | |
| マニュアル | mdBook + Node ツール（既存） | 3 章の更新と references の再生成 | |

## File Structure Plan

### 新規ファイル（テストだけ）
```
crates/pasta_dsl/tests/
└── literal_fixes_test.rs        # 要件 1〜4 のパース結果（値・候補・エラー行）を固定する
crates/pasta_lua/tests/runtime/
└── literal_fixes_test.rs        # 生成 Lua のロード・読み戻し・空の候補の実行時挙動を固定する
```

### Modified Files
- `crates/pasta_dsl/src/parser/grammar.pest` — `string_contents` から CR・LF を除く（1 規則）。
- `crates/pasta_dsl/src/parser/parse_elements.rs` — `parse_attr` と `parse_key_words` で `Rule::string_blank` を `Rule::string_contents` の分岐から分け、空文字列にする（2 分岐）。
- `crates/pasta_lua/src/string_literalizer.rs` — 改行を含む値の出力規則を足す。同じファイルの単体テストに出力形のテストを足す。ドキュメントコメントの規則一覧も合わせる。
- `crates/pasta_lua/tests/runtime/main.rs` — `mod literal_fixes_test;` を足す。
- `book/src/grammar/literals.md` — 「文字列（String）」の節（6.1・6.2・6.3）。
- `book/src/grammar/words.md` — グローバル単語定義の「値の書き方」の箇条書き（6.2・6.4）。
- `book/src/grammar/block-structure.md` — コメントの節（6.4）。
- `.claude/skills/pasta-ghost-authoring/references/` の該当ファイル — 手で編集しない。`node book/tools/gen-skill-refs.mjs` で再生成する（6.5）。

変更しないことを明示するファイル: `editors/vscode/syntaxes/pasta.tmLanguage.json`、`crates/pasta_lua/src/code_gen/*`、`crates/pasta_lua/pasta_scripts/**`、既存のスナップショット（`crates/pasta_lua/tests/transpiler/snapshots/`）。

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1 | 行内で閉じない引用を、改行入りの文字列として受理しない | 文法 `string_contents` | 文法規則 | パース |
| 1.2 | 改行をまたぐ引用で文法に合わないファイルは、行番号付きのパースエラー | 文法 `string_contents`、既存の `ParseError::SyntaxError` | 「エラー行の契約」表 | パース |
| 1.3 | 引用文字列の値に CR・LF を含めない | 文法 `string_contents` | 文法規則 | パース |
| 1.4 | 引用を書けるすべての場所に同じく適用 | 文法 `string_contents`（全文脈の集約点） | 「エラー行の契約」表 | パース |
| 1.5 | 行内で閉じた引用は中身をそのまま値にする | 文法 `string_contents`（改行以外は不変） | 文法規則 | パース |
| 2.1 | 別々の行の `""` はそれぞれ空文字列 | 文法 `string_contents`（囲みの分岐が失敗し `string_blank` に落ちる） | 文法規則 | パース |
| 2.2 | `""` の間の行は独立した行 | 文法 `string_contents` | 文法規則 | パース |
| 2.3 | 同じ行の `""…""` は 2 重の囲み（現行維持） | 文法（変更なし）、パーサテスト | — | パース |
| 3.1 | 単語値の `「」`・`""` は空文字列の候補 | `parse_key_words` | `KeyWords.words` に `""` | パース |
| 3.2 | 属性値の `「」`・`""` は空文字列（文字列型） | `parse_attr` | `AttrValue::String("")` | パース |
| 3.3 | どの文脈でも空文字列 | `parse_key_words`・`parse_attr`（式・キュー引数は既存） | — | パース |
| 3.4 | 空の候補が選ばれたら何も出力せず、警告しない | ランタイム（変更なし）、ランタイムテスト | `act:word` が `""` を返す | 実行 |
| 3.5 | 空の候補も他の候補と同じく消費の対象 | ランタイム（変更なし）、ランタイムテスト | 単語検索 | 実行 |
| 4.1 | 引用なしの単語値の `#`・`＃` は値の一部 | 文法（変更なし）、パーサテスト | — | パース |
| 4.2 | 囲んだ値の後ろの `#`・`＃` は行末コメント | 文法（変更なし）、パーサテスト | — | パース |
| 4.3 | グローバル・ローカル・アクター単語で同じ | `parse_key_words`（共通の 1 関数）、パーサテスト | — | パース |
| 5.1 | 改行入りの値を、構文エラーにせず同じ値で読み戻せる形で出す | `StringLiteralizer` | `literalize_with_span` | トランスパイル |
| 5.2 | 改行入りの値を持つ生成 Lua でシーン辞書のロードが失敗しない | `StringLiteralizer`、ランタイムテスト | — | ロード |
| 5.3 | 改行を含まない値の出力は不変 | `StringLiteralizer`（分岐を前に足すだけ） | `literalize_with_span` | トランスパイル |
| 6.1 | 引用は 1 行で閉じる・またぐとパースエラー | マニュアル `literals.md` | — | — |
| 6.2 | `「」`・`""` はどこでも空文字列（`""` の例付き） | マニュアル `literals.md`・`words.md` | — | — |
| 6.3 | 同じ行の `""…""` の注記 | マニュアル `literals.md` | — | — |
| 6.4 | 引用なしの単語値の `#`・`＃` は値の一部 | マニュアル `words.md`・`block-structure.md` | — | — |
| 6.5 | references の再生成と鮮度チェック | `gen-skill-refs.mjs`（既存） | `--check` | — |
| 6.6 | 回避の書き方を載せない | マニュアル 3 章（記述の方針） | — | — |
| 7.1 | 引用文字列の色分けは行末で終わる | ハイライト文法（変更なし。現状で満たす） | — | — |
| 7.2 | マニュアルのハイライトも同じ | ハイライト文法の再利用（変更なし） | — | — |
| 7.3 | 足した例が既存のハイライト検証を壊さない | マニュアルの検査（検証手順） | `book/tools/highlight/*-test.mjs` ほか | — |
| 8.1 | 要件 1〜4 の入力例のパース結果を確かめる | `pasta_dsl/tests/literal_fixes_test.rs` | — | — |
| 8.2 | 改行入りの値と複数の `""` で、生成 Lua のロードが成功する | `pasta_lua/tests/runtime/literal_fixes_test.rs` | — | — |
| 8.3 | 期待値の変更は、この仕様が足す・変える入力に限る | 既存スナップショットを変えない方針 | — | — |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|--------------|--------|--------------|------------------|-----------|
| 文法 `string_contents` | pasta_dsl / 文法 | 引用文字列の中身から改行を除く | 1.1〜1.5、2.1〜2.3 | pest（P0） | 文法規則 |
| `parse_key_words`・`parse_attr` | pasta_dsl / パーサ | 空の引用を空文字列にする | 3.1〜3.3、4.3 | 文法 `string_contents`（P0） | Service |
| `StringLiteralizer` | pasta_lua / 出力 | 改行入りの値を安全な Lua リテラルにする | 5.1〜5.3 | なし | Service |
| マニュアル 3 章と references | book | 新しい挙動を規範として書く | 6.1〜6.6、7.3 | `gen-skill-refs.mjs`（P0） | — |
| テスト 2 ファイル | tests | 修正と確定した挙動を固定する | 8.1〜8.3、3.4、3.5、5.2 | 既存のテストヘルパ（P0） | — |

### pasta_dsl / 文法

#### 文法 `string_contents`

| Field | Detail |
|-------|--------|
| Intent | 引用文字列の中身を「閉じの並びでも改行でもない文字の 1 個以上の並び」にする |
| Requirements | 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3 |

**変更の契約**

```pest
// 変更前
string_contents = @{ (!PEEK ~ ANY)+ }
// 変更後
string_contents = @{ (!(PEEK | "\r" | "\n") ~ ANY)+ }
```

- 変えるのはこの 1 規則だけである。`string_blank`・`strfence`・`string_literal` の並び順は変えない。
- 不変条件: `Rule::string_contents` の値は CR・LF を含まない（1.3）。改行以外の文字（空白・`#`・`＃`・読点・`\`）は現行どおり値に入る（1.5）。
- 別々の行の `""`: 1 つ目の `""` が囲みの開始として試されても、行内に閉じの `""` が無ければ `string_fenced` が失敗し、`string_blank` に落ちる（2.1・2.2）。同じ行に閉じの `""` があれば現行どおり 2 重の囲みになる（2.3）。

**エラー行の契約**（A1 を一時適用して `parse_str` で実測した結果。テストはこの行番号を固定する）

| 文脈 | 入力の例（`⏎` は改行） | エラーを報告する行 |
| ---- | ---------------------- | ------------------ |
| 式（変数代入の右辺） | `＄x＝「a⏎b」`、`＄x＝"a⏎b"` | 引用を開始した行 |
| Call の引数 | `＞t（「a⏎b」）` | 引用を開始した行 |
| キューコマンドの引数 | `＠＠cmd（「a⏎b」）` | 引用を開始した行 |
| 選択肢行の表示テキスト | `＠？t「a⏎b」` | 引用を開始した行 |
| 単語値（グローバル・ローカル） | `＠w：「a⏎b」` | 開始した行の次の行 |
| 属性値 | `＆k：「a⏎b」` | 開始した行の次の行 |
| アクション行の中の関数引数 | `さくら：＠f（「a⏎b」）` | 開始した行の次の行 |

- 単語値・属性値・アクション行では、開始した行が「引用なしの値」または台詞として文法に合うため、エラーは次の行で出る。次の行をインデントしても同じくエラーになる。
- Call の動的ターゲットは実測の表に無い。同じ `string_contents` を通るため同じく改行を含められない。エラー行は実装時に測ってテストに固定する。
- 既知の限界: 次の行が単独で文法に合う場合（例: `＠w：「a`＋改行＋`＠v：b」`）はエラーにならず、2 行とも引用なしの値として読まれる。どの文字列も改行を含まない（1.1・1.3 は成り立つ）。これは要件が範囲外とした「同じ行で閉じていない引用は引用なしの値として読む」挙動の帰結であり、1.2 は「ファイルが文法に合わない場合」を条件にしている。扱いは「未決事項」1 に記録する。

**Implementation Notes**
- エラーの報告は既存の `ParseError::SyntaxError{file,line,column,message}` とローダのログの経路をそのまま使う。メッセージの文言は変えない。
- リスク: 改行入りの引用を書いた既存の辞書がパースエラーになる。現行でもランタイムを壊しているため、動いている辞書には影響しない。

### pasta_dsl / パーサ

#### `parse_key_words`・`parse_attr`

| Field | Detail |
|-------|--------|
| Intent | `Rule::string_blank` を、囲み文字ではなく空文字列として記録する |
| Requirements | 3.1, 3.2, 3.3, 4.3 |

**Contracts**: Service [x]

```rust
pub(crate) fn parse_key_words(pair: Pair<Rule>) -> Result<KeyWords, ParseError>;
pub(crate) fn parse_attr(pair: Pair<Rule>) -> Result<Attr, ParseError>;
```

- シグネチャと AST の型は変えない。変えるのは `Rule::string_blank` の分岐の値だけである。
- 事後条件（単語値）: `「」`・`""` は `KeyWords.words` に `String::new()` として入る。候補の個数と順序は現行どおり。
- 事後条件（属性値）: `「」`・`""` は `AttrValue::String(String::new())` になる（`AttrString` ではない）。
- `Rule::string_contents`・`word_nofenced`・`sakura_script`・`attr_string` の分岐は変えない（4.1・4.2 は文法もパーサも変更なし）。
- `parse_key_words` はグローバル単語・ローカル単語・アクター単語が共有する 1 関数なので、1 箇所の修正が 3 種類すべてに効く。
- 前提: 文法 `string_contents` の修正。これが無いと、2 つ目以降の `""` が囲みとして消費される。

### pasta_lua / 出力

#### `StringLiteralizer`

| Field | Detail |
|-------|--------|
| Intent | 改行（CR・LF）を含む値を、Lua として構文が正しく、同じ値で読み戻せるリテラルにする |
| Requirements | 5.1, 5.2, 5.3 |

**Contracts**: Service [x]

```rust
pub fn literalize_with_span(text: &str, span: &Span) -> Result<String, TranspileError>;
```

出力規則（上から順に判定する）:

| 順 | 条件 | 出力 | 変更 |
| -- | ---- | ---- | ---- |
| 0 | `text` が `\r` または `\n` を含む | `"…"` 形式。`\` → `\\`、`"` → `\"`、CR → `\r`、LF → `\n` に置き換える | 新規 |
| 1 | `\`・`"` を含まない | `"text"` | 不変 |
| 2 | それ以外 | 長い文字列 `[=*[text]=*]` | 不変 |

- 事後条件（規則 0）: 出力を Lua で評価した値は `text` とバイト単位で一致する。出力は 1 行で、生の改行を含まない。規則 0 は失敗しない（`TranspileError` を返さない）。
- 置き換えは文字単位の 1 回の走査で行う（置き換えた結果を再び置き換えない）。`\` と改行が同時に来ても、`\` が先に `\\` になるため正しく読み戻せる。
- 長い文字列を使わない理由: Lua は長い文字列の中の CR・CRLF を LF に正規化し、先頭の改行を捨てるため、値が変わる（5.1 に反する）。
- 不変条件: 改行を含まない値の出力は現行と 1 バイトも変わらない（5.3）。既存スナップショットは変わらない（8.3）。
- 呼び出し元（`scope_gen.rs`・`element_gen.rs`）は変えない。すべてこの関数を通るため、1 箇所の修正で全経路に効く。

**Implementation Notes**
- 文法の修正後、パーサ経由では改行入りの文字列値は来ない。この規則は、AST を直接組み立てる経路や将来の別経路に対する多層防御である。
- 検証: 単体テストで出力形を固定し、ランタイムテストで Lua に読み戻して値の一致を確かめる。

### book / マニュアル

#### マニュアル 3 章と references

| 章 | 節 | 書くこと | 要件 |
| -- | -- | -------- | ---- |
| `grammar/literals.md` | 文字列（String） | 引用文字列は開始した行の中で閉じる。値に改行を含められない。行をまたいで書くとパースエラーになる | 6.1 |
| `grammar/literals.md` | 文字列（String） | `「」`・`""` は、式・単語値・属性値・キューコマンドの引数のどこでも空文字列になる。例に `""` を足す（現行の「式に書いた `「」` は空文字列」を置き換える） | 6.2 |
| `grammar/literals.md` | 文字列（String） | 同じ行に `""` を 2 つ書くと、最初の `""` から次の `""` までが 2 重の囲みの文字列になる（`＠w：""、""` の候補は `、` の 1 つ） | 6.3 |
| `grammar/words.md` | グローバル単語定義の「値の書き方」 | `「」`・`""` は空文字列の候補になる。引用なしの値の中の `#`・`＃` は値の一部になる（`＠w：あ、い ＃c` の候補は `あ` と `い ＃c`） | 6.2、6.4 |
| `grammar/block-structure.md` | コメント | 行末コメントを書けない行に「値を囲んでいない単語定義」を足す（アクション行・継続行と同じく `#`・`＃` 以降も値になる） | 6.4 |

- 記述は挙動の説明だけにする。「代わりにこう書く」という回避の書き方は足さない（6.6）。
- references は `node book/tools/gen-skill-refs.mjs` で再生成し、`node book/tools/gen-skill-refs.mjs --check` が通ることを確かめる（6.5）。生成物を手で直さない。
- ハイライト文法は変えない。足した例について、`book/tools/highlight/` の各テスト（`tokenizer-test.mjs`・`scope-map-test.mjs`・`highlight-html-test.mjs`・`neutralizer-test.mjs`）と `link-check.mjs`・`verify-content.mjs` が通ることを確かめる（7.3）。7.1・7.2 は現行の文法（全規則が 1 行の `match`）で満たしており、変更は無い。

## Error Handling

- 改行をまたぐ引用: パーサが `ParseError::SyntaxError`（ファイル・行・桁付き）を返す。ローダはファイルごとにエラーをログに出し、`LoaderError::PartialTranspileError` で返す（既存の経路）。作者は、ランタイムの起動失敗ではなく、行番号付きのパースエラーとして気づける。
- 改行入りの値が出力に渡った場合: エラーにしない。規則 0 で安全なリテラルにして出す（多層防御）。
- 空の候補・空の属性値: エラーにも警告にもしない（要件 3 の方針）。

## Testing Strategy

### パーサ（`crates/pasta_dsl/tests/literal_fixes_test.rs`、新規）— 8.1
- 改行をまたぐ引用がパースエラーになり、「エラー行の契約」の表の行番号を報告する（文脈ごとに 1 件。多重の囲み `「「a⏎b」」`・`""a⏎b""` と、CRLF の改行を含む）（1.1〜1.4）。
- 行内で閉じた引用の値に、空白・`#`・`＃`・読点・`\` がそのまま入る（1.5）。
- `＠a：""⏎＠b：""` が単語 `a`・`b` の 2 つになり、それぞれの候補が空文字列 1 つである。`＄x＝""⏎＄y＝""` が 2 つの代入になる。CRLF でも同じ（2.1、2.2）。
- `＠w：""、""` の候補が `、` の 1 つである（2.3）。
- `＠w：「」、""` の候補が空文字列 2 つ、`＠w：「」、あ` が `""` と `あ`、`＆k：「」`・`＆k：""` が `AttrValue::String("")` である。式とキューコマンド引数の `「」`・`""` も空文字列である（3.1〜3.3）。
- `＠w：あ、い ＃c` の候補が `あ`・`い ＃c`、`＠話題：＃伺か` の候補が `＃伺か`、`＠w：「あ」 ＃c` の候補が `あ` である。グローバル単語・ローカル単語・アクター単語のそれぞれで確かめる（4.1〜4.3）。

### 出力（`crates/pasta_lua/src/string_literalizer.rs` の単体テスト）— 5.1、5.3
- LF・CR・CRLF を含む値が、規則 0 の `"…"` 形式で 1 行に出る。
- `\`・`"`・`]` と改行を同時に含む値の出力形。
- 改行を含まない値の出力が現行と変わらない（既存の単体テストがそのまま通る）。

### ランタイム（`crates/pasta_lua/tests/runtime/literal_fixes_test.rs`、新規）— 8.2、5.2、3.4、3.5
- 読み戻し: 規則 0 の出力を `return <リテラル>` として Lua で評価し、元の値とバイト単位で一致する（LF・CR・CRLF、`\` と改行の同時、先頭が改行の値）（5.1）。
- 別々の行に `""` を複数書いた `.pasta` を、パース → トランスパイル → Lua へロード → `finalize_scene` まで通し、エラーにならない（8.2）。
- 改行入りの値: パーサが拒否するため、パースした AST の文字列値（単語定義の候補）を改行入りの値に差し替えてトランスパイルし、同じくロードと `finalize_scene` が通る（5.2、8.2）。
- 空の候補: `＠w：「」` を定義して単語参照を実行し、結果が `nil` ではなく空文字列である（未定義単語の警告の経路は `nil` を返す経路だけなので、これで警告が出ないことも固定できる）。`act` 経由とアクター単語の経路の両方で確かめる（3.4）。
- `＠w：「」、あ` を 2 回引くと、空文字列と `あ` が 1 回ずつ出る（3.5）。

### マニュアル — 6.5、7.3
- `node book/tools/gen-skill-refs.mjs --check`、`book/tools/highlight/` の各テスト、`link-check.mjs`、`verify-content.mjs`。

### 回帰 — 8.3
- `cargo test --workspace` と `cargo clippy` が通る。既存のスナップショット・期待値は変えない。変わるものが出た場合は、この仕様が足す・変える入力に関わるものだけを更新し、それ以外は原因を調べる。

## 前提と未決事項

### 前提（設計が置いた仮定）
- **仮定 A**: 8.2・5.2 の「シーン辞書のロードが成功する」は、生成 Lua をランタイムにロードして `finalize_scene` が通ることで確かめる。`pasta.scene_dic` は各モジュールを `require` で並べるだけなので、同等とみなす。改行入りの値はパーサ経由で作れないため、ローダ（`PastaLoader`）を通す形のテストにはできない。
- **仮定 B**: 3.4・3.5 はランタイムの変更なしで満たせる（コード上、単語検索は `result ~= nil` で判定し、`word.lua` の `entry` は値を素通しする）。テストが失敗した場合、直す場所（`act.lua` など）はこの仕様の境界の外にあるため、実装を止めて境界を見直す。
- **仮定 C**: 1.2 の「エラーの位置（行番号）」は、引用を開始した行、またはその次の行を指せばよい（文脈で決まる。「エラー行の契約」の表）。

### 未決事項（設計ディスカッションで決める）
1. 次の行が単独で文法に合う場合（`＠w：「a`＋改行＋`＠v：b」`、`さくら：＠f（「a`＋改行＋`：b」）` など）は、パースエラーにならない。要件 1.2 の条件（「ファイルが文法に合わない」）の範囲内とみなして現行のまま進めるか、マニュアル（6.1）の「行をまたぐとパースエラーになる」の書き方をどこまで厳密にするか。
2. 単語値・属性値・アクション行の中の関数引数では、エラーが「開始した行の次の行」で報告される（仮定 C）。これを受け入れるか。マニュアルにエラー行の違いまで書くかどうか。
3. 8.2・5.2 の確かめ方を仮定 A（生成 Lua のロード＋`finalize_scene`）でよしとするか、別々の行の `""` についてはローダ経由（実ファイル → `pasta.scene_dic`）のテストも足すか。
4. 仮定 B が外れた場合（空の候補が「見つからない」扱いになった場合）の扱い。境界を広げて `act.lua` などを直すか、同じ Wave の別仕様へ渡すか。
