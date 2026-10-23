# Brief: call-attribute-filter

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で `scene-attribute-semantics` から分割して起票）。Wave 5（機能拡張）。着手するときは `/kiro-start call-attribute-filter` で開始する。

## Problem

Call の属性フィルター（`＞シーン＆k＝v`）は、旧文法仕様（ch04 §4.2・ch12 §12.5）とマニュアル旧版で「将来予約」として定義されていたが、文法として受理されない（パースエラー）。シーンに属性を付けても、それで候補を絞り込む手段が無い。`scene-attribute-store` が属性を実行時に保持し型解釈を決めた後に、それを使う絞り込みを足す。

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（仕分け表 B1、食い違い grep 記録 D05）と 2026-10-04 の棚卸の再照合による。旧仕様の原文の要旨は `.kiro/specs/scene-attribute-store/brief.md` の「吸収元の内容」節にある（ch04 §4.2 フィルター・§12.5・§12.10・マニュアル旧版 `call-jump.md` の「フィルター」）。

- **構文（受理されない）**: `call_scene = { call_marker ~ (id | call_target_expr) ~ s ~ args? }` に `＆` を受ける規則が無く、`＞X＆k＝v` はパースエラー（`expected args`）。単語参照 `word_ref` も名前だけを受ける。式に比較演算子は無い（X04）。
- **実行時の経路**: `ACT_IMPL.call` → `find_handler` → `SCENE.search` → `search_scene` と進む。
  - 途中で `attrs` 引数は受け取るが、使われないと文書化されている。
  - `search_scene`（`crates/pasta_lua/src/search/context.rs` 73 行付近）は常に空の `filters` で検索する。
- **絞り込み**: `pasta_core` の `scene_table.rs` の `filter_by_attributes` は、`HashMap<String,String>` の文字列一致・AND だけ。比較演算子と型付きの値には新しいフィルター型が要る。候補キャッシュのキーにはフィルターが含まれる。
- **マニュアル**: `grammar/call-jump.md` の「フィルター」節は削除済み。

## Desired Outcome

- Call の属性フィルター構文が文法として受理され、実行時に候補を絞り込む。比較演算子と複数条件の結合（AND/OR）の規則が決まっている。
- 一致する候補が無いときの挙動が決まっている。前方一致・シャッフル消費（候補キャッシュ）と矛盾しない。
- マニュアル（`grammar/call-jump.md` ほか）が新しい構文と挙動を書き、スキル `references/` を再生成している。VSCode 拡張の TextMate 文法が新しい構文をハイライトする。

## Approach

要件フェーズで次を決める（未決定）。

- **構文**: `＞シーン＆k＝v` の文法。
  - 全角・半角の扱い。
  - `call_target_expr`（動的 Call）との組み合わせ。
  - 引数リストとの順序。
- **比較**: 比較演算子の種類（`＝`・`＞`・`＜` など）と、値の型解釈（`scene-attribute-store` が決めたもの）に基づく数値比較と文字列比較。
- **結合**: 複数条件の結合（AND/OR）。
- **候補なし**: 一致する候補が無いときの挙動（「見つからない」警告と同じか）。
- **単語参照へのフィルター**: §4.2 の `＠単語名＆category＝food` は、単語に属性を付ける構文が無いため対象外とし、バックログに置く（推奨）。
- **実行時の経路**: コード生成が `act:call` の `attrs` 引数にフィルターを出力し、`ACT_IMPL.call` → `find_handler` → `SCENE.search` → `search_scene` へ通す。

## Scope

- **In**:
  - Call の属性フィルター構文（文法・パーサ・AST）
  - コード生成と、`act.lua`・`scene.lua`・`search/context.rs` を通る受け渡し
  - `pasta_core` の型付き比較フィルターと候補キャッシュのキー
  - 修正を固定するテスト、マニュアル章の更新とスキル `references/` の再生成、TextMate 文法の追従
- **Out**:
  - 属性の保持・継承・読み出し（`scene-attribute-store`）
  - 単語参照への属性フィルター（バックログ）
  - 属性と無関係な Call・単語検索の挙動変更

## Boundary Candidates

- 文法（`grammar.pest`）とパーサ（`pasta_dsl`）: フィルター構文
- コード生成（`element_gen.rs` の Call の出力）
- ランタイム（`act.lua` の `call`・`find_handler`、`scene.lua` の `SCENE.search`、`search/context.rs`、`pasta_core` の `scene_table`）
- マニュアル・生成スキル・ハイライト文法

## Out of Boundary

- マニュアル権威化とスキル生成の仕組み（`manual-ssot-authority` が提供）

## Upstream / Downstream

- **Upstream**:
  - `scene-attribute-store`（Wave 4。属性の実行時の保持と型解釈）。
  - `scene-search-key-normalization`（Wave 1。`search_scene` の入口）。
  - `call-execution-correctness`（Wave 3。Call のコード生成と `ACT_IMPL.call`）。
- **Downstream**: なし（属性を使うゴースト）

## Existing Spec Touchpoints

