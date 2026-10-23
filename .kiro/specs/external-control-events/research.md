# ギャップ分析: external-control-events

- 日付: 2026-10-23
- 基準: main `b2f9a5cc`（ワークツリー `claude/external-control-events-ed3fb1`）
- 対象: `requirements.md`（要件 1〜7）と、今のコードの差

## 1. 分析のまとめ

- **ほぼ今ある部品の組み合わせで足りる。** `＞名前` の探し方（`ACT_IMPL.call` → `find_handler("scene", key)`）、コルーチンの 1 ビート目の返し方と続き（`EVENT.fire` → `EVENT.drive` → `set_co_scene`）、失敗の表記（`FAILURE.report`）、`SecurityLevel` の門と既定のハンドラの形（`scene_play.lua`）、JSON の読み取り（`@json` の `decode`）は、どれもそのまま使える。Rust の本体は触らなくてよい見込み。
- **呼び出しはコルーチンで包む必要がある。** `ACT_IMPL.call` は呼び先を直接呼ぶ。呼び先のシーンが `＞チェイントーク` で区切ると、コルーチンの外での中断になって止まる。ハンドラは「呼び先を引き、`(act, Reference1…)` で呼ぶ関数」をコルーチンで包んで返し、`EVENT.fire` に任せる形になる（`SCENE.co_exec` の `wrap_scene` と同じ形）。
- **見つからないときは `act:call` に任せると表記の対象が `＞名前` になる。** イベント名を対象に含めるなら（要件 4.2・4.6）、先に `act:find_handler("scene", 名前)` で引き、無ければ自分で表記を出す。`act:call` の中の「見つからない」の報告（`act.lua` L724〜729）は通らない。
- **brief の前提に 1 つずれがある。** brief の「バルーンに出るのはデバッグのときだけ」は今のコードと合わない。`[logging] balloon_warning` の既定は `true` で、デバッグとは関係なく、表記は応答（200）に載る（`failure.lua` L146〜151）。未決事項 1・2 の判断材料になる。
- **Lua のテストの土台に `@json` が無い。** `tests/lua_unittest_runner.rs` は `@json` を登録していない。代入のテストには、土台に 1 行足すか、テストで `@json` を差し替える必要がある。
- **規模は S〜M、危険度は低。** 新しいハンドラのファイル 1 つ、`event/init.lua` に 1 行、Lua のテスト 1〜2 本、マニュアル 2 ページとスキルの references。今ある道筋を外れる所は無い。

## 2. 今あるもの（要件ごとの対応表）

