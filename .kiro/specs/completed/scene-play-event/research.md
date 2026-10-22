# ギャップ分析: scene-play-event

- 日付: 2026-10-22
- 基準: main `dc46f446`（ワークツリー `claude/kiro-start-scene-play-event-210fdb`）
- 対象: `requirements.md`（要件 1〜7）と、今のコードの差

## 1. 分析のまとめ

- **ほぼ今ある部品の組み合わせで足りる。** 名前どおりの検索（`SEARCH:search_scene_exact(名前, 親)`）、位置からの引き当て（`SourceMap::scene_at`）、登録名からのシーン関数の取り出し（`SCENE.get`・`SCENE.get_start`）、コルーチンの 1 ビート目の返し方と差し替え（`EVENT.fire` → `EVENT.drive` → `set_co_scene`）、失敗の表記（`FAILURE.report`）は、どれもそのまま使える。
- **欠けているのは「Lua から見える口」だけ。** Lua からデバッグの有効を知る口と、Lua から位置でシーンを引く口が無い。ローカルシーンを名前どおりに引く Lua の口（`scene.lua`）も無い。
- **brief の前提に 1 つずれがある（重要）。** brief は「ポートが取れないときも位置の索引は作られる」としているが、今のコードでは作られない。ランタイムは待ち受けが立たなかったとき、ソースマップを捨て（`runtime/mod.rs` L267〜271）、索引の突合（`runtime/factory.rs` L241）も飛ばす。「設定で決める」（未決事項 1 の推奨）を採るなら、ここを直す必要がある。
- **相対パスは結ぶ処理が要る。** `uri_to_pasta_path` は相対パスを SSP の今のフォルダから絶対にする。`ghost/master/`（`runtime.base_dir`）に先に結ぶ必要がある。
- **規模は M、危険度は低〜中。** 新しいハンドラのファイル 1 つ、Lua の口 2 つ、Rust の小さな公開と 1 か所の直し、テスト、マニュアル。危険は「デバッグを無効にしたときに何も変えない」を守れるかと、`raise_event` が GET で届くかの実機の確かめ。

## 2. 今あるもの（要件ごとの対応表）