- **Adjacent**: `manual-ssot-authority`（吸収台帳 D05）、`pasta-manual-syntax-highlight`（TextMate 文法の再利用）

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- 現在 `＞X＆k＝v` はパースエラーのため、フィルター構文を足しても既存の辞書に非互換は生まれない。
- 現行実装を正として設計する。旧仕様の記述は材料であり規範ではない。
- 並走条件（Wave 5）: Wave 5 はこの spec だけ。
- `string-concat-operator` からの申し送り: 式の `＆` は連結演算子になった（`＞＄種類＆「_挨拶」` は受理される）。式の `＆` の後は項の開始文字（`＄`・`＠`・数字・`－`・文字列の開き・括弧の開き）だけ。フィルターのキーは識別子で始める。`＞＄名前＆k＝v`・`＞シーン名＆k＝v`・`＄x＝＠単語＆k＝v` は `string-concat-operator` の後もパースエラー（マニュアル `grammar/call-jump.md`・`grammar/variables.md` の「連結の評価」）。
- 申し送り（`call-execution-correctness` より、2026-10-05）: Call の実行経路が次のように変わった。「実行時の経路」と「候補なし」の論点はこれを前提に決め直す。詳細は `.kiro/specs/completed/call-execution-correctness/design.md` の CallCodeGen・ActCall・DynamicCallKey。
  - 生成形が 4 通りになった。末尾の Call は `return act:call(SCENE.__global_name__, <キー>, {}, <引数>)`、途中の Call は `act:call_restore(…同じ並び…)`。`attrs` はどちらも第 3 引数で、`call_restore` は `act:call` へそのまま渡す。フィルターを出すときは両方の形に同じ位置で出せる。
  - 動的コールのキーは `act:call_key(値, 変数の経路, 説明)` が判定する。値が nil・空文字列・使えない型なら「呼ばない」印を返し、`act:call` は検索の前（フィルターより前）に nil を返す。
  - ターゲットが見つからないとき、`act:call` は現行の警告に加えて失敗表記 `【Call失敗：「名前」が見つからない】` を出して次の行へ進むようになった（同 spec の要件 4.6）。「一致する候補が無いとき」をこれと同じ扱いにするか、フィルター条件を表記に含めるかを決める。

## 2026-10-07 棚卸の再測定（main 2cbaf510）

- **前提の変化**: `scene-search-key-normalization`・`call-execution-correctness`・`string-concat-operator` は完了した（申し送りは上の Constraints のとおりで、現行の main と一致する）。未完了の前提は `scene-attribute-store` だけ。
- **触るファイル**: `pasta_dsl` の `parser/grammar.pest`（272 行。`call_scene` 170 行）・`parse_action.rs`（456 行）・AST、`pasta_lua` の `code_gen/element_gen.rs`（555 行。Call の生成は 196 行付近）、`pasta_scripts/pasta/act.lua`（774 行。`call`・`find_act_handler`）、`scene.lua`（`SCENE.search`）、`search/context.rs`（614 行）・`search/mod.rs`、`pasta_core` の `registry/scene_table.rs`（434 行）、`editors/vscode/syntaxes/pasta.tmLanguage.json`、マニュアル `grammar/call-jump.md`・`lua/script-api.md`・`internals/registry-search.md` と生成スキル。`act.lua` は 774 行で、1,000 行に近づいている。
- **規模**: 約 16〜19 タスク（文法・パーサ・AST 3、生成 2、`act.lua`→`scene.lua`→`search_scene` の受け渡し 3、`pasta_core` の型付き比較と候補キャッシュのキー 3、候補なしの扱い 1、ハイライト 1、マニュアルと生成 2、テスト 2〜3）。上限の 20 に近いが、一度分割した spec なので再分割はしない。
- **先に要るもの**: `scene-attribute-store`（保持と型の解釈）。ファイルの重なり: `failure-output-unification`（`act.lua`）、`scene-attribute-store`（`scene.lua`・`pasta_core` の registry）。どちらとも同じウェーブに置けない。Phase 12・`release-ci` とはソースが重ならない（マニュアルは別の節）。
- **種別**: 機能（現在 `＞X＆k＝v` はパースエラー）。
- **要件定義のモデル**: Fable（新しい構文と意味。式の `＆`（連結）との切り分け、比較演算子、AND/OR の決定がある）。
- **分割の案**: なし。
- **見つけた穴・古くなった記述**:
  - `attrs` は `act:call` の中で捨てられる。`self:find_handler("scene", key)`（`act.lua` 670 行）は `attrs` を受けず、`search_dictionary` の `SCENE.search(key, scene_name)`（307 行）も渡さない。`SCENE.search` は `attrs` を受けるが `SEARCH:search_scene(name, global_scene_name)`（`scene.lua` 160 行）に渡さず、`search_scene` にはフィルターの引数が無い（`context.rs` 73〜78 行）。Current State の「途中で `attrs` 引数は受け取る」は、`act:call` の入口までの話である。
  - `find_act_handler` の 5 段の検索のうち、L1（実行中のシーン表の完全一致）・L3（`act` のメソッド）・L4（`GLOBAL`）はシーンの登録表を通らないので、属性で絞り込めない。フィルター付きの Call がこれらに当たったときの扱い（L2・L5 だけを探すか）を要件で決める。
  - 動的コールでは `act:call_key` の「呼ばない」印が先に返るので、フィルターは評価されない（申し送りどおり）。
  - `register_global_raw` がグローバルの属性をローカルシーンに複製する件は `scene-attribute-store` の brief に記した。その結論が、ローカルシーンを絞り込むときの前提になる。
  - マニュアル `lua/script-api.md` 304・317 行は「`attrs` は使わない」と書いている。本 spec で書き換える対象。
- **順序の提案**: 本 spec を `failure-output-unification` の後に置く。そうすれば「一致する候補が無い」の失敗表記を、最初から一本化した仕組みに載せられる（現在のロードマップは逆順）。

## 申し送り（scene-name-alias より）

