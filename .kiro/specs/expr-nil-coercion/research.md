# ギャップ分析: expr-nil-coercion

> 作成: 2026-10-08（`/kiro-start` の非対話フェーズ。要件ディスカッションの入力）。コードは読むだけで、ビルド・テストは実行していない。LuaJIT の数値の挙動は既存テストが固定している事実から引いた。

## 1. 現状の調査

### 1.1 算術の経路（DSL → 生成コード → ランタイム）

| 段 | 場所 | 現在の挙動 |
| -- | ---- | ---------- |
| トランスパイラー | `crates/pasta_lua/src/code_gen/expr_gen.rs` `binary_to_string`（163〜194 行）・`binary_operand`（198〜205 行）・`operand_desc`（207〜235 行） | 算術 1 演算を `act:arith(op, 左, 右, 左の説明, 右の説明)` に組む。説明は被演算子の出どころ: 変数参照 → `resolve_var_path`（`var.x`／`save.x`／`args[n]`）、ローカル関数 → `@名前()`、グローバル関数 → `@*名前()`、動的関数 → `@$var.x()`、括弧 → 中身の説明、リテラル・入れ子の演算 → `nil` |
| トランスパイラー | `crates/pasta_lua/src/code_gen/element_gen.rs` `resolve_var_path`（32〜42 行） | `VarScope::Property` は `TranspileError::PropertyInExpression`。**プロパティ参照 `＄％…` は式の中に書けず、`act:arith` に届かない**（`＄var＝＄％prop` は 88〜103 行の GET 代入で `act:get_property` に直結し、算術を経由しない。テスト `crates/pasta_lua/tests/property_scope_codegen_test.rs:251`） |
| 生成コード | `crates/pasta_lua/src/code_gen/scope_gen.rs:264` | シーン関数の先頭で `local args = { ... }`。渡されなかった引数は `nil`。`＄０` は `args[1]` |
| ランタイム | `crates/pasta_lua/pasta_scripts/pasta/act.lua` `arith_operand`（532〜542 行） | `number` はそのまま、`string` は `tonumber`、それ以外は `nil`。数値にできないとき、値も説明も `nil`（内側の失敗の伝播）なら黙り、それ以外は `log.warn("act:arith - operand is not a number: op=…, operand=…, value=…")` して `nil` |
| ランタイム | 同 `ACT_IMPL.arith`（554〜565 行） | 左右を `arith_operand` にかけ、どちらかが `nil` なら `nil`、両方数値なら `ARITH_OPS[op]`（Lua のネイティブ演算） |
| ランタイム | 同 `concat_operand`／`ACT_IMPL.concat`（573〜600 行） | 連結側。`nil` は説明があれば警告＋`nil`。**本仕様では触らない** |
| 変数の実体 | `act.lua` 176〜178 行・`pasta/save.lua` | `save` は `@pasta_persistence.load()` が返す素のテーブル、`var` は `{}`。未代入は単に `nil`（メタテーブル無し） |

**未代入の変数（値 `nil`・説明 `var.x` 等）と、値を返さない関数呼び出し（値 `nil`・説明 `@f()` 等）は、現行の `arith_operand` では区別されない。** 区別できる情報は説明文字列の形だけが既に運んでいる。

### 1.2 現行挙動を固定しているテスト

