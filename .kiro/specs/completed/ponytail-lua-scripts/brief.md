# Brief: ponytail-lua-scripts

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。0.4.5（2026-10-24 開発者の決定「ポニーテールは 0.0.1 後ろからの着手に変更」。経緯: 2026-10-19 に 0.4.3、2026-10-22 に版に縛らない隙間、2026-10-23 に 0.4.3）。0.4.3 の mlua 0.12 の後の main で測り直す。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域（`crates/pasta_lua/pasta_scripts`）に `/ponytail:ponytail-audit` をやり直す。控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとにする。
- **修正は設計以降**で行う。

## Problem

ゴーストの作者が困る。

- pasta.toml の `[ghost].hour_margin` を数でなく文字列（`"30"`）で書くと、話す時刻に毎秒、スクリプトのエラーのバルーンが出続ける。
- Lua の値に改行が入ると、SHIORI の応答が割れ、バルーンが切れる。
- 続きのトークの中の `get_property` が、すぐ時間切れになることがある。

加えて、写しと死んだ分岐が、読む人の手間を増やしている。

## Current State

2026-10-19 の監査で確かめた。Lua の spec 66 本は通る（`cargo test -p pasta_lua --test lua_unittest_runner`）。

1. **`hour_margin` が文字列だと、毎秒エラーになる**（Must fix）
   - 場所：`pasta/shiori/event/virtual_dispatcher.lua` L121-125・L245-247。
   - `talk_interval_min`・`talk_interval_max` は `resolve()` で数か確かめ、だめなら既定へ戻す。`hour_margin` だけは `ghost.hour_margin or 30` のまま。同じ表の 2 つのキーで、規則が違う。
   - Rust の `apply_shiori_defaults` は型を見ない。
   - 比べるところの誤りが、`next_talk_time` を計算し直す行（L251）より前にある。そのため、以後の OnSecondChange が毎回同じ誤りになる。
2. **応答の見出しの値の CR/LF**
   - 場所：`pasta/shiori/res.lua` L52-56（`RES.build`）。
   - Lua で `act.var.x = "上\n下"` とし、辞書で `＄x` と書くと、応答が割れて `\e` を失う。Lua の単語に改行が入っている場合も同じ。
   - `FAILURE.escape` は失敗の表記で CR/LF を空白にしているが、唯一の出口では効いていない（SSP での再現はしていない）。
3. **シーンのコルーチンを包む関数の、ほぼ同じ写しが 4 つ**
   - `pasta/scene.lua` L186-196 の `wrap_scene`
   - `pasta/shiori/event/choice_select.lua` L20-35 の `create_scene_coroutine`
   - `pasta/shiori/event/kick.lua` L51-88 の `wrap_local_func`・`wrap_reload_func`
4. **死んだ `coroutine.close` の分岐と、余計な `STORE.co_callback`**
   - 場所：`pasta/shiori/event/init.lua` L96-136（`set_co_scene`）・L203-205、`callback.lua` L73・L223、`pasta/store.lua` L77-92。
   - LuaJIT 2.1 では、`close` の分岐はどれも届かない。
   - `co_callback` は、`consume_staged` がもう boolean で返している値を、グローバルの旗で渡している。
5. **`SHIORI.load`・`SHIORI.unload` が、空の関数を xpcall で包んでいる**
   - 場所：`entry.lua` L45-59・L122-131。
6. **`ActorWordBuilder` は素通しの包み**
   - 場所：`pasta/actor.lua` L20-53。
7. **続きのトークの中の `get_property` の期限が、元の要求の時刻から数えられる**
   - 場所：`pasta/shiori/act.lua` L291。`ACT_IMPL.yield` は、再開のときの新しい act を捨てる。
   - 180 秒後に再開すると、期限は 175 秒前になる。次の OnSecondChange の sweep で時間切れになる。
8. **`get_config` が毎秒走る**
   - 場所：`virtual_dispatcher.lua` L231。
9. **小物**
   - `ACT_IMPL.find_handler` は、`find_act_handler` の別名にできる。
   - `SCENE.register` は、`create_global_table` を呼べる。

## Desired Outcome

- 数でない `hour_margin` は、ほかのキーと同じく既定（30）に戻る。
- 応答の見出しの値の改行は、空白になる（2026-10-19 開発者の決定）。
- 続きのトークの期限は、呼んだ時刻から数える。
- 写しと死んだ分岐が無くなる。振る舞いは変わらない。

## Approach

