# Brief: novel-talk-flow

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 4 番目。

## Problem

pasta の出力は、デスクトップマスコットの喋り方に合わせて作ってある。数分おきに勝手に喋り、1 回の発言は短く、読み手のクリックを待たない。ノベルゲームは逆である。物語は読み手のクリックで進み、1 つの場面が長く続き、決めた順に場面が流れる。今の pasta でノベルのさくらスクリプトを返すと、次の所で食い違う。

- 台詞ごとのクリック待ちを、作者が毎行 `\x` と書くしかない。しかも `\x` を書くと、出力の組み立てが乱れる（下の「穴」）。
- 話者名をさくらスクリプトに載せる手段が無い。さくらスクリプトに出るのはスコープの番号だけである。
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
  - さくらスクリプトに、話者名が載る。地の文（話者なし）も書ける。
  - ランダムトークと時報が起きない。
  - 起動すると、決めた場面（タイトル）が始まる。
  - 場面の名前は、完全一致で選ばれる。
- 長い一本道を、「次の場面へ」で区切れる。さくらスクリプトはそこで終わり、読み手が読み終えたら、次の場面が始まる。
- 既読のための「台本を直さない限り変わらない ID」が、行ごとにさくらスクリプトに載る。
- デスクトップマスコット向けの出力は、何も変わらない。

## Approach

要件定義で決めること。

- **切り替えの形**: pasta.toml のどこに書くか。`[package]` は、将来のエンジンのプロファイル用に空けてある（`pasta-config-restructure`）。
- **クリック待ち**: 台詞ごとに入れる。行末の待ちと改ページの待ちの 2 段にするか。入れない行の書き方。組み立てが `\x` を知るようにする（スコープ・段落の区切り・サーフェスの復旧）。
- **話者名と声**: 話者名は、アクターの名前から出す。表示用の名前を別に持つか。声は、変わらない行 ID からファイル名で引く案（Yarn Spinner・TyranoScript の流儀）。この案なら、DSL に声の記法を足さずに済む。
- **変わらない行 ID**: 場面の登録名と、その中での行の通し番号で作る案。台本を直すと番号がずれることを、どこまで許すか。
- **「次の場面へ」**: 書き方（末尾の Call をそう読むか、別の書き方か）。さくらスクリプトの終わりに、次の場面を起こす印を置く。`\e` と同じ応答に入れる必要がある。
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
- NOTIFY のイベントにはさくらスクリプトを返せない。「次の場面へ」は GET で受ける。

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

## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/completed/scene-condition-attributes/brief.md`。
- この spec への影響: 「条件つきの選択肢」の論点で確かめる先が `scene-condition-attributes` になった。あちらは選択肢の行そのものを隠さない（Out）。選択肢の飛び先の候補を選ぶときには、条件が効く。

## 申し送り（2026-10-29 の版の組み替えより）

- ノベルゲーム（Phase 13）は 0.5.0 から 0.6.0 へ順延した（開発者の決定。2026-11-14 に 0.7.0 へ送った）。この brief の「0.5.0」は、ノベルゲームの版のことで、今は 0.7.0 と読む。0.5.0 は「属性関係の整備」（`scene-attribute-store`・`scene-condition-attributes`・`scene-pick-modes`）になり、0.4.5（ponytail）の次に入る。

## 申し送り（actor-line-newlines より、2026-11-06）

