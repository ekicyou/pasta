# Technical Design: pasta-scene-kick

## Overview

**Purpose**: ゴースト作者がオーサリング／デバッグ中に「任意のシーンを今すぐライブ SSP 上で再生して観る」を実現する。VSCode 拡張のコマンドからシーン名を指名してキックすると、本物のゴーストが実 SSP 上で ≤1 秒（実 SSP の tick 周期依存）で反応する。

**Users**: ゴースト作者（`*.pasta` 辞書のオーサリング／デバッグ担当）。VSCode 上で pasta デバッグセッションに attach した状態でキックコマンドを実行する。

**Impact**: 既存の `pasta-actor-runtime`（アクタースレッド＋mailbox marshaling）と `pasta-vscode-lua-debug`（DAP-over-TCP）の上に、**キック追加経路**を載せる。SHIORI/3.0 の pull 契約（OnSecondChange の GET 機会）を破らず、キックされたシーンをアクタースレッドで非同期にレンダリングし talk FIFO に積み、OnSecondChange で無条件 drain してライブ SSP へ届ける。通常 SHIORI 会話挙動は不変（キックは追加経路）。

### Goals
- VSCode コマンドからシーン名を指名し、既存 debug DAP チャネル（`playScene` custom request）でエンジンへ運ぶ。
- キックされたシーンをアクタースレッド上で非同期に ctx 合成・レンダリングし talk FIFO へ投入する（GET ブロックを延ばさない）。
- OnSecondChange で talk FIFO を**無条件 drain**（抑制ゲート無し）し、ライブ SSP へ反映する。
- 会話中でも問答無用で**即時 preempt-and-abort**（進行中会話を中断・前 `co_scene` を閉じ自動復帰しない）。
- キック未使用時の通常 SHIORI 応答を**バイト不変**に保つ。

### Non-Goals
- 非即時（アイドル待ち）キックモード、SSP `Status: talking` を権威とする抑制ゲート（即時再生オンリーのため不採用）。
- 即時キックされた会話の退避→復帰セマンティクス（採用は preempt-and-abort のみ）。
- VSCode／キック要求からのシーン引数・アクター指定（ctx 合成はエンジン既定・UI 引数は将来別境界）。
- ライブ SSP 以外の出力先（別プレビュー画面）、SSTP / `\![raise]` 押し出し、`*.pasta` 編集ウィンドウからのキック、`pasta_novel` アダプタ。

## Boundary Commitments

### This Spec Owns
- **キック transport の一般化**: 既存 debug DAP チャネルへ `playScene` custom request を追加（decode・配線）。
- **kick sink seam**: `pasta_lua` の `debug::enable` に汎用キック注入口を新設し、`pasta_shiori` が `MAILBOX` 投函クロージャを注入する配線。
- **`ActorMsg::Kick` variant**: 同一 mailbox への variant merge（予約済み）と executor ループの実行ハンドラ。
- **キック起点の ctx 合成呼び出し**: 通常トーク再生の合成手順（`create_act`→`SCENE.co_exec`→`resume_until_valid`→`set_co_scene`）をキック起点から流用する結線。
- **talk FIFO（`TALK_QUEUE`）**: キック由来さくらスクリプトの順序保持蓄積と OnSecondChange での無条件 drain。
- **即時 preempt-and-abort**: 前 `co_scene` 参照の破棄（自動復帰なし）。
- **VSCode キックコマンド／シーン名入力 UI**。

### Out of Boundary
- ctx 合成手順そのものの実装（`create_act`/`SCENE.co_exec` 等は既存・本機能は呼ぶだけ）。
- アクター mailbox marshaling 本体（GET/NOTIFY・teardown）＝`pasta-actor-runtime` 所有。
- DAP transport の TCP/フレーミング・停止状態機械＝`pasta-vscode-lua-debug` 所有。
- レンダラ（さくらスクリプト描画）本体＝既存 `renderer_injection` 所有。
- 通常イベント経路の抑制ゲート（`is_blocked()`/`BLOCKED_STATUSES`）＝OnHour/OnTalk で従来どおり機能（本機能は触れない）。
- 非即時モード・抑制待ち・退避復帰・SSTP 押し出し・別プレビュー（全て Non-Goals）。

### Allowed Dependencies
- **`pasta-actor-runtime`（上流・同一クレート `pasta_shiori`）**: `ActorMsg`/`static MAILBOX`/executor ループ/`PastaShiori`/`RuntimeConfig` 透過。
- **`pasta-vscode-lua-debug`（上流・`pasta_lua` debug backend）**: `debug::enable`/`dap::decode`/`wiring::inbound`/socket-bridge スレッド/`pasta/sourcePresentation` 前例。
- **既存 Lua dispatch（`pasta_scripts`）**: `EVENT.fire`/`create_act`/`SCENE.co_exec`/`set_co_scene`/`second_change.lua`。
- **クレート依存方向の不変条件**: `pasta_shiori` → `pasta_lua` の一方向のみ。`pasta_lua` 側コードは `pasta_shiori::MAILBOX` を直接参照してはならない（kick sink は型を知らない汎用 seam）。

### Revalidation Triggers
- `ActorMsg` の variant 形状変更（`Kick` payload 追加・改名）→ executor ループ・marshaling の再検証。
- `KickSink` 型・`debug::enable` シグネチャ変更 → `pasta_shiori` 注入側・socket-bridge 呼び出し側の再検証。
- `playScene` custom request の payload スキーマ変更 → VSCode TS 側 `setPayload`/`decode.rs` の同期。
- ctx 合成手順（`create_act`/`SCENE.co_exec`/`set_co_scene`）の契約変更 → キック起点の流用結線の再検証。
- OnSecondChange ハンドラ（`second_change.lua`）の応答形状変更 → drain hook・バイト不変回帰テストの再検証。

## Architecture

### Existing Architecture Analysis

