# 暫定設計の突き合わせ: 口と持ち主（2026-11-19）

Phase 13 の 8 本の暫定設計（各 spec の `provisional-design.md`）を、口（関数・データの形・文法・タグ・イベント・設定のキー）と、同じファイルを触る所で突き合わせた記録である。8 本は別々に書かれたので、食い違いが在るのは当たり前で、それを並べるのが目的である。

- 正とした物: `final-model-2026-11-19.md`（以下「骨子」）。作者の目線は `play-script-guide.md`。
- 読んだ物: 骨子と手引きの全文。設計は、`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback`・`novel-save-slots`・`novel-areka-contract`・`shell-element-click` を全文。`hello-novel-sample` は 4〜7 章と 9〜12 章、`manual-novel-guide` は 3〜11 章を読んだ（調査の記録の章は読んでいない）。検証（`provisional-design-validation.md`）は、`novel-talk-flow`・`novel-checkpoint-rollback`・`cue-command-runtime`・`novel-save-slots` の見出しと、spec をまたぐ項目だけを読んだ。
- コードは読んだだけである。ビルドもテストも走らせていない。書いたのは、このファイルだけである。
- 設計のファイルは、spec 名と行番号で引く（例「talk-flow 465-472」は `novel-talk-flow/provisional-design.md` の 465〜472 行）。
- 区分: **cross-spec**＝設計どうしの食い違い・すき間。**model**＝骨子そのものの食い違い・不足。**within-spec**＝ 1 本の中の問題で、その検証も拾っていない物。
- 直し方は、開発者の決定を動かさない最小の案を書いた。決めてはいない（6 章に、決まっていない事を並べた）。

---

## 1. まとめ

| 重さ | 件数 | 中身 |
| ---- | ---- | ---- |
| 重大 | 4 | 「着いた時」の口が 3 通り（IF-01）・1 本の区切りと早回しが 3 通り（IF-02）・ト書きの頭で区切る役が居ない（IF-03）・ロードの道が 2 通り（IF-04） |
| 大きい | 14 | スロットの形・ID・アクター状態の置き場・出し直しと曲・再生位置の記録・戻った時のモード・`！select` の単位・設定の間の今のシーン・右クリックの口・見本の割り方・作例の引数・見出しの変数・設定を切り替える口・roadmap の順 |
| 小さい | 6 | 別名表の形・探す部品の名前・組み込みの名前の数・マニュアルの検査の数・組み立ての頭と末尾の順・字の無い 1 本で始まるイベントのシーン |

いちばん大きい見立ては 1 つである。**`cue-command-runtime`・`novel-talk-flow`・`novel-checkpoint-rollback` の 3 本は、同じ 3 つの時点（シーンに着いた時・1 本を区切る時・早回しの間）に、それぞれ自分の形の口を置いている。** 3 本を 1 つずつ要件定義へ進めると、後の spec が先の spec を作り直すことになる。3 つの時点の口の形を、先に 1 つに決めれば、残りの食い違いの多くは名前合わせで済む。

---

## 2. 口の対応表

「渡す」は、その設計が自分で作ると書いた物。「求める」は、ほかの設計に作ってほしいと書いた物。

### 2.1 シーンに着いた時

| 項目 | cue-command-runtime（渡す） | novel-talk-flow（求める） | novel-checkpoint-rollback（求める） |
| ---- | --------------------------- | ------------------------- | ----------------------------------- |
| 呼ばれる関数 | `CUE.arrive(act, global_name, local_name)` を `init_scene` から（cue 393・427） | `PLAY.arrive(act, here, mode)`（talk-flow 400-405） | `CHECKPOINT.arrive(act, ref) -> string|nil`（rollback 239-245） |
| 柱との順 | 柱を積んだ後に `on_arrive` の知らせ（cue 427） | 柱を積む前に 1 回（talk-flow 826） | 柱を打った後（rollback 640） |
| 走り方の渡し方 | `CUE.define_pillar("再生モード", fn(act, args, ctx))`。書いたシーンでだけ呼ぶ（cue 577） | 引数 `mode`（無ければ nil）。`PLAY_MODE_KEYS` は「柱として流さない名前」（talk-flow 829） | `PLAY.mode(ref)` を引く（rollback 625） |
| シーンの識別 | `act.arrival = {global_name, local_name}`（登録名。cue 427・542） | `PlayHere = {global, local_, gname, lname, key}`（talk-flow 383-388） | `ref = {scene, local_scene, scene_written, local_written, args}`（rollback 243） |
| 着いた時の引数 | 渡さない | 使わない | `args` が要る（rollback 161・716） |
| 戻り値 | 無い（知らせ） | 無い | 文字列なら、呼んだ側が `coroutine.yield` する（rollback 244） |
| 着いたかの判じ | `restore_scene` の深さ `act._call_depth`（cue 265-275） | cue に任せる（talk-flow 832） | cue に任せる。加えて `ARRIVAL.next_is_arrival(act)`・関数から登録名の逆引き `STORE.scene_ids`（rollback 641-642） |

### 2.2 1 本の区切り・枠・ID

| 項目 | novel-talk-flow | novel-checkpoint-rollback | cue-command-runtime | novel-areka-contract |
| ---- | --------------- | ------------------------- | ------------------- | -------------------- |
| 区切る関数 | `PLAY.seal(act)` が `coroutine.yield(act:build())`（talk-flow 465-472）。枠は `build` の中で付く（518-529） | 「組み立ての後・yield の前」に `CHECKPOINT.seal(act, script, spoke) -> string|nil` を呼んでほしい（rollback 247-253・624） | ト書きを積む `CUE.push` は `PLAY.seal` を呼ばない（cue 400） | — |
| ト書きの頭の区切り | cue が `PLAY.seal` を呼ぶ前提（talk-flow 825・941） | — | 約束していない（cue 582「区切りの数え方に入らない」） | — |
| 続きの ID | `STORE.play.seq` の十進（talk-flow 599） | `CHECKPOINT.current_id()` を使ってよい（rollback 283-285） | — | 例は、目印と続きが同じ ID（contract 296） |
| 目印の ID | 持たない（置く所だけ空ける。talk-flow 845） | セッションの頭＋通し番号 `pm3k9q2n42`（rollback 228） | — | 英数字だけ（contract 39） |
| 目印を付ける所 | `PLAY_EVENT.frame` の中（talk-flow 845） | 組み立ての外で、文字列の頭（rollback 329・774） | — | 1 本の頭（contract 284） |
| 戻る時の 1 本目の末尾 | `PLAY_EVENT.again_script` が組む（talk-flow 850） | `PLAY.next_tail()` を求める（rollback 347・626） | — | `\![raise,イベント,ID]`（contract 328） |
| 時間で次へ | `PLAY.set_line_timeout(act, ms)` を呼んでほしい。`act._line_timeout`（talk-flow 428-431・740） | — | `act._cue_xtimeout` に書く（cue 400・579） | `\![play,xtimeout,ms]` は末尾の `\x` の前（contract 287）。talk-flow は頭に置く（590） |
| 瞬間の命令を抜く | トークンの `instant = true` を見て抜く（talk-flow 600・835） | — | `BUILDER.build` の `config.cue_instant = false`（cue 466・497）。トークンは `def.kind`（386） | — |

### 2.3 早回し

| 項目 | novel-talk-flow | novel-checkpoint-rollback | cue-command-runtime | novel-save-slots |
| ---- | --------------- | ------------------------- | ------------------- | ---------------- |
| 進め方 | `act._play_target = n` の間、`PLAY.seal` が組み立てずにトークンを捨てる（talk-flow 851） | `CHECKPOINT.seal` が nil を返して yield させない。組み立ては済んでいる（rollback 315-319） | 組み立てずに捨ててほしい（覚えた値は組み立てで確定するため。cue 597） | — |
| 早回しの間かの述語 | 無い | `CHECKPOINT.replaying()`（rollback 279-281） | 受け手が無視する前提（cue 598） | `CHECKPOINT.fast_forwarding()` を求める（slots 317） |