- **変わった事実（生成）**: トランスパイラ（`code_gen/element_gen.rs` の `generate_line_marker`）が、アクション行と継続行の 1 行ごとに、その行の呼び出しより前に `act:line()` を 1 行出すようになった（InvalidContinuation の検査の後。ソースマップはその行の位置）。生成 Lua の行番号がずれるので、行番号の重なりに頼るテスト材料は見直すこと。`pasta_check` の act のメソッドの一覧（`validate/builtins.rs` の `ACT_METHODS`）は 28 になった。
- **変わった事実（トークン）**: `act:line()` は `[ghost] line_newlines` が 0 でないときだけ印 `_seam_next` を立てる。次の `talk`・`sakura_script` のトークンが `seams = { 1 }`（字のバイト位置。昇順・1 始まり）を持ち、続く同じ話し手のトークンをまとめるとき（`merge_consecutive_talks`）にも位置を足して保つ。`raw_script` は印を取らない。`act.lua` の局所関数 `take_seam` を参照。
- **変わった事実（出力）**: `sakura_builder.lua` の列の状態に `joinable`・`line_pending` が加わった。継ぎ目の改行（`\n[N]`、N = floor(値×100)）は、トークンの字の中の継ぎ目の位置に `join_lines` が差し込む（`put_break` は通らない）。`next_column`・`first_column` がこの状態を消し、話し手の切り替え（S1）では `break_line` が切り替え先の状態を消す。詳細は `.kiro/specs/completed/actor-line-newlines/design.md` と `book/src/internals/talk-output.md`。
- 見直しが要る論点: この spec で、字を出すトークンを足すなら `take_seam` で継ぎ目を取るか。改行・区切りを出す場所を足すなら、その後に `joinable`・`line_pending` を消すか（`next_column` を通せば消える）。行の頭の生成（`element_gen.rs`）を触るなら、`act:line()` の位置（呼び出しの前）を保つか。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化（行の頭）**: `actor-line-newlines` で、アクション行と継続行の 1 行ごとに、呼び出しの前に `act:line()` が出るようになった（`element_gen.rs` 316 行の `generate_line_marker`・`act.lua` 248 行。今は `[ghost] line_newlines` が 0 でないときだけ継ぎ目の印を立てる）。台詞の行ごとの入口が生成にできたので、「台詞ごとのクリック待ち」と「行 ID」を入れる場所の第一候補になる（生成を足さずに `act:line()` の中で済む見込み）。ID の作り方は決まっていない。`sakura_builder.lua` は 557 → 690 行（継ぎ目の改行 `join_lines` 389 行・列の状態 `joinable`・`line_pending`・`next_column` 206 行）。`\x` の後で列の状態を消すなら、`next_column` を通せば `joinable`・`line_pending` も消える（上の申し送り）。
- **前提の変化（イベントと選び方）**: `scene-condition-attributes` で、イベントの act は場面を選ぶ前に Reference と日時を `act.var` へ写す（`shiori/event/init.lua` の `create_act` 104 行）。「穴」の「場面ごとに `＞transfer_req_to_var` を書く」は要らなくなった。名前で選ぶ道（`SCENE.search(name, gsn, act)`）は条件と `＆優先`・`＆重み`・`＆順番` を見るが、完全一致の `SCENE.co_exec_exact`（`scene.lua` 284 行）は今も条件を見ない。ノベル向けに完全一致で選ぶとき、条件を効かせるかを決める（`scene-exact-match` と揃える）。`scene-play-locals` で、開発用の `OnPastaScene`・`OnPastaSceneAt` は Reference2 のローカル変数を入れてから始める。「次の場面へ」の受け口の手本は `scene_play.lua` 113・151 行（`STORE.co_preempt` の印）。`kick.lua` は消えた。
- **前の節から持ち越す前提**: `\x` を知っているのは装いの出し入れ（`actor_style.lua` の `STYLE.through`）だけ。残る穴は、同じアクターが続くと `\p` を出し直さない所（`sakura_builder.lua` 612 行の `last_actor ~= actor`）、段落の区切りの状態、吹き出しごとの列、立ち絵の復旧（`appearance.lua`）、budoux の列（`line_breaker.rs` の `column_move` 57 行に `\x` を足せば `\c` と同じ動き）。話者名は `emit_actor_switch`（72 行）の装いの後が自然で、`font` の `name`（書体の名前）と取り違えないキーにする。自動トークの止め（`STORE.auto_talk_paused`）・撫で・つつき（`mouse.lua` 143・165 行）・`＞yield`（`act.lua` 683 行）・トーク中でない OnSecondChange の時計の読み取りを、ノベルでどうするかを要件で決める。続きを捨てるのは、台詞を出した割り込み・`STORE.co_preempt` の印の付いた指名の再生・エラーだけ。
- **触るファイル**: 設定 `loader/config/`（`mod.rs` 359・`sections.rs` 324。検査を足すなら `validate/config.rs` 455）、`shiori/sakura_builder.lua` 690（`BUILDER.build` 561 行・`put_break` 423 行）、`actor_style.lua` 90、`appearance.lua` 567、`sakura_script/`（`line_breaker.rs` の `column_move` だけ）、`virtual_dispatcher.lua` 315、`boot.lua` 22、`mouse.lua` 176、受け口（新しいファイルと、`init.lua` 351 の読み込みの並び 85〜91 行の後ろに 1 行）、`act.lua` 857（`ACT_IMPL.line` 248 行）・`scene.lua` 305、行 ID を生成で埋めるなら `scope_gen.rs` 498・`element_gen.rs` 570。`act.lua`・`sakura_builder.lua` は 1,000 行に近づくので、クリック待ちの状態は `actor_style.lua` に倣った別のモジュールに出す。テストは `sakura_builder_test.lua`（3,638 行）・`virtual_dispatcher_spec.lua`（1,080 行）に足さず、新しいファイルに置く。
- **規模**: 16〜20 のまま（Reference の転記が入って 1 つ軽い。`act:line()` で生成を触らずに済めばさらに軽い）。
- **先に要るもの**: `novel-areka-contract`、`cue-command-runtime`、`scene-exact-match`（どれも未完了）。ファイルの重なり（未完了）: 0.7.0 の `cue-command-runtime`（`act.lua`・`sakura_builder.lua`・`loader/config/`・`validate/config.rs`）・`novel-checkpoint-rollback`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）・`shell-element-click`（`init.lua` の読み込みの行）・`novel-variable-scope`（`virtual_dispatcher.lua`、生成なら `scope_gen.rs`・`element_gen.rs`）、版の決まっていない `scene-anchor-link`（`act.lua`・`sakura_builder.lua`・`element_gen.rs`）・`scene-exact-match`（`act.lua`・`scene.lua`・`init.lua`・`virtual_dispatcher.lua`・`mouse.lua`・`loader/config/`）、`shiori4-v4-lua-api`（`init.lua`）。0.5.0 の 3 本と `debug-mcp-runtime` は入って外れた。
- **種別**: 機能（ノベル向けの出力の切り替えが無い）。開発者の 0.7.0 の目標（2026-10-10）から。
- **要件定義のモデル**: Fable のまま（分けるなら出力の分だけ）。areka が既読と声のために全セーブ共通で覚える「変わらない行 ID」は、後から変えにくい保存の形である。`act:line()` は ID を入れる場所の候補になったが、ID の細かさ・クリック待ちの単位・「次の場面へ」の書き方と印・途中の `＞yield` の扱いが絡み、推奨案がまだ無い（3 つ以上の絡む問い）。`.pasta` の行番号（`act.line_map`）は台本を直すとずれるので、ID の元にならない。
- **分割の案**: 20 を超えたら `novel-talk-output`（Fable。設定・クリック待ち・`\x`・話者名・行 ID と声・待ちと改行）と `novel-scene-flow`（Opus。ランダムトークと時報の停止・起動の場面・次の場面へ・途中の `＞yield`・割り込みの扱い・止めるスイッチとの整合）に分ける。切り替えの設定は前の 1 本に置き、後の 1 本はそれを読む。`init.lua`・`virtual_dispatcher.lua`・`mouse.lua` は後の 1 本だけが持つ。
- **見つけた穴・古くなった記述**: 「穴」の「イベントの Reference は場面の引数にならない。場面ごとに `＞transfer_req_to_var` を書く」は古い（上のイベントの変化）。止まっている場面を捨てる所は `init.lua` の `set_co_scene` 114 行。「完全一致で探す道は `kick.lua` にしか無い」は古い（`kick.lua` は消え、`SCENE.co_exec_exact` がある）。roadmap の Phase 13 の共有接点（`roadmap.md` 308・312 行）の `sakura_builder.lua` 557 行・`act.lua` 812 行は、690・857 行。

