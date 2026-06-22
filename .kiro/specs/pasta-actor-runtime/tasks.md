# Implementation Plan

> **全タスク横断の不変条件**: R1（外部 SHIORI 挙動バイト不変・全既存テスト回帰不変）と R9（Lua コルーチン/callback 意味論無改変）は全タスクに適用する。各タスクは「特性化テスト緑 → 抽出 → 再検証緑 → コミット」の 1 抽出=1 検証=1 コミット（revert 可能な小ステップ）で進める。本仕様はバイト不変の段階リファクタゆえ**意図的に逐次**（各段は特性化テストでゲートされるため `(P)` を付さない）。

- [ ] 1. Foundation: 特性化テスト基盤と依存セットアップ
- [x] 1.1 FFI 入口応答バイト列のゴールデン特性化テスト敷設
  - 代表 SHIORI イベント列（OnBoot／OnSecondChange／GET property／コルーチン継続）を流し、`request` の応答バイト列をゴールデン固定する
  - 以後の全段で緑維持する回帰ガードとして**最初に**確立（新依存・新実装の前）
  - 観測: ゴールデンテストが現行実装で緑になり、以後の各段で応答バイト差分を検出できる
  - _Requirements: 1.1, 1.2, 1.4_
- [x] 1.2 flume 依存追加と executor 依存のアダプタ移設
  - `pasta_shiori` に flume 0.12 を依存追加（mailbox＝`recv_async`／reply・done＝`recv_timeout` 兼用）
  - `wintf-winmsg-executor` をアクタースレッド所有者の `pasta_shiori` 側で利用可能にし、executor 選択をアダプタ層に閉じ込める
  - 観測: 新依存込みで `cargo build` 成功し、1.1 のゴールデン＋既存テストが回帰不変で緑
  - _Requirements: 4.4_

- [ ] 2. presentation event マーカー契約とさくらレンダラの論理デカップリング
- [x] 2.1 宿主非依存 presentation マーカー最小集合の導入
  - 既存 Lua トークン（talk ライン／アクター切替／wait／choice）を宿主非依存マーカーとして表現する薄い層を VM 内に導入する（最小集合のみ・破壊的変更なしに拡張できる型体系）
  - マーカー列を VM 内・`.pasta`/`.lua` source-map されたデバッグ可能フローに留める
  - 観測: マーカー導入後も talk／アクター切替／wait／choice の最終さくらスクリプト出力がゴールデンとバイト不変
  - _Requirements: 2.1, 2.2, 2.3, 2.5, 2.6, 2.7, 2.8_
  - _Boundary: PresentationMarker_
- [x] 2.2 さくらレンダラのアダプタ注入化（登録経路のデカップリング）
  - `@pasta_sakura_script` 登録を「コア無条件起点」から「アダプタが注入するレンダラ」へ変更し、マーカー列を消費してさくらスクリプト文字列へ変換する責務をアダプタ注入側に置く（注入なし時は既存どおりでバイト不変）
  - 描画コード（Rust・Lua）は `pasta_lua` に物理維持（Lua 集約死守）し、レンダリングは VM 内に維持する
  - 観測: 注入経路でも `talk_to_script` 出力がゴールデンとバイト不変・コアはさくら描画を必須責務として保持しない
  - _Requirements: 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  - _Boundary: SakuraRenderer_

- [ ] 3. アクタースレッド・単一直列 mailbox・marshaling の本番化
- [x] 3.1 flume Waker × wintf executor の wake 統合実証
  - アクタースレッドで wintf `block_on` 上の `recv_async().await` が、別スレッドからの `try_send` で起床することを最小実証する（RN1 の薄い実証）
  - 観測: 別スレッドからの `try_send` で `block_on` の future が起床しメッセージを消費するテストが緑
  - _Requirements: 4.1, 4.5_
  - _Boundary: ActorThread_
- [x] 3.2 単一直列 mailbox（flume）の本番化
  - flume unbounded で `ActorMsg{Get, Notify, Stop}` の単一直列 FIFO を確立する（単一 consumer・`select!` を張らない）
  - 観測: 近接した複数 `try_send` が投入順＝処理順で逐次処理され、同時並行 VM アクセスが発生しないテストが緑
  - _Requirements: 6.1, 6.2, 6.3, 6.4_
  - _Boundary: Mailbox_
- [x] 3.3 アクタースレッドへの VM pin と recv_async 駆動ループ
  - wintf `block_on` 上で `!Send` VM を生成・pin し、単一 `recv_async().await` ループで mailbox を消費して `SHIORI.request` Function を呼ぶ
  - 既存コルーチン（`co_scene`／`resume_until_valid`／`CALLBACK`）を executor 駆動下で無改変に resume・継続させる
  - 観測: VM 実行スレッド ID＝アクタースレッド ID（SHIORI スレッドと別）で、コルーチン継続・callback 解決テストが緑
  - _Requirements: 4.1, 4.2, 4.3, 4.5, 9.1, 9.2, 9.3, 9.4_
  - _Boundary: ActorThread_
  - _Depends: 3.1, 3.2_
- [x] 3.4 GET/NOTIFY/drop→204/timeout→204 marshaling の本番化
  - GET＝reply tx 同梱で `try_send`→同期 `recv_timeout(6.68ms)`→値 or 204、NOTIFY＝即 204、応答経路 drop（Disconnected）→204、アクター異常→204 を実装する
  - GET タイムアウト閾値は通常運転で発火しない安全網に設定し、デバッガ停止中も抑止せず、停止中の 204 は次 `OnSecondChange` で回復させる（コルーチン状態保存）
  - 観測: marshaling 置換後も応答バイト列がゴールデンと不変・timeout/drop で必ず 204・無限待機やデッドロックが発生しない
  - 観測（panic-free）: アクター/marshaling 正常経路に `unwrap`/`expect`/境界外索引が無いことを静的確認（R5.10）
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10, 5.11_
  - _Boundary: MarshalingLayer, Reply_
  - _Depends: 3.2, 3.3_

