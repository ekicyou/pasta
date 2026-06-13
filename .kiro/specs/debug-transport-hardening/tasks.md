# 実装計画: debug-transport-hardening

reload バグの根治（中断可能 accept ＋ 同期 teardown join）と防御層（`SO_REUSEADDR`）を、`Transport` → `DebugHandle` → テストの順で段階実装する。変更は `transport.rs` ＋ `mod.rs`（`DebugHandle::drop`）＋ Cargo ＋ テストに限定し、`wiring.rs` は無変更（既存の poll + by-value drop が同期 drop を駆動）。

- [ ] 1. Foundation: 依存追加
- [x] 1.1 socket2 依存をワークスペースと pasta_lua に追加
  - ルート `Cargo.toml` の `[workspace.dependencies]` に `socket2 = "0.5"` を追加し、`crates/pasta_lua/Cargo.toml` の `[dependencies]` に `socket2.workspace = true` を追加（cross-platform、`cfg` 分岐なし）
  - 観測: `cargo build -p pasta_lua` が `socket2` を解決して通り、`cargo-deny`（deny.toml）がライセンス（MIT OR Apache-2.0）・監査を通す
  - _Requirements: 4.5_

- [ ] 2. Core: Transport の bind/teardown ライフサイクル改修
- [x] 2.1 socket2 で SO_REUSEADDR を設定した待受ソケット生成へ切替
  - `Transport::start` の待受ソケット生成を `socket2` 経由（`Socket::new(IPV4,STREAM,TCP)` → `set_reuse_address(true)` → `bind` → `listen(1)` → `TcpListener::from` → `set_nonblocking(true)`）に切り替える。`backlog` は単一クライアント設計のため `1` で足りる
  - `listen == None` のゼロコスト無効パス（ソケット・ポート・スレッドを一切開かない）は不変。bind 失敗は従来どおり `DebugError::Bind` へマップ
  - 観測: enabled transport が `SO_REUSEADDR` 付きで bind して `local_addr` を返し、残存接続状態（TIME_WAIT 等）のみを理由としたバインド失敗を起こさない。無効時は一切のソケットを生成しない
  - _Requirements: 3.1, 3.2, 3.3, 4.5_
  - _Boundary: Transport_

- [x] 2.2 リスナースレッドの中断可能化（非ブロック accept + 内部 shutdown フラグ）
  - `Transport` に内部 `Arc<AtomicBool>` shutdown 信号を追加。`serve()` の `accept()` を非ブロック poll 化し、`WouldBlock` 時に `POLL_INTERVAL`（既存 5ms 規約）で shutdown を確認、立てば listener を drop して return。単一 accept 成功後は listener を即 drop（早期ポート解放）
  - 観測: クライアント未接続でも shutdown フラグにより `serve()` が有界時間で return する（`accept()` で永久ブロックしない）
  - _Requirements: 2.2, 2.3_
  - _Boundary: Transport_

- [x] 2.3 接続後の同期 teardown（writer ループ poll + reader join + socket 解放）
  - 接続確立後の writer ループを `out_rx.recv_timeout(POLL_INTERVAL)` + shutdown poll 化。teardown 時に `stream.shutdown(Both)` で reader を EOF させ、reader サブスレッドの `JoinHandle` を保持して join する
  - 観測: デバッガクライアント接続中に shutdown されても、接続中 socket を解放し reader を join して `serve()` が有界時間で return する
  - _Requirements: 2.5_
  - _Boundary: Transport_

- [ ] 2.4 Transport の同期 teardown 確定（Drop/shutdown でフラグ立て → serve handle join）
  - `Transport::drop` と `shutdown()` を「内部 shutdown フラグを立てる → serve handle を同期 join」に変更する。join 完了後はポートが解放され、同一構成での再 start が可能
  - 観測: `Transport::shutdown()` + watchdog 付き bounded join が有界時間で完了し、リスナースレッドがデタッチ生存しない。teardown 後の同一ポート再 start が成功する
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Boundary: Transport_

