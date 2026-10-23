# ギャップ分析: chain-talk-interrupt

- 実施: 2026-10-23（main `b2f9a5cc` の上のワークツリー）
- 対象: `requirements.md`（要件 1〜8）と、今のコード・テスト・マニュアル
- 位置の表記は `crates/pasta_lua/pasta_scripts/pasta/` からの相対パス（Lua）と、`book/src/` からの相対パス（マニュアル）

## 1. まとめ

- **直しの芯は 3 つ**。(1) その回の出力に台詞があったかの印、(2) 続きの置き換え（`set_co_scene`）の決まりの分岐、(3) `＊チェイントーク中断` の出力を、割り込んだシーンの出力の前に 1 本で組むこと。(1)(2) は既存の形に素直に乗る。(3) が設計の要点で、組み立ての順と `\e`・立ち位置・外見の状態の持ち越しをどう扱うかで案が分かれる。
- **印はトークンの段で取るしかない**。トークンは `act.lua` の `ACT_IMPL.build`（L637〜656）で空にされ、`EVENT.drive` には組み立て済みの文字列しか届かない。文字列からは表情と台詞の区別が付かない。
- **「はっきり捨てる」は 2 つのファイルで印を付け、続きの置き換えで読む**。キック（`shiori/event/kick.lua` の `wrap_local_func`・`wrap_reload_func`）とシーン再生（`shiori/event/scene_play.lua` の 2 つのハンドラ）は、どちらもコルーチンを返すだけで、ふつうの道（`EVENT.fire` → `EVENT.drive` → `set_co_scene`）を通る。
- **既存テストの多くは残る見込み**。今の続きの更新のテストは、act を通らない素のコルーチンが文字列を出す形が多い。要件 1.7（出力が空でなければ台詞に数える。2026-10-23 確定）なので、そのまま通る。
- **規模は M・リスクは中**。6〜7 タスク（brief の見積もりどおり）。リスクは (3) の組み立てに集まる。

## 2. 今の仕組み（調べた所）

### 2.1 続きの置き換え

- `shiori/event/init.lua`
  - `set_co_scene`（L101〜137）: (1) suspended でなければ `co = nil`、(2) コールバック待ちとして登録した `co`（`STORE.co_callback` と同じ）なら、`STORE.co_scene` を `nil` にして終わる（待っている別の続きも捨てる）、(3) 同じコルーチンなら何もしない、(4) 古い続きを捨てて置き換える。
  - `EVENT.drive`（L191〜208）: `resume_until_valid` で出力が出るか終わるまで再開する。失敗なら `STOP.hold`・`CALLBACK.discard_staged` の後に `set_co_scene(co)`（dead なので続きが空になる）。成功なら `CALLBACK.consume_staged` の後に `set_co_scene(co)`。
  - 呼び出し元は 3 つ: `EVENT.fire`（L239〜245）、`CALLBACK.try_route`（`callback.lua` L111）、`CALLBACK.sweep` の `time_out`（`callback.lua` L171）。
- 文字列か `nil` を返すハンドラは `set_co_scene` を通らない（`EVENT.fire` L246〜255）。要件の Out of scope（REG の文字列は続きに触れない）は今のままで成り立つ。
- 続きの再開は `shiori/event/virtual_dispatcher.lua` の `check_talk`（L213〜218）だけ。`STORE.co_scene` をそのまま返す。この場合 `EVENT.drive` に来る `co` は `STORE.co_scene` と同じものなので、「続きの再開か、割り込みか」は `co == STORE.co_scene` で見分けられる。
- 自動トークを止めている間は `dispatch`（L263〜267）が判定に進まない。捨てる決まりには触れない（`ontalk-suppression`）。

### 2.2 出力の種類と組み立て

- トークンの種類（`act.lua`）: `talk`（L241）・`sakura_script`（L251）・`raw_script`（L260）・`surface`（L269）・`wait`（L279）・`newline`（L288）・`clear`（L296）・`choice`（L308）・`choice_timeout`（L317）・`spot`（L823）・`clear_spot`（L831）。
- 失敗の表記は `raw_script`（`failure.lua` L171〜177）。未登録アクターの目印も `raw_script`。要件 1.2 のとおり台詞に数えない。
- `ACT_IMPL.build`（L637〜656）: トークンを取り出して空にし、すべて空の `raw_script` なら `nil`、そうでなければアクターごとにまとめた表を返す。
- `SHIORI_ACT_IMPL.build`（`shiori/act.lua` L65〜83）: 親の `build` の結果を `BUILDER.build` に渡し、`STORE.actor_spots`・`STORE.appearance` を直接変えながら文字列にする。
- `BUILDER.build`（`shiori/sakura_builder.lua`）: 組み立ての中の状態（直前の話し手 `last_actor`・吹き出しに字があるか `spot_has_text`・段落の区切りの保留 `pending_break`・BudouX の列）は 1 回の組み立ての中だけで持つ。最後に `\e` を付ける（L533〜536）。
- 出力が作られる所は 3 つ: シーンの包み（`scene.lua` の `wrap_scene` L186〜196。最後に `act:build()`）、区切り（`ACT_IMPL.yield` L664〜676）、コールバック待ち（`shiori/act.lua` の `get_property` L307。`\![get,property,…]` だけの `raw_script`）。
- `ACT_IMPL.yield` の先頭には、止めている間の見張り（`ontalk-suppression`）がある。

