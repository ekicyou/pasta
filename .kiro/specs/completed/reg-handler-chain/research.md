# ギャップ分析: reg-handler-chain

## Summary

- **Feature**: `reg-handler-chain`
- **Discovery Scope**: Extension（今の `REG` と `EVENT.fire` の上に足す）
- **基準**: main `b2f9a5cc`（2026-10-23。`scene-play-event`・`ontalk-suppression` の後）
- **Key Findings**:
  - 足りないのは「鎖を持つ所」と「`EVENT.fire` の引き方」の 2 つだけ。返り値の型で分ける枝（`event/init.lua` L238〜255）は、`true` も含めて今のままで要件を満たす（文字列でもコルーチンでもない値は 204）。
  - 表を空にする 3 つのテストは、どれも `package.loaded["pasta.shiori.event.register"] = nil` で登録の表を読み直している。鎖を `register.lua` のモジュールの中に持てば、読み直しだけで鎖も空になり、3 つのテストは直さずに通る見込み。brief の「鎖を空にする口」は要らないかもしれない（OPEN QUESTION 6）。
  - 鎖のハンドラに同じ act を渡すと、見送ったハンドラが積んだ台詞が、次のハンドラ・底・シーンの出力に混ざる（`act:build()` が呼ばれるまでトークンは残る）。止まるエラーの表記（`STOP.build_value`）も控えた act の末尾に積むので、同じく混ざる。ハンドラごとに act を作るなら、`STOP.hold_handler` もハンドラごとに呼び直す（OPEN QUESTION 1）。
  - 止まった場所の表記は、エラー文の「チャンク:行」から作る（`STOP.detail`）。ハンドラの act を正しく控え直せば、鎖の中で止まったハンドラ自身の場所を指す。鎖の位置を表記に足す必要は無い。

## Research Log

### 1. 今の登録表とイベントの振り分け

- **Sources**: `crates/pasta_lua/pasta_scripts/pasta/shiori/event/register.lua`（48 行）、`event/init.lua`（268 行）
- **Findings**:
  - `REG` は `local REG = {}` を返すだけの表。メタテーブルは無い。依存モジュールも無い（循環を避けるため、と冒頭の説明にある）。
  - `EVENT.fire`（L216〜256）の順: `CALLBACK.try_route`（L218）→ `CLOCK.observe`（L224）→ `create_act`（L229）→ `REG[req.id] or EVENT.no_entry`（L233）→ `STOP.hold_handler(act)`（L235）→ `handler(act)`（L236）→ 型で分ける枝（L238〜255）。
  - 型で分ける枝: thread は `EVENT.drive(result, act, act)`、文字列は `SHIORI/` で始まれば素通し・それ以外は `RES.ok`、それ以外（`nil`・`false`・`true`・数・表）は `RES.no_content()`。
  - `EVENT.no_entry`（L177〜181）は `SCENE.co_exec(act, act.req.id, nil, nil)`。見つからなければ `nil` → 204。
  - 既定のハンドラは 5 ファイル 7 つ（`boot.lua`・`choice_select.lua`・`second_change.lua`・`mouse.lua`（2 つ）・`scene_play.lua`（2 つ））。どれも `REG.X = REG.X or function … end`。`event/init.lua` L78〜82 で読む。
- **Implications**:
  - 変えるのは L231〜236 の「引いて呼ぶ」所だけで足りる。L238〜255 は、鎖が止まったハンドラの戻り値をそのまま渡せば再利用できる。
  - `REG` にメタテーブルの `__index` を付けると、`REG[req.id]` で `"register"` を引いたときに登録の口そのものが返る。L233 は `rawget(REG, req.id)` にする（brief どおり）。既定のハンドラの `REG.X = REG.X or …` は `register` 以外のキーなので、`__index` の影響を受けない。

### 2. 返り値の読み方（Requirement 2）

- `nil`・`false` → 次へ。真の値 → 止めて、今の型で分ける枝へ渡す。
- Lua の真偽のとおりにすると、空文字列 `""` も真なので止まり、今の決まりで 204 になる。数・表も止まって 204。brief の「Lua の真偽のとおり」と一致するが、作者が `""` を「何も無し」のつもりで返すと、古いハンドラと既定が呼ばれない（OPEN QUESTION 5）。
- 底の `REG[id]` の返り値は今のまま（`nil`・`false` でもシーンへは回らず 204）。要件 3.1 はこの「今のまま」を保つ。

