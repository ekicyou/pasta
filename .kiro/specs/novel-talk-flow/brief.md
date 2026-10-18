# Brief: novel-talk-flow

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 4 番目。

## Problem

pasta の出力は、デスクトップマスコットの喋り方に合わせて作ってある。数分おきに勝手に喋り、1 回の発言は短く、読み手のクリックを待たない。ノベルゲームは逆である。物語は読み手のクリックで進み、1 つの場面が長く続き、決めた順に場面が流れる。今の pasta でノベルの台本を返すと、次の所で食い違う。

- 台詞ごとのクリック待ちを、作者が毎行 `\x` と書くしかない。しかも `\x` を書くと、出力の組み立てが乱れる（下の「穴」）。
- 話者名を台本に載せる手段が無い。台本に出るのはスコープの番号だけである。
- 場面を名前で呼ぶと、前方一致で別の場面が選ばれることがある。
- 場面が終わっても、次の場面は始まらない。次に何かが起きるのは、タイマーのランダムトークである。

## Current State

- 1 回の応答は、場面が終わるか `＞yield` に当たるまでの出力を、1 本のさくらスクリプトにしたものである。途中で読み手の入力を待つ仕組みは、エンジンに無い。
- 出力の組み立ては `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`。アクターが替わると `\p[番号]` を出し、段落の区切りに `\n[150]` を出し、最後に必ず `\e` を付ける（284 行）。
- 1 字ごとの待ちと BudouX の改行は、エンジンの中で文に挟まれる（`crates/pasta_lua/src/sakura_script/`）。
- ランダムトークと時報は `pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua`。止める設定は無い。間隔は 10 秒以上に丸められる（58〜70 行）。`＊OnTalk`・`＊会話`・時報の場面が無ければ何も起きないが、止まっている場面（`STORE.co_scene`）があると、タイマーが再開してしまう（202〜204 行）。

### 2026-10-10 の検証で見つかった穴

- **`\x` で組み立てが乱れる**: ukadoc では、`\x` はバルーンを消し、スコープを `\0` に戻す。組み立ては `\x` を知らない。
  - 同じアクターが続くと `\p[番号]` を出さないので、`\x` の後の文が `\0` に出る（`sakura_builder.lua` 240 行）。
  - 消えたバルーンの頭に `\n[150]` が出る（245・263〜265 行）。
  - サーフェスの復旧（`appearance.lua` 112・123〜126 行）も `\x` を知らない。
- **前方一致**: 場面の検索は、前方一致で候補を集め、並べ替えて順に使う（`pasta_scripts/pasta/act.lua` 366〜370 行、`crates/pasta_core/src/registry/scene_table.rs` 185・270〜314 行）。`＞第1章` は `第10章` にも当たる。`OnNovelClick` は `＊OnNovelClickEx` にも当たる。完全一致で探す道は、デバッグ用のシーンキック（`shiori/event/kick.lua` 140〜146 行）にしか無い。
- **イベントの引数**: イベントの Reference は、場面の引数にならない。場面ごとに `＞transfer_req_to_var` を書く必要がある（`shiori/act.lua` 161〜190 行）。
- **止まっている場面が捨てられる**: 場面を起こすイベントは、止まっている場面を捨てる（`shiori/event/init.lua` 124〜132 行）。
- **BudouX の行の数え方**: 台詞のトークンごとに数え直す（`line_breaker.rs` 129〜131 行）。同じバルーンに前の文があっても、桁を知らない。
- **`\w[N]`（確かめて閉じた）**: areka の文書（`doc/PASTA_PROFILE.md` §5）は、pasta が出す `\w[N]` を上流の不具合とし、areka は無視している、と書いている。2026-10-10 に確かめた所、pasta のエンジンと見本のゴーストは `\w[` を出していない（エンジンが出す待ちは `\_w[ミリ秒]`）。出どころは、areka の文書にある古い作例の、手書きの `\w[500]` である。pasta の側に直す物は無い。areka の文書の作例を直すことを、`novel-areka-contract` の依頼に添える。

## Desired Outcome

- ノベル向けの出力に切り替える設定が 1 つある。切り替えると、次のようになる。
  - 台詞ごとにクリック待ちが入る。作者は `\x` を書かない。
  - 台本に、話者名が載る。地の文（話者なし）も書ける。
  - ランダムトークと時報が起きない。
  - 起動すると、決めた場面（タイトル）が始まる。
  - 場面の名前は、完全一致で選ばれる。
