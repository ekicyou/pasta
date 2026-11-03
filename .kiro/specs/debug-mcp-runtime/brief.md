# Brief: debug-mcp-runtime

> **ステータス**: 未着手（2026-11-02 の `/kiro-discovery`「Pasta DSL MCP移行検討」で起票）。0.5.0（開発者「0.5.0にしてもウェーブ数変わらなさそう」）。要件定義は Opus。対になる spec は `vscode-mcp-client`（VSCode の側）。

## Problem

pasta の開発の道具（VSCode の「▶ シーンを実行」・SHIORI のリロード）は、pasta.dll が 9276 番で待ち受ける DAP（デバッガーの通り道）に頼っている。

- デバッガーによる制御は不安定な点が多い。0.5.0 の監査（閉じた `ponytail-lua-debug`）だけで、`.pasta` のブレークポイントが死ぬ・ステップが抜けない・応答の無い要求で固まる、などの Must fix が 5 つあった。
- 実際の開発は、もう MCP で回っている。ghost_dev「ぱすた開発」は SSP の MCP だけを使い、DAP は一度も使っていない（2026-11-02 の聞き取り）。
- デバッグを有効にすると、行フックと `jit.off()` で JIT が全体で止まる。ポートが固定なので、2 体を同時に起こすと衝突した経緯もある（v0.4.2 で直した）。
- 実行時の Lua の例外は、転写キャッシュ（`cache/lua/…`）の行で出る。作者は `.pasta` の行へ手で戻している。ソースマップはあるのに、DAP しか使っていない。

## Current State

2026-11-02（main `1a02383a`）に確かめた。

- `crates/pasta_lua/src/debug` は 79 ファイル・約 22.4k 行。うち本番は約 8k 行、テストは約 14k 行。
- **DAP と一緒に消えるもの**（約 6.2k 行）
  - `transport/`・`dap/`・`session/`・`wiring/`
  - `breakpoints`・`hook`・`inspect`・`enable`・`handle`・`error`・`types`・`source_mode`・`kick`
  - 外の使い手
    - `pasta_shiori` の `actor/instance.rs:150` の `kick_sink_for`
    - `runtime/exec.rs:86` の `debug_source_mode()`
    - `lib.rs:52` の再輸出
- **キックの経路**: キックを生むのは DAP の `pasta/playSceneAt`・`pasta/reloadShiori` だけである。
  - Lua の側
    - `event/kick.lua`（167 行）
    - `entry.lua:12`・`:132` の `SHIORI.kick`
    - `virtual_dispatcher.lua:234-257` の `STORE.kick_force`・`kick_pending`
  - Rust の側
    - `actor/mailbox.rs:114` の `ActorMsg::Kick`
    - `actor/thread.rs:238`
    - `shiori.rs:336~` の `kick()`
    - `actor/instance.rs:83-98`
    - `actor/lifecycle.rs:137-160`
- **残るもの**（約 1.6k 行）
  - `source_map/`（`mod.rs` 675・`scene_index.rs` 214・`scene_join.rs` 289・`sidecar.rs` 145）
  - `playscene.rs` の `resolve_position`・`uri_to_pasta_path`（`@pasta_debug.scene_at` の芯）
  - `config.rs` の「設定で有効か」
- **門**
  - `[debug] enabled`（`loader/config/sections.rs:290-330`）と env `PASTA_DEBUG`（`debug/config.rs:151-160`）で決まる。
  - 有効なときに起きること
    1. ローダーが Phase 5.5（`loader/mod.rs:213-231`）で全部の `.pasta` をメモリ上でトランスパイルし直し、ソースマップを作る。
    2. `factory.rs:153-170` でシーン索引を突き合わせる。
    3. `runtime/mod.rs:233-248` で `debug::enable` を呼ぶ。フック・`jit.off()`・待ち受けが立つ。
  - `scene_play.lua:18-32` の `allowed()` が `@pasta_debug.enabled()` を見る。偽なら warn を出して 204。
  - `OnPastaCallFunction`・`OnPastaSetSaveVariable`（`external_control.lua`）には門が無い。
