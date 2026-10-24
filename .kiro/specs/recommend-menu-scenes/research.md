# ギャップ分析: recommend-menu-scenes

- 実施: 2026-10-24（main `55da9722` の上のワークツリー）
- 入力: `requirements.md`（Requirement 1〜12・未決事項 1〜10）、`brief.md`（P1〜P6・A1〜A5・3 回の再測定）、roadmap の「棚卸（2026-10-24）」の同じウェーブの約束
- 方針: 情報と選択肢を出し、決定は設計とディスカッションに回す

## 1. 今あるもの（Current State）

### 1.1 イベントの振り分けと既定ハンドラ

| 資産 | 場所 | この spec との関係 |
| ---- | ---- | ------------------ |
| `EVENT.fire` | `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` 309〜359 行 | 鎖（`REG:register`）→ `rawget(REG, id) or EVENT.no_entry` の順（338 行）。文字列を返すと `RES.ok`（空文字列は 204）。リソースの受け手はここに乗るだけでよい |
| 既定ハンドラの読み込みの並び | 同 85〜90 行（末尾は `external_control`） | 新しいファイルを末尾に 1 行足す（ウェーブの約束） |
| 既定ハンドラの置き方 | `scene_play.lua` 62 行・`external_control.lua` 65 行・`mouse.lua` 173 行 | `REG.X = REG.X or function(act) … end`。作者の代入が `main.lua`（先に走る）でも辞書の Lua ブロック（後で評価）でも勝つ（Req 8.1） |
| `EVENT.drive` / `set_co_scene` | 同 110〜154・257〜301 行 | 通すと割り込みの判定・続きの差し替え・`STOP.hold` が動く。メニューのシーンは**通さない**（Req 5.1） |
| `resume_until_valid` | 同 177〜201 行（`EVENT._resume_until_valid` でテストに公開） | nil の yield を読み飛ばして再開し続ける。メニューのシーンには使えない（チェイントークの区切りで打ち切りたいので、1 回の resume で止める必要がある） |
| `run_interrupt_scene` | 同 208〜234 行 | 「その場で動かし、`act.build` を ACT の素の build に差し替えてトークン列で受け取り、残りのコルーチンを捨て、`CALLBACK.discard_staged()` で予約を捨てる」という、その場で動かす道の**手本**がすでにある |
| `EVENT.no_entry` | 同 243〜247 行 | 前方一致で `SCENE.co_exec`。`OnRecommendsiteChoice` の受け手の落ち先（Req 6.3）にそのまま使える |

### 1.2 シーンの探し方と包み方

| 資産 | 場所 | 関係 |
| ---- | ---- | ---- |
| `SCENE.co_exec_exact(name)` | `pasta_scripts/pasta/scene.lua` 230〜251 行 | 完全一致・別名あり（`search_scene_exact` が別名を置き換えてから照合）。同じ名前が複数ならランダムに重ならず選ぶ（`scene_table.rs` の `resolve_scene_id_exact`）。Req 1.3・1.4・6.1 はそのまま満たせる |
| `wrap_scene` | 同 186〜196 行 | 最後に必ず `act:build()`。SHIORI の act の `build`（`shiori/act.lua` 68 行〜）は `_spoke` を立て、`BUILDER.build(…, STORE.actor_spots, STORE.appearance)` で**持ち越しを直接書き換える**。メニューのシーンには使えない（Req 5.2） |
| `SCENE.get(global, local)` | `scene.lua` | 包まずに関数だけを引ける。その場で動かす道は「完全一致で登録名を引く → 関数を引く → 自前のコルーチンで 1 回 resume」にできる。ただし `co_exec_exact` は登録名を返さず包んだコルーチンを返すので、登録名を返す引き方（`SEARCH:search_scene_exact` を直接呼ぶ）を受け手の中で使う |

### 1.3 act と組み込みの呼び出し

