# Requirements Document

## Project Description (Input)
エンジンが SHIORI スレッドに束縛され、自前の時計を持たない。これが SHIORI(pull) とノベルゲーム(push/常駐) の整合を阻み、任意シーンキックの土台を欠く。`unsafe impl Send` ＋ Mutex ハックも温存している。さくらスクリプト描画という SHIORI 固有の出力形式がエンジンコアに焼き込まれているため、宿主差し替えができない。

本仕様 `pasta-actor-runtime` は、先行 PoC 仕様 `pasta-actor-feasibility`（判定 GO+）の結論を着手前提として、pasta エンジンを「自前スレッドのアクター化」する本番リファクタを行う。具体的には、(a) 宿主非依存エンジンコア（`pasta_lua`）が presentation event stream を出力する契約を確立し、(b) さくらスクリプト描画を `pasta_shiori`（アダプタ）へ移設し、(c) VM を所有するアクタースレッド（`pasta_shiori` が `wintf_winmsg_executor` で所有）へ VM を pin し、(d) SHIORI event を CH（チャネル）で marshaling（GET/NOTIFY/drop→204）し、(e) `unsafe impl Send` ハックを解消する。**外部 SHIORI 挙動はバイト不変**（純内部リファクタ）であり、全既存テストは回帰不変でなければならない。

## Introduction

本仕様は pasta エンジンの内部アーキテクチャを「SHIORI スレッド束縛の反応専用エンジン」から「自前スレッドを持つアクターモデルエンジン」へ転換する**振る舞いバイト不変の内部リファクタ**である。利用者（ゴースト／SSP ホスト）から観測される SHIORI 応答は、本リファクタ前後でバイト単位で同一でなければならない。リファクタの目的はあくまで内部構造の健全化（宿主非依存コア、VM の安全な pin、`unsafe impl Send` 解消、出力形式のアダプタ移設）であり、新しいユーザー可視挙動は一切導入しない。

本仕様は先行 PoC 仕様 `pasta-actor-feasibility`（判定 GO+）が実証した方式（executor 上 `!Send` VM ホスト、reload clean teardown、GET=block-on-reply／NOTIFY=即 204／drop→204 ガード、executor 駆動下のコルーチン生存）を本番化する。リファクタは「特性化テスト先行・1 抽出=1 検証=1 コミットの revert 可能な小ステップ」で進め、検証は速度より優先する。

## Boundary Context

- **In scope（本仕様が責任を持つ範囲）**:
  - presentation event stream 契約の定義（コア↔アダプタ境界をマーカー列として確定。デバッガ観測可能に保つ）
  - さくらスクリプト描画の**論理デカップリング**（レンダラのアダプタ注入化。**物理的なクレート移動は行わない**・VM 内レンダリングを維持）
  - **デバッグ容易性の保全**（アクタースレッド化後も `.pasta`/`.lua` ソースレベルデバッグが完全動作・Lua コードは pasta_lua に全集約）
  - VM を所有するアクタースレッドの本番導入と VM pin（`wintf_winmsg_executor` を `pasta_shiori` 側に閉じ込め）
  - CH（チャネル）marshaling（GET=応答付き／NOTIFY=義務なし即 204／drop→204 ガード）
  - 単一直列キューによる全 VM アクセスの順序保存・データ競合排除
  - reload（unload→再ロード）teardown の本番化（clean teardown・リーク／枯渇不在）
  - `unsafe impl Send` / `unsafe impl Sync` ハックの解消
  - PoC `actor_poc/` の**デバッグ資産（`sim_driver`／`mailbox`／`responder`／`coroutine_probe` 検証等）を本番テスト基盤へ昇格**し、feature-gated 使い捨て足場（`verdict.rs`・PoC scaffold・`actor-poc` feature gate）を**最終タスクで撤去**（痕跡ゼロ＝出荷バイト不変を正規化 sha で確認）
  - 全既存テストの回帰不変維持
