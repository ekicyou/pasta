# Requirements Document

## Project Description (Input)
エンジンが SHIORI スレッドに束縛され、自前の時計を持たない。これが SHIORI(pull) とノベルゲーム(push/常駐) の整合を阻み、任意シーンキックの土台を欠く。`unsafe impl Send` ＋ Mutex ハックも温存している。さくらスクリプト描画という SHIORI 固有の出力形式がエンジンコアに焼き込まれているため、宿主差し替えができない。

本仕様 `pasta-actor-runtime` は、先行 PoC 仕様 `pasta-actor-feasibility`（判定 GO+）の結論を着手前提として、pasta エンジンを「自前スレッドのアクター化」する本番リファクタを行う。具体的には、(a) 宿主非依存エンジンコア（`pasta_lua`）が presentation event stream を出力する契約を確立し、(b) さくらスクリプト描画を `pasta_shiori`（アダプタ）へ移設し、(c) VM を所有するアクタースレッド（`pasta_shiori` が `wintf_winmsg_executor` で所有）へ VM を pin し、(d) SHIORI event を CH（チャネル）で marshaling（GET/NOTIFY/drop→204）し、(e) `unsafe impl Send` ハックを解消する。**外部 SHIORI 挙動はバイト不変**（純内部リファクタ）であり、全既存テストは回帰不変でなければならない。

## Introduction

本仕様は pasta エンジンの内部アーキテクチャを「SHIORI スレッド束縛の反応専用エンジン」から「自前スレッドを持つアクターモデルエンジン」へ転換する**振る舞いバイト不変の内部リファクタ**である。利用者（ゴースト／SSP ホスト）から観測される SHIORI 応答は、本リファクタ前後でバイト単位で同一でなければならない。リファクタの目的はあくまで内部構造の健全化（宿主非依存コア、VM の安全な pin、`unsafe impl Send` 解消、出力形式のアダプタ移設）であり、新しいユーザー可視挙動は一切導入しない。

本仕様は先行 PoC 仕様 `pasta-actor-feasibility`（判定 GO+）が実証した方式（executor 上 `!Send` VM ホスト、reload clean teardown、GET=block-on-reply／NOTIFY=即 204／drop→204 ガード、executor 駆動下のコルーチン生存）を本番化する。リファクタは「特性化テスト先行・1 抽出=1 検証=1 コミットの revert 可能な小ステップ」で進め、検証は速度より優先する。

## Boundary Context

- **In scope（本仕様が責任を持つ範囲）**:
  - presentation event stream 契約の定義（コア↔アダプタ境界をマーカー列として確定）
  - さくらスクリプト描画の `pasta_lua`（コア）→ `pasta_shiori`（アダプタ）への移設
  - VM を所有するアクタースレッドの本番導入と VM pin（`wintf_winmsg_executor` を `pasta_shiori` 側に閉じ込め）
  - CH（チャネル）marshaling（GET=応答付き／NOTIFY=義務なし即 204／drop→204 ガード）
  - 単一直列キューによる全 VM アクセスの順序保存・データ競合排除
  - reload（unload→再ロード）teardown の本番化（clean teardown・リーク／枯渇不在）
  - `unsafe impl Send` / `unsafe impl Sync` ハックの解消
  - 全既存テストの回帰不変維持
- **Out of scope（本仕様が扱わない範囲）**:
  - talk FIFO・`Status: talking` gate・即時 preempt・キック transport（後続仕様 `pasta-scene-kick`）
  - SSTP 出力、`*.pasta` 編集ウィンドウ、`pasta_novel` アダプタ
  - トーク／応答セマンティクスの変更（非同期トーク等の新しいユーザー可視挙動）
  - 新しいユーザー可視挙動全般（本仕様は挙動バイト不変）
  - 実機 SSP に対する絶対性能保証（PoC から申し送られた閾値候補を初期値として採用するに留める）
- **Adjacent expectations（隣接仕様・システムへの期待）**:
  - 上流 `pasta-actor-feasibility`（GO+）が確定した方式・前提結論を着手前提とする
  - 下流 `pasta-scene-kick` が、本仕様で確立した presentation event stream・アクタースレッド・marshaling を土台として利用する
  - 既存の yield/resume 機構（`STORE.co_scene`・`resume_until_valid`・`CALLBACK`）は意味論を変えずに維持する（Lua コルーチンモデルのまま）

## Requirements

### Requirement 1: 外部 SHIORI 挙動のバイト不変

