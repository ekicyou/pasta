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
- **走りかけは控えられない**: `STORE.co_scene`、`CALLBACK.pending[*].co`、`STORE.co_callback`。戻した後に古い `co_scene` が残ると、タイマーが再開する（`virtual_dispatcher.lua` 202〜204 行）。`set_co_scene` はファイルの中でしか呼べない（`shiori/event/init.lua` 97 行）。
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

## 2026-10-17 棚卸の再測定（main b4164100）

- **前提の変化**（止まるエラーのバルーン表示 `runtime-error-balloon` より）:
  - エラーの応答も 200 になった。`shiori/stop.lua` の `STOP.response` が表記を載せた 200 の応答を作り、理由を `X-Error-Reason` の行に入れる（`shiori/entry.lua` 79〜101 行、`shiori/event/callback.lua` のタイムアウト 173〜187 行）。「200 なら場面が最後まで走った」とは読めないので、区切りにしない応答にこの 2 つを足す。
  - 応答の値を包む所は、`shiori/event/init.lua` の 237・241・243 行と `callback.lua` の 116・192 行に動き、上の 2 か所が増えた。1 か所に寄せるかの議題（要件の 4 番）は、増えた分も含めて決める。
  - `stop.lua` は止まった場面とハンドラの act を、要求 1 回の間だけ控える（要求の始めに `STOP.clear` で空にする）。act の `call_restore`（`act.lua` 787〜795 行）が戻す名前と行の対応も、応答をまたがない。どちらも区切りの控えに入れなくてよい。
- **触るファイル**: 目印の出力 `shiori/sakura_builder.lua` 383、区切りの倉（新しい Lua のファイル）、応答の出口 `shiori/entry.lua` 132・`shiori/event/init.lua` 260・`shiori/event/callback.lua` 226、巻き戻しの受け口（新しいファイルと `init.lua` の読み込みの 1 行）、`shiori/appearance.lua` 567、`shiori/event/virtual_dispatcher.lua` 288、`pasta/store.lua` 124・`pasta/save.lua` 13。変数の保存の範囲を入れるなら、文法 `grammar.pest` 272、変数の Lua を出す `code_gen/element_gen.rs` 564（`save.名前` を出すのは 39・71 行）、永続化 `crates/pasta_lua/src/runtime/persistence.rs` 568・`lifecycle.rs` 63、エディタ。
- **規模**: 21〜25 タスク（目印 2、区切りの倉 2、出口を寄せる 2、巻き戻しの受け口 2、控えの中身 3、戻れない点 1、記録しない範囲 1、保存できない値 1、変数の保存の範囲 4〜6、マニュアル 2、テスト 2）。上限を超える。
- **分割の案**: 2 本に分ける。`novel-variable-scope`（変数の保存の範囲: セーブごとと全セーブ共通の書き方・今の `＄＊` との両立・保存できない値。`grammar.pest`・パーサ・`element_gen.rs`・`save.lua`・`persistence.rs`・`lifecycle.rs`・エディタ・マニュアルの変数の章。8〜10 タスク）と `novel-checkpoint-rollback`（目印・区切り・巻き戻し・戻れない点・記録しない範囲・控えの中身。`sakura_builder.lua`・新しい倉・応答の出口の 3 ファイル・新しい受け口・`appearance.lua`・`virtual_dispatcher.lua`・`store.lua`。13〜15 タスク）。変数の保存の範囲は要件の最初の議題なので、`novel-variable-scope` を先に置く。2 本は触るファイルが重ならない（`save.lua` は前の 1 本だけが持ち、後の 1 本はその口を使う）。前の 1 本は `novel-talk-flow` を待たないので、もっと早く始められる。
- **先に要るもの**: `novel-talk-flow`（ノベル向けの出力と場面の境目）、`novel-areka-contract`（目印とイベントの名前）。ファイルの重なり: `novel-talk-flow`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`cue-command-runtime`・`scene-anchor-link`（`sakura_builder.lua`）、`tutorial-ssp-gaps`（`virtual_dispatcher.lua`）、`shell-element-click`（`init.lua` の読み込みの行を足す場合）。変数の保存の範囲の分は、`call-attribute-filter`・`scene-anchor-link`（`grammar.pest`・`element_gen.rs`）と `novel-save-slots`（`save.lua`・`persistence.rs`・`lifecycle.rs`）とも重なる。
- **種別**: 機能（過去の時点を指す名前も、状態の控えも無い）。巻き戻しの方式は開発者の案（2026-10-10）。
- **要件定義のモデル**: Fable（分けても 2 本とも）。変数の保存の範囲は、今のゴーストの `＄＊x` に関わる保存の形で、取り消しにくい。書き方・どちらを今の `＄＊` にするか・両立・保存できない値が絡み、推奨案が無い。巻き戻しは、どの応答を区切りにするかが、Rust の 5 秒の待ち（届かなかった応答）・Lua のコルーチン・SHIORI の応答にまたがる時間の決まりで、areka がその形に頼る。
- **見つけた穴・古くなった記述**:
  - 行番号: 組み立てが立ち位置を書き換える所は `sakura_builder.lua` 226〜231 行 → 294〜304 行、`set_co_scene` は `init.lua` 97 行 → 99 行、`shiori/entry.lua` 79〜89 行 → 79〜101 行、並べ替えの位置は `scene_table.rs` 54・270〜314 行 → 62 行・319〜350 行。`appearance.lua`・`virtual_dispatcher.lua`・`marshaling.rs`・`lifecycle.rs` の行番号は合っている。

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

