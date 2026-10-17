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

---

# 設計の調べと決定（2026-10-17・設計フェーズ）

この節から下は設計フェーズで足した。要件の番号は `requirements.md` の今の番号（要件 1〜7）で書く。上の節（1〜6）は下書き時の旧番号のままで、旧 4〜8 が今の 3〜7 にあたる。正は `requirements.md` である。

## Summary

- **Feature**: `tutorial-ssp-gaps`
- **Discovery Scope**: Extension（既存の時報の振り分けと入門ガイドの手直し。新しい依存は無い）。軽い調べ（`design-discovery-light.md`）で行った。
- **Key Findings**:
  - 仮想の日時は「`system.*` プロパティを読む」ことでしか値として得られない。毎分・正時のイベントは、届く時機だけが仮想時刻に従う（3.1.1）。そこで、届く時機のずれを「仮想時刻が変わったかもしれない」合図に使い、合図があったときだけプロパティを 1 回読み、OS の時計とのずれ（秒）を持つ形にした。
  - プロパティを読む既存の口 `act:get_property` は、シーンのコルーチンの中でしか使えず、待機の登録（`CALLBACK.consume_staged`）が `set_co_scene` を通って**待っているチェイントークの続きを閉じる**。時刻の読み取りに使うと、要件 3.2（続きは自然なおしゃべりで出る）を壊す。そのため、時刻の読み取りは `CALLBACK` を使わず、専用のイベント名の往復にした。
  - Rust（`lua_request.rs`）を変えずに済む。`act.req.date` は OS の時計（または `X-Pasta-Time`）のまま残し、仮想時刻は「時報の判定」と「日時変数」の 2 か所だけが読む（決定 A2）。`X-Pasta-Time` の扱い（要件 7.1）はコードごと手つかずになる。

## Research Log

### 仮想時刻の値の取り方（要件 2）

- **Context**: 3.1.1 の実機の調べで、仮想時刻はヘッダーに載らず、`system.*` プロパティと、`OnMinuteChange`・`OnHourTimeSignal` の届く時機で伝わると分かった。
- **Sources Consulted**: 3.1.1 の実機の記録。`act.lua` の `get_property`（`\![get,property,{イベント名},{名前…}]` を返し、ベースウェアが同名のイベントの Reference で値を送り返す）。`event/callback.lua`・`event/init.lua` の `set_co_scene`。UKADOC の `OnHourTimeSignal`（トークできるまで待つので、少しずれる）。
- **Findings**:
  - プロパティを読む往復は、`\![get,property,…]` だけのスクリプトを応答として返せば、どのイベントの応答からでも始められる。シーンのコルーチンは要らない。
  - `OnMinuteChange` は、仮想時刻を入れた瞬間（Apply）に 1 回、その後は仮想の分の変わり目ごとに届く。`OnHourTimeSignal` は仮想の正時の数秒後に届く。どちらも Reference に日時は無い。
  - `OnSecondChange` の既定ハンドラは pasta のもので、時報・おしゃべりの応答を返す場所である。読み取りのスクリプトもここから返せば、ゴーストの `REG` やシーンの振り分けに触れずに済む。
- **Implications**: 仮想時刻の部品は「合図を見る」「読み取りのスクリプトを出す」「読んだ値からずれを決める」「ずれを足した時刻と日付の表を返す」の 4 つの働きを持つ 1 つの Lua モジュールにまとまる。

### 時刻を読む所（要件 2.2・2.5）

- **Context**: `act.req.date` は、時報の判定・日時変数・おしゃべりの間隔（`check_talk`）・撫での判定（`mouse.lua` の `request_time`）が読む。コールバックの待ち時間は `os.time()`。
- **Findings**: 仮想時刻に従わせるのは、`check_hour`（正時の判定と `{HH}`）、`check_talk` の正時までの残り（`hour_margin`）、`transfer_date_to_var`（日時変数）の 3 か所だけでよい。`act.req.date` は変えない。
- **Implications**: 公開の約束（`shiori-events.md` の `act.req.date` の表）は変えず、「`act.req.date` は仮想時刻に従わない。仮想時刻に従うのは時報と日時変数」と書き足すだけになる。

### 外から起こした OnTalk（要件 3）

- **Findings**: `\![raise,OnTalk]` は `EVENT.fire` → `EVENT.no_entry` で新しいシーンになり、`EVENT.drive` の `set_co_scene` が待っていた続きを閉じる。続きを拾うのは `check_talk` だけ（上の 2・4 節のとおり）。決定 A4 によりコードは変えない。
- **Implications**: 文書 3 か所（`call-jump.md` のチェイントーク、`shiori-events.md` の 3 か所、入門 11 段目）を直すだけ。絵は変わらない。

