# ギャップ分析: ponytail-dsl-parser

## Summary

- **Feature**: `ponytail-dsl-parser`
- **Discovery Scope**: Extension（既存の `pasta_dsl` のパーサの直しと撤去。新しい部品は、深さの守りの走査と数の値の補助くらい）
- **測った所**: ワークツリーの HEAD `ae04a28b`（main と同じ）。`crates/pasta_dsl` の最後のソースの変更は 2026-10-05 の `44aa8761`（#69）で、その後は crates.io のキーワードの #166 だけ。2026-11-02。
- **Key Findings**:
  - 控えの 1〜5 番は、**すべて残っている**。行番号も brief の 2026-11-02 の再測定のとおり。ただし brief の「18 か所」は、数え直すと `Span::from` の呼び出しが 17 か所（3 つのファイルに 16、`parser/mod.rs` の `parse_actor_scope` に 1）だった。
  - brief が「拾うかは要件で決める」とした `review-improvement-loop` の 3.3 のバグ（括弧式の第 1 項だけを包む）は、**監査より前の 2026-10-04 に `14be2195` で直っていた**。回帰テスト `test_paren_binary_keeps_all_terms` もある。消えた指摘として扱う。
  - 監査のやり直しで、同じ領域に **新しい指摘が 7 つ** 増えた（下の N1〜N7）。いちばん重いのは N1（`＋1` を 1 万個つないだ 1 行で、括弧が無くてもスタックがあふれる）。
  - 測り直した解析の時間は、8,000 行で 2.26 秒（pest だけなら 17 ms）。brief の 10.6 秒より速いが、2,000 行の 141 ms から 16 倍で、2 乗のまま。
  - 規模は M・危険は低〜中。どの直しも `pasta_dsl` の中で閉じる。危険は「span の値が変わらないこと」と「新しい誤りが正しい辞書を誤検知しないこと」の証明にある。

## 監査のやり直し（2026-10-19 の控えとの突き合わせ）

`/ponytail:ponytail-audit` の観点（使われない部品・テストだけが使う道・黙って捨てる誤り・起きない場合の守り・同語反復・データが化ける端）で、`parser/ast/span.rs`・`parse_scene.rs`・`parse_action.rs`・`parse_elements.rs`・`parser/mod.rs`・`error.rs`・`lib.rs` を読み直した。`partial.rs` は除いた（`ponytail-editor-tooling` の持ち場）。プラグインのスキルは呼ばず、コードを読み、スクラッチの probe クレート（`pasta_dsl` をパスで参照）で確かめた。cargo test と clippy は走らせていない。「使われない」の判断は、`crates/` 全体の grep による。

### 控えの指摘の今

| # | 指摘 | 今 | 今の場所 |
| - | ---- | -- | -------- |
| 1 | 解析が 2 乗に遅い | 残る | `ast/span.rs:120-137`（`impl From<&pest::Span>`。`start_pos().line_col()` と `end_pos().line_col()` が、どちらも入力の先頭から数え直す）。呼び出しは `parse_scene.rs:11・151・182・195・274・310・346`、`parse_action.rs:7・37・62・79・233`、`parse_elements.rs:7・51・92・117`、`parser/mod.rs:228`（brief の 18 でなく 17 か所）。ほかに `parse_scene.rs:93-94` が誤りのときだけ 1 回 `line_col()` を呼ぶ（誤りの道なので遅さには効かない） |
| 2 | 大きな数の字句が化ける | 残る | `parse_action.rs:351-358`（`try_parse_expr` の `parse().ok()?`）・`parse_elements.rs:40-47`（属性の `unwrap_or(0)`）・`parse_scene.rs:404-410`（キューの `unwrap_or(0)`）・`parse_scene.rs:160-163`（アクターの番号の `.ok()`）。`try_parse_expr` は、変数の代入（`parse_elements.rs:160`）・位置の引数と Call の動的ターゲット（`parse_action.rs:279`）・名前つきの引数（`parse_action.rs:310`）の 3 つの道から呼ばれる |
| 3 | 作られない誤りの型 | 残る | `error.rs:10-71`。作るのは `parser/mod.rs:105`・`parse_scene.rs:95` の `SyntaxError` と、`parse_file` の `?` が使う `From<io::Error>`（`error.rs:73-77`）だけ。`PestError`・`MultipleErrors`・`ParseErrorInfo`・`ParseResult`・`syntax_error`・`pest_error`・`io_error` を使うのは `tests/error_api_test.rs` だけ。ほかのクレートの使い手は `pasta_check/src/validate/syntax.rs`（`SyntaxError` と `other =>`）と `pasta_check/tests/validate_rules_test.rs:215`（`SyntaxError`）だけ |
| 4 | `PastaFile.span` の誤り | 残る | `parser/mod.rs:163-166`。終わりの桁が `l.len()`（バイト長）。probe の `"＊挨拶\r\n  さくら：こんにちは\r\n  ＄x＝（１＋２）\n"` で `end_col: 25`（文字なら 11）、`end_line: 3`（ほかの節の決まりなら、最後の改行の後の 4 行 1 桁）。読むのは `error_api_test.rs:194-195`・`span_byte_offset_test.rs:177-179・203` だけ |
| 5 | 深い括弧の入れ子でスタックがあふれる | 残る | 文法の再帰は `term → paren_expr → expr → term` と `term → fn_call → args → expr → term` の 2 つで、どちらも開き括弧を通る。pest 2.9.2 の `set_call_limit` は「深さ」でなく呼び出しの**累計**を数える（`parser_state.rs:92-105`）ので、大きい辞書ほど誤検知する。深さの守りには使えない（控えの「深さを制限する API が無い」のとおり） |

### 消えた指摘

- **3.3 の括弧式のバグ**（brief の Out of Boundary）。`parse_action.rs:411` は `parse_expr_from_parts` で括弧の中を全部読む。2026-10-04 の `14be2195`「括弧式の中の演算を最初の項だけでなく全体で解析する」で直り、`tests/expr_parse_test.rs:171` の `test_paren_binary_keeps_all_terms` が守る。控え（2026-10-19）を作った時点で、もう古い記述だった。要件 8.2 で回帰テストを保つことだけを書いた。