- **アクター単一 mailbox / 単一 consumer 不変条件**: `ActorMsg`（`#[non_exhaustive]`）は flume unbounded・FIFO・単一 consumer。別チャンネル＋`select!` は flume cancel 欠陥回避のため**禁止**（mailbox docstring が `Kick` を variant merge で予約）。
- **VM pin（`!Send`）**: Lua VM はアクタースレッドに pin され越境しない。キックの ctx 合成・レンダリングは必ずアクタースレッド上で行う。
- **ゼロコスト debug**: `DebugConfig` 既定 `enabled=false`／`listen=None`。無効時は port・hook・thread を起こさない。キック経路は debug 有効が前提（R2.6）。
- **DAP 停止ループ制約**: `SessionCommand` はブレーク停止中（stop_loop）のみ消費。ライブ実行中は stop_loop に入らないため、ライブキックは停止非依存の socket-bridge inbound から運ぶ必要がある。
- **クレート依存方向**: `pasta_shiori`（`MAILBOX` 所有）→ `pasta_lua`（debug backend 所有）。逆参照は不可。kick sink はこの制約を満たす汎用注入口とする。

### Architecture Pattern & Boundary Map

選定パターン: **Client-injected sink seam ＋ mailbox variant merge ＋ Lua-side FIFO**（research.md Option C）。VSCode→DAP→sink→mailbox→アクタースレッド ctx 合成→Lua FIFO→OnSecondChange drain→ライブ SSP の単方向追加経路。

```mermaid
graph TB
    subgraph VSCode
        Cmd[playScene command]
        Input[scene name input box]
    end
    subgraph PastaLua[pasta_lua debug backend upstream]
        Decode[dap decode playScene]
        SocketBridge[socket bridge inbound thread]
        KickSinkSeam[KickSink injection seam]
        Enable[debug enable]
    end
    subgraph PastaShiori[pasta_shiori actor runtime downstream]
        Inject[MAILBOX dispatch closure]
        Mailbox[static MAILBOX]
        Executor[actor thread executor loop]
    end
    subgraph LuaVM[Lua VM on actor thread]
        KickHandler[kick exec handler]
        CtxSynth[ctx synthesis reuse]
        Renderer[sakura renderer]
        TalkQueue[TALK_QUEUE fifo]
        SecondChange[OnSecondChange drain]
    end
    LiveSSP[Live SSP]

    Cmd --> Input
    Input --> Decode
    Decode --> SocketBridge
    SocketBridge --> KickSinkSeam
    Inject --> KickSinkSeam
    KickSinkSeam --> Mailbox
    Enable --> KickSinkSeam
    Mailbox --> Executor
    Executor --> KickHandler
    KickHandler --> CtxSynth
    CtxSynth --> Renderer
    Renderer --> TalkQueue
    SecondChange --> TalkQueue
    SecondChange --> LiveSSP
```

**Architecture Integration**:
- **Selected pattern**: client-injected sink で依存方向を順守しつつ debug backend をアクターのクライアント化（R2.4）。
- **Domain/feature boundaries**: transport/コマンド（`pasta_lua` debug）／sink 注入（`pasta_shiori`）／FIFO・preempt・ctx 流用（Lua `pasta_scripts`）を層で分離。
- **Existing patterns preserved**: 単一 mailbox・単一 consumer・ゼロコスト debug・`pasta/sourcePresentation` 前例・VM pin。
- **New components rationale**: `KickSink`（クレート越境を疎結合化）／`ActorMsg::Kick`（予約済み variant）／`TALK_QUEUE`（FIFO は内製ゼロのため新設）。
- **Steering compliance**: バイト不変（既存リリース DLL の挙動不変）・送信パスに Mutex を置かない（lock-free `MAILBOX.load_full`）。

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Frontend / CLI | VSCode 拡張（TypeScript・既存） | キックコマンド・シーン名入力・customRequest 送信 | `pasta/sourcePresentation` 前例を逐語ミラー。`showInputBox` は新規（標準 API） |
| Backend / Services | Rust（`pasta_lua` debug ＋ `pasta_shiori` actor・既存クレート） | `playScene` decode・kick sink seam・`ActorMsg::Kick`・executor ハンドラ | 新規外部依存ゼロ。flume / arc-swap は既存 |
| Messaging / Events | flume mailbox（既存）＋ `KickSink` クロージャ（新規） | DAP→sink→`MAILBOX`→アクタースレッド | 別チャンネル新設なし（R2.3）。単一 consumer 不変条件継承 |
| Data / Storage | Lua `TALK_QUEUE`（新規・in-memory FIFO） | キック由来さくらスクリプトの順序保持 | アクタースレッド VM 内・session スコープ |
| Infrastructure / Runtime | アクタースレッド（`wintf_winmsg_executor`・既存） | ctx 合成・レンダリングの実行コンテキスト | VM pin（`!Send`）。キック処理は全てこの上 |

## File Structure Plan

### Modified Files

**Rust — `pasta_lua`（debug backend・上流）**
- `crates/pasta_lua/src/debug/dap/decode.rs` — `"pasta/playScene"` 文字列マッチを追加。`args.scene`（文字列）を抽出し `Decoded` に詰める。空/欠落は `None`（不正）として扱う。
- `crates/pasta_lua/src/debug/wiring/inbound.rs` — `handle_inbound` に playScene 自己完結ハンドラ（`try_play_scene_kick()`）を `pasta/sourcePresentation` と同列で追加。decode 済みシーン名を `KickSink` へ渡し即応答（成功 ack／空名エラー）。汎用 routing（stop_loop 経由）に**落とさない**。
- `crates/pasta_lua/src/debug/enable.rs` — `enable(...)` に `kick_sink: Option<KickSink>` 引数を追加。socket-bridge spawn 時に sink を `run_socket_bridge` へ渡す。`enabled=false` 時は sink 未使用（経路非活性・R2.6）。
- `crates/pasta_lua/src/debug/wiring/mod.rs`（または socket-bridge 配線元） — `run_socket_bridge(...)` に `kick_sink` を追加し inbound ハンドラへ供給。
- `crates/pasta_lua/src/debug/config.rs` もしくは新規 `kick.rs` — `KickSink` 型エイリアス（`type KickSink = std::sync::Arc<dyn Fn(KickRequest) + Send + Sync>;`）と `KickRequest { scene: String }` を定義。`pasta_lua` は中身（`ActorMsg`/`MAILBOX`）を知らない。
- `crates/pasta_lua/src/runtime/runtime_config.rs` — `RuntimeConfig` に `kick_sink: Option<KickSink>` を builder（`with_kick_sink`）で持たせる。
- `crates/pasta_lua/src/runtime/mod.rs` — `with_config_and_source_map` の `debug::enable(...)` 呼び出しへ `config.kick_sink.clone()` を透過。