## 申し送り（onhour-talk-recount より、2026-11-15）

- **仮想ディスパッチャの状態と口が増えた**: `virtual_dispatcher.lua`（335 行）の状態は 4 つになった（`next_hour_unix`・`seen_epoch`・`next_talk_time`・新しい `talk_suspend_count`＝最後に自動トークを出したときの `CLOCK.suspend_count()`。22〜23 行）。`check_hour` は時報の thread を返すとき（192〜196 行）に `next_talk_time` を決め直して回数を控え、`check_talk` は thread（止まっている場面 `STORE.co_scene` か新しい OnTalk）を返すとき（234〜237 行）に回数を控える。止まっている場面を返す所は 234 行に動いた。公開の関数 `wake_talk(act)`（294 行）が増え、`next_talk_time` を今にする（門は飛ばさない。`restart_timers` が 0 に戻して予約を消す）。
- **OnSysResume に既定が入った**: 新しいモジュール `event/sys_resume.lua` が `REG.OnSysResume` の既定を置く（`init.lua` 92 行で読む）。シーンが無く Reference0 が `auto` でなければ `wake_talk` を呼び、次の門の開いた OnSecondChange でスリープ明けのおしゃべりが 1 回出る。ランダムトークを止める道を `dispatch` の門の外に作ると、この予約がその道をすり抜けて出るおそれがある。止める設計は `dispatch` の門で止めるか、`wake_talk` の条件に足す（本 spec の design.md の Revalidation Triggers）。