### 2.4 画面の状態・アクター状態

| 項目 | cue-command-runtime | novel-checkpoint-rollback | novel-talk-flow |
| ---- | ------------------- | ------------------------- | --------------- |
| 位置・重ね順・大きさ・透明度の今の値 | `STORE.cue_state`（キーは `頭@対象`。cue 432-448） | `STORE.actor_attrs`。「唯一の置き場所」。cue が `ACTOR_STATE.set` で書く前提（rollback 128-130・634） | — |
| 並び | `STORE.cue_state["play,cast@"]`（cue 447） | `STORE.actor_spots` を並びとして読む（rollback 413・731） | — |
| 写す・置き直す | `CUE_STATE.snapshot_actor()`・`restore_actor(tbl)`（cue 442-443） | `ACTOR_STATE.snapshot()`・`apply(state)`（rollback 385-392） | — |
| 全部書く | `CUE_BUILD.full_state(opts) -> string`。SSP は空（cue 459・500-503） | `CHECKPOINT.state_script(ref, before) -> string`。中で `CUE.pillar_script`・`CUE.cast_script`・`CUE.actor_attr_script` を呼ぶ（rollback 300-305・643-645） | `ROLLBACK.state_tokens(act)`（トークン）と `STAGE.pillar_tokens(act, here)` を求める（talk-flow 837・855） |
| 階層の柱を打つ | `CUE.apply_pillars(act, g, l, {notify=false})`（cue 591） | `CUE.pillar_script(pos, {full=true})`（rollback 643） | `STAGE.pillar_tokens` |
| 舞台を閉じた時 | `CUE_STATE.clear(pred)`（cue 444） | `ACTOR_STATE.on_stage_closed()` を cue が呼ぶ（rollback 635） | — |
| ベースウェアの判定 | `BASEWARE.kind()`・`at_least()`（cue 508-512） | — | `STAGE.is_areka(act)` を求める（talk-flow 836） |

### 2.5 控え・スロット・ロード

| 項目 | novel-checkpoint-rollback（渡す） | novel-save-slots（求める） |
| ---- | --------------------------------- | -------------------------- |
| スロット変数を写す・戻す | `SLOT_VARS.snapshot()`・`SLOT_VARS.replace(snap)`（`pasta/slot_vars.lua`。rollback 427-432） | `CHECKPOINT.copy_slot_vars()`・`CHECKPOINT.replace_slot_vars(tbl)`（slots 313-314） |
| いちばん新しい控え | `CHECKPOINT.latest()`。走っている足取りの pending、無ければ列の最新（rollback 271-274） | 物語が待っていない時は、控えが残っていても nil（slots 312） |
| 全部捨てる | `CHECKPOINT.clear()`（rollback 277） | `CHECKPOINT.discard_all()`（slots 316） |
| ロードの入口 | `return CHECKPOINT.jump(act, cp, {load=true})` と `finish_short`（rollback 659） | `CHECKPOINT.adopt(snapshot) -> id` の後に `\![raise,OnPlayRollback,id]` を積む（slots 103・315） |
| 形の検め・イベント名 | 無い | `CHECKPOINT.validate(snapshot)`・`CHECKPOINT.EVENT`（slots 318-319） |
| 控えの形 | `{v, id?, pos={scene, local_scene, scene_written, local_written?, args, beat}, head={var, slot}, var, slot, actor}`（rollback 147-172） | `Snapshot.vars` にスロット変数。ほかは読まない（slots 187-191） |
| 保存ファイルの形 | `[pasta_slots.1]` の直下に `v`・`var`・`slot`・`pos`・`head`・`actor`（rollback 176-201） | 直下にスロット変数を平らに置き、残りは `pasta_checkpoint`、日時は `pasta_saved_at`（slots 160-176） |

### 2.6 文法と生成

| 足す物 | 持ち主の設計 | 使う設計 | 合っているか |
| ------ | ------------ | -------- | ------------ |
| ト書きの対象と引数の変数（`cue_cmd_scope`・`cue_arg`） | cue（306-311） | hello（329-331） | 合う |
| `act:cue(名前, 対象, {引数})` の生成 | cue（335-344） | talk-flow（区切りの入口にしたい） | 口が無い（IF-03） |
| `init_scene` の 4 つ目にローカルの登録名 | cue（353-355） | talk-flow・rollback | 引数（`args`）が無い（IF-01） |
| 倉の項目の `args` | cue（363-366） | — | `append_set_scene_attrs` を talk-flow も触る（3 章） |
| 継続行の `act:line(true)` | talk-flow（691-692） | rollback（区切り） | 合う |
| 選択肢の条件 `choice_line … attrs?`・`act:choice(target, display, conds)` | talk-flow（684-688） | slots（141）・hello（298・370） | 合う |
| `！select（時間、飛び先）` の 2 つ目 | talk-flow（689-690） | hello（296） | 単位が割れた（IF-11） |
| 呼び出しの引数は式で、裸の名前は書けない | 今の文法（`grammar.pest:70-76`・`113-116`） | hello（294）・manual（63）が裸で書く | 割れた（IF-15） |

### 2.7 タグとイベント

| 物 | novel-areka-contract | 出す・受ける設計 | 合っているか |
| -- | -------------------- | ---------------- | ------------ |
| 舞台 | `\![play,stage,window／fullscreen／borderless]`、閉じるは引数なし（contract 239） | cue は `\![play,stage,開く／閉じる]`（cue 481） | 合わない（IF-17） |
| 並び | 番号の位置に名前。空いた番号は空の欄（contract 370） | cue は `spot` の並びの名前を順に（cue 455・491） | 空の欄が無い（IF-19） |
| 出し直しの頭 | `\![play,again]\__v[disable]\_q`（contract 285） | talk-flow は同じ（770）。rollback の 1 本目は `\![play,again]` だけ（570） | 字が無いので害は小さい（IF-08） |
| 着せ替え | 正典 `\![bind-noevent,分類,部品,1/0]`（contract 233） | rollback の例は `\![bind,…]`（rollback 570・574）。今の出力は `bind-noevent`（`appearance.lua:517`） | 例の書き違い（IF-23） |
| 続きのイベント | 名前と ID を引数で受ける | talk-flow `OnPlayNext`（十進の ID） | 合う |
| 目印のイベント | 同上 | rollback `OnPlayRollback`。slots も同じ名前で `\![raise]` を出す | 道が 2 つ（IF-04） |
| `OnChoiceTimeout` | 正典のまま | talk-flow が `REG` に置く（613-621） | 合う |
| `OnMouseClick` | R3・R4・R5 | shell は `REG` を作らない（426）。talk-flow は `EVENT.no_entry` を直す（653） | 合う。部品の名前だけ違う（IF-20） |
| リソース `sakura.popupmenu.visible` | 0 と答える口が要る（contract 447） | どの設計も作らない | 持ち主が無い（IF-13） |
| ベースウェアの版 | NOTIFY `basewareversion`（R0＝名前・R1＝版。contract 275） | cue は `Sender` と `get_property("baseware.version")`（cue 515） | 取り方が違う（IF-19） |

### 2.8 設定と保存のキー

| キー | 持ち主 | 形 | 気になる所 |
| ---- | ------ | -- | ---------- |
| `pasta.toml` `[cue.alias]` | cue | `揺らす = { tag = ["play","shake"], remain = false }`（cue 520-525） | contract は配列だけの形で書き、意味を知る名前も入れる（contract 380）。IF-19 |
| `pasta.toml` `[play] checkpoint_limit` | rollback（722） | 数 | 読み込みと検査のファイルが、共有の一覧に無い（IF-22） |
| `save.pasta_play_auto`・`save.pasta_read` | talk-flow（745-753） | 数・表 | 予約の変数の表（`book/src/grammar/variables.md`）に足す設計が無い（IF-21） |
| `save.pasta_slots` | slots | 2.5 の表 | 形が割れた（IF-05） |
| スロット変数の頭「スロット」・`slot_` | 判定は rollback（`SLOT_VARS.is_slot`）。文書は slots と rollback で分担が未定（slots 388） | — | 持ち主を 1 行で決める |
| 組み込みの名前（`GLOBAL`） | talk-flow 6 つ・slots 8 つ | — | 数の書き方が絶対値（IF-21）。見本は別の名前を使う（IF-17） |