- `search/context.rs` の `SearchContext::search_scene` のグローバル分岐（第 2 引数 `None`）に、別名の置き換え（`scene_aliases.resolve(name)` → `sanitize_name`）が入った。フィルターはこの後の `resolve_scene_id_unified` に足すことになる。候補キャッシュのキーは**置き換え後**の名前なので、フィルターをキーに含めるときも置き換え後の名前で揃える。
- `SearchContext` は別名表を持ち（`with_aliases`）、`search::register(lua, scene_reg, word_reg, aliases)` と `register_finalize_scene(lua, aliases)` の引数が増えた。rebase で署名を合わせる。
- 参照: `.kiro/specs/completed/scene-name-alias/design.md`「SearchAlias」「Revalidation Triggers」。

## 2026-10-10 棚卸の再測定（main add05022）

- **前提の変化**: シーン名の別名（`scene-name-alias`）が、シーンを探す関数 `search_scene`（`crates/pasta_lua/src/search/context.rs` 91 行）に入った。上の「申し送り（scene-name-alias より）」は現在のコードと一致する（別名の置き換えは 126〜127 行、空の絞り込み条件は 96 行、登録の関数 `search::register` は `crates/pasta_lua/src/search/mod.rs` 70 行）。式の中の値なしの扱い（`expr-nil-coercion`）で `act.lua` は 726 行に減ったが、Call の経路は変わっていない。未完了の前提は `scene-attribute-store` と `failure-output-unification` の 2 本。
- **触るファイル**: 文法 `crates/pasta_dsl/src/parser/grammar.pest` 272（Call の規則は 170 行）、Call を読む `parse_action.rs` 456、構文木 `ast/action.rs` 323、位置をずらす `crates/pasta_dsl/src/partial.rs` 477、Call の Lua を出す `crates/pasta_lua/src/code_gen/element_gen.rs` 564（205〜255 行）、`crates/pasta_lua/pasta_scripts/pasta/act.lua` 726（`call` 611 行）、`scene.lua` 216（`SCENE.search` 150 行）、`search/context.rs` 752、`crates/pasta_core/src/registry/scene_table.rs` 434、VSCode の文法定義 `editors/vscode/syntaxes/pasta.tmLanguage.json` 272、エディタ向けの色付け `crates/pasta_lsp/src/analysis/visit_action.rs` 431（Call は 199 行）、マニュアルと生成スキル。1,000 行に近いものは無い。
- **規模**: 約 17〜20 タスク。前回の見積もりに、エディタ向けの色付けの 1 タスクを足した。上限ぎりぎりだが、一度分けた spec なので、もう分けない。
- **先に要るもの**: `scene-attribute-store`（属性の保持と値の型）と `failure-output-unification`（「条件に合う候補が無い」の見せ方を載せる先）。ファイルの重なり: `failure-output-unification`（`act.lua`）、`scene-attribute-store`（`scene.lua`・`pasta_core` の登録表）、`scene-anchor-link`（`grammar.pest`・`parse_action.rs`・`ast/action.rs`・`partial.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義・`visit_action.rs`）。この 3 本とは同じ時期に進められない。
- **種別**: 機能（`＞シーン＆k＝v` は今は構文エラー）。
- **要件定義のモデル**: Fable（新しい書き方と意味。連結の `＆` との見分け、比べる記号、条件のつなぎ方を決める）。
- **分割の案**: なし。
- **見つけた穴・古くなった記述**:
  - `search_scene` は、「名前が見つからない」と「絞り込みで候補が無くなった」を同じ扱いにしている（`search/context.rs` 141〜153 行）。別名を使った Call では、絞り込みが原因でも「Scene not found (alias applied)」という警告が出る。警告の文言を分けるかを決める。
  - エディタ向けの色付け（`visit_action.rs` の Call）が Scope に無い。絞り込みの部分にも色を付けるなら Scope に足す。
  - マニュアルのコードの色付けは、VSCode の文法定義をそのまま読む（`book/tools/highlight/highlight-html.mjs` 39 行）。文法定義を変えると、マニュアルの表示と `book/tools/highlight/tokenizer-test.mjs` にも効く。
  - `act:find_scene`（`act.lua` 596 行）も `attrs` を受け取って捨てている。イベントからシーンを起こす `SCENE.co_exec`（`scene.lua` 193 行）がこれを呼ぶ。Call だけを絞り込むのか、こちらも通すのかを決める。
  - 行番号が動いた: 「Current State」の `search_scene` は 73→91 行。前回の節の `find_handler("scene", key)` は `act.lua` 670→622 行、`search/context.rs` は 614→752 行、`element_gen.rs` は 555→564 行。マニュアル `book/src/lua/script-api.md` の「`attrs` は使わない」は 306 行と 319 行。

## 2026-10-10 棚卸の申し送り

roadmap のバックログにあった「`pasta_core` の `resolve_scene_id` の整理」を、この spec の要件へ渡す。

- **事実**（main `add05022` で確かめた）:
  - `resolve_scene_id` は、親のシーンを指定せずに名前と絞り込みの条件からシーンを 1 つ選ぶ関数である（`crates/pasta_core/src/registry/scene_table.rs` 171 行）。シーン表 `SceneTable` の公開の関数で、`pasta_core` は crates.io に公開している。
  - 本番のコードからは呼ばれていない。呼んでいるのは、同じファイルの `find_scene`（404 行。410 行で呼ぶ）、`crates/pasta_core/README.md` の使い方の例（64 行）、テスト（`crates/pasta_core/src/registry/` の `scene_table_resolve_filter_tests.rs`・`scene_table_candidate_tests.rs`）だけである。`find_scene` も、呼んでいるのはテストだけである。
  - 本番の検索が使うのは、親のシーンも受け取る `resolve_scene_id_unified`（同じファイルの 218 行）で、呼び出し元は `crates/pasta_lua/src/search/context.rs` の 104 行と 130 行である。
  - 3 つの関数は、どれも絞り込みの条件を同じ形（文字列から文字列への表 `filters`）で受け取る。
- **この spec との関係**: この spec は、ちょうどこの絞り込みの条件（`filters`）の形と比べ方を作り直す。`resolve_scene_id` と `find_scene` を残すなら、使われていない 2 つの関数とそのテストも、新しい形に合わせて直すことになる。
- **要件で決めること**: `resolve_scene_id`（と `find_scene`）を消すか、残すか。消すと、公開している関数が無くなるので、版の付け方の決まり（semver）では互換性を壊す変更になる。絞り込みの条件の形を変えるなら、本番で使う `resolve_scene_id_unified` の引数も変わるので、どちらにしても互換性を壊す変更になる見込みである。消すなら、README の例を `resolve_scene_id_unified` を使う形に書き直す。

## 申し送り（2026-10-10 の discovery・0.5.0 のノベルゲームより）

0.5.0 の目標（areka でノベルゲームが作れる）の道筋に、この spec が入った。ノベルゲームの分岐は、選択肢と、フラグによる分岐の 2 種類である。pasta は `if` を持たないので、**フラグによる分岐を DSL で書く道は、Call の絞り込みだけである**（開発者の決定）。

```pasta
＊告白＆好感度：高
    さくら：……わたしも、好き。

