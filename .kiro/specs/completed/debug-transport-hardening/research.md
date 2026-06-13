# Gap Analysis: debug-transport-hardening

要件（WHAT）と既存コードベースの差分を分析し、設計フェーズ（HOW）の判断材料を提供する。本仕様は brief で真因をソース確定済みのため、本分析は「真因に対する修正方式の選択肢」と「#2〜#4 各堅牢化レイヤの実装着地点」を中心に整理する。

## 1. 既存コードベース調査（Current State）

### 関連アセットとアーキテクチャ

| 領域 | 場所 | 役割 |
|---|---|---|
| Transport（wire 層） | `crates/pasta_lua/src/debug/transport.rs` | `TcpListener` bind → `serve()` スレッドで accept → socket↔mpsc ブリッジ。`start`/`Drop`/`serve` が本仕様の主戦場。 |
| enable ゲート | `crates/pasta_lua/src/debug/mod.rs:564` `enable()` | `cfg.enabled` の門番。無効時 `Ok(None)`（ゼロコスト）。有効時 hook + Transport + 2 ブリッジスレッドを起こし `DebugHandle` を返す。 |
| DebugHandle teardown | `debug/mod.rs:482` `Drop` | `Terminated` 送出 → 30ms sleep → shutdown フラグ立て → ブリッジスレッドを **detach（join しない）**。 |
| socket bridge | `debug/wiring.rs:231` `run_socket_bridge` | `Transport` の唯一の所有者。`POLL_INTERVAL`（5ms）で shutdown フラグを poll、立てば return → `Transport` drop。 |
| config 解決 | `debug/mod.rs:218` `DebugConfig::resolve` / `debug/mod.rs:304` `from_env` | `enabled`/`port`/`source_mode`/`sidecar` の優先順位決定（env > file > 既定）。純粋関数で単体テスト済み。 |
| 既定ポート | `loader/config.rs:620` `default_debug_port()` = `9276` | `DebugFileConfig`（`loader/config.rs:585`）と env `PASTA_DEBUG_PORT` で上書き可。 |
| ランタイム保持 | `runtime/mod.rs:70` `debug_handle: Option<DebugHandle>` | `runtime/mod.rs:212` で `enable()` を呼び生成・保持。ランタイム drop で teardown 連鎖が起きる。 |
| SHIORI load/unload | `crates/pasta_shiori/src/shiori.rs:30` `runtime: Option<PastaLuaRuntime>` | `load()` 内でランタイム再生成、drop で unload。同一 `hinst`・同一プロセス常駐。 |
| VSCode attach 解決 | `editors/vscode/src/debugAttachTarget.ts:13` `DEFAULT_DEBUG_PORT = 9276` | `resolveAttachTarget` が host/port を解決。明示 port → それを優先、無効/不在 → 既定 9276 フォールバック（純粋・node テスト可）。 |

### 真因の teardown 連鎖（ソース確定済み）

```
ランタイム drop
  → DebugHandle::drop（mod.rs:482）: shutdown フラグを立て、ブリッジスレッドを detach
    → socket bridge が POLL_INTERVAL 内に return → Transport を drop
      → Transport::drop（transport.rs:307-314）: self.outbound = None だけ
         ✗ accept() でブロック中の serve() スレッドは起きない
         ✗ handle（JoinHandle）は join されず Option ごと drop = スレッドはデタッチ生存
           → serve() が TcpListener を握ったままプロセス寿命いっぱい居残る
             → 次回 reload の Transport::start で同一固定ポート bind → WSAEADDRINUSE (10048)
```

クライアント未接続の通常運用（VSCode が attach しない）では `serve()` が `listener.accept()`（`transport.rs:330`）で永久ブロックする点が核心。`shutdown()`/`Drop` の「outbound を落とす」信号は writer 側にしか効かず、accept で眠る listener を起こさない。

### 規約・既存パターン（流用候補）