| 要件 | 使える今のもの | 差 | 種別 |
| --- | --- | --- | --- |
| 1.1 大域を名前で | `SCENE.co_exec_exact(name)`（`pasta_scripts/pasta/scene.lua` L227）。完全一致・別名・順送り込み | そのまま使える | — |
| 1.2 ローカルを名前で | `SEARCH:search_scene_exact(name, parent)` は親を受ける（`src/search/context.rs` L105・L302〜306） | `scene.lua` に親を通す口が無い。親は**登録名**（`会話_1`）を渡す必要があるので、先に大域を引いてからローカルを引く 2 段になる | Missing |
| 1.3〜1.6 名前の規則 | `search_scene_by`（`context.rs` L114〜）が `sanitize_name`・別名（大域だけ）・`resolve_scene_id_exact`（順送り）を当てる | 差なし。別名はローカルには当たらない（今の規則どおり） | — |
| 1.7 空の Reference1 | なし | ハンドラで扱う | Missing |
| 1.8 同名の大域とローカル | 順送りは大域を 1 つ選ぶだけ | 「そのローカルを持つ大域から選ぶ」案を採った（2026-10-22）。今の検索の形では足りず、候補の絞り込みが要る（設計で決める） | Constraint |
| 2.1〜2.2 位置で | `SourceMap::scene_at`（`src/debug/source_map/mod.rs` L528）→ `SceneIdentityIndex::scene_at`（`scene_index.rs` L123）。返すのは `(scene_id, parent)` の登録名 | Lua から呼ぶ口が無い | Missing |
| 2.3 相対パス | `uri_to_pasta_path`（`src/debug/playscene.rs` L100）は `std::path::absolute` で SSP の今のフォルダに結ぶ | `runtime.base_dir`（`factory.rs` L176。`ghost/master/`）に先に結ぶ処理が要る | Missing |
| 2.4〜2.5 絶対パス・区切り・大小 | `canonicalize_pasta_file` が `\`→`/` と Windows の大小無視を担う | 差なし | — |
| 2.6 行の形 | なし | ハンドラで整数を確かめる | Missing |
| 2.7 同名の別ファイル | `scene-identity-format` の制約を受け継ぐ | 直さない（範囲外） | Constraint |
| 3.1〜3.3 中断と続き | `EVENT.fire` がハンドラの返したコルーチンを `EVENT.drive` で 1 ビート進め、`set_co_scene` で差し替える（`shiori/event/init.lua` L215〜255）。続きは `check_talk`（`virtual_dispatcher.lua` L188〜217）が次のトークの時刻に返す | 差なし。ハンドラはコルーチンを返すだけでよい | — |
| 3.4 Status の抑止 | 抑止は `virtual_dispatcher.dispatch`（OnSecondChange）の中だけ | 差なし（ほかのイベントは抑止されない） | — |
| 3.5 実行時のエラー | `EVENT.fire` が `error` を投げ、`runtime-error-balloon` の道で出る | 差なし | — |
| 4.1〜4.3 有効の判定と情報の保持 | Rust: `DebugConfig::from_env(...)`.enabled（`loader/mod.rs` L214〜216）、`runtime.debug_enabled()`（`runtime/exec.rs` L63。持ち手の有無） | Lua から見える口が無い。設定で決めるなら、待ち受けが立たないときもソースマップと索引を残す直しが要る（`runtime/mod.rs` L267〜271） | Missing / Constraint |
| 4.4 無効なら 204 | `REG.OnX = REG.OnX or function...` の先例（`choice_select.lua` L41） | ハンドラで nil を返せば 204。ログは `@pasta_log` | — |
| 4.5 作者の優先 | 同上の先例 | 差なし | — |
| 4.6 `SecurityLevel` | `act.req.security_level`（`shiori/res.lua` L20・`clock.lua` L151 が `external` を断る先例） | ハンドラで確かめる | Missing |
| 5.1〜5.5 失敗の表記 | `FAILURE.report(act, record)`・`FAILURE.LABEL.SCENE_NOT_FOUND`（`pasta/failure.lua`）。`[logging] balloon_warning` を毎回読む | 引数の誤りの種類の名前は無い（足すかは未決事項 5） | Missing（小） |
| 5.6 失敗時に会話を残す | 文字列を返せば `set_co_scene` は呼ばれない | 差なし | — |
| 6.x マニュアル | `book/src/lua/shiori-events.md`（既定ハンドラーの表 L196〜、`OnPastaClock` の節が先例）、`book/src/internals/debug.md`「シーンキック」L347〜、`book/src/debug/dev-actions.md`「シーン再生」L40〜 | 節が無い。スキルの references は `book/tools/gen-skill-refs.mjs` L60 で `lua/shiori-events.md` だけが写される（`internals/`・`debug/` は写されない） | Missing |
| 7.x 壊さない・テスト | Lua のテスト（`crates/pasta_lua/tests/lua_specs/`、一覧 `init.lua`。キックのテストが 5 本ある）、Rust のテスト（`src/debug/playscene_tests.rs`・`wiring_play_scene_at_tests.rs`） | 新しいテストを足す | Missing |

## 3. 決め事に効く事実

### 3.1 「ポートが取れない」ときに索引が残らない（未決事項 1 に効く）

- ローダーは設定（`[debug]`・`PASTA_DEBUG`）で有効ならソースマップを作る（`loader/mod.rs` L214〜232）。
- ランタイムの初期化は `debug::enable` を呼び、待ち受けが立たなければ持ち手を `None` にして起動を続ける（`debug/enable.rs` L120〜129）。
- そのあと「持ち手があるときだけソースマップを持つ」（`runtime/mod.rs` L267〜271）。持ち手が無いと `runtime.source_map` は `None` になり、`factory.rs` L241 の索引の突合も飛ぶ。
- つまり今は、2 体目のゴーストでは位置の索引が無い。設定で決めるなら、この 1 か所の条件を「設定が有効なら持つ」に変える必要がある。この変更は、待ち受けなしで持つソースマップを、ほかの消費者（ブレーク・ステップ）が使わないことを確かめてから入れる（消費者はデバッグの持ち手を通して使うので、影響は無い見込み。設計で確かめる）。
- 待ち受けで決めるなら、Rust 側の直しは要らない。ただし、2 体目では 2 つのイベントが黙って 204 になる。
- **決定（2026-10-22 要件の話し合い）**: 設定で決め、この 1 か所を直す。必要になったときに後から作る方式は採らない（書き換えたあとの辞書から作ると、読み込んだ中身と食い違うため）。呼ぶ前に SHIORI をリロードするのは呼び出し側の務めとする（要件 6.3）。

### 3.2 Lua への口の出し方

今の登録の形は `module_registry.rs` の `set_loaded_module(lua, "@pasta_xxx", ...)`（`@pasta_config`・`@pasta_log` など）。考えられる形:

- **(a) 新しいモジュール**（例 `@pasta_debug`）: `enabled()` と `scene_at(file, line)` を持つ。デバッグが無効なら `enabled()` は偽で、`scene_at` は常に nil。登録は `factory.rs` の `register_*` の並びに 1 行。
- **(b) 今のモジュールに足す**: `@pasta_config` に有効の印を足す（pasta.toml の表をそのまま映すモジュールなので、環境変数の結果を混ぜると意味がずれる）、か `@pasta_search` に足す（検索の責務からずれる）。

(a) が責務の分け方として素直。`ponytail-lua-loader`・`ponytail-lua-debug` が後から `runtime/`・`debug/` を触るので、口を小さく保つほど後の合わせが楽になる。

### 3.3 位置から引いたあとのシーン関数

- `scene_at` は登録名の組 `(scene_id, parent)` を返す。大域なら `SCENE.get_start(scene_id)`、ローカルなら `SCENE.get(parent, scene_id)` で関数が取れる。
- `KICK.try_dispatch`（`kick.lua` L115〜161）が同じ取り出しとコルーチン化（`wrap_local_func`）をしているが、`kick_pending` の消費と結びついている。使い回すには小さな取り出し関数に分ける必要があり、キックのファイルに触ることになる（キックの動きは変えない）。新しいハンドラの中で同じ数行を書く案もある。
- `scene.lua` にも同じラッパー（`wrap_scene`、L184〜）がある。ローカルを名前で引く口を `scene.lua` に足すなら、そのラッパーで足りる。

### 3.4 `raise_event` が GET か NOTIFY か

- SSP の MCP サーバーのツールの説明は「SSTP NOTIFY と同じく Event ヘッダでイベントを送り、返った台本を再生し、結果に載せる」。
- pasta の Rust 側は、SHIORI の NOTIFY にはすぐ 204 を返す。ただし中身は捨てずに VM で処理し、応答だけを捨てる（`pasta_shiori` の `actor/thread.rs` L204〜210）。NOTIFY で届くなら、ハンドラは動いて進行中の会話を差し替えるが、1 ビート目は捨てられ、2 ビート目からが後で出る。brief の「NOTIFY には中身を見ずに 204」は、この点で正確でない。
- 台本を再生するという説明からは、SSP が SHIORI に GET で送ると見られる。設計のときに実機で確かめる（Research Needed）。

### 3.5 同じ名前のイベントの今の動き（未決事項 3 に効く）

- 今は `OnPastaScene` に `REG` のハンドラが無く、`EVENT.no_entry` → `SCENE.co_exec(act, "OnPastaScene")` に進む。この道は前方一致なので、`＊OnPastaScene…` で始まる名前のシーンを書いたゴーストだけが今と変わる（`OnPastaSceneAt` は `OnPastaScene` の前方一致に当たる名前でもある）。
- 先に入った spec の名前の規則（`scene-exact-match` の brief L110）は、`OnPastaScene`・`OnPastaSceneAt` を英大文字の続きの組の例として確かめるとしている。

### 3.6 失敗の表記

- `FAILURE.report(act, record)` は、ログの警告を出し、`[logging] balloon_warning` が真なら表記を act のトークンに積む。ハンドラは積んだあと `act:build()` の文字列を返せば、200 の台本として MCP の結果に載る。設定が偽なら表記は積まれず、出力が空なら 204 になり、ログだけが残る。
- 表記の形は `【種類：対象（説明）】`。種類の名前は `FAILURE.LABEL` に足していく決まり（`SCENE_NOT_FOUND = "シーンが見つからない"` がある）。brief の例の文言（`OnPastaScene: シーン「会話・挨拶」が見つかりません`）とは形が違う（未決事項 5）。

## 4. 実装の進め方の案

### 案 A: 今の部品を広げる（新しいファイルは最小）

- `virtual_dispatcher.lua` か `kick.lua` に 2 つのイベントの処理を足す。
- 利点: ファイルが増えない。
- 欠点: `virtual_dispatcher.lua` は `ontalk-suppression` と同じウェーブで触らない約束がある（brief の再測定）。キックの動きは範囲外。採りにくい。

### 案 B: 新しいハンドラのファイルと新しい口（推奨の候補）

- `pasta_scripts/pasta/shiori/event/` に新しいファイル（例 `scene_play.lua`）を置き、`REG.OnPastaScene`・`REG.OnPastaSceneAt` を `REG.OnX = REG.OnX or ...` の形で定義する。`event/init.lua` L78〜81 に `require` を 1 行。
- `scene.lua` に、親を通して名前どおりに引く口を足す（`co_exec_exact` に省略できる引数を足すか、別の関数）。
- Rust に Lua の口（例 `@pasta_debug`: 有効の印と位置の引き当て）を足す。相対パスは `base_dir` に結んでから `uri_to_pasta_path` に渡す。
- 「設定で決める」を採るなら、`runtime/mod.rs` の 1 か所の条件を直す。
- 利点: キック・ディスパッチャに触らず、同じウェーブの spec とぶつからない。テストを分けやすい。
- 欠点: 位置からの関数の取り出しが `kick.lua` と数行重なる。

### 案 C: 混ぜる

- 案 B に加えて、`kick.lua` の「登録名からシーンのコルーチンを作る」数行を小さな関数に分けて、両方から使う。
- 利点: 重なりが無くなる。
- 欠点: キックのファイルに触る（動きは変えないが、範囲の外の差分が増える）。`chain-talk-interrupt` が後でキックと `OnPastaScene` の道の両方に手を入れるので、共通の関数があると向こうは楽になる面もある。

## 5. 規模と危険度

- **規模: M（3〜7 日）**。どれも今の型に沿うが、Rust と Lua の両側に口を足し、テストとマニュアルが要る。brief の再測定の見込み（7〜8 タスク）と合う。
- **危険度: 低〜中**。
  - 低: 部品はそろっていて、1 ビート目と続きの動きは今の仕組みのまま。
  - 中: (1) デバッグが無効なときに何も変えないこと（新しいハンドラが常に `REG` に入るので、無効なときの戻り値を確かめるテストが要る）。(2) `raise_event` が GET で届くかは実機でしか分からない。(3) 「設定で決める」なら `runtime/mod.rs` の条件を変えるので、待ち受けなしでソースマップを持つことの影響を確かめる。

## 6. 設計への申し送り

- **推奨の向き**: 案 B（必要なら案 C の小さな共通化）。決める事柄は次のとおり。
  1. デバッグの有効の決め方（設定か待ち受けか）と、設定なら `runtime/mod.rs` L267〜271 の直し方。
  2. Lua への口の形（新しいモジュールか、今のモジュールに足すか）と、`scene_at` の返す形（登録名の組）。
  3. 相対パスを `base_dir` に結ぶ場所（Rust の口の中か、Lua のハンドラか）。
  4. ローカルを名前で引く口の形（`co_exec_exact` を広げるか、新しい関数か）と、同名の大域が複数あるときの選び方（未決事項 2）。
  5. 失敗の表記の種類の名前と文言の形（未決事項 5）。
  6. `kick.lua` の取り出しを共通にするか。
- **Research Needed**
  - SSP の MCP サーバーの `raise_event` が SHIORI に GET で届くか、NOTIFY で届くか（実機）。
  - そのとき SSP が送る `SecurityLevel` の値（実機）。
  - 待ち受けなしでソースマップを持ったとき、ほかの消費者に影響が無いこと（コードで確かめる）。
- **同じウェーブ・後の spec との接点**
  - `ontalk-suppression`: `virtual_dispatcher.lua`・`shiori/entry.lua`・`store.lua` に触らなければ並べられる。案 B はどれにも触らない。
  - `external-control-events`・`chain-talk-interrupt`: `event/init.lua` と新しいハンドラのファイルに後から入る。新しいファイルの形を、後の spec が真似しやすい形にしておく。
  - `ponytail-lua-debug`・`ponytail-lua-loader`: `debug/`・`runtime/` に後から入る。口を小さく保つ。

---

# 設計の調べ（2026-10-22・設計フェーズ）

ここから下は、設計（`design.md`）のために足した調べである。上のギャップ分析は、そのまま残している。

## 7. まとめ

- **機能の種類**: 今の仕組みの拡張（light discovery）。新しい外部の依存は無い。
- **使ったスキルと手順**: `kiro-spec-design`（`design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）。伺かの仕様は、伺かの文書検索（UKADOC・SSP ヘルプ）で調べた。SSP の実機は動かしていない。
- **大事な発見**:
  1. 失敗の種類の表は、今のテストが「10 種」と数えている（`tests/lua_specs/failure_test.lua` L344〜348）。種類を足すと、今のテストを書き換えることになり、要件 7.3 に反する。引数の誤りは、今ある種類（`名前にならない値`・`数にできない値`）で表せる。
  2. 「そのローカルを持つ大域から順送りで選ぶ」（要件 1.8）は、今の検索の部品では作れない。順送りの状態（`SceneTable` の `select_from_cache`）が `pasta_core` の中に閉じているためである。`SceneTable` に関数を 1 つ足すのが、いちばん小さい。
  3. 待ち受けが立たなかったときにソースマップを持っても、ブレーク・ステップ・ソースの提示には影響しない。これらは `debug::enable` の中で、待ち受けが立った後にだけマップを受け取る。

