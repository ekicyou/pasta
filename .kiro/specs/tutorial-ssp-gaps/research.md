# ギャップ分析: tutorial-ssp-gaps

- 実施日: 2026-10-17（main `838fbde8` の上のワークツリー）
- 対象: `requirements.md`（要件 1〜8。前提 A1〜A7 つきの下書き）
- 方法: コードとマニュアルの読み取り、伺か文書（UKADOC・SSP ヘルプ）の検索。実機の SSP は動かしていない（要件 1 の調べは設計か実装の最初の作業になる）

## 1. まとめ

- **時刻の分の分かれ目は、SSP が仮想時刻を「何で」伝えるかである。** pasta の側は、時刻を作る所が 1 か所（`lua_request.rs` の `lua_date`）と、正時を決める所が 1 か所（`virtual_dispatcher.lua` の `check_hour`）に集まっていて、差し込みは小さい。届く手がかりの候補は 3 つ見つかった（下の 3.1）。どれも公式の文書では「仮想時刻の影響を受ける」と書かれていない。→ 2026-10-17 に実機で調べ、b（`OnHourTimeSignal` の時機）と c（`system.*` プロパティ）が仮想時刻に従い、a（ヘッダー）は使えないと分かった（3.1.1）。
- **`act.req.date` をそのまま仮想時刻にすると、撫での判定（`mouse.lua`）とおしゃべりの間隔（`check_talk`）も巻き込む。** 要件 2.5（実際に過ぎた時間で数え続ける）を守るには、「時報と日時変数のための時刻」と「経過を測るための時刻」を分ける必要がある。
- **起動の立ち絵は、brief の「`[actor]` からは決まらない」が今のコードと合わない。** `[actor]` に省略可のキー `surface`（既定サーフェス）が既にある（`appearance.lua`・`reference/pasta-toml.md`）。作者が書いた場合だけ使えば「推測しない」を守れる。ただし今の意味は「同じスポットを共有するときの外見の復旧」で、起動に使うと意味が広がる。下書きは前提 A6（エンジンではやらない）に立つ。
- **準備の章で `＊OnBoot` を書かせる案は、入門の検査（`book/tools/tutorial-check.mjs`）に阻まれる。** 章の ` ```pasta ` ブロックは同じ名前の辞書の抜き出しでなければならず、`setup.md` には辞書が無い（`no-dic` で失敗する）。前提 A7（最初の起動を 1 段目へ移す）は文章だけで済む。
- **リファレンスの 2 章（`lua/shiori-events.md`・`grammar/call-jump.md`）は、スキルの references に生成される。** 直すと `node book/tools/gen-skill-refs.mjs` で `.claude/skills/` の下も書き換わる。`.claude/skills` の書き換えは分類器が拒むことがある（覚え書き `skills-edit-blocked-self-modification`）。

## 2. 今あるもの（要件ごとの対応表）

| 要件 | 今あるもの | ギャップ |
| ---- | ---------- | -------- |
| 1 調べ | 2026-10-10 の実機確認（`getting-started-story-guide` の tasks.md 5.3）: 仮想時刻を「明日 12:00」にして 90 秒待っても時報は出ない。SHIORI 呼出ログは見ていない | **Missing**: 呼出ログ・`OnHourTimeSignal` の時刻・`system.*` プロパティの値を、仮想時刻を入れた状態で見る調べ |
| 2 仮想時刻で時報 | 時刻は `lua_request.rs` 34 行 `OffsetDateTime::now_local()`。上書きは 59 行の `X-Pasta-Time` だけ。正時の判定は `virtual_dispatcher.lua` の `check_hour`（`next_hour_unix` との比較） | **Unknown**: 仮想時刻の届き方（3.1）。**Missing**: 時刻の出どころの優先順位（`X-Pasta-Time` → 仮想 → OS）。**Missing**: 時刻が戻ったときの決め直し（今は `next_hour_unix` が先に残り、戻った分だけ時報が止まる）。**Constraint**: `act.req.date` は撫での判定と `check_talk` も使う |
| 2.4 正時をまたいで進む | 今も、`current_unix >= next_hour_unix` なら進んだ先の時刻で 1 回だけ出す（スリープ明けと同じ） | 無し（今の動きが要件どおり） |
| 3 時刻を指定して時報 | `REG.OnHour` は無い。外から `\![raise,OnHour]` を起こすと `EVENT.no_entry` が `OnHour` の前方一致でシーンを探す（`＊時報12` は候補にならない）。`\![raise,時報12]` なら同名のシーンは見つかるが、日時変数は移されない | **Missing**: 時刻を受け取り、4 段の候補と日時変数の転記を `check_hour` と同じに行う口。**Constraint**: 自然な時報の状態（`next_hour_unix`・`next_talk_time`）を動かさない |
| 4 外から起こした OnTalk | `\![raise,OnTalk]` は `EVENT.fire` → `EVENT.no_entry` で新しいシーンになり、`set_co_scene` が待っていた続きを閉じる。続きを拾うのは `check_talk` の 202〜204 行だけ | **Missing**: 文書の明記。`call-jump.md` 345 行「次に OnTalk（ランダムトーク）が発生する機会」と `shiori-events.md` 53・444・603 行「次の OnTalk の機会」は、外から起こした `OnTalk` を含むようにも読める |
| 5 起動の既定 | `boot.lua` は `REG.OnBoot = REG.OnBoot or function(act) return SCENE.co_exec(...) end`。シーンが無ければ 204。`OnGhostChanged` は既定ハンドラが無く `EVENT.no_entry` | 前提 A6 なら**コードの変更は無い**。`shiori-events.md` の `OnBoot` 節に「シーンが無いと何も返さず、立ち絵は最初の台詞と一緒に出る」を足すだけ |
| 6 6 段目 | `06-hour.md` 98〜120 行（正時を待つ手順と「仮想時刻は効かない」）。開発者用機能の節は `07-greeting.md` 220 行。`13-nar.md` 26・48 行と `STAGES.md` 23・90・91 行が「7 段目で有効にした」を参照 | 決めた結果に合わせて書き換える。節を 6 段目へ移すなら、参照 4 か所と見出しへのリンクを直す（`link-check.mjs` が切れを見る） |
| 7 準備の章 | `setup.md` 195〜220 行（起動・「何も表示されない」・タスクバー・`pasta.log`・うまく起動しないとき）。`01-boot.md` は辞書を足して「SSP を終了し、もう一度起動する」 | 前提 A7 なら文章の移し替えだけ。準備の章に絵は無い（申し送りのとおり）。1 段目の絵（`01-boot.png`）は手順が変わっても画面は同じ見込み |
| 8 変えないもの | `X-Pasta-Time` の結合テスト多数（`lua_request_parse_test.rs`・`lua_request_date_test.rs`・`scene_kick_*`・`common/test_env.rs` ほか）。既定ハンドラの `REG.X = REG.X or …` の形（touch-poke-shortcuts 要件 8.7） | 既存テストがそのまま守る。新しいテストは仮想時刻が無いときのバイト不変を確かめる |

## 3. 調べが要る点（Research Needed）

### 3.1 SSP が仮想時刻を伝える手がかり（議題 1）

公式の文書で分かったこと:

- SSP ヘルプ「開発用パレット」: 仮想時刻は「SSPが認識している現在日時を、設定した日時に仮想的に置き換えます」とだけ書く。SHIORI に何が届くかは書いていない。
- UKADOC の `OnNotifyHolidayInfo` だけが「タイムマシンの影響を受ける」と明記している（日付だけ。起動時に 1 回）。
- `OnSecondChange`・`OnMinuteChange` の Reference に日時は無い（OS の連続起動時間・見切れ・重なり・トーク可否・放置時間）。

実機で確かめる候補（どれも未確認）:

| 候補 | 何が分かれば使えるか | pasta 側の重さ |
| ---- | -------------------- | -------------- |
| a. SHIORI リクエストのヘッダー | 仮想時刻を入れた後の毎秒のリクエストに、日時らしいヘッダーが載る | 小。`lua_request.rs` で読む |
| b. `OnHourTimeSignal`（SSP の正時のイベント） | 仮想時刻の正時で SSP がこのイベントを送る | 中。今の pasta はこのイベントを使っていない。時刻の値は Reference に無いので、「仮想の正時が来た」合図としてだけ使える。日時変数の値は別に要る |
| c. プロパティ `system.year`〜`system.second` | 仮想時刻を入れると、これが仮想の日時を返す | 大。読むには `\![get,property,…]` を返して次のイベントで受け取る往復が要る（`act:get_property` はシーンのコルーチンの中だけで使える）。毎秒の判定には重い。起動時と仮想時刻の変更時だけ読み、OS の時計とのずれ（差分）を持つ形なら軽い |
| d. さくらスクリプトの `%hour` など | — | 使えない。展開するのは SSP で、pasta の判定には届かない |

調べの手順の案（brief の再測定どおり）: 開発用パレットの「SHIORI呼出ログ有効」を入れ、仮想時刻を正時の 30 秒前に合わせて 1 分待ち、`ssp_SHIORI_log.txt` の `OnSecondChange`・`OnHourTimeSignal` を見る。あわせて SSP の MCP（`get_property`）で `system.hour` を読む。

### 3.1.1 実機の調べの結果（2026-10-17・要件 1.1〜1.3）

- **環境**: SSP 2.9.18（`C:\wintools\ssp`）、ゴースト「えも2DEBUG」（`pasta.dll` 0.4.0。SHIORI の応答は `Sender: Pasta`）。
- **手順**:
  1. 開発用パレット（`\![open,developer]`）で「SHIORI呼出ログ有効」を入れる。
  2. 実際の時刻 23:38:04 に、「現在時刻の仮想的変更」（TimeMachine）を同じ日の 23:59:30 にして Apply する。
  3. 1 分ほど待って、ゴーストの `ghost/master/ssp_shiori_log.txt` と、SSP の MCP の `get_property` を見る。
  4. 最後に Reset し、ログを切る。
- **結果**（表の時刻は、ログに記された実際の時刻）:

| 見たもの | 結果 | 候補 |
| -------- | ---- | ---- |
| リクエストのヘッダー | 日時らしいヘッダーは無い。出てきた名前は `Charset`・`ID`・`ReferenceN`・`SecurityLevel`・`SecurityOrigin`・`Sender`・`SenderType`・`Status`・`Value` だけ。`OnSecondChange` の Reference も、仮想時刻の前後で同じ形（OS の連続起動時間ほか） | a: **使えない** |
| `OnMinuteChange` | Apply の直後（23:38:04）に 1 回届いた。仮想の 0:00:00 にあたる 23:38:34 にも届いた。実際の分の変わり目（23:38:00）にも届いている | 毎分の合図は仮想時刻に従う |
| `OnHourTimeSignal` | 仮想の 0:00 の 7 秒後（23:38:41）に届いた。実際の時刻は 23 時台のまま。Reference は文書どおりで、日時は無い（UKADOC: トークできるまで待つので、少しずれることがある）。pasta は 204 を返した | b: **使える**（仮想の正時が来た合図としてだけ） |
| プロパティ `system.*` | 設定前は `system.hour`=23。Apply 後は `system.minute`=59・`system.hour`=23。仮想の 0:00 を過ぎてからは `system.hour`=0・`system.day`=18（実際は 17 日 23 時台）。Reset の後は `system.hour`=23 に戻った | c: **使える**（仮想の日時を返す） |
| pasta の時報 | 出なかった（pasta は OS の時計で正時を決めるため。今の動きのとおり） | — |

- **結論**: SSP は仮想時刻を SHIORI に伝えている。ただし、リクエストの値としてではない。伝わり方は 2 つある。
  - 毎分・正時のイベントが届く時機
  - プロパティ `system.*` の値
  
  要件 2（仮想時刻で時報）を残し、伝わらない場合の要件（時刻を指定して時報を起こす口）は消した。どの手がかりをどう組み合わせて仮想の日時を得るか（`OnHourTimeSignal` を合図に使う、プロパティを `\![get,property,…]` の往復で読んで OS の時計との差を持つ、など）は、設計で決める。
- **要件の番号**: 要件 3 を消したので、旧 4〜8 は 3〜7 になった。この文書のほかの節の番号は、下書き時（旧番号）のまま。

### 3.2 時刻の出どころを分ける形（議題 2）

- 今は `act.req.date` 1 つが、時報の判定・日時変数・おしゃべりの間隔・撫での判定（`mouse.lua` 132 行。`unix` と `ns` から秒を作る）の 4 つに使われている。コールバックの待ち時間は `os.time()` で別。
- 要件 2.5 を守るには、`act.req.date` を仮想にして経過は別の時刻で測るか、`act.req.date` は OS のままで時報用の時刻を別に持つか、のどちらかになる。Lua の作者が `act.req.date` を読む（`shiori-events.md` 124〜131 行に表がある）ので、どちらにするかは公開の約束に触れる。設計で決める。

### 3.3 時刻を指定する口の名前（議題 1・前提 A3）

- `OnHour` という名前で受けると、今の `EVENT.no_entry` の前方一致（`OnHour12`・`OnHourOther` のシーンを当てる）と動きが変わる。既存のゴーストで `＊OnHour…` を外から起こして使っている例があるかは分からない。
- pasta 専用の名前（例: `OnPastaHour` など）にすると衝突は無いが、入門で覚える名前が 1 つ増える。
- 値の渡し方（`Reference0` に時刻）と、範囲外・無指定のときの扱いは要件 3.4 のとおり。

## 4. やり方の選択肢

### 時刻の分

- **案 A（既存を広げる・手がかり a か b が使えた場合）**: `lua_request.rs` で仮想時刻の日時を作り、`X-Pasta-Time` の後ろ・OS の時計の前に置く。`check_hour` に「時刻が戻ったら決め直す」を足す。経過を測る所（`check_talk`・`mouse.lua`）は OS の時計を使うよう分ける。
  - 良い点: 触るファイルが brief の見込み（`lua_request.rs`・`virtual_dispatcher.lua`）に収まる。テストは `X-Pasta-Time` と同じ作りで書ける。
  - 悪い点: `act.req.date` の意味が変わるなら、`mouse.lua` も触る（brief の見込みの外。`novel-*` とは重ならない）。
- **案 B（新しい口を作る・手がかりが無い場合）**: 時刻を指定して時報を起こすハンドラを 1 つ足す（`virtual_dispatcher.lua` の 4 段の候補と日時変数の転記を関数として使う）。仮想時刻には乗らない。
  - 良い点: SSP の動きに左右されない。実機の調べの結果を待たずに作れる。
  - 悪い点: 入門で覚える手順が「パレットで時刻を変える」から「スクリプト入力でイベントを起こす」になる（7 段目で覚える操作と同じなので、6 段目へ前倒しになる）。
- **案 C（やらない）**: 調べの結果と理由を `STAGES.md`・6 段目に書くだけ（2〜3 タスク）。
  - 良い点: 小さい。悪い点: 入門の 6 段目は「待つ」のまま。

### 外から起こした OnTalk（議題 3）

- **案 D（今の動きを書く・前提 A4）**: 文書 3〜4 か所だけ。絵の撮り直しは無い。
- **案 E（続きを出す）**: `init.lua` の `EVENT.fire` か `EVENT.no_entry` で、`OnTalk` のときだけ `STORE.co_scene` を拾う。`STOP.hold_handler` はハンドラの直前に残す。`novel-talk-flow`・`novel-checkpoint-rollback`・`shell-element-click` と `init.lua` を共有するので、同じウェーブに置けない（今のウェーブでは問題ない）。11 段目の 2 枚の撮り方が変わる。

### 起動の立ち絵（議題 5・6）

- **案 F（やらない・前提 A6）**: コードは変えない。`shiori-events.md` の `OnBoot` 節に動きを書く。
- **案 G（`[actor].surface` を使う）**: `OnBoot`（と `OnGhostChanged`）に応えるシーンが無いとき、`surface` を書いたアクターだけ `\p[spot]\s[surface]` を返す。書いていなければ今と同じ 204。作者が書いた値だけを使うので推測ではない。ただし、`surface` を書いていて `＊OnBoot` が無い既存のゴーストは、起動の見え方が変わる（要件 8.4 とぶつかる）。`surface` の意味が「復旧の既定値」から広がるので、`pasta-toml.md` の説明も変わる。
- **案 H（`0`・`10` の決め打ち）**: 辞書を書いた人の意図を推測するので、制約（「出したくない絵を勝手に出さない」）と覚え書き `no-guessing-for-unintended-dsl-usage` に反する。勧めない。

### 準備の章（議題 7）

- **案 I（最初の起動を 1 段目へ・前提 A7）**: `setup.md` は置くところまで。起動・成功の目印・うまく起動しないときの節を `01-boot.md` へ移す。辞書と段は変えない。`tutorial-check.mjs` に引っかからない。
- **案 J（準備の章で `＊OnBoot` を書かせる）**: `tutorial-check.mjs` が `setup.md` の ` ```pasta ` ブロックを `no-dic` で落とす。通すには hello-pasta に `setup.pasta` を足すか検査を変える必要があり、段の辞書と `tutorial_stages_test.rs`（触らない約束）に響く。1 段目の内容とも重なる。
- **案 K（今の書き方のまま目印を言い直す）**: 一番小さいが、「最初の成功体験が何も起きない」は残る。