### 新しく見つけた指摘（同じ領域）

- **N1. 長い式で、括弧が無くてもスタックがあふれる**（バグ・5 番と同じ症状）
  - `＄x＝1＋1＋1…` は、`build_left_assoc_expr`（`parse_action.rs:333-346`）が左に深い `Expr::Binary` の木にする。項の数だけ深くなる。
  - probe（release・1 MiB のスレッド）で、`＋1` が 7,000 個なら通り、10,000 個であふれた。2 MiB では 10,000 個が通り、20,000 個であふれた。
  - `std::mem::forget` で片付けを飛ばすと 100,000 個でも通るので、あふれるのは解析でなく **AST の片付け（`Box<Expr>` の再帰の Drop）**。`pasta_lua` のコード生成（`expr_gen.rs`）も式を再帰でたどるので、片付けの前にそこであふれる見込みもある（測っていない）。
  - 括弧の深さの守りでは防げない。要件 3.2 に【前提】で書いた（上限 256）。
- **N2. シーン引数の番号 256 以上が、黙って `＄０` になる**（2 番と同じ種類）
  - `parse_action.rs:445-447` が `parse::<u8>().unwrap_or(0)`。probe で `＄２５６` は `VarScope::Args(0)`。
  - マニュアル `grammar/variables.md:93` が「256 以上を書くと 0 として扱われ、`＄０` と同じ値になる」と書いている。書かれた挙動だが、brief の「黙って誤った値になる」と同じ形である。要件 2.8 に【前提】で書いた（構文の誤りにする）。
- **N3. 有限の小数に収まらない数が、空の文字列か `inf` になる**（2 番と同じ根）
  - 400 桁の整数は、今は `i64` の失敗で空の文字列（probe で `Expr(BlankString)`）。brief の「だめなら f64」をそのまま入れると、Rust の `parse::<f64>()` は `inf` を返し、コード生成（`expr_gen.rs:147` の `write!("{}", f)`）は Lua に `inf` と書く。Lua では未定義の大域変数で `nil` になる。
  - 小数点のある数（`1000…0.5`）は、今でも `inf` になる（`unwrap_or(0.0)` は数字だけの字句では失敗しない）。
  - 要件 2.5 に【前提】で書いた（構文の誤りにする）。コード生成に `math.huge` を書かせる案は、`ponytail-lua-transpiler` の持ち場に触る。
- **N4. 扱っていない規則を、項として黙って拾う**（小物・次の不具合の種）
  - `try_parse_expr` の既定の腕（`parse_action.rs:421-429`「Try recursively for nested expressions」）が、扱いの無い規則の中をたどり、最初の値を項にする。
  - 同じ形の不具合が 2 回出て、そのたびに明示の腕を足している（`parse_action.rs:403`「the default arm would recurse into args and return the first argument」・`parse_elements.rs:150`）。
  - 今の文法で項の位置に来る規則は、すべて明示の腕がある。既定の腕に届く入力は、読んだ限り無い（設計で確かめる。Research Needed）。要件 6.1。
- **N5. テストだけが使う `Span::byte_len`**（小物・撤去）
  - `ast/span.rs:115-118`。使うのは `tests/span_byte_offset_test.rs:157-164` だけ。要件 6.2。
- **N6. キューコマンドのスコープの分け方の、起きない場合の守り**（小物）
  - `parse_scene.rs:352-379`。`find` で見つけた位置の文字を `if let Some` で守り（「for defensive safety」）、文法上届かない戻り値（「should not be reached」）を持つ。`split_once` 1 つで同じ結果になる。要件 6.3。
- **N7. 同語反復の分岐**（小物）
  - `parser/mod.rs:188-189` の `Rule::EOI => {}` と `_ => {}`。要件 6.4。

### 指摘にしなかったもの（記録）

- **`Span::extract_source` と `SpanError`**：本番では使われず、テストだけが使う（`span_byte_offset_test.rs`・`dynamic_word_ref_test.rs`・`partial_parse_test.rs`）。ただし、テストが span の値を確かめる道具として 3 つのファイルで使い、逆向きの span を `OutOfBounds` にする守り（`review-improvement-loop` 3.4）もここにある。消すと `ponytail-editor-tooling` の持ち場の `partial_parse_test.rs` に触る。要件には入れず、持ち越し 7 に書いた（推奨は残す）。
- **`SyntaxError` の文言の二重の位置**：`Display` は `Parse error: {file}:{line}:{column}: {message}` で、`message` 自体も `Parse error in {file} at {line}:{column}: …` で始まる。位置が 2 度出る。だが `pasta_check` は `message` の前置きを外して読み（`syntax.rs:45-46`）、部分解析は `Display` の文字列を誤りの文にしている。roadmap の「同じウェーブの約束」で `parse_str` の誤りの形を保つことになっているので、直さない。新しい誤りもこの形に合わせる（要件 4.3）。
- **`parse_global_scene_start` の誤りの文言**（`parse_scene.rs:99`）：`Parse error in …` の前置きが無い。`pasta_check` は前置きが無ければ文言をそのまま使うので、困りごとは無い。今の挙動として残す。
- **たくさんの関数が、失敗しないのに `Result` を返す**（`parse_attr`・`parse_key_words`・`parse_args` など）：要件 2.5・2.7・2.8・3.2 で、数の範囲と式の長さの誤りを返すようになるので、`Result` は要る形になる。指摘にしない。逆に `try_parse_expr` は `Option` を返し、`parse_fn_call_inner(pair).ok()?`（`parse_action.rs:387・395・404`）で誤りを飲む。引数の中の数の誤りが黙って消えないよう、設計で誤りを上へ返す形にする（設計への申し送り）。
- **`build_left_assoc_expr` が余った演算子を捨てる**（`parse_action.rs:330-338` の zip）：今の文法では、項と演算子の数がずれるのは、項が落ちたとき（2 番）だけである。2 番を直せばずれなくなる。2 番に含めた。
- **`lib.rs`・`parser/mod.rs` の古い名前**（`PastaParser2`・`pasta2.pest`・「Pest 2.8」）：説明の古さだけで、挙動に関わらない。`lib.rs` の `ParseErrorInfo` の記述だけは、要件 4 の撤去で嘘になるので要件 7.2 で直す。
- **`ast/` の span.rs 以外**：監査の対象の外（brief の領域に無い）。`PastaFile` の `span` の欄は要件 5 で消すので、`ast/mod.rs` には触る。

