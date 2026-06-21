# Implementation Plan

> 使い捨て feature-gated PoC（`actor-poc`・default off）。出荷コードは読み取り・再利用のみ、振る舞い非改変。リスク順に R1（本丸）を最初の実証スライスへ。

- [ ] 1. Foundation: 隔離足場と共有基盤
- [x] 1.1 release バイト・ベースラインの取得（actor-poc 導入前）
  - `actor-poc` 関連コードを一切入れていない現状の release ビルド成果物（両クレート）のダイジェストを採取・保存する。
  - 観測可能な完了条件: 導入前 release 成果物のダイジェスト基準が記録され、後続のバイト不変検証（8.1）が参照できる。
  - _Requirements: 7.2_
- [x] 1.2 feature-gate と依存の新設（両クレート）
  - `pasta_lua`／`pasta_shiori` に `[features]` を新設、`actor-poc`（default off）、`wintf-winmsg-executor = { version = "0.0.3", optional = true }`、`lib.rs` に `#[cfg(feature = "actor-poc")]` の mod 宣言を追加。`pasta_shiori` は `pasta_lua/actor-poc` を伝播。
  - 観測可能な完了条件: feature off では `actor_poc` 不在のまま `cargo build` 成功、`--features actor-poc` で `actor_poc` モジュールがコンパイル対象になる。
  - _Requirements: 7.1, 7.2_
- [x] 1.3 テスト隔離土台の写経（env 中和・エフェメラルポート）
  - debug 前例から `#[ctor]` による `PASTA_DEBUG`／`PASTA_DEBUG_PORT` 中和、socket2 `set_reuse_address`＋port 0 のエフェメラル待受土台を写経。
  - 観測可能な完了条件: `actor-poc` テストが固定ポート枯渇・`PASTA_DEBUG` 汚染なしで反復実行できる。
  - _Requirements: 7.4_
- [x] 1.4 単一直列 mailbox
  - `GetMsg`／`NotifyMsg`／`KickMsg` 判別共用体、enqueue（SSP 側）／drain（アクター側）、FIFO 順序保証、スレッド分離。
  - 観測可能な完了条件: enqueue 順に drain され、VM 操作が drain 側スレッドに閉じることを示す単体テストが緑。
  - _Requirements: 2.3_
  - _Boundary: Mailbox_
- [x] 1.5 Verdict レコーダ土台
  - 各 probe が成否・採用方式・制約・ブロッカーを記録する累積器（`record_item`／`record_blocker`）と、隔離前提（default off・バイト不変・非汚染）の `assert_isolation`。
  - 観測可能な完了条件: 項目別の結果・ブロッカーが蓄積され取り出せる単体テストが緑。
  - _Requirements: 8.2, 8.3, 7.3_
  - _Boundary: Verdict_

- [ ] 2. R1（本丸）: executor 上 VM ホスト＋reload teardown
- [x] 2.1 アクタースレッドで `block_on` ＋ `!Send` VM pin
  - `std::thread::spawn` 内で `wintf-winmsg-executor::block_on(actor future)` を回し、future が `PastaLuaRuntime`（`!Send` VM）を生成・所有。VM はアクタースレッドを越えない。`JoinHandle` 保持・shutdown `AtomicBool` idiom 写経。
  - 観測可能な完了条件: VM がアクタースレッド内で Lua 実行を完了し、スレッド境界を越えないことを assert するテストが緑。
  - _Requirements: 1.1, 2.3_
  - _Boundary: ActorThread_
  - _Depends: 1.2, 1.4_
- [x] 2.2 reload teardown と反復リーク検査
  - shutdown→再 spawn の reload サイクルを N 回反復。メッセージ専用ウィンドウ・スレッド・チャネルの解放、ポート/ハンドル枯渇なしを確認。`DebugHandle::Drop` idiom（shutdown フラグ→join）を写経。
  - 観測可能な完了条件: N 回 reload 後もハンドル/ポートが枯渇せず clean teardown する統合テストが緑。
  - _Requirements: 1.2, 1.3_
  - _Depends: 2.1_
- [x] 2.3 R1 ブロッカー記録経路
  - VM ホスト/teardown 不成立（`!Send` 違反・リーク・reload 後クラッシュ等）の条件を切り分け `record_blocker` で残す。
  - 観測可能な完了条件: 失敗注入で `record_blocker` がブロッカー条件を記録し NO-GO 根拠化されることをテストで確認。
  - _Requirements: 1.4_
  - _Depends: 1.5, 2.1_

