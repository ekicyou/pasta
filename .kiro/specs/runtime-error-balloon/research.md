# ギャップ分析: runtime-error-balloon

調べた時点: 2026-10-17・main `30576a4b`（`failure-output-unification` の `e9b321e0` を含む）。

## Summary
- **Feature**: `runtime-error-balloon`
- **Discovery Scope**: Extension（既存の 500 の出口に、既存の失敗表記の仕組みをつなぐ）
- **Key Findings**:
  - Lua の中で捕まる止まるエラーは、すべて 2 つの出口を通る。要求を受ける関数 `SHIORI.request` の `xpcall`（`entry.lua` 79〜89 行）と、時間切れの掃引の `time_out`（`callback.lua` 160〜182 行）である。Rust の側は Lua の応答を素通しするので、Lua の側だけで要件 1〜4 を満たせる（`pasta_shiori` のソースは触らずに済む）。
  - 表記・エスケープ・切り替えは `pasta/failure.lua` に揃っている。`FAILURE.notation(record)` が「警告を出し、表記の文字列を返す（出さない設定なら nil）」なので、出口で `nil` か否かを見るだけで、要件 4.2（出さない設定で今と同じ応答）を満たせる。種類の名前は `FAILURE.LABEL` に 2 行足す。
  - `RES.build` は任意のヘッダーを足せるので、`RES.err(reason, { Value = 表記 })` の形で 500 に値を付けられる。公開の `RES.err` の中身は変えずに済む。200 にする案でも `RES.build("200 OK", …)` で足りる。
  - 未確定で最大の不明点は、SSP が 500 の応答の `Value` をバルーンに出すか（UKADOC は定めていない）。実機の確認が最初の作業になる。
  - 途中まで積んだ台詞（要件 3.4・Q5）は、`SHIORI.request` の出口からは見えない（act は `EVENT.fire` の局所変数・待機の entry の中にある）。採るなら `event/init.lua` の失敗の経路に手を入れる必要があり、brief の見立て「`init.lua` は触らずに済む」が崩れる。

## Research Log

### 止まるエラーが通る出口（Lua の側）
- **Context**: 要件 1 の対象を、どの出口で拾えば漏れが無いか。
- **Sources Consulted**: `crates/pasta_lua/pasta_scripts/pasta/shiori/entry.lua`・`event/init.lua`・`event/callback.lua`・`event/second_change.lua`・`res.lua`、`book/src/internals/shiori.md`。
- **Findings**:
  - `SHIORI.request(req)` は `xpcall(EVENT.fire, error_handler)`。`error_handler`（34 行）は、エラーが文字列なら最初の 1 行、それ以外は `nil` を返し、`RES.err(result)` が 500 を作る（`nil` は `"Unknown error"`）。
  - `EVENT.fire` の中のエラーはすべてここへ来る: REG のハンドラの例外、シーンの失敗（`EVENT.drive` が `false` を返し、229 行の `error(yielded_value)` で投げ直す）、待機の再開の失敗（`CALLBACK.try_route` 110 行の `error(value)`）。
  - 時間切れは `OnSecondChange` の既定ハンドラ → `CALLBACK.sweep` → `time_out`。理由付き（`entry.on_timeout` が文字列）で失敗したら `RES.err(reason)` を返し、`EVENT.fire` が `SHIORI/` で始まる文字列として素通しする。理由の無い時間切れの後の失敗は `log.warn` だけで `nil`（応答の候補にならない）。`act:get_property` は既定の `timeout_message` に `"callback timeout: get_property"` を入れるので、作者が明示しない限り「理由付き」になる。
  - 掃引の採用の規則（最初の候補だけ・残りは警告）は `CALLBACK.sweep` にある（要件 1.5 はそのまま満たせる）。
  - NOTIFY は Rust の側（`actor/thread.rs` 209 行）で応答を捨てる。Lua の側で表記を作っても表示されないので、要件 1.7 は何もしなくても満たせる。ただし Lua の側は GET/NOTIFY を `req.method` で区別できる（表記を作る手間を省くなら見る）。
- **Implications**: 出口は 2 か所（`entry.lua` の `SHIORI.request`、`callback.lua` の `time_out`）で足りる。`res.lua` と `event/init.lua` は、Q5（途中の台詞）を「捨てる」にすれば触らない。

### Rust の側の出口
- **Context**: Q4（Rust まで届くエラーを含めるか）。
- **Sources Consulted**: `crates/pasta_shiori/src/error.rs`（79 行 `single_line`・104 行 `to_shiori_response`）、`shiori.rs`（137 行 `request`・266 行 `call_lua_request`）、`actor/thread.rs`（197 行）、`book/src/internals/shiori.md` 154 行・205 行、`book/src/reference/startup.md` 72・126 行。
- **Findings**:
  - Rust の 500 は 3 つ: `MyError::Load`（ランタイムの構築の失敗）、`NotInitialized`、`Script`（`SHIORI.request` の呼び出しそのものの失敗。`xpcall` の外のエラー、戻り値を文字列にできない、など）。
  - 読み込みの失敗は、SSP 2.8.98 では要求そのものが来ない（マニュアルの実測）。`Script` は、標準の `entry.lua` では `RES.err` 自体が失敗したときくらいで、作者が `scripts/pasta/shiori/entry.lua` を上書きした場合に限られる。
  - Rust の側には `balloon_warning` を読む道が今は無い（設定は Lua の `pasta.config` が読む）。読み込みに失敗した状態では設定そのものが無い場合もある。
  - `call_lua_request` は Lua の応答の文字列をそのまま返し、ステータスを見ない（debug のログに出すだけ）。Lua が 500＋`Value` や 200 を返しても、Rust の側は変更なしで通る。
- **Implications**: Rust の側を含めると `pasta_shiori` に設定の読み取りと表記の形の写しが要り、`failure.lua` の「形を 1 か所で決める」に反する。得るものは、作者が `entry.lua` を上書きした場合だけ。仮の案（含めない）の根拠になる。