## 測った値（2026-11-02・probe）

probe は、スクラッチの使い捨てクレート（`pasta_dsl` を worktree のパスで参照・pest 2.9.2）。x64・release（深さの一部は debug も）。

| 測ったもの | 結果 |
| ---------- | ---- |
| 1 シーン 4 行の辞書の `parse_str` | 2,000 行で 141 ms（pest だけ 4.3 ms）、8,000 行で 2.26 秒（pest だけ 17 ms）。4 倍の行で 16 倍 |
| 括弧の入れ子 `（（…1…））`、release・1 MiB | 650 段は通り、700 段であふれる |
| 同、release・256 KiB | 120 段は通り、150 段であふれる |
| 同、debug・1 MiB | 180 段は通り、200 段前後であふれる |
| 関数呼び出しの入れ子 `＠f（＠f（…`、release・1 MiB | 200 段は通り、300 段であふれる |
| 同、debug・1 MiB | 100 段は通り、150 段であふれる |
| `＋1` の長い式、release・1 MiB | 7,000 個は通り、10,000 個であふれる（片付けを飛ばすと 100,000 個でも通る） |
| 同、release・2 MiB | 10,000 個は通り、20,000 個であふれる |
| SSP の主スレッドのスタック | `C:\wintools\ssp\ssp.exe` は 32 ビットの PE で、`SizeOfStackReserve` が 2 MiB |
| リポジトリの `.pasta` と `.md` の 1 行の括弧の重なり | いちばん深い行で 8 段（完了した spec の設計の文書）。マニュアルの例は 4 段まで |

大きな数（`Debug` の表示から）:

| 辞書 | 今の AST |
| ---- | -------- |
| `＄x＝10000000000000000000` | `Expr(BlankString)` |
| `＄y＝1＋10000000000000000000` | `Expr(Integer(1))`（演算子も消える） |
| `＞t（10000000000000000000、2）` | 引数 `[Positional(Integer(2))]` の 1 つ |
| `!wait(99999999999999999999)` | `[Integer(0)]` |
| `％a、b＝4294967296` | 番号 `0`・`1` |
| `＆n：99999999999999999999` | `AttrValue::Integer(0)` |
| `＄k＝＠f（a：10000000000000000000）` | `Keyword { key: "a", value: BlankString }`（N に入れず 2 番の 1 例） |
| `a：＄２５６` | `VarScope::Args(0)`（N2） |
| `＄z＝9223372036854775808` | `Expr(BlankString)`（i64 の上限＋1） |
| `＄z＝-9223372036854775808` | `Integer(-9223372036854775808)`（i64 の下限は通る） |
| 400 桁の整数 | `Expr(BlankString)`（N3） |

## brief の読み方（食い違いの解き方）

- **「18 か所」**：数え直して 17 か所。`parser/mod.rs:228`（`parse_actor_scope`）を含めて全部を置き換える。
- **「8,000 行が 1 秒未満」**：この機械では今 2.26 秒。目標はそのまま使える（要件 1.1）。機械による差を除くため、伸び方の比（要件 1.2）も要件にした。
- **「i64 に入らなければ f64（Lua と同じ）」**：pasta の実行時は LuaJIT（数はすべて 64 ビットの浮動小数点数）なので、i64 の範囲の中でも 2^53 を超える整数は実行時に丸まる。brief の規則は「パーサが値を落とさない」ための規則として採り、マニュアルには「下の桁は丸められる」と書く（持ち越し 9）。
- **アクターの番号**：brief は「`u32::MAX` に飽和させるか構文の誤りにするかを、要件で決める」。推奨は構文の誤り（持ち越し 1）。
- **深さの上限**：brief は「正しい辞書を誤検知しない上限は、要件で決める」。推奨は 1 行 64 段（持ち越し 2）。

## マニュアル

直す所（要件 7）:

| 頁 | 今 | 直し |
| -- | -- | ---- |
| `grammar/literals.md` 77-90 行「数値（i64 / f64）」 | 範囲の記述が無い | 64 ビットの整数に収まらない整数は小数になる（下の桁は丸められる）・有限の小数に収まらない数は構文の誤り |
| `grammar/variables.md` 93 行 | 「256 以上を書くと 0 として扱われ、`＄０` と同じ値になる」 | 256 以上は構文の誤り（持ち越し 5 で決まれば） |
| `grammar/variables.md` 198 行「優先順位と括弧」 | 上限の記述が無い | 1 行の括弧の重なり 64 段・1 つの式の演算子 256 個まで |
| `grammar/actor-dictionary.md` 52 行 | 範囲の記述が無い | 0〜4294967295 を超える番号は構文の誤り |
| `internals/transpiler.md` 144-150 行「パースと AST 構築時の正規化」 | 数の正規化だけ | 数の値の規則・範囲の検査・深さと長さの守り |
| `crates/pasta_dsl/README.md` 17・60-62 行 | `ParseErrorInfo`・`PestError`・`MultipleErrors`・`ParseResult` を挙げる | 今の型に書き直す（マニュアルではないが、要件 7.2） |

`literals.md`・`variables.md`・`actor-dictionary.md` は `gen-skill-refs.mjs` の元（`book/tools/gen-skill-refs.mjs:44・47・49`）なので、`.claude/skills/pasta-ghost-authoring/references/` を作り直す（要件 7.3）。