- **Out of scope（本仕様が扱わない範囲）**:
  - talk FIFO・`Status: talking` gate・即時 preempt・キック transport（後続仕様 `pasta-scene-kick`）
  - SSTP 出力、`*.pasta` 編集ウィンドウ、`pasta_novel` アダプタ
  - **さくらスクリプト描画コード（Lua・Rust）の物理的なクレート移動**（Lua 集約死守・デッドコード除去前提により不要と判断。将来 `pasta_novel` が要すれば別仕様で再検討）
  - **VM 外（post-VM）レンダリング方式**（デバッグ容易性を劣化させるため採らない）
  - debug backend のアクタークライアント化（後続仕様 `pasta-scene-kick`。本仕様は既存デバッグの**動作保全**のみを負う）
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
8. The presentation event stream（マーカー列） shall VM 内に保持され、`.pasta`/`.lua` デバッグ時にデバッガから観測可能な経路（source-map されたフロー内）に留まる（デバッグ容易性の保全。Requirement 10 と整合）。

> **決定（ディスカッション #1, 2026-06-22）**: マーカー粒度は「実装は最小・設計は将来対応可能」（B 寄りの C）に確定。実装するマーカーは最小集合（talk / アクター切替 / wait / choice）に限定し、現状の Lua 側トークン（`{type="talk", actor, text}` 等）と既存さくらスクリプト出力からバイト不変で逆算する。ただしマーカー型体系は将来宿主（ノベルゲーム push/常駐）のマーカーを破壊的変更なしに追加できる拡張アーキテクチャとして設計する（単なる `#[non_exhaustive]` 注釈ではなく、コア↔アダプタ境界 API が新マーカー追加に耐える構造であること）。最小集合外のマーカーの実装は本仕様の対象外。具体スキーマ・型表現の確定は設計フェーズ（RN2）に委ねる。

### Requirement 3: さくらスクリプト描画の論理デカップリング（レンダラのアダプタ注入）

**Objective:** pasta 開発者として、SHIORI 固有のさくらスクリプト描画を「コアに焼き込まれた必須処理」から「アダプタが注入するレンダラ」へ論理的にデカップリングしたい。これによりコアを宿主非依存に保ちつつ、**デバッグ容易性を一切損なわず**（VM 内レンダリング維持・Lua 集約死守）、宿主差し替えを可能にする。

#### Acceptance Criteria
1. The pasta システム shall さくらスクリプト描画（wait 挿入・トークナイズ・行分割等のレンダリング）を、コアが無条件に保持・実行する処理ではなく、**アダプタ層（`pasta_shiori`）が注入するレンダラ**として位置づける（`@pasta_sakura_script` の登録をコア無条件起点ではなくアダプタ起点とする）。
2. When 宿主が SHIORI であるとき, the アダプタ層 shall さくらスクリプトレンダラを注入し、コアのトーク構築フローがそれを呼び出してさくらスクリプト文字列を得る。
3. While レンダラをデカップリングしている間, the pasta システム shall レンダリング結果（最終的なさくらスクリプト文字列）をデカップリング前とバイト不変に保つ。
4. The エンジンコア（`pasta_lua`） shall SHIORI 固有のさくらスクリプト描画を**宿主非依存コアの必須責務として保持しない**（レンダラはアダプタ注入であり、コアはレンダラ形の差し替え可能な接点のみを持つ）。物理的なコード配置はこの責務境界を規定しない。
5. The pasta システム shall さくらスクリプト描画コード（Lua・Rust とも） を `pasta_lua` 内に物理的に維持し、クレート間の物理移動を行わない（Lua 集約死守・デバッグ容易性保全。未使用レンダラはデッドコード除去に委ねる）。
6. While レンダラのデカップリング後も, the pasta システム shall トーク構築フロー（`sakura_builder.lua`／`BUILDER.build` 等）を VM 内・`.pasta`/`.lua` source-map されたデバッグ可能フロー内に維持する（VM 外レンダリングを採らない。Requirement 10 と整合）。

