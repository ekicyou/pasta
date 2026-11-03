# ギャップ分析: debug-mcp-runtime

調べた版: main `96b75cbf`（brief の再測定の `c575febd` の 1 つ後。差分は roadmap だけ）。閉じた `ponytail-lua-debug` のノート 2 つ（`side-session-notes\local_0e05a9a5-…`）はこの機械に無かった。代わりに、そのブランチ `claude/kiro-start-ponytail-lua-debug-49bccb`（`577548ec`）の research.md（N1〜N13）を読んだ。

## Summary

- **範囲**：撤去が 8 割。`pasta_lua/src/debug` 22,423 行のうち、残すのは約 3,100 行（`config.rs`・`playscene.rs`・`source_map/` とそのテスト）。キックは Rust 4 か所・Lua 4 ファイル・テスト 14 本。新しく足すのは `OnPastaDumpVariables` と、止まるエラーの位置の小さな直しだけである。
- **brief の 1 つ目の穴は本当だった**：止まるエラーの位置は、行の対応でもう出ている。ソースマップの新しい使い手（Approach 4）が要るのは、行の対応で引けない行（別のグローバルシーン・別のファイルの Lua のブロックの関数）だけである。加えて、別のファイルのときに**違うファイルの行を出す**誤りを見つけた（`stop.lua` 90〜99 行）。
- **波及の大きいところ**：`spawn_actor_thread_with`（`pasta_shiori/src/actor/thread.rs`）が kick の送り先を引数に取り、kick と関係の無いテストが 8 本使う。`PASTA_DEBUG` の中和の `#[ctor]` は 17 ファイルに写しがある。どちらも消すだけだが、本数が多い。
- **推奨**：Option A（今の部品の中で消し、残す。`OnPastaDumpVariables` は `scene_play.lua` の門を共有する）。規模は M〜L、危険は中の下。brief の見積もり 12〜16 タスクに収まる。
- **議題**：requirements.md の末尾の議題 1〜10 は、要件ディスカッション（2026-11-03）で全部を推奨案で確定した。設計で調べる項目は下の「Research Needed」。

## brief の今の状態の確かめ

