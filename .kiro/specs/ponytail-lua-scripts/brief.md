# Brief: ponytail-lua-scripts

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

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

## 2026-10-23 棚卸の再測定（main 8fc9bdff）

- **前提の変化**: 0.4.2 の 2 本が、この spec の領域（`crates/pasta_lua/pasta_scripts`）に新しいコードを足した。監査のやり直しでは、これも対象に入る。
  - `ontalk-suppression`: 自動トークを止める・再開する関数（`shiori/entry.lua` 29〜48 行）、印 `STORE.auto_talk_paused`（`store.lua`）、ランダムトークと時報の門（`shiori/event/virtual_dispatcher.lua` 263〜267 行）と時計の決め直し `M.restart_timers`（282 行）、撫で・つつきの分岐（`shiori/event/mouse.lua` 143・165 行）、区切りの関数 `ACT_IMPL.yield`（`pasta/act.lua` 664 行）の見張り。`check_talk` には手が入っていない。止めている間は門で返るので、設定の読み直し（8 番）は止めている間は走らない。
  - `scene-play-event`: 指名したシーンを再生する既定のハンドラ（新しいファイル `shiori/event/scene_play.lua` 107 行。読み込みは `shiori/event/init.lua` 82 行）、`scene.lua` の `SCENE.co_exec_exact` の広がりと、登録名の組から再生する `SCENE.co_exec_registered`（258 行）。
  - 3 番（包む関数の写し）に効く: `SCENE.co_exec_registered` は、登録名の組から関数を引いて `wrap_scene` で包む。キック（`shiori/event/kick.lua` 59 行の `wrap_local_func`）と選択肢（`shiori/event/choice_select.lua` 25 行の `create_scene_coroutine`）は、登録名の組か検索の結果を持っているので、これを呼べば写しが消える。新しく `SCENE.wrap` を外へ出さずに済む見込み（再読み込みの `wrap_reload_func` 78 行は別の形なので残るかを確かめる）。
- **触るファイル**: `virtual_dispatcher.lua` 328、`res.lua` 141（変わっていない）、`scene.lua` 266、`choice_select.lua` 80、`kick.lua` 163、`init.lua` 268（`set_co_scene` 101〜136 行・`EVENT.drive` 191 行から）、`callback.lua` 226、`store.lua` 131、`entry.lua` 153、`actor.lua` 291、`shiori/act.lua` 325、`pasta/act.lua` 837（1,000 行に近い。この spec は別名にする 1 か所と、区切りの関数を触るなら減らす向き）、`mouse.lua` 176、テスト `tests/lua_specs/`（Lua の spec は 4 本増えた）。新しいファイルは無い見込み。
- **規模**: 10〜13 タスク（1〜9 番の直しと、監査のやり直しの増分。増えたコードの分は、写しの消し方が楽になった分とほぼ打ち消し合う）。
- **先に要るもの**: 機能の上では無い。順番の約束が 2 つある。
  - `chain-talk-interrupt` の後（`init.lua` の `set_co_scene`・`EVENT.drive` の決まりを、あちらが先に変える。4 番の整理は新しい決まりを保ったまま行う）。
  - `reg-handler-chain` と同じウェーブに置かない（あちらは `init.lua` の `EVENT.fire`。こちらは同じファイルの `set_co_scene`・`EVENT.drive`）。どちらも 0.4.2 なので、「0.4.2 を出した後に入れる」決まりで自然に守られる。
  - 0.5.0 との重なり: `novel-talk-flow`（`virtual_dispatcher.lua`・`init.lua`・`act.lua`・`scene.lua`）、`scene-attribute-store`（`scene.lua`）、`call-attribute-filter`・`scene-exact-match`・`cue-command-runtime`・`scene-anchor-link`（`act.lua`。`scene.lua` も一部）、`shell-element-click`（`choice_select.lua`・`init.lua`）、`novel-checkpoint-rollback`（`init.lua`・`virtual_dispatcher.lua`・`store.lua`）、`novel-variable-scope`（`virtual_dispatcher.lua` の保存の名前を動かす場合）、`recommend-menu-scenes`（`init.lua`・`scene.lua`）。
- **種別**: バグ（数でない `hour_margin` で毎秒エラー、値の改行で応答が割れる、続きのトークの `get_property` がすぐ時間切れになる）。残りは写しと死んだ分岐の整理。
- **要件定義のモデル**: Opus。決めごとは監査の指摘の直し方で、挙動を変える直しは brief で決まっている（改行は空白）。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 「Current State」の行番号: `hour_margin` は `virtual_dispatcher.lua` 121〜125・245〜247 行 → 84 行（設定の表）と 206 行（比べる所）、計算し直す行 251 → 211 行、毎秒の設定の読み直し 231 → 191 行（ほかに 100 行）。`set_co_scene` 96〜136 行 → 101〜136 行、203〜205 行 → 200・206 行。`store.lua` 77〜92 行 → 83 行あたり。`entry.lua` の `SHIORI.load` 45〜59 行 → 69〜84 行、`SHIORI.unload` 122〜131 行 → 140〜150 行。`shiori/act.lua` 291 → 292 行。
  - 「Lua の spec 66 本は通る」は古い（4 本増えた）。
  - ロードマップの台帳（`roadmap.md` 449 行）の「Dependencies: none」は、上の順番の約束（`chain-talk-interrupt`、`reg-handler-chain`）を書いていない。運用ルールの「同じファイルを触る順も Dependencies に書く」に合わせるなら足す。

## 申し送り（reg-handler-chain より、2026-10-23 完了時）

- `shiori/event/init.lua` の `EVENT.fire` の、act を作ってハンドラを呼ぶ所が、鎖のループ（ハンドラごとに `create_act` と `STOP.hold_handler`）と底の `rawget(REG, req.id) or EVENT.no_entry` に変わった。冒頭の説明も伸びたので、このファイルの行番号は測り直す。型判定の枝・`set_co_scene`・`EVENT.drive` には触れていない。
- `event/register.lua` の `REG` にメタテーブル（`__index` の `register` と、欄 `handlers`）が付いた。`REG` を空にするテストは、`package.loaded` から読み直せば鎖も空になる。
- 参照: `.kiro/specs/completed/reg-handler-chain/design.md` の「File Structure Plan」「Components and Interfaces」。
