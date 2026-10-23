# Brief: chain-talk-interrupt

> **ステータス**: 未着手（2026-10-20 の `/kiro-discovery` で起票）。0.4.2 に入れる（2026-10-20 開発者の決定「次のリリースに含める」）。要件定義は Opus。

## Problem

`＞チェイントーク` で区切った話の続きが、何も喋らないイベントで消える。

- 窓際のぱすたさん（ghost_dev）で、`＞チェイントーク` の後半がまったく出なかった（2026-10-20、「ぱすた開発」セッションが実機のログで確認）。
- そのとき動いていたのは、`＊OnMouseMove`・`＊OnMouseEnter`・`＊OnMouseLeave` のシーンである。どれも、ほとんどの回は Lua ブロックで `return nil` し、何も出さずに終わっていた。その間の応答は、すべて 204 だった。
  - `OnMouseEnterAll`／`OnMouseLeaveAll` も、前方一致で `＊OnMouseEnter`／`＊OnMouseLeave` に入る。シーンが動く回数は、見かけの約 2 倍になる。
- 作者から見ると、ゴーストは何も言っていないのに、話の続きを忘れている。
- また、話の途中で割り込まれたことを、台詞で受ける書き方が無い。`＊つつき_ぱすた_頭` に「話の途中ですわよ」と書くと、話していないときにも出てしまう。

## Current State

2026-10-20（main `36a0a8db`）に確かめた。

- **続きの置き場所**: 中断したシーンのコルーチンは `STORE.co_scene` に 1 つだけ置かれる。`set_co_scene`（`pasta_scripts/pasta/shiori/event/init.lua` L100-136）が置き換える。
- **捨てられる仕組み**: `EVENT.drive`（L190-207）は、シーンを再開した後に必ず `set_co_scene(co)` を呼ぶ。
  - 最後まで動いたシーンのコルーチンは dead なので、`co = nil` として扱われる（L102-107）。
  - 待っている続きは、これと別のものなので、破棄されて `nil` になる（L128-135）。
  - シーンが何も出さなかった（応答が 204）ときも同じである。
- **続きを再開する場所**: 仮想イベントの OnTalk だけである（`virtual_dispatcher.lua` の `check_talk`。L213-218 で `STORE.co_scene` を返す）。
- **撫での受け口**: `shiori/event/mouse.lua` L155-164 は、撫でと決まらなかった `OnMouseMove` を、毎回 `＊OnMouseMove` のシーンへ回す（`SCENE.co_exec`）。
- **出力の種類**: act のトークンには `type` がある（`act.lua`）。台詞は `talk`（L111・L241）。ほかに `sakura_script`・`raw_script`・`surface`・`wait`・`newline`・`clear`・`choice`・`spot` などがある。台詞を出したかどうかは、トークンから分かる。
- **マニュアルの今の決まり**:
  - `book/src/grammar/call-jump.md` L362「別のイベントでシーンが実行される（時報のシーンを含む）と、待っていた残りは破棄される」
  - L347「外から起こした `OnTalk` では続きは出ず、新しいトークが始まり、待っていた続きは破棄される」
  - 内部設計は `book/src/internals/execution-model.md` L244-266（`set_co_scene` の規則）。

## Desired Outcome

- 台詞を出さなかったシーンは、待っているチェイントークの続きに触れない。窓際のぱすたさんの症状は、ゴースト側を変えずに消える。
- 台詞を出したシーンが割り込んだら、続きは今どおり捨てる（忘れてよい）。
- そのとき、作者が `＊チェイントーク中断` を書いていれば、その台詞を割り込んだシーンの台詞の前に添える。

```pasta
＊OnTalk
  ぱすた：昔話をしますわね。むかしむかし……
  ＞チェイントーク
  ぱすた：おじいさんは山へ芝刈りに行きましたの。

＊チェイントーク中断
  ぱすた：あら、お話の途中でしたのに。まあ、よろしいですわ。
```

