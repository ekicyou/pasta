# Brief: debug-transport-hardening

## Problem
開発（DEBUG）ビルドのゴーストを SSP 上で一度別ゴースト（えみり等）へ切り替えてから再選択すると、ゴーストが表示されなくなる。SHIORI `load()` がランタイム初期化（Phase 6）で **debug transport（DAP 待受 TCP ソケット）の再バインドに失敗**して落ちるため。OnBoot 不発は症状であって原因ではなく、真因はその手前の `load()` 破綻にある。SSP を完全再起動するまで当該ゴーストが起動しなくなり、**開発ビルドでのゴースト切替運用が事実上不可能**。

決定的エラー（pasta.log:148）:
```
ERROR pasta::shiori: PastaShiori load failed
  error=Failed to initialize Lua runtime: debug transport bind failed: ... (os error 10048)
```
`os error 10048` = Windows `WSAEADDRINUSE`（アドレス/ポート使用中）。`hinst` が1回目・2回目で同一 ＝ pasta.dll は同一 SSP プロセスに常駐し続けており、Lua ランタイムのみが破棄・再生成される。

## Current State
**根本原因をソースで確定済み（2026-06-13）。** TIME_WAIT ではなく「生きたリスナースレッドがポートを保持し続ける」リソースリーク。

1. `Transport::start` で `TcpListener` を **別スレッド `serve()` へ move** して所有させる（`crates/pasta_lua/src/debug/transport.rs:247`）。
2. その `serve()` スレッドはクライアント未接続の間ずっと `listener.accept()` で**ブロック**する（`transport.rs:330`）。VSCode デバッガが attach しない通常運用では永久に返らない。
3. teardown 経路は `DebugHandle::drop`（`debug/mod.rs:482`）→ 共有 shutdown フラグ → socket bridge が `Transport` を drop。
4. しかし `Transport::drop`（`transport.rs:307-314`）がするのは `self.outbound = None`（outbound センダーを落とす）**だけ**。`accept()` で眠るリスナースレッドはこの信号で起きず、`handle`（`JoinHandle`）は join されず **`Option` ごと drop ＝ スレッドはデタッチされて生き残る**。
5. 結果、`serve()` スレッドが固定ポートにバインドした `TcpListener` を **SSP プロセスの寿命いっぱい握り続ける**。

- 待受ポートは**固定**。既定 `9276`（`crates/pasta_lua/src/loader/config.rs:620` `default_debug_port`）。`PASTA_DEBUG_PORT` / pasta.toml `[debug] port` で上書き可。
- 待受ソケットに **`SO_REUSEADDR` 未設定**（`TcpListener::bind` 直叩き・`transport.rs:238`）。
- デバッグ有効化は opt-in（`PASTA_DEBUG` / pasta.toml `[debug] enabled`、既定 false）。**リリース/デバッグのビルド種別による無効化はない**（`enabled` フラグのみが門番）。無効時は `enable()` が即 `Ok(None)` を返しポートを開かないゼロコストパス（`debug/mod.rs:569`）。
- **VSCode クライアント側は固定ポート `9276` へ attach する契約**（`editors/vscode/src/debugAttachTarget.ts:13` `DEFAULT_DEBUG_PORT = 9276`「MUST match the Rust default」、`editors/vscode/package.json:225/249/261`）。

## Desired Outcome
- DEBUG ビルドでも、ゴーストを切り替えて再選択したとき `load()` が成功し、OnBoot が発動してゴーストが表示される（同一 SSP プロセス内での unload→reload で debug transport が確実に再バインドできる）。
- unload / ランタイム破棄時に debug transport の待受ソケットが**確実に解放**される（リスナースレッドが終了し、ポートがプロセス内に居残らない）。
- 配布（リリース）ゴーストが不要な待受ポートを開かない（セキュリティ・無駄ポート回避）。
- デバッグ無効時は従来どおり完全に無言・ゼロコスト・サンドボックス維持を堅持する。

## Approach
ユーザー決定（2026-06-13）により**推奨#1〜#4を全部入り**で扱う。根本原因（#1）が根治の本丸で、#2〜#4はそれぞれ独立した堅牢化レイヤ。

1. **#1 teardown でリスナースレッドを確実に停止・ポート解放（根治）**: `accept()` のブロックを中断可能にする。候補＝`TcpListener::set_nonblocking(true)` ＋ shutdown フラグの poll ループ化（既存 `wiring::POLL_INTERVAL` パターンに整合）、もしくは自己接続で `accept()` を起こす。teardown 時にリスナースレッドを join して `TcpListener` を drop し、ポートを解放する。設計フェーズで方式を確定。
2. **#2 `SO_REUSEADDR` を待受ソケットに設定**: TIME_WAIT 残存にも耐える防御。`socket2` 等での生成に切替（依存追加の要否は設計で検討）。
3. **#3 リリースビルドでの無効化 / オプトイン化**: 配布ビルドで debug transport を既定無効化、またはビルドフィーチャ/設定でのオプトインに。既存 `enabled` ゲートとの整合・優先順位を設計で確定。
4. **#4 エフェメラルポート（port 0）束縛＋実バインドポートのアドバタイズ**: 固定ポート衝突・多重起動を一般的に回避。**ただしクライアントは固定 9276 へ attach する契約**のため、エフェメラル化は「実バインドポートを VSCode 側へ伝える経路（ログ/ファイル/DAP）」が前提。`debug-startup-logging`（実バインド `host:port` ログ）と接続する論点。固定ポート契約を維持しつつ #1+#2 で再バインドを解くのか、エフェメラル＋アドバタイズへ移行するのかは設計の中心的トレードオフ。