| brief の記述 | 確かめた結果 |
| ------------ | ------------ |
| `crates/pasta_lua/src/debug` は 79 ファイル・約 22.4k 行 | そのとおり（79 ファイル・22,423 行）。残すもの（`config.rs` 221・`playscene.rs` 213・`playscene_tests.rs` 366・`source_map/` 1,435・`source_map_builder_tests.rs` 143・`source_map_resolve_tests.rs` 608・`source_map_test_support.rs` 29・`mod.rs` 73）は計 3,087 行。サイドカー（`source_map/sidecar.rs` 145・`source_map_sidecar_tests.rs` 247）は消す側 |
| `kick_sink_for` は `actor/instance.rs` 76 行、使う所は 139・284 行 | そのとおり。139 行は `ActorInstance` の起動、284 行は同じファイルの単体テスト。ほかに使い手は無い |
| `ActorMsg::Kick`（`mailbox.rs`）・Kick の腕（`thread.rs`）・`PastaShiori::kick`（`shiori.rs`） | そのとおり（`mailbox.rs` 99 行・テスト 147〜155 行、`thread.rs` 204〜207 行、`shiori.rs` 337 行〜）。加えて `PastaShiori::with_kick_sink`（48 行）・`load` の 124 行、`thread.rs` 110〜144 行の引数、`marshaling.rs`・`teardown.rs` の説明、`tests/actor_test_harness.rs` 153 行の腕がある |
| `DebugConfig` のキー | 構造体の欄は `enabled`・`listen`・`source_mode`・`source_map_sidecar`（`config.rs` 24〜44 行）。`pasta.toml` のキーは `enabled`・`port`・`present_as`・`source_map_sidecar`（`loader/config/sections.rs` 299〜320 行）。env は `PASTA_DEBUG`・`PASTA_DEBUG_PORT`・`PASTA_DEBUG_SOURCE_MODE`・`PASTA_DEBUG_SOURCE_MAP_SIDECAR`（`config.rs` 150〜173 行）。`resolve` の引数は 8 つ（97 行の `too_many_arguments` の許可） |
| 古いキーは「知らないキー」の扱いに合わせる | `[debug]` は `PastaConfig::get_custom_config`（`loader/config/mod.rs` 246〜250 行）で serde の `try_into` を通る。`deny_unknown_fields` は pasta_lua・pasta_shiori のどこにも無く、知らないキーは黙って読み飛ばす。**注意**：型の合わない値（`enabled = "yes"`、今なら `port = "x"` も）は節ごと `None` になり、黙って無効になる。`port` の欄を消せば、型の合わない古い `port` で無効になる穴は消える |
| `PASTA_DEBUG` の中和は `pasta_sample_ghost` の `tests/common/mod.rs:7-13`・`Cargo.toml:25` | そのとおりだが、それだけではない。`#[ctor]` で `PASTA_DEBUG`・`PASTA_DEBUG_PORT` を消す写しは **17 ファイル**ある（`pasta_lua/tests/common/mod.rs`・`pasta_shiori/tests/common/mod.rs`・`pasta_sample_ghost/tests/common/mod.rs`・`pasta_check/tests/validate_parity_test.rs`・`scene_identity_index_test.rs`・`actor_marshaling_test.rs` など）。`ctor` の dev 依存は 4 クレート |
| 中和は固定ポートが無くなれば要らない | **要る**。`PASTA_DEBUG=1` はポートが無くなっても開発用のイベントとソースマップを有効にする。`scene_play_event` 系の「無効なら 204」、`external_control_events_test.rs` 74 行の `!runtime.debug_enabled()` などが環境で変わる。`PASTA_DEBUG_PORT` の行だけ要らなくなる（議題 3） |
| ソースマップの使い手は DAP と `@pasta_debug.scene_at` だけ | そのとおり。`resolve_lua_to_pasta` を debug の外から呼ぶのはテストだけ（`code_gen/source_map.rs` の説明の参照を除く） |
| 実行時エラーの文言・バルーンにソースマップは使われていない | そのとおり。ただし**位置はもう出ている**（次の節）。brief の Problem の「作者は `.pasta` の行へ手で戻している」は、行の対応で引けない行だけに当てはまる |
| `scene_play.lua` の `allowed()` が `@pasta_debug.enabled()` を見る | そのとおり（20〜32 行）。`enabled()` は待ち受けの有無でなく設定の値（`factory.rs` 77 行・`module_registry.rs` 191〜224 行）。一方、ランタイムの `debug_enabled()`（`runtime/exec.rs` 63 行）は `debug_handle.is_some()`＝待ち受けの有無で、2 つが食い違う（`scene-play-event` の申し送りの「混ぜない」） |
| `OnPastaCallFunction`・`OnPastaSetSaveVariable` に門が無い | そのとおり（`external_control.lua` 8 行。external だけ断る） |
| 保存変数の書き出しの道具 | `persistence.rs` 330 行の `lua_table_to_toml_document` が、書けないキーを `DroppedKey` で返す（`save_table` 210〜230 行が警告を出す）。これを使えば「書けない値を飛ばしてログに残す」はセーブと同じになる。`@toml` は `decode` だけで、`encode` は無い（`module_registry.rs` 164 行） |
| `pasta_check validate` の組み込みの名前 | 直す要が無い。`validate/builtins.rs` は Call の飛び先（GLOBAL の関数・act のメソッド）の表で、`On` で始まる飛び先は見ない（`validate/references.rs` 109 行）。`pasta_check` は `[debug]` も検査しない |
| DAP だけが使う依存 | `socket2` は `pasta_lua` だけが使い、`src/debug/` の外に使い手が無い（`Cargo.lock` で `socket2` に頼るのも `pasta_lua` だけ）。`serde_json` は `pasta_lua` では `src/debug/` と `tests/runtime/` の DAP のテストだけ、`pasta_shiori` の dev では `debug_preservation_test.rs` だけ。ルートの `serde_json` は `pasta_check`・`pasta_lsp` が使うので残る |
| マニュアルの頁 | brief の一覧のほかに、`internals/transpiler.md`（27・366 行。`debug.md` への参照とサイドカー）・`internals/logging-encoding.md`（149・158 行。デバッグバックエンドのスレッドのログ）・`reference/runtime-failures.md`（場所の表）がある。`introduction.md` 19 行・`grammar/call-jump.md` 465・547 行は利用者の章へのリンクで、`vscode-mcp-client` の見出しに合わせるだけ |
| 章の数 | `ponytail-manual-ci` が目次から数えるようにした。`internals/debug.md` の題は `SUMMARY.md` 82 行 |

## Approach 4 の確かめ（止まるエラーの位置）

**今の道**：止まるエラーは全部 `STOP.response`（`shiori/stop.lua` 179 行）を通る。入口は `entry.lua` 99 行（要求の中のエラー）と `event/callback.lua` 184・189 行（非同期の続き）。`STOP.detail`（127 行）がエラー文の最初の行の「チャンク:行:」を `STOP.location`（110 行）で直し、表記とログの警告の両方に出す。`X-Error-Reason` は元のエラー文のまま。

**行の対応**：`transpiler.rs` の `fill_line_maps`（259 行）が、各シーン関数の `act:init_scene(…)` の最後の引数に `"<.pasta>|<lua の行>:<pasta の行>,…"` を埋める。デバッグの有無にかかわらず埋まる。範囲は `scope_gen.rs` の `LineMapSlot`（`code_gen/mod.rs` 74 行）で、そのシーン関数の本体と、同じグローバルシーンの Lua のブロック（`scope_gen.rs` 183〜187 行）。`act:call` の後の `restore_scene`（`act.lua` 755〜762 行）が、戻った後に呼び出し前の行の対応へ戻す。

**引けない行**：

| 止まった所 | 今の表示 | ソースマップ（デバッグが有効なとき）なら |
| ---------- | -------- | ---------------------------------------- |
| シーン関数・同じグローバルシーンの Lua のブロック | `dic\天気.pasta:5:`（正しい） | 同じ |
| アクター辞書の単語定義・ファイルの単語定義（`ACTOR:create_word(…):entry(…)`） | — 実行時のエラーはここから出ない（読み込みのときに走るデータの行） | — |
| 同じファイルの別のグローバルシーンの Lua のブロックの関数 | 場所なし（マニュアル 193 行のとおり） | 引ける |
| 別のファイルの Lua のブロックの関数（`GLOBAL.名前 = function` など） | **番号が重なれば、実行中のシーンのファイルの違う行**。重ならなければ場所なし | 引ける |
| 作者の `.lua`（`scripts/`） | `scripts\天気.lua:6:`（正しい） | 対象外 |
| 読み込みのときの Lua のブロックの構文の誤り・トップレベルの文 | 読み込みの失敗として、キャッシュの行（`error loading module … from file '…'`） | 引ける（ソースマップは Phase 5.5 で VM より先にできる）が、道が別（議題 8） |

