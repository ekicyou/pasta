# Brief: runtime-error-balloon

起点: 2026-10-10 の棚卸で `failure-output-unification` から分割した。失敗の出力の一本化に、スクリプトが止まるエラー（500 の応答）のバルーン表示まで入れると 25〜29 タスクになり、20 を超える。そこで、シーンが生きている失敗を元の spec に残し、シーンや要求そのものが止まるエラーをこの spec に出した。

> **ステータス**: 起票だけ。開発者は、500 の応答になるエラーをバルーンに出すと、まだ決めていない。要件定義の最初の議題で「やるか、やらないか」を決める。やらないと決まったら、この spec は却下する。

## Problem

辞書の中の Lua のブロックや、ゴースト作者が書いた Lua のスクリプトでエラーが起きると、そのシーンは止まる。ベースウェアには 500 の応答が返り、理由は応答の中の 1 行とログにしか出ない。バルーンには何も出ないので、作者には「ゴーストが急に黙った」としか見えない。原因を知るには、ログのファイルを開くしかない。

`failure-output-unification` が入ると、シーンが生きている失敗（未代入の変数、見つからない単語や Call など）はバルーンで見えるようになる。止まるエラーだけがバルーンに出ない、という不揃いが残る。

## Current State

2026-10-10 の main（`add05022`）で確かめた。

- **Lua の側の出口**:
  - SHIORI の要求を受ける関数 `SHIORI.request`（`crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua` 79 行）は、イベントの処理全体を守って呼ぶ。エラーが起きると、エラー文の最初の 1 行を取り出し（34 行の `error_handler`）、500 の応答を作る関数 `RES.err` に渡す（87 行）。
  - `RES.err`（`crates/pasta_lua/pasta_scripts/pasta/shiori/res.lua` 119 行）は、理由を `X-Error-Reason` の行に入れた 500 の応答を作る。
  - シーンの途中で起きたエラーは、イベントを振り分ける関数 `EVENT.fire` が投げ直す（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` 228 行）。そのまま上の出口へ行く。途中まで積んだ台詞は捨てられる。
  - 待ち合わせの時間切れ（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/callback.lua` の `time_out`）は、理由の文が付いていれば、その文を理由にした 500 を作る（172 行）。理由の無い時間切れの後のエラーは、警告のログだけを出し、応答を作らない。
- **Rust の側の出口**:
  - エラーを 500 の応答にする関数 `to_shiori_response`（`crates/pasta_shiori/src/error.rs` 104 行）は、理由を `X-ERROR-REASON` の行に入れる。理由は 1 行にまとめる（79 行の `single_line`）。
  - 要求を処理するスレッド（`crates/pasta_shiori/src/actor/thread.rs`）は、エラーをこの 500 の応答として返す（197 行）。対象は、読み込みの失敗・まだ読み込んでいない状態・Lua の実行エラーの 3 つ。通知（NOTIFY）の応答は捨てる（209 行）。
  - 要求の入口 `request`（`crates/pasta_shiori/src/shiori.rs` 137 行）は、読み込みに失敗したままのとき、最後の読み込みのエラーを返す（142 行から）。
- **行の名前がそろっていない**: 理由の行の名前は、Lua の側が `X-Error-Reason`、Rust の側が `X-ERROR-REASON` である。
- **マニュアル**: 500 の応答と理由の行は、`book/src/lua/shiori-events.md` の「エラーハンドリング」、`book/src/reference/startup.md` の「ゴーストが起動しない・喋らないとき」、`book/src/internals/shiori.md`、`book/src/debug/troubleshooting.md` が説明している。
- **伺かの仕様**（SSP の公式の仕様書 UKADOC の「SHIORI/3.0」の応答の項で確かめた）:
  - ステータスコードは、200 番台が成功、ほかは何かの失敗を表す。500 は「SHIORI の内部でエラーが起き、応答を返せなかった」である。
  - `X-ERROR-REASON` は UKADOC に載っていない。pasta が独自に付けている行である。
  - UKADOC が決めているエラーの知らせ方は、`ErrorLevel`（エラーの重さ）と `ErrorDescription`（エラーの説明）の 2 つの行である（どちらも SSP の拡張）。`ErrorLevel` があると、SSP はエラーのログに記録し、通知領域にエラーのアイコンを出す。pasta は、この 2 つを使っていない。
  - 未確認: 500 の応答に台詞（`Value`）を付けたとき、SSP がそれをバルーンに出すか。SSP が `X-ERROR-REASON` の行を何かに使うか。
- **起動の失敗はバルーンに出せない**: マニュアル（`book/src/reference/startup.md`）によると、SSP 2.8.98 は、読み込みに失敗したゴーストへ要求を送らない。500 の応答そのものが起きない。

