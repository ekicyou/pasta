# ギャップ分析: shiori4-v3-portable

- 分析日: 2026-10-25
- 基準: pasta main `b16ef6bb`（このワークツリーの起点）。shiori4 はローカルの clone（`C:\home\maz\git\shiori4`、タグ `v0.1.0` の後の `04e4318`。crates.io の 0.1.0 と同じ版）の `shiori4_abi/src/lib.rs`・`shiori4_host/src/discover.rs`・`doc/textbook/testhost.md` を読んだ。
- 対象: `requirements.md` の Requirement 1〜11 と OQ-1〜13。

## 1. 分析の要約

- **入口は小さい**: `shiori4_abi` 0.1.0 の `Shiori` トレイト（`SUPPORTS_V3`・`load(mode, dir)`・`request`・`notify`・Drop＝unload）と `ShioriApi::new::<S, 3>()` が、ABI の検査・UTF-8 の検査・`copy_from_slice` での写し・`catch_unwind` を全部持つ。pasta が書くのは、自前の `shiori_get_api`（版 3 だけ・10 行ほど）と、`Shiori` の実装（今のアクターを包む）と、応答の開始行からステータスコードを読む小さな関数だけである。`export_shiori!` は版 4 にもテーブルを返すので使えない（brief のとおり。`lib.rs` 1003〜1018 行で確かめた）。
- **いちばん大きいのはインスタンス化**: 今のアクターは `actor/lifecycle.rs` の `static MAILBOX`・`static ACTOR_HANDLE` と、`kick_sink()` が `MAILBOX` を読む作りに縛られている。`lifecycle` の自由関数（`spawn_actor`・`marshal_request`・`teardown_actor`）か旧来の入口を通して静的な置き場に触る試験が、`tests/` に 12 ファイル（`actor_marshaling`・`actor_teardown`・`ffi_*` の 3 本・`load_failure_visibility`・`path_robustness_e2e`・`runtime_error_balloon_e2e`・`scene_kick_*` の 4 本）と、`src/windows_tests.rs` にある。インスタンスの型を作り、旧来の入口の置き場とテストの書き換えを決めるのが、この spec の中心の仕事になる。
- **実行器は外すだけで済む見込み**: アクターの中の待ちは `thread.rs` 173 行の `rx.recv_async().await` の 1 つだけで、`wintf-winmsg-executor` は `block_on` にしか使っていない（brief の再測定のとおり）。flume の同期の `recv()` のループにすれば、依存が 1 つ減り、全部の OS で同じ形になる。`wintf-winmsg-executor` は `windows` クレートを無条件に引くので、Linux ではそもそもビルドできない（`Cargo.toml` で確かめた）。
- **最大の不確かさは Linux のテスト**: CI は Linux で一度も cargo を回していない。`pasta_lua` には `cfg(windows)` の分けがすでにあるが、`"C:\\…"` の文字列を持つファイルが 11、Lua の生成の改行がプラットフォームで変わる（`LineEnding::native()`）所、`.gitattributes` が無いリポジトリの CRLF のフィクスチャ、`pasta_check`・`pasta_sample_ghost` の `.exe`・`pasta.dll` を前提にするテストがある。数は実際に回すまで分からない（Research Needed）。
- **推奨**: 選択肢 C（ハイブリッド）。芯のインスタンスの型を新しく作り、版 3 の口と旧来の口の両方をそこへつなぐ。実行器は全 OS で同期の `recv()` ループに替える。CI は Linux・macOS のジョブと試験用のホストの段を足す。工数 L、リスク Medium〜High（Linux のテストの未知の量による）。

## 2. 現状の調査

### 2.1 入口（`crates/pasta_shiori/src/windows.rs`、335 行・`cfg(windows)`）