| 要件 | 使える今のもの | 差 | 種別 |
| --- | --- | --- | --- |
| 1.1〜1.2 `＞名前` と同じ探し方 | `ACT_IMPL.find_act_handler`（`pasta_scripts/pasta/act.lua` L361〜402）。L1 今のシーン → L2 ローカル辞書 → L3 `act` のメソッド → L4 `GLOBAL` 完全一致 → L5 グローバルシーン前方一致。別名は L5 の検索の中で当たる | 外からの act には今のシーンが無いので、L1・L2 は飛び、L3〜L5 だけが働く。L3 で `act` のメソッド（`wait`・`build` など）にも届く（未決事項 4） | Constraint |
| 1.3 ローカルを呼ばない | 上と同じ（今のシーンが無いので L1・L2 が飛ぶ） | 差なし | — |
| 1.4 引数を文字列のまま | `ACT_IMPL.call` は `handler(self, ...)` で可変長の引数を渡す（L719〜721）。シーンなら `＄０`… に入る | Reference1 から順に集める処理が要る（`act.req.reference` は 0 始まりの表） | Missing（小） |
| 1.5〜1.6 応答 | `wrap_scene`（`scene.lua` L186〜199）は最後に `act:build()` を返す。出力が無ければ `nil` → `RES.ok(nil)` は 204（`shiori/res.lua` L70〜74） | 呼び先を包むコルーチンを作る（`wrap_scene` は `scene.lua` の内部関数で、外へ出していない） | Missing（小） |
| 1.7 戻り値を載せない | `GLOBAL` の関数の戻り値は、包む関数で捨てればよい | 差なし | — |
| 2.1 続き | `EVENT.drive` → `set_co_scene`（`shiori/event/init.lua` L101〜137・L190〜206）。続きは仮想ディスパッチャが次のトークの時刻に返す | 差なし（コルーチンを返すだけ） | — |
| 2.2 Status | 抑止は OnSecondChange の中だけ | 差なし | — |
| 2.3 実行時のエラー | `EVENT.fire` が `error` を投げ、止まるエラーの応答になる | 差なし | — |
| 2.4 チェイントークの続き | 今は、ハンドラがコルーチンを返すと `set_co_scene` が待っている続きを無条件に捨てる（出力の無い `自動トーク停止` でも） | `chain-talk-interrupt` が決まりを変える。こちらはふつうのイベントと同じ道を通れば、あちらの決まりがそのまま効く | Constraint |
| 3.1〜3.3 代入・削除 | `pasta.save`（`save.lua`）は `@pasta_persistence.load()` の表をそのまま返す。`save[名前] = 値`・`save[名前] = nil` で足りる | ハンドラで書く | Missing（小） |
| 3.2 JSON の読み取り | `mlua_stdlib::json::register`（`src/runtime/mod.rs` L227〜228。`libs` に `json` があるときだけ）。`json.decode(s)` は失敗で `nil, エラーメッセージ`。**`null` は既定で `json.null`（light userdata）を返す**。`{ null_as_nil = true }` を渡すと `nil` を返す（`mlua-stdlib-0.1.0/src/json.rs` L63・L195〜205） | `null` と読めない値の見分けは、2 つ目の戻り値か `json.null` との比較で行う。どちらにするかは設計 | Unknown（小） |
| 3.4 入れ子にしない | `save[名前]` に名前をそのまま使えば区切らない | 差なし | — |
| 3.5 予約変数 | `virtual_dispatcher.lua` L63〜79 の `resolve` は呼ばれるたびに `save` を読み、数値でなければ無視する | 差なし。効くのは次にトークの時刻を決めるとき（今の決まりどおり） | — |
| 3.6〜3.7 すぐに見える・終了時に保存 | `save` は 1 つの表を共有。保存はランタイムの終了時（`src/runtime/lifecycle.rs` L38〜46） | 差なし | — |
| 3.9 配列・オブジェクト | `json.decode` は表を返す | 断るなら型を確かめる（未決事項 5） | Missing（小） |
| 4.1〜4.4 引数の誤り・見つからない | `FAILURE.report`・`FAILURE.LABEL.BAD_NAME`・`SCENE_NOT_FOUND`・`FAILURE.kind_of`・`FAILURE.value_of`（`failure.lua`）。先例は `scene_play.lua` の `bad_reference0`・`fail` | 読めない JSON に当てる種類の名前が無い（今の `NOT_NUMBER`・`NOT_STRING` は意味が合わない）。足すなら `FAILURE.LABEL` に 1 行（未決事項 2） | Missing（小） |
| 4.5 JSON の部品を外したゴースト | `pcall(require, "@json")` で確かめられる（`libs` に `-json` があると登録されない。`runtime_config.rs` L349〜356） | ハンドラで扱う | Missing（小） |
| 4.6〜4.7 表記の出し方 | `FAILURE.enabled()` を毎回読む。ログの警告は設定によらず出る | 差なし | — |
| 4.8 失敗で続きを残す | 文字列か `nil` を返せば `set_co_scene` は呼ばれない | 差なし | — |
| 5.1 いつでも働く | デバッグの門を置かなければよい | 差なし | — |
| 5.2 `SecurityLevel` | `act.req.security_level`。先例は `scene_play.lua` の `allowed`・`clock.lua` L151 | ハンドラで確かめる（デバッグの門は写さない） | Missing（小） |
| 5.3 作者の優先 | `REG.X = REG.X or function` の形（`scene_play.lua`・`choice_select.lua` など） | 差なし | — |
| 5.4 フォールバックしない | 既定のハンドラがあれば `EVENT.no_entry` は通らない | 差なし | — |
| 6.x マニュアル | `book/src/lua/shiori-events.md`（既定ハンドラの表と「7 つ」の数の文 L67〜79、主要イベントの表 L198〜、`OnPastaScene` の節 L528〜、「MCP からの呼び方」L620〜640）、`book/src/grammar/variables.md`（グローバル変数 L50〜、予約グローバル変数 L402〜412） | 2 つの節と案内が無い。スキルの references は `book/tools/gen-skill-refs.mjs` で作り直す（`--check` で鮮度を照合） | Missing |
| 7.x 壊さない・テスト | Lua のテスト（`crates/pasta_lua/tests/lua_specs/`、一覧 `init.lua` の L78 が `scene_play_event_test`）。`auto_talk_pause_test` が呼び先の 4 つの名前を確かめている | 新しいテストを足す。土台（`tests/lua_unittest_runner.rs` L42〜66）に `@json` が無い | Missing |