### 2.3 act とコルーチンの対応

- シーンが書き込む act は、コルーチンを最初に動かしたときの act（`wrap_scene` の `resumed_act`）。続きを再開するときに `EVENT.drive` が受け取る act は別のもの。
- `act:init_scene`（`act.lua` L208〜220）が `FAILURE.bind(self)` で「実行中のスレッド → act」を記録する。外から引くのは `FAILURE.act_of(co)`（`failure.lua` L203）。弱い参照の表なので、捨てたコルーチンは GC で消える。
- ただし、`init_scene` を通らないコルーチンは対応が無い。キックのリロード用の包み（`kick.lua` の `wrap_reload_func`）と、ゴーストが Lua で作ったコルーチン、テストの素のコルーチンが当たる。

### 2.4 「はっきり捨てる」道

- キック: `KICK.try_dispatch`（`kick.lua` L115〜161）が `wrap_local_func` か `wrap_reload_func` でコルーチンを作って返す。`virtual_dispatcher.dispatch`（L257〜261）→ `OnSecondChange` の既定ハンドラ → `EVENT.fire` → `EVENT.drive`。
- シーン再生: `REG.OnPastaScene`・`REG.OnPastaSceneAt`（`scene_play.lua` L61〜107）が `SCENE.co_exec_exact`・`SCENE.co_exec_registered` のコルーチンを返す。失敗は `act:build()` の文字列を返すので、続きに触れない（要件 4.4 は今のままで成り立つ）。
- どちらも「返したコルーチン」に印を付ける形になる。`scene_play.lua` は `event/init.lua` を require しない（循環させない）ので、印は `pasta.store` か、両方が読める小さな表に置く。`ontalk-suppression` 完了時の申し送りでは、キックの印は `kick.lua` と `STORE` に置くことになっている。

### 2.5 シーンの完全一致の検索

- `SCENE.co_exec_exact(name)`（`scene.lua` L230〜251）がそのまま使える。Rust の `search_scene_exact`（`src/search/context.rs` L119）→ `resolve_scene_id_exact`（`pasta_core/src/registry/scene_table.rs` L255〜286）は、同じ名前の候補をふつうのシーンと同じキャッシュで選ぶ（要件 3.5）。別名表の置き換えも同じく効く。
- 撫で・つつきのショートカット（`mouse.lua`）とシーン再生が、すでに同じ関数を使っている。

### 2.6 既存のテスト

- 続きの更新を見ているもの: `event_coroutine_test.lua`・`integration_coroutine_test.lua`・`callback_module_test.lua`・`second_change_thread_test.lua`・`virtual_dispatcher_thread_test.lua`・`global_chaintalk_integration_test.lua`・`scene_play_event_test.lua`（と Rust の `tests/scene_play_event_test.rs`）・`kick_*_test.lua`・`auto_talk_pause_test.lua`・`virtual_dispatcher_pause_gate_test.lua` など。
- 「古い続きが新しいシーンで捨てられる」テストの多くは、act を通らない素のコルーチンが `"new"`・`"second result"` などの文字列を出す（`event_coroutine_test.lua` L268〜298、`integration_coroutine_test.lua` L202〜240）。要件 1.7 で台詞に数えるので、今のまま通る。
- コールバックの枝: `callback_module_test.lua` L233〜235 などは、待ちに入ったシーンが続きに入らないことを見ている（今も `STORE.co_scene` は無い前提）。待ちに入るときに別の続きがあれば捨てる、を確かめるテストがあれば、要件 2.6 で直す対象になる（下の「調べが要る所」）。
- 一覧は `tests/lua_specs/init.lua` に 1 行ずつ足す。

### 2.7 マニュアルとスキル

今の「捨てる」決まりを書いている所:

- `grammar/call-jump.md` L349（外から起こした `OnTalk`）・L364（別のイベントでシーンが実行されると破棄）
- `getting-started/11-jump.md` L87（入門。ダブルクリックの反応や時報で捨てられる）。brief の一覧に無いが、同じ決まりを書いている
- `lua/shiori-events.md` L604〜607（シーン再生の「再生と続き」）・L943〜944（OnTalk の節）
- `internals/execution-model.md` L247〜270（`set_co_scene` の規則）・L283〜293（コールバック待ちとの関係）・L108・L168・L212
- `internals/shiori.md` L388（OnHour とキックが続きを置き換える）
- `internals/debug.md` L34（キックしたシーンが続きを置き換える）
- `internals/internal-modules.md` L36（`co_scene` の説明）

スキルの references は `book/tools/gen-skill-refs.mjs` で作り直す。`チェイントーク` を含む references は、`pasta-ghost-authoring` の `call-jump.md`・`authoring-patterns.md`・`runtime-failures.md`・`variables.md`、`pasta-lua-coding` の `shiori-events.md`・`script-api.md`・`internal-modules.md`。