- [ ] 3. R2/R3: block-on-reply marshaling と drop→204 ガード
- [x] 3.1 Responder drop→204 ガード
  - GET 応答 oneshot（`std::sync::mpsc` 1 回受信）を包み、未 reply のまま drop（panic 巻き戻し含む）したら 204 を自動送信。「reply 1 回」または「drop→204」で必ず終結。
  - 観測可能な完了条件: 未 reply drop と panic 注入の双方で SSP 側が 204 を受け取り無限待機しない単体テストが緑。
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Boundary: Responder_
  - _Depends: 1.4_
- [x] 3.2 Marshal GET/NOTIFY 分岐＋pest 再パース
  - PoC ハーネスが request 文字列を pest で再パースして `method` を得（出荷 `shiori.rs` 非改変）、`get`→`GetMsg`＋block-on-reply／`notify`→即 204 fire-and-forget へ分岐。決定論ロジックは Rust 側で完結。
  - 観測可能な完了条件: GET が応答値（または drop→204）、NOTIFY が即 204 を返し、VM 操作が drain 側に閉じる統合テストが緑。
  - _Requirements: 2.1, 2.2, 2.4_
  - _Boundary: Marshal_
  - _Depends: 1.4, 2.1, 3.1_

- [ ] 4. R4: coroutine/callback 生存
- [x] 4.1 実 `*.lua` を executor 駆動で resume／callback 生存検証
  - `store.lua`／`event/init.lua`／`callback.lua`／`second_change.lua` を無改変で executor 駆動。`STORE.co_scene` を中断地点から resume、`CALLBACK.pending` を後続契機で解決、喪失条件を記録。シーン中核・コルーチン意味論は Lua のまま（Rust 化しない）。
  - 観測可能な完了条件: executor 駆動下で `co_scene` が中断地点から継続し `CALLBACK` が解決する統合テストが緑。
  - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - _Boundary: CoroutineProbe_
  - _Depends: 2.1_

- [ ] 5. R5: 忠実シミュレータとキック配信
- [x] 5.1 (P) SimDriver 忠実シミュレータ
  - OnSecondChange を `tick(playable)` で発火し、`playable=true→GET(Ref3=1)`／`false→NOTIFY(Ref3=0)` として**自身が method タグ付け**（Marshal の再パースには依存しない生成器）。`set_talking` で `Status: talking` 遷移を制御。
  - 観測可能な完了条件: `tick(playable)` が GET/NOTIFY tick を発火し `set_talking` が遷移する単体テストが緑。
  - _Requirements: 5.6_
  - _Boundary: SimDriver_
  - _Depends: 1.2_
- [x] 5.2 KickHarness: talk FIFO・二層 gate・即時 preempt
  - talk FIFO 投入→OnSecondChange drain で再生。二層 gate（①礼儀＝`talking` 中は非即時 drain を抑止／②配信可否＝GET tick のみ配信、NOTIFY/Ref3=0 では無視）。即時 preempt は礼儀 gate を無視し先行トークを `coroutine.close()` で閉じ GET tick で上書き、NOTIFY 状態では次 GET tick まで遅延。
  - 観測可能な完了条件: FIFO 投入→GET tick drain で再生、`talking` 中は非即時抑止、即時は GET tick で上書き配信される統合テストが緑。
  - _Requirements: 5.1, 5.2, 5.3_
  - _Boundary: KickHarness_
  - _Depends: 5.1, 1.4, 2.1_
- [x] 5.3 キック→配信 ≤1 秒レイテンシ実測
  - キック指示から配信までの所要時間を忠実シミュレータ上で実測し ≤1 秒を確認・記録。未達条件（drain 不発・gate 誤動作・preempt 不能・遅延 >1 秒）と実測値を `record_blocker`。
  - 観測可能な完了条件: キック→配信レイテンシを実測し ≤1 秒判定（または未達条件と実測値）を出力する統合テストが緑。
  - _Requirements: 5.4, 5.5_
  - _Boundary: KickHarness_
  - _Depends: 5.2_

- [ ] 6. R6: GET レイテンシ実測とフォールバック判断
- [ ] 6.1 GET block-on-reply レイテンシ実測＋フォールバック要否
  - 忠実シミュレータ（実機 attach 任意）の呼び出しパターンで GET を反復実行し代表値（最大・分布）を集計。GET タイムアウト→204 フォールバックの要否と閾値候補を判断・文書化、超過経路は `pasta-actor-runtime` へ申し送り。
  - 観測可能な完了条件: n 回 GET の代表値を集計し、フォールバック要否判断＋閾値候補を出力する統合テストが緑。
  - _Requirements: 6.1, 6.2, 6.3_
  - _Boundary: Latency_
  - _Depends: 3.2_

