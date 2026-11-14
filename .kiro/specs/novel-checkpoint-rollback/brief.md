# Brief: novel-checkpoint-rollback

`/kiro-discovery` で起票（2026-10-10・main `88c4bc0e`）。0.5.0「areka でノベルゲームが作れる」の道筋の 5 番目。

2026-10-17 の棚卸で、変数の保存の範囲の分を `novel-variable-scope` へ分けた。今の範囲は、末尾の「2026-10-17 棚卸の分割」が正である。

## Problem

市販品の体裁のノベルゲームには、バックログから前の場面へ巻き戻る機能と、セーブ・ロードがある。どちらも「過去のある時点の状態へ戻る」ことである。pasta の場面は Lua のコルーチンで走り、走りかけのコルーチンは控えられない。Ren'Py も同じ制約を持ち、公式の文書で「コルーチンは保存できない」と断っている。今の pasta には、過去の時点を指す名前も、その時点の状態の控えも無い。

## Current State

- 保存されるのは、`pasta.save` の表（DSL の `＄＊x`）1 つだけである。1 つのファイルへ、終了時と明示の呼び出しで書く。
- 場面の実行位置の保存、区切り、既読、バックログ、巻き戻しは、どれも無い。
- 選択肢が選ばれると、新しい場面が新しいコルーチンで始まる。前の場面のローカル変数は引き継がれない。つまり、台本を返し終えた時点では、走りかけのコルーチンは残らない（`＞yield` と、プロパティの読み出しの途中を除く）。

### 2026-10-10 の検証で分かったこと

- **控えに入れる物は `pasta.save` だけでは足りない**: 次の 2 つを戻さないと、出力が狂う。
  - `STORE.actor_spots` — 組み立てが書き換え、`\p[番号]` を決める（`pasta_scripts/pasta/shiori/sakura_builder.lua` 226〜231 行）。
  - `STORE.appearance` — 覚えているサーフェスと同じなら `\s[…]` を出さない（`shiori/appearance.lua` 392〜395 行）。キーに整数・空文字列、値に `false` があり、そのままでは JSON にならない（204〜209・247・254 行）。
- **`pasta.save` は表の同一性を保つ**: `act.save`・生成コードの `local save`・`virtual_dispatcher.lua` 46 行・終了時の保存が、同じ表を握っている。戻すときは、表を取り替えずに中身を入れ替える。
- **`pasta.save` には、遊びの進み具合でない設定も入っている**: `pasta_talk_interval_min/max`（`virtual_dispatcher.lua` 72〜73 行）。丸ごと戻すと、これも戻る。
- **走りかけは控えられない**: `STORE.co_scene`、`CALLBACK.pending[*].co`（待ちのコルーチンは `CALLBACK.pending` だけが持つ）。戻した後に古い `co_scene` が残ると、タイマーが再開する（`virtual_dispatcher.lua` 202〜204 行）。`set_co_scene` はファイルの中でしか呼べない（`shiori/event/init.lua` 97 行）。
- **台本そのものを取れる場所が 1 つではない**: 応答の値は 5 か所で包まれる（`shiori/event/init.lua` 231・235・237 行、`callback.lua` 113・179 行）。1 か所に絞れるのは `shiori/entry.lua` 79〜89 行だが、そこではヘッダーつきの応答全体しか見えない。
- **区切りにしてはいけない応答がある**: プロパティの読み出し（`act:get_property`）は、`\![get,property,…]` だけの台本を返す（`shiori/act.lua` 288〜296 行）。
- **届かなかった応答**: GET が 5 秒を超えると、DLL は 204 を返すが、Lua は走り続ける（`crates/pasta_shiori/src/actor/marshaling.rs` 71・213〜223 行）。宿主が受け取っていない応答を、区切りとして控えるおそれがある。
- **並べ替えの位置は戻せない**: 場面と単語の候補の並びは Rust の側にあり、Lua から読み書きできない（`crates/pasta_core/src/registry/scene_table.rs` 54・270〜314 行）。控えた台本を使う限り、害は無い。

## Desired Outcome

