# 暫定設計の突き合わせ: 骨子の決まりごとの対応（2026-11-19）

Phase 13 の 8 本（`novel-areka-contract`・`cue-command-runtime`・`shell-element-click`・`novel-talk-flow`・`novel-checkpoint-rollback`・`novel-save-slots`・`hello-novel-sample`・`manual-novel-guide`）の暫定設計（`provisional-design.md`）と、その検証（`provisional-design-validation.md`）を、骨子（`final-model-2026-11-19.md`）の決まり 1 つずつに当てて読んだ記録である。

- 正は骨子である。brief と骨子が食い違う所は、骨子を正として指摘に挙げた。
- 観点は「骨子の決まりを、どの設計が、どう作るか」。作る設計が無い決まり・2 つの設計が別の形で作る決まり・決まりに反する作り・作ろうとして曖昧さや矛盾が出た決まりを拾った。
- 指摘の区分: **spec 間**＝2 つ以上の設計が食い違うか、すき間が在る。**骨子**＝骨子そのものが矛盾する・足りない・書いたとおりに作れない。**spec 内**＝設計も検証も見落とした物だけ。
- 重さ: **重大**＝このままでは骨子の基本の例が動かない。**大**＝要件定義の前に決めないと、後で設計を作り直す。**小**＝言葉や名前をそろえれば済む。
- コードは読んだだけで、ビルドもテストもしていない。読んだ行は 5 章に並べた。

## 1. 結論

1. **骨子の芯は、どれも作れる見込みである。** 走り方を柱で決める・1 行ずつ進む・分岐は 1 つの形・控えは 1 本ごと・状況は柱から決まる、のどれにも、作る設計が在る。骨子の層まで差し戻して形を変える必要が在る決まりは、見つからなかった。
2. **ただし、設計どうしのつなぎ目は、ほぼ全部が合っていない。** 8 本は互いを見ずに書かれたので、同じ物を 2〜3 本が別の形で持っている。重い物は 7 つ（指摘 1〜7）で、どれも「持ち主を 1 本に決めて、口を 1 つにする」で直る。新しい仕組みは要らない。
3. **骨子に足す・直す所が 15 在る**（4 章の表で「要る」とした行）。決まりの中身を選び直す物は、`！select` の単位（指摘 8）・台詞を出さないシーンの柱（指摘 10）・近道と中断の止め方（指摘 13）・セーブの見出し（指摘 24）・右クリックの口（指摘 26）の 5 つで、ほかは 1〜2 文の補いである。
4. **どの設計も作らない決まりが 5 つ在る**（2 章の表の「無い」）。右クリックのメニューを止める口・窓の形の設定・並びの下でスコープの番号からアクターを引く口・話者の名前を書く口・呼び出しの引数に裸の名前を書く文法、である。

## 2. 骨子の決まりごとの対応

「判定」の印: ○＝作る設計が 1 つで、骨子と合う。△＝作るが、条件つき（指摘を見よ）。二重＝2 つ以上の設計が別の形で作る。反＝骨子と違う作り。曖昧＝作ろうとして、骨子の文が 2 とおりに読めた。無い＝作る設計が無い。

### 2.1 再生モード

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| 走り方は柱 `＆再生モード`。値は 3 つ | `novel-talk-flow`: `PLAY.arrive(act, here, mode)` で受け、`PLAY_MODE_KEYS` を「柱として流さない名前」にする。不正な値は読み込みのエラー。`cue-command-runtime`: `CUE.define_pillar("再生モード", fn)` で、打ち手として登録させる | 二重 | 1 |
| 柱なので上書き。書いていなければ前のモードが続く。その行のモードは階層を引いて決まる | `novel-talk-flow`: `act._mode = mode or act._mode or STORE.play.mode`。確定は `PLAY.commit` | 曖昧 | 11 |
| 演劇のモードのシーンへ飛べば、着いた時に演劇になる | `novel-talk-flow`（着いた時）。前の字は最初の演劇の 1 本に入る | ○ | — |
| シーンの途中では切り替えられない | `cue-command-runtime` の深さの数え方（途中の `＞` では着いたと数えない） | ○ | 20 |
| `＞` が無くてもモードは変わらない（4 章 A） | `novel-talk-flow`: `STORE.play.mode`・`STORE.play.here` はゴーストに 1 つで残る。`novel-checkpoint-rollback`: 足取りはコルーチンに付くので、コルーチンが終わると消える | 二重・反 | 3 |
| 演劇と設定の間は、時報とランダムトークを返さない | `novel-talk-flow`: `virtual_dispatcher.dispatch` の門に 1 条件 | ○ | — |
| 設定から単に戻るのは `＞演劇再開`（何も戻さない） | `novel-talk-flow`: `GLOBAL.play_resume` と `STORE.play.story`。画面の状態は `novel-checkpoint-rollback` の `reapply_story_actor`・`state_script` か、`cue-command-runtime` の `CUE_BUILD.full_state` | 二重・曖昧 | 7・18 |
| 設定は、記録せず、物語の再生位置を動かさない | `novel-talk-flow`: 頭に `\__v[disable]`、物語を `STORE.play.story` に控える。`novel-checkpoint-rollback`: 設定では `seal` を呼ばれない | △ | 9 |

### 2.2 区切り

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| 1 本は、次のアクション行（と継続行）の末尾まで。字の無い行も 1 本 | `novel-talk-flow`: `PLAY.line`・`PLAY.seal`。継続行の生成を `act:line(true)` に替える | ○ | — |
| ト書き・柱・アクター指定行は、直後のアクション行の 1 本に入る | `novel-talk-flow` は「ト書きの入口から `PLAY.seal` を呼んでもらう」。`cue-command-runtime` の `CUE.push` は呼ばない | 無い | 2 |
| 末尾に `\x\![raise,OnPlayNext]` | `novel-talk-flow`: `PLAY_EVENT.frame`（ID 付き）。取り決め §6.4 も ID 付き | △ | 3・4 |
| 選択肢と `！select` は、直前のアクション行の 1 本に入る | `novel-talk-flow`: `PLAY.choice`。`！select` は区切らない | ○ | — |
| 待たない印 `\![parallel]` | `novel-talk-flow`: `PLAY.parallel_text` と `PLAY_EVENT.finish`。取り決めは `\![parallel,1]`・`\![parallel,0]` | ○ | — |
| 後ろにアクション行が無い残りは、待たない 1 本。目印は付かない | `novel-talk-flow`: `SCENE.wrap` の最後の `act:build(true)` | △ | 20 |
| 行の番号は、区切りの通し番号。途中の `＞` の先も同じ番号 | `novel-talk-flow`: `act._line`。`novel-checkpoint-rollback`: `Track.pos.beat` | 二重 | 3 |

### 2.3 分岐

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| 演劇の再生位置は、いつも 1 つ | `novel-talk-flow`: `STORE.play` に 1 つ。`novel-checkpoint-rollback`: コルーチンごとの足取り | 二重・反 | 3 |
| イベントでシーンが選ばれたら移る。台詞を出さないシーンは、続きに触れない | `novel-talk-flow`: `PLAY.commit(scene_act, replace)`（続きと同じ決まり）。`cue-command-runtime`: 台詞の有無によらず柱を積み、知らせる | 曖昧 | 10 |
| 演劇の間、ローカルシーンを先に探す | `novel-talk-flow`: `PLAY_EVENT.event_scene`（`EVENT.no_entry` と `mouse.lua` の 9 段目）。`shell-element-click`: `SCENE.search_scoped` と `PLAY.scope()` を前提 | 二重・△ | 9・14 |
| 元の場面を読ませ直すのは、最後の行の `＞` | 今の作り（`return act:call`） | ○ | — |
| どのシーンも拾わなかった左クリックは「進める」。選択肢の 1 本では進めない | `novel-talk-flow`: `PLAY_EVENT.advance` | ○ | — |
| いつもの反応は `＆＄演劇＝「」` で止める。`＄演劇` はイベントのたびに写す | `novel-talk-flow`: `PLAY_EVENT.on_request`。近道と中断は、走り方で止める | 反・曖昧 | 9・13 |
| 選択肢の条件・`！select` の 2 つ目の引数 | `novel-talk-flow`: `choice_line` に `attrs?`、`act:choice(target, display, conds)`、`SEARCH:cond_holds`、`act:choice_timeout(seconds, target)` | △ | 8 |