---

## 3. 2 本以上の設計が触る既存のファイル

| ファイル（関数） | 触る設計と中身 | ぶつかり方 |
| ---------------- | -------------- | ---------- |
| `pasta/act.lua` `init_scene`（221-233 行） | cue: 引数を 1 つ足し、`CUE.arrive` を呼ぶ。talk-flow: ここから `PLAY.arrive` を「柱の前」に。rollback: ここから `CHECKPOINT.arrive` を「柱の後」に、戻り値を yield | 同じ 1 か所に 3 つの呼び出し。順と形が合わない（IF-01） |
| `pasta/act.lua` `call`（725-756 行） | talk-flow: 頭で `PLAY.seal`。rollback: 関数と引数から登録名を引く口を cue に求める（742-744 行） | cue はここを触らない。求めだけが残る（IF-01） |
| `pasta/act.lua` `restore_scene`（800-808 行） | cue: 深さを数える | 単独。ただし、深さは talk-flow と rollback の判じの元になる |
| `pasta/act.lua` `line`・`talk`・`choice`・`choice_timeout` | talk-flow だけ | roadmap の決まり「`act.lua` は 1 ウェーブに 1 spec」の下で、cue の後 |
| `pasta/scene.lua` | shell: `SCENE.search_scoped` を足す。talk-flow: `SCENE.wrap` の最後を `act:build(true)`。rollback: メタテーブルに `__newindex`（cue に求める） | 別の関数。`__newindex` は、どの設計も作らない |
| `pasta/store.lua` `STORE.reset`（89-106 行） | cue: `cue_state`。talk-flow: `play`。rollback: `actor_attrs`・`checkpoint` | 行は別。`cue_state` と `actor_attrs` は同じ物を 2 つ持つ（IF-07） |
| `pasta/shiori/act.lua` `SHIORI_ACT_IMPL.build`（72-107 行） | talk-flow: `final`・枠・`finish`。cue: `escape_tag_arg` を別ファイルへ移す（別の関数） | rollback の早回しが、この関数の `_spoke` の印を動かす（rollback の検証の重大 2）。IF-02 |
| `pasta/shiori/sakura_builder.lua` `BUILDER.build` | cue: cue の枝・転換を閉じる 3 か所・並び・`config.cue_instant`・`\e` の前の `finish`。talk-flow: `config.head`・`config.tail` を頭と `\e` の前に | `\e` の前に 2 つ入る。順は「cue の `finish`（`\_w` が出うる）→ talk-flow の `tail`（`\x`）」でないと、待つ転換が `\x` の後になる（IF-23） |
| `pasta/shiori/event/init.lua` 読み込みの並び（85-92 行） | talk-flow・rollback が 1 行ずつ | 行を足すだけ |
| 同 `EVENT.no_entry`（222-226 行） | talk-flow が直す。shell は触らず、`SCENE.search_scoped` を使ってもらう前提 | 部品の名前が合わない（IF-20） |
| 同 `EVENT.drive`（236-282 行） | talk-flow: 4 か所。rollback: 続きの決まりの変更を talk-flow に求める（628） | talk-flow の設計に、その変更が無い（IF-04） |
| 同 `EVENT.fire` | cue: `BASEWARE.observe(req)` を 1 行 | cue の 10 章は「`event/init.lua` は触らない」とも書く（cue の検証の副次 1） |
| `shiori/event/choice_select.lua`（51-70 行） | shell: `ROUTE.to_scene` へ取り出す。talk-flow: 同じ物を `CHOICE_ROUTE.to_scene` として使う。先に入らなければ自分で取り出す | 名前だけ違う（IF-20） |
| `shiori/entry.lua` | slots: `require` を 1 行 | talk-flow の組み込みは `event/play.lua` に置く。置き場所が 2 つに分かれる（害は無い） |
| `pasta_dsl` `grammar.pest`・`parse_scene.rs` | cue: ト書きの変数。talk-flow: 選択肢の条件 | 別の規則 |
| `code_gen/scope_gen.rs` `append_set_scene_attrs` | cue: 項目に `args`。talk-flow: `＆再生モード` の不正な値を止める | 同じ関数。後に入る側が合わせる |
| `tests/fixtures/sample.generated.lua`・`sample.expected.lua` | cue: `init_scene` の引数。talk-flow: `act:line(true)` | どちらも全行に近い差分。順に作り直す |
| `pasta_check` `builtins.rs` | cue: `ACT_METHODS` 28 → 29。talk-flow: `GLOBAL_NAMES` 16 → 22。slots: 16 → 24 | 数が絶対値（IF-21） |
| `pasta_check` `validate_parity_test.rs` | cue・talk-flow（slots は頼るだけ） | 行を足すだけ |
| `tests/lua_specs/init.lua` | cue 3・shell 2・talk-flow 2・rollback 3・slots 1 | 行を足すだけ |
| `book/src/SUMMARY.md` | contract: 内部設計に 1 行。rollback: 内部設計に `checkpoint.md`。manual: 新しい部 13 章 | 内部設計の章の数が 2 回動く（IF-22） |
| `book/tools/verify-content.mjs`・`verify-scripts-test.mjs` | contract: 内部設計の数（10 → 11・8 → 9）。manual: N 系 | rollback の新しい頁も同じ 3 か所を動かすが、一覧に無い（IF-22） |
| `book/src/lua/shiori-events.md` | shell（`OnMouseClick` の節）・talk-flow（`OnPlayNext`・`OnChoiceTimeout`・「進める」・出し直し）・rollback（`OnPlayRollback`） | `OnMouseClick` の節を 2 本が書く。先の spec が節を作り、後が足す |
| `book/src/internals/talk-output.md` | cue・talk-flow | 別の節 |
| `book/src/reference/pasta-toml.md` | cue（`[cue.alias]`）・rollback（`[play]`） | 別の表 |
| `book/src/grammar/variables.md` | slots（`＄＊pasta_slots`）。スロット変数の節は rollback と分担 | 分担が未定 |
| `.claude/skills/*/references/` | 6 本が「作り直す」。manual は手書きの節を足す | 作り直しは道具がするので、順に走らせるだけ |

---

## 4. 問題点

### 重大

#### IF-01 「シーンに着いた時」の口が 3 通りで、着いた時の引数を渡す所が無い（cross-spec。持ち主は cue-command-runtime）

- **根拠**: 2.1 の表。cue は `init_scene` の中で柱を積み、その後に `on_arrive(fn(act, global_name, local_name))` を呼ぶ（cue 427）。talk-flow は、柱の前に `PLAY.arrive(act, here, mode)` を求める（talk-flow 826）。rollback は、柱の後に `CHECKPOINT.arrive(act, ref)` を求め、戻り値の文字列を呼んだ側が yield する（rollback 244・640）。シーンの識別の表も 3 つ在る。
- rollback の `ref.args`（着いた時の引数。rollback 161・716）は、どの設計も渡さない。今の生成は `local args = { ... }` の後に `act:init_scene(SCENE, 書いた名前, 書いた名前)` を呼び、引数を渡していない（`crates/pasta_lua/src/code_gen/scope_gen.rs:266`・`273-277`、`pasta/act.lua:221`）。cue が足すのは、ローカルの登録名だけである（cue 353-355）。
- rollback が求める `ARRIVAL.next_is_arrival(act)` と `STORE.scene_ids`（rollback 641-642）も、cue の設計に無い。
- **起きること**: 3 本をそのまま作ると、`init_scene` の 1 か所に、順の違う 3 つの呼び出しが要る。引数つきで着いたシーン（`＞消えて現れる（「さくら」、「窓際」）` を最後の行に書いた場合）は、ロードで頭から同じに走れない。
- **最小の直し**: 骨子 5 章のとおり、着いた時の判じと知らせは cue が 1 つだけ持つ。知らせの形を 1 つに決める: `fn(act, here)`、`here = {global, local_, gname, lname, args}`（登録名 2 つ・書いた名前 2 つ・引数）。知らせは 2 回（柱を積む前と後）にせず、**柱を積む前に 1 回**にする。talk-flow は、そこで走り方を決める（`＆再生モード` は、talk-flow が階層から自分で引く。cue の `define_pillar` は要らなくなる）。rollback は、同じ知らせで控えを取り直す（柱は出力で、控えの中身＝変数とアクター状態を変えないので、前でも後でも同じ控えになる）。引数は、`init_scene` の最後から 2 つ目に `args` を渡す（生成の 1 行の中で済む。行は増えない）。
- `next_is_arrival` は、cue の「深さ」の作りなら要らない（`CHECKPOINT.jump` を最後の行の呼び出しで呼べば、深さは 0 のままである）。`STORE.scene_ids` も、登録名を `init_scene` が受けるので要らない。