**違うファイルの行を出す理由**：`STOP.location` は、チャンクが「辞書から変換した Lua」（`@pasta_chunks` の値が `true`。`runtime/searcher.rs` 208〜260 行）なら、`pasta_location(line_map, line)` で行の番号だけを引く。どの `.pasta` から変換したチャンクかを `@pasta_chunks` が持たないので、行の対応のファイルと比べられない。

**結論**：Approach 4 は落とさず、「行の対応で引けない行を、デバッグが有効なときだけソースマップで引く」に絞る（要件 6.3・議題 1）。違うファイルの行を出す誤りは、デバッグの有無によらず直す（要件 6.2・議題 2）。`LineShift` は今の生成ではふつう恒等なので、行の対応とソースマップは同じ行を指す（brief の再測定のとおり）。

## Requirement-to-Asset Map

| 要件 | 今ある資産 | ギャップ |
| ---- | ---------- | -------- |
| 1 待ち受け・行フックを無くす | `debug/enable.rs`・`hook.rs`・`transport/`・`dap/`・`session/`・`wiring/`・`breakpoints.rs`・`inspect.rs`・`handle.rs`・`error.rs`・`types.rs`・`source_mode.rs`。`runtime/mod.rs` 211〜259 行の `debug::enable`・`debug_handle` | **Missing**：撤去そのもの。**Missing**：有効な 2 体が並ぶことの確かめ（`multi_instance_test.rs` の DAP の部分を書き直す）。**Missing**：起動の info の 1 行（議題 6）。**Constraint**：JIT が止まらないことの確かめ方（`jit.status()` が VM から見えるか。Research Needed） |
| 2 キックを無くす | Rust：`ActorMsg::Kick`・`thread.rs` の腕と `kick_sink` 引数・`PastaShiori::kick`・`with_kick_sink`・`kick_sink_for`・`RuntimeConfig.kick_sink`・`with_kick_sink`・`debug/kick.rs`。Lua：`event/kick.lua` 167 行・`entry.lua` 12・112〜121 行・`virtual_dispatcher.lua` 246〜272 行・`store.lua` 78・82・105・106 行 | **Missing**：撤去。**Constraint**：`spawn_actor_thread_with` の引数が変わり、kick と関係の無いテスト 8 本（`actor_log_context`・`actor_marshaling`・`actor_reload_leak`・`actor_teardown`・`actor_thread_vm`・`ffi_logging`・`runtime_error_balloon_e2e`・`shiori_lifecycle_tests.rs`）が直る。**Constraint**：`STORE.co_preempt`（`store.lua` 22・70・108 行、`event/init.lua` 129 行）は `scene_play.lua` が使うので残す。説明の「キック・シーン再生」だけ直す |
| 3 設定を 1 つにする | `DebugConfig`・`DebugFileConfig`・`default_debug_port`（`lib.rs` 56 行・`loader/mod.rs` 38 行で再輸出）・`RuntimeConfig::with_debug_from_file_and_env`・`loader/mod.rs` 205〜232 行（Phase 5.5）・`factory.rs` 61〜78 行 | **Missing**：欄とキーと env の整理。**Constraint**：`[debug]` は `custom_fields` に残り、`@pasta_config` から Lua に見える（消さない）。**Constraint**：`loader/mod.rs` と `factory.rs` が別々に `from_env` を呼ぶ（2 回の解決。同じ値になるが、1 回にできるか設計で見る） |
| 4 開発用のイベントの門 | `scene_play.lua`（86 行）・`@pasta_debug`（`module_registry.rs` 191〜224 行）・`playscene.rs` の `resolve_position`・`uri_to_pasta_path` | **Missing**：`OnPastaDumpVariables` を門に入れる。**Unknown**：URI の扱いを外すか（議題 5）。外すなら `strip_file_scheme`・`percent_decode`・`hex_val`・`fix_leading_drive` と `playscene_tests.rs` の URI の分が消え、`std::path::absolute` だけが残る |
| 5 `OnPastaDumpVariables` | `pasta/save.lua`（`@pasta_persistence.load()` の表）・`persistence.rs` の `lua_table_to_toml_document`・`DroppedKey`・`save_to_file`（難読化あり） | **Missing**：イベントのハンドラ、書き出しの関数（難読化しない・一時ファイルからの置き換え・フォルダを作る）、マニュアルの節。**Constraint**：セーブの門（`SaveGate::Closed`。読めないセーブを確かめずに上書きしないための印）は書き出しに効かせない |
| 6 止まるエラーの位置 | `stop.lua` の `STOP.location`・`pasta_location`・`STOP.detail`、`searcher.rs` の `@pasta_chunks`、`source_map/mod.rs` の `resolve_lua_to_pasta`・`pasta_file_for_chunk`、`stop_test.lua` | **Missing**：チャンクがどの `.pasta` かを知る手段（6.2）。**Missing**：デバッグが有効なときのソースマップの引き（6.3）。**Unknown**：エラー文のチャンク名は 60 バイトで縮む（`LUA_IDSIZE`。`...` ＋末尾 56 バイト）。ソースマップのキーは正規化した全体のパスなので、縮んだ名前からどう引くか（Research Needed） |
| 7 残す部品を整える | 上の資産と、受け継いだ N4・N5・N6・N11・N12、`ponytail-lua-transpiler` の申し送り、`scene-play-event` の申し送り | **Missing**：整理そのもの。**Constraint**：`pasta_lua` は crates.io に出す。pub の型を消すのは README 88 行の断り書きで許される（議題 9） |
| 8 マニュアル | `internals/debug.md` 510 行（DAP・キックの記述 118 か所）・`internals/shiori.md`（30 か所）・`execution-model.md`（12）・`reference/pasta-toml.md`（12）・`internal-modules.md`（6）・`loader.md`（5）・`lua/shiori-events.md` 655〜745 行・`reference/runtime-failures.md` 176〜200 行・`DEBUGGING.md`（15 行。利用者の章へ案内するだけ） | **Missing**：書き直し。**Constraint**：スキルの references（`.claude/skills/pasta-ghost-authoring/references/pasta-toml.md`・`pasta-lua-coding/references/shiori-events.md`・`internal-modules.md`）は `node book/tools/gen-skill-refs.mjs` で作り直す。`.claude/skills/` への書き込みは分類器が拒むことがある（メモリ「.claude/skills 編集は分類器が拒否」）。拒まれたら、入った分と残りを示して開発者に頼む |
| 9 テスト | 消す：`pasta_lua/src/debug/` の DAP 系のテスト（約 14k 行の大半）、`pasta_lua/tests/runtime/` の `debug_break_coalesce_fixture_test`・`debug_integration_test`・`debug_integration_zero_cost_test`・`runtime_toggle_e2e_{basic,common,initial_mode,step}` の 7 本とフィクスチャ 3 つ（`debug_break_coalesce.pasta`・`debug_toggle_e2e.pasta`・`debug_toggle_step_e2e.pasta`）と `main.rs` の行、`pasta_shiori/tests/` の kick 6 本・`debug_preservation_test.rs` とフィクスチャ・`multi_instance_test.rs` の DAP の部分と `common/mod.rs` の `listening_port`、`lua_specs/` の kick 8 本と `init.lua` 51〜59 行、`loader_source_map_build_test.rs` のサイドカーの 2 本 | **Constraint**：kick を mock する Lua のテスト（`chain_talk_interrupt_test.lua` 23・46・438 行〜、`virtual_dispatcher_pause_gate_test.lua` 72〜79 行、`actor_proxy_act_delegation_test.lua` 22 行、`recommend_menu_test.lua` 421 行）は、kick の行だけ外す。`chain_talk_interrupt_test.lua` の `OnCtiKick` は中断の続きを捨てる印（`co_preempt`）を確かめるので、名前を直して残す。**Constraint**：`[debug] enabled = true, port = 0` を書くテスト（`scene_alias_wiring_test`・`source_map_handoff_test`・`scene_identity_index_test`・`scene_play_event_test`・`multi_instance_test`）は `port = 0` を外す（残しても読み飛ばされる） |