> **決定（ディスカッション #2, 2026-06-22）**: 「物理移設」案を撤回し、**論理デカップリング（レンダラのアダプタ注入・VM 内レンダリング維持・物理移動なし）**に確定。決定理由 = (1) デバッグ容易性が最優先であり、VM 外（post-VM）レンダリングは最終さくらスクリプト文字列とトーク組み立てを `.pasta` デバッグ可視範囲から外すため不可。(2) Lua コードを複数クレートに分散させるとソースマップ／DAP デバッグが崩壊するため Lua は `pasta_lua` に全集約。(3) Rust のデッドコード除去により、コアにさくらレンダラが物理的に在ること自体は汚染・肥大とならず、将来 `pasta_novel` ビルドでは未使用レンダラが除去される。宿主非依存は「レンダラのアダプタ注入（差し替え可能性）」で達成し、クレート配置を責務境界としない。レンダラ注入 IF の具体形・`TalkConfig` の受け渡し・登録経路のアダプタ起点化は設計フェーズ（RN3）に委ねる。

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
7. If GET の block-on-reply がタイムアウト閾値を超えてアクタースレッドの応答が得られないとき, then the marshaling 層 shall 204 No Content を返して SHIORI スレッドの待機を打ち切る（SSP を凍結させない）。このとき the pasta システム shall シーンコルーチン（`STORE.co_scene`）の中断状態を破棄せず生かし、後続の `OnSecondChange`（`resume_until_valid` 駆動）が同コルーチンを resume して処理を回復できるようにする。
8. The GET タイムアウト閾値 shall 通常運転（デバッガ非停止かつアクタースレッドが正常進行）における正規の GET 処理では発火しない値に設定し、通常経路の応答バイト列をバイト不変に保つ（Requirement 1 と整合）。タイムアウト発火は、デバッガ一時停止・異常ブロック等の非通常時に限る安全網とする。
9. While デバッガがブレークポイントで一時停止している間, the marshaling 層 shall GET タイムアウト→204 を抑止せず通常通り発動させ、SSP の応答性を維持する（停止中の 204 は次 tick の `OnSecondChange` で回復するため、デバッグ容易性を損なわない。Requirement 10 と整合）。
10. The アクタースレッド・marshaling・teardown の正常経路 shall 既存コードベースの panic 回避姿勢を継続し、fallible 操作を `Result` で扱う（`unwrap`／`expect`／境界外索引等の panic 経路を正常系に持ち込まない）ことで、リリースビルド（`panic=abort`）でのプロセス abort を構造的に誘発しない。
11. While drop→204 ガードが panic 経路の保険として働く場合, the pasta システム shall その厳密保証が unwind プロファイル（dev/test）に限られることを前提とし、リリース（`panic=abort`）では既存 `windows.rs` の `catch_unwind` 同様「到達不能の dev/test 向け保険」として扱う（リリースのビルドプロファイルは変更しない）。

> **決定（ディスカッション #3, 2026-06-22）**: GET timeout→204 フォールバックを**本仕様で実装**し、初期閾値は PoC 申し送りの **6.68ms 候補**を採用する。**デバッガ停止中もタイムアウトを抑止しない**——停止中に 204 が返っても SSP を凍結させない方が望ましく、`co_scene`/`resume_until_valid` のポーリング回復モデルにより次 `OnSecondChange` で同コルーチンが resume され処理が回復する（work は失われない）。重要な制約として、LuaJIT はプリエンプション不可ゆえタイムアウトは SHIORI スレッドの待機を打ち切るのみ（アクタースレッドの Lua 実行は止めない）。したがって閾値は「通常処理では決して発火せず、停止／異常ブロック時のみ発動する安全網」として設定し（AC8）、通常経路のバイト不変（R1）を担保する。実機実測に基づく閾値の最終確定・チューニングは設計フェーズ以降に委ねる（初期値は 6.68ms）。
> **決定（ディスカッション #4, 2026-06-22）**: `panic=abort` 下の drop→204 ガードは「**dev/test/unwind プロファイル限定の安全網**」と明示割り切り（受容）＋ アクター/marshaling/teardown **正常経路の構造的 panic-free 化を受入基準化**（R5-AC10/AC11）。リリースのビルドプロファイル（`panic=abort`）は変更しない（横断的ビルド変更は本仕様スコープ外）。理由 = (1) `panic=abort` は意図的な既存方針で、既存 `windows.rs` の `catch_unwind` も既に「リリース到達不能・dev/test 向け」と割り切っており、二重基準を作らない一貫姿勢が健全。(2) 既存コードは元来 panic 回避を旨としているため、構造的 panic-free 化は新規負担ではなく既存姿勢の新経路への継続。(3) dev=unwind（panic 捕捉・ログ・drop→204 で観測可）／release=abort（高速）の分割は、リリース前に unwind 下で panic 経路を炙り出せるためデバッグ容易性に資する（R10 と整合）。

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
4. The teardown 処理 shall 二重解放／二重 teardown を防止し（冪等）、アクタースレッドの終了およびリソース解放の完了を、再 spawn 前に**確実に確認**する（完了確認の機構は設計に委ねる＝終了 ack／join 等。実装は終了 ack 方式を採用）。
5. If teardown の途中で異常（終了確認の不成立・リソース解放漏れ）が生じたとき, then the pasta システム shall その異常を記録し、ホスト（SSP）プロセスを巻き込んで落とさない。