- `DllMain`（FreeLibrary の detach で `unload()`、プロセスの終了では何もしない）、`loadu`（UTF-8）・`load`（ANSI。`loadu` の後は何もしない）・`request`・`unload`。戻り値は `-> bool`（brief の Adjacent: BOOL は i32 であるべき）、長さは `usize`／`&mut usize`（Windows の `long` は 32 ビット）。
- `load_entry` → `load_impl` → `lifecycle::spawn_actor(0, dir)`。`LOADU_INITIALIZED`（`AtomicBool`）がプロセスに 1 つ。
- `request_impl` → `lifecycle::marshal_request(req)` → HGLOBAL に載せる。UTF-8 でない要求は 204。panic は `catch_unwind` で 204（unwind のビルドだけ）。
- `lib.rs` は `#[cfg(windows)] mod windows;` と `pub use windows::{load, loadu, request, unload}` を持つ。`util/hglobal` も `cfg(windows)`。

### 2.2 アクター（`crates/pasta_shiori/src/actor/`）

| ファイル | 行数 | 今の役目 | この spec での当たり方 |
|---|---|---|---|
| `lifecycle.rs` | 329 | `static MAILBOX: ArcSwapOption<Sender<ActorMsg>>`・`static ACTOR_HANDLE: Mutex<Option<ActorThread>>`・`static REQUEST_SEQ`・`spawn_actor`・`marshal_request`・`teardown_actor`・`kick_sink`（`MAILBOX` を読む） | インスタンスの型へ移す中心 |
| `thread.rs` | 272 | `spawn_actor_thread`。`wintf_winmsg_executor::block_on(async { … rx.recv_async().await … })` | 同期の `recv()` ループへ |
| `mailbox.rs` | 417 | `ActorMsg{Get,Notify,Kick,Stop}`・flume unbounded | 変えない見込み |
| `marshaling.rs` | 506 | `determine_method`（pest の読み取り）・`marshal_get`（`GET_TIMEOUT` 5 秒）・`marshal_notify`・`default_204` | 版 3 の 400 の判定に `determine_method` を使える |
| `teardown.rs` | 345 | `teardown_via_sender`・漏れの測り（`cfg(windows)` で `GetProcessHandleCount`・`GetGuiResources`、ほかは `None`） | 文言（ウィンドウ）の直し。ロジックはそのまま |
| `mod.rs` | 42 | モジュールの説明（wintf に触れる） | 文言の直し |

- `shiori.rs`（397 行）: `PastaShiori::load` が `GlobalLoggerRegistry::instance().register(load_dir, …)`・`init_tracing_with_reload(..)`（89 行）・`RuntimeConfig::new().with_kick_sink(Some(lifecycle::kick_sink()))` を呼ぶ。インスタンスごとの kick の送り先は、ここで自分の `Sender` を閉じ込めた sink を渡せば足りる（`KickSink = Arc<dyn Fn(KickRequest) + Send + Sync>`、`pasta_lua/src/debug/kick.rs` 37 行）。
- ロードの失敗: `PastaShiori::load` は `Ok(false)` を返すが、アクターは生きたまま `MAILBOX` に登録され、以後の GET は理由つきの 500（`load_failure_visibility_test.rs`）。ディレクトリが無いときはロガーを作る前に返るので理由の無い 500（`ponytail-shiori` の 2 番）。版 3 の NULL（Requirement 2.4・OQ-1）は、この「失敗しても残す」と逆の向きになる。

### 2.3 プロセスに 1 つのもの（brief の一覧の確かめ）

