# ギャップ分析: dynamic-word-reference

- 実施日: 2026-10-02
- 対象: `requirements.md`（R1–R6、OQ-1〜OQ-8）
- 前提: 現行実装を正とする（brief Constraints）。旧仕様の記述は材料であり規範ではない。

## 1. 分析サマリー

- **構文は局所的な追加で足りる**: `＠＄` は現行で必ずパースエラーになる字句であり、`action` と `set` の選択肢に動的単語参照の規則を 1 つ足すだけで既存の字句（`＠＠`・`＄＄`・`＠名前（）`・`＠名前`）の意味を変えずに受理できる。PEG の順序上の衝突は無い（`at_escape` が `＠＠` を先に取り、`fn_call`/`word_ref` は `id` 先頭を要求するため `＠＄` に一致しない）。
- **ランタイムの単語検索はそのまま使える**: `act:word(name)`・`act.アクター:word(name)` は任意文字列キーを受け、静的単語参照と同じ段（アクター A1/A2 → L1〜L5）で探す。トランスパイラが変数の値を渡すだけで R3.1〜3.4 を満たす。
- **最大のギャップは「未代入・空」の警告**: 現行の `word()` は `nil`/空キーを**警告なしで**黙って `nil` にする。生成コード `talk(word(var.x))` だけでは R4.1・4.2（参照変数名を含む警告）を満たさない。変数展開の `act:talk(value, "var.x")` と同じく、変数パスを第 2 引数で渡す経路が要る。
- **値の型の扱いが未整備**: 単語検索の Rust バインディング `search_word` は引数を `String` で受ける。真偽値・テーブルが渡ると Lua エラーで会話が止まる（R3.6・R4.5 違反）。数値は L1（`current_scene[key]`）で数値キーとして引かれ、文字列キーと結果が食い違う。単語キーを文字列化してから検索する必要がある。
- **公開 API の破壊的変更**: `pasta_dsl` の公開 enum `Action`・`SetValue` は `#[non_exhaustive]` でないため、変種の追加は crates.io 利用者にとって破壊的変更（0.x のマイナー版上げ相当）。

## 2. 現状調査（要件 ↔ 既存資産マップ）

### 2.1 文法・パーサ（`crates/pasta_dsl`）

| 要素 | 現状 | 位置 |
| ---- | ---- | ---- |
| 単語参照 | `word_ref = { word_marker ~ id ~ s }` | `src/parser/grammar.pest:170` |
| アクション | `action = _{ at_escape \| dollar_escape \| sakura_escape \| fn_call \| word_ref \| var_ref \| sakura_script \| talk }` | `grammar.pest:167` |
| 代入の右辺 | `set = _{ set_marker ~ s ~ ( expr \| word_ref ) }`（ローカル・グローバル・`＄＝`・プロパティ代入で共通） | `grammar.pest:96` |
| 変数参照 | `var_ref = _{ var_ref_property \| var_ref_global \| var_ref_local }`、`var_ref_local` は `var_id = { id \| digit_id }`（`＄０` を含む）、`var_ref_global` は `id` のみ | `grammar.pest:80-83` |
| プロパティ | `property_marker = _{ dollar ~ modulo }`、`property_id` は ASCII のみ | `grammar.pest:86-89` |
| 関数呼び出し | `fn_call_local = { fn_marker ~ id ~ args }`、`fn_call_global = { fn_marker ~ global_marker ~ id ~ args }` | `grammar.pest:99-101` |
| エスケープ | `at_escape = @{ at{2} }`、`dollar_escape = @{ dollar{2} }` | `grammar.pest:172-173` |
| 台詞 | `talk_word = _{ !(at \| dollar \| sakura_marker \| eol) ~ ANY }` — `＠`・`＄` は台詞に入らない | `grammar.pest:175` |
| AST | `Action::WordRef { name: String, span }`、`SetValue::WordRef { name: String }` | `src/parser/ast/action.rs:18, 251` |
| 変換 | `Rule::word_ref` → `Action::WordRef`（`parse_action.rs:87`）、代入の右辺（`parse_elements.rs:121-155`） | |
| span 補正 | `partial.rs:330-340` の `shift_action` は `Action` を網羅 match | |
| 先例 | 動的コール `call_scene = { call_marker ~ (id \| call_target_expr) ~ s ~ args? }`、`CallTarget::{Static, Dynamic(Expr)}` | `grammar.pest:160-161`、`tests/dynamic_call_test.rs` |