**Rust — `pasta_shiori`（actor runtime・下流）**
- `crates/pasta_shiori/src/actor/mailbox.rs` — `ActorMsg` に `Kick { scene: String }` variant を追加（予約 docstring を実体化）。
- `crates/pasta_shiori/src/actor/thread.rs` — executor `match msg` に `ActorMsg::Kick { scene }` 腕を追加。アクタースレッド上で `shiori` 経由 Lua のキック実行入口（後述 `SHIORI.kick(scene)` 相当）を呼ぶ。reply 無し（fire-and-forget）。
- `crates/pasta_shiori/src/actor/lifecycle.rs`（または VM 構築点） — VM 構築前に `RuntimeConfig` へ `MAILBOX` 投函クロージャ（`|req| if let Some(tx)=MAILBOX.load_full() { let _ = tx.try_send(ActorMsg::Kick{scene:req.scene}); }`）を `with_kick_sink` で束縛する。
- `crates/pasta_shiori/src/shiori.rs`（VM 入口） — Lua の `SHIORI.kick(scene)` を呼ぶ Rust 側入口（GET/NOTIFY と同列の薄いブリッジ）。

**Lua — `pasta_lua/pasta_scripts`**
- `crates/pasta_lua/pasta_scripts/pasta/shiori/talk_queue.lua` —（新規）`TALK_QUEUE.enqueue(sakura)` / `TALK_QUEUE.drain() -> sakura|nil` / `TALK_QUEUE.is_empty()`。session スコープ in-memory FIFO。
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/kick.lua` —（新規）キック実行ハンドラ `KICK.exec(scene_name)`: ctx 合成（`create_act` 流用）→ preempt（`set_co_scene` で前シーン破棄）→ `SCENE.co_exec`→`resume_until_valid`→レンダリング結果を `TALK_QUEUE.enqueue`。解決不能シーンは何も積まず診断ログ（R3.5）。
- `crates/pasta_lua/pasta_scripts/pasta/shiori/event/second_change.lua` — `CALLBACK.sweep()` 後・`dispatcher.dispatch(act)` 前に `TALK_QUEUE.drain()` を呼び、非 nil なら**抑制ゲート無しで** GET 応答として返す。空なら従来 dispatch へ素通り（バイト不変・R6.3）。
- `crates/pasta_lua/pasta_scripts/pasta/shiori/init.lua`（SHIORI ディスパッチ登録元） — `SHIORI.kick(scene)` を公開し `KICK.exec` へ委譲。

**TypeScript — `editors/vscode`**
- `editors/vscode/src/playSceneRequest.ts` —（新規）`requestCommand = 'pasta/playScene'`、`setPayload(scene: string): { scene: string }`、`validateSceneName(name): boolean`（空チェック）。`sourcePresentationToggle.ts` の純ロジック構造をミラー。
- `editors/vscode/src/extension.ts` — `registerCommand('pasta.debug.playScene', ...)`: `isPastaSession()` ガード（未接続なら案内）→`showInputBox` でシーン名取得（取消なら送信しない・R1.4）→`session.customRequest('pasta/playScene', setPayload(scene))`→try/catch でエラー提示（R1.3/R2.5）。
- `editors/vscode/package.json` — `contributes.commands` に `pasta.debug.playScene`、`contributes.menus`（`commandPalette`/`debug/toolBar`・`when: debugType == 'pasta'`）を追加。

### New Files Summary
- Rust: `KickSink`/`KickRequest` 定義（`pasta_lua/src/debug/kick.rs` 想定）。
- Lua: `talk_queue.lua`、`event/kick.lua`。
- TS: `playSceneRequest.ts`、`test/playSceneRequest.test.ts`。

## System Flows

### キック起動〜ライブ SSP 反映（ライブ経路・停止ループ迂回）

```mermaid
sequenceDiagram
    participant Author
    participant VSCode
    participant SocketBridge as socket-bridge inbound thread
    participant Sink as KickSink closure
    participant Mailbox as static MAILBOX
    participant Actor as actor thread executor
    participant Lua as Lua VM kick handler
    participant Queue as TALK_QUEUE
    participant SSP as Live SSP

    Author->>VSCode: playScene command
    VSCode->>VSCode: showInputBox scene name
    VSCode->>SocketBridge: customRequest pasta/playScene scene
    SocketBridge->>SocketBridge: decode + validate scene
    alt scene empty or invalid
        SocketBridge-->>VSCode: error response (R2.5)
    else valid
        SocketBridge->>Sink: invoke KickRequest scene
        SocketBridge-->>VSCode: ack response
        Sink->>Mailbox: load_full + try_send ActorMsg Kick
        Mailbox->>Actor: FIFO deliver Kick
        Actor->>Lua: SHIORI.kick scene
        Lua->>Lua: ctx synthesis (create_act reuse)
        Lua->>Lua: preempt prev co_scene (set_co_scene)
        Lua->>Lua: SCENE.co_exec + resume + render
        alt scene unresolved
            Lua->>Lua: drop + diagnostic log (R3.5)
        else rendered
            Lua->>Queue: enqueue sakura
        end
    end
    Note over SSP,Queue: 別 tick の OnSecondChange GET
    SSP->>Lua: OnSecondChange GET
    Lua->>Queue: drain (unconditional, no status gate)
    alt queue non-empty
        Lua-->>SSP: sakura output (R4.2)
    else empty
        Lua-->>SSP: normal dispatch response (byte-invariant R6.3)
    end
