# Research & Design Decisions — string-concat-operator（ギャップ分析）

## Summary
- **Feature**: `string-concat-operator`
- **Discovery Scope**: Extension（既存の式の文法・算術ヘルパー・LSP・マニュアルへの追加）
- **Key Findings**:
  - 式の文法は「項と演算子の平らな列」で、パーサは優先順位なしの左結合木を作り、優先順位はコード生成（`element_gen.rs` の `arith_to_string`）で Lua の順に組み直している。`＆` を足すには、この組み直しに「算術より低い第 3 の段」を加える必要がある。`BinOp` に変種を足すだけでは `arith_to_string` が `＋`・`－` と同じ段に黙って混ぜる（コンパイルエラーにならない）。
  - 実行時は `act:arith`（`act.lua` 488–548 行）が「数値化 → 失敗なら被演算子ごとに警告＋nil、説明の無い nil は黙って伝播」を実装済み。連結は同じ流儀の新しいヘルパー（文字列・数値を受け、その他は警告＋nil）で表せる。Lua の素の `..` は nil・真偽値でエラー（500）になるため、ヘルパーが必須。
  - `＆` の構文上の衝突は小さい。式の項（`term`）は `＄`・`＠…（`・数字・`－`・文字列・括弧でしか始まらず、識別子は項にならない。そのため `＞＄名前＆k＝v`・`＞シーン名＆k＝v`・`＄x＝＠単語＆k＝v` は本仕様の後もパースエラーのまま残り、属性フィルター構文と切り分けられる。
  - 唯一の既存の書き方の意味の変化: アクション行の `＠f（＄a＆＄b）` は、現在は `args` が `＆` で失敗して「単語参照 `＠f`＋台詞」にフォールバックしている。本仕様の後は関数呼び出しになる（リポジトリ内の `.pasta` に該当なし）。
  - LSP の意味トークン（`visit_expr.rs` の Binary）は演算子を `OPERATOR` として出しており、`BinOp` の網羅 `match` で `＆` の追加が強制される。ただし `find_binary_op` は文字列リテラルを飛ばさないため、`「A＆B」＆＄x` で分割位置を誤る。TextMate 文法（VSCode・マニュアル共通）には演算子のスコープが無く、変更不要の見込み。

## Research Log

### 文法と AST（`pasta_dsl`）
- **Context**: `＆` を式の演算子にする位置と、既存の `＆`（属性）との衝突。
- **Sources Consulted**: `crates/pasta_dsl/src/parser/grammar.pest`、`parse_action.rs`、`parse_elements.rs`、`ast/action.rs`、`tests/expr_parse_test.rs`。
- **Findings**:
  - `amp = _{ "＆" | "&" }`（27 行）は `attr_marker`（47 行）でだけ使われている。`attr_value` は `expr` を使わず、`attr_string` は `attr_marker` を含まない。
  - `expr = _{ term ~ s ~ bin* }`・`bin = _{ bin_op ~ s ~ term ~ s }`・`bin_op = _{ add_op | sub_op | mul_op | div_op | modulo_op }`（60–67 行）。`term = paren_expr | fn_call | var_ref | number_literal | string_literal`（69–75 行）。**単語参照 `＠x`・`＠＄x` は項ではない**（`set` の右辺全体としてだけ書ける。96 行）。
  - 識別子は XID なので `＆` で名前が終わる（`＄a＆＄b` は空白なしでも分かれる）。
  - 式が現れる位置: `set`（ローカル・グローバル・プロパティ・式文 `＄＝`）、`args` の `positional_arg`・`key_expr`、`call_target_expr`（168 行）。アクション行は裸の `expr` を持たず、関数呼び出しの `args` の中だけが式。
  - 木の構築: `bin_op_from_rule`（`parse_action.rs` 258–267）、`parse_expr_from_parts`（271）、`build_left_assoc_expr`（332–345）。呼び出し元は `parse_call_scene`・`parse_args`・`parse_key_arg`・`paren_expr`・`parse_var_set`（`parse_elements.rs` 126–173）。`try_parse_expr` の 411 行に演算子規則の列挙があり、新しい規則を加える必要がある。
  - `enum BinOp { Add, Sub, Mul, Div, Mod }`（`ast/action.rs` 310–321）。
  - 既存のテスト `test_binary_no_operator_precedence`（`expr_parse_test.rs` 131）は「パーサは優先順位を持たない」ことを固定している。
