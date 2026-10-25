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

## 7. 設計フェーズの調べ（2026-10-25・`/kiro-spec-design -y`）

- 種別: 既存の仕組みの拡張（Extension）。`rules/design-discovery-light.md` に沿って、つなぎ目・既存の型・新しい依存を確かめた。新しく足す依存は `shiori4_abi` だけで、ローカルの clone（`C:\home\maz\git\shiori4`、`v0.1.0-2-g04e4318`）で API を確かめたので、ウェブの調べは要らなかった。
- 使ったスキルの決まり: `kiro-spec-design` の `rules/design-principles.md`（境界を先に・依存の向き）・`design-discovery-light.md`・`design-synthesis.md`（一般化・作るか採るか・削る）・`design-review-gate.md`（要件の番号の機械的な照合・境界の欄・ファイルの計画）。レビューの関門は 1 回の直しで通った（試験の観測点の具体化・文書の部品の追加・9.3 の囲いの明記）。

### 7.1 確かめたこと（設計の根拠）

- **`shiori4_abi` 0.1.0 の試験の口**: `ShioriApi::load_v3(dir) -> Option<ShioriInstance>` と `ShioriInstance::{request, notify}`（Drop で unload）が `shiori4_abi` にある（`lib.rs` 857〜866 行・1026〜1062 行）。`V3_HOST`（`copy_from_slice` だけのテーブル）を渡すので、pasta の static のテーブル（`shiori_get_api(3)` の戻り値）を、同じプロセスの中で ABI のテーブル越しに試せる。→ `shiori4_host` を dev 依存に入れなくてよい（Research Needed 7 は消えた。`libloading`・`deny.toml`・cdylib のパスの見つけ方の心配も無くなる）。ライブラリの境界をまたぐ試験は CI の `shiori4-testhost` が持つ。
- **`ShioriApi::new` は `const fn`**: `static API_V3: ShioriApi = ShioriApi::new::<PastaV3, { SHIORI_ABI_VERSION_3 }>();` と書ける（`export_shiori!` と同じ作り）。
- **`shiori4_host` の `V3Runtime` の Drop**: unload の直後にライブラリを閉じる（`v3.rs` 117〜122 行）。unload が返った時点でアクタースレッドが残っていると、閉じたライブラリのコードを走ることになる。→ ack の後の join（R4.4）は要る（Research Needed 5 はこれで閉じる）。
- **`GetGuiResources` の 0**: wintf を外すと、試験のプロセスの USER オブジェクトが 0 になりうる。今の `sample_resources` は 0 を「測れない」とするので、`actor_reload_leak_test` の `leak_metric.expect(..)` が Windows で落ちる。`SetLastError(0)`→呼ぶ→0 なら `GetLastError()` で読み分ける直しが要る（設計の Teardown）。
- **ログの量の設定**: `FILTER_HANDLE`（`reload::Handle<EnvFilter>`）は `PastaLoader` の Stage 1.5（`loader/mod.rs` 274 行）から `update_tracing_filter` で替えられる。その時点で `LoadDirGuard(base_dir)`（同 111 行）が張られている。`REGISTRY` の振り分けの決まり（文脈あり→その項目、文脈なしでロガー 1 つ→それ）は、量の設定の選び方にもそのまま使える。既存の `logging_filter_reload_test.rs`（文脈なしの試験のスレッドで `tracing::enabled!` を見る）は、ロガーが 1 つなので期待を変えずに通る見込み。
- **同じディレクトリの見分け**: `REGISTRY` の鍵は `PathBuf`。`Path` の比較は成分ごとなので、末尾の区切りと Windows の `/`・`\` の違いは同じ鍵になる。`PastaLoader` は `base_dir` を正規化せずに鍵にしている（Stage 1 と Stage 1.5 で同じ鍵）。
- **占有の外れ方**: `PastaShiori::release_runtime` は `self.load_dir` が `Some` のときだけ `unregister` する。今はディレクトリが無いと `load_dir` を持つ前に返るので、占有が外れなくなる。→ `load_dir` を有無の確かめより前に持つ直しを入れる。
- **kick の送り先**: `pasta_lua` の `KickSink` は `None` が既定で、そのとき DAP の kick・`pasta/reloadShiori` は働かない（`debug/kick.rs`・`wiring/inbound.rs`）。インスタンスの送信端の `WeakSender`（flume 0.12 の `Sender::downgrade`）を閉じ込めれば、送り先が受け口を生かし続けない。
- **CI のパスの渡し方**: shiori4 の `textbook` ジョブは Windows でも `shell: bash` と `cygpath -w` でパスを作り、末尾に `\` を付けている。PowerShell の `"…\"` は、ネイティブのプログラムの引数で末尾の `\"` が引用符の逃がしに読まれるので避ける。