### 2.8 pasta_check

- `pasta_check` は、イベントの決まった名前の一覧を持っていない（`validate/config.rs` の別名表の検査だけ）。LSP にも無い。`チェイントーク中断` はふつうのシーンの名前として通る。
- 組み込みの飛び先と同じ名前の検査（`scene-shadowed-by-builtin`）は、名前が違うので当たらない。
- `＞チェイントーク` の検索は 4 段目（`GLOBAL`・完全一致）が 5 段目（全グローバルシーン・前方一致）より先なので、`＊チェイントーク中断` を書いても `＞チェイントーク` は組み込みのまま（`grammar/call-jump.md` L205〜218）。ローカルシーンに `・チェイントーク中断` と書くと 2 段目の前方一致で `＞チェイントーク` を奪うが、これは今もある決まりで、この機能の範囲ではない。
- 結論: `pasta_check` は触らない（要件 8.6）。`builtins.rs` の `ACT_METHODS` も触らない（設計への申し送り）。

## 3. 要件と今の資産の対応

| 要件 | 使える資産 | ギャップ |
| -- | -- | -- |
| 1.1・1.3・1.4・1.5 続きを残す・再開 | `set_co_scene`・`co == STORE.co_scene` の見分け | **Missing**: 「別のシーンで、台詞が無い」ときに古い続きを残す分岐。区切った無言のシーンの残りを捨てる扱い（1.3） |
| 1.2 台詞に数えるもの | トークンの `type`（`talk`・`choice`） | **Missing**: その回の出力に台詞があったかの印。トークンは組み立てで消える。**Constraint**: act のメソッドにしない |
| 1.6 イベントの種類によらない | すべて `EVENT.drive` を通る | なし（`EVENT.drive` の中で決めれば全部に効く） |
| 1.7 シーンでないコルーチン | `FAILURE.act_of(co)` が `nil` を返す | **Missing**: act の無いコルーチンの扱い（出力の文字列が空かどうかで決める） |
| 2.1・2.2 台詞なら捨てる・新しい残り | 今の `set_co_scene` の (4) | なし（今の振る舞い） |
| 2.5・2.6 コールバック待ち | `set_co_scene` の (2) の枝 | **Missing**: 待ちに入るときに続きを捨てない分岐。戻った回に台詞で捨てるのは 1.x と同じ道 |
| 3.1・3.6 中断の台詞を前に 1 本で | `SCENE.co_exec_exact`・`BUILDER.build` | **Missing**: 前に添える組み立て。**Unknown**: 組み立ての順・`\e`・状態の持ち越し（4 章） |
| 3.2・3.3・3.8 流す条件・入れ子にしない | — | **Missing**: 中断を流す道の中で、中断自身を割り込みに数えない印 |
| 3.4・3.5 完全一致・同名から選ぶ | `SCENE.co_exec_exact`（シャッフルのキャッシュ込み） | なし |
| 3.9・3.10 中断の中の区切り・エラー | `EVENT.drive` の失敗の扱い・`STOP` | **Missing**: 中断のコルーチンの残りを捨てる扱いと、エラーの伝え方 |
| 4.1〜4.3 はっきり捨てる | `kick.lua`・`scene_play.lua` のコルーチンを返す所 | **Missing**: 返すコルーチンへの印と、続きの置き換えでそれを読む分岐 |
| 4.4・4.5 再生しないときは触れない | 今の振る舞い（文字列か `nil`） | なし |
| 5.1〜5.3 エラー | `EVENT.drive` の失敗の道 | **Constraint**: 失敗の道では中断を流さない。台詞を出した後のエラーも同じ |
| 6.1 外から起こした `OnTalk` | `EVENT.no_entry` → `SCENE.co_exec` | なし（ふつうのイベント） |
| 6.2 `OnPastaCallFunction` | まだ無い（`external-control-events`） | **Unknown**: あちらのハンドラがコルーチンを返す形なら、ふつうの道で効く |
| 7 テスト | 2.6 のテスト群・`lua_test` のモック | **Missing**: 新しいテスト。**Unknown**: 直すテストの数（要件 1.7 は確定済みで、少ない見込み） |
| 8 マニュアル・スキル | 2.7 の各所・`gen-skill-refs.mjs` | **Missing**: 書き直し。**Constraint**: スキルの references の書き込みが分類器に止められることがある（6 章） |

## 4. 実装の案

### 4.1 台詞の印（要件 1.2）

- **案 1-a: 組み立ての時に印を立てる**。`ACT_IMPL.build`（か `SHIORI_ACT_IMPL.build`）で、取り出したトークンに字のある `talk` か `choice` があれば、act の欄（例: `act._spoke = true`）に印を立てる。`EVENT.drive` は、再開の前に印を下ろし、再開の後に `FAILURE.act_of(co)` の act の印を読む。
  - ✅ 実際に出した回の出力だけを見る。`get_property` がトークンを退避する（L297〜310）場合も、退避した台詞はまだ出ていないので数えない。
  - ✅ act の欄なので `ACT_METHODS` は変わらない。
  - ❌ `init_scene` を通らないコルーチンは act を引けない。要件 1.7 の扱いで補う。