## 2026-10-18 棚卸の再測定（main facd6cf4）

- **前提の変化**:
  - 区切りにしない応答が 1 つ増えた。仮想時刻の読み取り（`tutorial-ssp-gaps`）で、OnSecondChange の既定の受け口（`shiori/event/second_change.lua` 37 行）が、`\![get,property,OnPastaClock,system.year,…]\e` だけの台本（`shiori/clock.lua` 33〜34・176〜180 行）を返し、`shiori/event/init.lua` 250 行の `RES.ok` で包まれる。その返事（OnPastaClock）は `init.lua` 223〜225 行で 204 になる。プロパティの読み出しと同じく区切りにしない。
  - 応答を包む所は、`init.lua` 244・250 行（248 行は `SHIORI/` で始まる文字列をそのまま返す）、`callback.lua` 116・192 行、止まるエラーの `STOP.response`（`callback.lua` 178・183 行、`entry.lua` 91 行）。
  - 出力の組み立てに、1 回の出力の中だけの状態が 2 つ増えた（装いの状態 `STYLE.new()`・吹き出しごとの列 `columns`。`sakura_builder.lua` 426・441 行）。どちらも出力ごとに作って捨てるので、控えに入れなくてよい。控えに入れるのは今までどおり `STORE.actor_spots`（446〜450・453 行で書き換える）と `STORE.appearance`。時計（`shiori/clock.lua`）のずれと時刻の飛びも保存しない状態で、巻き戻しで戻さない。
  - 目印を `sakura_builder.lua` で出すなら、`tracking_buffer` の `put` を通し、字にも列にも数えない。話者の切り替えの位置に出すなら、`emit_actor_switch`（72〜86 行）の装いの出し入れとの順を決める。
  - 控えた台本を目印の位置から切り出すとき、切り出した頭が前の部分のタグに頼っていないかを確かめる。装いのタグ（`\f[…]`）は話者の切り替えと `\c`・`\x` の後にしか出ないので、行の途中から切ると装いが落ちる。スコープと立ち絵も同じ。
- **触るファイル**: 目印の出力 `shiori/sakura_builder.lua` 540、区切りの倉（新しい Lua のファイル）、応答の出口 `shiori/entry.lua` 132・`shiori/event/init.lua` 267・`shiori/event/callback.lua` 226、巻き戻しの受け口（新しいファイルと `init.lua` の読み込みの 1 行）、`shiori/appearance.lua` 567、`shiori/event/virtual_dispatcher.lua` 303（止まっている場面を返す所 215〜217 行）、`pasta/store.lua` 124、マニュアルと生成スキル。1,000 行を超えそうなものは無い（目印で `sakura_builder.lua` に 30〜60 行。テストは `sakura_builder_test.lua` 3,839 行に足さず、新しいファイルに置く）。
- **規模**: 13〜15 のまま（時計の読み取りを除くのは、区切りにする応答の決まりのタスクに入る）。
- **先に要るもの**: `novel-talk-flow`、`novel-variable-scope`、`novel-areka-contract`。ファイルの重なり: `novel-talk-flow`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`cue-command-runtime`・`scene-anchor-link`（`sakura_builder.lua`）、`shell-element-click`（`init.lua` の読み込みの行）、`novel-variable-scope`（予約の名前を動かす場合の `virtual_dispatcher.lua`。前後に置くのでぶつからない）。`tutorial-ssp-gaps` は完了したので外れた。`talk-grapheme-clusters`・`lua-cmodule-robustness` とは重ならない。
- **種別**: 機能（過去の時点を指す名前も、状態の控えも無い）。巻き戻しの方式は開発者の案（2026-10-10）。
- **要件定義のモデル**: Fable のまま。決めるのは、どの応答を区切りにし、何を控えるか（控えは `novel-save-slots` がスロットへ書くので、保存の形になる）。区切りの決まりは、Rust の 5 秒の待ち（届かなかった応答。`crates/pasta_shiori/src/actor/marshaling.rs` 71・213〜223 行。前回から変わっていない）・Lua のコルーチン（`＞yield`・プロパティの読み出しの途中）・SHIORI の応答（止まるエラーの 200・時計の読み取り）にまたがる時間の決まりで、areka がその形に頼る。
- **分割の案**: 不要（2026-10-17 に分けた後の 13〜15）。
- **見つけた穴・古くなった記述**:
  - 行番号（2026-10-10 の節と前回の節）: 組み立てが立ち位置を書き換える所は `sakura_builder.lua` 294〜304 行 → 446〜450 行（`spot`）と 453 行（`clear_spot`）、`pasta_talk_interval_min/max` は `virtual_dispatcher.lua` 72〜73 行 → 77〜78 行、止まっている場面を返す所は 202〜204 行 → 215〜217 行、`set_co_scene` は `init.lua` 99 行 → 100 行、プロパティの読み出しの台本は `shiori/act.lua` 288〜296 行 → 295〜309 行。
  - ファイルの大きさ: `sakura_builder.lua` 383 → 540、`init.lua` 260 → 267、`virtual_dispatcher.lua` 288 → 303。