＊告白＆好感度：低
    さくら：ごめんなさい。

＊放課後
    さくら：話があるの。
    ＞告白＆好感度＝＄＊好感度
```

（絞り込みの書き方は、この spec の要件で決める。上は例である。）

要件で決めることに、次を足す。

- **条件の値に変数を書けること**: フラグは変数に入っている。`＞告白＆好感度＝＄＊好感度` のように、比べる相手を変数で書けないと、分岐に使えない。
- **「それ以外」の場面**: どの場面も条件に合わないと、物語が止まる。属性を書いていない場面を「それ以外」として選ぶ決まりを作るかを決める。
- **数のしきい値**: 「好感度が 50 以上」のような分岐の書き方。場面の属性と、呼ぶ側の値の、どちらをどちらと比べるか。
- **候補が 2 つ以上残るとき**: ノベルでは、条件に合う場面が複数残って、どれかが気まぐれに選ばれると困る。実行時の挙動は今のまま（並べ替えて順に使う）とし、辞書の検査（`pasta-check-dic-validate`）で知らせる案。
- **演出の属性とぶつからないこと**: 同じ場面に、演出が読む属性（`＆曲：夕焼け`）と、絞り込みが読む属性（`＆好感度：高`）が並ぶ。呼ぶ側が条件に挙げない属性は、選ばれ方に影響しない。この性質を、要件に書いて守る。
- 参照: `.kiro/specs/cue-command-runtime/brief.md`（2026-10-17 に `scene-stage-attributes` をまとめた）、`.kiro/steering/roadmap.md` の「Phase 13」。

## 申し送り（touch-poke-shortcuts より、2026-10-16）
- `search_scene` の本体は私的な関数 `search_scene_by(name, global_scene_name, exact)`（`crates/pasta_lua/src/search/context.rs` 115 行付近）に移り、`search_scene`（91 行付近）は `exact=false` で、新しい `search_scene_exact` は `true` で呼ぶ。空の `filters` で検索する点は変わらない。フィルターを通すときは、完全一致の道（`SceneTable::resolve_scene_id_exact`）も同じ `filters` を受け取る形になっている。参照: `.kiro/specs/completed/touch-poke-shortcuts/design.md` の SearchExact。

## 申し送り（pasta-check-dic-validate より）

- 辞書の検査は `pasta_check validate`（と `release` の段 4 の関門、VSCode 拡張のゴースト全体の診断）として入った。核は `crates/pasta_check/src/validate/`（ファイルシステムに触れないライブラリ・WASM でも動く）。
- 検査を足すときに触る所: 報告の種類 `FindingKind`（`report.rs`。誤り／注意の分類と kebab-case の code・文言の `msg` を 1 か所で持つ）、検査の段（`validate/mod.rs` の `validate_ghost` が段の順に呼ぶ）、マニュアルの章 `book/src/debug/validate.md` の報告の表。
- 種類の追加・分類・出力の 1 行の形を変えるのは本 spec の design.md の Revalidation Triggers に当たる（CI・作者のスクリプト・マニュアル・`wasmBridge.ts` の形の検査に効く）。実行時と同じことを守る一致テスト `crates/pasta_check/tests/validate_parity_test.rs` も合わせて見直す。
- どちらの spec が検査を足すか・誤りか注意かは、この spec の要件で決める論点として渡す（本 spec では決めていない）。

## 申し送り（failure-output-unification より、2026-10-17）

- 上の Constraints にある失敗表記 `【Call失敗：「名前」が見つからない】` は無くなった。Call の失敗は `pasta/failure.lua` の `FAILURE.report(act, record)` から出し、シーンが見つからないときは `【シーンが見つからない：＞書いた名前】`、キーが名前にならないときは `【名前にならない値：対象（説明）】` になる（種類の名前は `FAILURE.LABEL`）。
- 論点「一致する候補が無いとき」は、この出口に載せる前提で決め直す。種類を足すなら `FAILURE.LABEL` に足し、表記の形・エスケープ・`[logging] balloon_warning` の切り替えは出口に任せる。
- 動的コールで関数の参照が先に報告したときは、`call_key` が重ねて出さない（「報告済み」の印 `take_fn_failed`）。フィルターの評価を入れる位置は、この印の読み取りより後になる。
- 参照: `.kiro/specs/completed/failure-output-unification/design.md` の「報告箇所」「重ねない規則」、マニュアル `book/src/reference/runtime-failures.md`。

## 申し送り（runtime-error-balloon より、2026-10-17）

- `FAILURE.LABEL` に `SCRIPT_ERROR = "スクリプトのエラー"`・`TIMEOUT = "時間切れ"` が入った（10 種）。種類を足すときは、hello-pasta のテスト（`tutorial_stages_test.rs`）が `LABEL` を全部読んで見張ることを前提にしてよい。
- `FAILURE.format` は `FAILURE.body`（枠なし・無効化なし）を無効化して【】で囲む形になった（出力は不変）。表記の形を変えるときは `FAILURE.body` も見る。
- 生成コードのシーン関数の先頭は `act:init_scene(SCENE, "書いたグローバル名", "書いたローカル名"|nil, "<.pasta>|<lua>:<pasta>,...")` になった。act は `scene_name`・`local_scene_name`・`line_map`・`scene_names` を控え、`call_restore` は 4 つを戻し、`restore_scene` はグローバル名だけを戻す。`act.lua`・コード生成・`init_scene` の行の数に触るときは、行の対応（`transpiler.rs` の `fill_line_maps`）と止まるエラーの表記を見直す。
- 参照: `.kiro/specs/completed/runtime-error-balloon/design.md` の Revalidation Triggers と `research.md` の「実装の記録」、マニュアル `book/src/reference/runtime-failures.md`「止まるエラーの表記」。

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**: `failure-output-unification` と `runtime-error-balloon` が完了し、未完了の前提は `scene-attribute-store` だけになった。`runtime-error-balloon` は `act.lua` の途中の Call の戻し（`call_restore`）を、書いた名前 2 つと行の対応も戻す形にした（上の申し送り）。`attrs` を第 3 引数で `act:call` へそのまま渡す点は変わらない。Call の Lua を出す `element_gen.rs` は変わっていない。場面の関数の頭の `init_scene` の引数が増えたので、生成した Lua の期待値（スナップショット）は全部変わった。絞り込みを足すと、Call を含む期待値がまた変わる。
- **行番号の直し**: `act.lua` は 829 行（`find_scene` 680 行、`call` 695 行、その中の `find_handler("scene", key)` 709 行、`search_dictionary` の `SCENE.search(key, scene_name)` 336 行）。`scene.lua` は 240 行（`SCENE.search` 150 行、`search_scene` を呼ぶのは 160 行、`SCENE.co_exec` 208 行）。`search/context.rs` は 788 行（`search_scene` 91 行、`search_scene_exact` 105 行、空の `filters` を作る `search_scene_by` 115〜121 行）。`scene_table.rs` は 483 行（`resolve_scene_id` 179 行、`resolve_scene_id_unified` 226 行、`resolve_scene_id_exact` 251 行、`filter_by_attributes` 287 行、`find_scene` 453 行）。Call の Lua は `element_gen.rs` 205〜236 行。マニュアル `book/src/lua/script-api.md` の「使わない」は 318 行と 331 行。
- **触るファイル**: 文法 `crates/pasta_dsl/src/parser/grammar.pest`（272。Call の規則 170 行）、`parse_action.rs`（456）、`parser/ast/action.rs`（323）、`partial.rs`（477）、`element_gen.rs`（564）、`act.lua`（829。1,000 行に近づいている）、`scene.lua`（240）、`crates/pasta_lua/src/search/context.rs`（788）・`search/mod.rs`（84）、`pasta_core` の `registry/scene_table.rs`（483）・`scene_types.rs`（36）とその試験（`scene_table_resolve_filter_tests.rs` 387・`scene_table_candidate_tests.rs` 717）、`crates/pasta_core/README.md`、`visit_action.rs`（431）、`pasta.tmLanguage.json`（272）、候補が複数残る場面の検査を足すなら `crates/pasta_check/src/validate/`、マニュアル（`grammar/call-jump.md`・`lua/script-api.md`・`internals/registry-search.md`・`debug/validate.md`）と生成スキル、期待値ファイル。新しいファイルは無い見込み。
- **規模**: 19〜21 タスク（前回の 17〜20 に、0.5.0 の申し送りの「条件の値に変数」「それ以外の場面」「候補が複数残るときの検査」を足した）。上限を少し超えうるが、一度分けた spec なので、もう分けない。
- **先に要るもの**: `scene-attribute-store`（属性の保持と値の型）。ファイルの重なり: `scene-attribute-store`（文法・`partial.rs`・`element_gen.rs`・`act.lua`・`scene.lua`・`search/context.rs`・`scene_table.rs`・`scene_types.rs`）、`scene-anchor-link`（文法・`parse_action.rs`・構文木・`partial.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義・`visit_action.rs`・検査の段）、`cue-command-runtime`（文法・`act.lua`・検査の段）、`novel-talk-flow`（`act.lua`・`scene.lua`・`search/context.rs`・`scene_table.rs`・検査の段）、`novel-checkpoint-rollback`（`act.lua`・`scene_table.rs`）、`shell-element-click`（`act.lua`）、`pasta-check-validate-edges`（検査の段の `validate/mod.rs`）。
- **種別**: 機能（`＞シーン＆k＝v` は今は構文エラー）。0.5.0 のフラグによる分岐の本命（開発者の決定）。
- **要件定義のモデル**: Fable のまま。後から戻しにくい決定は、Call の絞り込みの書き方と意味である（新しい文法で、見本のノベルと作者の辞書がそのまま使う）。2026-10-10 の discovery が決めたのは方向（分岐は Call の絞り込みで書く・属性の倉は 1 つ）だけで、推奨の無い論点が 3 つ以上、絡み合って残る。比べる記号の種類と数のしきい値の向き（場面の属性と呼ぶ側の値のどちらを左に置くか）、条件のつなぎ方（かつ・または）、「それ以外」の場面と候補が無いときの動き（候補の控えのキーとも絡む）、条件の値に変数を書く形と連結の `＆` との見分け。値の型の解釈も、まだ `scene-attribute-store` の要件で決まっていない。
- **分割の案**: 分けない（2026-10-04 に分けた spec）。
- **見つけた穴・古くなった記述**:
  - 冒頭の「ステータス」の「Wave 5」と、「Constraints」の「並走条件（Wave 5）: Wave 5 はこの spec だけ」は古い。今は 0.5.0 の道筋の 5 番で、ウェーブは 0.5.0 の棚卸で組む。
  - 「Constraints」の失敗表記 `【Call失敗：…】` は無くなった（申し送り（failure-output-unification より）のとおり）。
  - 「候補が 2 つ以上残るとき」の「辞書の検査（`pasta-check-dic-validate`）で知らせる案」は、その spec が完了したので、この spec が `validate` に足す（または `novel-talk-flow` に渡す）と読み替える。足すならタスクに数える。
  - イベントから場面を起こす `SCENE.co_exec`（`scene.lua` 208 行）も `attrs` を受けて捨てている。