- **Implications**: 文法は `bin_op` に連結の規則を 1 つ足す最小変更で済む。優先順位をパーサで持つかコード生成で持つかは設計判断（下記 Decision 候補）。

### コード生成（`element_gen.rs`）
- **Findings**:
  - `generate_expr_to_buffer` の Binary アーム（559 行）は `arith_to_string`（568–598）へ。`flatten_binary` で左の背骨を平らにし、`＊／％` を先に畳み、次に `＋－` を畳む 2 段構成。589 行 `matches!(op, BinOp::Mul | BinOp::Div | BinOp::Mod)` 以外はすべて加算段に入る。
  - `arith_node`（45–67）は `act:arith("op", 左, 右[, 左の説明, 右の説明])` を出す網羅 `match`。`operand_desc`（611–635）は変数パス・`@名()`・`@*名()`・`@$パス()`・括弧の中身から説明を作る（ネストした演算は説明なし → 実行時に黙って伝播）。
  - 動的 Call のターゲットは `tostring(...)` で包まれる（`element_gen_tests.rs` 238）。値なしは `"nil"` をキーに検索して見つからない警告になる（`call-jump.md` 209 行）。
  - `element_gen.rs` は 761 行で、`oversized-file-decomposition` の「600 行未満」の目安をすでに超えている。
- **Implications**: 3 段（連結 < 加減 < 乗除）への一般化が必要。`dsl-codegen-runtime-safety` の design.md 46 行は `arith` の名前・引数・戻り値を固定しているため、連結は別ヘルパー（または `arith` の op 追加）として足すかを設計で決める。

### ランタイム（`act.lua`）
- **Findings**:
  - `ARITH_OPS`（489–495）、`arith_value_text`（500–506。table は `tostring` しない）、`arith_operand`（514–524。`v == nil and desc == nil` なら黙って nil）、`ACT_IMPL.arith`（536–548）。
  - 「警告＋値なし」= 結果 nil、被演算子ごとに 1 行の警告、説明の無い nil（内側の失敗）は黙る。nil を代入した変数は未代入となり、アクション行で `act:talk - undefined variable` の警告と空文字。
  - DSL に真偽値リテラルは無い（`variables.md` 20 行）。真偽値は Lua の関数・変数からだけ来る。
  - LuaJIT 2.1（mlua `luajit52`）。整数型が無く、`tostring`・`..` は `%.14g`: 3 → `3`、7/2 → `3.5`、1/3 → `0.33333333333333`、15 桁以上は `1e+15`、1/0 → `inf`。アクション行の `＄n` は `ACT_IMPL.talk` の `tostring(text)`（202 行）なので、「アクション行と同じ表記」は `tostring` で実現できる。
  - `dynamic-word-reference` は文字列・数値以外を `unsupported value type` として扱っており、「文字列・数値だけが有効な値」の前例になる。
- **Implications**: 連結ヘルパーは `arith` の被演算子処理（説明付き警告・説明なし nil の黙った伝播）をそのまま再利用できる。`act.lua` は `act-token-grouping-fix` と並走するため、算術ヘルパーの近辺（488–548 行）だけを触る。

### 構文上の切り分け（Call 行・単語参照）
- **Findings**:
  - `call_scene = { call_marker ~ (id | call_target_expr) ~ s ~ args? }`。`＞シーン名＆…` は `id` 分岐で確定し、`＆` 以降で `or_comment_eol` が失敗 → パースエラー（PEG は `call_target_expr` へ戻らない）。
  - `＞＄名前＆k＝v`: `bin` が `＆` の後の項 `k` で失敗して巻き戻り、`or_comment_eol` が `＆` で失敗 → パースエラーのまま。フィルターのキーが識別子である限り、将来 `call-attribute-filter` が `call_target_expr ~ filters?` を足しても PEG で区別できる。
  - `＞＄名前＆＄x`・`＞＄種類＆「_挨拶」` は本仕様で有効な動的ターゲットになる（現在はパースエラー）。
  - `＄x＝＠単語＆…`: `expr` が失敗し `word_ref` が一致、行末で失敗 → パースエラーのまま。単語参照を項にしない限り、単語フィルターと衝突しない（項にしても、識別子のキーなら区別できる）。
  - アクション行: `talk_word` は `＆` を文字として含む。ただし `＠f（＄a＆＄b）` は現在 `args` の失敗で「`word_ref ＠f`＋台詞 `（`・`＄a`・`＆`・`＄b`・`）`」にフォールバックしており、本仕様の後は関数呼び出しになる（R6.2・OQ-6）。`＠＊f（…）`・`＠＄f（…）` も同じ。
  - リポジトリ内の `.pasta` で行頭以外に `＆` を含むものは無い（`crates/pasta_lua/tests/fixtures/sample.pasta:14` の `＆天気：晴れ` は属性）。