- **ソースマップの使い手**: DAP と `@pasta_debug.scene_at` だけ。
  - 実行時エラーの文言にも、エラーのバルーンにも使われていない。
  - `resolve_lua_to_pasta`（`source_map/mod.rs:550`）を debug の外から呼ぶのは、テストだけ。
- **保存変数**
  - `＄＊名前` は `save.名前`。実体は `pasta/save.lua` で、`@pasta_persistence` が runtime の Drop で `save.toml` へ書く。
  - 書くイベントはある（`OnPastaSetSaveVariable`）。読むイベントは無い。
  - `save.toml` は Drop まで書かれないので、動いている間の値はファイルから見えない。
- **リロード**: `\![reload,shiori]` も MCP の `reload` も、引数を取れない（UKADOC・SSP ヘルプ）。
  - load のたびに `pasta.toml` を読み直し、トランスパイルとソースマップを作り直す（`shiori.rs:92`・`actor/lifecycle.rs:37`）。

## Desired Outcome

- pasta.dll から DAP の待ち受け・セッション・行フック・`jit.off()`・キックの経路が無くなる。
  - デバッグを有効にしても、JIT は止まらない。ポートも取らない。
- `[debug] enabled`（と `PASTA_DEBUG=1`）は、「開発用のイベントを許す」旗になる。待ち受けは無い。
  - 有効なときだけ、読み込みでソースマップとシーンの索引を作る（今のまま）。
- 有効なとき、Lua の実行時の例外に `.pasta` の位置が添えられる。
  - 添える先は `pasta.log` の行と、実行時エラーのバルーン。
  - 例: `（dic/pose.pasta:42）`。
- 開発用のイベント（どれも有効なときだけ、external は断る）
  - `OnPastaScene`・`OnPastaSceneAt`: 今のまま。
  - `OnPastaDumpVariables`（新設）: 今の保存変数（`＄＊名前`）を `profile/pasta/debug/variables.toml` に書き出して、何も喋らずに 204 を返す。
    - 値を応答で返さない理由は 2 つ。`raise_event` は返した台本を再生してしまう。応答が届かない（「No response from server」）ことも多い。
- マニュアルの internals とリファレンスの頁が、新しい形に揃う。

## Approach

撤去を主にし、残す部品を小さく保つ。

1. **撤去（pasta_lua）**
   - DAP の部品を消す。`transport/`・`dap/`・`session/`・`wiring/`・`breakpoints`・`hook`・`inspect`・`enable`・`handle`・`error`・`types`・`source_mode`・`kick` と、そのテスト。
   - `tests/runtime/` の DAP の結合テストとフィクスチャも消す。
2. **撤去（キックの経路）**
   - Lua: `event/kick.lua`、`SHIORI.kick`、`STORE.kick_*`、`virtual_dispatcher.lua` の門の分岐。
   - Rust: `pasta_shiori` の `ActorMsg::Kick` から `kick_sink_for` まで。
   - テスト: `pasta_shiori/tests/` の kick 系 6 本、`lua_specs/` の kick 系 8 本。
   - 指名した再生は `OnPastaScene`・`OnPastaSceneAt` に一本化する。どちらも応答でそのまま台本を返す。
3. **門を縮める**
   - `DebugConfig` を `enabled` だけにする。
   - `port`・`present_as`・`source_map_sidecar` のキーと、env の `PASTA_DEBUG_PORT`・`PASTA_DEBUG_SOURCE_MODE`・`PASTA_DEBUG_SOURCE_MAP_SIDECAR` を消す。`PASTA_DEBUG` は残す。
   - 古いキーが書かれたままの `pasta.toml` は、`pasta.toml` の今の「知らないキー」の扱いに合わせる（設計で確かめる）。
   - 「設定の有効」と「待ち受けの有無」を混ぜない（`scene-play-event` の申し送り）。待ち受けが消えるので、`debug_enabled()` と `debug_handle` の役目を見直す。
4. **ソースマップの新しい使い手**
   - Lua の例外の行（`chunk:行`）を `resolve_lua_to_pasta` で引き、`pasta.log` の行とエラーのバルーンに `.pasta` の位置を添える。
   - 置き場は、実行時の失敗の出力を一本化した所（`pasta.failure` と Rust の側の例外の扱い）。
   - 引けないとき（デバッグが無効・Lua だけのファイル・`scripts/`）は、今の文言のまま。
   - sidecar（`.lua.map`）は使い手が無くなるので消す。