- **案 1-b: トークンを積む時に印を立てる**（`ACT_IMPL.talk`・`ACT_IMPL.choice`）。
  - ❌ 積んだが出していない台詞（`get_property` の退避・最後の組み立ての前のエラー）まで数える。1-a の方が要件に合う。
- **案 1-c: 出力の文字列から推す**。
  - ❌ 表情・`\![bind,…]` と台詞を区別できない。要件 1.2 を満たせない。act の無いコルーチンの代わり（要件 1.7）にだけ使う。

### 4.2 続きの置き換えの決まり（要件 1・2・4・5）

`set_co_scene` に「割り込みの種類」を渡す形が素直である。判断の表:

| 状況 | 続き | 中断の台詞 |
| -- | -- | -- |
| `co` が待っている続きそのもの（再開） | 今どおり置き換え | 流さない |
| 待っている続きが無い | 今どおり | 流さない |
| エラーで止まった | 捨てる | 流さない |
| キック・シーン再生の印がある | 捨てる（区切れば新しい残りを置く） | 流さない |
| 別のシーン・台詞あり | 捨てる（区切れば新しい残りを置く） | 流す |
| 別のシーン・台詞なし・最後まで動いた | 残す | 流さない |
| 別のシーン・台詞なし・区切った | 残す（新しい残りは捨てる。確定） | 流さない |
| 別のシーン・台詞なし・コールバック待ちに入った | 残す（確定） | 流さない |

- 書き換えは `event/init.lua` の L97〜208 の中に収まる（brief の約束）。`EVENT.fire` は触らない。
- `coroutine.close` の分岐は残したまま（整理は `ponytail-lua-scripts`）。

### 4.3 `＊チェイントーク中断` の組み立て（要件 3）— 設計の要点

割り込んだシーンが台詞を出したかは、そのシーンの出力を組み立てた後でないと分からない。中断の台詞は前に置く。今の組み立ては 1 回ごとに状態を持ち、最後に `\e` を付ける。この 3 つの食い違いをどう解くかで案が分かれる。

- **案 A: 文字列でつなぐ**（`EVENT.drive` の中）
  - 割り込んだシーンを組み立てた後、中断のシーンを別のコルーチンで動かして文字列を得る。中断の文字列の末尾の `\e` を外し、前につなぐ。
  - ✅ `event/init.lua` の中だけで済む（約束どおり）。
  - ❌ 組み立ての状態が逆順になる。`STORE.actor_spots`・`STORE.appearance` は、表示の順（中断 → 反応）と逆に、反応 → 中断の順で書き換わる。中断の後の表情の持ち越しがずれうる。
  - ❌ 反応の側の組み立ては、中断の台詞が同じ吹き出しにあることを知らない。同じアクターが続くと、段落の区切りの改行が入らずに字がくっつく。話し手の無い先頭のグループ（区切りの直後の表情など）は、中断の最後のスコープに出る。要件 3.6 を満たすには、つなぎ目に区切り（スコープの明示や改行）を足す細工が要る。
- **案 B: トークンでつなぐ**（組み立てを 1 回にする）
  - 割り込んだシーンの、この回の出力をトークン（まとめた表）の段で止めておき、台詞があれば中断のシーンのトークンを前に足してから、1 回だけ文字列にする。
  - ✅ 組み立ての状態・`\e`・段落の区切り・外見の持ち越しが、1 本のシーンと同じ規則で正しく出る（要件 3.6 をそのまま満たす）。
  - ❌ 出力を作る 3 か所（`wrap_scene`・`yield`・`get_property`）は、組み立てた文字列を `coroutine.yield`／`return` で渡している。トークンのまま `EVENT.drive` へ渡すには、出力の受け渡しの形が変わる。触る所が増え、`ponytail-lua-scripts`・`cue-command-runtime`・`novel-talk-flow` とぶつかりやすい。
- **案 C: 組み立ての中で差し込む**（`SHIORI_ACT_IMPL.build` の中）
  - `EVENT.drive` が、再開の前に「割り込みとして中断の台詞を流してよい」状況（待っている続きがあり、`co` が続きでなく、はっきり捨てる印が無い）を印にしておく。`SHIORI_ACT_IMPL.build` は、親の `build` の結果に台詞があり、その印が立っていれば、中断のシーンを動かしてトークンを得て、前に足してから `BUILDER.build` を 1 回だけ呼ぶ。中断のシーンは、文字列にしない act（親の `ACT` の `build` はまとめた表を返す）で動かせばトークンが得られる。
  - ✅ 組み立ては 1 回で、案 B と同じく状態が正しい。
  - ✅ 出力の受け渡しの形は変えない。触るのは `shiori/act.lua` の `build` と、`event/init.lua` の約束の範囲。
  - ❌ 中断のシーンを、割り込んだシーンのコルーチンの中から入れ子で動かす。LuaJIT では入れ子の `resume` はできるが、中断のシーンの中の `＞チェイントーク`（要件 3.9）とエラー（要件 3.10）、`FAILURE.bind` の対応、止まるエラーの控え（`STOP.hold`）の扱いを設計で詰める必要がある。
  - ❌ 印を読む所（`build`）と、続きを捨てる所（`set_co_scene`）が分かれる。両方が同じ状況の判断を使うように、判断を 1 か所にまとめる必要がある。