| テスト | 固定している期待 | 本仕様での扱い |
| ------ | ---------------- | -------------- |
| `crates/pasta_lua/tests/lua_specs/act_runtime_safety_test.lua` 「act:arith - 数値にできない被演算子」（326〜335 行） | `act:arith("+", nil, 1, "var.x")` が `nil`＋警告 1 行 | 期待を 1・警告 0 に更新 |
| 同 「入れ子で内側が失敗すると外側は値なし」（377〜386 行） | `act:arith("*", (act:arith("+", nil, 1, "var.x")), 2)` が `nil`、`act:arith("+", 1, act:arith("*", 2, nil, nil, "var.y"))` が `nil` | 内側が変数なので 2・1 になる。伝播の確認は非数値文字列などに差し替え |
| 同 「数値にできる被演算子」（271〜298 行） | 0 除算を含め、ネイティブ演算と一致 | 不変（`／`・`％` の 0 除算の事実の出どころ） |
| `crates/pasta_lua/tests/lua_specs/act_concat_test.lua` 「act:concat - 失敗の伝播」（143〜170 行） | `act:concat("合計", (act:arith("+", nil, 1, "var.x")))` が `nil`＋`act:arith` の警告 | 算術側が 1 になるので `"合計1"`。伝播の確認は `"abc"` などに差し替え |
| `crates/pasta_lua/tests/transpiler/runtime_safety_test.rs` `RUNTIME_SAFETY_SOURCE`（64〜82 行）・期待（131〜162 行） | `＄x＝＄未代入＋１` → `x=nil`、警告 `operand='var.未代入', value=nil`、`さくら：＄x` が空（トークンなし） | `x=1`、警告 1 行減、`さくら：＄x` が `talk|さくら|true|1` を積む |
| 同 `test_concat_failures_warn_and_yield_nil`（270〜303 行） | `＄r2＝「a」＆（＄未代入＋1）` → `r2=nil`＋`act:arith` の警告 | `r2="a1"`、警告 1 行減 |
| `crates/pasta_lua/tests/transpiler/snapshots/transpiler__runtime_safety_test__runtime_safety.snap`（38 行） | 生成形 `var.x = act:arith("+", var.未代入, 1, "var.未代入")` | 生成コードを変えない案なら不変 |
| その他 `act:arith` を含むスナップショット 3 件（`dynamic_call_binary_expr`・`dynamic_word_ref`・`final_regression … fixture_sample`） | 生成形 | 同上。生成形を変える案では更新が要る |
| `crates/pasta_lua/tests/runtime/syntax_test.rs:413`（アクション行の `＄未代入`） | 空文字展開＋警告 | 不変（Requirement 4.3） |

テストの警告の捕捉は `log.warn` だけ（`with_captured_act` と `run_main_scene` は `debug` を no-op にしている）。0 とみなしたときに `debug` を出す案を採っても、警告数の期待には影響しない。

### 1.3 マニュアルとスキル

| 文書 | 現行の記述 | 本仕様での変更 |
| ---- | ---------- | -------------- |
| `book/src/grammar/variables.md` 「算術の評価」（144〜168 行） | 「数値にできない被演算子（値を代入していない変数・…）があると、その演算の結果は値なし」。作例 `＄x＝＄未代入＋1` → 「結果はとです」 | 規則の書き直し・作例の差し替え（`＄x` は 1）。数値にできない例は `「abc」` 側に寄せる |
| 同 「連結の評価」（170〜228 行） | 作例 `＄y＝「合計」＆（＄未代入＋1）` → 値なし。「警告は内側の 1 行だけ」の例に `（「a」＆＄未代入）＋1` と `「合計」＆（＄未代入＋1）` | `＄y` は `合計1`。伝播の例は算術の内側が変数でないものに差し替え |
| 同 110 行の補足（アクション行の未代入） | `act:talk - undefined variable` | 不変 |
| `book/src/lua/script-api.md` `arith(op, lhs, rhs, lhs_desc, rhs_desc)`（424〜441 行） | 「`nil`・数字でない文字列・真偽値・表などがあれば `nil` を返し警告」 | 未代入の変数（説明が変数の場所）の 0 扱いを追記 |
| `book/src/internals/internal-modules.md` 「生成コード用のメソッド」（196〜204 行） | `arith_operand` の数値化の説明 | 区別の仕組みを追記 |
| `book/src/internals/transpiler.md`（223〜236 行） | 生成形と説明の形 | 生成コードを変えない案なら不変 |
| `book/src/grammar/call-jump.md:116` | `＞＄未代入＆「_挨拶」` の連結の失敗 | 不変（連結側） |
| `.claude/skills/pasta-ghost-authoring/SKILL.md:178` | 「数値にできない被演算子（未代入の変数・数字でない文字列など）があると、その演算は値なしになり警告ログが出る」（手書き） | 1 行を書き直す |
| `.claude/skills/pasta-ghost-authoring/references/variables.md`・`call-jump.md` | `book/src/grammar/*.md` からの生成物（`book/tools/gen-skill-refs.mjs` の `GENERATION_MAP`） | `node book/tools/gen-skill-refs.mjs` で再生成。`--check` と `link-check.mjs` を通す |

### 1.4 規約・パターン

