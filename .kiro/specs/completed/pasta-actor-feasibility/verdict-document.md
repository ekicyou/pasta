<!--
この文書は自動生成された成果物です（手書き禁止）。
生成元: crates/pasta_lua/tests/actor_poc_integration.rs::generate_verdict_document_artifact
再生成: cargo test -p pasta_lua --features actor-poc --test actor_poc_integration generate_verdict_document_artifact -- --ignored
実 run_all()（実アクタースレッド／実 !Send mlua VM）の end-to-end 実走を反映。
確定段階: GO+（高信頼）
-->

# pasta-actor-feasibility 段階判定文書（R8）

## 判定段階: GO+（高信頼）

## 隔離前提（R8.3・判定妥当性の前提）
- default 無効 feature-gate: true
- バイト不変（前提）: true
- テスト非汚染（前提）: true

## 項目別試行結果（R8.2・全項目を成否にかかわらず記録）
- R1: 成立 — executor 上 !Send VM ホスト＋reload teardown 成立（8 サイクル clean）
- R2: 成立 — GET block-on-reply が応答値を持ち帰り、NOTIFY が即 204 fire-and-forget で終結（ActorThread+Mailbox+Responder プリミティブで実駆動・VM はアクタースレッドに pin）
- R3: 成立 — 応答未送信のまま responder を drop（VM 失敗）しても Drop ガードが 204 を撃ち、block-on-reply は無限待機せず 204 で終結（デッドロック経路の原理的消滅）
- R4: 成立 — executor 駆動下で実 *.lua の co_scene が中断地点から継続（observed=[1, 2, 3]・5 drives）し CALLBACK.pending が後続契機で解決（実 *.lua 無改変・コルーチン意味論は Lua のまま）
- R5: 成立 — 忠実シミュレータで ≤1 秒キック配信を実測（1 tick = 1s sim / 76.7µs wall-clock）
    - 制約: LuaJIT 2.1 に coroutine.close が無く suspended コルーチンを強制終了できないため、preempt の close は破棄（abandon＝STORE.co_scene=nil ＋ live レジストリ除去 ＋GC）で成立する（preempt 自体は破棄上書きで機能する）。pasta-actor-runtime へ申し送り。
    - 制約: NOTIFY 状態では次 GET tick まで配信を遅延（held）
- R6: 成立 — GET block-on-reply 代表値（in-process 実測 n=32・実 VM 往復=32）: min=26.3µs median=26.9µs p95=132.4µs p99=668µs max=668µs・mean=54.918µs。フォールバック推奨=true 閾値候補=6.68ms
    - 制約: in-process 実測は sub-ms（NW/プロセス間境界なし）。実機 SSP の絶対性能保証ではない（R6.3 申し送り）

## ブロッカーと回避候補（R8.4・NO-GO/制約付き判定の根拠）
- R6.3: GET block-on-reply の in-process 実測（max=668µs）は宿主の許容応答時間に対し 十分速いが、実機 SSP の OnSecondChange 周期・プロセス間 marshaling・実シーン VM 負荷下の絶対レイテンシ保証は本 PoC では確定できない。実機 SSP に対する 絶対性能保証と GET タイムアウト方針の最終確定は後続実装仕様 pasta-actor-runtime へ申し送る（R5.6/R6.3）
    - 回避候補: GET タイムアウト→204 フォールバックを設け、閾値候補 6.68ms（観測最大に安全 マージンを乗せた値）を初期値として pasta-actor-runtime で実機実測に基づき調整

## 後続実装仕様 pasta-actor-runtime への前提結論（R8.5）
- 採用 executor 統合方式: std::thread::spawn 内で wintf-winmsg-executor の block_on を回し、その future が !Send な実 PastaLuaRuntime（mlua VM）を生成・所有する。再ポーリングは executor の MSG_ID_WAKE（Waker）で駆動し、値のみを mpsc で越境させる。
    - 根拠: R1 実証: executor 上 !Send VM ホスト＋reload teardown 成立（8 サイクル clean）
