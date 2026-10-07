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
- **申し送り（`call-execution-correctness` より、2026-10-05）**: 完了時点の失敗表記の出口は次のとおり。詳細は `.kiro/specs/completed/call-execution-correctness/design.md` の ActFailure・DynamicCallKey。
  - `act:failure(text, warning)` は、`warning` があれば `log.warn` し、既存の `raw_script` トークン `【text】` を積む。当初はアクター nil の `talk` だったが、句読点ウェイト（例 `var.\_w[950]`）と budoux の改行が入るため、開発者の判断で `raw_script` に変えた。新しいトークン型は作っていない。呼び出し元は `act:call`（見つからない）と `act:call_key`（キーにならない値）の 2 か所だけ。
  - `raw_script` は直前に話したアクターのグループに入り、グループを区切らない。そのため（1）スコープを戻った直後が発言以外で始まる場合、段落区切りの改行が失敗表記の前に入らない（受け入れた差）。（2）`act:actor_proxy` の直前の話者の遡りが失敗表記で止まらず、未登録アクターの目印と警告は話者の切り替わりで 1 回だけ出る。他の失敗を載せ替えるときもこの性質を引き継ぐか決める。
  - 失敗表記の中身はエスケープしていない。見つからない名前や値にさくらスクリプトのタグ（`\-` など）が入っているとそのまま実行される（アクション行の変数展開と同じ扱い）。一本化するときにエスケープするかを論点にする。
  - 「Current State」の警告一覧に、次の警告が増えた: `act:call - key is not a string or number: [operand='…', ]value=…`（動的コールのキーが使えない値のとき）と、`WORD.dynamic_key(値, 経路, "act:call")` が出す `act:call - undefined variable`・`empty variable`・`unsupported value type`。`act:call - handler not found` は失敗表記を伴うようになった。

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

## 2026-10-07 棚卸の再測定（main 2cbaf510）

- **前提の変化**: `call-execution-correctness` は完了し、失敗表記の出口 `act:failure(text, warning)`（`act.lua` 744〜750 行。`raw_script` トークンに `【…】` を積む）がある。呼び出し元は `act:call`（679 行）と `act:call_key`（706 行）の 2 か所で、申し送りどおり。
- **触るファイル**: `pasta_scripts/pasta/act.lua`（774 行。1,000 行に近づいている）・`actor.lua`（272 行）・`word.lua`（185 行）。範囲しだいで `pasta_lua` の `loader/config/`（切り替えの設定）、`pasta_scripts/pasta/shiori/event/`・`shiori/res.lua`・`pasta_shiori/src/error.rs`（500 の経路）。マニュアル `grammar/action-line.md`・`grammar/call-jump.md`・`lua/script-api.md`・`internals/talk-output.md`・`internals/internal-modules.md`（失敗表記をすでに書いている 5 ページ）と生成スキル。
- **規模**: Lua 側の一本化と載せ替えだけなら約 14〜16 タスク（出口の抽出 1、表記とエスケープの規則 2、`act.lua` の 9 か所・`actor.lua` の 2 か所・`word.lua` の 3 か所の載せ替え 5〜6、重ねない規則 1、マニュアルと生成 2、テスト 2〜3）。設定での切り替え（+2）と 500 のバルーン表示（+3〜4）まで入れると 20 を超える。
- **先に要るもの**: 機能の前提は無い（`call-execution-correctness` は完了）。ロードマップの `call-attribute-filter` への依存は、`act.lua` の重なりによる順序だけである。`scene-attribute-store` が読み出しの口を `SCENE` 側に置き `act.lua` を触らなければ、ソースは重ならず同じウェーブで並走できる。Phase 12・`release-ci` とも重ならない。
- **種別**: 機能（失敗をバルーンに見せる。空文字で展開する現行の方針を変える）。
- **要件定義のモデル**: Fable（破壊的変更の扱い・配布ゴーストでの切り替え・500 をバルーンに出すかの、開発者の判断の分かれ道が多い。`pasta_shiori` をまたぐ）。
- **分割の案**: 要件で 500 の経路まで含めると決めたら、`failure-output-unification`（Lua 側の出口と載せ替え）と、500 の実行時エラーをバルーンに出す spec（`shiori/event/`・`res.lua`・`error.rs`）に分ける。境目は「`act` のトークンに積めるか」。含めないなら分割は要らない。
- **見つけた穴・古くなった記述**:
  - Current State の「失敗をさくらスクリプトへ出す仕組みは無い」は古い。現在は 2 つある: `act:failure`（`raw_script`、Call だけ）と、`act:actor_proxy` の目印 `【未登録アクター：名前】`（`act.lua` 482 行。`talk` トークンにアクター付きで積む。`dsl-codegen-runtime-safety` が入れた）。形もトークン型も違う。この 2 つを 1 つにするのが最初の仕事になる。
  - 警告一覧に `act:arith - unknown operator`（557 行）が無い。プロキシ側は `proxy:expr_fn`・`proxy:word`（`actor.lua` 195・249 行）。`act:call - nil key`（662 行）は失敗表記を伴わないが、生成コードは `act:call_key` を通すので、Lua から直接 `act:call` を nil で呼んだときだけ届く。
  - `act:global_fn`（499 行）は関数が無いときに警告だけを出す。戻った後のシーンの復元（497 行）は `call-execution-correctness` が入れた。
  - 再利用する部品の場所は現行と一致する: `operand_desc`（`expr_gen.rs` 207 行）・`dynamic_ref_args`（`element_gen.rs` 48 行）・`single_line`（`error.rs` 79 行）。
- **順序の提案**: 本 spec を `call-attribute-filter` より先に置く（同 brief に記した）。フィルターの「候補なし」は、一本化した仕組みに直接載せる。