## Desired Outcome

- シーンや要求そのものが止まるエラーでも、辞書の作者は、ゴーストを動かしているだけで「どこで何が起きたか」をバルーンで読める。
- 見せる形（文言・囲み方）と、出す・出さないの切り替えは、`failure-output-unification` が決めたものと同じである。作者は、2 つの仕組みを覚えなくてよい。
- 配るゴーストでは、エラーが利用者のバルーンに出ない。
- ログと、応答の理由の行は、今までどおり残る。マニュアルの切り分けの手順は、そのまま使える。
- マニュアルが、止まるエラーの見え方を説明している。

## Approach

起票時の見立て。要件で確定する。

- **境目**: 「実行中のシーンが生きていて、バルーンに続きを書けるか」。書ける失敗は `failure-output-unification` が持つ。書けない（シーンや要求が止まる）エラーを、この spec が持つ。
- **入口は 2 つある**: Lua の中で捕まえるエラー（`SHIORI.request` の出口）と、Rust まで届くエラー（`to_shiori_response`）。Lua の実行エラーの多くは Lua の側で捕まる。Rust まで届くのは、読み込みの失敗などに限られる。Lua の側だけで足りるかを、要件で先に確かめる。足りれば `pasta_shiori` のソースを触らずに済む。
- **見せ方の候補**:
  - 応答を 200 にして、エラーの表記を台詞として返す。
  - 500 のまま、台詞を付けて返す（SSP が出すかは未確認）。
  - 500 のまま返し、`ErrorLevel` と `ErrorDescription` を付けて、SSP のエラーのログとアイコンで知らせる（バルーンではないが、作者には見える）。
- **表記と切り替えは作らない**: `failure-output-unification` の報告の関数・表記の決まり・pasta.toml の切り替えを使う。足りないものが見つかったら、向こうに頼む。

## Scope

- **In**:
  - Lua の側の 500 の出口（`SHIORI.request`・`RES.err`・時間切れ）で、エラーをバルーンに見せること
  - Rust の側の出口（含めるかは要件で決める）
  - `failure-output-unification` が置く pasta.toml の切り替えに従うこと
  - テスト（`pasta_lua` の Lua のテストと、`pasta_shiori` の結合テスト）
  - マニュアルの該当するページの更新と、スキル用の文書の再生成
- **Out**:
  - シーンが生きている失敗の見せ方（`failure-output-unification` が持つ）
  - 起動（読み込み）の失敗をバルーンに見せること（SSP は、読み込みに失敗したゴーストへ要求を送らない。ログで見せる仕組みは、完了した `lua-require-robustness`・`load-error-logging` が入れた）
  - エラーの後でシーンを続けること・やり直すこと（止まることは変えない）
  - 通知（NOTIFY）で起きたエラー（応答を表示する道が無い）
  - 動かす前に書き間違いを見つける検査（`pasta-check-dic-validate` が持つ）
  - デバッガへのエラーの知らせ

## Boundary Candidates

- Lua の側の出口（`entry.lua`・`res.lua`・`event/init.lua`・`event/callback.lua`）
- Rust の側の出口（`error.rs`・`shiori.rs`・`actor/thread.rs`）
- 表記と切り替えを使う側のつなぎ（`failure-output-unification` の関数と設定を呼ぶだけ）
- 結合テストとマニュアル

## Out of Boundary

- 失敗を報告する関数・表記の形・切り替えの設定そのもの（`failure-output-unification` が持つ）
- `crates/pasta_lua/pasta_scripts/pasta/act.lua`・`actor.lua`・`word.lua`（この spec は触らない）
- 応答の理由の行とログの中身（残す。行の名前の不揃いを直すかだけを要件で決める）

## Upstream / Downstream

- **Upstream**:
  - `failure-output-unification`（報告の関数・表記の決まり・pasta.toml の切り替え。これを使う）
  - `shiori-test-support-runtime`（`crates/pasta_shiori/tests/` を先に片付ける。先に入っていないと、新しい結合テストが古いランタイムの写しを避ける設定を、また写すことになる）
  - 完了した `lua-require-robustness`（500 の応答と理由の行を入れた）、完了した `callback-resume-unification`（時間切れの 500 を入れた）
- **Downstream**: なし。

## Existing Spec Touchpoints