- VM pin / teardown 方針: mlua の !Send 制約により VM はアクタースレッドを構造的に越えない（pin）。teardown は Arc<AtomicBool>(SeqCst)→wake→JoinHandle::join、Drop は take() で二重 join を回避する （debug DebugHandle::Drop idiom）。reload は shutdown→再 spawn の反復で clean teardown とハンドル/ポート非枯渇を確認する。
    - 根拠: R1 teardown 実証: executor 上 !Send VM ホスト＋reload teardown 成立（8 サイクル clean）
- marshaling 契約: GET = block-on-reply（ActorMsg::Get に Responder を載せ enqueue→応答受信までブロック し応答値を戻す・SHIORI/3.0 同期契約）。NOTIFY = 即 204 fire-and-forget（応答経路なし）。 method 判定・marshaling 分岐は決定論ロジックとして Rust 側で完結し、シーン中核は Lua の まま保つ。
    - 根拠: R2 実証: GET block-on-reply が応答値を持ち帰り、NOTIFY が即 204 fire-and-forget で終結（ActorThread+Mailbox+Responder プリミティブで実駆動・VM はアクタースレッドに pin）
- drop→204 ガード方針: GET 応答 Responder を採用し、応答未送信のまま drop（応答忘れ／panic 巻き戻し）されたら 自動的に 204 を撃つ。これにより『応答未送信のまま drop』のデッドロック経路が原理的に 消滅する。注意: release は panic=abort のため unwind に依存する panic 経路の保証は test/unwind profile でのみ成立する（pasta-actor-runtime へ申し送り）。
    - 根拠: R3 実証: 応答未送信のまま responder を drop（VM 失敗）しても Drop ガードが 204 を撃ち、block-on-reply は無限待機せず 204 で終結（デッドロック経路の原理的消滅）
- coroutine 生存条件: 駆動主体をホスト tick から executor へ移しても、実 *.lua（STORE.co_scene／CALLBACK.pending）が中断地点から resume し callback が解決する。各 resume は別個の executor 駆動（ホスト tick ではない）であり、コルーチン意味論は Lua のまま保つ。
    - 根拠: R4 実証: executor 駆動下で実 *.lua の co_scene が中断地点から継続（observed=[1, 2, 3]・5 drives）し CALLBACK.pending が後続契機で解決（実 *.lua 無改変・コルーチン意味論は Lua のまま）
- GET レイテンシとフォールバック要否: R5: 忠実シミュレータ上で ≤1 秒キック配信を実測（GET tick=Ref3=1 のみ配信・NOTIFY 状態では次 GET tick まで held）。 R6: GET block-on-reply は in-process 実測で十分速いが、防御として GET タイムアウト →204 フォールバックを推奨する。 R5 制約: LuaJIT 2.1 に coroutine.close が無く suspended コルーチンを強制終了できないため、preempt の close は破棄（abandon＝STORE.co_scene=nil ＋ live レジストリ除去 ＋GC）で成立する（preempt 自体は破棄上書きで機能する）。pasta-actor-runtime へ申し送り。 / NOTIFY 状態では次 GET tick まで配信を遅延（held）。推奨フォールバック方針: GET タイムアウト→204 フォールバックを設け、閾値候補 6.68ms（観測最大に安全 マージンを乗せた値）を初期値として pasta-actor-runtime で実機実測に基づき調整
    - 根拠: R5 実証: 忠実シミュレータで ≤1 秒キック配信を実測（1 tick = 1s sim / 76.7µs wall-clock）／R6 実証: GET block-on-reply 代表値（in-process 実測 n=32・実 VM 往復=32）: min=26.3µs median=26.9µs p95=132.4µs p99=668µs max=668µs・mean=54.918µs。フォールバック推奨=true 閾値候補=6.68ms
