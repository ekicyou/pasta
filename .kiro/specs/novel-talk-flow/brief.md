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

## 2026-10-18 棚卸の再測定（main facd6cf4）

- **前提の変化**:
  - `\x`: 「組み立ては `\x` を知らない」は半分古い。装いの出し入れ（`shiori/actor_style.lua` 70〜76 行の `STYLE.through`）は、引数の無い `\x` でスコープが `\0` に戻ることを知り、装いを出し直す。残る穴は、同じアクターが続くと `\p` を出し直さない所（`sakura_builder.lua` 464 行の `last_actor ~= actor`）、段落の区切りの状態（`spot_has_text`・`pending_break`・`spot_tail`）、吹き出しごとの列（`columns`）、立ち絵の復旧（`appearance.lua`）、budoux の数え直し（`crates/pasta_lua/src/sakura_script/line_breaker.rs` 56〜66 行の `column_move` は `\n`・`\c`・`\_l` だけを列の動きに数える）。`\c` を読む所（`sakura_builder.lua` 149〜189 行の `clear_span`・`text_parts`）で `\x` も読めば、組み立ての側は 1 か所で直せる。
  - 話者名: 1 つのスポットを複数のアクターで共有する形が正式になった（`actor-display-style`）。`\p[番号]` では話者が分からないので、話者名を載せる値打ちが上がった。出す所は、スポットが同じでも話者が替われば通る `emit_actor_switch`（72〜86 行）の装いの後が自然。表示用の名前は `[actor."名前"]` の表（`spot`・`font`・`budoux` がもうある）に置ける。ただし `font` の属性 `name` は書体の名前なので、話者名のキーと取り違えない名前にする。
  - BudouX: 「台詞のトークンごとに数え直す」は直った（`vertical-balloon-lines`。1 回の出力の中で、吹き出しごとに列を持ち越す）。応答をまたいだ持ち越しは無い。
  - ランダムトークと時報: 持ち主は今も `shiori/event/virtual_dispatcher.lua`（303 行）の `M.dispatch`（227〜271 行）。呼ぶのは `second_change.lua` の既定の OnSecondChange（23〜38 行）で、前にコールバックの時間切れ（`CALLBACK.sweep`）、後に仮想時刻の読み取り（`CLOCK.take_probe`）がある。止めるのは `dispatch` の `check_hour`・`check_talk` で、キック（`KICK.try_dispatch`）は残す。止まっている場面を返す所（215〜217 行）も `check_talk` の中なので、一緒に止まる。
  - 新しい穴: 仮想時刻の読み取り `\![get,property,OnPastaClock,…]\e`（`shiori/clock.lua` 33〜34・176〜180 行）は、トーク中でない OnSecondChange に返る。読み手が台本を読んでいる間にこれが出て、吹き出しが消えないかを確かめ、ノベル向けの設定では出さない案を考える。撫で・つつきの既定の受け口（`shiori/event/mouse.lua`）も場面を起こす。