## 決定（2026-11-19 の discovery「ノベルゲームの深掘り」）

### トークの区切り

- **ノベルゲームモードでは、アクション行 1 つが、さくらスクリプト 1 本になる**（開発者の決定「ノベルゲームモードの時は 1 アクション行＝1 さくらスクリプトに分割されて欲しい」）。作者は区切りを書かない。継続行は、前のアクション行と同じ 1 本に入る。

```pasta
＊教室
    さくら：楽しい？
    うにゅう：それほどでもない。
```

```
応答 1: \![play,mark,OnRollback,12]\0楽しい？\x\![raise,OnPlayNext]\e
応答 2: \1それほどでもない。\x\![raise,OnPlayNext]\e
```

- **理由**: クリック待ち `\x` をさくらスクリプトの中に並べるだけでは、さくらスクリプトの再生が途切れず、pasta が区切りを知れない（開発者の指摘）。areka の「1 つのトーク」も SHIORI の応答 1 回で、`\x`・スコープの切り替え・`\c` では切れない（2026-11-19 に areka のセッションが確かめた。`crates/areka-talk/src/lib.rs` の `TalkId`・`StartTalk`・`TalkDone`）。
- **areka の側は「クリック待ちトーク」**（開発者の決定）。待ちの解き方が 3 通りあり、どれも解けた後に同じ続きのイベントを送る。pasta は再生のしかたを知らなくてよい。
  1. ふつう: クリックを待つ。
  2. オート: 末尾まで再生したら、自動で進む。
  3. 高速再生: 早送りして、すぐ進む。
- **pasta は、アクション行ごとの末尾に既定のスクリプトを自動で付ける**（discovery の推奨。開発者「台本に毎回このコードを書かされたら大変」）。既定は、待ち `\x` と、続きのイベントを運ぶ正典の `\![raise,イベント名]` である（イベントの名前は仮に `OnPlayNext`）。`pasta.toml` で差し替えられるようにする。areka がモードで待つ形（さくらスクリプトに何も付けない）は採らない見込み（固定のイベント名が要り、SSP で次が来ない）。「続きのイベント」を本物の選択肢（`OnChoiceSelect`）と同じ名前にするかは、開発者に確かめている。
- **pasta の中の動きは、チェイントークと同じ止まり方**である。アクション行 1 つ分を出したら止まってさくらスクリプトを返し、続きのイベントで止まった所から再開する。行と行の間の変数の代入とト書きは、次の行のさくらスクリプトに入る。上の「Approach」の「台詞ごとにクリック待ちを入れる。組み立てが `\x` を知るようにする」は、この形で読み替える（1 本のさくらスクリプトの途中に `\x` が入らないので、`\x` の後のスコープ・段落の区切り・立ち絵の復旧の乱れは起きにくくなる。要件で測り直す）。
- **この形で片づくもの**: 「次の場面へ」の書き方は要らない（最後の行の後で再開すると、最後の行の `＞`（ジャンプ）がそのまま走り、ジャンプ先の最初の行で止まる）。1 本のさくらスクリプトに目印は 1 つまでになり、遊ぶ人が今どのシーンかを pasta が自分で知る。変数が遊ぶ人より先へ進まない。
- **再開できる所は 2 つだけ**（開発者の決定）。止まっているシーンの続きと、シーンの頭（目印の控えから走り直す。`novel-checkpoint-rollback` の「巻き戻しの方式」）。行の途中からシーンを作り直して再開することはしない。ロードや巻き戻しの後は、シーンの頭から読み直しになる。行から再開する道（頭から走り直して、読んだ行まで黙って進める）は、乱数の引き直しで行がずれるので、要望が出るまで作らない。
- **まだ決まっていないこと**: システム画面（セーブ・設定）が割り込んだとき、止まっている続きを残すか（今のチェイントークは、台詞を出すシーンが割り込むと続きを捨てる）。自動トーク停止の間の区切り（今は警告して止めを解く）との整理。話者の名前の運び方（areka は、スコープを分け合う話者の名前をさくらスクリプトから受け取るしかない。仮 `\![play,speaker,名前]`）。バックログの 1 行をどこで切るか（areka でも未決）。