- 長い一本道を、「次の場面へ」で区切れる。台本はそこで終わり、読み手が読み終えたら、次の場面が始まる。
- 既読のための「台本を直さない限り変わらない ID」が、行ごとに台本に載る。
- デスクトップマスコット向けの出力は、何も変わらない。

## Approach

要件定義で決めること。

- **切り替えの形**: pasta.toml のどこに書くか。`[package]` は、将来のエンジンのプロファイル用に空けてある（`pasta-config-restructure`）。
- **クリック待ち**: 台詞ごとに入れる。行末の待ちと改ページの待ちの 2 段にするか。入れない行の書き方。組み立てが `\x` を知るようにする（スコープ・段落の区切り・サーフェスの復旧）。
- **話者名と声**: 話者名は、アクターの名前から出す。表示用の名前を別に持つか。声は、変わらない行 ID からファイル名で引く案（Yarn Spinner・TyranoScript の流儀）。この案なら、DSL に声の記法を足さずに済む。
- **変わらない行 ID**: 場面の登録名と、その中での行の通し番号で作る案。台本を直すと番号がずれることを、どこまで許すか。
- **「次の場面へ」**: 書き方（末尾の Call をそう読むか、別の書き方か）。台本の終わりに、次の場面を起こす印を置く。`\e` と同じ応答に入れる必要がある。
- **完全一致**: ノベル向けの設定では、場面の名前を完全一致で選ぶ。同じ名前の場面が複数あるときの扱い（Call の絞り込みで 1 つに決まる前提か）。
- **場面の途中の `＞yield`**: ノベルでは、場面の境目だけを区切りにする（`novel-checkpoint-rollback`）。途中の `＞yield` をどう扱うか。
- **1 字ごとの待ちと改行**: ノベルでは、文字の速度と折り返しを areka に任せ、エンジンでは挟まない案。
- **条件つきの選択肢**: フラグが立っているときだけ出す選択肢を、この spec に入れるか。入れないなら、場面を条件で分ける書き方（`call-attribute-filter`）で足りるかを確かめる。

## Scope

- **In**: ノベル向けの出力の切り替え。クリック待ちと、組み立ての `\x` への対応。話者名・声・変わらない行 ID の出力。ランダムトークと時報の停止。起動時の場面。「次の場面へ」。完全一致の場面選び。テスト、マニュアル、スキル references の再生成。
- **Out**: 目印の ID と変数の控え（`novel-checkpoint-rollback`）。セーブとロード（`novel-save-slots`）。`！` 行と柱の属性（`cue-command-runtime`。2026-10-17 に `scene-stage-attributes` をまとめた）。Call の絞り込み（`call-attribute-filter`）。

## Boundary Candidates

- 設定の読み込み（`loader/config/`）。
- 出力の組み立て（`sakura_builder.lua`・`appearance.lua`）と、文の加工（`sakura_script/`）。
- イベントの入口（`shiori/event/`。起動・タイマー・「次の場面へ」）。
- 場面の検索（`act.lua`・`search/`）。

## Out of Boundary

- 文字の速度・オート・スキップ・早送り（areka）。
- バックログの一覧（areka）。

## Upstream / Downstream

- **Upstream**: `novel-areka-contract`（話者名・声・ID・「次の場面へ」の載せ方の名前）、`cue-command-runtime`（`act.lua` を先に触る）。
- **Downstream**: `novel-checkpoint-rollback`、`shell-element-click`、`hello-novel-sample`、`manual-novel-guide`。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `failure-output-unification`（`act.lua`・`loader/config/`・失敗の表記）、`choice-line-layout`（選択肢の前の改行。`sakura_builder.lua`）、`tutorial-ssp-gaps`（タイマーと起動のイベント。2026-10-17 に `baseware-virtual-time` と `boot-surface-without-dic` をまとめた）、`.kiro/specs/completed/paragraph-break-tag-only-talk/`（段落の区切り）、`.kiro/specs/completed/actor-surface-restore/`（サーフェスの復旧）。

## areka への依頼

- クリック待ち `\x` と早送り（areka に brief がある: `areka-P0-talk-fast-forward`）。
- 話者名の欄。名前を指定してバルーンへ書くこと。
- 「読み終えたので次の場面へ」を、GET のイベントとして送ること。
- 変わらない行 ID を使った既読の記録と、声のファイルの引き方。