```

**Flow-level decisions**:
- **停止ループ迂回**: sink 呼び出しは socket-bridge スレッド（停止非依存）で行い、`SessionCommand`/stop_loop を経由しない（ライブ実行中もキック可能）。
- **非同期分離**: キックの ctx 合成・レンダリングはアクタースレッド上で `ActorMsg::Kick` 受信後に行い、OnSecondChange GET の内側で同期実行しない（R3.4・GET を短く保つ）。
- **無条件 drain**: OnSecondChange は `is_blocked` を介さず `TALK_QUEUE` を drain（R4.2）。空なら従来応答へ素通り（R6.3）。

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1 | VSCode がシーン名を受け取る | PlaySceneCommand | `showInputBox` | キック起動 |
| 1.2 | シーン名をデバッグチャネルで送信 | PlaySceneCommand / PlaySceneRequest | `customRequest('pasta/playScene')` | キック起動 |
| 1.3 | 未接続時はエラー／案内 | PlaySceneCommand | `isPastaSession()` ガード | キック起動 alt |
| 1.4 | 取消時は送信しない | PlaySceneCommand | `showInputBox` 取消分岐 | キック起動 |
| 2.1 | `playScene` custom request 受理 | PlaySceneDecode | `decode_request` マッチ | キック起動 |
| 2.2 | シーン名抽出 | PlaySceneDecode | `args.scene` 抽出 | キック起動 |
| 2.3 | 別 transport 新設せず既存拡張 | PlaySceneDecode / KickInboundHandler | 既存 DAP チャネル | （構造制約） |
| 2.4 | debug backend をアクタークライアント化・取次 | KickSinkSeam / MailboxKickInjector | `KickSink` / `MAILBOX.try_send` | キック起動 |
| 2.5 | 空/不正シーン名はエラー返却 | PlaySceneDecode / KickInboundHandler | validate→error response | キック起動 alt |
| 2.6 | debug 無効時は経路非活性 | KickSinkSeam | sink 未注入（`enabled=false`） | （ゲート） |
| 3.1 | 非同期実行（GET を待たせない） | ActorKickMsg / KickExecHandler | `ActorMsg::Kick`（fire-and-forget） | キック起動 |
| 3.2 | ctx 合成を通常再生から流用 | KickExecHandler | `create_act`/`SCENE.co_exec`/`resume_until_valid` | キック起動 |
| 3.3 | レンダリング結果を FIFO 投入 | KickExecHandler / TalkQueue | `TALK_QUEUE.enqueue` | キック起動 |
| 3.4 | GET 内で同期レンダリングしない | ActorKickMsg | アクタースレッド分離 | キック起動 |
| 3.5 | 未解決シーンは破棄＋診断記録 | KickExecHandler | drop + `tracing`/log | キック起動 alt |
| 4.1 | FIFO 順序保持 | TalkQueue | `enqueue`/`drain` | drain |
| 4.2 | OnSecondChange 無条件 drain | SecondChangeDrain / TalkQueue | `TALK_QUEUE.drain()` | drain |
| 4.3 | 空 FIFO 時は通常応答 | SecondChangeDrain | drain=nil→従来経路 | drain alt |
| 4.4 | ≤1 秒（tick 周期依存）で SSP 反映 | SecondChangeDrain | 次 GET で必ず drain | drain |
| 4.5 | drain は pull 機会限定・押し出ししない | SecondChangeDrain | OnSecondChange のみ | drain |
| 5.1 | 会話中でも抑制ゲート無く preempt | KickExecHandler | `set_co_scene` 置換 | キック起動 |
| 5.2 | 中断側の前 `co_scene` を閉じる | KickPreempt | 前 `co_scene` 参照破棄 | キック起動 |
| 5.3 | 自動復帰しない | KickPreempt | 退避スタック非保持 | キック起動 |
| 5.4 | 即時単一モード（非即時提供しない） | KickExecHandler | モードフラグ無し | （構造制約） |
| 6.1 | ライブ SSP 反映・別プレビュー無し | SecondChangeDrain | 既存レンダラ直結 | drain |
| 6.2 | 通常会話挙動を不変 | SecondChangeDrain | 追加経路（独立） | drain alt |
| 6.3 | キック未使用時はバイト不変 | SecondChangeDrain | drain=nil 素通り | drain alt |
| 6.4 | `Status` ヘッダ準拠維持 | SecondChangeDrain | 既存解釈不変 | drain |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies (P0/P1) | Contracts |
|-----------|--------------|--------|--------------|--------------------------|-----------|
| PlaySceneCommand | VSCode TS | コマンド登録・シーン名入力・送信 | 1.1, 1.2, 1.3, 1.4 | activeDebugSession (P0), PlaySceneRequest (P1) | Service |
| PlaySceneRequest | VSCode TS | 純ロジック（command 名・payload・validate） | 1.2, 2.5 | — | Service |
| PlaySceneDecode | Rust debug dap | `playScene` decode・scene 抽出・validate | 2.1, 2.2, 2.3, 2.5 | DapAdapter (P0) | Service |
| KickInboundHandler | Rust debug wiring | 自己完結ハンドラ・sink 呼び出し・即応答 | 2.3, 2.5 | KickSinkSeam (P0) | Service |
| KickSinkSeam | Rust debug enable/kick | 汎用 `KickSink` 注入口（依存方向順守） | 2.4, 2.6 | RuntimeConfig (P0) | Service, State |
| MailboxKickInjector | Rust pasta_shiori | `MAILBOX` 投函クロージャ注入 | 2.4 | static MAILBOX (P0) | Service |
| ActorKickMsg | Rust pasta_shiori | `ActorMsg::Kick` variant＋executor 腕 | 3.1, 3.4 | mailbox (P0) | State |
| KickExecHandler | Lua | ctx 合成流用→レンダリング→enqueue | 3.2, 3.3, 3.5, 5.1, 5.4 | create_act/SCENE.co_exec (P0), TalkQueue (P0) | Service |
| KickPreempt | Lua | 前 `co_scene` 破棄・自動復帰なし | 5.2, 5.3 | set_co_scene (P0) | State |
| TalkQueue | Lua | FIFO 蓄積・drain | 3.3, 4.1, 4.2 | — | State |
| SecondChangeDrain | Lua | OnSecondChange 無条件 drain・素通り | 4.2, 4.3, 4.4, 4.5, 6.1, 6.2, 6.3, 6.4 | TalkQueue (P0), dispatcher (P1) | Service |

### VSCode TS Layer

#### PlaySceneRequest

| Field | Detail |
|-------|--------|
| Intent | `playScene` の command 名・payload・シーン名検証を純関数として提供 |
| Requirements | 1.2, 2.5 |

**Responsibilities & Constraints**
- `requestCommand = 'pasta/playScene'`（Rust decode 文字列と一致が契約）。
- payload は `{ scene: string }` のみ（モードフラグ無し・即時単一）。
- 空文字／空白のみのシーン名は invalid（送信前検証）。

**Contracts**: Service [x] / API [ ] / Event [ ] / Batch [ ] / State [ ]

##### Service Interface
```typescript
export const requestCommand = 'pasta/playScene' as const;
export function setPayload(scene: string): { scene: string };
export function validateSceneName(name: string): boolean; // trims; false if empty
```
- Preconditions: `name` は UI 入力文字列。
- Postconditions: `setPayload` は `{ scene }` を返す。`validateSceneName` は非空のとき true。
- Invariants: command 文字列は Rust 側 decode と完全一致。

**Implementation Notes**
- Integration: `sourcePresentationToggle.ts` の純ロジック構造を逐語ミラー。
- Validation: 単体テスト（`playSceneRequest.test.ts`）で payload／validate を検証。
- Risks: 低（前例網羅）。

#### PlaySceneCommand

| Field | Detail |
|-------|--------|
| Intent | VSCode コマンド登録・セッションガード・シーン名入力・customRequest 送信 |
| Requirements | 1.1, 1.2, 1.3, 1.4 |

**Responsibilities & Constraints**
- `registerCommand('pasta.debug.playScene')`。
- `isPastaSession(activeDebugSession)` が false なら警告提示し送信しない（R1.3）。
- `showInputBox` でシーン名取得。取消（undefined）なら送信しない（R1.4）。
- `session.customRequest(requestCommand, setPayload(scene))` を try/catch で囲み失敗を `showErrorMessage`（R2.5 のバックエンドエラー含む）。

**Dependencies**
- Inbound: VSCode command palette / debug toolbar — トリガ（P0）
- Outbound: `activeDebugSession.customRequest` — DAP 送信（P0）
- External: `vscode.window.showInputBox` — シーン名入力（P1・新パターン）

**Contracts**: Service [x]

**Implementation Notes**
- Integration: `extension.ts` L150-219 の toggle 登録と同型。`package.json` に command＋menu（`when: debugType == 'pasta'`）。
- Validation: モック session（`vscodeModules.test.ts` 流）で送信 payload と未接続／失敗時メッセージを検証。
- Risks: `showInputBox` は既存未使用だが標準 API。

### Rust debug backend Layer（pasta_lua・上流）

#### PlaySceneDecode

| Field | Detail |
|-------|--------|
| Intent | `pasta/playScene` を decode しシーン名を抽出・検証 |
| Requirements | 2.1, 2.2, 2.3, 2.5 |

**Responsibilities & Constraints**
- `decode_request` に `"pasta/playScene"` 文字列マッチを追加（`pasta/sourcePresentation` と同列）。
- `args.scene`（文字列）を抽出。欠落／空は `None`（invalid）として表現。
- 別 transport を新設せず既存 DAP frame の拡張（R2.3）。

**Contracts**: Service [x]

##### Service Interface
```rust
// decode.rs 内 match 腕（擬似シグネチャ）
"pasta/playScene" => Decoded {
    kick_scene: parse_scene_strict(args), // Option<String>: None when empty/missing
    ..Decoded::default()
}
```
- Preconditions: inbound JSON frame（socket-bridge 由来）。
- Postconditions: 有効時 `kick_scene = Some(name)`、無効時 `None`。
- Invariants: `Decoded` の他フィールドは default（routing に落とさない）。

**Implementation Notes**
- Integration: `Decoded` に `kick_scene: Option<String>` フィールド追加。
- Validation: decode 単体テスト（空名→None・正常→Some）。
- Risks: 低（前例どおり）。

#### KickInboundHandler

| Field | Detail |
|-------|--------|
| Intent | playScene を自己完結処理し sink を呼ぶ・即応答 |
| Requirements | 2.3, 2.5 |

**Responsibilities & Constraints**
- `handle_inbound` の固定順に `try_play_scene_kick()` を追加（`try_source_presentation_toggle` と同列・汎用 routing に落とさない）。
- `kick_scene = Some(name)` かつ sink あり → sink 呼び出し＋成功 ack。
- `None`（空/不正）→ キック実行せずエラー応答（R2.5）。
- sink が None（debug 無効・未注入）→ 経路非活性（R2.6 と整合）。

**Dependencies**
- Inbound: socket-bridge inbound decode — トリガ（P0）
- Outbound: KickSinkSeam（`KickSink`）— キック取次（P0）

**Contracts**: Service [x]

**Implementation Notes**
- Integration: `wiring/inbound.rs` の A→B→C→D→E 系に自己完結ハンドラを 1 つ追加。
- Validation: sink をモックし「有効名→sink 1 回呼ばれる」「空名→sink 呼ばれずエラー応答」。
- Risks: ハンドラ順序の誤りで routing に漏れる→順序テストで担保。

#### KickSinkSeam

| Field | Detail |
|-------|--------|
| Intent | `pasta_lua` 側の汎用キック注入口（中身を知らない） |
| Requirements | 2.4, 2.6 |

**Responsibilities & Constraints**
- `KickSink = Arc<dyn Fn(KickRequest) + Send + Sync>`、`KickRequest { scene: String }` を `pasta_lua` に定義。
- `enable(lua, cfg, source_map, kick_sink)` で受け取り socket-bridge へ供給。
- `enabled=false` 時は sink を消費せず経路非活性（ゼロコスト・R2.6）。
- `pasta_lua` は sink の実体（`ActorMsg`/`MAILBOX`）を知らない（依存方向順守）。

**Dependencies**
- Inbound: RuntimeConfig（`kick_sink`）— 注入（P0）
- Outbound: KickInboundHandler — 供給（P0）

**Contracts**: Service [x] / State [x]

##### Service Interface
```rust
pub type KickSink = std::sync::Arc<dyn Fn(KickRequest) + Send + Sync>;
pub struct KickRequest { pub scene: String }