挙動保存方針: 正常系（デバッグ機能：BP/ステップ/変数/提示モード/サイドカー）の外部挙動は厳密保存。変化させるのは bind ポリシー（#3 無効化・#4 ポート割当）に限り、境界をテストで明示する。

## Scope
- **In**:
  - debug transport の **bind/teardown ライフサイクル修正**（リスナースレッドの確実な停止と待受ソケット解放）。`crates/pasta_lua/src/debug/transport.rs`（`Drop`/`serve`/`start`）、必要に応じて `debug/wiring.rs`・`debug/mod.rs` の teardown 連携。
  - **`SO_REUSEADDR`** 設定。
  - **リリースビルドでの無効化/オプトイン化** 方針の実装。
  - **エフェメラルポート対応＋実バインドポートのアドバタイズ**（クライアント固定ポート契約との整合を含む）。
  - **同一プロセス内 unload→reload で同一構成の再バインドが成功する回帰テスト**（WSAEADDRINUSE 10048 の再現と解消の証拠）。
  - 必要に応じた VSCode 拡張側の attach ポート解決の調整（#4採用時）。
- **Out**:
  - デバッグ機能そのもの（BP/ステップ/変数 inspect/コルーチン/提示モード/ソースマップ/サイドカー）の挙動変更。
  - 起動確認ログの新規追加そのもの（`debug-startup-logging` の領域。本仕様は #4 のアドバタイズ経路として接続するに留める）。
  - SHIORI `load()`/`unload()` の一般的なライフサイクル再設計（debug transport 以外）。
  - ゴースト辞書（`.pasta`/Lua）側の変更（本件は pasta_lua の問題でゴースト側の不具合ではない）。

## Boundary Candidates
- 待受ソケットのライフサイクル（生成 → accept の中断可能化 → teardown での停止・解放）。
- bind ポリシー（`SO_REUSEADDR` / 固定 vs エフェメラルポート / リリース無効化ゲート）。
- クライアント側ポート解決（固定契約の維持 or アドバタイズされた実ポートへの追従）。

## Out of Boundary
- デバッグ機能の振る舞い、起動ログ基盤、SHIORI ライフサイクル全般、ゴースト辞書。

## Upstream / Downstream
- **Upstream**: `pasta-vscode-lua-debug`（完了）— `enable()`/`Transport`/`wiring` の本番実装。本仕様はその transport ライフサイクルの後始末漏れを根治する。`pasta-source-map`（完了）。
- **Downstream**: `debug-startup-logging`（未着手・Phase 5 派生）— #4 採用時の「実バインド `host:port` アドバタイズ」と直接連動。`pasta-manual-debugging`（実装済み）— リリースビルド無効化やポート挙動が変われば、デバッグ章の前提（固定 9276・ポート確認手順）を更新する小追補が要る可能性。

## Existing Spec Touchpoints
- **Extends**: 実質 `pasta-vscode-lua-debug`（完了・閉鎖済み）のデバッグ transport 領域への堅牢化だが、bind ポリシー変更（#3/#4）が外部挙動・クライアント契約に波及するため、独立した新規単一仕様として切り出す。
- **Adjacent**: `debug-startup-logging`（#4 のポートアドバタイズで接点）、`pasta-manual-debugging`（挙動変更時にドキュメント追補）、`editors/vscode`（attach ポート解決）。

## Constraints
- pasta.dll は emo2 と同一バイナリの複製（バージョン固定）として配布されるが、**修正の上流はこのリポジトリの `pasta_lua` クレート**であり、ビルド済み pasta.dll の差し替えで反映する。
- 全ゴースト（emo2 含む）が同一 debug transport 機構を共有するため、修正は共通基盤として効く。
- デバッグ無効時のゼロコスト・無言・サンドボックス（`std_debug` 非露出）維持を厳守（R5 系の既存保証を壊さない）。
- 同一 SSP プロセス内での unload→reload（約15秒間隔・同一 hinst）で再バインドが成功すること。
- `cargo test --all` 緑・既存デバッグ挙動の無回帰。LuaJIT ビルドは環境変数 `NoDefaultCurrentDirectoryInExePath` を外して実行する点に留意。
- 環境: Windows 11（`WSAEADDRINUSE`）。クロスプラットフォームでのソケット option 差異に留意。