昔話の前半 → 頭をつつく →「あら、お話の途中でしたのに。まあ、よろしいですわ。」＋ふだんのつつきの反応。昔話の後半は出ない。

- マウスを動かしただけ（何も喋らない）なら、後半は次の自然なおしゃべりで出る。
- × ボタンに乗せたときの着せ替え（`\![bind]` だけ）も、続きに触れない。

## Approach

2026-10-20 の話し合いで、開発者が決めた（案 D と、議案のすべての推奨）。

1. **続きを捨てるのは、台詞を出したシーンだけ**。
   - 台詞に数えるのは、台詞（`talk` のトークン）と選択肢（`choice`）。選択肢を出すのは、話しかけているのと同じだからである。
   - 数えないのは、表情（`surface`）・`raw_script`・`sakura_script`・ウェイト・改行・`spot` などだけのシーンと、何も出さないシーン。窓際のぱすたさんで × ボタンに乗せて `\![bind,閉じる,赤,1]` だけを出しても、続きは残る。
2. **割り込まれたら、続きは忘れる**。続きを後で再開する仕組み（`＊チェイントーク再開` など）は作らない。「チェイントークは、そこまでガードすべきものではない」（開発者）。
3. **`＊チェイントーク中断`**: 台詞を出したシーンが、待っている続きを捨てるときだけ流す。
   - 置く位置: 割り込んだシーンの台詞の前（同じ応答の中）。後ろに置くと、反応の後に「お話の途中でしたのに」が来て、ちぐはぐになる。
   - 探し方: 名前の完全一致（前方一致で `＊チェイントーク中断ごっこ` などに当たらないように）。同じ名前のシーンが複数あれば、ふだんのシーンと同じく、その中から選ぶ。
   - 書かなければ、今と同じ（黙って捨てる）。
4. **デバッグのキックと `OnPastaScene`（`scene-play-event`）**: 今どおり、続きを捨てる。開発者が指名した割り込みなので、`＊チェイントーク中断` は流さない。
5. **割り込んだシーン自身もチェイントークで区切ったとき**: 新しい方の続きが残る（古い続きは忘れる）。割り込んだことには変わりがないので、`＊チェイントーク中断` も流す。
6. **エラーで止まったシーン**: 今どおり、続きを捨てる。バルーンにエラーの表記が出るので、`＊チェイントーク中断` は流さない。
7. **外から起こした `OnTalk`（`\![raise,OnTalk]` など）**: ほかのイベントと同じ決まりに従う。新しいトークが台詞を出せば、続きは忘れ、`＊チェイントーク中断` を流す。

## Scope

- **In**
  - `set_co_scene`／`EVENT.drive` の決まりの変更（台詞を出さずに終わったシーンは、待っている続きを残す）
  - `＊チェイントーク中断` の検索と、割り込んだシーンの台詞の前への差し込み
  - キックと `OnPastaScene` の道で、続きをはっきり捨てる（`＊チェイントーク中断` は流さない）
  - エラーで止まったシーンの道で、`＊チェイントーク中断` を流さない
  - Lua のテスト（`crates/pasta_lua/tests/lua_specs/`）。マウスの空振りで続きが残る、台詞の割り込みで捨てる、`＊チェイントーク中断` が前に付く
  - `pasta_check` が `チェイントーク中断` を、作者が書いてよい決まった名前として知る必要があるかを確かめる（シーンの名前なので、飛び先の検査には関わらない見込み）
  - マニュアル（`grammar/call-jump.md` のチェイントークの節、`lua/shiori-events.md`、`internals/execution-model.md` の `set_co_scene` の規則）と、スキルの references の作り直し