## 3. 決め事に効く事実

### 3.1 呼び出しをコルーチンで包む形（要件 1・2）

- `ACT_IMPL.call` は `handler(self, ...)` を直接呼ぶ（`act.lua` L717〜721）。シーンの中から `＞` で呼ぶときは、呼ぶ側がすでにコルーチンの中にいるので困らない。
- ハンドラの中で直接呼ぶと、呼び先のシーンが `＞チェイントーク`（`coroutine.yield`）で区切ったとき、コルーチンの外での中断になり止まる。
- 形の候補:
  - **(a) ハンドラで引いて包む**: `act:find_handler("scene", 名前)` で引き、関数なら `coroutine.create(function(a) fn(a, 引数…); return a:build() end)` を返す。見つからなければ表記を自分で出す（イベント名を対象に含められる）。
  - **(b) `act:call` を包む**: `coroutine.create(function(a) a:call(nil, 名前, nil, 引数…); return a:build() end)` を返す。探し方は `＞` と完全に同じになるが、見つからないときの表記は `【シーンが見つからない：＞名前】` で、イベント名が付かない。しかも見つからなくてもコルーチンを返すので、`set_co_scene` が待っている続きを捨てる（要件 4.8 に反する）。
  - **(c) `SCENE` に包む口を足す**: `wrap_scene` を外へ出し、(a) の包みに使う。`scene.lua` を触ることになり、再測定の「`act.lua`・シーン再生のファイルは触らない」には当たらないが、`scene.lua` は約束の外。
- (a) が要件 4.2・4.8 と先例（`scene_play.lua`）に素直に合う。包み方は数行なので、(c) を足すほどではない見込み。

### 3.2 `act` のメソッドに届く（未決事項 4）

- 外からの act は `SHIORI_ACT.new(STORE.actors, req)` で作られ、今のシーンを持たない。探し方の L3 は `self[key]` が関数なら返すので、`Reference0=wait` なら `act:wait("500")`、`Reference0=build` なら `act:build()` が呼ばれる。
- 辞書のシーンの外に書いた `＞wait` も同じ動きなので、「`＞名前` と同じ」には合う。除くなら、`find_act_handler` の `skip_methods`（L361。動的参照のために L3 を飛ばす）を `true` で渡せば済む。ただし `skip_methods` は L1 を `rawget` にもするので、今のシーンが無い外からの act では差が出ない。
- どちらを採っても実装の大きさは同じ。決め手は「`＞名前` と完全に同じ」か「呼び先はシーンと `GLOBAL` の関数だけ」か。

### 3.3 `null` と読めない値の見分け（要件 3.3・4.4）

- `json.decode("null")` は既定で `json.null`（light userdata）を返す。`json.decode("nul")` は `nil, "…"` を返す。
- `{ null_as_nil = true }` を渡すと `null` は `nil` になり、失敗との見分けは 2 つ目の戻り値に頼る。
- 既定のまま `json.null` と比べる方が、配列の中の `null` を含めて素直（ただし配列は断る案なので、比べるのは一段目だけで済む）。

### 3.4 失敗の表記が既定で応答に載る（未決事項 1・2）

- `FAILURE.enabled()` は `[logging] balloon_warning` を読み、既定は `true`（`failure.lua` L146〜151）。デバッグの有無は見ない。
- そのため先例に揃えると、配るゴーストでも、見つからない名前を外から呼んだときは表記が 200 の応答に載り、バルーンに出る。brief の「エラーの応答にはしない」は、止まるエラーの応答（`EVENT.fire` の `error`）にしないという意味では満たす。
- 代入の失敗も表記に載せると、MCP のツールの結果に理由が出る。載せないと、204 の理由はログでだけ分かる。

