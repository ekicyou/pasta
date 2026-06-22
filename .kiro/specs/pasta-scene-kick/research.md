# Gap Analysis: pasta-scene-kick

要件（requirements.md・R1〜R6）と既存コードベースの差分を分析し、実装戦略の判断材料を提示する。本書は決定ではなく情報・選択肢の提示であり、最終的なアーキテクチャ選定は design フェーズに委ねる。

前提: 本機能は `pasta-actor-runtime`（完了済み）と `pasta-vscode-lua-debug`（完了済み）の上に構築される。

> **方針確定（要件ディスカッション 2026-06-23）**: 本機能は**即時再生オンリー**。SSP `Status` を権威とする恒常抑制ゲート（旧 R5）と非即時アイドル待ちモード（旧 R6）は**廃止**。キックは常に即時 preempt-and-abort（現 R5）。シーン実行 ctx（`act`）は**通常トーク再生と同一の合成手順を流用**してエンジンが与える（現 R3.2）。
>
> **配信モデル改訂（設計ディスカッション 2026-06-23・D4 参照）**: 当初仮置きの **talk FIFO ＋ 無条件 drain は不採用**。実コード検証でマルチ yield シーンと両立しないと判明したため、**既存の `co_scene` 継続機構（`check_talk`）を流用**し、キックは保留フラグ（`STORE.kick_pending`/`kick_force`）を立てて dispatch 前段フックで起動する方式に改訂。**以下本文中の `TALK_QUEUE`／「talk FIFO に enqueue」／「無条件 drain」の記述（特に §3 Option C・§4 課題3・Key Findings#4）は、すべて D4 の `co_scene` 継続流用へ読み替えること**。`is_blocked` はキック初回ビートのみワンショット突破し、後続ビート・通常イベント経路では従来どおり機能する。

## 1. 現状調査（Current State）

### 1.1 アクターランタイム（`crates/pasta_shiori/src/actor/`）

- `actor/mailbox.rs`: 単一直列 mailbox。`ActorMsg`（`#[non_exhaustive]`・L91-111）= `Get { req, reply }` / `Notify { req }` / `Stop { done }`。flume unbounded、単一 consumer、FIFO 順序保証。`Sender<ActorMsg>` は `Send + Sync + Clone`。reply/done は `flume::bounded(1)`。
  - **重要**: モジュール docstring（mailbox.rs L88-90）が `Kick`（talk FIFO 投入・後続仕様 `pasta-scene-kick`）の追加を**明示的に予約**している。設計上の要請は「別チャンネル＋`select!` ではなく**同一 mailbox への variant merge**」であり、単一 consumer 不変条件（flume cancel 欠陥 #104/#135 回避）を保つ唯一の拡張手段とされる。
- `actor/thread.rs`: VM を専用スレッドへ pin（`wintf_winmsg_executor::block_on`）。単一 `recv_async().await` ループ（L172-215）で mailbox を drain・`match msg` で各 variant を処理。`select!` を張らない。新 variant は match 腕の追加で処理。**`debug::enable` はこのスレッド上で VM 構築（`load`）の一部として実行され、束縛アドレスを `runtime().debug_local_addr()` で観測**（L152-158）。
- `actor/marshaling.rs`: `marshal_request()` が GET/NOTIFY を Rust レベルで判定（`determine_method` → pest parser）。GET=`recv_timeout(5s)` block-on-reply、NOTIFY=即 204、drop/timeout→`default_204()`。`ShioriMethod{Get, Notify}`。
- `actor/lifecycle.rs`: `static MAILBOX: ArcSwapOption<Sender<ActorMsg>>`（lock-free スロット・L61）。`spawn_actor()`（`MAILBOX.store`）／`marshal_request()`（`MAILBOX.load_full()`・未初期化なら 204）／`teardown_actor()`（`MAILBOX.swap(None)` で送信遮断）。
- presentation/レンダリング: `crates/pasta_lua/src/presentation/mod.rs`（`PresentationMarker`・`RenderBoundary` trait）、`crates/pasta_lua/src/runtime/renderer_injection.rs`（既定 = SHIORI さくらレンダラ、アダプタ注入）。シーン実行は Lua 側（`SCENE.co_exec` 等）でコルーチンを回し、yield 値が presentation event → さくらスクリプトへ描画される。