- 台本の行ごとに、目印の ID が載る。ID は areka にとってただの文字列で、中身は pasta だけが知る。
- 応答を返すたびに、その時点の状態と、返した台本を、番号を付けて控える。これを区切りと呼ぶ。直近の一定数を持つ。
- areka が ID を知らせてきたら、その区切りの状態へ戻し、控えた台本を目印の位置から切り出して返す。場面を走り直さない。乱数で言い回しが変わらない。
- 変数に、保存の範囲が 2 つある。セーブごと（巻き戻る）と、全セーブ共通（巻き戻らない）。
- 作者が「ここより前へは戻れない」点を置ける。

## Approach

決定（2026-10-10 の discovery・開発者の案）: **さくらスクリプトに、バックログ用の ID を埋める。巻き戻しは、areka がバックログの行の直近の ID をイベントで知らせることで起きる。**

```
\![目印,41-1]さくら：おはよう。今日は早いのね。\x
\![目印,41-2]窓の外では、蝉が鳴いていた。\x
\![目印,41-3]さくら：ねえ、放課後どうする？
（選択肢「屋上へ行く」「まっすぐ帰る」）
```

（記法とイベント名は仮。`novel-areka-contract` が決める。）

遊ぶ人がバックログで「窓の外では…」を選んだとき:

1. areka は、目印 `41-2` を通り過ぎた時点の画面と音に、演出なしで戻す。
2. areka が、ID `41-2` をイベントで知らせる（GET）。
3. pasta は、`41-2` を「区切り 41 の 2 行目」と読む。状態を 41 の控えに戻し、42 以降の控えを捨てる。控えた 41 の台本を、目印 `41-2` の位置から切り出して返す。
4. areka は、受け取った台本をいつもどおり再生する。

持ち物の分担:

| 持ち物 | pasta | areka |
| --- | --- | --- |
| ID の中身と細かさ | 決める | 文字列として運ぶ |
| 状態の控え・返した台本の控え（区切りごと） | 持つ | — |
| バックログの一覧（顔・話者・文・声・直近の ID） | — | 持つ |
| 目印を通り過ぎた時点の画面と音 | — | 持つ |

採らなかった案: 1 行ごとに pasta へ「次」を求める形（Yarn Spinner の型）。pasta が位置を知れるが、場面の途中の状態はコルーチンで、控えられない。巻き戻しに「同じ入力で最初から走り直す」仕掛けが要り、調べた範囲では、全面的に採った実例が無い（`novel-areka-contract/research-western-engines.md` の 8 節）。

要件定義で決めること。

1. **変数の保存の範囲の書き方**（最初の議題）: 今の `＄＊x` を、セーブごとにするか、全セーブ共通にするか。もう一方の書き方。今のゴーストの `＄＊x`（消えない記憶）と、どう両立させるか。
2. **区切りにする応答**: 場面が最後まで走った応答だけを区切りにする。プロパティの読み出しの途中と、`＞yield` で止まった応答を除く。届かなかった応答（5 秒超え）の扱い。
3. **控えに入れる物**: `pasta.save` のうちセーブごとの部分、`STORE.actor_spots`、`STORE.appearance`（JSON にできる形へ）。入れない物（タイマー・並べ替えの位置）。
4. **台本を取る場所**: 5 か所の包みを、1 か所に寄せるか。
5. **戻れる数**: 直近の何個を持つか。ロードの直後に、どこまで戻れるか。
6. **戻れない点**: 置き方（`！` 行の語彙か、属性か）。
7. **目印を打たない行**: タイトル画面などの「記録しない」範囲。
8. **保存できない値**: 変数へ入れられる型を絞るか。今は、1 つでも保存できない値があると、終了時の保存がまるごと失敗する（`crates/pasta_lua/src/runtime/lifecycle.rs` 40〜43・59〜61 行）。

## Scope

- **In**: 目印の ID の出力。区切りごとの状態と台本の控え。巻き戻しのイベントの処理。変数の保存の範囲。戻れない点。テスト、マニュアル、スキル references の再生成。
- **Out**: セーブとロード（スロットへの読み書き。`novel-save-slots`）。既読のための変わらない行 ID（`novel-talk-flow`）。バックログの一覧と画面（areka）。

## Boundary Candidates

- 目印の出力（`sakura_builder.lua`）。
- 区切りの倉（新しい Lua モジュール）と、応答の出口。
- 巻き戻しのイベントの入口（`shiori/event/`）。
- 変数の保存の範囲（文法・コード生成・`save.lua`・永続化）。規模が大きければ、ここを別の spec に分ける。