**Objective:** ゴースト作者／SSP ホストとして、内部アーキテクチャがリファクタされても観測される SHIORI 応答が一切変わらないことを保証したい。これにより既存ゴーストが無改修で動作し続ける。

#### Acceptance Criteria
1. When 同一の SHIORI リクエスト列（同一の固定時刻・同一の reference を含む）がリファクタ前後のエンジンへ送られたとき, the pasta SHIORI エンジン shall リファクタ前と**バイト単位で同一**の SHIORI 応答列を返す。
2. The pasta SHIORI エンジン shall 既存の全テストスイートを改変なし（テスト期待値の変更なし）で回帰不変に通過する。
3. While `actor-poc` の使い捨て検証 feature が無効なリリースビルドにおいて, the pasta SHIORI エンジン shall リファクタ後も外部から観測可能な応答バイト列を変えない。
4. If リファクタのいずれかのステップで既存テストが失敗するか応答バイト列が変化したとき, then the 開発プロセス shall そのステップを完了とみなさず、原因を解消するか当該ステップを revert する。

### Requirement 2: presentation event stream 契約（宿主非依存コア出力）

**Objective:** pasta 開発者として、エンジンコアが SHIORI 固有の出力形式（さくらスクリプト）に依存せず、宿主非依存の presentation event（talk ライン／アクター切替／wait／choice 等のマーカー）を出力する契約を確立したい。これにより宿主（SHIORI／将来のノベルゲーム）を差し替え可能にする。

#### Acceptance Criteria
1. The エンジンコア（宿主非依存層） shall シーン実行の出力を、宿主非依存の presentation event のマーカー列（talk ライン・アクター切替・wait・choice 等）として表現する。
2. When シーンが talk ラインやアクター切替や wait や choice を出力したとき, the エンジンコア shall 当該出力を SHIORI 固有のさくらスクリプト文字列としてではなく、宿主非依存マーカーとして presentation event stream に載せる。
3. The エンジンコア shall presentation event stream の生成にあたり、特定宿主（SHIORI）の出力形式や executor 選択に依存しない。
4. Where presentation event stream の契約が確立されたとき, the アダプタ層（`pasta_shiori`） shall 当該マーカー列を消費して宿主固有の最終出力（さくらスクリプト文字列）へ変換する責務を負う。
5. The presentation event stream 契約 shall 設計哲学「UI 独立性: Wait/Sync はマーカーのみ」を出力全体へ一貫して適用する。
6. The presentation event stream 契約 shall マーカー種別の追加（将来のノベルゲーム宿主等が要する push/常駐系マーカー）を**既存コア・既存アダプタ・既存テストの破壊的変更なしに**受け入れられる拡張可能な構造とする。
7. While 本仕様の実装範囲において, the エンジンコア shall マーカーの**実装は最小集合（talk ライン・アクター切替・wait・choice）に限定**し、最小集合外のマーカーの具体実装は導入しない（拡張は構造的余地として確保するに留める）。

> **決定（ディスカッション #1, 2026-06-22）**: マーカー粒度は「実装は最小・設計は将来対応可能」（B 寄りの C）に確定。実装するマーカーは最小集合（talk / アクター切替 / wait / choice）に限定し、現状の Lua 側トークン（`{type="talk", actor, text}` 等）と既存さくらスクリプト出力からバイト不変で逆算する。ただしマーカー型体系は将来宿主（ノベルゲーム push/常駐）のマーカーを破壊的変更なしに追加できる拡張アーキテクチャとして設計する（単なる `#[non_exhaustive]` 注釈ではなく、コア↔アダプタ境界 API が新マーカー追加に耐える構造であること）。最小集合外のマーカーの実装は本仕様の対象外。具体スキーマ・型表現の確定は設計フェーズ（RN2）に委ねる。

### Requirement 3: さくらスクリプト描画のアダプタ移設

**Objective:** pasta 開発者として、SHIORI 固有のさくらスクリプト描画ロジックをエンジンコア（`pasta_lua`）からアダプタ層（`pasta_shiori`）へ移設したい。これによりコアを宿主非依存に保ち、宿主差し替えを可能にする。

#### Acceptance Criteria
1. The pasta システム shall さくらスクリプト描画（wait 挿入・トークナイズ・行分割等のレンダリング）の責務をアダプタ層（`pasta_shiori`）に配置する。
2. When アダプタ層が presentation event stream を受け取ったとき, the アダプタ層 shall それを SHIORI 固有のさくらスクリプト文字列へレンダリングする。
3. While さくらスクリプト描画を移設している間, the pasta システム shall レンダリング結果（最終的なさくらスクリプト文字列）を移設前とバイト不変に保つ。
4. The エンジンコア（`pasta_lua`） shall 移設後、SHIORI 固有のさくらスクリプト描画ロジックを宿主非依存コアの責務として保持しない。