| 資産 | 場所 | 関係 |
| ---- | ---- | ---- |
| `GLOBAL` の組み込み 8 つ | `shiori/entry.lua` 18〜48 行（`close_ghost`/`ゴースト終了` など） | 同じ形で `GLOBAL.recommend_item` と `GLOBAL["おすすめ項目"]` を足す（ブラケット記法。luacheck 互換） |
| Call の引き方 | `act.lua` 703 行 `ACT_IMPL.call` → `find_handler`（L1 今のシーン → L2 ローカル → L3/L4 組み込み・`GLOBAL` → L5 グローバルシーン） | `＞おすすめ項目（a、b、c、d）` は `GLOBAL["おすすめ項目"](act, a, b, c, d)` で呼ばれる。同名のシーンより組み込みが勝つ |
| `ACT_IMPL.yield` | `act.lua` 664〜676 行 | 自動トークを止めているときに**再開してしまう**（`resume_auto_talk`）。メニューのシーンで `＞チェイントーク` に着くと Req 5.4 を破る |
| `act:build` の素の版 | `act.lua` 641 行 `ACT_IMPL.build` | トークンを取り出して捨てるだけ（持ち越しに触れない）。メニューのシーンの台詞を捨てるのに使える |
| 項目の入れ物 | 無し | act の欄（例 `act._recommend_items`）に積む。`act.lua` は触らない約束なので、`GLOBAL` の関数の中で欄を作る（ウェーブの約束どおり） |

### 1.4 別名の表

| 資産 | 場所 | 関係 |
| ---- | ---- | ---- |
| `SceneAliasTable` | `crates/pasta_core/src/registry/scene_alias.rs`（284 行） | `builtin_default()` が 1 件。`from_entries` の検証は空 → 重複 → 連鎖。キーごとの合成（既定に書いた行を重ねる）口は無い |
| `parse_scene_aliases` | `crates/pasta_lua/src/loader/config/mod.rs` 107〜158 行 | 「`alias` 表があれば書いたとおり（既定と混ぜない）」。A1〜A4 で書き直す中心 |
| `SceneAliasSource` | `loader/config/sections.rs` 349〜356 行 | `BuiltinDefault` / `PastaToml`。**型は変えない**約束。見出しだけ → `BuiltinDefault`、行あり → `PastaToml`（合成した表）と `config/mod.rs` の中で決める |
| info ログ | `loader/mod.rs` 125〜129 行 | `source` と `table`（`Display`）を出す。合成後の表を渡せば Req 2.6 を満たす |
| 指紋とキャッシュ | `scene_alias.rs` の `fingerprint`、`loader/mod.rs` 135 行 `prepare_cache_dir` | 既定が変わるので最初の読み込みで作り直る（Req 2.7 は今の仕組みのまま） |
| `pasta_check` の読み方 | `crates/pasta_check/src/validate/config.rs` 145〜220 行 `read_aliases` | pasta.dll と**同じ読み方を別に実装**している（`SceneProbe` の写し）。A1〜A4 を両方に入れる必要がある。合成の規則を `pasta_core` に置けば 1 か所で済む |

### 1.5 辞書の検査

| 資産 | 場所 | 関係 |
| ---- | ---- | ---- |
| `GLOBAL_NAMES` | `pasta_check/src/validate/builtins.rs`（8 件） | 12 件にする。実行時の `GLOBAL` との一致テストがある |
| Call・選択肢の飛び先 | `validate/references.rs`（135 行） | `CallScene.args`（`pasta_dsl` の `Option<Args>`）は今は見ていない。文字列だけの式の判定 `string_literal` と、別名を当ててから探す `found_global`（前方一致）がある。項目のシーンは**完全一致**なので、別の照合が要る |
| 報告の種類 | `validate/report.rs`（`FindingKind` 25 件・`ALL`・コード名・重さ） | 足すなら `report.rs`・`msg`・`references.rs`・`debug/validate.md` と、種類の数のテスト |
| 組み込みに隠れる | `FindingKind::SceneShadowedByBuiltin`（`scenes.rs`） | `＊おすすめ項目` を書けば自動で報告される（Req 10.1 の後半は名前を足すだけ） |