## Out of Boundary

- 画面と音の控えと再現（areka）。
- 場面を走り直す方式。

## Upstream / Downstream

- **Upstream**: `novel-talk-flow`（ノベル向けの出力と、場面の境目）、`novel-areka-contract`（目印とイベントの名前）。
- **Downstream**: `novel-save-slots`（控えをスロットへ書く）、`hello-novel-sample`、`manual-novel-guide`。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**: `failure-output-unification`（`act.lua`）、`.kiro/specs/completed/actor-surface-restore/`（`STORE.appearance`）、`.kiro/specs/completed/callback-resume-unification/`（応答の出口と再開）。

## areka への依頼

- 目印の ID を台本に書ける。通り過ぎた ID を、バックログの行に結び付けて覚える。
- 目印を通り過ぎた時点の画面と音を控え、ID を指定されたら、演出なしで即時に再現する。
- バックログの行が選ばれたら、その行の直近の ID を、GET のイベントで知らせる。
- 「記録しない」の印。「ここより前へは戻れない」の印。

2 つ目が、一番重い依頼である。要件が固まったら、取り決めの文書へ書き、areka のセッションへ知らせる。手順は `novel-areka-contract` の brief の「依頼の出し方」。この spec は、要件定義の途中で areka のセッションを立てて、同時に説明する値打ちがある。

## Constraints

- LuaJIT 2.1 には `coroutine.close` が無い。走りかけのコルーチンは、参照を外して GC に任せる。
- 全セーブ共通の変数は、巻き戻さない。
- ノベル向けの設定でないとき、出力と保存の挙動を変えない。
- 変数の保存の範囲は、文法に触る見込みである。マニュアルが文法の唯一の権威である。

## 2026-10-17 棚卸の分割

再測定で 21〜25 タスクになり、上限の 20 を超えたので、2 本に分けた。**この節が、この spec の今の範囲の正である**（上の節の「変数の保存の範囲」の記述は、`novel-variable-scope` へ移った）。

- **移した物**（`novel-variable-scope` の brief へ）: 「Desired Outcome」の 4 点目（変数の保存の範囲の 2 つ）、「Approach」の要件の 1 番（書き方）と 8 番（保存できない値）、「Boundary Candidates」の 4 点目、「Constraints」の 4 点目（文法に触る見込み）。`pasta.save` の表の同一性と予約の `pasta_` の名前の話（「2026-10-10 の検証で分かったこと」の 2・3 点目）は、事実として両方の brief に残す。どう分けて持つかは `novel-variable-scope` が決め、この spec はその口で中身を入れ替える。
- **In（分割の後）**: 目印の ID の出力。区切りにする応答の決まり（場面が最後まで走った応答だけ。プロパティの読み出しの途中・`＞yield`・止まるエラーと時間切れの 200・届かなかった応答を除く）。区切りごとの状態と台本の控え（控えの中身は、`novel-variable-scope` の口から取るセーブごとの変数、`STORE.actor_spots`、JSON にできる形にした `STORE.appearance`）。応答の出口を 1 か所に寄せるか。巻き戻しのイベントの受け口。戻れる数。戻れない点。記録しない範囲。テスト、マニュアル、スキル references の再生成。
- **Out（分割の後）**: 変数の保存の範囲の書き方・今の `＄＊x` との両立・保存できない値（`novel-variable-scope`）。セーブとロード（`novel-save-slots`）。既読のための変わらない行 ID（`novel-talk-flow`）。バックログの一覧と画面（areka）。
- **要件で決めること（残り）**: 上の「Approach」の 2〜7 番（区切りにする応答・控えに入れる物・台本を取る場所・戻れる数・戻れない点・記録しない行）。番号は振り直さない。
- **順番**: `novel-variable-scope` は `novel-talk-flow` を待たないので、それより先に始められる。`novel-checkpoint-rollback` は、`novel-talk-flow` と `novel-variable-scope` の両方の後に置く（控えに入れる変数がどれかは、`novel-variable-scope` が決める）。
- **規模**: `novel-variable-scope` は 8〜10 タスク。`novel-checkpoint-rollback` は 13〜15 タスク（目印 2、区切りの倉 2、出口を寄せる 1〜2、巻き戻しの受け口 2、控えの中身 2〜3、戻れない点 1、記録しない範囲 1、マニュアル 1、テスト 1）。
- **触るファイル（この spec）**: 目印の出力 `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`（383 行）、区切りの倉（新しい Lua のファイル）、応答の出口 `shiori/entry.lua`（132）・`shiori/event/init.lua`（260）・`shiori/event/callback.lua`（226）、巻き戻しの受け口（新しいファイルと、`init.lua` の読み込みの 1 行）、`shiori/appearance.lua`（567）、`shiori/event/virtual_dispatcher.lua`（288。`co_scene` を消す所）、`pasta/store.lua`（124）、マニュアルと生成スキル。`save.lua` は触らない（`novel-variable-scope` が作る口を使う）。
- **触るファイル（`novel-variable-scope`）**: 文法 `grammar.pest`・パーサ・構文木・`partial.rs`、`code_gen/element_gen.rs`、`save.lua`・`runtime/persistence.rs`・`runtime/lifecycle.rs`、`failure.lua`、辞書の検査 `transpile_rules.rs`、LSP `visit_expr.rs`、VSCode の着色、マニュアルの変数の章と入門 10 段目。表を 2 つにするなら `act.lua` と `scope_gen.rs` も。詳しくは、その brief の「測定」。
- **2 本のファイルの重なり**: 無いことを確かめた。例外は、`novel-variable-scope` が予約の `pasta_` の名前の置き場所を動かす場合の `virtual_dispatcher.lua` だけ（あちらは 46 行の設定の読み込み、こちらは 288 行）。2 本は前後に置くので、ぶつからない。
- **先に要るもの（この spec）**: `novel-talk-flow`（ノベル向けの出力と場面の境目）、`novel-areka-contract`（目印とイベントの名前）、`novel-variable-scope`（控えに入れる変数と、中身を入れ替える口）。ファイルの重なりは、上の再測定の節のうち、変数の保存の範囲の分を除いたもの（`novel-talk-flow`・`cue-command-runtime`・`scene-anchor-link`・`tutorial-ssp-gaps`・`shell-element-click`）。
- **要件定義のモデル**: 2 本とも Fable のまま。この spec は、区切りにする応答が、Rust の 5 秒の待ち・Lua のコルーチン・SHIORI の応答にまたがる時間の決まりで、areka がその形に頼るため。

