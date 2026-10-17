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