- **触るファイル**: 設定 `loader/config/`（`mod.rs` 338・`sections.rs` 356。検査も足すなら `crates/pasta_check/src/validate/config.rs` 403）、`shiori/sakura_builder.lua` 540、`shiori/actor_style.lua` 90（`\x` の扱いを合わせるなら）、`shiori/appearance.lua` 567、`sakura_script/`（`mod.rs` 322、`\x` を列の動きに足すなら `line_breaker.rs` 1,134。本体は 289 行で、残りは中のテスト）、`virtual_dispatcher.lua` 303、`boot.lua` 22、受け口（新しいファイルと `init.lua` 267 の読み込みの 1 行）、`act.lua` 829・`scene.lua` 240、行 ID を生成で埋めるなら `scope_gen.rs` 445・`element_gen.rs` 564。
- **1,000 行の見込み**: `sakura_builder.lua` は 120〜200 行足しても届かない。ただ `BUILDER.build` は複雑度 28 なので、クリック待ちと `\x` の状態は `actor_style.lua` に倣って別のモジュールに出す案がある。`act.lua` は `cue-command-runtime` の後にこの spec が足すと 1,000 行に近づく。テストは `sakura_builder_test.lua`（3,839 行）・`virtual_dispatcher_spec.lua`（1,016 行）に足さず、新しいファイルに置く。
- **規模**: 17〜21 のまま（話者名の置き場所が見えた分と、`\x` の列・時計の読み取りの分が、ほぼ打ち消し合う）。
- **先に要るもの**: `cue-command-runtime`（`act.lua`・`sakura_builder.lua`・`loader/config/` を先に触る）、`novel-areka-contract`（名前）。ファイルの重なり: `cue-command-runtime`、`novel-checkpoint-rollback`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`scene-anchor-link`（`act.lua`・`sakura_builder.lua`・`sakura_script/`）、`call-attribute-filter`（`act.lua`・`scene.lua`、生成なら `element_gen.rs`）、`talk-grapheme-clusters`（`sakura_script/` の `line_breaker.rs`・`wait_inserter.rs`。`\x` や待ちを挟まない分を足す場合）、`novel-variable-scope`（予約の名前を動かすなら `virtual_dispatcher.lua`、生成で埋めるなら `scope_gen.rs`・`element_gen.rs`）、`shell-element-click`（`init.lua` の読み込みの行・`scene.lua`）、`scene-attribute-store`（`scene.lua`・`scope_gen.rs` を触る場合）。`tutorial-ssp-gaps` は完了したので外れた。
- **種別**: 機能（ノベル向けの出力の切り替えが無い）。開発者の 0.5.0 の目標（2026-10-10）から。
- **要件定義のモデル**: Fable のまま。決めるのは、areka が既読と声の引き当てのために全セーブ共通で覚える「変わらない行 ID」の作り方（保存の形で、後から変えにくい）。ID の細かさが、クリック待ちの単位と入れない行の書き方・「次の場面へ」の書き方と印・途中の `＞yield` の扱いと絡み、どれにも推奨案がまだ無い（3 つ以上の絡む問い）。話者名は置き場所が見えたので、深い論点から外れた。
- **分割の案**: 要件で 20 を超えたら、決めてあるとおり `novel-talk-output`（設定・クリック待ち・`\x`・話者名・行 ID と声・待ちと改行を挟まない）と `novel-scene-flow`（ランダムトークと時報の停止・起動の場面・次の場面へ・完全一致の切り替え・途中の `＞yield`）に分ける。後の 1 本は `tutorial-ssp-gaps` が完了したので、`virtual_dispatcher.lua`・`boot.lua` を誰とも取り合わない。要件定義は、前の 1 本が Fable、後の 1 本は Opus でよい（行 ID を持たず、残る問いは 2 つに絞って推奨案を付けられる）。
- **見つけた穴・古くなった記述**:
  - 「Current State」と「穴」の行番号: `\e` は 284 行 → `sakura_builder.lua` 536 行、間隔の丸めは `virtual_dispatcher.lua` 58〜70 行 → 63〜78 行、止まっている場面の再開は 202〜204 行 → 215〜217 行、アクターの切り替えは 240 行 → 464・468 行（`emit_actor_switch` は 72〜86 行）、段落の区切りは 245・263〜265 行 → 501〜503 行（`put_break`）、Reference を変数へ移す関数は `shiori/act.lua` 161〜190 行 → 171 行から。`line_breaker.rs` 129〜131 行の「トークンごとに数え直す」は直った。
  - 前回の節: `sakura_builder.lua` の 310・346〜347・379 行 → 468・501〜503・536 行。ファイルの大きさは `sakura_builder.lua` 383 → 540、`virtual_dispatcher.lua` 288 → 303、`line_breaker.rs` 462 → 1,134、`sakura_script/mod.rs` 247 → 322、`sections.rs` 344 → 356。

## 申し送り（ontalk-suppression より、2026-10-19）

- 0.4.2 の `ontalk-suppression` が、ランダムトークと時報を止めるスイッチを入れる。`GLOBAL` の `pause_auto_talk`／`resume_auto_talk`（別名 `自動トーク停止`／`自動トーク再開`）で、状態は `STORE` に持ち、`virtual_dispatcher.lua` の `dispatch` が見る。止めるのは OnTalk・OnHour・止まっている場面の続きで、キックは止めない。
- 本 spec の「ランダムトークと時報の停止」は、新しい止め方を作らず、ノベル向けの設定のときにこのスイッチを常に入れる形で済ませる案を推奨にする。ただし、ノベルの中で作者が `＞自動トーク再開` を呼んだときに、設定とどちらを勝たせるかを要件で決める。

## 申し送り（chain-talk-interrupt より、2026-10-20）