### 2.4 状況（柱）とト書き

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| 柱は、着いた時に自動で打たれるト書き。上書き・同じ値は何もしない・「なし」 | `cue-command-runtime`: `CUE.arrive`・`CUE.pillars`・`STORE.cue_state` | △ | 12・21 |
| 打たれるのは、着いた時だけ（4 章 F） | `cue-command-runtime`: `restore_scene` の `_call_depth` | ○ | 1・20 |
| 階層（ファイル属性 → グローバル → ローカル）を引く | `cue-command-runtime`: 実行時に `STORE.scene_attrs` の `__start__` とローカルを重ねる | ○ | — |
| ト書きは `\![名前,対象,引数…]`。別名表は `pasta.toml` | `cue-command-runtime`: `act:cue`・`[cue.alias]`・`pasta_core::CueTable` | △ | 16 |
| 柱に値を並べて書ける | `cue-command-runtime`: 倉の項目に `args` を足す | ○ | — |
| 対象と引数に変数。束は引数つきの呼び出しで呼ぶ | `cue-command-runtime`: `cue_cmd_scope`・`cue_arg` に `dyn_name`。呼ぶ側の文法は、どの設計も触らない | 無い（作例） | 19 |
| 時間は、どれもミリ秒 | `cue-command-runtime`: 換算しない。`！select` だけ秒のまま | 反 | 8 |
| 意味を知るト書きは 10 個 | `cue-command-runtime`: 8.4 の表 | △ | 16 |
| 転換 | `cue-command-runtime`: `CUE_BUILD.text`・`wait`・`finish`。取り決め: `\![play,trans,ミリ秒]` | ○ | — |
| アクター指定行は、シーンの頭にだけ | 今の文法。並びは `cue-command-runtime` の `\![play,cast,…]` | △ | 15 |
| 同じ台本を SSP でも areka でも。判定は Lua の公開関数 | `cue-command-runtime`: `pasta.baseware`。`novel-talk-flow` は `STAGE.is_areka` を前提 | △ | 17・22 |

### 2.5 戻る

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| 戻るのは、バックログの行を選んだ時とロードだけ | `novel-checkpoint-rollback`: `CHECKPOINT.jump` の 1 本 | △ | 18 |
| 戻るのは、変数・再生位置・アクター状態 | `novel-checkpoint-rollback`: `Checkpoint{pos, var, slot, actor}`。`head` と `pos.args` を足す | △ | 23 |
| 曲などは保存しない。戻った時に、階層の柱を打つ | `novel-checkpoint-rollback` は `CUE.pillar_script(pos, {full=true})` を求める。`cue-command-runtime` が渡すのは `CUE.apply_pillars` と `CUE_BUILD.full_state` | 二重・反 | 7・12 |
| 演劇のどの 1 本にも目印 | `novel-checkpoint-rollback`: 字の在る 1 本だけ。取り決め: 付ける。`novel-talk-flow`: クリック待ちと選択肢待ち | 二重・反 | 6 |
| 控えは 1 本ごと。着いたら取り直す | `novel-checkpoint-rollback`: 止まる所で取る（`seal`）、着いた時（`arrive`） | ○ | 1 |
| バックログの行を選ぶと、その控えへ戻り、後ろを捨てる | `novel-checkpoint-rollback`: `REG.OnPlayRollback`。取り決め: R4.2 | ○ | — |
| セーブは最新の控え。ロードは巻き戻しと同じ仕組み | `novel-save-slots`: 控えを入れ直して `\![raise,OnPlayRollback,ID]`。`novel-checkpoint-rollback`: `return CHECKPOINT.jump(…)` | 二重 | 5 |
| 戻るとき 2 本返す | `novel-checkpoint-rollback`: `land` が 1 本目を組み、末尾は `PLAY.next_tail()`。`novel-talk-flow` に、その口は無い | 無い（つなぎ） | 4 |
| 早回しで着く | `novel-checkpoint-rollback`: 組み立てた後に捨てる。`novel-talk-flow`: 組み立てる前に捨てる案 | 二重 | 5 |
| スロット変数（名前の頭） | `novel-checkpoint-rollback`: `SLOT_VARS`。`novel-save-slots`: `CHECKPOINT.copy_slot_vars` を前提 | 二重（名前） | 5 |
| セーブデータは pasta だけ。`save.toml` の予約の表。関数 4 つ | `novel-save-slots`: `pasta.shiori.slot` | △ | 5・24 |
| 外から消されたら、毎秒のイベントで出し直す | `novel-talk-flow`: `PLAY_EVENT.reshow` | △ | 7 |

### 2.6 遊ぶ人の操作

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| オートと「時間で次へ」 | `novel-talk-flow`: `＞オート（ミリ秒）`、枠が `xtimeout`・`timerraise` を出す。`cue-command-runtime`: `act._cue_xtimeout` | △ | 17・25 |
| 高速再生 | `novel-talk-flow`: `＞高速再生（１／０）`、頭に `\_q`。瞬間の命令を抜く口が 2 つ | △ | 17・25 |
| 既読は、再生位置で覚える | `novel-talk-flow`: `save.pasta_read`（書いた名前と行の番号） | △ | 23 |
| 設定の入口は、右クリック・ボタン・キーボード | ボタンとキーは今の条件の属性で書ける。右クリックを止める口は、どの設計にも無い | 無い（右クリック） | 26 |

### 2.7 舞台とアクター

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| 画面の部品は、全部アクター。役や種別を作らない | 取り決め: 役と席を `descript.txt` の独自キーに持たせる | △ | 28 |
| 舞台の開け閉めは、ト書きか柱（`＆舞台：開く`） | `cue-command-runtime`: `\![play,stage,開く／閉じる]`。取り決め: `\![play,stage,window／fullscreen／borderless]` と引数なし | 二重 | 15 |
| 窓の形は、遊ぶ人の設定 | 取り決めは「pasta が保存して打つ」。保存する設計が無い | 無い | 15 |
| 字は、トークの場所の箱へ | 取り決め R9.6（pasta は何もしない） | ○ | — |

### 2.8 取り決め・2.9 フェーズ

| 骨子の決まり | 作る設計と作り方 | 判定 | 指摘 |
| ------------ | ---------------- | ---- | ---- |
| 正典に在る物は正典で。`play` は正典に無い物だけ | 取り決め §6.1。位置・大きさ・透明度・重ね順は、SSP に正典のタグが在るのに `play` | 曖昧 | 28 |
| `play` の命令の一覧 | 取り決め §6.1（`unmark` を足す案）。`speaker` を書く台本の口は無い | △ | 28 |
| talking を立てたまま | 取り決め R3.3 | ○ | — |
| 取り決めは、内部設計の部の 1 章 | 取り決め: `book/src/internals/baseware-contract.md` | 曖昧 | 27 |
| フェーズ 1 は、今の SSP で遊べる | `hello-novel-sample`: フェーズ 1 の見本だけ。`cue-command-runtime`: SSP ではロードの後にアクター状態を書かない | △ | 7・22 |

