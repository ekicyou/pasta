# Gap Analysis: pasta-scene-kick

要件（requirements.md・R1〜R6）と既存コードベースの差分を分析し、実装戦略の判断材料を提示する。本書は決定ではなく情報・選択肢の提示であり、最終的なアーキテクチャ選定は design フェーズに委ねる。

前提: 本機能は `pasta-actor-runtime`（完了済み）と `pasta-vscode-lua-debug`（完了済み）の上に構築される。

> **方針確定（要件ディスカッション 2026-06-23）**: 本機能は**即時再生オンリー**。SSP `Status` を権威とする抑制ゲート（旧 R5）と非即時アイドル待ちモード（旧 R6）は**廃止**。キックは常に即時 preempt-and-abort（現 R5）で、talk FIFO は OnSecondChange で**無条件 drain**（現 R4）する。シーン実行 ctx（`act`）は**通常トーク再生と同一の合成手順を流用**してエンジンが与える（現 R3.2）。以下の本文中、抑制ゲート（`is_blocked()`/`BLOCKED_STATUSES`）に関する記述は「キック側では不使用（通常イベント経路でのみ従来どおり使用）」と読み替えること。

## 1. 現状調査（Current State）

### 1.1 アクターランタイム（`crates/pasta_shiori/src/actor/`）

- `actor/mailbox.rs`: 単一直列 mailbox。`ActorMsg`（`#[non_exhaustive]`）= `Get { req, reply }` / `Notify { req }` / `Stop { done }`。flume unbounded、単一 consumer、FIFO 順序保証。
  - **重要**: モジュール docstring（mailbox.rs L88-90）が `Kick`（talk FIFO 投入・後続仕様 `pasta-scene-kick`）の追加を**明示的に予約**している。設計上の要請は「別チャンネル＋`select!` ではなく**同一 mailbox への variant merge**」であり、単一 consumer 不変条件（flume cancel 欠陥 #104/#135 回避）を保つ唯一の拡張手段とされる。
- `actor/thread.rs`: VM を専用スレッドへ pin（`wintf_winmsg_executor::block_on`）。単一 `recv_async().await` ループで mailbox を drain。`select!` を張らない。
- `actor/marshaling.rs`: `marshal_request()` が GET/NOTIFY を Rust レベルで判定（`determine_method`）。GET=`recv_timeout(5s)` block-on-reply、NOTIFY=即 204、drop/timeout→`default_204()`。
- `actor/lifecycle.rs`: `static MAILBOX: ArcSwapOption<Sender<ActorMsg>>`（lock-free スロット）。`spawn_actor()` / `marshal_request()` / `teardown_actor()`。
- presentation/レンダリング: `crates/pasta_lua/src/presentation/mod.rs`（`PresentationEvent`・`RenderBoundary`）、`crates/pasta_lua/src/runtime/renderer_injection.rs`（既定 = SHIORI さくらレンダラ、アダプタ注入）。シーン実行は Lua 側（`SCENE.co_exec` 等）でコルーチンを回し、yield 値が presentation event → さくらスクリプトへ描画される。

### 1.2 SHIORI ディスパッチと OnSecondChange（`pasta_shiori` ＋ `pasta_lua/pasta_scripts/`）

- `crates/pasta_shiori/src/lua_request.rs`: SHIORI/3.0 リクエストを Lua テーブル化。**`Status` ヘッダは `req.status` として既にパース済み**（生文字列、`"talking,choosing"` 等）。
- `pasta/shiori/event/init.lua`: `EVENT.fire(req)` がハンドラ解決・act 生成・コルーチン resume・`set_co_scene(co)` で状態保存・`RES.ok()` 応答。
- `pasta/shiori/event/second_change.lua`: OnSecondChange ハンドラ。`CALLBACK.sweep(os.time())` → `dispatcher.dispatch(act)`。
- `pasta/shiori/event/virtual_dispatcher.lua`: `BLOCKED_STATUSES = {"talking", "choosing", ...}`、`has_status(status, keyword)`、`is_blocked(status)`。**`Status: talking` ゲートの既存実装が OnHour/OnTalk 自動発火の抑制に既に使われている**（本機能の抑制ゲートと同じ判定軸）。
- talk FIFO 相当は**存在しない**。`CALLBACK.pending` は非同期コールバック用コルーチン保留であり、汎用 talk キューではない。

