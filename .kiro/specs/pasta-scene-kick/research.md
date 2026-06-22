# Gap Analysis: pasta-scene-kick

要件（requirements.md・R1〜R6）と既存コードベースの差分を分析し、実装戦略の判断材料を提示する。本書は決定ではなく情報・選択肢の提示であり、最終的なアーキテクチャ選定は design フェーズに委ねる。

前提: 本機能は `pasta-actor-runtime`（完了済み）と `pasta-vscode-lua-debug`（完了済み）の上に構築される。

> **方針確定（要件ディスカッション 2026-06-23）**: 本機能は**即時再生オンリー**。SSP `Status` を権威とする抑制ゲート（旧 R5）と非即時アイドル待ちモード（旧 R6）は**廃止**。キックは常に即時 preempt-and-abort（現 R5）で、talk FIFO は OnSecondChange で**無条件 drain**（現 R4）する。シーン実行 ctx（`act`）は**通常トーク再生と同一の合成手順を流用**してエンジンが与える（現 R3.2）。以下の本文中、抑制ゲート（`is_blocked()`/`BLOCKED_STATUSES`）に関する記述は「キック側では不使用（通常イベント経路でのみ従来どおり使用）」と読み替えること。

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
| R4 talk FIFO ＋ OnSecondChange 無条件 drain | OnSecondChange ハンドラ・`RES.ok` はあるが **talk FIFO は皆無**。抑制判定は不使用 | **Missing**（FIFO 新規・drain は抑制ゲート無し）|
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
3. **talk FIFO の保持層（Lua 側に確定）**: presentation→さくらスクリプト描画は Lua 集約。FIFO を Lua 側（`pasta_scripts` の `TALK_QUEUE`）に置き、`ActorMsg::Kick` 受信 → アクタースレッド上で ctx 合成・レンダリング → 結果さくらスクリプトを `TALK_QUEUE` へ enqueue。OnSecondChange GET が `TALK_QUEUE` を無条件 drain。drain は抑制ゲート無し（「次の OnSecondChange で必ず drain」のみ）。
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

- **採用アプローチ**: Option C（既存 DAP・mailbox variant merge ＋ kick sink seam ＋ Lua 側 `TALK_QUEUE`）。
- **確定した主要判断** → 「Design Decisions」参照（D1〜D6）。
- **Research Needed（design で解決）**:
  - socket-bridge スレッドから sink 経由 `MAILBOX` 送信時のライフサイクル（teardown／reload 時の swap 競合）→ **D2 / D6 で解決**。
  - LuaJIT での「中断シーンを確実に再開不能化」する具体手段と検証方法 → **D5 で解決（参照 nil 化＋GC・観測契約）**。
  - キック由来出力と通常 OnSecondChange dispatch（OnHour/OnTalk）の drain 順序・共存ルール → **D4 で解決（FIFO drain を dispatch 前に・抑制無し）**。

## 7. オープンクエスチョン（要件ディスカッションで解決済み・2026-06-23）

1. **即時／非即時モードの UI 既定値** → **解決: 即時再生オンリー**。
2. **debug backend 無効時のキック** → **解決: debug 有効が前提条件**（現 R2.6）。
3. **複数即時キックの連続** → **解決: 即時 preempt-and-abort の再帰適用**。単一 mailbox／単一 consumer 上で FIFO 順に処理。
4. **キック対象シーンの指定粒度** → **解決: シーン名のみ**（現 R1/R2.2）。
5. **非即時キックの溜まり方** → **失効**: 非即時モード廃止。
6. **`talking` 以外の抑制ステータス** → **解決: 抑制無し**（現 R4 無条件 drain）。

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
  4. **talk FIFO は内製ゼロ・Lua 側に新設**: `CALLBACK.pending` は talk キューではない。Lua `TALK_QUEUE` を新設し OnSecondChange で無条件 drain。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A 既存拡張中心 | mailbox variant ＋ Lua FIFO ＋ DAP 前例ミラー | 不変条件・前例最大活用、新規ファイル最小 | sink のクレート越境・FIFO 保持層を詰めないと依存方向／OnSecondChange を壊す | 採用ベース |