## Requirement-to-Asset Map

| 要件 | 今ある部品 | ギャップ |
| ---- | ---------- | -------- |
| 1.1-1.2 速さ | `Span::from(&pest::Span)`（17 か所） | **Missing**：入力の先頭から数え直さない span の作り方。pest 2.9.2 の `Pair::line_col()`（`iterators/pair.rs:290`。`Pairs` が 1 回だけ作る行の索引 `LineIndex` を二分探索する）が始まりに使える。終わりの位置の API は無いので、`pair.as_str()` の中を数える |
| 1.3 span の値 | テスト `span_byte_offset_test.rs` ほか | **Constraint**：今の値との一致を、リポジトリの全 `.pasta` で比べる仕組みが無い。古い計算（`Position::line_col`）をテストの中に正解として残して比べるのが簡単 |
| 1.4 pest の誤り | `parser/mod.rs:99-111` | 変えない |
| 2.1-2.6 数の値 | `normalize_number_str` と 4 か所のばらばらの扱い | **Missing**：1 つの補助（brief の `number_value`）。AST の型（`Expr`・`AttrValue`・`CueArgToken` の `Integer(i64)`・`Float(f64)`）は今のままで足りる |
| 2.4 落ちない | `try_parse_expr` の `Option` と `.ok()?` | **Constraint**：数の誤り（2.5）を上へ返すには、`try_parse_expr` が誤りを返せる形が要る |
| 2.7 アクターの番号 | `parse_scene.rs:160-163`・`saturating_add` | 範囲外を誤りにする分岐が要る。誤りの位置は `digit_id` の pair から取れる |
| 2.8 シーン引数の番号 | `parse_action.rs:445-447` | 同上。`VarScope::Args(u8)` の型は変えない |
| 2.9 `pasta_check` | `syntax.rs` の `SyntaxError` の扱い | 直さずに済む（同じ `parse_str` を呼ぶ） |
| 3.1 括弧の深さ | 無い | **Missing**：`parse_str` の入口の、1 行ずつ括弧を数える走査。文法の式は 1 行の中で閉じる（`s` は改行を含まず、文字列も行をまたげない）ので、1 行ずつ数えれば再帰の深さの上限になる |
| 3.2 式の長さ | `build_left_assoc_expr` | **Missing**：演算子の数の上限。3 つの呼び出し元（`parse_expr_from_parts`・`parse_key_arg`・`parse_var_set`）が通る所で 1 回数えられる |
| 3.4-3.5 落ちない | `pasta_lsp/src/analysis/mod.rs` の `catch_unwind`（スタックのあふれは捕まえられない）・`partial.rs` の `parse_str` の呼び出し（143-251 行） | `partial.rs` は第 1 段も各チャンクも `parse_str` を呼ぶので、入口の守りが効く。直さない |
| 3.6 上限の値 | 上の表の測定 | **Unknown**：`pasta_lua` のコード生成の再帰の深さの限界（Research Needed） |
| 4 誤りの型 | `error.rs`・`lib.rs:32` | 撤去と、テスト `error_api_test.rs` の直し |
| 5 `PastaFile.span` | `parser/mod.rs:163-166`・`ast/mod.rs:96` | 撤去と、テスト 2 つの直し |
| 6 小物 | 上の N4〜N7 | 撤去・簡単化 |
| 7 マニュアル | 上の表 | 書き直し |
| 8 保つ挙動 | 既存の 24 本の統合テスト | **Missing**：直す前と後の AST を全 `.pasta` で比べる、振る舞いを固めるテスト |

## Implementation Approach Options

### Option A: 既存の部品の中で直す（brief の Approach そのまま）

- span：`Span::from_pair(&Pair)` を足し、始まりは `pair.line_col()`、終わりは `pair.as_str()` の `'\n'` の数と、最後の `'\n'` の後の文字の数で出す。17 か所を置き換え、`From<&pest::Span>` を消す。
  - pest の `Position::line_col` は `"\r\n"` を 1 つの改行、単独の `'\r'` を 1 桁と数える。`LineIndex` と「`'\n'` を数える」終わりの計算も同じ数え方になる（読んで確かめた）。全 `.pasta` での一致は、振る舞いを固めるテストで確かめる。
  - 各節が自分の文を 1 回ずつ数えるので、全体の手間は「入力の長さ × AST の深さ（5〜6）」で、行数に比例する。
- 数：`parse_elements.rs` に 1 つの補助を置き、4 か所（＋アクターとシーン引数の番号の範囲）で使う。
- 深さ：`parse_str` の入口に、1 行ずつ括弧の重なりを数える走査を置く。式の長さは `build_left_assoc_expr` の前で数える。
- 誤りの型・`PastaFile.span`・小物：消すだけ。

**Trade-offs**：✅ 変更が `pasta_dsl` の中で閉じ、関数の形もほとんど変わらない。✅ 呼び出し元（`pasta_lua`・`pasta_check`・`pasta_lsp`）を直さずに済む。❌ `try_parse_expr` の戻り値の形は変える必要がある（2.4・2.5）。

### Option B: `parse_str` で行頭の索引を 1 つ作り、二分探索する（brief の別案）

- `parse_str` で行頭のバイトの位置の表を作り、すべての解析の関数に渡す（17 か所の関数の引数が増える）。始まりも終わりも、バイトの位置から二分探索で出す。
- **Trade-offs**：✅ 終わりの位置も索引から出せ、`as_str()` を数える必要が無い。✅ pest の `Pair::line_col` に頼らないので、pest の版の違いに強い。❌ 索引を引数で回す変更が、ほぼすべての関数の形に広がる。❌ pest が中で同じ索引を作っているのと二重になる。

### Option C: 混ぜる

