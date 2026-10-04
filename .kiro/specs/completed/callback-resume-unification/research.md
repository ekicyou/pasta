# ギャップ分析: callback-resume-unification

## Summary
- **Feature**: `callback-resume-unification`
- **Discovery Scope**: Extension（既存のイベント配送・コールバック管理の修正。新しい外部依存なし）。設計フェーズでは light discovery を行い、結果を「設計フェーズの追加調査」と Design Decisions に追記した
- **Key Findings**:
  - 通常イベントの再開経路（`resume_until_valid` → `consume_staged` → `set_co_scene`）は `event/init.lua` のローカル関数として完結しており、`callback.lua` の `try_route`・`sweep` はそのどれも通らない。再開経路を共有できる形にすることが修正の中心。
  - 4 つの不具合のうち、タイムアウトの二重包みと U23 は同じ 1 箇所（`EVENT.fire` の「文字列は常に `RES.ok`」分岐）に根がある。U23 の方式（素通し or 設計どおり）を決めると、タイムアウト応答の直し方もほぼ決まる。
  - 二重包みを「既知の挙動」として 200 で固定している既存テストが `crates/pasta_shiori/tests/async_callback_chain_test.rs` にあり、`REG.OnSecondChange` を直接呼んで戻り値の文字列を見るテスト（`shiori_entry_test.lua`）・`sweep` の戻り値を直接見るテスト（`callback_module_test.lua`）も契約変更の影響を受ける。
  - `get_property`（`pasta/shiori/act.lua`）は本仕様の編集範囲外（API スコープ外かつ並走条件の `event/` 外）。タイムアウトのエラーは `error(reason)`（レベル 1）で投げられるため、エラー文字列にはソース位置の前置きが付く。`X-Error-Reason` を `timeout_message` そのものに保つには、`event/` 側で理由を持つ必要がある。

## Research Log

