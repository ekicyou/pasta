# Brief: dsl-codegen-runtime-safety

> **ステータス**: 未着手（ロードマップ棚卸 2026-10-04 で起票）。Wave 1（バグ修正・最優先）。着手するときは `/kiro-start dsl-codegen-runtime-safety` で開始する。

## Problem

トランスパイラが、存在を確かめない Lua の直接アクセス（`GLOBAL.名前(…)`・`act.アクター名`・生の算術演算子）や、さくらスクリプトとして不正な文字列を出力する。そのため、作者の小さな書き間違いや想定外の値で、イベント全体が実行時エラー（SHIORI 500）になるか、壊れたさくらスクリプトになる。同じ種類の参照でも、ローカルの `＠名前（）` は警告を出して空で続くため、挙動がそろっていない。

| ID | 書き方 | 現象 |
| -- | ------ | ---- |
| U18 | 未定義の `＠＊関数（）` | Lua の実行時エラー → 500 |
| U19 | アクター辞書にも `[actor]` にも無い名前のアクション行 | 実行時エラー → 500 |
| U20 | `talk`・`word`・`call`・`var`・`save` など act のメンバー名と同名のアクターのアクション行 | 実行時エラー（メソッド・フィールドが返る） |
| U22 | 数字でない文字列・未代入の変数・nil を返す関数の算術（`＄x＋1` など） | 実行時エラー → 500 |
| U08 | アクション行の `\\` | `\` 1 文字の台詞になり、直後の文字とタグを作る（`C:\\new` → `\n` 改行タグ、行末の `\\` が終端の `\e` を壊す） |

## Current State

照合記録は `manual-ssot-authority` の吸収台帳（付録「未記載の実装事実」U08・U18・U19・U20・U22）。2026-10-04 の棚卸で現行 main でも再現することをコード上で確認した。共通の根は `crates/pasta_lua/src/code_gen/element_gen.rs` の出力形にある。

- **U18**: `element_gen.rs` のアクション行（372 行付近）と式（489 行付近）が、存在確認なしに `GLOBAL.name(act, …)` を出力する。
  - マニュアル `book/src/grammar/variables.md` 110 行付近は、ローカルの `＠名前（）` が見つからない場合（警告＋空）だけを書く。
- **U19・U20**: アクション行が `act.<アクター名>:talk(…)` を出力する（316–416 行付近）。
  - `crates/pasta_lua/pasta_scripts/pasta/act.lua` の `ACT_IMPL.__index`（146–158 行付近）は、未知の名前に nil を返す。
  - act のメソッド名は先にメソッドを返す。`actors`・`save`・`var`・`token`・`current_scene`・`app_ctx`・`req` は実フィールドのため、`__index` を通らない。
  - マニュアル `book/src/lua/script-api.md` 207 行付近と `book/src/internals/internal-modules.md` 117 行付近は、この制限を書いている。手書き Lua の `act.名前` については今後も正しい記述。
- **U22**: `generate_expr_to_buffer` の Binary アーム（505 行付近）が生の ` + - * / % ` を出力する。`resolve_var_path` に nil ガードは無い。
  - 括弧式は棚卸の即時修正（U12）で全体が `Expr::Paren` に入るようになった。
  - 文字列の `＋` は連結にならない（U11・現行仕様）。
- **U08**: `element_gen.rs` の `Action::Escape` アーム（385–392 行付近）が 2 文字目の `\` を `talk()` で出力する。`sakura_builder.lua` は台詞中の `\` をエスケープしない。
  - 文法は `grammar.pest` 182 行付近の `sakura_escape`。
  - マニュアルは意図して `\\` を書いていない（吸収台帳 U08 の判定）。

## Desired Outcome

- DSL の書き間違い（未定義のグローバル関数・未登録のアクター・act のメンバーと同名のアクター・数値にできない算術）で、イベントが 500 にならない。警告ログ（参照した名前付き）を出し、決めた既定の結果（空・nil など）で続く。ローカルの `＠名前（）` の「警告＋空」とそろう。
- アクション行の `\\` が、さくらスクリプトの規約どおり `\` 1 文字の表示になる。
- マニュアル（`grammar/action-line.md`・`grammar/variables.md`・`lua/script-api.md`・`internals/internal-modules.md`・`internals/transpiler.md`）が新しい挙動を書き、スキル `references/` を再生成している。

## Approach

要件フェーズで次を決める。棚卸での推奨を併記する。

- **U18**: `act` に存在確認付きの呼び出し口（例 `act:global_fn("名前", …)`。無ければ警告して nil）を足し、アクション行と式の両方から使う。ローカルの `＠名前（）` と同じ「警告＋空」にそろえる。
- **U19・U20**: アクション行を `act.名前` ではなく、名前を文字列で受けてプロキシを返す口（例 `act:actor_proxy("名前")`）で出力する。これで act のメンバー名との衝突も消える。
  - 未知のアクターの扱いを決める。候補は 2 つ: 警告して行を飛ばす（`％` 行の無視 U02 と同じ考え方）か、`ACTOR.get_or_create` で自動作成するか。
- **U22**: Binary を実行時ヘルパー（例 `act:arith(op, a, b)`。`tonumber` で変換、失敗時は警告＋決めた結果）経由で出力する。入れ子の括弧は入れ子の呼び出しになる。
  - 失敗時の結果（nil か空か 0 か）を決める。
  - 文字列の `＋` を連結にしない現行仕様（U11）は維持するかを確認する。
- **U08**: `\\` を、後処理で分割されない経路（`sakura_script` など）で `\\` のまま出力する。DSL の `\\` を「`\` の表示」と定義し、`grammar/action-line.md` に書く。
- 生成コードの形が変わるため、トランスパイラのスナップショットは広く更新される。ソースマップ（`.pasta` 行 ↔ 生成 Lua 行）が崩れないことを既存テストで確かめる。

## Scope

- **In**:
  - U18・U19・U20・U22・U08 の修正（`element_gen.rs` の該当アーム、`act.lua` の新しいヘルパー）
  - 修正を固定するテスト（トランスパイラのスナップショット、Lua 実行テスト、500 にならないことの SHIORI 経由のテスト）
  - マニュアルの該当章の更新とスキル `references/` の再生成
- **Out**:
  - アクタープロキシが ACT を前提とする関数に渡る問題（`actor-proxy-act-delegation`。Wave 2）
  - Call の生成形（`call-execution-correctness`。Wave 3）
  - 文字列リテラル・単語値（`dsl-literal-fixes`）

## Boundary Candidates

- コード生成（`element_gen.rs`）: アクション行のアクター参照・`＠＊` 関数呼び出し・Binary・Escape の各アーム
- ランタイム（`act.lua`）: 存在確認付きの呼び出し口・プロキシ取得口・算術ヘルパー
- マニュアル・生成スキル

## Out of Boundary

- `actor.lua`（`PROXY_IMPL`）の変更。Wave 1 では `scene-search-key-normalization` が持つ。
- シーン・単語の検索アルゴリズム

## Upstream / Downstream

- **Upstream**: `manual-ssot-authority`（吸収台帳の照合記録）、棚卸の即時修正 U12（括弧式）
- **Downstream**: `actor-proxy-act-delegation`（新しいプロキシ取得口を前提にする）、`act-token-grouping-fix`・`call-execution-correctness`（`act.lua`・`element_gen.rs` をこの spec の後に触る）

## Existing Spec Touchpoints

- **Adjacent**: `dsl-literal-fixes`（同じ Wave。`pasta_dsl` の文法を持つ）、`dynamic-word-reference`（完了。`dynamic_ref_args` で変数パスを警告に渡す前例）

## Constraints

- マニュアルが文法・API の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- 現行実装を正として設計する。
- 手書き Lua の `act.名前` の挙動（`script-api.md` の制限の記述）は変えない。
- 並走条件（Wave 1）: Wave 1 の中で `element_gen.rs` と `act.lua` を編集するのはこの spec だけ。`actor.lua`・`pasta_dsl` の文法・`shiori/event/*` は触らない。