- ランタイムの Lua テストは `crates/pasta_lua/tests/lua_specs/*_test.lua`（lua_test の BDD）。`act.lua` の局所関数は `with_captured_act` 経由で `pasta.act` を再 require して観測する。
- トランスパイル→実行のテストは `crates/pasta_lua/tests/transpiler/runtime_safety_test.rs` の `run_main_scene` パターン（`@pasta_log` を差し替えて警告を `RS_WARNS` に集める）。
- `ACT_IMPL.arith` の引数・戻り値は `actor-proxy-act-delegation`・`call-execution-correctness` が前提にしているため変えない（brief の制約）。
- 「現行実装を正とし、マニュアルを同じ PR で直す」（roadmap の前提）。

## 2. 要件の実現性

### 2.1 要件と資産の対応

| 要件 | 既存資産 | ギャップ |
| ---- | -------- | -------- |
| R1.1〜1.3（変数は 0） | `arith_operand`（数値化の分岐）、説明文字列（`var.`／`save.`／`args[`） | **Missing**: 「値 `nil` かつ説明が変数の場所」を 0 にする分岐 |
| R1.2（シーン引数） | `args[n]` の説明は既に生成されている | Missing なし（R1.1 の分岐に含めるかどうかの判断だけ） |
| R1.4（除数） | `ARITH_OPS` の `/`・`%`。LuaJIT の `1/0`＝`inf`、`-1/0`＝`-inf`、`0/0`＝非数、`5%0`＝非数（既存テストが固定） | **Constraint**: 規則を 1 つに保つ案ではコード変更なし。除数だけ値なしにする案では `ACT_IMPL.arith` で右の被演算子と演算子を見る分岐が要る |
| R1.5（全位置） | 生成形はどの位置でも同じ `act:arith` | Missing なし |
| R1.6（入れ子） | 内側が数値を返せば外側はそのまま計算 | Missing なし |
| R1.7（値なし代入後の変数） | 「値なし代入＝未代入」規則 | Missing なし（帰結）。**Unknown**: 作者にとって望ましいかは要件ディスカッションで確認 |
| R1.8（再起動後の継続） | `save` は `@pasta_persistence` が JSON に保存 | Missing なし。数値 1 の保存・復元は既存機能 |
| R2（0 にしない失敗） | 現行の `arith_operand` の警告分岐 | Missing なし（分岐の順序を守るだけ） |
| R3（警告なし） | — | Missing なし（0 を返す分岐で `log.warn` を呼ばない）。`debug` 案なら `log.debug` を 1 行 |
| R4（連結・アクション行の不変） | `concat_operand`・`ACT_IMPL.talk` | Missing なし（触らない） |
| R4.5（手書き Lua からの呼び方） | `act:arith` の公開シグネチャ | **Unknown**: 区別の仕組み（論点 4）で決まる |
| R5（マニュアル・スキル） | 3 ページ＋SKILL.md 1 行＋`gen-skill-refs.mjs` | Missing: 本文の書き直し |
| R6（テスト） | 1.2 の表のテスト | Missing: 新しいケース（`save.`・`args[`・除数・値なし代入後）。既存期待の更新 |

### 2.2 複雑さの信号

- アルゴリズム上は 1 分岐の追加。ワークフローや外部連携は無い。
- 影響の広がりはテストとマニュアルの「現行挙動を書いている箇所」の棚卸にある（1.2・1.3 の表で網羅した）。

## 3. 実装アプローチの選択肢

### 選択肢 A: `arith_operand` で説明文字列の形から見分ける（brief の既定）

- **変更**: `act.lua` の `arith_operand` に「`v == nil` かつ `desc` が変数の場所（`@` で始まらない）なら `0` を返す」分岐を足す。生成コード・トランスパイラー・`ACT_IMPL.arith` のシグネチャは不変。
- **判定の具体案**: `desc:sub(1, 1) ~= "@"`（関数の説明は `@名前()`・`@*名前()`・`@$var.x()` がすべて `@` 始まり）。または `desc:match("^var%.")`／`^save%.`／`^args%[` の肯定一致（意図が明示的で、将来の説明の追加に対して安全側）。
- **利点**: 変更が数行。スナップショット 4 件・`transpiler.md` が不変。`actor-proxy-act-delegation`・`call-execution-correctness` の前提（`arith` の引数・戻り値）を守る。
- **欠点**: 「説明は警告用の文字列」という既存の位置づけに、判定の意味が乗る（`script-api.md` の `act:arith` に規則として書く必要がある）。手書き Lua で `act:arith("+", nil, 1, "var.x")` と呼ぶと 0 扱いになる（R4.5 の既定どおり）。