| もの | 場所 | 今の害 | 要件の扱い |
|---|---|---|---|
| `MAILBOX`・`ACTOR_HANDLE` | `actor/lifecycle.rs` | 2 つ目の `spawn_actor` は 1 つ目を teardown する | インスタンスへ移す（R5.1） |
| `kick_sink` の送り先 | `lifecycle.rs` 184〜227 行 | デバッガの頼みが「今の MAILBOX」へ行く | 自分の `Sender` を閉じ込める（R5.5） |
| `LOADU_INITIALIZED` | `windows.rs` | 旧来の入口だけの印 | 旧来の入口に閉じ込めたまま（R5.7） |
| `FILTER_HANDLE`・tracing の購読者 | `pasta_lua/src/logging/tracing_init.rs` | 後から読んだインスタンスの `[logging]` が全体に効く | インスタンスごとに分ける（R5.4・OQ-6） |
| `REGISTRY`（ロガーの登録） | `pasta_lua/src/logging/registry.rs` | ディレクトリが鍵。違うディレクトリなら複数のゴーストを前提にした作り。同じディレクトリだと取り合う | プロセスに 1 つ残す（R5.7）。同じディレクトリの 2 つ目を断る見分けにも使う（R6.2。2026-10-25 確定） |
| `CURRENT_LOAD_DIR` | `registry.rs` の thread_local | スレッドごとなので害は無い | そのまま |
| `REQUEST_SEQ`・`UNIQUE_COUNTER` | `lifecycle.rs`・`pasta_lua/src/loader/extract.rs` 55 行 | 数を振るだけ | そのまま（R5.7） |

### 2.4 Windows だけの依存と、すでにある分け

- まだ囲っていない Windows だけの依存は `wintf-winmsg-executor`（`pasta_shiori` の通常の依存、ルート `Cargo.toml` 47 行）だけ。
- すでに囲ってあるもの: `pasta_shiori` の `windows-sys`（`cfg(windows)` の依存と dev 依存）・`mod windows`・`util/hglobal`、`teardown.rs` の測り、`pasta_lua` の `encoding/{windows,unix}.rs`・`runtime/ffi_load`（`cfg(windows)`。Linux では `ffi.load` を置き換えない）・`searcher.rs` の区切り・`config.rs` の改行、`pasta_lua` の `windows-sys`、`tests/ffi/main.rs`、`tests/ffi_*.rs`（3 本とも `#![cfg(windows)]`）。
- `build.rs` の `embed-resource` は Windows 以外では何もしない（コメントのとおり）。`.cargo/config.toml` の crt-static は i686・x86_64 の MSVC だけに掛かる（Linux・macOS には当たらない。Requirement 9.3 に合う）。
- cdylib の名前は `[lib] name = "pasta"` なので、Linux は `libpasta.so`、macOS は `libpasta.dylib` がそのまま出る（shiori4 のファイル名の決まりに合う。Requirement 9.1・9.2）。

### 2.5 shiori4 0.1.0 の API で確かめたこと

- `pub trait Shiori: Sized + 'static { const SUPPORTS_V3: bool = false; fn load(mode: ShioriLoadMode, dir: &str) -> Result<Self, ShioriStatus>; fn request(&mut self, req: &str) -> ShioriResponse; fn notify(&mut self, _req: &str) -> ShioriStatus { 204 } }`。unload は Drop。`Send` は要らない（インスタンスはホストの 1 スレッドから呼ばれる）。
- `shiori_load::<S, 3>` は、`copy_from_slice` の欄があるかを確かめ（無ければ NULL・R2.6 は自動で満たされる）、dir の UTF-8 を確かめ、`S::load(ShioriLoadMode::V3, dir)` を呼ぶ。`Err` なら NULL（版 3 ではホストにログを出さない）。
- `shiori_request` は `S::request` の `ShioriResponse { status, body }` の body を `copy_from_slice` で `out` に写し、`status` を戻り値にする。戻り値と開始行のコードを合わせるのは pasta の責任（R3.1・R3.7）。`ShioriBytes` は `From<String>` を持つ。
- `guard`／`catch_unwind` で `SHIORI_E_PANIC` を返すが、pasta のリリースは `panic = "abort"`（ルート `Cargo.toml`）なので届かない（R9.7）。
- 規約: 版 3 の request は NOTIFY の全文も受ける。notify に GET が来たら 400。SHIORI/2.x は 400。UTF-8 固定。「unload は長く止まらないこと」。「SHIORI は 2 つ目のインスタンスを拒否してよいが、できれば受ける」。
- 試験: `shiori4_host::load_ghost_with(lib, dir, host, &[EntryKind::V3])` で Rust の試験からライブラリの境界まで試せる。`shiori4_host` の依存は `shiori4`（→`shiori4_abi`）と `libloading 0.8`（ISC。`deny.toml` の許可に入る）。試験用のホスト `shiori4-testhost` は v3 のとき `status:` を応答の開始行から読む（戻り値ではない）。終了の状態 0＝2xx。