### 状態はシーンで決まる（2026-11-19。開発者の決定）

- **原則**（開発者の言葉）: 全ての画面はシーンであり、さくらスクリプトである。areka から見たら、イベント起動が付いたさくらスクリプトを延々と再生しているに過ぎない。シーンによって状態を管理する責務は、SHIORI の側にある。
- だから、「画面」「システム画面」「タイトル画面」という物を、pasta にも areka にも作らない。シーンの種類も増やさない。文法とパーサーは触らない。
- **区切りは、グローバルシーンの名前で決まる**（開発者が承認）。
  - 名前が `On` で始まるグローバルシーン（別名を畳んだ後の名前で見る）＝イベントの受け口。演劇の外。
  - それ以外のグローバルシーン＝演劇。
  - ローカルシーンは、親のグローバルシーンに従う。

| | 演劇のシーン（`On` 以外） | `On` で始まるシーン |
| --- | --- | --- |
| 行の区切り | アクション行 1 つ＝さくらスクリプト 1 本（上の「トークの区切り」） | 今までどおり |
| 目印 | シーンの頭に作る | 作らない |
| バックログ | 貯める（さくらスクリプトの頭に `\![play,backlog,1]`） | 貯めない（さくらスクリプトの頭に `\![play,backlog,0]`） |
| 続き | 演劇の続きとして持つ。`On` のシーンが走っても残る | 今のチェイントークと同じ決まり |

- **移り変わりは、今の `＞` のジャンプで書く**。新しい口を作らない。
  - `On` のシーンから演劇のシーンへ飛ぶ＝演劇を、そこから始める。「はじめから」も、ロードも、当たり判定のクリックで進む物語も、同じ形である。前に止まっていた演劇の続きは捨てる。
  - 演劇のシーンから `On` のシーンへ飛ぶ＝演劇を終える（タイトルへ戻る、など）。
  - 演劇が待っている間に、イベントで `On` のシーンが走る＝演劇の続きに触れない。そのシーンが終わると演劇へ戻り、最後に返した行を出し直す（`\![play,again]` を付ける。`novel-areka-contract` の「決定」）。
- **タイトル画面は特別な物ではない**（開発者の言葉「本質的にはタイトル画面も、色んな分岐があるシーンに過ぎない。最初のシーンということ」）。`＊OnBoot` の中に書けば演劇の外、`＊OnBoot` から `＞タイトル` へ飛べば演劇の最初のシーンになる。どちらにするかは作者が選ぶ。pasta はタイトルを知らない。
- 選択肢の先はローカルシーンなので、親に従うだけで扱いが決まる。「選択肢のイベントは、出した側の扱いを引き継ぐ」という決まりは要らない。
- 上の「まだ決まっていないこと」の「システム画面が割り込んだとき、止まっている続きを残すか」は、これで閉じた（残す）。
- **まだ決まっていないこと**:
  - 今のゴースト（`＊OnTalk` から `＞雑談ネタ` へ飛ぶ形）の動きを変えないための、ゴーストごとの入切。
  - 演劇へ戻る細部。`On` のシーンがさくらスクリプトを返して終わると、次に届くのは `OnSecondChange` で、最大 1 秒の空きができる（areka の報告）。`On` のシーンの行にも既定の末尾（続きのイベントの起動）を付けるか。
  - 途中の行の `＞`（呼び出し）で、演劇と `On` の境目をまたいだときの扱い。
  - バルーンの速い 2 連打が「中断」になり、末尾の `\![raise,…]` が出ずに物語が止まる落とし穴の避け方（areka の報告。演劇のさくらスクリプトの頭で正典の中断の抑止を打つか、`OnBalloonBreak` の返事で出し直すか）。

### 待ちの解き方の読み替え（2026-11-19）

- 上の「トークの区切り」の「待ちの解き方が 3 通り」は、`novel-areka-contract` の「待ちの解き方（2026-11-19）」で読み替える。ふつうとオートは、`\x` が自動で解けるまでの時間（仮 `\![play,xtimeout,時間]`。-1 で無限に待つ）で表す。高速再生は別の口にする（形は未決）。
- pasta が持つもの: オートの入切と時間（さくらスクリプトの頭に毎回打つ）。背景のクリック（`OnMouseClick`）を拾って、次の行を返すこと。