- **POLL ループ規約**: socket bridge が `recv_timeout(POLL_INTERVAL)` で shutdown フラグを協調 poll する確立パターンあり（`wiring.rs:249-275`）。#1 の「accept 中断可能化」を同じ poll 流儀で実装すれば整合する。
- **TEST-ONLY watchdog/bounded join**: `transport.rs:829` `join_transport_with_watchdog`、`set_read_timeout(WATCHDOG)`。production はタイムアウトを焼き込まない設計方針（スレッドモデル ④）。回帰テストはこの既存ヘルパ流儀に乗せられる。
- **テスト隔離**: メモリ memo の通り `PASTA_DEBUG=1` 固定ポート枯渇問題は `#[ctor]` で中和済み（`ctor = "0.2"` が dev-dep にあり）。load を叩く新テストは env 非依存＝ポート 0 で書く既存規約に従う必要がある。
- **純粋 resolve のテスタビリティ**: `DebugConfig::resolve` は 8 引数の純粋関数。#3（リリース無効化）の優先順位は、ここに入力を 1 本足して単体テストで pin する流儀が自然。

### 依存関係の現状

- ワークスペース内 Cargo.toml に **`socket2` / `libc` / `nix` は未導入**（grep 0 件）。
- `windows-sys` は `[target.'cfg(windows)'.dependencies]` に **導入済み**。
- `set_nonblocking` は std（`TcpListener`/`TcpStream`）で利用可、追加依存不要。`SO_REUSEADDR` と port 0 の実バインド取得（`local_addr()`）も std で可能だが、bind 前の `SO_REUSEADDR` 設定だけは std `TcpListener::bind` 直叩きでは不可（生ソケットへの setsockopt が要る）。

## 2. 要件 → アセット対応表（gap タグ: 充足 / 欠落 / 制約）

| 要件 | 着地点 | gap |
|---|---|---|
| **R1** 再バインド成立（根治） | `transport.rs` `Drop`/`serve`/`start` + `wiring.rs` teardown | **欠落**: accept 中断 → listener 停止 → ポート解放の経路が無い。回帰テスト（同一プロセス unload→reload・10048 再現/解消）も新規。 |
| **R2** teardown 確実解放 | `transport.rs` `Drop`（join 追加）/ `serve`（中断応答）, `DebugHandle::drop` 連携 | **欠落**: 現状 detach。**制約**: production に無限ブロック join を入れると teardown をハングさせうる（中断可能化が前提）。有界時間完了が必須。 |
| **R3** `SO_REUSEADDR` 相当 | `Transport::start` の bind 生成 | **欠落**: 現状 `TcpListener::bind` 直叩きで option 未設定。**制約**: bind 前 setsockopt には生ソケット制御が要る（socket2 か OS 別 raw API）。 |
| **R4** リリース無効化/オプトイン | `enable()` ゲート + `DebugConfig::resolve` 優先順位 + ビルド種別判定 | **欠落**: `debug_assertions`/feature 等のビルド種別ゲートは**現状皆無**（`enabled` フラグのみ）。優先順位（ビルド種別 × 設定 × env × オプトイン手段）の一意決定ルールが未定義。 |
| **R5** エフェメラル + アドバタイズ | `Transport::start`（port 0 bind は可・`local_addr` 取得済み）+ **アドバタイズ経路（新規）** + VSCode 解決調整 | **欠落/制約**: 実バインド port の取得は既に可能。ただし「VSCode へ伝える経路」が無い。固定 9276 契約との両立が中心トレードオフ。アドバタイズ手段の実体は下流 `debug-startup-logging` 領域と接続（本仕様の Out 境界に注意）。 |
| **R6** 無回帰と検証 | 既存 transport/wiring テスト群 + 新規回帰テスト | **充足寄り**: 豊富な既存テスト（frame codec / enable ゲート / bridge）あり。**欠落**: unload→reload 再バインド・10048 再現の統合テスト、クロスプラットフォーム非破壊の担保。 |

## 3. 実装アプローチ選択肢

### #1 根治（accept 中断方式）— 設計の本丸

- **Option A: `set_nonblocking(true)` + shutdown フラグ poll ループ**（推奨候補）
  - `serve()` の `accept()` を非ブロック化し、`WouldBlock` 時に shutdown フラグを `POLL_INTERVAL` 流儀で poll。フラグ検知で listener を drop して return。`Transport::drop` で handle を join。
  - ✅ 既存 socket bridge の poll 規約と完全整合・std のみ・クロスプラットフォーム素直。✅ production に無限 join を入れても中断可能なので安全。
  - ❌ 接続確立後の read もタイムアウト/poll 設計に巻き込むか要整理（現状 reader は別スレッドでブロック read）。idle ポーリングの僅かな wakeup コスト。