### 選択肢 B: トランスパイラーが被演算子の種類を渡す

- **変更**: `expr_gen.rs` の `operand_desc`／`binary_node` が、変数参照に別の印（例: 説明の接頭辞を `$var.x` にする、または第 6・7 引数に種類を渡す）を付け、`arith_operand` はその印で判定する。
- **利点**: 判定が説明の文面に依存しない。
- **欠点**: 生成形が変わるため、`act:arith` を含むスナップショット 4 件・`transpiler.md`・`script-api.md`・`internal-modules.md` の生成形の記述・既存の警告文（`operand='var.x'` の形を変えるなら `runtime_safety_test.rs` の期待警告と `call-jump.md:116` も）が連鎖して変わる。`ACT_IMPL.arith` のシグネチャ変更は brief の制約に抵触する。規模が brief の見積もり（5〜8 タスク）を超えやすい。

### 選択肢 C: A を採り、判定を局所関数に切り出す（ハイブリッド）

- **変更**: A の判定を `is_variable_desc(desc)` のような局所関数にし、`arith_operand` はそれを呼ぶ。`concat_operand` は呼ばない（連結は対象外）。
- **利点**: 「変数の説明とは何か」が 1 か所に書かれ、`failure-output-unification` が説明の形を整理するときの rebase 点が明確。
- **欠点**: A との差は小さく、過剰な抽象化になりうる（1 か所でしか使わない）。

**推奨**: A（必要なら C）。brief の制約（`arith` の引数・戻り値不変、`arith_operand` の数行）と一致し、下流 `hello-pasta-tutorial-stages` を早く通せる。

## 4. 工数・リスク

- **工数: S**（1〜3 日）。ランタイム 1 分岐＋Lua テスト＋Rust テストの期待更新＋マニュアル 3 ページ＋スキル再生成。
- **リスク: Low**。既存パターンの延長で、外部依存なし。残るリスクは「現行挙動を書いている箇所の取りこぼし」（1.2・1.3 の表が棚卸）と、除数・警告の論点が要件ディスカッションで別案に倒れたときの小さな手戻り。

## 5. 設計フェーズへの推奨と持ち越し

### 5.1 推奨

- 選択肢 A。`arith_operand` の分岐順は「数値にできる → 変数の未代入なら 0 → それ以外は従来の警告＋`nil`」。
- 既存テストの更新は「期待値の書き換え」ではなく、「内側の失敗の伝播」を確かめていたケースの被演算子を `"abc"` などに差し替えて、伝播の規則の検証を失わないようにする。
- マニュアルの作例は、「未代入は 0」の作例と「数値にできない」の作例を分け、空文字列の境界と「値なしを代入した変数が次の算術で 0 になる」帰結を 1 行ずつ書く。

### 5.2 要件ディスカッションへの持ち越し（OPEN QUESTIONS）

brief の 6 論点に、調査で見つかった 2 点を足した。各項の「既定」は `requirements.md` に【仮定】として書いた内容。

要件ディスカッション（2026-10-08）での扱い: 1（演算子の範囲の部分）・3・5・6・8 は自明として確定（要件に反映済み）。4 は設計判断（5.4）へ。残る議題は 1（除数）・2（シーン引数）・7（値なし代入後の変数）の 3 つ。