### 1.6 マニュアルとスキル

- `book/src/lua/shiori-events.md`: 既定ハンドラの表 119〜131 行（「次の 9 つのイベント」）、`REG:register` の節 70 行（リソースの ID も書けると 1 文だけある）、`On` で始まらない要求は表記を出さない 1086 行。
- `book/src/reference/pasta-toml.md`: `[scene]` 362〜425 行（「丸ごと置き換わる」「見出しだけで空」の表と例。全面に書き直し）。冒頭の既定の説明 145〜147 行も。
- `book/src/grammar/call-jump.md`: シーン名の別名 156〜200 行（「既定は 1 件」「見出しだけで別名なし」）。
- `book/src/debug/validate.md`: `config-scene-alias` 116 行、`choice-target-missing` 178 行の並び。
- `book/src/reference/runtime-failures.md` 246 行: `On` で始まらない要求のエラー（500 と `X-Error-Reason`）。
- おすすめ・ポータル・`OnRecommendsiteChoice` の節はどこにも無い。
- スキルの references は `book/tools/gen-skill-refs.mjs` が `.claude/skills/*/references/` に生成する。

### 1.7 外の決まり（UKADOC）

- `sakura.recommendsites` などの値は「項目名 `\1` URL `\1` バナー `\1` 選択時トーク `\2` …」。トークの欄はさくらスクリプト。URL が `script:` ならそのスクリプトを走らせる。
- `OnRecommendsiteChoice`: Reference0＝名前、1＝URL、2＝バナー、3＝`recommend`/`portal`、4＝0/1（sakura/kero）、5＝上からの位置（0 始まり）。3〜5 は SSP のみ。
- **書いていないこと**: 選んだときに SSP が URL を開くか（ゴーストが応答したときに止めるか）、トークの欄を流すか、Reference5 が区切り線を数えるか、`char*` のメニューで Reference4 が何になるか。→ 実機の確認が要る（Research Needed）。

## 2. 要件ごとの対応（Requirement-to-Asset Map）