要件が固まったら、取り決めの文書へ書き、areka のセッションへ知らせる。手順は `novel-areka-contract` の brief の「依頼の出し方」。

## Constraints

- 切り替えない限り、今のゴーストの出力を 1 バイトも変えない。
- `act.lua` と `sakura_builder.lua` は、1 つのウェーブに 1 つの spec だけが持つ。
- 規模が 20 タスクを超えそうなら、要件定義の中で「出力の組み立て」と「場面の流れ（起動・次の場面へ・完全一致）」に分ける。
- NOTIFY のイベントには台本を返せない。「次の場面へ」は GET で受ける。

## 申し送り（choice-line-layout より）

2026-10-11 に `choice-line-layout` が完了し、出力の組み立てが次のように変わった。詳細は `.kiro/specs/completed/choice-line-layout/design.md` の「出力層」と「Revalidation Triggers」、マニュアルの `book/src/internals/talk-output.md`。

- **変わった事実**: 台詞・`sakura_script` に書いた引数の無い `\c` を、組み立てが吹き出しの消去として数えるようになった（段落の区切りの状態 `spot_has_text`・`pending_break` と、新しい状態 `spot_tail` を戻す）。`\x` と引数つきの `\c[…]` は、今も数えない（`choice-line-layout` の範囲の外）。
- **新しい状態**: `spot_tail`（吹き出しごとの「最後に表示したもの」。`"text"` か `"choice"`）で、選択肢の前後の改行を決める。設定は `[ghost]` の `choice_newlines`（既定 1.5）。
- **見直しが要る論点**: 「組み立てが `\x` を知るようにする」ときは、`\c` を読む場所（局所関数 `clear_span`・`text_parts`）と同じところで扱うかどうか、`\x` で戻す状態に `spot_tail` を入れるかどうかを決める。上の「穴」に書いた行番号（240・284 行）は動いている。
- **複雑度**: `BUILDER.build` の循環的複雑度は 28 になった（`-- luacheck: ignore 561` を付けている）。分岐を足すなら、分割するかどうかも論点になる。

## 申し送り（touch-poke-shortcuts より、2026-10-16）
- 完全一致でシーンを 1 つ選ぶ道が入った。`SceneTable::resolve_scene_id_exact`（`crates/pasta_core/src/registry/scene_table.rs`）・`SearchContext::search_scene_exact` と `@pasta_search` の `SEARCH:search_scene_exact(name, global_scene_name?)`（`crates/pasta_lua/src/search/context.rs`）・`SCENE.co_exec_exact(name)`（`pasta_scripts/pasta/scene.lua`。グローバルシーンだけ）。
- 同じ名前のシーンが複数あるときは、前方一致と同じ「並べ替えて使い切るまで繰り返さない」で選ぶ。キャッシュは `SceneCacheKey` の `exact` で前方一致と分けてある。
- 「完全一致の場面選び」の論点は、この部品を前提に決め直す（切り替えの仕組みはまだ無い）。参照: `.kiro/specs/completed/touch-poke-shortcuts/design.md` の SceneTableExact・SearchExact・CoExecExact と Revalidation Triggers。

## 申し送り（failure-output-unification より、2026-10-17）

- `act.lua` の組み立てが変わった。話者のトークンは talk・sakura_script と、アクター付きの `raw_script`（未登録アクターの目印）。アクターの無い `raw_script`（失敗表記）は直前の話者の遡りで飛ばされ、直前の話者のバルーンに続けて出る。`act:build` はトークンが 0 件か、すべて空の `raw_script` のとき nil を返す（204）。
- 実行時の失敗は `pasta/failure.lua` の `FAILURE.report(act, record)` から出す。この spec で足す失敗も同じ出口に載せる。
- 参照: `.kiro/specs/completed/failure-output-unification/design.md` の「ActGrouping」、マニュアル `book/src/reference/runtime-failures.md`・`book/src/internals/talk-output.md`。

## 申し送り（pasta-check-dic-validate より・2026-10-17 の棚卸で補った）