5. **`OnPastaDumpVariables`**
   - `external_control.lua` か `scene_play.lua` の隣に置く。
   - 保存の書式は TOML（`@toml`。方針「シリアライズは TOML」）。
   - 書けない値（関数など）は飛ばしてログに残す。
6. **マニュアル**（担当は internals とリファレンス。利用者の章 `book/src/debug/` は `vscode-mcp-client`）
   - `internals/debug.md`（510 行）を書き直す。「デバッグ基盤とシーンキック」から「開発用のイベントとソースマップ」へ。
   - `reference/pasta-toml.md` の `[debug]` 節（741-802 ほか）。
   - `lua/shiori-events.md`（660-740 のデバッグ有効の決まり、`OnPastaDumpVariables` を足す）。
   - `internals/{shiori,execution-model,loader,index,internal-modules}.md` のキックと DAP の記述。
   - `grammar/call-jump.md`・`reference/startup.md`・`introduction.md:19`・`internals/registry-search.md`・`lua/modules/pasta-search.md`・`lua/patterns.md` の言及。
   - `DEBUGGING.md`。スキルの references（`gen-skill-refs`）の作り直し。

採らなかった案:

- **リロード専用のイベント**（`OnPastaReload`・`OnPastaReloadDebug`）は作らない。
  - `[debug] enabled` は load のたびに `pasta.toml` から読み直されるので、リロードに引数は要らない。
  - リロードそのものは、MCP の `reload` がもうできる。
  - 「次の読み込みだけ有効」は、アンロードをまたいで覚える仕組みが要るので見送った（開発者の承認、2026-11-02）。
- **ソースマップを後から作る案**も採らない。
  - 作ること自体はできる（ディスクの `.pasta` をトランスパイルし直せば同じ表が出る）。
  - けれど、読み込みの後に作者がファイルを書き換えていると、動いているコードと表が黙ってずれる。
  - 読み込みのときに作る今の形が正確である。`vscode-mcp-client` の「▶ シーンを実行」は毎回リロードするので、表はいつも最新の読み込みと揃う。
- **ソースマップをいつも作る案**（本番でもエラーに `.pasta:行`）と **sidecar を本番の道具に作り直す案**は、範囲を超えるので入れない。
- **式を評価して返すイベント**（ぱすた開発の要望。モジュールの中の状態を見たい）は、バックログに残す。開発者の要望は「グローバル変数が見られれば良い」である。

## Scope

- **In**
  - DAP とキックの撤去（Rust・Lua・テスト）
  - 門の縮小と設定のキーの整理
  - 実行時エラーへの `.pasta` の位置の添え書き
  - `OnPastaDumpVariables`
  - `pasta_check validate` の組み込みの名前（新しいイベント名が要れば）
  - 上のマニュアルの頁とスキルの references
  - `pasta_sample_ghost` のテストの `PASTA_DEBUG` の中和（`tests/common/mod.rs:7-13`・`Cargo.toml:25`）の見直し。固定ポートが無くなれば要らない
- **Out**
  - VSCode 拡張の一切（`vscode-mcp-client`）
  - `book/src/debug/` の利用者の章（`vscode-mcp-client`）
  - 式の評価イベント
  - 本番でのソースマップ
  - リロード専用のイベント

## Boundary Candidates

- 撤去（DAP・キック）と、残す部品の整え（門・ソースマップ・新しいイベント）。設計では 1 本のまま、タスクを分ける。
- Rust の `pasta_lua/src/debug`・`pasta_shiori/src/actor` と、Lua の `pasta_scripts/pasta/shiori/event`。

## Out of Boundary

- MCP のクライアント・ゴーストやベースウェアの見つけ方（`vscode-mcp-client`）
- SSP・areka の MCP の挙動そのもの（「No response from server」は SSP の側の事情かもしれない。`vscode-mcp-client` が設計で確かめる）

## Upstream / Downstream