### 表記と切り替えの部品（`pasta/failure.lua`）
- **Context**: 要件 2・4 を、作らずに満たせるか。
- **Sources Consulted**: `crates/pasta_lua/pasta_scripts/pasta/failure.lua`、`.kiro/specs/completed/failure-output-unification/`（requirements・design・research）、`book/src/reference/runtime-failures.md`。
- **Findings**:
  - `FAILURE.format(label, subject, detail)` が `【label：subject（detail）】` を作り、中身全体に `FAILURE.escape`（`\`→`\\`、`%`→`\%`、CR・LF→空白）をかける。Windows のパスの `\` もこれで書いたとおりに出る（要件 2.6）。
  - `FAILURE.notation(record)` は `record.warning` を warn で出し、`FAILURE.enabled()` が偽なら `nil`、真なら表記を返す。act に積まない（act の無い出口向けに、`failure-output-unification` の Q12 で用意されたもの）。
  - `FAILURE.enabled()` は報告のたびに `pasta.config` から `[logging] balloon_warning` を読み、真偽値でなければ `true`（要件 4.3）。
  - `failure.lua` はログと設定を呼んだ時点で `require` するので、`shiori/` の出口から読んでも循環しない（申し送りのとおり）。
  - `FAILURE.LABEL` は表で、種類を足すのは 1 行ずつ（Out of Boundary「報告の関数・表記の形・切り替えの設定そのもの」と食い違わない）。
- **Implications**: 要件 2.1・2.6・4.1〜4.3 は部品の呼び出しだけで満たせる。要件 5.6（ログ）は `record.warning` にエラー文を渡せば、表記の設定にかかわらず warn が 1 行出る（Q10 の仮の案と一致）。

### エラー文の中身（場所・行番号・投げ直し）
- **Context**: Q2。`failure-output-unification` は「表記に行番号を入れない」と決めた。止まるエラーの文には場所が入る。
- **Sources Consulted**: `entry.lua` の `error_handler`、`event/init.lua` 229 行、`callback.lua` 110 行、`book/src/lua/shiori-events.md` 621〜633 行（「`error("…")` のメッセージには、通常、発生位置が前に付く」）、`crates/pasta_lua/src/debug/`（ソースの対応表 `source_map`）。
- **Findings**:
  - Lua の `error(msg)` は、`msg` が文字列なら呼んだ場所（`ファイル:行:`）を前に付ける。シーンの失敗は `EVENT.drive` が受けた文（元の場所付き）を `error(yielded_value)` で投げ直すので、応答の理由の行は `…/event/init.lua:229: <元の場所>: <文>` になっている見込み（**Research Needed**: 既存のテストはハンドラの直接の例外しか見ていない。実際の文を Lua のテストで確かめる）。`try_route` の 110 行も同じ。
  - LuaJIT は、エラー文の中のファイルの場所を、長いと先頭を `...` に縮める（チャンク名の長さの上限）。`scripts/` の Lua は `...ster\scripts\天気.lua:12:` のような形になる見込み（**Research Needed**: 実機・テストで確認）。
  - 辞書の中の Lua のブロックは、生成した Lua のキャッシュ（`profile/pasta/cache/lua/`）の場所と行番号になる見込みで、`.pasta` の行ではない。デバッガ用のソースの対応表（`pasta_lua/src/debug/source_map`）は `.pasta` の行へ引けるが、既定では無効でゼロコストの約束がある。
- **Implications**: 応答の理由の行は変えない（要件 5.1）まま、表記の側でだけ投げ直しの接頭辞を除くか、投げ直しを `error(v, 0)`（場所を足さない）に変えて理由の行も直すかは、設計の判断になる。後者は理由の行の中身が変わるので要件 5.1 と衝突する（Q2・Q9 と合わせて決める）。`.pasta` の行への読み替えは本仕様の範囲外が妥当（デバッガの仕組みに依存するため）。

### 途中まで積んだ台詞
- **Context**: Q5（要件 3.4）。
- **Sources Consulted**: `event/init.lua`（`EVENT.fire`・`EVENT.drive`）、`pasta/act.lua` 628 行（`build` が `self.token = {}` で空にする）、`pasta/shiori/act.lua` 269〜315 行（`get_property` がトークンを退避・復元）。
- **Findings**:
  - 途中の台詞は、シーンの act の `token`（直前の `yield`・`build` の後に積んだ分）にある。チェイントークで先に出した分は、すでに前の応答で出ている。
  - act は `EVENT.fire` の局所変数（シーンのとき）と、待機の entry（`CALLBACK.pending` から外した後の局所変数）にしかない。`SHIORI.request` の `xpcall` の外からは見えない。`FAILURE.running_act()` はコルーチンをキーにするので、メインスレッドの `SHIORI.request` からは引けない。
  - 理由付きの時間切れでは、`get_property` が退避したトークンを戻してから `error(reason)` するので、待つ前に積んだ台詞が残っている。
- **Implications**: 「途中まで出す」を採ると、`EVENT.fire`／`EVENT.drive` の失敗の経路（`init.lua`）か `try_route`・`time_out` で、act を出口へ渡す（エラーの値を表に包む、など）変更が要る。`EVENT.drive` は `touch-poke-shortcuts`・`callback-resume-unification` が「本体は変えない」とした共有点で、`scene-anchor-link` も近くを触る。「エラーだけ出す」なら出口 2 か所で済む。工数とファイルの重なりに効く分かれ道である。

### SSP が何を表示するか
- **Context**: Q1（要件 3.1・3.2）。
- **Sources Consulted**: UKADOC「SHIORI/3.0」の応答の項（ステータスコード・`ErrorLevel`・`ErrorDescription`）、UKADOC「capability」（NOTIFY）、里々のマニュアル（プロトコルとリクエスト処理・ログとデバッグ）。
- **Findings**:
  - UKADOC: 200 番台が成功、ほかは失敗。500 の応答の `Value` をベースウェアが表示するかは書いていない。
  - `ErrorLevel`（`info`〜`critical`）・`ErrorDescription` は SSP の拡張。`ErrorLevel` があると SSP はエラーのログに記録し、通知領域にエラーのアイコンを出す（通知するレベルは利用者が設定でき、通常は `error` 以上）。里々は、ベースウェアが NOTIFY `capability` で `response.errorlevel` を知らせたときだけ付ける。pasta は `capability` を扱っていない。
  - `X-ERROR-REASON` は UKADOC に無い pasta 独自の行で、SSP がどう扱うかは分からない。
- **Implications**: **Research Needed（最優先）**: SSP の実機で、(1) 500＋`Value` の台詞がバルーンに出るか、(2) 200＋`Value`＋`X-Error-Reason` が普通に出るか、(3) `OnSecondChange` の応答の扱い（会話中に毎秒エラーが来たとき）を確かめる。SSP を操作するツール（SSP MCP）で確かめられる（`hello-pasta-tutorial-stages` が使った）。`ErrorLevel` を足す案は、`capability` の扱いを足すか、無条件に付けるかの判断も伴い、配るゴーストでは利用者にアイコンが出るので、Q8 と絡む。

### 配るゴーストと切り替え
- **Context**: Q8（要件 4.4）。
- **Sources Consulted**: `book/src/reference/runtime-failures.md`（「表記の切り替え」・締めの台詞）、`book/src/reference/pasta-toml.md` 70・167・528・539 行、`crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml`、`failure-output-unification` の決定 Q1・Q11。
- **Findings**:
  - `balloon_warning` の既定は `true`。配る前に `false` にする案内は、`runtime-failures.md` の締めの台詞にしかない。
  - hello-pasta の `pasta.toml` に `balloon_warning` の行は無い（既定の `true`）。hello-pasta は既定の設定で表記を出さないことが、テスト（`tutorial_stages_test.rs`）で確かめられている。
  - `pasta_check` は `balloon_warning` を見ない（`failure-output-unification` の決定 Q11「連携させない」）。
- **Implications**: 仮の案 (a)（案内だけ）なら、追加の作業はマニュアルだけ。(c)（`pasta_check release` で知らせる）は `pasta_check` を触り、過去の決定を覆すので、採るなら議題として明示する。(b)（既定を変える）は `failure-output-unification` の表記の既定も同時に変わる。

### ログ
- **Context**: Q10（要件 5.6）。brief の「ログと、応答の理由の行は、今までどおり残る」。
- **Sources Consulted**: `entry.lua`（ログを出していない）、`shiori.rs` 290〜304 行（204 以外の応答の全文を debug で出す・`SHIORI.request` の呼び出しの失敗だけ error）、`actor/thread.rs` 197 行（debug）、`pasta-toml.md` 526 行（既定の `level` は `info`）。
- **Findings**: Lua の中で捕まえた止まるエラーは、既定の `level = "info"` ではログに 1 行も残らない（応答の全文が debug で出るだけ）。hello-pasta は `level = "debug"` なので残る。brief の「ログにしか出ない」は debug のときの話である。
- **Implications**: `FAILURE.notation` の `warning` にエラー文を渡すと、warn で 1 行残る（既定のレベルで見える）。スタックトレースを残すには `error_handler` で `debug.traceback` を取る変更が要る（今は最初の 1 行だけ返している）。

### テストとフィクスチャ
- **Sources Consulted**: `crates/pasta_lua/tests/lua_specs/shiori_entry_test.lua`（529 行。500 と理由の行を 4 か所で確かめる）、`callback_module_test.lua`（859 行。時間切れの 500 を 6 か所）、`failure_test.lua`（575 行）、`crates/pasta_lua/tests/shiori/res_test.rs`・`event_dispatch_test.rs`、`crates/pasta_shiori/tests/failure_notation_switch_e2e_test.rs` とフィクスチャ `tests/fixtures/failure_notation_switch/`（`pasta.toml` の末尾に `[logging] balloon_warning` を書き足して同じシーンを起動する形）、`async_callback_chain_test.rs` 264 行（理由付きの時間切れの `X-Error-Reason`）、`actor_marshaling_test.rs`・`load_failure_visibility_test.rs`（Rust の 500。触らない）、`common/mod.rs` の `copy_fixture_to_temp`。
- **Findings**:
  - 既存の Lua のテストは「500 であること」「理由の行」「`Value` が無いこと」（`shiori_entry_test.lua` 428 行）を確かめている。テストの環境の既定（`balloon_warning` 未設定 = `true`）では、変更後に `Value` が付くので、期待値の更新か、テストで出さない設定に固定するかが要る（要件 6.5）。
  - 結合テストは `failure_notation_switch_e2e_test.rs` の形（フィクスチャに `pasta.toml` の行を足す）をそのまま手本にできる。フィクスチャは、エラーを起こす `dic/*.pasta`（Lua のブロック）と、要るなら `scripts/` の Lua だけを持てばよい。
- **Implications**: テストのファイルはどれも 1,000 行未満で、足しても分割は要らない見込み（`callback_module_test.lua` は 859 行なので、足す量によっては新しいファイルに分ける）。

### マニュアル
- **Sources Consulted**: `book/src/reference/runtime-failures.md`（15 行目「500 応答はこのページの対象外」）、`book/src/lua/shiori-events.md`（51・150・450・621〜633 行）、`book/src/reference/startup.md`（72・126〜182 行）、`book/src/internals/shiori.md`（154・205・308・327・345・381〜388・444 行）、`book/src/debug/troubleshooting.md`（13 行）、`book/src/reference/pasta-toml.md`（528・539 行）。
- **Findings**: 章を足す必要は無い見込み（63 章のまま）。`runtime-failures.md` に節を足すのが自然で、`shiori-events.md` の「エラーハンドリング」の応答の例（500 と理由の行だけ）を直す。スキル用の文書は `gen-skill-refs.mjs` で再生成する。`shiori-events.md` はほかの spec も別の節を触る。

## 要件と既存の資産の対応

| 要件 | 既存の資産 | ギャップ |
| ---- | ---------- | -------- |
| 1.1〜1.2 止まるエラーを出す | `SHIORI.request` の `xpcall`・`RES.err` | **Missing**: 出口で表記を作って応答に付ける処理 |
| 1.3 理由付きの時間切れ | `callback.lua` `time_out` 172 行 | **Missing**: 同上（出口 2 つ目） |
| 1.4 理由の無い時間切れの後のエラー | `time_out` 175 行（警告だけ） | **Missing**＋**Constraint**: 今は応答の候補にならない。候補にすると、その回の `OnSecondChange` の応答と仮想イベントの扱いが変わる（Q6） |
| 1.5 掃引の採用の規則 | `CALLBACK.sweep` | 無し（そのまま） |
| 1.6 リソースの要求 | `req.id` | **Missing**: ID を見て出さない判定（Q7） |
| 1.7 NOTIFY | Rust が応答を捨てる | 無し |
| 1.8 抑止しない | — | 無し |
| 2.1・2.6・2.7 形・エスケープ | `FAILURE.format`・`FAILURE.escape`・`FAILURE.notation` | 無し（呼ぶだけ） |
| 2.2 種類の名前 | `FAILURE.LABEL` | **Missing**: 2 行（Q3） |
| 2.3 最初の行 | `error_handler` | 無し |
| 2.4〜2.5 場所・投げ直し | `init.lua` 229 行・`callback.lua` 110 行の `error(v)` | **Unknown**: 実際の文を確かめる（Research Needed）。**Constraint**: 投げ直しを直すと理由の行が変わる（要件 5.1） |
| 3.1〜3.2 応答のステータス | `RES.build`・`RES.err(reason, dic)` | **Unknown**: SSP の実機の挙動（Q1・最優先） |
| 3.3 理由の行を保つ | `RES.err` | 無し |
| 3.4 途中の台詞 | act の `token`・`build` | **Missing**＋**Constraint**: 出口から act が見えない。`init.lua`／`callback.lua` の失敗の経路の変更が要る（Q5） |
| 3.5〜3.6 止まったまま・次の要求 | `EVENT.drive` の失敗の経路（予約を捨て継続を空に） | 無し |
| 4.1〜4.3 切り替え | `FAILURE.enabled` | 無し |
| 4.4〜4.5 配るゴースト | `runtime-failures.md` の案内 | **Missing**: マニュアルの案内（Q8） |
| 5.1 理由の行 | `RES.err`・`to_shiori_response` | 無し（触らない。Q9） |
| 5.2 `RES.err` | `res.lua` 119 行（公開） | 無し（中を変えない。出口で `dic` に `Value` を足す） |
| 5.4 Rust の出口 | `error.rs`・`thread.rs`・`shiori.rs` | 無し（含めない案。Q4） |
| 5.6 ログ | — | **Missing**: 今は既定のレベルで残らない（Q10） |
| 6.x テスト | `shiori_entry_test.lua`・`callback_module_test.lua`・`failure_notation_switch_e2e_test.rs` の形 | **Missing**: 新しいテストと、既存の期待値の更新 |
| 7.x マニュアル | 6 ページと生成 | **Missing**: 記述の更新 |

## 実装の選択肢

### 選択肢 A: 2 つの出口だけを広げる（Lua の側だけ・途中の台詞は捨てる）
- 触るもの: `shiori/entry.lua`（`SHIORI.request` の失敗の枝）、`shiori/event/callback.lua`（`time_out`）、`failure.lua`（`LABEL` に 2 行）。`res.lua`・`event/init.lua`・`pasta_shiori` のソースは触らない。
- 出口で `FAILURE.notation{ label=…, subject=エラー文, warning=… }` を呼び、`nil` なら今の `RES.err(reason)`、文字列なら（Q1 の結果に従い）`RES.err(reason, { Value = 台詞 })` か 200 を返す。台詞の形（`\0` や `\e` を付けるか）は設計で決める。
- ✅ 最小の差分（brief の見立て 6〜7 タスク）。共有点の `init.lua` を触らず、`scene-anchor-link` などと重ならない。
- ✅ 出さない設定の応答が今と同じになることが、`nil` の分岐 1 つで保証できる。
- ❌ 要件 3.4（途中の台詞）の仮の案を満たせない（Q5 で「エラーだけ」に決めた場合の案）。
- ❌ 投げ直しの接頭辞（Q2）は、表記の側で文字列から除くことになる（推測に頼る文字列処理）。

### 選択肢 B: 失敗の経路で act を出口へ渡す（途中の台詞も出す）
- A に加えて、`EVENT.fire`（シーンの失敗）・`CALLBACK.try_route`・`time_out` の失敗の経路で、エラーの値と act（または組み立て済みの途中の台詞）を一緒に出口へ渡す。投げ直しの場所の接頭辞も、ここで値を包めば付かない。
- ✅ 要件 3.4 の仮の案を満たす。投げ直しの接頭辞を文字列処理なしで避けられる（Q2）。
- ❌ `event/init.lua` の `EVENT.fire` を触る（`touch-poke-shortcuts`・`callback-resume-unification` が「本体は変えない」とした共有点。`scene-anchor-link` も近くを触る）。
- ❌ エラーの値を表に包むと、`xpcall` の `error_handler` の「文字列でなければ Unknown error」と、作者が `scripts/` で上書きした出口との約束が変わる。包まない方法（モジュールの局所に直前の失敗の act を置く）は、状態を 1 つ増やす。
- ❌ 途中の台詞の `build` を失敗の経路で呼ぶので、組み立ての途中で再びエラーが起きる場合の扱いが要る。

### 選択肢 C: 段階的（A を先に、B の部分を議題の結論しだいで足す）
- 実機の確認（Q1）→ A の出口 2 か所 → テスト → マニュアル、を先に通す。Q5 を「途中も出す」に決めた場合だけ、B の失敗の経路の変更を 1 タスクとして後に足す。
- ✅ プロジェクトの「1 抽出 = 1 検証 = 1 コミット」に合い、途中で止めても見える価値が出る。
- ❌ Q5 を後から決めると、A で書いたテストの期待値（途中の台詞の有無）を書き直す。Q5 は要件ディスカッションで先に決めておくのがよい。

## 工数とリスク
- **工数**: S〜M。選択肢 A なら S（2〜3 日。出口 2 か所・表の 2 行・テスト・マニュアル）。B を含めると M（`init.lua` の失敗の経路と、その回帰の確認が増える）。brief の測定（7〜10 タスク）と合う。
- **リスク**: Medium。コードは既存の型どおりだが、SSP が 500 の `Value` を表示するかが分からず、応答のステータスの選び方が実機の結果に依存する。`OnSecondChange` の応答（会話中・毎秒）の SSP での扱いも未確認。

## 設計への申し送り
- **推奨の形（情報として）**: Q5 が「エラーだけ」なら選択肢 A、「途中も出す」なら選択肢 C（A の後に B の部分を足す）。どちらでも、表記は `FAILURE.notation` を出口から呼び、`RES.err` の中は変えない（公開の関数のため。brief の再測定の推奨と同じ）。
- **最初の作業**: SSP の実機で、500＋`Value`、200＋`Value`＋理由の行、`OnSecondChange` の応答の見え方を確かめる（Q1）。結果を `research.md` に追記する。
- **Research Needed**:
  1. SSP の 500 の応答の `Value` の表示（Q1）。
  2. シーンの中のエラーの実際の文（投げ直しの接頭辞・LuaJIT のチャンク名の縮め方・辞書の Lua のブロックの場所の形）（Q2）。
  3. 理由の無い時間切れの後のエラーを応答の候補にしたときの、`OnSecondChange` の仮想イベント（`OnTalk`・`OnHour`）の扱い（Q6）。
  4. 表記を値にする台詞の形（`\0` のスコープ・`\e`・バルーンの選び方）。表記だけの台詞で SSP が何を出すか。
- **既存のテストへの影響**: `shiori_entry_test.lua` 428 行の「`Value` が無い」と、`callback_module_test.lua`・`async_callback_chain_test.rs` の理由付きの時間切れは、既定の設定で応答が変わる。期待値を更新するか、出さない設定に固定して「今と同じ」の確認に使い、出す設定の確認は新しいテストに分ける。
- **ファイルの重なり**: `failure.lua` の `LABEL` は `call-attribute-filter`・`scene-anchor-link`・`shell-element-click` も足すので、同じウェーブに置かない（brief どおり）。`shiori-events.md` は他の spec も別の節を触る（後から入る側が合わせる）。

## 要件ディスカッションの決定（2026-10-17）
- Q5 は「途中までの台詞の後に表記を出す」に決まった。実装の形は選択肢 B（または C）になる。`event/init.lua` の失敗の経路を触る。
- Q2 は「トランスパイラを見せない」に決まった（要件 2.2〜2.7）。設計で決めること:
  1. 作者が書いたシーン名の取り方。act の `current_scene.__global_name__` は登録名（`メイン_1`・`OnTalk_1` のような基本名＋連番。別名は置き換え後）なので、書いた名前に戻す方法が要る（連番を落とす・別名を戻す・トランスパイラが書いた名前を表に残す、など）。
  2. ローカルシーン名の取り方。act はグローバルシーンの表しか持たない。止まったコルーチンの呼び出し履歴（`debug.getinfo(co, level, "f")`）の関数を、シーンの表の値と照らして引く案がある。失敗したときだけ調べるので、普段の重さは増えない。
  3. エラー文の場所の見分け方。変換した Lua のキャッシュ（`set_name(module_name)` のチャンク名）・`pasta_scripts/` の標準ランタイム・投げ直しの接頭辞を除き、ゴーストのフォルダーの中の `.lua` だけを、ゴーストのフォルダーから見た形で残す。LuaJIT が長いチャンク名の先頭を `...` に縮めることも考える。
  4. `.pasta` の行への読み替えはしない（`debug/source_map` はデバッガを有効にしたときだけ作るため）。

---

# 設計フェーズの調査（2026-10-17・main `29dcdb56`）

## Summary（設計）
- **Discovery Scope**: Extension（軽い調査。既存の 500 の出口に `pasta/failure.lua` をつなぐ。外部ライブラリは足さない）
- **使ったスキル・規則**: `kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、`kiro-validate-design`（`design-review.md`）、`ponytail`（足さない・既存の部品を使う）。スキルの方針として、新しい抽象は 1 つ（止まるエラーの組み立て）に絞り、表記・エスケープ・切り替えは `FAILURE` をそのまま使った。
- **Key Findings**:
  1. **pasta のランタイムには Lua の `debug` ライブラリが無い**（`RuntimeConfig` の既定は `std_all` = `StdLib::ALL_SAFE` で、`std_debug` を除く。`debug.getinfo` を呼ぶと `attempt to index global 'debug' (a nil value)`）。要件ディスカッションの申し送り 2「止まったコルーチンの呼び出し履歴を `debug.getinfo(co, …)` で調べる」は、そのままでは使えない。
  2. **登録名からは作者が書いた名前に戻せない**。グローバルシーンの登録名は「別名を置き換えた名前 → `sanitize_name`（英数字と `_` 以外を `_` にする）→ `_連番`」で作る（`transpiler.rs` `process_global_scene`・`scope_gen.rs`）。`＊会話`（既定の別名）は `OnTalk_1`、`＊OnTalk＿新しいおもちゃ` は `OnTalk_新しいおもちゃ_1` になる。連番は落とせるが、別名と記号は戻せない。実行時のどこにも書いた名前は残っていない（`@pasta_search` の表も Lua のシーンの表から作り直す）。
  3. **書いた名前を運べる場所は、生成コードの `act:init_scene(SCENE)` だけ**。どのシーン関数（`__start__`・名前付きのローカルシーン）も最初にこれを呼ぶ。Lua のブロックが定義した関数（`function SCENE.次(act)`）は呼ばないので、ブロックの関数の中のエラーは、それを呼んだシーンの名前になる（要件 2.2 の例「ローカルシーン `・雨の日` の Lua のブロックで止まった → `＊天気・雨の日`」と合う）。
  4. **エラー文の場所は、LuaJIT が 60 バイトに縮める**。手元の実験で、応答の理由の行は次の形だった（一時ゴーストのパスは長い）。
     - 辞書の Lua のブロック: `.../profile/pasta/pasta_scripts/pasta\shiori\event/init.lua:229: ...d/exp/ghost/profile/pasta/cache/lua/pasta\scene\test.lua:30: attempt to index local 'data' (a nil value)`（投げ直しの場所が先頭に付く。研究ログの見込みどおり）
     - `scripts/` の Lua（REG のハンドラから呼んだ）: `...0d1-71b684a71d92/scratchpad/exp/ghost/scripts/天気.lua:4: attempt to index local 'data' (a nil value)`
     - エラーの値が表: `Unknown error`
     - チャンク名は `@` ＋ `package.path` の型に `?`（モジュール名の `.` を `\` にしたもの）を入れた文字列。区切りは `/` と `\` が混ざる。縮めた形は `...` ＋ 末尾 56 バイトで、バイトで切るので、先頭で日本語の文字が割れることがある。
  5. **`package.searchpath` は日本語のファイル名を引けない**（`package.searchpath("天気", package.path)` が `nil`。LuaJIT の `fopen` が UTF-8 のパスを開けない。pasta の探索関数は Rust の `std::fs` で開くので読み込める）。縮めたチャンク名から元のパスを引き直すのに、ファイルを開く方法は使えない。
  6. **チェイントークで再開したシーンの act は、`EVENT.fire` が作った act ではない**。コルーチンは最初の再開で受けた act に台詞を積み続ける。`EVENT.drive(co, act, …)` の `act` は、チェイントークの再開では新しい act になる。コルーチンに対応する act は `FAILURE.bind` の表（`RUNNING[co]`、`init_scene` が書く）にだけある。

## Research Log（設計）

### 手元の実験（スクラッチの一時ゴースト）
- **Context**: 申し送りの Research Needed 2（シーンの中のエラーの実際の文・LuaJIT のチャンク名の縮め方・辞書の Lua のブロックの場所の形）と、呼び出し履歴を調べられるか。
- **方法**: スクラッチのクレートから `pasta_shiori` の `PastaShiori::load`・`request` を呼び、`dic/test.pasta`（Lua のブロックで `nil` を添字にする）・`scripts/天気.lua`・`scripts/main.lua`（REG のハンドラと、`EVENT.drive` を包んでエラーの値を書き出す）を持つ一時ゴーストで、4 つのイベントを起こした。リポジトリには何も置いていない。
- **Findings**: Key Findings 1・4・5 のとおり。加えて、エラーで終わった後の act の `current_scene.__global_name__` は `OnErrBlock_1`、別名の `＊会話` は `OnTalk_1` だった。
- **Implications**: 呼び出し履歴を調べる案（申し送り 2）は捨てる。名前は生成コードから運び、場所はエラー文の文字列とゴーストのフォルダーの文字列だけで見分ける。

### 書いた名前の運び方
- **Context**: 要件 2.4・2.7（作者が書いた名前で示す・同じ名前は区別しない）。
- **Alternatives**:
  1. 生成コードの `act:init_scene(SCENE)` に、書いたグローバルシーン名とローカルシーン名を足す（`act:init_scene(SCENE, "天気", "雨の日")`）。`init_scene` が act に控え、途中の Call（`call_restore`）が戻った後に呼ぶ前の名前へ戻す。
  2. Rust に、止まったコルーチンの呼び出し履歴を読む小さな関数を足す（`mlua::Thread::state()` と `lua_getstack`）。関数がどのシーンかは分かるが、書いた名前（別名・記号）は結局生成コードから運ぶ必要があり、1 の変更も要る。unsafe の FFI と、作者のスクリプトから呼べる新しい口が増える。
  3. 登録名から近い名前を作る（連番を落とす）。`＊会話` が `＊OnTalk` と出て、要件 2.4 を満たさない。
- **Selected**: 1。生成コードの行数は変わらない（同じ行に引数が増えるだけ）ので、デバッガのソースの対応表は影響を受けない。実行時の重さは、シーン関数の入口ごとに文字列の定数 2 つを渡して act のフィールドに書くだけ。
- **Trade-offs**: トランスパイラ（`pasta_lua` の Rust のコード生成）と `act.lua` を触る。brief の Out of Boundary は `act.lua` を「触らない」としていた（`failure-output-unification` との分担のための記述）。設計ディスカッションで確かめる（OPEN QUESTION）。生成コードの期待値（`sample.expected.lua` と、`init_scene(SCENE)` の文字列を確かめる 2 つのテスト）を直す。
- **Follow-up**: Lua の関数のハンドラ（`＠関数（）` の `word`・`expr_fn`・アクタープロキシ）の中で別のシーンへ入って戻った後は、名前が呼んだ先のままになる（`restore_scene` は名前を戻さない）。止まった場所の名前を取り違えるのは、その後の呼び出し元の行で止まったときだけ。

### エラー文の場所の見分け方
- **Context**: 要件 2.5・2.6。
- **Findings**: 位置の接頭辞 `チャンク:行: ` は、エラー文の先頭に 0 個以上並ぶ（外側の投げ直しが先、元の場所が後）。チャンクの文字列は 3 通りある。
  - `[string "…"]`（Rust が `@` を付けずに名前を付けたチャンク: 旧経路の生成コード・作者が上書きした `entry.lua`・`pasta.scene_dic`）
  - 縮めていないパス（60 バイト未満）
  - `...` ＋ 末尾 56 バイト
- **Selected**: ゴーストのフォルダー（`SHIORI.load` の `load_dir`。Rust の `PastaLoader` が `base_dir` に使う文字列と同じ）を `\`→`/` にそろえて控える。接頭辞ごとに、
  - 縮めていなければ、パスが「ゴーストのフォルダー ＋ `/`」で始まるかを見る。
  - 縮めていれば、末尾の文字列を「ゴーストのフォルダーの末尾 ＋ `/` ＋ 残り」に分けられる最初の位置を探す（ゴーストのフォルダーの最後の数バイトと、`/` が続く位置）。
  - 残り（ゴーストのフォルダーから見たパス）が `profile/` で始まらなければ、作者の `.lua` として `scripts\天気.lua:12` の形（区切りを `\` にする）で残す。それ以外（標準ランタイム・生成コードのキャッシュ・`[string …]`・ゴーストの外・見分けられない）は落とす。
- **Rationale**: ファイルを開かず、`debug` も要らない、文字列だけの純粋な関数になる。テストで表を回せる。
- **Ceiling**: ゴーストのフォルダーから見たパスが約 55 バイトより長いと（`scripts/` の下の日本語 16 文字ほどの名前）、縮めた末尾にフォルダーの境目が残らず、場所を落とす（エラー文は出す）。`dofile` などで `package.path` の外から読んだファイルも、ゴーストの中なら残り、外なら落ちる。

### 途中までの台詞の取り方
- **Selected**: `EVENT.drive` の失敗の枝で、止まったコルーチンに対応する act（`FAILURE.act_of(co)`。`FAILURE` に 1 つ足す読み取り関数）とエラーの値を控える（1 要求に 1 つの控え）。出口で表記を act の末尾に積み、`act:build()` で組み立てる。表記の位置と話者の決まりは、`failure-output-unification` の act を通じた報告（`FAILURE.report`）と同じになる。
- **Rationale**: `EVENT.drive` は、通常のイベント・コールバックの再開・時間切れの掃引の 3 つが通る 1 か所なので、ここに 1 行足せば全部の経路が控えを作る（一般化）。エラーの値は変えずに投げ直す（`error(yielded_value)` は今のまま）ので、応答の理由の行は 1 バイトも変わらない（要件 5.1）。エラーの値を表に包む案は、`error_handler` の「文字列でなければ Unknown error」と、理由の行の中身を変えるので採らない。

### SSP の実機の確かめ（未実施）
- 指示により、設計では SSP を起動していない。最初の実装タスクで確かめる: (1) 500 ＋ `Value` の台詞がバルーンに出るか、(2) 200 ＋ `Value` ＋ `X-Error-Reason` が普通に出るか、(3) `OnSecondChange` の応答にエラーの表記を返したときの見え方（トーク中に毎秒来るとき）。結果をこの文書に書き足し、応答のステータスを 1 つに決める（要件 3.2）。

## Design Synthesis（設計）
- **一般化**: 止まるエラーの出口は 3 つ（`SHIORI.request`・理由付きの時間切れ・理由の無い時間切れ）あるが、どれも「控え（act とエラーの値）または要求の ID → 表記 → 応答」の同じ手順になる。1 つの関数（`STOP.response`）に入力の違い（種類・対象の代わり・ログの文）だけを渡す形にした。シーンの失敗の控えは、3 つの再開の経路が通る `EVENT.drive` の 1 か所で作る。
- **作るか・使うか**: 表記・エスケープ・切り替え・ログの警告・act への積み方は `FAILURE`（`failure-output-unification`）を使う。応答は `RES.err`・`RES.ok`（どちらも中を変えない）。新しく作るのは、場所の見分けと名前の組み立てと控えだけ。
- **簡素化**: 
  - 呼び出し履歴を調べる仕組み（Rust の関数）は作らない。
  - 応答のステータスを選ぶ設定は作らない。実機の結果で 1 つに決めて定数にする。
  - `error_handler` は変えない（シーンの外のエラーの文は、今の `result`（最初の行）から作れる）。
  - 書いた名前を別の表に登録する仕組みは作らない（`init_scene` の引数で足りる。シーンの表に文字列を置くと、Rust の `collect_scenes` がローカルシーンとして拾ってしまう）。
  - リソースの要求の判定は ID の先頭 2 文字だけ。NOTIFY は Rust の側が応答を捨てるので判定しない。

## Risks（設計）
- SSP が 500 の `Value` を出さず、200 にした場合 — 失敗の応答であることは、理由の行（`X-Error-Reason`）だけで示す。実機の結果を research.md に残す。
- 理由の無い時間切れの後のエラーが応答の候補になる — その回の `OnSecondChange` は掃引の応答を返すので、その秒の仮想イベント（`OnTalk`・`OnHour`）は次の秒へ回る（理由付きの時間切れの今の動きと同じ）。表記を出さない設定では今のまま。
- トランスパイラの出力が変わる — 古いキャッシュ（書いた名前の無い `init_scene(SCENE)`）では、登録名から連番を落とした名前で示す（取り違えではなく、別名・記号が戻らないだけ）。キャッシュは版が変わると作り直される。

## SSP の実機の確かめ（タスク 1・2026-10-17）
- 要件 3.2 は「設計より前の最初の作業」としていたが、設計で実装の最初のタスク（タスク 1）に移した。設計の時点では SSP を起動していない。
- 環境: SSP 2.9.18（手元の版。要件の 2.8.98 より新しい）。スクラッチに一時ゴースト（`scripts/main.lua` の REG のハンドラが応答の全文を返す）を置き、`--ghost` で起動した。リポジトリには何も置いていない。`pasta.dll` は手元でビルド済みの i686 の release 版を写した（応答は Lua の文字列を素通しするので、本仕様の変更の有無は結果に関係しない）。
- 確かめた形と見え方:
  - 500 ＋ `Value`（`OnProbe500`、SSTP の NOTIFY から起こす）: SSP は `NG:SHIORI returned status 500` を返し、台詞を再生しない。バルーンにも出ず、スクリプトのログにも残らない。SSP のエラーのログにも何も出ない。
  - 200 ＋ `Value` ＋ `X-Error-Reason`（`OnProbe200`）: 普通に再生され、バルーンに出た。`X-Error-Reason` の行は何も起こさない（エラーのログ・通知領域の反応なし）。
  - `OnSecondChange`（SSP 自身が起こすイベント）に 500 ＋ `Value`: 再生されない（ログにも残らない）。200 ＋ `Value` ＋ `X-Error-Reason`: 再生された。
- **決定**: 応答のステータスは **200**（`STOP.STATUS = "200 OK"`）。500 の値は表示されないため（決定 Q1 の「表示されないなら 200」）。失敗の応答であることは `X-Error-Reason` の行だけで示す。`ErrorLevel` は付けない。

# 実装の記録（2026-10-17）

## 行の対応の形（タスク 2.2）
- 生成コードの `init_scene` の 4 つ目の引数は `"<.pasta のファイル>|<lua の行>:<pasta の行>,..."` の 1 行の文字列（圧縮なし）。`__start__` は 3 つ目に `nil` を置く。ファイルに `\` があると `StringLiteralizer` が `[[...]]` の長い文字列にする（1 行のまま）。読み解くときは最後の `|` で切る。
- ファイルはローダーがゴーストのフォルダーから見た形（`dic\天気.pasta`）にする。メモリ上でトランスパイルする場合（テストなど）は、パースに渡した名前のまま。
- **設計からのずれ**: 設計は「そのシーン関数の行」の対応としていたが、Lua のブロックはシーン関数の外（グローバルシーンの `do` の末尾）に出るので、関数の行だけでは要件 2.2・2.5 の例（Lua のブロックで止まる）を引けない。各シーンの対応に、同じグローバルシーンのすべての Lua のブロックの行も入れた（ブロックの行は、そのグローバルシーンのどのシーンの文字列にも載る）。別のグローバルシーン・別のファイルから呼んだブロックの関数は対応に無く、場所を示さない。
- 記録はデバッガと同じ（`LineRecorder` がトランスパイルの間だけ小さな表を持ち、デバッガのシンクがあればそのまま流す）。デバッガの `SourceMap` は今までどおりデバッガを有効にしたときだけ作る。

## 読み込んだ Lua のファイルの表（タスク 2.4）
- **名前**: `package.loaded["@pasta_chunks"]`。Lua からは `require("@pasta_chunks")` で読む（`@pasta_config` などと同じ、Rust が登録する表）。`setup_package_path` が探索関数の設置の直後に空の表として作り、探索関数（`searcher.rs` の `find_module`）がファイルを読み込めたときに書き足す。ローダーを通さない VM（`install_module_searcher` だけの VM）では作らず、控えない。
- **キー**: エラー文の位置の接頭辞に出るチャンクの形（`@` を除いた候補パス）。LuaJIT の `lj_debug_shortname` のとおり、60 バイト**以上**なら `...` ＋ 末尾 56 バイト、59 バイトまではそのまま。Rust のテストで、59・60・61 バイトと日本語の文字が割れる名前のチャンクのエラー文と、バイト単位で一致することを確かめた。キーはバイト列なので、先頭で UTF-8 の文字が割れていてもよい（Lua の文字列として表に入る）。
- **値**:
  - 作者の `.lua`: ゴーストのフォルダーから見たパスの文字列。区切りは `\`（例 `scripts\天気.lua`）。表記にそのまま使える形にするのは探索関数の責任（表記の側はゴーストのフォルダーを控えない）。
  - 変換した Lua のキャッシュ（`[loader] transpiled_output_dir`、既定 `profile/pasta/cache/lua` の下。`pasta.scene_dic` も含む）: `true`（辞書の印。行は act が控えた行の対応で引く）。
  - 2 つの違うファイルが同じキーになった: `false`（見分けられない。表記の側は無いキーと同じに扱う）。一度 `false` になったキーは、どちらのファイルを読み直しても `false` のまま。同じファイルを読み直しても重複にはしない。
- **控えないもの**: 標準ランタイム（`profile/pasta/pasta_scripts/` の下）と、ゴーストのフォルダーの外のファイル。ただし重複の見分けには数える（標準ランタイムと同じキーになった作者の `.lua` は `false` になり、標準ランタイムの中のエラーを作者のファイルと取り違えない）。`[string "…"]`・`=…` の名前のチャンクは探索関数を通らないので表に無い。
- **見分け方**: 候補パスの `\` を `/` にそろえ、ゴーストのフォルダー（`LoaderContext.base_dir` を同じくそろえた形）＋ `/` で始まるかを見る。`package.path` はローダーが同じ `base_dir` から作るので、文字列の前方一致で足りる（ファイルは開かない）。`profile/pasta/save/lua`・`scriptlibs` など、キャッシュと標準ランタイム以外のゴーストの中の `.lua` は作者の `.lua` として控える。
- **チャンク名**は変えていない（`@` ＋ 候補パスのまま。既存のテストと、新しいテストで `debug.getinfo` の `source` を確かめた）。
- キャッシュの場所を渡すため、`LoaderContext` に `transpiled_output_dir` を足した（`from_config` は設定の値、`new` は既定値）。

## 場所の見分けとエラー文の整えの引数（タスク 3.2）
- **設計からのずれ**: 設計は `STOP.location(chunk, ghost_dir)`・`STOP.detail(err, ghost_dir)` としていたが、`STOP.location(chunk, line, chunks, line_map)`・`STOP.detail(err, chunks, line_map)` にした。
  - `chunks` は 2.4 の読み込んだ Lua のファイルの表（省略すると `package.loaded["@pasta_chunks"]`。ローダーを通さない VM では無く、そのときは場所を示さない）。ゴーストから見たパスは表の値が持つので、ゴーストのフォルダーの引数と控え（`STOP.set_ghost_dir`）は要らない。
  - `line_map` は act が控えた行の対応（2.3 の `act.line_map`）。変換した Lua のキャッシュの場所を `.pasta` の行に直すのに要る。`line` は接頭辞の行番号（キャッシュでは対応を引くキー、作者の `.lua` ではそのまま出す）。
  - `STOP.location` は行まで付けた `scripts\天気.lua:12`・`dic\天気.pasta:15` の形を返す（設計は場所だけを返し `detail` が行を付ける形だった）。
- 表で引くので、設計の Ceiling（ゴーストから見たパスが約 55 バイトより長いと場所を落とす）は無くなった。末尾 56 バイトが同じファイルが 2 つあるときだけ（表の値が `false`）場所を落とす。
- 位置の接頭辞として扱うのは、チャンクが表にある（値が `false` でも）か、`.lua`・`]` で終わるものだけ。行の対応は止まったときに初めて読み解く（最後の `|` で切り、`<lua>:<pasta>` を順に見る）。

## 警告の中身（タスク 3.3）
- `failure.lua` に `FAILURE.body(label, subject, detail)`（【】の枠なし・さくらスクリプトの文字の無効化なし）を足し、`FAILURE.format` を「`FAILURE.body` を無効化して枠を付けたもの」に書き直した。設計の Out of Boundary（`FAILURE` に足すのは `LABEL` の 2 行と `FAILURE.act_of` だけ・`FAILURE.format` は呼ぶだけ）から外れる。
- 理由: 要件 5.6 は、ログの警告を「表記と同じ中身から枠と無効化を除いたもの」にするよう求める。`FAILURE.notation` の `warning` は出来上がった文字列しか受けず、`FAILURE.format` の出力から無効化は戻せない。形の決まり（`種類：対象（説明）` と対象が無いときの扱い）を `stop.lua` に写すと、表記の形が 2 か所に分かれる。
- `FAILURE.format` の出力は 1 バイトも変わらず、既存の `failure_test` はすべて通る。後の spec が `FAILURE.format` の形を変えるときは、`FAILURE.body` も見ること（Revalidation Triggers の「`FAILURE.format` の形が変わる」に含める）。
- 応答のステータスは `STOP.STATUS = "200 OK"`。応答は `RES.ok(値, { ["X-Error-Reason"] = 理由 })`（理由が無い・空なら `"Unknown error"`）。登録名から作る名前（古いキャッシュ）にも `＊` を付ける（`天気_3` → `＊天気`）。
