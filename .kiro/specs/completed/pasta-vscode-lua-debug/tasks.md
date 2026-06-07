# Implementation Plan

> 出荷コア = 生成 `.lua` レベルのデバッグ。R-2（コルーチン state FFI 走査）と採択B（thread 追跡ステップ）を Core 前半で先行 de-risk する。`.pasta` ソースマップは実現可能性確定（シーム＋薄い実証スライス）までで、本番化は別仕様 `pasta-source-map`。PoC 除去（R9）は検証完了後の最終ゲート。

- [x] 1. Foundation: デバッグモジュールの土台・共有型・フック
- [x] 1.1 デバッグモジュールの新設と有効化ゲート
  - pasta_lua に独立デバッグモジュールを新設し、設定（pasta.toml `[debug]` の `enabled`/`port`、既定 `port = 9276`）と環境変数（`PASTA_DEBUG`/`PASTA_DEBUG_PORT`）から有効化を判定する `DebugConfig` を確立
  - `enable()` の骨格を用意：無効時は `None` を返し、フック非設置・接続口非開放・`std_debug` 非露出。`DebugError` 型を定義
  - enable() の transport 起動は 4.1 で結線する（本タスクでは有効時にハンドル骨格を返すところまで・増分前提を明示）
  - 観測: 無効ビルドはフック痕跡なし・ポート非開放で、有効指定時はデバッグハンドルを返す単体テストが通る
  - _Requirements: 5.1, 5.2, 5.3, 5.5_

- [x] 1.2 共有デバッグ型の確立（PoC harness_types から昇格）
  - `LineEvent`/`Variable`/`FrameInfo`/`SessionCommand`/`SessionEvent`/`StopReason`/`Breakpoint`/`ThreadId` 等、DAP 非依存の素の型を定義
  - `mlua::Error`（`!Send`）越境用に `SessionEvent::Error(String)` を含める
  - 観測: 型がコンパイルし、`Variable`/`FrameInfo` のラウンドトリップ単体テストが通る
  - _Requirements: 6.1_
  - _Boundary: DebugSession, FrameInspector 共有型_

- [x] 1.3 VM フック設置と jit.off・コルーチン横断発火
  - 有効時のみ無引数 `jit.off()`（エンジン全体）を適用し、`set_global_hook`（EVERY_LINE）で全コルーチン横断の line フックを設置。callback はセッションへ接続し常に継続を返す
  - hook 内 panic はサイドチャネルへ記録する
  - 観測: 動的生成されたシーンコルーチン群を横断して line フックが発火することを示す統合テストが通る
  - _Requirements: 1.7, 5.2, 5.4_
  - _Boundary: VmHook_

- [x] 2. Core: 停止制御と inspect（高リスク先行 de-risk）
- [x] 2.1 ブレークポイントストアと解決
  - 実行中も設定可能な共有ブレークポイント集合と、`(source, line)` 包含述語による停止判定を実装
  - 観測: `should_pause` が `(source, line)` 一致で停止判定し不一致で継続する単体テストが通る
  - _Requirements: 1.1_
  - _Boundary: Breakpoints_

- [x] 2.2 停止ループと continue（DebugSession）
  - 停止イベント送出と無期限ブロッキング待機、続行コマンドでの再開を実装（watchdog なし）
  - 観測: ブレークポイント到達で停止イベントを送出し、続行コマンドで実行を再開する統合テストが通る
  - _Requirements: 1.2, 1.6, 3.4_
  - _Depends: 1.3, 2.1_
  - _Boundary: DebugSession_

- [x] 2.3 コールスタックと変数取得（メインフレーム）
  - 安全 API と FFI でコールスタックと局所変数・上位値を取得。number/string/boolean/table を判別し、未対応型は取得不能表現で graceful に継続（VM スタック規律厳守）
  - 観測: 停止時にフレーム一覧と局所変数（各型）を返し、未対応型・到達不能フレームでもエラー停止せず継続する統合テストが通る
  - _Requirements: 2.1, 2.2, 2.3, 2.5_
  - _Depends: 2.2_
  - _Boundary: FrameInspector_