## Implementation Approach Options

### Option A: 今の部品の中で消し、残す（推奨）

- `pasta_lua/src/debug/` から DAP の部品を消し、`mod.rs`・`config.rs`・`playscene.rs`・`source_map/` を残す。`config.rs` は `enabled` だけを解決する形に縮める。
- `runtime/mod.rs` の `debug::enable` と `debug_handle` を消し、`source_map` は設定が有効なら持つ（今の 244〜249 行の分岐だけが残る）。`debug_enabled()` は設定の値を返す。
- `pasta_shiori` から kick の送り先の受け渡しを消す（`with_kick_sink` は `new` に、`spawn_actor_thread_with` の引数を 1 つ減らす）。
- `OnPastaDumpVariables` は `scene_play.lua` に置き、`allowed()` を共有する。書き出しは `@pasta_debug` に関数を 1 つ足し、中で `lua_table_to_toml_document` を使う（本番の入口だけが登録する今の決まりのまま）。
- 位置は、`@pasta_chunks` の辞書の印（`true`）を「どの `.pasta` か」が分かる値にし、`stop.lua` で行の対応のファイルと比べる。デバッグが有効なら `@pasta_debug` の関数でソースマップを引く。
- ✅ 新しいファイルがほぼ無い。モジュール名（`@pasta_debug`・`debug/`）とテストの置き場が今のまま
- ✅ 消す差分が大半で、残す部品のテスト（`source_map_*`・`playscene_tests`・`scene_index_tests`・`scene_join_tests`）がそのまま守りになる
- ❌ 「debug」という名前が、DAP の無い「開発用のイベント」の部品に残る（マニュアルの題だけ変える）

### Option B: 「開発用のイベント」の部品を新しく作る

- `debug/` を `dev/`（など）に作り直し、`@pasta_debug` を新しい名前にし、Lua の 3 つのイベントを新しいファイルに集める。
- ✅ 名前と中身が揃う
- ❌ 名前の変更がテスト・マニュアル・スキル・`vscode-mcp-client` の決まり（ログの文言など）に波及する。撤去の spec で名前まで動かすと、差分が読みにくい
- ❌ `scene_play.lua` を動かすと `scene-condition-attributes` などの後の spec と重なりやすい

