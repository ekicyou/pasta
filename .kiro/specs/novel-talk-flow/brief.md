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

## 申し送り（onhour-talk-recount より、2026-11-15）

- **仮想ディスパッチャの状態と口が増えた**: `virtual_dispatcher.lua`（335 行）の状態は 4 つになった（`next_hour_unix`・`seen_epoch`・`next_talk_time`・新しい `talk_suspend_count`＝最後に自動トークを出したときの `CLOCK.suspend_count()`。22〜23 行）。`check_hour` は時報の thread を返すとき（192〜196 行）に `next_talk_time` を決め直して回数を控え、`check_talk` は thread（止まっている場面 `STORE.co_scene` か新しい OnTalk）を返すとき（234〜237 行）に回数を控える。止まっている場面を返す所は 234 行に動いた。公開の関数 `wake_talk(act)`（294 行）が増え、`next_talk_time` を今にする（門は飛ばさない。`restart_timers` が 0 に戻して予約を消す）。
- **OnSysResume に既定が入った**: 新しいモジュール `event/sys_resume.lua` が `REG.OnSysResume` の既定を置く（`init.lua` 92 行で読む）。シーンが無く Reference0 が `auto` でなければ `wake_talk` を呼び、次の門の開いた OnSecondChange でスリープ明けのおしゃべりが 1 回出る。ランダムトークを止める道を `dispatch` の門の外に作ると、この予約がその道をすり抜けて出るおそれがある。止める設計は `dispatch` の門で止めるか、`wake_talk` の条件に足す（本 spec の design.md の Revalidation Triggers）。

## 決定（2026-11-19 の discovery「ノベルゲームの深掘り」。最終形）

**この節が正である。** 上の「Desired Outcome」「Approach」「Scope」と、2026-11-19 より前の申し送りのうち、この節と食い違う所は、この節で読み替える。骨子の全体は `novel-areka-contract/final-model-2026-11-19.md`（以下「骨子」）、台本を書く人の目線のまとめは `novel-areka-contract/play-script-guide.md`、議論の経緯は `novel-areka-contract/discovery-2026-11-19.md` に在る。この節は、2026-11-19 の最後の洗い直しで、決まった順に書き足してきた節を 1 つにまとめ直した物である（前の形は git の履歴に在る）。名前は、どれも仮である。

### 上の節の読み替え

| 上の節に在る物 | 今の形 |
| -------------- | ------ |
| ノベル向けの出力に切り替える設定（`pasta.toml`） | 作らない。走り方は、柱 `＆再生モード` で書く |
| 台詞ごとのクリック待ちを `\x` で入れ、組み立てが `\x` を知るようにする | アクション行ごとに、さくらスクリプトを 1 本返す。1 本の途中に `\x` は入らない |
| 「次の場面へ」の書き方 | 要らない。最後の行の `＞`（ジャンプ）が、そのまま走る |
| 起動すると、決めた場面が始まる | 作者が `＊OnBoot` に書く。pasta はタイトルを知らない |
| ランダムトークと時報の停止 | 演劇と設定のモードの間は返さない（下の「時報と雑談」） |
| 完全一致の場面選び | `scene-exact-match` が持つ（申し送りのとおり） |
| 変わらない行 ID・既読・声をファイル名で引く | 既読は、再生位置（今のシーンの名前と行の番号）で覚える。新しい ID は作らない。声をファイル名で引く仕組みは、要望が出るまで作らない（開発者の承認。骨子 4 章の D） |
| 話者名 | 取り決めの `speaker`（`novel-areka-contract`）。既定は並びの名前 |

### 再生モード（走り方）

開発者: 「うーん、やりたいことは分かるんだけど、表現が直観的では無い。＆再生モード：トーク　演劇　設定　とか。。。」「（`＞演劇開始`／`＞トーク開始` も）柱に置き換える」「明示的に『＞演劇再開』でジャンプした方が良いのでは？」

- **走り方は、柱 `＆再生モード`（仮）で書く。** 値は、トーク・演劇・設定の 3 つである。呼び出しの対（`＞演劇開始`／`＞トーク開始`）は作らない。シーンに種類は無い。

| 値 | 走り方 |
| -- | ------ |
| トーク | シーンをまとめて、さくらスクリプト 1 本で返す（今までのゴースト） |
| 演劇 | アクション行ごとに 1 本返して待つ。記録する（目印・バックログ・セーブ） |
| 設定 | アクション行ごとに 1 本返して待つ。記録しない。物語の再生位置を動かさない |

- **柱なので、柱の決まりが、そのまま効く**（`cue-command-runtime` の「柱と状況」）。上書きで、書いていなければ前のモードが続く。その行のモードは、シーンの階層（ファイル属性 → グローバル → ローカル）を引いて決まる。ロードとバックログの巻き戻しでは、その行の階層の柱が打たれるので、モードも一緒に決まる。
- ふつうのゴーストの途中から始められる。演劇のモードのシーンへ飛べば、着いた時に演劇になる。トークのモードのシーンへ飛べば、ふつうのゴーストに戻る。物語のファイルは、ファイル属性の行に `＆再生モード：演劇` を 1 行書けば済む。
- **シーンの途中では、走り方を切り替えられない。** 切り替えたい所で、シーンを分ける。途中の行の `＞` で呼んだ先でも、モードは変わらない（呼んだ先の柱は打たれない。`cue-command-runtime` の「柱と状況」）。
- **台詞を出さなかったイベントのシーンは、着いたと数えない**（柱を打たない・今のシーンを動かさない。開発者の決定。2026-11-19 の暫定設計の議論 D2）。走り方も柱なので、台詞を出さないイベントのシーンが、別の走り方のファイルに在っても、走り方は変わらない。ローカルを先に探す先（今のシーン）も、再生位置も、動かない。
- 演劇と設定のモードの間は、どのシーンも 1 行ずつ進む。作者は、どのシーンも台詞だけで書く（`\x` を書かない。開発者: 「面倒な書き方はしたくない。この書き方じゃないとダメ」）。
- シーンが最後の行まで走って `＞` が無くても、モードは変わらない。次のイベントを待つ（開発者の承認。骨子 4 章の A）。
- 作りは、柱を打つ仕組み（`cue-command-runtime`）の上に載る。pasta が意味を知る柱が 1 つ増える。文法とパーサーは触らない。