所見:
- `＠＄x` の現行エラーは、`word_ref` の `id` が `＄` を受けないことによる（`expected id`）。`＠` は `talk_word` からも除外されているため、どの選択肢にも一致せずパースエラーになる。
- 動的単語参照の規則（例: `word_marker ~ (var_ref_global | var_ref_local)`）を `word_ref` の前後どちらに置いても、先頭 2 文字で他の選択肢と素集合になる。`var_ref_*` は末尾の `~ s` を含むため、R1.7（直後の空白を出力しない）は既存規則の再利用で満たせる。
- R2.4（`＠＄x（` をパースエラー）・R2.5（`＠＄％` をパースエラー）・R2.6（`＠＄＄`）は、規則を足さなければ自然に「後続が一致しない」形で失敗する見込み。ただし R2.4 は `＠＄x` の一致後に `（…）` が `talk` として受理される可能性が高い（`（` は `talk_word` に入る）。パースエラーにするには否定先読み（`!lparen`）が要る — **Constraint**。
- `＄％` を除外するには、プロパティを含む `var_ref` ではなく `var_ref_global | var_ref_local` を直接参照する必要がある。
- **パーサの落とし穴（明示アームが必須）**: `parse_action.rs:158` の `_ => {}` は未知の規則を**黙って捨てる**。`parse_elements.rs` の `parse_var_set`（L141–147）の既定アームは `try_parse_expr` に渡し、その `_` アーム（L367–374）が子を再帰するため、`var_ref_local` を含む新規則は明示アームが無いと **`SetValue::Expr(VarRef)` として誤解釈される**（`＄y＝＠＄x` が `＄y＝＄x` と同じ動作になり、パースエラーにもならない）。また L125 の最初の `Rule::id` 取得が `var_set_none` の名前に漏れる可能性がある — **Constraint（テストで固定）**。
- キューコマンド行（`cue_arg_id`・`cue_cmd_scope`、L201–208）は現行で `＠＄x` を文字列として受理している。本仕様の範囲外で、挙動を変えないこと。

### 2.2 トランスパイラ（`crates/pasta_lua/src/code_gen/element_gen.rs`）

| 要素 | 現状の生成コード |
| ---- | ---------------- |
| アクションの単語参照 | `act.アクター:talk(act.アクター:word("単語"))`（`element_gen.rs:283-291`） |
| 代入の右辺 | `var.x = act:word("単語")`／`save.x = …`／右辺のみ `act:word("単語")`（`element_gen.rs:109-127`） |
| プロパティ代入 | `act:set_property("name", act:word("単語"))`（`element_gen.rs:144-163`） |
| 変数展開 | `act.アクター:talk(var.x, "var.x")` — nil のとき変数パス付きで警告（`element_gen.rs:293-315`、`resolve_var_path`） |
| 動的コール | `act:call(SCENE.__global_name__, tostring(<expr>), {}, …)`（`element_gen.rs:202-208`） |

所見:
- `resolve_var_path(name, scope)` と `expr_to_string` が既にあり、参照変数の Lua 表現（`var.x`・`save.x`・シーン引数）を得る部品は揃っている — **再利用可**。
- 動的コールの先例は `tostring(<expr>)` で包むが、`tostring(nil)` は `"nil"` になるため、`ACT_IMPL.call` の nil ガード（`act.lua:459-462`）が働かず、未定義変数は「`nil` という名のシーンが無い」警告になる。**動的単語参照で同じ包み方をすると R4.1 の警告内容（参照変数名）を満たせない** — 文字列化は nil/空の判定の後に行う必要がある — **Constraint**。
- `SetValue` の網羅 match が 3 箇所（`element_gen.rs:109, 124, 156`）、`Action` の網羅 match が 2 箇所（`action_span` の 29 行目、`generate_action`）。変種を足すとコンパイラが漏れを検出する。

