# Brief: failure-output-unification

> **ステータス**: 未着手（2026-10-05、`call-execution-correctness` の要件ディスカッション中に起票）。起票だけを行い、質問による掘り下げはしていない。未確定の点は「Approach」に要件フェーズの論点として残した。着手するときは `/kiro-start failure-output-unification` で開始する。

## Problem

ゴースト作者は、辞書の書き間違い（未代入の変数、存在しない単語・関数・シーン、未登録のアクター、数値にできない算術）に、ゴーストを動かしているだけでは気づけない。

- 実行時の失敗は、ログの警告（`log.warn`）にしか出ない。バルーンでは、その箇所が空になるか、行が黙って飛ばされるだけである。
- 失敗の出し方が場所ごとにばらばらである。警告の文言・変数や関数の表記・重ねて警告するかどうかの規則を、各ヘルパーが個別に持っている。
- スクリプトの実行時エラーは 500 応答の `X-ERROR-REASON` に出るが、これもバルーンには出ない。
- `call-execution-correctness` が「Call 行の失敗をバルーンに見える文字列（失敗表記）で出す」ことを先に決めた（要件ディスカッション #3・#4）。Call 行だけが見え、他の失敗は見えないという不揃いが生じる。

## Current State

2026-10-05 時点の main（`b7379789`）をコード上で確認した。

- **失敗をさくらスクリプトへ出す仕組みは無い**。失敗の出口は次の 2 つだけである。
  - ログの警告: `crates/pasta_lua/pasta_scripts/pasta/act.lua` の `log.warn`（`act:talk - undefined variable`・`act:word - handler not found`・`act:expr_fn - handler not found`・`act:actor_proxy - unregistered actor`・`act:global_fn - function not found`・`act:arith - operand is not a number`・`act:concat - operand is not a string or number`・`act:call - nil key`・`act:call - handler not found`）、`pasta/word.lua` の `WORD.dynamic_key`（`undefined variable`・`empty variable`・`unsupported value type`）、`pasta/actor.lua` のプロキシ側の同種の警告。
  - 500 応答: `pasta/shiori/res.lua` の `RES.err`、`crates/pasta_shiori/src/error.rs` の `to_shiori_response`（`X-ERROR-REASON` を単一行に整形）。起動失敗の可視化は `lua-require-robustness`（#33）と `load-error-logging` が扱った。
- **再利用できる部品（サルベージの対象）**:
  - 警告用の説明を作る `operand_desc`（`crates/pasta_lua/src/code_gen/expr_gen.rs`）と、値と変数の経路を渡す `dynamic_ref_args`（`code_gen/element_gen.rs`）。
  - 「内側が警告済みなら重ねない」慣行（`act:talk`・`act:arith`・`act:concat` は、値が nil で説明も無いとき黙る）。
  - `X-ERROR-REASON` の単一行化（`error.rs` の `single_line`）。
  - `call-execution-correctness` が Call 行のために入れる失敗表記の出力（同 spec の完了後に存在する）。
- **現行の方針**: 未定義の変数・単語・関数の参照は空文字で展開する（be9bbed7、2026-09-26。`dsl-codegen-runtime-safety` が 500 を警告＋空へ変えた）。マニュアルもこの挙動を書いている。

## Desired Outcome

- 実行時の失敗が、ログとさくらスクリプトの両方に、1 つの仕組みから出る。失敗の種類ごとに出口を個別に書かない。
- 作者がゴーストを動かせば、どの行の何が失敗したかをバルーンで分かる。
- 失敗表記の形（文言・変数や関数の表記）と、重ねて出さない規則が 1 か所で決まっている。
- `call-execution-correctness` が入れた Call 行の失敗表記が、この仕組みに載っている。
- マニュアルが失敗時の出力を 1 か所で説明し、各章はそこを参照する。

## Approach

起票時の見立て。要件フェーズで確定する。