- 辞書の検査は `pasta_check validate`（と `release` の段 4 の関門、VSCode 拡張のゴースト全体の診断）として入った。核は `crates/pasta_check/src/validate/`（ファイルシステムに触れないライブラリ・WASM でも動く）。
- 検査を足すときに触る所: 報告の種類 `FindingKind`（`report.rs`。誤り／注意の分類と kebab-case の code・文言の `msg` を 1 か所で持つ）、検査の段（`validate/mod.rs` の `validate_ghost` が段の順に呼ぶ）、マニュアルの章 `book/src/debug/validate.md` の報告の表。
- 種類の追加・分類・出力の 1 行の形を変えるのは、`pasta-check-dic-validate` の design.md の Revalidation Triggers に当たる（CI・作者のスクリプト・マニュアル・`wasmBridge.ts` の形の検査に効く）。実行時と同じことを守る一致テスト `crates/pasta_check/tests/validate_parity_test.rs` も合わせて見直す。
- ノベル向けの設定（完全一致の場面選びなど）に合わせた検査を足すかどうか・誤りか注意かは、この spec の要件で決める。

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**:
  - 名前が完全に一致する場面を選ぶ部品が入った（`touch-poke-shortcuts`。`crates/pasta_core/src/registry/scene_table.rs` の `resolve_scene_id_exact` 251 行、`crates/pasta_lua/src/search/context.rs` の `search_scene_exact` 105 行、`pasta_scripts/pasta/scene.lua` の `SCENE.co_exec_exact` 227 行。最後のものはグローバルシーンだけ）。残るのは、ノベル向けの設定のときに Call と選択肢の探し方をこれに切り替える仕組みだけ。
  - 止まるエラーのバルーン表示（`runtime-error-balloon`）で、応答の出口が増えた。止まったときは `shiori/stop.lua` の `STOP.response` が、表記を載せた 200 の応答を作る（`shiori/entry.lua` 79〜101 行・`shiori/event/callback.lua` 173〜187 行）。「次の場面へ」の印をこの応答に付けない、と決める必要がある。
  - act は、実行中の場面の「行の対応」（生成コードの行と `.pasta` の行の組。`act.line_map`）を持つようになった。ただし `.pasta` の行番号は台本を直すとずれるので、変わらない行 ID の元にはならない。
- **触るファイル**: 設定の読み込み `crates/pasta_lua/src/loader/config/`（`sections.rs` 344）、出力の組み立て `shiori/sakura_builder.lua` 383、サーフェスの復旧 `shiori/appearance.lua` 567、1 字ごとの待ちと改行 `crates/pasta_lua/src/sakura_script/`（`mod.rs` 247・`wait_inserter.rs` 397・`line_breaker.rs` 462）、ランダムトークと時報 `shiori/event/virtual_dispatcher.lua` 288、起動 `shiori/event/boot.lua` 22、「次の場面へ」の受け口（新しいファイルと、`shiori/event/init.lua` に読み込みの 1 行）、場面の探し方 `act.lua` 829（1,000 行に近い）・`scene.lua` 240、行 ID を生成で埋めるなら `code_gen/scope_gen.rs` 445・`element_gen.rs` 564、検査を足すなら `crates/pasta_check/src/validate/`。
- **規模**: 17〜21 タスク（設定 1、クリック待ち 2、組み立ての `\x` 対応 3、話者名 1〜2、行 ID と声 2、ランダムトークと時報の停止 1、起動の場面 1、次の場面へ 2、完全一致の切り替え 1、途中の `＞yield` 1、待ちと改行を挟まない 1、マニュアル 2、テスト 2）。完全一致の部品の分は減ったが、20 を超えうる。
- **先に要るもの**: `cue-command-runtime`（同じ 3 か所を先に触る）、`novel-areka-contract`（名前）。ファイルの重なり: `cue-command-runtime`（`act.lua`・`sakura_builder.lua`・`loader/config/`、生成で埋めるなら `scope_gen.rs`）、`novel-checkpoint-rollback`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`scene-anchor-link`（`act.lua`・`sakura_builder.lua`・`sakura_script/`）、`call-attribute-filter`（`act.lua`・`scene.lua`、生成で埋めるなら `element_gen.rs`）、`tutorial-ssp-gaps`（`virtual_dispatcher.lua`・`boot.lua`）、`shell-element-click`（`init.lua` の読み込みの行・`scene.lua` に足す場合）、検査を足すなら `pasta-check-validate-edges`（`validate/`）。`scene-attribute-store` とは `scene.lua` か `scope_gen.rs` を触る場合だけ重なる。
- **種別**: 機能（ノベル向けの出力の切り替えが無い）。開発者の 0.5.0 の目標（2026-10-10）から。
- **要件定義のモデル**: Fable。決めるのは、areka が既読と声の引き当てのためにセーブをまたいで覚える「変わらない行 ID」の作り方と、ノベル向けの切り替えの形（後から変えにくい保存の形と設定の約束）。ID の作り方、2 段のクリック待ちと入れない行の書き方、「次の場面へ」の書き方と印、途中の `＞yield` の扱いの 4 つが互いに絡み、推奨案がまだ無い。
- **分割の案**: 要件で 20 を超えたら、決めてあるとおり 2 本に分ける。`novel-talk-output`（出力の組み立て: 切り替えの設定・クリック待ち・`\x` への対応・話者名・行 ID と声・待ちと改行を挟まない。`loader/config/`・`sakura_builder.lua`・`appearance.lua`・`sakura_script/`・生成）と `novel-scene-flow`（場面の流れ: ランダムトークと時報の停止・起動の場面・次の場面へ・完全一致の切り替え・途中の `＞yield`。`virtual_dispatcher.lua`・`boot.lua`・新しい受け口・`act.lua`・`scene.lua`）。切り替えの設定は前の 1 本に置き、後の 1 本はそれを読む。後の 1 本は `tutorial-ssp-gaps` と同じ 2 ファイルを触る。
- **見つけた穴・古くなった記述**:
  - 行番号: 組み立てのアクターの切り替えは `sakura_builder.lua` 240 行 → 310 行、段落の区切りの改行は 245・263〜265 行 → 346〜347 行、`\e` は 284 行 → 379 行。前方一致の探し方は `act.lua` 366〜370 行 → `find_act_handler` 361〜402 行、`scene_table.rs` 185・270〜314 行 → 179 行・319〜350 行。止まっている場面を捨てる所は `shiori/event/init.lua` 124〜132 行 → 127〜134 行。Reference を変数へ移す関数は `shiori/act.lua` 161〜190 行 → 165〜194 行。`appearance.lua`・`virtual_dispatcher.lua`・`kick.lua`・`line_breaker.rs` の行番号は合っている。