```pasta
＆再生モード：演劇

＊第一章
    さくら：むかしむかし。
    うにゅう：ほう。
    ＞エンディング

＊エンディング
    さくら：おしまい。
    ＞あとがき

＊あとがき　＆再生モード：トーク
    さくら：また読みたくなったら言ってね。

＊OnMouseClick　＆＄ｒ４＝「設定」　＆再生モード：設定
    さくら：どうする？
    ＠？セーブする「セーブ」
    ＠？ロードする「ロード」
    ＠？もどる「物語へ戻る」

    ・セーブする
        ＞スロット保存（１）
        さくら：保存したよ。
        ＞演劇再開

    ・ロードする
        ＞スロット読込（１）

    ・もどる
        ＞演劇再開
```

### 区切り

- **1 本は「前の区切りの次から、次のアクション行（と、その継続行）の末尾まで」である**（開発者: 「『次のアクション行の末尾まで』がアクション行の区切り」）。字の有無は問わない。
- ト書き・柱・アクター指定行は、直後のアクション行の 1 本に、書いた順で入る（開発者: 「ト書きは直後のアクション行を修飾する」）。
- pasta は、1 本の末尾に、待ち `\x` と、続きのイベントを運ぶ正典の `\![raise,OnPlayNext]` を付ける。`OnPlayNext` は pasta の中の名前で、ベースウェアは名前を知らない。
- pasta の中の動きは、チェイントークと同じ止まり方である。1 本ぶんを出したら止まって返し、続きのイベントで、止まった所から再開する。行の頭ごとに呼ばれる `act:line()`（`actor-line-newlines`）が、入口の第一候補である。
- **選択肢**: 選択肢の行（`＠？`）と `！select` は、直前のアクション行の 1 本に入る。その 1 本は、末尾に `\x` を付けず、選ばれるのを待つ。
- **待たない印**（開発者: 待たないのは、表情の単語が展開されるさくらスクリプトの持つ機能）: 作者は、アクター辞書の単語の中に、印（仮 `\![parallel]`）を 1 つ書く。印の在るアクション行は、末尾に `\x` を付けず、次のアクション行と同じ 1 本になる。印から単語の終わりまでが、並走の区間になる。areka へは、区間の開く印と閉じる印（仮 `\![parallel,1]`・`\![parallel,0]`）を出す。areka でないベースウェアへは、印を出さない（順に再生される）。戻る・ロード・高速再生では、区間は出さず、区間の最後の表情だけを置く。
- 後ろにアクション行が無い残り（シーンの終わりのト書きなど）は、待たない 1 本として返す。目印は付けない。
- 行の番号は、区切りの通し番号で数える（`novel-checkpoint-rollback`）。

```pasta
％教室
    ＠稲光：\![parallel]\s[12]\_w[200]\s[10]\_w[200]\s[12]

＊嵐の教室
    教室：＠稲光
    さくら：＠驚き　きゃっ！
```

areka へ返す形: `\![parallel,1]\p[0]\s[12]\_w[200]\s[10]\_w[200]\s[12]\![parallel,0]\p[1]\s[2]きゃっ！\x\![raise,OnPlayNext]`

### 分岐は 1 つの形

開発者: 「ノベルゲームの分岐はあっても、それが寄り道かどうかで処理が変わるのは問題の一般化が出来ていない兆候」「あるグローバルシーンの中でだけ発動する、OnMouseClick ローカルシーンが書ければよい」

- **再生位置は、いつも 1 つである。** 再生位置は、今のシーン（イベントか、最後の行の `＞` で、最後に着いたシーン）と、そこからの行の番号である。
- **イベントは分岐である。** クリックでも、選択肢でも、時間切れでも、イベントでシーンが選ばれたら、再生位置はそのシーンへ移る。今のトークと同じ動きである（続きは `STORE.co_scene` の 1 つで、台詞を出すシーンが選ばれたら置き換わる。台詞を出さないシーンは、続きに触れない。`chain-talk-interrupt`）。演劇の続きを、別に持たない。
- **その場面だけのイベントは、ローカルシーンで書く。** 演劇の間、イベントのシーンは、今のシーンのグローバルシーンのローカルシーン（`・OnMouseClick`）を先に探し、無ければグローバルシーン（`＊OnMouseClick`）を探す。選択肢の飛び先や `＞` が、ローカルを先に探すのと同じ順である。**この探し方は、今の pasta に無い（足す物）。** ローカルシーンの宣言行に条件を付ける書き方は、今の文法に在る。
- **前へ進む。** 元の場面を読ませ直したいときは、最後の行に `＞` を書いて、そのシーンへ飛ぶ。最後の行に `＞` が無ければ、物語はそこで止まり、次のイベントを待つ。
- **進める**: 演劇の間に届いた左クリック（Reference5＝0）で、どのシーンも拾わなかったものは、「進める」として、続きを再開し、次の行を返す。今の「`＊OnMouseClick` のシーンを探して、無ければ何も返さない」（`event/init.lua` の `no_entry`）の、最後の 1 段を替える。選択肢を待っている 1 本では、進めない（クリックは無視され、選択肢が残る）。
- **いつもの反応との書き分け**: **演劇と設定の間、イベントで選ばれるのは、`＆再生モード` に演劇か設定を書いたシーンだけである**（開発者: 「演劇の時はトークのイベントを発動させないとか」。2026-11-19 の暫定設計の議論 D3。骨子 2.3）。イベントのシーンを選ぶ時、候補のシーンの階層（ファイル属性 → グローバル → ローカル）の `＆再生モード` を見て、演劇でも設定でもない物を外す。今のシーンのローカルシーンは、書いていなくても、先に探して選ぶ。選択肢と時間切れの飛び先は、対象の外である。つつきと撫での近道・時報・ランダムトーク・`＊チェイントーク中断` は、どれも同じ決まりで選ばれなくなる（走り方ごとの場合分けを、別に作らない）。変数 `＄演劇` と、条件 `＆＄演劇＝「」` は、作らない（取り下げ）。暫定設計の「演劇と設定の間は、近道を探さない・中断を掛けない」（別々の自動の動き）も、この 1 つの決まりに替える。