### 2.6 CI（`.github/workflows/build.yml`、149 行）

- `build` は `windows-latest` の x86・x64 の行列で、ビルド・`cargo test --all`・長いパスの FFI の試験・clippy（x64 だけ）・成果物の upload。Linux のジョブ（`deny`・`luacheck`・`wasm`・`release-scripts`）は cargo test を回していない。
- `build.yml` は `workflow_call` でリリース CI の関門にもなる。Linux・macOS のジョブを足すと、その失敗はリリースも止める（R10.4）。

### 2.7 文書

- `book/src/internals/shiori.md`: 「スレッドとデータの経路」（39 行〜）・「load とアクターの起動」（111 行〜）・「アクタースレッドのメッセージループ」（186 行〜。188 行に wintf）・「unload・DllMain と teardown」（218 行〜。247 行にメッセージ専用ウィンドウ）・「DLL のビルド構成」（464 行〜）・「不変条件と制約」（498 行〜）が当たる。
- `crates/pasta_shiori/README.md` 92 行（wintf の行。消す）、`tech.md` 62 行（wintf）・64 行（エンジン駆動モデル）・CI の節（199 行〜）。

## 3. 要件と資産の対応表

タグ: **Missing**（無い・作る）／**Unknown**（調べが要る）／**Constraint**（既存の作りや外の決まりによる縛り）／**Exists**（今のものを使える）

| 要件 | 使える今の資産 | ギャップ |
|---|---|---|
| R1 入口の書き出し | `shiori4_abi::ShioriApi::new` | **Missing**: 自前の `shiori_get_api`（版 3 だけ）。**Constraint**: `export_shiori!` は使えない |
| R2 版 3 の load | `PastaShiori::load`・`spawn_actor_thread`（`loaded()` を待って返る） | **Missing**: `Shiori::load` の実装。失敗で NULL にするとき、起こしたアクターを止めて片付ける道（今は失敗しても残す）。**Constraint**: ディレクトリが無いとログを置けない（OQ-1） |
| R3 request・notify | `marshal_get`・`marshal_notify`・`determine_method`・`default_204` | **Missing**: 開始行のコードの読み取り、版 3 だけの 400（SHIORI/2.x・notify の GET・読めない要求）。**Constraint**: pest の文法（`req_parser.pest`）は 2.x も受けるので、版 3 の口で弾く |
| R4 unload | `teardown_via_sender`（`TEARDOWN_TIMEOUT` 5 秒） | **Exists**（インスタンスの Drop から呼ぶだけ）。**Unknown**: ライブラリを外すこととの競り（OQ-8） |
| R5 インスタンスごとの状態 | `PastaShiori`（もともとインスタンスごと）・`REGISTRY`（ディレクトリで振り分け） | **Missing**: インスタンスの型（`Sender`＋`ActorThread`）、自分の `Sender` を閉じ込めた kick の sink。**Unknown**: ログの量をディレクトリごとに分ける作り（OQ-6） |
| R6 2 つ目の load | `REGISTRY` の振り分け・デバッグのポートの取り合いの扱い（`pasta_lua/src/debug/enable.rs`） | **Missing**: 同じディレクトリの検出（生きているインスタンスのディレクトリの集合。これはプロセスに 1 つの状態になる——R5.7 の例外の書き足しが要る）|
| R7 旧来の入口 | `windows.rs` 全体・既存の試験 | **Missing**: 旧来の入口のインスタンスの置き場（`windows.rs` の中）。BOOL の i32 化。**Constraint**: SSP の振る舞いのバイトを変えない |
| R8 実行器 | flume の同期 `recv()` | **Missing**: `block_on` を外す書き換え（`thread.rs` の 1 か所）。wintf の依存の削除 |
| R9 Linux・macOS | `pasta_lua` の `cfg(windows)` の分け | **Unknown**: Linux で落ちるテストの数と種類。macOS の LuaJIT（vendored）のビルドの条件 |
| R10 CI | `build.yml` の Windows のジョブの形 | **Missing**: Linux・macOS のジョブ、`shiori4-testhost` の取り込み（`cargo install shiori4_host --version 0.1.0 --example shiori4-testhost`）と段 |
| R11 文書 | `internals/shiori.md` の各節 | **Missing**: 2 つの入口・インスタンス・実行器の記述への書き換え |