#### IF-02 1 本の区切りと早回しの作りが 3 通りで、どれも他の 2 本の前提を壊す（cross-spec。区切りは novel-talk-flow、早回しの述語は novel-checkpoint-rollback）

- **根拠**: 2.2・2.3 の表。talk-flow は、`PLAY.seal` が `act:build()` を呼び、枠（末尾・ID・待っている 1 本の控え・既読）を `build` の中で付ける（talk-flow 465-472・518-534・599-601）。rollback は、組み立てが済んだ文字列を `CHECKPOINT.seal(act, script, spoke)` で受け、早回しの間は nil を返す（rollback 247-253・315-319）。cue は、早回しの間のトークンを「組み立てずに」捨ててほしい（cue 597。覚えた値は組み立てで確定する。cue 498）。talk-flow の早回しの案は `act._play_target`（talk-flow 851）で、rollback の `replay`（rollback 216-223）と別物である。
- **起きること**（rollback の形で作った場合）: 早回しで捨てる 1 本も組み立てるので、(a) cue の覚えた値が早回しで進み、戻った後の同じ値のト書きが出ない。(b) `STORE.play.seq`・`STORE.play.wait`・`pasta_read`・`act._line` が、捨てる 1 本のぶん進む（既読が、読んでいない行に付く）。(c) `_spoke` の印と中断のシーンが動く（`pasta/shiori/act.lua:76-93`。rollback の検証の重大 2）。
- **最小の直し**: 口を 2 つに絞る。
  1. 区切る所は talk-flow の `PLAY.seal` の 1 か所だけにする。`PLAY.seal` は、組み立ての前に `CHECKPOINT.replaying()`（rollback の述語）を見て、真ならトークンと `act._cue_pending` を捨てて数だけ進める。組み立ても枠も動かない。
  2. 目当てに着いた時と、ふつうの 1 本の時に、rollback へ 1 回ずつ知らせる（`CHECKPOINT.seal(act)` を、文字列を受けない形にする）。目印は、talk-flow の枠の頭に rollback が返す文字列を入れる。
- `act._play_target` と、slots の `fast_forwarding()` は、`CHECKPOINT.replaying()` に寄せる（名前を 1 つ）。`＞演劇再開` も同じ述語を見る（rollback 620）。

#### IF-03 ト書きの頭で区切る役を、どの設計も持たない（cross-spec。持ち主は novel-talk-flow。cue-command-runtime は入口を渡す）

- **根拠**: talk-flow は「cue が `PLAY.seal` を呼ぶ」前提である（talk-flow 825・941）。cue の `CUE.push` は呼ばない（cue 400）。roadmap の順は cue が 4、talk-flow が 6 なので（`.kiro/steering/roadmap.md:313-324`）、cue が talk-flow の関数を呼ぶ形は、順とも合わない。talk-flow の検証の重大 2 が、同じ物を拾っている。
- あわせて、名前の違う口が 4 つ在る: 時間で次へ（`act._cue_xtimeout` と `PLAY.set_line_timeout`）・瞬間の命令（`config.cue_instant` と `instant = true`）・判定（`BASEWARE.kind()` と `STAGE.is_areka`）・柱のトークン（`CUE.apply_pillars` と `STAGE.pillar_tokens`）。
- **起きること**: 次の台本で、雷が「降ってきたね。」の 1 本に入る。骨子 2.2「ト書きは、直後のアクション行を修飾する」に反する。

```pasta
＊嵐の教室　＆再生モード：演劇
    さくら：降ってきたね。
    ！効果音（雷）
    うにゅう：うわっ。
```

- **最小の直し**: cue は `ACT_IMPL.cue = CUE.push` を置くだけにする（今の設計のまま）。talk-flow が、自分の持ち分の `act.lua` の中で、`cue` を「`PLAY.seal(self)` の後に `CUE.push`」に包む。新しい知らせの口は要らない。名前の 4 つは、先に入る cue の名前（`act._cue_xtimeout`・`def.kind`・`BASEWARE.kind()`・`CUE.apply_pillars`）に talk-flow が合わせる。瞬間の命令を抜く所は 1 つにする（cue の `config.cue_instant`）。

#### IF-04 ロードの道が 2 通りで、口の名前が 7 つ合わない（cross-spec。持ち主は novel-checkpoint-rollback）

- **根拠**: 2.5 の表。slots は、控えを倉に入れ直して `\![raise,OnPlayRollback,ID]` を出し、巻き戻しの受け口に任せる（slots 39・103・433）。rollback は、`スロット読込` が `return CHECKPOINT.jump(act, cp, {load=true})` を返す形である（rollback 532-535・659）。
- rollback の道は、設定の場面のコルーチンがそのまま物語のコルーチンになるので、続きの決まりの変更が要る（rollback 628。今は `shiori/event/init.lua:269-280` の `spoke` で決まる）。talk-flow の設計に、その変更は無い。talk-flow は、別の口 `PLAY.set_continuation` を用意している（talk-flow 849）。rollback は、それを使わない。
- slots の道は、受け口のコルーチンに `STORE.co_preempt` の印が付くので（rollback 455）、続きの決まりを変えずに済む。代わりに、往復が 1 回増え、`\![raise]` が届かないと物語へ戻れない（slots の検証の議題 2）。
- **最小の直し**: 道を 1 つにする。骨子 2.5「ロードは、バックログの巻き戻しと、同じ仕組み」を、入口も同じ（受け口 1 つ）と読めば、slots の道になる。その場合、rollback は `adopt(cp) -> id` を 1 つ足し、`EVENT.drive` の変更・`PLAY.set_continuation` のロードの使い道・`finish_short` を `スロット読込` から呼ぶ約束は、要らなくなる。名前は rollback の物に寄せる: `SLOT_VARS.snapshot`／`replace`、`CHECKPOINT.clear`、`CHECKPOINT.replaying`。`validate` と `EVENT` は rollback が足す。どちらの道にするかは、開発者の決定が要る（6 章）。
- どちらの道でも決める事: ロードの 2 本目の目印の ID（slots の道では `reuse_id` になり、`adopt` の ID が付く。rollback の道では新しい ID）。contract の案 `\![play,unmark]`（contract 369）を 1 本目に入れる所（rollback の `replay.load`）は、slots の道では受け口が「ロードか」を知らないので、`adopt` が「ロードである」印を残す必要がある。

### 大きい

#### IF-05 スロットの記録の形が合わない（cross-spec。保存ファイルの形は novel-save-slots、控えの形は novel-checkpoint-rollback）