- [ ] 4. reload teardown の本番化
- [x] 4.1 Stop{done} ack による clean teardown と reload リーク検査
  - unload／detach で `ActorMsg::Stop { done }` を送り、アクターが残メッセージ drain 後に VM 破棄・debug teardown・ウィンドウ破棄を終えて `done` ack を返し、再 load で新規 spawn する（teardown は冪等・スレッドは detach）
  - teardown 途中異常を記録しホストプロセスを巻き込まない
  - 観測: unload→load 反復でスレッドハンドル／USER オブジェクト／port のリーク・枯渇が無い（done ack 後に計測）・二重 teardown が安全に no-op
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  - _Boundary: Teardown, ActorLifecycle_
  - _Depends: 3.3_

- [ ] 5. FFI 所有モデル再設計と unsafe 撤去
- [x] 5.1 static MAILBOX 所有モデルへの再配線と unsafe impl Send/Sync 撤去
  - `OnceLock<RawShiori>`＋`Arc<Mutex<Option<PastaShiori>>>`＋`unsafe impl Send/Sync` を、flume `Sender` を保持する `static MAILBOX` 所有モデルへ置換する（lock-free 送信・送信パスに Mutex を置かない）
  - `load`=spawn_actor／`unload`・detach=teardown_actor へ結線（thread spawn は load 起点として loader lock を回避）
  - 観測: `unsafe impl Send/Sync` が撤去され `cargo build`/`clippy` 緑・応答バイト列ゴールデン不変・VM はアクタースレッドを越えない
  - 観測（RN6 記録）: 採用したスロット型（persistent の `OnceLock<Sender>` か respawn の `ArcSwapOption<Sender>` 等）をコード/コメントに明記し、送信パスが Mutex フリーであることを確認
  - _Requirements: 8.1, 8.2, 8.3, 8.4_
  - _Boundary: ActorLifecycle_
  - _Depends: 3.4, 4.1_

- [ ] 6. デバッグ容易性の保全と開発観測性
- [x] 6.1 アクタースレッド上でのデバッグバックエンド動作保全
  - VM がアクタースレッドへ pin された後も `set_global_hook` がアクタースレッドで発火し、`.pasta`/`.lua` 行ブレークポイント・ステップ・変数 inspect・コルーチン inspect・VSCode attach が成立することを保証する
  - 観測: アクター化後の DAP デバッグ E2E（BP 停止・変数取得・attach）が緑で、既存デバッグに無回帰
  - _Requirements: 10.1, 10.2, 10.3_
  - _Boundary: DebugBackend 統合_
  - _Depends: 3.3_
- [x] 6.2 観測ログ点の付与と決定論テストハーネスの本番昇格
  - marshaling/teardown の主要シーム（try_send／recv／reply／drop／timeout／spawn／stop／done）に `tracing`/`@pasta_log` のログ点を付与する（無効時ゼロコスト）
  - `actor_poc` の `sim_driver`／`mailbox`／`coroutine_probe` 検証＋reply move/drop の exactly-once をホスト非依存の決定論テストハーネスへ昇格する
  - 観測: ログ無効時に応答バイト不変・決定論ハーネスが緑でアクター機構を観測・デバッグできる
  - _Requirements: 10.4, 10.5, 10.6, 10.7_
  - _Boundary: ActorTestHarness_
  - _Depends: 3.4, 4.1_

- [ ] 7. 統合・機能レベル検証・PoC 足場撤去
- [x] 7.1 全経路統合と機能レベルバイト不変回帰
  - 全コンポーネントを出荷 `request` 経路へ統合し、代表 SHIORI イベント列で機能レベルのバイト不変と全既存テスト回帰不変を確認する
  - 観測: `cargo test --all` 緑・ByteInvariantSuite 緑・OnBoot/OnSecondChange/GET property/コルーチン継続が end-to-end でバイト不変
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 9.1, 9.2, 9.3, 9.4_
  - _Depends: 5.1, 6.1, 6.2_
- [ ] 7.2 PoC actor_poc 足場撤去と出荷バイナリ不変検証
  - 本番接合完了後に `actor_poc` 使い捨て足場（`verdict.rs`・PoC scaffold・`actor-poc` feature gate）を撤去する（デバッグ資産は 6.2 で昇格済み）
  - 観測: 撤去前後で出荷 `pasta.dll` の正規化 sha が一致・`actor-poc` feature 参照が消滅・`cargo build`/`test` 緑
  - _Requirements: 10.8_
  - _Depends: 7.1_

## Implementation Notes
- 4.1: reload リーク検査の許容値は 1-handle/cycle 漏れが境界（tolerance == RELOAD_CYCLES）に乗る。実害は複数資源/cycle 漏れで確実に検出されるが、将来 tolerance を `< RELOAD_CYCLES` か N 非依存の小定数へ厳格化する余地あり（7.1/7.2 で再検討可）。
- 5.1: invalid-UTF-8 リクエストは 204 を返す（旧 500）。R5.6「常に文字列を返す/ハングしない」契約への意図的整合。golden は正常 UTF-8 のみゆえ非カバー・正常系バイト不変は維持。7.1 の機能レベル検証で留意。GET_TIMEOUT は 5s（RN4・通常経路非発火）。スロットは ArcSwapOption（RN6・respawn）。