- 始まりは `Pair::line_col()`、終わりは Option B のような小さな索引を `parse_str` で 1 回作って使う。
- **Trade-offs**：✅ 終わりの計算が O(log n) で、深い節が長い文を何度も数えない。❌ Option A の手間（深さ 5〜6 倍）は実害が無く、索引を回す変更の方が大きい。

推奨の方向（設計で決める）：Option A。測った遅さはすべて `Position::line_col` の数え直しで、A で消える。深さの守りは、どの案でも入口の走査になる。

## Effort and Risk

- **Effort: M**（3〜7 日）。brief の見込みの 9〜12 タスクに、N1〜N7 の分で 1〜2 タスク増える。どれも既存の形の中で閉じる。
- **Risk: Low〜Medium**。span の値の一致は、古い計算を正解にしたテストで機械的に確かめられる（Low）。新しい誤り（範囲・深さ・長さ）は挙動の変化で、正しい辞書を誤検知しない証明（リポジトリの全辞書・マニュアルの例）と上限の値の決めごとが要る（Medium）。

## 設計への申し送り

- **span の一致の確かめ方**：古い `From<&pest::Span>` の計算を、テストの中に正解として残す（本番からは消す）。リポジトリの全 `.pasta` と、CRLF・BOM・全角・絵文字・単独の `\r` を含む小さな辞書で、全節の 6 つの値を比べる。
- **速さの確かめ方**：時間のかかるテストは揺れる。8,000 行と 32,000 行の比を測る `#[ignore]` のテストか、release の手順で測って記録するかを、設計で決める。
- **`try_parse_expr` の誤り**：数の範囲の誤り（要件 2.5）が、関数呼び出しの引数の中から `.ok()?` で消えないよう、誤りを上へ返す形にする。`parse_fn_call_inner` などの `.ok()?` 3 か所も同じ。
- **誤りの位置**：数の字句・`digit_id`・65 段目の括弧・257 個目の演算子の pair（か文の位置）から、1 から数えた行と文字の桁を出す。`Pair::line_col()` が使える。文言は `Parse error in {file} at {line}:{column}: ` で始める（要件 4.3）。
- **括弧の走査**：数えるのは `（`・`(` と `）`・`)`。閉じ括弧が開き括弧より多いときは 0 で止める（負にしない）。台詞・文字列・コメント・Lua ブロックの中も数える（持ち越し 2 の推奨）。数え方が文法とずれても、誤検知の向きにしかずれない（数えすぎるだけ）ので、あふれは必ず防げる。
- **BOM**：`parse_str` は先頭の BOM を外してから解析する。走査も外した後の文で行う（誤りの位置を pest の誤りとそろえる）。

### Research Needed

- `Pair::line_col()` が、ワークスペースの下限の `pest = "2.8.6"` にもあるか（今の Cargo.lock は 2.9.2）。無ければ下限を上げるか（`Cargo.toml` は `ponytail-editor-tooling` が席を持つ。同じウェーブの約束）、Option B にする。
- `try_parse_expr` の既定の腕（N4）に届く入力が本当に無いか。腕を `None` にして、全テストと全 `.pasta` の AST の比較で確かめる。
- `pasta_lua` のコード生成（`expr_gen.rs` の再帰）が、括弧 64 段・演算子 256 個の式を、debug・1 MiB のスレッドで変換しきれるか。`cargo test -p pasta_lua` の中で 1 本確かめる（要件 3.6）。
- VSCode 拡張の WASM（wasm32）のスタックの大きさと、上限の式の解析が通るか（要件 3.5）。
- 上の深さの測定は x64 だけである。pasta.dll の本番は x86 なので、x86（`i686-pc-windows-msvc`）の release・debug でも上限の式が通ることを確かめる（要件 3.6。x86 のテストは省かない）。

## 要件ディスカッションへの持ち越し

> 2026-11-02 の要件ディスカッションで、1〜9 をすべて推奨案で確定した（開発者の一括承認）。10 の申し送りは `ponytail-lua-transpiler` の brief に書いた。

1. **アクターの番号が 0〜4294967295 に収まらないとき**（要件 2.7）
   - 案：(a) 構文の誤りにする、(b) `4294967295` に飽和させる、(c) 今のまま（番号の指定を黙って無視して自動の番号）。
   - 推奨：(a)。番号は `\p[番号]` になり、`4294967295` に飽和しても意味のある立ち位置にはならない。飽和は「黙って誤った値」のままである。誤りにすれば、`pasta_check` と pasta.dll が位置つきで知らせる。
   - 例：
     ```pasta
     ＊挨拶
       ％さくら、うにゅう＝4294967296
       さくら：やあ
     ```
     → `Parse error in dic/a.pasta at 2:13: アクターの番号 4294967296 は 0〜4294967295 の範囲を超えています`（文言は設計で決める）。今は、さくら 0・うにゅう 1 で黙って動く。
2. **括弧の重なりの上限と、数える所**（要件 3.1）
   - 案：上限 (a) 64 段、(b) 32 段、(c) 100 段、(d) 256 段。数える所 (x) その行のすべての括弧、(y) Lua ブロックの中を除く。
   - 推奨：(a)＋(x)。リポジトリでいちばん深い行は 8 段、マニュアルの例は 4 段で、64 段はその 8 倍ある。いちばん厳しい測定（関数呼び出しの入れ子・debug・1 MiB）でも 100 段は通るので、64 段はあふれない。(x) は、数えすぎる向きにしかずれないので、あふれを必ず防げ、走査も単純である。(y) は、Lua ブロックの境目の見分けを文法とずらすと、数え損ないの向き（あふれる向き）にずれる。
   - 例：`＄x＝` の後に `（` を 65 個重ねた行 → `Parse error in dic/a.pasta at 2:70: 括弧の重なりが 64 段を超えています`。台詞 `さくら：（（（…` に `（` を 65 個書いた行も、同じ誤りになる（マニュアルに書く）。