| Req | 必要なもの | 今ある資産 | ギャップ |
| --- | ---------- | ---------- | -------- |
| 1.1〜1.2 既定の別名 3 つ | 既定の表 | `builtin_default` | **Missing**（3 件足す）。`builtin_default()` を使う箇所は 37（テスト多数）。「1 件」を前提にした表示・指紋・数のテストの直しが要る（`scene_alias.rs` 159 行のコメントとテスト、`config_tests.rs`、`scene_alias_cache_test.rs`、`pasta_check` の `config.rs` 358〜401 行のテスト） |
| 1.3〜1.4 完全一致・ランダム | 完全一致の引き方 | `search_scene_exact`・`resolve_scene_id_exact` | 無し |
| 1.5 `char*.recommendsites` | 任意の `charN` の ID に受け手を置く | `REG` は ID の完全一致（`rawget`）だけ | **Constraint**: 既定ハンドラは決まったキーにしか置けない。別名の表は Lua から見えない（`@pasta_search` に別名の一覧を返す口が無い）。下の 3.4 |
| 2.1〜2.5 読み方 A1〜A4 | キーごとの合成・空の配列・見出しだけ・既定とのぶつかりの文 | `from_entries`（検証）、`parse_scene_aliases` | **Missing**: 合成の手順、既定とのぶつかりを見分けたエラー（今の `DuplicateAlias` は相手が既定かを知らない）。`SceneAliasError` に種類を足すか、文言を足すかは設計 |
| 2.6〜2.8 ログ・指紋・規則 | — | そのまま | 無し（出どころの印の意味の書き直しだけ） |
| 3.1〜3.8 `＞おすすめ項目`・`＞おすすめ区切り` | `GLOBAL` の関数と項目の入れ物 | `GLOBAL` の先例 | **Missing**。メニューのシーンの外で呼ばれたら何もしない（3.8）＝入れ物の欄が無い act では積まない、で足りる |
| 4.1〜4.7 値の組み立て | その場で動かす道・値の組み立て | `run_interrupt_scene` の手本、`RES.ok`（空で 204） | **Missing**（新しいハンドラのファイル） |
| 5.1 続きを残す | `EVENT.drive` を通さない | — | 自前の道なら自然に満たす |
| 5.2 持ち越しを変えない | SHIORI の `build` を通さない | 素の `ACT_IMPL.build` | `wrap_scene` を使わない |
| 5.3〜5.4 チェイントークで打ち切り・自動トークを再開しない | 区切りで止める・`ACT_IMPL.yield` の副作用を避ける | — | **Constraint**: `act.lua` は触らない約束。`yield` が `resume_auto_talk` を呼ぶのは `act:yield` の中。打ち切る方法の候補は 3.3 |
| 5.5〜5.6 コールバック待ち | 予約を捨てる | `CALLBACK.has_staged`・`discard_staged`（`run_interrupt_scene` と同じ） | 無し（手本どおり） |
| 6.1〜6.6 `OnRecommendsiteChoice` | 既定の受け手・開いたときの並びの記憶 | `co_exec_exact`、`no_entry` | **Missing**: 並びの記憶（メモリ上。保存しない）。特定の手順は **Unknown**（Reference5 の数え方） |
| 6.4 シーンが無いときの表記 | `FAILURE.report` の `SCENE_NOT_FOUND` | `external_control.lua` の `fail` の形 | 無し（失敗の種類は足さない約束どおり） |
| 7.1〜7.5 ボタンの名前 | 単語を引いて値にする | `SEARCH:search_word`（**前方一致**） | **Missing**（受け手）。**Unknown**: 結びつけの変え方（未決事項 2）。前方一致なので `＠おすすめボタン名_冬` も候補に入る（単語のふつうの引き方と同じ。気にするなら完全一致の口が要る） |
| 8.1〜8.3 作者の `REG` が勝つ | `REG.X = REG.X or …` | 既定ハンドラの置き方・鎖 | 無し |
| 9.1〜9.3 書き間違いの項目 | 引数の検査と warn | `@pasta_log` | **Missing**。**Unknown**: 単語が見つからないとき引数の式が返す値（`FAILURE.report` が表記の文字列を返す道があると、表記が項目名になりうる）→ 設計で確かめる |
| 9.4〜9.6 失敗の扱い | 表記を混ぜない・500・状態を変えない | `SHIORI.request` の xpcall と `STOP.response`（`On` で始まらない ID は表記なし） | ほぼ無し。メニューのシーンの中で `FAILURE.report` された表記はトークンとして積まれ、台詞と一緒に捨てられる。エラーは `EVENT.drive` を通らないので `STOP.hold` が置かれず、`STOP.take()` が nil のまま 500 になる（brief の指摘どおり。出方をテストで確かめる） |
| 10.1 組み込みの名前 | `GLOBAL_NAMES` | 一致テスト | 8 → 12 |
| 10.2 別名の表の読み方 | `read_aliases` | — | pasta.dll と同じ合成を入れる |
| 10.3〜10.5 項目のシーン・引数の数 | Call の引数を見る検査 | `references.rs`、`string_literal` | **Missing**（報告の種類を足すなら `report.rs` も）。完全一致の照合は `SceneTable` の完全一致の口が要る（`found_global` は前方一致） |
| 11.x マニュアル | 新しい節と 4 頁の書き直し | — | **Missing**。references の作り直しは `.claude/skills/` への書き込み（**Constraint**: 分類器に拒まれることがある。拒まれたら開発者の許可を求める） |
| 12.x テスト | Rust・Lua・pasta_check | `tests/lua_specs/`（75 本・`lua_unittest_runner.rs`）、`reg_handler_chain_test.lua` にリソースの ID の例 | 新しい Lua の spec ファイル 1〜2 本と、Lua のテストの一覧の 1 行 |

## 3. 実装の選択肢

### 3.1 全体の形