## 申し送り（pasta-check-validate-edges より）

- `pasta_check validate` の飛び先の検査は、辞書の中身が分からないとき（構文エラー・読めない辞書ファイル・pasta.toml の誤り）に丸ごと省く形になった。判定は `crates/pasta_check/src/validate/report.rs` の `skip_reasons`（統計の `skipReasons`）で、すべての段の報告がそろった後に 1 回だけ行う（`validate/mod.rs`）。
- 見直しが要る論点: この spec が `validate` に辞書のシーンの集まりに頼る検査を足すなら、`skip_reasons` が空のときだけ走らせるか。辞書の中身が分からないことを表す誤りの種類を足すなら、`skip_reasons` の対応に入れるか。
- 参照: `.kiro/specs/completed/pasta-check-validate-edges/design.md` の「Revalidation Triggers」と「Report（report.rs）」の省く理由の表。

## 2026-10-18 棚卸の再測定（main facd6cf4）

- **前提の変化**: 前回（main b4164100）から入った 4 本のうち、この spec の持ち場に触ったのは `pasta-check-validate-edges` だけ。辞書の検査の段の順（`crates/pasta_check/src/validate/mod.rs` 67〜103 行）で、飛び先の検査 `references::check` は省く理由（`skip_reasons`）が空のときだけ走る（86〜92 行）。報告の種類は `actor-display-style` の `ConfigActorFont` が増えて 25 種（`report.rs` 56 行の `FindingKind::ALL`）。VSCode 拡張の `wasmBridge.ts` は報告の code を文字列のまま受けるので、種類を足しても TS は直さなくてよい（省く理由を足すときだけ、`wasmBridge.ts` 50 行の `SKIP_REASONS` も直す）。
- **変わっていないもの**: Call の文法・生成・`act.lua`・`scene.lua`・`search/`・`pasta_core` の登録表・LSP（`visit_action.rs`）・VSCode の文法定義。2026-10-17 の節の行番号はそのまま合う（Call の Lua を出す `generate_call_scene` は `element_gen.rs` 212 行）。
- **触るファイル**: 2026-10-17 の節と同じ。候補が複数残る場面の検査を足すなら、`validate/references.rs`（135。Call の腕は 32〜56 行）・`report.rs`（475。前回の 379 から増えた）、一致テスト `crates/pasta_check/tests/validate_parity_test.rs`、マニュアル `book/src/debug/validate.md` の報告の表（163〜164 行の並び）。検査は `pasta_core` の `SceneTable` で飛び先を探す（`references.rs` 5 行）ので、絞り込みの形を変えると検査の側もそろえる。1,000 行に近いのは `act.lua`（829）だけ。新しいファイルは無い見込み。
- **規模**: 19〜21 のまま。2026-10-04 に一度分けた spec なので、20 をわずかに超えても分けない。
- **先に要るもの**: `scene-attribute-store`（属性の保持と値の型）。ファイルの重なり: `scene-attribute-store`（文法・`partial.rs`・`element_gen.rs`・`act.lua`・`scene.lua`・`search/context.rs`・`scene_table.rs`・`scene_types.rs`）、`scene-anchor-link`（文法・`parse_action.rs`・構文木・`partial.rs`・`element_gen.rs`・`act.lua`・VSCode の文法定義・`visit_action.rs`・`validate/references.rs`・`report.rs`）、`cue-command-runtime`（文法・`act.lua`・検査の段）、`novel-talk-flow`（`act.lua`・`scene.lua`・`search/context.rs`・`scene_table.rs`・検査の段）、`novel-checkpoint-rollback`（`act.lua`・`scene_table.rs` ほか）、`novel-variable-scope`（文法・`element_gen.rs`・`act.lua`・VSCode の文法定義）、`shell-element-click`（あちらが `scene.lua` に口を足す場合だけ）。`pasta-check-validate-edges` は完了したので、検査の段の重なりは消えた。
- **種別**: 機能（`＞シーン＆k＝v` は今は構文エラー）。0.5.0 のフラグによる分岐の本命（開発者の決定）。
- **要件定義のモデル**: Fable のまま。後から戻しにくい決定は、Call の絞り込みの書き方と意味（新しい文法で、見本のノベルと作者の辞書が上に乗る）。推奨の無い論点が 3 つ以上絡んで残る（比べる記号と数のしきい値の向き、条件のつなぎ方、「それ以外」の場面と候補が無いときの動きと候補の控えのキー、条件の値に変数を書く形と連結の `＆` との見分け）。今回の変化は、このどれにも触れていない。
- **分割の案**: 分けない。
- **見つけた穴・古くなった記述**:
  - 2026-10-17 の節の重なりの `pasta-check-validate-edges`（`validate/mod.rs`）は、完了したので外す。`shell-element-click`（`act.lua`）は誤りで、あちらは `act.lua` を触る予定が無い。
  - 候補が複数残る場面の検査は、`references.rs` の Call の腕に置くのが素直で、そうすれば省く理由があるときは走らない（申し送り（pasta-check-validate-edges より）の論点は「空のときだけ走らせる」を推奨にできる）。種類を足すと `FindingKind::ALL` の数も変わる。
  - 「申し送り（pasta-check-validate-edges より）」の 5 行と見出しの前の空行は改行が LF だった。今回 CRLF にそろえた。
  - roadmap 191 行の「辞書の検査に検査を足す spec は `pasta-check-validate-edges` と同じウェーブに置かない」は、完了したので古い（即時修正の候補）。