### 1.3 debug backend（`crates/pasta_lua/src/debug/`）と VSCode 拡張（`editors/vscode/`）

- transport/（TCP＋Content-Length フレーミング・I/O 専用・Lua 非接触）、dap/（最小サブセット・`decode_request`/`response`/`event`）、wiring/（`handle_inbound` の A→B→C→D→E 固定順）、session/（停止状態機械）。
- **custom request 前例 = `pasta/sourcePresentation`**（`pasta-debug-lua-view-toggle`）:
  - Rust: `dap/decode.rs` L195-222 で文字列マッチ → `requested_source_mode` を返し、`wiring/inbound.rs` `try_source_presentation_toggle()` が自己完結で即応答＋イベント発行（汎用 routing に落ちない）。
  - TS: `editors/vscode/src/sourcePresentationToggle.ts`（`requestCommand = 'pasta/sourcePresentation'`）＋ `extension.ts` L150-219（`registerCommand` ＋ `session.customRequest(...)`）＋ `package.json` `contributes.commands`/`menus`。
- `DebugConfig`（`debug/config.rs`）: 既定 `enabled=false`／`listen=None`（ゼロコスト）。`enable.rs` が有効時のみ port・hook・thread を起こす。
- **停止ループ制約（stop_loop.rs L77 `self.cmd_rx.recv()`）**: `SessionCommand`（Continue/Step/Inspect/Disconnect/SetBreakpoints）は **Stopped イベント後＝ブレークで VM が停止中にのみ**消費される。ライブ実行中（ブレークなし）は stop_loop に入らないため、`SessionCommand` をそのまま増やしてもライブキックには使えない。inbound デコード（socket-bridge スレッド）は停止状態に依存せず常時動く。

## 2. 要件 → 資産マップ（Requirement-to-Asset Map）

| 要件 | 既存資産 | ギャップ種別 |
| --- | --- | --- |
| R1 VSCode キックコマンド（単一即時モード）| `extension.ts` `registerCommand`／`sourcePresentationToggle.ts`／`package.json` 前例 | **Constraint**（前例に倣う・拡張）|
| R2 transport 一般化（`playScene`・シーン名のみ）| `dap/decode.rs` custom request マッチ・`wiring/inbound.rs` 自己完結ハンドラ前例 | **Constraint**（前例あり）＋ **Unknown**（停止ループ外の経路設計）|
| R2.4 debug backend のアクタークライアント化 | debug は現状 VM スレッド内 hook／stop_loop 経由のみ。アクター mailbox への送信経路は未配線 | **Missing**（inbound→mailbox 配線が新規）|
| R3.1/3.3-3.5 非同期実行・レンダリング | `actor/thread.rs` executor ループ・`presentation`・`renderer_injection`・Lua `SCENE.co_exec` | **Missing**（Kick 実行ハンドラ）＋ 既存基盤再利用可 |
| R3.2 ctx（`act`）合成 | 通常トーク再生の ctx 合成手順（`EVENT.fire`→act 生成→`SCENE.co_exec`）が既存 | **Constraint**（既存合成手順を流用・キック専用 ctx 構築は不要）|
| R4 talk FIFO ＋ OnSecondChange 無条件 drain | OnSecondChange ハンドラ・`RES.ok` はあるが **talk FIFO は皆無**。抑制判定は不使用 | **Missing**（FIFO 新規・drain は抑制ゲート無し）|
| R5 即時 preempt-and-abort（唯一の再生挙動）| `set_co_scene(co)` で前シーン状態は保持されるが、**強制 close／中断 API はない** | **Missing**（preempt ＋ `co_scene` close）|
| R6 ライブ SSP・既存挙動不変 | レンダラはライブ SSP 直結・通常イベント経路は独立 | **Constraint**（追加経路ゆえ不変が自然・要回帰検証）|

> 旧 R5 抑制ゲート（`is_blocked()`/`has_status()` 再利用）・旧 R6 非即時アイドル待ち（保留 drain ロジック）は**廃止**。キック側に抑制・保留意味論は無く、対応ギャップ（保留 drain ロジック新規）は消滅した。`is_blocked()`/`BLOCKED_STATUSES` は通常イベント経路（OnHour/OnTalk）でのみ従来どおり機能する。