### 1.2 SHIORI ディスパッチと OnSecondChange（`pasta_shiori` ＋ `pasta_lua/pasta_scripts/`）

- `crates/pasta_shiori/src/lua_request.rs`: SHIORI/3.0 リクエストを Lua テーブル化。**`Status` ヘッダは `req.status` として既にパース済み**。
- `pasta/shiori/event/init.lua`: `EVENT.fire(req)`（L171-207）がハンドラ解決・`create_act(req)`→`SHIORI_ACT.new(STORE.actors, req)` で act 生成・`resume_until_valid(co, act)` でコルーチン resume・`set_co_scene(co)` で状態保存・`RES.ok(yielded)` 応答。`create_act`／`resume_until_valid`／`set_co_scene` が ctx 合成の中核（流用対象）。
- `pasta/scene.lua`: `SCENE.co_exec(act, name, global_scene_name, attrs)`（L193-214）がシーン関数をコルーチンで包む。シーン実行の最終エントリ。
- `pasta/shiori/event/second_change.lua`: OnSecondChange ハンドラ。`CALLBACK.sweep(os.time())` → `dispatcher.dispatch(act)`。talk FIFO drain はここに hook する（`sweep` 後・`dispatch` 前）。
- `pasta/shiori/event/virtual_dispatcher.lua`: `BLOCKED_STATUSES = {"talking", "choosing", ...}`、`is_blocked(status)`。**通常イベント経路（OnHour/OnTalk）の抑制に使われる**。本機能のキック側では**不使用**。
- talk FIFO 相当は**存在しない**。`CALLBACK.pending` は callback 待ちコルーチン保留（`{co, act, timeout_at, on_timeout}`）であり、汎用 talk キューではない。

### 1.3 debug backend（`crates/pasta_lua/src/debug/`）と VSCode 拡張（`editors/vscode/`）

- transport/（TCP＋Content-Length フレーミング・I/O 専用・`mlua::Lua` 非接触・`!Sync` 単一所有・socket-bridge スレッドで稼働・停止状態非依存で常時動く）、dap/（最小サブセット・`decode_request`）、wiring/（`handle_inbound` の A→B→C→D→E 固定順）、session/（停止状態機械）。
- **custom request 前例 = `pasta/sourcePresentation`**（`pasta-debug-lua-view-toggle`）:
  - Rust: `dap/decode.rs` L195-221 で文字列マッチ → `requested_source_mode` を `Decoded` に詰める。`wiring/inbound.rs` `try_source_presentation_toggle()` が自己完結で即応答＋イベント発行（汎用 routing に落ちない）。
  - TS: `editors/vscode/src/sourcePresentationToggle.ts`（`requestCommand = 'pasta/sourcePresentation'`・`setPayload`・`parseMode`）＋ `extension.ts` L150-219（`registerCommand` ＋ `isPastaSession()` ガード ＋ `session.customRequest(...)` ＋ try/catch エラー提示）＋ `package.json` `contributes.commands`/`menus`（`when: debugType == 'pasta'`）。**`showInputBox`/`showQuickPick` の既存使用は無い**（シーン名入力 UI は新パターン）。
- `DebugConfig`（`debug/config.rs`）: 既定 `enabled=false`／`listen=None`（ゼロコスト）。`enable.rs`（L86-）が有効時のみ port・hook・thread を起こす。socket-bridge スレッドは `cmd_tx`（→ session）/`out_rx`（→ socket）を多重化。
- **停止ループ制約（`session/stop_loop.rs` L66-205, `cmd_rx.recv()`）**: `SessionCommand`（Continue/Step/Inspect/Disconnect/SetBreakpoints/RefreshPresentation）は **Stopped イベント後＝ブレークで VM が停止中にのみ**消費される。ライブ実行中（ブレークなし）は stop_loop に入らないため、`SessionCommand` をそのまま増やしてもライブキックには使えない。inbound デコード（socket-bridge スレッド）は停止状態に依存せず常時動く。
- **クレート依存方向（design 決定の前提）**: `pasta_shiori` → `pasta_lua`（`pasta_shiori/Cargo.toml` L18 `pasta_lua.workspace = true`）。**debug backend は `pasta_lua`（上流）に、`static MAILBOX` は `pasta_shiori`（下流）に在る**。したがって socket-bridge スレッドから `pasta_shiori::MAILBOX` を直接参照することは**依存方向違反（上方参照）で不可**。これが design の最重要構造制約。