監査の最小の直しを採る。

- `hour_margin` は、`type(...) == "number"` で確かめる。
- 改行は、`RES.build` の 1 か所で空白に置き換える。
- 包む関数は、`SCENE.wrap = wrap_scene` を出して、ほかから呼ぶ。
- 受け渡しは、`EVENT.drive` で `if CALLBACK.consume_staged(co, act) then co = nil end; set_co_scene(co)` とする。
- 期限は、CLOCK が OnSecondChange で見た直近の時刻から計算する。

## Scope

- **In**: 上の 1〜9 と、ギャップ分析で増えた同じ領域の指摘。あわせて、次も直す。
  - テスト：`tests/lua_specs/*`、`tests/shiori/virtual_event_config_resolution_test.rs`
  - マニュアル：`book/src/internals/shiori.md`（L383・時計の節）、`book/src/lua/shiori-events.md`（設定の表・RES の表）、`book/src/internals/execution-model.md`、`book/src/internals/internal-modules.md`
  - `.kiro/specs/novel-checkpoint-rollback/brief.md` L24（`co_callback` の言及）
- **Out**: 空の `pasta.areka`（`pasta/areka/init.lua`）は残す（2026-10-19 開発者の決定）。0.5.0 の `novel-areka-contract` の置き場のため。

## Boundary Candidates

- 設定の読み（virtual_dispatcher）
- 応答の組み立て（res）
- シーンの包み（scene・choice_select・kick）
- コルーチンの受け渡し（init・callback・store）
- 時計と期限（clock・act）

## Out of Boundary

- Rust 側（`pasta_lua/src`。`ponytail-lua-loader` と `ponytail-lua-transpiler` が持つ）
- `scriptlibs/`（同梱の luacheck）
- `sakura_builder.lua` の書記素の扱い（`talk-grapheme-clusters`）

## Upstream / Downstream

- **Upstream**: Rust の実行時（`talk_to_script`・persistence）
- **Downstream**: SSP への応答、ゴーストの scripts/

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-lua-loader`・`ponytail-shiori`（`book/src/internals/shiori.md` の別の節を触る。後から入る側が rebase する）
  - `novel-checkpoint-rollback`（`STORE.co_scene` の扱いを前提にする）
  - `ontalk-suppression`（0.4.2。`virtual_dispatcher.lua` の `dispatch`・`check_talk` に、止めている間の分岐と、再開した時点での `next_talk_time` の決め直しを足す。こちらが後から入るので、行番号を測り直して合わせる。2026-10-19 の申し送り）
  - `chain-talk-interrupt`（0.4.2。`shiori/event/init.lua` の `set_co_scene`・`EVENT.drive` の決まりを変える。台詞を出さなかったシーンは待っている続きに触れず、台詞を出したシーンが捨てるときは `＊チェイントーク中断` を添える。キック・シーン再生・エラーは、はっきり捨てる。こちらが後から入るので、4 番の整理は新しい決まりを保ったまま行う。2026-10-20 の申し送り）

## Constraints

- 挙動を変える直しは、同じ変更でマニュアルを直し、`gen-skill-refs` を走らせる。
- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo test -p pasta_lua`（Lua の spec を含む）、luacheck（luarocks の lfs・argparse を `LUA_PATH`・`LUA_CPATH` に足せば、手元でも動く）。
- cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。`sample.generated.lua` の改行だけの差分は戻す。

## 申し送り（ontalk-suppression より、2026-10-22 完了時）

- `virtual_dispatcher.lua` の `dispatch` に、キックの後・`check_hour` の前の門（`if STORE.auto_talk_paused then return nil end`）を足した。公開の関数 `M.restart_timers(act)` も足した（`next_talk_time`・`next_hour_unix`・`seen_epoch` を書く）。`check_talk` には手を入れていない。行番号は測り直して合わせる。

## 申し送り（reg-handler-chain より、2026-10-23 完了時）

- `shiori/event/init.lua` の `EVENT.fire` の、act を作ってハンドラを呼ぶ所が、鎖のループ（ハンドラごとに `create_act` と `STOP.hold_handler`）と底の `rawget(REG, req.id) or EVENT.no_entry` に変わった。冒頭の説明も伸びたので、このファイルの行番号は測り直す。型判定の枝・`set_co_scene`・`EVENT.drive` には触れていない。
- `event/register.lua` の `REG` にメタテーブル（`__index` の `register` と、欄 `handlers`）が付いた。`REG` を空にするテストは、`package.loaded` から読み直せば鎖も空になる。
- 参照: `.kiro/specs/completed/reg-handler-chain/design.md` の「File Structure Plan」「Components and Interfaces」。