- **Extends**: なし（新規。`failure-output-unification` の論点「500 になる実行時エラーもバルーンに出すか」を引き取った）
- **Adjacent**:
  - `failure-output-unification`: 表記と切り替えを共有する。ソースは重ならない（向こうは `act.lua`・`actor.lua`・`word.lua`、こちらは `shiori/` の下と `pasta_shiori`）。
  - `shiori-test-support-runtime`: `crates/pasta_shiori/tests/` を共有するので、その後に置く。
  - `scene-anchor-link`: 同じフォルダーの別のファイル（`shiori/event/choice_select.lua`）を触る。ファイルは重ならない見込みだが、クリックの受け口の登録の置き場所しだいで `shiori/event/` の中で近づく。
  - マニュアルの `book/src/lua/shiori-events.md` は、ほかの spec も別の節を触る。後から入る側が rebase で合わせる。

## Constraints

- マニュアルが唯一の権威。挙動を変えたら同じ変更でマニュアルを直し、`node book/tools/gen-skill-refs.mjs` でスキル用の文書を再生成する。
- 現行実装を正とする。エラーでシーンが止まること、ログと理由の行が出ることは変えない。
- 完成度を優先する。開発者が「要らない」と決めたら、縮めて出さずに却下する。
- 配るゴーストの利用者に、エラーの文を見せない。
- エラーの文には、ファイルの場所や辞書の中身が入る。表記の中でさくらスクリプトとして実行されないようにする決まりは、`failure-output-unification` のものに従う。
- `crates/pasta_shiori/tests/` は、`shiori-test-support-runtime` と同じウェーブに置かない。

## 2026-10-10 棚卸の測定（main add05022）

- **触るファイル**: `crates/pasta_lua/pasta_scripts/pasta/shiori/` の `entry.lua` 120・`res.lua` 141・`event/init.lua` 254・`event/callback.lua` 213。Rust の側まで含めるなら `crates/pasta_shiori/src/` の `error.rs` 406・`shiori.rs` 397・`actor/thread.rs` 272。テストは `crates/pasta_lua/tests/lua_specs/`（`shiori_entry_test.lua`・`callback_module_test.lua`）・`crates/pasta_lua/tests/shiori/`（`res_test.rs`・`event_handler_test.rs`）と、`crates/pasta_shiori/tests/` の結合テスト（500 を確かめている `shiori_response_test.rs`・`async_callback_chain_test.rs`・`actor_marshaling_test.rs` など）。マニュアルは `book/src/lua/shiori-events.md`・`book/src/reference/startup.md`・`book/src/internals/shiori.md`・`book/src/debug/troubleshooting.md` と、生成するスキル用の文書。1,000 行に近いファイルは無い。
- **規模**: 8〜10 タスク（実機での確かめ 1、Lua の側の出口 2〜3、時間切れ 1、Rust の側の出口 1〜2、切り替えのつなぎ 1、テスト 1、マニュアルと生成 1）。Lua の側だけで足りるなら 6〜7 タスク。
- **先に要るもの**: `failure-output-unification`（表記・切り替え）と `shiori-test-support-runtime`（結合テストの片付け）。どちらも未完了。
- **種別**: 機能（止まるエラーが、ログと応答の 1 行にしか出ない）。ただし、やるかどうかは開発者がまだ決めていない。
- **要件定義のモデル**: Fable（やるかどうか、応答コード、配るゴーストでの扱いという、開発者の判断の分かれ道が続く。伺かの仕様の読み方と、SSP の実機の動きの確かめが要る）。
- **要件定義の議題**:
  - そもそもやるか。開発者は、500 の応答になるエラーをバルーンに出すと、まだ決めていない。要らないなら、この spec は却下する。
  - バルーンに出すときに返す応答コード。UKADOC では、500 は「応答を返せなかった」で、成功は 200 番台である。200 にして台詞として返すか、500 のまま台詞を付けるか（SSP が出すかは未確認。実機で確かめる）、500 のまま `ErrorLevel`・`ErrorDescription` で SSP のエラーのログに出すかを選ぶ。`X-ERROR-REASON` は UKADOC に無い pasta 独自の行で、SSP がどう扱うかは未確認。
  - 配るゴーストで、エラーを利用者に見せない方法。`failure-output-unification` の切り替えと同じ設定に従うか、別の設定にするか。既定はどちらか。配布物を作る道具（`pasta_check` の `release`）で切り替えを確かめるか。
  - Rust まで届くエラーを含めるか。Lua の側だけで足りるなら、`pasta_shiori` のソースを触らずに済む。
  - 途中まで積んだ台詞の扱い。今は捨てている。途中までを見せてからエラーを見せるか、エラーだけを見せるか。
  - 時間切れのエラーを含めるか。理由の付いた時間切れは 500 を返す。理由の無い時間切れの後のエラーは、今は応答を作らない。
  - 理由の行の名前の不揃い（`X-Error-Reason` と `X-ERROR-REASON`）を、この機会にそろえるか。