- **Upstream**: 0.5.0 のウェーブ 1 のうち、ファイルを共有する spec。
  - `ponytail-shiori`（`actor/`）
  - `ponytail-lua-scripts`（`entry.lua`・`virtual_dispatcher.lua`・`kick.lua`）
  - `ponytail-lua-transpiler`（`source_map/mod.rs` の説明と、`ACTOR:create_word` の行の記録）
- **Downstream**
  - `vscode-mcp-client`: このイベントの決まりを使う。
  - `shiori4-v4-host`（0.6.0）: キックの即時化の対象が無くなる。
  - `shiori4-v4-lua-api`（0.6.0）。

## Existing Spec Touchpoints

- **Extends**: なし。閉じた `ponytail-lua-debug` の、残る部品への指摘を受け継ぐ（下の「受け継ぐもの」）。
- **Adjacent**
  - `vscode-mcp-client`
  - `scene-count`・`scene-condition-attributes`（`entry.lua`・`search/context.rs` の近く）
  - `scene-attribute-store`（`factory.rs`・`scene.lua` の近く）
  - `shiori4-v4-host`

## 受け継ぐもの（閉じた `ponytail-lua-debug` から。2026-11-02）

材料は、`ponytail-lua-debug` のセッションが残した 2 つのノート。

- `C:\Users\maz-o\AppData\Roaming\Claude\side-session-notes\local_0e05a9a5-790a-4790-bfe9-70c088c4cc63\ponytail-lua-debug-closed-to-dap-removal.md`
- 同じフォルダの `ponytail-lua-debug-research.md`（ギャップ分析の全文 N1〜N13）

要件・ギャップ分析は、ブランチ `claude/kiro-start-ponytail-lua-debug-49bccb`（`0817e2cf`〜`577548ec`。push していない）にある。brief は git の `ae04a28b` で読める。

- `playscene.rs:31`・`source_map/scene_index.rs:39` の `#![allow(dead_code)]`。結線の前の名残り。
- `insert_file`（`scene_join.rs:130` が呼ぶ）。呼ばれているが働きが無く、上のコメント「未検出を正しく返すため」も誤り（N5）。
- テストだけが使う対応表の公開の問い合わせ。`ChunkSourceMap::new`・`len`・`is_empty`・`lua_lines_for_pasta`・`MapBuilderSink::chunk_name`（N11）。
- `canonicalize_pasta_file` は `canonicalize_chunk_name` をそのまま呼ぶだけ（N12）。
- `source_map/mod.rs:518-521` の `scene_index()` は使い手が 0。`:638-648` は再輸出の抑止（N4）。
- `DebugConfig::resolve` の引数が 8 つ（監査の 12 番）。テストだけが使う `DebugConfig::from_file`（N6）。
- UNC の `file://server/share/...` でホスト名が消える（`playscene.rs` の `strip_file_scheme`。N13）。`OnPastaSceneAt` が絶対パスを受けるので、パスの扱いとして残る。
- `ponytail-lua-transpiler` からの申し送り
  - `source_map/mod.rs` 25 行の説明の `normalize_output` を `normalize_output_with_shift` に直す。
  - アクター辞書の単語定義の行（`ACTOR:create_word(…)`）もソースマップに載る（対応は増えるだけ）。
  - `normalize_output_with_shift` が消すのは末尾の空白の行だけになり、実際の生成コードでは `LineShift` が恒等になる。`MapBuilderSink::finish`・`loader/source_map_build.rs` の「行がズレ得る」という説明の見直しが要る（`ponytail-lua-transpiler` の design.md「Revalidation Triggers」）。

## Constraints

- **x86 で走らせる**: pasta.dll の本番は x86 である。テストは 64 ビットだけに限らない。
- **load を叩く新しいテスト**には `PASTA_DEBUG` の中和が要る。固定ポートが無くなれば中和は要らなくなるので、そこを確かめてから外す。
- **`OnPastaScene`・`OnPastaSceneAt` の引数と応答は変えない**: ぱすた開発とスキルが使っている。
- **`[debug]` を書いた既存のゴースト**（ghost_dev の DEBUG 版）が、読み込みエラーで止まらないこと。
- **マニュアルの章の数**: 頁を消したり足したりするなら、章の数を決め打ちする検査を合わせる（`ponytail-manual-ci` が決め打ちをやめていれば要らない）。