## 申し送り（chain-talk-interrupt より、2026-10-23 の完了時）

- 前の申し送り（2026-10-20）の「`＊チェイントーク中断` を割り込んだシーンの台詞の前に添える」は設計で改めた。中断の台詞は反応の**代わりに**出る（優先）。中断の台詞が出た回は、割り込んだシーンが区切った残りも捨て、続きを残さない。
- 続きを捨てるのは 3 つだけになった: 台詞（字のある `talk` か `choice`）を出した割り込み・指名した再生（キックと `OnPastaScene`・`OnPastaSceneAt`。`STORE.co_preempt` の印）・エラー。台詞を出さないシーンや、台詞を出さずにコールバック待ちに入ったシーンは、待っている続きに触れない。
- 仕組み: `shiori/event/init.lua` の `set_co_scene(co, keep)`・`drive_kind(co)`・`run_interrupt_scene(act)`・`EVENT.drive`（種類を決める → act の欄 `_spoke`・`_on_speech`・`_interrupted` を整える → 再開 → 欄を外す → 台詞の印を読む）。台詞の判定は `sakura_builder.lua` の `BUILDER.speaks`、印は `shiori/act.lua` の `SHIORI_ACT_IMPL.build` が立てる。詳しくは `.kiro/specs/completed/chain-talk-interrupt/design.md` の「続きの決まり」と ContinuationRule。
- 見直しが要る論点: 4 番の整理（`coroutine.close` の死んだ分岐・`STORE.co_callback`）で、`set_co_scene` の `keep` の引数と `_interrupted` の枝（`set_co_scene(nil, false)`）を保つこと。brief の L70 の案（`if consume_staged then co = nil end; set_co_scene(co)`）はもう今の形に合わない。`EVENT.drive` の手順の順は `tests/lua_specs/chain_talk_interrupt_test.lua` が守っている。

## 申し送り（2026-10-29 の discovery「シーン属性による候補の絞り込み」より）

- `call-attribute-filter` は閉じて、新しい spec `scene-condition-attributes` に統合した（開発者の決定）。この brief の `call-attribute-filter` は `scene-condition-attributes` と読み替える。
- 中身が変わった。条件は呼ぶ側（`＞シーン＆k＝v`）でなく、場面の側に書く（`＊雑談_今の部屋　＆＄＊経験値≧500`）。
  - 呼ぶ側の Call の文法（`call_scene`・`parse_action.rs`・`element_gen.rs` の Call）は触らない見込みである。
  - 触るのは、属性の文法、`scene.lua`・`act.lua` の検索の道、`search/context.rs`・`scene_table.rs`、イベントからシーンを起こす道（選ぶ前の自動転記）、`pasta_check validate`、LSP と VSCode の文法定義である。
- 参照: `.kiro/specs/completed/scene-condition-attributes/brief.md`。

## 2026-11-02 棚卸の再測定（main 491cc23c）

- **前提の変化**: 前の測り直し（82176f19）の後、3 本がこの spec の領域に手を入れた。どれも監査のやり直しの対象に入る。
  - `recommend-menu-scenes`: 新しいファイル `shiori/event/recommend.lua`（285 行。読み込みは `event/init.lua` 91 行）。130 行でシーン関数を `coroutine.create` でじかに動かす（写しの 5 つ目。その場で走らせて捨てる形）。119〜122 行の名前の引き当ては、下の共通の部品と同じ題材。
  - `mlua-stdlib-removal`: `external_control.lua` 145 → 152 行（Reference の型の読み）。3 番の `coroutine.create` は 83 行。`date-var-season-sekku`: `shiori/act.lua` 344 → 358 行（`transfer_date_to_var` に季節・節句。7 番とは別の関数で重ならない）。
  - 残っている（行は今の値）: 1 番 `virtual_dispatcher.lua`（328 行。変わっていない）84 行 `ghost.hour_margin or 30`、206 行で比べ、211 行の計算し直しより前で返る。設定の読み直しは 100・191 行。2 番 `res.lua` の `RES.build`（39 行から。変わっていない）。3 番 `kick.lua` 59〜72・80〜92 行（`STORE.co_preempt` の印は呼ぶ側に残す）・`choice_select.lua` 25〜35 行・`scene.lua` 186〜197 行。4 番 `set_co_scene(co, keep)` 111〜155 行、死んだ `close` の分岐 113〜118・134・147〜151 行と `store.lua` 97〜105 行、`EVENT.drive` 258 行から（287 行が `consume_staged` の戻り値を捨てる。296 行の `_interrupted` の枝 `set_co_scene(nil, false)` を保つ。手順の順は `chain_talk_interrupt_test.lua` が守る）、`callback.lua` 73・230 行。7 番は `shiori/act.lua` 324 行の `CLOCK.steady(self.req)`（`self` は区切る前の act。`pasta/act.lua` 664 行の `ACT_IMPL.yield` が再開の値を捨てる）。5・6・9 番は `entry.lua` 69・140 行、`actor.lua` 20〜53 行、`pasta/act.lua` 410 行。
  - README と `tech.md` の断り書きは `ponytail-lua-loader` が書いた。この spec は書かない。