## 申し送り（chain-talk-interrupt より、2026-10-23 の完了時）

- 前の申し送り（2026-10-20）の「`＊チェイントーク中断` を割り込んだシーンの台詞の前に添える」は設計で改めた。中断の台詞は反応の**代わりに**出る（優先）。中断の台詞が出た回は、割り込んだシーンが区切った残りも捨て、続きを残さない。
- 続きを捨てるのは 3 つだけになった: 台詞（字のある `talk` か `choice`）を出した割り込み・指名した再生（キックと `OnPastaScene`・`OnPastaSceneAt`。`STORE.co_preempt` の印）・エラー。台詞を出さないシーンや、台詞を出さずにコールバック待ちに入ったシーンは、待っている続きに触れない。
- 仕組み: `shiori/event/init.lua` の `set_co_scene(co, keep)`・`drive_kind(co)`・`run_interrupt_scene(act)`・`EVENT.drive`（種類を決める → act の欄 `_spoke`・`_on_speech`・`_interrupted` を整える → 再開 → 欄を外す → 台詞の印を読む）。台詞の判定は `sakura_builder.lua` の `BUILDER.speaks`、印は `shiori/act.lua` の `SHIORI_ACT_IMPL.build` が立てる。詳しくは `.kiro/specs/completed/chain-talk-interrupt/design.md` の「続きの決まり」と ContinuationRule。
- 見直しが要る論点: 「止まっている場面が捨てられる」を前提にする所は、台詞を出したシーン・指名した再生・エラーの 3 つに狭まったことを前提に測り直すこと。`STORE.co_preempt`（弱い参照の表）が `STORE` に増え、`STORE.reset` で作り直される。

## 2026-10-23 棚卸の再測定（main 82176f19）

- **前提の変化**（前回 2026-10-18・`facd6cf4` から）:
  - 続きの決まり（`chain-talk-interrupt`）: 待っている続き（`STORE.co_scene`）を捨てるのは、台詞（`sakura_builder.lua` の `BUILDER.speaks` 413 行）を出した割り込み・印の付いた指名の再生（キック・`OnPastaScene`・`OnPastaSceneAt`。`STORE.co_preempt`）・エラーだけになった。台詞の無い割り込みは前の続きを残し、自分の残りを捨てる（`shiori/event/init.lua` の `set_co_scene(co, keep)` 110 行・`EVENT.drive` 257〜301 行）。「場面が最後まで走った応答だけを区切りにする」の「最後まで」に、この「残りを捨てた割り込み」と「前の続きが残ったままの応答」をどう数えるかが加わる。
  - 中断の台詞（`＊チェイントーク中断`）が出た回は、割り込んだシーンのトークン列が中断のシーンのものに差し替わり、1 回の組み立てで台本になる（`shiori/act.lua` 81〜91 行）。目印を組み立てで打つなら、差し替えた後の列に打つことになる。台本は中断の台詞で、状態は割り込んだシーンが動かした後、という組を区切りとして控えてよいかを要件で決める。
  - 控えの中身: `STORE` に `co_preempt`（コルーチンを弱い参照で持つ表。`store.lua` 71 行、`STORE.reset` 120 行で作り直す）が増えた。走りかけのコルーチンの印なので控えない。巻き戻しで `co_scene` を捨てれば、印は GC で消える。act の欄 `_spoke`・`_on_speech`・`_interrupted` も再開 1 回の間だけのもので、控えない。`set_co_scene` は今もファイルの中の関数で、引数が 2 つになった。巻き戻しの受け口から続きを捨てる口をどこに置くかを決める。
  - 外からの書き込み（`external-control-events`）: `OnPastaSetSaveVariable`（`shiori/event/external_control.lua` 105 行）は、デバッグの有無によらず `pasta.save` を書き換え（143 行）、台本を出さない。区切りの後に書いた値は、巻き戻すと控えの値で上書きされる。`OnPastaCallFunction`（65 行）のコルーチンは印を持たないので、ふつうの割り込みになる。どちらの応答を区切りにするかも、要件の 2 番に入れる。
  - 鎖（`reg-handler-chain`）: `EVENT.fire` は、鎖のハンドラごとに act を作り直す（`init.lua` 324〜339 行）。応答を作る所は前と同じく `EVENT.fire` の 3 つの枝で、増えていない。
  - 起票された `shiori4-v4-host`（0.5.0）は、応答の外から台本を送る道（`SEND`＋`Script`）と、完了で待っているシーンを起こす `OnHostComplete` を足す。応答の出口だけで台本を控える形は、この道の台本を取りこぼす。先に入った側に、後から入る側が合わせる（区切りにする応答の決まりに含める）。