| B 新規中心 | 独立 Kick サービス＋専用チャネル | 責務分離・テスト容易 | 別チャンネル化を mailbox docstring が禁止・R2.3 が独立 transport を禁止 | 不採用 |
| C ハイブリッド | A ＋ kick sink seam（`pasta_lua`）＋ Lua `TALK_QUEUE` 薄分離 | 不変条件・依存方向順守しつつ中核をテスト可能に分離 | 計画調整コスト | **採用** |

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
- **Selected Approach**: 案2。`#[non_exhaustive] ActorMsg` に `Kick { scene: String }` を追加。executor ループ（`thread.rs`）に match 腕を追加し、reply 無し（NOTIFY 同様 fire-and-forget）。受信時にアクタースレッド上で ctx 合成→レンダリング→`TALK_QUEUE` enqueue。
- **Rationale**: 単一 consumer・FIFO・ゼロコスト不変条件を継承。GET/NOTIFY と同列の third method。
- **Trade-offs**: ✅ 不変条件継承・予約どおり。❌ executor ループに分岐 1 つ増。
- **Follow-up**: 複数キック連続は FIFO 順に再帰 preempt（D5）。

### Decision D4: talk FIFO は Lua `TALK_QUEUE` に新設し OnSecondChange で無条件 drain（dispatch 前）
- **Context**: R4（talk FIFO ＋ 無条件 drain）。既存 talk キューは皆無（`CALLBACK.pending` は callback 待ち）。
- **Alternatives Considered**:
  1. Rust アクタースレッド側に FIFO 保持 — レンダリング（Lua 集約）と保持層が分離し drain で Lua 往復増。
  2. Lua `pasta_scripts` 側に `TALK_QUEUE` 保持 — レンダリング結果（さくらスクリプト文字列）と同一層・drain は Lua 内で完結。
- **Selected Approach**: 案2。`pasta/shiori/talk_queue.lua`（`TALK_QUEUE.enqueue(sakura)`/`TALK_QUEUE.drain()→sakura|nil`）を新設。`Kick` 実行ハンドラがレンダリング結果を `enqueue`。`second_change.lua` が `CALLBACK.sweep()` 後・`dispatcher.dispatch(act)` 前に `TALK_QUEUE.drain()` を呼び、非 nil なら **抑制ゲート無しで**その出力を GET 応答として返す（`is_blocked` を介さない）。空なら従来 dispatch 経路へ。
- **Rationale**: レンダリングと保持が同一層で順序・drain が自然。drain は「次の OnSecondChange で必ず」のみ（条件分岐なし）。
- **Trade-offs**: ✅ Lua 内完結・順序保証単純。❌ OnSecondChange ハンドラに drain 分岐 1 つ追加（回帰検証要・R6.3）。
- **Follow-up**: キック出力と通常 dispatch（OnHour/OnTalk）の優先順位＝キック drain を先に消費。複数キック分は FIFO で逐次 drain（1 tick 1 出力）。

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
- **R-2 OnSecondChange drain 分岐が通常応答をバイト変化させる** — キック未使用時は `TALK_QUEUE.drain()=nil` で従来経路へ素通り。特性化テスト（バイト不変・PASTA_DEBUG ガード）で回帰検証（R6.3）。
- **R-3 LuaJIT で中断シーンが GC まで生存** — 参照 nil 化で resume 不到達を保証。メモリは GC 回収。観測契約をテストで明示（D5）。
- **R-4 teardown 競合中のキック消失** — 仕様上許容（デバッグキックは礼儀正しいキューでない）。診断ログ（`seam = "kick.drop"`）を残す。
- **R-5 空/不正シーン名** — decode 段で空チェック→要求元へエラー（R2.5）。解決不能シーンは `TALK_QUEUE` へ何も積まず破棄＋診断ログ（R3.5）。

## References
- `crates/pasta_shiori/src/actor/mailbox.rs`（`ActorMsg` 予約 docstring L88-90）
- `crates/pasta_shiori/src/actor/lifecycle.rs`（`static MAILBOX` L61・`marshal_request` lock-free read）
- `crates/pasta_shiori/src/actor/thread.rs`（executor ループ・`debug_local_addr` 観測）
- `crates/pasta_lua/src/debug/enable.rs`（`enable` 注入口・socket-bridge spawn）
- `crates/pasta_lua/src/debug/dap/decode.rs` L195-221・`wiring/inbound.rs`（`pasta/sourcePresentation` 前例）
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/init.lua`（`create_act`/`resume_until_valid`/`set_co_scene`）
- `crates/pasta_lua/pasta_scripts/pasta/scene.lua`（`SCENE.co_exec`）
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/second_change.lua`（drain hook 点）
- `editors/vscode/src/sourcePresentationToggle.ts`・`extension.ts` L150-219・`package.json`（前例）