## 2. 要件 → 資産マップ（Requirement-to-Asset Map）

| 要件 | 既存資産 | ギャップ種別 |
| --- | --- | --- |
| R1 VSCode キックコマンド（単一即時モード）| `extension.ts` `registerCommand`／`sourcePresentationToggle.ts`／`package.json` 前例。`showInputBox` は新規 | **Constraint**（前例に倣う）＋ **Missing**（シーン名入力 UI）|
| R2 transport 一般化（`playScene`・シーン名のみ）| `dap/decode.rs` custom request マッチ・`wiring/inbound.rs` 自己完結ハンドラ前例 | **Constraint**（前例あり）＋ **Unknown**（停止ループ外の経路設計）|
| R2.4 debug backend のアクタークライアント化 | debug は現状 VM スレッド内 hook／stop_loop 経由のみ。アクター mailbox への送信経路は未配線。**クレート依存方向の制約あり** | **Missing**（kick sink seam ＋ inbound→sink 配線が新規）|
| R3.1/3.3-3.5 非同期実行・レンダリング | `actor/thread.rs` executor ループ・`presentation`・`renderer_injection`・Lua `SCENE.co_exec` | **Missing**（Kick 実行ハンドラ）＋ 既存基盤再利用可 |
| R3.2 ctx（`act`）合成 | 通常トーク再生の ctx 合成手順（`create_act`→`SHIORI_ACT.new`→`SCENE.co_exec`→`resume_until_valid`→`set_co_scene`）が既存 | **Constraint**（既存合成手順を流用・キック専用 ctx 構築は不要）|
| R4 OnSecondChange でのキック出力配信（co_scene 継続流用）| `check_talk` の `STORE.co_scene` 継続（`virtual_dispatcher.lua:202-205`）が既存。マルチビート配信機構そのものを流用 | **Constraint**（既存継続流用・dispatch 前段フックのみ新規・D4）|
| R5 即時 preempt-and-abort（唯一の再生挙動）| `set_co_scene(co)` は前 `co_scene` を `coroutine.close`（LuaJIT 非搭載なら no-op）で置換。強制中断 API は実質これのみ | **Missing**（preempt ＋ `co_scene` close 観測契約）|
| R6 ライブ SSP・既存挙動不変 | レンダラはライブ SSP 直結・通常イベント経路は独立 | **Constraint**（追加経路ゆえ不変が自然・要回帰検証）|

> 旧 R5 抑制ゲート・旧 R6 非即時アイドル待ちは**廃止**。`is_blocked()`/`BLOCKED_STATUSES` は通常イベント経路（OnHour/OnTalk）でのみ従来どおり機能する。

ギャップ凡例: **Missing**=新規実装、**Unknown**=design で要調査、**Constraint**=既存パターン準拠で吸収。

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張中心（推奨ベース）
- `ActorMsg::Kick { scene }` variant を mailbox.rs へ追加（docstring が予約済み・`#[non_exhaustive]`）。
- talk FIFO はアクタースレッド側（VM 同居）に Lua 側で保持。OnSecondChange GET ハンドラが drain。
- VSCode／DAP は `pasta/sourcePresentation` 前例の逐語ミラーで `playScene` を追加。
- **トレードオフ**: ✅ 既存パターン・不変条件（単一 consumer・ゼロコスト debug）を最大限活用、新規ファイル最小。❌ talk FIFO の保持層と drain タイミング、および kick sink のクレート越境を慎重設計しないと OnSecondChange 経路／依存方向を壊す。

### Option B: 新規コンポーネント中心
- キック専用の talk FIFO サービスと Kick 実行器を独立モジュール化。
- **トレードオフ**: ✅ 責務分離・単体テスト容易。❌ アクター単一 mailbox／単一 consumer 不変条件と二重管理になりやすく、別チャンネル化は設計が明示的に禁ずる方向（mailbox docstring）。debug 経路の独立 transport 新設も R2.3 が禁ずる。

### Option C: ハイブリッド（現実解・採用）
- transport／コマンド層は Option A（既存 DAP・mailbox variant merge）。
- kick sink は `pasta_lua` に**汎用 seam**（依存方向順守）として定義し、`pasta_shiori` が `MAILBOX` 投函クロージャを注入。
- talk FIFO ＋ drain ＋ preempt の中核ロジックは Lua 側の薄い `TALK_QUEUE` モジュールに切り出し、アクター単一 mailbox 上に載せ別チャンネルを作らない。
- **トレードオフ**: ✅ 不変条件・依存方向を守りつつ中核ロジックをテスト可能に分離。❌ 計画調整コストがやや高い。