## 8. 調べた事柄

### 8.1 raise_event は GET か NOTIFY か（Research Needed）

- **調べた文書**: SSP ヘルプ「その他の機能」の MCP サーバーの節（`ssphelp:other`）、UKADOC の SHIORI/3.0 の「メソッド」（`spec_shiori3`）、SSTP の request の共通ヘッダ（`spec_sstp`）、SSP の MCP ツール `raise_event` の説明。
- **分かったこと**:
  - SSP ヘルプは `raise_event` を「SHIORI イベントを発生させ（SSTP NOTIFY の Event 相当）、ゴーストが返したスクリプトを再生する。返されたスクリプトは結果にも含まれる」と書く。
  - SHIORI/3.0 の NOTIFY は「レスポンスの Value ヘッダは無視される」。返した台本を再生するには、SSP は SHIORI に GET で送る必要がある。
  - SSTP の `Option: strict` の説明に「NOTIFY/GIVE でゴーストが返したスクリプト」とある。SSTP の NOTIFY でも、ゴーストの返した台本が再生される。これも、SHIORI へは GET で届くことと合う。
  - SSP ヘルプは、MCP サーバーを「ローカル専用の機能」とし、送ったスクリプトを「対象ゴースト自身の処理（Owned SSTP）と同様に扱」うと書く。Owned SSTP は `SecurityLevel` を `local` にする（`spec_sstp` の ID ヘッダ）。`raise_event` の `SecurityLevel` も `local` の見込みが高い。