## 申し送り（pasta-check-validate-edges より）

- `pasta_check validate` の飛び先の検査は、辞書の中身が分からないとき（構文エラー・読めない辞書ファイル・pasta.toml の誤り）に丸ごと省く形になった。判定は `crates/pasta_check/src/validate/report.rs` の `skip_reasons`（統計の `skipReasons`）で、すべての段の報告がそろった後に 1 回だけ行う（`validate/mod.rs`）。
- 見直しが要る論点: この spec が `validate` に辞書のシーンの集まりに頼る検査を足すなら、`skip_reasons` が空のときだけ走らせるか。辞書の中身が分からないことを表す誤りの種類を足すなら、`skip_reasons` の対応に入れるか。
- 参照: `.kiro/specs/completed/pasta-check-validate-edges/design.md` の「Revalidation Triggers」と「Report（report.rs）」の省く理由の表。

## 申し送り（tutorial-ssp-gaps より、2026-10-18）

- **イベントの入口に時計が入った**: `EVENT.fire`（`shiori/event/init.lua` 215 行〜）の順が `CALLBACK.try_route` → `CLOCK.observe`（223 行。`OnPastaClock` を受けた回は 204 を返してハンドラへ進まない）→ `create_act` → `STOP.hold_handler`（234 行）→ ハンドラになった。読み込みの行に `pasta.shiori.clock` が足され（75 行）、ファイルは 267 行になった。brief に書いた `init.lua` の行番号はずれているので、要件定義で測り直す（`set_co_scene` は 100 行、ハンドラを `EVENT.drive(result, act, act)` で始める所は 239 行）。
- **時刻の読み方が変わった**: 新しいモジュール `pasta.shiori.clock` が「今」を一手に持つ。時報の判定と日時変数は `CLOCK.now`・`CLOCK.date`（SSP の仮想時刻に従う）、おしゃべりの間隔・コールバックの待ち時間・撫では `CLOCK.steady`（設定し直しなどの時刻の飛びを差し引いた時計）で数える。SSP 2.9.18 の TimeMachine は `act.req.date` 自体も仮想時刻にする。仕組みは本 spec の design.md の末尾「設計の見直し（7.2 の後・2026-10-18）」とマニュアル `lua/shiori-events.md` の「OnPastaClock」「時刻の飛び」。
- **仮想ディスパッチャの状態の意味が変わった**: `virtual_dispatcher.lua`（303 行）の状態は `next_hour_unix`（`CLOCK.now` で数えた次の正時）・`seen_epoch`（前に見た時刻の飛びの通し番号。飛ぶと「今 − 70 秒」から次の正時を決め直す）・`next_talk_time`（`CLOCK.steady` の時刻で数える）。間隔の設定の読み込みは 50〜83 行、止まっている場面を返す所は 215〜217 行に動いた。ランダムトークと時報を止める設計では、この 3 つの状態と `CLOCK.steady` の意味を前提にする（本 spec の design.md の Revalidation Triggers）。