### 演劇は「さくらスクリプトの切り方」の状態（2026-11-19。開発者の決定。上の「状態はシーンで決まる」の名前の決まりを改める）

- 開発者の言葉: 「なぜ演劇かどうかを切り分ける必要があるの？ シーンはシーンですよね。ジャンプの仕方が違うだけで」「アクターのさくらスクリプトの切り方が変わりますね」「結局のところ状態でしかないのでは？」「演劇かどうかは動的に切り替えたい。例えば通常は普通のゴーストとして動いていて、特定の選択肢を選んだら演劇を起動するなど」。
- **演劇かどうかは、シーンの種類ではなく、さくらスクリプトの切り方の状態である**。
  - トークの状態: シーンをまとめて、さくらスクリプト 1 本で返す（今までどおり）。
  - 演劇の状態: アクション行 1 つを、さくらスクリプト 1 本にして返す（上の「トークの区切り」）。
- **切り替えは、組み込みの呼び出しの対で書く**（開発者の決定）: `＞演劇開始`（英字 `play_start`）と `＞トーク開始`（英字は仮 `talk_start`）。前例は `＞自動トーク停止`／`＞自動トーク再開`。トグル 1 つにはしない（台本を読んでも、今どちらの状態かが分からないため）。
- **上の「状態はシーンで決まる」のうち、名前の決まり（グローバルシーンの名前が `On` で始まるか）は取り下げる**。シーンに種類を作らない。`pasta.toml` の表でゴーストごとに入切する案も採らない。原則（画面は全部シーン・状態の管理は SHIORI の側・タイトルも特別でない）は、そのまま残る。
- 状態は、呼び出した流れ（シーンを走らせている Lua のコルーチン）に付く（discovery の説明。開発者はこの説明の例で対を承認した）。
  - `＞演劇開始` の後は、その流れが 1 行ずつ進む。そこから `＞` で飛んだ先・呼んだ先も、同じ流れなので同じ切り方になる。
  - イベントで新しく走る流れ（ボタンで開く設定のシーンなど）は、トークの状態から始まる。止まっている演劇の流れには触れない。その流れが終わると、演劇へ戻る（最後に返した行を出し直す）。
  - 目印を作る・バックログに貯める・止まっている続きを残す、は、演劇の状態で走っている流れに付く性質である。上の節の「演劇のシーン」は「演劇の状態で走っている流れ」、「`On` で始まるシーン」は「イベントで新しく走る、トークの状態の流れ」と読み替える。
- 書き方の例:

```pasta
    ・物語を読む
        ＞演劇開始
        ＞第一章

＊第一章
    さくら：むかしむかし。
    うにゅう：ほう。
    ＞エンディング

＊エンディング
    さくら：おしまい。
    ＞トーク開始
    さくら：また読みたくなったら言ってね。
    うにゅう：ほなな。
```

- **再生は、内部では細切れの単位で持つ**（開発者「バックログがある。結局のところ、演劇中の再生は内部的には細切れになるしかない」）。単位は 2 つ: アクション行 1 つ（areka へ返すさくらスクリプト 1 本・バックログの 1 行）と、シーンの頭（目印。戻れる場所）。コルーチンは前へ進むためだけに使う。戻るとき・ロードしたとき・起動し直したときは、コルーチンを捨てて、控えたシーンの頭から走り直す（上の「再開できる所は 2 つだけ」）。areka は、コルーチンの状態を知らない。
- **まだ決まっていないこと**:
  - `＞トーク開始` の英字の名前。
  - シーンの途中で `＞演劇開始` を呼んだときの、最初の目印の置き方（目印はシーンの頭に作る決まりなので、次のシーンの頭まで戻れる場所が無い）。
  - 演劇の流れが裏で止まっている間に走る、トークの状態のさくらスクリプトのバックログ（設定のシーンの字を貯めないために、pasta が 0 を打つ見込み）。
  - 演劇の状態の流れが最後まで走ったときの扱い（状態は流れと一緒に消える見込み）。

### 再開できる所の読み替え（2026-11-19。開発者の決定）