### 7.2 決めたこと

| 論点 | 決め | 理由 |
|---|---|---|
| 芯の型 | `actor::instance::ActorInstance`（送信端・スレッド・ディレクトリ・連番）。`Drop` で ack を待ち、届けば join | インスタンスごとの状態の持ち主を 1 つにする。版 3 の Drop＝unload にそのまま合う |
| 後始末の道の分け方 | `JoinPolicy::{JoinAfterAck, Detach}` を `shutdown` に渡す。`DllMain` の道だけ `Detach` | loader lock の中で待たない。呼ぶ側が道を知っている |
| 旧来の入口の置き場 | 案 C。`actor::lifecycle` に `static LEGACY: ArcSwapOption<ActorInstance>` と今の名前の窓口を残す（全 OS） | 12 ファイルの試験を期待を変えずに全 OS で走らせたまま芯へ移せる。送信の道にロックを置かない決まりを保つ |
| 版 3 の口 | `PastaV3`（`ActorInstance` 1 つ）と手書きの `shiori_get_api`（3 だけ） | `export_shiori!` は版 4 にもテーブルを返す |
| 版 3 の判定 | `classify_request` → `Get`／`Notify`／`Shiori2`（`Rule::header2`）／`Unreadable`。戻り値は開始行の 3 桁、読めなければ 500 | 今の文法を読み取りだけで使う。文法は変えない |
| 実行器 | E1（`while let Ok(msg) = rx.recv()`）。`spawn_actor_thread` の今の形は残し、`spawn_actor_thread_with` を足す | 待ちは 1 か所。既存の試験の呼び出しを変えない |
| ログの量 | L1。購読者は 1 つ、層のフィルタが `REGISTRY` の項目の `EnvFilter` で判定。`FILTER_HANDLE` を消す。`max_level_hint` は項目の最大、表を変えたら `rebuild_interest_cache` | 書き先の選び方と量の選び方を同じ決まりにでき、「共有」の逃げ道が要らない。L2 は FFI の入口のスレッドやデバッグの橋のスレッドのログの扱いが要る |
| 同じディレクトリの 2 つ目 | `GlobalLoggerRegistry::try_claim`（項目が無ければ足す）を、スレッドを起こす前に呼ぶ。外すのは VM の片付け（または起動の失敗） | 新しいグローバルな状態を足さない（R5.7・R6.2）。確かめと占有を 1 回のロックで行う |
| 旧来の FFI の型 | `loadu`・`load`・`unload`・`DllMain` を i32、長さを `c_long` | R7.3。`DllMain` は同じ欠陥なので一緒に（OPEN QUESTION） |
| Rust の試験 | `shiori4_abi::ShioriApi::load_v3` で in-process。境界をまたぐのは CI の `shiori4-testhost` | dev 依存を増やさない |
| CI | `build-unix`（ubuntu・macos。release のビルド・`cargo test --workspace`・Linux の clippy・testhost）と Windows x64 の testhost の段 | R10。Windows はすでにビルドしたリリースの DLL を使う |

### 7.3 合成（design-synthesis）