- **根拠**: slots は、スロットの直下にスロット変数を平らに置く（slots 166-168）。`スロット参照（１、「スロット章」）` は `record[name]` を読む（slots 105）。rollback の控えは、スロット変数を `slot` の下に持ち、そのままスロットに書く例を載せる（rollback 176-180）。slots が読む欄の名前は `vars`、rollback の欄は `slot` である（slots 189、rollback 153）。
- rollback の `head.slot`（着いた時のスロット変数。rollback 151・189-191）が入ると、1 つのスロットにスロット変数が 3 つ入る（平らな物・`slot`・`head.slot`）。
- 再生位置のシーンは、rollback が登録名（`放課後_1`）、slots の例が書いた名前（`第一章本文`）である（rollback 183、slots 172）。登録名は読み込みの順で振られるので、台本を直すとずれる（rollback の検証の大きい 8）。
- **起きること**: rollback の形のまま書くと、見本の「つづきから」（`＠スロット参照（「オート」、「スロット章」）`。hello 366）が、いつも値なしになる。
- **最小の直し**: ファイルの形は slots が決める（骨子 5 章）。rollback は、控えの欄の名前を `vars` に合わせるか、slots が `cp.slot` を読む。`head` を持つかは、rollback の「差し戻しではないが議題」（rollback 745）なので、要件定義で先に決める。位置は、書いた名前（グローバル・ローカル）と行の番号で持ち、登録名はロードの時に引き直す。

#### IF-06 ID が 2 系列になり、戻る時の 1 本目の続きが受け口で捨てられる（cross-spec。続きの ID は novel-talk-flow、目印の ID は novel-checkpoint-rollback）

- **根拠**: 続きの ID は `STORE.play.seq` の十進（talk-flow 599）、目印の ID はセッションの頭つき（rollback 228）。contract の例は、同じ ID を両方に使う（contract 296）。slots の例は、ロードの ID を `8` と書く（slots 113-117）。
- rollback の 1 本目の末尾は `PLAY.next_tail()` である（rollback 347）。talk-flow に、この関数は無い。talk-flow の `REG.OnPlayNext` は、`STORE.play.wait` が在って ID が合う時だけ再開する（talk-flow 606-611）。1 本目は `build` を通らない文字列なので、`STORE.play.wait` が置かれない。
- **起きること**: 巻き戻しとロードで、1 本目（画面の状態）は出るが、続きの `OnPlayNext` が 204 になり、2 本目（選ばれた行）が出ない。
- **最小の直し**: 続きの ID は talk-flow が 1 つだけ作る（骨子 6 章の持ち主）。目印の ID を別に作るかは、rollback が決める（別でよい。ベースウェアは中身を見ない）。talk-flow は `PLAY.next_tail(act) -> string` を渡し、その中で、待ちの種類「続きだけ」を `STORE.play.wait` に置く。セッションの頭（読み直しの後に古い ID と重ねない）は、両方の ID に同じ決まりを当てる。

#### IF-07 アクター状態の置き場が 2 つ、「全部書く」関数が 3 つ在る（cross-spec。今の値は cue-command-runtime、控えは novel-checkpoint-rollback）

- **根拠**: 2.4 の表。cue は `STORE.cue_state` に、rollback は `STORE.actor_attrs` に、同じ 4 つの値を置く。rollback は「cue が `ACTOR_STATE.set` で書く」前提だが（rollback 634）、cue は roadmap で先に入る（4 と 8）。rollback が cue に求める 3 つの関数（`CUE.pillar_script`・`CUE.cast_script`・`CUE.actor_attr_script`。rollback 643-645）と、talk-flow が求める 2 つ（`ROLLBACK.state_tokens`・`STAGE.pillar_tokens`）は、どの設計にも無い。
- 並びも 2 通りである（`STORE.cue_state["play,cast@"]` と `STORE.actor_spots`）。
- cue の `full_state` は、SSP では空である（cue 503）。SSP のロードで、位置・透明度・大きさが画面に戻らない（cue の検証の重大 1）。フェーズ 1 の中身（骨子 2.9）が欠ける。
- **最小の直し**: 今の値は、先に入る cue の `STORE.cue_state` の 1 か所に置く（同じ値を打たない判定に、もともと要る）。rollback は、`CUE_STATE.snapshot_actor()`・`restore_actor()` を使い、`STORE.actor_attrs` と `ACTOR_STATE.set`／`get` を作らない。表情と着せ替えは、今の `STORE.appearance` のままである。「全部書く」は 1 つにする: cue がアクターの値のタグを返し（SSP でも、位置・透明度・大きさは書く）、rollback が表情と着せ替えを足す。talk-flow は、その文字列を受ける（トークンで受ける形はやめる）。並びは、`STORE.actor_spots` を元にして、cue が覚える値を作らない（同じ物を 2 か所に持たない）。

#### IF-08 出し直しで曲をどうするかが、設計ごとに違う（cross-spec。命令の意味は novel-areka-contract、打つ・打たないは cue-command-runtime）

- **根拠**:
  - contract: 曲と環境の音は、残る命令の表に入れない。出し直しの頭でも止まらない。同じ曲を打たないのは SHIORI の仕事（contract 261・430）。
  - cue: `full_state` は、areka では曲と環境音も全部書く。SSP では書かない（cue 502-503）。
  - rollback: 1 本目の前に、アクターに付かない残る命令の記憶を「無い」に戻し、階層の柱を全部書く（rollback 643-644）。
  - talk-flow: 出し直しの 1 本に、いつも柱を打つ。同じ値は打たれないので害が無い（talk-flow 954）。
- **起きること**: rollback の形だと、巻き戻しのたびに、鳴っている曲と同じ曲の `\![sound,loop,…]` が出る。SSP で頭から鳴り直すかは、確かめていない（cue 764）。
- 出し直しの 1 本を組む所も 2 つ在る（talk-flow の `PLAY_EVENT.again_script` と、rollback の `land`。talk-flow 850、rollback 338-348）。
- **最小の直し**: contract の読みに揃える。出し直しは、アクターの値だけを既定へ戻す。曲は、「階層の柱を、ふつうに打つ」だけにする（骨子 2.5「シーンに着いた時と、することは同じ」）。IF-02 の直しで、早回しの間に cue の記憶が進まなければ、記憶は「今、鳴っている物」のままなので、同じ曲は打たれず、違う曲だけが替わる。`full=true` も、記憶を「無い」に戻す手順も、SSP と areka の場合分けも要らない。出し直しの 1 本を組むのは talk-flow の 1 か所にし、rollback は中身（画面の状態の文字列）を渡す。

#### IF-09 再生位置と今のシーンを、2 本が別々に記録する（cross-spec。持ち主は novel-talk-flow）

- **根拠**: talk-flow は `STORE.play.here` と `act._line` を持ち、`PLAY.position(act)` を渡す（talk-flow 433-436・481）。rollback は、コルーチンごとの足取り `tracks[co].pos`（`beat`）を自分で数え、`CHECKPOINT.position()` を talk-flow の既読に渡す（rollback 116・287-289・617）。行の番号を数える所が 2 つ在る。
- 設定の間、talk-flow は物語のコルーチンを `STORE.play.story.co` に移す（talk-flow 338・490）。rollback の `story_actor()`・`current_id()`・`position()` は「`STORE.co_scene` の足取り」を読む（rollback 283・287・291）。設定の間、そこに在るのは設定の場面のコルーチンである。
- **起きること**: 設定の画面から `＞演劇再開` で戻る時に、物語の行の画面を引けない。骨子 3 章が取り下げた「状態は、呼んだ流れに付く」に戻っている（rollback の検証の重大 1）。
- **最小の直し**: 今のシーンと行の番号は、talk-flow が 1 か所で持つ（骨子 5 章）。rollback は `PLAY.position(act)` を読み、自分では数えない。コルーチンごとの足取りに残すのは、「まだ続きになっていないコルーチンの仮の控え」だけにする。既読の鍵も、`PLAY.position` の 1 つにする。

#### IF-10 控えへ戻った時の再生モードが、階層から決まらない場合が在る（model）