## 5. 規模と危うさ

- **規模: M（3〜7 日）**。時刻の分は、手がかりが使えれば 4〜6 タスク（案 A）、使えなければ案 B で 3〜4、案 C で 2〜3。起動の分は案 F・I で 2〜3 タスク（文書が主）。外から起こした OnTalk は案 D で 1。合わせて 7〜11 の見当は brief と同じ。
- **危うさ: 中**。SSP の仮想時刻の届き方が分からない（外の仕様に依存）。`act.req.date` の意味を変えるなら、Lua の作者に見える約束に触れる。文書の側は、決まった型に沿うので低い。

## 6. 設計への申し送り

- 最初に 3.1 の実機の調べを行い、結果で要件 2 と要件 3 のどちらを残すかを確定する（要件定義の議題で決め切れなければ、設計の最初のタスクにする）。
- `X-Pasta-Time` の扱い（形式・失敗時の 400）は変えない。新しい時刻の出どころが読めないときは、エラーにせず OS の時計へ落ちる（要件 2.9）。
- 経過を測る時刻（おしゃべりの間隔・撫で・コールバック）と、時報・日時変数の時刻を分ける。
- `boot.lua`・`second_change.lua` などの既定ハンドラは `REG.X = REG.X or function …` の 1 行の形を保つ。
- `event/init.lua` を触るなら、`STOP.hold_handler(act)` をハンドラを呼ぶ直前に残す。
- リファレンス 2 章を直したら `node book/tools/gen-skill-refs.mjs` を回す。`.claude/skills` の書き換えが拒まれたら、回避を探さずに開発者へ許可を求める。
- 入門の章を直したら、`link-check.mjs`・`tutorial-check.mjs` と、絵の撮り直す決まり（`book/SCREENSHOTS.md`）を確かめる。影響しうる絵は `06-hour.png`（6 段目）と、案 E のときの `11-jump-detour.png`・`11-jump-continue.png`。
- テストの置き場: Lua は `crates/pasta_lua/tests/lua_specs/`（`second_change_thread_test.lua` と同じ形）、結合は `crates/pasta_shiori/tests/`（`lua_request_date_test.rs` と同じ形。本物のランタイムで、回避を書かない）。