ギャップ凡例: **Missing**=新規実装、**Unknown**=design で要調査、**Constraint**=既存パターン準拠で吸収。

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張中心（推奨ベース）
- `ActorMsg::Kick { scene, mode }` variant を mailbox.rs へ追加（docstring が予約済み・`#[non_exhaustive]`）。
- talk FIFO はアクタースレッド側（VM 同居）に Lua/Rust いずれかで保持。OnSecondChange GET ハンドラが drain。
- 抑制ゲートは既存 `is_blocked()`/`req.status` を再利用・拡張。
- VSCode／DAP は `pasta/sourcePresentation` 前例の逐語ミラーで `playScene` を追加。
- **トレードオフ**: ✅ 既存パターン・不変条件（単一 consumer・ゼロコスト debug）を最大限活用、新規ファイル最小。❌ talk FIFO の保持層（Lua vs Rust）と drain タイミングを慎重設計しないと OnSecondChange 経路を肥大化させる。

### Option B: 新規コンポーネント中心
- キック専用の talk FIFO サービスと Kick 実行器を独立モジュール化。
- **トレードオフ**: ✅ 責務分離・単体テスト容易。❌ アクター単一 mailbox／単一 consumer 不変条件と二重管理になりやすく、別チャンネル化は設計が明示的に禁ずる方向（mailbox docstring）。debug 経路の独立 transport 新設も R2.3 が禁ずる。

### Option C: ハイブリッド（現実解）
- transport／コマンド層は Option A（既存 DAP・mailbox variant merge）。
- talk FIFO ＋ drain ＋ preempt の中核ロジックは新規の薄いモジュールとして切り出し（Option B 的）、ただしアクター単一 mailbox 上に載せ別チャンネルを作らない。
- **トレードオフ**: ✅ 不変条件を守りつつ中核ロジックをテスト可能に分離。❌ 計画調整コストがやや高い。Kick 実行・FIFO・preempt の責務境界を design で明確化する必要。

## 4. 主要な統合課題（Integration Challenges）

1. **ライブキック経路 vs DAP 停止ループ（最重要・Unknown）**: 既存 `SessionCommand` はブレーク停止中（stop_loop）にのみ消費される。本機能はブレークなしのライブ実行中にキックする必要があるため、`playScene` を `SessionCommand` に足して stop_loop で処理する素朴案は不成立。inbound デコード（socket-bridge スレッド、停止非依存）から **アクター `static MAILBOX` へ `ActorMsg::Kick` を送る経路**を新設するのが筋。これが「debug backend をアクターのクライアント化（R2.4）」の実体。design で thread 越境（socket-bridge → flume `Sender` は `Send+Sync`、送信は可）と enable ゲートの整合を確定すること。
2. **talk FIFO の保持層（Lua か Rust か・Unknown）**: presentation→さくらスクリプト描画は Lua 集約。FIFO を Lua 側（`pasta_scripts`）に置くか、アクタースレッドの Rust 側に置くかで、drain・preempt・順序保証の実装位置が変わる。R4.1 の FIFO 順序と R5 の preempt close は単一 consumer 上で自然に表現できるが、層の選択で複雑度が変動。なお drain は抑制ゲート無し（即時再生オンリー）のため、保留・条件分岐は不要で「次の OnSecondChange で必ず drain」のみ。
3. **preempt-and-abort の `co_scene` close（Missing）**: 現状 `set_co_scene(co)` は状態保持のみで、進行中コルーチンを強制終了する API がない。さらに `MEMORY` の既知事項として **LuaJIT は `coroutine.close` 非搭載**（suspended を強制 dead にできない）。preempt は「参照 nil 化＋GC」でモデル化する必要があり、確実な「閉じた」観測契約（R5.2）を design でどう保証するかが課題。
4. **ctx 合成の流用（Constraint・低リスク）**: キックされたシーンは SHIORI イベント由来の `act` を持たないため、エンジンが ctx を合成する（R3.2）。**通常トーク再生の ctx 合成手順（`EVENT.fire` 系で行う act 生成→`SCENE.co_exec` 起動）をそのまま流用**する方針が確定済み（ユーザー判断 2026-06-23：「トーク再生時の ctx 合成と同じことをすればよい、難しくない」）。design では既存合成手順の再利用点（どの関数を共通化し、キック起点からどう呼ぶか）を特定する。キック専用の特別な ctx 構築・初期束縛は要件外。
5. **≤1 秒レイテンシ（R4.4）**: 実 SSP の OnSecondChange tick 周期依存。エンジン側は「次の GET で必ず drain」を保証するのみで、絶対時間はホスト依存——要件文言（「tick 周期に依存して概ね 1 秒以内」）と一致しており、エンジンの責務境界を design で明記。
6. **既存挙動不変の回帰検証（R6）**: キック未使用時に通常 SHIORI 応答がバイト不変であることを保証する特性化テストが要る（`shiori-event-test-framework` 既存基盤・PASTA_DEBUG ガード留意）。