- **一般化**: 旧来の入口と版 3 の入口は、どちらも「インスタンスを 1 つ作り、全文の要求を送り、片付ける」の変種なので、芯を `ActorInstance` 1 つにした。違いは入口の層（HGLOBAL・ABI のテーブル・判定）にだけ置く。版 4 は同じ芯の上に足せる（`shiori4-v4-host`）。
- **作るか採るか**: ABI の検査と応答の写しは `shiori4_abi` を採る（自分で書かない）。試験のホストも `shiori4_abi` の `load_v3` と、CI の `shiori4-testhost` を採る。実行器は新しいクレートを採らず、flume の同期 `recv()` で足りる（E3 を採らない）。同じディレクトリの見分けは新しい集合を作らず `REGISTRY` を使う。
- **削ったもの**: `MAILBOX`・`ACTOR_HANDLE`・`REQUEST_SEQ`・`FILTER_HANDLE`・`wintf-winmsg-executor`。`shiori4_host` の dev 依存。版 3 の口のための別の応答の型（`ShioriResponse` に `String` を入れるだけ）。版 3 の 400 の本文は、今の `to_shiori_400_response` と同じ形の小さな関数 1 つ。

### 7.4 Research Needed の行き先

| 番号 | 行き先 |
|---|---|
| 1. Linux・macOS で落ちるテスト | 実装の最初の段で `build-unix` を足したブランチを `workflow_dispatch` で回して数える（設計の OPEN QUESTION 8）。直しが膨らめば `pasta-portable-build` に分ける |
| 2. macOS の vendored LuaJIT | 1 と同じ回しで確かめる（OPEN QUESTION 9） |
| 3. ログの量を分ける作り | L1 に決めた（7.2）。`pasta_lua/tests/logging_per_dir_filter_test.rs` と `multi_instance_test.rs` で確かめる |
| 4. load の失敗で片付ける待ち | 消えた。NULL はディレクトリの確かめ・占有の失敗（どちらもスレッドの前）と、スレッドを起こせない（待つものが無い）・ready の前に終わった（終わったスレッドを join するだけ）のときだけ |
| 5. `dlclose` との競り | `V3Runtime` は unload の直後にライブラリを閉じると確かめた。ack の後の join で閉じる |
| 6. x86 で testhost | 要件で対象外にした（R10.3） |
| 7. `shiori4_host` の dev 依存 | 消えた（`shiori4_abi::ShioriApi::load_v3` を使う） |

### 7.5 リスク

- Linux・macOS のテストの直しの量（High のまま。7.4 の 1）。
- ログの量の判定がイベントごとに `REGISTRY` のロックを取る。`max_level_hint` と `rebuild_interest_cache` で、誰も求めない量のコールサイトは先に弾く。`rebuild_interest_cache` は `REGISTRY` のロックを外してから呼ぶ（持ったまま呼ぶと止まる）。
- 占有は「項目が在ること」で判定するので、同じテストのプロセスで `PastaLoader` を直に使って登録したまま、同じディレクトリで `ActorInstance` を起こすと断られる（`PastaLoader` を直に使う道は `unregister` しない）。出荷の道は必ず `PastaShiori` を通るので当たらない。
- `PastaShiori::default()` の kick の送り先は `None` になる（今は `lifecycle::kick_sink()`）。`PastaShiori` を直に使う試験で DAP の kick を試すものは無いと見ている（実装で確かめる）。
- 旧来の入口で、規約に反して片付けと要求が同時に走ると join を保証しない（OPEN QUESTION 10）。

### 7.x 設計のディスカッションの決定の根拠（2026-10-25）

- **占有の世代の札**: 検証で、旧来の口の再読み込みが「手放したスレッドの占有」で断られる退行が見つかった（DAP で止めたまま SSP で再読み込みするなど）。札なしで力ずくで引き継ぐと、古い VM の後片付けの `unregister` が新しい持ち主の項目を消すので、世代の数を項目に持たせ、札が今の世代のときだけ消す。世代の数は表の中にあり、プロセスに 1 つの新しい状態は足さない。
- **`pasta_lua` のログの関数の破壊的な変更**: 形を残すと `init_tracing_with_reload(config)` の `config` を黙って無視し、`update_tracing_filter(config)` が文脈の無いスレッドで何もしない。黙って引数を無視する関数は誤用を招くので、形を正直に変える。公開クレートの pub API を変えてよいという 2026-10-19 の開発者の決定にも合う。
- **測りを設計の段で行う**: Linux・macOS で落ちるテストの数しだいで `pasta-portable-build` への分割があり、それがタスクを作った後で分かると作り直しになるため。