```pasta
＊教室　＆曲：昼のテーマ
    さくら：気になる所を押してみて。

    ・OnMouseClick　＆＄ｒ４＝「ドア」
        さくら：鍵がかかってる。
        うにゅう：開かへんな。

    ・OnMouseClick　＆＄ｒ４＝「窓」
        さくら：もう行こうか。
        ＞廊下
```

| 押した所 | 届くイベント | 動く物 |
| -------- | ------------ | ------ |
| 台詞の箱（吹き出し） | pasta へは届かない。待ちが解けて `OnPlayNext` | ベースウェアが `\x` を解く |
| 当たり判定の在る所で、拾うシーンが在る | `OnMouseClick`（Reference4＝名前） | そのシーンへ、再生位置が移る |
| それ以外 | `OnMouseClick` | pasta が「進める」として、次の行を返す |

### 待っている行が、外から消されたとき

- 待っている間に、別のさくらスクリプト（外からの SSTP など）が再生を置き換えたときと、台詞の箱を速く 2 回押して再生が中断されたときは、末尾の `\![raise,OnPlayNext]` が実行されない。
- 毎秒のイベント（`OnSecondChange`）が届いたとき、演劇の間で、待っている行が在り、Status に talking も choosing も無ければ、pasta は、その行を出し直しの印（`\![play,again]`）付きでもう一度返す。続きは進めない。
- 中断のイベント（`OnBalloonBreak`）の受け口は作らない。中断を禁じるタグ（`\![enter,nouserbreakmode]`）は使わない。
- 前提: `\x` で待つ間は、Status に talking が立つ（SSP 2.9.34 の実機で確かめた。areka へは取り決めで頼む）。

### 時報と雑談

- 演劇と設定のモードの間は、時報とランダムトークを返さない（`virtual_dispatcher.lua` の止める門に条件を足す。`wake_talk` の予約がすり抜けないよう、門で止める）。
- SSP の実機で、最後の 1 本が終わった 0.3 秒後の `OnSecondChange` でランダムトークが出た回が在った。この決まりは要る。

### オート・時間で次へ・高速再生

- **オートと「時間で次へ」**: `\x` が自動で解けるまでの時間で表す。作者は、その行だけなら `！時間で次へ（ミリ秒）` と書く。オートの入切と時間は、pasta が覚える。areka へは `\![play,xtimeout,ミリ秒]`、SSP へは、`\x` の直前に `\![timerraise,ミリ秒,1,OnPlayNext]` を、`\x` の直後に `\![timerraise,0,1,OnPlayNext]` を打つ（SSP 2.9.34 で動いた。フェーズ 1）。
- **高速再生**: pasta が、`\_q` を付け、`\x` と瞬間の命令を出さない。入切は pasta が覚える。入力のイベントが届いたら止める。選択肢では止まる。
- **既読**（開発者の承認。骨子 4 章の D）: pasta は、読んだ行を、再生位置（今のシーンの名前と行の番号）で覚える。置き場は、スロット変数でないグローバル変数の側（全セーブ共通）。高速再生を「まだ読んでいない行で止める」のに使う。同名のシーンは、同じ名前として数える。
- 字の前に間を置くト書き（`！待つ`）は、正典の `\_w` で出す。

### SSP の実機で分かったこと（2.9.34。`novel-areka-contract/research-ssp-capabilities.md` の D 節）

- `\x` と `\![raise]` で 1 行ずつ渡す形は、今の SSP で動く。`\x` を解くのは、吹き出しのクリックだけである（シェルのクリック・Enter・スペースでは解けない）。
- 待ちの間のクリックは `OnMouseClick` として届き、返したさくらスクリプトは、待っていた行を置き換える。元の行の `\x` の後ろは実行されない。
- `\x` で待つ間、Status は talking のままである。

### 文法に足す物

- 選択肢の行に、シーンの宣言と同じ形の条件を付ける（`＠？手を振る「手を振ってみる」　＆＄＊スロット訪ねた数≧３`）。今の `choice_line` は、飛び先と表示だけである。
- `！select（２００００、時間切れ）` の 2 つ目の引数で、時間切れの飛び先を書く。
- **`！select` の時間は、ミリ秒に替える**（開発者: 「ミリ秒にそろえる。リリースにちゃんと変更を書けば『ぱすた開発』も『emo2開発』も追従してくれるはず」。骨子 2.4）。今は秒で、pasta が 1000 倍している（`sakura_builder.lua` の `choice_timeout`・`scope_gen.rs` の `generate_choice_timeout`）。この spec が、飛び先を足すのと同じ版で替える。**破壊的な変更である。** 一緒に直す物: Lua の `act:choice_timeout`（引数をミリ秒に）・入門 9 章（`book/src/getting-started/09-choice.md`）・見本ゴースト hello-pasta の `09-choice.pasta`（`!select(10)` → `!select(10000)`）・マニュアルの `！select` の記述（`grammar/`・`lua/script-api.md`・`lua/patterns.md`）・リリースノート（変更の前後の書き方を、例つきで書く）。ゴーストを作っているセッション（「ぱすた開発」・「emo2 開発」）は、リリースノートを見て追従する。小さい数を書いた時に知らせる検査は、足さない。