- 傾き（決定ではない）: 要件 3.6 を細工なしで満たせるのは案 B と案 C。触る範囲の狭さでは案 C が有利。案 A は、つなぎ目の細工で 3.6 を満たせるかを設計で確かめてから選ぶ。

### 4.4 はっきり捨てる印（要件 4）

- **案 4-a: 返すコルーチンを弱い参照の表に記録する**（`STORE` か、`kick.lua`・`scene_play.lua` が読める小さなモジュール）。`set_co_scene` が `co` で引く。
  - ✅ 捨てたコルーチンは GC で消える。2 回目以降のビート（続きの再開）は「続きそのもの」として扱われるので、印は 1 ビート目にしか効かない。
- **案 4-b: 「次の再開ははっきり捨てる」旗を `STORE` に 1 つ立てる**。
  - ❌ 再開の前に別のイベントが割り込むと、旗が別のシーンに効く。4-a の方が取り違えない。

### 4.5 規模とリスク

- **規模: M**（3〜7 日）。brief の見積もり 6〜7 タスク（台詞の印 1、続きの置き換え 1、中断の台詞の組み立て 1〜2、はっきり捨てる道 1、テスト 1、マニュアルとスキル 1）。
- **リスク: 中**。台詞の印・続きの置き換え・はっきり捨てる印は、既存の形の延長でリスクが低い。中断の台詞の組み立て（4.3）が、組み立ての状態・入れ子のコルーチン・エラーの扱いにまたがり、ここにリスクが集まる。

## 5. 設計への申し送り

### 5.1 決めること

- 中断の台詞の組み立て（4.3 の案 A・B・C）。要件 3.6（1 本のスクリプト・話し手と表情が崩れない）を、どの案でどう満たすか。
- 中断のシーンを動かす act の種類と、`FAILURE.bind`・`STOP.hold` との関係（エラーの表記の「止まった場所」が中断のシーンを指すか）。
- 中断のシーンの残り（要件 3.9）を確実に捨てる方法。待っている続きの置き場所に入れない。
- はっきり捨てる印の置き場所（4.4）。`ontalk-suppression` の申し送りどおり `STORE` に置くか。
- 要件 1.7 の扱い（act の無いコルーチン）を、`FAILURE.act_of(co)` が `nil` のときの枝として書くか。

### 5.2 調べが要る所（Research Needed）

- 今のテストのうち、「台詞の無いシーンでも続きが捨てられる」を前提にしたものの洗い出し。特にコールバックの枝（待ちに入るときに別の続きを捨てる）を見ているテストがあるか。`callback_module_test.lua`・`get_property_test.lua`・`second_change_thread_test.lua` を全部読む。
- 中断の台詞を前に足したとき、`sakura_builder.lua` の段落の区切り（`spot_newlines`）と選択肢の前後の改行（`choice_newlines`・`after_choice_newlines`）が、1 本のシーンと同じに出るか（案 B・C）。
- 反応の側の出力の先頭が、話し手の無いグループ（表情だけ）だったときの出る位置（案 A を選ぶ場合）。
- `scene_play_event_test.rs`（Rust の結合テスト）が、台詞の無い再生シーンで続きを捨てる形を見ているか。要件 4 では捨てるままなので、通る見込み。
- `OnPastaCallFunction`（`external-control-events`）のハンドラが、呼び出しをコルーチンで包んで返す形になるか（あちらの brief の L129）。なれば、ふつうの道で要件 6.2 が効く。

## 6. 進め方の注意

- **同じウェーブの約束**: `event/init.lua` は L97〜208 と、その間の新しい内部の関数だけを触る（`external-control-events`・`reg-handler-chain` と並べるため）。案 C を選ぶと、中断を差し込む本体は `shiori/act.lua` に入る。約束の「`EVENT.drive` か、それが呼ぶ関数の中で行う」に当たるかを、設計で明記しておく。
- **スキルの references**: `.claude/skills/` への書き込みは、分類器に Self-Modification として止められることがある（SKILL.md の編集で確認済み）。`gen-skill-refs.mjs` による作り直しも止められたら、回避を探さずに、入った分と残りを示して開発者に許可を求める。
- **テストの書き換え**: `pasta_lua` のテストは `sample.generated.lua` の改行コードを書き換えることがある。コミットに混ぜない。
- **マニュアル**: 入門（`getting-started/11-jump.md`）にも同じ決まりが書いてある。brief の一覧に無いので、書き直しの対象に足す。マニュアルには回避のレシピを載せない（部品の能力だけを書く）。

## 7. 設計の調査と決定（kiro-spec-design、2026-10-23）

### 7.1 まとめ