### 起動の既定の動き（要件 4）

- **Findings**: `boot.lua` は同名のシーンを探すだけで、無ければ 204。`OnGhostChanged` は既定ハンドラが無く `EVENT.no_entry`。決定 A6 によりコードは変えない。
- **決めた理由（要件 4.5）**: 何も書かなければ何も表示しないのが pasta の既定の動きである（2026-10-16 の開発者の決定）。辞書が無いと、どのサーフェスを出すかを pasta は決められない。`0`・`10` の決め打ちは辞書を書いた人の意図の推測になる。`[actor].surface` を起動に使うと、`surface` を書いていて `＊OnBoot` が無い既存のゴーストの起動の見え方が変わる（要件 7.4）。入門の体験は、最初の起動を 1 段目へ移すことで手当てする（決定 A7）。

### 入門ガイドの絵（要件 5.4・6.5）

- **Findings**: `06-hour.png` は時報の吹き出し（午後4時）を写していて、確かめ方が変わっても成功の目印の画面は同じである。`01-boot.png` は 1 段目の最初の台詞の画面で、準備の章から起動を移しても同じ画面になる。11 段目の 2 枚は、決定 A4 で動きが変わらないので変わらない。準備の章に絵は無い。
- **Implications**: 撮り直す絵は無い見込み。`book/SCREENSHOTS.md` の 77 行「6 段目は、実際の正時を待つ（SSP の仮想時刻は pasta に届かない）」は古くなるので直す。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| P1. 合図があったときだけ読む（採用） | 毎分・正時のイベントが、今のずれから見込む時機から外れて届いたら、次の `OnSecondChange` でプロパティを 1 回読む | 仮想時刻を使わない間は、応答が今とバイト単位で同じ（要件 7.3） | 合図を見落とす組み合わせがある（下の Risks） | 合図の許し幅は定数 |
| P2. 毎分読む | `OnMinuteChange` のたびにプロパティを読む | 見落としが無い。判定が単純 | 仮想時刻を使わないゴーストでも、毎分、文字の無いスクリプトを返す。表示中の吹き出しへの影響を確かめていない。要件 7.3 に触れる | 却下 |
| P3. `OnHourTimeSignal` で時報を出す | 正時のイベントを時報の合図にし、時刻だけプロパティで読む | 時報の時機は SSP に任せられる | 日時変数（要件 2.2）が正時以外のシーンで仮想にならない。仮想時刻を使わないときの時報の出方も変わる（7.3） | 却下 |
| P4. `act:get_property` を使う | 小さなコルーチンで `get_property` を呼ぶ | 既存の待機・時間切れの仕組みに乗る | 待機の登録が待っている続きを閉じる（要件 3.2 を壊す） | 却下 |
| P5. Rust で時刻を作り直す | `lua_request.rs` にずれを持たせる | `act.req.date` 自体が仮想になる | 撫で・おしゃべりの間隔まで仮想になり、要件 2.5 を守るには結局 Lua 側で OS の時刻を別に持つ必要がある。Rust と Lua で状態が分かれる | 却下 |

## Design Decisions

### Decision: 仮想時刻は「OS の時計とのずれ（秒）」として Lua の 1 モジュールが持つ

- **Context**: 要件 2.1〜2.9、決定 A2。
- **Alternatives Considered**: 上の P1〜P5。
- **Selected Approach**: 新しいモジュール `pasta.shiori.clock` が、ずれ `offset`（整数秒、0 は仮想時刻なし）と、読み取りの予約 `probe_armed` を持つ。`clock.now(req)` は `req.date.unix + offset`、`clock.date(req)` はその時刻の日付の表を返す。`X-Pasta-Time` があるリクエストでは、ずれを足さない（要件 2.8）。
- **Rationale**: 仮想に従う所（時報の判定・日時変数）と従わない所（`act.req.date`・間隔・撫で・コールバック）を、読む関数の違いだけで分けられる。Rust を変えない。
- **Trade-offs**: 読み取りの往復の遅れと秒の切り捨てで、ずれには 1〜2 秒の誤差が出る。時報が仮想の正時から 1〜2 秒ずれて出ることを許す（`OnHourTimeSignal` 自体も数秒ずれる）。
- **Follow-up**: 実装の最後に、実機の SSP で TimeMachine の Apply・Reset と時報を確かめる。

### Decision: 読み取りは専用のイベント名の往復で行い、`CALLBACK` を使わない

