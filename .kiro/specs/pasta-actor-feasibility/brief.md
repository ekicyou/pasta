# Brief: pasta-actor-feasibility

## Problem
ゴースト作者のデバッグ／オーサリング体験において、本当に必要なのは「デバッグ位置でのブレーク」ではなく「**任意シーンの再生を今すぐキック**」すること（Phase 5 デバッガ実装後の結論）。それを実現するにはエンジンを SHIORI スレッド束縛から解放し、自前スレッドのアクター化が必要だが、FFI 境界・`!Send` VM・SSP reload・既存コルーチンに構造的未知が残る。**本番実装に踏み込む前に、最小の使い捨て PoC で可否（GO/no-go）を確定する**。

## Current State
- Lua VM（`mlua`、`!Send + !Sync`）は `PastaLuaRuntime` が保持し、`Arc<Mutex<Option<PastaShiori>>>` ＋ `unsafe impl Send` で SHIORI スレッドに束縛（`crates/pasta_shiori/src/windows.rs`, `crates/pasta_shiori/src/shiori.rs`）。
- エンジンは反応専用。トーク継続（`STORE.co_scene`）も非同期 callback（`CALLBACK.pending` / `get_property`）も「次の SHIORI リクエスト（OnSecondChange / OnPastaCallBack）で resume」。ホストのリクエスト周期がエンジンの唯一の時計。
- debug backend（`crates/pasta_lua/src/debug/`）は既に「外部スレッド → mpsc → VM フック点でコマンド適用」を実証済み。アクターモデルはこれの一般化。
- 道具は準備済み: `wintf_winmsg_executor`（`winmsg-executor` の便利フォーク・公開済み）。`block_on()` / `spawn_local()` / `MessageLoop` / `JoinHandle` / `FilterResult`、Send/Sync 不要、メッセージ専用ウィンドウに各タスクを背負わせメッセージループで poll。

## Desired Outcome
最小の使い捨て PoC（feature-gated）で以下を GO/no-go 判定する:
1. `wintf_winmsg_executor` が `!Send` Lua VM を素直にホストし、SSP reload で **clean teardown** できる（過去のソケット/ポート枯渇系を繰り返さない）
2. SSP スレッド → executor スレッドへの **block-on-reply marshaling** が同期契約を守る（GET）／NOTIFY は即 204 で fire-and-forget
3. **drop→204 ガード**で「応答未送信のまま drop（panic/忘れ）」のデッドロック経路が原理的に消えることを実証
4. ホスト tick 駆動 → executor 駆動へ移って既存 coroutine/callback（`STORE.co_scene` / `CALLBACK`）が生存
5. **talk FIFO ＋ OnSecondChange drain ＋ `Status: talking` gate ＋即時 preempt** が実 SSP で **≤1秒キック配信**を実現
6. GET block-on-reply のレイテンシ実測（GET タイムアウト→204 フォールバックの要否を実機で判断）

## Approach
- `pasta-lua-debug-feasibility` と同型の feasibility gate。検証コードは feature（例 `actor-poc`）で default 無効・使い捨て。開始時に専用ブランチを切る。
- 最小スライス: 実 SSP 相当の呼び出しパターンで「1 シーンを VSCode 風キック → executor 非同期実行 → FIFO → OnSecondChange 応答 → SSP 再生」を 1 往復実証。
- 理由: 本番の大規模 refactor 前に「一番怖い」block-on-reply 反転・reload teardown・coroutine 生存・実 SSP キック配信を潰す。

## Scope
- **In**: 上記 6 項目の GO/no-go 実証。最小 PoC ハーネス。GET レイテンシ実測とタイムアウト要否判断。
- **Out**: 本番化（`pasta-actor-runtime`）、キック機能の作り込み（`pasta-scene-kick`）、presentation event stream 契約の確定（PoC では最小限の仮契約で可）、UI、挙動保存の網羅検証。

## Boundary Candidates
- executor 統合 ＋ VM pin ＋ reload teardown（最大の未知）
- FFI marshaling（CH ＋ GET/NOTIFY ＋ drop→204）
- coroutine/callback 生存
- talk FIFO drain ＋ `Status: talking` gate ＋即時 preempt の実 SSP 配信・レイテンシ

## Out of Boundary
- 本番コードへの恒久組込み（feature-gated・使い捨て）
- 挙動保存の網羅検証（本番 spec の責務）
- `pasta_novel` アダプタ・`*.pasta` ウィンドウ・SSTP 出力

## Upstream / Downstream
- **Upstream**: `pasta-lua-debug-feasibility`（PoC ゲート文化の前例）、debug backend（スレッド/チャネル前例）、`wintf_winmsg_executor`（executor）
- **Downstream**: `pasta-actor-runtime`（GO 判定を前提に着手）

## Existing Spec Touchpoints
- **Extends**: なし（新境界）
- **Adjacent**: `pasta-vscode-lua-debug` / `debug-transport-hardening`（同じ `pasta_lua/src/debug/` とソケット/reload 再バインドの知見）

## Constraints
- LuaJIT 2.1 / `mlua` 0.11 の `!Send` 制約内
- Windows DLL（`pasta_shiori`）・SSP reload 下で検証
- SHIORI/3.0 準拠（GET=Value 応答前提、NOTIFY=応答無視、204=成功・返り値なし）
- 既存 yield/resume 基盤（`STORE.co_scene`、`resume_until_valid`、`CALLBACK`）との互換
- 検証コードは feature gate・使い捨て・本番バイト不変（PASTA_DEBUG 系テスト汚染を避けるガードも踏襲）
