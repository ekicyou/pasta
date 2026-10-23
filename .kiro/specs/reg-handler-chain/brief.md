# Brief: reg-handler-chain

2026-10-22 の `/kiro-discovery`（開発者の提案「`REG:register("OnSecondChange", func)` で登録し、func が true を返すまで次の登録を呼ぶ。先に登録したものが後で呼ばれる」）から起票した。roadmap のバックログの「`REG` に 1 つのイベントのハンドラを重ねる仕組み」（2026-10-21 のヒアリング）を引き取る。議題は全部、推奨のとおりに開発者が承認した。0.4.2 に入れる（2026-10-22 開発者の決定「次のリリースに含める」）。

## Problem

- 1 つのイベントに、いくつもの処理を足したいゴーストがある。窓際のぱすたさん（ghost_dev）は、`REG.OnSecondChange` を 3 か所で包み合っている。
  - `dic/pose.pasta`: 8 秒静かなら表情を戻す。読み込みのときに `local prev_on_second_change = REG.OnSecondChange` を控え、自分の処理の後で呼ぶ。
  - `dic/tategaki.pasta`: 窓あわせと重ね順。同じく読み込みのときに前のハンドラを控えて包む。
  - `dic/boot.pasta`: 初回の演出の間、毎秒のイベントを全部止める門。シーン（`SCENE.shokai_matsu`）の中で、後から一番外側に包む。
- 包み合いには決まり文句が要る（前のハンドラを控える・呼び忘れない・`nil` のときの扱い）。どちらが外側になるかは、ファイルを読む順で決まる。作者自身が「読み込みの順に頼っていて壊れやすい」と書いた。
- 呼び忘れると、組み込みの既定の OnSecondChange（コールバックの期限切れの掃除・時報・ランダムトーク）が黙って止まる。

## Current State

- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/register.lua`（48 行）: `REG` はただの表（`local REG = {}`）。ハンドラの形は `function(act) -> string|thread|nil` で、戻り値の決まりは同じファイルの冒頭の説明にある（`SHIORI/` で始まる文字列は応答全体、ほかの文字列は 200（空なら 204）、`nil` は 204、コルーチンは `EVENT.drive` で動かす）。
- `shiori/event/init.lua`（267 行）の `EVENT.fire`（L215 あたり）: コールバックの振り分け（`CALLBACK.try_route`）と時計の合図（`CLOCK.observe`）の後、`local handler = REG[req.id] or EVENT.no_entry`（L232）で 1 つだけを呼ぶ。`EVENT.no_entry` は、イベントと同じ名前のシーンを前方一致で探す（`SCENE.co_exec`）。
- 組み込みの既定のハンドラは、`REG.X = REG.X or function … end` の形で `event/boot.lua`・`second_change.lua`・`choice_select.lua`・`mouse.lua` に置かれ、`event/init.lua` L78-81 で読まれる。作者が `REG.X = …` と代入すれば、丸ごと差し替わる。
- テストのうち 3 つ（`tests/lua_specs/event_clock_hook_test.lua`・`event_coroutine_test.lua`・`second_change_thread_test.lua`）は、`for k in pairs(REG) do REG[k] = nil end` で表を空にしている。
- 重ねて登録する口は、どこにも無い。

## Desired Outcome

- 作者は `REG:register("OnSecondChange", function(act) … end)` と書くだけで、前のハンドラを控えずに処理を足せる。
- 返り値は Lua の真偽のとおりに読む。`nil`・`false` なら次（先に登録したもの）へ回る。`true` ならそこで止めて 204。文字列・コルーチンならそこで止めて、今の決まりどおりの応答にする。
- 新しく登録したものから呼ぶ。シーンの中で後から登録した門は、最初に効く（今の `boot.pasta` の包み方と同じ順）。
- 登録したものが全部見送ったら、今どおり `REG[id]`（組み込みの既定か、作者の代入）を呼び、それも無ければイベントと同じ名前のシーンを探す。既定の OnSecondChange は、いつも鎖の底に残る。`REG.X = …` の代入は底を差し替えるだけで、今のゴーストはそのまま動く。
- `register` は、外すための関数を返す。一時的な門を、演出が終わったら外せる。
- すべてのイベントと、リソースの問い合わせ（`sakura.recommendsites` など、`On` で始まらない ID）に効く。

窓際のぱすたさんの書き換えの例:

```lua
-- pose.pasta
REG:register("OnSecondChange", function(act)
    if act.req.reference[3] == "1" then quiet = quiet + 1 else quiet = 0; pending = true end
    if pending and quiet >= QUIET_SECONDS then
        pending = false
        return [=[\0]=] .. dress.build("0000") .. layout.BALL_OFF .. [=[\e]=]
    end
    -- 何も返さなければ、次（先に登録したもの）へ回る
end)