### 3. act の共有（Requirement 2.9）

- **Sources**: `pasta/shiori/act.lua` L48（`SHIORI_ACT.new(actors, req)`）・L65（`build`）、`pasta/act.lua` L641〜643（`build` がトークンを空にする）
- **Findings**:
  - トークンは `act:build()` が呼ばれたときだけ空になる。`act.sakura:talk(...)` で積んだまま `nil` を返すと、後のハンドラが `act:build()` したときに前の分が先頭に混ざる。底の既定 OnSecondChange が返すシーン（`dispatcher.dispatch(act)`）も同じ act で動く。
  - act には `scene_name`・`line_map` など、シーンを動かすと立つ欄もある（`act.lua` L207〜215・L780〜800）。前のハンドラが act でシーンを呼んでいれば、後のハンドラの止まる表記の名前（`STOP.subject`）にも影響しうる。
  - `SHIORI_ACT.new` は設定を 3 つ読む程度で軽い。OnSecondChange は毎秒届くが、鎖の長さぶん作っても負担は小さい見込み。
- **Options**:
  - (a) ハンドラごとに act を作る（底とシーンにも新しい act）。混ざらない。`STOP.hold_handler` を呼ぶたびに控え直す。
  - (b) 1 つの act を共有し、マニュアルに「見送るなら積まない」と書く。コードは最小だが、書き忘れると台詞が混ざる。
  - (c) 共有し、見送ったときにトークンだけ捨てる。act のほかの欄（シーンの名前など）は残る。
- **推奨（要件の仮定）**: (a)。

### 4. 止まるエラーの表記（Requirement 6）

- **Sources**: `pasta/shiori/stop.lua`（212 行）、`pasta/shiori/entry.lua` L103・L115
- **Findings**:
  - `entry.lua` は要求の始めに `STOP.clear()`、エラーのときに `STOP.take_handler()` で控えた act を取り、`STOP.response` に渡す。
  - 表記の対象（subject）は `STOP.subject(act)`（シーンの名前。ハンドラの act では普通 nil）→ 無ければ要求の ID。場所（detail）はエラー文の先頭の「チャンク:行」を `@pasta_chunks` で引いて作る。`scripts/` の `.lua` ならそのファイルと行が出る。辞書の Lua ブロックに書いたハンドラは、`act.line_map` が無いので場所が落ちる（今の `REG` のハンドラと同じ。この spec では変えない）。
  - 控えは 1 要求に 1 つで、`hold_handler` は上書き。鎖のハンドラを呼ぶたびに控え直せば、最後に控えた＝止まったハンドラの act になる。
  - 鎖の中でエラーが投げられれば、そのまま `SHIORI.request` の `xpcall` に抜けるので、古い登録・底は呼ばれない（要件 6.2 は自然に満たす）。エラーで登録を外す仕組みも無い（要件 6.5 も自然に満たす）。
- **Implications**: 要件 6 は、act を正しく控え直すことだけで満たせる見込み。表記に鎖の何番目かを足すことは要件にしていない。

### 5. テストへの影響（Requirement 7）

- **Sources**: `crates/pasta_lua/tests/lua_specs/init.lua`（L91〜94 で各テストを 1 つの VM で順に `require`）、`event_clock_hook_test.lua` L14・L19〜24、`event_coroutine_test.lua` L14・L22〜29、`second_change_thread_test.lua` L130〜139
- **Findings**:
  - Lua のテストは 1 つの VM で続けて走る。鎖が前のテストから持ち越されると、後のテストの結果が変わる。
  - 3 つのテストは、`pasta.shiori.event.register` を `package.loaded` から外して読み直したうえで `pairs(REG)` で空にしている。鎖を `register.lua` の中（モジュールのローカル）に持てば、読み直しで鎖も新しくなる。`pairs(REG)` に鎖が出ないので、空にするループも鎖に触れない。
  - Rust 側のテスト（`tests/shiori/event_dispatch_test.rs` ほか多数）は `REG.X = …` の代入だけを使い、`pairs(REG)` は使わない。鎖を足しても影響しない見込み。
  - fixtures（`pasta_shiori/tests/fixtures/runtime_error_balloon/scripts/main.lua` の `REG.plain_fail` など）は、`On` で始まらない ID のハンドラを `REG` に代入している。底として今どおり動く必要がある（要件 3.3・5.1）。
- **Implications**: 専用の「鎖を空にする口」は、読み直しで足りるなら作らなくてよい。新しいテストも、同じく読み直してから始めるか、外す関数で後片付けする。

