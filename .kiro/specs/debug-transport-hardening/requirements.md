# Requirements Document

## Project Description (Input)

開発（DEBUG）ビルドのゴーストを SSP 上で一度別ゴースト（えみり等）へ切り替えてから再選択すると、ゴーストが表示されなくなる。SHIORI `load()` がランタイム初期化で debug transport（DAP 待受 TCP ソケット）の再バインドに失敗して落ちるためである（`os error 10048` = Windows `WSAEADDRINUSE`）。真因は TIME_WAIT ではなく「生きたリスナースレッドが待受ポートを保持し続けるリソースリーク」であり、teardown 経路がリスナースレッドを停止・join せずにデタッチするため、`serve()` スレッドが固定ポートにバインドした `TcpListener` を SSP プロセスの寿命いっぱい握り続ける。SSP を完全再起動するまで当該ゴーストが起動しなくなり、開発ビルドでのゴースト切替運用が事実上不可能になっている。

本仕様は、ユーザー決定（2026-06-13）に基づき、debug transport の堅牢化を推奨#1〜#4 全部入りで行う。すなわち、(#1) teardown 時にリスナースレッドを確実に停止し待受ソケットを解放する根治、(#2) 待受ソケットへの `SO_REUSEADDR` 設定、(#3) リリースビルドでの debug transport の無効化／オプトイン化、(#4) エフェメラルポート束縛＋実バインドポートのアドバタイズ、を扱う。同一 SSP プロセス内での unload→reload（約15秒間隔・同一 hinst）で同一構成の再バインドが成功すること、デバッグ無効時の完全な無言・ゼロコスト・サンドボックス維持を厳守することが必須である。デバッグ機能そのもの（BP/ステップ/変数 inspect/コルーチン/提示モード/ソースマップ/サイドカー）の外部挙動は厳密に保存する。

## Boundary Context

- **In scope**:
  - debug transport の bind/teardown ライフサイクル修正（リスナースレッドの確実な停止と待受ソケットの解放）。
  - 待受ソケットの再利用耐性向上（`SO_REUSEADDR` 相当の設定）。
  - リリースビルドでの debug transport の無効化／オプトイン化方針。
  - エフェメラルポート対応と実バインドポートのアドバタイズ（クライアント固定ポート契約との整合を含む）。
  - 同一プロセス内 unload→reload で同一構成の再バインドが成功することの回帰検証（`WSAEADDRINUSE` 10048 の再現と解消の証拠）。
  - 上記#4採用時に必要な範囲での VSCode 拡張側 attach ポート解決の調整。
- **Out of scope**:
  - デバッグ機能そのもの（ブレークポイント／ステップ／変数 inspect／コルーチン／提示モード／ソースマップ／サイドカー）の振る舞い変更。
  - 起動確認ログ基盤の新規追加（`debug-startup-logging` の領域。本仕様は#4採用時のアドバタイズ経路として接続するに留める）。
  - SHIORI `load()`／`unload()` の debug transport 以外の一般的なライフサイクル再設計。
  - ゴースト辞書（`.pasta`／Lua）側の変更。
- **Adjacent expectations**:
  - 上流 `pasta-vscode-lua-debug`（完了）が提供する `enable()`／`Transport`／`wiring` の本番実装と、`pasta-source-map`（完了）のソースマップ注入経路は前提として維持し、その外部挙動を壊さない。
  - VSCode クライアントは既定で固定ポート（`9276`）へ attach する契約を持つ。エフェメラル化を採用する場合、実バインドポートをクライアントが解決できる経路の存在を前提とする。
  - 下流 `debug-startup-logging`（未着手）は#4の実バインド `host:port` アドバタイズと連動しうる。
  - 下流 `pasta-manual-debugging`（実装済み）はポート挙動やリリース無効化が変わる場合にドキュメント追補の対象となりうる。
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

### Requirement 4: リリースビルドでの無効化／オプトイン化

**Objective:** 配布（リリース）ゴーストの利用者および配布者として、リリースビルドが不要な待受ポートを開かないでほしい。そうすれば、無駄なポート露出を避け、セキュリティ面の懸念を低減できる。

#### Acceptance Criteria

1. While 配布（リリース）ビルドである, the pasta debug transport は 既定で無効化され、待受ポートを開かない。
2. Where 配布ビルドで debug transport を意図的に有効化する手段が提供される, the pasta debug transport は その明示的なオプトインが指定されたときに限り待受を開始する。
3. While debug transport が無効である, the pasta runtime は ソケットを開かず、リスナースレッドを起動せず、デバッグ用機能をスクリプトへ露出しない（既存のゼロコスト・無言・サンドボックス保証を維持する）。
4. When 有効化に関する複数の指定（ビルド種別・設定・環境変数・オプトイン手段）が併存する, the pasta debug transport は 定義された優先順位に従って一意に有効／無効を決定する。

### Requirement 5: エフェメラルポート対応と実バインドポートのアドバタイズ

**Objective:** デバッグビルドのゴースト作者として、固定ポートの衝突や多重起動に強いポート割当を使いつつ、デバッガが実際の待受ポートへ接続できてほしい。そうすれば、ポート固定に起因する再バインド衝突を一般的に回避できる。

#### Acceptance Criteria

1. Where エフェメラルポート（OS 任意割当）でのバインドが構成される, the pasta debug transport は OS が割り当てた実バインドポートを取得して保持する。
2. When エフェメラルポートで待受が開始される, the pasta debug transport は 実バインドした `host:port` を、デバッガクライアントが解決できる経路へアドバタイズする。
3. While 固定ポート契約（既定 `9276`）が維持される構成である, the pasta debug transport は 従来どおり当該固定ポートで待受を開始する。
4. Where #4 のエフェメラル化に伴い VSCode 拡張側の attach ポート解決が調整される, the VSCode 拡張は アドバタイズされた実ポートへ attach できる（明示ポート指定がある場合はそれを優先し、既存の有効ポート指定の解決挙動は保存する）。
5. If アドバタイズされた実バインドポート情報が利用できない, then the VSCode 拡張は 既定の固定ポート（`9276`）へのフォールバックを行う。

### Requirement 6: 既存デバッグ挙動の無回帰と検証

**Objective:** pasta ランタイムの保守担当として、本堅牢化がデバッグ機能の正常系挙動を一切変えないことを保証してほしい。そうすれば、bind ポリシーの変更（無効化・ポート割当）以外の外部挙動を壊さずに出荷できる。

#### Acceptance Criteria

1. While debug transport が有効でデバッガが接続している, the pasta debug transport は ブレークポイント・ステップ・変数 inspect・コルーチン・提示モード・ソースマップ・サイドカーの外部挙動を従来どおり提供する。
2. When 本堅牢化が適用される, the 変化する外部挙動は bind ポリシー（無効化ゲートとポート割当・アドバタイズ）に限定され、それ以外のデバッグ機能の観測可能な挙動は保存される。
3. The 自動テストは 同一プロセス内 unload→reload の再バインド成立（Requirement 1）と待受ソケット解放（Requirement 2）を、`WSAEADDRINUSE` 10048 の再現・解消を含めて検証する。
4. When プロジェクトの全テストが実行される, the 変更一式は 既存のデバッグ挙動を回帰させずにテストを成功させる。
5. Where ソケット設定の挙動がプラットフォームによって差異を持つ, the pasta debug transport は 対象環境（Windows）で要求挙動を満たし、他プラットフォームでビルド・テストを破壊しない。