- [x] 2.4 コルーチン本体フレームの変数 inspect（R2.4 本番対応・最高リスク）
  - フックの `&Lua`（メインステート）ではなく走行中コルーチンの生 `lua_State`（`current_thread().state()`）を走査し、コルーチン本体フレームの局所変数へ到達。ThreadId の resume 跨ぎ安定性をここで先行確認
  - 観測: 走行中シーンコルーチン本体フレームの局所変数を取得できる統合テストが通る
  - _Requirements: 2.4_
  - _Depends: 2.3_
  - _Boundary: FrameInspector_

- [x] 2.5 (P) StepController（thread identity 追跡・採択B）
  - ステップを `(thread, base_depth)` で鍵付け。現在 thread 一致時のみ深さ判定し over/into/out を実装、thread 不一致（ホスト/別コルーチン）行はスキップ、yield/resume を跨いで成立。リクエスト跨ぎ非同期 yield は次 resume で停止
  - 観測: yield するコルーチンで step over/into/out が期待される `.lua` 行で停止する統合テストが通る
  - _Requirements: 1.3, 1.4, 1.5_
  - _Depends: 2.2_
  - _Boundary: DebugSession StepController_

- [x] 3. Core: プロトコルとトランスポート
- [x] 3.1 (P) トランスポート（TCP・Content-Length フレーミング）
  - `listen=None` で非開放、有効時は 1 接続を accept、DAP 準拠 Content-Length フレーミングで読み書きする I/O 専用スレッド（Lua 非アクセス）
  - 観測: クライアント接続でフレーム化 JSON を往復でき、無効時はポートを開かない統合テストが通る
  - _Requirements: 3.1, 5.5_
  - _Boundary: Transport_

- [x] 3.2 (P) DAP 最小サブセットアダプタ
  - initialize / setBreakpoints / configurationDone / threads / stackTrace / scopes / variables / continue / next / stepIn / stepOut と stopped / terminated を serde_json で手書きし、`SessionCommand`/`SessionEvent` へ相互変換
  - 観測: 各 DAP メッセージのラウンドトリップ単体テストが通り、initialize がケイパビリティを応答する
  - _Requirements: 3.2, 3.3, 3.4, 3.5_
  - _Depends: 1.2_
  - _Boundary: DapAdapter_

- [x] 4. Integration: バックエンド全結線とランタイム統合
- [x] 4.1 transport↔dap↔session↔hook の全結線
  - トランスポート I/O・DAP 変換・セッション停止コア・フックをチャネルで結線し、attach から terminate までの全経路を成立させる（停止中の inspect/stack/step は VM スレッドのフック内ループで実行）。SHIORI 非依存の `enable` をテストハーネスが直接利用
  - 観測: TCP 経由で DAP セッションを駆動し、`.lua` に BP 設定→ヒット→stackTrace→variables→step→continue→terminated まで通る統合テストが通る
  - _Requirements: 3.3, 3.5, 6.1, 6.2, 6.3_
  - _Depends: 2.2, 2.3, 2.5, 3.1, 3.2_

- [x] 4.2 ランタイム VM 初期化への統合と runtime スコープ永続
  - runtime の VM 初期化で `debug::enable` を呼び（runtime/mod.rs を改修）、`DebugHandle` を runtime スコープで保持。セッション/ブレークポイント状態を多数の短命 SHIORI リクエスト跨ぎで永続させ、停止はリクエスト処理中のみ・無効時はゼロ
  - 観測: 有効ランタイムがフックを一度だけ設置し、複数スクリプト実行（リクエスト）を跨いで BP/セッション状態が永続する統合テストが通り、無効時はフック非設置
  - _Requirements: 5.2, 5.4, 6.1_
  - _Depends: 1.1, 1.3, 4.1_
  - _Boundary: Runtime integration (runtime/mod.rs)_

- [x] 5. `.pasta` ソースマップ実現可能性確定（R4: シーム＋薄い実証スライス）
- [x] 5.1 code_gen ソースマップ・シーム
  - コードジェネレータに出力行カウンタと差し込み可能な記録シンクを追加し、span を出力記録点へ引き回す。本番トランスパイルは無シンクで出力バイト一致を維持
  - 観測: 無シンクで従来とバイト一致の回帰テストが通り、捕捉シンク装着時は出力行→`.pasta` span を記録できる
  - _Requirements: 4.1, 4.2, 4.6_
  - _Boundary: CodeGenSourceMapHook, SourceMapSeam_

