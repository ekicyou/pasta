# Requirements Document

## Introduction

pasta ゴースト作者が `.pasta` 行にブレークポイントを張ってデバッグするとき、停止位置の生成 `.lua` コードを確認したい場面がある（トランスパイル結果の挙動確認・不具合切り分け）。現状、提示モード（`.pasta` / `.lua`）はデバッグセッション開始時（attach 引数・環境変数・`pasta.toml`）に固定で決まるのみで、利用者がセッション中に切り替える手段が露出していない。体感としては `.pasta` 提示に固定されている。

本機能は、デバッグセッション中に VSCode のコマンド／ボタンから提示モードを `.pasta` ⇔ `.lua` へ即時に切り替えられるようにする。`.pasta` 行に張ったブレークポイントはそのまま有効で、停止時のスタックトレース・source 応答が選択中モードに応じて `.pasta` か `.lua` を提示する。提示モード切替の内部基盤（`SourceMode` / `SharedSourceMode` / レゾルバ差し替え）は既存仕様（pasta-source-map）で実装済みであり、本機能はそこへ実行時トグルの制御経路（DAP カスタムリクエスト）と VSCode 側の UI を追加して、attach 時固定から実行時トグルへ前進させる。

本機能は提示モード切替の制御と提示のみを対象とし、レゾルバ・ソースマップそのものの再設計は行わない（既存の双方向マップを流用する）。

## Boundary Context

- **In scope（利用者から観測できる振る舞い）**:
  - デバッグセッション中に提示モードを `.pasta` ⇔ `.lua` へ実行時に切り替える制御経路（DAP カスタムリクエスト）。
  - VSCode 拡張のトグル操作（コマンドパレットのコマンド＋デバッグツールバーのボタン）。
  - 切替がスタックトレース・source 提示へ即時に反映され、停止中であれば現在の提示が新モードで再描画されること。
  - attach 引数 `sourcePresentation` による初期モード指定と、実行時トグルによる上書きの整合。
  - 提示モードに応じたステップ粒度（`.lua` モードでは `.lua` 粒度、`.pasta` モードでは `.pasta` 粒度）の整合。
  - 「`.pasta` 行にブレークポイントを張ったまま `.lua` 表示へ切り替える」ワークフローの End-to-End 検証。
  - マニュアル デバッグ章（提示モード切替の節）の実行時トグル手順への更新。
- **Out of scope（本機能が所有しない振る舞い）**:
  - 同一 `.pasta` 行の再ブレーク抑制・停止制御フローの判定（pasta-debug-break-coalesce が所有）。
  - 提示レゾルバ／ソースマップ生成そのものの再設計（既存の双方向マップを流用）。
  - `.lua` 提示時の独自シンタックス装飾など、提示の表示拡張。
- **Adjacent expectations（隣接仕様・基盤への期待）**:
  - pasta-source-map（完了）が提供する `.pasta`/`.lua` 双方向マップと提示モード切替基盤（`SharedSourceMode` の実行時更新で提示が切り替わること）に依存する。
  - pasta-vscode-lua-debug（完了）が提供する DAP-over-TCP attach バックエンドと VSCode attach 接続経路に依存する。
  - ブレークポイントは提示モードに依存せず維持される（`.pasta` 行に張ったブレークポイントは `.lua` 表示へ切り替えても引き続き有効）。

## Requirements

### Requirement 1: 提示モードの実行時切替（制御経路）

**Objective:** As a pasta ゴースト作者, I want デバッグセッション中に提示モードを実行時に切り替える制御要求を発行できる, so that セッションを張り直さずに `.pasta` と生成 `.lua` の見え方を行き来できる

#### Acceptance Criteria