### 3.5 テストの土台（要件 7.4）

- `tests/lua_unittest_runner.rs` は `@pasta_persistence`・`@pasta_search`・`@pasta_sakura_script`・`@pasta_log` を登録するが、`@json` を登録しない。
- 候補: (a) 土台に `mlua_stdlib::json::register(&lua, None)` と `loaded.set("@json", …)` を足す（Rust のテストのファイルに数行）。(b) 代入のテストで `package.loaded["@json"]` を差し替える（本物の `null` の扱いを確かめられない）。
- (a) なら `json.null` の扱いまで本物で確かめられる。ほかのテストへの影響は無い（読まれていない）。

## 4. 実装の形の選択肢

### Option A: 既存のファイルに足す

- `scene_play.lua` に 2 つのハンドラを足す。門と失敗の出し方を共有できる。
- ❌ brief の決定（新しいファイル）と、ウェーブの約束（シーン再生のファイルは触らない）に反する。シーン再生はデバッグの門を持ち、こちらは持たないので、門の関数も共有しきれない。

### Option B: 新しいファイル（推奨の候補）

- `pasta_scripts/pasta/shiori/event/` の下に新しいファイル（例 `external_control.lua`）を置き、`REG.OnPastaCallFunction`・`REG.OnPastaSetSaveVariable` を `REG.X = REG.X or function` の形で定める。`event/init.lua` の読み込みの並びの末尾に `require` を 1 行足す。
- 門（`SecurityLevel`）・`quoted`・`fail` の小さな関数は `scene_play.lua` から写す（数行。共通にするほどではない）。
- ✅ brief とウェーブの約束に合う。テストを分けやすい。
- ❌ 小さな関数が 2 か所に重なる。

### Option C: B ＋ 共通の門を切り出す

- `SecurityLevel` の門と `fail` を共通のモジュールに出し、`scene_play.lua` からも使う。
- ❌ シーン再生のファイルを触ることになり、ウェーブの約束に反する。後の ponytail の spec に回す方がよい。

## 5. 規模と危険度

- **規模: S（1〜3 日）。** ハンドラ 2 つ（合わせて 80〜120 行の見込み）、`event/init.lua` に 1 行、`failure.lua` に種類 1 行（足すなら）、Lua のテスト 1〜2 本と一覧に 1 行、テストの土台に数行、マニュアル 2 ページ、references の作り直し。brief の見込み（5〜6 タスク）と合う。
- **危険度: 低。** 今ある道筋（`EVENT.fire`・`FAILURE.report`・`save`）を外れない。新しい探し方を作らない。気を付けるのは、呼び先をコルーチンで包むこと（3.1）と、見つからないときにコルーチンを返さないこと（要件 4.8）だけ。

## 6. 設計への申し送り

- **推奨の形**: Option B。呼び出しは 3.1 の (a)。代入は `pcall(require, "@json")` → `json.decode` → 一段目の型の確かめ → `save[名前]` への代入か削除。
- **要件の話し合いで閉じたこと**（2026-10-23）: `requirements.md` の「要件の話し合いでの決定」1〜10。失敗は 2 つのイベントとも表記とログの警告（3.1 の (a) が前提）、`external` は断る、`act` のメソッドにも届く、配列・オブジェクトは断る、JSON の部品が無いときも表記に載せる。
- **設計で決めること**
  - 読めない JSON に当てる種類の名前（`FAILURE.LABEL` に足すか）と、表記の対象・説明の形（対象はイベント名と `＄＊名前`、説明は受け取った値の見込み）。
  - `null` の見分け方（3.3）。
  - Lua のテストの土台に `@json` を足すか（3.5）。
  - 非 ASCII の識別子はブラケット記法にする（luacheck との互換。`entry.lua` の `GLOBAL["自動トーク停止"]` が先例）。
- **Research Needed**
  - SSP の MCP の `raise_event` で、Reference1 に `"01"` のような引用符付きの文字列を、崩さずに送れるか（JSON の文字列を MCP の引数の文字列の中に書く形。`scene-play-event` は引用符の要らない値しか試していない）。設計のときに実機で確かめる。
  - `chain-talk-interrupt` が先に入った場合の、台詞を出さない呼び出しの扱い（その spec の設計を読んで合わせる）。