### 4 章の A〜F・承認が要らない整理

| 骨子の決まり | 作る設計 | 判定 | 指摘 |
| ------------ | -------- | ---- | ---- |
| A 演劇は、モードを替えるまで続く | `novel-talk-flow` | ○（足取りは 3） | 3 |
| B 柱は上書き。ロードでは階層の柱を打つ。保存しない | `cue-command-runtime`・`novel-checkpoint-rollback` | △ | 7・12 |
| C 設定からの出口 3 つ | `novel-talk-flow`・`novel-save-slots` | △ | 5・9・18 |
| D 既読は再生位置で。声の仕組みは作らない | `novel-talk-flow` | △ | 23 |
| E バックログを捨てる決まり（取り下げ） | 取り決め §14.16・§14.17（開発者の判断へ） | ○ | — |
| F 途中の `＞` の先の柱は打たない | `cue-command-runtime` | ○ | 20 |
| 「今のシーン」は 1 つ | `novel-talk-flow` と `novel-checkpoint-rollback` が別々に持つ | 二重 | 3 |
| 出し直しの 1 本目は、いつも画面の状態を全部書く | `cue-command-runtime`: areka だけ | 反 | 7・12 |

### 5 章（持ち分）・6 章（要件定義へ送ること）

| 項目 | 状態 | 指摘 |
| ---- | ---- | ---- |
| 「柱 `＆再生モード` を打つ仕組みも」は `cue-command-runtime` | `novel-talk-flow` は、自分の関数を柱より前に呼ばせる形を求める | 1 |
| `shell-element-click`（起票のまま） | 中身は関数 2 つとテストと文書。`novel-talk-flow` は、同じ物を自分でも持つ | 14 |
| 持ち主が表に無い物 | 右クリックの口・窓の形の設定・並びの下のスコープ → アクター・着いた時の識別・新しい文法の検査（`pasta_check`） | 1・15・26・29 |
| 遅れて届いた続き | `novel-talk-flow` R5（ID を照らす）。ID の持ち主が 2 つ | 3 |
| 早回しの間に実行しない物 | 止める物の一覧が、設計ごとに違う | 5 |
| 控えを全部捨てた後のバックログ | 取り決めの `unmark` 案／`novel-checkpoint-rollback` の「ID を重ねない」。開発者の判断待ち | 30 |
| 再生モードの細かい所 | 控えを捨てる時（設定 → 演劇が未決）・`＄演劇` の設定の間の値（未決） | 9・30 |
| 設定の値を切り替える口 | オートと高速再生は `novel-talk-flow`。窓の形は無い。見本は別の名前で書いた | 15・25 |
| 「スロットにセーブが在るか」 | `novel-save-slots`: 作者の見出しの変数で見る。見本は、その形で壊れる | 24 |
| 舞台だけ開いている間のランダムトーク・既定の反応 | `novel-talk-flow` は変えない（未決のまま） | 30 |
| 見本とマニュアル | 段に分けるかで、2 本が逆を向く | 29 |

## 3. 指摘

指摘の番号は、2 章の表から引くための通し番号である。重さと区分で節に分けたので、節の中では番号が飛ぶ（1〜7 は重大、8〜13 は骨子の大、14〜17・24・25・29 は spec 間の大、残りは小の表）。

### 重大

#### 指摘 1（spec 間）「シーンに着いた」を知らせる口が、3 本で 3 つの形である

- 根拠:
  - `cue-command-runtime` 8.2・10 章: `CUE.define_pillar("再生モード", fn(act, args, ctx))`（書いたシーンでだけ呼ぶ）と、`CUE.on_arrive(fn(act, global_name, local_name))`（柱を積んだ後に呼ぶ。戻り値は使わない）。
  - `novel-talk-flow`「他の spec との境界」: `PLAY.arrive(act, here, mode)` を、柱を積む前に 1 回。`mode` は階層を引いた値。`PLAY_MODE_KEYS` は柱として流さない。
  - `novel-checkpoint-rollback` 5.1・10.2: `CHECKPOINT.arrive(act, ref)` を、柱を打った後に、演劇のときだけ。**戻り値が文字列なら、呼んだ側が `coroutine.yield` する。** `ref` に、着いた時の引数 `args` が要る。別に `ARRIVAL.next_is_arrival(act)` と、関数から登録名を逆に引く表（`scene_table_mt` の `__newindex`）を求める。
  - 今のコード: `act:init_scene(SCENE, 書いたグローバル名, 書いたローカル名, 行の対応)` は登録名も引数も受けない（`crates/pasta_lua/pasta_scripts/pasta/act.lua:221-233`、`crates/pasta_lua/src/code_gen/scope_gen.rs:273-277`）。
- 起きること: `cue-command-runtime` のとおりに作ると、(a) 柱を書かないシーンでは走り方の受け手が呼ばれず、`novel-talk-flow` は「今のシーン」を書けない。(b) 早回しの頭（行の番号 0）で 1 本目を返す道が無い。(c) 引数つきの最後の行の `＞` で着いたシーンを、頭から同じに走らせ直せない。
- 持ち主と直し方: 骨子 5 章のとおり、`cue-command-runtime` が持つ。口を 1 つにする。
  - 着いた時に渡す物を 1 つの表にそろえる: `{ global, local_, gname, lname, args }`（登録名は、`cue-command-runtime` が足す `init_scene` の 4 つ目の引数で足りる。逆引きの表は要らない）。
  - 呼ぶ順を決める: 着いたと判じる → 走り方を決める（`novel-talk-flow`）→ 柱を積む → 控えを取り直す（`novel-checkpoint-rollback`）。
  - 受け手の戻り値で「ここで 1 本返す」と言える形にする（早回しの頭）。
  - `＆再生モード` は、「登録された柱」か「柱として流さない名前」かを、どちらか 1 つに決める。

#### 指摘 2（spec 間）ト書きの頭で区切る役を、誰も持たない

- 根拠: `novel-talk-flow`（食い違いの 2）は、ト書きの入口から `PLAY.seal(act)` を呼んでもらう前提。`cue-command-runtime` 8.2 の `CUE.push` の手順に、その呼び出しは無く、10 章は「1 本の区切りの数え方に入らない」と書く。今の生成は、`！select` のほかのト書きを Lua に出さない（`scope_gen.rs:355-360`）ので、入口は新しく作る物である。
- 起きること: 骨子 2.2 の作例で、雷が前の 1 本に入る。

```pasta
＊嵐の教室　＆曲：雨音
    さくら：降ってきたね。
    ！効果音（雷）
    うにゅう：うわ、光った。
```

  「降ってきたね。」と一緒に雷が鳴り、「うわ、光った。」の 1 本には何も付かない。骨子の「ト書きは、直後のアクション行を修飾する」に反する。

- 持ち主と直し方: 区切りの判定は `novel-talk-flow`、呼ぶのは `cue-command-runtime`。`CUE.push` の頭（同じ値かの判定より前）で、登録された 1 つの関数を呼ぶ。継続行の前にト書きを挟んだ時の扱い（`novel-talk-flow` の検証の主要 1）も、同じ所で決める。

#### 指摘 3（spec 間）再生位置・今のシーン・行の番号・ID を、2 本が別々に持つ