### 設定のモード（開発者の決定。骨子 4 章の C）

- 設定やセーブの画面のシーンには、`＆再生モード：設定` を書く。どの場面からも開ける（入口は、今の条件の属性で書いたイベントのシーン）。
- **設定のモードの場面は、物語の再生位置を動かさない。** 物語は、待っていた行で止まったまま残る。設定の行は、記録しない（目印を付けず、バックログに貯めない。`\__v[disable]` の区間で出す）。

| 設定からの出口 | 書き方 | 起きること |
| -------------- | ------ | ---------- |
| 単に戻る | `＞演劇再開` | 何も戻さない。待っていた物語の行を、出し直しの印付きで、もう一度出す（字を一度に出し、瞬間の命令は出さない） |
| ロードで戻る | `＞スロット読込` | セーブした行の控えへ戻る。変数とアクター状態が戻り、行はト書きごと再生される |
| 別の場面へ行く | `＞シーン` | 飛んだ先のモードになり、物語はそこから続く |

- 状態が元へ戻るのは、ロードとバックログの巻き戻しだけである（開発者の原則）。`＞演劇再開` は、何も戻さない。
- 設定の場面で変えた画面（メニューの絵・並び）は、物語のアクター状態に入れない。`＞演劇再開` で、物語の画面を出し直す（出し直しの印の 1 本は、画面の状態を全部書く）。音は、物語の行の階層の柱を打つ。
- 設定の場面の中の `＞スロット保存` は、待っている物語の行の控えを書く（設定の行は記録されないので、いちばん新しい控えが物語の行になる）。
- `＞演劇再開`（英字は仮 `play_resume`）は、`＞自動トーク停止` と同じ作りの、組み込みの呼び出しである。

### 要件定義へ送ること

- 遅れて届いた `OnPlayNext` の扱い。前の行のタイマーや `raise` が、別の行へ移った後に届くと、行を 1 つ飛ばす。`\![raise,OnPlayNext,ID]` のように今の行の ID を運んで、合わない物を捨てる形が候補である。行が置き換わったときに、前の行のタイマーを消すか。
- キーで進める（`OnKeyPress`）ときの、キーの決め方。
- 前の物語の控えを全部捨てる時（今までは `＞演劇開始` が受け持っていた。トークから演劇のモードに入った時、が候補）。
- 物語が待っていない時（ふつうのゴーストから設定を開いた時）の `＞演劇再開` の動き（開く前のモードへ戻る、が候補）。
- 演劇の間でも走ってほしい、いつもの辞書のシーン（`＊OnClose` など）の扱い。再生モードを書いていないので、今の決まりのままでは選ばれない。
- 字を出さないシーンがクリックを拾ったときは、「進める」にしない、でよいか。待つ場面で、外れた所を押して先へ進むのを避けたい作者は、条件なしの `・OnMouseClick` を足し、狙った反応に `＆優先` を付ける（「ぱすた開発」の台本の検証で出た）。
- ゴーストを終了したとき・切り替えたとき、演劇の状態は消える（状態はメモリの中だけに在る）。続きは、作者がセーブと `＊OnBoot` で書く。
- オート・高速再生の入切を、設定のシーンから切り替える口（組み込みの呼び出しにするか）。
- 既定の末尾（`\x\![raise,OnPlayNext]`）を `pasta.toml` で差し替えられるようにするか。
- 演劇の間に `＞チェイントーク`（`＞yield`）を呼んだときの扱い。自動トーク停止のスイッチ（`STORE.auto_talk_paused`）との関係。
- pasta の既定の反応（おすすめメニューなど）を、演劇の間に出すか。
- 高速再生で `\x` を出さないと、行が画面に出る前に次の行で置き換わるおそれ（areka の指摘。起きるなら、行の末尾に短い待ちを入れる）。
- 継続行の前にト書きを挟む書き方が、今の検査を通るか。
- ローカルシーンのイベントの中に居る間に、同じイベントがもう一度来たときの動き（同じローカルシーンが、頭からもう一度走る見込み）。途中の `＞` で呼んだ先の行が待っている間のイベントは、今のシーン（呼んだ側）のローカルシーンを探す見込み。
- 規模は、要件定義で測り直す（Reference の転記と `act:line()` が入って軽くなり、ローカルを先に探す動きと出し直しが足された）。

### モデル

- 要件定義は Opus でよい（骨子が決まったため。前の「Fable のまま」は改める）。

## 申し送り（2026-11-19 の暫定設計の照らし合わせより。持ち主の整理）

8 本の暫定設計を別々の係が書いたら、同じ仕事を 2〜3 本が別々の形で持っていた。骨子 5 章の持ち分どおりに、持ち主を 1 つにする。くわしくは `novel-areka-contract/provisional-design-issues-2026-11-19.md` の 4 章と、`provisional-cross-check-interfaces.md`。この spec の暫定の設計と検証は、このフォルダの `provisional-design.md`・`provisional-design-validation.md`（正式な設計ではない。要件定義と設計の材料にする）。

- **1 本を区切る所は、この spec の 1 か所だけにする。** 早回しの間かどうかは `novel-checkpoint-rollback` に聞き、早回しなら、組み立ての前に捨てて、数だけ進める。
- **今のシーン・区切りの通し番号・待っている 1 本の ID は、この spec がゴーストに 1 つ持つ。** `novel-checkpoint-rollback` は、読んで控えに写す。ID は 1 系列にする。
- 1 本の枠（頭・末尾・待っている物・ID）は、この spec が組む。戻る時の 1 本目（字も `\x` も無く `\![raise]` で終わる 1 本）を、枠の種類に足す。中身は `novel-checkpoint-rollback` が渡す。
- イベントのシーンを探す所は、どのイベントでも、同じ 1 つの関数を通す。
- オートと高速再生を切り替える口の形と、今の値を読む口を、要件定義で決める（見本と手引きが、別々の形で書いていた）。
- 先に入る spec は、後に入る spec の関数を呼ばない。先に入る spec が知らせの口を置き、後の spec が登録する。