- **Out**
  - 割り込まれた続きを後で再開する仕組み（`＊チェイントーク再開`）。開発者が不要と決めた
  - 割り込んだイベントの種類ごとに中断の台詞を変える仕組み（`＊チェイントーク中断_つつき` など）
  - 撫での受け口が、決まらなかった `OnMouseMove` をシーンへ回すこと自体の見直し
  - `OnMouseEnterAll` が `＊OnMouseEnter` に前方一致で入ること（シーンの検索の決まり）
  - 見本のゴースト（hello-pasta）の書き換え

## Boundary Candidates

- 続きの更新の決まり（`shiori/event/init.lua` の `set_co_scene`・`EVENT.drive`）
- 台詞を出したかの判定（act のトークン。`act.lua`・`shiori/act.lua`）
- `＊チェイントーク中断` の検索と差し込み（応答の組み立て）
- キックの道（`shiori/event/kick.lua`）と、`scene-play-event` が足すシーン再生の道
- マニュアルとスキルの references

## Out of Boundary

- 自動トークの停止（`ontalk-suppression`。続きを再開する道を止めるだけで、捨てる決まりには触れない）
- シーン再生のイベントそのもの（`scene-play-event`）
- `set_co_scene` の死んだ `coroutine.close` の分岐と `STORE.co_callback` の整理（隙間の spec の `ponytail-lua-scripts`）
- ノベルの場面の流れ（0.5.0 の `novel-talk-flow`）

## Upstream / Downstream

- **Upstream**: チェイントーク（完了 spec `yield-continuation-token`）、続きの更新の決まり（`callback-resume-unification`）、キック（`pasta-scene-kick`）、撫で・つつきのショートカット（`touch-poke-shortcuts`）
- **Downstream**: `novel-talk-flow`（0.5.0）。brief の「止まっている場面が捨てられる」（`shiori/event/init.lua` の `set_co_scene`）が、この spec で台詞を出したシーンだけに狭まる。

## Existing Spec Touchpoints

- **Extends**: なし（新しい境界）
- **Adjacent**
  - `scene-play-event`（0.4.2）: シーン再生の道で、続きを捨てる決まり（キックと同じ）を、この spec の新しい決まりの中で保つ。この spec はあちらの後に入り、キックとシーン再生の「はっきり捨てる」を 1 か所で直す。
  - `ponytail-lua-scripts`（ponytail・隙間の spec）: 同じ `set_co_scene`・`EVENT.drive` を整理する。同じウェーブに置かない。この spec が先に入り、あちらが後から合わせる。
  - `ontalk-suppression`（0.4.2）: ファイルを共有しない（あちらは `virtual_dispatcher.lua`・`entry.lua`）。並べられる。
  - `novel-talk-flow`（0.5.0）: 上の Downstream。あちらの要件で、新しい決まりに合わせて読み直す。

## Constraints

- 台詞を出したシーンが割り込んだときの挙動は、`＊チェイントーク中断` が無ければ今と同じにする。
- `＊チェイントーク中断` 自身は、待っている続きを捨てる道の中で流すので、さらに `＊チェイントーク中断` を呼ばない（入れ子にしない）。
- LuaJIT 2.1 には `coroutine.close` が無い。捨てるのは参照を外すだけ（`STORE.co_scene = nil`）で、GC に任せる。
- 非 ASCII のシーン名は、ふだんのシーンと同じく DSL で書ける。Lua から探すときは、ブラケット記法か文字列で引く。
- 規模は 5 タスク前後の見込み（続きの更新の決まり 1、中断の台詞の差し込み 1、キックとシーン再生の道 1、テスト 1、マニュアルとスキル 1）。

## 2026-10-22 棚卸の再測定（main 6a8dc479）