## 申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）

ghost_dev の「emo2 開発」「ぱすた開発」セッションに、Lua で組んでいるが DSL で書きたいものを聞いた。いちばん数が多かったのは、**イベントの Reference や保存の変数で振り分けて、合わなければ何も返さない（204）**シーンである。今は「`＊イベント` → `＞lua関数` → `act:call(SCENE.__global_name__, …)`」の 3 段で書き、分岐の中身が 1〜3 行なのに定型が 5〜10 行ある。要件の材料にする実例は次のとおり。

- **2 つの値の等しさ**: `OnShellChanging` は Reference0＝Reference1（同じシェルを選び直した）なら 204、違えば「着替え開始」（emo2 `system.pasta`）。比べる相手が両方とも変数になる。
- **値との等しさ・空**: `OnUpdateComplete` は Reference0＝`changed` なら「更新成功」、それ以外は「更新無し」（emo2 `update.pasta`）。`OnGhostChanging`／`OnGhostChanged` は Reference0 が空か自分の名前なら 204（emo2 `boot.pasta`・`boot.lua`）。`OnBoot` は Reference6＝`halt` かつ Reference7 が空でないなら「起動halt」。
- **値の前方一致**: `OnInstallComplete` は Reference0 が `ghost`／`shell`／`balloon` で始まるかで 4 つに分ける（`ghost with balloon` のような複合形があるため）。動的 Call の前方一致は「呼ぶ名前がシーン名の頭」の向きなので、「値がシーン名で始まる」の逆向きには使えない。
- **複数の条件（かつ）**: 窓際のぱすたさんの `OnMouseClick` は、Reference3＝0・Reference5＝0・Reference4＝`セキュリティボール` で、さらに `＄＊初回待ち` が立っていれば「初回の続き」、立っていなければ「ボール巡回」（`window.pasta`）。`OnMouseEnter`／`OnMouseLeave` は Reference4＝`閉じる` かつ Reference3＝0 で、Status に `talking` を含まないときだけ動く。
- **数の範囲**: 起動の挨拶を時刻で振り分ける（emo2 は 4 帯、窓際のぱすたさんは 5 帯。`＄時` の範囲）。
- **一度だけ**: `＊ギャル抽選` は `＄＊ギャル初対面済み` が立っていなければ「ギャル出会い」を出してフラグを立て、立っていれば `＞ギャル_`（窓際のぱすたさん `talk_gal.pasta`）。
- **候補が無いとき**: 両ゴーストとも「条件に合う候補だけで抽選し、どれも合わなければ 204」を望んだ。中身の無いシーンは今も 204 になる（`act:build()` が nil。`book/src/lua/shiori-events.md` L47-48・L630）ので、「候補なし」を何も出さない扱いにすれば、204 を返す専用の書き方は要らない。
- **作者が想像した書き方**（文法は想像）: 呼ぶ側の `？＄ｒ０＝＝「changed」　＞更新成功`、受ける側の `＊OnMouseClick　＆r4：セキュリティボール　＆＄＊初回待ち`、前方一致の `＆r0^：shell`。両方向（呼ぶ側で絞る・場面の側に条件を書く）の要望がある。2026-10-10 の決定（分岐は Call の絞り込みで書く）のどちらの向きで受けるかを、要件で決める。
- **Reference の読み方**: 条件に Reference を使うには、今は先に `＞transfer_req_to_var` が要る（自動では入らない。`book/src/grammar/variables.md` L345・L379）。イベントのシーンで自動で転記する件は roadmap の Direct Implementation Candidates に置いた。こちらの要件で Reference を直接書ける形にするなら、そちらは要らなくなる。
- 今の DSL だけで書ける分（`OnInstallRefuse` の転記だけ、季節の動的 Call）は、両セッションへ書き方を伝えた。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**: `scene-play-event` が、シーンを探す道を 1 本足した。Call の道（`act:call` → `find_handler` → `SCENE.search` → `search_scene`）は変わっていない。
  - シーン表 `crates/pasta_core/src/registry/scene_table.rs` に `resolve_global_owning_local_exact`（297 行）が増えた。指名したローカルシーンを持つグローバルシーンだけを候補にする。絞り込み条件 `filters` を受ける関数は、これで 4 つになった（`resolve_scene_id`・`resolve_scene_id_unified`・`resolve_scene_id_exact` と、この関数）。条件の形を変えるなら、これとその試験（`scene_table_resolve_filter_tests.rs` に 3 本増えて 497 行）も合わせる。
  - 候補の控えのキー `SceneCacheKey` に「持ち主のローカル名」の欄が増えた（25〜26 行）。条件をキーに含める作り直しでは、この欄を残す。
  - 検索の入口 `crates/pasta_lua/src/search/context.rs` に、グローバルとローカルの名前の組で完全一致に探す `search_local_scene_exact`（204 行）が増え、ここも空の条件で探す（215 行）。Lua 側は `SCENE.co_exec_exact(名前, ローカル名)`（`scene.lua` 230 行）で、外から指名したシーンを再生するイベント（`OnPastaScene`）と撫で・つつきの近道が使う。`attrs` を受けない。要件で「指名の再生は絞り込まない」と書いておくのを推奨する。
  - `ontalk-suppression` が、区切りの関数 `ACT_IMPL.yield`（`act.lua` 664 行）の頭に、自動トークを止めている間の見張りを足した。Call の道には触れていない。