3. **1 つの式の演算子の上限**（要件 3.2・N1）
   - 案：(a) 256 個、(b) 1,000 個、(c) 上限を置かず、AST の片付けとコード生成を再帰でない形に直す、(d) 直さない。
   - 推奨：(a)。片付け（release・1 MiB）は 7,000 個まで通ったが、debug やコード生成の再帰はもっと浅い見込みで、256 個なら余裕がある。実際の辞書の式は 10 項もない。(c) は `ast/action.rs` の型と `pasta_lua` の `expr_gen.rs`（`ponytail-lua-transpiler` の持ち場）に手が届く。
   - 例：`＄x＝1＋1＋…`（`＋1` を 257 個）→ 257 個目の `＋` の位置の構文の誤り。
4. **有限の小数に収まらない数**（要件 2.5・N3）
   - 案：(a) 構文の誤りにする、(b) 無限大にし、コード生成に `math.huge` を書かせる、(c) 今のまま。
   - 推奨：(a)。309 桁を超える数を書く辞書は、書き間違いしか考えられない。(b) は `ponytail-lua-transpiler` の持ち場に触り、属性値・キューの引数でも無限大の扱いを決める必要がある。
   - 例：`＄h＝1` の後に `0` を 400 個 → `Parse error … 数 1000…（401 桁）は大きすぎます`。今は `＄h` が空の文字列になる。
5. **シーン引数の番号 256 以上**（要件 2.8・N2）
   - 案：(a) 構文の誤りにする、(b) 今のまま（マニュアルのとおり `＄０` として扱う）、(c) 番号の型を広げて 256 以上も使えるようにする。
   - 推奨：(a)。今の挙動は書かれているが、`＄３００` が黙って `＄０` の値を出すのは、この spec が直す「黙って化ける」と同じ形である。256 個を超える引数を渡す辞書は考えにくく、(c) は `VarScope::Args(u8)` の型と `pasta_lua` に広がる。
   - 例：`さくら：＄２５６` → `Parse error … シーン引数の番号 256 は 0〜255 の範囲を超えています`。今は `＄０` の値を話す。
6. **`PastaFile.span` を消すか、正しく計算するか**（要件 5）
   - 案：(a) 欄を消す、(b) 文字の数で正しく計算し直す。
   - 推奨：(a)。本番で読む所は無く、テスト 2 本だけが読む。ponytail の方針（使われないものは消す）と brief の「無くなる」に合う。
7. **`Span::extract_source` と `SpanError`**（指摘にしなかったもの）
   - 案：(a) 残す、(b) 消して、テストは `&source[span.start_byte..span.end_byte]` で確かめる。
   - 推奨：(a)。テスト 3 ファイルが span の値を確かめる道具で、文字の境目と逆向きの span の守りを持つ。(b) は `ponytail-editor-tooling` の持ち場の `partial_parse_test.rs` に触る。
8. **3.3 の括弧式のバグを拾うか**（brief の Out of Boundary）
   - 結論：拾う作業は無い（2026-10-04 の `14be2195` で直っている）。回帰テストを保つことだけを要件 8.2 にした。閉じてよい。
9. **大きな整数の小数への写し方と、マニュアルの文言**（要件 2.2・7.1）
   - 推奨の文言（`grammar/literals.md` の「数値」の節に足す）：「小数点の無い数が -9223372036854775808〜9223372036854775807 に収まらないときは、いちばん近い小数として扱う（Lua と同じ）。桁が多いと、下の桁は丸められる。小数でも表せないほど大きい数（およそ 1.8×10^308 を超える数）は、構文の誤りになる。」
   - 写し方は、10 進の字句から 64 ビットの浮動小数点数への、いちばん近い値への丸め（Rust の `str::parse::<f64>` と Lua の字句の読み方が同じ結果になる）。
10. **他の spec への申し送り（この spec では直さない。コントローラが相手の brief に書く）**
    - `ponytail-lua-transpiler`：`code_gen/element_gen.rs:40` の `format!("args[{}]", index + 1)` は、`index` が `u8` なので `＄２５５` で桁があふれる（debug はパニック、release は `args[0]` になり、常に値なしになる）。マニュアルは 0〜255 を使えると書いている。`u32::from(*index) + 1` などで直す。読んで確かめた（実行はしていない）。
    - `ponytail-lua-transpiler`：`code_gen/scope_gen.rs:424-430` の `!select` の小数の引数は、`(*f as i64)` で整数にする。この spec で `!select(10000000000000000000)` が `Float(1e19)` になると、`9223372036854775807` に飽和して出る。害は小さいが、要件 2.2 の後に見えるようになる。
    - `ponytail-editor-tooling`：要件 3 の守りは `parse_str` の入口に入るので、部分解析（`partial.rs`）も守られる。部分解析が、この誤りをどの行に出すかは、その spec の診断の直し（1 つの誤りで全体に波線）の中で扱う。

## 設計フェーズの調査（2026-11-02・`/kiro-spec-design -y`）