### Option C: 混ぜる

- Rust は Option A のとおり `debug/` の中で消す。Lua だけ、3 つのイベントを `event/dev.lua`（など）に集め、`scene_play.lua` を畳む。
- ✅ Lua の門が 1 か所になる
- ❌ `scene_play.lua` 86 行に 1 つ足すだけで済むものに、ファイルの移動を足す。得が小さい

## Effort and Risk

- **Effort: L**（1〜2 週）。撤去は機械的だが、本数が多い（Rust のファイル 70 本超・テスト 30 本超・マニュアル 15 頁）。新しく書くのは `OnPastaDumpVariables` と位置の直しの 2 つで、どちらも小さい。brief の見積もり 12〜16 タスクのまま。議題 1 で 6.3 を落とせば 1 減る。
- **Risk: Medium-Low**。消すものの境が brief で決まっている。気をつけるのは 3 つ。(1) キックと一緒に、`co_preempt` と Status の抑止（`virtual_dispatcher.lua` の `is_blocked`）を壊さないこと。(2) `spawn_actor_thread_with` の引数の変更が kick と関係の無いテストに広がること。(3) `.claude/skills/` の references の作り直しが分類器に拒まれうること。

## Research Needed（設計へ）

1. **縮んだチャンク名からのソースマップの引き方**（要件 6.3）：エラー文のチャンク名は `LUA_IDSIZE`（60 バイト）で `...`＋末尾 56 バイトに縮む（`searcher.rs` 181〜205 行）。ゴーストのパスはふつう 60 バイトを超える。`@pasta_chunks` のキー（縮んだ形）からチャンクの全体のパスか `.pasta` のパスを引き、ソースマップ（正規化した全体のパスのキー。`canonicalize_chunk_name`）に渡す道を決める。重複（`false`）のときは引かない。
2. **`@pasta_chunks` の辞書の印の値**（要件 6.2）：今は `true`。`.pasta` のパス（ゴーストのフォルダーから見た形）にするなら、読み込みの探索関数（`record_chunk`）がキャッシュのパスから `.pasta` を知る手段が要る（`CacheManager::source_to_cache_path` の逆。モジュール名の `.`・`-` を `_` にするので、文字列からは戻せない）。ローダーが変換のときに表を渡すか、行の対応の文字列にチャンクの印を足すか。`stop.lua` の純粋な関数の形（引数で表を受ける）を保つ。
3. **JIT が止まらないことの確かめ方**（要件 1.2）：VM から `jit.status()` が見えるか（ALL_SAFE の VM で `jit` の表があるか）。見えなければ、フックが付かないことを別の形で確かめる。
4. **書き出しの原子性**（要件 5.5）：Windows で一時ファイルからの置き換え（`std::fs::rename` が既存のファイルを置き換えるか）。`save_to_file` の書き方に倣う。
5. **`multi_instance_test.rs` の書き直し**：有効な 2 体が並ぶ確かめ（要件 1.3）を、ポートの代わりに何で見るか（2 体とも `OnPastaScene` で再生できる、など）。
6. **SSP の `raise_event` の応答**：204 を返したとき、SSP の MCP が「No response from server」を返すか（`vscode-mcp-client` が設計で確かめる。この spec はファイルを 204 の前に書き終えるだけ）。

## 設計への申し送り

- 撤去と整えは 1 本の設計のまま、タスクを分ける（brief の Boundary Candidates）。撤去を先に、整え（門・位置・イベント）を後に。
- 同じウェーブの約束：`act.lua`・`scene.lua`・`search/`・`code_gen/element_gen.rs` に触らない。位置の直し（要件 6）は `stop.lua`・`searcher.rs`（とローダー）で閉じる。`act.lua` の `restore_scene`・`init_scene` は変えない。`fill_line_maps`（`transpiler.rs`）を変えるなら、`element_gen.rs` に触れずに済むことを確かめる。
- ルートの `Cargo.toml` の `socket2` を消す。`Cargo.lock` は `cargo` に作り直させる（足さない）。`pest-2-9-4-upgrade` と `Cargo.lock` を共有する見込み（後から入る側が合わせる）。
- `vscode-mcp-client` との決まり：`OnPastaSceneAt` の Reference0 は絶対パス（議題 5 を採るなら URI は受けない。あちらの brief もパスを送る）。`OnPastaDumpVariables` の書き出す先は `ghost/master/profile/pasta/debug/variables.toml`。デバッグが無効なら 204。警告の文言 `<イベント名>: debug is disabled; …` は今のまま。
- steering の古い記述：`tech.md` 36 行（`serde_json` は DAP のメッセージ）・44 行（`socket2`）・256 行（`@pasta_debug` の行）、`structure.md` 61〜73 行（`debug/` の木）・319・331 行、`product.md` の「VSCode 拡張・デバッグ基盤」の節。kiro-complete のスキルの同期で直す。
- `ponytail-lua-debug` の N1〜N3・N7〜N10（`threads`・受け口の無い要求・`Decoded::reload_shiori`・encoder の join handle・`capture_*` の引数・対応表の無いブレークポイント・`debug_source_mode` の説明）は、部品ごと消えるので直す要が無い。N13（UNC の URI）は議題 5 しだいで消える。