1. When 提示モードを `.lua` に切り替える要求がデバッグセッションへ送られたとき, the Pasta デバッグバックエンド shall 以後の提示モードを `.lua` に更新する。
2. When 提示モードを `.pasta` に切り替える要求がデバッグセッションへ送られたとき, the Pasta デバッグバックエンド shall 以後の提示モードを `.pasta` に更新する。
3. When 提示モード切替の要求を受け取ったとき, the Pasta デバッグバックエンド shall 要求が受理されたことを利用者が確認できる応答を返す。
4. If 切替要求が認識できない提示モード値を含むとき, then the Pasta デバッグバックエンド shall 現在の提示モードを変更せず, かつデバッグセッションを継続する。
5. While デバッグセッションが実行中（停止していない）であるとき, the Pasta デバッグバックエンド shall 切替要求を受理し, 次の停止時の提示へ反映する。

### Requirement 2: VSCode からのトグル操作

**Objective:** As a pasta ゴースト作者, I want VSCode のコマンドおよびボタンから提示モードを切り替えられる, so that デバッグ中に手を止めず素早く `.pasta`/`.lua` の見え方を変えられる

#### Acceptance Criteria

1. The pasta VSCode 拡張 shall 提示モードを切り替えるコマンドをコマンドパレットから実行できるよう提供する。
2. While Pasta デバッグセッションが実行中であるとき, the pasta VSCode 拡張 shall デバッグツールバーから提示モードを切り替えるボタンを利用できるよう提供する。
3. When 利用者がトグル操作を実行したとき, the pasta VSCode 拡張 shall 切替の制御要求を実行中の Pasta デバッグセッションへ送る。
4. If Pasta デバッグセッションが実行中でないときにトグル操作が実行されたとき, then the pasta VSCode 拡張 shall 提示モードを変更せず, かつ操作が無効である旨を利用者が把握できるようにする。
5. While Pasta デバッグセッションが実行中であるとき, the pasta VSCode 拡張 shall 現在の提示モード（`.pasta` か `.lua` か）を, 利用者が常時判別できるように表示し続ける。
6. When 提示モードの切替が反映されたとき, the pasta VSCode 拡張 shall 常時表示している現在の提示モードを切替後の値へ更新する。

### Requirement 3: 切替の提示への即時反映

**Objective:** As a pasta ゴースト作者, I want 切替した提示モードがスタックトレースと source 提示へ即時に反映される, so that 切替直後に意図したソース座標で停止位置を確認できる

#### Acceptance Criteria

1. When 提示モードが切り替わった後にスタックトレースが要求されたとき, the Pasta デバッグバックエンド shall 切替後のモードに応じた座標（`.pasta` または `.lua`）でフレームを提示する。
2. When 提示モードが切り替わった後に source 内容が要求されたとき, the Pasta デバッグバックエンド shall 切替後のモードに応じたソース（`.pasta` または `.lua`）を提示する。
3. While デバッグセッションが停止中であるとき, when 提示モードが切り替わったとき, the Pasta デバッグセッション shall 現在の停止位置の提示を, 利用者の追加操作（ステップ実行・フレーム再選択等）を要さずに, 切替後のモードで即座に再描画させる。
4. When 提示モードが `.pasta` から `.lua` へ切り替わったとき, the Pasta デバッグバックエンド shall 停止位置・コールスタック・source を生成 `.lua` の座標で提示する。
5. When 提示モードが `.lua` から `.pasta` へ切り替わったとき, the Pasta デバッグバックエンド shall 停止位置・コールスタック・source を `.pasta` の座標で提示する。

### Requirement 4: 初期モード指定と実行時トグルの整合

**Objective:** As a pasta ゴースト作者, I want attach 時の初期モード指定が尊重されつつ実行時トグルで上書きできる, so that 起動時の既定と途中の切替が予測どおりに両立する

#### Acceptance Criteria

1. When デバッグセッションが attach 引数 `sourcePresentation` 付きで開始されたとき, the Pasta デバッグバックエンド shall その値を初期提示モードとして適用する。
2. When 実行時トグルによる切替要求を受け取ったとき, the Pasta デバッグバックエンド shall 初期提示モード指定を上書きして新しいモードを適用する。
3. While 実行時トグルで上書きされた状態であるとき, the Pasta デバッグバックエンド shall 以後の提示で上書き後のモードを採用する。
4. Where attach 引数 `sourcePresentation` が指定されていない場合, the Pasta デバッグバックエンド shall 既存の初期解決（環境変数 `PASTA_DEBUG_SOURCE_MODE` > `pasta.toml` の `present_as` > 既定 `.pasta`）の結果を初期提示モードとし, 実行時トグルでそれを上書きできるようにする。