- [x] 5.2 DAP source 取り扱い口
  - コールスタックの source を既定で生成 `.lua` とし、将来 `.pasta` パスを提示できる差し替え可能な構造で口を用意
  - 観測: stackTrace が `.lua` source を返し、source 表現が `.pasta` 向けに差し替え可能であることをテストが示す
  - _Requirements: 4.3_
  - _Depends: 3.2_
  - _Boundary: DapAdapter, SourceMapSeam_

- [x] 5.3 薄い実証スライス（feature gate）
  - feature gate 下で代表 1 経路の行対応マップを構築し、停止位置の `.lua` 行→`.pasta` 行へ変換、`.pasta` 行ブレークポイントのヒットを実コードで実証。残課題（全 generate_* 網羅・currentline 端ケース・双方向変換）を research へ記録
  - 観測: feature＋debug 有効で代表経路の `.pasta` 行 BP がヒットし停止が `.pasta` 行を報告、feature 無効時は経路非露出
  - _Requirements: 4.4, 4.5, 4.6_
  - _Depends: 5.1, 4.1_
  - _Boundary: SourceMapSeam_

- [x] 6. VSCode 拡張統合
- [x] 6.1 (P) デバッグ貢献と DebugAdapterServer ファクトリ
  - `contributes.debuggers`（type `pasta`・attach）/ `activationEvents` / `contributes.breakpoints` を追加し、`registerDebugAdapterDescriptorFactory` で `DebugAdapterServer(host, port)`（既定 `127.0.0.1:9276`）を返す薄い Factory を実装。ビルド（esbuild→vsce）は不変
  - 観測: attach デバッグ構成が VSCode に現れ、接続でバックエンドへ繋がり、`.pasta`/`.lua` にブレークポイントを設定できる
  - _Requirements: 3.6_
  - _Depends: 4.1_
  - _Boundary: VscodeDebugFactory_

- [x] 7. 撤去と運用ガイダンス
- [x] 7.1 ブレーク中応答停止の運用ガイダンス（R7）
  - 停止中は VM スレッドが SHIORI リクエストの Mutex を保持し当該＋後続の全 SHIORI リクエストが待機する構造的制約、SSP タイムアウト回避の運用注意・緩和策、根本解決がスコープ外であることをデバッグ利用ガイダンスに明示
  - 観測: ガイダンス節が構造的制約・緩和策・スコープ外を記述している
  - _Requirements: 7.1, 7.2, 7.3_
  - _Boundary: BreakStallGuidance_

- [x] 7.2 旧 luasocket デバッグ資産の撤去（R8）
  - `vscode-debuggee.lua` / `socket/core.dll` / `mime/core.dll` / `dkjson.lua` を削除し、内蔵 zip を再生成。撤去後の起動・スクリプト実行に回帰がないことを確認
  - 観測: 配布物（内蔵 zip）に該当 4 資産が含まれず、既存の loader/runtime テストがパスする
  - _Requirements: 8.1, 8.2, 8.3_
  - _Boundary: LegacyAssetRemoval_

- [x] 8. Validation と完了
- [x] 8.1 E2E Lua レベルデバッグセッション
  - attach→`.lua` 行 BP→ヒット→step over/into/out→変数 inspect（コルーチン本体フレーム含む）→continue→terminated の全経路を VSCode 相当の DAP クライアントから実施
  - 観測: 全通し E2E がパスする
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 2.5, 3.6_
  - _Depends: 4.1, 4.2, 6.1_

- [x] 8.2 ゼロコスト/サンドボックス回帰
  - 無効ビルドでフック非設置・接続口非開放・`std_debug` 非露出・トランスパイル出力バイト一致を回帰として検証
  - 観測: 無効時の各回帰テスト（フックなし/ポートなし/サンドボックス維持/出力バイト一致）がパスする
  - _Requirements: 5.2, 5.3, 5.5, 4.6_
  - _Depends: 4.2, 5.1_

- [x] 8.3 薄い実証スライス E2E（gate 有効）と残課題申し送り
  - gate 有効で代表経路の `.pasta` 行 BP ヒット・`.pasta` 行報告を E2E で確認し、本番化の残課題を別仕様 `pasta-source-map` へ申し送る記録を残す
  - 観測: gated E2E がパスし、research に残課題と分割境界が記録される
  - _Requirements: 4.4, 4.5, 4.1_
  - _Depends: 5.3_