## 要件ディスカッションの結果

requirements.md の末尾の議題 1〜10 は、2026-11-03 に開発者が全部を推奨案で承認した。根拠は上の各節にある。

## 設計フェーズの調べ（2026-11-03・kiro-spec-design）

参照したスキルとガイド: `kiro-spec-design`（`rules/design-principles.md`・`design-discovery-light.md`・`design-synthesis.md`・`design-review-gate.md`）、`kiro-validate-design`（`rules/design-review.md`）。発見は「既存の拡張」として軽い調べ（Light Discovery）で行った。新しい外部の依存は無い。

### Summary（設計）

- **Feature**: `debug-mcp-runtime`
- **Discovery Scope**: Extension（撤去が主の既存の拡張）
- **Key Findings**:
  - 「デバッグが有効か」はソースマップの有無と 1 対 1 に揃えられる。ローダは有効なときだけ `Some` を作り、`build_source_map` は失敗を飛ばすだけで必ず `Some` を返す。`RuntimeConfig::debug` と 2 回目の解決（`factory.rs`）は要らない。
  - 縮んだチャンク名からの引き（調べ 1）は、Rust の `ChunkTable::seen`（キー → 最初の候補の全体のパス）が既に持っている。Lua に全体のパスを出さずに済む。
  - チャンクがどの `.pasta` か（調べ 2）は、モジュール名からは戻せないが、ローダが `.pasta` ごとのモジュール名を知っているので、表を渡せば searcher が控えられる。キャッシュの中身は変わらない。

### 調べ 1: 縮んだチャンク名からソースマップを引く道（要件 6.3）