- **根拠**: 骨子 2.1 は「書いていなければ、前のモードが続く」、2.5 は「戻った時に、その行のシーンの階層の柱を打って決める」。物語のシーンの階層に `＆再生モード` が無いと、設定の画面や `＊OnBoot`（トーク）からロードした時に、値が出ない。
- 設計は 3 通りに読んだ。slots と rollback は「柱で演劇になる」と仮定する（slots 263、rollback 730）。talk-flow の `PLAY.arrive` は、値が無ければ `STORE.play.mode` のままにする（talk-flow 477）。設定からロードすると、物語が設定のモードで走り、記録されない。talk-flow の `PLAY.set_continuation` は走り方を運ぶ（talk-flow 849）が、これは走り方を控えに入れることになる。
- **最小の直し**: 骨子に 1 文足す。「控えへ戻った時の再生モードは、演劇である」。控えは演劇の 1 本にしか無いので、保存せずに決まる。階層に `＆再生モード` が在れば、同じ値なので食い違わない。talk-flow の検証の主要 8 の案 A と同じである。

#### IF-11 `！select` の時間の単位が、設計の間で割れた（model）

- **根拠**: 骨子 2.4「時間は、どれもミリ秒で書く」。今の `！select` は秒である（`scope_gen.rs:395-411`、`pasta/shiori/sakura_builder.lua:456-458`）。cue と talk-flow は秒のまま（cue 708、talk-flow 100）。見本はミリ秒で書いた（`！select（１００００、迷う）`。hello 296・550）。manual は、どちらとも書いていない。
- **起きること**: 2 本の設計のまま作ると、見本の時間切れは 10000 秒になる。
- **最小の直し**: 開発者の決定が要る。秒のままなら、骨子 2.4 に `！select` の例外を 1 行足し、骨子と手引きの作例を `！select（２０、時間切れ）` に直す。ミリ秒に替えるなら、今のゴーストの `!select(10)` が 10 ミリ秒になる（破壊的）。

#### IF-12 設定の間の「今のシーン」と `＄演劇` の値が、骨子に書いていない（model）

- **根拠**: 骨子 1 章の「再生位置＝今のシーンと行の番号」と、2.1 の「設定は、物語の再生位置を動かさない」。設定の場面にも着くので、2 つを満たすには、設定の間は「ローカルを探す範囲」と「物語の再生位置」が別になる。骨子 2.3 は「演劇の間」としか書かない。
- 設計は、同じ向きに仮定している: 設定の間もローカルを先に探し、範囲は設定の場面（talk-flow 945、shell 423）。`＄演劇` は、設定の間も空でない（talk-flow 946）。見本は「物語が待っていれば空でない」と読んだ（hello 539）。タイトルの画面（物語が待っていない設定）では、2 つの読みが分かれる。
- **起きること**: 見本の `30-desktop.pasta`（`＆＄演劇＝「」`）の `＊OnMouseClick` が、タイトルの画面で動くかどうかが、読みで変わる。
- **最小の直し**: 骨子に 2 文足す。「設定の間、今のシーンは設定の場面である。物語の再生位置は、別に止めて持つ」「`＄演劇` は、再生モードがトークでない間、今のシーンのグローバルシーンの名前である」。変数の名前（`＄演劇` か `＄再生モード`）も、ここで 1 つに決める（骨子 6 章の候補のまま）。

#### IF-13 右クリックのメニューを止める口の持ち主が無い（model）

- **根拠**: 骨子 2.6 は、設定の画面の入口に右クリックを挙げる。contract は「pasta がリソース `sakura.popupmenu.visible` に 0 と答える」を取り決めに書く（contract 63）。どの設計も、答える口を作らない（contract 447、hello 531、manual 310）。シーンの出力は `\p[N]` で始まり `\e` で終わる（`pasta/shiori/sakura_builder.lua:45`・`686`）ので、DSL のシーンでは 0 を返せない。
- **起きること**: 見本とマニュアルの入口が、2 つ（ボタンとキー）になる。骨子 2.6 の 3 つに届かない。
- **最小の直し**: 骨子 5 章の表に持ち主を足す（talk-flow が候補）。書き方は、今のシーンの属性から導く形が、原則に合う（例: 設定のモードへ入るシーンの宣言に書いた属性を、pasta が照会の答えに使う）。別の柱を足すか、再生モードから導くかは、開発者の決定が要る。

#### IF-14 見本の辞書の割り方が、マニュアルの照合と合わない（cross-spec。持ち主は hello-novel-sample。決めるのは 2 本の要件定義の前）

- **根拠**: 見本は 4 枚（`01-actors`・`10-story`・`20-system`・`30-desktop`）で、段階に分けない（hello 184-189・536）。テストは `novel_sample_test.rs` の 1 本（hello 592）。manual は、1 段 1 ファイルの `NN-name.pasta`（`01-play`〜`09-stage`）と、段の表と、段ごとのテストを求める（manual 57-73・212-219）。照合は、章と辞書を同じ名前で突き合わせる（manual 55）。
- **起きること**: どちらかを作り直す。見本を先に作ると、マニュアルの照合（`tutorial-check.mjs` の 2 つ目の組）が全部落ちる。
- **最小の直し**: 見本の要件定義で、辞書を「段のファイル」に切るかを先に決める。切らないなら、manual は照合の組を足さず、章の作例を見本からの抜き出しとして人が確かめる（道具の変更 4 つが要らなくなる）。切るなら、見本の 4 枚の中身を段へ割り直す（`20-system` は 1 段にまとまるので、写して使う 1 枚は残る）。

#### IF-15 作例の呼び出しの引数に、今の文法で書けない形が在る（cross-spec。持ち主は cue-command-runtime の決まり。直すのは hello・manual と、骨子の作例）

- **根拠**: 呼び出しの引数は式で、裸の名前は項に無い（`crates/pasta_dsl/src/parser/grammar.pest:70-76`・`113-116`）。cue は「作例を「」で囲む」と決めた（cue 712）。見本は `＞消えて現れる（うにゅう、「５０」）`（hello 294）、manual は `＞消えて現れる（さくら、窓際）`（manual 63）と書く。骨子 2.4 と手引きの作例も裸である。
- manual は、数の引数を `！待つ（「３００」）`・`！時間で次へ（「１０００」）` と囲む（manual 63・69）。骨子 2.4 の「数字で始まる引数は「」で囲む」は、数字で始まる**文字**の話で、数は囲まない（cue 148 の事実: 数の引数は数に直る）。
- **起きること**: 見本の検査（誤り 0。hello 601）が、構文エラーで落ちる。
- **最小の直し**: 文法は足さない。作例を `＞消えて現れる（「うにゅう」、５０）` の形に直す（骨子・手引き・見本・マニュアル）。裸の名前を書けるようにするかは、別の議題である（決めていない）。

#### IF-16 見本の見出しの変数が、頭の自動セーブに入らない（cross-spec。根は novel-save-slots の差し戻し 1）

- **根拠**: 見本は、シーンの頭に `＞スロット保存（「オート」）`、その次の行に `＄＊スロット章＝「はじまり」` を書く（hello 263-264・274-275・292-293）。控えは着いた時の状態なので、代入は入らない（slots 381、rollback 548）。slots の型は「見出しの変数は、飛ぶ前のシーンで書く」である（slots 152）。
- **起きること**: `＊はじまり` の間のセーブに `スロット章` が無く、タイトルの「つづきから」が出ない。`＊教室` のセーブは、章の名前が 1 つ前（「はじまり」）になる。見本のテスト T10 が落ちる。
- **最小の直し**: slots の差し戻し 1 の (c) を採れば、見出しは「保存した再生位置のシーンの書いた名前」から引ける（保存せず、位置から導く）。その場合、見本は見出しの変数を書かなくてよく、`スロット参照` に位置の名前を読む口が 1 つ要る。採らないなら、見本を slots の型に合わせて書き直す。決めるのは開発者である。

#### IF-17 設定を切り替える口が、設計の間で合わない。窓の形の持ち主が無い（cross-spec。持ち主は novel-talk-flow と cue-command-runtime）