**案 A: 今のファイルを広げる**
- `event/init.lua` にリソースの受け手とその場で動かす道を書き足す。
- ✅ 新しいファイルが無い。❌ `init.lua`（371 行）はウェーブの約束で「読み込みに 1 行だけ」。`run_interrupt_scene` と並ぶ 2 つ目の自前の道が `init.lua` に入り、膨らむ。**約束に反するので取れない**。

**案 B: 新しいハンドラのファイル（brief の推奨）**
- `pasta_scripts/pasta/shiori/event/recommend.lua`（名前は設計で決める）に、リソースの受け手・ボタンの名前・`OnRecommendsiteChoice`・その場で動かす道・並びの記憶を置く。`init.lua` は読み込みに 1 行。`GLOBAL` の 2 つは `entry.lua` に置く（または新しいファイルから `GLOBAL` に足し、`entry.lua` は触らない）。
- ✅ 持ち場が閉じる。`external_control.lua` と同じ形。❌ 既定ハンドラのファイルが 1 つ増える。

**案 C: 混ぜる**
- Lua は案 B。別名の合成の規則は `pasta_core` の `SceneAliasTable` に置き（例: 既定の上に書いた行を重ねる関数）、pasta.dll（`config/mod.rs`）と `pasta_check`（`validate/config.rs`）の両方から呼ぶ。
- ✅ 2 つの読み方がずれない。❌ `pasta_core` の公開 API が 1 つ増える。
- 推奨の候補: **案 C**（Lua は B、Rust は合成を `pasta_core` に 1 か所）。

### 3.2 その場で動かす道

- **案 1: 素の関数を自前のコルーチンで 1 回 resume**: `SEARCH:search_scene_exact` で登録名 → `SCENE.get` → `coroutine.create(fn)` → `coroutine.resume(co, act)`。戻ったら（終わった・yield した）トークンを素の `ACT_IMPL.build` で捨て、項目の欄を読み、予約を捨て、コルーチンは参照を外して捨てる。`run_interrupt_scene` と同じ考え。✅ `EVENT.drive`・`set_co_scene`・`wrap_scene` に触れない。
- **案 2: コルーチンを使わずに関数を直接呼ぶ**: `fn(act)` を `pcall` で呼ぶ。❌ `＞チェイントーク`（`coroutine.yield`）が「コルーチンの外からの yield」のエラーになる。取れない。

### 3.3 チェイントークで打ち切るときの自動トークの再開（Req 5.4）

`ACT_IMPL.yield` は yield の**前に** `STORE.auto_talk_paused` を見て再開する。`act.lua` は触らない約束。
- **案 a: メニューの act だけ `yield` を差し替える**: 受け手が作る act の欄 `act.yield` に「`build` して `coroutine.yield` するだけ」の関数を置く（`run_interrupt_scene` が `iact.build` を差し替えるのと同じ手口）。`GLOBAL.yield`（`＞チェイントーク`）は `act:yield()` を呼ぶので、欄の関数が先に引かれる。✅ `act.lua` を触らない。
- **案 b: 動かす間だけ `STORE.auto_talk_paused` を退避**: ❌ ほかの状態を一時的に書き換えるので、エラーの途中で戻し損なう恐れ。
- **案 c: `act.lua` の `yield` を変える**: ❌ 約束に反する。

### 3.4 `char*.recommendsites` の受け手（Req 1.5）

- **案 i: 決まった数だけ先に置く**（`char2`〜`char9` など）: ✅ 小さい。❌ 上限が恣意的（`shortcut:` のコメントが要る）。
- **案 ii: 別名の表の置き換え先を Lua に見せ、`char%d+%.recommendsites` に当たるキーにだけ受け手を置く**: ❌ `@pasta_search`（`pasta_lua/src/search/mod.rs`）に口を足す必要があり、ウェーブの持ち場の外。既定ハンドラの登録（`require` の時点）は検索の登録（`finalize_scene`）より前なので、順番の問題もある。
- **案 iii: `EVENT.no_entry` の前で `char%d+%.recommendsites` を見分ける**: ❌ `init.lua` に 1 行を超えて手が入る（約束に反する）。
- **案 iv: 鎖 `REG:register` で全 ID を見る口**: 今の鎖は ID ごと（`handlers(req.id)`）で、全 ID を見る口は無い。❌
- 設計で決める。要件は「別名を書けば効く」なので、案 i で満たせる。ボタンの名前 `char*.recommendsites.caption` も同じ問題。