> **assumption（移設の物理形態）**: さくらスクリプトのレンダリング実装（現 `crates/pasta_lua/src/sakura_script/`）と Lua への公開（現 `@pasta_sakura_script` モジュール登録、現 `BUILDER.build`／`sakura_builder.lua` 経由の呼び出し）が `pasta_shiori` 側のアダプタ責務として再配置される。Rust 実装コードを物理的に `pasta_shiori` crate へ移すか、`pasta_lua` 内に残しつつ「アダプタ責務」として論理的に隔離・登録経路をアダプタ起点にするかは設計フェーズで確定する（OPEN QUESTION Q2）。

### Requirement 4: VM を所有するアクタースレッドと VM pin

**Objective:** pasta 開発者として、`!Send` な Lua VM を専用アクタースレッドに pin し、全 VM アクセスを単一スレッドに閉じたい。これにより `mlua` の `!Send` 制約を構造的に遵守し、`unsafe` ハックなしでスレッド安全性を達成する。

#### Acceptance Criteria
1. The pasta システム shall `!Send` な Lua VM（`PastaLuaRuntime`）を専用のアクタースレッド上で生成・所有し、当該 VM をそのスレッドに pin する。
2. The Lua VM shall 生成元のアクタースレッドを越えて他スレッドへ移動・共有されない。
3. While アクタースレッドが稼働している間, the pasta システム shall VM への全アクセスをアクタースレッド経由でのみ行う。
4. The アクタースレッドの所有・executor 選択（`wintf_winmsg_executor`）の決定 shall アダプタ層（`pasta_shiori`）に閉じ込められ、エンジンコアの純度を損なわない。
5. When アクタースレッドが起動されたとき, the pasta システム shall VM が当該スレッド上で初期化・常駐し、SHIORI スレッドとは別スレッドであることを保証する。

### Requirement 5: CH（チャネル）marshaling — GET/NOTIFY/drop→204

**Objective:** SSP ホストとして、SHIORI リクエストの同期契約（GET は応答を要する、NOTIFY は応答不要）が、SHIORI スレッドとアクタースレッドの分離後も正しく守られることを期待する。これにより既存の同期挙動を保ったままアクター化する。

#### Acceptance Criteria
1. When SHIORI スレッドが GET リクエストを受け取ったとき, the marshaling 層 shall 応答経路（応答 tx）を付与したメッセージをアクタースレッドの単一直列キューへ enqueue し、応答受信までブロックして応答値を GET の戻り値として返す。
2. When SHIORI スレッドが NOTIFY リクエストを受け取ったとき, the marshaling 層 shall 応答義務なしのメッセージを enqueue し、アクタースレッドの完了を待たず即座に 204 No Content を返す。
3. If GET の応答が送信されないまま応答経路が drop（応答忘れ／処理失敗）されたとき, then the marshaling 層 shall 自動的に 204 No Content を返し、SHIORI スレッドを無限待機させない。
4. The GET 処理 shall アクタースレッドのブロック時間を短く保ち、エンジンはブロック待機ではなく yield により他処理を進められるようにする。
5. The GET/NOTIFY のメソッド判定および marshaling 分岐 shall 決定論的ロジックとしてアダプタ層（Rust 側）で完結し、シーン実行・コルーチン継続・callback 待機の意味論は Lua のまま保つ。
6. While アクタースレッドが応答可能でない異常状態（例: パニック巻き戻し）にある間, the marshaling 層 shall GET 呼び出し元を 204 No Content で終結させる（デッドロック経路を生じさせない）。

> **assumption（GET タイムアウト→204 フォールバック）**: GET block-on-reply に対し、PoC が申し送った GET タイムアウト→204 フォールバックを防御として設け、閾値候補 6.68ms を初期値として採用する。実機実測に基づく最終確定はディスカッション／設計フェーズ以降に委ねる（OPEN QUESTION Q3）。
> **assumption（panic=abort 前提）**: drop→204 ガードは unwind に依存する。リリースビルドは `panic=abort` のため、panic 経路での drop→204 保証は test/unwind プロファイルでのみ厳密に成立する（PoC 申し送り）。本仕様はこの前提を維持し、リリースビルドでの panic 時挙動の扱いはディスカッションで確認する（OPEN QUESTION Q4）。