- **Sources**: `runtime/searcher.rs` 175〜263 行（`LUA_IDSIZE`・`error_form`・`ChunkTable`・`record_chunk`）、`debug/source_map/mod.rs` 316 行（`canonicalize_chunk_name`）・550 行（`resolve_lua_to_pasta`）、`loader/source_map_build.rs`（チャンク名は `CacheManager::source_to_cache_path`）、`loader/context.rs` 35〜68 行（`base_dir` は `std::path::absolute` で、8.3 の名前を解かない）。
- **Findings**: `ChunkTable::seen: HashMap<Vec<u8>, String>` が、エラー文の形のキーから、そのキーを最初に作った候補（`package.path` を埋めた全体のパス）を引ける。重複は Lua の表で `false` になり、`stop.lua` が引かない。候補のパスとソースマップのキーは、どちらも同じ `base_dir` から作られ、`canonicalize_chunk_name`（`@`・`\`→`/`・Windows の大小）で揃う。DAP のブレークポイントが同じ一致に頼っていた。
- **Implications**: `searcher::chunk_path(lua, key)` を足し、`@pasta_debug.pasta_line(chunk, line)` が `resolve_lua_to_pasta` を呼ぶ。`SourceMap` に新しい問い合わせは要らない。一致が崩れたときに黙って引けなくなる危険は、結合テスト（6.3）で守る。

### 調べ 2: `@pasta_chunks` の辞書の印の値（要件 6.2）

- **Sources**: `loader/cache.rs` 249〜276 行（`source_to_module_name`・`source_to_cache_path`）・300 行（`get_relative_path`）、`loader/process.rs` 150〜200 行（行の対応の `.pasta` は `get_relative_path` の形）、`loader/mod.rs` 125〜250 行（`LoaderContext` は Phase 1.7 で作り、Phase 6 でファクトリに渡る）、`runtime/searcher.rs` 115〜147 行（`find_module` はモジュール名を持つ）、`shiori/stop.lua` 86〜119 行。
- **Findings**: モジュール名は `.`・`-` を `_` にするので、キャッシュのパスから `.pasta` に戻せない。ローダは Phase 4 で `.pasta` ごとにモジュール名を作るので、モジュール名 → `.pasta` のパス（`get_relative_path`）の表を `LoaderContext` に積めば、`record_chunk` がモジュール名で引ける。行の対応の `.pasta` のパスも同じ `get_relative_path` から作られるので、`\` に揃えるだけで比べられる。値の型は、作者の `.lua` の文字列と見分けるため表 `{ pasta = … }` にした（`.pasta` の拡張子で見分ける案は、`[loader] pasta_patterns` が拡張子を縛らないので採らない）。`dic/` の `.lua` はキャッシュに写され、今は `true` になって行の対応で引かれる（違う行を出しうる）。表に入れなければ場所を出さない。
- **Implications**: `LoaderContext::pasta_sources`・`register_chunk_table` の引数・`record_chunk` のモジュール名の引数を足す。`fill_line_maps`・`act.lua`・キャッシュの版は変えない。

### 調べ 3: JIT が止まらないことの確かめ方（要件 1.2）

- **Sources**: `runtime/runtime_config.rs` 15 行（`default_libs` は `std_all`）、`debug/debug_mod_tests.rs` 205〜236・341〜345 行（ALL_SAFE の VM で `jit.status()` を呼んでいる）。
- **Findings**: 既定の VM で `jit.status()` が呼べる。今のテストは、デバッグを有効にすると偽になることを確かめている。
- **Implications**: 有効なゴーストを読み込んだ VM で `jit.status()` が真であることを確かめる。フックを付けないことは、フックの部品（`hook.rs`・`enable.rs`）が無くなることと、grep（`set_hook`・`jit.off`）で確かめる。x86 の LuaJIT も JIT を持つので、期待は分けない見込み。

### 調べ 4: 書き出しの原子性（要件 5.5）

- **Sources**: `runtime/persistence.rs` 260〜313 行（`save_to_file`）・680〜691 行（`quarantine` の「rename は Windows でも行き先を置き換える」）。Rust の `std::fs::rename` の説明（Windows では在るファイルを置き換える）。
- **Findings**: `save_to_file` は、親のフォルダを作り、`<path>.tmp` に書いて `sync_all` し、`rename` で置き換える。`rename` の失敗のときだけ一時ファイルを消し、書き込み・`sync_all` の失敗では一時ファイルが残る。最後のファイルは `rename` まで触らないので、前のファイルは消えない。読み手が共有の削除を許さずに開いていると、Windows の `rename` は拒まれる。
- **Implications**: `dump_table` は `save_to_file(&doc, path, false)` を使う。`save_to_file` を、書き込み・`sync_all` の失敗でも一時ファイルを消すように直す（`save.toml` にも効く小さな直し）。

### 調べ 5: 有効な 2 体が並ぶことの確かめ（要件 1.3）

- **Sources**: `pasta_shiori/tests/multi_instance_test.rs` 156〜241 行、`tests/fixtures/multi_instance/dic/multi.pasta`（`＊会話` などの大域シーン）。
- **Findings**: 今の `log_levels_and_debug_port_stay_per_instance` は、同じポートを指した後の方が警告を出すことを確かめている。ポートが無くなれば、2 体とも `[debug] enabled = true` で読み込みが通り、どちらも `OnPastaScene` に 200 で答えることが「並ぶ」の確かめになる。
- **Implications**: テストを書き直す。ログの量の確かめはそのまま残し、ポートの確かめを「両方のログに info の 1 行があり、待ち受けのログが無い」に替える。`dap_kick_reaches_only_its_own_instance` は消す。

### 調べ 6: SSP の `raise_event` の 204

- **Findings**: この spec の境の外（`vscode-mcp-client` が確かめる）。この spec が約束するのは、ハンドラの中で書き出しを同期に終えてから 204 を返すことだけ。

### そのほかに見つけたこと

- `pasta_lua/tests/scene_alias_search_test.rs` 330 行の埋め込みの Lua が `KICK.try_dispatch` を使う（ギャップ分析の一覧に無かった）。キックの分を外す。
- `pasta_sample_ghost/tests/tutorial_stages_test.rs` 370 行の `E2E_PREFIXES`（`Kick`・`Gate`）は、消す e2e（`scene_kick_*`）のシーン名との衝突を避けるためのもの。使い手が無くなる。
- `pasta_lua/tests/transpiler/final_regression_test.rs` 257・292 行に、`enable`・`SourceMode` を確かめる 2 本がある。消す。
- `pasta_lua/tests/scene_play_event_test.rs` 251 行の「待ち受けを塞いでも位置で再生できる」は、`debug_enabled()` が待ち受けの有無を返すことを前提にしている。ポートの話を外して直す。
- DAP を消すと、`SourceMap` の逆引き（`reverse`・`resolve_pasta_to_lua`・`nearest_pasta_line_with_mapping`）と `ChunkSourceMap::from_forward` も本番の使い手を失う（使い手はブレークポイントとサイドカーだった）。
- `PASTA_DEBUG_PORT` の中和の写しは、`pasta_shiori/src/shiori_request_tests.rs` にもある（`src` の中）。

## Architecture Pattern Evaluation（設計）

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 今の部品の中で消し、残す | `debug/` の DAP を消し、`config`・`playscene`・`source_map` を残す。Lua の門は `scene_play.lua` | 新しいファイルがほぼ無い。残す部品のテストがそのまま守り | 「debug」の名前が残る | 採った（ギャップ分析の推奨のまま） |
| B: 開発用の部品を作り直す | `debug/` を `dev/` などにし、`@pasta_debug` の名前を変える | 名前と中身が揃う | 名前の変更が `vscode-mcp-client`・マニュアル・スキルに波及する | 採らない |
| C: Lua だけ集める | 3 つのイベントを新しいファイルへ | 門が 1 か所 | `scene_play.lua` に 1 つ足すだけで済む | 採らない |

## Design Decisions

### Decision: 「デバッグが有効か」をソースマップの有無で持つ

- **Context**: 要件 7.5・7.6。今は解決が 2 回（ローダとファクトリ）で、`debug_enabled()` は待ち受けの有無を返す。
- **Alternatives Considered**:
  1. `DebugConfig { enabled }` を残し、`RuntimeConfig::debug` に積む。
  2. `PastaLuaRuntime` に `debug_enabled: bool` を足し、ソースマップと別に持つ。
  3. ソースマップの有無だけで持つ。
- **Selected Approach**: 3。ローダが `resolve_enabled` を 1 回だけ呼び、真のときだけ `Some(Arc<SourceMap>)` をファクトリに渡す。`debug_enabled()` と `@pasta_debug.enabled()` はその有無を返す。
- **Rationale**: 2 つの値を持つと食い違いうる。有効 ⇔ ソースマップがある、は作り方で決まる（3.5・3.6）。`RuntimeConfig` から `debug`・`kick_sink` が消え、`Debug` の手書きも要らなくなる。
- **Trade-offs**: ローダを通らない VM でデバッグを有効にする道が無くなる（使い手は DAP のテストだけ）。
- **Follow-up**: 設計ディスカッションで確かめる（design.md の前提 A2）。

### Decision: 6.3 の引きは「チャンク名と行」で Rust に渡す

- **Context**: 要件 6.3、調べ 1。
- **Alternatives Considered**:
  1. `@pasta_chunks` の値に全体のパスを持たせ、Lua から渡す。
  2. `.pasta` のパスと行で引く新しい問い合わせを `SourceMap` に足す。
  3. 縮んだチャンク名と行を渡し、Rust が `ChunkTable::seen` で全体のパスに戻して `resolve_lua_to_pasta` を呼ぶ。
- **Selected Approach**: 3（`@pasta_debug.pasta_line`・`searcher::chunk_path`）。
- **Rationale**: 既にある表と問い合わせだけで済む。返すのは行だけで、ファイルは `stop.lua` が `@pasta_chunks` の値から出すので、ソースマップの絶対パスを相対に直す手間も無い。
- **Trade-offs**: `@pasta_debug` が searcher の表を読む（同じ `runtime` の中の `pub(crate)`）。

### Decision: 辞書のチャンクの値を `{ pasta = … }` にする

- **Context**: 要件 6.2、調べ 2。
- **Selected Approach**: ローダがモジュール名 → `.pasta` のパスの表（`LoaderContext::pasta_sources`）を渡し、`record_chunk` が表の値を控える。表に無いキャッシュは控えない。
- **Rationale**: 行の対応の書式とキャッシュの中身を変えずに、チャンクの `.pasta` が分かる。`act.lua`・`transpiler.rs` に触れない。
- **Trade-offs**: `LoaderContext` に欄が増える（pub の struct）。

### Decision: `OnPastaDumpVariables` は `scene_play.lua` と `@pasta_debug` に置く

- **Context**: 要件 5、brief の Approach 5。
- **Selected Approach**: ハンドラは `scene_play.lua` で `allowed()` を共有する。書き出しは `@pasta_debug.dump_variables(save)` → `persistence::dump_table` → `save_to_file(…, false)`。
- **Rationale**: 門が 1 か所のまま。変換はセーブと同じ関数で、外したキーの警告も共有する。セーブの門（`SaveGate`）は通らないので、読めないセーブがあっても書き出せ、`save.toml` に触れない。
- **Trade-offs**: `allowed()` の警告の後半を引数にする（`scene not played` と `variables not dumped`）。

### Decision: `OnPastaSceneAt` の Reference0 はパスとしてだけ扱う

- **Context**: 要件 4.4（議題 5）。
- **Selected Approach**: `resolve_position` は `std::path::absolute(base_dir.join(file))` だけにし、`uri_to_pasta_path` とその下の 4 つの関数を消す。
- **Rationale**: `%41` を別のファイルに直す誤りと、UNC の URI のホスト名が消える誤り（N13）が、道ごと消える。

### Decision: DAP だけが使っていた残す部品の口も消す

- **Context**: 要件 7.1・7.3。
- **Selected Approach**: 7.3 の一覧に加え、`SourceMap` の逆引き（ブレークポイントだけが使った）と sidecar の再輸出を消す。`ChunkSourceMap::from_forward` はテストの補助が使うので `#[cfg(test)]` にする。
- **Rationale**: 7.1 の「DAP のためだけのコードを残さない」。後の spec が読む量が減る。
- **Follow-up**: 設計ディスカッションで確かめる（design.md の前提 A1）。