> **設計補足（ディスカッション #3）**: 完了確認は `ActorMsg::Stop { done }` の done ack で実現（アクターが VM 破棄・debug teardown・ウィンドウ破棄を全て終えた後に ack 送信＝ack 受信時に全資源解放済み）。スレッドは detach し `JoinHandle`／二重 join 回避の `take()` は持たない。AC4 の「二重 join 回避」は join 廃止により構造的に成立する。

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

### Requirement 10: デバッグ容易性の保全（作者デバッグ＋開発デバッグ）

**Objective:** 本リファクタの最優先関心事として、(A) ゴースト作者の `.pasta`/`.lua` ソースレベルデバッグ（Phase 5 資産）を一切劣化させず、かつ (B) 本リファクタが導入するアクタースレッド・marshaling・VM pin といった並行機構を**開発者自身が実装中にデバッグできる環境**を確保したい。これによりデバッグ容易性を要件レベルで担保し、難物の並行化を「観測不能なブラックボックス」にしない。

#### Acceptance Criteria

**(A) 作者向けデバッグの保全**
1. While VM がアクタースレッドへ pin された後, the pasta システム shall 既存の `.pasta`/`.lua` ソースレベルデバッグ（行ブレークポイント・ステップ・変数 inspect・コルーチン inspect・VSCode attach）を完全に機能させ続ける。
2. When デバッグバックエンドが有効化されたとき, the pasta システム shall デバッグフック（`set_global_hook`）と DAP トランスポートを VM が常駐するアクタースレッド上で正しく動作させる（スレッドモデル転換後もフック発火・停止・再開・変数取得が成立する）。
3. The pasta システム shall ゴースト作者がデバッグ対象とするロジック（シーン論理・トーク構築・レンダラ呼び出しとその結果）を、`.pasta`/`.lua` の source-map されたデバッグ可能フロー内（VM 内）に留める（VM 外レンダリングを採らない。Requirement 2/3 と整合）。

**(B) 開発（リファクタ）向けデバッグ環境**
4. The pasta システム shall アクタースレッド・単一直列キュー・marshaling（GET/NOTIFY/drop→204）・teardown の主要シーム（メッセージの enqueue／dispatch／応答／drop、スレッド起動／shutdown／join）に、`@pasta_log`／`tracing` による観測可能なログ点を備える（無効時はゼロコスト維持）。
5. The pasta システム shall アクター機構の振る舞いを、ホスト（SSP）非依存に再現・検証できる決定論的テストハーネス（PoC `actor_poc/` の `sim_driver`／`mailbox` 検証等を本番化した形）で観測・デバッグ可能にする。
6. While 並行機構を実装・デバッグしている間, the 開発プロセス shall 特性化テスト先行・1 抽出=1 検証=1 コミットの revert 可能な小ステップを守り、各ステップで挙動差分を局所的に観測・切り分け可能に保つ。
7. The pasta システム shall データ競合・デッドロック・スレッド非決定性を、単一直列キューによる順序保存（Requirement 6）と drop→204 ガード（Requirement 5）により構造的に排除し、再現困難な並行バグの混入経路を最小化する。
8. When 本番接合（アクター機構・marshaling・teardown の出荷経路化）が完了したとき, the 開発プロセス shall PoC `actor_poc/` の使い捨て足場（`verdict.rs`・PoC scaffold・`actor-poc` feature gate）を最終タスクで撤去し、撤去前後で出荷 `pasta.dll` のバイト列が不変であること（正規化 sha 一致）を検証する。デバッグ資産（`sim_driver`／`mailbox`／`responder`／`coroutine_probe` 検証）は撤去対象に含めず、AC5 の本番テスト基盤へ昇格済みであることを前提とする。