- 根拠:
  - `novel-talk-flow`: `STORE.play.here`・`act._line`・`STORE.play.seq`（十進の通し番号）・`STORE.play.wait`。ゴーストに 1 つ。
  - `novel-checkpoint-rollback` 3.3・4.3・4.4: `STORE.checkpoint.tracks[co]`（コルーチンごと。弱い参照）の `pos.beat`・`last_id`。ID は `session .. serial`（例 `pm3k9q2n42`）。`CHECKPOINT.current_id()` を `\![raise,OnPlayNext,ID]` に使う前提。
  - 骨子 3 章は「演劇の状態は、呼んだ流れ（コルーチン）に付く」を取り下げた。4 章 A は「シーンが終わっても、今のシーンは変わらない」。今の `set_co_scene` は、終わったコルーチンを nil にする（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua:115-122`）。
- 起きること: (a) 最後の行まで読むと、`novel-checkpoint-rollback` の側の今のシーンが消える。(b) 設定の場面が台詞を出すと、続きが置き換わり（`init.lua:274-280`）、`CHECKPOINT.story_actor()` が引く先（`STORE.co_scene` の足取り）は設定の場面になる。(c) 目印の ID と `OnPlayNext` の ID が、2 つの数え方になる。(d) `STORE.play.seq` は読み直しで 0 に戻るので、前のセッションの行と重なる。
- 持ち主と直し方: 今のシーン・行の番号・待っている 1 本の ID は、`novel-talk-flow` がゴーストに 1 つ持つ（設定の 1 本にも ID が要るため）。`novel-checkpoint-rollback` は、それを読んで控えに写し、目印には同じ ID を使う。ID の形（セッションの頭を付ける）は、`novel-talk-flow` が受け継ぐ。コルーチンごとの足取りは「まだ続きになっていない間の仮の物」に限る。

#### 指摘 4（spec 間）戻る時の 1 本目から 2 本目へ、つながらない

- 根拠:
  - `novel-checkpoint-rollback` 5.1 の `land`: 1 本目は `\![play,again]` ＋ `CHECKPOINT.state_script(…)` ＋ `PLAY.next_tail()`。`PLAY.next_tail()` は `novel-talk-flow` に求める口（10.1 の 3）。
  - `novel-talk-flow` に `PLAY.next_tail` は無い。`REG.OnPlayNext` は `STORE.play.wait` が在り、`kind == "click"` で、Reference0 が `wait.id` と同じ時だけ再開する。1 本目は `PLAY_EVENT.frame` を通らないので、`STORE.play.wait` は置かれない。
  - 取り決め §7 の F4 と §14.2: 1 本目と 2 本目が同じ ID の `\![raise]` を持つと、「出す」と「進める」を見分けられない。
- 起きること: バックログの行を選ぶ・ロードする、のどちらでも、1 本目（画面の状態だけ）の後に届く `OnPlayNext` が捨てられ、2 本目が出ない。
- 持ち主と直し方: 「字も `\x` も無く、`\![raise]` で終わる 1 本」を、`novel-talk-flow` の枠の 1 つの種類にする（高速再生の 1 本と同じ形である）。その枠が、待っている物（種類と新しい ID）を置く。`novel-checkpoint-rollback` は中身（`\![play,again]` と画面の状態）だけを渡す。取り決め §6.4 の「末尾」にも、この形を足す（取り決めの検証の M1）。

#### 指摘 5（spec 間）早回しとロードの道が、設計ごとに違う

- 根拠（早回し）:
  - `novel-checkpoint-rollback` 5.1: `CHECKPOINT.seal(act, script, spoke)` は**組み立ての後**に呼ばれ、早回しの間は nil を返して捨てる。
  - `novel-talk-flow`: `act._play_target` が在る間、`PLAY.seal` が**組み立てずに**トークンを捨てる案。
  - `cue-command-runtime` 10 章: 「早回しの間に積まれたトークンと `act._cue_pending` を、組み立てずに捨てること」を求める（覚えた値は、組み立てで確定するため）。
  - 今のコード: 組み立ては `self._spoke` を立て、割り込みなら `＊チェイントーク中断` を走らせる（`crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua:72-107`）。
- 根拠（ロード）:
  - `novel-save-slots` 4.1・6.2: `CHECKPOINT.adopt(snapshot)` で控えを入れ替え、`\![raise,OnPlayRollback,ID]` を 1 片だけ出す。届いたイベントが 2 本を返す（往復が 1 回多い）。
  - `novel-checkpoint-rollback` 6.4・10.3: `スロット読込` は `return CHECKPOINT.jump(act, cp, {load=true})` を最後の行の呼び出しとして返す。`adopt`・`validate`・`EVENT` は無い。
  - 口の名前も合わない: `copy_slot_vars`／`SLOT_VARS.snapshot`、`replace_slot_vars`／`SLOT_VARS.replace`、`discard_all`／`clear`、`fast_forwarding`／`replaying`。`latest()` の意味も違う（`novel-save-slots`: 物語が待っていなければ nil。`novel-checkpoint-rollback`: 走っている足取りの控え → 列の最新 → nil）。
  - スロットの形も違う: `novel-save-slots` 5.1 は「スロット変数を平らに置き、残りを `pasta_checkpoint` に」。`novel-checkpoint-rollback` 4.2 は「`{v, pos, head, var, slot, actor}` を、そのまま `pasta_slots.<名前>` に」。
  - 早回しの間に止める物: `novel-save-slots` は `スロット保存` だけ。`novel-checkpoint-rollback` は保存・読込・消去・演劇再開の 4 つ。既読の記録（`novel-talk-flow` の枠の中）と、走り方の打ち手（`cue-command-runtime`）は、どちらの一覧にも無い。
- 起きること: 早回しのたびに、覚えた曲の値・外見の記録・既読・ID が進む。設定の画面の選択肢からロードすると、ゴーストに `＊チェイントーク中断` が在れば、ロードした物語が捨てられる（`novel-checkpoint-rollback` の検証の重大 2）。`novel-save-slots` の形では、`\![raise]` が届かない時に、控えだけが入れ替わる（同検証の議題 2）。
- 持ち主と直し方:
  - 戻る道は `CHECKPOINT.jump` の 1 本にする（骨子「2 本返す」「同じ仕組み」に、道ごと合う）。`novel-save-slots` は、それを最後の行の呼び出しとして返すだけにする。
  - 早回しで捨てる所は、組み立ての前の 1 か所（`novel-talk-flow` の区切り）にする。`novel-checkpoint-rollback` は、その 1 か所へ「今は早回しか・あと何本か」を答える。
  - 「早回しの間は何もしない物」を 1 つの一覧にして、骨子 6 章の持ち主（`novel-checkpoint-rollback`）の要件に置く: 保存・読込・消去・演劇再開・既読の記録・オートと高速再生の入切・柱の打ち手と知らせ・覚えた値の確定。
  - スロットの中の形は `novel-save-slots` が決める（`スロット参照` が読む形）。控えの形は `novel-checkpoint-rollback` が決める。詰め替えは `novel-save-slots` が深い写しで行う。

#### 指摘 6（spec 間）目印を付ける 1 本の範囲が、3 とおりである

- 根拠: 骨子 2.5「演劇のどの 1 本にも、目印が付く」、2.2「後ろにアクション行が無い残り…目印は付かない」。`novel-checkpoint-rollback` R2 は「字が在るとき」だけ。取り決め §10 の 8 は「pasta は付ける。字の無い 1 本は行を作らないので、使われない」。`novel-talk-flow` の枠は「演劇のクリック待ちと選択肢待ちの 1 本の頭」に置く所を空ける。目印の置き方も違う（`novel-talk-flow`: `BUILDER.build` の `head`。`novel-checkpoint-rollback`: 組み立ての外で、文字列の頭に足す）。
- 起きること: 表情だけの行（字が無くて待つ 1 本）で、設計が割れる。控えの数の数え方（brief は目印の数）も一緒に揺れる。
- 持ち主と直し方: 骨子のとおり「待つ 1 本には、どれも付ける。待たない残りには付けない」。`novel-checkpoint-rollback` の R2 を直す。置く所は、枠の頭の 1 か所にする（取り決め §6.4 の並び `[目印][出し直し]本体`）。

#### 指摘 7（spec 間）画面の状態を全部書く口が 3 つ在り、SSP ではロードの後に画面が戻らない

- 根拠:
  - `cue-command-runtime` 8.3・8.4: 位置・重ね順・大きさ・透明度の今の値は `STORE.cue_state`。`CUE_BUILD.full_state` は、areka では**覚えた値を全部**（曲と環境音も）書き、SSP では空文字列。
  - `novel-checkpoint-rollback` 4.1・5.2・10.2: 同じ値の置き場を `STORE.actor_attrs`（`ACTOR_STATE.set`）とし、「唯一の置き場所」と書く。1 本目は `CHECKPOINT.state_script` が組み、`CUE.pillar_script`・`CUE.cast_script`・`CUE.actor_attr_script` を求める（`cue-command-runtime` に無い）。アクターに付かない残る命令の記憶は、1 本目の前に「無い」に戻すことを求める。
  - `novel-talk-flow`: `ROLLBACK.state_tokens(act)`（トークン）と `STAGE.pillar_tokens(act, here)` を求める。
  - 骨子 2.5:「ト書きで途中に変えた音・ト書きで開いた舞台は、戻らない」。骨子 4 章:「出し直しの 1 本目は、いつも画面の状態を全部書く」。骨子 2.9: フェーズ 1（SSP）に「セーブとロード」と「位置と透明度と大きさ」が両方在る。
- 起きること: (a) 同じ値が 2 か所に置かれる。(b) `full_state` を巻き戻しに使うと、ト書きで途中に変えた曲と舞台まで「覚えた値」で書き直され、骨子 2.5 に反する。(c) SSP では、ロードの後の透明度・大きさ・位置が、ロードの前の画面のまま残り、覚えた値は戻した値になっているので、その後の同じ値のト書きも出ない（`cue-command-runtime` の検証の重大 1）。
- 持ち主と直し方:
  - アクターに付く値の置き場は 1 つにする。骨子 5 章のとおり、記録の持ち主は `novel-checkpoint-rollback`、書くのは `cue-command-runtime`（`ACTOR_STATE.set`）。`STORE.cue_state` は、アクターに付かない残る命令（曲・環境音・舞台・別名表の残る命令）だけにする。
  - 「全部書く」を 2 つに分ける（指摘 12）。**アクター状態**は、どのベースウェアでも全部書く。**状況**は、出し直しでは覚えた値、巻き戻しとロードでは階層の柱で決める。口は、`cue-command-runtime` が「アクターの値を書く」「柱を打つ（同じ値でも書く）」「覚えた状況を書く」の 3 つを文字列で返し、組むのは 1 本（`novel-checkpoint-rollback` の `state_script`）にする。

### 大（骨子に決めてもらう物）

#### 指摘 8（骨子）`！select` の時間の単位

- 根拠: 骨子 2.4「時間は、どれもミリ秒で書く」、手引きの例 `！select（２００００、時間切れ）`。今は秒で、組み立てで 1000 倍する（`crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua:456-458`、`scope_gen.rs:395-411`）。見本も秒（`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/09-choice.pasta:4,11`）。
- 設計の割れ方: `novel-talk-flow` と `cue-command-runtime` は秒のまま。`hello-novel-sample` はミリ秒（`！select（１００００、迷う）`）。このまま合わせると、時間切れは約 2.8 時間になる。持ち主も割れている（`cue-command-runtime` は `novel-talk-flow` へ、その検証は「単位は 2.4 の持ち分」）。
- 判断: **骨子の層の決定が要る。** 骨子の文と作例はミリ秒で、ぶつかるのは `novel-talk-flow` の brief の「今のゴーストの出力を 1 バイトも変えない」である。「演劇の間だけミリ秒」は場合分けなので採れない。決めるのは「例外を骨子に書く（秒のまま）」か「ミリ秒に替えて、入門とマニュアルを直す」かの 2 つ。

#### 指摘 9（骨子）設定の間の「今のシーン」・ローカルを探す範囲・`＄演劇` の値

- 根拠: 骨子 1 章「今のシーン＝最後に着いたシーン」「再生位置＝今のシーンと行の番号」、2.1「設定は、物語の再生位置を動かさない」、2.3「**演劇の間**、ローカルを先に探す」「`＄演劇` は、演劇でなければ空」。
- 設計の割れ方: `novel-talk-flow` は「設定の間も、今のシーンは設定の場面へ移り、物語は別に控える。`＄演劇` は設定の間も名前」。`shell-element-click` と `hello-novel-sample` は、設定の場面のローカル `・OnMouseClick` でボタンを書く前提。骨子の文のとおりなら、設定の間は `＄演劇` が空で、いつもの反応が設定の画面の間に走る。
- 判断: **骨子に 2 文足す。** 決まりの中身は変わらない。「設定の間、今のシーン（ローカルを探す先）は設定の場面へ移る。物語の再生位置は、止めたまま別に持つ」「`＄演劇` は、演劇と設定の間は、今のシーンのグローバルシーンの名前である」。骨子 6 章の候補 `＄再生モード` を採らないことも、ここで決める。

#### 指摘 10（骨子）台詞を出さないイベントのシーンに着いた時、柱を打つか

- 根拠: 骨子 2.3「台詞を出さないシーンは、続きに触れない」、2.4「柱が打たれるのは、イベントで…着いた時」。`cue-command-runtime` 7.1 は、台詞の有無によらず柱を積む。柱のトークンは台詞に数えない（要件 1.6）。`novel-talk-flow` の `PLAY.commit` は、台詞を出さない割り込みでは、今のシーンも走り方も動かさない。
- 起きること（設計も検証も書いていない帰結）: 台詞の無いシーンが柱だけを積むと、組み立ては空でない文字列を返す。取り決め R6.1 では、待ちの間に返したさくらスクリプトは、待っている 1 本を置き換える。つまり、今のシーンは動かないのに、曲は替わり、遊ぶ人の画面からは待っていた行（と `\x\![raise]`）が消える。次の毎秒のイベントで出し直しが走る。

```pasta
＆曲：昼のテーマ