## 5. 実装規模・リスク

| 区分 | 評価 | 根拠 |
| --- | --- | --- |
| 規模 | **L（1〜2 週間）** | 複数レイヤー横断（VSCode TS／DAP Rust／actor mailbox／Lua talk FIFO・preempt）。個々は前例ありだが結線とライブ経路設計が新規。 |
| リスク | **Medium〜High** | DAP 停止ループ外のライブキック経路（課題1）と preempt close（課題3・LuaJIT 制約）が未確立。transport／コマンド／ゲートは前例ありで Medium、ライブ経路と preempt は High。 |

## 6. design フェーズへの申し送り

- **推奨アプローチ**: Option C（既存 DAP・mailbox variant merge ＋ 中核ロジックの薄い分離）。
- **確定すべき主要判断**:
  1. ライブキック経路（socket-bridge inbound → `static MAILBOX` → `ActorMsg::Kick`）の thread 越境・enable ゲート整合。debug 無効時のキック経路非活性（R2.6）の表現。
  2. talk FIFO の保持層（Lua 側 `pasta_scripts` か Rust アクタースレッド側か）と drain 契約（OnSecondChange GET の内側で短く・抑制ゲート無しの無条件 drain）。
  3. preempt-and-abort の `co_scene` close 観測契約（LuaJIT `coroutine.close` 非搭載前提・参照 nil 化＋GC モデル）。
  4. ctx 合成の共通化点——通常トーク再生の ctx 合成（act 生成→`SCENE.co_exec`）をキック起点から流用するための再利用 IF（R3.2）。
  5. `playScene` 引数表現（シーン名のみ・即時単一モードゆえモードフラグ不要）と、inbound→mailbox 取り次ぎの結線。
- **Research Needed（design へ持ち越し）**:
  - socket-bridge スレッドから flume `Sender` 送信時のライフサイクル（teardown／reload 時の MAILBOX swap との競合）。
  - LuaJIT での「中断シーンを確実に再開不能化」する具体手段と検証方法。
  - キック由来出力と通常 OnSecondChange dispatch（OnHour/OnTalk）の drain 順序・共存ルール（キック側は無条件 drain・通常側は従来の `is_blocked()` 判定を維持）。

## 7. オープンクエスチョン（要件ディスカッションで解決済み・2026-06-23）

1. **即時／非即時モードの UI 既定値** → **解決: 即時再生オンリー**。非即時モードを廃止し、キックは常に即時 preempt-and-abort（現 R5）。モード選択 UI 自体が不要。
2. **debug backend 無効時のキック** → **解決: debug 有効が前提条件**（現 R2.6）。transport が既存 debug DAP チャネルを再利用するため。debug 非依存の常時キック経路は将来別境界。
3. **複数即時キックの連続** → **解決: 即時 preempt-and-abort の再帰適用**。後続キックも同様に進行中（＝前キック）を preempt する。単一 mailbox／単一 consumer 上で FIFO 順に処理（二重 preempt の特別契約は不要・design で自然表現）。
4. **キック対象シーンの指定粒度** → **解決: シーン名のみ**（現 R1/R2.2）。ただしシーンは ctx 無しに走らないため、**エンジンが通常トーク再生と同一手順で ctx（`act`）を合成**（現 R3.2）。UI からの引数／アクター指定は将来別境界。
5. **非即時キックの溜まり方** → **失効**: 非即時モード廃止により該当せず。キックは即時 drain のため滞留しない。
6. **`talking` 以外の抑制ステータス** → **解決: 抑制無し**。即時再生オンリーのためキック側に抑制集合の概念が無い（現 R4 無条件 drain）。`is_blocked()`/`BLOCKED_STATUSES` は通常イベント経路でのみ従来どおり使用。