- **結論**: 文書からは「GET・`local`」と強く推せるが、言い切れない。設計は GET を前提にする。実機での確かめは、設計の話し合いの未決事項に残す。
- **NOTIFY だった場合**: pasta は NOTIFY でもハンドラを動かし、応答を捨てる（`pasta_shiori` の `actor/thread.rs` L204〜210）。そのため、シーンは差し替わるが 1 ビート目は捨てられ、MCP の結果にも何も載らない。直すなら、NOTIFY のときは 1 ビート目を捨てない別の道（例: `KICK.install` に渡して次の OnSecondChange で出す）を設計し直す。MCP の結果に台本は載らないままである。

### 8.2 待ち受けなしでソースマップを持つことの影響

- **調べたコード**: `runtime/mod.rs` L257〜271、`debug/enable.rs` L98〜160、`runtime/exec.rs` L58〜87、`runtime/factory.rs` L241〜258、`pasta_shiori/src` 全体の `source_map`・`debug_enabled` の利用。
- **分かったこと**:
  - ブレークポイントの翻訳・ステップ・ソースの提示の 3 つの消費者は、`debug::enable` が待ち受けを立てた後に、`SourceMapWiring` とセッションへ渡したマップだけを使う。bind に失敗した `enable` はマップを渡す前に `Ok(None)` を返すので、消費者そのものが作られない。
  - `runtime.source_map` を読むのは、索引の突合（`factory.rs` L241）と `debug_source_map()` だけである。`debug_source_map()` を呼ぶのはテストだけで、`pasta_shiori` は呼ばない。
  - ローダーはデバッグが有効なときだけマップを作る（`loader/mod.rs` L217）。無効なら `source_map` は初めから `None` である。
  - bind の失敗を扱う今のテスト（`debug/debug_mod_tests.rs` の `enable_bind_failure_*`）は `enable` を直接呼び、ランタイムのマップは見ない。マップが `None` であることを確かめるテスト（`source_map_handoff_test.rs` L149 など）は、どれもデバッグが無効のときである。