### 6. マニュアルと references（Requirement 8）

- `book/src/lua/shiori-events.md`
  - L10〜17: ページ冒頭の処理の順（3. `REG` のハンドラ・4. シーン）。鎖を加える。
  - L19〜72: REG の節。L72「1 つのイベントに登録できるハンドラは 1 つである。」を改める。返り値の表（L49〜54）は鎖のハンドラの読み方（偽なら次へ・`true` は 204）を別に示す。
  - L81〜95: 既定ハンドラを包む段落と作例。鎖で書く書き換え例を足す。
  - L69〜79（既定ハンドラの表と数の文）: `external-control-events` が行を足すので、この spec は触らない（2026-10-23 の約束）。
  - L737〜746: シーン関数フォールバックの図。鎖の段を足す。
- `book/src/internals/execution-model.md` L157〜161: `EVENT.fire` の手順の 3.（`(REG[req.id] or EVENT.no_entry)(act)`）。
- `book/src/internals/shiori.md` L314〜318（振り分けの順の 3.）・L410〜413（既定の OnSecondChange の前段の図）。
- 生成の references: `node book/tools/gen-skill-refs.mjs` で `.claude/skills/pasta-lua-coding/references/shiori-events.md`・`internal-modules.md` ほかを作り直す。
- 手書きの `.claude/skills/pasta-lua-coding/SKILL.md` L111（`REG.EventName = function(act) ... end`）と L174 に `REG` の説明がある。直すなら開発者の許可が要る（分類器が `.claude/skills` の編集を止める）（OPEN QUESTION 7）。

## Requirement-to-Asset Map

| 要件 | 今ある資産 | ギャップ |
| ---- | ---------- | -------- |
| 1. 重ねて登録 | `register.lua`（ただの表） | **Missing**: `register` の口・鎖の置き場所・外す関数。**Constraint**: `pairs(REG)` に出さない・`REG.X` の代入と読み出しを変えない・LuaJIT 2.1（`goto`・`<close>` 不可） |
| 1.7 引数の確かめ | なし | **Missing**: 文字列でない ID・関数でない値・`.` で呼んだときのエラー（仮定。OQ 3） |
| 2. 呼ぶ順と止まり方 | `EVENT.fire` L231〜255 | **Missing**: 鎖を新しい順に呼ぶループ。止まった後は今の型で分ける枝をそのまま使える |
| 2.8 コールバック・時計が先 | L218〜226 | なし（ループを L229 より後に置けば満たす） |
| 2.9 台詞を混ぜない | `create_act`（L93〜95） | **Unknown**: act をハンドラごとに作るか（OQ 1） |
| 3. 鎖の底 | L233 `REG[req.id] or EVENT.no_entry` | **Constraint**: `__index` を付けるなら `rawget` に変える。それ以外は今のまま |
| 4. 外す | なし | **Missing**: 外す関数・二重に外しても無害・途中の登録と外しの反映の時期（OQ 2） |
| 5. すべての ID | Rust 側は ID で分けない（`SHIORI.request` → `EVENT.fire`） | `register` という ID だけ `rawget` で守る |
| 6. エラー | `entry.lua` の `xpcall`・`STOP.hold_handler`・`STOP.response` | **Constraint**: ハンドラごとに `STOP.hold_handler` を控え直す |
| 7. テスト | 3 つのテストは register を読み直している | **Unknown**: 専用の空にする口が要るか（OQ 6）。新しいテストファイルと一覧（`lua_specs/init.lua`）の 1 行は Missing |
| 8. マニュアル | 上の 6. の行 | **Missing**: 鎖の節・書き換え例・図・内部の説明・references の作り直し |

## Implementation Approach Options