## 決定（2026-11-19 の暫定設計の議論。骨子 8 章より、この spec の分）

骨子（`novel-areka-contract/final-model-2026-11-19.md`）の 8 章が正である。上の節と食い違う所は、この節が勝つ。

- 今のシーンは、登録名（同名のシーンを順に数えた内部の名前）で覚える。台本を直した後のロードは、同じ所へ戻るとは約束しない（議題 1）。
- areka では、拾わなかったクリックに何も返さない（areka が「進める」を扱う。字の途中なら、出し切るだけ）。SSP は、今のまま、pasta が受けて進める（議題 8）。
- 右クリックは、SSP では設定の入口にしない。入口は、常に出ているボタンとキーボードである。右クリックのメニューの差し替えは、pasta と areka で検討する（議題 4。`novel-areka-contract`）。
- `shell-element-click` を、この spec に入れて閉じる（議題 14）。絵の中の当たり判定のクリックを受けるシーンの探し方と、選択肢の振り分けの取り出しを、この spec が持つ。
- 骨子に足した文: 設定の間、今のシーンは、設定のシーンである（物語の再生位置は、別に止めて持つ）。演劇と設定の間は、アクション行の 1 本を返したシーンが、選ばれたシーンである（字の有無に依らない）。途中の `＞` で呼んだ先の中の `＞` と残りは、呼んだ側の続きとして数える。
- 記録の要らない反応は、設定のモードのローカルシーンで書き、`＞演劇再開` で戻る（決まりは足さない。議題 12）。前の字を残したまま続きを足す口は、作らない。

## areka の側の決定（2026-11-20。areka のセッションで、開発者が決めた（2026-11-20。areka からの連絡。全文の写しは `novel-areka-contract/areka-developer-decisions-2026-11-20.md`））

この spec に効く物（骨子 8 章の追記が正）。SSP へ出す形と、作者の書き方は、変わらない。命令の名前と引数は、どれも仮である。取り決めの要件定義で決める。

- **絵を押したら読み進める（議題 8 を改める）**: 「areka では、pasta は何も返さず、areka が進めるを扱う」は、採らない（ふつうのゴーストの動きが変わるため）。areka は、今までどおり `OnMouseClick` を投げる。pasta は、「再生を早送りする命令」を持つさくらスクリプトを、並走の 1 本として返す（仮 `\![parallel]\![play,fastforward]`）。areka は、今の行を替えずに、今の行を早送りする（字の途中なら残りを出す。待っていれば解く）。開発者: 「『特定のリージョンをマウスクリックされたら、再生を早送りするコマンドを持つさくらスクリプト』を、並走モードでなげればよい」。どこを押したら早送りかは、シェルと台本の作り（当たり判定の領域）で決まる。SSP は、今のまま。上の「決定（2026-11-19 の暫定設計の議論）」の「areka では、拾わなかったクリックに何も返さない」は、これで改める。
- **高速再生（areka では、命令で入切する）**: 仮 `\![play,fastmode,1]`・`0`。「入」の間、areka は、届いた 1 本を次々に早送りする（選択肢では止まる）。pasta は、高速再生の時も、ふつうの時と同じさくらスクリプトを返す。読んでいない行で止める時は、pasta が、その行で「切る」を打つ。開発者: 「\_qだと、もともとのスクリプトに毎回いじる必要が出ます…\_qは必要ですが、それは『高速再生のため』ではありません」。SSP へは、今までどおり `\_q` で出す。
- **続きの在る 1 本（areka では、末尾の形が替わる）**: `\x\![raise,OnPlayNext]` でなく、続きを示す命令を 1 本の末尾に置く（仮 `\![play,next,OnPlayNext]`）。areka は、そこで止まって押されるのを待ち、押されたらイベントを投げ、続きの 1 本が届くまで、話は続いていると扱う（バルーンを消さない・字も残す）。行の ID と、自動で進むまでの時間（今の `xtimeout`）は、この命令の引数にまとめられる。開発者: 「『このスクリプトには続きがあります』『クリックで進めてください』を示すキューコマンドがあった方がよい」「SSPには『続きのさくらスクリプトです』という概念自体が無い」。正典の `\x`・`\![raise]` の意味は変えない。SSP へは、今の形（`\x\![raise,…]`）で出す。areka では、物語の続きを再開するのは、この命令が投げるイベントだけになる。

## 右クリックのメニュー（2026-11-20 の決定）

- **右クリックのメニューは、差し替えない。セーブや設定のメニューは、シェルの絵（アクター）と当たり判定で出す**（開発者の決定。2026-11-20。開発者: 「右クリックメニューではなく、シェルで出すのはダメなのか？」「シェルで出す」）。設定の画面への入口は、常に出ているボタンとキーボードの 2 つである（SSP でも areka でも同じ）。右クリックは、ベースウェアのいつものメニュー（終了・ゴーストの切り替え）のままで、正典の動きを変えない。取り決めに足す物は、無い。前の決定「右クリックメニューを差し替えられるよう pasta と areka で検討する」（8 章の議題 4）は、「検討した結果、差し替えない」で閉じた。areka が挙げた 3 つの道（止める・「おすすめ」に足す・項目を答える）は、どれも採らない。この spec は、右クリックのリソース（`sakura.popupmenu.visible`）に答える口を、作らない。

## areka の側の決定 2（2026-11-20。areka のセッションで、開発者が「推奨どおり承認」した 4 件のうち、取り決めに響く物。areka からの連絡）