## 申し送り（ponytail-shiori より・2026-11-03）

上の Current State の行番号と部品は、ponytail-shiori（0.4.5）で次のように変わった。撤去の範囲を測り直すときはこちらを正とする。

- キックの Rust の側のうち、置き場へ kick を積む `lifecycle::kick_sink`・`kick_into` と `ActorInstance::kick` は既に消えた（上の `actor/instance.rs:83-98`・`actor/lifecycle.rs:137-160` は無い）。残るのは `ActorMsg::Kick`（`mailbox.rs`）、アクタースレッドの Kick の腕（`thread.rs`）、`PastaShiori::kick`（`shiori.rs`）、`kick_sink_for`（`instance.rs`。`pub` にした）。
- `pasta_shiori/tests/` の kick の e2e 4 本（`scene_kick_*_e2e_test.rs`）は、自分の mailbox と `kick_sink_for` の送り先・`spawn_actor_thread_with` でアクターの道を回す形になった。kick を消すときは、この 4 本と `actor_kick_test.rs`・`multi_instance_test.rs` の `dap_kick_reaches_only_its_own_instance` が対象になる。
- DAP の待受の番号を外から観る `ActorThread::debug_local_addr` は消えた。テストはゴーストのログの `debug backend listening addr=` の行から読む（`tests/common/mod.rs` の `listening_port`。`debug_preservation_test.rs`・`multi_instance_test.rs` が使う）。待ち受けを消すときは、この補助と使い手も一緒に直す。
- `tests/common` の `#[ctor]` は `PASTA_DEBUG`・`PASTA_DEBUG_PORT`・`PASTA_LOG` を消す。固定ポートが無くなったら中和を外せるか（上の「load を叩く新しいテスト」の行）は、ここを見る。

## 2026-11-03 棚卸の再測定（main c575febd）