### LSP・シンタックスハイライト
- **Findings**:
  - `crates/pasta_lsp/src/analysis/visit_expr.rs` 330–370: Binary アームの網羅 `match op` が演算子文字を引き、`find_binary_op`（`text_utils.rs` 142）で深さ 0 の最初の演算子文字を探して `OPERATOR`（13）トークンを出す。`find_binary_op` は文字列リテラルを飛ばさない（`「A＆B」＆＄x` で誤分割。既存の `＋` でも同じ潜在問題）。`×`・`÷` は列挙に無い（既存の欠け、本仕様の範囲外）。
  - 属性は `DECORATOR`（`visit_scope.rs` 97–99）。AST の属性からだけ出すので、式の `＆` が属性として着色されることはない。
  - TextMate 文法 `editors/vscode/syntaxes/pasta.tmLanguage.json` は行頭基準。`attribute` は `^(\s*)([＆&])(.+)$` で行頭の `＆` だけ。`variable`・`call` は行の残りを 1 色で塗り、演算子のスコープは無い。マニュアルのハイライト（`book/tools/highlight/highlight-html.mjs` 39 行）は同じ文法を vscode-textmate で使う。→ 式の中の `＆` は `＋` と同じ表示（行の色の一部）になり、変更不要の見込み（確認テストは `editors/vscode/src/test/tmGrammar.test.ts` 299–305 付近に足せる）。
- **Implications**: R7.6・R6.7 は LSP の Binary アームへの `＆` 追加で満たせる。R7.7 は確認のみの見込み。

### マニュアル・スキル生成物
- **Findings**: 更新対象
  - `book/src/grammar/variables.md`: 122 行（式の表）、146–167 行（算術の評価。153 行「文字列を連結する演算子は無い」）、197–211 行（DSL→Lua 表）。
  - `book/src/grammar/markers.md`: 20 行（属性の行）、104–119 行（「## 演算子」。106 行「式で使える演算子は算術演算子だけ」）。
  - `book/src/grammar/index.md` 55–70 行（演算子の表）・19 行。
  - `book/src/grammar/call-jump.md` 38 行（動的ターゲット）・227 行（引数）。
  - `book/src/grammar/action-line.md`（台詞の `＆` と関数呼び出しの引数。R6.2 の変化の記載）。
  - `book/src/lua/script-api.md` 328・353–369 行（`act:arith` の API 説明。連結ヘルパーを作者向け API として載せるかは設計で決める）、`book/src/internals/*`（生成コードの形）。
  - 再生成: `node book/tools/gen-skill-refs.mjs`（`--check` で鮮度照合）、`node book/tools/link-check.mjs`。生成物は `.claude/skills/pasta-ghost-authoring/references/{variables,markers,grammar-index,call-jump,literals}.md` など。手書きの `SKILL.md` も確認。
  - 開発者向けステアリング `.kiro/steering/grammar.md` のマーカー早見表の「属性 `＆`」にも、式の連結演算子を併記するのが望ましい（マニュアルが権威、ステアリングは要約）。