- **舞台の台詞の箱を押した時も、`OnMouseClick` が pasta に届く。** areka は、シェルの中の箱・舞台の箱のクリックを、今のまま SHIORI へ送る（areka が自分で使うのは、選択肢とアンカーの行だけ）。pasta は、拾わなかったクリックに、早送りの 1 本（並走）を返す。ふつうの吹き出しを押したら早送りになる動きは、areka が命令なしで足す（正典には無い動き。areka の `talk-fast-forward`）。
- 高速再生が「入」の間は、主の 1 本の中の並走の区間も、待ちを飛ばして一度に配る。先頭に並走の印の在る 1 本は、速く進めない。

## `shell-element-click` から引き継いだ物（2026-11-20）

`shell-element-click` は、この spec に入れて閉じた（roadmap の「統合して閉じたもの（2026-11-20）」）。元の brief と暫定の設計は、このフォルダの `merged-shell-element-click/` に在る（経緯として読む。決定は、骨子とこの brief が正）。

- 絵の中の当たり判定を押した時のイベント（`OnMouseClick`。Reference4 が当たり判定の名前）を受けるシーンの探し方を、この spec が持つ。今の場面のローカルシーンを先に探し、無ければグローバルを探す。条件（`＆＄ｒ４＝「ドア」`）で場面を選ぶ。
- 選択肢の振り分け（`shiori/event/choice_select.lua`）を、関数として取り出す。`scene-anchor-link` と共有する。
- シェルの側の決まり（当たり判定の名前の付け方）は、取り決め（`novel-areka-contract`）と見本（`hello-novel-sample`）に書く。

## 2026-11-20 棚卸の再測定（main 58aba27c）

- **前提の変化（決定）**: 2026-11-19 の決定で、ノベル向けの設定・行 ID・「次の場面へ」の書き方は無くなり、柱 `＆再生モード`（トーク・演劇・設定）・アクション行ごとに 1 本返す区切り・イベントのシーンをローカルから先に探す・進める・出し直し・時報と雑談の門・設定のモードと `＞演劇再開`・オートと時間で次へ・高速再生・既読・選択肢の条件と時間切れの飛び先（文法）・`！select` のミリ秒化（破壊的な変更）になった。2026-11-20 に `shell-element-click` を入れた。areka の決定（A〜C）で、1 本の末尾・高速再生・拾わなかったクリックへの返事の 3 か所を、SSP と areka で出し分ける（areka へは早送りの並走の 1 本を返す。「何も返さない」は改まった）。
- **前提の変化（ソース）**: `onhour-talk-recount` で `virtual_dispatcher.lua` は 335 行（`wake_talk` 294 行。止める門は `dispatch` に置く。上の申し送り）、`init.lua` は 352 行（読み込みの並びの末尾に `sys_resume`）。`act.lua` 857・`sakura_builder.lua` 690・`choice_select.lua` 73・`grammar.pest` 285・`element_gen.rs` 570・`scope_gen.rs` 498・`loader/config/`・`validate/config.rs` 455 は動いていない。`date-var-calendar` は `shiori/act.lua` だけを触った。
- **触るファイル**: 新しいファイル（暫定設計の `pasta/play.lua`・`pasta/shiori/event/play.lua`）、`act.lua`（`ACT_IMPL.line` 248・`choice` 339・`choice_timeout` 350・`yield` 683 行）・`scene.lua` 305・`shiori/act.lua` 386・`sakura_builder.lua` 690（`choice_timeout` 456 行）・`init.lua` 352（`EVENT.no_entry` 222 行）・`virtual_dispatcher.lua` 335・`mouse.lua` 176・`choice_select.lua` 73・`shiori/entry.lua`（組み込みの名前）、文法（`grammar.pest` の `choice_line` 236 行・`ast/scene.rs`・`parse_scene.rs`）・`scope_gen.rs`（`generate_choice`・`generate_choice_timeout` 395 行）・`element_gen.rs`（継続行の印）・`search/context.rs`（条件を量る口）・`scene_types.rs`、検査（`transpile_rules.rs`・`report.rs`・`builtins.rs`・`references.rs`・`validate_parity_test.rs`）、VSCode の文法定義、hello-pasta の `09-choice.pasta`・入門 9 章・マニュアル・リリースノート。テストは新しいファイルに置く。
- **規模**: 24〜27（暫定設計の見積もり 18〜20 に、`shell-element-click` の 3〜4、ミリ秒化の見本と入門の直し 1、areka 向けの出し分け 2〜3 が足された）。
- **先に要るもの**: `cue-command-runtime`（柱を打つ仕組みと着いた時の知らせ。分けるなら柱の分）、`novel-areka-contract`（`again` などの名前）。`scene-exact-match` は働きの上で必須ではないが、先に入れば `OnMouseClick` の受け口が `＊OnMouseClickEx` を拾わない。ファイルの重なり（未完了）: `cue-command-runtime`・`scene-exact-match`・`scene-anchor-link`（`act.lua`・文法・`choice_select.lua`・`sakura_builder.lua`・`references.rs`）、`novel-checkpoint-rollback`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`novel-save-slots`（`builtins.rs`・`shiori/entry.lua`）、`shiori4-v4-lua-api`（`init.lua`）、hello-pasta の辞書と入門の章を触る spec。
- **フェーズ**: フェーズ 1（今の SSP）＝再生モード・区切り（`\x\![raise,OnPlayNext]`）・分岐とローカルの先探し・進める・出し直し・門・設定のモード・オートと時間で次へ（`\![timerraise]`）・高速再生（`\_q`）・既読・選択肢の条件と時間切れ。フェーズ 2（SSP の対応待ち）＝話者の名前（`speaker`）。フェーズ 3（areka）＝続きの命令（仮 `next`）・高速再生の入切（仮 `fastmode`）・早送りの並走の 1 本（仮 `fastforward`）・並走の区間の印の対。SSP の形だけなら、areka を待たずに入れられる。
- **種別**: 機能（ノベルの走り方が無い）。開発者の目標（2026-10-10）。ミリ秒化は開発者の指示（2026-11-19）。
- **要件定義のモデル**: Opus（2026-11-19 の決定のまま。前回の Fable の理由だった行 ID と「次の場面へ」は無くなった）。
- **分割の案**: 前回の案（出力の分と場面の流れの分）は、ノベル向けの設定と行 ID が無くなったので使えない。暫定設計（`provisional-design.md` の 27）の案どおり、文法の分を先に出す。(1) `choice-condition-timeout`（仮。フェーズ 1。6〜8）: 選択肢の行の条件・`！select` の時間切れの飛び先・ミリ秒化（hello-pasta・入門 9 章・マニュアル・リリースノート）・検査とエディタ。走り方を待たないので `cue-command-runtime` より先にも置ける（`act.lua`・`grammar.pest`・`scope_gen.rs` を共有するので、同じウェーブには置かない）。(2) `novel-talk-flow`（フェーズ 1。16〜19）: 残り全部と `shell-element-click` の分。(3) areka 向けの出し分けの 3 か所は、`cue-command-runtime` の areka だけの出力と一緒に、フェーズ 3 の新しい spec（仮 `novel-areka-output`）へ送る。(2) が要件定義で 20 を超えたら、次に切るのはオート・高速再生・既読である。
- **見つけた穴・古くなった記述**: 「要件定義へ送ること」の「高速再生で `\x` を出さないと…行が置き換わるおそれ」は、areka では命令の入切になったので、SSP だけの話になった見込み（骨子 8 章の追記 4 の残した問い）。hello-pasta の `09-choice.pasta` 4 行目の注記「`!select(秒)`」も、ミリ秒化と一緒に直す。前回から残る: 「穴」の「場面ごとに `＞transfer_req_to_var` を書く」（`create_act` が写すので要らない）と「完全一致で探す道は `kick.lua` にしか無い」（`kick.lua` は消え、`SCENE.co_exec_exact` 284 行がある）は古い。止まっている場面を捨てる所は `init.lua` の `set_co_scene` 115 行。