### 2.3 ランタイム（`crates/pasta_lua/pasta_scripts/pasta/`）

| 要素 | 現状 | 位置 |
| ---- | ---- | ---- |
| `ACT_IMPL.word(self, name)` | `not name or name == ""` → **警告なしで** `nil`。未発見 → `log.warn("act:word - handler not found: key=…")`＋`nil`。関数 → `h(self)`、その他 → `tostring(h)` | `act.lua:368-383` |
| `PROXY_IMPL.word(self, name)` | 同上（`proxy:word - handler not found …`）。先に A1（`proxy.actor[key]`）・A2（アクター辞書前方一致）を探す | `actor.lua:124-147, 191-205` |
| 検索コア | `find_act_handler`: L1 `current_scene[key]` → L2 ローカル辞書 → L3 `self[key]` が関数なら採用 → L4 `GLOBAL[key]` → L5 グローバル辞書 | `act.lua:315-350` |
| `ACT_IMPL.talk(self, actor, text, var_name)` | `text == nil` かつ `var_name` ありで `act:talk - undefined variable: '…'` を警告 | `act.lua:194-203` |
| 単語検索バインディング | `search_word(name: String, global_scene_name: Option<String>)` | `src/search/context.rs:224-233` |

所見:
- R3.1〜3.4（静的と同じ段・前方一致・巡回の共有）: `word(<値>)` に渡すだけで満たせる。巡回は「スコープ＋単語キー」で記録されるため、同じキーの静的参照と自然に共有される — **既存で充足**。
- R4.1・4.2（未代入・空の警告）: 現行 `word()` は空キーを黙って捨てる — **Missing**。
- R4.3（該当なしの警告）: 既存の `handler not found` 警告で充足。ただし警告に参照変数名は出ない（R4.3 は単語キーを含めば足りる）。
- R3.5・3.6（数値・その他の型）: `search_word` は `String` 引数。mlua の文字列変換で数値は文字列化される見込みだが、L1・L3・L4・A1 はテーブル添字として生の値で引くため、数値 `1` と文字列 `"1"` で結果が変わりうる。真偽値・テーブルは L2/L5 で変換エラー（Lua エラー）となり会話が止まる — **Missing**（検索前の文字列化が要る）。
- OQ-2（関数・メソッドの段）: L3 は `self[key]` が関数なら採用するため、変数の値が `talk`・`word`・`yield` など act のメソッド名と一致すると、そのメソッドが `h(self)` で呼ばれ、戻り値（多くは `self` のテーブル）が `tostring` されて `table: 0x…` が台詞に出るか、コルーチン操作が走る。A1 はアクターのフィールド（`name`・`spot` 等）を値として返す。静的参照では作者が書いた名前に限られるが、動的参照では変数の値（セーブデータ・SHIORI 由来の値・ユーザー入力を経由しうる）で到達する — **Research Needed / 要決定（OQ-2）**。
- `＄％`（プロパティ）は `act:get_property()` で非同期に yield する。OQ-3 で組み合わせを受理する場合はコルーチン内評価が前提になるが、アクション行は常にシーン関数（コルーチン）内なので技術的障害は小さい。

### 2.4 エディタ支援・マニュアル