// enable シグネチャ拡張
pub fn enable(
    lua: &mlua::Lua,
    cfg: &DebugConfig,
    source_map: Option<Arc<SourceMap>>,
    kick_sink: Option<KickSink>,   // 追加
) -> Result<Option<DebugHandle>, DebugError>;
```
- Preconditions: `enabled=true` のとき sink が `Some` なら経路有効。
- Postconditions: socket-bridge が inbound playScene で sink を呼ぶ。
- Invariants: sink は `Send + Sync`（スレッド越境可）。`pasta_lua` は型を不透明に扱う。

**Implementation Notes**
- Integration: `RuntimeConfig.with_kick_sink` → `with_config_and_source_map` → `enable` → `run_socket_bridge`。
- Validation: enable 有効＋sink Some で inbound→sink 到達、`enabled=false` で sink 非消費。
- Risks: 配線段増による注入忘れ→型で Option を明示し未注入=非活性で安全側。

#### MailboxKickInjector

| Field | Detail |
|-------|--------|
| Intent | `pasta_shiori` 側で `MAILBOX` 投函クロージャを sink として注入 |
| Requirements | 2.4 |

**Responsibilities & Constraints**
- VM 構築前に `RuntimeConfig.with_kick_sink(closure)` を束縛。
- クロージャ: `move |req: KickRequest| { if let Some(tx) = MAILBOX.load_full() { let _ = tx.try_send(ActorMsg::Kick { scene: req.scene }); } }`。
- teardown/reload 競合は `load_full()=None` で no-op（D6）。`try_send` 失敗（満杯/切断）は診断ログ後に破棄（デバッグ用途で許容）。

**Dependencies**
- Inbound: KickSinkSeam（`KickSink` 型）— 注入先（P0）
- Outbound: static MAILBOX — `ActorMsg::Kick` 送信（P0）

**Contracts**: Service [x]

**Implementation Notes**
- Integration: アクタースレッド spawn 時の VM 構築（`RuntimeConfig` 生成箇所）で束縛。
- Validation: `MAILBOX` 設定時→`Kick` 到達、`swap(None)` 後→no-op。
- Risks: クロージャが `MAILBOX` の `'static` 寿命に依存（`static` なので満たす）。