- [x] 8.4 PoC ハーネス除去（R9・完了ゲート・最終）
  - 本番実装の検証完了・PoC 知見の本番移行完了・本番側の同等以上の自動テスト存在を満たした上で、`lua-debug-poc` feature とテストモジュール一式・gated `mod` 宣言行を削除。GO+ の担保を本番テストと feasibility の research へ移行（前提未充足なら残置）
  - 観測: `lua-debug-poc` が消えてワークスペースがビルド/テストを通過し、PoC ハーネスへの参照が残存しない
  - _Requirements: 9.1, 9.2, 9.3, 9.4_
  - _Depends: 8.1, 8.2, 8.3_

## Implementation Notes

実装中に判明した横断的知見（後続タスクはこれを前提にしてよい）:

- **ビルド環境（必須）**: `cargo build`/`test` の前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外さないと mlua-sys/LuaJIT の vendored ビルドが exit 101 で死ぬ。PowerShell で毎回 `Remove-Item Env:\NoDefaultCurrentDirectoryInExePath -ErrorAction SilentlyContinue;` を前置する。
- **.gitignore 修正済み**: ルートの bare `debug` パターンが `src/debug/` を巻き込んでいたため `/debug`（ルート限定）へ修正済み。デバッグモジュール配下の新規ファイルは追跡される。
- **R2.4 解決済み（最重要）**: 走行中 Lua コルーチン本体フレームの局所変数は、フック内で `lua.current_thread().state()`（走行コルーチンの生 `lua_State*`）を走査すれば到達できる。mlua 0.11.6 は `global_hook_proc`＋`callback_error_ext` の `StateGuard` がコールバック実行中 `RawLua.state` を走行コルーチンへ差し替えるため、フック内 `current_thread()` は走行コルーチンを返す（PoC の「main 固定」記述は `exec_raw` 由来の制約だった）。`ThreadId`（state ポインタ）は同一コルーチンの yield/resume を跨いで安定。
- **FrameInspector（inspect.rs）**: `capture_stack(lua,&thread)` / `capture_variables(lua,&thread,level)` は渡された `thread.state()` を直接 FFI 走査する（`exec_raw` 不使用 → C フレームオフセット無し、level 0＝停止フレーム）。number/string/boolean/table を判別、未対応種別は `<unsupported T>`、`lua_gettop`→`lua_settop` 対称・graceful（Vec 返し、エラーで停止しない）。コルーチン対応は呼び出し側で thread を渡すだけ。
- **DebugSession（session.rs）**: 停止コアは無期限 `recv()`（watchdog はテスト専用）。フックは常に `VmState::Continue` を返す（LuaJIT は Yield 不可）。`mlua::Lua` はスレッド境界を越えない（`!Send`）。`mlua::Error` は `SessionEvent::Error(String)` で越境。`LineHook::on_line(&self,...)` seam にプラグインする。
- **StepController（session.rs）**: `RunMode::Stepping{kind, thread, base_depth, start_line}`。深さは `inspect::capture_stack(lua,&thread).len()`。thread 不一致行はスキップ。over/in/out は深さ＋行変化で判定し、coroutine yield/resume を跨いで成立。
- **全結線（4.1, wiring.rs）**: VM ホストスレッド（`mlua::Lua`＋フック内 `DebugSession`・停止中に inspect/step を処理）＋ socket-bridge スレッド（`Transport` を単独所有・inbound を 5ms poll しつつ outbound を drain）＋ event-encoder スレッド。`DapAdapter` は `Arc<Mutex<…>>` 共有。`setBreakpoints` は実行中でも共有 `BreakpointSet` へ直接適用（停止ループを経由しない）。`scopes`/`threads`/`setBreakpoints` は単一応答保証。`DebugHandle::Drop` は shutdown フラグ＋detach（非ブロッキング）。
- **4.2 への申し送り（必須）**: 自然な実行終了（`exec()` 復帰）時の `terminated` イベントは 4.1 では未発火（disconnect→terminated 経路のみ結線済み）。ランタイム（runtime/mod.rs）が `exec()` 復帰後に `SessionEvent::Terminated` を発火する責務を 4.2 で実装すること（R3.5 = 「実行が終了したとき」）。