## 4. 主要な統合課題（Integration Challenges）

1. **ライブキック経路 vs DAP 停止ループ（最重要・解決方針確定）**: 既存 `SessionCommand` はブレーク停止中（stop_loop）にのみ消費される。本機能はブレークなしのライブ実行中にキックする必要があるため、`playScene` を `SessionCommand` に足して stop_loop で処理する素朴案は不成立。**socket-bridge inbound（停止非依存）→ kick sink → アクター `static MAILBOX` へ `ActorMsg::Kick` を送る経路**を新設する。これが「debug backend をアクターのクライアント化（R2.4）」の実体。
2. **クレート依存方向の越境（最重要・新規発見）**: debug backend（`pasta_lua`・上流）から `MAILBOX`（`pasta_shiori`・下流）を直接参照できない。**解決: `pasta_lua` の `debug::enable` に汎用 `KickSink`（`Box<dyn Fn(KickRequest) + Send + Sync>` もしくは型付きチャネル）注入口を追加し、`pasta_shiori` 側（アクタースレッド）が `MAILBOX` 投函クロージャを注入する**。`pasta_lua` は sink の中身を知らない（疎結合・依存方向順守）。
3. **キック出力の配信機構（D4 で改訂・既存 `co_scene` 継続流用に確定）**: 当初は Lua `TALK_QUEUE` 新設＋無条件 drain を検討したが、設計ディスカッションの実コード検証でマルチ yield シーンと両立しないと判明。**既存の `STORE.co_scene`＋`check_talk` 継続をそのまま流用**し、`ActorMsg::Kick` 受信時は `kick_pending`/`kick_force` フラグ設置のみ、次 OnSecondChange の dispatch 前段フックが当該シーンを起動（`set_co_scene` 経由 preempt）→初回ビート配信、後続ビートは既存継続が担う。専用キューは設けない（D4）。
4. **preempt-and-abort の `co_scene` close（Missing・LuaJIT 制約）**: 現状 `set_co_scene(co)` は前 `STORE.co_scene` を `coroutine.close`（status≠suspended 時）または置換で破棄する。**LuaJIT は `coroutine.close` 非搭載**（MEMORY 既知事項）であり suspended を強制 dead にできない。preempt は「`STORE.co_scene` 参照の nil 化＋GC」でモデル化する。R5.2 の「閉じた」観測契約は「前 `co_scene` 参照が破棄され、以後 resume されない」を観測点とする（強制終了の即時性ではなく参照不到達を契約とする）。
5. **ctx 合成の流用（Constraint・低リスク）**: キック起点（`ActorMsg::Kick` 受信）から **通常トーク再生の合成手順（`create_act(req)`→`SHIORI_ACT.new`→`SCENE.co_exec(act, name)`→`resume_until_valid`→`set_co_scene`）をそのまま呼ぶ**。キック専用の特別な ctx 構築・初期束縛は要件外。`req` は最小合成（`id="OnKickScene"` 等のキック由来 act）で良い。
6. **≤1 秒レイテンシ（R4.4）**: 実 SSP の OnSecondChange tick 周期依存。エンジン側は「次の GET で必ず drain」を保証するのみ。
7. **既存挙動不変の回帰検証（R6）**: キック未使用時に通常 SHIORI 応答がバイト不変であることを保証する特性化テストが要る（`shiori-event-test-framework` 既存基盤・PASTA_DEBUG ガード留意）。

## 5. 実装規模・リスク

| 区分 | 評価 | 根拠 |
| --- | --- | --- |
| 規模 | **L（1〜2 週間）** | 複数レイヤー横断（VSCode TS／DAP Rust／kick sink クレート越境／actor mailbox／Lua talk FIFO・preempt）。個々は前例ありだが結線とライブ経路設計が新規。 |
| リスク | **Medium〜High** | DAP 停止ループ外のライブキック経路（課題1）・クレート越境 sink（課題2）・preempt close（課題4・LuaJIT 制約）が未確立。transport／コマンド／UI は前例ありで Medium、ライブ経路・sink・preempt は High。 |

