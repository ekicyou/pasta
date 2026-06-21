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
5. **リエントランシー順序**【要件ディスカッション#2で決定: 設計不変条件として解決済み・新規要件不要】: 順序はアクターモデルの**単一 mailbox 直列処理**で構造的に確定し、キックは**FIFO 投入→OnSecondChange の排出点でのみ消費**、即時再生は**`talking` を無視して常にさくらスクリプトで上書き（preempt＝破棄、保全分岐なし）**。よってレース／状態破壊は原理的に発生しない。PoC では独立試験を立てず、**R1（executor/アクター）＋R4（coroutine 生存）＋R5（FIFO drain＋preempt）を複合シナリオで走らせて**暗黙にカバーする。設計はこの不変条件（順序＝直列 mailbox 保証／即時＝常時上書き）を前提とすること。
6. **隔離の二択**: 実行時config 流儀（debug 同様ゼロコスト）と cargo feature 流儀（撤去済み `lua-debug-poc` 同様）の最終選択。R7.2 バイト不変要件は後者寄り。

## 推奨

- **Option C（ハイブリッド・default-off `actor-poc` feature）** を採用し、debug backend の thread/channel/socket2/`#[ctor]` 前例を最大限写経する。
- 設計フェーズの最重要決定: ①アクタースレッド型（`std::thread`＋`block_on`）の確定、②marshaling 契約（GET/NOTIFY 判定点＋responder drop→204）、③実 SSP 計測ハーネスの実現方式（attach vs 忠実ドライバ）。
- R1（executor 上 VM ホスト＋teardown）が NO-GO 本丸——設計はここを最初の実証スライスに置くべき。

---

# 設計フェーズ追記（2026-06-21）

## Summary（設計フェーズ）
- **Feature**: `pasta-actor-feasibility`
- **Discovery Scope**: Extension（既存 `pasta_lua`／`pasta_shiori` への feature-gated PoC 追加・Light Discovery）
- **Key Findings（設計フェーズで確定）**:
  1. **GET/NOTIFY 判定点は VM 投入前に確定済み（Research Needed #2 解決）**: pest パーサ（`crates/pasta_shiori/src/lua_request.rs:86-87`）が `Rule::get`/`Rule::notify` を `req.method` として設定し、`SHIORI.request()` 内で VM 投入前に参照可能。marshaling 分岐（block-on-reply か即 204）の判断点として利用できる。
  2. **debug 前例は VM＝ホストスレッド／補助 I/O スレッドのみ spawn**: `debug/enable.rs:154` `hook::install(lua, session)` でセッションは VM スレッド内に閉じ、socket-bridge／encoder のみ別スレッド。本 PoC は **VM 自体を専用アクタースレッドへ移す**点で構造的に逸脱（最大の未知＝R1）。
  3. **teardown idiom は写経可能**: `DebugHandle::Drop`（`debug/handle.rs:133-177`）は terminate 送信→30ms bounded sleep→shutdown `AtomicBool`→socket-bridge join（ポート解放待ち）→encoder detached。socket2 エフェメラルは `transport/mod.rs:162-178`（`set_reuse_address(true)`＋port0＋`local_addr()` 読み戻し）。`#[ctor]` env ガードは `tests/common/mod.rs:26-32`。
  4. **`wintf_winmsg_executor` は未導入**: 全 Cargo.toml/Cargo.lock に不在。`[features]` セクションも両クレートに現存せず——新設＋optional 依存追加が必要。upstream `winmsg-executor` は `block_on`／`spawn_local`／message-only window／`Send` 不要を提供することを確認。

## Architecture Pattern Evaluation（設計フェーズ）

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A 既存クレート拡張 | 両クレートに feature-gated PoC モジュール、実 VM・実コルーチン駆動 | 前例最大再利用・実経路忠実 | 2 クレート跨り・feature 衛生 | gap 分析 Option A |
| B 隔離ハーネス | 別クレート/テストバイナリで pasta_lua/shiori をリンク | バイト不変最強・撤去容易 | 実 FFI/実 SSP 再現が弱い | R2/R5 説得力不足 |
| **C ハイブリッド（採用）** | default 無効 `actor-poc` feature＋debug idiom 写経＋実 VM/実コルーチン＋忠実シミュレータ | 実経路忠実＋バイト不変＋前例整合 | 2 クレート feature 衛生に注意 | 本設計の採用方針 |

## Design Decisions（設計フェーズ）

### Decision: アクタースレッド型＝`std::thread::spawn` → `block_on(actor future)`
- **Context**: `!Send` VM を executor 上にホストする方式の確定（R1）。`block_on` は呼び出しスレッドのメッセージループを回すため自前スレッド spawn が必須。
- **Selected Approach**: 専用 `std::thread` 内で `wintf_winmsg_executor::block_on` を呼び、その future が `PastaLuaRuntime`（`!Send` VM）を生成・所有・pin。`JoinHandle` で teardown 待ち、`spawn_local` でサブタスク化。
- **Rationale**: debug 前例（VM＝ホストスレッド）からの逸脱だが、アクター化に必須。VM 操作をアクタースレッドに閉じ `!Send` 制約を遵守。
- **Trade-offs**: 三重未知（winmsg_executor×mlua×reload）で High リスク——だからこそ R1 を最初の実証スライスに置く。
- **Follow-up**: `block_on` のメッセージループ回転前提・メッセージ専用ウィンドウ Drop 解放を実機確認（OPEN Q2）。