- **Option B: 自己接続で `accept()` を起こす**
  - teardown 時にループバックへ自己接続し accept を 1 回返させてから停止判定。
  - ✅ 既存のブロッキング accept 構造をほぼ温存。❌ ポート/ファイアウォール事情・競合接続（実クライアントとのレース）でのエッジが増える。fixed-port では自己接続先が自明だが ephemeral と組むと順序依存。
- **Option C: ハイブリッド（nonblocking accept ＋ 接続後は現行ブロック bridge 温存）**
  - accept だけ poll 化し、確立後の reader/writer は現行のまま。teardown は「未接続なら poll が即停止」「接続中なら socket shutdown で EOF」。
  - ✅ 変更を accept 段に局所化し正常系 bridge 挙動を厳密保存（R6 に最適）。❌ 2 状態（未接続 poll / 接続中ブロック）の停止経路を両方検証する必要。

### #2 `SO_REUSEADDR`

- **Option A: `socket2` クレート導入**（推奨候補）: `Socket::new` → `set_reuse_address(true)` → `bind` → `listen` → `TcpListener::from`。クロスプラットフォーム・port 0 / nonblocking とも素直に共存。❌ 新規依存 1 本。
- **Option B: OS 別 raw setsockopt**（`windows-sys` 既存 + 他 OS は `libc`）: 依存追加を最小化できるが `#[cfg]` 分岐コードが増え保守コスト高・テスト面倒。

### #3 リリース無効化／オプトイン

- **Option A: `cfg!(debug_assertions)` ゲート**: release プロファイルで既定無効、明示オプトイン（env/設定）でのみ有効。配布 = release ビルドという前提と素直に一致。
- **Option B: Cargo feature（例 `debug-transport`）**: ビルド時 opt-in。配布バイナリは feature off。より明示的だが、emo2 と同一バイナリ配布の運用（pasta.dll 複製）と feature 切替の整合を設計で確認要。
- **共通課題（要件 4.4）**: ビルド種別 × `[debug] enabled` × `PASTA_DEBUG` × オプトイン手段の**優先順位を一意化**し `DebugConfig::resolve` に入力として畳み込む。既存の env>file>既定 規約を壊さない拡張が望ましい。

### #4 エフェメラルポート + アドバタイズ

- **Option A: 固定ポート契約維持（#1+#2 で再バインドを解く）**: クライアントは 9276 のまま、アドバタイズ経路は不要。最小変更・契約破壊なし。**ただし多重起動衝突は解かない**。
- **Option B: エフェメラル化 + アドバタイズ**: port 0 bind → 実 port を取得（既に `local_addr` で可）→ ログ/ファイル/DAP で VSCode へ伝達 → `resolveAttachTarget` が実 port を解決、不在時 9276 フォールバック（要件 5.5 と既存フォールバック挙動が整合）。アドバタイズ実体は下流 `debug-startup-logging` 領域に踏み込むため、本仕様では「経路の接続点」をどこまで持つかを境界として明確化する必要。
- **中心トレードオフ**: A（契約維持・最小）か B（多重起動耐性・要アドバタイズ基盤）か。#1 根治が入れば A だけでも reload 衝突（10048）は解消する。B は「固定ポート衝突・多重起動」というより一般的な問題への投資。

## 4. 実装規模 / リスク

| 項目 | Effort | Risk | 根拠 |
|---|---|---|---|
| #1 根治（accept 中断 + join） | **M** | **Medium** | スレッド停止の正確な順序づけと正常系 bridge 挙動の厳密保存。既存 poll 規約に乗れば抑制可。10048 再現テストの安定化が肝。 |
| #2 `SO_REUSEADDR` | **S** | **Low** | socket2 採用なら局所変更。OS 別 raw なら cfg 分岐で Medium 寄り。 |
| #3 リリース無効化 | **S–M** | **Medium** | コード変更は小だが、優先順位の一意化と「無効時ゼロコスト/無言/サンドボックス」保証（R5 系既存契約）を壊さない検証が要。 |
| #4 エフェメラル + アドバタイズ | **M–L** | **High** | クライアント契約・下流仕様（`debug-startup-logging`）・VSCode 拡張に波及。境界設定次第で L。 |
| R6 回帰/無回帰テスト | **M** | **Medium** | unload→reload 統合テストの確実な 10048 再現、クロスプラットフォーム非破壊、CI 8.3 短縮名パス等の既知の罠（memo 参照）に注意。 |