- **根拠**: talk-flow は `＞オート（ミリ秒）`・`＞高速再生（１／０）`（値を置く。talk-flow 629-633）。見本は `＞オート切替`・`＞高速再生切替`（入切を返す。hello 406・410）。値を置く形で入切を書くには、今の値を読む口が要る。オートは `＄＊pasta_play_auto` で読めるが、高速再生は読めない（talk-flow 956）。
- 窓の形（骨子 6 章「設定の値を切り替える口」）は、contract が「pasta が保存して `stage` の引数で打つ」と書く（contract 255）。cue の舞台は `\![play,stage,開く／閉じる]` で、形を持たない（cue 481）。保存する所と、切り替える書き方は、どの設計にも無い。
- **最小の直し**: 口は talk-flow の「値を置く」形 1 つにし、見本は選択肢を 2 つ（入・切）並べて書く（今の値を読む口を足さない）。舞台のタグは contract の形（開く＝形の名前、閉じる＝引数なし）に cue が合わせる。窓の形は、フェーズ 3 の前半の物なので、持ち主（cue）だけを骨子 6 章に書き、作りは舞台の要件へ送る。

#### IF-18 roadmap の順に対して、後の spec が先の spec に関数を求めている（cross-spec）

- **根拠**: 順は contract 1・cue 4・shell 5・talk-flow 6・rollback 8・slots 9・hello 10・manual 11（`.kiro/steering/roadmap.md:313-324`）。

| 求める側（後） | 求められる側（先） | 求める物 |
| -------------- | ------------------ | -------- |
| talk-flow（6） | cue（4） | `PLAY.seal`・`PLAY.arrive`・`PLAY.set_line_timeout` を呼ぶこと。`instant = true`。`STAGE.is_areka`・`STAGE.pillar_tokens` |
| rollback（8） | cue（4） | `CHECKPOINT.arrive` を呼ぶこと。`ARRIVAL.next_is_arrival`・`CUE.pillar_script`・`CUE.cast_script`・`CUE.actor_attr_script`・`STORE.scene_ids`・`ACTOR_STATE.set` を呼ぶこと |
| rollback（8） | talk-flow（6） | `CHECKPOINT.seal` を呼ぶこと。`PLAY.mode(ref)`・`PLAY.next_tail()`・続きの決まりの変更 |
| slots（9） | rollback（8） | `adopt`・`discard_all`・`fast_forwarding`・`validate`・`EVENT`・どのモードでも受ける受け口 |
| talk-flow（6） | rollback（8。後） | `ROLLBACK.state_tokens`（出し直しの画面）。無い間、`＞演劇再開` は画面を戻せない |
| shell（5） | talk-flow（6。後） | `PLAY.scope()`・`EVENT.no_entry` の直し。shell の関数は、talk-flow が入るまで使われない |

- **起きること**: 先に入った spec のファイルを、後の spec が書き直す。rollback は「talk-flow のファイルに書き足す」と明記している（talk-flow 843）。
- **最小の直し**: 決まりを 1 つ置く。「先に入る spec は、後の spec の関数を呼ばない。先に入る spec が知らせの口を置き、後の spec が登録する」。IF-01・IF-02・IF-03・IF-07 の直しは、この決まりに沿っている。先の spec の要件定義に、後の spec が要る口（着いた時の知らせ・区切りの知らせ・早回しの述語を見る所）を、今のうちに入れる。

### 小さい

#### IF-19 別名表・並び・版の取り方の細かい食い違い（cross-spec。形は cue-command-runtime、名前は novel-areka-contract）

- 別名表の値の形: contract は配列（`揺らす = ["play","shake"]`）、cue は表（`{ tag = […], remain = … }`）（contract 380、cue 520-525）。形は cue が持つので、contract の章は cue の形で引く。
- 既定の表の中身: contract は `転換`・`透明度`・`大きさ`・`位置`・`舞台` も並べる。cue は、意味を知る 10 個を表に書くと読み込みのエラーにする（cue 530）。contract の表は「英字の命令と区分」だけを渡し、日本語の名前との対応は cue が持つ、と分ける。
- 並びの空の欄（`\![play,cast,教室,,さくら]`。contract 370）を、cue は出さない。アクター指定行の番号の明示（`grammar.pest:163-164`）を使う台本でだけ効く。cue の要件に 1 行足す。
- 重ね順のフェーズ: 手引きは 1、contract は 3 の前半で SSP は出さない（contract 242、cue 709）。骨子の「意味を知る 10 個」に無いので、contract の読みでよい。手引きの表を直す。
- 版: contract は NOTIFY `basewareversion`、cue は `get_property`。cue の `BASEWARE.observe(req)` が `basewareversion` も読めば、足りる。

#### IF-20 イベントのシーンを探す部品の名前が合わない（cross-spec。持ち主は shell-element-click）

- shell: `SCENE.search_scoped(name, scope, act)`・`ROUTE.to_scene(act, id, scope, subject)`・求める `PLAY.scope()`（shell 224-234・260-271・341）。talk-flow: `SCENE.search` を直に 2 回・求める `CHOICE_ROUTE.to_scene`・範囲は `STORE.play.here.global`（talk-flow 303・624・874）。中身は同じである。
- 直し: 先に入る shell の名前（`SCENE.search_scoped`・`ROUTE.to_scene`）に talk-flow が合わせ、`PLAY.scope()` を 1 行で渡す。shell を talk-flow に畳むか（shell 439）は、棚卸の判断である。

#### IF-21 組み込みの名前の数と、予約の保存キーの文書（cross-spec）

- `GLOBAL_NAMES` は今 16（`crates/pasta_check/src/validate/builtins.rs:8`）。talk-flow は 16 → 22、slots は 16 → 24 と書く。両方入ると 30 である。後に入る側が数を直す。決め打ちの並びのテスト（`validate_scenes_test.rs`）を、talk-flow の一覧は挙げていない。
- 見本の「シーンの名前の決まり」（hello 228）が挙げる組み込みの名前は、talk-flow の `オート`・`高速再生` を含まない。`・オート` というローカルシーンは、`＞オート` より先に当たる（`pasta/act.lua:410-417` の L2 が前方一致で、L4 の `GLOBAL` より先）。見本の `・オート`（hello 405）が、ちょうどこれに当たる。
- `save.pasta_play_auto`・`save.pasta_read` を、予約の変数の表に足す設計が無い。talk-flow の一覧に `book/src/grammar/variables.md` を足す。

#### IF-22 マニュアルの検査の数と、設定のキーの読み込み（cross-spec）

- contract は、内部設計の章の数を 10 → 11、題材章を 8 → 9 に直す（`book/tools/verify-scripts-test.mjs:64-67`、`book/tools/verify-content.mjs:507-510`）。rollback の新しい頁 `internals/checkpoint.md` も、同じ 3 か所を動かすが、共有の一覧に無い（rollback 702）。題材章にするなら、H2 の 7 つの形も要る。
- rollback の `[play] checkpoint_limit` は、マニュアルだけが一覧に在る。読み込み（`crates/pasta_lua/src/loader/config/`）と検査（`crates/pasta_check/src/validate/config.rs`）は、cue が `[cue.alias]` で触る所と同じである。一覧に足す。

#### IF-23 組み立ての頭と末尾に入れる順・例のタグの書き違い（cross-spec）

- `BUILDER.build` の `\e` の前に、cue の `finish`（待つ転換の `\_w` が出うる）と talk-flow の `tail`（`\x`）が入る。順は cue → talk-flow でないと、待ちが `\x` の後になる。talk-flow の要件に 1 行足す。
- rollback の 1 本目の例は `\![bind,…]` だが、今の出力と contract は `\![bind-noevent,…]` である（`pasta/shiori/appearance.lua:517`、contract 233）。`APPEARANCE.restore` を使い回す（rollback 412）なら、自然に合う。例だけ直す。

#### IF-24 演劇の間、字の無い 1 本で始まるイベントのシーンは、続きにならない（model）

