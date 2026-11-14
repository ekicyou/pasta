# ギャップ分析: onhour-talk-recount

- 日付: 2026-11-14
- 対象: `requirements.md`（生成済み・未承認）と、この作業ツリーのコード（base `89657ec1`。main `cc7b3353` の上に spec の初期化だけ）
- 方法: `kiro-validate-gap` の枠組み（現状調査 → 要件と資産の対応 → 実装の選択肢 → 工数とリスク → 設計への申し送り）。外部の資料は ukadoc の `OnSysResume` だけを引いた

## 1. 分析の概要

- **仮実装の差分は、この作業ツリーには無い**。`virtual_dispatcher.lua` L189-192 の候補ループは今も `if t then return t end` のままで、`virtual_dispatcher_spec.lua` L978 のテスト名は「おしゃべりはすぐ出る」、L987 は `thread` を期待している。brief の差分の全文から写し直すことになる。
- **時報の数え直し（要件 1）は小さい**。候補ループで thread が返ったときに `next_talk_time = calculate_next_talk_time(CLOCK.steady(act.req))` を入れるだけで、要件 1.1〜1.6・2.2〜2.4・2.6 は今の門と状態の作りから自然に満たされる。
- **スリープ明けのおしゃべり（要件 3・4）は、既定ハンドラの今の作法（`REG.X = REG.X or function`）にそのまま乗る**。`REG.OnSysResume` の既定ハンドラを置けば、鎖・代入・シーンの判定（要件 4.1〜4.4）は `EVENT.fire` の今の引き方で決まる。足すのはディスパッチャの公開の関数 1 つと、それが持つ状態である。
- **要件の側に、コードと食い違う点が 3 つある**（§4）。(a) 要件 2.1・6.6「時報の後も続きを捨てない」は、台詞を出した時報が続きを捨てる今の決まり（`chain-talk-interrupt`）と合わない。(b) 長いスリープの後は、今でも復帰後の最初の秒で溜まった OnTalk が出る（`OnSysResume` より前）。要件 3.1 をそのまま足すと、続けて 2 回目のおしゃべりが出る。(c) 要件 3.4 の順番（時報 → 数秒後に `OnSysResume`）は、brief の推奨案「`OnSysResume` で `next_talk_time` を今にするだけ」では満たせない。
- 工数は S、リスクは Low〜Medium（スリープ明けの順番の扱いと実機での確かめだけが Medium）。

## 2. 現状調査

### 2.1 関わるファイルと役目