| 要素 | 現状 | 位置 |
| ---- | ---- | ---- |
| TextMate 単語参照 | `inline-word-ref`: `[＠@]([^＠@＄$\\\s]+)` — `＄` を名前に含めないため `＠＄x` に一致しない | `editors/vscode/syntaxes/pasta.tmLanguage.json:181-184` |
| TextMate 変数参照 | `inline-var-ref`: `[＄$][＊*]?([^＠@＄$\\\s]+)` | 同 `:185-188` |
| TextMate 台詞 | `inline-talk`: `[^＠@＄$\\]+` — `＠` 単独は台詞にも一致せず未着色で残る | 同 `:201-204` |
| TextMate 代入行 | `^(\s*)([＄$])(.+)$`（行全体） | 同 `:94` |
| LSP | `visit_action` が `Action::WordRef` を `token_type::WORD` に分類（`pasta_lsp/src/analysis/visit_action.rs:275`）。代入の右辺は `SetValue::WordRef` の名前を `＠name` で文字列探索して位置を出す（`visit_expr.rs:140-160`） |
| マニュアル | `book/src/grammar/words.md`（単語の参照・スコープと優先順位・未定義単語・`＠＠` エスケープ）、`markers.md:11,21`、`variables.md:110,119,186`（未定義時の警告、代入右辺の表、Lua 展開表） |
| マニュアル（追加） | `action-line.md:47-63`（インライン要素の表）・`:82`（判定順「エスケープ → 関数呼び出し → 単語参照 → 変数参照 → …」）、`actor-dictionary.md:59-74`（A1/A2。代入の右辺はアクター辞書を探さない） |
| スキル生成 | `book/tools/gen-skill-refs.mjs` の `GENERATION_MAP` に `grammar/markers.md`・`action-line.md`・`variables.md`・`words.md`・`actor-dictionary.md` → `pasta-ghost-authoring` が登録済み（35-57 行） |
| スキル手書き部分 | 生成対象外: `.claude/skills/pasta-ghost-authoring/SKILL.md`（L71、L108–129 のインライン要素・エスケープ・最長一致）と `references/authoring-patterns.md` は手で揃える必要がある |
| マニュアルのハイライト | `book/tools/highlight/` が VSCode TextMate 文法を読み取り再利用 → TextMate を直せば R6.4 は追従 |
| マニュアル例の構文検証 | CI はチュートリアル成果物（hello-pasta）のみ `cargo test -p pasta_sample_ghost` で検証。文法章のコード例を読み込み検証する仕組みは無い |

所見:
- R6.1: TextMate に `[＠@][＄$][＊*]?…` の規則を `inline-word-ref`・`inline-var-ref` より前に追加すれば足りる — **Missing（小）**。
- R6.3: AST に変種を足せば LSP の網羅 match がコンパイルエラーで検出する。代入右辺の位置探索は文字列探索のため、動的版は `＠＄name` の探索を足す — **Missing（小）**。
- R5.5（マニュアル例が読み込み可能）: 自動検証の仕組みが無い。本機能のテスト（パーサ・トランスパイラのテスト）にマニュアル例と同じ入力を含めるのが現実的 — **Constraint**。
- マニュアル `words.md` の「多段階参照は無く、`＠＠` はエスケープ専用」記述は R2.1・Out of scope と整合しており維持。

### 2.5 テスト配置

- パーサ: `crates/pasta_dsl/tests/`（先例 `dynamic_call_test.rs`・`property_scope_test.rs`・`digit_id_var_test.rs`）
- トランスパイラ: `crates/pasta_lua/tests/transpiler/`（`dynamic_call_test.rs`・`snapshot_test.rs` の insta スナップショット）、単体 `src/code_gen/element_gen_tests.rs`
- ランタイム: `crates/pasta_lua/tests/runtime/`（E2E）、`tests/lua_specs/`（lua_test BDD）
- 既存の関連テスト: `pasta_lua/tests/transpiler/code_generator_test.rs:25`（単語参照の生成）・`:116-182`（代入右辺の単語参照）、`tests/property_scope_codegen_test.rs:118`、スナップショット `final_regression_test.rs:120`（`kind_word_reference`）・`snapshot_test.rs:180-217`（動的コールのスナップショット＝新規スナップショットの手本）、`lua_specs/act_word_expr_test.lua:22-60`（`word(nil)`・`word("")` が nil を返すことを固定 — 第 2 引数を省略した既存呼び出しの挙動は変えない）、`runtime/syntax_test.rs:400`（未定義の単語・変数・関数が空で描画される E2E）、`pasta_lsp/tests/var_set_token_test.rs:180`、`editors/vscode/src/test/tmGrammar.test.ts:309`、`book/tools/highlight/tokenizer-test.mjs`。
- 既存テストに「`＠＄` がパースエラーであること」を固定するものは見当たらない（grep で `＠＄` の出現は 0 件）→ 置き換え対象なし。