## 6. design フェーズへの申し送り（設計で確定済み）

- **採用アプローチ**: Option C（既存 DAP・mailbox variant merge ＋ kick sink seam ＋ **既存 `co_scene` 継続流用**・保留フラグ駆動。当初の Lua `TALK_QUEUE` は D4 検証で不採用に改訂）。
- **確定した主要判断** → 「Design Decisions」参照（D1〜D6）。
- **Research Needed（design で解決）**:
  - socket-bridge スレッドから sink 経由 `MAILBOX` 送信時のライフサイクル（teardown／reload 時の swap 競合）→ **D2 / D6 で解決**。
  - LuaJIT での「中断シーンを確実に再開不能化」する具体手段と検証方法 → **D5 で解決（参照 nil 化＋GC・観測契約）**。
  - キック由来出力と通常 OnSecondChange dispatch（OnHour/OnTalk）の配信順序・共存ルール → **D4 で解決（dispatch 前段フックで `kick_pending` シーンを起動・既存 `co_scene` 継続流用・初回のみ `is_blocked` 突破）**。

## 7. オープンクエスチョン（要件ディスカッションで解決済み・2026-06-23）

1. **即時／非即時モードの UI 既定値** → **解決: 即時再生オンリー**。
2. **debug backend 無効時のキック** → **解決: debug 有効が前提条件**（現 R2.6）。
3. **複数即時キックの連続** → **解決: 即時 preempt-and-abort の再帰適用**。単一 mailbox／単一 consumer 上で FIFO 順に処理。
4. **キック対象シーンの指定粒度** → **解決: シーン名のみ**（現 R1/R2.2）。
5. **非即時キックの溜まり方** → **失効**: 非即時モード廃止。
6. **`talking` 以外の抑制ステータス** → **解決: キック初回ビートのみ `is_blocked` をワンショット突破**（現 R5.5）。後続ビートは既存 `is_blocked`（`talking`/`choosing` 等）の通常ペース配分に従う。恒常抑制ゲートは設けない。

---

# 設計フェーズ追記（Discovery & Design Decisions・2026-06-23）

## Summary
- **Feature**: `pasta-scene-kick`
- **Discovery Scope**: Extension（`pasta-actor-runtime` ＋ `pasta-vscode-lua-debug` の上に追加経路を載せる拡張）
- **Discovery Process**: Light（拡張・統合点中心。WebSearch 不要＝既存内製基盤の再利用のみ・新規外部依存ゼロ）
- **Key Findings**:
  1. **クレート依存方向が経路設計を支配する**: debug backend（`pasta_lua`・上流）は `static MAILBOX`（`pasta_shiori`・下流）を直接参照できない。kick sink を `pasta_lua` の汎用 seam として定義し `pasta_shiori` が `MAILBOX` 投函クロージャを注入する（依存方向順守）。
  2. **ライブキックは停止ループを迂回する**: `SessionCommand`/stop_loop はブレーク停止中のみ消費。ライブキックは停止非依存の socket-bridge inbound から sink → `MAILBOX` → `ActorMsg::Kick` で運ぶ。
  3. **ctx 合成は既存関数の素直な呼び出しで足りる**: `create_act`→`SHIORI_ACT.new`→`SCENE.co_exec`→`resume_until_valid`→`set_co_scene` がそのまま流用可能。キック専用 ctx 構築は不要。
  4. **配信は既存 `co_scene` 継続を流用（talk FIFO 不採用・D4 検証で改訂）**: `resume_until_valid` は 1 yield で停止し、残りビートは `STORE.co_scene`＋`check_talk` 継続が次 tick 以降に配信する既存機構がある。当初の Lua `TALK_QUEUE` 新設案は、完成さくらを貯める方式がマルチ yield と二重発火するため不採用。キックは保留フラグ（`kick_pending`/`kick_force`）を立て、dispatch 前段フックで既存機構へ載せる。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A 既存拡張中心 | mailbox variant ＋ Lua FIFO ＋ DAP 前例ミラー | 不変条件・前例最大活用、新規ファイル最小 | sink のクレート越境・FIFO 保持層を詰めないと依存方向／OnSecondChange を壊す | 採用ベース |
