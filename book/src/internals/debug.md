# デバッグ基盤とシーンキック

ごきげんよう。VSCode から `.pasta` の行で実行を止められる――その魔法の仕掛けを、わたくしが分解してお見せいたしますわ。
DAP の受け答えからソースマップ、シーンキックまで。仕掛けを知れば、もう怖いものなどございませんの。さあ、参りましょう。

---

この章では、DAP バックエンドを中心とするデバッグ基盤と、シーンキックを扱う。利用者から見たデバッグ操作（有効化の設定と優先順位・接続手順・`.pasta` 粒度の操作・提示モード・シーン再生と SHIORI リロード・ブレーク中の制約）は [デバッグ概要](../debug/index.md) 以下の章が正である。

## 目的と責務

デバッグ基盤は、DAP クライアント（VSCode の pasta 拡張）から TCP で接続を受け、実行中の Lua VM を行単位で止め、`.pasta` の座標でブレークポイント・ステップ・コールスタック・変数参照を提供する。シーンキックは、DAP の独自リクエストで指定された位置のシーンを、次の OnSecondChange で割り込み再生する。

この章が責務を持つのは次の事項である。

- 有効化の判定（`DebugConfig` の解決）と、バックエンドの起動・終了（`debug::enable`・`DebugHandle`）
- デバッグ通信: loopback に固定した TCP リスナー、DAP の `Content-Length` フレーミング、スレッド構成
- DAP の受け答え（`DapAdapter`）と、受信したリクエストの振り分け（wiring）
- セッション: 行フック、停止判定、停止ループ、ステップ、`.pasta` 行単位の停止の合体（アンカー）
- ブレークポイントの保持と照合、停止中の inspect（コールスタックと変数の取得）
- ソースマップ: ローダでの構築、構造、チャンク名の正規化、双方向の解決、任意のサイドカー出力
- シーンキック: シーン identity 索引、位置からシーンへの解決、`KickSink`、`pasta.shiori.event.kick` の保留と起動

次の事項は他の章が権威を持ち、この章では再記述しない。