- **触るファイル**: 上のファイルと `recommend.lua` 285、`init.lua` 372、`register.lua` 117、`scene_play.lua` 114、`sakura_builder.lua` 558（読むだけの見込み）、`pasta/act.lua` 837（1,000 行に近い。この spec は減らす向き）、`mouse.lua`、テスト `tests/lua_specs/`・`tests/shiori/virtual_event_config_resolution_test.rs`、マニュアル（Scope のとおり）。新しいファイルは無い見込み。
- **規模**: 12〜15 タスク（`recommend.lua` の監査で 1 つ増える）。
- **先に要るもの**: 機能の上では無い。ponytail のほかの 7 本とはソースが重ならない（`internals/shiori.md`・`execution-model.md` は `ponytail-shiori` と節が別）。`pasta/act.lua` は 1 ウェーブに 1 本の決まり。0.5.0 以降の重なり: `scene-attribute-store`・`scene-pick-modes`・`scene-condition-attributes`・`hello-novel-sample`（`scene.lua`・`act.lua`）、`scene-count`（`entry.lua`）、`scene-exact-match`・`scene-anchor-link`・`shell-element-click`・`cue-command-runtime`（`choice_select.lua`・`init.lua`・`act.lua` ほか）、`novel-talk-flow`・`novel-checkpoint-rollback`・`novel-variable-scope`（`init.lua`・`virtual_dispatcher.lua`・`store.lua` ほか）、`shiori4-v4-host`・`shiori4-v4-lua-api`（`kick.lua`・`scene_play.lua`・`init.lua`・`callback.lua`・`external_control.lua`・`shiori/act.lua`）。
- **種別**: バグ（数でない `hour_margin` で毎秒エラー・値の改行で応答が割れる・続きのトークの `get_property` がすぐ時間切れ）。残りは写しと死んだ分岐の整理。
- **要件定義のモデル**: Opus。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**: Approach の「`if CALLBACK.consume_staged(co, act) then co = nil end; set_co_scene(co)`」は今の形に合わない（真偽を `keep` の枝と捨てる枝の両方へ渡す）。`virtual_dispatcher.lua` 252 行の説明「`set_co_scene` 置換で前 co_scene を閉じる」は古い（今は `STORE.co_preempt` の印で捨て、LuaJIT に `close` は無い）。申し送り（`external-control-events`）: `scene_play.lua`・`external_control.lua`（と `recommend.lua`）の名前の引き当てと、見つからないときの失敗の表記の共通の部品の取り出しは、この spec の題材。申し送り（`chain-talk-interrupt`）: `EVENT.drive` の `_interrupted` の枝は `STORE.co_callback` を戻さない。`get_property` の組み立てが台詞に数えられるように変わるなら見直す。

## 申し送り（2026-11-02 の discovery「Pasta DSL MCP移行検討」より）

- 0.5.0 のウェーブ 3 の見込みの `debug-mcp-runtime` が、`pasta/shiori/event/kick.lua` を丸ごと消す。`SHIORI.kick`（`entry.lua`）と `STORE.kick_force`・`kick_pending`（`virtual_dispatcher.lua`）も消す。
- この spec の `kick.lua` への指摘（`wrap_local_func`・`wrap_reload_func` の包む関数の写し）は、直しても捨てることになる。範囲から外してよい。ほかの包み（scene・choice_select）の写しを 1 つにまとめるなら、`kick.lua` を数に入れなくてよい。
- `entry.lua`・`virtual_dispatcher.lua` への直しは、そのまま進めてよい。`debug-mcp-runtime` は、この spec が入った後の main で直す。