- 巻き戻しの単位がアクション行になった（`novel-checkpoint-rollback` の「巻き戻しの単位はアクション行」）。上の「再開できる所は 2 つだけ」は、「どの行でも」に改める。
- 演劇の今の位置は、「控えの番号と行の番号」で表す。止まっている続き（コルーチン）が在ればそれを再開し、無ければ位置から作り直す（シーンの頭から、出力を捨てて早回しする）。

### 演劇の細かい決まり（2026-11-19。開発者の承認）

discovery が議題 1〜3 として出し、開発者が推奨どおり承認した。説明に使った例の台本と順の表は、進行メモ `novel-areka-contract/discovery-2026-11-19.md` の「やり直すディスカッション」に在る。

以下の「演劇の続き」は、`＞演劇開始` を呼んだ流れ（Lua のコルーチン）のことである。「待っている」は、pasta が行を返して続きを止め、ベースウェアが `\x` で待っている状態のことである。

**待っている行が画面から消えたとき**

待っている間に、別のさくらスクリプトが再生を置き換えたとき（セーブのボタンを押して「保存したわ。」が出た）と、台詞の箱を速く 2 回押して再生が中断されたときは、末尾の `\![raise,OnPlayNext]` が実行されない。続きのイベントが届かないので、pasta の側で戻す。

- **続きを持つ**: 演劇の続きは、チェイントークの続き（`STORE.co_scene`）とは別に持つ。別のシーンが台詞を出しても捨てない。チェイントークの今の決まり（台詞を出す割り込みで続きを捨てる。`pasta/shiori/event/init.lua`）は変えない。これは速く進めるための工夫で、続きを失ったときは、位置（控えの番号と行の番号）から作り直す。
- **出し直す**: 毎秒のイベント（`OnSecondChange`）が届いたとき、演劇の続きが止まっていて、Status に talking も choosing も無ければ、今の行を `\![play,again]` 付きでもう一度返す。続きは進めない。中断の後も、この決まりで戻る。中断のイベント（`OnBalloonBreak`）の受け口は作らない。中断を禁じるタグ（`\![enter,nouserbreakmode]`）は使わない。
- **時報と雑談**: 演劇の続きが在る間は、止まっている間も、返さない（`virtual_dispatcher.lua` の止める門に条件を足す）。つつき・撫での反応は止めない。遊ぶ人の操作への返事なので、ほかの割り込みと同じ扱いにする。
- **貯めない**: 演劇の続きが在る間に、別のイベントで走ったシーンのさくらスクリプトには、pasta が `\![play,backlog,0]` を自動で付ける。
- **読ませたい台詞**: 割り込んだ台詞（「保存したわ。」）は、出終わってから最大 1 秒で、演劇の行に置き換わる。読ませたいときは、作者が行の末尾にクリック待ちを書く（`さくら：保存したわ。\x`）。pasta は自動で足さない（絵や音だけのシーンで、見えない待ちになるため）。
- **演劇を終える時**: 続きを捨てるのは 5 つ。最後まで走った・新しく `＞演劇開始` を呼んだ・ロード・巻き戻し・別の流れから `＞トーク開始` を呼んだ。演劇の続きの中で `＞トーク開始` を呼んだときは、その流れの残りを 1 本にまとめて返す。どちらも「演劇を終える」で、その後はふつうのゴーストに戻る（時報と雑談も戻る）。

```pasta
＊OnMouseClick　＆＄ｒ４＝「やめる」
    ＞トーク開始
    さくら：またね。
```

- 前提: `\x` で待つ間は、Status に talking が立つ（`novel-areka-contract` の「ベースウェアへ頼む命令の確定」）。立たないと、毎秒出し直してしまう。

**待っている間のクリック**

| 押した所 | 届くイベント | 動く物 |
| -------- | ------------ | ------ |
| 台詞の箱 | クリックは pasta へ届かない。待ちが解けて `OnPlayNext` | ベースウェアが `\x` を解く。字が出ている途中なら、字を一度に出す（areka の予定 `talk-fast-forward` のまま） |
| 当たり判定の在る所で、拾うシーンが在る | `OnMouseClick`（Reference4＝名前） | 条件の属性で書いたシーンが走る（`＊OnMouseClick　＆＄ｒ４＝「ドア」`。今の文法のまま。近道の名前は足さない） |
| それ以外 | `OnMouseClick` | pasta が「進める」として、演劇の続きを再開し、次の行を返す |

