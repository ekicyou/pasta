# Requirements Document

## Project Description (Input)

開発（DEBUG）ビルドのゴーストを SSP 上で一度別ゴースト（えみり等）へ切り替えてから再選択すると、ゴーストが表示されなくなる。SHIORI `load()` がランタイム初期化で debug transport（DAP 待受 TCP ソケット）の再バインドに失敗して落ちるためである（`os error 10048` = Windows `WSAEADDRINUSE`）。真因は TIME_WAIT ではなく「生きたリスナースレッドが待受ポートを保持し続けるリソースリーク」であり、teardown 経路がリスナースレッドを停止・join せずにデタッチするため、`serve()` スレッドが固定ポートにバインドした `TcpListener` を SSP プロセスの寿命いっぱい握り続ける。SSP を完全再起動するまで当該ゴーストが起動しなくなり、開発ビルドでのゴースト切替運用が事実上不可能になっている。

本仕様は、要件ディスカッション（2026-06-13）での決定に基づき、**reload バグの根治**に純化する。すなわち、(#1) unload（teardown）時に待受待ち状態のリスナースレッドを中断・終了させ、その終了を**同期的に join して確認**したうえで待受ソケットを解放する根治と、(#2) 残存接続状態（TIME_WAIT 等）に対する防御として待受ソケットへ `SO_REUSEADDR` 相当を設定する根治の完成、を扱う。同一 SSP プロセス内での unload→reload（約15秒間隔・同一 hinst）で同一構成の再バインドが成功すること、デバッグ無効時の完全な無言・ゼロコスト・サンドボックス維持を厳守することが必須である。デバッグ機能そのもの（BP/ステップ/変数 inspect/コルーチン/提示モード/ソースマップ/サイドカー）の外部挙動は厳密に保存する。

なお当初検討の (#3) リリースビルドでの無効化／オプトイン化と (#4) エフェメラルポート＋実バインドポートのアドバタイズは、ディスカッションの結論により**本仕様から除外**した。理由は次のとおり。pasta.dll は **release ビルド1種類**を配布し、Lua/pasta デバッグ機能は**その配布版 release DLL が opt-in（既定 off）で持つ機能**であって DLL を2種に分けない設計思想のため、#3 のビルド種別ゲート（コンパイル除外／release 無効化）はこの思想に反し、かつ「既定 off ＋ opt-in」ゲートと loopback 固定（後述）により目標は既存実装で達成済みである。#4 は #1+#2 の根治で固定ポート契約のまま 10048 が解消されるため不要であり、固定ポート衝突・多重起動という別問題への投資（High リスク・クライアント契約／下流 `debug-startup-logging` への波及）に当たる。これらは「根治済みなら不要な対応はしない」という方針に沿って外した。

## Boundary Context

- **In scope**:
  - debug transport の bind/teardown ライフサイクル修正（待受待ち状態のリスナースレッドの**中断・終了と同期的 join**、待受ソケットの確実な解放）。
  - 待受ソケットの再利用耐性向上（`SO_REUSEADDR` 相当の設定）。
  - 同一プロセス内 unload→reload で同一構成の再バインドが成功することの回帰検証（`WSAEADDRINUSE` 10048 の再現と解消の証拠）。
- **Out of scope**:
  - **リリースビルドでの debug transport の無効化／オプトイン化（旧 #3）**。pasta.dll は単一 release ビルドを配布し、デバッグは同一 DLL の opt-in（既定 off）機能とする設計思想のため、ビルド種別ゲートは本仕様で扱わない（要件ディスカッション 2026-06-13 決定）。既存の「既定 off ＋ opt-in」ゲートと loopback 固定で目標は達成済み。
  - **エフェメラルポート対応と実バインドポートのアドバタイズ（旧 #4）**。#1+#2 の根治で固定ポート契約のまま 10048 が解消されるため不要（同決定）。固定ポート衝突・多重起動の一般対策は本仕様の対象外。
  - **VSCode 拡張側 attach ポート解決の変更**。固定ポート `9276` 契約を**維持する**ため、本仕様では拡張側を変更しない。
  - デバッグ機能そのもの（ブレークポイント／ステップ／変数 inspect／コルーチン／提示モード／ソースマップ／サイドカー）の振る舞い変更。
  - 起動確認ログ基盤の新規追加（`debug-startup-logging` の領域）。
  - SHIORI `load()`／`unload()` の debug transport 以外の一般的なライフサイクル再設計。
  - ゴースト辞書（`.pasta`／Lua）側の変更。
- **Adjacent expectations**:
  - 上流 `pasta-vscode-lua-debug`（完了）が提供する `enable()`／`Transport`／`wiring` の本番実装と、`pasta-source-map`（完了）のソースマップ注入経路は前提として維持し、その外部挙動を壊さない。
  - VSCode クライアントは既定で固定ポート（`9276`）へ attach する契約を持つ。本仕様はこの契約を**変更せず維持**する（エフェメラル化を行わないため、実バインドポートのクライアント解決経路は不要）。
  - debug transport の待受アドレスは loopback（`127.0.0.1`）固定であり、host は env / pasta.toml から上書きできない（外部非公開）。本仕様はこの不変条件を維持する。
  - 下流 `pasta-manual-debugging`（実装済み）はポート挙動（固定 9276）・リリース時のデバッグ可否を**変更しない**ため、ドキュメント追補は原則不要。
  - 全ゴースト（emo2 含む）が同一 debug transport 機構を共有するため、本修正は共通基盤として効く。

## Requirements

### Requirement 1: 同一プロセス内 unload→reload での再バインド成立（根治）

**Objective:** デバッグビルドのゴースト作者として、SSP 上でゴーストを切り替えて再選択したときにゴーストが再び表示されてほしい。そうすれば、SSP を完全再起動せずに開発ビルドのままゴースト切替運用を続けられる。

#### Acceptance Criteria

1. When 同一 SSP プロセス内で debug transport が有効なランタイムが unload された後に同一構成で再度 load される, the pasta debug transport は 待受ポートへの再バインドに成功する（`WSAEADDRINUSE`／`os error 10048` を発生させない）。
2. When 同一構成での unload→reload が成功する, the SHIORI load 処理は ランタイム初期化を最後まで完了し、OnBoot が発動してゴーストが表示される状態を返す。
3. While debug transport が有効である, when ランタイムが連続して複数回（最低でも約15秒間隔で2回以上）unload→reload される, the pasta debug transport は 各 reload で待受ポートへの再バインドに成功する。
4. If teardown 後に待受ポートが解放されずに次回 reload の bind が失敗する状況が再現する, then the 回帰テストは その失敗（`WSAEADDRINUSE` 相当）を検出して fail する。

### Requirement 2: teardown による待受ソケットの確実な解放

**Objective:** pasta ランタイムの保守担当として、ランタイム破棄（unload）時に debug transport が確保した待受ポートとリスナースレッドが、unload 完了までに**同期的に**確実に後始末されてほしい。そうすれば、同一プロセス内にポートやスレッドが居残らず、リソースリークを起こさない。

#### Acceptance Criteria

1. When debug transport が有効なランタイムが unload される, the pasta debug transport は 待受ソケットを閉じ、当該待受ポートを同一プロセス内に保持し続けない。
2. When unload（teardown）が要求される, the pasta debug transport は 待受待ち状態のリスナースレッドを中断・終了させ、その終了を同期的に join して確認したうえで unload を完了する（デタッチしたまま生存させない・投げっぱなしにしない）。
3. While teardown が進行している, the pasta debug transport は リスナースレッドの中断機構により join を有界な時間内に完了し、debug transport 以外のランタイム破棄処理を不当にブロックしない（無限ブロックする accept を join してハングすることがない）。
4. The pasta debug transport は teardown 完了後、同一構成での再 start を可能な状態にする。
5. While デバッガクライアントが接続中に unload が要求される, the pasta debug transport は 接続中の socket を含めて同期的に解放し、リスナースレッドの join を有界な時間内に完了する。

### Requirement 3: 待受ソケットの再利用耐性（SO_REUSEADDR 相当）

**Objective:** pasta ランタイムの保守担当として、待受ソケットが残存接続状態（TIME_WAIT 等）に対しても耐性を持って再バインドできてほしい。そうすれば、根治後も残るプラットフォーム依存のポート滞留に対して防御層が働く。

#### Acceptance Criteria

1. Where debug transport が有効で待受ソケットを生成する, the pasta debug transport は 待受アドレスの再利用を許可する設定（`SO_REUSEADDR` 相当）を適用する。
2. When 直前の同一アドレスへのバインドが残存接続状態にある中で再バインドが要求される, the pasta debug transport は 当該残存状態のみを理由としたバインド失敗を起こさない。
3. The pasta debug transport は 待受ソケットの再利用設定を、デバッグが有効なときに限り適用し、無効時には一切のソケットを生成しない。

### Requirement 4: 既存デバッグ挙動の無回帰と検証

**Objective:** pasta ランタイムの保守担当として、本堅牢化がデバッグ機能の正常系挙動を一切変えないことを保証してほしい。そうすれば、teardown の同期化とソケット再利用設定以外の外部挙動を壊さずに出荷できる。

#### Acceptance Criteria

1. While debug transport が有効でデバッガが接続している, the pasta debug transport は ブレークポイント・ステップ・変数 inspect・コルーチン・提示モード・ソースマップ・サイドカーの外部挙動を従来どおり提供する。
2. When 本堅牢化が適用される, the 変化する外部挙動は teardown の同期化（リスナースレッドの中断・join）と待受ソケットへの `SO_REUSEADDR` 相当設定に限定され、待受アドレス（loopback `127.0.0.1` 固定）・待受ポート（既定 `9276`）・有効化ゲート（既定 off ＋ opt-in）・その他のデバッグ機能の観測可能な挙動は保存される。
3. The 自動テストは 同一プロセス内 unload→reload の再バインド成立（Requirement 1）と待受ソケット解放（Requirement 2）を、`WSAEADDRINUSE` 10048 の再現・解消を含めて検証する。
4. When プロジェクトの全テストが実行される, the 変更一式は 既存のデバッグ挙動を回帰させずにテストを成功させる。
5. Where ソケット設定の挙動がプラットフォームによって差異を持つ, the pasta debug transport は 対象環境（Windows）で要求挙動を満たし、他プラットフォームでビルド・テストを破壊しない。