### 既存テストの置き場所
- パーサ: `crates/pasta_dsl/tests/expr_parse_test.rs`。
- コード生成の単体: `crates/pasta_lua/src/code_gen/element_gen_tests.rs`（`arith_regroups_left_assoc_tree_by_lua_precedence` 594 ほか）。
- スナップショット（insta）: `crates/pasta_lua/tests/transpiler/runtime_safety_test.rs`（`test_regrouped_arith_matches_flat_lua` は組み直しを平らな Lua と比較する仕組みで、連結の優先順位の固定にも流用できる）、`snapshot_test.rs` 217 行。
- Lua ランタイム: `crates/pasta_lua/tests/lua_specs/act_runtime_safety_test.lua`（265・321 行の `describe`）。`lua_specs/init.lua` 65 行で登録。
- SHIORI 経由の 500 回避: `crates/pasta_shiori/tests/codegen_runtime_safety_e2e_test.rs`（フィクスチャ `fixtures/codegen_runtime_safety/dic/runtime_safety.pasta`）。

## Requirement-to-Asset Map

| 要件 | 既存の資産 | ギャップ |
| ---- | ---------- | -------- |
| R1 連結の構文・位置 | `grammar.pest` の `expr`/`bin_op`、`bin_op_from_rule`・`try_parse_expr`、`BinOp` | **Missing**: 連結の演算子規則と `BinOp` の変種 |
| R1.7・1.8 結果の受け渡し | 代入・引数・Call の生成は式の生成を共有 | なし（式の生成に乗る） |
| R2 優先順位 | `arith_to_string` の 2 段の組み直し | **Missing**: 第 3 段（連結）。**Constraint**: 現状のままでは連結が加算段に黙って混ざる |
| R3 値ごとの扱い | `act:arith` の被演算子処理・警告・伝播 | **Missing**: 連結ヘルパー。**Unknown**: OQ-1・OQ-3 の決定しだいで処理が変わる |
| R3.2 数値の表記 | `ACT_IMPL.talk` の `tostring` | なし（同じ `tostring` を使えば一致） |
| R4.1・4.3 属性・その他の位置 | 属性は `expr` を使わない | なし（テストで固定するだけ） |
| R4.2・R6.2 アクション行 | `talk_word`・`args` のフォールバック | **Constraint**: `＠f（…＆…）` の意味が変わる（OQ-6） |
| R4.4–4.8 Call・単語参照の切り分け | `call_scene`・`set` の PEG の挙動 | なし（文法上そのまま成り立つ。テストで固定） |
| R5 書き間違い | 既存のパースエラー経路 | なし（`bin` の構造上、被演算子の欠けはパースエラー） |
| R6.5 行の対応 | 算術の行の対応の仕組み | なし（式の生成に乗る見込み） |
| R6.7・R7.6 LSP | `visit_expr.rs` の Binary アーム | **Missing**: `＆` の演算子文字。**Constraint**: `find_binary_op` が文字列リテラルを飛ばさない |
| R7.7 マニュアルのハイライト | TextMate 文法の共用 | なし（確認のみの見込み） |
| R7.1–7.5・7.8 マニュアル・スキル | `gen-skill-refs.mjs`・`link-check.mjs` | **Missing**: 各章の記述 |

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存の拡張（コード生成で優先順位） | `bin_op` に連結を足し、`BinOp::Concat` を追加。`arith_to_string` を 3 段に一般化し、連結の段は新ヘルパーを出す。パーサは平らな左結合のまま | 変更が最小。「パーサは優先順位を持たない」既存の設計・テストと整合。LSP は網羅 `match` に 1 行 | `element_gen.rs`（761 行）がさらに大きくなる。名前が `arith_*` のまま連結も扱うと読みにくい | `dsl-codegen-runtime-safety` の組み直しの流儀を延長 |
| B: 新規（パーサで優先順位） | 文法を `concat_expr = arith_expr ~ (amp ~ arith_expr)*` の 2 層にし、AST に `Expr::Concat` などの別ノードを持つ。コード生成は別関数 | 連結と算術の責務が分かれ、AST だけで優先順位が読める。LSP・将来の演算子にも素直 | パーサ・AST・LSP・コード生成の変更が広い。`test_binary_no_operator_precedence` と組み直しの前提が二重構造になる | 「パーサで算術の優先順位も持つ」への移行は範囲外 |
| C: ハイブリッド | 文法・AST は A（`BinOp::Concat`）、コード生成は「平らにした列を `＆` で区切り、各区間を既存の `arith_to_string` に渡し、区間を連結ヘルパーで畳む」別関数（必要なら `element_gen` から式生成を別ファイルへ切り出す） | 既存の算術の組み直しに手を入れずに済む。区間ごとに算術を再利用できる。ファイルサイズの問題を同時に扱える | 切り出しは振る舞い不変のリファクタリングを伴い、差分が増える | ロードマップの「`element_gen.rs` は式生成（Binary）だけを触る」に収まる |