- **Discovery Scope**: Extension（light discovery）。新しい依存も外部の API も無く、既存の `pasta_dsl` の部品の中で閉じる。
- **使ったスキルと指針**: `kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、テンプレート `.kiro/settings/templates/specs/design.md`・`research.md`。steering は `tech.md`（pest 2.8.6・テストの置き場）と `roadmap.md`（同じウェーブの約束）。ponytail の指針（いちばん小さい完全な直し・既存の形を守る）。
- **測った所**: ワークツリーの HEAD `b8e2e276`。probe はスクラッチの `probe`（`pasta_dsl` だけ）と `probe2`（`pasta_dsl`＋`pasta_lua`、pest を `=2.9.2` に固定）。

### Research Needed の結論

| 項目 | 結論 | 根拠 |
| ---- | ---- | ---- |
| `Pair::line_col()` が pest 2.8.6 にあるか | **ある**。下限を上げる必要は無い（`Cargo.toml`・`Cargo.lock` に触らない） | `C:\rust\cargo\registry\src\…\pest-2.8.6\src\iterators\pair.rs:290` に `line_col`、`line_index.rs` の `LineIndex` は 2.9.2 と書式の 1 行を除いて同じ（`diff`）。`Position::line_col` も 2.8.6 と 2.9.2 で同じ |
| `LineIndex` と `Position::line_col` の数え方が同じか | **同じ**。`"\r\n"` は 1 つの改行、単独の `'\r'` は 1 桁、桁は文字の数 | 両方のソースを読んだ。`LineIndex` は `'\n'` の後ろを行の頭にし、行の頭から `chars().count()`。`Position::line_col` は `'\r'` の次が `'\n'` なら改行、それ以外の `'\r'` は桁を 1 つ進める。位置が `'\r'` と `'\n'` の間でも、どちらも `'\r'` を 1 桁と数える |
| `try_parse_expr` の既定の腕に届く入力 | **無い** | `grammar.pest` を読んだ。呼び出し元（`parse_var_set`・`parse_expr_from_parts`・`parse_key_arg`）が渡す子は、`expr`・`bin`・`term`・`string_literal`・`string_fenced`・`strfence` がすべて無音の規則なので、`paren_expr`・`fn_call_local`・`fn_call_global`・`fn_call_dynamic`・`var_ref_local`・`var_ref_global`・`var_ref_property`・`number_literal`・`string_contents`・`string_blank` と演算子の規則だけ。どれも明示の腕がある（演算子は呼び出し元が先に分ける）。`id`・`property_id`・`word_ref`・`word_ref_dynamic` は呼び出し元が先に扱う |
| `pasta_lua` のコード生成が上限の式を 1 MiB で変換しきれるか | **しきれる**（debug の x64・x86） | probe2（pest 2.9.2）で、上限ちょうどの 5 つの形（関数呼び出し 64 段・括弧 64 段・動的な関数呼び出し 64 段・アクション行の関数呼び出し 64 段・Call の引数の中の 63 段。どれも内側に演算子 256 個）を `parse_str` → `LuaTranspiler::default().transpile` → 片付け。x64 debug・x86 debug とも 1024 KiB と 768 KiB で全部通った。release は測っていない |
| x86 の深さ | **通る** | probe（`parse_str` と片付け）で、上の 5 つの形が x86 debug・release の 1 MiB で通った。x86 debug は 384 KiB でも通り、256 KiB であふれた。x64 debug は 640 KiB で通り、512 KiB であふれた。x64 release は 256 KiB で通り、128 KiB であふれた |
| VSCode 拡張の WASM のスタック | **測っていない（前提）** | `editors/vscode/scripts/build-wasm.ps1` は `wasm-pack build --release` で、スタックの大きさを変えていない（wasm32 の既定の 1 MiB）。native の release の余裕から通る見込みとした（設計の OPEN QUESTION 6） |

### Research Log

#### pest の行の索引

- **Context**: 要件 1 の速さ。`Pair::line_col` が下限の 2.8.6 にあるかと、値が今と同じか。
- **Findings**: `Pairs::new` が `LineIndex::new(&input[..last_input_pos])` を 1 回だけ作り、`into_inner`・`flatten` は同じ `Rc<LineIndex>` を渡す。`Pair::line_col` は `self.pos(self.start)` の位置を二分探索し、行の頭からの文字を数える。終わりの位置の API は無いので、`as_str()` の中を数える。
- **Implications**: `Span::from_pair` で 17 か所を置き換えるだけで済み、下限の変更（ウェーブの約束で `ponytail-editor-tooling` の席）は要らない。

#### pest 2.9.3 から入ったスタックの守り

- **Context**: probe2 を最初に作ったとき、`Cargo.lock` が無く pest 2.9.5 が選ばれ、x64 debug・512 KiB で、あふれる代わりに `stack limit reached` の解析の誤りが返った。
- **Sources**: `pest-2.9.5/src/parser_state.rs:95-125`（`set_call_limit` の説明：既定の native の対象では、スタックの残りを自動で見て「stack limit reached」で失敗させる。`no_std` と既定でない対象は `stack-guard` の機能が要る）。2.9.3・2.9.4 にも同じ文言がある。2.9.2 には無い。
- **Implications**: 今の `Cargo.lock`（2.9.2）には効かない。この spec の括弧の走査は、pest の守りに頼らない（版に依らず、AST の片付けとコード生成も守る）。`pest-2-9-4-upgrade` への申し送りに書いた（design.md の Open Questions）。

#### 大きな小数の Lua への書き方

- **Context**: 要件 2.2 で `Float(1e19)` などがコード生成に届く。
- **Findings**: `code_gen/expr_gen.rs:147` は `write!(buf, "{}", f)`。Rust の `f64` の `Display` は指数を使わず、`1e19` を `10000000000000000000` と書く。Lua はそれを同じ 64 ビットの浮動小数点数として読む。有限でない値は要件 2.5 で解析の誤りになるので、`inf` は届かない。
- **Implications**: `code_gen/`（`ponytail-lua-transpiler` の持ち場）を直さずに済む。

#### 誤りの位置に要るファイル名

- **Context**: 新しい誤りは `Parse error in {file} at …` で始める（4.3）。今 `filename` を受け取る解析の関数は `parse_global_scene_scope`・`parse_global_scene_start` だけ。
- **Findings**: 範囲の検査が要る関数（属性・アクターの番号・キューの引数・式の項・シーン引数の番号・演算子の数）の呼び出し元をたどると、ほぼすべての解析の関数が通り道になる（23 個）。すべて `pub(crate)` か非公開で、`pasta_dsl` の外から呼ぶ所は無い（`partial.rs` は `parse_str` だけを呼ぶ）。
- **Implications**: 下の「Design Decisions」の `filename` の受け渡し。

### Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A（採用） | 既存の部品の中で直す。`Span::from_pair`・`number_value`・入口の走査・式の検査 | 変更が `pasta_dsl` の中で閉じ、呼び出し元を直さない | `try_parse_expr` などの戻り値の形と、`filename` の引数が変わる | ギャップ分析の推奨のまま |
| B | `parse_str` で行頭の索引を作り、全関数に渡す | pest の版に依らない | 全関数の引数が増え、pest の索引と二重 | 下限の問題が無いと分かったので利点が消えた |
| C | 始まりは `Pair::line_col`、終わりは自前の索引 | 終わりの計算が O(log n) | 深さ 5〜6 倍の手間に実害が無い | 採らない |

### Design Decisions

#### Decision: `filename` を引数で渡す

- **Context**: 深い所の誤りに、ファイル名と前置きが要る。
- **Alternatives Considered**:
  1. 23 個の関数に `filename: &str` を足す（`parse_global_scene_scope` と同じ形）。
  2. 深い所は `file` が空の `SyntaxError` を作り、`parse_str` で埋めて前置きを付ける。
  3. ファイル名を持つ文脈の構造体を渡す。
- **Selected Approach**: 1。
- **Rationale**: 今のコードの形のまま、型で漏れが見つかる。2 は引数が増えないが、「空の `file` は未完成」という隠れた決まりになり、`parse_str(src, "")` で `parse_global_scene_start` の誤り（前置きが無い）にまで前置きが付いて文言が変わる。3 は中身が 1 つの欄だけで、1 と同じ行数の変更になる。
- **Trade-offs**: 引数の追加の行数が多い。挙動を変えないコミット（Migration Strategy の手順 2）に切り分ける。

#### Decision: 演算子の数は「式を書ける位置に書いた式の全体」で数える

- **Context**: 要件 3.2「1 つの式の二項演算子が 256 個を超える」。3.6 は 1 MiB であふれないこと。
- **Alternatives Considered**: (a) 式の全体（括弧と引数の中を含む）、(b) 文法の `expr` の鎖ごと、(c) 1 行ごと。
- **Selected Approach**: (a)。鎖を組む 3 つの関数の頭で、`Pairs::flatten` で子孫の演算子を数える。外側が先に止まる。
- **Rationale**: (b) は括弧 64 段 × 256 個で木の深さが約 16,000 になり、片付けが 1 MiB の release で 10,000 個の鎖であふれた測定に届く。(c) は台詞の行に関数呼び出しを 2 つ書くと合わさり、3.3 を破る。
- **Follow-up**: 設計ディスカッションで確かめる（design.md の OPEN QUESTION 1）。

#### Decision: 数の値は 2 つの腕の内部の型で返す

- **Context**: brief の `number_value(s) -> Expr` は、属性値（`AttrValue`）とキューの引数（`CueArgToken`）にそのまま使えない。
- **Selected Approach**: `enum Number { Integer(i64), Float(f64) }` を返し、3 つの使い手が 2 腕の `match` で自分の型に写す。誤りの位置を出すため、文字列でなく `Pair` を受ける。
- **Rationale**: `Expr` を返して属性値で写し直すと、届かない腕が要る。

#### Decision: span の値は「控え」でなく「正解の計算」で守る

- **Context**: 要件 1.3「すべての span を直す前と同じ値に」。
- **Selected Approach**: テストの中で、AST の `{:?}` から span を拾い、バイトの位置から pest の `Position::line_col`（直す前の計算そのもの）で出した行と桁と比べる。
- **Rationale**: バイトの位置は pest がそのまま返すので変わらない。直す前の値は「バイトの位置から `Position::line_col` で出した値」と同じなので、控えを置かずに常に同じことを確かめられ、後の spec が AST の型を変えても崩れない。
- **Trade-offs**: `{:?}` の書式に頼る。`Span` の `Debug` は derive なので、欄の名前を変えない限り崩れない。

### Synthesis

- **Generalization**: 範囲の外の値・深さ・長さの誤りと pest の誤りを、1 つの補助 `syntax_error` で作る（4.3 の形を 1 か所で守る）。4 か所の数の読み方を `number_value` 1 つにする。番号（`digit_id`）の 2 か所は、型（`u32`・`u8`）と文が違うだけなので、補助を作らず各所で 1 行の `map_err` にする。
- **Build vs. Adopt**:
  - 行と桁：pest の `LineIndex`（`Pair::line_col`）を採る。自前の索引（Option B）は作らない。
  - 深さの守り：pest 2.x の `set_call_limit` は累計を数えるので採らない。pest 2.9.3 以降の stack guard は今の版（2.9.2）に無く、AST の片付けとコード生成も守らないので頼らない。入口の走査を作る（10 行ほど）。
  - 数の値：`str::parse::<i64>`・`str::parse::<f64>`（いちばん近い値への丸め）・`f64::is_finite` を採る。
- **Simplification**:
  - 新しいモジュール・層・公開 API は作らない。新しい部品はすべて既存のファイルの `pub(crate)` の関数で、テストのほかに新しいファイルは無い。
  - 全辞書の AST の控えはリポジトリに置かない（`#[ignore]` の書き出しで、直す前と後を 1 回比べる）。
  - `try_parse_expr` の演算子の腕と既定の腕を 1 つにまとめる。`parse_attr_number` を消す。`build_ast` の `Rule::EOI` の腕を消す。

#### 設計の関門で直したこと

- 1 回目の下書きは「深い括弧の行より前に文法の誤りがある辞書は、直す前は落ちていた」と書いていた。pest は前の行の誤りで繰り返しを抜け、深い行まで進まないので、この辞書は直す前も落ちずに前の行の誤りを返す。走査を pest の前に置くと、この辞書だけ返す誤りが変わる。1.4 と 3.1 の両方が当てはまる辞書として、3.1 を採る前提を書き、design.md の OPEN QUESTION 7 にした。
- 速さのテストを、時間を測るので、別のテストのファイル（`parse_speed_test.rs`）に分けた（同じプロセスの並列のテストとぶつからないように）。
- span の正解のテストが歩く場所を、リポジトリのルートでなく `crates/` と `.kiro/` にした（本体の作業ツリーでは、ルートの下の `.claude/worktrees/` に別のワークツリーの複製がある）。
