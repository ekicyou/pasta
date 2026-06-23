# Requirements Document

## Project Description (Input)
現状の scene-kick（`pasta-scene-kick`、完了済み）は、VSCode のコマンドパレット／デバッグツールバーから作者がシーン名を手入力（`showInputBox`）してキックする設計になっている。しかしシーン名は `.pasta` の `＊`（global）/`・`（local）宣言を `pasta_core::SceneRegistry` がサニタイズして生成する内部識別子（例: `会話_1`）であり、作者が知り得る情報ではない。「唯一持っていない情報（シーン名）を、唯一それを知らない者（作者）に要求している」状態であり、動線として破綻している。

本仕様は、作者が `.pasta` を開いて任意のシーン位置で右クリックし、コンテキストメニュー最上段の「▶ シーンを実行」を選ぶだけで、即時にそのシーンがライブ SSP で再生される動線を確立する。エディタは位置 (uri + line) のみを送り、実行中エンジンがロード済みソースマップで位置→シーンを確定し、既存 kick backend へ取り次ぐ（案 B・エンジン解決）。シーン名の手入力は不要・廃止とし、これがシーンキックの唯一の動線となる。旧 `pasta.debug.playScene`（`showInputBox`）コマンドは存在自体を廃止（リジェクト）する。

## Introduction
本機能は、ゴースト作者の開発体験を中核に据えたシーンキック動線の刷新である。作者は実装上の内部識別子（サニタイズ済みシーン名）を知らないため、シーン名入力を要求する既存動線は破綻している。本機能は「作者が知っている情報＝編集中の `.pasta` ファイル上のカーソル位置」を起点とし、行→シーンの確定を、トランスパイラと同一の `SceneRegistry` 由来情報を持つ実行中エンジン側に委ねる。これにより、作者は位置を指すだけでシーンを即時再生でき、内部識別子の知識を一切要求されない。

## Boundary Context
- **In scope（本機能が責任を持つ振る舞い）**:
  - `.pasta` エディタのネイティブ右クリックコンテキストメニュー最上段に「▶ シーンを実行」項目（新コマンド `pasta.runSceneAtCursor`（仮称））を提供する。
  - 当該コマンドがアクティブエディタのカーソル位置 (uri + line) を取得し、新トランスポート経由でエンジンへ送る。
  - エンジンがロード済みソースマップ／シーン同一性索引を用いて (uri, line) からシーンを確定し、既存 kick backend へ取り次ぐ。
  - ソースマップへの「シーン同一性索引」（`.pasta` の (file, 行範囲) → シーン識別子）の追加と、トランスパイル時の `SceneRegistry` 由来シーン位置情報の受け渡し。
  - 旧 `pasta.debug.playScene`（シーン名入力）コマンドおよび `showInputBox` フローの廃止。
  - 「SHIORIリロード」コマンド（右クリックメニュー＋デバッグツールバー）による `\![reload,shiori]` 送出と、リロードで切れたデバッグセッションの自動再アタッチ（リロード指示→数秒待機→`vscode.debug.startDebugging` で再アタッチ）。
- **Out of scope（本機能が再設計しない／別境界の振る舞い）**:
  - kick 実行本体のセマンティクス（co_scene 設置・preempt-and-abort・初回ビートのワンショット抑制突破・OnSecondChange シーン継続）の再設計。`pasta-scene-kick` を流用し、振る舞いは継承する。
  - LSP（エディタ側）でのシーン解決（案 A・不採用）。
  - 第三者拡張 Code Runner（`formulahendry.code-runner`）連携（不採用）。
  - SSTP・ライブ SSP 以外の出力先・別プレビュー画面。
  - 将来の `*.pasta` 編集ウィンドウからのキック（別境界・`pasta-authoring-window`）。