## 申し送り（tutorial-ssp-gaps より、2026-10-18）

- **イベントの入口に時計が入った**: `EVENT.fire`（`shiori/event/init.lua` 215 行〜）の順が `CALLBACK.try_route` → `CLOCK.observe`（223 行。`OnPastaClock` を受けた回は 204 を返してハンドラへ進まない）→ `create_act` → `STOP.hold_handler`（234 行）→ ハンドラになった。読み込みの行に `pasta.shiori.clock` が足され（75 行）、ファイルは 267 行になった。brief に書いた `init.lua` の行番号はずれているので、要件定義で測り直す（`set_co_scene` は 100 行、ハンドラを `EVENT.drive(result, act, act)` で始める所は 239 行）。
- **時刻の読み方が変わった**: 新しいモジュール `pasta.shiori.clock` が「今」を一手に持つ。時報の判定と日時変数は `CLOCK.now`・`CLOCK.date`（SSP の仮想時刻に従う）、おしゃべりの間隔・コールバックの待ち時間・撫では `CLOCK.steady`（設定し直しなどの時刻の飛びを差し引いた時計）で数える。SSP 2.9.18 の TimeMachine は `act.req.date` 自体も仮想時刻にする。仕組みは本 spec の design.md の末尾「設計の見直し（7.2 の後・2026-10-18）」とマニュアル `lua/shiori-events.md` の「OnPastaClock」「時刻の飛び」。
- **仮想ディスパッチャの状態の意味が変わった**: `virtual_dispatcher.lua`（303 行）の状態は `next_hour_unix`（`CLOCK.now` で数えた次の正時）・`seen_epoch`（前に見た時刻の飛びの通し番号。飛ぶと「今 − 70 秒」から次の正時を決め直す）・`next_talk_time`（`CLOCK.steady` の時刻で数える）。間隔の設定の読み込みは 50〜83 行、止まっている場面を返す所は 215〜217 行に動いた。ランダムトークと時報を止める設計では、この 3 つの状態と `CLOCK.steady` の意味を前提にする（本 spec の design.md の Revalidation Triggers）。
- **応答を包む所**: `init.lua` の `RES.ok` は 244・250 行に動いた（`callback.lua` の行は変わっていない。`callback.lua` は注釈 2 行だけ変えた）。

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

