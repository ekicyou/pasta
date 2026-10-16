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