- **Adjacent expectations（隣接仕様・前提）**:
  - kick backend は `pasta-scene-kick` が提供する。本機能はシーン名（または確定済みシーン）を渡す既存取次点を再利用する。
  - ソースマップ基盤は `pasta-source-map` が提供する。本機能はそこへシーン同一性索引を追加する。
  - DAP custom request チャネルは `pasta-vscode-lua-debug` が提供する。本機能は新リクエストを同チャネルに追加する（`pasta-debug-lua-view-toggle` の前例に倣う）。
  - 解決はロード済み辞書（実行中エンジンが保持する状態）を権威とする。エンジン未接続（デバッグセッション未確立）時は再生できない。

## Requirements

### Requirement 1: カーソル位置からのシーン実行動線（作者向けエントリポイント）
**Objective:** ゴースト作者として、`.pasta` の任意のシーン位置で右クリックして即座にそのシーンを再生したい。これにより、内部識別子であるシーン名を知らなくてもシーンキックができる。

#### Acceptance Criteria
1. While `.pasta` 言語のファイルがアクティブエディタで開かれている, when 作者がエディタ上で右クリックする, the VSCode Pasta 拡張 shall コンテキストメニュー最上段（ナビゲーション群の先頭）に「▶ シーンを実行」項目を表示する。
2. When 作者が「▶ シーンを実行」項目を選択する, the VSCode Pasta 拡張 shall アクティブエディタの現在のカーソル位置（対象ファイルの uri と行番号）を取得する。
3. When カーソル位置の取得に成功する, the VSCode Pasta 拡張 shall 取得した位置 (uri, line) を実行中エンジンへ新トランスポートで送り、シーン実行を要求する。
4. The VSCode Pasta 拡張 shall シーン実行要求にあたり作者へシーン名の手入力を一切要求しない。
5. When エンジンが位置からシーンを確定して再生を開始する, the Pasta エンジン shall 確定したシーンをライブ SSP 上で再生する。

### Requirement 2: 位置→シーン解決（エンジン権威・案 B）
**Objective:** ゴースト作者として、編集中バッファとロード済み実態がずれても、実際に動いているゴースト基準で「カーソル下のシーン」を確定してほしい。これにより、エディタ側の重複実装による名前ドリフトを避け、確実なシーン解決を得られる。

#### Acceptance Criteria
1. When エンジンが位置 (uri, line) を受け取る, the Pasta エンジン shall ロード済みソースマップ／シーン同一性索引を用いて当該位置が属するシーンを確定する。シーンの範囲は「宣言行〜次の同レベル以上のシーン宣言の直前」と定義する。
2. While カーソル位置が複数のシーン（global と内包する local）の範囲に該当する, the Pasta エンジン shall 最内シーン（local 優先）を選択する（包含による確定を後方フォールバック（5項）に優先する）。
3. When シーンが確定する, the Pasta エンジン shall 確定したシーンを既存 kick backend（`pasta-scene-kick`）の取次点へ渡す。
4. The Pasta エンジン shall 位置→シーンの確定を、エディタ側ではなくエンジン側（トランスパイラと同一の `SceneRegistry` 由来情報を保持する側）で行う。
5. If 受け取った位置がどのシーンの範囲にも属さない（最初のシーンより上のヘッダ部・シーン群の外の領域等）, then the Pasta エンジン shall クリック行と同じか下方（より大きい行番号）にある最も近い有効なシーン宣言のシーンを選択する（後方フォールバック）。
6. If 後方フォールバックの対象となる有効なシーンが下方に存在しない（最後のシーンより後ろ等）, then the Pasta エンジン shall シーン未検出として扱い、再生を行わず、作者へ「カーソル下にシーンがありません」を提示する。

### Requirement 3: シーン同一性索引（ソースマップ拡張・ビルド側）
**Objective:** 機能保守者として、`.pasta` の (file, 行範囲) からシーン識別子を逆引きできる索引をソースマップに持たせたい。これにより、エンジンが位置からシーンを確定できる。