＊OnMouseMove
    ＄＊撫でた数＝＄＊撫でた数＋１
```

  演劇の間に撫でると、台詞は無いのに `\![sound,loop,昼のテーマ]` が返り、読んでいた行が消える。

- 判断: **骨子に 1 文足す。** 骨子の「続きに触れない」と揃えるなら、「台詞を出さなかったイベントのシーンは、着いたと数えない（柱を打たない・今のシーンを動かさない）」である。作るには、柱のトークンと覚えた値の確定を、続きが置き換わると決まった時まで待つ形になる（`cue-command-runtime` と `novel-talk-flow` で決める）。

#### 指摘 11（骨子）ロードと巻き戻しで、再生モードが階層から決まらない場合が在る

- 根拠: 骨子 2.1「書いていなければ前のモードが続く」と、2.5「戻った時に、階層の柱を打って決める（保存しない）」。物語のシーンの階層に `＆再生モード` が無い（演劇のシーンから最後の行の `＞` で着いただけ）とき、設定の画面からロードすると、階層を引いても値が無く、設定のままになる。`novel-talk-flow` は `PLAY.set_continuation({mode=…})` で走り方を運ぶ（＝控えに入れる）形で逃げている。
- 判断: **骨子に 1 文足す。** 控えは演劇の 1 本でしか作られないので、「控えへ戻った先の走り方は、いつも演劇である」と言える。保存せずに導ける。

#### 指摘 12（骨子）「出し直しの 1 本目は、いつも画面の状態を全部書く」の「画面の状態」

- 根拠: 骨子 4 章の整理は、areka が出し直しの頭で残る命令を全部既定へ戻すことを前提にする。骨子 2.5 は「ト書きで途中に変えた音・ト書きで開いた舞台は、戻らない」。`cue-command-runtime` は「全部書くのは areka だけ」と、ベースウェアで分けた（場合分け）。取り決めの検証の C1: `again` が舞台を既定（閉じている）へ戻すと、閉じる時の決まりで目印が全部外れる。
- 判断: **骨子の言い直しが要る。** 次の形なら、ベースウェアで分けずに済む。
  - 出し直しの頭は、アクター状態を全部書く（どのベースウェアでも）。
  - 状況（曲・環境音・舞台・色調など）は、単に出し直す時（`＞演劇再開`・外から消された時）は触らない。巻き戻しとロードでは、階層の柱を打つ。
  - 取り決めの `again` が既定へ戻すのは、アクターに付く値だけにする（舞台と音は戻さない）。これで C1 も消える。

#### 指摘 13（骨子）いつもの反応を止める 1 行が、つつき・撫での近道と中断のシーンに届かない

- 根拠: 骨子 2.3「いつもの反応には、条件 `＆＄演劇＝「」` を書く」。近道は `SCENE.co_exec_exact` で引き、条件を見ない（`crates/pasta_lua/pasta_scripts/pasta/shiori/event/mouse.lua:117-124`、`crates/pasta_lua/pasta_scripts/pasta/scene.lua:255-262,284-290`）。`novel-talk-flow` は、演劇と設定の間は近道を探さず、`＊チェイントーク中断` も掛けない（R6.3・R6.4）。`hello-novel-sample` は、近道を使わずに避けた。
- 判断: **開発者の決定が要る。** 走り方で止めるのは、骨子に無い自動の動きである。条件を見て引く形にすれば、骨子の 1 行がそのまま効き、場合分けが要らない（`scene-exact-match` と揃える）。

### 大（spec 間）

#### 指摘 14（spec 間）ローカルを先に探す口と、選択肢の振り分けの取り出しが、2 本に在る

- 根拠: `novel-talk-flow`: `PLAY_EVENT.event_scene(act, name)`（`SCENE.search(name, here.global, act)` の後に `SCENE.co_exec`）と、`CHOICE_ROUTE.to_scene`（無ければ自分で取り出す）。`shell-element-click`: `SCENE.search_scoped`・`ROUTE.to_scene`・`PLAY.scope()`（`novel-talk-flow` に無い）。探し方が入るのは `EVENT.no_entry`（`init.lua:222-226`）と `mouse.lua` の 9 段目だけで、既定のハンドラを持つイベント（`OnBoot` など）には効かない（`shell-element-click` の検証の指摘 3）。
- 直し方: 探し方の持ち主は `novel-talk-flow`（骨子 5 章）。`shell-element-click` の残りは、選択肢の振り分けの取り出し・クリックのテスト・マニュアルの節・取り決めの文面である。残すか、`novel-talk-flow` の持ち場へ入れて閉じるかは、棚卸（`/kiro-next`）で決める。どちらでも、「イベントのシーンを探す所は、どのイベントでも同じ 1 つの関数を通る」を `novel-talk-flow` の要件に書く。

#### 指摘 15（spec 間）舞台と並びの命令の形が、取り決めと `cue-command-runtime` で違う

- 根拠:
  - 舞台: `cue-command-runtime` 8.4 は `\![play,stage,開く／閉じる]`。取り決め §6.1 は `\![play,stage,window／fullscreen／borderless]` で開き、引数なしで閉じる。「窓の形は、pasta が保存して打つ」（取り決め §6.1 の注）が、保存する設計は無い（骨子 6 章の「窓の形の入切」の持ち主は `novel-talk-flow` と `cue-command-runtime`）。
  - 並び: 取り決め §10 の 7 は「番号の位置に名前を置き、空いた番号は空の欄」（`\![play,cast,教室,,さくら]`）。`cue-command-runtime` は名前を並べるだけ。`novel-checkpoint-rollback` は並びを `STORE.actor_spots` で表すが、この表は読み込みの時に `pasta.toml` の `actor.spot` で埋まる（`crates/pasta_lua/pasta_scripts/pasta/store.lua:111-121`）ので、舞台に出ていないアクターも入る。
  - 並びの下で、スコープの番号からアクター名を引く口（`shell-element-click` 17 章の 6）は、どの設計にも無い。
- 直し方: 英字の層の形は取り決めが正。`cue-command-runtime` は、柱の値「開く」を、遊ぶ人の設定の形へ直して出す。設定の値の置き場（スロット変数でないグローバル変数）と切り替える口は、オート（`save.pasta_play_auto`）と同じ形で `novel-talk-flow` が持つ。並びは「アクター指定行が書いた物だけ」と決め、番号 → 名前の表は `cue-command-runtime` が 1 つ持つ。

#### 指摘 16（spec 間）別名表の既定と、SSP への出し方

- 根拠: 取り決め 11 章は、既定値の例に `転換`・`透明度`・`大きさ`・`位置`・`舞台` を挙げる。`cue-command-runtime` は、この 5 つを「意味を知る名前」として別名表に書けない誤りにし（要件 2.4）、既定は 5 行（揺らす・フラッシュ・重ね順・色調・カメラ）。`cue-command-runtime` は、頭が `play` の別名を SSP にもそのまま出す（骨子 2.4「直せない物は、そのベースウェアでは出さない」に反する。検証の主要 6）。重ね順は、骨子 1 章でアクター状態だが、別名表の形に「アクター状態」の印が無く、控えに入らない（検証の主要 5）。
- 直し方: 取り決めは英字の命令の表だけを持ち、別名表の既定の中身は `cue-command-runtime` が決める（取り決め 11 章の例を消す）。頭が `play` の命令は、areka でなければ出さない。「対象の付く残る命令は、アクター状態」の 1 つの決まりで、重ね順を控えに入れる。

#### 指摘 17（spec 間）小さい口の名前と置き方（まとめ）

| 物 | `cue-command-runtime` | `novel-talk-flow` | 取り決め |
| -- | --------------------- | ----------------- | -------- |
| その行だけ時間で次へ | `act._cue_xtimeout` に値を残す | `PLAY.set_line_timeout(act, ms)` を呼ばせる | — |
| 瞬間の命令を出さない | `config.cue_instant = false` | トークンの `instant = true` を見て抜く | — |
| ベースウェアの判定 | `BASEWARE.kind()` | `STAGE.is_areka(act)` | `Sender` と `basewareversion` |
| `xtimeout` の位置 | — | 1 本の頭 | `\x` の直前 |
| 走り方を引く | — | `PLAY.mode_of(act)`（`novel-checkpoint-rollback` は `PLAY.mode(ref)`） | — |

- 直し方: 左の列（持ち主の形）に合わせる。`xtimeout` の位置は、取り決め R6.2 が「含む 1 本」としか言わないので、どちらでも動く。取り決め §6.4 の文法を、決めた方に合わせる。

#### 指摘 24（spec 間・骨子）セーブの見出しと、「セーブが在るか」

- 根拠: 骨子 2.5「控えは、その 1 本の始まりの状態」「セーブは、いちばん新しい控えを書く」。`novel-save-slots` の brief「見出しは、作者が決めたスロット変数で持つ」。同じシーンの頭で見出しを書いて保存しても、控えは代入の前の値を持つ。`hello-novel-sample` 5.2 は、まさにその形で書いていて、「つづきから」が出ない（同検証の C2）。保存した日時は、控えに入りえない。
- 判断: **骨子の層で決める。** 候補は `novel-save-slots` 12 章の (a)〜(c)。原則「保存より、シーンの属性と階層から導く」に合うのは (c)（見出しは、保存した再生位置のシーンから読む。日時だけ pasta が保存の瞬間に足す）。「セーブが在るか」も、作者の変数でなく、スロットの有無から答える形にする。

#### 指摘 25（spec 間）オートと高速再生の口

- 根拠: `novel-talk-flow` は値を置く呼び出し（`＞オート（ミリ秒）`・`＞高速再生（１／０）`）。`hello-novel-sample` は切り替える呼び出し（`＞オート切替`・`＞高速再生切替`）。高速再生の今の値を読む口は無い。手引き 5 章は「オートと高速再生: 遊ぶ人の設定。台本には書かない」と書き、骨子 6 章（設定のシーンから切り替える口）と揃わない。
- 直し方: `novel-talk-flow` の要件で、口の形と、今の値を読む口（どちらも変数で読める形が軽い）を決める。見本は、その形で書く。手引きの 1 行は直す。

#### 指摘 29（spec 間）見本とマニュアルが、逆を向いている。共有するファイルの数え違い

- 根拠:
  - `manual-novel-guide` は、見本に「1 段 1 ファイルの `NN-name.pasta`、段 N は段 1〜N の集まり、前の段を書き換えない」を求め、`tutorial-check.mjs` で章と照合する。`hello-novel-sample` は「段に分けず、仕上がった 1 本」（6 章の 10）で、ファイルは `01-actors`・`10-story`・`20-system`・`30-desktop`。物語は最後の行の `＞` でつながるので、段ごとに 1 枚足す形では、前の段の飛び先が無い（`manual-novel-guide` の検証の問題 1）。
  - 新しい文法（選択肢の条件・時間切れの飛び先・ト書きの変数）の `pasta_check` の検査のうち、時間切れの飛び先の検査の持ち主が無い（`hello-novel-sample` の検証の M9）。
  - `GLOBAL_NAMES` の数: `novel-talk-flow` は 16 → 22、`novel-save-slots` は 16 → 24 と書く（両方入ると 30）。内部設計の章の数: 取り決めは 10 → 11。`novel-checkpoint-rollback` も `internals/checkpoint.md` を足すが、`verify-content.mjs`・`verify-scripts-test.mjs` の数を挙げていない。`event/init.lua`: `shell-element-click` は「触らない」前提だが、3 本（`novel-talk-flow`・`cue-command-runtime`・`novel-checkpoint-rollback`）が触る。`store.lua`: 3 本が欄を足す（指摘 7 の二重を含む）。
- 直し方: 見本の形（段に分けるか・舞台の見本を別の spec にするか）を、2 本の要件定義の前に 1 回で決める。数は、先に入った spec の後の数から数え直す、と各設計に書く。

### 小（骨子・手引きの言葉）

このうち、23（控えの形と再生位置の鍵。スロットのファイルの形に効く）と 26（右クリックの口。骨子 2.6 の入口の 1 つを作る設計が無い）は、重さは「大」である。決めることが 1〜2 文で済むので、この表に入れた。

| 番号 | 区分 | 中身 | 直し方 |
| ---- | ---- | ---- | ------ |
| 18 | 骨子 | `＞演劇再開` は「何も戻さない」だが、`novel-checkpoint-rollback` の R16 は、設定の場面で変わった並びと表情を、物語の物へ置き直す（「戻るのはロードとバックログだけ」に触れる） | 骨子に 1 文: 「出し直しは、待っている行のアクター状態を書く。変数と再生位置は戻さない」。指摘 12 と一緒に書く |
| 19 | 骨子 | 作例 `＞消えて現れる（さくら、窓際）` は、今の文法では読めない。呼び出しの引数は式で、裸の名前は無い（`crates/pasta_dsl/src/parser/grammar.pest:70-76,113-116`）。見本も同じ形で読み込めない | 文法は足さず、骨子・手引き・brief の作例を `＞消えて現れる（「さくら」、「窓際」）` に直す |
| 20 | 骨子 | 途中の `＞` の先の中の、最後の行の `＞` を「着いた」と数えるか。ト書きだけの束の残りが、束の終わりで待たない 1 本になるか、呼んだ側の次の 1 本に入るか | 骨子に 1 文: 「途中の `＞` で呼んだ先の中の `＞` と残りは、どれも呼んだ側の続きとして数える」。2 本の設計は、もうこの読みで書いている |
| 21 | 骨子 | 柱として打つ名前の範囲。骨子 2.4 をそのまま読むと、どの属性も流れる（`＆天気：晴れ` が `\![天気,晴れ]` になる） | 骨子に 1 文: 「柱として打つのは、意味を知る 10 個・別名表の名前・登録された柱だけ」（`cue-command-runtime` の仮定のまま） |
| 22 | 骨子 | 「同じ台本を、SSP でも areka でも」（2.4）と「背景もアクター」（2.7）。背景を並びの頭に置くと、SSP では背景が `\p[0]` の窓になる | 開発者の読みを 1 文で: 「書き方は 1 つ。舞台の台本を SSP で遊べるとは約束しない」。見本は、フェーズ 1 の台本に背景を入れない |
| 23 | 骨子 | 今のシーンの識別（書いた名前か登録名か）と、控えの形。既読は書いた名前（同名の `＊結末` 2 つを分けられない）、控えは登録名（台本を直すと、別のシーンを黙って指す）。控えに `head`（着いた時の変数）と `args` を足す案 | 骨子に 2 文: 再生位置の鍵と、早回しを始める変数。決まるまで、スロットの形を固めない |
| 26 | 骨子 | 右クリックのメニューを止める口（リソース `sakura.popupmenu.visible`）に、持ち主と書き方が無い。pasta にリソースへ答える口は無い（`crates/pasta_lua/pasta_scripts` に `popupmenu` の記述が無い）。シーンの出力は `\e` で終わる（`sakura_builder.lua:686`）ので、値を返せない | 持ち主を骨子 5 章に足す。書き方は、宣言なので属性（今のシーンの階層を引いて答える柱）が原則に合う。決まるまで、見本はボタンとキーで書く |
| 27 | 骨子 | 取り決めの章を内部設計の部に置くと、その部の決まり（未実装を書かない・利用者向けの章から逆リンクしない）とぶつかる（`book/src/internals/index.md:17,70`、`book/AUTHORING.md:237,239`） | 開発者が、例外を足すか、リファレンスの部へ置くかを決める。`manual-novel-guide` のリンクは、その後に書く |
| 28 | 骨子 | (a) 位置・大きさ・透明度・重ね順は、SSP に正典のタグが在るのに `play` の命令である（2.8 の「正典を先に」と合わない）。(b) 「役を作らない」（2.7）と、取り決めの役・席。(c) `speaker` を台本から書く口が無い | (a)(b) は、取り決めの要件定義で 1 文ずつ理由を書く。(c) はフェーズ 2 なので、今は「無い」と記録するだけでよい |
| 30 | 骨子 6 章 | 要件定義へ送られたまま、設計でも決まらなかった物: 控えを全部捨てた後のバックログ（`unmark`／空にする／ID を重ねない）・設定から演劇へ入った時に控えを捨てるか・舞台だけ開いている間のランダムトーク・既定の反応を演劇の間に出すか・ローカル変数の日時と Reference を戻すか | どれも、持ち主の spec の要件定義の議題にする。骨子は変えない |
| 31 | 手引き | `play-script-guide.md` の古い行: 「残る命令は、pasta が最後の値を覚え、巻き戻し・ロードで作り直す」（骨子 2.5 は、状況を柱から決める）・重ね順をフェーズ 1 に置く表（骨子 2.9 に無い）・「条件なしの `・OnMouseClick` を足す」（右クリックと設定のボタンまで拾う） | 手引きを骨子に合わせて直す。3 つ目は `・OnMouseClick　＆＄ｒ５＝０` の形にする |

## 4. 骨子の層へ上がった差し戻しと未決（重ねを除いた一覧）

設計と検証が「骨子の層」と書いた物を、全部並べた。「要るか」は、この突き合わせの見立てである。

| # | 中身 | 挙げた所 | 骨子の決定が要るか | 指摘 |
| - | ---- | -------- | ------------------ | ---- |
| 1 | `！select` の単位 | `novel-talk-flow`・`hello-novel-sample`・`cue-command-runtime` と各検証 | **要る**（中身の決定） | 8 |
| 2 | 設定の間の今のシーン・探す範囲・`＄演劇` | `shell-element-click`・`novel-talk-flow`・`hello-novel-sample` | **要る**（2 文の補い） | 9 |
| 3 | 近道と中断に条件が効かない | `hello-novel-sample`・`novel-talk-flow`・`shell-element-click` の検証 | **要る**（どちらで止めるか） | 13 |
| 4 | 同じ台本と、背景のアクター | `hello-novel-sample` | **要る**（読みの 1 文） | 22 |
| 5 | 右クリックのメニューを止める口 | 取り決め E1・`hello-novel-sample` | **要る**（持ち主と書き方） | 26 |
| 6 | 取り決めの章の置き場所 | 取り決め E2 | **要る**（軽い。文書の決まり） | 27 |
| 7 | セーブの見出し | `novel-save-slots`・`hello-novel-sample` の検証 | **要る** | 24 |
| 8 | 打つ柱の名前の範囲 | `cue-command-runtime` | 要る（1 文。仮定のままでよい） | 21 |
| 9 | 出し直しの「全部書く」と SSP | `cue-command-runtime`・`novel-checkpoint-rollback`・取り決めの検証 | **要る**（言い直し） | 12 |
| 10 | 台詞を出さないシーンの柱 | `cue-command-runtime` の検証 | **要る** | 10 |
| 11 | ロードで走り方が決まらない | `novel-talk-flow` の検証 | 要る（1 文） | 11 |
| 12 | 控えの `head`・`args`・再生位置の鍵 | `novel-checkpoint-rollback`・`novel-save-slots` の検証 | 要る（2 文） | 23 |
| 13 | `＞演劇再開` がアクター状態を置き直す | `novel-checkpoint-rollback` の検証 | 要る（1 文） | 18 |
| 14 | 途中の `＞` の先の `＞`・束の残り | `cue-command-runtime`・`hello-novel-sample` の検証 | 要る（1 文。読みは揃っている） | 20 |
| 15 | 作例の裸の名前 | `cue-command-runtime`・`hello-novel-sample` の検証 | 要る（作例を直すだけ） | 19 |
| 16 | 正典のタグが在るのに `play`・役 | 取り決めの検証 | 要らない（取り決めの要件定義で理由を書く） | 28 |
| 17 | 早回しのずれの範囲（`＆順番`・スロットでない変数の分岐） | `novel-checkpoint-rollback` の検証 | 要らない（骨子の「受け入れる」の帰結。マニュアルの 1 行を広げる） | — |
| 18 | 字の無い 1 本の目印 | `novel-checkpoint-rollback`・取り決め | 要らない（骨子は決めている。設計を合わせる） | 6 |
| 19 | ロードに要る口が 2 つでは足りない | `novel-save-slots` | 要らない（spec 間の口） | 5 |
| 20 | フェーズ 2 の縮小画像 | `novel-save-slots` | 要らない（フェーズ 2 の取り決めで） | — |
| 21 | `shell-element-click` の存続 | `shell-element-click` | 要らない（棚卸で決める） | 14 |
| 22 | 舞台を閉じた時に、pasta の記録を空にする | `cue-command-runtime` の検証 | 要らない（取り決めで areka の動きを決めてから導く） | — |
| 23 | 骨子 6 章に残った未決 | 各設計 | 要らない（各 spec の要件定義） | 30 |

## 5. 読んで確かめたコード

| 事実 | 場所 |
| ---- | ---- |
| `！select` は秒で受け、1000 倍して出す | `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua:456-458`、`crates/pasta_lua/src/code_gen/scope_gen.rs:395-411` |
| 見本の `!select(10)` は秒 | `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/dic/09-choice.pasta:4,11` |
| 組み立ては、最後に `\e` を付ける | `sakura_builder.lua:686` |
| 呼び出しの引数は式で、裸の名前は無い | `crates/pasta_dsl/src/parser/grammar.pest:70-76,113-116` |
| ト書きの対象と引数に、変数の規則は無い。選択肢の行に条件は無い | `grammar.pest:222-231,236` |
| `select` のほかのト書きは、Lua を生成しない | `scope_gen.rs:355-360` |
| ローカルシーンの倉は、親の属性を受け継がない | `scope_gen.rs:116-125` |
| `init_scene` は、書いた名前 2 つと行の対応だけを受ける。アクター指定行は `__start__` だけ | `scope_gen.rs:273-292`、`crates/pasta_lua/pasta_scripts/pasta/act.lua:221-233` |
| `act:line()` は引数を取らず、アクション行と継続行で同じ行が出る | `act.lua:248-254`、`crates/pasta_lua/src/code_gen/element_gen.rs:282,304,316-321` |
| 途中の `＞` は `restore_scene` を通る | `act.lua:800-808,821-823` |
| 再開しても `act.req` は替わらない | `act.lua:683-697` |
| 名前の探し方は、ローカル → act のメソッド → `GLOBAL` の完全一致 | `act.lua:409-431` |
| イベントのシーンは `SCENE.co_exec(act, act.req.id, nil, nil)` で探す | `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua:222-226` |
| 終わったコルーチンは、続きに置かない。台詞の無い割り込みは、続きを残す | `init.lua:115-122,268-280` |
| 割り込みには、中断のシーンが掛かる | `init.lua:239-248` |
| つつき・撫での近道は、完全一致で引き、条件を見ない | `crates/pasta_lua/pasta_scripts/pasta/shiori/event/mouse.lua:117-124`、`crates/pasta_lua/pasta_scripts/pasta/scene.lua:255-262,284-290` |
| `SCENE.wrap` は、最後に `act:build()` を返す | `scene.lua:221-226` |
| `STORE.actor_spots` は、読み込みの時に `pasta.toml` の `actor.spot` で埋まる | `crates/pasta_lua/pasta_scripts/pasta/store.lua:111-121` |
| 条件で、値の無い変数は空の文字列として比べる | `crates/pasta_lua/src/search/scene_cond.rs:53-58` |
| pasta に、リソース `popupmenu` へ答える口は無い | `crates/pasta_lua/pasta_scripts`（検索で 0 件） |
| 内部設計の部の決まり（権威は現行実装・逆リンクしない） | `book/src/internals/index.md:17,70`、`book/AUTHORING.md:237,239` |

## 6. 確かめていないこと

- どの設計も、動かしていない。この突き合わせも、文書とコードを読んだだけである。
- areka の側の事実（バックログ・舞台・`\![raise]`・版の名乗り）は、取り決めの設計の記述に頼った。areka のリポジトリは読んでいない。
- SSP の実機の事実（`\![play,…]` を読み飛ばす・`\![sound,loop]` の打ち直し・`\![move]` の基準・`\![raise]` の後の talking）は、各設計が引いた調査ノートの記述に頼った。
- `manual-novel-guide` の暫定設計は、要約と検証と、食い違いの章だけを読んだ。本文の章立ての細目は、照らしていない。
- 各検証の「小さい」指摘は、spec 間と骨子に関わる物だけを拾った。spec の中で閉じる物は、ここに写していない。
