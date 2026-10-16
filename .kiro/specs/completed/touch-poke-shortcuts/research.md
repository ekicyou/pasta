# ギャップ分析: touch-poke-shortcuts

2026-10-16 に `/kiro-validate-gap` で作成した。対象は `requirements.md`（生成済み・未承認）である。emo2 の実装の控えは、同じフォルダの `research-emo2.md` にある。

## 要約

- **範囲**: Lua のイベント層に、つつき（`OnMouseDoubleClick`）と撫で（`OnMouseMove`）の既定のハンドラを足す。名前を段々に探す処理・撫での判定・アクター名の引き方を持つ。hello-pasta の 8・9 段目とマニュアルも書き直す。足場はほぼそろっている。時報の 4 段フォールバック（`virtual_dispatcher.check_hour`）、Status の抑止判定（`virtual_dispatcher.is_blocked`）、`SCENE.co_exec`、空の出力を 204 にする `RES.ok` がある。
- **一番大きいギャップ**は、要件 6.2（ショートカットの段を完全一致で探す。仮定）である。シーンの検索は前方一致で候補を集め、通し番号の付いた登録名をシャッフルして順に使う（`pasta_core` の `scene_table.rs`）。「同じ名前のシーンの中から抽選」を保ったまま完全一致にする道は、今は無い。Rust の検索に手を入れるか（brief の Approach は「Rust の検索は触らない」）、Lua でシーン表を走査するかを選ぶ必要がある。後者だと、同名シーンの「使い切るまで繰り返さない」抽選を作り直すことになる。
- **秒より細かい時計**は、新しく作らなくてよい。`req.date.ns`（秒の中のナノ秒）が既に Lua に渡っている。`X-Pasta-Time` ヘッダーで時刻を上書きできるので、撫での判定はテストで時刻を進めて確かめられる。
- **アクター名**: 同じ `spot` を共有するアクターは仕様で許されている（`pasta-toml.md` 439 行）。`spot` から名前は一意に決まらない。実行中のスポットの表（`STORE.actor_spots`）は `％` 行と表の初期化で変わり、空にもなる。そのスポットで最後に話したアクターは `STORE.appearance.owners[spot]` に記録されている。
- **推奨**: 新しいモジュール（Option B）を軸にする。完全一致を採るかどうかで、Option C（検索に完全一致の道を足す。`novel-talk-flow` と根が同じ）に広がる。規模は M（完全一致まで持つと M〜L）、リスクは Medium。

## 1. 今の状態

### 1.1 イベントの振り分け

- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` の `EVENT.fire`:
  - `CALLBACK.try_route` → `REG[req.id] or EVENT.no_entry` → 戻り値の型で応答を作る。
  - thread なら `EVENT.drive` で再開し、`RES.ok(value)` を返す。`value` が nil か空文字なら 204。
  - 既定のハンドラは `require` 1 行ずつで読み込む（`boot`・`choice_select`・`second_change`）。どれも `REG.X = function…` と無条件に代入する。
- `EVENT.no_entry` は `SCENE.co_exec(act, act.req.id)` を呼ぶ。`OnMouseDoubleClick`・`OnMouseMove` は今ここを通る。
- `SCENE.co_exec` は `act:find_scene` → `find_handler("scene", key)` の 5 段で探す。L1 は今のシーン表、L2 はローカルの辞書の前方一致、L3 は `act` のメソッド、L4 は `GLOBAL` の完全一致、L5 はグローバルの辞書の前方一致である。見つかれば、最後に `act:build()` を呼ぶコルーチンにして返す。
- シーンが終わると、`EVENT.drive` → `set_co_scene` が `STORE.co_scene` を空にする。続きを待っていたチェイントークは捨てられる（今の `＊OnMouseDoubleClick` も同じ）。

### 1.2 手本になる段々の探し方

- 時報: `virtual_dispatcher.check_hour` が候補の配列 `{"時報"..hh, "OnHour"..hh, "時報その他", "OnHourOther"}` を順に `create_scene_thread` で探す。テスト用に `_set_scene_executor` でシーンの実行を差し替えられる。
- 選択肢: `choice_select.lua` は明示の `＊OnChoiceSelectEx` を先に探し、無ければ選択肢の ID で `SCENE.search` する。

### 1.3 抑止

- `virtual_dispatcher.is_blocked(status)` は公開されている。`BLOCKED_STATUSES` の 9 語（`talking`・`choosing`・`online`・`opening`・`passive`・`induction`・`timecritical`・`nouserbreak`・`minimizing`）を、部分文字列で探す。
- `req.status` は Status ヘッダーの生の文字列である（`crates/pasta_shiori/src/lua_request.rs` 144 行）。`balloon(0=0)` は 9 語のどれも含まないので、抑止されない。要件 5.4 に合う。

### 1.4 時刻

- Lua の実行環境に、ミリ秒の時計は無い。`os.clock` は CPU 時間で使えない。
- `req.date` には `unix`（秒）と `ns`（秒の中のナノ秒）がある（`lua_request.rs` 11〜31 行）。`unix + ns / 1e9` で、秒より細かいリクエストの時刻になる。
- `X-Pasta-Time` ヘッダー（RFC3339）で、リクエストの時刻を上書きできる（`lua_request.rs` 57〜71 行）。テストで時刻を進められる。

### 1.5 アクターとスポット

- `store.lua` 108〜121 行: `STORE.actors = CONFIG.actor`。`actor.spot` が数値のときだけ `STORE.actor_spots[name]` へ写す。
- 設定の読み込み（`loader/config/mod.rs`）は、アクターごとの検査をしない。`spot` が無くても、重なっても、エラーにならない。
- `STORE.actor_spots` は出力の組み立てが書き換える。`sakura_builder.lua` の 293〜299 行で、`％` 行のトークンが代入し、`clear` で全部を消す。
- `shiori/appearance.lua` 556〜561 行: `state.owners[spot] = name`（そのスポットで最後に出たアクター）と `state.last_spots[name]` を記録している。共有のスポットで、最後に話したアクターを引ける。

### 1.6 シーンの検索

- `crates/pasta_lua/src/search/context.rs` の `search_scene`:
  - 名前を `SceneRegistry::sanitize_name` で整えてから、`resolve_scene_id_unified` で探す。
  - 前方一致で候補を集め、属性で絞り、キャッシュを使って「使い切るまで繰り返さない」順に選ぶ。
  - グローバルの検索だけ、別名（`[scene.alias]`）の完全一致を先に置き換える。
- 検索のキーは、登録名から通し番号（`_N`）を除いた名前の部分である（`scene_table.rs` の `fn_name_to_search_key`）。
- 完全一致で引く道は、デバッグ用のシーンキック（`kick.lua`）にしか無い。これは登録名（通し番号つき）で 1 つを引くので、同じ名前のシーンの抽選にはならない。
- Lua の側では、`SCENE.get_all_scenes()`（`STORE.scenes`）で、登録名をキーにしたシーン表を走査できる。

### 1.7 見本のゴーストとテスト

- `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/`:
  - `dic/08-touch.pasta`: `＊OnMouseDoubleClick` が 3 つ。どれも `＞transfer_req_to_var` と `＄ｒ４` を使う。
  - `dic/09-choice.pasta`: 選択肢を出す `＊OnMouseDoubleClick` が 1 つ。8 段目と一緒に抽選される。
  - `pasta.toml`: `[actor."女の子"] spot = 0`、`[actor."男の子"] spot = 1`。
  - 当たり判定: `shell/master/surfaces.txt` の `Face`・`Head`・`Body`（`tests/shell_assets_test.rs` が検査する）。
- `src/scripts.rs` 145〜160 行 `test_touch_pasta_contains_events`: `＊OnMouseDoubleClick` が 3 つ以上あること、本文に `＞transfer_req_to_var` と `＄ｒ４` があることを検査する。**8 段目を書き直すと落ちる。**
- `tests/tutorial_stages_test.rs`:
  - `STAGES.md` の表を見出しで見つけ、段ごとに OnBoot と表のイベントを 1 回ずつ送る。
  - `MUST_SPEAK` に入っているのは `OnChoiceSelectEx` だけ。
  - `stage_table_matches_dic_files` は、表に載ったファイルと `dic/*.pasta` の集合が等しいことを確かめる。
  - `scene_names_do_not_prefix_collide`（409 行）は、dic の中のグローバルシーンの名前どうしが前方一致で重ならないこと、`Kick`・`Gate` で始まらないこと、`GLOBAL` のキーと同じでないことを確かめる。新しい名前は、互いに頭が重ならないように選ぶ必要がある（`つっつき反応Head` と `つっつき反応女の子Head` は重ならない）。
  - イベント 1 件ずつの表なので、撫で（時間をおいて何件も送る）は今の形では確かめられない。
- `STAGES.md` の検証イベント表:
  - 37 行 `| 8 | OnMouseDoubleClick | 3=0, 4=Head |`
  - 38 行 `| 9 | OnChoiceSelectEx | 0=おやつの話をする, 1=おやつの話, 2=OnMouseDoubleClick |`

### 1.8 マニュアルとスキル references

- 入門 `book/src/getting-started/08-touch.md`・`09-choice.md`: `book/tools/tutorial-check.mjs` が照合する。辞書全体が 1 つの ```pasta ブロックとして逐語で載っていること、章の中の ```pasta ブロックはどれも辞書の全体か連続する行であること、の 2 つである。09 の本文（114・134 行）は「8 段目と同じ名前で 4 つから抽選」と説明している。
- `book/src/lua/shiori-events.md`: 203 行の表、346〜374 行の `### OnMouseDoubleClick`（Reference の表・作例・`REG.OnMouseDoubleClick` の Lua 例）。**`OnMouseMove` はマニュアルのどこにも書かれていない。**
- `OnMouseDoubleClick` を例にしているほかの章: `grammar/variables.md`（370・383・387・392 行）、`lua/patterns.md`（97 行）。
- `book/tools/gen-skill-refs.mjs` は、`shiori-events.md` → `.claude/skills/pasta-lua-coding/references/`、`variables.md`・`pasta-toml.md` → `.claude/skills/pasta-ghost-authoring/references/` を生成する。`getting-started/*` は対象外。
- 手書きのスキル（生成でない）も `OnMouseDoubleClick` に触れている。`.claude/skills/pasta-ghost-authoring/SKILL.md`（321・393・417 行）、同 `references/authoring-patterns.md`（125・177・253 行）、`.claude/skills/pasta-lua-coding/SKILL.md`（174 行）。

## 2. 要件と資産の対応

| 要件 | 使える資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 つつきの段々 | `check_hour` の候補配列の形、`SCENE.co_exec`、`REG` | **Missing**: `REG.OnMouseDoubleClick`、候補の名前の組み立て。**Constraint**: `co_exec` は L3（`act` のメソッド）・L4（`GLOBAL`）にも当たる。`OnPokeHead` という名前の `GLOBAL` 関数も拾う（今のイベントと同じ） |
| 1.8 右ボタン | Reference5（左 0・右 1） | 区別しないなら、追加の作業は無い |
| 2 撫での判定 | `research-emo2.md` の `touch_detect.lua`、`req.date.unix`/`ns`、`X-Pasta-Time` | **Missing**: 判定の本体（状態・数え直し・1 回だけ）。時計を外から渡せる形にする |
| 2.6 1 秒未満のずれ | `req.date.ns` | Rust の追加は要らない。areka が同じ時刻を渡すかは **Unknown** |
| 3 撫での段々 | 要件 1 と同じ | **Missing**: `REG.OnMouseMove`。決めなかった移動では、7 段目だけを探す分岐 |
| 4 アクター名 | `CONFIG.actor`（`name`・`spot`）、`STORE.appearance.owners` | **Missing**: 番号 → 名前の引き方。**Constraint**: `spot` の重なりは正当。`STORE.actor_spots` は実行中に変わり、空にもなる |
| 5 抑止 | `virtual_dispatcher.is_blocked` | そのまま使える（9 語）。絞るなら別の一覧が要る |
| 6.1 部位の名前 | `sanitize_name` | **Unknown**: 部位の名前に `・` などがあるとき、整えた後の名前で照合される。辞書の側も同じ規則で登録されるので、ずれないはず（Research Needed） |
| 6.2 完全一致 | `kick.lua`（登録名で引く）、`SCENE.get_all_scenes` | **Missing**（最大のギャップ）。前方一致＋シャッフルのキャッシュしか無い |
| 7.1 空のシーンで 204 | `RES.ok(nil)` → 204、`act:build()` の早期 return | 既存で満たす。台詞の無いシーンを DSL で書けるかは **Unknown** |
| 7.2 `＞transfer_req_to_var` | 既存 | ギャップなし |
| 8.1 今までのゴースト | 7 段目＝今の `no_entry` と同じ探し方 | 7 段目を `co_exec(act, req.id)` にすれば変わらない |
| 8.2 自前のハンドラの優先 | `REG` の上書き | **Unknown**: 既定のハンドラは `require` のときに無条件に代入する。ゴーストの Lua がそれより先に `REG.OnMouseMove` を代入すると、上書きされる。既存の `OnBoot` などと同じ条件なので、どの時点でゴーストの Lua が走るかを確かめる（Research Needed） |
| 8.3・8.4 軽さ | — | 毎回のログは trace 以下にする。決めなかった移動では 7 段目だけを探す |
| 9 hello-pasta | 辞書・`STAGES.md`・2 つのテスト | **Constraint**: `scripts.rs` の検査を新しい書き方へ書き直す。撫での確かめ方（時刻を進めた複数のイベント）を足す。9 段目の名前を変えると、検証イベント表の Reference2 も変わる |
| 10 マニュアル | `tutorial-check.mjs`・`gen-skill-refs.mjs` | **Missing**: `OnMouseMove` の節。**Constraint**: 手書きの `SKILL.md` の更新は、分類器がセルフ編集として拒むことがある（作業の時点でユーザーの許可が要る） |

## 3. 実装の選択肢

### Option A: 既存のファイルを広げる

- `virtual_dispatcher.lua` に、段々の探し方と撫での判定を足す。`init.lua` にハンドラを直に書く。
- ✅ ファイルが増えない。`is_blocked` と候補の探し方がすぐ隣にある。
- ❌ `virtual_dispatcher` は「`OnSecondChange` から仮想イベントを出す」もので、責務が混ざる。`init.lua` は `runtime-error-balloon` も触るので、衝突が増える。roadmap の「`init.lua` には `require` を 1 行足すだけ」に反する。

### Option B: 新しいモジュールを足す（推奨の軸）

- `shiori/event/` に新しいファイルを足す。中身は 3 つ。
  - つつき・撫での既定のハンドラ（`REG.OnMouseDoubleClick`・`REG.OnMouseMove`）と、候補の名前を組み立てて上から探す処理。時報と同じく、シーンの実行を差し替えられる形にする。
  - 撫での判定。時刻を引数で受け取る、純粋な状態機械にする。
  - スコープ番号 → アクター名。
- `init.lua` には `require` を 1 行足す。抑止は `virtual_dispatcher.is_blocked` を呼ぶ。
- ✅ roadmap の境界（`shiori/event/` の新しいファイル＋`require` 1 行）に合う。判定を単体で試せる。
- ❌ 完全一致（要件 6.2）を採るなら、これだけでは足りない（Option C）。
- 判定と探し方を 1 つのファイルにまとめるか分けるかは、設計で決める。

### Option C: B ＋ 検索に完全一致の道を足す

- 6.2 を採る場合に限る。
- **C-1（Rust）**: `search_scene` に完全一致の指定を足す（候補を、キーが探す名前と等しいものに絞る）。同じ名前の中の「使い切るまで繰り返さない」抽選は、そのまま使える。
  - ✅ 抽選の決まりが今と同じ。`novel-talk-flow`（ノベル向けの完全一致）と同じ部品を使える。
  - ❌ brief の Approach（「Rust の検索は触らない」）と食い違う。`pasta_core`・`pasta_lua/src/search`・`@pasta_search` の束縛まで広がる。3〜4 タスク増える。`novel-talk-flow` と、どちらが先に入れるかを合わせる必要がある。
- **C-2（Lua）**: `SCENE.get_all_scenes()` を走査し、登録名から通し番号を除いた名前が候補と等しいシーンを集め、Lua で抽選する。
  - ✅ Rust を触らない。
  - ❌ 抽選が今の同名シーンの決まり（使い切るまで繰り返さない）と変わり、要件 1.2・3.2 の「今までと同じ決まり」を満たさない。同じものを作り直すなら、その分の作業がかかる。走査は「撫でた」と決めたとき・つつきのときだけなので、重さは問題にならない。
- **C-3**: `SCENE.search` の結果の登録名を見て、前方一致の外れを捨てて引き直す。外れが続くと何度も引くうえ、キャッシュを外れで消費するので採りにくい。

## 4. 規模とリスク

- **規模**: M（3〜7 日）。新しいモジュール、テスト、hello-pasta の 8・9 段目、マニュアルの 4〜5 ページ。完全一致を C-1 で持つと M〜L（3〜4 タスク増え、brief の見当どおり 13〜17 タスク）。
- **リスク**: Medium。
  - 振り分けは時報の形をなぞるだけで、低い。
  - 撫での判定は、実機の届き方（頻度・空の部位・トーク中の Status の出方）に左右される。`X-Pasta-Time` でテストはできるが、実機の確認は `getting-started-screenshots` に回る。
  - 完全一致は検索の中心に触る。
  - areka での届き方は確かめていない。

## 5. 設計フェーズへの申し送り

### 推奨

- Option B を軸にする。要件ディスカッションで 6.2（完全一致）が確定したら、C-1 と C-2 を比べる。抽選の決まりを保つなら C-1 が筋で、`novel-talk-flow` との順番を決める。
- 7 段目は、今の `EVENT.no_entry` と同じ `co_exec(act, req.id)` で探す（要件 8.1 を構造で守る）。
- 撫での判定は、時刻を引数に取る形にする（`req.date.unix + req.date.ns / 1e9`）。テストは `X-Pasta-Time` か直接の引数で時刻を進める。
- 抑止は `virtual_dispatcher.is_blocked` を呼ぶ（一覧を 2 つ持たない）。

### 要件ディスカッションの結果（2026-10-16）

- 完全一致（要件 6.2）は採る。Option C を前提にし、設計で C-1 と C-2 を比べる。抽選の決まりを今と同じに保つこと（要件 1.2・3.2）が条件。
- アクターだけの段を足したので、段は 7 段でなく 9 段になる（本書の「7 段目」は、要件の 9 段目＝`OnMouseDoubleClick`・`OnMouseMove` に読み替える）。
- 自前のハンドラの優先（下の Research Needed 2）は、要件 8.2・8.3・8.6 で確定した。ゴーストの登録が既定の受け方より先でも後でも、ゴーストのハンドラが勝ち、既定の受け方は動かない。`register.lua` のドキュメントが `REG.OnMouseDoubleClick = function(act)` を作例に挙げているので、そう書いたゴーストを壊さないことが要る。設計では、読み込み順を確かめたうえで、登録のしかた（`REG.X = REG.X or …` など）を決める。

### Research Needed

1. **完全一致の方式**（6.2）: C-1 で、`resolve_scene_id_unified` のキャッシュのキーと絞り込みをどう足すか。`novel-talk-flow` の設計と部品を共有できるか。
2. **ゴーストの Lua が走る時点**（8.2）: 既定のハンドラの登録（`require` のときに無条件に代入）より後にゴーストの `REG` の代入が走るか。先に走るなら、`OnBoot` などの既存の既定ハンドラも同じ問題を持つ。既存の決まりに合わせるか、`REG.X = REG.X or …` にするかを決める。
3. **Status の出方**（5.2）: 撫での反応を返した直後の `OnMouseMove` に、SSP が `talking` を付けるか。「2 回目は反応の再生が終わった後」という要件 5.2 の見込みが成り立つか。SSP の実ログ（`crates/pasta_shiori/doc/shiori-sample.log`）で確かめる。
4. **空の部位の頻度**: 当たり判定の外で、SSP が `OnMouseMove` を Reference4 を空にして送り続けるか。要件 2.5 の数え直しが、そのイベントで起きるか。
5. **最後に話したアクター**（4.3）: `appearance.owners` が更新されるのは、出力を組み立てたとき（再生の前）である。この値を「そのスポットに今いるアクター」として使ってよいか。
6. **台詞の無いシーン**（7.1）: 本文の無い `＊つっつき反応閉じる` を、DSL がシーンとして登録するか。登録しないなら、マニュアルで案内する書き方（何も出さない行など）が要る。
7. **部位の名前の整え方**（6.1）: `sanitize_name` が部位の名前（日本語・記号）をどう変えるか。辞書の側と同じ規則になるかを確かめる。
8. **見本のゴーストのテスト**（9.6）: `tutorial_stages_test.rs` の表に撫で（複数のイベントと時刻）をどう載せるか。表を広げるか、別のテストにするか。
9. **手書きのスキル**（10）: `.claude/skills/pasta-ghost-authoring/SKILL.md` などの手書きの記述を、生成の対象外として直すか。編集にはユーザーの許可が要る。

### 隣の spec との重なり

- `runtime-error-balloon`: `shiori/event/init.lua` を触る。この spec は `require` 1 行だけなので、後から入る側が合わせる。
- `novel-talk-flow`: 完全一致の場面選び。C-1 を採るなら部品を共有するので、順番と境界を決める。
- `failure-output-unification`・`boot-surface-without-dic`: 撫での設定を `pasta.toml` に置く場合（今の仮定は「置かない」）だけ、設定の読み込みで重なる。
- `pasta-check-dic-validate`: 接頭辞だけの `＊さわり反応`・`＊つっつき反応` の検査を、後から足す。
- `getting-started-screenshots`: 8・9 段目の絵と実機の確認は、この spec の後に行う。
- `shell-element-click`: 「触られたときの反応の受け方を変えない」としている。この spec が受け方を足すので、その brief の Current State を読み替える。

---

# 設計フェーズの調査と決定（2026-10-16・`/kiro-spec-design -y`）

## Summary

- **Feature**: `touch-poke-shortcuts`
- **Discovery Scope**: Extension（既存のイベント層と検索に足す）。light discovery。外部ライブラリは足さないので Web の調査はしていない。
- **Key Findings**:
  - 起動の順は `main` → `pasta.shiori.entry`（ここで既定ハンドラが登録される）→ `pasta.scene_dic`（辞書の Lua ブロック）。`scripts/main.lua` で先に `REG.OnMouseDoubleClick` を登録したゴーストは、既定ハンドラを無条件に代入すると上書きされる。マニュアルは「既定ハンドラを持たないイベントは、どちらに書いてもよい」と書いているので、`REG.X = REG.X or 既定` で登録する。
  - シーンの検索の RadixMap のキーは「登録名から通し番号を外した名前」なので、同じ名前のシーンは 1 つのキーの下に並ぶ。完全一致は「キーが等しいものだけを候補にする」だけで作れ、抽選は既存のキャッシュ（`select_from_cache`）をそのまま使える。C-1 の追加は小さい。
  - SSP の実ログで、トーク中の `OnMouseMove` に `talking,balloon(0=0)` が付くこと、当たり判定の外では Reference4 が空で届くことを確かめた（Research Needed 3・4 は解決）。

## Research Log

### ゴーストの Lua が走る時点（Research Needed 2・要件 8.2）

- **Sources**: `crates/pasta_lua/src/runtime/factory.rs` 214〜225 行、`pasta_scripts/pasta/shiori/event/init.lua` 70〜73 行、`book/src/lua/shiori-events.md`「既定ハンドラと上書き」。
- **Findings**:
  - `require("main")` → `require("pasta.shiori.entry")`（`entry.lua` が `pasta.shiori.event` を require し、`init.lua` が `boot`・`choice_select`・`second_change` を require して `REG.X = function…` と代入）→ `require("pasta.scene_dic")`（辞書のモジュールが読まれ、モジュールの直下に置かれた Lua ブロックがこの時点で評価される）。
  - マニュアルは、既定ハンドラのある 3 つについて「`main.lua` で上書きするときは、先に `require("pasta.shiori.event")` を呼ぶ」と回避を書き、「既定ハンドラを持たないイベントのハンドラは、どちらに書いてもよい」と書いている。`register.lua` のドキュメントも `REG.OnMouseDoubleClick = function(act)` を作例に挙げている。
- **Implications**: `OnMouseDoubleClick`・`OnMouseMove` の既定は `REG.X = REG.X or 既定` で登録する。先に登録したゴースト（`main.lua`）では代入されず、後に登録したゴースト（辞書の Lua ブロック）では置き換わる。どちらでも `REG[req.id]` はゴーストのハンドラになり、既定は一切動かない（要件 8.3）。

### シーンの検索の中身（Research Needed 1・要件 6.2）

- **Sources**: `crates/pasta_core/src/registry/scene_table.rs`（`from_scene_registry`・`fn_name_to_search_key`・`collect_scene_candidates`・`select_from_cache`・`SceneCacheKey`）、`crates/pasta_lua/src/search/context.rs`（`search_scene`・束縛）、`pasta_scripts/pasta/scene.lua`、`pasta_scripts/pasta/act.lua`（`find_act_handler` の 5 段）。
- **Findings**:
  - RadixMap のキーは、グローバルなら「登録名から通し番号を外した名前」、ローカルなら `:親の登録名:名前`。同じ名前のシーンは 1 つのキーの `Vec<SceneId>` に入る。
  - 抽選は `(module_name, search_key, filters)` をキーにしたキャッシュで、候補を並べ替えて使い切るまで順に使う。
  - 同じ文字列で前方一致と完全一致を探すと候補が違うので、キャッシュのキーに「完全一致か」を足さないと混ざる。
  - `search_scene` はグローバルのときだけ別名（`[scene.alias]`）の完全一致を先に置き換える。
  - イベントのハンドラの `act` は今のシーンを持たないので、`co_exec` は L3（`act` のメソッド）・L4（`GLOBAL`）・L5（グローバルの前方一致）を順に見る。
- **Implications**: C-1 を採る。`SceneTable::resolve_scene_id_exact`（候補集めだけ違い、絞り込みとキャッシュは共通。キャッシュのキーに `exact` を足す）、`SearchContext::search_scene_exact` と束縛、`SCENE.co_exec_exact`（グローバルだけ・シーンだけ）。前方一致の関数は変えない。`novel-talk-flow` は同じ部品（ローカルの範囲も使える）を使える。
- `@pasta_search` は公開モジュールなので、`book/src/lua/modules/pasta-search.md`（生成でスキル references に入る）に新しいメソッドを書く。`SCENE.co_exec_exact` は `book/src/internals/internal-modules.md` に書く。

### Status の出方と空の部位（Research Needed 3・4）

- **Sources**: `crates/pasta_shiori/doc/shiori-sample.log`。
- **Findings**: `OnMouseMove` 108 件の Status は `balloon(0=0)` 36・`talking,balloon(0=0)` 30・`choosing,balloon(0=2,1=0)` 16・`talking,choosing,…` 24。Reference4 は空が 41 件（ほかは `Face`・`Bust`・`Image` など）。Status ヘッダーは ID の前に来る。
- **Implications**: トーク中の `OnMouseMove` に `talking` が付くので、撫での反応の再生中は抑止され、2 回目は再生の後になる（要件 5.2 の見込みが成り立つ）。当たり判定の外では空の Reference4 が届くので、要件 2.5 の数え直しはそのイベントで起きる。

### 本文の無いシーン（Research Needed 6・要件 7.1）

- **Sources**: `crates/pasta_dsl/src/parser/grammar.pest` 259〜263 行、`crates/pasta_lua/src/code_gen/scope_gen.rs`。
- **Findings**: `global_scene_scope` は `local_start_scene_scope` を必ず持ち、これは 0 行を許す。名前の無い開始シーンは `__start__` として生成される。本文が無くても `__start__` が生成される見込み。`act:build()` はトークンが 0 件なら nil を返し、`RES.ok(nil)` は 204。
- **Implications**: `＊つっつき反応閉じる` を本文なしで書けば、その部位では反応しない。統合テストで確かめる（設計のテスト計画に入れた）。

### 部位の名前の整え方（Research Needed 7・要件 6.1）

- **Sources**: `SceneRegistry::sanitize_name`（`c.is_alphanumeric() || c == '_'` 以外を `_` にする）、`split_registered_name`。
- **Findings**: 日本語の文字は `is_alphanumeric` に入るので変わらない。`・`・空白・`-` などの記号は `_` になる。辞書の側の登録も検索の側も同じ関数で整えるので、照合はずれない。`胸・右` と `胸_右` は同じ名前とみなされる（頭が同じ別の名前を拾うことは無い）。
- **Implications**: 部位の名前は加工せずに候補へ入れ、整えは検索に任せる。特別な処理は足さない。

### 最後に話したアクター（Research Needed 5・要件 4.3）

- **Sources**: `pasta_scripts/pasta/shiori/appearance.lua` 551〜565 行、`sakura_builder.lua` 69〜81 行、`store.lua` 108〜121 行、`crates/pasta_lua/src/runtime/module_registry.rs` 93 行（`actor` の各表に `name` を入れる）。
- **Findings**: `owners[spot]` は出力を組み立てたとき（`\p[spot]` を出すとき）に、実行中のスポットで更新される。`％` 行でスポットを入れ替えると、設定の `spot` と違うスポットのアクターが入ることがある。
- **Implications**: 共有のスポットでは `owners[番号]` を見るが、それが「設定の `spot` がその番号のアクター」の中にいるときだけ採る。組み立ての時点（再生の前）で更新されることは、要件 4.3 の「最後に話した」と読んで差し支えない。

### 時刻

- **Sources**: `crates/pasta_shiori/src/lua_request.rs` 11〜72 行。
- **Findings**: `req.date` は `OffsetDateTime::now_local()` から作り、`unix` と `ns` を持つ。`X-Pasta-Time`（RFC3339・小数秒可）で上書きできる。LuaJIT の数は倍精度で、`unix + ns / 1e9` はマイクロ秒程度の分解能を持つ。
- **Implications**: 判定の時刻は `unix + ns / 1e9`。時計が戻ったときは数え直す。`req.date` の無いリクエスト（Lua で組み立てたテスト用のもの。`tutorial_stages_test.rs` の `shiori_request` もそう）では判定しない。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| C-1 Rust に完全一致 | `resolve_scene_id_exact`＋束縛＋`SCENE.co_exec_exact` | 抽選の決まりが今と同じ（要件 1.2・3.2）。`novel-talk-flow` と部品を共有できる | `pasta_core`・`pasta_lua/src/search`・マニュアル（`@pasta_search`）まで触る | **採用** |
| C-2 Lua で走査 | `STORE.scenes` を走査して同名を集め Lua で抽選 | Rust を触らない | 抽選が今と変わる（要件 1.2・3.2 を満たさない）。作り直すなら同じものを Lua に持つことになる | 不採用 |
| C-3 引き直し | `SCENE.search` の結果を見て外れを捨てて引き直す | 小さい | 外れでキャッシュを消費する。何度も引く | 不採用 |
| 登録 A（or） | `REG.X = REG.X or 既定` | 1 行。先でも後でもゴーストが勝つ | 既存の 3 つと登録のしかたが分かれる | **採用** |
| 登録 B（既定表） | `EVENT.fire` を `REG[id] or DEFAULTS[id] or no_entry` に | 順に一切依存しない | `init.lua` の振り分けを変える（`runtime-error-balloon` と衝突・境界の「require 1 行」を越える） | 不採用 |

## Design Decisions

### Decision: 1〜8 段は完全一致・グローバルシーンだけ

- **Context**: 要件 6.2（その名前と完全に同じ名前のシーンだけを候補にする）、要件 1.2・3.2（抽選は今と同じ）。
- **Alternatives Considered**: C-1・C-2・C-3（上の表）。1〜8 段で `co_exec` と同じく `GLOBAL` の関数や `act` のメソッドも見る案。
- **Selected Approach**: C-1。`SCENE.co_exec_exact(name)` はグローバルシーンだけを見る。9 段目だけ今の `co_exec`。
- **Rationale**: 抽選の決まりを保てるのは C-1 だけ。`GLOBAL` の関数は「シーン」ではなく、要件 6.2 の文面に合わせた。
- **Trade-offs**: Rust とマニュアル（`@pasta_search`・内部モジュール）の変更が増える（3〜4 タスク）。
- **Follow-up**: 前方一致と完全一致のキャッシュが混ざらないことを `pasta_core` のテストで固定する。

### Decision: 撫での判定は時刻を引数に取る状態機械（emo2 の 3 状態を引き継ぐ）

- **Context**: 要件 2・5。emo2 のはまりどころ 2〜5。
- **Selected Approach**: 状態 1 つ（所・始め・前回・発火済み）。抑止・空の部位・時刻なしで Off。所の変化・2 秒より長い途切れ・時計の戻りで数え直し。2 秒以上で 1 回だけ true。
- **Rationale**: 時刻を引数にすると、テストで時刻を自由に進められる。抑止を Off に戻すと、emo-gs の「抑止の間は撫でていない」と同じになり、2 回目が反応の再生の後に出る。
- **Trade-offs**: emo-gs は抑止の最後のイベントから数え始めていた。こちらは解けた後の最初のイベントから数える。差はイベントの間隔 1 つ分。

### Decision: 候補の名前・アクター名・判定・ハンドラを 1 ファイルにまとめる

- **Context**: Option B の「1 つにまとめるか分けるか」。
- **Selected Approach**: `event/mouse.lua` 1 ファイル。テスト用に `M.candidates`・`M.resolve_actor`・`M.check_stroke`・`M._reset`・`M._get_state` を出す。
- **Rationale**: どれも 2 つのハンドラだけが使い、別々に差し替える理由が無い（Simplification）。時報の `virtual_dispatcher` も判定と探し方を 1 ファイルに持つ。

### Decision: hello-pasta の 8・9 段目の名前の割り当て

- **Selected Approach**: `＊つっつき反応女の子Head`・`＊つっつき反応男の子その他`・`＊つっつき反応Face`・`＊つっつき反応その他`・`＊さわり反応女の子Head`・`＊さわり反応その他`。9 段目の選択肢は `＊つっつき反応その他`。
- **Rationale**: 4 種の段が 1 つずつあり、どの段も実際の操作で出会える（男の子の顔は 3 段目で止まるので、5 段目の `Face` は女の子の顔で出会う。この「上の段が先」の例をそのまま入門で説明できる）。名前どうしは前方一致で重ならない。

## Synthesis

- **Generalization**: 「完全一致で、同じ名前の中から今と同じ決まりで選ぶ」は、`novel-talk-flow` の完全一致の場面選びと同じ問題である。インターフェースは範囲（グローバル・ローカル）を取れる形にし（`resolve_scene_id_exact(module_name, …)`・`search_scene_exact(name, global_scene_name?)`）、実装は既存の範囲の決まりを使うだけにした。切り替えの仕組みは作らない。
- **Build vs. Adopt**: 抽選・属性の絞り込み・名前の整え・別名の置き換えは既存のものを使う。抑止は `virtual_dispatcher.is_blocked` を使い、語の一覧を 2 つ持たない。新しく作るのは、候補の集め方（キーの等しさ）と撫での判定だけ。
- **Simplification**: 戦略オブジェクト・設定・シーン実行の差し替え口（時報の `_set_scene_executor` に当たるもの）は作らない。ハンドラの探し方は実ローダーの統合テストで確かめ、純粋な部分（候補・判定・アクター名）だけを単体テストする。

## Risks & Mitigations

- 前方一致と完全一致のキャッシュが混ざる — キャッシュのキーに `exact` を足し、`pasta_core` のテストで固定する。
- 実機の届き方（頻度・空の部位・Status）が想定と違う — SSP の実ログで確かめた範囲で設計した。実機の手触りは `getting-started-screenshots` で確かめる。areka は確かめていない。
- 本文の無いシーンが登録されない — 文法上は登録される見込み。統合テストで確かめ、違えばマニュアルの書き方を変える。
- `context.rs`（752 行）が 600 行の目安を超えたまま大きくなる — 足すのは 40 行程度。目安のために責務をまたぐ分割はしない（steering の方針）。
- 手書きのスキルが古い書き方のまま残る — 設計ディスカッションで扱う（設計書の未決の論点 4）。

## 参照したスキル・規約

- `kiro-spec-design` の規則（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）と、`.kiro/settings/templates/specs/design.md`・`research.md`。
- steering の `structure.md`（レイヤー分離・テストの置き場所・ファイルの大きさの目安）と `tech.md`（LuaJIT 2.1・依存の一覧）。
- `pasta-lua-coding` スキルの §6（REG・既定ハンドラ）と §7（Lua のテストは `*_test.lua` を作り `init.lua` の `specs` に登録する）。

## 設計ディスカッションの結果（2026-10-16）

- 議題 2〜7 を推奨案で確定した（design.md「設計ディスカッションの決定」）。根拠の要点:
  - 既存の 3 つの既定ハンドラを `or` 登録にそろえるのは、要件 8.2 と根が同じで、マニュアルの決まりを 1 本にするため（「起票に安易に逃がさない」）。挙動が変わるのは、`main.lua` で `require` せずに登録していて、今は上書きされて動いていないゴーストだけ。
  - 要件 2.6 は、イベント駆動の判定では「しきい値を過ぎた後の最初の移動」より早く決められないため、文言を実際の意味に合わせた。
  - 本文の無いシーンが登録されない場合も、回避の書き方をマニュアルに載せず、この spec で登録されるように直す（「マニュアルに回避レシピを載せない」）。
- 議題 1（完全一致の作り方）は個別に議論し、開発者が C-1（Rust に足す）を選んだ。抽選の決まりを持つのは Rust のキャッシュだけなので、そこに候補の集め方を 1 つ足すのが最小で、`novel-talk-flow` も同じ部品を使える。