## 4. 実装の選択肢

### 選択肢 A: 今の static を残し、版 3 の口を上に載せる

- `lifecycle` の `MAILBOX` をそのまま使い、版 3 の `Shiori::load` から `spawn_actor` を呼ぶ。2 つ目の `load` は断る（NULL＋ログ）。
- 触るもの: 新しい入口のファイル 1 つ、`thread.rs`（実行器）、`Cargo.toml`、`build.yml`。
- ✅ いちばん小さい（10 タスク前後）。既存の試験はそのまま。
- ❌ 2026-10-23 の開発者の決定（エンジンはインスタンスごと・グローバルな状態を避ける）と R5・R6 に反する。`shiori4-v4-host` で作り直しになる。
- 参考として挙げる。要件のままでは取れない。

### 選択肢 B: インスタンスの型を作り、`lifecycle` の自由関数を消す

- 芯の型（名前は設計で決める。例: アクター 1 つを持つ「インスタンス」）が `Sender<ActorMsg>`・`ActorThread`・ディレクトリを持ち、`load`／`request`／Drop（teardown）を持つ。kick の sink は自分の `Sender` の clone を閉じ込める。
- 版 3 の口は芯を `shiori4_abi::Shiori` で包む。旧来の口は `windows.rs` の中に `Mutex<Option<芯>>`（または `ArcSwapOption`。送信の道にロックを置かない R8 の決まりを保つなら後者）を 1 つ持つ。
- `lifecycle.rs` の static と自由関数を消し、これらの試験を芯の型へ書き換える。
- ✅ 決定どおり。試験はインスタンスを自分で持つので、`MAILBOX_TEST_LOCK` の直列化が要らなくなる（`ponytail-shiori` の 5 番も消える）。
- ❌ 試験の書き換えが広い。旧来の入口の試験（`windows_tests.rs`）は Windows だけになるが、`scene_kick_*`・`load_failure_visibility` などは全 OS で走らせたいので、芯の型の上で書き直す。

### 選択肢 C: ハイブリッド（推奨）

- 芯の型は B と同じに作る。ただし移行を 2 段に分ける。
  1. 芯の型を作り、`lifecycle` の自由関数を「旧来の入口の置き場（芯を 1 つ持つ）」への薄い窓口として残す（OS を問わずコンパイルする）。既存の試験は、そのまま全 OS で走る（`ffi_*` の 3 本と `windows_tests.rs` は今どおり Windows だけ）。`windows.rs` はこの窓口を呼ぶ。
  2. 版 3 の口と、複数のインスタンスの試験（R5・R6）は、芯の型を直に使う。