| Option | 中身 | Strengths | Risks / Limitations |
| ------ | ---- | --------- | ------------------- |
| A. 既存を延ばす | `register.lua` に、モジュールのローカルの鎖（ID ごとの配列）と、メタテーブルの `__index`（`register` と、`EVENT.fire` が鎖を引くための内部の関数）を足す。`EVENT.fire` L231〜236 を「鎖を新しい順に呼ぶ → `rawget(REG, id)` → `EVENT.no_entry`」に替える | 触るのは 2 ファイル。`register.lua` は依存の無いまま。読み直しで鎖も空になる | `REG` に初めてメタテーブルが付く。`__index` で引ける名前（`register` と内部の関数）が、イベント名と重ならないようにする |
| B. 新しいモジュール | 鎖を `pasta.shiori.event.chain` のような新しいモジュールに置き、`REG` の `__index` から `register` だけを借りる。`EVENT.fire` は新しいモジュールから鎖を引く | 鎖の責務が分かれる。テストで鎖だけを読み直せる | ファイルが 1 つ増える。`register.lua` を読み直しても鎖が残るので、3 つのテストに空にする口が要る |
| C. 混ぜる | A の置き場所で、鎖を呼ぶループだけを `register.lua` の関数（例: 鎖を新しい順に回す関数）として持ち、`EVENT.fire` は呼ぶだけにする | `EVENT.fire` の差分が最小（`event/init.lua` を共有するほかの spec とのぶつかりが小さい） | `register.lua` が「呼ぶ」責務も持つ。act の作り直しと `STOP.hold_handler` を `register.lua` に持ち込むと依存が増える（循環に注意） |

- **ponytail の目**: A が最小。鎖は ID ごとの配列 1 つと、外す関数（登録の目印を配列から外す）だけで足りる。優先度・見張り役・鎖を列挙する公開の口は作らない。
- **途中の登録と外し**: 呼ぶ前に、その回の鎖の写し（配列のコピー）を取ってから回せば、要件 4.5（その回の順を変えない）を素直に満たす。毎秒 1 つの小さな配列を作る程度の負担。

## Implementation Complexity & Risk

- **Effort**: S（1〜3 日）。触るのは Lua の 2 ファイル（`register.lua`・`event/init.lua` の数行）、新しいテスト 1 本と一覧の 1 行、マニュアル 3 ページ、references の作り直し。brief の見積もり 5〜7 タスクと合う。
- **Risk**: Low。今の型で分ける枝・エラーの道・既定のハンドラの置き方をそのまま使う。鎖が空なら今と同じ道を通る。注意は `REG` にメタテーブルを付けることと、act の扱い（OQ 1）だけ。

## 同じウェーブの spec とのファイルの持ち場（2026-10-23）

| ファイル | この spec | `chain-talk-interrupt` | `external-control-events` |
| -------- | --------- | ---------------------- | ------------------------- |
| `event/init.lua` | `EVENT.fire` のハンドラを引いて呼ぶ所（L231〜236）と、モジュールの冒頭の説明（L1〜66） | `set_co_scene`〜`EVENT.drive`（L97〜208）。`EVENT.fire` に入れるなら型で分ける枝（L238〜255）だけ | 読み込みの並び（L78〜82）に `require` を 1 行 |
| `event/register.lua` | 全体（鎖・`register`・冒頭の説明） | 触らない | 触らない |
| `lua/shiori-events.md` | REG の節（L19〜72 の返り値と「1 つだけ」の文、L81〜95 の包む段落）、冒頭の処理の順（L10〜17）、フォールバックの図（L737〜746） | シーン再生の「再生と続き」・OnTalk の節 | 既定ハンドラの表と数の文（L69〜79）・新しい節 |
| `internals/execution-model.md`・`internals/shiori.md` | `EVENT.fire` の振り分けの順 | `EVENT.drive` の手順（`execution-model.md` L247〜270・`shiori.md` L388） | 既定ハンドラの中身の列挙に足す可能性 |
| `lua_specs/init.lua` | 一覧に 1 行 | 一覧に 1 行 | 一覧に 1 行 |
| 生成の references | 作り直す | 作り直す | 作り直す |

- 約束（2026-10-23 の棚卸）: 「後から入る側が rebase で合わせ、生成の references は作り直す」。行が隣り合う所（`EVENT.fire` の L231〜236 と L238〜255、マニュアルの L69〜79 と L81〜95）は、間の行を残して触る。
- `chain-talk-interrupt` は `EVENT.drive(result, act, act)` の呼び方を変えうる。鎖のハンドラが返したコルーチンは今と同じ所（型で分ける枝）に渡すので、あちらの変更はそのまま効く。act をハンドラごとに作る場合（OQ 1 の (a)）は、コルーチンを返したハンドラの act を `EVENT.drive` に渡すこと。
- 隙間の spec `ponytail-lua-scripts`（`pasta_scripts/**` の整理）は、0.4.2 の後に入るので、この spec が先。

## 要件ディスカッションの決定（2026-10-23 開発者の一括承認）