| B 新規中心 | 独立 Kick サービス＋専用チャネル | 責務分離・テスト容易 | 別チャンネル化を mailbox docstring が禁止・R2.3 が独立 transport を禁止 | 不採用 |
| C ハイブリッド | A ＋ kick sink seam（`pasta_lua`）＋ 既存 `co_scene` 継続流用（保留フラグ駆動・D4 改訂） | 不変条件・依存方向順守・マルチビート配信を既存機構で正しく実現 | dispatch 前段フックの回帰検証 | **採用** |

## Design Decisions

### Decision D1: キック transport は `playScene` custom request で `pasta/sourcePresentation` を逐語ミラー
- **Context**: R2（既存 debug DAP チャネルの一般化・別 transport 新設禁止）/ R1（VSCode コマンド）。
- **Alternatives Considered**:
  1. 新規 TCP/IPC チャネル — R2.3 違反・配線二重化。
  2. `SessionCommand` 拡張＋stop_loop 処理 — ライブ実行中は stop_loop に入らず不成立。
- **Selected Approach**: TS 側 `playSceneRequest.ts`（`requestCommand='pasta/playScene'`・`setPayload(scene)→{scene}`）＋ `extension.ts` に `registerCommand('pasta.debug.playScene')`（`isPastaSession()` ガード→`showInputBox` でシーン名→`session.customRequest`→try/catch）。Rust 側 `dap/decode.rs` に `"pasta/playScene"` 文字列マッチを追加し scene 名を抽出。
- **Rationale**: 前例があり最小差分。単一制御面（debug チャネル）に統合（R2.3）。
- **Trade-offs**: ✅ 低リスク・前例網羅。❌ `showInputBox` は新パターン（既存使用なし）だが標準 VSCode API。
- **Follow-up**: 空シーン名・未接続セッションのエラー提示（R1.3/R2.5）。

### Decision D2: kick sink を `pasta_lua` の汎用 seam とし `pasta_shiori` が `MAILBOX` 投函クロージャを注入（依存方向順守）
- **Context**: R2.4（debug backend のアクタークライアント化）。クレート依存は `pasta_shiori`→`pasta_lua` の一方向のみ。socket-bridge（`pasta_lua`）は `MAILBOX`（`pasta_shiori`）を参照不可。
- **Alternatives Considered**:
  1. socket-bridge から `pasta_shiori::MAILBOX` を直接 `use` — **依存方向違反（上方参照）でコンパイル不能**。
  2. `MAILBOX` を `pasta_lua` へ移設 — アクターランタイムの所有境界を壊す（pasta-actor-runtime の確定境界に反する）。
  3. `pasta_lua` に汎用 `KickSink` 注入口を設け、`pasta_shiori` が `MAILBOX.try_send(ActorMsg::Kick{..})` するクロージャを注入 — 疎結合・依存方向順守。
- **Selected Approach**: 案3。`debug::enable(lua, cfg, source_map, kick_sink)` に `kick_sink: Option<KickSink>` を追加（`KickSink = Arc<dyn Fn(KickRequest) + Send + Sync>`）。`RuntimeConfig` に `kick_sink` を builder で持たせ `with_config_and_source_map` から `enable` へ透過。`pasta_shiori` のアクタースレッドが VM 構築前に `MAILBOX` 投函クロージャ（`load_full()`→`try_send`）を `RuntimeConfig` へ束縛する。socket-bridge inbound が `pasta/playScene` を decode したら sink を呼ぶ（停止非依存・常時）。
- **Rationale**: `pasta_lua` は sink の中身（`ActorMsg`/`MAILBOX`）を知らずに済み、依存方向と pasta-actor-runtime の所有境界を双方保つ。
- **Trade-offs**: ✅ 構造的に正しい疎結合。❌ 注入の配線が 1 段増える（`RuntimeConfig`→`enable`→socket-bridge）。
- **Follow-up**: debug 無効時は sink 未注入＝経路非活性（R2.6）。teardown 時 `MAILBOX.swap(None)` 後の sink 呼び出しは `load_full()=None` で no-op（D6）。

### Decision D3: `ActorMsg::Kick { scene }` を同一 mailbox に variant merge（NOTIFY 流・fire-and-forget）
- **Context**: R3.1（非同期実行）/ mailbox docstring の予約。
- **Alternatives Considered**:
  1. 別チャンネル＋`select!` — docstring が明示的に禁止（flume cancel 欠陥回避の単一 consumer 不変条件を壊す）。
  2. 同一 mailbox に `Kick` variant 追加 — 予約どおり・FIFO 順序保証を継承。