- **結論**: 条件を「`debug_handle` があるとき」から「設定でデバッグが有効なとき（`config.debug.enabled`）」に変えても、ほかの消費者と今のテストに影響は無い。

### 8.3 そのローカルを持つ大域から選ぶ（要件 1.8）

- **調べたコード**: `pasta_core/src/registry/scene_table.rs`（`resolve_scene_id_exact`・`select_from_cache`・`SceneCacheKey`・`prefix_index`）、`pasta_lua/src/search/context.rs`（`search_scene_by`）。
- **分かったこと**:
  - 大域シーンの索引のキーは書いた名前を直した形（`会話`）で、同じキーに `会話_1`・`会話_2` の ID が並ぶ。ローカルシーンのキーは `:会話_1:挨拶` の形で、親の登録名を含む。
  - 順送り（重ならない順）の状態は `SceneTable` の中のキャッシュにあり、外からは使えない。
  - Lua の側で候補を絞るには、名前の直し方と別名の当て方を Lua に写すことになり、要件 1.5（同じ規則）を守れない。
- **結論**: `SceneTable` に「その名前の大域のうち、そのローカルを持つものだけを候補にして順送りで選ぶ」関数を 1 つ足す。キャッシュの鍵は、ふつうの完全一致の鍵と混ざらないようにする（混ざると、片方の候補の表をもう片方が使ってしまう）。