- **出口を 1 つにする**: 「失敗を報告する」関数を 1 つ置き、ログの警告と失敗表記のトークンの両方をそこから出す。既存の各 `log.warn` の呼び出しをこの関数へ寄せる。
- **段階**: まず `call-execution-correctness` の失敗表記の出力をこの関数として切り出し（挙動を変えない抽出）、次に他の失敗へ広げる。
- **要件フェーズの論点**:
  - 対象にする失敗の範囲。アクション行の未定義の参照（変数・単語・関数）、未登録のアクター、算術・連結の被演算子、動的単語参照の型、Call。どこまでを「バルーンに出す」にするか。
  - 空文字で展開する現行の方針（be9bbed7）を変えることになる。破壊的変更の扱いと、マニュアルの書き換えの範囲。
  - 配布するゴーストでも常に出すか、設定（`pasta.toml`）で切り替えるか。`pasta_check` の検証・リリースとの関係。
  - 500 になる実行時エラーもバルーンに出すか（現行は `X-ERROR-REASON` だけ）。出す場合、応答コードをどうするか。
  - 発言より前・アクター未確定の位置で失敗した場合の出し方。
  - 入れ子の失敗（内側が報告済み）で表記を 1 つに保つ規則。
  - 変数・関数の表記を DSL の書き方（`＄x`・`＠名前（）`）に揃えるか。`call-execution-correctness` は、ログの警告と同じ表記（`var.x`・`@名前()`）を `【Call失敗：…】` の枠で出し、失敗の出口を `act:failure(text, warning)` の 1 関数にした（同 spec の設計ディスカッション #2）。

## Scope

- **In**:
  - 失敗を報告する仕組みの一本化（ログの警告と、さくらスクリプトへの失敗表記）
  - 既存の警告箇所の載せ替えと、失敗表記を出す対象の拡大
  - 失敗表記の形・重ねない規則の統一
  - マニュアルの該当章の更新とスキル `references/` の再生成、テスト
- **Out**:
  - Call 行の失敗表記そのものの導入（`call-execution-correctness` が持つ）
  - 起動失敗の可視化（`lua-require-robustness`・`load-error-logging` で完了）
  - トランスパイル時・`pasta_check` での静的な検出（実行前に見つける仕組みは別の話）
  - ログの設定・ローテーション（`pasta-toml-logging-consistency` で完了）

## Boundary Candidates

- 失敗の報告の入口（ランタイムの 1 関数）と、ログ・トークンへの振り分け
- 失敗表記の形（文言・表記の規則）
- 既存の警告箇所の載せ替え（`act.lua`・`actor.lua`・`word.lua`）
- 500 応答の経路（`shiori/event/`・`res.lua`・`pasta_shiori` の `error.rs`）。含めるかは要件フェーズで決める
- マニュアル・生成スキル

## Out of Boundary

- 失敗したときの制御の流れ（空として続行する・行を飛ばす・500 にする、の選び方）。この spec は「どう見せるか」を持ち、「続行するか止めるか」は変えない
- シーン・単語の検索アルゴリズム
- デバッガ（DAP）への失敗の通知

## Upstream / Downstream

- **Upstream**:
  - `call-execution-correctness`（Call 行の失敗表記を先に入れる。この spec が載せ替える最初の対象）
  - `dsl-codegen-runtime-safety`（完了。警告＋空で続行する形と `operand_desc` を入れた）
  - `act-token-grouping-fix`（完了。ACT のトークンのグループ化）
- **Downstream**: 特になし。`scene-attribute-store`・`call-attribute-filter` が新しい失敗（属性の型・フィルターの不一致）を足す場合は、この仕組みに載せる。

## Existing Spec Touchpoints

- **Extends**: なし（新規）
- **Adjacent**: `call-execution-correctness`（失敗表記の形の出発点）、`dynamic-word-reference`（完了。型を含む警告）、`string-concat-operator`（完了。連結の警告）、`load-error-logging`・`lua-require-robustness`（完了。`X-ERROR-REASON`）

## Constraints

- マニュアルが文法の唯一の権威。挙動を変えたら同じ変更でマニュアルを更新し、生成スキルを再生成する。
- 現行実装を正として設計する。
- `act.lua` は多くの spec が触る共有接点で、1 ウェーブに 1 spec だけが持つ。`call-attribute-filter` と同じウェーブに置かない。
- リファクタリングは挙動を変えない抽出を先に行い、1 抽出 = 1 検証 = 1 コミットで進める。
