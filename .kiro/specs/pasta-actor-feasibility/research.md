# Gap Analysis: pasta-actor-feasibility

実施日: 2026-06-21 ／ 対象: 要件 R1〜R8（アクター化 go/no-go PoC）

## 現状調査サマリ

pasta は SHIORI DLL（`pasta_shiori`、Windows 専用）がホスト（SSP）スレッド上で `pasta_lua` の `!Send` Lua VM を同期駆動する反応専用エンジン。VM は `Arc<Mutex<Option<PastaShiori>>>` ＋ `unsafe impl Send` で SHIORI スレッドに束縛。トーク継続・非同期 callback は Lua 側コルーチンで保持され、ホストの SHIORI リクエスト周期が唯一の駆動軸。debug backend が既に「別スレッド＋mpsc チャネル＋socket2 エフェメラルポート」で VM へコマンドを送り込む前例を実証済み——本 PoC はこれを mailbox へ一般化する。

## Requirement → 既存資産マップ（ギャップタグ: Missing / Unknown / Constraint）

| Req | 必要能力 | 既存資産（ファイル:行） | ギャップ |
|---|---|---|---|
| **R1** executor 上 `!Send` VM ホスト＋reload teardown | VM 生成・pin・解放 | `runtime/mod.rs:48-84`（`PastaLuaRuntime{lua}`）、`runtime/mod.rs:166`（`Lua::unsafe_new_with`）、`windows.rs:63/93`（`load`/`unload`）、`debug/enable.rs:129-220`（thread spawn 前例） | **Missing**: winmsg_executor 統合。VM は現状 `load()` の SHIORI スレッドで生成。PoC は専用 std::thread→`block_on(actorフューチャ)` で VM 所有へ。**Constraint**: `block_on` は呼び出しスレッドのメッセージループを回す→自前スレッド spawn 必須。reload=unload+load サイクルで漏れなく teardown |
| **R2** GET block-on-reply / NOTIFY fire-and-forget marshaling | FFI→executor 橋渡し | `windows.rs:115`（`request`）、`windows.rs:148`（`RawShiori`）、`shiori.rs`（`request()`→Lua `SHIORI.request`） | **Missing**: 現状 `request()` は SSP スレッドで VM 同期実行（marshaling なし）。PoC は enqueue→oneshot block-on-reply へ置換。**Unknown**: GET/NOTIFY メソッド判定の取得点（pest プロトコルパーサ）が VM 投入前に得られるか要確認 |
| **R3** drop→204 デッドロック消滅 | 応答チャネル drop ガード | 204 応答生成は `shiori.rs` の応答組立に存在 | **Missing**: `oneshot::Sender` を包み Drop で未送信時 204 を撃つ responder 型（panic/忘れ安全） |
| **R4** executor 駆動下の coroutine/callback 生存 | シーン継続・callback resume | `store.lua:56`（`co_scene`）、`event/init.lua:128-152`（`resume_until_valid`）、`callback.lua:22/58`（`pending`/`consume_staged`）、`second_change.lua:15-24`（OnSecondChange） | **Constraint（低リスク）**: コルーチンは VM 内に存在し VM 移設で保持される。resume の**駆動主体**をホスト tick→executor へ移すのが要点。実シーンモデル（継続＋callback 待機）に忠実な再現が必要 |
| **R5** ≤1秒キック配信（FIFO＋Status gate＋preempt） | talk FIFO・drain・gate・preempt | `second_change.lua`（drain 契機）、`set_co_scene`→`coroutine.close()`（preempt 素地）、SSP `Status: talking`（baseware 提供） | **Missing**: pasta_shiori 層の talk FIFO＋`Status:talking` gate＋即時 preempt（PoC は最小ハーネス）。**Research Needed**: ≤1秒の実測には実 SSP または忠実な OnSecondChange ドライバが必要 |
| **R6** GET レイテンシ実測＋204フォールバック要否 | 計測器 | なし | **Missing/Research**: block-on-reply 計測の instrumentation。代表的 SSP 呼び出しパターンの定義 |
| **R7** 隔離・バイト不変・feature-gate | 使い捨て隔離 | `tests/common/mod.rs:26-32`（`#[ctor]` で `PASTA_DEBUG`/`_PORT` 中和）、`transport/mod.rs:172/176`（`set_reuse_address`＋port 0）、`DebugConfig::default()`（ゼロコスト無効） | **Missing**: `pasta_lua/Cargo.toml` に `[features]` セクションが**現存しない**。default-off の `actor-poc` feature を新設（撤去済み `lua-debug-poc` 前例に倣う）。**Constraint**: 実行時config 流儀（debug）か cargo feature 流儀か。R7.2 バイト不変は default-off cargo feature が有利（無効時は何もコンパイルしない） |
| **R8** 段階判定文書 | go/no-go 文書 | roadmap の GO+ 文言前例（`pasta-lua-debug-feasibility`） | **None**: 成果物は文書のみ |

## 実装アプローチ選択肢