## 5. Research Needed（設計フェーズへ持ち越す論点）

1. **依存方針**: `socket2` 導入を許容するか（#2/#4 が大幅に単純化）、`windows-sys`＋`libc` の OS 別 raw に留めるか。ライセンス/ビルド（LuaJIT・`NoDefaultCurrentDirectoryInExePath` 環境変数の罠）への影響確認。
2. **#1 方式の確定**: nonblocking poll（Option A/C）か自己接続（B）か。接続確立後の reader スレッド停止（現状ブロック read）を teardown でどう確実に巻き取るか（socket shutdown による EOF 誘発で足りるか）。
3. **#4 のスコープ確定**: 固定契約維持（A）に倒すか、エフェメラル+アドバタイズ（B）まで踏み込むか。B の場合、アドバタイズ実体（ログ/ファイル/DAP のいずれか）と下流 `debug-startup-logging` との責務境界。本仕様 Out 境界（起動ログ基盤の新規追加は対象外）との整合。
4. **#3 のゲート手段**: `cfg!(debug_assertions)` か Cargo feature か。emo2 同一バイナリ配布運用での切替整合と、要件 4.4 の優先順位一意化ルールの正準形。
5. **回帰テストの 10048 再現設計**: 同一プロセス内 unload→reload を固定ポートで再現しつつ、CI で flaky にならない隔離（既存 `#[ctor]` env 中和・ポート 0 規約との両立、固定ポートが必須な再現テストの安全な書き方）。
6. **クロスプラットフォーム差異**: `SO_REUSEADDR` の Windows と Unix での意味差（Unix の `SO_REUSEADDR` ≒ Windows の挙動差、`SO_REUSEPORT` は対象外か）。他 OS でビルド/テストを壊さない条件。

## 6. 設計フェーズへの推奨

- **推奨アプローチ**: #1 は **Option C（accept のみ poll 中断・接続後 bridge は厳密保存）** を軸に、#2 は **socket2 導入（Option A）**、#3 は **`cfg!(debug_assertions)` 既定無効＋明示オプトイン（Option A）**、#4 は **まず固定契約維持（Option A）を既定線**とし、エフェメラル化（B）は下流 `debug-startup-logging` のアドバタイズ基盤と接続できる範囲に限定する——という「根治優先・契約温存・段階導入」構成を設計の出発点に推奨する。
- **キー判断**: (a) socket2 を入れるか、(b) #1 の accept 中断方式、(c) #4 を固定契約維持に留めるかエフェメラルへ移行するか、(d) #3 のビルド種別ゲート手段と優先順位の正準形。
- **持ち越し研究項目**: 上記 §5 の 1〜6。とりわけ #4 の境界（本仕様 Out と下流仕様の責務分担）と 10048 再現テストの flaky 回避が設計品質の分かれ目。

---

## スコープ更新（要件ディスカッション 2026-06-13）

要件ディスカッションの結論により、本仕様は **reload バグの根治（#1 同期 teardown ＋ #2 `SO_REUSEADDR`）に純化**された。以下を**設計フェーズの対象外**とする：

- **#3 リリースビルドでの無効化／オプトイン化（旧 R4）— 除外**。pasta.dll は単一 release ビルドを配布し、デバッグは同一 DLL の opt-in（既定 off）機能とする設計思想のため、ビルド種別ゲート（`cfg!(debug_assertions)` / Cargo feature）はこの思想に反する。「既定 off ＋ opt-in」ゲートと **loopback（`127.0.0.1`）固定・host 上書き不可**（`debug/mod.rs:67`/`:269-270`、外部非公開）により、配布物のポート露出リスクは既存実装で十分に低減済み。→ §5.4（#3 ゲート手段）は不要。
- **#4 エフェメラルポート＋アドバタイズ（旧 R5）— 除外**。#1+#2 の根治で固定ポート契約のまま 10048 が解消されるため、エフェメラル化は「根治済みなら不要な対応」。固定ポート衝突・多重起動の一般対策は別問題（High リスク・クライアント契約／下流 `debug-startup-logging` への波及）として本仕様から外す。→ §5.3（#4 スコープ）は「固定契約維持（Option A）に確定」、アドバタイズ経路・VSCode 追従は本仕様で扱わない。