### 3.5 選ばれた項目の特定（Req 6.2）

- **案 α: 開いたときの並びをリソースの ID ごとにメモリに覚え、Reference3・4 で ID を決め、Reference5 で引き、Reference0・1 で確かめる。食い違えば名前と URL で引き直す**: ✅ ランダムで並びが変わっても正しい。❌ Reference5 が区切りを数えるかで添字がずれる（確かめが要る）。食い違いの引き直しで吸収できる。
- **案 β: 名前と URL だけで引く**: ✅ Reference5 に頼らない。❌ 同じ名前・URL の項目が 2 つあると区別できない（まれ）。
- **案 γ: 項目のシーン名を値のどこかに埋める**（例 バナーの欄や URL の後ろ）: ❌ SSP に渡す値を汚す。取れない。
- 記憶は保存しない（読み直し・再起動で消える。そのときは Req 6.3 の落ち先）。

### 3.6 ボタンの名前の結びつけの変え方（未決事項 2）

- **案 1: `[scene.alias]` と同じ読み方の pasta.toml の新しい表**（例 `[word.alias]` に似たもの）: ✅ 変えられる・`char*` も書ける。❌ pasta.toml の新しい節（`sections.rs` は `toml-serialization` と同じファイル。節が別なら約束の範囲内だが、`SceneAliasSource` と同じく慎重に）。「ゴーストの振る舞いは DSL で」とも少しずれる。
- **案 2: 結びつけは固定。`char*.recommendsites.caption` などの追加は `REG` で書く**: ✅ 最小。❌ 「些細なことに Lua を出さない」に触れる。
- **案 3: 単語の名前をリソースの ID でも引く**（`＠sakura.recommendbuttoncaption` を書けば効く）: ❌ DSL の単語名に `.` を書けるか確かめが要る。日本語の名前を書きたい作者には不便。
- **案 4: シーン名の別名の表を単語にも当てる**: ❌ 別名の規則（グローバルシーン名だけ）を変える。Constraints に反する。

### 3.7 辞書の検査の項目のシーン（Req 10.3〜10.4）

- **案 A: 新しい種類を 1 つ足す**（例 `recommend-target-missing`、注意）: `references.rs` の `CallScene` の分岐で、飛び先が `おすすめ項目`/`recommend_item` のときに引数を見る。✅ 選択肢と同じ粒度で報告できる。❌ `report.rs` の `ALL`・コード名・テスト・マニュアルの表に手が入る（同じウェーブで `validate/` に触るのはこの spec だけなので、重なりは無い）。
- **案 B: 既存の `call-target-missing` を流用**: ❌ 「Call の飛び先」と意味が違い、直し方の文が合わない。
- **案 C: 検査しない**（実行時の表記だけ）: ✅ 最小。❌ 書き間違いに SSP で選ぶまで気付けない。

## 4. 規模とリスク

- **規模: M（3〜7 日）**。既存の手本（`run_interrupt_scene`・`external_control.lua`・`GLOBAL` の組み込み・`references.rs`）に沿う作業が多いが、別名の合成（Rust 2 か所 + `pasta_core`）、その場で動かす道、並びの記憶、`pasta_check` の新しい検査、マニュアル 5 頁が重なる。brief の見込みの 10〜12 タスクと合う。
- **リスク: Medium**。理由: SSP の振る舞い（URL を開くか・Reference5 の数え方・`char*` の Reference4）が文書に無く、実機の確認が要る。`ACT_IMPL.yield` の副作用と `char*` の受け手の置き方は、ウェーブの約束（`act.lua`・`init.lua` を広げない）の中で解く必要がある。別名の既定が 4 件になり、テストの直しが広い（`builtin_default` の使用 37 か所）。