### 8.4 位置から引いた結果の形

- `SourceMap::scene_at` は `SceneIdentity { scene_id, parent }` を返す（大域は `("会話_1", None)`、ローカルは `("挨拶_1", Some("会話_1"))`）。
- 名前の検索（`search_scene_exact`）は `(大域の登録名, ローカルの登録名)` を返す（大域は `("会話_1", "__start__")`）。
- Lua の口では、位置の結果も名前の検索と同じ `(大域の登録名, ローカルの登録名)` に揃える。そうすると、どちらの道も `SCENE.get(大域, ローカル)` の 1 本で関数が取れる。

### 8.5 相対パスと base_dir

- `LoaderContext.base_dir` は pasta.toml のあるフォルダ（`ghost/master/`）で、`factory.rs` L176 で `runtime.base_dir` に入る。
- `uri_to_pasta_path` は `std::path::absolute` で SSP の今のフォルダに結ぶので、相対パスは先に `base_dir` に結ぶ。結ぶかどうかは `Path::is_absolute()` で決める。Windows で `\dic\a.pasta` のようなドライブの無い根からのパスは、std の規則で `base_dir` のドライブの根に結ばれる（ゴーストの中を指さないので、見つからないになる）。
- 引くのは読み込みのときに作った索引だけで、ファイルは開かない。どんなパスを渡されても、ファイルの中身が漏れることは無い。