## 申し送り（chain-talk-interrupt より、2026-10-23 の完了時）

- 前の申し送り（2026-10-20）の「`＊チェイントーク中断` を割り込んだシーンの台詞の前に添える」は設計で改めた。中断の台詞は反応の**代わりに**出る（優先）。中断の台詞が出た回は、割り込んだシーンが区切った残りも捨て、続きを残さない。
- 続きを捨てるのは 3 つだけになった: 台詞（字のある `talk` か `choice`）を出した割り込み・指名した再生（キックと `OnPastaScene`・`OnPastaSceneAt`。`STORE.co_preempt` の印）・エラー。台詞を出さないシーンや、台詞を出さずにコールバック待ちに入ったシーンは、待っている続きに触れない。
- 仕組み: `shiori/event/init.lua` の `set_co_scene(co, keep)`・`drive_kind(co)`・`run_interrupt_scene(act)`・`EVENT.drive`（種類を決める → act の欄 `_spoke`・`_on_speech`・`_interrupted` を整える → 再開 → 欄を外す → 台詞の印を読む）。台詞の判定は `sakura_builder.lua` の `BUILDER.speaks`、印は `shiori/act.lua` の `SHIORI_ACT_IMPL.build` が立てる。詳しくは `.kiro/specs/completed/chain-talk-interrupt/design.md` の「続きの決まり」と ContinuationRule。
- 見直しが要る論点: 「止まっている場面が捨てられる」を前提にする所は、台詞を出したシーン・指名した再生・エラーの 3 つに狭まったことを前提に測り直すこと。`STORE.co_preempt`（弱い参照の表）が `STORE` に増え、`STORE.reset` で作り直される。

## 申し送り（mlua-stdlib-removal より・2026-10-24）

- `OnPastaSetSaveVariable` の契約が変わった: 値は JSON として読まず、Reference1 の文字列を Reference2 の型（省けば `string`・`number`・`boolean`・`delete`）で読む。消すのは `delete` のときだけ（`null` では消えない）。上の `external_control.lua` の行番号は古い。`pasta.save` を外から書き換えることは変わらない。参照先: `.kiro/specs/completed/mlua-stdlib-removal/design.md` の SetSaveVariableHandler。


## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/completed/scene-condition-attributes/brief.md`。

## 申し送り（2026-10-29 の版の組み替えより）

- ノベルゲーム（Phase 13）は 0.5.0 から 0.6.0 へ順延した（開発者の決定。2026-11-14 に 0.7.0 へ送った）。この brief の「0.5.0」は、ノベルゲームの版のことで、今は 0.7.0 と読む。0.5.0 は「属性関係の整備」（`scene-attribute-store`・`scene-condition-attributes`・`scene-pick-modes`）になり、0.4.5（ponytail）の次に入る。

## 申し送り（actor-line-newlines より、2026-11-06）

- **変わった事実（生成）**: トランスパイラ（`code_gen/element_gen.rs` の `generate_line_marker`）が、アクション行と継続行の 1 行ごとに、その行の呼び出しより前に `act:line()` を 1 行出すようになった（InvalidContinuation の検査の後。ソースマップはその行の位置）。生成 Lua の行番号がずれるので、行番号の重なりに頼るテスト材料は見直すこと。`pasta_check` の act のメソッドの一覧（`validate/builtins.rs` の `ACT_METHODS`）は 28 になった。
- **変わった事実（トークン）**: `act:line()` は `[ghost] line_newlines` が 0 でないときだけ印 `_seam_next` を立てる。次の `talk`・`sakura_script` のトークンが `seams = { 1 }`（字のバイト位置。昇順・1 始まり）を持ち、続く同じ話し手のトークンをまとめるとき（`merge_consecutive_talks`）にも位置を足して保つ。`raw_script` は印を取らない。`act.lua` の局所関数 `take_seam` を参照。
- **変わった事実（出力）**: `sakura_builder.lua` の列の状態に `joinable`・`line_pending` が加わった。継ぎ目の改行（`\n[N]`、N = floor(値×100)）は、トークンの字の中の継ぎ目の位置に `join_lines` が差し込む（`put_break` は通らない）。`next_column`・`first_column` がこの状態を消し、話し手の切り替え（S1）では `break_line` が切り替え先の状態を消す。詳細は `.kiro/specs/completed/actor-line-newlines/design.md` と `book/src/internals/talk-output.md`。
- 見直しが要る論点: この spec で、字を出すトークンを足すなら `take_seam` で継ぎ目を取るか。改行・区切りを出す場所を足すなら、その後に `joinable`・`line_pending` を消すか（`next_column` を通せば消える）。行の頭の生成（`element_gen.rs`）を触るなら、`act:line()` の位置（呼び出しの前）を保つか。