- **触るファイル**: 2026-10-17 の節と同じ。行数は `act.lua` 837（1,000 行に近い）、`scene.lua` 266、`search/context.rs` 1,017（本体は 437 行まで、残りは中のテスト。試験を足すなら別のファイルに置く）、`scene_table.rs` 539、`scene_table_resolve_filter_tests.rs` 497。文法・Call の Lua を出すコード生成・構文木・編集中の解析・エディタの着色・VSCode の文法定義・辞書の検査は変わっていない。
- **規模**: 19〜21 のまま（増えた関数 1 つを合わせる分は小さい）。2026-10-04 に一度分けた spec なので、分けない。
- **先に要るもの**: `scene-attribute-store`。ファイルの重なり（前回のものに足す）:
  - `scene-exact-match`（Call の文法 `call_scene`、Call の Lua を出す `element_gen.rs`、`act.lua` の 5 段の探し方 `find_act_handler` と `search_dictionary`、`scene.lua` の `SCENE.search`、`search/context.rs` の `search_scene_by`、エディタの着色、VSCode の文法定義）。どちらも `act:call` の 3 つ目の引数のあたりを使いたくなるので、先に入る側が受け渡しの形を決める。`scene-exact-match` が先なら、`attrs` を条件のために空けておく約束にする。
  - `chain-talk-interrupt`（`act.lua` の `ACT_IMPL.build` のあたり。同じファイルなので同じウェーブに置かない）。
  - 隙間の spec: `ponytail-dsl-parser`（文法）、`ponytail-lua-transpiler`（`element_gen.rs`）、`ponytail-core-registry`（`scene_table.rs`・`search/context.rs`・`pasta_core` の README）、`ponytail-editor-tooling`（`visit_action.rs`・文法定義・`partial.rs`）、`ponytail-lua-scripts`（`act.lua`・`scene.lua`）。