## 2026-11-20 棚卸の分割

上の「2026-11-20 棚卸の再測定」で 24〜27 タスクになり、上限の 20 を超えたので、3 つに分けた。分け方は、再測定の「分割の案」（暫定設計 `provisional-design.md` の 27 の案）のとおりである。**この節が、この spec の今の範囲の正である。** 上の節の決定は、そのまま正で、持ち主だけが下のとおりに移る。

- **In（分割の後。フェーズ 1）**: 「決定（最終形）」の残り全部と、`shell-element-click` から引き継いだ分。柱 `＆再生モード`（トーク・演劇・設定）の走り方と、柱を打つ仕組みへの登録。区切り（アクション行ごとに 1 本、末尾に SSP の形 `\x\![raise,OnPlayNext]`）と 1 本の枠（頭・末尾・待っている物・ID）。分岐と、イベントのシーンをローカルから先に探すこと（当たり判定のクリックを受けるシーンの探し方を含む）。演劇と設定の間に `＆再生モード` で候補を絞る決まり。進める（SSP では、拾わなかったクリックを pasta が受けて進める）。待っている行が外から消されたときの出し直し。時報と雑談の門。設定のモードと `＞演劇再開`。オートと時間で次へ（SSP の `\![timerraise]`）。高速再生（SSP の `\_q`）。既読。待たない印 `\![parallel]` の読み取り（印の在る行を次の行と同じ 1 本にする。areka でないベースウェアへは印を出さない）。遅れて届いた `OnPlayNext` の扱い。今のシーン・区切りの通し番号・待っている 1 本の ID をゴーストに 1 つ持つこと。早回しの間かどうかを `novel-checkpoint-rollback` に聞く所。継続行の印（`act:line()` の継続行の区別）。テスト、マニュアル、スキル references の再生成。
- **口を置く物（新しい）**: 1 本の末尾・高速再生・拾わなかったクリックへの返事の 3 か所に、ベースウェアごとの形を差し替える口を置く。この spec は SSP の形で作り、areka の形は `novel-areka-output` が登録する（先に入る spec が口を置き、後の spec が登録する）。`novel-areka-output` が入るまでは、どのベースウェアへも SSP の形で出す（分割の帰結。要件定義で確かめる）。
- **移した物（`choice-condition-timeout` へ。フェーズ 1）**: 「文法に足す物」の 3 つ全部（選択肢の行の条件・`！select` の時間切れの飛び先・`！select` の時間のミリ秒化）と、それに伴う直し（`act:choice_timeout` の引数・hello-pasta の `09-choice.pasta`（4 行目の注記「`!select(秒)`」を含む）・入門 9 章・マニュアル・リリースノート）、条件を量る口（`search/context.rs`・`scene_cond.rs`）、検査（`pasta_check validate`）と LSP・VSCode の文法定義の選択肢の分。上の「Approach」の「条件つきの選択肢」の論点も、そちらで閉じる。選択肢の振り分け（`choice_select.lua`）を関数として取り出すのは、時間切れの飛び先も同じ探し方を使うので、先に入る `choice-condition-timeout` の議題にした。取り出したら、この spec はそれを使う。取り出さなかったら、今までどおり、この spec が取り出す。
- **移した物（`novel-areka-output` へ。areka だけへ出す形。フェーズ 3）**: 「areka の側の決定（2026-11-20）」の 3 つ（続きの命令（仮 `next`。行の ID と、自動で進むまでの時間を引数に持つ）・高速再生の入切（仮 `fastmode`）・拾わなかったクリックへの早送りの並走の 1 本（仮 `fastforward`））、「areka の側の決定 2（2026-11-20）」の 2 つ（舞台の台詞の箱を押した時の早送り・高速再生の間の並走の区間）、「区切り」の並走の区間の印の対（仮 `\![parallel,1]`・`\![parallel,0]`）。
- **動かしていない物**: 話者の名前（取り決めの `speaker`。フェーズ 2）。分割の案に入っていなかったので、この spec に残す。出す形は `play` の命令（`novel-areka-contract` の 2.8）なので、`novel-areka-output` へ移すかは、次の棚卸で決める。
- **順番**: `choice-condition-timeout` → `novel-talk-flow`（`act.lua`・`scope_gen.rs`・`choice_select.lua`・`sakura_builder.lua` を共有する）。この spec は、柱を打つ仕組み（`scene-stage-attributes`。`cue-command-runtime` の後）の後に置く。`novel-areka-output` は、この spec の後。
- **規模**: この spec は 16〜19（暫定設計の見積もり 18〜20 から文法の 2〜3 とミリ秒化の 1 を除き、`shell-element-click` の 3〜4 を足し、areka の出し分け 2〜3 を除いて口だけを残した）。20 を超えたら、次に切るのはオート・高速再生・既読である。`choice-condition-timeout` は 6〜8。`novel-areka-output` は 11〜14（`cue-command-runtime` の areka の分を含む）。
- **触るファイル（この spec）**: 新しいファイル（暫定設計の `pasta/play.lua`・`pasta/shiori/event/play.lua`）、`crates/pasta_lua/pasta_scripts/pasta/act.lua`（`ACT_IMPL.line` 248・`choice` 339（選択肢を待つ 1 本の印）・`call` 725・`yield` 683 行。入口だけ）・`scene.lua` 305・`store.lua`・`shiori/act.lua` 386・`shiori/sakura_builder.lua` 690（枠の頭と末尾）・`shiori/event/init.lua` 352（`EVENT.no_entry` 222 行・`EVENT.drive`）・`virtual_dispatcher.lua` 335・`second_change.lua`・`mouse.lua` 176・`choice_select.lua` 73（振り分けを使う）・`shiori/entry.lua`（組み込みの名前）、生成 `crates/pasta_lua/src/code_gen/element_gen.rs`（継続行の印）・`scope_gen.rs`（`＆再生モード` の値の検査）・`error.rs`、`crates/pasta_core/src/registry/scene_types.rs`（再生モードの名前と値）、検査 `crates/pasta_check/src/validate/transpile_rules.rs`・`report.rs`・`builtins.rs`・`crates/pasta_check/tests/validate_parity_test.rs`、マニュアルとスキル references。文法（`grammar.pest`・`ast/scene.rs`・`parse_scene.rs`）・条件を量る口・VSCode の文法定義・hello-pasta の `09-choice.pasta`・入門 9 章・リリースノートは、`choice-condition-timeout` へ移ったので触らない。テストは新しいファイルに置く。
- **触るファイル（`choice-condition-timeout`）**: 文法 `grammar.pest` の `choice_line` 236 行・`ast/scene.rs`・`parse_scene.rs`、生成 `scope_gen.rs` の `generate_choice`・`generate_choice_timeout`、実行時 `act.lua` の `choice`・`choice_timeout`・`sakura_builder.lua` 456〜458 行・`choice_select.lua`・時間切れの受け口、`search/context.rs`・`scene_cond.rs`、検査（`references.rs`・`report.rs`・一致テスト）、LSP `visit_action.rs`、VSCode の文法定義、hello-pasta・入門 9 章・マニュアル・リリースノート。詳しくは、その brief の「測定（2026-11-20）」。
- **触るファイル（`novel-areka-output`）**: この spec の 1 本の枠への areka の枝と、`cue-command-runtime` のタグへ直す所。詳しくは、その brief の「測定（2026-11-20）」。
- **要件定義のモデル**: 3 本とも Opus。