- OQ 1: act はハンドラごと（鎖の底とシーンも）に作り直し、`STOP.hold_handler` もそのたびに控え直す（R2.9）。
- OQ 2: 呼び始める前にその回の鎖の写しを取り、途中の登録と外しは次のイベントから効かせる（R4.5）。
- OQ 3・4: 文字列でない ID・関数でない値はエラーで登録しない。同じ関数の 2 回の登録は別の登録（R1.3・R1.7）。
- OQ 5: Lua の真偽のとおり。`""` も止まって 204。マニュアルの返り値の表に書く（R2.6・R8.1）。
- OQ 6: 鎖を空にする専用の口は作らず、`register.lua` の読み直しで空に戻す（R7.1）。
- OQ 7: 手書きの SKILL.md のモジュール表と §6 に `REG:register` を足す。直すその場で開発者の許可を得る（R8.8）。

## Recommendations for Design Phase

- **推奨の形**: Option A。`register.lua` に鎖と `__index`、`EVENT.fire` は `rawget` と新しい順のループ。返り値の型で分ける枝は今のまま。
- **決めること**:
  - act の扱い（OQ 1。推奨はハンドラごとに作る）と、それに合わせた `STOP.hold_handler` の控え直し。
  - 途中の登録と外しの反映の時期（OQ 2。推奨は呼ぶ前に写しを取る）。
  - `EVENT.fire` が鎖を引くための内部の口の名前（`pairs(REG)` に出ず、イベント名と重ならない名前）。
  - 鎖を空にする口を作るか、読み直しで足りるとするか（OQ 6）。
- **Research Needed**:
  - `REG` にメタテーブルを付けたとき、`pasta_lua` の Rust 側が `REG` の表を直接読む所が無いかの確認（grep では見つからなかった。`pasta_shiori` の fixtures を含めて設計で再確認）。
  - 鎖の中で `SCENE.co_exec` や `act:call` を使うハンドラが、作り直した act で今と同じに動くこと（`act.req` を持つ新しい act なら同じはず）。

## 設計フェーズの調査と決定（2026-10-23、/kiro-spec-design -y）

- **Discovery**: light（既存の延長）。使ったスキルの手引き: `kiro-spec-design` の `design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`・`design-principles.md`。ponytail（最小の完全な変更）を Option A の選定に当てた。

### Rust 側が REG の表を直接読むか（Research Needed の解消）

- **Sources**: `crates/*/src` と `crates/*/tests`（`pasta_shiori/tests/fixtures` を含む）の grep（`event.register`・`REG`・`pairs(REG)`・`next(REG)`・`getmetatable`）。
- **Findings**:
  - `crates/*/src` に `REG` を読む Rust のコードは無い。Rust は `SHIORI.request` → `EVENT.fire` を通すだけ。
  - Rust のテストと fixtures は、Lua の文字列の中で `REG.X = …` の代入と `type(REG.X)` の読み出しを使う。表の形を見るのは `tests/shiori/event_dispatch_test.rs` L40 の `type(REG) == "table" and next(REG) == nil` だけ。鎖をモジュールのローカルに置き、`register` を `__index` からだけ見せれば、`next(REG)` は今どおり `nil`。
  - fixtures のうち `async_callback`・`debug_preservation` は `scripts/pasta/shiori/entry.lua` を上書きするが、どちらも `EVENT.fire` を呼ぶ。`register.lua`・`event/init.lua` を上書きする fixture は無い。`runtime_error_balloon/scripts/main.lua` の `REG.plain_fail` は底として今どおり動く。
- **Implications**: メタテーブルを付けても Rust 側の変更は要らない。

### 鎖を引く内部の口の置き場所

- `__index` に置くと `REG.<名前>` が `nil` でなくなり、要件 1.6（`REG.register` 以外の読み出しは今と同じ）に反する。
- メタテーブルの普通の欄（`handlers`）に置き、`EVENT.fire` は `getmetatable(REG).handlers(id)` で引く。`__index` を通らないので `REG.handlers` は `nil` のまま。手元の Lua で `getmetatable(setmetatable({}, {__index = {...}, handlers = f})).handlers` が引けること、`rawget({}, nil)` が `nil` を返すことを確かめた。

### 外す関数と同じ関数の 2 回の登録

- 登録ごとに目印の表 `{ fn = fn }` を積み、外す関数はその目印を探して `table.remove` する。関数そのもので探すと、同じ関数の 2 回の登録を区別できない（要件 1.3）。

### テストの持ち越し