- 置き場を `windows.rs` の中に閉じ込める brief の形（「HGLOBAL の口は、芯を 1 つだけ置く場所を `windows.rs` に閉じ込め」）とは少しずれる。置き場は「旧来の入口のための 1 つ」と名前で分かる所に置き、アクターと Lua の側にはグローバルな状態を持ち込まない、という決定の芯は守れる。窓口を残すか、B のように書き換えきるかは設計で決める。
- ✅ 安全で戻しやすい（特性化の試験を先に全 OS で通してから、芯へ移せる。メモリの「リファクタリングは安全かつ可逆に」に合う）。差分が読みやすい。
- ❌ 窓口が残る分、`ponytail-shiori` で片付ける物が 1 つ増えうる。

### 実行器の小さな選択肢

| 案 | 中身 | 評価 |
|---|---|---|
| E1（推奨） | 全 OS で `while let Ok(msg) = rx.recv()` の同期ループ。wintf を外す | 依存が減る。async は要らない（待ちは 1 か所）。Windows でウィンドウを作らなくなる（R8.2） |
| E2 | Windows は wintf のまま、ほかは別の `block_on` | 2 つの道を CI で守ることになる。メッセージ専用ウィンドウが要る事情は見つかっていない |
| E3 | 小さな executor のクレートを足す（全 OS） | 待ちが 1 か所しかないので過剰 |

### ログの量の小さな選択肢（OQ-6）

| 案 | 中身 | 評価 |
|---|---|---|
| L1 | 購読者は 1 つのまま、書き出す層のフィルタを「今のスレッドの `CURRENT_LOAD_DIR` のロガーの設定」で決める（ディレクトリごとの `EnvFilter`） | 要件どおり。tracing のコールサイトの関心のキャッシュ（`Interest`）を「sometimes」にする必要があり、作りの調べが要る（Research Needed） |
| L2 | アクタースレッドごとに `tracing::dispatcher::set_default` で購読者を持つ | FFI 入口のスレッドのログ（`windows.rs` の `info!(entry, loaded, …)`）は既定の購読者へ行くので、取りこぼしの扱いが要る |
| L3 | 共有のまま、理由を書いて閉じ込める | 最小。R5.4 を落とし、R5.7 の例外に入れることになる |

## 5. 規模とリスク

- **工数: L（1〜2 週）**。入口と実行器は S〜M。インスタンス化と試験の書き換え、Linux・macOS の CI を初めて通す作業が大きい。brief の見積もり 14〜18 タスクに収まる見込み。Linux のテストの直しが 10 ファイルを大きく超えるなら、brief の分割の案（`pasta-portable-build` と分ける）を考える。
- **リスク: Medium〜High**。入口と実行器は手本（shiori4 の教科書・CI）があり Medium。Linux で初めて回すワークスペースのテストの量が読めないこと、macOS の LuaJIT の vendored ビルドを試したことが無いことが High の理由。

## 6. 設計への申し送り

### 推奨の方針