### Option A: 既存クレート拡張（pasta_shiori＋pasta_lua 内に PoC モジュール）
debug/ と同型の sibling モジュール（feature `actor-poc` でガード）を両クレートに置き、実 `PastaLuaRuntime` と実コルーチンスクリプトをそのまま駆動。
- ✅ debug の thread/channel/socket2/`#[ctor]` 前例を最大再利用。実 FFI 経路に忠実（R2/R4/R5 が説得力を持つ）
- ❌ 2 クレートに跨る。feature 衛生が緩むと release へ漏れるリスク

### Option B: 隔離ハーネス（別クレート or 統合テストバイナリ）
出荷クレートの非テストコードを触らず、別の使い捨てクレート/テストハーネスが pasta_lua/pasta_shiori をリンクして executor＋VM＋marshaling を駆動。
- ✅ バイト不変が最強（出荷コード無改変）・撤去が容易
- ❌ 実 `request()` FFI 経路・実 SSP を忠実に再現しづらく、R2/R5 の end-to-end 実証が弱い

### Option C: ハイブリッド（**推奨**）
`pasta_lua`／`pasta_shiori` に default-off の `actor-poc` feature を最小新設し、debug/ の thread＋channel＋socket2 流儀を写した小ハーネスをガード。実 VM・実コルーチンを使用。R5/R6 の実 SSP 計測は薄い attach または忠実な OnSecondChange ドライバで実施。本番移行完了時に撤去。
- ✅ 実経路忠実（R2/R4/R5 が信頼可）＋ default-off feature でバイト不変 ＋ 前例整合（撤去済み `lua-debug-poc` と同ライフサイクル）
- ❌ 2 クレートの feature 衛生に注意が必要

## 工数・リスク（PoC 全体）

| 項目 | 工数 | リスク | 根拠 |
|---|---|---|---|
| 全体 | **M〜L**（3日〜2週） | **High** | 未経験の executor 統合＋`!Send` VM＋FFI 反転＋実 SSP タイミング。高リスクゆえの PoC |
| R1 executor host/teardown | L | High | winmsg_executor×mlua×reload の三重未知（本丸） |
| R2 marshaling | M | High | 同期契約×スレッド分離×デッドロック回避 |
| R3 drop→204 | S | Low | Drop guard は定型・debug の前例近傍 |
| R4 coroutine 生存 | M | Medium | VM 内保持で素地あり・駆動移行が要点 |
| R5 ≤1秒実 SSP | M | High | 実 SSP/忠実ドライバ依存・配信タイミング |
| R6 レイテンシ実測 | S | Medium | 計測器のみ・パターン定義が鍵 |
| R7 feature/隔離 | S | Low | `[features]` 新設＋`#[ctor]`＋socket2 で定型 |
| R8 文書 | S | Low | 文書のみ |

## 設計フェーズへの申し送り（Research Needed）

1. **winmsg_executor 統合形**: `block_on` が呼び出しスレッドでメッセージループを回す前提を確認し、`std::thread::spawn`→`block_on(actor)` のアクタースレッド型を確定。`spawn_local`（サブタスク）・`JoinHandle`（teardown 待ち）・メッセージ専用ウィンドウの drop 時解放の挙動を実機確認。
2. **GET/NOTIFY 判定点**: `pasta_shiori` の pest プロトコルパーサでメソッド（GET/NOTIFY）が VM 投入**前**に取得できるか。marshaling 分岐（block-on-reply か即 204）の判断に必要。
3. **実 SSP 計測**【要件ディスカッション#1で決定: 忠実シミュレータ採用】: R5/R6 の「実 SSP 相当」は OnSecondChange 周期＋`Status: talking` 遷移を忠実に再現する自前ドライバ（実機 attach は任意）と要件側で定義済み（R5.6）。設計は忠実シミュレータの構築を主とし、再生中に SSP が tick を送り続けるか（gate 検証前提）は ukadoc/任意実機スモークで補助確認。実機絶対性能保証は `pasta-actor-runtime` へ申し送り。
4. **feature 衛生とバイト不変検証法**: `pasta_lua`（必要なら `pasta_shiori`）に default-off `actor-poc` を導入し、無効時の release 成果物バイト不変を成果物 diff で検証する手順を確立。
5. **リエントランシー順序**: executor がキック処理中に到着する `OnPastaCallBack` GET（pending コルーチン resume 必須）の単一 mailbox 内順序制御。
6. **隔離の二択**: 実行時config 流儀（debug 同様ゼロコスト）と cargo feature 流儀（撤去済み `lua-debug-poc` 同様）の最終選択。R7.2 バイト不変要件は後者寄り。

## 推奨

- **Option C（ハイブリッド・default-off `actor-poc` feature）** を採用し、debug backend の thread/channel/socket2/`#[ctor]` 前例を最大限写経する。
- 設計フェーズの最重要決定: ①アクタースレッド型（`std::thread`＋`block_on`）の確定、②marshaling 契約（GET/NOTIFY 判定点＋responder drop→204）、③実 SSP 計測ハーネスの実現方式（attach vs 忠実ドライバ）。
- R1（executor 上 VM ホスト＋teardown）が NO-GO 本丸——設計はここを最初の実証スライスに置くべき。