- **Selected Approach**: 案2。`#[non_exhaustive] ActorMsg` に `Kick { scene: String }` を追加。executor ループ（`thread.rs`）に match 腕を追加し、reply 無し（NOTIFY 同様 fire-and-forget）。受信時にアクタースレッド上で `SHIORI.kick(scene)` を呼ぶ（＝`STORE.kick_pending`/`kick_force` フラグ設置のみ・非ブロッキング。実シーン起動は次 OnSecondChange の dispatch フック＝D4）。
- **Rationale**: 単一 consumer・FIFO・ゼロコスト不変条件を継承。GET/NOTIFY と同列の third method。フラグ設置のみで GET をブロックしない（R3.1）。
- **Trade-offs**: ✅ 不変条件継承・予約どおり。❌ executor ループに分岐 1 つ増。
- **Follow-up**: 複数キック連続は `kick_pending` 上書きで最後のキックが次 tick 起動。前キック co は `set_co_scene` 置換で preempt（D5）。

### Decision D4: キック配信は既存 `co_scene` 継続を流用（talk FIFO 不採用・設計ディスカッション検証で改訂）
- **Context**: R4（キック出力の OnSecondChange 配信）。当初は「完成さくらを貯める talk FIFO ＋ 無条件 drain」を仮置きしたが、**設計ディスカッションの実コード検証（2026-06-23）で破綻が判明**した。
- **検証で判明した事実**: `resume_until_valid`（`event/init.lua`）は**最初の有効 yield で停止**し 1 ビートのみ返す。マルチビート・トークの残りビートは `set_co_scene` で `STORE.co_scene` に保存され、次 OnSecondChange の `check_talk`（`virtual_dispatcher.lua:202-205`）が `if STORE.co_scene then return STORE.co_scene` で**既存 co を継続 resume** する（`is_blocked` がビート間ペースをゲート）。
- **却下した当初案（talk FIFO）**: 「初回ビートを `TALK_QUEUE` へ・残りを `co_scene` 継続へ」は、同一 tick で `TALK_QUEUE` drain と `co_scene` 継続 resume が**二重発火**し、順序逆転・co 状態不定を招く。完成さくらを貯める FIFO はマルチ yield シーンと両立しない。
- **Selected Approach（改訂）**: **talk FIFO を設けず、既存 `co_scene` 継続機構を流用**する。
  1. `SHIORI.kick(scene)`＝`STORE.kick_pending=scene`／`STORE.kick_force=true` を設置するだけ（非ブロッキング・resume しない）。
  2. 次 OnSecondChange の `dispatch(act)` 前段で、`kick_force` 真なら `is_blocked` を**ワンショット突破**し、`kick_pending` があれば当該 OnSecondChange の `act` を流用して `act:find_scene`→`SCENE.co_exec`→`set_co_scene`（前 `co_scene` close＝preempt）→`resume_until_valid` で初回ビートを返す。フラグは消費。
  3. 2 ビート目以降は既存 `check_talk` の `STORE.co_scene` 継続が後続 tick に配信（`is_blocked` 通常ペース）。
- **Rationale**: マルチビート配信を**既存機構そのもの**で正しく実現。キュー新設ゼロ・ctx 合成も既存 `act` 流用で「トーク再生と同じ」を徹底（ユーザー方針）。preempt は `set_co_scene` 既存挙動で自然成立。
- **Trade-offs**: ✅ 二重発火/順序問題が原理的に発生しない・新規ファイル最小（`event/kick.lua` のみ・`talk_queue.lua` 不要）。❌ `dispatch` 前段にフック分岐＋`is_blocked` 突破条件を追加（回帰検証要・R6.3／突破ワンショット消費をテストで担保）。
- **Follow-up**: 初回ビートのみ突破・後続は通常ペース（R5.5）の特性化テスト。マルチ yield シーンの順序・二重出力なしの E2E。

### Decision D5: preempt-and-abort は前 `co_scene` 参照の nil 化＋GC でモデル化（LuaJIT `coroutine.close` 非搭載前提）
- **Context**: R5（即時 preempt-and-abort・前 `co_scene` を閉じ自動復帰しない）。MEMORY 既知: LuaJIT は `coroutine.close` 非搭載。
- **Alternatives Considered**:
  1. `coroutine.close(prev)` で強制 dead — LuaJIT 非搭載で no-op、確実性なし。
  2. 前 `STORE.co_scene` 参照を nil 化（`set_co_scene` の既存置換ロジックを流用）し GC 回収・以後 resume されないことを契約とする。
