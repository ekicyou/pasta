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