- 「進める」の見分け方: 演劇の続きが待っている間に届いた左クリック（Reference5＝0）で、どのシーンも拾わなかったもの。当たり判定の名前が付いていても、拾うシーンが無ければ進める。今の「`＊OnMouseClick` のシーンを探して、無ければ何も返さない」（`event/init.lua` の `no_entry`）の、最後の 1 段を替える。

**`＞演劇開始` の作り**

- `＞自動トーク停止` と同じ作りで足す（名前で登録した関数。`pasta/shiori/entry.lua`）。文法とパーサーは触らない。`pasta_check` の組み込みの名前の表（`validate/builtins.rs`）に足す。
- 英字の名前は `play_start` と `talk_start`。
- シーンの途中で呼んだとき、それより前に溜まった字は、次の行と一緒に 1 本で返す。呼んだ所に控えは作らない。最初の控えは、次に着いたシーンの頭である。

```pasta
＊OnBoot
    さくら：ようこそ。
    ＞演劇開始
    さくら：では始めます。
    ＞第一章
```

  返すさくらスクリプトは `\0ようこそ。では始めます。\x\![raise,OnPlayNext]`。

**確かめていないこと**

- `\x` で待つ間に talking が立つか（areka の実機と SSP）。
- 最大 1 秒の空きが、遊んで気になる長さか。
- 速い 2 回のクリックで、`OnMouseClick` が何回届くか。

### 区切りはト書きごと・高速再生は正典のタグ（2026-11-19。開発者の承認）

discovery が議題 6〜9 として出し、開発者が承認した。開発者の言葉: 「ト書きは直後のアクション行を修飾する存在と考えた方がよい」「『次のアクション行の末尾まで』がアクション行の区切り」。例と表は `novel-areka-contract/discovery-2026-11-19.md` の「議題 6〜9（組み直した形）」、書き方のまとめは `novel-areka-contract/play-script-guide.md`。

- **区切り**: 1 本のさくらスクリプトは「前の区切りの次から、字を出すアクション行（と、その継続行）の末尾まで」。間のト書き・字の無いアクション行（表情だけ）・柱・アクター指定行は、その 1 本に、書いた順で入る。上の「トークの区切り」の「アクション行 1 つがさくらスクリプト 1 本」は、こう読み替える。
- **字の無い 1 本**: シーンの終わりにト書きだけが残ったときなどは、待たない 1 本として返す。ベースウェアは、バックログに行を作らず、目印も付けない。
- **字の途中の命令**: 継続行の前に書いたト書きや、別のアクターの表情だけの行は、字の途中の、書いた位置で効く。今の検査（継続行は直前のアクション行に続く）を通るかは未確認で、要件定義で確かめる。
- **出し直し**: `again` 付きの 1 本で、画面の状態（並び・表情・残る命令の最後の値）と、待っていた行を返す。瞬間の命令は出さない。上の「演劇の細かい決まり」の「出し直す」に、画面の状態を頭に打つことを足す。
- **高速再生**: pasta が、`\_q` を付け、`\x` と瞬間の命令を出さない（正典のタグだけ）。`\![play,skip,1]` は取り下げる。`\x` を出さないと、行が画面に出る前に次の行で置き換わるおそれが在る（areka の指摘。実機は未確認）。起きるなら、行の末尾に短い待ちを入れる。
- **貯めない**: `\__v[disable]` の区間で出す（正典。SSP は、この区間をバックログに記録しない）。`\![play,backlog,0]` は取り下げる。
- **時間で次へ進む行**: ト書きで書ける（その 1 本の `\![play,xtimeout,ミリ秒]`）。字の前に間を置くト書きは、正典の `\_w` で出す。
- **文法に足す 2 件**（「ぱすた開発」のセッションの希望。この spec の要件定義で扱う）:
  - 選択肢の行に、シーンの宣言と同じ形の条件を付ける（`＠？手を振る「手を振ってみる」　＆＄＊スロット訪ねた数≧３`）。今の `choice_line` は、飛び先と表示だけである。
  - `！select（２０、時間切れ）` の 2 つ目の引数で、時間切れの飛び先を書く。
- **ゴールの分け方**（開発者「最低限実現に必要なもの → 実装が重いが後回しでもよいもの、は分けるべき」）: この spec の中身は、どれもゴール 1（最低限）に入る。表は `play-script-guide.md` の 6 章。