## Synthesis（設計の 3 つの見方）

- **Generalization**: 3 つの開発用のイベントは「デバッグが有効で external でなければ許す」の同じ門の変種なので、`allowed(act, event, outcome)` 1 つにした。止まるエラーの場所は「チャンク → 出どころ（作者の `.lua`／`.pasta`／不明）→ 行」の 1 つの引きで、行の対応とソースマップはその中の 2 つの引き方にした。
- **Build vs Adopt**: 書き出しは `persistence.rs` の変換と `save_to_file` をそのまま使う（新しい書式・依存を足さない。シリアライズは TOML の方針）。原子性は std の `rename`。チャンク名の全体のパスは searcher の既存の表を使う。
- **Simplification**: `DebugConfig` の型・`RuntimeConfig::debug`・2 回目の解決・`with_debug*`・`SourceMode`・sidecar・逆引き・URI の復号を消した。新しい型は足さない（`LoaderContext` の欄 1 つと関数 3 つ）。

## Risks & Mitigations（設計）

- 候補のパスとソースマップのキーが一致しないと 6.3 が黙って引けない — 結合テスト（`runtime_error_balloon_e2e_test.rs` の有効の 1 本）で守る。
- 撤去の差分が大きく、途中でビルドが壊れる — design.md の Implementation Order のとおり、どの段でもビルドが通る順に切る。`runtime_config.rs` の `kick_sink` と `pasta_shiori` は同じタスクにする。
- `.claude/skills/` の references の作り直しが分類器に拒まれる — 入った分と残りを示して開発者に頼む（メモリの方針）。
- 環境変数を置くテスト（3.4）がほかのテストに漏れる — 専用のテストのバイナリにする。