### 8.6 行の値の範囲

- Lua の数は倍精度の浮動小数点数で、`"99999999999999999999"` も `tonumber` で数になる。Rust の口で `u32` に直すと、範囲の外は mlua の変換エラー（500）になる。
- Lua のハンドラは「半角の数字だけの、1 以上の数」だけを通し、Rust の口は数を受けて、`1..=u32::MAX` の整数でなければ「見つからない」（nil）を返す。行が 2^32 を超えるファイルは無いので、意味の上でも正しい。

## 9. 方式の比べ

| 案 | 中身 | 良い点 | 悪い点 | 採否 |
| --- | --- | --- | --- | --- |
| A | `virtual_dispatcher.lua` か `kick.lua` に足す | ファイルが増えない | 同じウェーブの約束（`virtual_dispatcher.lua` に触らない）とキックの範囲に触れる | 採らない |
| B | 新しいハンドラのファイル＋新しい口 | 同じウェーブの spec とぶつからない。後の spec が真似しやすい | `kick.lua` と数行が重なる | **採る** |
| C | B＋`kick.lua` の取り出しを共通にする | 重なりが無くなる | キックのファイルに触る。範囲の外の差分が増える | 採らない（重なりは `scene.lua` の今のラッパーで吸収する） |

## 10. 設計の決定

### 決定: Lua への口は新しいモジュール `@pasta_debug`

- **背景**: Lua からデバッグの有効と、位置からの引き当てを知る口が無い（要件 4.1〜4.3・2.1〜2.5）。
- **候補**: (1) 新しいモジュール `@pasta_debug`、(2) `@pasta_config` に足す、(3) `@pasta_search` に足す。
- **採った案**: (1)。関数は `enabled()` と `scene_at(file, line)` の 2 つだけ。
- **理由**: `@pasta_config` は pasta.toml の表をそのまま映す（環境変数の結果を混ぜると意味がずれる）。`@pasta_search` は検索の責務。後の `ponytail-lua-debug` が合わせる面を小さく保てる。
- **得失**: モジュールが 1 つ増える。作者も `require` できてしまうが、内部用として内部設計の章にだけ書く（作者向けのモジュールの章には載せない）。

### 決定: 相対パスは Rust の口の中で base_dir に結ぶ

- **理由**: `base_dir` は Rust の側にあり、Lua に渡さずに済む。結ぶ処理と `uri_to_pasta_path` を 1 つの純粋な関数にまとめ、単体テストにできる。

### 決定: kick.lua は触らない。scene.lua に小さな口を 2 つ

- `SCENE.co_exec_exact(name, local_name)` に省略できる 2 つ目の引数を足し、見つからないとき「どちらが無いか」を 2 つ目の戻り値で返す。
- `SCENE.co_exec_registered(global_name, local_name)` を足す（登録名の組からコルーチン。今の `wrap_scene` を使う）。
- キックと同じ取り出しの数行は、`scene.lua` の今のラッパーで足りるので、`kick.lua` との共通の関数は作らない。`chain-talk-interrupt` が後でまとめたくなったら、`SCENE.co_exec_registered` を使える。

### 決定: 引数の誤りに新しい種類の名前を足さない

- **背景**: 要件 5.5 の申し送り（種類の名前を足すか）。
- **採った案**: 今ある種類を使う。Reference0 が無い・空 → `名前にならない値`（説明は `値なし`・`空文字列`）。`OnPastaSceneAt` の Reference1 が 1 以上の整数でない → `数にできない値`（説明は `値なし`・`空文字列`・`"４２"` のような引用符付きの値）。
- **理由**: 種類の表を「10 種」と数える今のテストを書き換えずに済む（要件 7.3）。種類の意味（名前にならない・数にできない）とも合う。
- **得失**: `"0"` は数だが行にならない値で、`数にできない値` とは少しずれる。説明に値をそのまま出すので、読み手は直し方が分かる。