- **種類**: 既存の仕組みの拡張（Extension）。軽い調査（design-discovery-light）で、つなぎ目の所だけ深く読んだ。外の依存は足さない。
- **使ったスキルと規則**: `kiro-spec-design`（design-principles・design-discovery-light・design-synthesis・design-review-gate）、`kiro-validate-design`（design-review）。ステアリングは `product.md`・`tech.md`・`structure.md`（Lua の循環を作らない・`store` は何も読まない・テストは `tests/lua_specs/`）。
- **決めたこと**: 中断の台詞は「組み立ての中で差し込む」（4.3 の案 C）。流し方の本体は `event/init.lua` の約束の範囲に置き、`EVENT.drive` が act の欄に関数を掛け、`SHIORI_ACT_IMPL.build` が台詞を見つけたときに呼ぶ。組み立ては 1 回のまま。
- **見つけたこと**: 表情の単語は `talk` のトークンになる（`＠笑顔` → 字が `\s[5]`）。台詞の判定は「字のある talk」でなく、組み立ての `has_text` と同じ「字が出るか」で取る必要がある。

### 7.2 調べたこと

#### 表情の単語のトークン

- 調べた所: `tests/fixtures/sample.generated.lua`（`act:actor_proxy("さくら"):talk(act:actor_proxy("さくら"):word("笑顔"))`）、`tests/fixtures/sample.pasta`（`＠通常：\s[0]`）、`pasta/actor.lua` の `PROXY_IMPL.word`。
- 分かったこと: 表情の単語は `talk` のトークンで、字はさくらスクリプトのタグだけになる。`text ~= ""` で数えると、表情だけのシーンが台詞に数えられ、要件 1.2 に反する。
- 設計への影響: 台詞の判定を `sakura_builder.lua` に置き、`has_text`（組み立ての段落の区切りと同じ判定）を使う。`BUILDER.speaks` として公開する。

#### 組み立てのつなぎ目

- 調べた所: `shiori/sakura_builder.lua` の `BUILDER.build`（話し手の切り替え・`pending_break`・`spot_has_text`・`spot_tail`・列）、`act.lua` の `group_by_actor`・`merge_consecutive_talks`、`grammar/action-line.md`（同じアクターの行は続けて出る）。
- 分かったこと:
  - まとめたトークン列を 2 つつないで 1 回で組み立てると、表情と立ち位置の持ち越し（`STORE.appearance`・`STORE.actor_spots`）が表示の順に書き換わる。案 A（文字列でつなぐ）では逆順になり、中断の台詞で変えた表情を、割り込んだシーンの組み立てが知らない。
  - 同じ話し手のまま 2 つをつなぐと、切り替えのタグも段落の区切りも出ず、字が続けて出る。割り込んだシーンの頭が話し手の無いグループ（選択肢だけ・表情だけ）だと、中断の最後のスコープに出る。
- 設計への影響: つなぎ目のトークン `seam` を置き、組み立てがそこで「直前の話し手とスコープを空に戻す（0 以外なら `\p[0]` を出す）」。割り込んだシーンの頭は、中断が無いときと同じ切り替えで出て、同じ吹き出しなら段落の区切りが 1 つ入る。

#### act とコルーチンの対応

- 調べた所: `failure.lua` の `FAILURE.bind`・`act_of`、`act.lua` の `init_scene`、`scene.lua` の `wrap_scene`、`event/init.lua` の `EVENT.fire`（`EVENT.drive(result, act, act)`）、`callback.lua`（`entry.act`）。
- 分かったこと: 新しいシーンは、最初の再開の前には `act_of(co)` が無い。最初の再開の引数の act が、シーンの書き込む act になる。続きとコールバックでは `act_of(co)` が引ける。
- 設計への影響: 再開の前は `FAILURE.act_of(co) or act`、後も同じ式で act を引く。

#### 止まるエラーの控え

- 調べた所: `shiori/stop.lua` の `STOP.hold`・`STOP.response`・`build_value`。
- 分かったこと: 控えは `FAILURE.act_of(co)` の act を使い、表記の止まった場所（`act.scene_name`）と行の対応（`act.line_map`）をそこから取る。`STOP.response` は後で `act:build()` を呼んで、止まるまでの出力に表記を足す。
- 設計への影響:
  - 中断のシーンがエラーで止まったら、投げ直す前に `FAILURE.bind(iact)` で、いま動いている割り込んだシーンのコルーチンを中断の act に結び直す。結び直さないと、中断のエラーの行を割り込んだシーンの行の対応で読み、違う場所を表記しうる。
  - `EVENT.drive` は、act の欄の関数を再開の後すぐに外す。外さないと、止まるエラーの応答の `act:build()` が中断のシーンを動かす（要件 5.1 に反する）。

#### 選択肢の検索の親

- 調べた所: `choice_select.lua` の `OnChoiceSelectEx`（`STORE.last_global_scene` は Reference2 が使えないときの予備）、`act.lua` の `init_scene`。
- 分かったこと: 中断のシーンを動かすと、`STORE.last_global_scene` は中断のシーンの名前になる。選択肢のトークンは出した時点のシーン名を持ち、Reference2 で戻る。
- 設計への影響: 戻さない（書く所を増やさない）。

#### 既存のテストの洗い出し（5.2 の調べが要る所）