- **方針の組み替え（2026-10-08、議題 #3）**: 開発者の判断で、規則を「式の中の nil は、算術の文脈なら 0、連結の文脈なら `""`。ログなし」に組み替えた。論点 4（区別の仕組み）・6（連結は範囲外）・7（値なし代入後の変数）・8（動的関数の変数）は、この規則で解消した（出どころを問わないため）。新しい議題は、nil 以外の変換できない値の扱い（num／str 案の帰結）・アクション行の `＄未代入` の警告・Call のターゲットが連結のときの前方一致の 3 つ。
- **議題 #4（変換できない値）**: nil 以外の変換できない値（`「abc」`・空文字列・真偽値・表など）も、警告を 1 行出したうえで算術なら 0、連結なら `""` とみなす。式の演算は必ず値を返す。これで num／str に通す形（5.3）が素直に作れる（num／str は必ず数値・文字列を返す）。
- **議題 #5（Call のターゲット）**: Call が失敗するのは受け取った値が nil・空文字列・文字列と数値以外のとき、という規則は変えない。連結・算術の結果は nil にならないので、`＞＄時間帯＆「の挨拶」` は `＄時間帯` が未代入なら `の挨拶` を探す。ターゲットが変数 1 つ・関数呼び出し 1 つで nil のときは、今どおり `【Call失敗：… が nil】`。
- **議題 #6（台詞の中の未代入の変数）**: 台詞の中の変数は文字列の文脈で、nil は既にブランク（空文字）に展開される。表示は本仕様の規則と一致しているので変えず、警告 `act:talk - undefined variable` も残す。

- 議題 #1（論点 1・除数）: 既定どおり確定。除数も 0 とみなす（`１／＄未代入` は `inf`、`１％＄未代入` は非数）。非数・負の 0 の表記の実機確認は 5.4 の Research Needed として設計へ。
- 議題 #2（論点 2・シーン引数）: 既定どおり確定。`var.`・`save.`・`args[` の 3 つすべてを 0 とみなす。引数の渡し忘れがログに出なくなることは受け入れる。リクエスト変数 `＄ｒ０`…（`＞transfer_req_to_var` が入れる）はローカル変数なので、シーン引数ではなく `var.` の側で対象になる。

1. **除数が未代入のとき**（brief 論点 1、R1.4）
   - 事実: LuaJIT のネイティブ演算は `1/0`＝`inf`、`-1/0`＝`-inf`、`0/0`＝非数、`5%0`＝非数（`act_runtime_safety_test.lua` 271〜298 行が固定）。マニュアルの連結の節は「0 での除算は `inf` になる」と既に書いている（`variables.md:213`）。非数を `tostring` したときの表記は環境依存の可能性がある（Windows の CRT は `-nan(ind)` を出すことがある。**Research Needed**: 実機で `tostring(0/0)` を確認）。負の 0（`＄未代入＊－１` → `-0`）の表記も `-0` になりうる。
   - 既定: 規則を 1 つに保ち、除数も 0 とみなす（`１／＄未代入` は `inf`）。
   - 別案: 除数（`／`・`％` の右）だけは従来どおり警告＋値なし。`ACT_IMPL.arith` で右の被演算子と演算子を見る分岐が要る。
2. **0 とみなす変数の範囲**（brief 論点 2、R1.2）
   - 事実: プロパティ参照 `＄％…` は式の中に書けずトランスパイルエラー（`PropertyInExpression`）。`＄var＝＄％prop` は算術を経由しない GET 代入。したがって範囲の候補は `var.`／`save.`／`args[`（シーン引数）の 3 つだけ。シーン引数は `local args = { ... }` で、渡されなければ `nil`、説明は `args[n]`。
   - 既定: 3 スコープすべて（シーン引数を含む）。
   - 別案: `var.`・`save.` だけ（引数不足は書き間違いに近いとみる）。
3. **0 とみなしたときの警告**（brief 論点 3、R3.1）
   - 事実: テストの警告捕捉は `warn` だけで `debug` は no-op。`failure-output-unification` が `act.lua` の `log.warn` の出口を一本化する予定。
   - 既定: 黙る（ログを出さない）。
   - 別案: `log.debug` を 1 行。入門ガイドの読者は `debug` を見ないので戸惑わないが、「新しい出口を増やさない」との整合を確認する。
4. **区別の仕組み**（brief 論点 4、R4.5・R6.4）
   - 事実: 説明文字列は変数が `var.x`／`save.x`／`args[n]`、関数が `@f()`／`@*g()`／`@$var.f()`、括弧は中身の説明、リテラル・入れ子の演算は `nil`。関数の説明はすべて `@` 始まり。生成形を変えるとスナップショット 4 件と `transpiler.md` が連鎖して変わる。
   - 既定: 説明文字列で見分ける（選択肢 A）。手書き Lua から `act:arith("+", nil, 1, "var.x")` と呼ぶと 1 になる。
   - 別案: トランスパイラーが種類を別引数で渡す（選択肢 B）。brief の「`arith` の引数・戻り値を変えない」制約と抵触。