- 0.4.2 の `chain-talk-interrupt` が、止まっている場面を捨てる決まり（Current State の「止まっている場面が捨てられる」）を狭める。台詞（`talk`・`choice`）を出さなかったシーンは、止まっている場面に触れない。台詞を出したシーンが割り込んだときは今どおり捨て、作者が `＊チェイントーク中断` を書いていれば、割り込んだシーンの台詞の前に添える。
- 本 spec の要件では、ノベルの場面の続きが、この新しい決まりの上でどう守られるか（クリックや表情だけのイベントで消えないか）を読み直す。

## 申し送り（2026-10-21 の discovery「PastaDSL機能ヒアリング」より）

- 「完全一致の場面選びの切り替え」は `scene-exact-match`（2026-10-21 起票・開発者の承認）へ移した。そちらが `＞＞シーン名`（いつも完全一致）と、`pasta.toml` の `[scene] match = "exact"`（`＞`・イベントの受け口・OnTalk・時報・選択肢の飛び先を全部完全一致にする）を持つ。
- この spec のノベル向けの設定は、`match = "exact"` を入れるだけになる。規模の見積もりの「完全一致の切り替え 1」は外せる。Dependencies に `scene-exact-match` を足す。

## 申し送り（ontalk-suppression より、2026-10-22 完了時）

- 入った形: 印は `STORE.auto_talk_paused`（真偽・既定 false・保存しない）。書くのは `GLOBAL.pause_auto_talk`／`resume_auto_talk`（`shiori/entry.lua`）と `STORE.reset` だけという契約にした。ノベル向けに常に止めるなら、印を直に書かず停止の関数を呼ぶか、この契約を見直す（`.kiro/specs/completed/ontalk-suppression/design.md` の PauseFlag）。
- 止めている間に `＞チェイントーク`（`ACT_IMPL.yield`。`＞yield`・Lua の `act:yield()` も同じ所を通る）を呼ぶと、ログに警告を出して止めを解く（要件 2.8）。ノベルで常に止めたままチェイントークを使う形とは真っ向からぶつかるので、要件で扱いを決める論点になる。
- 再開すると `virtual_dispatcher.restart_timers` がランダムトークの間隔と次の正時を決め直す。止めている間に過ぎた時報は出さない。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**:
  - 自動トークを止めるスイッチが入った（`ontalk-suppression`）。印は `STORE.auto_talk_paused`（`crates/pasta_lua/pasta_scripts/pasta/store.lua`。保存しない）、止める・再開する関数は `GLOBAL.pause_auto_talk`／`resume_auto_talk`（`shiori/entry.lua` 29〜48 行。別名 `自動トーク停止`／`自動トーク再開`）。ランダムトークと時報の持ち主 `shiori/event/virtual_dispatcher.lua`（303 → 328 行）の `M.dispatch` は、キックの後・時報の前の門（263〜267 行）で止まる。止まっている場面の続きも止まり、捨てはしない。再開すると `M.restart_timers`（282 行）が間隔と次の正時を決め直す。「ノベル向けの設定ではこのスイッチを常に入れる」案は、そのまま使える。
  - ただし、スイッチの効き目はランダムトークと時報だけではない。撫で・つつきの受け口（`shiori/event/mouse.lua` 143・165 行）は、止めている間は、つつきを 9 段目だけにし、撫でを決めない。区切りの関数 `ACT_IMPL.yield`（`pasta/act.lua` 664 行）は、止めている間に呼ばれると警告を出して再開する。ノベルで常に止めるなら、この 2 つ（撫で・つつきの近道を止めてよいか、`＞yield` で再開してしまう決まりをどうするか）を要件で決める。
  - 指名したシーンを再生するイベント（`OnPastaScene`）と、完全一致の部品の広がり（`scene-play-event`）は、この spec の持ち場を変えない。完全一致の切り替えは `scene-exact-match` の設定を入れるだけ（2026-10-21 の申し送りのとおり）。
  - 出力の組み立て `sakura_builder.lua`（540）・立ち絵の復旧 `appearance.lua`（567）・文の加工 `sakura_script/`・設定の読み込み `loader/config/` は、前回から変わっていない。
