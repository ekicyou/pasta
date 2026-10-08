# Brief: arith-unassigned-var-zero

> **ステータス**: 未着手（2026-10-08、`hello-pasta-tutorial-stages` の設計ディスカッション #1 の決定を受けて起票）。決定の正本は同 spec の `design.md` の Q1 と `requirements.md` の Requirement 3.6（コミット `0e1f53a0`）。本 brief は、その決定を 1 つの spec に切り出したもの。着手するときは `/kiro-start arith-unassigned-var-zero` で開始する。

## Problem

入門ガイドの 10 段目「覚えていてほしい（変数の保存）」で、読者に次の 1 行だけで回数を数えさせたい。

```pasta
＊会話
    ＄＊回数＝＄＊回数＋１
    女の子：この話をするのは＄＊回数　回目ですね
```

現行の pasta では、未代入の変数を算術に使うと**その演算が値なし**になる（`act:arith - operand is not a number` の警告）。値なしを代入した変数は未代入のままなので、`＄＊回数` は初回に 1 にならず、**永久に数え始めない**。DSL には条件分岐が無いため、「未代入なら 0」という初期化を DSL だけでは書けず、回避には Lua 関数（`＄＊回数＝＠回数を進める（）`）が要る。

「この程度の表現で Lua を出させるなら、それは DSL の問題」として扱う（2026-10-08 設計ディスカッション #1 の決定。[no-lua-for-trivial-dsl-gaps] の方針）。入門ガイドの読者に限らず、回数・得点・好感度のような「増やしていく値」を書く作者はみな同じ場所でつまずく。

## Current State

- 算術 `＋ － ＊ ／ ％` の 1 演算は `act:arith(op, 左, 右, 左の説明, 右の説明)` になる（`crates/pasta_lua/src/code_gen/expr_gen.rs` の `binary_operand`・`operand_desc`）。説明は被演算子の出どころで、変数なら `var.x`・`save.x`・`args[1]`、関数呼び出しなら `@f()`・`@*g()`・`@$var.f()`、リテラルや入れ子の演算なら `nil`。
- `crates/pasta_lua/pasta_scripts/pasta/act.lua` の局所関数 `arith_operand`（532〜542 行）が被演算子を数値にする。`number` はそのまま、`string` は `tonumber`、それ以外は数値にできない。数値にできないときは警告して `nil` を返し、値も説明も `nil` のとき（内側の演算が既に失敗した伝播）だけ黙る。**未代入の変数（値 `nil`・説明 `var.x` など）と、値を返さない関数呼び出し（値 `nil`・説明 `@f()`）は、ここでは区別されず、どちらも警告＋値なし**になる。
- マニュアル `book/src/grammar/variables.md`「算術の評価」が「値を代入していない変数は数値にできない被演算子であり、演算の結果は値なし」と書いている（作例 `＄x＝＄未代入＋1` → 「結果はとです」）。「連結の評価」の作例 `＄y＝「合計」＆（＄未代入＋1）` も同じ前提に立つ。`book/src/lua/script-api.md` の `act:arith` の節、`book/src/internals/internal-modules.md`「生成コード用のメソッド」も同じ挙動を書いている。
- この挙動を固定しているテスト: `crates/pasta_lua/tests/lua_specs/act_runtime_safety_test.lua`（`act:arith - 数値にできない被演算子`。`act:arith("+", nil, 1, "var.x")` が `nil` と警告 1 行）、`crates/pasta_lua/tests/transpiler/runtime_safety_test.rs`（157・298 行の期待警告 `operand='var.未代入', value=nil`）とそのスナップショット。
- 挙動の出どころは完了 spec `dsl-codegen-runtime-safety`（R3.5「数値にできない被演算子は警告＋値なし」）。当時は「500 にしない」ことが目的で、未代入の変数を 0 とみなすかは論点にしていない。`string-concat-operator` は「算術の結果と警告は本仕様で変えない」として引き継いだ。

## Desired Outcome

- 算術の被演算子が**未代入の変数**（ローカル `＄x`・グローバル `＄＊x`・シーン引数 `＄０`… のうち、要件で決めた範囲）なら、その被演算子を 0 とみなして演算が成立する。`＄＊回数＝＄＊回数＋１` は初回に 1 になり、以後 1 ずつ増え、再起動しても保存された値から続く。
- 値を返さない関数呼び出し（`＠f（）`・`＠＊g（）`・`＠＄x（）`）、数字でない文字列、真偽値・テーブルなどは従来どおり警告＋値なし。「書き間違いを 500 にしない」という `dsl-codegen-runtime-safety` の方針は保つ。
- マニュアル（`variables.md`・`script-api.md`・`internal-modules.md`）とスキル `references/` が新しい挙動を書き、入門ガイドの 10 段目の作例がそのまま動くことを `hello-pasta-tutorial-stages` が確認できる。

## Approach

決定済み（設計ディスカッション #1）: **算術の被演算子が未代入の変数なら 0 とみなす。関数呼び出しが値を返さないときは従来どおり値なし＋警告。** 両者はトランスパイラーが渡す被演算子の説明（`var.x`／`save.x`／`args[n]` と `@f()`）で区別できる。

要件フェーズで決める論点（brief では決めない）:

1. **0 とみなす変数の範囲** — `var.`・`save.` に加えて、シーン引数 `args[n]`（`＄０`…）も含めるか。プロパティ参照 `＄％…` は被演算子としてどう扱われているかを確かめてから決める。動的関数呼び出しの変数（`＠＄x（）`）は「関数」側なので対象外。
2. **警告の有無** — 0 とみなした被演算子で警告を出すか、黙るか。入門ガイドの初回起動でログに警告が出ると読者が戸惑うので、黙る案を既定とする。出すなら `warn` ではなく `debug` 程度。
3. **区別の仕組み** — `arith_operand` が説明文字列の先頭（`@`）で見分けるか、トランスパイラーが被演算子の種類を別引数で渡すか。生成コードの形を変えると `transpiler.md` とスナップショットが広く変わるので、説明文字列で見分ける案を既定とする。
4. **空文字列の変数** — `＄x＝「」` のあとの `＄x＋1` は `tonumber("")` が `nil` で、従来どおり警告＋値なしのまま（「未代入」ではないため）。この境界をマニュアルに書く。
5. **連結 `＆` との対称性** — 未代入の変数を `＆` で空文字とみなすかは、本 spec の範囲外（下の Out of Boundary）。要件で「変えない」と明記する。

## Scope

- **In**: `act.lua` の `arith_operand`（未代入の変数の 0 扱い）、必要なら `expr_gen.rs` の被演算子の説明、既存テスト（`act_runtime_safety_test.lua`・`runtime_safety_test.rs`・スナップショット）の更新と新しいケース、マニュアル 3 ページ（`grammar/variables.md`「算術の評価」「連結の評価」の作例、`lua/script-api.md`、`internals/internal-modules.md`）、スキル `references/` の再生成（`node book/tools/gen-skill-refs.mjs`）と `pasta-ghost-authoring/SKILL.md` の算術の 1 行。
- **Out**: 連結 `＆` の未代入の扱い、アクション行の変数参照（`＄x` を空文字に展開して警告）の変更、数字でない文字列・真偽値の扱いの変更、DSL の文法変更（条件分岐・代入演算子 `＋＝` など）、警告の出口の一本化（`failure-output-unification`）。

## Boundary Candidates

- `arith_operand` の被演算子の数値化（ランタイム）
- 被演算子の説明の生成（トランスパイラー）— 区別の仕組みを説明文字列に寄せれば触らない
- マニュアルとスキル references の同期

## Out of Boundary

- 連結 `＆` で未代入の変数を空文字とみなすこと。対称性からは自然だが、動機（困った作例）が無いので入れない。動機が生じたら、その動機から別に起票する。
- 「値なしを代入した変数は未代入になる」という規則の変更。
- アクション行で未代入の変数を参照したときの空文字展開と警告（`act:talk - undefined variable`）。
- `＋＝` のような複合代入の文法。`＄x＝＄x＋1` で足りる。

## Upstream / Downstream

- **Upstream**: `dsl-codegen-runtime-safety`（完了。算術の失敗の扱いと `act:arith` の形）、`string-concat-operator`（完了。説明文字列の形と内側の失敗の伝播規則）。
- **Downstream**: `hello-pasta-tutorial-stages`（10 段目 `10-save.pasta` の作例がこの挙動に依存する。本 spec が main に入ることを同 spec の実装着手のゲートに加えている）、`getting-started-story-guide`（10 段目の章の説明）。

## Existing Spec Touchpoints

- **Extends**: なし（`dsl-codegen-runtime-safety` は完了済みなので更新しない。挙動の変更はこの spec が持つ）。
- **Adjacent**: `failure-output-unification`（未着手。`act.lua` の `act:arith - operand is not a number` を含む警告の出口を一本化する。本 spec は同じ関数 `arith_operand` の数値化の分岐だけを触り、警告の文言・出口は触らない。同じウェーブに置くなら、本 spec を先に入れて `failure-output-unification` が rebase する）、`call-attribute-filter`（`act.lua` を触るが `call`・`find_act_handler` 側なので重ならない）。

## Constraints

- 現行実装を正とし、マニュアルを同じ PR で直す（roadmap「前提として確定している方針」）。マニュアルの作例 `＄x＝＄未代入＋1` と `＄y＝「合計」＆（＄未代入＋1）` は、新しい挙動では結果が変わる（`＄x` は 1、`＄y` は `合計1`）ので、作例ごと書き直す。「数値にできない被演算子」の例は数字でない文字列などに差し替える。
- 「書き間違いを 500 にしない」は保つ。0 とみなすのは未代入の変数だけで、それ以外の失敗は従来どおり警告＋値なし。
- 変更は `arith_operand` の数行に収まる見込みで、`act.lua` の他の関数と `ACT_IMPL.arith` の引数・戻り値は変えない（`actor-proxy-act-delegation`・`call-execution-correctness` が前提にしている）。
- 規模: 5〜8 タスク（ランタイムの分岐、Lua テスト、Rust テストとスナップショット、マニュアル 3 ページ、スキル references）。要件定義は Fable でなくてよい（決定済みの挙動を要件に落とすだけ）。
- `hello-pasta-tutorial-stages` が待っているので、`scene-name-alias` と同じく早く main に入れる。