- `lua_specs/init.lua` は全スイートを 1 つの VM で順に `require` する。`event_no_entry_test.lua` などは `pasta.shiori.event` だけを読み直し、`register` は読み直さない。新しいテストが鎖に登録を残すと、後のスイートの `EVENT.fire` に効く。新しいテストは各テストの終わりに外す関数で後片付けする。
- エラーの確かめは `shiori_entry_test.lua` と同じく、entry の依存の閉包を一括で読み直してから `SHIORI.request` を通す（`STOP.clear`・`take_handler` の道を通すため）。

### マニュアルの追加の行

- ギャップ分析の一覧に加え、`internals/shiori.md` L96（`REG` の説明「依存を持たない空の表」）と L358（`hold_handler` を呼ぶ時期）、`internals/execution-model.md` L174（コルーチンを返す経路の列挙）も鎖に合わせて直す。
- `reference/runtime-failures.md` L159・L183 の「`REG` に登録した関数」は鎖のハンドラも含むので直さない。

## Design Synthesis（2026-10-23）

- **Generalization**: 鎖のハンドラの戻り値と底の戻り値は、止めた後は同じ型判定の枝で応答にする。鎖の専用の応答の規則は作らず、「真なら止める」だけを足す。すべての ID（イベント・リソース）を同じループで扱う。
- **Build vs Adopt**: 標準の Lua の `setmetatable`・`rawget`・`table.remove` で足りる。イベントのライブラリ（優先度付きのディスパッチャなど）は、要件が「新しい順・真で止める」だけなので取らない。
- **Simplification**: 専用の鎖を空にする口・鎖の列挙の口・優先度・見張り役・数の上限・新しいモジュール（Option B）を作らない。型判定の枝を関数に切り出すこともしない（`chain-talk-interrupt` の持ち場の行を動かさないため）。`EVENT.fire` は `act`・`result` の 2 つの変数をループと底で埋め、今の枝がそれを読む形にする。

## Design Decisions（2026-10-23）

### Decision: `EVENT.fire` の変える行の範囲

- **Context**: ハンドラごとに act を作る（要件 2.9）には、今の L229 `local act = create_act(req)` をループの中へ移す必要がある。棚卸の約束は L231〜236。
- **Selected Approach**: L228〜236 をまとめて替え、L238〜255 の型判定の枝は 1 行も動かさない。
- **Trade-offs**: 約束より 2 行広いが、`chain-talk-interrupt` の持ち場（型判定の枝・`EVENT.drive`）とは重ならない。設計ディスカッションで確かめる（design.md の OPEN QUESTION 1）。

### Decision: `REG.register = 関数` を守らない

- **Context**: 生のキーが `__index` より先に引かれるため、代入すると `REG:register` が隠れる。
- **Alternatives**: (1) 守らない（2）`__newindex` で `register` への代入をエラーにする。
- **Selected Approach**: (1)。要件 1.6（代入を今と同じに働かせる）を優先し、`register` という ID の底を置く作者はまず居ないので、内部の説明に書くだけにする（design.md の OPEN QUESTION 2）。

## Risks & Mitigations（設計）

- 鎖を残したまま後のスイートが走る — 新しいテストが外す関数で後片付けする。
- `event/init.lua` を同じウェーブの 2 本と共有 — 変える行を L228〜236 に閉じ、後から入る側が rebase で合わせる。
- 生成の references の古さで CI が落ちる — マニュアルを直したタスクで `gen-skill-refs.mjs` を走らせ、`--check` を通す。

## 設計ディスカッションの決定（2026-10-23 開発者の一括承認）

- `event/init.lua` は L228〜236 をまとめて替える（act をループの中で作り直すため）。`act`・`result` の名前は残し、型判定の枝は変えない。
- `REG.register = 関数` の代入は守らない（`__newindex` は要件 1.6 とぶつかる）。内部の説明にだけ書く。
- 鎖の写しの口はメタテーブルの `handlers` 欄（`REG.handlers` は `nil` のまま・読み直しで鎖も空）。
- エラーの場所の表記は Lua テストで確かめ、e2e fixture には足さない（`@pasta_chunks` の変換は鎖と無関係で既に e2e がある）。
- 引数の誤りは英語の `REG:register: …`（`act.lua` の口とそろえる）。
- マニュアル L69 は新しい小節で補い、`external-control-events` が先に入っていれば 1 語（登録すると→代入すると）だけ直す。
- テストの後片付けは読み直しで行う（lua_test に `after_each` は無い。assert が落ちても鎖を持ち越さない）。