- **触るファイル**: 2026-10-18 の節と同じ。行数が動いたのは `virtual_dispatcher.lua` 328、`act.lua` 837（1,000 行に近い）、`scene.lua` 266、`shiori/event/init.lua` 268（読み込みの行は 78〜82 行）。ほかは `loader/config/`（`mod.rs` 343・`sections.rs` 356）、`sakura_builder.lua` 540、`actor_style.lua` 90、`appearance.lua` 567、`sakura_script/`（`mod.rs` 322・`line_breaker.rs` 1,134。本体は 289 行で残りは中のテスト）、`boot.lua` 22、`mouse.lua` 176、生成で行 ID を埋めるなら `scope_gen.rs` 445・`element_gen.rs` 564。
- **規模**: 16〜20 タスク（完全一致の切り替え 1 を外し、止めるスイッチとの整合（撫で・つつき・`＞yield`）の 1 を足した。止める仕組みそのものは作らないので、停止の 1 は軽くなる）。
- **先に要るもの**: `novel-areka-contract`、`cue-command-runtime`、`scene-exact-match`。ファイルの重なり（前回のものに足す）:
  - 0.4.2 の残り: `chain-talk-interrupt`（`init.lua` の `set_co_scene`・`EVENT.drive` と `act.lua`）、`reg-handler-chain`（`init.lua` の `EVENT.fire`）、`external-control-events`（`init.lua` の読み込みの 1 行。受け口を新しいファイルに置くなら、後から入る側が合わせる約束で並べられる）。
  - 隙間の spec: `ponytail-lua-scripts`（`virtual_dispatcher.lua` の設定の読み・`init.lua`・`act.lua`・`scene.lua`）、`ponytail-lua-loader`（`loader/config/`）、行 ID を生成で埋めるなら `ponytail-lua-transpiler`（コード生成）。
- **種別**: 機能（ノベル向けの出力の切り替えが無い）。開発者の 0.5.0 の目標（2026-10-10）から。
- **要件定義のモデル**: Fable のまま。areka が既読と声の引き当てのために全セーブ共通で覚える「変わらない行 ID」は、後から変えにくい保存の形である。ID の細かさが、クリック待ちの単位・「次の場面へ」の書き方と印・途中の `＞yield` の扱い（今回、止めている間の `＞yield` が自動トークを再開する決まりも加わった）と絡み、推奨案がまだ無い。
- **分割の案**: 2026-10-18 の節のとおり。要件で 20 を超えたら `novel-talk-output`（Fable）と `novel-scene-flow`（Opus）に分ける。止めるスイッチの整合は後の 1 本に入る。
- **見つけた穴・古くなった記述**:
  - 2026-10-18 の節の行番号: `virtual_dispatcher.lua` 303 → 328 行（`M.dispatch` は 227 行から。止まっている場面を返す所は `check_talk` の中の 215〜217 行で、門の後ろ）、`init.lua` 267 → 268 行。`act.lua` 829 → 837、`scene.lua` 240 → 266。
  - 2026-10-22 の申し送りの「止めている間に `＞チェイントーク` を呼ぶと止めを解く」は、区切りの関数 `ACT_IMPL.yield` に入った（上の 2 点目）。

## 申し送り（talk-grapheme-clusters より、2026-10-23 完了時）