- 読んだ所: `event_coroutine_test.lua`・`integration_coroutine_test.lua`・`callback_module_test.lua`・`second_change_thread_test.lua`・`get_property_test.lua`・`shiori_entry_test.lua`・`scene_play_event_test.lua`・`kick_*_test.lua`・`virtual_dispatcher_*_test.lua`・`auto_talk_pause_test.lua`・`global_chaintalk_integration_test.lua`・`store_coroutine_test.lua`、Rust の `tests/scene_play_event_test.rs`。
- 分かったこと:
  - `STORE.co_scene` に続きを置いてから別のシーンを動かすテストは、素のコルーチンが空でない文字列を出す形（`"new"`・`"second result"`）か、エラーの形か、キック・シーン再生の形である。要件 1.7・4・5 で今と同じ結果になる。
  - コールバック待ちに入るときに別の続きを捨てることを確かめるテストは無い（`get_property_test.lua` は `co_scene` を見ない）。
  - **直す必要があるのは `shiori_entry_test.lua` の 2 本**。「コールバック側のシーンが中断すれば、既存の継続を置き換える」と「コールバック側のシーンが終われば、既存の継続を空にする」。戻ったシーンが `raw_script` だけを出すので、新しい決まりでは別のシーンの続きが残る。
  - キックの試験（`virtual_dispatcher_kick_hook_test.lua`・`kick_position_path_inheritance_test.lua`）は `dispatch` が返すコルーチンだけを見て、`EVENT.drive` を通らない。変わらない。
  - Rust の `scene_play_event_test.rs` は、再生したシーンが古い続きを置き換えることを見ている。はっきり捨てる印で、今と同じ。
- 設計への影響: 直すテストは 2 本。`reg-handler-chain` が触る `event_coroutine_test.lua`・`second_change_thread_test.lua` は触らない。

#### OnPastaCallFunction の形

- 調べた所: `.kiro/specs/completed/external-control-events/brief.md` L129・L153。
- 分かったこと: あちらは呼び出しをコルーチンで包んで返す形を前提にしている。
- 設計への影響: 要件 6.2 は、あちらがコルーチンを返す限り、ふつうの道で効く。文字列を返す形になったら再確認する（Revalidation Triggers）。

### 7.3 案の比較（4.3 の案の決着）

| 案 | 中身 | 強み | 弱み | 判断 |
| -- | -- | -- | -- | -- |
| A 文字列でつなぐ | `EVENT.drive` で中断を別に動かし、`\e` を外して前につなぐ | `event/init.lua` だけで済む | 表情・立ち位置の持ち越しが逆順。段落の区切りと話し手の無い頭の位置に細工が要る。3.6 を満たしにくい | 採らない |
| B トークンでつなぐ | 出力をトークンのまま `EVENT.drive` へ渡し、1 回で組み立てる | 状態が正しい | 出力の受け渡しの形（3 か所の `yield`・`return`）が変わる。ゴーストが Lua で `act:build()` の文字列を使う所も壊れる | 採らない |
| C 組み立ての中で差し込む | `EVENT.drive` が act の欄に関数を掛け、組み立てが台詞を見つけたら呼んで前に足す | 組み立ては 1 回。受け渡しの形は変えない。本体は `event/init.lua` に置ける | 入れ子のコルーチン。エラーの止まった場所の結び直しと、欄の外し忘れに注意 | **採る** |

### 7.4 決定

#### 決定: 中断の台詞を組み立ての中で差し込む

- **背景**: 割り込んだシーンが台詞を出したかは、組み立ての時にしか分からない。中断の台詞は前に置く。組み立ては 1 回ごとに状態を持つ。
- **選んだ形**: 案 C。`EVENT.drive` が割り込みのときだけ `act._on_speech = run_interrupt_scene` を掛ける。`SHIORI_ACT_IMPL.build` は台詞を見つけたら欄を外してから呼び、返ったまとめたトークン列と `seam` を前に足して 1 回で組み立てる。
- **理由**: 3.6 を細工なしで満たす。受け渡しの形を変えない。本体を同じウェーブの約束の範囲に置ける。
- **引き換え**: `shiori/act.lua`・`sakura_builder.lua` を小さく触る。入れ子のコルーチンのエラーの扱いを詰める必要がある。
- **後で確かめること**: 中断のエラーの止まった場所、止まるエラーの応答に中断が混ざらないこと、`\e` が 1 つだけのこと。

#### 決定: 台詞の印を act の欄に置く

- **選んだ形**: `_spoke`（`boolean|nil`）。`EVENT.drive` が再開の前に `nil`、組み立てが `self._spoke or 台詞あり`。`nil` のままなら act を通らないコルーチンとして出力の有無で決める（1.7）。
- **理由**: act のメソッドにしない（`ACT_METHODS`・`SHIORI_ACT_METHODS` と実行時の一致のテスト `validate_parity_test.rs` は `IMPL` の関数だけを見るので、インスタンスの欄は関わらない）。`pasta/act.lua` を触らない。

#### 決定: はっきり捨てる印を `STORE.co_preempt` に置く