#### Acceptance Criteria
1. When トランスパイラが `.pasta` をトランスパイルする, the Pasta トランスパイラ shall 各シーンの `.pasta` 上の (ファイル, 行範囲) と、対応するシーン識別子（`SceneRegistry` 由来）の対応をソースマップへ記録する。
2. The Pasta ソースマップ shall (ファイル, 行) を入力としてそれが属するシーン識別子を返す逆引き索引を提供する。
3. The Pasta トランスパイラ shall シーン同一性索引に記録するシーン識別子を、`SceneRegistry` がサニタイズして生成した内部識別子と一致させる（同一の識別子規則を二重実装しない）。
4. Where 既存ソースマップが行マッピング（`.pasta` 行 ↔ 生成 `.lua` 行）のみを保持している, the Pasta ソースマップ shall 既存の行マッピング機能（`resolve_lua_to_pasta` / `resolve_pasta_to_lua`）を後方非破壊で維持しつつシーン同一性索引を追加する。

### Requirement 4: 位置ベース実行用トランスポート
**Objective:** 機能保守者として、位置 (uri, line) を送る専用トランスポートを既存デバッグチャネル上に用意したい。これにより、エディタとエンジン間で位置ベースのシーン実行要求を授受できる。

#### Acceptance Criteria
1. The VSCode Pasta 拡張と Pasta エンジン shall 位置 (uri, line) を引数とするシーン実行要求を、既存のデバッグ用カスタムリクエストチャネル上で授受する。
2. When エンジンが当該要求を受理しシーン確定と再生開始に成功する, the Pasta エンジン shall 要求元へ成功応答を返す。
3. If エンジンが当該要求を受理したがシーンを確定できない, then the Pasta エンジン shall 要求元へエラー応答（理由を含む）を返す。
4. The Pasta エンジン shall シーン名文字列ではなく位置 (uri, line) を当該要求の入力として受け取る。

### Requirement 5: 旧シーン名入力動線の廃止
**Objective:** ゴースト作者として、破綻していたシーン名手入力動線を完全に取り除き、混乱の元を残さないでほしい。これにより、シーンキックの動線が位置ベース一本に統一される。

#### Acceptance Criteria
1. The VSCode Pasta 拡張 shall 旧コマンド `pasta.debug.playScene`（シーン名を `showInputBox` で入力する動線）をコマンド・メニュー貢献から削除し、提供しない。
2. The VSCode Pasta 拡張 shall コマンドパレットおよびデバッグツールバーからシーン名入力によるキック手段を提供しない。
3. The Pasta システム shall カーソル位置ベースの動線（Requirement 1）を、シーンキックの唯一の作者向け動線とする。
4. The VSCode Pasta 拡張と Pasta エンジン shall 旧シーン名ベースの外部トランスポート（DAP custom request `pasta/playScene`{scene 名}）を撤去し、外部へ公開するシーン実行口を位置ベース（Requirement 4 の位置ベース要求）一本に統一する。
5. The Pasta エンジン shall kick backend の内部取次点（確定済みシーン識別子を受け取り co_scene に据える `pasta-scene-kick` 由来のエントリ）を保持し、位置→シーン解決後の内部呼び出しとして再利用する（外部トランスポートではないため作者には露出しない）。

### Requirement 6: メニュー表示条件とセッション未接続時の挙動
**Objective:** ゴースト作者として、いつコマンドが使えるか・使えないときにどうなるかを明確に知りたい。これにより、デバッグセッション未接続でも迷わず操作できる。

#### Acceptance Criteria
1. The VSCode Pasta 拡張 shall コンテキストメニュー項目「▶ シーンを実行」を、対象ファイルが `.pasta` 言語である場合に常時表示する（デバッグセッションの接続有無に依存しない）。
2. While Pasta デバッグセッションが接続されていない, when 作者が「▶ シーンを実行」を選択する, the VSCode Pasta 拡張 shall 再生要求を送らず、「Pasta デバッグセッションが接続されていません」の警告と、デバッグセッションの開始（起動／アタッチ）へ誘導するアクションを作者へ提示する。
3. When 作者がセッション未接続時の誘導アクション（デバッグ開始）を選択する, the VSCode Pasta 拡張 shall Pasta デバッグセッションの開始（起動／アタッチ）を起動する。なお当該アクションの具体的な起動手段（`launch.json` 構成の参照・自動アタッチ可否）は設計フェーズで確定する。
4. If シーン実行要求がエンジンからエラー応答を受け取る, then the VSCode Pasta 拡張 shall 失敗理由を作者へ提示する。