- **Selected Approach**: 案2。`Kick` 実行時に新シーンの ctx 合成→`set_co_scene(new_co)` を呼ぶことで、既存ロジックが前 `co_scene` を破棄（参照 nil 化）する。R5.2 の「閉じた」観測契約 = **前 `co_scene` 参照が `STORE` から不到達になり、以後どの経路からも resume されない**（強制終了の即時性ではなく参照不到達を観測点とする）。自動復帰なし（退避スタックを持たない＝R5.3）。
- **Rationale**: LuaJIT 制約下で唯一確実なモデル。`set_co_scene` の既存「前シーン置換」が preempt と同型。
- **Trade-offs**: ✅ 既存ロジック流用・LuaJIT 互換。❌ 中断コルーチンは GC まで生存しうる（メモリは GC が回収・実害なし）。観測契約が「即 dead」でなく「参照不到達」になる点をテストで明示。
- **Follow-up**: 「前 `co_scene` が resume されない」ことの特性化テスト（中断後に通常 dispatch が前シーンを再開しない）。

### Decision D6: teardown/reload 時の sink no-op は `MAILBOX.load_full()=None` で自然吸収
- **Context**: R2.6（debug 無効時非活性）／ライフサイクル競合（teardown 中のキック）。
- **Selected Approach**: sink クロージャは毎回 `MAILBOX.load_full()` を読む。`teardown_actor()` が `MAILBOX.swap(None)` 済みなら `None`→送信せず no-op。reload 後は新 `Sender` が `store` 済みで新アクターへ届く。debug 無効時は sink 未注入で経路自体が非活性。
- **Rationale**: 既存 `marshal_request` と同一の lock-free read パターン。新たな同期プリミティブ不要。
- **Trade-offs**: ✅ 既存不変条件（送信パスに Mutex なし）を維持。❌ teardown 競合中のキックは黙って捨てられる（デバッグ用途では許容・診断ログを残す）。

## Risks & Mitigations
- **R-1 クレート越境 sink の配線ミス** — `RuntimeConfig`→`enable`→socket-bridge の透過を型で固定（`KickSink` 型エイリアス）。注入忘れ＝sink None＝経路非活性で安全側。
- **R-2 OnSecondChange dispatch フックが通常応答をバイト変化させる** — キック未使用時は `STORE.kick_pending=nil` でフックが完全素通り（既存 dispatch のまま）。特性化テスト（バイト不変・PASTA_DEBUG ガード）で回帰検証（R6.3）。
- **R-3 LuaJIT で中断シーンが GC まで生存** — 参照 nil 化で resume 不到達を保証。メモリは GC 回収。観測契約をテストで明示（D5）。
- **R-4 teardown 競合中のキック消失** — 仕様上許容（デバッグキックは礼儀正しいキューでない）。診断ログ（`seam = "kick.drop"`）を残す。
- **R-5 空/不正シーン名** — decode 段で空チェック→要求元へエラー（R2.5）。解決不能シーンは `co_scene` を据えず（前会話保持）破棄＋診断ログ（R3.5）。

## References
- `crates/pasta_shiori/src/actor/mailbox.rs`（`ActorMsg` 予約 docstring L88-90）
- `crates/pasta_shiori/src/actor/lifecycle.rs`（`static MAILBOX` L61・`marshal_request` lock-free read）
- `crates/pasta_shiori/src/actor/thread.rs`（executor ループ・`debug_local_addr` 観測）
- `crates/pasta_lua/src/debug/enable.rs`（`enable` 注入口・socket-bridge spawn）
- `crates/pasta_lua/src/debug/dap/decode.rs` L195-221・`wiring/inbound.rs`（`pasta/sourcePresentation` 前例）
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`（`create_act`/`resume_until_valid`/`set_co_scene`）
- `crates/pasta_lua/pasta_scripts/pasta/scene.lua`（`SCENE.co_exec`）
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/second_change.lua`・`virtual_dispatcher.lua:202-205`（`check_talk` の `co_scene` 継続・dispatch フック点）
- `editors/vscode/src/sourcePresentationToggle.ts`・`extension.ts` L150-219・`package.json`（前例）