## Design Decisions（候補。設計フェーズで確定）

### Decision 候補: 優先順位をどこで持つか
- **Context**: R2。パーサは優先順位を持たず、コード生成で組み直している。
- **Alternatives**: (1) コード生成で第 3 段を足す（Option A/C）、(2) 文法の 2 層化（Option B）。
- **推奨の方向**: (1)。既存の設計とテストの前提を崩さない。

### Decision 候補: 連結ヘルパーの形
- **Context**: R3。`arith` の名前・引数・戻り値は前 spec の design で固定済み。
- **Alternatives**: (1) 新ヘルパー `act:concat(左, 右, 左の説明, 右の説明)`（仮名）、(2) `act:arith("&", …)` で op を増やす、(3) 可変長で区間全体を 1 回で連結。
- **推奨の方向**: (1)。`arith` の契約（数値を返す）を変えずに、被演算子の警告・伝播の処理（`arith_operand` 相当）を共有する。(3) は警告の伝播規則（説明なし nil は黙る）を被演算子ごとに保てるかの確認が要る。

### Decision 候補: マニュアルでの連結ヘルパーの扱い
- **Context**: `lua/script-api.md` は `act:arith` を作者向け API として載せている。
- **Alternatives**: 作者向け API として載せる／内部扱いとして内部設計の章だけに載せる。

## Implementation Complexity & Risk
- **Effort**: M（3–7 日）。文法・AST・コード生成・ランタイム・LSP・マニュアル・4 層のテストにまたがるが、どれも既存の算術の流儀の延長。
- **Risk**: Low〜Medium。パターンは確立済み。リスクは (a) 組み直しの第 3 段の誤り（加算段への黙った混入）、(b) アクション行の `＠f（…＆…）` の意味の変化、(c) `act-token-grouping-fix` との `act.lua` の並走による衝突。

## Risks & Mitigations
- 連結が加算段に黙って混ざる — `test_regrouped_arith_matches_flat_lua` と同じ「平らな Lua（`..` と算術）との一致」比較で、混在式（`「x」＆1＋2＊3` など）を固定する。
- アクション行の意味の変化（OQ-6） — 要件ディスカッションで受け入れを確認し、マニュアルに記載する。回帰テストで新しい挙動を固定する。
- `find_binary_op` の文字列リテラル内の誤分割 — 連結は文字列と並べて書くのが主な用途のため顕在化しやすい。LSP の修正を本仕様に含めるか（R7.6 の範囲）を設計で決める。
- `act.lua` の並走 — 算術ヘルパーの近辺（488–548 行）だけを触り、`build`・グループ化には触れない。
- `element_gen.rs` の肥大 — Option C の切り出しを検討する（振る舞い不変の小さなステップで）。

## Research Needed（設計へ持ち越し）
- 連結ヘルパーが受ける数値の表記に `tostring` を使った場合、`-0`・`inf`・`nan` の表記がアクション行と一致することの確認（同じ `tostring` なら一致する見込み）。
- `find_binary_op` を文字列リテラル対応にする範囲（`＋` など既存の演算子にも効く改善になる）。
- `test_regrouped_arith_matches_flat_lua` の比較の仕組みを、`..` を含む式に広げられるか。
- `call-attribute-filter` への申し送り事項の書き方（「フィルターのキーは識別子で始まる」「`call_target_expr` の後に `＆識別子` が来たらフィルター」）。

## References
- `.kiro/specs/completed/dsl-codegen-runtime-safety/design.md`（算術ヘルパーの契約・生成形。46・156・234–236 行）
- `.kiro/specs/call-attribute-filter/brief.md`（フィルター構文の論点。13・30–37・78 行）
- `.kiro/specs/scene-attribute-store/brief.md`（旧文法の `filter_list`。56–61 行）
- `.kiro/steering/roadmap.md`（Wave 2 の並走条件。108・124・127・140 行）