## 3. 要件ごとの充足状況

| 要件 | 状態 | 備考 |
| ---- | ---- | ---- |
| R1.1–1.4, 1.6, 1.7 | Missing | 文法規則と AST 変種の追加。全角半角はマーカー規則の再利用で自動 |
| R1.5 | Missing | プロパティ代入の右辺も `set` 共通のため R1.4 と同時に入る |
| R2.1–2.3, 2.7 | 既存で充足（回帰テストで担保） | 先頭 2 文字が素集合 |
| R2.4 | Missing / Constraint | 否定先読みが要る（無いと `（…）` が台詞になる） |
| R2.5, 2.6 | 既存で概ね充足 | 規則が一致しないためパースエラー。エラー文言（`expected …`）は要確認 |
| R3.1–3.4, 3.8, 3.9 | 既存で充足 | `word()` に値を渡すだけ |
| R3.5, 3.6 | Missing | 検索前の文字列化（nil/空判定の後） |
| R3.7 | 既存で充足 | 値は検索キーとして使われるだけで再パースされない |
| R4.1, 4.2 | Missing | `word()` の空キー無警告。変数パス付き警告の経路が要る |
| R4.3–4.5 | 既存で充足 | `handler not found` 警告・nil 代入・実行継続 |
| R5.1–5.4, 5.6 | Missing | マニュアル 5 章の追記と再生成、手書きスキル資料の追従 |
| R5.5 | Constraint | 自動検証なし。テスト入力で担保 |
| R6.1, 6.2, 6.4 | Missing（小） | TextMate 規則追加。マニュアルは自動追従 |
| R6.3 | Missing（小） | LSP の match・右辺位置探索 |

## 4. 実装アプローチの選択肢

### Option A: 既存の構造を拡張（静的単語参照の AST を一般化）

- `Action::WordRef { name: String, span }` を `Action::WordRef { key: WordKey, span }`（`WordKey::{Static(String), Var { name, scope }}`）のように一般化し、`SetValue::WordRef` も同様にする。
- 生成コード: 静的は従来どおり `word("単語")`、動的は `word(var.x, "var.x")` のように値と変数パスを渡す。ランタイム `ACT_IMPL.word`・`PROXY_IMPL.word` に省略可能な第 2 引数（変数パス）を足し、`talk(text, var_name)` と同じ型で「nil/空 かつ 変数パスあり → 警告」「非文字列 → `tostring`」を行う。
- ✅ 静的・動的の分岐が 1 箇所にまとまり、LSP・`partial.rs` の match も既存アームの中で吸収できる。
- ✅ ランタイムの変更は既存の `talk(text, var_name)` パターンの踏襲で小さい。
- ❌ 既存の `WordRef { name }` のフィールド形が変わり、`pasta_dsl` 公開 API の破壊的変更が大きくなる（既存の利用箇所すべての書き換え）。
- ❌ 静的参照の生成コードが変わらないことの保証をスナップショットで固める必要がある。

### Option B: 新しい変種・新しいランタイム関数を追加（静的は無変更）

- AST に `Action::DynWordRef { name, scope, span }`・`SetValue::DynWordRef { name, scope }` を追加。静的 `WordRef` には触れない。
- ランタイムに専用関数（例: `ACT_IMPL.word_var(self, value, var_path)`・`PROXY_IMPL.word_var`）を追加し、nil/空の警告・文字列化をしてから既存 `word()` に委譲する。
- ✅ 静的単語参照のコード経路・生成コードがバイト単位で不変（R2.7 の担保が容易）。
- ✅ OQ-2 で「動的は単語辞書の段だけを探す」に決まった場合、専用関数側で段を絞れる（静的に影響しない）。
- ❌ AST 変種の追加は依然として公開 API の破壊的変更（ただし既存変種の形は変わらない）。
- ❌ ランタイムの公開メソッドが 2 つ増える（Lua API としてマニュアルに載せるかの判断が要る）。