- **前提の変化**: 2026-10-20 の後、続きの更新の決まり（`pasta_scripts/pasta/shiori/event/init.lua` の `set_co_scene` L100〜136・`EVENT.drive` L190〜207）、撫での受け口（`shiori/event/mouse.lua` L155〜165）、トークンの種類（`act.lua` L241・L308）は変わっていない。Current State は今も正しい。
- **触るファイル**: `event/init.lua`（267 行）、台詞を出したかの印を立てる所（トークンは `act:build()` の中で空にされる（`act.lua` L641〜643）ので、`EVENT.drive` からは見えない。`shiori/act.lua` 325 行の `SHIORI_ACT_IMPL.build` L65 か、`act.lua` 829 行の `ACT_IMPL.build`）、キックの道 `shiori/event/kick.lua`（163 行）、`scene-play-event` が足すハンドラのファイル、中断の台詞を前に添える所（組み立て `shiori/sakura_builder.lua` 540 行に触る見込み）、Lua のテストと一覧 `crates/pasta_lua/tests/lua_specs/init.lua`、マニュアル。
- **規模**: 6〜7 タスク（中断の台詞の添え方が見込みより重い）。
- **先に要るもの**
  - `scene-play-event`: 本当に待つ必要がある。あちらが足すハンドラのファイルと `event/init.lua` を共有し、シーン再生の道で「はっきり捨てる」を入れる先があちらにあるため。順を逆にして、こちらが捨てる口を先に作り、あちらが使う形もとれるが、どちらにせよ同じウェーブには置けない。
  - `external-control-events`: `event/init.lua` を共有する（あちらは読み込みの 1 行だけ）。その 1 行を「後から入る側が合わせる」扱いにしない限り、同じウェーブに置けない。
  - `ontalk-suppression`: キックの「はっきり捨てる」を `virtual_dispatcher.lua` のキックの所ではなく `kick.lua` に置き、印を `STORE`（`store.lua`）に置かなければ、ソースを共有しない。その条件で並べられる。
  - 0.4.2 の後では、`act.lua`・`sakura_builder.lua` を `cue-command-runtime`・`novel-talk-flow`・`scene-anchor-link` などが、`set_co_scene` を隙間の `ponytail-lua-scripts` が触る。
- **種別**: バグ（何も喋らないイベントで続きが消える。窓際のぱすたさんの実機で確認）＋機能（`＊チェイントーク中断`）。開発者の決定「次のリリースに含める」。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 組み立てた台本は、終わりに `\e` が付く（`sakura_builder.lua` L534〜536）。中断の台詞の台本と割り込んだシーンの 1 ビート目を文字列でつなぐと、前の `\e` で止まる。しかも、割り込んだシーンが喋るかは組み立てた後でないと分からない。添え方（トークンの段でつなぐか、`\e` を外すか、立ち位置と表情の持ち越し `STORE.actor_spots`・`STORE.appearance` との順）は設計の要点になる。
  - 台詞を出したかの印を act のメソッドにすると、`pasta_check` の act のメソッドの表（`crates/pasta_check/src/validate/builtins.rs` の `ACT_METHODS`、27 個）と、実行時との一致のテストが変わり、`ontalk-suppression` と `builtins.rs` を共有してしまう。印は act の欄か関数の中の変数にとどめる。
  - コールバックの再開（`shiori/event/callback.lua` の `CALLBACK.try_route`・`CALLBACK.sweep`）も `EVENT.drive` を通るので、新しい決まりが効く。テストに含める。

## 申し送り（ontalk-suppression より、2026-10-22 完了時）

