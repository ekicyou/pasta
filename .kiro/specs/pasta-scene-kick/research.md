# Gap Analysis: pasta-scene-kick

要件（requirements.md・R1〜R8）と既存コードベースの差分を分析し、実装戦略の判断材料を提示する。本書は決定ではなく情報・選択肢の提示であり、最終的なアーキテクチャ選定は design フェーズに委ねる。

前提: 本機能は `pasta-actor-runtime`（完了済み）と `pasta-vscode-lua-debug`（完了済み）の上に構築される。

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
| R1 VSCode キックコマンド | `extension.ts` `registerCommand`／`sourcePresentationToggle.ts`／`package.json` 前例 | **Constraint**（前例に倣う・拡張）|
| R2 transport 一般化（`playScene`）| `dap/decode.rs` custom request マッチ・`wiring/inbound.rs` 自己完結ハンドラ前例 | **Constraint**（前例あり）＋ **Unknown**（停止ループ外の経路設計）|
| R2.4 debug backend のアクタークライアント化 | debug は現状 VM スレッド内 hook／stop_loop 経由のみ。アクター mailbox への送信経路は未配線 | **Missing**（inbound→mailbox 配線が新規）|
| R3 非同期実行・レンダリング | `actor/thread.rs` executor ループ・`presentation`・`renderer_injection`・Lua `SCENE.co_exec` | **Missing**（Kick 実行ハンドラ）＋ 既存基盤再利用可 |
| R4 talk FIFO ＋ OnSecondChange drain | OnSecondChange ハンドラ・`RES.ok` はあるが **talk FIFO は皆無** | **Missing**（FIFO 新規）|
| R5 抑制ゲート（`Status: talking`）| `req.status` パース済み・`is_blocked()`/`has_status()` 既存 | **Constraint**（再利用・拡張）|
| R6 非即時アイドル待ち | アイドル判定軸（`is_blocked`）はあるが「待って吐く」キュー意味論なし | **Missing**（保留 drain ロジック）|
| R7 即時 preempt-and-abort | `set_co_scene(co)` で前シーン状態は保持されるが、**強制 close／中断 API はない** | **Missing**（preempt ＋ `co_scene` close）|
| R8 ライブ SSP・既存挙動不変 | レンダラはライブ SSP 直結・通常イベント経路は独立 | **Constraint**（追加経路ゆえ不変が自然・要回帰検証）|

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
2. **talk FIFO の保持層（Lua か Rust か・Unknown）**: presentation→さくらスクリプト描画は Lua 集約。FIFO を Lua 側（`pasta_scripts`）に置くか、アクタースレッドの Rust 側に置くかで、drain・preempt・順序保証の実装位置が変わる。R4.1 の FIFO 順序と R7 の preempt close は単一 consumer 上で自然に表現できるが、層の選択で複雑度が変動。
3. **preempt-and-abort の `co_scene` close（Missing）**: 現状 `set_co_scene(co)` は状態保持のみで、進行中コルーチンを強制終了する API がない。さらに `MEMORY` の既知事項として **LuaJIT は `coroutine.close` 非搭載**（suspended を強制 dead にできない）。preempt は「参照 nil 化＋GC」でモデル化する必要があり、確実な「閉じた」観測契約（R7.2）を design でどう保証するかが課題。
4. **≤1 秒レイテンシ（R4.4）**: 実 SSP の OnSecondChange tick 周期依存。エンジン側は「次の GET で必ず drain」を保証するのみで、絶対時間はホスト依存——要件文言（「tick 周期に依存して概ね 1 秒以内」）と一致しており、エンジンの責務境界を design で明記。
5. **既存挙動不変の回帰検証（R8）**: キック未使用時に通常 SHIORI 応答がバイト不変であることを保証する特性化テストが要る（`shiori-event-test-framework` 既存基盤・PASTA_DEBUG ガード留意）。

## 5. 実装規模・リスク

| 区分 | 評価 | 根拠 |
| --- | --- | --- |
| 規模 | **L（1〜2 週間）** | 複数レイヤー横断（VSCode TS／DAP Rust／actor mailbox／Lua talk FIFO・preempt）。個々は前例ありだが結線とライブ経路設計が新規。 |
| リスク | **Medium〜High** | DAP 停止ループ外のライブキック経路（課題1）と preempt close（課題3・LuaJIT 制約）が未確立。transport／コマンド／ゲートは前例ありで Medium、ライブ経路と preempt は High。 |

## 6. design フェーズへの申し送り

- **推奨アプローチ**: Option C（既存 DAP・mailbox variant merge ＋ 中核ロジックの薄い分離）。
- **確定すべき主要判断**:
  1. ライブキック経路（socket-bridge inbound → `static MAILBOX` → `ActorMsg::Kick`）の thread 越境・enable ゲート整合。debug 無効時のキック経路非活性（R2.6）の表現。
  2. talk FIFO の保持層（Lua 側 `pasta_scripts` か Rust アクタースレッド側か）と drain 契約（OnSecondChange GET の内側で短く）。
  3. preempt-and-abort の `co_scene` close 観測契約（LuaJIT `coroutine.close` 非搭載前提・参照 nil 化＋GC モデル）。
  4. 即時／非即時モードの transport 表現（`playScene` 引数）と抑制ゲート（`is_blocked()` 再利用）の結線。
- **Research Needed（design へ持ち越し）**:
  - socket-bridge スレッドから flume `Sender` 送信時のライフサイクル（teardown／reload 時の MAILBOX swap との競合）。
  - LuaJIT での「中断シーンを確実に再開不能化」する具体手段と検証方法。
  - キック由来出力と通常 OnSecondChange dispatch（OnHour/OnTalk）の drain 順序・共存ルール。

## 7. オープンクエスチョン（要件ディスカッションで解決）

1. **即時／非即時モードの UI 既定値**: VSCode のキックは既定でどちらか（例: ボタン＝非即時、修飾キー＝即時）。brief は両モード提供を求めるが既定の指定はない（R1.3）。【ドラフト前提: 両モードを明示選択可能とし、既定は design/discussion で確定】
2. **debug backend 無効時のキック**: brief は「既存 debug チャネル再利用」を前提とするため debug 無効時はキック不可（R2.6）と解釈した。debug 非依存の常時キック経路を将来求めるかは別境界候補。【ドラフト前提: debug 有効が前提条件】
3. **複数即時キックの連続**: 即時キック中にさらに即時キックが来た場合の挙動（現キックも preempt するか）。brief 未言及。【ドラフト前提: FIFO ＋単一 consumer ゆえ後続も同様に処理されるが、二重 preempt の明示契約は未定】
4. **キック対象シーンの指定粒度**: シーン名のみか、引数／アクター指定も許すか。brief は「シーンを指名」とのみ。【ドラフト前提: シーン名指名のみを R に記載、引数は design スコープ】
5. **非即時キックの溜まり方**: 会話が長引き複数の非即時キックが滞留した場合、アイドル時に全件吐くか上限を設けるか。brief 未言及。【ドラフト前提: FIFO 全件をアイドルで順次吐く】
6. **`talking` 以外の抑制ステータス**: 既存 `BLOCKED_STATUSES` には `choosing` 等も含まれる。非即時キックの抑制を `talking` のみに限るか既存セット全体に合わせるか。brief は `talking` を例示。【ドラフト前提: R は `talking` を権威例として記載・実際の抑制集合は既存 `is_blocked()` 準拠で design 確定】