## 5. 設計への申し送り

### 推奨の方向（決定ではない）

- Lua は新しいハンドラのファイル 1 つ（案 B）。その場で動かす道は 3.2 の案 1、自動トークは 3.3 の案 a（メニューの act の `yield` を差し替え）。台詞は素の `ACT_IMPL.build` で捨てる。
- 別名の合成の規則は `pasta_core` に 1 か所（案 C）。`SceneAliasSource` の型は変えず、見出しだけ → `BuiltinDefault` を `config/mod.rs` で決める。既定とのぶつかりの文は、相手が既定の表のものと分かる形にする（`SceneAliasError` に情報を足すか、合成の側で文を作るか）。
- 失敗の種類は足さない（`SCENE_NOT_FOUND` を使う）。リソースの受け手は `FAILURE.report` を使わず warn のログだけ。
- 並びの記憶は 3.5 の案 α（引き直しつき）。

### Research Needed

1. **SSP の実機**: 項目を選んだときに URL を開くか（`OnRecommendsiteChoice` に 200 で答えたとき・204 のとき）、トークの欄を流すか、Reference5 が区切り線を数えるか、`char*` のメニューを出す条件と Reference3・4 の値。SSP の MCP（`mcp__ssp__*`）と、readonly・別ポートの SSP で試す（memory の手順）。
2. **引数の式の失敗の値**: `＞おすすめ項目（＠未定義の単語、…）` で、引数に表記の文字列が入るか、nil が入るか（`act:word` の失敗の戻り値）。表記が項目名にならないようにする。
3. **DSL の名前に `.` を書けるか**: `[scene.alias]` のキー（`sakura.recommendsites`）は TOML の引用符つきキーで書ける。単語名・シーン名に `.` を書けるかは、未決事項 2 の案 3 を取るときだけ要る。
4. **単語の完全一致の口**: ボタンの名前を完全一致で引くなら、`@pasta_search` に口が要るか（今の `search_word` は前方一致）。前方一致のままでよいかを決める。
5. **スキルの references の作り直し**: `.claude/skills/*/references/` への書き込みが分類器に拒まれないか。拒まれたら、入った分と残りを示して開発者の許可を求める。
6. **テストの直しの範囲**: `builtin_default()` を使う 37 か所のうち、既定の件数・表示・指紋を前提にしたものの洗い出し。

### ウェーブの約束の確認

- 触るファイル: 新しいハンドラのファイル、`event/init.lua`（1 行）、`shiori/entry.lua`、`pasta_core/src/registry/scene_alias.rs`（と `error.rs` の `SceneAliasError` を広げるなら）、`pasta_lua/src/loader/config/mod.rs`・`loader/mod.rs`・`loader/config_tests.rs`、`pasta_check/src/validate/builtins.rs`・`config.rs`（と、報告の種類を足すなら `report.rs`・`references.rs`・`msg`）、テスト、マニュアル 5 頁、生成の references。
- 触らない: `loader/config/sections.rs` の `SceneAliasSource`（型）、`act.lua`、`pasta/failure.lua`（種類を足さない）、`Cargo.toml`・`Cargo.lock`。`lua/shiori-events.md` の既定ハンドラの表は行を足すだけ（`mlua-stdlib-removal` と隣の行）。
- 未決事項 2 で pasta.toml の新しい表を取ると、`sections.rs`（`toml-serialization` の持ち場のファイル）に節を足すことになる。節は別だが、設計で重なりを確かめる。

### brief の古くなった記述（参考）

- 冒頭のステータスの「推奨は 0.4.2」と「要件で決めること」の版の行（0.4.3 に決まった）。
- Adjacent の「`ponytail-check-release` が `builtins.rs` を触る」は誤り（あちらは `release.rs` などで、`validate/` に触らない）。
- P5 の「組み込みの受け手は作者のスクリプトより先に置かれる」は不正確（`main.lua` が先に走り、既定は空のキーにだけ置く。どちらの置き場所でも作者が勝つ）。要件 8.1 はこの読みで書いた。