- `pasta/act.lua` の `ACT_IMPL.yield` の先頭に、止めている間の見張り（警告を出して `GLOBAL.resume_auto_talk` を呼び、その後は今どおり区切る）を足した。台詞を出したかの印などで `ACT_IMPL.yield` を触るときは、この分岐と並べる。
- 止める門は `set_co_scene` を通らないので、続きを捨てる決まりには触れていない。`builtins.rs` の `GLOBAL_NAMES` は 8 つになった（`ACT_METHODS` は触っていない）。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**
  - `ontalk-suppression` が入った。止めている間は、自動トークの判定の入口（`pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` の `dispatch` L262〜267）が続きを返さず、捨てもしない。撫での受け口（`event/mouse.lua` L159〜171）は、止めている間は撫でと決めないので、届いた `OnMouseMove` はすべて `＊OnMouseMove` のシーンへ回る。今のままだと、止めている間に何も喋らないマウスのシーンが続きを捨てる。この spec の直しが、これもそのまま直す。チェイントークの区切り（`act.lua` L664 の `ACT_IMPL.yield`）の先頭には、止めている間の見張り（自動トークを再開してから区切る）が入った。
  - `scene-play-event` が入った。シーン再生のハンドラ（`event/scene_play.lua`）は自分で続きを扱わず、コルーチンを返してふつうの道（イベントの振り分け → `EVENT.drive` → 続きの置き換え）に任せる。失敗のときは文字列を返すので続きに触れない。キックも、自動トークの判定 → 毎秒の既定のハンドラ → イベントの振り分けの、ふつうの道を通る。だから「はっきり捨てる」は、シーン再生のファイルとキックのファイル（`event/kick.lua` のシーンを包む 2 つの関数）で印を付け、続きの置き換えの所で読む形になる。
  - 行の位置: 続きの置き換え `set_co_scene` は `event/init.lua` L101〜137、`EVENT.drive` は L191〜208、`EVENT.fire` は L216〜256。トークンを空にする所は `act.lua` L641〜643。
- **触るファイル**: `event/init.lua`（268 行）、`event/scene_play.lua`（107 行）、`event/kick.lua`（163 行）、台詞を出した印を立てる所（`act.lua` 837 行の `ACT_IMPL.build` か `shiori/act.lua` の `build`）、中断の台詞を前に添える所（`shiori/sakura_builder.lua` 540 行の見込み）、Lua のテストと一覧の 1 行、マニュアル（`grammar/call-jump.md` L349・L364、`lua/shiori-events.md` のシーン再生の「再生と続き」L605 と OnTalk の節 L937〜、`internals/execution-model.md` L247〜270、`internals/shiori.md` L388）。`pasta_check` は触らない（中断はシーンの名前なので）。
- **規模**: 6〜7 タスク。
- **先に要るもの**
  - 働きの上では無い。
  - `external-control-events`・`reg-handler-chain` と同じウェーブに置ける。約束は「`event/init.lua` は続きの置き換えから `EVENT.drive` まで（L97〜208）と、その間に置く新しい内部の関数だけを触る。イベントの振り分け `EVENT.fire`・読み込みの並び・モジュールの冒頭の説明は触らない。中断の台詞を添えるのも `EVENT.drive` か、それが呼ぶ関数の中で行う」。どうしても `EVENT.fire` に入れるなら、型で分ける枝（L238〜255）だけにして、`reg-handler-chain` が触るハンドラを引く行（L231〜236）との間の行を残す。
- **種別**: バグ（何も喋らないイベントで続きが消える）＋機能（`＊チェイントーク中断`）。開発者の決定「次のリリースに含める」。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 「`OnPastaScene` は今どおり捨てる」には、位置で指名する `OnPastaSceneAt`（同じファイルの 2 つ目のハンドラ）も含める。
  - 外から `＞XXX` を呼ぶ `OnPastaCallFunction`（`external-control-events`）をどの決まりに当てるか。推奨はふつうのイベントと同じ（台詞を出さない `自動トーク停止` は続きを残す）。
  - シーンが書き込む act は、コルーチンを最初に動かしたときの act で、続きを再開するときに `EVENT.drive` が受け取る act とは別のもの。印を読むときは、失敗の部品の「コルーチンから act を引く」関数（`pasta/failure.lua` の `act_of`）を使える。
  - コールバック待ちから戻ったシーン（続きの置き換えのコールバックの枝 L110〜121）も、今は待っている続きを捨てる。そこも台詞の有無で分けるかを決める。