- 生成器がソースマップへ対応を記録する入口（`transpile_with_source_map`・`SourceMapSink` の呼び出し位置・`LineShift`）は [ソースマップ生成の出入口](transpiler.md#ソースマップ生成の出入口) で扱う。
- ローダの段階構成の中でソースマップを作る位置（段階 5.5）は [起動の段階構成](loader.md#起動の段階構成) で、基準ディレクトリを `canonicalize` せず `std::path::absolute` で絶対化する理由（チャンク名とキャッシュのパスの一致）は [モジュール検索パス](loader.md#モジュール検索パス) で扱う。
- VM の構築順序の中で `debug::enable` とシーン identity 索引の突合が入る位置は [VM の構築とモジュール登録](execution-model.md#vm-の構築とモジュール登録) で扱う。
- `ActorMsg::Kick` が mailbox を通ってアクタースレッドで `SHIORI.kick` になるまでは [アクタースレッドのメッセージループ](shiori.md#アクタースレッドのメッセージループ) で、`STORE.kick_force` を消費する仮想イベントディスパッチャの判定順は [仮想イベントディスパッチャ（OnTalk・OnHour）](shiori.md#仮想イベントディスパッチャontalkonhour) で扱う。
- キックしたシーンのコルーチンが応答になり `STORE.co_scene` を置き換える流れは [継続トーク（チェイントーク）と co_scene の更新](execution-model.md#継続トークチェイントークと-co_scene-の更新) で扱う。

## 構成要素

### スレッドとデータの経路

デバッグが有効なとき、`debug::enable` は VM のスレッドのほかに 2 本のスレッドを起こし、トランスポートが内部でさらに 2 本を使う。スレッドをまたぐのは `std::sync::mpsc` のチャネルと、`Arc` で共有する 2 つの状態だけである。

```text
VM のスレッド（enable の呼び出し元。SHIORI ではアクタースレッド）
  行フック → DebugSession::on_line
    停止ループ: cmd_rx.recv() で無期限に待つ / inspect をこのスレッドで実行
        ▲ SessionCommand                         │ SessionEvent
        │                                        ▼
socket bridge スレッド（run_socket_bridge）    event encoder スレッド（run_event_encoder）
  Transport の唯一の所有者                       SessionEvent → DapAdapter::encode_event
  受信: inbound を 5ms ごとに poll                  → DAP のフレーム（serde_json::Value）
    → handle_inbound（応答の送信・BP の適用・      │
       コマンドの転送・キックの取り次ぎ）          │ out チャネル
  送信: out チャネルを吸い出してソケットへ  ◀─────┘
        │ inbound / outbound チャネル（Value）
        ▼
Transport の serve スレッド（accept の poll と書き込み）＋読み取りスレッド（フレームの解析）
        │
TCP 127.0.0.1:<port> ── DAP クライアント（1 接続だけ）

共有状態: BreakpointSet（Arc<Mutex<HashSet<Breakpoint>>>）・SharedSourceMode（Arc<AtomicU8>）
          DapAdapter（Arc<Mutex<…>>。socket bridge と event encoder が使う）
```

`mlua::Lua` は `!Send` であり、VM のスレッドから出ない。コールスタックや変数の取得は、停止ループの中で VM のスレッドが行う。

### Rust 側のモジュール

| 要素 | 所在 | 役割 |
| ---- | ---- | ---- |
| 入口と公開面 | `crates/pasta_lua/src/debug/mod.rs` | `pasta_shiori` を参照しない（依存は `pasta_shiori` → `pasta_lua` の一方向）。公開する型の再エクスポート |
| `DebugConfig` | `crates/pasta_lua/src/debug/config.rs` | `[debug]` と環境変数から有効化・待ち受けアドレス・提示モード・サイドカーの要否を決める純粋な解決（`resolve`）と、その包み（`from_env`・`from_file`）。待ち受けのホストは定数 `LOOPBACK`（`127.0.0.1`） |
| `enable` | `crates/pasta_lua/src/debug/enable.rs` | 無効なら何もせず `Ok(None)`。有効ならフック・トランスポート・2 本のスレッドを結線して `DebugHandle` を返す |
| `DebugHandle` | `crates/pasta_lua/src/debug/handle.rs` | 待ち受けアドレスの保持と、`Drop` での後始末（`terminated` の送信とポートの解放） |
| `DebugError` | `crates/pasta_lua/src/debug/error.rs` | `Bind`・`Protocol`・`Vm`・`Disconnected`。`mlua::Error` は `!Send` のため文字列にして運ぶ |
| `SourceMode`・`SharedSourceMode` | `crates/pasta_lua/src/debug/source_mode.rs` | 提示モード（`Pasta`・`Lua`）と、スレッド間で共有する実効モード（`AtomicU8`） |
| `Transport` | `crates/pasta_lua/src/debug/transport/mod.rs` | TCP の bind・accept（1 接続）と、ソケットとチャネルの橋渡し。Lua に触れない |
| フレーミング | `crates/pasta_lua/src/debug/transport/framing.rs` | `read_frame`・`write_frame`。`Content-Length` のバイト長と、本体の上限（16 MiB） |
| `DapAdapter` | `crates/pasta_lua/src/debug/dap/mod.rs` | 出力の `seq` の採番と応答・イベントの封筒。DAP の crate は使わず `serde_json` で手書きする |
| 受信の変換 | `crates/pasta_lua/src/debug/dap/decode.rs` | `decode_request` が DAP リクエストを `Decoded`（コマンド・即時応答・即時イベント・独自リクエストの引数）にする。`RELOAD_SENTINEL` もここにある |
| 送信の変換 | `crates/pasta_lua/src/debug/dap/encode.rs` | `encode_event` が `SessionEvent` を DAP の応答・イベントにする |
| 変換の部品 | `crates/pasta_lua/src/debug/dap/codec.rs` | 引数の解析と、フレーム・変数・ブレークポイントの JSON 化 |
| 応答の対応づけ | `crates/pasta_lua/src/debug/dap/pending.rs` | 遅れて返す応答の `request_seq` を種類ごとの FIFO で覚える `PendingTable` |
| ソースの提示 | `crates/pasta_lua/src/debug/dap/resolver.rs` | フレームの `(lua_source, lua_line)` を DAP の `source` と行にする `SourceResolver`。既定（生成 `.lua` のまま）と `.pasta` 用の 2 つ |
| wiring | `crates/pasta_lua/src/debug/wiring/mod.rs` | `SourceMapWiring`（ソースマップと実効モード。`pasta_active` で `.pasta` 化の要否を判定）と `SharedAdapter` |
| socket bridge と encoder | `crates/pasta_lua/src/debug/wiring/bridge.rs` | `run_socket_bridge`・`drain_outbound`・`run_event_encoder` |
| 受信の振り分け | `crates/pasta_lua/src/debug/wiring/inbound.rs` | `handle_inbound` と、提示モードの切替・位置からのキック・SHIORI リロードの独自リクエストの処理 |
| `.pasta` 化の結線 | `crates/pasta_lua/src/debug/wiring/resolver.rs` | `attach_pasta_resolver`（`SourceResolver` の差し替え）と `translate_pasta_breakpoints`（`.pasta` 行のブレークポイントの変換） |
| `DebugSession` | `crates/pasta_lua/src/debug/session/mod.rs` | 停止の状態機械。`RunMode`（`Running`・`Stepping`）と注入口 |
| 停止ループ | `crates/pasta_lua/src/debug/session/stop_loop.rs` | 行ごとの判定（`on_line_impl`）と停止ループ（`stop_loop`） |
| ステップの判定 | `crates/pasta_lua/src/debug/session/stepping.rs` | over・into・out の判定と、その `.pasta` 行単位への絞り込み |
| アンカー | `crates/pasta_lua/src/debug/session/anchor.rs` | 同じ `.pasta` 行でのブレークポイントの再ヒットを合体させる `update_break_anchor` |
| 行フック | `crates/pasta_lua/src/debug/hook.rs` | `install`。`jit.off()` と `set_global_hook(EVERY_LINE)`、ハンドラの panic の捕捉 |
| `BreakpointSet` | `crates/pasta_lua/src/debug/breakpoints.rs` | 提示ソースと実行座標の 2 段のキーを持つブレークポイントの共有ストア |
| inspect | `crates/pasta_lua/src/debug/inspect.rs` | `capture_stack`・`capture_variables`。`mlua::ffi` で `lua_getstack` などを直接呼ぶ |
| 型 | `crates/pasta_lua/src/debug/types.rs` | `SessionCommand`・`SessionEvent`・`Breakpoint`・`FrameInfo`・`Variable`・`ThreadId` など、チャネルを渡る `Send` な型 |
| ソースマップ | `crates/pasta_lua/src/debug/source_map/mod.rs` | `MapBuilderSink`・`ChunkSourceMap`・`SourceMap`・`canonicalize_chunk_name` |
| シーン identity 索引 | `crates/pasta_lua/src/debug/source_map/scene_index.rs` | `.pasta` の（ファイル, 行）から `(scene_id, parent)` を引く `SceneIdentityIndex` |
| シーンの突合 | `crates/pasta_lua/src/debug/source_map/scene_join.rs` | `build_scene_index`。トランスパイル時の記録と実行時のシーン名を突き合わせる |
| サイドカー | `crates/pasta_lua/src/debug/source_map/sidecar.rs` | `write_sidecar`・`read_sidecar`・`SidecarFile` |
| 位置の解決 | `crates/pasta_lua/src/debug/playscene.rs` | `resolve_and_kick`・`uri_to_pasta_path` |
| `KickSink` | `crates/pasta_lua/src/debug/kick.rs` | `KickRequest { scene }` と、外側のホストが注入するクロージャの型 |
| ソースマップの構築 | `crates/pasta_lua/src/loader/source_map_build.rs` | `PastaLoader::build_source_map` |
| 記録の受け口 | `crates/pasta_lua/src/code_gen/source_map.rs` | 生成器が呼ぶ `SourceMapSink` トレイトと `PastaPos` |

ランタイムとの接点は `crates/pasta_lua/src/runtime/mod.rs`（`enable` の呼び出しと `DebugHandle`・ソースマップの保持）、`crates/pasta_lua/src/runtime/factory.rs`（シーン identity 索引の突合）、`crates/pasta_lua/src/runtime/runtime_config.rs`（`RuntimeConfig::debug`・`with_kick_sink`）にある。これらのファイルは [ランタイム実行モデル](execution-model.md) の範囲である。

### Lua 側のモジュール

| モジュール | 役割 |
| ---------- | ---- |
| `pasta.shiori.event.kick` | `KICK.install`（キックの保留）と `KICK.try_dispatch`（保留の消費とシーンのコルーチンの生成） |
| `pasta.shiori.entry` | `SHIORI.kick(scene)` が `KICK.install` に委ねる |
| `pasta.shiori.event.virtual_dispatcher` | `dispatch` の入口で `STORE.kick_force` を消費し、`KICK.try_dispatch` を呼ぶ |
| `pasta.store` | `STORE.kick_pending`（保留中のシーン名）と `STORE.kick_force`（割り込みの許可）を持つ。`reset` で両方を初期値に戻す |

## 処理とデータの流れ

### 有効化の判定

有効化の判定は `DebugConfig::resolve` だけで行う。入力は `[debug]` の解析結果（`DebugFileConfig`）、環境変数 `PASTA_DEBUG`・`PASTA_DEBUG_PORT`・`PASTA_DEBUG_SOURCE_MODE`・`PASTA_DEBUG_SOURCE_MAP_SIDECAR` の解析結果、DAP の `attach` で渡された提示モードである。`resolve` は環境変数を読まない純粋な関数で、`from_env` が環境変数を読んで `resolve` に渡す。真偽値の環境変数は `parse_env_bool` で解析し、解釈できない値は「未指定」として扱う。

- `enabled` が偽のとき、`listen` は必ず `None` になる。真のときだけ `127.0.0.1:<port>` の `SocketAddr` を作る。ホストは定数で、設定で変える経路は無い。
- 各値の優先順位と既定値は利用者向け章が正である（[優先順位](../debug/index.md#優先順位)・[初期モードの指定](../debug/source-level.md#初期モードの指定)・[サイドカー出力（任意）](../debug/source-level.md#サイドカー出力任意)）。`from_env` の段階では `attach` の値は無く、`attach` による上書きは後述の wiring が実効モードに書き込む形で行う。

判定はローダとランタイムで 2 回行うが、入力は同じである。ローダは段階 5.5 で `DebugConfig::from_env` を呼び、有効ならソースマップを作る。`PastaLuaRuntime::from_loader_with_scene_dic` は `RuntimeConfig::with_debug_from_file_and_env` で同じ解決を `RuntimeConfig::debug` に載せ、`with_config_and_source_map` が `debug::enable` を 1 回だけ呼ぶ。`debug::enable` が `None` を返したとき、ランタイムはソースマップを保持しない。

### バックエンドの起動

`debug::enable(lua, cfg, source_map, kick_sink)` は、`cfg.enabled` が偽なら引数を捨てて `Ok(None)` を返す。真なら次の順に組み立てる。

```text
enable
 1. SharedSourceMode を cfg.source_mode で初期化する
 2. SourceMapWiring { source_map, source_mode } を作る（マップは提示モードに関わらず渡す）
 3. BreakpointSet を作る
 4. cmd（SessionCommand）と event（SessionEvent）のチャネルを作り、event の送信端を 1 本複製する
 5. DebugSession を作り、ソースマップ・cfg.source_mode・共有の実効モードを注入する
 6. hook::install(lua, session)        … jit.off() と行フック。失敗は DebugError::Vm
 7. Transport::start(cfg.listen)       … bind の失敗は warn ログと DebugError::Bind
    成功したら "debug backend listening" と実際のアドレスを info ログに出す
 8. DapAdapter を Arc<Mutex<…>> で作り、encoder → bridge のフレーム用チャネルを作る
 9. socket bridge スレッドを起こす（Transport・BreakpointSet・cmd の送信端・SourceMapWiring・KickSink を移す）
10. event encoder スレッドを起こす
11. DebugHandle（設定・待ち受けアドレス・停止フラグ・2 本のスレッド・複製した event の送信端）を返す
```

`enable` は `with_config_and_source_map` の最後（VM を作って `@pasta_search` などを登録した後で、`@pasta_config` などの登録と起動モジュールの `require` より前）に呼ばれる。そのため起動モジュールの評価中から行フックが働く。ポート 0 を指定したときは OS が割り当てたポートが `DebugHandle::local_addr` から読め、`PastaLuaRuntime::debug_local_addr` がそれを返す。

### デバッグ通信

`Transport::start` は、`listen` が `None` なら何も開かず、閉じた受信チャネルだけを持つ `Transport` を返す。`Some` なら `socket2` でソケットを作り、Windows 以外では `SO_REUSEADDR` を立ててから bind し、backlog 1 で listen し、非ブロッキングにして serve スレッドを起こす。Windows で立てないのは、立てると同じポートを使う 2 体目のゴーストの bind まで通ってしまうためである（Windows は立てなくても TIME_WAIT 後の再 bind ができる）。

```text
serve（Transport のスレッド）
 accept の poll: 停止フラグを見る → accept → WouldBlock なら 5ms 眠って繰り返す
 接続したら: ソケットをブロッキングに戻し、リスナーをすぐ drop する（以後 accept しない）
 読み取りスレッド: 停止フラグを見る → fill_buf（5ms の読み取りタイムアウト）
                   → データがあればタイムアウトを外して 1 フレームを読み切る → inbound へ送る
 書き込みループ:   停止フラグが立ったら溜まっているフレームを書き切って抜ける
                   recv_timeout(5ms) で outbound を待ち、届いたフレームを書く
 終わり:           shutdown(Both) → 読み取りスレッドを join
```

- フレームは `Content-Length: <N>\r\n\r\n` に続く `N` バイトの UTF-8 の JSON である。`N` は文字数ではなくバイト長で、ヘッダは名前の大小を区別せず `Content-Length` だけを見る。`N` が 16 MiB を超えるフレームは本体を確保する前にエラーにする。
- 読み取り・書き込み・接続待ちのどの待ちも、5ms ごとに停止フラグを見る。Windows ではローカルの `shutdown` が実行中のブロッキング受信を確実には解かないため、読み取りスレッドの終了はこのフラグの poll に依っている。
- `Transport` は Lua に触れない。DAP の意味は扱わず、`serde_json::Value` のフレームを運ぶだけである。

### DAP の受け答え

socket bridge は、`Transport` の唯一の所有者として、受信の poll（5ms）と、encoder が作ったフレームの送信を 1 本のスレッドで交互に行う（`Transport` は `!Sync` で、`mpsc` には `select` が無いため）。ループの前に 1 回 `attach_pasta_resolver` を呼ぶ。受信チャネルが閉じたら（クライアントの切断）、溜まったフレームを送ってから戻る。戻ると `Transport` が drop され、ポートが解放される。

1 つのリクエストを受け取ると、`handle_inbound` が次の固定の順で処理する。独自リクエストの 3 つは `command` の文字列で判定し、処理したらそこで戻る。

```text
handle_inbound(req)
 decode_request（DapAdapter のロックの中で）
 A   pasta/sourcePresentation … 実効モードの書き換えと `attach_pasta_resolver` → 応答 → イベント → RefreshPresentation の転送
 A'  pasta/playSceneAt        … 位置からシーンを解決して KickSink を呼ぶ → 成功かエラーの応答
 A'' pasta/reloadShiori       … RELOAD_SENTINEL を KickSink に渡す → 応答
 B   attach に sourcePresentation があれば実効モードを書き換え、attach_pasta_resolver を呼び直す
 C   即時の応答 → 即時のイベント（initialize の後の initialized）
 D   attach なら、実効モードを pasta/sourcePresentation イベントで通知する
 E   コマンドの振り分け: setBreakpoints は BreakpointSet に直接適用して応答を送る（セッションへ送らない）
                         それ以外のコマンドは cmd チャネルでセッションへ送る
```

リクエストごとの扱いは次のとおりである。

| リクエスト | 即時の応答 | セッションへのコマンド | 後から返すもの |
| ---------- | ---------- | ---------------------- | -------------- |
| `initialize` | `supportsConfigurationDoneRequest: true` と `initialized` イベント | — | — |
| `attach` | ack | — | 実効モードのイベント（D） |
| `configurationDone` | ack | — | — |
| `setBreakpoints` | — | （送らない。bridge が適用する） | bridge が作る `setBreakpoints` 応答 |
| `threads` | — | `Threads` | `threads` 応答 |
| `stackTrace` | — | `StackTrace` | `stackTrace` 応答 |
| `scopes` | `Locals` スコープ 1 つ（`variablesReference = frameId + 1`） | `Scopes` | （`SessionEvent::Scopes` は送信時に捨てる） |
| `variables` | — | `Variables` | `variables` 応答 |
| `continue` | `allThreadsContinued: true` | `Continue` | — |
| `next`・`stepIn`・`stepOut` | ack | `Next`・`StepIn`・`StepOut` | 次の `stopped` イベント |
| `disconnect` | ack | `Disconnect` | `terminated` イベント |
| その他 | なし（無視する） | — | — |

- 出力する応答とイベントは、`DapAdapter` の 1 本のカウンタで `seq` を振る。後から返す応答は、リクエストを受けたときに `PendingTable` へ種類ごとに `request_seq` を積み、対応する `SessionEvent` を変換するときに古い順に取り出す。TCP の 1 本の順序付きの流れなので、種類ごとの FIFO で対応がつく。取り出せなければ `request_seq` は 0 になる。
- フレームの `id` はコールスタックの中の位置（0 始まり）であり、スコープの `variablesReference` はそれに 1 を足した値である。変数はすべて葉として返し（`variablesReference: 0`）、表の中身は展開しない。
- `SessionEvent::Error` は `stderr` カテゴリの `output` イベントになる。

### 提示モードの切替

提示モード（`.pasta` の座標で見せるか、生成 `.lua` の座標で見せるか）の実効値は `SharedSourceMode` の 1 つのセルにあり、socket bridge が書き、`.pasta` 用の `SourceResolver` の差し替えと VM のスレッドのステップ判定が読む。

- `enable` は `cfg.source_mode` でセルを初期化する。
- `attach` の引数に `sourcePresentation` があれば、B でセルを書き換える。値は `SourceMode::parse` で解析し、不正な値は警告を出して `Pasta` にする。引数が無ければセルを変えない。
- `pasta/sourcePresentation` は、`mode` が `pasta` か `lua` のときだけセルを書き換える。それ以外の値ではセルを変えず、現在のモードを応答とイベントで返す。最後にセッションへ `RefreshPresentation` を送る。停止中なら、セッションは同じ停止理由で `stopped` を送り直し、クライアントに新しいモードでコールスタックを取り直させる。実行中なら、コマンドは次に止まるまで cmd チャネルに残る。
- `.pasta` 化が働くのは、`SourceMapWiring::pasta_active`（マップがあり、かつ実効モードが `Pasta`）のときだけである。マップは提示モードに関わらず渡されるため、`attach` や実行時の切替で `Lua` から `Pasta` へ移ることができる。

### セッション：停止の判定

`hook::install` は、まず引数なしの `jit.off()` で JIT をエンジン全体で止める（関数単位の `jit.off(true, true)` では後から読み込むチャンクやコルーチンがコンパイルされ、行フックが漏れる）。次に `set_global_hook(HookTriggers::EVERY_LINE, …)` を登録する。LuaJIT の `lua_sethook` は主状態に対して全体に効き、Lua 側で作ったコルーチンの行でも発火する。コールバックはハンドラを `catch_unwind` で包み、ハンドラが `Err` を返しても panic しても、常に `Ok(VmState::Continue)` を返す（LuaJIT はフックから yield できない）。panic の原因は `HookHandle` が持つ記録用の格納場所（`Arc<Mutex<Option<String>>>`。最初の 1 件だけ）に残るが、`enable` はこのハンドルを使わない。

行ごとに `DebugSession::on_line_impl` が次の順で判定する。

```text
on_line_impl(lua, debug)
 1. (source, line) を取る（source はチャンク名 @<絶対 .lua パス>、無ければ short_src）
 2. pasta = 実効モードが Pasta かつソースマップがある
    cur = pasta なら resolve_lua_to_pasta(source, line)、でなければ None
    suppress = pasta なら update_break_anchor(cur)、でなければ false
 3. BreakpointSet::should_pause(source, line) が真なら
      suppress なら何もせず続行（同じ .pasta 行での再ヒットを合体）
      そうでなければ cur をアンカーにして stop_loop（理由 breakpoint）
 4. Stepping なら
      step_should_stop（.lua 単位の判定）が真で、
      pasta なら pasta_step_should_stop でも真なら stop_loop（理由 step）
 5. 続行
```

- ブレークポイントの判定はステップより先に行う。ステップ中でも、別の場所のブレークポイントでは必ず止まる。
- アンカーは、直前にブレークポイントで止まった `.pasta` の位置である。1 つの `.pasta` 行は複数の `.lua` 行に展開されるため、その各行のブレークポイントで止まり直さないよう、アンカーと同じ `.pasta` 行の再ヒットは止まらずに通す。対応の無い `.lua` 行ではアンカーを保ち、別の `.pasta` 行に移ったときだけアンカーを消す。アンカーを置くのはブレークポイントでの停止のときだけである。

`step_should_stop` は、ステップを始めた停止点で捉えた `(thread, base_depth, start_line)` と、現在のスレッド・深さ・行を比べる。スレッドは実行中のコルーチンの `lua_State` のアドレス（`ThreadId`）で、`yield` と `resume` をまたいでも変わらない。深さは `capture_stack` のフレーム数である。

| 種類 | 止まる条件（同じスレッドのときだけ） |
| ---- | ------------------------------------ |
| over | 深さが `base_depth` より浅い、または同じ深さで行が `start_line` から変わった |
| into | 深さが `base_depth` より深い、または行が `start_line` から変わった |
| out | 深さが `base_depth` より浅い |

`.pasta` 単位のときは、`.lua` 単位で止まる行のうち、`.pasta` に対応しない行と、始点と同じフレームで始点と同じ `.pasta` 行に対応する行を通す。始点の `.pasta` 位置（`origin_pasta`）は、ステップを始めたときに解決しておく。

### セッション：停止ループ

`stop_loop` は `SessionEvent::Stopped`（停止理由とスレッド ID 1）を送り、`cmd_rx.recv()` で次のコマンドを無期限に待つ。待ちに時間制限は無い。

| コマンド | 動作 |
| -------- | ---- |
| `Continue` | `RunMode::Running` に戻して VM を進める |
| `Next`・`StepIn`・`StepOut` | 現在のスレッド・深さ・行と `origin_pasta` を捉えて `RunMode::Stepping` にし、VM を進める |
| `Disconnect` | `Terminated` を送って VM を進める |
| `StackTrace` | `capture_stack(lua, &lua.current_thread())` を送り、待ち続ける |
| `Variables { var_ref }` | フレーム `var_ref - 1` の `capture_variables` を送り、待ち続ける |
| `Scopes { frame_id }` | `Locals` スコープ 1 つを送り、待ち続ける |
| `Threads` | 固定のスレッド（ID 1、名前 `main`）を送り、待ち続ける |
| `RefreshPresentation` | 同じ停止理由で `Stopped` を送り直し、待ち続ける |
| `SetBreakpoints` | `BreakpointSet` に適用して待ち続ける（bridge はこのコマンドを転送しないため、本番の経路では届かない） |
| チャネルの切断 | VM を進める |

停止理由として送るのは `breakpoint` と `step` だけである。停止中は VM のスレッドが止まるため、SHIORI ではアクタースレッドが止まり、後続のリクエストも処理されない。利用者から見たこの制約と緩和策は [構造的制約と緩和策](../debug/constraints.md) が正である。セッションと `BreakpointSet` はランタイムと同じ寿命で、リクエストをまたいで保たれる。

### ブレークポイント

`Breakpoint` は 2 段のキーを持つ。提示ソース（`present_source`。IDE がブレークポイントを置いたファイルのパス）は置き換えのキーで、実行座標（`chunk` と `lua_line`）は停止判定のキーである。

- `BreakpointSet::register(present_source, entries)` は、同じ提示ソースのブレークポイントだけを捨てて `entries` を入れる。DAP の `setBreakpoints` がファイル単位で全量を送る仕様に合わせたもので、`.pasta` から変換したものと `.lua` に直接置いたものが同じチャンクを指しても互いに消し合わない。
- `should_pause(chunk, line)` は、フックが報告したチャンク名と各ブレークポイントの `chunk` を `canonicalize_chunk_name` で正規化して比べる。提示ソースは停止判定に使わない。ロックは判定の間だけ取り、停止ループに入る前に離す。ロックが毒されていれば止まらない。

`setBreakpoints` の適用は socket bridge が行い、VM が実行中でも即座に効く。

| 条件 | 適用 |
| ---- | ---- |
| `pasta_active` で、ファイルの拡張子が `.pasta`（大小を区別しない） | `translate_pasta_breakpoints`。行ごとに `resolve_pasta_to_lua` の全座標を登録して `verified: true`。対応が無ければ `nearest_pasta_line_with_mapping` で後ろにある最も近い対応行の座標を登録し、その行で `verified: true`。後ろにも無ければ元の行で `verified: false`。全行の座標を 1 回の `register` でまとめて入れる |
| それ以外 | `set_breakpoints`。提示ソース・`chunk` ともに渡されたパスとし、全行を `verified: true` で返す |

### inspect

`capture_stack` と `capture_variables` は、Lua の `debug` ライブラリを使わず、`mlua::ffi` で `thread.state()` を直接たどる。フックの中では `lua.current_thread()` が実行中のコルーチンを指すため、コルーチンの中で止まってもその本体のフレームに届く。

- `capture_stack` は `lua_getstack` をレベル 0 から最大 256 まで進め、`lua_getinfo("Snl")` でソース・行・関数名を読む。C のフレーム（`what == "C"`）は除く。
- `capture_variables` は、指定したレベルの `lua_getstack` に対して `lua_getlocal` でローカル変数を、`lua_getinfo("f")` と `lua_getupvalue` でアップバリューを読む。型は `number`・`string`・`boolean`・`table`（`table: 0x…` のアドレス表記）を区別し、それ以外は `<unsupported 型名>` にする。スタックの深さを入口で覚え、出口で `lua_settop` で戻す。
- どちらも `Result` を返さず、読めなかった分は空や途中までの結果にする。

### ソースマップ

#### 構築

デバッグが有効なとき、ローダは段階 5.5 で `PastaLoader::build_source_map(全 .pasta, CacheManager, サイドカーの要否)` を呼ぶ。ファイルごとに次を行い、失敗したファイルは警告を出して飛ばす。

```text
build_source_map_inner
 for .pasta ファイル
   読み込み → pasta_dsl::parse_str でパースし直す
   chunk_name = CacheManager::source_to_cache_path(ファイル)   … 実行時の require と同じキャッシュのパス
   sink = MapBuilderSink::new(ファイルのパス, chunk_name)
   transpile_with_source_map(…, Some(&mut sink)) → LineShift
   chunk_map = sink.finish(&LineShift)        … 正規化前の行を最終の .lua 行へ写す
   サイドカーが有効なら write_sidecar（失敗は警告だけ）
   SourceMap::insert_chunk(chunk_name, ファイルのパス, chunk_map)
 → Arc<SourceMap>
```

`MapBuilderSink` は `SourceMapSink` の実装で、`record_line` を正規化前の `.lua` 行から `PastaPos`（`.pasta` のファイルと行）への `BTreeMap` に入れ（同じ行は後勝ち）、`record_scene` をシーン宣言の記録（突合キーと宣言行）として宣言順に積む。`finish` は `LineShift::map` で各行を最終の `.lua` 行へ写し、正規化で消えた行を捨てる。シーン宣言の記録は `.pasta` の行なので写さない。

#### 構造

| 型 | 中身 |
| -- | ---- |
| `ChunkSourceMap` | 1 チャンクの前方写像（最終の `.lua` 行 → `PastaPos`。`BTreeMap` なので 1 つの `.lua` 行は高々 1 つの `.pasta` 位置に対応する）と、シーン宣言の記録 |
| `SourceMap` | 正規化したチャンク名 → `ChunkSourceMap`（`HashMap`）、逆引き（正規化した `.pasta` ファイル → `.pasta` 行 → `[(チャンク名, .lua 行)]`。チャンク名と行の昇順に並べる）、`.pasta` ファイルごとのシーン宣言の記録、シーン identity 索引の書き込み 1 回だけのスロット（`OnceLock`） |

`SourceMap` は構築後に変わらず、`Arc` で socket bridge（ソースの提示・ブレークポイントの変換・キックの解決）と VM のスレッド（ステップ判定・アンカー）が読む。唯一の例外はシーン identity 索引のスロットで、ランタイムの構築の最後に 1 回だけ書かれる。

#### チャンク名とファイル名の正規化

フックが報告するチャンク名は `@` で始まる絶対パスで、ローダが作るキャッシュのパスには `@` が無い。また Windows では、`require` のチャンク名が `package.path` の部分は `/`、モジュール名を展開した部分は `\` の混在になる。`canonicalize_chunk_name` はこれを 1 つの形に落とす。

1. 先頭の `@` を除く。
2. `\` を `/` に置き換える。
3. Windows では小文字にする（Windows 以外では大小を保つ）。

ファイルシステムには問い合わせない。`.pasta` のファイルのパス（DAP の `source.path` と `PastaPos::file`）も同じ関数で正規化する。格納と問い合わせの両方がこの関数を通るため、両者が一致する。この一致は、両方の側が同じ絶対パスの基準ディレクトリから作られていることを前提にしている（[モジュール検索パス](loader.md#モジュール検索パス)）。

#### 解決

| 関数 | 向き | 使う側 |
| ---- | ---- | ------ |
| `resolve_lua_to_pasta(チャンク名, .lua 行)` | `.lua` → `.pasta`。チャンクか行が見つからなければ `None` | `.pasta` 用の `SourceResolver`（`stackTrace` の各フレーム）、ステップ判定、アンカー |
| `resolve_pasta_to_lua(.pasta ファイル, 行)` | `.pasta` → 全 `(チャンク名, .lua 行)` | ブレークポイントの変換 |
| `nearest_pasta_line_with_mapping(.pasta ファイル, 行)` | 指定の行以降で対応を持つ最初の `.pasta` 行 | ブレークポイントの変換 |

`.pasta` 用の `SourceResolver` は、対応が見つからないフレームを既定の提示（生成 `.lua` のパスと行）にする。誤った `.pasta` の位置を出すことはない。

#### サイドカー

サイドカーは、チャンクの前方写像を生成 `.lua` の隣の `<.lua のパス>.map` に JSON（`version`（1）・`pasta_file`・`[.lua 行, .pasta 行]` の配列 `pairs`）で書いたものである。ランタイムはサイドカーを読まず、デバッグには常にメモリ上の `SourceMap` を使う（`read_sidecar` を呼ぶのはテストだけである）。シーン宣言の記録はサイドカーに入らない。

### シーンキック

#### シーン identity 索引

位置からのキックは、`.pasta` の（ファイル, 行）を、実行時のシーンの名前（`(scene_id, parent)`）に引く必要がある。実行時のシーン名は辞書確定（`finalize_scene`）で連番が付いて決まるため、トランスパイル時には決まらない。そこで、トランスパイル時に再現できる突合キーを記録し、辞書確定の後に突き合わせる。

- 生成器は `record_scene` で、グローバルシーンを `G:{サニタイズした名前}#{出現順}`、名前付きローカルシーンを `L:{親のサニタイズした名前}#{親の出現順}:{関数名}` として記録する（関数名は `挨拶_1` の形、または `__start__`）。出現順はサニタイズした名前ごとの順で、実行時の連番と一致する。
- ランタイムの構築の最後（`scene_dic` の `require` の後）に、`build_scene_index` が `collect_scenes` で実行時の `(グローバル名, ローカル名)`（`(会話1, 挨拶_1)` の形）を集める。グローバル名は末尾の ASCII 数字を連番として名前と分け（`split_runtime_global`）、`G:` の記録と突き合わせる。`L:` の記録は、親を同じ方法で実行時のグローバル名にし、その配下に同じ関数名のローカルシーンがあるときだけ採る。`__start__` の記録は索引に入れない（グローバルシーンの本体の範囲はグローバルの項目が覆う）。突き合わなかった記録も索引に入れない。
- 各シーンの範囲は、同じファイルの宣言行の昇順で「次の同レベル以上の宣言の前の行」まで（無ければファイルの末尾まで）とし、`SourceMap::set_scene_index` で書き込む。突合の失敗は警告だけで、起動は続く。

`SceneIdentityIndex::scene_at(ファイル, 行)` は次の順で解決する。

1. 行を範囲に含むシーンのうち、最も内側（ローカル）のもの。
2. 含むシーンが無ければ、その行以降で最も近い宣言のシーン。
3. それも無ければ見つからない。索引が書かれる前やファイルが未登録の場合も見つからない。

#### 位置からのキック要求と SHIORI リロード要求

`pasta/playSceneAt` の引数 `{ uri, line }` は、`decode_request` が厳密に解析する（`uri` が空でない文字列で、`line` が `u32` に収まる非負の数のときだけ有効）。socket bridge はこれを `resolve_and_kick` に渡す。

```text
resolve_and_kick(map, sink, uri, line)
 uri_to_pasta_path: file:// とホスト部を除く → パーセントデコード
                    → 先頭の /c: の / を除く → std::path::absolute（canonicalize は使わない）
 map.scene_at(パス, line)
   見つかった → KickRequest.scene を組む
                  グローバル: scene_id（例 会話1）
                  ローカル:   :parent:scene_id（例 :会話1:挨拶_1）
                → sink(KickRequest) を呼ぶ（結果を待たない）→ 成功の応答
   見つからない → sink を呼ばずにエラーの応答
```

`KickSink` が注入されていなければ、リクエストは認識するだけで、解決も応答もしない。ソースマップが無いとき、位置の引数が不正なときはエラーの応答（`success: false` と理由の `message`）を返す。エラーの文言と拡張側の表示は [シーン再生](../debug/dev-actions.md#シーン再生-シーンを実行) が正である。

`pasta/reloadShiori` は、予約した文字列 `RELOAD_SENTINEL`（`@@pasta/reloadShiori@@`）をシーン名として同じ `KickSink` に渡し、成功の応答を返す。この文字列は `crates/pasta_lua/src/debug/dap/decode.rs` と `crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua` の 2 か所に同じバイト列で書かれている。グローバルのシーン名はサニタイズで英数字と `_` だけになり、ローカルの合成名は `:` で始まるため、`@` と `/` を含みローカルの合成名でもないこの文字列は実際のシーンと衝突しない。

#### KickSink から VM まで

`KickSink` は `Arc<dyn Fn(KickRequest) + Send + Sync>` である。`pasta_lua` はこれを中身を知らずに保持し、socket bridge のスレッドから呼ぶだけである。`pasta_shiori` は `load` で `RuntimeConfig::with_kick_sink(Some(kick_sink()))` を渡す。クロージャの中身（`kick_into_mailbox`）は `MAILBOX` に `ActorMsg::Kick { scene }` を `try_send` し、アクタースレッドがそれを `SHIORI.kick(scene)` にする（[アクタースレッドのメッセージループ](shiori.md#アクタースレッドのメッセージループ)）。`MAILBOX` が空のとき、送信に失敗したときは捨てる。

#### キックの保留と起動（kick.lua）

`SHIORI.kick(scene)` は `KICK.install(scene)` を呼ぶだけで、シーンは動かさない。`KICK.install` は `STORE.kick_pending` にシーン名を入れ、`STORE.kick_force` を真にする。進行中の `STORE.co_scene` には触れない。連続したキックは `kick_pending` を上書きし、最後の 1 つだけが残る。

保留は、次の OnSecondChange の仮想イベントディスパッチャで消費される。`dispatch` の入口が `STORE.kick_force` を読んで偽に戻し（真ならその回だけ Status によるブロックを飛ばす）、OnHour と OnTalk の判定より前に `KICK.try_dispatch(act)` を呼ぶ（[仮想イベントディスパッチャ（OnTalk・OnHour）](shiori.md#仮想イベントディスパッチャontalkonhour)）。

```text
KICK.try_dispatch(act)
 1. kick_pending が nil なら nil（通常の判定へ進む）
 2. kick_pending を nil にする（解決できてもできなくても再発火させない）
 3. RELOAD_SENTINEL と完全一致なら、act:raw_script("\![reload,shiori]") と act:build() を行う
    コルーチンを返す（シーンは探さない）
 4. ^:([^:]+):(.+)$ に一致すれば（ローカルの合成名）
      SCENE.search(ローカル名, 親) の関数を、build を最後に呼ぶ関数で包んでコルーチンにする
    一致しなければ（グローバル）
      SCENE.co_exec(act, シーン名)
 5. コルーチンが作れなければ log.warn（seam=kick.unresolved）を出して nil
```

ローカルの合成名を `SCENE.co_exec` に渡さないのは、`co_exec` の検索（`act:find_scene`）が親のグローバル名を使わず、第 2 引数なしの検索ではローカル名が `__start__` になるためである。返したコルーチンは `EVENT.fire` が再開して応答にし、`STORE.co_scene` を置き換える。SHIORI リロードの場合は、その応答のさくらスクリプトでベースウェアが SHIORI を読み込み直し、`unload` でランタイムと `DebugHandle` が破棄されてポートが解放され、次の `load` で新しいバックエンドが同じポートを bind する。

## 境界の受け渡し

| 境界 | 渡す側 → 受け取る側 | 渡すもの | 所有 |
| ---- | ------------------- | -------- | ---- |
| socket bridge → VM のスレッド | `handle_inbound` → `DebugSession` | `SessionCommand`（mpsc） | VM のスレッドは停止ループの中でだけ受信する |
| VM のスレッド → encoder | `DebugSession` → `run_event_encoder` | `SessionEvent`（mpsc）。エラーは文字列 | `DebugHandle` も送信端を 1 本持ち、`Drop` で `Terminated` を送る |
| encoder → socket bridge | `run_event_encoder` → `drain_outbound` | DAP のフレーム（`serde_json::Value`） | ソケットへの書き込みは socket bridge だけが行う |
| socket bridge ⇔ Transport | `Transport::send`・`inbound` | `serde_json::Value` のフレーム | `Transport` は socket bridge が値で所有し、戻るときに drop する |
| socket bridge ⇔ VM のスレッド（共有） | `BreakpointSet`・`SharedSourceMode` | ブレークポイントの集合（`Arc<Mutex>`）・実効モード（`Arc<AtomicU8>`） | socket bridge が書き、VM のスレッドが行ごとに読む |
| `code_gen` → `debug` | 生成器 → `MapBuilderSink` | `record_line`・`record_scene` の呼び出し | 依存は `debug` → `code_gen` の向きで、`code_gen` は `debug` を参照しない（`SourceMapSink` と `PastaPos` は `code_gen` にある） |
| ローダ → ランタイム → バックエンド | `build_source_map` → `from_loader_with_scene_dic` → `debug::enable` | `Option<Arc<SourceMap>>` | ランタイムが 1 本、wiring とセッションがそれぞれ 1 本を持つ |
| ランタイム → ソースマップ | `build_scene_index` → `SourceMap::set_scene_index` | `SceneIdentityIndex` | 書き込みは 1 回だけ（`OnceLock`） |
| `pasta_lua` → `pasta_shiori`（キック） | socket bridge → `KickSink` | `KickRequest { scene }`（シーン名の文字列だけ） | `pasta_lua` は `pasta_shiori` を参照しない。クロージャは `pasta_shiori` が作り、`RuntimeConfig` で渡す |
| Rust → Lua（キック） | `PastaShiori::kick` → `SHIORI.kick` | シーン名・合成名・`RELOAD_SENTINEL` の文字列 | 保留の状態は `STORE` が持ち、Rust 側はコルーチンを持たない |

## 不変条件と制約

- デバッグは opt-in である。無効のとき `debug::enable` は何もせず、行フック・`jit.off()`・ポート・スレッドのいずれも作らない。ローダはソースマップを作らず、生成 `.lua` のバイト列は変わらない。`KickSink` は渡されても使われない。
- 有効・無効に関わらず、Lua の `debug` ライブラリ（`std_debug`）をスクリプトに開かない。行フックは Rust 側の `set_global_hook` で、inspect は `mlua::ffi` で行う。
- 待ち受けアドレスは IPv4 の loopback（`127.0.0.1`）に固定される。変えられるのはポートだけである。
- 1 回の `enable` で受け付ける接続は 1 つである。接続した時点でリスナーを閉じ、クライアントが切断すると socket bridge が終わってポートが解放される。同じランタイムに再び接続する経路は無く、ランタイムの作り直し（SHIORI の読み込み直し）で新しいリスナーができる。
- ポートは `DebugHandle` の `Drop` が戻る前に解放される。`Drop` は `Terminated` を送り、30ms 待ってから停止フラグを立て、socket bridge を join する。socket bridge が `Transport` を drop すると、`Transport` の `Drop` が serve スレッドを join し、serve スレッドが読み取りスレッドを join する。どの待ちも 5ms ごとに停止フラグを見るため、join は有限時間で終わる。encoder スレッドは join しない（`DebugHandle` が event の送信端を持っているため、join すると互いに待つ）。
- `mlua::Lua` はスレッドをまたがない。チャネルを渡る値はすべて `Send` である。
- 行フックのコールバックは常に `Ok(VmState::Continue)` を返す。停止は VM のスレッドを止めることで実現し、停止ループの待ちに時間制限は無い。
- デバッグが有効な間は JIT がエンジン全体で止まる。
- リリースビルドは `panic = "abort"` のため、行フックの `catch_unwind` が働くのは dev・test のビルドだけである（[DLL のビルド構成](shiori.md#dll-のビルド構成)）。
- socket bridge はロックが毒されても panic しない（その場で処理を終えるか、何もしない）。
- `SourceMap` は構築後に変わらない。シーン identity 索引だけが 1 回書き込まれる。
- `.pasta` の座標への変換は、マップがあり実効モードが `Pasta` のときだけ働く。それ以外では生成 `.lua` の座標のまま扱う。対応の無い位置を別の `.pasta` 位置に結びつけることはない。
- ローカルの合成名（`:` で始まる `KickRequest.scene`）と `RELOAD_SENTINEL` は、デバッグのキックの経路でだけ生じる。通常のトークの再生はこれらを作らない。
- キックは要求を受けた時点ではシーンを動かさない。実行は次の OnSecondChange（`act.req.date` があるもの）まで遅れ、1 回の OnSecondChange で起動するキックは 1 つだけである。
- `RELOAD_SENTINEL` は Rust と Lua の 2 か所に書かれており、片方だけ変えるとリロードが届かない。
- テストの環境: 開発環境で `PASTA_DEBUG=1` が設定されていると、`PastaLoader::load` を呼ぶテストがすべて固定ポートに bind しようとして `AddrInUse` で失敗する。そのため、ロードを伴う結合テストは `#[ctor]` で `main` の前に `PASTA_DEBUG`・`PASTA_DEBUG_PORT` を消している（`crates/pasta_lua/tests/common/mod.rs` の `neutralize_debug_env`、`crates/pasta_shiori/tests/` の各アクターのテスト）。ロードを伴うテストを追加するときは同じ処置が要る。クレート内の単体テストは環境変数を読まない `DebugConfig::resolve` を使う。

## ソースの所在

- `crates/pasta_lua/src/debug/`
- `crates/pasta_lua/src/loader/source_map_build.rs`
- `crates/pasta_lua/src/code_gen/source_map.rs`
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua`

テストは `crates/pasta_lua/src/debug/` の `*_tests.rs`・`*_e2e.rs`・`crates/pasta_lua/tests/loader_source_map_build_test.rs`・`crates/pasta_lua/tests/scene_identity_index_test.rs`・`crates/pasta_lua/tests/chunk_name_validation_test.rs`・`crates/pasta_lua/tests/lua_specs/kick_try_dispatch_test.lua`・`crates/pasta_shiori/tests/scene_kick_e2e_test.rs`・`crates/pasta_shiori/tests/actor_kick_test.rs` にある。

## 経緯

- [pasta-lua-debug-feasibility](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-lua-debug-feasibility) — 行フック・inspect・ステップの実現性の検証（PoC）
- [pasta-vscode-lua-debug](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-vscode-lua-debug) — DAP バックエンド（トランスポート・`DapAdapter`・セッション・wiring）
- [pasta-source-map](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-source-map) — ソースマップと `.pasta` 座標での提示・ブレークポイント・ステップ
- [pasta-debug-break-coalesce](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-debug-break-coalesce) — 同じ `.pasta` 行でのブレークポイントの合体（アンカー）
- [pasta-debug-lua-view-toggle](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-debug-lua-view-toggle) — 提示モードの実行時の切替（`pasta/sourcePresentation`）
- [debug-transport-hardening](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/debug-transport-hardening) — 後始末でのポートの同期解放
- [debug-startup-logging](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/debug-startup-logging) — 待ち受け開始と bind 失敗のログ
- [pasta-scene-kick](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-kick) — `KickSink`・`ActorMsg::Kick`・`kick.lua` の保留と起動
- [pasta-scene-kick-from-cursor](https://github.com/ekicyou/pasta/tree/main/.kiro/specs/completed/pasta-scene-kick-from-cursor) — シーン identity 索引、`pasta/playSceneAt`、`pasta/reloadShiori`

---

仕掛けが分かれば、バグなど恐るるに足りませんわ。おほほ、頼もしい道具でしょう？
最後は、縁の下の力持ち――ロギングとエンコーディングへ参りましょう！