- **前提の変化**: 先に要った 3 本（`ponytail-shiori`・`ponytail-lua-scripts`・`ponytail-lua-transpiler`）は全部入った。`pasta_lua/src/debug`・`runtime/`・`loader/` は、テスト 1 本（`source_map_builder_tests.rs`）のほかは変わっていない。`pasta_shiori` の側は上の申し送り（ponytail-shiori）のとおり（`kick_sink_for` は `actor/instance.rs` 76 行、使う所は 139・284 行）。Lua の側の行が動いた。`entry.lua` の `KICK` は 12 行、`SHIORI.kick` は 112〜121 行（`SHIORI.load`・`unload` の中身は `ponytail-lua-scripts` が消した）、`virtual_dispatcher.lua` の門の分岐は 246〜272 行、`STORE.kick_*` は `store.lua` 78・82・105・106 行。`kick.lua` 167 行は手つかず（53 行の古い説明は、この spec がファイルごと消す約束で残してある）。
- **章の数**: `ponytail-manual-ci` が章の数を目次（`SUMMARY.md`）から数えるようにした。Constraints の「章の数」は、頁を消す・足すときに `SUMMARY.md` を直すだけで済む（`internals/debug.md` の題は 82 行）。
- **触るファイル**: Rust は `crates/pasta_lua/src/debug/`（DAP の部品とテストを消し、`source_map/`・`playscene.rs`・`config.rs` 221 を残す）、`runtime/exec.rs` 149（86 行）・`runtime/mod.rs` 259（81・233 行）・`runtime/factory.rs` 209（67 行の説明・146〜160 行のシーン索引）・`lib.rs` 69（52 行）・`loader/mod.rs` 329（214 行）・`loader/config/sections.rs` 358・`loader/source_map_build.rs` 172、`pasta_shiori` の `actor/mailbox.rs`・`thread.rs`・`instance.rs`・`shiori.rs`、テスト（`pasta_shiori/tests/` の kick 6 本・`debug_preservation_test.rs`・`multi_instance_test.rs`・`tests/common/mod.rs`、`pasta_lua/tests/runtime/` の `debug_*` 3 本、`lua_specs/` の kick 8 本と一覧）。Lua は `shiori/entry.lua` 130・`event/kick.lua`（消す）・`virtual_dispatcher.lua` 340・`store.lua` 127・`scene_play.lua` 86 か `external_control.lua` 123、位置を添えるなら `shiori/stop.lua` 212。`act.lua`・`element_gen.rs`・`scene.lua`・`search/context.rs` は触らない。1,000 行に近いものは無い。
- **規模**: 12〜16 タスクのまま（下の 1 つ目の穴しだいで 1〜2 減る）。
- **先に要るもの**: 機能の上では無い。ファイルの重なり（未完了）: `scene-count`（`entry.lua`。あちらが GLOBAL 関数を `pasta/global.lua` に置けば重ならない）、`scene-condition-attributes`（イベントの自動転記を `virtual_dispatcher.lua` に置くと重なる。`SCENE.co_exec` に置けば重ならない）、`scene-attribute-store`（属性をシーン表のフィールドに置くと `source_map/scene_join.rs` 87 行も直す要がある。`finalize_scene_impl` の署名を変えると `runtime/factory.rs` 120 行も動く。どちらもしなければ重ならない）、0.6.0 の `shiori4-v4-host`（`actor/`・キック）。`vscode-mcp-client` とはソースを共有しない（`SUMMARY.md` は別の行）。`scene-count`・`scene-attribute-store` の側の約束があれば、ウェーブ 2 に入れられる。
- **種別**: 基盤（開発の道具の通り道を DAP から開発用のイベントへ替え、DAP とキックを消す）。開発者が承認した移行。
- **要件定義のモデル**: Opus。撤去が主で、残す形（門・イベント・設定のキー）は brief で決まっている。
- **分割の案**: 不要。
- **見つけた穴・古くなった記述**:
  - 実行時エラーのバルーンとログの警告は、もう `.pasta` の位置を出している。`runtime-error-balloon`（#132）の `shiori/stop.lua` の `STOP.detail`・`STOP.location`（127・111 行）が、`act:init_scene` の最後の引数の行の対応（`transpiler.rs` の `fill_line_maps` 259 行。デバッグの有無にかかわらず生成コードに埋まる）で、変換した Lua の行を `dic\天気.pasta:15` の形に直す。Problem の「作者は `.pasta` の行へ手で戻している」と Approach 4 は、行の対応で引けない行（シーンの関数の外。アクター辞書の単語定義など）に絞るか、要らないかを、要件の最初に確かめる。`LineShift` が恒等になったので、行の対応とソースマップは同じ行を指す。
  - `source_map/mod.rs` 25 行の `normalize_output` は、まだ `normalize_output_with_shift` に直っていない（`ponytail-lua-transpiler` からの申し送りの 1 行目）。
  - Current State の `actor/instance.rs:150`・`entry.lua:132`・`virtual_dispatcher.lua:234-257` は、上の行に動いた。

## 申し送り（vscode-mcp-client より・2026-11-03）

- **info の行の文言**: `vscode-mcp-client` のマニュアル（`book/src/debug/index.md` の「有効になったかの確かめ方」・`book/src/debug/troubleshooting.md`）は、デバッグが有効になったことを仮の言い方「開発用のイベントを許したことを示す info の行が 1 行出る」で書いた。この spec で行の文言を決めたら、2 頁にその文言の引用を足す。
- **読み込みの終わりの印**: 今の pasta.dll は `SHIORI load entry completed … loaded=` を `info!` で書く（`pasta_shiori/src/windows.rs`・`v3.rs`）ので、`[logging] level`・`filter`・`PASTA_LOG` で外すと拡張は読み込みの終わりを確かめられない。`troubleshooting.md` はそう条件なしで書いてある。この spec で印の行を設定に関係なく出すようにしたら、`troubleshooting.md` のその項（と `dev-actions.md` の時間切れの案内）を直す。拡張は文と `loaded=` だけで判じるので、target を変えても拡張は直さなくてよい（`editors/vscode/src/test/reloadShiori.test.ts` の契約の試験は、文と `loaded` が 2 か所にあることだけを確かめる）。
- **`PASTA_DEBUG_PORT`**: `book/src/debug/vscode-setup.md` の「前の版から移るとき」は、`PASTA_DEBUG_PORT` を「拡張はもう番号を合わせてつながないので要らない」と書いた。この spec で環境変数を消したら、文はそのままでよい。