## 2026-11-12 棚卸の再測定（main 2381bf52）

- **前提の変化（出力）**: `actor-line-newlines` で `sakura_builder.lua` は 557 → 690 行。列の状態に `joinable`・`line_pending` が増えたが、1 回の出力の中だけの状態なので控えない。目印は字を出さない `put` で足し、継ぎ目の改行（`join_lines` 389 行）の位置を動かさない。アクション行ごとに呼び出しの前に `act:line()` が出る（`element_gen.rs` 316 行の生成・`act.lua` 248 行）ので、行ごとの目印を入れる場所の候補になる（ID の中身は `novel-talk-flow` が決める）。
- **前提の変化（イベント・止まるエラー）**: `scene-condition-attributes` で、イベントの act は Reference と日時を `act.var` へ写してから場面を選ぶ（`shiori/event/init.lua` の `create_act` 104 行）。`act.var` は act ごとの物で控えに入れない（場面を走り直さないので害は無い）。止まるエラーに `FAILURE.raise` の記録（選択肢・`OnPastaCallFunction` の「条件に合うシーンが無い」）が加わり、`shiori/entry.lua` 101 行の `STOP.response` が同じ 200 で返す。区切りにしない「止まるエラーの 200」に、これも入る。`scene-play-locals` の Reference2 のローカル変数は `act.var` に入るだけで、控えの中身（`STORE.actor_spots`・`STORE.appearance`・セーブごとの変数）は変わらない。`store.lua`・`appearance.lua`・`callback.lua` は今回動いていない。
- **前の節から持ち越す前提**: 区切りにしない応答は、プロパティの読み出し（`shiori/act.lua` 322 行の `get_property`）・時計の読み取り（`shiori/clock.lua` 33〜34 行。返事の OnPastaClock は `init.lua` 297 行の `CLOCK.observe` で 204）・`＞yield`・止まるエラーと時間切れの 200・届かなかった応答（`crates/pasta_shiori/src/actor/marshaling.rs` 71 行の 5 秒）。待っている続きを捨てるのは、台詞（`BUILDER.speaks` 542 行）を出した割り込み・`STORE.co_preempt`（`store.lua` 76 行。印を付けるのは `scene_play.lua` 113・151 行だけ）の印の付いた指名の再生・エラーだけ。「残りを捨てた割り込み」「前の続きが残ったままの応答」「中断の台詞に差し替わった回」（`shiori/act.lua` 86〜89 行）を区切りにどう数えるかを要件で決める。1 回の出力の中だけの状態・`co_preempt`・act の欄 `_spoke`・`_on_speech`・`_interrupted`・時計のずれは控えない。行の途中から切り出すと、装い・スコープ・立ち絵が落ちる。外からの書き込み `OnPastaSetSaveVariable`（`external_control.lua` 97 行。129 行で `pasta.save` を書く）と、`shiori4-v4-host` の応答の外の台本（`SEND`＋`Script`・`OnHostComplete`）も、区切りの決まりに入れる。
- **触るファイル**: 目印の出力 `shiori/sakura_builder.lua` 690（立ち位置を書き換えるのは 595〜601 行の `spot` と `clear_spot`）、区切りの倉（新しい Lua のファイル）、応答の出口 `shiori/entry.lua` 120（`STOP.response` 101 行）・`shiori/event/init.lua` 351（`RES.ok` 328・334 行、`SHIORI/` で始まる文字列をそのまま返すのは 331 行）・`shiori/event/callback.lua` 232（`RES.ok` 121・198 行）、巻き戻しの受け口（新しいファイルと、`init.lua` の読み込みの並び 85〜91 行の後ろに 1 行）、`shiori/appearance.lua` 567、`shiori/event/virtual_dispatcher.lua` 315（止まっている場面を返す所 227〜229 行）、`pasta/store.lua` 124、マニュアルと生成スキル。`sakura_builder.lua` は 4 本が足すと 1,000 行に届きうるので、目印は別のモジュールに出す。テストは `sakura_builder_test.lua` に足さず、新しいファイルに置く。
- **規模**: 14〜16 のまま。
- **先に要るもの**: `novel-talk-flow`、`novel-variable-scope`、`novel-areka-contract`（どれも未完了）。ファイルの重なり（未完了）: `novel-talk-flow`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`cue-command-runtime`・`scene-anchor-link`（`sakura_builder.lua`）、`shell-element-click`（`init.lua` の読み込みの行）、`novel-variable-scope`（予約の名前を動かす場合の `virtual_dispatcher.lua`。前後に置くのでぶつからない）、`shiori4-v4-host`・`shiori4-v4-lua-api`（`callback.lua`・`init.lua`）、`scene-exact-match`（`init.lua`・`virtual_dispatcher.lua`）。`debug-mcp-runtime`・`scene-count` は入って外れた。
- **種別**: 機能（過去の時点を指す名前も、状態の控えも無い）。巻き戻しの方式は開発者の案（2026-10-10）。
- **要件定義のモデル**: Fable のまま。控えは `novel-save-slots` がスロットへ書く保存の形で、areka が頼る。区切りの決まりは、Rust の 5 秒の待ち・Lua のコルーチン・SHIORI の応答にまたがる時の流れの決まりで、推奨案がまだ無い。今回の 5 本は、この決まりを決めていない。
- **分割の案**: 不要（2026-10-17 に分けた後の 14〜16）。
- **見つけた穴・古くなった記述**: 「Current State」の行番号（2026-11-03 の節で直した値から、さらに動いた）: `sakura_builder.lua` 463〜471 行 → 595〜601 行、`init.lua` 110 行 → 114 行（`set_co_scene`）・324・327・330 行 → 328・331・334 行、`entry.lua` 99 行 → 101 行、`shiori/act.lua` 312〜341 行 → 322 行の `get_property`、`external_control.lua` 89・121 行 → 97・129 行。`virtual_dispatcher.lua` 62・88〜89・227〜229 行、`callback.lua` 121・198 行は合っている。並べ替えの位置は `scene_table.rs` 73 行の `cache`・299 行の `select_from_cache`（前は 64・315 行）。roadmap の `sakura_builder.lua` 557 行は 690 行。