#### ActorKickMsg

| Field | Detail |
|-------|--------|
| Intent | `ActorMsg::Kick` variant と executor 実行腕 |
| Requirements | 3.1, 3.4 |

**Responsibilities & Constraints**
- `ActorMsg`（`#[non_exhaustive]`）に `Kick { scene: String }` を追加（予約 docstring 実体化）。
- executor `match msg` に腕を追加し、アクタースレッド上で `SHIORI.kick(scene)` を呼ぶ。reply 無し（fire-and-forget・NOTIFY 同型）。
- FIFO 順序を継承（複数キックは順に処理）。

**Dependencies**
- Inbound: MailboxKickInjector（`Kick` 送信）— 取次（P0）
- Outbound: KickExecHandler（Lua `SHIORI.kick`）— 実行（P0）

**Contracts**: State [x]

##### State Management
- State model: mailbox の 1 メッセージ（`Kick`）。VM 状態（`STORE.co_scene`/`TALK_QUEUE`）はアクタースレッド VM 内。
- Persistence & consistency: VM session スコープ。FIFO 一貫性は単一 consumer が担保。
- Concurrency strategy: 単一 consumer・単一 mailbox（`select!` 不使用）。

**Implementation Notes**
- Integration: `thread.rs` の `match` に 1 腕。Lua 入口は `shiori.rs` の薄いブリッジ。
- Validation: `Kick` 受信で `SHIORI.kick` が 1 回呼ばれる。複数 `Kick` が FIFO 順。
- Risks: 低（GET/NOTIFY と同型）。

### Lua Layer（pasta_scripts）

#### KickExecHandler

| Field | Detail |
|-------|--------|
| Intent | キック実行: ctx 合成流用→preempt→レンダリング→enqueue |
| Requirements | 3.2, 3.3, 3.5, 5.1, 5.4 |

**Responsibilities & Constraints**
- `KICK.exec(scene_name)`: 通常トーク再生の合成手順を流用 —
  1. `act = create_act(kick_req)`（`kick_req` は最小合成。例 `{ id = "OnKickScene" }`）。
  2. preempt: 進行中会話があれば前 `co_scene` を破棄（KickPreempt・`set_co_scene` 経由）。
  3. `co = SCENE.co_exec(act, scene_name)`。未解決なら何も積まず診断ログして return（R3.5）。
  4. `ok, yielded = resume_until_valid(co, act)`。
  5. `set_co_scene(co)`（成功時の状態保存／前シーン置換）。
  6. レンダリング結果（さくらスクリプト）を `TALK_QUEUE.enqueue(sakura)`（R3.3）。