### 決定: 失敗の表記の対象の形

- 対象は「イベント名、半角の空白、探したもの」にする。
  - 大域が無い: `【シーンが見つからない：OnPastaScene ＊会話】`
  - ローカルが無い: `【シーンが見つからない：OnPastaScene ＊会話 ・挨拶】`
  - 位置に無い: `【シーンが見つからない：OnPastaSceneAt dic/boot.pasta:42】`
  - 引数の誤り: `【名前にならない値：OnPastaScene Reference0（値なし）】`、`【数にできない値：OnPastaSceneAt Reference1（"４２"）】`
- 大域とローカルの間を空けるのは、大域の名前に `・` が入りうる（`＊会話・朝`）ためである。辞書の印（`＊`・`・`）を付けて、どちらの名前かを示す。

### 決定: 選び方の状態

- ローカルを省いたとき（要件 1.1・1.6）は、今の `search_scene_exact(名前, nil)` をそのまま使う。順送りの状態は、つつきのショートカットなど、ほかの完全一致の検索と同じものを使う（ふだんと同じ選び方）。
- ローカルを指名したとき（要件 1.8）は、そのローカルを持つ大域だけの順送りで、状態は別に持つ。

### 一般化・採否・簡素化（design-synthesis）

- **一般化**: 名前の道と位置の道は、どちらも「登録名の組を得る → `SCENE.get` → コルーチンを返す」の同じ形になる。口の戻り値を `(大域の登録名, ローカルの登録名)` に揃えて、ハンドラの後半を 1 本にした。
- **採否**: 新しいライブラリは使わない。順送り・別名・名前の直し方・位置の索引・失敗の表記・続きの流し方は、どれも今の部品を使う。
- **簡素化**: 足さないもの — 引数の誤りの新しい種類の名前、キックとの共通の関数、`@pasta_debug` の作者向けの章、NOTIFY のための別の道（GET が確かめられるまで作らない）、必要になったときに索引を作る方式（要件の話し合いで採らないと決めた）。

## 11. 危険と対策

- `raise_event` が NOTIFY で届く — 実機で確かめるまで GET を前提にする。NOTIFY なら 8.1 の別の道を設計し直す（未決事項）。
- デバッグが無効なときに何かが変わる — 2 つのイベント以外のハンドラは足さない。無効のときの 204 とログをテストで確かめる。
- 順送りのキャッシュの鍵が混ざる — 鍵に「持ち主のローカルの名前」を含め、ふつうの完全一致と別の鍵にする。混ざらないことを単体テストで確かめる。
- `@pasta_debug` が無いランタイム（`PastaLuaRuntime::new` などの簡易な入口、Lua の単体テストの実行器） — ハンドラは `pcall(require, "@pasta_debug")` で読み、無ければ無効として扱う。
- 同じ名前のシーンが別のファイルにあるときの位置の索引の制約 — 受け継ぎ、直さない（範囲外）。

## 12. 参考

- SSP ヘルプ「その他の機能」MCP サーバー対応機能: https://ssp.shillest.net/ukadoc/ssphelp/other.html
- UKADOC SHIORI/3.0 メソッド・SecurityLevel: https://ssp.shillest.net/ukadoc/manual/spec_shiori3.html#request
- UKADOC SSTP request の共通ヘッダ（SecurityLevel・Option strict・ID）: https://ssp.shillest.net/ukadoc/manual/spec_sstp.html#req_res

## 13. 設計の話し合いの決定（2026-10-22）

設計の未決事項 6 件は、すべて推奨案で確定した（`design.md` の Decisions）。根拠の要点:

- GET・`local`: UKADOC の SHIORI/3.0 は「NOTIFY ではレスポンスの Value ヘッダは無視される」とする。SSP ヘルプは `raise_event` が返したスクリプトを再生し、MCP の操作を Owned SSTP と同様に扱うとする。実機での確かめは実装の最後に行う。
- pasta は SHIORI の応答に `ErrorLevel` を載せないので、警告は SSP の MCP の `get_log` に出ない。AI エージェントが読むのは、応答の表記と pasta のログファイルである。