### 現行の再開経路（`event/init.lua`）
- **Context**: Requirement 1・2 の「通常イベントと同じ規則」が何を指すかの確認。
- **Sources Consulted**: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`（`set_co_scene` 87–123 行、`resume_until_valid` 130–154 行、`EVENT.fire` 173–209 行）
- **Findings**:
  - `resume_until_valid(co, ...)` は初回 resume に可変長引数を渡せる。コールバックの初回引数（`refs`、または `nil, reason`）もそのまま渡せる形になっている。
  - `EVENT.fire` の thread 分岐は「resume → エラーなら `set_co_scene` して `error` → `consume_staged(co, act)` → `set_co_scene(co)` → `RES.ok(value)`」。この一連がコールバック経路に無い。
  - `set_co_scene` は `STORE.co_callback` と一致したコルーチンを `co_scene` に入れず、既存の `co_scene` を破棄して `nil` にする（`get_property` で待ちに入った時点で、それまでの継続は消える）。
  - `EVENT._resume_until_valid` はテスト用に公開済み。`set_co_scene` は非公開。
- **Implications**: 再開〜継続保存の一連を、`EVENT.fire` から切り出して `callback.lua` からも使える形にするか、`try_route` の結果を `EVENT.fire` 側で駆動する形にするかが設計の分かれ目（後述のオプション）。

### `try_route` の欠落（Requirement 1）
- **Sources Consulted**: `event/callback.lua` 71–104 行、`book/src/internals/execution-model.md`「コールバック待ちとの関係」
- **Findings**:
  - resume は 1 回だけ。出力の無い中断（`build()` が `nil`）なら `RES.ok(nil)` = 204 を返し、コルーチンはどこにも保存されない。
  - suspended で `stage_pending` していなければ（チェイントーク）、`consume_staged` は false を返し、`set_co_scene` も呼ばれないため継続が失われる。
  - 再予約した場合は `consume_staged` が `STORE.co_callback = co` を立てるが、`set_co_scene` を通らないため印が残る（マニュアル `execution-model.md` 104 行付近が現行挙動として明記）。一本化すれば印は `set_co_scene` で消える。
  - エラー時は `error(yielded)` で `SHIORI.request` の xpcall に任せ 500。この点は現行のままで Requirement 1.5 を満たす（継続にもならない）。
  - 再開に渡す ACT は登録時の `entry.act`（`consume_staged` の第 2 引数）であり、`EVENT.fire` が作る新しい ACT ではない。一本化時もこの区別を保つ必要がある。
- **Implications**: Requirement 1.1–1.5 は「`resume_until_valid(co, refs)` → `consume_staged(co, entry.act)` → `set_co_scene(co)` → 応答」の順で満たせる。Requirement 1.7・1.8（既存の継続との関係、未決 Q1）は `set_co_scene` の既存規則をそのまま使えば「最後に中断したものが勝つ・終われば空にする」になる。

### `sweep` の欠落（Requirement 2・3）
- **Sources Consulted**: `event/callback.lua` 109–131 行、`event/second_change.lua`、`review-improvement-loop/matrix.md` 3.49、`pasta-runtime-internals-doc/absorption-ledger.md` 付録 B（300–303 行）
- **Findings**:
  - `pairs(CALLBACK.pending)` の走査中に `coroutine.resume` しており、再開したシーンが `get_property` → `consume_staged` すると走査中の表へ挿入することになる（Lua の `next` 仕様で未定義動作）。3.49 はこれを理由に修正を見送った。
  - 戻り値（エラー・yield 値）を捨てる。予約（`_staged`）も消費しないため、次のコルーチンを返す `EVENT.fire` で「multiple staging」エラー、または別シーンの古いイベント名での誤登録になる。
  - 理由付き（既定）経路では、`get_property` が `error(reason)` を投げてシーンは dead になる。現行の 500 応答は `RES.err(entry.on_timeout)`（捕捉したエラー文字列ではない）で作っている。
  - 静かな経路（`on_timeout` が文字列でない）は、`get_property` に `timeout_message = false` などを渡したときだけ生じる（既定値は文字列）。マニュアル `lua/script-api.md` は `timeout_message` を string としてのみ記載。
  - 複数期限切れ時の順序は `pairs` 依存で非決定。応答には「最初に見つかった文字列 on_timeout」を使う。
  - `second_change.lua` は `sweep` の応答文字列を `REG.OnSecondChange` の戻り値としてそのまま返し、`EVENT.fire` が `RES.ok` で包む（二重包み）。応答があれば `dispatcher.dispatch` は呼ばない。
- **Implications**: 「期限切れを集める → `pending` から外す → 走査の外で再開」の 2 段にすれば挿入問題は消える（brief の方針どおり）。再開ごとに `consume_staged`・`set_co_scene` を通せば予約の取り残しも消える。順序の決定性（Requirement 2.7 の仮定）は番号の昇順ソートで足りる。掃引の出力をどう応答にするか（未決 Q2）が `second_change.lua` と `EVENT.fire` の間の受け渡し形を決める。

### 応答の二重包み（Requirement 3・4、U23）
- **Sources Consulted**: `event/init.lua` 202–204 行、`pasta/shiori/res.lua`（`RES.ok`・`RES.err`・`RES.warn`・`RES.not_enough`・`RES.advice`）、`manual-ssot-authority/absorption-ledger.md` U23、`entry.lua` `error_handler`
- **Findings**:
  - すべての文字列戻り値を `RES.ok` で包む。`RES.build` が作る応答は必ず `"SHIORI/3.0 "` で始まるため、接頭辞判定で素通しは 1 行で書ける。
  - `RES.warn`（204 理由付き）・`RES.not_enough`（311）・`RES.advice`（312）は既に `res.lua` に存在し、マニュアルの RES 表にも載っている。素通し案なら新しい API は不要。
  - 設計どおり案（素通ししない）を取る場合、タイムアウト 500 は「掃引が `error(reason)` を投げ、`SHIORI.request` の `error_handler`（先頭行のみ）→ `RES.err`」で作ることになる。`error` をレベル 0 で投げれば前置きは付かない。ただしこの案では、同じ回に他の期限切れ待機を処理し終える前に例外で抜けるため、Requirement 2.1・2.9 と衝突しやすい（全件処理後に投げる順序の工夫が要る）。
  - 素通し案の副作用: `SHIORI/` で始まるトーク文字列を返すハンドラは誤って素通しされる（実用上はさくらスクリプトが `\` で始まるため稀）。
- **Implications**: 素通し案は変更が最小で、タイムアウトの二重包みも同時に消える。設計どおり案はテスト修正が多く、掃引の例外の扱いに注意が要る。

### テスト資産
- **Sources Consulted**: `crates/pasta_lua/tests/lua_specs/`（`callback_module_test.lua`・`shiori_entry_test.lua`・`event_coroutine_test.lua`・`get_property_test.lua`・`second_change_thread_test.lua`）、`crates/pasta_lua/tests/shiori/`（`event_dispatch_test.rs`・`event_handler_test.rs`）、`crates/pasta_shiori/tests/`（`async_callback_chain_test.rs`・`async_callback_simple_test.rs`、フィクスチャ `fixtures/async_callback/`）
- **Findings**:
  - `callback_module_test.lua` は `try_route`・`sweep` を直接呼び、戻り値の応答文字列（`sweep` の 500、`try_route` の 200）を検査している（`sweep` 関連 10 箇所）。`try_route`・`sweep` の戻り値の契約を変えると、これらの随伴更新が必要。
  - `shiori_entry_test.lua` 238 行以降は `REG.OnSecondChange(act)` を直接呼んで、戻り値の文字列に理由が含まれることを `find` で検査している（`EVENT.fire` を通していないため二重包みを見逃す。台帳付録 B 302 行の指摘どおり）。
  - `event_dispatch_test.rs`（120・296・337・341・345・435・472 行）・`event_handler_test.rs`（212・251・295・337・380・423・470 行）は `return RES.ok(…)` / `return RES.no_content()` を返し、応答を部分一致で検査している。素通し案ならそのまま正しくなる（検査を厳密化する余地はある）。設計どおり案なら全件書き換え。
  - `async_callback_chain_test.rs` 207–212 行は「sweep は 500 を返すが RES.ok で 200 になる（二重ラップは既知の設計上の挙動）」として 200 を固定している。修正後は 500 を期待するよう改める必要がある（未決 Q4: `brief.md` の挙げるテスト場所の外）。
  - `async_callback_chain_test.rs` のシナリオ 3 は「チェイントーク → get_property」の順であり、逆順（get_property → チェイントーク、Requirement 1.3）の E2E は無い。`async_callback_support.rs` は本番の `pasta_scripts/` をコピーして使うため、新しい E2E シナリオはフィクスチャの `entry.lua` にハンドラを足すだけで書ける。
  - `async_callback_simple_test.rs` シナリオ 4（待機中の無関係イベント）は Requirement 5.4 の既存被覆。
- **Implications**: Lua 側（`lua_specs`）で `EVENT.fire` を通す回帰テストを足し、Rust E2E（`pasta_shiori`）でチェイントーク継続とタイムアウト 500 を固定するのが最小で確実。

### マニュアルと生成物
- **Sources Consulted**: `book/src/lua/shiori-events.md`（37–48・172・389–395 行付近）、`book/src/internals/execution-model.md`（96–104・146–156・200–245 行付近）、`book/src/internals/shiori.md`（268–294 行付近）、`book/src/internals/internal-modules.md` 37 行、`book/tools/gen-skill-refs.mjs`
- **Findings**:
  - `shiori-events.md` の戻り値表と「`REG` のハンドラは応答全体ではなく `Value` にする文字列を返す」（172 行付近）は U23 の決定に連動。OnPastaCallBack 節（389 行付近）は「出力を `Value` にして 200 OK」「上限超過はタイムアウトとして処理」までしか書いておらず、継続とタイムアウト応答の形が無い。
  - `execution-model.md` は「コルーチンの持ち主」表、`EVENT.fire` の流れ図、「このループは `EVENT.fire` だけが使う。コールバックの再開は resume を 1 回だけ行う」、`co_callback` の印が残る記述、`set_co_scene` 規則、コールバック待ちの流れ図を持ち、いずれも書き換え対象。
  - `shiori.md` 271 行（「文字列なら `RES.ok` で包み」）・279 行（OnSecondChange の既定ハンドラ）も対象。
  - スキル references へ生成される章は `lua/shiori-events.md` と `internals/internal-modules.md`（`pasta-lua-coding`）。`execution-model.md`・`shiori.md` は生成対象外。
  - ソース内の doc コメント（`init.lua` 37 行・`register.lua` 25 行の「EVENT.fire が RES.ok で包む」、`second_change.lua` の戻り値説明、`callback.lua` の各関数説明）も同期対象。
- **Implications**: マニュアル更新は 3〜4 章に及ぶ。生成 references の再生成（`gen-skill-refs.mjs`）とリンク検証を DoD に含める必要がある。

## Requirement-to-Asset Map

| 要件 | 既存資産 | ギャップ | 種別 |
| ---- | -------- | -------- | ---- |
| 1.1 一致したコールバックの再開・応答 | `try_route` | 応答は返すが単発 resume | Constraint（既存の振る舞いの一部を保つ） |
| 1.2 出力の無い中断の読み飛ばし | `resume_until_valid`（init.lua） | `try_route` が使わない | Missing |
| 1.3 チェイントークの継続保存 | `set_co_scene`（init.lua ローカル） | `try_route` から呼べない | Missing |
| 1.4 再予約 | `consume_staged` | 呼ばれているが `set_co_scene` 不通過で `co_callback` 印が残る | Missing（部分） |
| 1.5 エラー時の 500・残さない | `error(yielded)` + xpcall | 現行で充足（`pending` は再開前に削除済み） | — |
| 1.6 不一致は通常イベント | `try_route` が `nil` | 現行で充足 | — |
| 1.7・1.8 既存の継続との関係 | `set_co_scene` 規則 | 決定待ち（Q1） | Unknown |
| 2.1・2.3 全件 1 回・走査中挿入なし | `sweep` の `to_remove` | 再開が走査中 | Missing |
| 2.2・2.4・2.5 予約・継続の扱い | `consume_staged`・`set_co_scene` | `sweep` が通らない | Missing |
| 2.6・2.7 掃引の出力・順序 | `second_change.lua` | 決定待ち（Q2）。順序は `pairs` で非決定 | Unknown |
| 2.9 エラーでも残りを続ける | — | 現行は resume の失敗を無視して続行（結果は捨てる） | Missing（結果の扱い） |
| 3.1・3.2 タイムアウト 500 | `RES.err(entry.on_timeout)` | `EVENT.fire` が `RES.ok` で包む | Missing |
| 3.1 理由の形 | `get_property` の `error(reason)` | エラー文字列には位置の前置き。`act.lua` は編集範囲外 | Constraint（Q6） |
| 3.3 警告ログ | `log.warn(event_id .. ": " .. reason)` | 現行で充足 | — |
| 3.5 静かなタイムアウト | `sweep` の else 分岐 | 結果を捨てる | Missing |
| 4.1–4.3 文字列・nil・thread | `EVENT.fire` | 現行で充足 | — |
| 4.4・4.5・4.6 応答全体の素通し | `RES.*` | 方式決定待ち（Q3） | Unknown |
| 5.x 既存挙動の維持 | 既存テスト群 | 回帰の網はある | Constraint |
| 6.1–6.3 回帰テスト | `lua_specs`・`tests/shiori`・`pasta_shiori/tests` | `EVENT.fire` を通す検査が無い・逆順シナリオが無い | Missing |
| 6.4・6.5 既存テストの整理 | `event_dispatch_test.rs` ほか、`async_callback_chain_test.rs` | U23 決定と Q4 に依存 | Unknown |
| 7.x マニュアル・doc コメント | 4 章＋生成 references＋doc コメント | 旧記述が残る | Missing |

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: `EVENT.fire` が駆動（try_route は co を返す） | `try_route` は一致したエントリを `pending` から外して `(co, refs, act)` を返すだけにし、`EVENT.fire` の thread 分岐（初回引数だけ差し替え）で駆動する。`sweep` も期限切れのエントリ一覧を返し、駆動は `EVENT.fire` 側の共通ルーチンが行う | 再開〜継続保存が `init.lua` の 1 箇所に集まる（brief の推奨そのもの）。`set_co_scene` を公開しなくてよい | `sweep` は `REG.OnSecondChange`（利用者が上書き可能）の中から呼ばれるため、ハンドラの戻り値で「駆動してほしいコルーチン群」を `EVENT.fire` に渡す形が要る（戻り値の型が増える）。`try_route`・`sweep` の戻り値の契約が変わり、`callback_module_test.lua` の随伴更新が多い | 掃引の出力を応答にする（Q2 仮定）場合と相性が良い |
| B: 共通駆動ルーチンを公開し callback.lua から呼ぶ | `resume_until_valid`＋`consume_staged`＋`set_co_scene`（＋応答化）を 1 関数にまとめ、`init.lua` の公開関数（例: テスト用と同様の `EVENT._drive`）または `event/` 内の新しい小モジュールに置く。`try_route`・`sweep` はそれを呼ぶ | `try_route`・`sweep` の戻り値の契約（応答文字列 or nil）をほぼ保てる。`second_change.lua` の形も保てる | `callback.lua` → `init.lua` の逆向き依存（循環）になるため、遅延 `require` か新モジュールが要る。新モジュールにすると `set_co_scene` も移す必要がある | 変更の局所性が高い。`callback.lua` 冒頭の「このモジュールは act を require しない」方針と同様の循環回避の注記が要る |
| C: ハイブリッド | `try_route` は A（`EVENT.fire` が駆動）、`sweep` は B（共通ルーチンを呼んで応答文字列を返す） | 同期経路（コールバック）は brief の推奨どおり 1 本、掃引は既存の受け渡し形を保つ | 駆動ルーチンの所在が 2 箇所から参照される点は B と同じ | Q2 の決定次第で A/B のどちらに寄せるか決まる |

U23（Q3）の方式は上の 3 案と直交する。

| U23 案 | 変更 | 影響 |
| ------ | ---- | ---- |
| 素通し（推奨） | `EVENT.fire` の文字列分岐で `SHIORI/` 接頭辞なら包まない（1 行） | タイムアウトの二重包みが同時に消える。既存の `RES.ok` を返すテストはそのまま正しくなる。マニュアルの戻り値表に 1 行追加と、204 理由付き・311・312 の返し方の追記 |
| 設計どおり | `EVENT.fire` は変えない。テストの `RES.ok` を書き換え、掃引は `error(reason, 0)` を投げる（全件処理後） | テスト書き換えが十数件。ハンドラから 200 以外を返す手段は無いまま。掃引の例外順序に注意 |

## Design Decisions

### Decision: 再開経路の一本化の形（設計フェーズで決定）
- **Context**: Requirement 1・2。
- **Alternatives Considered**: 上表 A / B / C。B の置き場所として (B1) `init.lua` に置き `callback.lua` が呼び出し時に `require` する、(B2) `resume_until_valid`・`set_co_scene` ごと `callback.lua` へ移す、(B3) `event/` に新しい小モジュールを作る。
- **Selected Approach**: B1。`init.lua` に `EVENT.drive(co, act, ...)`（`resume_until_valid` → `consume_staged` → `set_co_scene`、戻り値は `ok, value`）を置き、`EVENT.fire`・`try_route`・`sweep` の 3 箇所が呼ぶ。`try_route`・`sweep` の戻り値の型（応答文字列または `nil`）は変えない。
- **Rationale**: 掃引の結果をハンドラの戻り値で運ぶ（次の Decision）ため、`callback.lua` から再開の手順へ届く経路はどのみち要る。そうなると `try_route` だけを A にする利点が無く、契約とテストが変わるだけになる。B2 は依存の向きがきれいだが、テスト群の読み直しと衝突する（下の「設計フェーズの追加調査」）。B3 は `drive` が `consume_staged` を呼び、`sweep` が `drive` を呼ぶため、モジュールを分けても循環は消えない。
- **Trade-offs**: `callback.lua` → `pasta.shiori.event` の逆向きの依存が、呼び出し時の `require` として 1 つ増える。同じ書き方は `EVENT.no_entry`・`virtual_dispatcher.lua`・`kick.lua` に既にある。
- **Follow-up**: `callback_module_test.lua` の読み直し補助関数に `pasta.shiori.event` を加える。

### Decision: 掃引の結果を `EVENT.fire` へ運ぶ形（設計フェーズで決定）
- **Context**: Requirement 2.6・2.7・3.1。`REG.OnSecondChange` は利用者が上書き・ラップできる。
- **Alternatives Considered**: (1) 掃引が SHIORI 応答の全文を作り、ハンドラの戻り値（文字列）で返す。(2) ハンドラが「駆動してほしい待機の一覧」という新しい型を返し、`EVENT.fire` が駆動する。(3) 掃引を既定ハンドラから `EVENT.fire` へ移す。
- **Selected Approach**: (1)。出力は `RES.ok`、理由付きタイムアウトは `RES.err(entry.on_timeout)`。`EVENT.fire` の `SHIORI/` 接頭辞の素通しで、その回の応答になる。
- **Rationale**: ハンドラの戻り値の型を増やさず、`second_change.lua` のコードも変わらない。既定ハンドラを変数に取って呼ぶ使い方がそのまま動く。(3) は「`REG.OnSecondChange` を上書きすると掃引が止まる」という、マニュアルに書かれた既存の規則を変える。
- **Trade-offs**: 掃引の中で応答の文字列まで作るため、`callback.lua` は `RES` に依存し続ける（現行と同じ）。

### Decision: `STORE.co_callback` の印（設計フェーズで決定）
- **Context**: 一本化の後、`consume_staged` と `set_co_scene` は必ず `EVENT.drive` の中で続けて呼ばれる。
- **Alternatives Considered**: 残す / 削除して `consume_staged` の戻り値で分岐する。
- **Selected Approach**: 残す。
- **Rationale**: 印の説明は `pasta/store.lua` のコメント（Wave 1 の編集範囲の外）にあり、削除すると食い違う記述が範囲外に残る。削除しても挙動は変わらない。一本化だけで「`try_route` の経路で印が残る」現象は消える。
- **Follow-up**: 範囲外のコメント修正を許すかどうかを設計ディスカッションで確認する。許すなら削除できる。

### Decision: U23 の方式（要件ディスカッションで決定）
- **Context**: Requirement 3・4、未決 Q3。
- **Alternatives Considered**: 素通し / 設計どおり。
- **Selected Approach**: 素通し（要件ディスカッション #1 で決定）。`EVENT.fire` の文字列分岐で `SHIORI/` 接頭辞なら包まない。シーンの出力には適用しない。
- **Follow-up**: マニュアルに「`SHIORI/` で始まる文字列は応答全体として扱う」規則を 1 行書く。

## Implementation Complexity & Risk

- **Effort: M（3–7 日）** — 修正自体は `event/` の 3 ファイルに収まるが、`try_route`・`sweep` の契約変更に伴う既存テストの随伴更新、Lua と Rust E2E の回帰テスト追加、マニュアル 3〜4 章と生成 references・doc コメントの同期が伴う。
- **Risk: Medium** — 既存パターン（`resume_until_valid`・`consume_staged`・`set_co_scene`）の再利用で済み新技術は無いが、継続（`co_scene`）とコールバック待ち（`pending`・`co_callback`）の状態遷移が絡み、OnSecondChange・OnTalk・キックとの相互作用を壊すと回帰が見えにくい。

## Risks & Mitigations
- 継続の置き換え規則（Q1）を変えると OnTalk のチェイントーク・キックの preempt と干渉する — `virtual_dispatcher_*`・`kick_*`・`global_chaintalk_*` の既存スイートを安全網にし、Requirement 5 の維持をテストで確認する。
- `sweep` を「集めてから再開」に変えると、再開中に登録された新しい待機の期限（`os.time() + timeout`、`timeout = 0` なら即時）の扱いが変わりうる — Requirement 2.3（同じ回では扱わない）を回帰テストで固定する。
- `X-Error-Reason` に位置の前置きが混ざる — 掃引側で `entry.on_timeout` を保持して応答に使う（`act.lua` は触らない）。Q6 で形を確定する。
- `LuaJIT 2.1` に `coroutine.close` が無い — 破棄は参照を外して GC に任せる既存方針を踏襲する（`set_co_scene` のコメントどおり）。
- 並走条件（Wave 1）: `pasta/act.lua`・`pasta/shiori/act.lua`・`choice_select.lua` を触らずに済むことを設計で確認する。`crates/pasta_shiori/tests/async_callback_chain_test.rs` の更新可否は Q4。

## 設計フェーズの追加調査（2026-10-04）

### テストのモジュール読み直しと `STORE` の取り違え
- **Context**: `set_co_scene` をどのモジュールに置くかの判断材料。
- **Sources Consulted**: `crates/pasta_lua/tests/lua_specs/` の `event_coroutine_test.lua`・`global_chaintalk_integration_test.lua`・`global_fallback_integration_test.lua`・`integration_coroutine_test.lua`・`callback_module_test.lua`・`get_property_test.lua` の読み直し補助関数。
- **Findings**:
  - `EVENT.fire` を検査するスイートは `pasta.store` と `pasta.shiori.event` を `package.loaded` から外すが、`pasta.shiori.event.callback` は外さない。`callback.lua` は古い `STORE` を持ち続ける（現行は印の読み書きだけなので害が無い）。
  - `callback_module_test.lua`・`get_property_test.lua` は逆に `callback` と `store` だけを外す。`get_property_test.lua` は `try_route`・`sweep` を呼ばない。
- **Implications**: `set_co_scene` を `callback.lua` へ移す（B2）と、`STORE.co_scene` の書き込み先が古い `STORE` になり、`EVENT.fire` を検査する複数のスイートが壊れる。`init.lua` に置いたまま（B1）なら、直すのは `callback_module_test.lua` の補助関数 1 箇所で済む。

### 編集範囲と `STORE.co_callback`
- **Findings**: `pasta/store.lua` 21 行・77–79 行のコメントが印の持ち主（`consume_staged` が書き、`set_co_scene` と `CALLBACK.reset` が戻す）を説明している。`store.lua` は `event/` の外。B1 かつ印を残す場合、このコメントは一本化の後も正しいままである。

### 掃引の応答の候補と順序
- **Findings**:
  - 理由付きの待機は `get_property` が `error(reason)` を投げるため、`EVENT.drive` は `ok=false` を返す。応答は保持していた `entry.on_timeout` から作る（Q6）。
  - 再度の `get_property` で中断した場合の出力は get タグであり、応答としてベースウェアへ届かなければコールバックは来ない。2.7 の「残りの出力は捨てる」に従うと、2 番目以降の待機の get タグは捨てられ、その待機は次の期限でタイムアウトする（設計の Open Questions 3）。
  - テストは番号を持たないイベント名（`OnPastaCallBackEntryTimeout` など）を `pending` に直接入れる。並べ替えは、番号の無い名前を末尾（文字列順）に置く全順序にする。

### 設計の統合（Synthesis）
- **Generalization**: 4 つの不具合のうち 2 つ（継続の消失・掃引の予約の残留）は「再開の手順を通らない」という同じ問題で、`EVENT.drive` の 1 箇所で直る。残りの 2 つ（タイムアウトの二重包み・U23）は「文字列を常に包む」という同じ問題で、接頭辞の判定 1 つで直る。
- **Build vs. Adopt**: 新しい部品は作らない。`resume_until_valid`・`set_co_scene`・`consume_staged`・`RES.*` を中身を変えずに使う。
- **Simplification**: 新しいモジュール、ハンドラの新しい戻り値の型、`pending` のエントリへの項目の追加（登録順の番号など）はどれも採らない。順序はイベント名の番号から得る。印の削除と、予約を捨てる関数の追加は、要件に無いため見送る。

## References
- `.kiro/specs/completed/callback-resume-unification/brief.md` — 問題・方針・スコープ
- `.kiro/specs/completed/pasta-runtime-internals-doc/absorption-ledger.md` 付録 B（300–303 行）— 継続の消失・二重包み・掃引の予約残留の照合記録
- `.kiro/specs/completed/manual-ssot-authority/absorption-ledger.md` U23 — REG 戻り値の二重包みの照合記録
- `.kiro/specs/review-improvement-loop/matrix.md` 3.48・3.49 — `EVENT.fire`×CALLBACK 統合テストの追加と、掃引修正の見送り理由
- `book/src/internals/execution-model.md`・`book/src/lua/shiori-events.md`・`book/src/internals/shiori.md` — 現行挙動の記述