### Requirement 6: 単一直列キューによる順序保存とデータ競合排除

**Objective:** pasta 開発者として、全 VM アクセスを単一の直列キュー（mailbox）で処理し、データ競合をゼロにしつつ処理順序を保存したい。これによりアクター化後もリエントランシー順序を構造的に確定する。

#### Acceptance Criteria
1. The pasta システム shall アクタースレッドへ投入される全メッセージ（GET／NOTIFY 等）を単一の直列キューで処理する。
2. While 複数のメッセージがキューに存在する間, the アクタースレッド shall それらを enqueue された順序で逐次（直列）処理する。
3. The pasta システム shall 単一直列キュー化により、VM 状態に対する同時並行アクセス（データ競合）を発生させない。
4. When 複数の SHIORI イベントが時間的に近接して到着したとき, the pasta システム shall それらの VM への適用順序を直列キューの順序として一意に確定する。

### Requirement 7: reload teardown の本番化

**Objective:** SSP ホストとして、ゴーストの reload（unload→再ロード）が繰り返されてもリソースが漏れず、再起動が clean に成立することを期待する。これにより長時間運用・頻繁な reload でも安定動作する。

#### Acceptance Criteria
1. When SHIORI の unload が呼ばれたとき, the pasta システム shall アクタースレッド・VM・関連リソース（メッセージ専用ウィンドウ・チャネル等）を clean に teardown する。
2. When unload に続いて再 load が呼ばれたとき, the pasta システム shall アクタースレッドと VM を新規に再生成し、正常に稼働させる。
3. While reload サイクルを繰り返している間, the pasta システム shall スレッドハンドル・ソケット／ポート・メモリのリーク／枯渇を発生させない。
4. The teardown 処理 shall 二重 join／二重解放を防止し（shutdown フラグ→wake→join の順序）、アクタースレッドの終了を確実に待ち合わせる。
5. If teardown の途中で異常（join 失敗・リソース解放漏れ）が生じたとき, then the pasta システム shall その異常を記録し、ホスト（SSP）プロセスを巻き込んで落とさない。

### Requirement 8: `unsafe impl Send` / `unsafe impl Sync` ハックの解消

**Objective:** pasta 開発者として、`PastaShiori` に付与された `unsafe impl Send` / `unsafe impl Sync` と `Arc<Mutex<Option<...>>>` による VM 束縛ハックを解消したい。これにより `!Send` 制約をアーキテクチャ（VM pin）で満たし、unsafe な健全性仮定を排除する。

#### Acceptance Criteria
1. The pasta システム shall `!Send` な Lua VM をアクタースレッドへ pin する設計により、`PastaShiori` 相当の型へ手動の `unsafe impl Send` / `unsafe impl Sync` を付与しない。
2. When VM へのアクセスが必要なとき, the pasta システム shall `Arc<Mutex>` ＋ `unsafe impl Send` によるスレッド束縛ではなく、アクタースレッドへのメッセージ送信経路を通じてアクセスする。
3. The リファクタ後のコード shall VM のスレッド安全性を `unsafe` の健全性仮定ではなく、VM がアクタースレッドを越えないという構造的不変条件によって担保する。
4. While `unsafe impl Send` ハックを解消する間, the pasta システム shall 外部 SHIORI 挙動をバイト不変に保つ（Requirement 1 と整合）。

### Requirement 9: 既存コルーチン／callback 意味論の維持

**Objective:** ゴースト作者として、トーク継続（`STORE.co_scene`）や非同期 callback（`CALLBACK.pending`）を用いた既存ゴーストの挙動が、駆動主体が executor へ移った後も変わらないことを期待する。これにより既存スクリプトが無改修で動作する。

#### Acceptance Criteria
1. While 駆動主体がホスト tick からアクタースレッド／executor へ移った後, the pasta システム shall 既存のシーンコルーチン（`STORE.co_scene`・`resume_until_valid`）を中断地点から正しく resume する。
2. When 非同期 callback の応答契機が到来したとき, the pasta システム shall 既存の `CALLBACK` 機構（`pending`・`consume_staged`・`sweep`・`OnSecondChange` 駆動）により保留コルーチンを解決・継続する。
3. The pasta システム shall シーン実行・コルーチン継続・callback 待機の意味論を Lua コルーチンモデルのまま保ち、Rust 側へ意味論を移さない（marshaling／dispatch の殻のみ Rust 化）。
4. When 既存の `*.pasta` / `*.lua` スクリプトが駆動されたとき, the pasta システム shall スクリプト無改変のまま既存挙動を再現する。