- [ ] 7. Integration: 段階判定オーケストレータ
- [ ] 7.1 Verdict 段階判定ロジックと run_all 結線
  - 各 probe 結果を集約し段階を確定（**NO-GO**（R1 不成立）／**条件付き GO**（R1+R2+R3）／**GO**（+R4）／**GO+**（+R5+R6））。全項目を成否にかかわらず試行する `run_all` を結線。
  - 観測可能な完了条件: R1〜R6 の成否組合せに対し正しい段階が決まる単体テストが緑。
  - 観測可能な完了条件: 最低ライン（R1+R2+R3）未達時に NO-GO 文書（ブロッカー＋回避候補）が出力される。
  - 観測可能な完了条件: 条件付き GO 以上で後続前提結論（採用 executor 統合方式・VM pin/teardown 方針・marshaling 契約・drop→204 ガード方針・coroutine 生存条件・GET レイテンシとフォールバック要否）が明記される。
  - _Requirements: 8.1, 8.4, 8.5, 7.3_
  - _Boundary: Verdict_
  - _Depends: 1.5, 2.2, 2.3, 3.2, 4.1, 5.3, 6.1_

- [ ] 8. Validation: バイト不変・統合走行・撤去
- [ ] 8.1 バイト不変検証（actor-poc 無効）
  - `actor-poc` 無効の release ビルド成果物が 1.1 のベースライン・ダイジェストと一致（diff ゼロ）することを確認。
  - 観測可能な完了条件: feature off ビルド成果物がベースラインとバイト一致することを確認する検証が緑。
  - _Requirements: 7.2_
  - _Depends: 1.1, 1.2_
- [ ] 8.2 統合走行と VerdictDocument 生成
  - 全 probe（R1〜R6）を結線して `run_all` を end-to-end 実行し、項目別試行結果・段階判定・後続申し送りを含む実 `VerdictDocument` 成果物を生成・出力する。
  - 観測可能な完了条件: end-to-end 走行が全項目試行を含む段階判定文書を成果物として出力する。
  - _Requirements: 7.3, 8.2_
  - _Depends: 7.1_
- [ ] 8.3 撤去手順の確認（使い捨て）
  - `actor-poc` feature・`actor_poc/` モジュール・`lib.rs` の cfg-mod 宣言・`Cargo.toml` の feature/依存を削除する撤去手順を確認し、痕跡なく本体バイト不変へ戻ることを検証。
  - 観測可能な完了条件: 撤去手順適用後に `actor-poc` 関連が完全除去され、release 成果物が 1.1 ベースラインへ戻る。
  - _Requirements: 7.5_
  - _Depends: 8.1_

## Implementation Notes