## 申し送り（line-end-wait より、2026-11-26）

- **変わった事実（トークン）**: `act:line()` は `[ghost] line_newlines` によらず、いつも印 `_seam_next` を立てる。`act:build()` の `talk`・`sakura_script` のトークンの表には `seams` の欄がいつも付く（意味とずらし方は今のまま）。上の「`line_newlines` が 0 でないときだけ印を立てる」は古い。
- **変わった事実（組み立て）**: `BUILDER.build` は、アクターのまとまりごとに `find_line_ends` を 1 回呼び、トークンの添字ごとの行末（`{ pos, author_wait }`）を `emit_inner_token` → `inner_token_to_string` → `join_lines`（位置の写し替え）→ `talk_to_script(actor, text, column, line_ends)` へ渡す。行末は継ぎ目・`choice`・まとまりの終わりで閉じる。
- **変わった事実（変換）**: `talk_to_script` に 4 つ目の引数 `line_ends` がある。省くと最後の見える字が行末になる。行末のキー（`script_wait_*_eol`）を書いたアクターだけ、行末の待ちが変わる。
- 見直しが要る論点: この spec がまとまりの切れ目・台詞の結合・トークンの種類を変える、または台詞の外に字を出すトークンを足すなら、行末の位置が変わるか（`find_line_ends` が閉じる所と読み飛ばす型）。
- 参照: `.kiro/specs/completed/line-end-wait/design.md` の「LineEndFinder」「Revalidation Triggers」、マニュアル `book/src/internals/talk-output.md` の「行末の待ち」。
- **クリック待ちとの順**: 行末の待ちは、自動の待ちと同じ `\_w[…]` として、行末の並びのすぐ後ろ（後ろのタグより前）に入る。この spec が組み立ての出力の末尾に `\x` を足すなら、行末の待ちはその前に来る（line-end-wait の要件 8.2）。line-end-wait は `\x` を足していないので、`\x` と並べたテストはこの spec で置く。