### Option C: ハイブリッド（新しい AST 変種＋既存 `word()` の後方互換拡張）

- AST は Option B（新変種、静的は無変更）。ランタイムは Option A（`word(name, var_path)` に省略可能な第 2 引数を追加。第 2 引数なしの既存呼び出しは挙動不変）。
- 生成コード: `act.アクター:talk(act.アクター:word(var.x, "var.x"))`、`var.y = act:word(var.x, "var.x")`。
- ✅ 静的の生成コード・挙動が不変、ランタイム関数は増えない、既存の `talk(text, var_name)` 規約と対称。
- ✅ 変更量が最小（文法 1 規則＋右辺 1 箇所、AST 2 変種、パーサ 2 アーム、`partial.rs` 1 アーム、codegen 4 アーム、Lua 2 関数に数行、TextMate 1 規則、LSP 2 箇所）。
- ❌ OQ-2 で段を絞る決定になった場合、`word()` 内に「動的なら L1/L3/L4/A1 を飛ばす」分岐が入り、検索コア（`find_act_handler`）にモード引数が増える。

## 5. 工数とリスク

- **工数: S（1–3 日）** — 既存パターン（動的コール・変数展開の nil 警告）の踏襲で、新しい依存も新しい仕組みも要らない。マニュアル 5 章の追記と再生成を含めても小さい。OQ-5（式への一般化）や OQ-3（プロパティ）を取り込むと M。
- **リスク: Low〜Medium** — 字句の衝突は無く既存辞書への影響も無い（現行は必ずパースエラー）。Medium 要因は (1) OQ-2 の検索段の扱い（変数の値が act のメソッド名に一致したときの副作用）、(2) `pasta_dsl` 公開 enum の破壊的変更に伴う版上げの扱い。

## 6. 設計フェーズへの申し送り

### 推奨（情報提供・最終決定ではない）

- Option C を第一候補とする。静的単語参照の経路を一切変えずに、既存の `talk(text, var_name)` と対称な警告経路を足すのが最小の差分で、R2.7・R4.1・R4.2 を同時に満たせる。
- 文字列化は「nil/空の判定 → 警告」の**後**に行う（動的コールの `tostring(<expr>)` 方式は nil を `"nil"` に変えてしまい、R4.1 の警告を満たせない）。

### 設計で決めること

1. AST の表現（新変種 vs `WordRef` の一般化）と、`pasta_dsl` 公開 API 変更に伴う版上げの扱い（`#[non_exhaustive]` を付けるかを含む）。
2. `＠＄変数名（` をパースエラーにする規則（否定先読み）とエラー文言。パーサの既定アームに落ちないよう、行動・代入の両方に明示アームを置くこと。
3. ランタイムの警告文言（未代入／空／該当なしの区別、参照変数パスの表記）。
4. 代入右辺の LSP トークン位置の求め方（文字列探索の拡張か、`SetValue` に span を持たせるか）。

### Research Needed

- **RN-1（OQ-2）**: 動的単語参照で L1（シーンテーブル）・L3（act のメソッド）・L4（`GLOBAL`）・A1（アクターのフィールド）を探すことの安全性。変数の値の出どころ（セーブデータ・SHIORI リファレンス・`OnUserInput` 等）と、到達しうる act のメソッド一覧を洗い、「静的と同じ」「単語辞書の段のみ」「L3 のみ除外」のどれにするかの材料を揃える。
- **RN-2**: mlua の `String` 引数への数値・真偽値・テーブルの変換挙動の確認（数値は文字列化、その他はエラーになるか）。設計では検索前の明示的な文字列化で回避する前提。
- **RN-3**: `＠＄＄`・`＠＄` 単独・`＠＄％x` がパースエラーになったときのエラーメッセージが作者に分かる文言か（`expected id` 等）の実測。
- **RN-4（付随発見）**: 動的コール `＞＄未定義` は `tostring(nil)` により `"nil"` という名のシーンを探し、`act:call` の nil ガードが働いていない。本仕様の範囲外だが、警告の考え方を揃える際の参考として記録する（修正は別件）。