- **触るファイル**: 目印の出力 `shiori/sakura_builder.lua` 558（立ち位置を書き換えるのは 465〜467 行の `spot` と 469〜471 行の `clear_spot`）、区切りの倉（新しい Lua のファイル）、応答の出口 `shiori/entry.lua` 153（止まるエラーの `STOP.response` 112 行）・`shiori/event/init.lua` 371（`RES.ok` 348・354 行、`SHIORI/` で始まる文字列をそのまま返すのは 352 行）・`shiori/event/callback.lua` 233（`RES.ok` 123・199 行、`STOP.response` 185・190 行）、巻き戻しの受け口（新しいファイルと、`init.lua` の読み込みの並び 85〜90 行の後ろに 1 行）、`shiori/appearance.lua` 567、`shiori/event/virtual_dispatcher.lua` 328（止まっている場面を返す所 215〜217 行）、`pasta/store.lua` 139、マニュアルと生成スキル。1,000 行に近づくものは無い。テストは `sakura_builder_test.lua` に足さず、新しいファイルに置く。
- **規模**: 13〜15 → 14〜16（区切りにする応答の決まりに、残りを捨てた割り込み・中断の台詞の回・外からの書き込みが加わる。続きを捨てる口の置き場所で 1）。
- **先に要るもの**: `novel-talk-flow`、`novel-variable-scope`、`novel-areka-contract`（どれも未完了）。0.4.2 の 3 本は完了したので外れた。ファイルの重なり:
  - 0.5.0 の道筋: `novel-talk-flow`（`sakura_builder.lua`・`appearance.lua`・`virtual_dispatcher.lua`・`init.lua`）、`cue-command-runtime`（`sakura_builder.lua`）、`shell-element-click`（`init.lua` の読み込みの行）、`novel-variable-scope`（予約の名前を動かす場合の `virtual_dispatcher.lua`。前後に置くのでぶつからない）、`shiori4-v4-host`（応答の出口と待ちの再開。`callback.lua`・`init.lua` を触る見込み）。
  - 版の決まっていないもの: `scene-anchor-link`（`sakura_builder.lua`）、`scene-exact-match`（`init.lua`・`virtual_dispatcher.lua`）、`recommend-menu-scenes`（`init.lua` の読み込みの行）。
  - 隙間の spec: `ponytail-lua-scripts`（`init.lua` の `set_co_scene`・`callback.lua`・`store.lua`・`entry.lua`・`virtual_dispatcher.lua`）。
- **種別**: 機能（過去の時点を指す名前も、状態の控えも無い）。巻き戻しの方式は開発者の案（2026-10-10）。
- **要件定義のモデル**: Fable のまま。控えは `novel-save-slots` がスロットへ書く保存の形で、areka が頼る。区切りの決まりは、Rust の 5 秒の待ち（届かなかった応答。`crates/pasta_shiori/src/actor/marshaling.rs` 71・213〜223 行。変わっていない）・Lua のコルーチン（`＞yield`・プロパティの読み出しの途中・今回の続きの決まり）・SHIORI の応答（止まるエラーの 200・時計の読み取り・v4 の応答の外の台本）にまたがる時の流れの決まりで、推奨案がまだ無い。
- **分割の案**: 不要（14〜16）。
- **見つけた穴・古くなった記述**:
  - 2026-10-18 の節の応答を包む所（`init.lua` 244・250 行、`callback.lua` 116・192 行、`STOP.response` の `callback.lua` 178・183 行と `entry.lua` 91 行）→ 上の「触るファイル」の行。`set_co_scene` は `init.lua` 100 行 → 110 行。立ち位置を書き換える所は `sakura_builder.lua` 446〜450・453 行 → 465〜467・469〜471 行。
  - ファイルの大きさ: `entry.lua` 132 → 153、`init.lua` 267 → 371、`callback.lua` 226 → 233、`store.lua` 124 → 139、`virtual_dispatcher.lua` 303 → 328、`sakura_builder.lua` 540 → 558。