- 即時単一モード（モードフラグ無し・R5.4）。抑制ゲート（`is_blocked`）を**呼ばない**（R5.1）。

**Dependencies**
- Inbound: ActorKickMsg（`SHIORI.kick`）— 起動（P0）
- Outbound: create_act/SCENE.co_exec/resume_until_valid/set_co_scene（既存・流用）— ctx 合成（P0）、TalkQueue — 投入（P0）

**Contracts**: Service [x]

##### Service Interface
```
KICK.exec(scene_name: string) -> nil   -- 副作用: TALK_QUEUE enqueue or drop+log
```
- Preconditions: アクタースレッド VM 上で呼ばれる（`!Send` 制約）。
- Postconditions: 解決成功→`TALK_QUEUE` に 1 件 enqueue。未解決→enqueue 無し＋診断ログ。
- Invariants: ctx 合成はキック専用構築をせず既存関数を流用（R3.2）。

**Implementation Notes**
- Integration: `event/kick.lua` 新規。`init.lua` の `SHIORI.kick` から委譲。
- Validation: 解決成功→enqueue 1 件、未解決→enqueue 0＋ログ、ctx 合成が既存関数経由であること。
- Risks: `kick_req` 最小合成で既存 `create_act` が要求するフィールド不足の可能性→`create_act(nil)` 許容性を実装時に確認（Open Question 1）。

#### KickPreempt

| Field | Detail |
|-------|--------|
| Intent | 進行中会話の前 `co_scene` 破棄・自動復帰なし |
| Requirements | 5.2, 5.3 |

**Responsibilities & Constraints**
- 前 `STORE.co_scene` を破棄（参照 nil 化）。LuaJIT `coroutine.close` 非搭載前提のため**強制 dead 化せず参照不到達＋GC**でモデル化（research D5）。
- 実体は `set_co_scene(new_co)` の既存「前シーン置換」ロジックを流用（前 `co_scene` ≠ 新 `co` のとき前を破棄）。
- 退避スタックを持たず自動復帰しない（R5.3）。
- R5.2 の「閉じた」観測契約 = **前 `co_scene` が `STORE` から不到達になり以後 resume されない**。

**Dependencies**
- Inbound: KickExecHandler — preempt 要求（P0）
- Outbound: set_co_scene（既存）— 状態置換（P0）

**Contracts**: State [x]

##### State Management
- State model: `STORE.co_scene`（`thread|nil`・session スコープ）。
- Persistence & consistency: 単一 VM・単一 consumer 上で逐次更新。
- Concurrency strategy: アクタースレッド単独（競合なし）。

**Implementation Notes**
- Integration: `set_co_scene` の置換経路をキックから利用（新規 close API は作らない）。
- Validation: preempt 後に前シーンが通常 dispatch で再開されないこと（特性化テスト）。
- Risks: 中断コルーチンが GC まで生存（実害なし・観測契約は参照不到達）。

#### TalkQueue

| Field | Detail |
|-------|--------|
| Intent | キック由来さくらスクリプトの FIFO 蓄積・drain |
| Requirements | 3.3, 4.1, 4.2 |

**Responsibilities & Constraints**
- `TALK_QUEUE.enqueue(sakura)` / `TALK_QUEUE.drain() -> sakura|nil` / `TALK_QUEUE.is_empty()`。
- 投入順（FIFO）保持（R4.1）。1 回の drain で 1 件取り出す（1 tick 1 出力）。
- session スコープ in-memory（`STORE.reset` で初期化）。

**Contracts**: State [x]

##### State Management
- State model: 配列ベース FIFO（head/tail）。
- Persistence & consistency: VM 内・単一 consumer（順序保証は自明）。
- Concurrency strategy: アクタースレッド単独アクセス。

**Implementation Notes**
- Integration: `talk_queue.lua` 新規。`STORE.reset` でクリア。
- Validation: enqueue 順＝drain 順、空 drain は nil。
- Risks: 低。

#### SecondChangeDrain

| Field | Detail |
|-------|--------|
| Intent | OnSecondChange で無条件 drain・空時は従来応答へ素通り |
| Requirements | 4.2, 4.3, 4.4, 4.5, 6.1, 6.2, 6.3, 6.4 |

**Responsibilities & Constraints**
- `second_change.lua` で `CALLBACK.sweep()` 後・`dispatcher.dispatch(act)` 前に `TALK_QUEUE.drain()` を呼ぶ。
- 非 nil → **抑制ゲート無しで** GET 応答として返す（R4.2・`is_blocked` を介さない）。
- nil（空）→ 従来 dispatch 経路へ素通り（R4.3・R6.3 バイト不変）。
- drain は OnSecondChange の pull 機会限定（R4.5・押し出ししない）。
- `Status` ヘッダ準拠は既存解釈を不変に保つ（R6.4）。

**Dependencies**
- Inbound: OnSecondChange GET — トリガ（P0）
- Outbound: TalkQueue（drain）— 取り出し（P0）、dispatcher（既存）— 空時の従来経路（P1）

**Contracts**: Service [x]

**Implementation Notes**
- Integration: `second_change.lua` に drain 分岐 1 つ。キック未使用時は `drain=nil` で完全素通り。
- Validation: ①キック後の OnSecondChange で出力が返る ②空時は従来応答とバイト一致（特性化・PASTA_DEBUG ガード）③`Status: talking` でも drain される（抑制無し）。
- Risks: drain 分岐が空時の応答をバイト変化させる懸念→nil 早期 return で素通りを保証し回帰テスト（R6.3）。

## Error Handling