### Requirement 5: 提示モードとステップ粒度の整合

**Objective:** As a pasta ゴースト作者, I want 提示モードに応じたステップ粒度でステップ実行できる, so that 表示中のソース単位で一貫してステップを進められる

#### Acceptance Criteria

1. While 提示モードが `.pasta` であるとき, the Pasta デバッグセッション shall `.pasta` 粒度でステップ実行する。
2. While 提示モードが `.lua` であるとき, the Pasta デバッグセッション shall `.lua` 粒度でステップ実行する。
3. When 停止中に提示モードが切り替わったとき, the Pasta デバッグセッション shall 以後のステップ操作を切替後の粒度で実行する。
4. When コルーチンを跨ぐ実行で提示モードを切り替えたとき, the Pasta デバッグセッション shall 切替後の粒度でステップ実行を継続する。

### Requirement 6: 既存挙動への無回帰

**Objective:** As a pasta ゴースト作者, I want 実行時トグルの追加が既存のデバッグ挙動を壊さない, so that これまでの `.pasta`/`.lua` デバッグ運用を安心して続けられる

#### Acceptance Criteria

1. While デバッグセッション中に実行時トグルを一度も使用していないとき, the Pasta デバッグバックエンド shall 初期解決どおりの提示モードで従来どおり動作する。
2. While デバッグ機能が無効（OFF 経路）であるとき, the Pasta ランタイム shall 実行時トグルに関わる処理を一切実行せず, デバッグ無効時の挙動を不変に保つ。
3. When `.pasta` 行にブレークポイントを張ったまま提示モードを切り替えたとき, the Pasta デバッグバックエンド shall そのブレークポイントを引き続き有効に保ち, 切替後のモードに応じて停止を提示する。
4. The pasta VSCode 拡張 shall 既存の attach 接続・診断・構文ハイライト等の機能を, トグル機能の追加によって損なわない。

### Requirement 7: 実 DAP-over-TCP の End-to-End 検証

**Objective:** As a pasta ゴースト作者, I want 「`.pasta` ブレークのまま `.lua` 表示」を含む実セッションが検証されている, so that 実運用と同じ経路でトグルが確実に機能すると信頼できる

#### Acceptance Criteria

1. The Pasta デバッグ機能 shall 実 DAP-over-TCP セッションで, `.pasta` 行に張ったブレークポイントで停止した状態から `.lua` 提示へ切り替え, 停止位置・スタックトレースが `.lua` 座標で提示されることを検証する。
2. The Pasta デバッグ機能 shall 実 DAP-over-TCP セッションで, `.lua` 提示から `.pasta` 提示へ切り替え, 提示が `.pasta` 座標へ戻ることを検証する。
3. When 実 DAP-over-TCP セッションで提示モードを切り替えたとき, the Pasta デバッグ機能 shall `.pasta` 行に張ったブレークポイントが切替の前後で有効であり続けることを検証する。

### Requirement 8: マニュアルの更新

**Objective:** As a pasta ゴースト作者, I want マニュアルのデバッグ章が実行時トグル手順を反映している, so that 切替操作の使い方を権威あるドキュメントで確認できる

#### Acceptance Criteria

1. The pasta 利用者マニュアル shall デバッグ章の提示モード切替の節を, デバッグセッション中に VSCode のコマンド／ボタンから提示モードを切り替える手順を含む内容へ更新する。
2. The pasta 利用者マニュアル shall 実行時トグルと既存の初期モード指定（attach 引数 `sourcePresentation` / 環境変数 / `pasta.toml`）との関係（初期値を実行時トグルで上書きできること）を説明する。