### 設計フェーズで残る判断（縮小後）
1. **#1 accept 中断方式の確定**（§5.2）: `set_nonblocking(true)` + shutdown フラグ poll ループ（既存 `wiring::POLL_INTERVAL` 規約に整合）か、自己接続で accept を起こすか。接続確立後の reader スレッド停止（socket shutdown による EOF 誘発）と、**unload 時の同期 join の有界性**の担保。
2. **#2 `SO_REUSEADDR` の実装手段**（§5.1/§5.6）: `socket2` 導入か `windows-sys`＋`libc` の OS 別 raw setsockopt か。Windows/Unix の `SO_REUSEADDR` 意味差と、他プラットフォーム非破壊。
3. **回帰テストの 10048 再現設計**（§5.5）: 真因は live-thread リークで TIME_WAIT 非依存＝即時再現可（実時間 15秒待機は不要）。固定ポート再現テストの flaky 回避（既存 `#[ctor]` env 中和・ポート 0 規約との両立）。

---

## 設計フェーズの決定（design synthesis 2026-06-13）

縮小後スコープ（R1+R2+R3+R4）に対し、設計フェーズで以下を確定（詳細は `design.md`）。

1. **#1 accept 中断方式 → 非ブロック poll（Option A/C）に確定**。`serve()` で listener を `set_nonblocking(true)` し `POLL_INTERVAL`（5ms・既存規約）で内部 `Arc<AtomicBool>` shutdown を poll。単一 accept 成功後は listener を即 drop（早期ポート解放）。接続後の writer ループも `recv_timeout(POLL_INTERVAL)` + フラグ poll、reader は `stream.shutdown(Both)` の EOF で停止させ join。`Transport::drop`/`shutdown()` が「フラグ立て → serve handle 同期 join」を完結。自己接続（Option B）は競合接続レースのため不採用。
2. **#2 `SO_REUSEADDR` 実装 → `socket2 = "0.5"` 採用**。`Socket::new(IPV4,STREAM,TCP)` → `set_reuse_address(true)` → `bind` → `listen` → `TcpListener::from` → `set_nonblocking(true)`。OS 別 raw setsockopt（`windows-sys`+`libc`）案は cfg 分岐の保守コストで不採用。MIT/Apache-2.0・`cargo-deny` 整合要確認。workspace deps へ `socket2 = "0.5"` 追加、`pasta_lua` で `socket2.workspace = true`。
3. **teardown 同期化の境界 → socket-bridge スレッドの join**。`DebugHandle::drop` を `socket_handle` detach → join へ変更（`encoder_handle` は socket/port 非保持のため detach 維持、`terminate_tx` 保持中 join のデッドロック回避）。bridge join ⊇ `Transport` drop ⊇ serve 同期 join ⊇ ポート解放。全待ちが `POLL_INTERVAL` 境界 + 中断可能 accept で有界。
4. **Web 検証で得た重要知見（Windows `SO_REUSEADDR` の masking）**: Windows では `SO_REUSEADDR` が「listen 中ソケットにも bind 許可（hijack）」の意味を持つため、**#1 を欠いたまま #2 だけ入れると漏れた listener が生きていても rebind が成功し真因をマスクする**。よって #1（同期 join）が一次治療、#2 は接続中 reload の TIME_WAIT 防御層に限定。回帰テスト（1.4）は「rebind 成功」ではなく**リスナースレッド終了（join 完了）**を一次シグナルとする。hijack リスクは loopback 限定・dev opt-in・既定 off により許容。
   - Sources: [socket2 Socket docs](https://docs.rs/socket2/latest/socket2/struct.Socket.html) / [Windows SO_REUSEADDR](https://learn.microsoft.com/en-us/windows-hardware/drivers/network/so-reuseaddr) / [Windows SO_EXCLUSIVEADDRUSE](https://learn.microsoft.com/en-us/windows-hardware/drivers/network/so-exclusiveaddruse)
5. **`wiring.rs` は変更不要**: 既存の shutdown poll + `Transport` by-value drop がそのまま同期 join を引き起こす。回帰テストはポートを `:0` で取得・capture して同一ポート teardown→rebind を回し（固定 9276 直書きの CI flakiness 回避）、`#[ctor]` env 中和・watchdog bounded-join を流用。