- **Context**: 要件 3.2 を守る（上の Research Log）。
- **Selected Approach**: 読み取りのスクリプトは `\![get,property,OnPastaClock,system.year,system.month,system.day,system.hour,system.minute,system.second]\e`。返ってきた `OnPastaClock` は `EVENT.fire` が `REG` より前に受け取り、ずれを決めて 204 を返す。`SecurityLevel` が `external` の `OnPastaClock` は無視する。
- **Trade-offs**: 時間切れの扱いは持たない。返事が来なければ（SSP 以外のベースウェアなど）、ずれは前の値のまま、次の合図まで読み直さない。

### Decision: 合図の判定（許し幅は定数）

- `OnMinuteChange`: `clock.now(req) % 60` が 5 以下か 55 以上なら見込みどおり。外れたら予約を立てる。
- `OnHourTimeSignal`: `clock.now(req) % 3600` が 60 以下か 3595 以上なら見込みどおり（トークできるまで待つので、遅れを 60 秒まで許す）。外れたら予約を立てる。
- 読んだずれの絶対値が 2 秒以下なら 0 にする（仮想時刻なし。要件 2.6・2.7 を OS の時計と同じ値で満たす）。
- 定数は設定にしない（pasta.toml のキーを増やさない。要件 7.2 の「別の設定を要する作りにしない」）。

### Decision: 時刻が戻ったら次の正時を決め直す（要件 2.3）

- **Selected Approach**: `check_hour` は、`now < next_hour_unix - 3600`（記録した正時の 1 時間前より前）なら、`next_hour_unix` を `now` から決め直し、時報は出さない。進んだとき（`now >= next_hour_unix`）は今のまま、進んだ先の時刻で 1 回だけ出す（要件 2.4）。
- **Trade-offs**: この決め直しは、ずれが 0 のまま OS の時計が戻ったとき（手で時計を戻したなど）にも働く。今は、戻る前の次の正時まで時報が止まる。設計の議題に残す（design.md の OPEN QUESTIONS）。

### Decision: 起動の立ち絵はエンジンでやらない（要件 4。決定 A6）

- 上の Research Log「起動の既定の動き」の理由のとおり。`boot.lua` は変えない。

## Synthesis

- **一般化**: 「時報の判定」と「日時変数」は、どちらも「今の日時」を読む所である。読む口を `clock.now`・`clock.date` の 2 つにまとめ、どちらも `X-Pasta-Time` → 仮想時刻 → OS の時計の順を 1 か所で決める（要件 2.8）。将来、ほかの所を仮想に従わせるときも、この 2 つを呼ぶだけで済む（今は広げない）。
- **作るか使うか**: 日付の計算は Lua 標準の `os.time{…}`（プロパティの日時 → Unix 時刻）と `os.date("*t", t)`（Unix 時刻 → 年月日時分秒・曜日・通し日）を使う。どちらも OS の地方時で、Rust の `OffsetDateTime::now_local()` と同じ時刻帯になる。フレームワークのスクリプトは既に `os` を使っている（`std_os`）。プロパティの読み取りは SSP の `\![get,property]` をそのまま使う。新しい依存は無い。
- **簡素化**: Rust の変更・新しい設定キー・新しい `REG` の既定ハンドラ・時間切れの処理・ベースウェアの判定（`Sender`）は入れない。起動の立ち絵と外から起こした `OnTalk` はコードを変えず、文書だけにする。

## Risks & Mitigations

- 実機で確かめていない SSP の動きがある（Reset の瞬間に `OnMinuteChange` が届くか、文字の無いスクリプトが表示中の吹き出しを消すか、プロパティの値の形）。— 実装の最後の作業で実機の SSP で確かめる。合図が来なくても、次の分の変わり目のずれで読み直す。
- 合図の見落とし: ずれの変化が 60 秒の倍数（±5 秒）で、かつ Apply が分の変わり目の近くだったときは、毎分の合図では気づかない（見込み 3% ほど）。— `OnHourTimeSignal` の時機で気づく。気づくのは仮想の正時の後で、時報は正時をまたいだ扱い（要件 2.4）で 1 回出る。ずれの変化が 1 時間の倍数に近いときは気づけない（記録して許す）。
- 時報の時機の誤差（1〜2 秒）。— 許す（上の Decision）。
- `.claude/skills` の references の生成が分類器に拒まれる。— 回避を探さず、開発者に許可を求める（覚え書き）。

## 使ったスキルと規則

- `kiro-spec-design` の規則（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）。
- 作者向けのスキル（`pasta-ghost-authoring`・`pasta-lua-coding`）は、マニュアルの章（`shiori-events.md`・`call-jump.md`・`variables.md`）が正本なので、章を直接読んだ。