## 申し送り（vertical-balloon-lines より、2026-10-18）

- **出力の組み立てが、吹き出しごとの列の状態を持つようになった**: `sakura_builder.lua` の `BUILDER.build` は、1 回の出力の中でスポットごとに `columns`（`{line, width, jump}`）を持ち、`talk`・`sakura_script` の変換 `SAKURA_SCRIPT.talk_to_script(actor, text, column)` に 3 つ目の引数で渡す。表情・ウェイト・`choice_timeout`・選択肢・`raw_script` は状態を動かさない。`newline` は n 列進め、`clear` は最初の列、`clear_spot` は全スポットの状態を捨てる。
- **区切りは 1 つの関数を通す**: pasta が出す区切り（段落の区切り・選択肢の前後・話者の切り替えの後の `spot_newlines`）は、局所関数 `put_break` で「出して 1 列進める」。改行を出す場所を足す・変えるときは `put_break` を通すこと（通さないと、`tests/lua_specs/sakura_builder_test.lua` の「作例・同値・不変条件」の不変条件のテストが落ちる）。
- **単独の `\` の守り**: 出力バッファ（`tracking_buffer`）が、最後の片が単位にならない `\` で終わるかを覚え、pasta が出すタグの前でエスケープの `\\` にする。新しいタグを出す所を足すときも、このバッファを通す。
- **`[ghost]` に `after_choice_newlines`（既定 1.0）が増えた**（選択肢の後の区切りの幅）。`[ghost]` のキーは 6 つになった（照合テスト・`reference/pasta-toml.md` の表・`book/AUTHORING.md` のキーの数）。
- 見直しが要る論点: この spec が `sakura_builder.lua` に改行や新しいタグを出す所を足すなら、`put_break` と `tracking_buffer` を通すか。字を出すトークンを足すなら、列の状態を `talk_to_script` に渡すか。
- 参照: `.kiro/specs/completed/vertical-balloon-lines/design.md`（SakuraBuilder の節・Revalidation Triggers・決定 7〜9）、マニュアル `book/src/internals/talk-output.md`。

## 申し送り（actor-display-style より、2026-10-18）

- **話者の切り替えで装いのタグが出るようになった**: `sakura_builder.lua` の `emit_actor_switch` は `\p[spot]` → 立ち絵の復旧（`APPEARANCE.restore`）→ 装いの出し入れ（`STYLE.switch`。`pasta/shiori/actor_style.lua`）の順に出す。装いの状態（スポットごとの `used`・`styled` と、出力全体の `any`）は、`BUILDER.build` が 1 回の出力ごとに `STYLE.new()` で作り、`emit_actor_switch` と `emit_inner_token` に渡す。
- **内側のトークンの出力は `STYLE.through` を通る**: 変換後の文字列の中で、引数の無い `\c`・`\x` の直後に話し手の装いのタグを差し込む（SSP 2.9.18 は `\c`・`\x` で文字の装飾を既定に戻すため）。最上位の `raw_script` は通さない。片を `put` と `put_raw` のどちらで足すかは、差し込む前の文字列で決める。
- **装いのあるアクターが出ない出力は 1 文字も変わらない**（`any` が偽の間、`switch` は空文字列を返し、`through` は文字列をそのまま返す）。
- 見直しが要る論点: この spec が次のどれかをするなら、装いの出し入れ（状態表と出し直し）をどう合わせるか。`emit_actor_switch` の出力順や `emit_inner_token` の形を変える。話者の切り替えの位置に新しいタグを出す。内側に `\c`・`\x` を出すトークンを足す。
- 参照: `.kiro/specs/completed/actor-display-style/design.md`（ActorStyle・BuilderHook・Revalidation Triggers）、マニュアル `book/src/internals/talk-output.md` の「装いの出し入れ」。
- 見直しが要る論点（この spec だけ）: 話者名を出すなら、装いのタグと同じ所（`emit_actor_switch` の装いの後）で出すか。話者名にも装いを効かせるか。