- **種別**: 機能（`＞シーン＆k＝v` は今は構文エラー）。0.5.0 のフラグによる分岐の本命（開発者の決定）。
- **要件定義のモデル**: Fable のまま。新しい書き方と意味（見本のノベルと作者の辞書が上に乗る）で、推奨の無い論点が 3 つ以上絡む（比べる記号としきい値の向き、条件のつなぎ方、「それ以外」の場面と候補が無いときの動き、条件の値に変数を書く形と連結の `＆` との見分け）。今回の変化は、このどれにも触れていない。
- **分割の案**: 分けない。
- **見つけた穴・古くなった記述**:
  - 2026-10-17 の節の行番号: `act.lua` 829 → 837（`find_scene` 680 → 688、`call` 695 → 703、その中の `find_handler("scene", key)` 709 → 717。`search_dictionary` の `SCENE.search` 336 は同じ）。`search/context.rs` 788 → 1,017（`search_scene` 91 → 105、`search_scene_exact` 105 → 119、`search_scene_by` 115〜121 → 129〜135）。`scene_table.rs` 483 → 539（`resolve_scene_id` 179 → 183、`resolve_scene_id_unified` 226 → 230、`resolve_scene_id_exact` 251 → 255、`filter_by_attributes` 287 → 343、`find_scene` 453 → 509）。`scene.lua` の 150・160・208 行は同じ。

## 申し送り（external-control-events より、2026-10-23）

- `FAILURE.LABEL` に `NOT_JSON = "JSON として読めない値"`・`NOT_ASSIGNABLE = "代入できない値"` が入り、12 種になった（上の「10 種」は古い）。種類を足すときは `failure_test.lua` の数のテストも合わせて直す。
- 既定のハンドラの読み込みの並び（`pasta/shiori/event/init.lua`）の末尾に、`scene_play` の次の 1 行として `external_control` が入った。読み込みの行を足すときはその後ろになる。