### Error Strategy
- **入力検証（fail fast）**: VSCode 側で空シーン名は送信前に弾く（R1.4 取消含む）。Rust decode 側でも空/欠落を `None` として再検証しエラー応答（R2.5）。二重防御。
- **未接続セッション**: `isPastaSession()` false → 警告提示し送信しない（R1.3）。
- **解決不能シーン**: アクタースレッド上で `SCENE.co_exec` が nil → `TALK_QUEUE` へ何も積まず破棄＋診断ログ（R3.5・`seam = "kick.unresolved"`）。
- **ライフサイクル競合**: teardown/reload 中のキックは `MAILBOX.load_full()=None` で黙って no-op＋診断ログ（`seam = "kick.drop"`・デバッグ用途で許容）。
- **debug 無効**: sink 未注入＝経路非活性（R2.6・エラーですらない＝そもそも到達しない）。

### Error Categories and Responses
- **User Errors**: 空シーン名／未接続→VSCode 上で警告・エラーメッセージ（実行されない）。
- **System Errors**: `try_send` 失敗（mailbox 切断）→破棄＋ログ。VM レンダリング失敗→`resume_until_valid` の `ok=false` を捕捉しログ（enqueue しない）。
- **Business Logic Errors**: 解決不能シーン→破棄＋診断（R3.5）。

### Monitoring
- `tracing` seam ログ: `kick.inbound`（受理）/`kick.unresolved`（未解決）/`kick.drop`（ライフサイクル競合破棄）/`kick.enqueue`（投入）。debug 無効時ゼロコスト（既存方針）。

## Testing Strategy

### Unit Tests
- `PlaySceneRequest`（TS）: `setPayload('intro')={scene:'intro'}`、`validateSceneName('')=false`、`validateSceneName(' x ')=true`。
- `PlaySceneDecode`（Rust）: `pasta/playScene` 正常名→`kick_scene=Some`、空/欠落→`None`。
- `TalkQueue`（Lua）: enqueue 順＝drain 順、空 drain→nil、`is_empty` 整合。
- `KickPreempt`（Lua）: preempt 後に前 `co_scene` が `STORE` から不到達（参照 nil 化観測）。

### Integration Tests
- **キック取次経路**（Rust）: inbound `pasta/playScene` decode → sink（モック）1 回呼ばれる → `ActorMsg::Kick` 送信（`MAILBOX` モック）。空名→sink 呼ばれずエラー応答。
- **debug 無効ゲート**（Rust）: `enabled=false` で sink 非注入・キック経路非活性（R2.6）。
- **ctx 合成流用**（Lua）: `KICK.exec(scene)` が `create_act`/`SCENE.co_exec`/`set_co_scene` を経由しキック専用構築をしない（R3.2）。解決成功→`TALK_QUEUE` 1 件、未解決→0＋ログ（R3.5）。
- **OnSecondChange 無条件 drain**（Lua）: キック後 GET で出力が返る／`Status: talking` でも drain される（抑制無し・R4.2/R5.1）。

### E2E / Regression Tests
- **バイト不変回帰**（特性化・最重要 R6.3）: キック未使用時、OnSecondChange を含む通常 SHIORI 応答がキック導入前とバイト一致（`shiori-event-test-framework`・PASTA_DEBUG ガード留意）。
- **即時 preempt-and-abort**: 進行中会話中にキック→前会話が中断され前 `co_scene` が再開されない（R5.2/R5.3）。複数キック連続→FIFO 順に再帰 preempt（後続が前キックを preempt）。
- **VSCode コマンド**（モック session）: 未接続→警告・送信なし、取消→送信なし、失敗→エラーメッセージ。

### Performance
- **GET 短さ（R3.4/R4.4）**: OnSecondChange GET 内でレンダリングを同期実行しない（ctx 合成・レンダリングはアクタースレッドの `Kick` 受信側）。drain は FIFO pop のみで O(1)。

## Open Questions / Risks
（design-discussion フェーズで解決。本設計は best-effort 仮置きで進行）

1. **`create_act` のキック用最小 `req` 受容性**: `create_act(req)` は `SHIORI_ACT.new(STORE.actors, req)` を呼ぶ。キックは SHIORI イベント由来 `req` を持たないため最小合成（`{ id="OnKickScene" }` 等）または `nil` を渡す想定。既存 `create_act`/`SHIORI_ACT.new` が `nil`/部分 `req` を許容するか、最小フィールド集合が何かは実装時に確定が必要。**仮置き: `req=nil` 許容 or 最小ダミー `req` を合成**（通常合成手順の流用範囲内）。
2. **`KickSink`/`KickRequest` の定義クレートと配置**: `pasta_lua/src/debug/kick.rs` に置く想定だが、`RuntimeConfig`（`runtime/`）からも参照するため公開パスを確定する必要。**仮置き: `debug` モジュール公開・`runtime_config` が `use`**。
3. **`SHIORI.kick` の Rust↔Lua ブリッジ形**: GET/NOTIFY は SHIORI リクエスト文字列を Lua テーブル化する既存経路。キックは SHIORI リクエストでないため、`shiori.rs` に専用の薄い呼び出し（`lua.globals().SHIORI.kick(scene)` 直呼び）を置くか、擬似 SHIORI リクエスト（`NOTIFY ... OnKickScene`）を合成して既存経路に載せるかの二択。**仮置き: 専用薄ブリッジ（擬似リクエスト合成より単純で ctx 流用と独立）**。discussion で「擬似 NOTIFY 合成のほうが ctx 流用が自然」か判断。
4. **複数キック連続時の talk FIFO と preempt の相互作用**: 後続キックが前キックを preempt（前 `co_scene` 破棄）する一方、前キックが既に `TALK_QUEUE.enqueue` 済みの出力は残る（FIFO に積まれた完成さくらは破棄しない）。「preempt は進行中コルーチンのみ・enqueue 済み出力は保持」で良いか確認。**仮置き: enqueue 済みは保持・進行中 co のみ preempt**（要件は会話 preempt であり完成出力の取消ではない）。
5. **`kick.drop`（teardown 競合破棄）の許容範囲**: デバッグ用途で黙って破棄＋ログは妥当だが、VSCode へ失敗通知すべきか。**仮置き: ack はベストエフォート（sink 呼び出し成立で ack・実 mailbox 到達の保証はしない）**。discussion で UX 要否を確認。