5. **空文字列の変数**（brief 論点 5、R2.3）
   - 事実: `tonumber("")` と `tonumber("   ")` は `nil`。`＄x＝「」` のあとの `＄x＋1` は従来どおり警告＋値なし。空文字列は「未代入」ではない。
   - 既定: 現行どおり（0 にしない）。マニュアルに境界として書く。
6. **連結 `＆` との対称性**（brief 論点 6、R4.1・R4.2）
   - 事実: `concat_operand` は別関数で、本仕様は触らない。ただし `「合計」＆（＄未代入＋1）` は内側の算術が 1 になるため `合計1` に変わり、`「合計」＆＄未代入` は値なしのまま。この非対称をマニュアルに書く。
   - 既定: `＆` は変えない（brief の Out of Boundary）。
7. **値なしを代入した変数が次の算術で 0 になる帰結**（新規、R1.7）
   - 事実: 「値なしを代入した変数は未代入になる」規則（変えない）により、`＄y＝「abc」＊２` のあとの `＄y＋１` は 1、`＄z＝＠＊未定義関数（）` のあとの `＄z＋１` も 1 になる。最初の失敗の警告は出るが、2 段目は黙る。`dsl-codegen-runtime-safety` の旧 OQ-3 は「0 は失敗をもっともらしい値で隠す」ことを理由に当時の算術の失敗の既定値を 0 にしなかった。
   - 既定: 帰結として受け入れる（規則を増やさない）。
   - 別案: 「値なしを代入した」変数を「一度も代入していない」変数と区別する——規則の変更になり、brief の Out of Boundary と抵触。
8. **動的関数呼び出しの変数が未代入のとき**（新規、R2.2）
   - 事実: `＠＄x（）` で `＄x` が未代入なら、`act:expr_fn_var(nil, "var.x")` の警告（`undefined variable`）と、`act:arith` の警告（`operand='@$var.x()', value=nil`）の 2 行が出る。説明が `@` 始まりなので、選択肢 A では自然に「関数側」になる。
   - 既定: 現行どおり（brief の「動的関数呼び出しの変数は関数側なので対象外」）。確認のみ。

### 5.3 設計判断（`/kiro-design` で決める）

> 2026-10-08 の方針の組み替え（`requirements.md` の「方針の組み替え」）で、選択肢 A〜C（変数と関数を説明文字列で見分ける）は不要になった。nil は出どころを問わず、算術なら 0、連結なら `""` になる。

- **被演算子を num／str に通す形（開発者の案）**: 算術の被演算子をすべて `num(x)`、連結の被演算子をすべて `str(x)` に通すようにトランスパイルし、演算そのものは Lua の `+`・`..` などで行う形。`act:arith`・`act:concat` の中で同じ変換をする現行の形を残す案と比べて決める。
  - num／str に通す形では、演算子に渡る値が必ず数値・文字列でなければならない（nil や表が残ると Lua の実行時エラーになる）。議題 #4 で「変換できない値も警告を出して 0・`""`」と決まったので、num／str は必ず数値・文字列を返す関数として作れる。
  - 生成形が変わると、`act:arith` を含むスナップショット 4 件・`internals/transpiler.md`・`lua/script-api.md` の記述が連鎖して変わる。`act:arith` の引数・戻り値を前提にしている spec（`actor-proxy-act-delegation`・`call-execution-correctness`）への影響も確かめる。
  - 警告に被演算子の場所（`operand='var.x'`）を含め続けるなら、num／str にも説明を渡す。
- **既存テストの差し替え方**: 伝播の検証は、規則の変更に合わせて期待を書き直す（「外側も値なし」から「外側で 0／`""`」へ）。
- **マニュアルの作例の構成**: 「nil は 0／`""`」の作例と「nil 以外の変換できない値」の作例を分ける。

### 5.4 Research Needed（設計へ）

- Windows（MSVC CRT）での `tostring(0/0)`・`tostring(-0)` の表記（論点 1 の判断材料。マニュアルに書くなら実機で確認）。
- `luacheck` の複雑度しきい値に `arith_operand` の分岐追加が収まるか（手元で `luacheck` を走らせて確認）。