- [ ] 3. Integration: DebugHandle teardown 同期化（unload 連鎖）
- [ ] 3.1 DebugHandle::drop を detach→join 化
  - `DebugHandle::drop` の `socket_handle` を detach から join へ変更（`encoder_handle` は socket/port を持たないため detach 維持）。Terminated 送出 + 30ms flush sleep は既存挙動として保存（R4.2）。これで runtime drop（unload）→ `DebugHandle::drop` → bridge join → `Transport` drop → serve join → ポート解放まで同期する
  - 観測（reload 成立）: enable した runtime を drop すると unload 完了までに待受ポートが解放され、同一プロセス・同一構成で再 enable→bind が成功する。drop がハングしない
  - 観測（wiring 無変更）: `wiring::run_socket_bridge` は変更せず、その shutdown 観測 → 関数 return（`Transport` を by-value drop）が `Transport` の同期 drop（serve join）を駆動し続けることを、上記 reload 成功で確認する
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2_
  - _Depends: 2.4_
  - _Boundary: DebugHandle_

- [ ] 4. Validation: テストと無回帰
- [ ] 4.1 (P) Transport 単体テスト（中断 join・SO_REUSEADDR・接続中 teardown・リーク検出）
  - クライアント未接続で `start` → `shutdown()` + bounded join が有界完了し、リスナースレッドが終了する（watchdog 付き join 完了）。これが 1.4 のリーク検出器：中断可能 accept + 同期 join が無ければ join が完了せず/タイムアウトする形で居残りを検出する
  - enabled transport が `SO_REUSEADDR` 付き bind と双方向フレーミング round-trip を維持。接続中クライアントを drop すると reader を join して return。`listen == None` のゼロコスト無効パスを維持
  - 観測: 上記が緑で、スレッド終了（join 完了）を一次シグナルとして観測できる
  - _Requirements: 1.4, 2.2, 2.3, 2.5, 3.1, 3.3_
  - _Depends: 2.4_
  - _Boundary: Transport tests_

- [ ] 4.2 (P) 同一プロセス reload 結合テスト（runtime/enable レベルの再 bind 成功）
  - enable した runtime を同一プロセスで drop → 再 enable し、同一構成・固定ポートで再 bind が成功する（no-client 経路＝真因）。接続中クライアントを伴う reload も成功する（`SO_REUSEADDR` の TIME_WAIT 防御を確認）。連続 2 回以上の reload が各回成功する
  - ポートは `:0` で OS 割当を capture し同一ポートで teardown→rebind を実施。`#[ctor]` による `PASTA_DEBUG*` env 中和・watchdog bounded-join を流用。スレッド終了の直接検証は 4.1 が所有し、本タスクは結合レベルの rebind 成功を担う
  - 観測: runtime レベルの同一ポート reload が各回緑
  - _Requirements: 1.1, 1.3, 2.4, 2.5, 3.2, 4.3_
  - _Depends: 3.1_
  - _Boundary: Integration tests_

- [ ] 4.3 全テスト無回帰・クロスコンパイル非破壊確認
  - `cargo test --workspace` 緑（LuaJIT ビルドは環境変数 `NoDefaultCurrentDirectoryInExePath` を外して実行）。既存デバッグ挙動（BP/ステップ/変数 inspect/コルーチン/提示モード/ソースマップ/サイドカー）の無回帰を確認
  - `socket2` は cross-platform で新たな `cfg(windows)` 分岐を増やさず、クロスコンパイルを壊さない（実行検証は対象環境 Windows で 10048 解消を確認）
  - 観測: 全テスト緑、既存 transport/wiring テストを維持し、変化が teardown 同期化 ＋ `SO_REUSEADDR` に限定されている
  - _Requirements: 4.1, 4.2, 4.4, 4.5_
  - _Depends: 4.1, 4.2_

## Implementation Notes
- 2.3: Windows では `TcpStream::shutdown(Both)` が peer 生存中の in-flight blocking `recv` をキャンセルしない（実証済み・design Risks 節が予見）。reader の teardown は `shutdown(Both)` 依存ではなく、**read timeout（POLL_INTERVAL）＋フレーム境界での shutdown フラグ poll** による協調的中断で有界化する（フレーム parse 中は timeout を解除し framing を割らない）。`shutdown(Both)` は best-effort 併用。2.4（Drop join）・4.x テストでもこの前提を踏襲する（production に hard deadline を焼き込まない）。
- 2.3: writer ループの flag-break パスは break 前に `try_recv` で滞留 outbound を drain し、「teardown 時の pending flush」契約（wiring e2e）を保持する。