### Decision: marshaling 契約＝pest method 判定 → GET block-on-reply／NOTIFY 即 204
- **Context**: SSP↔アクター橋渡しの同期契約（R2）。GET/NOTIFY 判定点（Research Needed #2）。
- **Selected Approach**: pest 解決済み `method` で分岐。GET は `Responder` 内包 `GetMsg` を enqueue し応答までブロック、NOTIFY は responder なしで即 204。PoC は既存 `shiori.rs:request()` 同期経路を**置換せず**別ハーネスで pest 解決値を消費（出荷経路バイト不変）。
- **Rationale**: 判定点が VM 投入前に確定済みのため分岐が成立。出荷経路非干渉でバイト不変を優先。
- **Trade-offs**: pest を PoC 側で再解決する取り回しが必要（OPEN Q3）。

### Decision: drop→204 responder ガード＝`Drop` で未送信時 204
- **Context**: 応答未送信のまま drop（panic/忘れ）のデッドロック経路消滅（R3）。
- **Selected Approach**: oneshot を包む `Responder`。「reply 1 回」または「drop→204」のいずれかで必ず終結。oneshot は追加依存を避け `std::sync::mpsc`（1 回受信）＋ Drop ガードで実装（OPEN Q5）。
- **Rationale**: debug 近傍の定型・Low リスク。最小依存でバイト不変を優先。

### Decision: 「実 SSP 相当」＝忠実シミュレータ（要件 R5.6 を設計化）
- **Context**: ≤1 秒キック配信・レイテンシ実測（R5/R6）。要件ディスカッション#1で忠実シミュレータ採用が決定済み。
- **Selected Approach**: `SimDriver` が OnSecondChange 周期＋`Status: talking` 遷移を再現。実機 SSP attach は任意スモーク、絶対性能保証は `pasta-actor-runtime` へ申し送り。
- **Follow-up**: 再生中の tick 継続（gate 前提）を ukadoc／任意実機で補助確認（OPEN Q4）。

### Decision: 順序＝単一 mailbox 直列保証（要件ディスカッション#2を設計不変条件化）
- **Context**: リエントランシー順序。独立要件は不要（討議#2で解決済み）。
- **Selected Approach**: 単一 mailbox の直列処理で順序を構造的に確定。キックは FIFO 投入→OnSecondChange 排出点でのみ消費、即時再生は `talking` 無視で常時上書き＝破棄（保全分岐なし）。独立試験を立てず R1＋R4＋R5 複合シナリオで暗黙カバー。

## Synthesis Outcomes（設計フェーズ）
- **Generalization**: R1〜R6 は「単一 mailbox アクター上で `!Send` VM を駆動し、SSP からの同期/非同期要求を marshaling する」という単一能力の側面。mailbox＋responder を共通基盤に据え、各 probe をその上の検証シナリオとして配置。
- **Build vs. Adopt**: executor は自作せず `wintf_winmsg_executor`（公開フォーク）を adopt。thread/channel/teardown/socket2/`#[ctor]` は debug backend の確立 idiom を adopt（写経）。oneshot は `std::sync::mpsc` を adopt し追加依存を回避。
- **Simplification**: 出荷経路の置換・本番アクター API・presentation event stream 契約を排除（PoC スコープ外）。順序の独立試験を排除（直列 mailbox で構造保証）。判定文書は単一 `Verdict` に集約。

## Risks & Mitigations（設計フェーズ）
- R1 三重未知（winmsg_executor×mlua×reload）— 最初の実証スライスに集中投資、不成立は NO-GO 根拠として記録。
- feature 衛生（2 クレート跨り）— default 無効＋cfg-gated mod 宣言＋バイト不変 diff 検証で担保（R7.2）。
- 実 SSP タイミング差異 — 忠実シミュレータ＋任意実機スモーク、絶対保証は後続申し送り。
- pest 出荷経路非干渉の取り回し — PoC 側再解決を仮置き（OPEN Q3）。

## OPEN QUESTIONS（設計ディスカッションへ申し送り）
1. **Q1 依存解決**: `wintf_winmsg_executor` の crate 名／version／source（crates.io か git フォークか）と公開 API（`MessageLoop`/`JoinHandle`/`FilterResult`）の正確な形。
2. **Q2 executor 統合形**: `block_on` のメッセージループ回転前提・`spawn_local`・`JoinHandle`・メッセージ専用ウィンドウ Drop 解放の実機確認（R1 検証と一体）。
3. **Q3 marshaling 反転の出荷経路非干渉**: pest 解決値を出荷経路改変なしで PoC ハーネスへ供給する取り回し（再解決 or 共有）。
4. **Q4 gate 検証の実機補助**: 再生中に SSP が OnSecondChange tick を送り続けるかの ukadoc／任意実機スモーク要否・範囲。
5. **Q5 responder oneshot 実装選択**: `std::sync::mpsc`（1 回受信）か軽量 oneshot crate か。