-- boot.pasta（初回待ちの門）
local off = REG:register("OnSecondChange", function(act)
    if SAVE["初回待ち"] then return true end   -- true：ここで止めて 204
end)
-- 演出が終わったら off() で外せる
```

## Approach

- `register.lua` に鎖を持たせる。鎖はイベントの ID ごとの配列で、`REG` の表のキーにはしない（メタテーブルの `__index` で `register` を引けるようにし、鎖そのものは外から見えない所に置く）。`pairs(REG)` にも出ない。
- `EVENT.fire` のハンドラの引き方を、「鎖を新しい順に呼ぶ → 全部見送ったら `REG[id]`（`rawget` で引く）→ 無ければ `EVENT.no_entry`」にする。`rawget` にするのは、`register` という名前のイベントが来たときに `register` 自身を呼ばないためである。
- 鎖の 1 つが真の値を返したら止める。`true` は 204 にし、文字列・コルーチンは今の型判定にそのまま渡す。
- テストのために、鎖を空にする口を置く（`pairs(REG)` で表を空にする 3 つのテストが、鎖も空にできるように）。
- エラーは今どおり `SHIORI.request` の `xpcall` と `pasta/shiori/stop.lua` が受ける。止まったハンドラの表記（`STOP.hold_handler`）も、鎖の中のハンドラで働くようにする。

## Scope

- **In**:
  - `REG:register(id, fn)` と、それが返す外す関数。
  - `EVENT.fire` の引き方（鎖 → `REG[id]` → `EVENT.no_entry`）と、返り値の読み方（偽なら次へ・`true` は 204・文字列とコルーチンは今の決まり）。
  - テストのための鎖を空にする口と、既存の 3 つのテストの合わせ。
  - Lua のテスト（順番・止まり方・`true` の 204・外す・底の既定と代入・`register` という ID・リソースの ID・コルーチンを返すハンドラ・エラー）。
  - マニュアル `lua/shiori-events.md` の REG の節（鎖の作例と、包み合いからの書き換え）と、`node book/tools/gen-skill-refs.mjs` での references の作り直し。
- **Out**:
  - 優先度の数での並べ替え（順番は登録の新しい順だけ）。
  - 応答が出た回にも、全部のハンドラに見せるだけの「見張り役」の口（`pose.pasta` より新しいハンドラが応答した秒は、`pose.pasta` が呼ばれず数え漏れる。包み合いの今も同じ）。
  - 窓際のぱすたさんの辞書の書き換え（ghost_dev の側の作業。マニュアルの書き換えの例を手がかりにする）。
  - DSL の文法（これは Lua の API で、DSL には足さない）。

## Boundary Candidates

- 登録の口（`register.lua`）と、配り方（`event/init.lua` の `EVENT.fire`）。
- マニュアルの REG の節。

## Out of Boundary

- 組み込みの既定のハンドラの置き方（`REG.X = REG.X or …`）は変えない。既定は鎖の底に残る。
- `CALLBACK.try_route`・`CLOCK.observe` の順（鎖より前）は変えない。
- 自動トークを止める口は `ontalk-suppression` が持つ。

## Upstream / Downstream

- **Upstream**: なし（今の `REG` と `EVENT.fire` の上に足す）。
- **Downstream**: `recommend-menu-scenes`（組み込みの受け手を `REG[...] = REG[...] or …` で置き、「Lua の `REG` が勝つ」とする。鎖で登録した作者の受け手も勝つかを、あちらの要件で確かめる）。ghost_dev の窓際のぱすたさんの 3 か所の包み合いの書き換え。

## Existing Spec Touchpoints

- **Extends**: なし。
- **Adjacent**:
  - `scene-play-event`・`external-control-events`: `event/init.lua` の `require` の行に既定のハンドラを足す。
  - `chain-talk-interrupt`: `event/init.lua` の `set_co_scene`・`EVENT.drive` を直す。
  - 上の 3 本とは `event/init.lua` を共有するので、同じウェーブに置かない。
  - `ponytail-lua-scripts`（隙間の spec）: `pasta_scripts/**` の整理。同じウェーブに置かない。
  - `ontalk-suppression`: `virtual_dispatcher.lua`。ファイルは共有しない。その brief は、`REG` に重ねる仕組みを範囲の外としてバックログへ送っていた（この spec が引き取った）。

## Constraints

- LuaJIT 2.1（Lua 5.1）。`goto`・`<close>` は使わない。
- 今のゴーストの `REG.X = …` の代入と包み合いは、変えずに動き続けること。
- `REG` の表の見え方（`pairs(REG)` で回るのはイベントのハンドラだけ）を変えない。
- 手書きのスキル `.claude/skills/pasta-lua-coding/SKILL.md` を直す場合は、分類器が止めるので、その場で開発者の許可を求める（references は生成なので `gen-skill-refs.mjs` で作り直す）。

## 測定（2026-10-22 の discovery）

- 触るファイル: `pasta_scripts/pasta/shiori/event/register.lua`（48 行）、`event/init.lua`（267 行。`EVENT.fire` のハンドラの引き方）、`tests/lua_specs/` の新しいテストと一覧（`init.lua`）への 1 行、`pairs(REG)` で表を空にする 3 つのテスト、マニュアル `book/src/lua/shiori-events.md`、生成の references。
- 規模: 5〜7 タスク。
- 先に要るもの: なし（機能の上では）。ファイルの順番で、`scene-play-event` の後に入る。`chain-talk-interrupt`・`external-control-events` とは同じウェーブに置かない。
- 種別: 機能（開発者の提案。ghost_dev の包み合いの実例が動機）。
- 要件定義のモデル: Opus（Lua の API の取り決めだが、議題は全部推奨のとおりに承認済み）。
- 決めたこと（2026-10-22 開発者の承認）: 返り値は真偽のとおり（偽なら次・`true` は 204・文字列とコルーチンは今の決まり）、新しい順、底は今の `REG[id]` → シーン、呼び方は `REG:register(id, fn)`（`register` は表のキーにせず、引くときは `rawget`）、外す関数を返す、全部のイベントとリソースの ID に効く、優先度と見張り役は作らない、マニュアルを直す、0.4.2 に入れる。

## 申し送り（ontalk-suppression より、2026-10-22 完了時）

- 自動トークの停止（`＞自動トーク停止`）は、既定の OnSecondChange の中の `dispatch` だけを止める。作者の `REG` のハンドラ（鎖に重ねたものも）は止めない。鎖の底の既定のハンドラは、止めている間も期限切れの掃除と仮想時刻の読み取りを続ける。
- 窓際のぱすたさんの `boot.pasta` の「初回待ちの門」は、ランダムトークと時報を止める目的なら `＞自動トーク停止` で置き換えられる。表情や窓あわせの毎秒の処理まで止めたいなら、鎖の門が要る。例を書くときに使い分けを示すとよい。

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**
  - `scene-play-event` が入った。既定のハンドラの読み込みの並びは `pasta_scripts/pasta/shiori/event/init.lua` の L78〜82 になり、シーン再生の 2 つ（`event/scene_play.lua` の `OnPastaScene`・`OnPastaSceneAt`）を足して 7 つ。ハンドラを引く行は、イベントの振り分け `EVENT.fire`（L216〜）の中の L233。
  - `true` を返したときは、今の型で分ける枝（L238〜255）で、すでに 204 になる（文字列でもコルーチンでもない値は 204）。`EVENT.fire` で変えるのは、ハンドラを引いて呼ぶ L231〜236 だけで足りる見込み。
  - `ontalk-suppression` の申し送りは上のとおり。
- **触るファイル**: `event/register.lua`（48 行）、`event/init.lua`（268 行。ハンドラを引いて呼ぶ所だけ）、表を空にする 3 つのテスト（`event_clock_hook_test.lua` L22・`event_coroutine_test.lua` L27・`second_change_thread_test.lua` L136）、新しいテストと一覧の 1 行、マニュアル `lua/shiori-events.md`（REG の節の既定ハンドラを包む段落 L81〜95、シーン関数フォールバックの図 L745）、`internals/execution-model.md` L161、`internals/shiori.md` L318・L413、生成の references。
- **規模**: 5〜7 タスク。
- **先に要るもの**
  - 働きの上では無い。
  - `external-control-events`・`chain-talk-interrupt` と同じウェーブに置ける。約束は「`event/init.lua` は `EVENT.fire` のハンドラを引いて呼ぶ所（L231〜236）と、モジュールの冒頭の説明（L1〜66）だけを触る。マニュアルの既定ハンドラの節は表と数の文（L69〜79）を触らない。後から入る側が rebase で合わせ、生成の references は作り直す」。前回の「別のウェーブ」は、この約束で 1 つにまとめられる。
  - 後ろに `recommend-menu-scenes` が乗る（下）。
- **種別**: 機能（開発者の提案。窓際のぱすたさんの包み合いが動機）。
- **要件定義のモデル**: Opus（取り決めの形は承認済み）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**
  - 鎖の中のハンドラは同じ act を受け取る。前のハンドラが台詞を積んだまま `nil` を返すと、積んだ分が次のハンドラの出力に混ざる。ハンドラごとに act を作り直すか、マニュアルにそう書くかを要件で決める。
  - 止まったハンドラの控え（`pasta/shiori/stop.lua` の `hold_handler`）は 1 回のイベントに 1 つ。鎖の途中で止まったとき、止まった場所の表記がどのハンドラを指すかを確かめる。
  - `recommend-menu-scenes` の「Lua の `REG` が勝つ」は、この鎖でも保たれる。鎖のハンドラが `nil` を返せば、鎖の底の組み込みの受け手（辞書のおすすめ）へ回る。あちらの要件で、この順を書く。