- 1.1: `pasta.dll`（cdylib）は同一ソースのクリーンビルド間で 20 バイト（PE TimeDateStamp/CheckSum/DebugDir timestamps×3/RSDS build-id GUID）が非決定。生 sha256 比較は actor-poc 無関係に偽 FAIL する。ベースラインは `.kiro/specs/pasta-actor-feasibility/baseline/`（`baseline.json`＋`capture_baseline.ps1`）に**正規化 sha256**（PE 非決定領域ゼロ埋め）で保存済み。rlib（`libpasta.rlib`/`libpasta_lua.rlib`）は exact 一致。**タスク 8.1 は必ず `capture_baseline.ps1 -Mode verify`（正規化比較）を使うこと**。release は `lto=true/codegen-units=1/strip=true/panic=abort`。クリーン release ビルドは実機で約 1〜1.5 分/回。
- env: cargo build/test/clean は毎回 PowerShell 同一呼び出し内で `Remove-Item Env:\NoDefaultCurrentDirectoryInExePath -ErrorAction SilentlyContinue;` を前置しないと LuaJIT/mlua-sys ビルドが exit 101 で死ぬ。
- release profile が `panic = "abort"` のため、Responder の drop→204 ガード（3.1/3.2、panic unwind 依存）は release では巻き戻らない。検証は `cargo test`（dev/test profile = unwind）でのみ成立。3.1/3.2 はこの前提でテストすること。
- 2.1（R1 本丸＝GO）: 実 `PastaLuaRuntime::new(TranspileContext::new())`（重フィクスチャ不要・軽量構築）を `std::thread::spawn` 内の `wintf_winmsg_executor::block_on(future)` の future ローカルとして所有すれば `!Send` mlua VM をアクタースレッドに pin できる（mlua の `!Send` が越境を構造的に禁止、値のみ mpsc で越境）。executor 0.0.3 実 API（registry source 実読）: `block_on<'a,T:'a>(future: impl Future<Output=T>+'a)->T`（呼び出しスレッドの message loop を回す）／`spawn_local<T:'static>(...)->JoinHandle<T>`／`JoinHandle`（Drop で detach・Future 実装）／`FilterResult{Forward,Drop}`／`MessageLoop`（直接構築不可）。再ポーリングは executor の `MSG_ID_WAKE`(WM_USER)＋`Waker` 機構を使う（`poll_fn` で waker 捕捉→producer が `wake_by_ref`、spin 回避）。teardown は `Arc<AtomicBool>`(SeqCst)→`wake`→`JoinHandle::join`、`Drop` は `take()` で二重 join 回避（debug `DebugHandle::Drop` idiom 写経）。`ActorThread` は `crates/pasta_lua/src/actor_poc/actor_thread.rs`。2.2/2.3/3.2/4.1 はこの土台に乗る。
- 既知の非ブロッキング lint: `tests/actor_poc_actor_thread.rs:20-21` に `clippy::doc_overindented_list_items`（doc コメント整形のみ）。CI は clippy 非実行・`-D warnings` 不使用のため無害。mailbox.rs:106 の `clippy::needless`（task 1.4 由来）も既存・境界外。
- 5.2（重要 R5 制約・実機検証済み）: **mlua LuaJIT 2.1（luajit52）には `coroutine.close` が存在しない**（`type(coroutine.close)=="nil"`・`_VERSION=="Lua 5.1"`・`LuaJIT 2.1` を実 VM で probe 確認）。suspended コルーチンは ref drop＋`collectgarbage` でも `dead` にできない。出荷 `store.lua` の `STORE.reset()` の `coroutine.close` 分岐も LuaJIT では no-op＝実質 ref-drop＋GC。よって設計の「先行トークを `coroutine.close()` で閉じる」は LuaJIT では literal には不可能。`KickHarness` は preempt の「閉じる」を**破棄（abandon＝`STORE.co_scene=nil` と live registry から除去・二度と resume しない）＋GC**として実機挙動に忠実にモデル化（要件 5.3／不変条件「上書き＝破棄」を満たす）。`KickHarness`＝`crates/pasta_lua/src/actor_poc/kick_harness.rs`。二層 gate＝配信可否（GET tick=Ref3=1 のみ配信／NOTIFY held）＋礼儀（talking 中は非即時抑止）。**→ この R5 制約（coroutine.close 不在・preempt=破棄+GC）は task 5.3/7.1 が `VerdictRecorder` に R5 制約として記録し、`pasta-actor-runtime` へ申し送ること。**
- 3.2: 設計どおり primitives を再利用する形に remediation 済み（初版は ActorThread/Mailbox を複製してリジェクト）。`ActorMsg::Get` は Responder 付きへ進化（`{payload, script, responder}`）、`Notify` は責任なし fire-and-forget の `script` 持ち。`ActorThread` に追加 API `mailbox_sender()`／`submit(ActorMsg)`＋actor loop の mailbox-drain 分岐（Get→VM 実行→`responder.reply`／VM 失敗→drop→204／Notify→VM 実行 reply なし／Kick→task5 scope で無視）。`Marshal`（`crates/pasta_shiori/src/actor_poc/ffi_marshal.rs`）は `ActorThread` を保持し `submit` で enqueue・`reply_rx.recv()` で block-on-reply。**executor 依存は pasta_lua に限定**（pasta_shiori の `actor-poc` は `["pasta_lua/actor-poc"]` のみ）。pest 再パースは既存 `crate::util::parsers::req::{Parser,Rule}` を read-only 再利用、出荷 `shiori.rs`/`lua_request.rs` diff-zero。R2.4 故障注入は Lua `error()` で実 VM 失敗→Responder Drop→204。7.1 の run_all 結線はこの ActorThread.submit/Mailbox/Responder 経路に乗せること。
- 2.2: ハンドルリーク計測 API（`GetProcessHandleCount`/`GetGuiResources`）のため `crates/pasta_lua/Cargo.toml` の `actor-poc` feature に `windows-sys/Win32_System_Threading` を追加（default/base には未追加＝feature off で非活性）。`ReloadProbe`＝`crates/pasta_lua/src/actor_poc/teardown.rs`。`PastaLuaRuntime::new(TranspileContext::new())` は in-memory・disk 自己展開なし（reload 反復で profile 汚染なし）。**8.1 はこの Cargo.toml 変更後の feature-off release を再ビルドして 1.1 正規化ベースラインと一致確認すること**（feature 加算はビルド単位ごとなので off ビルドに windows-sys 追加 feature は載らない想定）。