- **選んだ形**: キーを弱い参照にした表。キックの 2 つの包みとシーン再生の 2 つのハンドラが付け、`drive_kind` だけが読む。
- **理由**: `scene_play.lua` は `event/init.lua` を読まない（循環させない）。`ontalk-suppression` の申し送り（キックの印は `kick.lua` と `STORE`）のとおり。「次の再開は捨てる」旗を 1 つ立てる形（4.4 の案 4-b）は、別のイベントが先に来ると取り違える。

#### 決定: 続きの置き換えに `keep` の引数を 1 つ足す

- **選んだ形**: `set_co_scene(co, keep)`。`keep` は「割り込み・台詞なし・成功」のときだけ真。種類は `drive_kind(co)` で再開の前に決める（続きなし → 再開 → はっきり捨てる → 割り込みの順）。
- **理由**: 今の規則（コールバックの印・同じコルーチン・置き換え）を残したまま、残す場合だけを足せる。表「続きの決まり」の全部の行がこの 2 つで決まる。

### 7.5 合成（design-synthesis）

- **一般化**: キック・`OnPastaScene`・`OnPastaSceneAt` は「開発者が指名した再生」として 1 つの印にまとめた。新しい指名の再生を足すときも、印を付けるだけで済む。イベントの種類（撫で・時報・外から起こした `OnTalk`・コールバック・`OnPastaCallFunction`）は分けず、すべて `EVENT.drive` の 1 か所で決める。
- **採用か自作か**: 外のライブラリは使わない。今ある部品を使う。完全一致の検索と同名からの選択は `SCENE.co_exec_exact`、act の引き当ては `FAILURE.act_of`、字の判定は `has_text`、止まるエラーの応答は `STOP` の道。
- **簡素化**:
  - 中断のシーンの残りを `STORE` に置かない。参照を外すだけ。
  - `STORE.last_global_scene` を戻さない。
  - つなぎ目は新しいトークン 1 つで表し、組み立てに専用の設定を足さない。
  - 中断のシーンの名前は固定の文字列。設定・別名表・`pasta_check` の組み込みの名前には足さない。
  - act を通らないコルーチンには中断の台詞を流さない（前提。未決事項 1）。

### 7.6 リスクと手当て

- 入れ子のコルーチンのエラーで、止まった場所を取り違える → 投げ直す前に `FAILURE.bind(iact)` で結び直し、テストで止まった場所の名前を確かめる。
- act の欄の関数の外し忘れで、止まるエラーの応答に中断が混ざる → `EVENT.drive` が再開の後すぐに、成否にかかわらず外す。テストで確かめる。
- ゴーストの Lua が、シーンの途中で `act:build()` を自分で呼んで文字列を使う → 台詞があれば、そこで中断の台詞が付く（その回の出力の組み立てと同じに扱う）。まれな書き方なので手当てはしない。
- マニュアルのぶつかり → `lua/shiori-events.md` は `external-control-events`（表と新しい節）・`reg-handler-chain`（L81〜95・L745）と同じページ、`internals/execution-model.md` は `reg-handler-chain`（L161）と近い。触る節を分け、後から入る側が rebase で合わせる。references は rebase の後に作り直す。
- スキルの references の書き込みが分類器に止められる → 回避を探さず、入った分と残りを示して開発者に許可を求める（6 章のとおり）。

## 8. 設計ディスカッションの決定（2026-10-23）

- 5 つの議題を、すべて推奨案で確定した（開発者の承認）。内容は `design.md` の「決めたこと」にある。
- 下書きの前提から変わったのは 1 つ（中断の中の区切りと待ち）。
  - 出力の無い区切りを飛ばすのは、ふつうのシーンの再開（`resume_until_valid`）と同じ決まりにそろえるためである。
  - 待ちに入った回に中断の台詞を付けないのは、`SHIORI_ACT_IMPL.get_property` が待つ前にトークンをよけて get のタグだけを出すためである。そのまま出すと、宛先の待ちが無いタグだけが SSP に届く。待ちに入ったかは、消費されていない予約があるかで見る。そのため `CALLBACK.has_staged()` を足す（`callback.lua` は同じウェーブの spec が触らないことを確かめた）。
- 議題 2 は、開発者の指摘で要件ごと改めた（中断の台詞を反応の前に添える → 反応の代わりに出す）。
  - 理由（開発者）: 中断の台詞に反応を続けても、会話がうまくつながらない。
  - 比べた代わり: 反応を出さないと、メニュー（ダブルクリック）や `OnClose` の回に、メニューが開かず終了もしない。ただし続きは捨て済みなので、もう一度操作すれば今どおり反応する。開発者はこれを受け入れ、中断の台詞を優先と決めた。
  - 設計への影響: つなぎ目（`seam`）の組み立ては要らなくなった。差し替えは `SHIORI_ACT_IMPL.build` の中で、割り込んだシーンのトークンを文字列にする前に行う。そのため、出さなかった表情・立ち位置が `STORE` に残らない。中断の台詞が出た回は、`EVENT.drive` が `_interrupted` を読んで続きを空にする（要件 3.12 を足した）。