- **根拠**: 骨子 2.2「字の無いアクション行も 1 本になる」と、2.3「台詞を出すシーンが選ばれたら置き換わる。台詞を出さないシーンは、続きに触れない」。今の判定は、最初に返した 1 本に字が在るかである（`pasta/shiori/act.lua:76-78`、`shiori/event/init.lua:269-280`）。
- **起きること**: 演劇の間、クリックの反応を表情だけの行で始めると、その 1 本は出るが、続きは元の物語のままになる。反応の残りの行は捨てられる。戻る時の 1 本目（字が無い）にも、同じ事が起きる（rollback 628）。

```pasta
    ・OnMouseClick　＆＄ｒ４＝「窓」
        うにゅう：＠驚き
        さくら：どうしたの？
```

- **最小の直し**: 骨子 2.3 に 1 文足す。「演劇と設定の間は、アクション行の 1 本を返したシーンが、選ばれたシーンである（字の有無に依らない）」。アクション行が在るかは台本から決まるので、新しい状態は要らない。talk-flow の続きの決まりは、この読みで書く。

---

## 5. 設計どうしで合っていた物（直さなくてよい）

- 続きと目印のイベント名は、タグの引数で渡す。取り決めは名前を固定しない（contract 25、talk-flow 886、rollback 461）。
- 柱は階層を重ねて打つ。ローカルの倉は親を受け継がないが、打つ時に重ねる（cue 403-423、rollback 734、hello 543、manual 301）。manual の brief の古い 1 行だけが違う。
- 途中の `＞` の先は、着いたと数えない（cue 278、talk-flow 481、rollback 17）。
- 設定の 1 本は `\__v[disable]` を付けて貯めず、控えも目印も作らない（talk-flow 37、rollback 15、contract 40）。
- 当たり判定のクリックは、今の条件の属性で書ける。新しいイベント名も `REG` の既定も作らない（shell 426、talk-flow 871、hello 676）。`＆＄ｒ３＝１` は、Reference の文字列 `"1"` と数として比べられる（`crates/pasta_lua/src/search/scene_cond.rs:50`・`111-120`）。shell の「確かめていないこと」の 1 つ目（shell 466）は、読む限り通る。
- セーブデータは pasta だけが持つ。取り決めに、セーブとロードの命令は無い（contract 383、slots 338）。
- 組み込みの呼び出しは、`＞自動トーク停止` と同じ作り（`pasta/shiori/entry.lua:31-35`）。talk-flow・slots・hello が同じ前提である。

---

## 6. 決まっていない事（この点検で決めなかった物）

1. ロードの入口を、巻き戻しの受け口 1 つにするか（slots の形）、`スロット読込` の中で飛ぶか（rollback の形）（IF-04）。
2. 着いた時の知らせを、柱を積む前の 1 回にしてよいか。`＆再生モード` を、cue の柱の打ち手でなく、talk-flow が階層から引く形にしてよいか（IF-01）。
3. 続きの ID と目印の ID を、同じ物にするか（IF-06）。
4. 早回しの種（`head`）と、着いた時の引数（`args`）を、控えとスロットに入れるか（rollback 745-746、IF-05）。
5. 出し直しで、曲を打ち直さないと読んでよいか（contract の読み。IF-08）。SSP が、同じ名前の `\![sound,loop,…]` で鳴り直すかは、実機で確かめていない。
6. 戻った時の再生モードを「演劇」と骨子に書くか（IF-10）。
7. `！select` の単位（IF-11）。
8. 設定の間の今のシーンと、`＄演劇` の値と名前（IF-12）。
9. 右クリックの口の持ち主と書き方（IF-13）。取り決めの章の置き場所（contract の E2）も、開発者の決定待ちである。
10. 見本の辞書を、段のファイルに切るか（IF-14）。
11. 呼び出しの引数に、裸の名前を書けるようにするか（IF-15）。この点検は、作例を直す側で書いた。
12. セーブ画面の見出しを、再生位置のシーンの名前から導くか（slots の差し戻し 1。IF-16）。
13. オートと高速再生の口を「値を置く」形にするか、入切を返す形にするか（IF-17）。
14. 演劇の間の「選ばれたシーン」を、アクション行の 1 本で決めるか（IF-24）。
15. `shell-element-click` を小さい spec として残すか、`novel-talk-flow` に畳むか（shell 439）。

---

## 7. 確かめたコード

| 事実 | 場所 |
| ---- | ---- |
| `init_scene` は、書いた名前 2 つと行の対応だけを受ける。登録名も引数も受けない | `crates/pasta_lua/pasta_scripts/pasta/act.lua:221-233` |
| 生成は `local args = { ... }` の次に `act:init_scene(SCENE, …, …)` を出す | `crates/pasta_lua/src/code_gen/scope_gen.rs:266`・`273-277` |
| `！` 行は `select` だけを生成する。`select` は 1 つ目の引数だけを読む | `scope_gen.rs:355-360`・`395-411` |
| `act:line()` は引数を取らない | `act.lua:248-254` |
| `act:call` は、関数を引いたら `handler(self, ...)` を呼ぶ。登録名は残らない | `act.lua:739-744` |
| `restore_scene` は、エラーで抜けた時は戻さない | `act.lua:800-808` |
| 名前の解決は、ローカルの前方一致（L2）が `GLOBAL` の完全一致（L4）より先 | `act.lua:410-417`・`427-430` |
| 選択肢の範囲は、実行中のグローバルシーン。時間切れは秒 | `act.lua:339-344`・`350-353` |
| `SCENE.wrap` は、最後に `act:build()` を返す | `crates/pasta_lua/pasta_scripts/pasta/scene.lua:221-226` |
| 続きの決まり。`spoke` は台詞の印か、出力が空でないかで決まる | `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua:236-282`（`269-272`・`274-280`） |
| `EVENT.no_entry` は `SCENE.co_exec(act, act.req.id, nil, nil)` | 同 `222-226` |
| 既定のハンドラの読み込みの並び | 同 `85-92` |
| 組み立てのたびに `_spoke` を立て、割り込みなら中断のシーンを掛ける | `crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua:72-107` |
| 時間切れは `seconds * 1000`。出力の最後は `\e` | `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua:456-458`・`686` |
| 着せ替えのタグは `\![bind-noevent,…]` | `crates/pasta_lua/pasta_scripts/pasta/shiori/appearance.lua:517` |
| `STORE.reset` の中身 | `crates/pasta_lua/pasta_scripts/pasta/store.lua:89-106` |
| 選択肢の振り分け（範囲のローカル → グローバル・条件漏れは止まるエラー） | `crates/pasta_lua/pasta_scripts/pasta/shiori/event/choice_select.lua:51-70` |
| 呼び出しの引数は式。項は、括弧・関数・変数・数・文字列だけ | `crates/pasta_dsl/src/parser/grammar.pest:60`・`70-76`・`113-116` |
| ト書きと選択肢の行の文法 | `grammar.pest:222-231`・`236` |
| `GLOBAL_NAMES` は 16、`ACT_METHODS` は 28 | `crates/pasta_check/src/validate/builtins.rs:8`・`28` |
| 条件は、文字列を数として読めれば数で比べる | `crates/pasta_lua/src/search/scene_cond.rs:50`・`111-120` |
| 内部設計の章は 10、題材章は 8 と決め打ち | `book/tools/verify-scripts-test.mjs:64-67`、`book/tools/verify-content.mjs:507-510` |
| Phase 13 の順 | `.kiro/steering/roadmap.md:313-324` |

## 8. 確かめていないこと

- `hello-novel-sample` と `manual-novel-guide` の設計の、調査の記録の章と、冒頭の要件の表（上に挙げた章の外）。
- 検証の 8 本のうち、`novel-areka-contract`・`shell-element-click`・`hello-novel-sample`・`manual-novel-guide` の検証は読んでいない。この文書の指摘と重なる物が在るかもしれない。
- 設計が引く行番号のうち、7 章に無い物（`mouse.lua`・`virtual_dispatcher.lua`・`persistence.rs` など）は、自分では開いていない。
- どの直し方も、机の上の案である。動かしていない。