- 台詞の「1 文字」が、コードポイントから拡張書記素クラスタ（見た目の 1 文字）に変わった。`sakura_script/tokenizer.rs` の `next_cluster`（2 つ目以降のコードポイントの `\` の手前で切る）が切り出しの正で、`line_breaker.rs` もこれを呼ぶ。
- 自動ウェイトは 1 トークン（クラスタ）に 1 つ（`wait_inserter.rs` の `append_with_wait`。旧名 `append_with_per_char_wait`）。テキストがちょうど `\` のトークンの後ろには入れない。
- budoux の幅は `line_breaker.rs` の `cluster_width`（1 コードポイントは今までどおり・`\r\n` は 0・それ以外は文字列の東アジアの幅）で数え、改行はクラスタの境目にだけ入る。`line_breaker.rs` は 1,155 行、`tokenizer.rs` は 608 行になった。
- 公開 Rust API: `CharSets` の各フィールドは `HashSet<String>`、`CharSets::classify` は `&str`（クラスタ）を取る。Lua からの呼び方（`talk_to_script`・`break_lines`）は変わらない。
- 見直しが要る論点: この spec が待ち・改行・列の数え方に手を入れるなら、上の単位（クラスタ）と関数名を前提にする。参照は `.kiro/specs/completed/talk-grapheme-clusters/design.md` の LineBreaker・WaitInserter の節と `book/src/internals/talk-output.md`。

## 申し送り（chain-talk-interrupt より、2026-10-23 の完了時）

- 前の申し送り（2026-10-20）の「`＊チェイントーク中断` を割り込んだシーンの台詞の前に添える」は設計で改めた。中断の台詞は反応の**代わりに**出る（優先）。中断の台詞が出た回は、割り込んだシーンが区切った残りも捨て、続きを残さない。
- 続きを捨てるのは 3 つだけになった: 台詞（字のある `talk` か `choice`）を出した割り込み・指名した再生（キックと `OnPastaScene`・`OnPastaSceneAt`。`STORE.co_preempt` の印）・エラー。台詞を出さないシーンや、台詞を出さずにコールバック待ちに入ったシーンは、待っている続きに触れない。
- 仕組み: `shiori/event/init.lua` の `set_co_scene(co, keep)`・`drive_kind(co)`・`run_interrupt_scene(act)`・`EVENT.drive`（種類を決める → act の欄 `_spoke`・`_on_speech`・`_interrupted` を整える → 再開 → 欄を外す → 台詞の印を読む）。台詞の判定は `sakura_builder.lua` の `BUILDER.speaks`、印は `shiori/act.lua` の `SHIORI_ACT_IMPL.build` が立てる。詳しくは `.kiro/specs/completed/chain-talk-interrupt/design.md` の「続きの決まり」と ContinuationRule。
- 見直しが要る論点: 「止まっている場面が捨てられる」を前提にする所は、台詞を出したシーン・指名した再生・エラーの 3 つに狭まったことを前提に測り直すこと。`STORE.co_preempt`（弱い参照の表）が `STORE` に増え、`STORE.reset` で作り直される。

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**（前回 `8fc9bdff` から）:
  - 書記素クラスタ（`talk-grapheme-clusters`）: 字の単位が見た目の 1 文字になった（`sakura_script/tokenizer.rs` の `next_cluster` 119 行、`CharSets::classify(&str)` 95 行）。`\x` は今も列の動きに数えない（`line_breaker.rs` の `column_move` 57〜67 行は `\n`・`\c`・`\_l` だけ）。budoux で `\x` の後を最初の列に戻すなら、ここに 1 つ足すだけで済む（`\c` と同じ動き）。`line_breaker.rs` は 1,155 行だが、本体は 1〜310 行で、残りは中のテスト。テストは中に足さず、`tests/sakura_script/`（`grapheme_test.rs` が手本）に置く。
  - 「待ちと改行を挟まない」は、Rust を触らずに設定の既定で済む見込み。待ちは 50 ミリ秒以下なら `\_w` を出さず（`wait_inserter.rs` の `append_with_wait` 154 行・`flush_pending` 133 行）、アクターに budoux の幅が無ければ改行しない（`sakura_script/mod.rs` 132〜134 行）。ノベル向けの設定がこの既定を入れる形なら、`sakura_script/` に触るのは `\x` の 1 か所だけになる。
  - チェイントークの続き（`chain-talk-interrupt`）: 続きを捨てるのは、台詞（字のある `talk` か `choice`。`sakura_builder.lua` の `BUILDER.speaks` 413 行）を出した割り込み・印の付いた指名の再生（`STORE.co_preempt`）・エラーの 3 つだけになった。表情だけのイベントや台詞を出さないクリックの受け口では、ノベルの続きは消えない。逆に、台詞を出すイベント（ダブルクリックのシーン・`OnPastaCallFunction`・作者の `REG` のハンドラ）が来ると続きは捨てられ、`＊チェイントーク中断` があればその台詞が代わりに出る。ノベルの間に、どのイベントの割り込みを許すかを要件で決める。
  - 「次の場面へ」の受け口は、待っている続き（`STORE.co_scene`）をそのまま返せば、`EVENT.drive` が「再開」として扱う（`shiori/event/init.lua` の `drive_kind` 161 行。`virtual_dispatcher.lua` 215〜217 行が同じ返し方）。新しい場面を始めるなら、`kick.lua` 69 行・`scene_play.lua` 71 行と同じく `STORE.co_preempt[co] = true` を付ければ、待っている続きをはっきり捨て、中断の台詞も出ない。仕組みはもうあるので、作るのは受け口だけになる。
  - 止めている間の `＞yield` が自動トークを再開する決まり（`pasta/act.lua` 664 行の `ACT_IMPL.yield`）は変わっていない。前回の論点のまま。
- **触るファイル**: 行数が動いたのは `shiori/sakura_builder.lua` 540 → 558（`BUILDER.build` 432 行・`last_actor ~= actor` 482 行・`\e` 554 行。`emit_actor_switch` 72 行・`put_break` 278 行は動かず）と `shiori/event/init.lua` 268 → 371（読み込みの並びは 85〜90 行で、最後が `external_control`。受け口の 1 行はその後ろ）。`sakura_script/` は `\x` を列に数えるなら `line_breaker.rs` の `column_move` だけ。ほかは前回と同じ（`act.lua` 837・`scene.lua` 266・`virtual_dispatcher.lua` 328・`appearance.lua` 567・`actor_style.lua` 90・`loader/config/`（`mod.rs` 343・`sections.rs` 356）・`boot.lua` 22・`mouse.lua` 176、生成で行 ID を埋めるなら `scope_gen.rs` 445・`element_gen.rs` 564）。新しいファイルは受け口と、クリック待ちの状態を出すなら `actor_style.lua` に倣った別のモジュール。
- **規模**: 16〜20 のまま。待ちと改行の分は設定の既定で軽くなり、割り込みの扱い（どのイベントに印を付けるか・止めるか）の分が足され、打ち消し合う。
- **先に要るもの**: `novel-areka-contract`、`cue-command-runtime`、`scene-exact-match`（どれも未完了）。0.4.2 の 3 本と `talk-grapheme-clusters` は完了したので、重なりから外れた。ファイルの重なり:
  - 0.5.0 の道筋: `cue-command-runtime`（`act.lua`・`sakura_builder.lua`・`loader/config/`）、`novel-checkpoint-rollback`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`call-attribute-filter`（`act.lua`・`scene.lua`）、`scene-exact-match`（`act.lua`・`init.lua`・`virtual_dispatcher.lua`・`loader/config/`）、`shell-element-click`（`init.lua`・`act.lua`）、`novel-variable-scope`（`virtual_dispatcher.lua`、生成なら `scope_gen.rs`・`element_gen.rs`）、`scene-attribute-store`（`scene.lua`・`scope_gen.rs` を触る場合）。
  - 版の決まっていないもの: `scene-anchor-link`（`act.lua`・`sakura_builder.lua`）、`recommend-menu-scenes`（`init.lua` の読み込みの行・`loader/config/mod.rs`）。
  - 隙間の spec: `ponytail-lua-scripts`（`virtual_dispatcher.lua`・`init.lua`・`act.lua`・`scene.lua`）、`ponytail-lua-loader`（`loader/config/`）、生成で行 ID を埋めるなら `ponytail-lua-transpiler`。
- **種別**: 機能（ノベル向けの出力の切り替えが無い）。開発者の 0.5.0 の目標（2026-10-10）から。
- **要件定義のモデル**: Fable のまま（分けるなら出力の分だけ）。決めるのは、areka が既読と声のために全セーブ共通で覚える「変わらない行 ID」の作り方で、後から変えにくい保存の形である。ID の細かさが、クリック待ちの単位・「次の場面へ」の書き方と印・途中の `＞yield` の扱いと絡み、推奨案がまだ無い（3 つ以上の絡む問い）。今回入った 4 本は、どれもこの ID を決めていない。割り込みの扱いは `co_preempt` の印という部品ができ、推奨案が立つので、深い論点に入れない。
- **分割の案**: 2026-10-18 の節のとおり。20 を超えたら `novel-talk-output`（Fable。設定・クリック待ち・`\x`・話者名・行 ID と声・待ちと改行）と `novel-scene-flow`（Opus。ランダムトークと時報の停止・起動の場面・次の場面へ・途中の `＞yield`・割り込みの扱い）に分ける。`init.lua`・`virtual_dispatcher.lua`・`mouse.lua` は後の 1 本だけが持つ。
- **見つけた穴・古くなった記述**:
  - 前回の節の `init.lua` 268 行・読み込みの行 78〜82 行 → 371 行・85〜90 行。2026-10-18 の節の `sakura_builder.lua` の `BUILDER.build` 414 行・`last_actor` 464 行・`\e` 536 行 → 432・482・554 行。
  - 2026-10-18 と前回の節の「`talk-grapheme-clusters` と `sakura_script/` で重なる」は、完了したので外れた。roadmap の Phase 13 の共有接点（369 行）と台帳の `talk-grapheme-clusters` の行（442 行）にも、同じ古い重なりが残っている。