### Requirement 7: ロード済み辞書とディスク内容の不整合（staleness）時の挙動
**Objective:** ゴースト作者として、`.pasta` を編集・保存したがエンジンが未リロードの状態でキックしたときに、誤ったシーンが再生されたり黙って失敗したりしないでほしい。これにより、編集途中でも結果の信頼性を保てる。

#### Acceptance Criteria
1. The Pasta エンジン shall 位置→シーンの確定をロード済み辞書（実行中エンジンが保持するソースマップ）に基づいて行う。
2. While ロード済み内容がディスク上の `.pasta`（またはエディタ上の編集中バッファ）と一致しない可能性がある, when 作者がシーン実行を要求する, the Pasta システム shall 黙ってロード済みデータで解決せず、作者へリロードを促し、「SHIORIリロード」コマンド（Requirement 9）の使用へ誘導する。
3. The Pasta システム shall kick 経路の内部で自動的に `\![reload,shiori]` を送出しない（リロードはデバッグセッションのデタッチを伴うため、独立した「SHIORIリロード」コマンド（Requirement 9）に一本化する）。
4. Where エディタバッファに未保存の編集（dirty）が存在する, the VSCode Pasta 拡張 shall リロードはディスク内容を読むため未保存の変更が反映されない旨を作者へ示し、保存を促す。
5. The Pasta システム shall staleness の検知方式（保存状態・mtime・ハッシュ照合等）を設計フェーズで確定する。
6. If ロード済みソースマップで位置がどのシーンにも解決できない（後方フォールバック対象も存在しない）, then the Pasta システム shall シーン未検出として扱い（Requirement 2.6 に従う）、誤ったシーンの再生を行わない。

### Requirement 8: 既存キック制約の継承
**Objective:** 機能保守者として、本機能が既存のランタイム制約・kick セマンティクスを破らないことを保証したい。これにより、SHIORI 整合と応答性を維持できる。

#### Acceptance Criteria
1. The Pasta システム shall 位置ベースキックの実行セマンティクス（co_scene 設置・preempt-and-abort・初回ビートのワンショット抑制突破・OnSecondChange シーン継続）を `pasta-scene-kick` から継承し、変更しない。
2. The Pasta システム shall SHIORI/3.0 整合および OnSecondChange を 1 秒以内に保つ既存制約を維持する。
3. The Pasta システム shall SHIORI GET 要求の処理ブロックを短く保つ既存制約を維持する。

### Requirement 9: SHIORI リロードコマンド（手動・デタッチ自動復帰）
**Objective:** ゴースト作者として、staleness を解消するために SHIORI を手動でリロードし、リロードで切れたデバッグ接続を自動で復帰してほしい。これにより、`.pasta` 編集後も再アタッチの手間なくシーン実行へ戻れる。

#### Acceptance Criteria
1. The VSCode Pasta 拡張 shall `.pasta` エディタの右クリックコンテキストメニューおよびデバッグツールバーの両方に「SHIORIリロード」コマンドを提供する。
2. When 作者が「SHIORIリロード」を実行する, the Pasta システム shall SSP へ `\![reload,shiori]`（SHIORI のみ再読み込み・非同期）を送出して SHIORI の再読み込みを起動する。
3. When `\![reload,shiori]` の送出によりデバッグセッションがデタッチされる, the VSCode Pasta 拡張 shall 一定時間待機した後、`vscode.debug.startDebugging` を用いて `type: 'pasta'` のアタッチ構成でデバッグセッションを自動的に再アタッチする。
4. The VSCode Pasta 拡張 shall 再アタッチの基本動作を「リロード指示 → 数秒待機 → 自動再アタッチ」とし、待機時間・リトライ・タイムアウト方式、および再アタッチ完了後にシーンキックを自動再実行するか否かを設計フェーズで確定する。
5. The VSCode Pasta 拡張 shall 「SHIORIリロード」コマンドの表示・有効化条件を設計フェーズで確定する（リロードは接続中のデバッグセッションに対する操作である点を考慮する）。