- 選択肢 C（芯の型＋旧来の入口の窓口）と E1（同期 `recv()` ループ）。
- 版 3 の口の判定の順: `determine_method` で GET／NOTIFY を決める → SHIORI/2.x（`header2`）を 400 → `request` は GET を `marshal_get`、NOTIFY を `marshal_notify`（204 の全文）→ 応答の開始行から戻り値を読む。`notify` は NOTIFY だけを受け、GET は 400。
- 同じディレクトリの検出（R6.2）は、新しい集合を作らず `REGISTRY` で見分ける（2026-10-25 の要件のディスカッションで確定。R5.7 に `REGISTRY` と tracing の購読者を挙げた）。
- 版 3 の `load` の失敗（R2.4・2.7。確定）: NULL はディレクトリが無い・スレッドを起こせないときだけ。辞書・Lua・`SHIORI.load` の失敗はインスタンスを返し、旧来と同じ 500 を返す。失敗のたびにアクターを止めて片付ける道は、ほぼ要らなくなった（Research Needed の 4 番は、ディレクトリの確かめをスレッドを起こす前に置けば消える）。
- unload の join（R4.4・7.6。確定）: 後始末の完了が上限の中で届いたら、芯の型の Drop で `JoinHandle::join` してから返る。`DllMain` の FreeLibrary の detach から呼ばれる片付けでは join しない（loader lock の中でスレッドの終わりを待つとデッドロックする）。どちらの道から呼ばれたかを芯へどう渡すかを設計で決める。
- 旧来の入口の型（R7.3。確定）: 戻り値は `i32`（1／0）、長さは `c_long`（`load`・`loadu` の `len` と `request` の `*mut len`）。
- ログの量（R5.4。確定）: インスタンスごとに分ける。L3（共有）は採らない。L1 か L2 を設計で選ぶ。
- CI: `build.yml` に `linux`（`ubuntu-latest`。ビルド・`cargo test --workspace`・clippy）と `macos`（`macos-latest`。ビルド・テスト）のジョブを足し、Windows x64・Linux・macOS で `shiori4-testhost --entry v3 <ライブラリ> <hello-pasta の master の絶対パス>` を回す。`cargo install` の結果はキャッシュする。
- `Cargo.toml`: ルートに `shiori4_abi = "0.1"` を足し、`wintf-winmsg-executor` を消す。`shiori4_host = "0.1"` は dev 依存（Rust の試験で使うなら）。`Cargo.lock` の席を持つので、`pest-2-9-4-upgrade`・`ponytail-editor-tooling`・`ponytail-core-registry` と同じウェーブに置かない。

### Research Needed（設計で調べる）

1. Linux（と macOS）で今のワークスペースのテストを回したときに落ちるもの全部（WSL か CI の `workflow_dispatch` で一度回して数える）。特に `"C:\\…"` の文字列を持つ 11 ファイル、`LineEnding::native()` に頼る生成の比較、CRLF のフィクスチャ（`.gitattributes` が無い）、`pasta_check`・`pasta_sample_ghost` の `.exe`・`pasta.dll` を前提にする試験。
2. macOS の vendored LuaJIT のビルドの条件（`MACOSX_DEPLOYMENT_TARGET` など）と、arm64 のランナーでの試験の通り方。
3. ログの量をディレクトリごとに分ける作り（L1 の `Interest` の扱い、L2 の取りこぼし）。
4. 版 3 の `load` の失敗で、起こしたアクターを止めて片付ける道の待ち時間（失敗のたびに teardown の ack を待つ）。
5. ライブラリを外すこととの競り（OQ-8）が、Linux の `dlclose` で実際に起きる窓の大きさ。`shiori4_host` が unload の後にいつライブラリを閉じるか。
6. `shiori4-testhost` を Windows の x86（出荷する版）でも回せるか（`cargo install --target i686-pc-windows-msvc`）。
7. `shiori4_host` を dev 依存に入れたとき、`deny.toml` の検査（`libloading`・重複版）と、Linux の `cargo test` で cdylib のパスをどう見つけるか（`CARGO_TARGET_DIR`・`target/<profile>/libpasta.so`）。

### ほかの spec との重なり（確かめたもの）

- `ponytail-shiori`: `windows.rs`・`actor/{lifecycle,thread,teardown,mod}.rs`・`README.md`・`tests/actor_*` を共有。この spec が先。BOOL（4 番の一部）と 5 番（MAILBOX の試験の競り）は、この spec で自然に片付く見込み。1 番（unload と FreeLibrary の競り）は OQ-8 しだい。
- `shiori4-v4-host`: この spec の芯の型と `Shiori` の実装の上に版 4 を足す（`SUPPORTS_V3` のまま版 4 のテーブルも返す形へ。`export_shiori!` に替えられる）。
- `pasta-dll-multiarch`: `request` の長さの `c_long`（OQ-7）をどちらが直すか。x64 の配布の前に要る。
- `recommend-menu-scenes`（完了済み）とは重ならない。