| ファイル | 役目 | この機能との関わり |
|---|---|---|
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/virtual_dispatcher.lua` | 仮想イベント（時報・OnTalk）の判定と発行。状態は `next_hour_unix`・`seen_epoch`・`next_talk_time` のモジュールローカルだけ | 数え直しの置き場所（`check_hour` L186-193）。スリープ明けの「今にする」口も、`next_talk_time` を持つここに置くのが自然 |
| 同 `check_talk`（L199-234） | 期限前・`hour_margin`（L216-220）で `nil`。発行するときは L223 で決め直し、`STORE.co_scene` があればそれを返す（L227-230） | 要件 1.5・3.3・3.5・3.8 は今のままで満たす |
| 同 `dispatch`（L239-264） | `date` 無し → Status ブロック → 止める門（L252）→ `check_hour` → `check_talk` | 要件 2.2・2.4・3.3・3.6 の門。触らない |
| 同 `restart_timers`（L269-283） | 再開の時に `next_talk_time = 0`（初回扱い） | 要件 3.6「止めている間の `OnSysResume` を持ち越さない」は、ここで消える。新しい状態を足すなら、ここでも消す |
| 同 `_reset`・`_get_internal_state`（L288-303） | テスト用 | 新しい状態を足すなら両方に入れる。マニュアルの「テスト用関数」節（`shiori-events.md` L1482-）も直す |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` | `EVENT.fire`: コールバック → `CLOCK.observe` → 鎖（L305-312。真の値で止まる）→ 底 `rawget(REG, id) or EVENT.no_entry`（L315-319） | 要件 4 の判定は、この順番がそのまま担う。既定ハンドラの読み込みの並び（L85-91。末尾は `recommend`） |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/boot.lua` | `REG.OnBoot = REG.OnBoot or function(act) return SCENE.co_exec(act, act.req.id, nil, nil) end` | 「シーンがあればそれ、無ければ既定の動き」の手本。循環参照を避けて `pasta.shiori.event` を require せず、`pasta.scene` を直接使う |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/event/second_change.lua` | `REG.OnSecondChange` の既定。`dispatcher.dispatch(act)` を呼ぶ | 触らない |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/clock.lua` | `observe` が `OnSysResume`・`OnSysSuspend` で印を立て（L155-156）、`watch_jump`（L123-143）がスリープ明けの差を飛びに数えない | 触らない（要件 3.10）。ただし「復帰した」ことを外に知らせる口は今は無い（§4.2） |
| `crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua` | `GLOBAL.pause_auto_talk`／`resume_auto_talk`（L31-46）。再開で `restart_timers(act)` | 触らない |
| `crates/pasta_lua/pasta_scripts/pasta/scene.lua` `SCENE.co_exec`（L238-244） | `act:find_scene` → `find_act_handler` の 5 段（current_scene → ローカル前方一致 → act のメソッド → `GLOBAL[key]` → グローバル前方一致）。条件漏れでも `nil` | 要件 4.1・4.4 の「シーンがある／見つからない」はこれで決まる。`GLOBAL.OnSysResume` の関数も「受けている」に数わる点に注意 |

### 2.2 守るべき作法

- 既定ハンドラは `REG.X = REG.X or function`。ゴーストが `scripts/main.lua` で先に代入していれば上書きしない。辞書の Lua ブロック（後から評価）で代入すれば既定を置き換える（`shiori-events.md`「既定ハンドラと上書き」L137）。
- 鎖のハンドラが真の値を返せば止まり、底（`rawget(REG, id)`）は呼ばれない。全部見送れば底を呼ぶ（`register.lua` L45-61、`init.lua` L301-319）。要件 4.3 の「真の値で止めた回は既定を行わない・全部見送った回は受けていない」は、既定を底に置けばそのまま満たす。
- `event/*` のモジュールは `pasta.shiori.event` を require しない（循環を避ける）。`clock.lua` は `pasta.shiori.event.*`・`pasta.store` を require しない。
- テストは `crates/pasta_lua/tests/lua_specs/*.lua` に置き、新しいスイートは `lua_specs/init.lua` の `specs` に足す。依存の閉包を読み直すスイート（`chain_talk_interrupt_test.lua` L13-37・`recommend_menu_test.lua` L421 付近・`reg_handler_chain_test.lua` L625 付近・`shiori_entry_test.lua`）は、既定ハンドラのモジュールを並べている。モジュールを足すなら、この並びにも足すのが今の流儀。
- マニュアルが権威。`book/src/` を直してから `node book/tools/gen-skill-refs.mjs` で `.claude/skills/pasta-lua-coding/references/shiori-events.md` を作り直す（`--check`・`link-check.mjs` が CI で見る）。
- テストの前に `NoDefaultCurrentDirectoryInExePath` を外す。`cargo test -p pasta_lua` が書き換える `sample.generated.lua` は戻す。

### 2.3 外の仕様（ukadoc）

- `OnSysResume`: サスペンド（スリープ・休止状態の両方）の解除で発生。Reference0 は解除の理由で、`normal`・`auto`（システムのスケジュールによる自動の解除）・`critical`（Vista 以降廃止）。対応は SSP。
- ukadoc の項目そのものには GET か NOTIFY かが書かれていない（brief は GET としている）。要件 3.2 は応答でトークを返さないので、どちらでも動きは変わらない。

## 3. 要件と資産の対応

タグ: **Exists**（今のまま満たす）／**Missing**（足す）／**Constraint**（今の作りが縛る）／**Unknown**（確かめが要る）／**Conflict**（要件とコードが食い違う）

| 要件 | 対応する資産 | 判定 | メモ |
|---|---|---|---|
| 1.1 時報で数え直す | `check_hour` L189-192 | Missing | 候補ループの thread を返す枝に 1 行（brief の仮実装） |
| 1.2 時報から `talk_interval_min` 秒は出さない | `calculate_next_talk_time`（L110-114） | Missing→1.1 で満たす | 乱数の下限が `talk_interval_min` なので、1.1 だけで満たす |
| 1.3 見送った分を時報の直後に出さない | `check_talk` L216-220 | Missing→1.1 で満たす | 溜まりは 1.1 の上書きで消える |
| 1.4 候補が無ければ変えない | L193 `return nil` | Exists | `co_exec` は条件漏れでも `nil` を返すので、「名前はあるが条件を満たさない」も同じ |
| 1.5 見送るたびに決め直さない | L218-220 | Exists | 変えない |
| 1.6 スリープ明けも例外にしない | 1.1 と同じ場所 | Missing→1.1 で満たす | |
| 1.7 出力の有無にかかわらず数え直す | `create_scene_thread` は実行前の thread を返す | Exists（仮定どおりなら）／Constraint | 「何も出さなかった回は変えない」を選ぶと、出力は `EVENT.drive` の後でしか分からない（§4.4） |
| 2.1 続きを捨てず、決め直した時刻に出す | `EVENT.drive` の続きの決まり（`init.lua` L268-279） | **Conflict** | 台詞を出した時報は「interrupt」として続きを捨てる（§4.1） |
| 2.2 止めている間は数え直さない | `dispatch` L252 | Exists | 門の間は `check_hour` が呼ばれない |
| 2.3 再開で数え直す | `restart_timers` | Exists | 変えない |
| 2.4 抑止が解けた最初の秒の時報から数え直す | `dispatch` L246 → `check_hour` | Missing→1.1 で満たす | |
| 2.5 スリープ明けの時報は 1 回・進んだ先 | `clock.lua` L127-130・`check_hour` | Exists | テスト L1058-1068 が守っている |
| 2.6 探し方・転記・飛び・なめらかな時計 | 同 | Exists | 変えない |
| 2.7 外から起こした `OnTalk` | `EVENT.no_entry` | Exists | 変えない |
| 3.1 受けていなければ次のトークの時刻を今にする | — | Missing | ディスパッチャに公開の関数（例: `talk_soon(act)`）と、`REG.OnSysResume` の既定 |
| 3.2 応答はトークを返さない（204） | 既定ハンドラが `nil` を返せば 204 | Missing（簡単） | |
| 3.3 Status・`hour_margin` の間は門が開くまで待つ | `dispatch` L246・`check_talk` L218 | Exists | 「今」にした `next_talk_time` は見送りの間も残る |
| 3.4 時報が先に出たらおしゃべりを出さない | — | **Missing／Conflict（推奨案の形）** | 実機の順番では時報が `OnSysResume` より先。「今にするだけ」では時報の 4 秒後に OnTalk が出る（§4.3） |
| 3.5 続きがあれば続きを出す | `check_talk` L227-230 | Exists | |
| 3.6 止めている間は出さない・持ち越さない | `dispatch` L252・`restart_timers` L270 | Exists（「今にする」だけなら） | 新しい印を足すなら `restart_timers` で消す |
| 3.7 `auto` なら出さない | `act.req.reference[0]` | Missing（簡単） | |
| 3.8 OnTalk のシーンが無いとき | `check_talk` L223 → `create_scene_thread` が `nil` | Exists | |
| 3.9 ふつうの OnTalk と同じシーン | `check_talk` | Exists | 専用の名前を設けないので、足すものは無い |
| 3.10 スリープを飛びとみなさない扱い | `clock.lua` | Exists | 変えない |
| — 長いスリープで溜まった OnTalk | `CLOCK.steady` はスリープの間も進む（スリープは飛びに数えないため `_jump_total` に入らない） | **Conflict（要件に無い）** | 復帰後の最初の秒で、`OnSysResume` より前に溜まった OnTalk が出る（§4.2） |
| 4.1 シーンがあればそれ | `SCENE.co_exec`（`boot.lua` と同じ形） | Missing（簡単） | 既定ハンドラの中でシーンを先に探す |
| 4.2 `REG.OnSysResume` の代入 | `REG.X = REG.X or …` | Exists（既定を底に置けば） | |
| 4.3 鎖で止めた回は行わない | `init.lua` L305-319 | Exists（既定を底に置けば） | |
| 4.4 名前はあっても条件漏れ | `co_exec` が `nil` | Exists（既定の中で `nil` を見れば） | |
| 4.5 受けていれば変えない | — | Exists（既定が呼ばれないので） | |
| 5.1〜5.4 マニュアル | `shiori-events.md` L119-135（「13 行」の表）・L1392-1406・L1441-1447・L1482-、`internals/shiori.md` L115・L431-433・L505-524・L563 | Missing | 既定ハンドラの表に `OnSysResume` を足すなら「13 行」を「14 行」に。主要イベントの節に `OnSysResume` の項を足すかは任意 |
| 5.5 references の作り直し | `book/tools/gen-skill-refs.mjs` | Exists（手順） | |
| 5.6 completed を書き換えない | — | Constraint | `tutorial-ssp-gaps` の design.md L641 は残す |
| 6.1〜6.6 テスト | `virtual_dispatcher_spec.lua`（L594 の候補が全部無いとき・L978-992 の実機の順番のスリープ明け・L994-1004 の `hour_margin`）、`event_clock_hook_test.lua`・`reg_handler_chain_test.lua`（EVENT.fire を通す形の手本） | Missing | 6.4・6.5 は `EVENT.fire` を通すテストにしないと、鎖・代入・シーンの判定を確かめられない |
| 6.7 今のテストを通す | `virtual_dispatcher_spec.lua` L987 | Constraint | 期待が変わるのはこの 1 か所（時報の 1 秒後の `check_talk` が `thread` → `nil`）。Rust の `virtual_event_dispatch_test.rs`・`virtual_clock_e2e_test.rs` は時報の後に OnTalk を続けて見ていない |

## 4. 要件とコードの食い違い（ディスカッションへ）

### 4.1 時報は、台詞を出せば待っている続きを捨てる

`internals/shiori.md` L523 と `grammar/call-jump.md`「続きが捨てられるとき」（L461-）のとおり、時報のシーンは `SCENE.co_exec` の印の無いコルーチンで、続きが待っていれば「interrupt」になる。台詞を出せば続きを捨て（`＊チェイントーク中断` があればその台詞が付く）、出さなければ残す。

要件 2.1・6.6 は「時報を発行したとき、その続きを捨てず、決め直した時刻の最初の機会に続きを出す」と書くが、台詞のある時報ではこれは成り立たない。Boundary の「`chain-talk-interrupt` の続きの扱いには触れない」とも食い違う。

- 直し方の候補: 要件 2.1 を「時報が台詞を出さず続きが残ったときは、決め直した時刻の最初の機会に続きを出す（台詞を出した時報が続きを捨てる今の決まりは変えない）」に改める。6.6 も同じく。

### 4.2 長いスリープの後は、`OnSysResume` より前に溜まった OnTalk が出る

`CLOCK.steady` は飛びを差し引いた時計だが、スリープ明けの差は飛びに数えない（`clock.lua` L127-130）。そのため、スリープの長さだけ `steady` が進み、スリープの前の `next_talk_time` は期限を過ぎている。正時をまたがなければ、復帰後の最初の OnSecondChange で `check_talk` が溜まった OnTalk を出す。SSP の `OnSysResume` は、その数秒後に届く（2026-10-18 実機）。

- 今の「スリープ明けはおしゃべりがすぐ出る」は、正時をまたがないスリープでは、この溜まりで今も起きている。時報の数え直し（要件 1）は、正時をまたいだ場合だけを変える。
- 要件 3.1 をそのまま足すと、最初の秒で溜まった OnTalk が出て `next_talk_time` が決め直された後に、`OnSysResume` が「今」に戻し、喋り終えた直後にもう 1 回 OnTalk が出る。
- Project Description の「溜まりに頼らず `OnSysResume` を合図に出す」と、この溜まりをどう扱うかを要件が決めていない。

候補（§5.2 の S1〜S3 と対応）:

- 溜まりは今のまま出し、`OnSysResume` は「復帰の後に自動トーク（時報かランダムトーク）がまだ出ていないとき」だけ今にする（短いスリープを拾う役になる）。
- 復帰の最初の秒では溜まりを出さず、`OnSysResume` を待つ。`OnSysResume` が来なかったとき（スリープを取りやめた・ほかのベースウェア）の逃げ道が要る（roadmap L461 の「`OnSysSuspend` が来て `OnSysResume` が来ない」）。

### 4.3 要件 3.4 には、「復帰の後に時報を出したか」を覚える状態が要る

実機の順番は「（復帰）→ `OnSysSuspend` → 最初の秒で時報（数え直し）→ 数秒後に `OnSysResume`」である。brief の推奨案（`OnSysResume` で `next_talk_time` を今にするだけ）では、時報が決め直した時刻を `OnSysResume` が「今」で上書きし、時報の数秒後に OnTalk が出る。要件 1.2 と 3.4 に反する。

満たす形の候補:

- ディスパッチャが「最後に自動トーク（時報・ランダムトーク）を出した時刻」（`steady`）を覚え、`OnSysResume` では「その時刻が復帰より前」のときだけ今にする。「復帰」の時刻をどこから取るかが要る（`OnSysSuspend` を見た時刻・時計がスリープの差を飲み込んだ秒など）。今の `CLOCK` には、それを外へ知らせる口が無い。
- 時間の窓で決める: 最後の自動トークから一定の秒数（例: `talk_interval_min`、または `REBASE_GRACE` と同じ 70 秒）以内なら今にしない。状態は 1 つで済むが、窓の幅の根拠が要る。
- §4.2 の「溜まりを出さず `OnSysResume` を待つ」形にすれば、時報が数え直した時刻と「待っている印」の関係だけで決まる（時報を出したら印を消す）。

### 4.4 OPEN QUESTION 1（何も出さない時報）を「変えない」にすると、置き場所が変わる

`check_hour` が返すのは実行前のコルーチンで、出力があったかは `EVENT.fire` → `EVENT.drive` の後でしか分からない。「何も出さなかった回は数え直さない」を選ぶと、数え直しを `check_hour` に置けず、`second_change.lua` か `EVENT.drive` の後に、時報のコルーチンかどうかを見分ける印が要る。要件の仮定（出力の有無にかかわらず数え直す）なら、`check_hour` の 1 行で済む。

## 5. 実装の選択肢

### 5.1 全体の置き方

#### 選択肢 A: 今のモジュールを延ばす（ディスパッチャ＋既定ハンドラのモジュール 1 つ）

- `virtual_dispatcher.lua`: `check_hour` の候補ループに数え直しを 1 行。スリープ明けの公開の関数（例: `M.talk_soon(act)`。`next_talk_time` を今にする。§4.3 の判定の状態も持つ）を足し、`_reset`・`_get_internal_state`・`restart_timers` に新しい状態を入れる。
- 新しいモジュール `event/sys_resume.lua`（`boot.lua` と同じ形）: `REG.OnSysResume = REG.OnSysResume or function(act)` で、`SCENE.co_exec(act, act.req.id)` が thread を返せばそれを返す（要件 4.1）。無ければ（条件漏れも）、Reference0 が `auto` でなければ `dispatcher.talk_soon(act)` を呼び、`nil`（204）を返す。`event/init.lua` L91 の後ろに `require` を 1 行。
- 利点: 既定ハンドラの作法・鎖の底の決まりにそのまま乗るので、要件 4.1〜4.5 に足すコードが無い。マニュアルの「既定ハンドラと上書き」の表に 1 行足すだけで、止め方（何も返さないハンドラを代入する・鎖で `true` を返す）も今の説明で足りる。
- 欠点: 新しいモジュールのぶん、`internals/shiori.md` の表（L115）・読み込みの順（L431-）と、テストの読み直しの並び（4 スイート）に 1 行ずつ足す。

#### 選択肢 A′: 既定ハンドラを既存のファイルに同居させる

- `REG.OnSysResume` を `second_change.lua` などに書く。新しいファイルは要らず、読み直しの並びも変えない。
- 欠点: ファイルの名前と中身がずれる（`second_change` に別のイベント）。今の作りは 1 ファイル＝1 つのイベントの族。

#### 選択肢 B: `EVENT.fire` か `CLOCK.observe` に差し込む

- `EVENT.fire` の底の呼び出しの後で「`req.id == "OnSysResume"` で、`EVENT.no_entry` が `nil` を返した」ときにディスパッチャを呼ぶ。あるいはディスパッチャに `observe` の口を設けて `EVENT.fire` の入口で毎回呼ぶ。
- 利点: `REG` に既定を置かないので、既定ハンドラの表を変えない。
- 欠点: 核の `EVENT.fire` にイベント名の特別扱いが入る。「ゴーストが受けている」の判定を、鎖・代入・シーンについて自分で書き直すことになる（要件 4.2・4.3 の判定が二重になる）。今の作法から外れる。

#### 選択肢 C: 段を分ける

- 第 1 段で時報の数え直し（要件 1・2・6.1〜6.3）だけを入れ、第 2 段でスリープ明けのおしゃべり（要件 3・4）を A の形で入れる。1 本の PR の中のコミットの分け方の話で、spec を分けるものではない。
- 利点: 不具合の直し（数え直し）が、スリープ明けの順番の議論（§4.2・§4.3）に引きずられない。

### 5.2 スリープ明けの順番の扱い（§4.2・§4.3）

| 案 | 中身 | 溜まりの扱い | 要件 3.4 | 足す状態 | 気になる点 |
|---|---|---|---|---|---|
| S1 今にするだけ（brief の推奨案） | `OnSysResume` で `next_talk_time = steady(now)` | 今のまま出る | 満たさない | なし | 時報の後・溜まった OnTalk の後に、もう 1 回出る |
| S2 復帰の後に自動トークが出ていなければ今にする | ディスパッチャが最後の自動トーク（時報・OnTalk）の `steady` を覚え、復帰の時刻と比べる | 今のまま出る（長いスリープはこれが復帰のおしゃべりになる） | 満たす | 最後の自動トークの時刻（＋復帰の時刻の取り方） | 「復帰の時刻」の取り方。`CLOCK` に口を足すか、`OnSysSuspend` を見た時刻を使うか、時間の窓にするか |
| S3 溜まりを出さず `OnSysResume` を待つ | 時計がスリープの差を飲み込んだ秒に、ディスパッチャが「復帰待ち」の印を立てて `check_talk` を見送る。`OnSysResume` で今にし、時報を出したら印を消す | 出さない（溜まりに頼らない） | 満たす | 復帰待ちの印 | `OnSysResume` が来ないときの逃げ道（一定時間で今の間隔に戻すなど）。`CLOCK` から「飲み込んだ」を知らせる口が要る。要件 3.10 の「時計を変えない」に触れる |

- S2 は、Project Description の「溜まりに頼らず」を「復帰の直後の 1 回を意図して出す（溜まりが出ていればそれを 1 回と数える）」と読むことになる。
- S3 は文言どおりだが、`clock.lua` に口を足し、逃げ道の決まりも要る。小さな spec の見込み（brief の Constraints）より大きくなる。

## 6. 工数とリスク

- **工数: S（1〜3 日）**。数え直しは 1 行と 3 つのテスト（brief の差分がそのまま使える）。スリープ明けは既定ハンドラ 1 つ（20 行ほど）とディスパッチャの関数 1 つ、テストは `EVENT.fire` を通す形で 6〜8 本、マニュアル 2 章と references の作り直し。S3 を選ぶと M 寄り。
- **リスク: Low（数え直し）／Medium（スリープ明け）**。数え直しは今の作りに沿い、影響が `check_hour` の 1 枝に閉じる。スリープ明けは、SSP が送る順番（`OnSysSuspend` が復帰の後に届く・`OnSysResume` が数秒遅れる）に頼るうえ、確かめは作者が PC をスリープさせる実機だけで、CI では順番の再現しかできない。

## 7. 設計フェーズへの申し送り

### 推奨の進め方（決めるのはディスカッション）

- 全体は選択肢 A（ディスパッチャの延長＋`event/sys_resume.lua` の既定ハンドラ）。コミットは選択肢 C のとおり、数え直しを先に分ける。
- スリープ明けの順番は S2 が今の作りに近く、要件 3.4 を満たす最小の形。S3 は文言に忠実だが範囲が広がる。
- 要件 2.1・6.6 は、台詞を出した時報が続きを捨てる今の決まりに合わせて書き直す。

### 要件ディスカッションからの申し送り（設計で決める）

- 要件 2.1・6.6 は、台詞を出した時報が続きを捨てる今の決まりに合わせて書き直した（§4.1 は解決済み）。
- 全体の置き方（選択肢 A／A′／B）とコミットの分け方（選択肢 C）。
- スリープ明けの「復帰の後」を何で測るか（`OnSysSuspend` を見た時刻・時計がスリープの差を飲み込んだ秒・時間の窓）。要件ディスカッションで決めた「復帰の後に喋るのは 1 回」をどう覚えるかの形。
- 要件 6.4・6.5 のテストは `EVENT.fire` を通す形にする（鎖・代入・シーンの判定を確かめるため）。
- 仮実装は作業ツリーに無いので、brief の差分の全文を正として当て直す。

### Research Needed

1. SSP（2.9.x）が `OnSysResume` を GET で送るか NOTIFY で送るか（応答でトークを返さないので動きは変わらないが、マニュアルの書き方に関わる）。
2. Modern Standby（S0 低電力アイドル）の PC で、SSP が `OnSysSuspend`・`OnSysResume` を送るか。送るなら Reference0 は何か（`auto` で来ることが多いと、既定の動きがほとんど出なくなる）。
3. 正時をまたがない短いスリープ（差が 90 秒以下で飛びに数えない）で、`OnSysSuspend`・`OnSysResume` の印がどう残るか（S2・S3 の「復帰の時刻」の取り方に効く）。
4. `GLOBAL.OnSysResume` に関数を置いたゴースト（シーン関数フォールバックの 4 段目で見つかる）を「受けている」に数えてよいか（要件 4.1 の「今のシーン関数フォールバックの探し方で見つかるもの」には入る）。

### ディスカッションの議題の候補

1. 要件 2.1・6.6 と、台詞を出した時報が続きを捨てる今の決まりとの食い違い（§4.1）。
2. 長いスリープの後に溜まった OnTalk を、復帰の最初の秒で今のまま出すか（§4.2）。
3. 要件 3.4 を満たす判定の形（S2 の「復帰の後に自動トークが出たか」・時間の窓・S3）（§4.3・§5.2）。
4. OPEN QUESTION 1: 何も出さない時報で数え直すか。「変えない」なら置き場所が `check_hour` の外になる（§4.4）。
5. OPEN QUESTION 3: Reference0 が `auto` のときに出さないか（Research 2 と合わせて）。
6. OPEN QUESTION 4: 鎖で全部見送った回・条件漏れの回を「受けていない」とみなすか。既定を鎖の底に置けば、今の決まりで自然にそうなる。
7. 既定ハンドラの置き場所（新しいファイル `event/sys_resume.lua` か、既存のファイルに同居か）と、既定ハンドラの表を「14 行」にすること。
8. 仮実装の差分（brief）を写し直すときの扱い: この作業ツリーには無いので、brief の差分を正として当て直す。