## 申し送り（onhour-talk-recount より、2026-11-15）

- **仮想ディスパッチャの状態と口が増えた**: `virtual_dispatcher.lua`（335 行）の状態は 4 つになった（`next_hour_unix`・`seen_epoch`・`next_talk_time`・新しい `talk_suspend_count`＝最後に自動トークを出したときの `CLOCK.suspend_count()`。22〜23 行）。`check_hour` は時報の thread を返すとき（192〜196 行）に `next_talk_time` を決め直して回数を控え、`check_talk` は thread（止まっている場面 `STORE.co_scene` か新しい OnTalk）を返すとき（234〜237 行）に回数を控える。止まっている場面を返す所は 234 行に動いた。公開の関数 `wake_talk(act)`（294 行）が増え、`next_talk_time` を今にする（門は飛ばさない。`restart_timers` が 0 に戻して予約を消す）。
- **OnSysResume に既定が入った**: 新しいモジュール `event/sys_resume.lua` が `REG.OnSysResume` の既定を置く（`init.lua` 92 行で読む）。シーンが無く Reference0 が `auto` でなければ `wake_talk` を呼び、次の門の開いた OnSecondChange でスリープ明けのおしゃべりが 1 回出る。ランダムトークを止める道を `dispatch` の門の外に作ると、この予約がその道をすり抜けて出るおそれがある。止める設計は `dispatch` の門で止めるか、`wake_talk` の条件に足す（本 spec の design.md の Revalidation Triggers）。
- **触るファイルの行数**: 上の「2026-11-12 棚卸の再測定」の `virtual_dispatcher.lua` 315（止まっている場面を返す所 227〜229 行）は、335 行（234 行）に読み替える。`init.lua` の読み込みの並びは 85〜92 行（末尾に `sys_resume` が増えた）。
