# Brief: ponytail-lua-debug

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。版には縛らない。0.4.2 を出した後、0.5.0 の spec が並べられずに空いた所へ入れる（2026-10-22 開発者の決定。2026-10-19 の「0.4.3 に入れる」から改めた）。同じ監査の 10 本と同じウェーブで並走できる。

## 進め方（2026-10-19 開発者の決定）

- 下の「Current State」は、2026-10-19 の監査の控え（出発点）である。
- **ギャップ分析**で、この spec の領域（`crates/pasta_lua/src/debug`）に `/ponytail:ponytail-audit` をやり直す。控えと突き合わせ、違いをギャップ分析の報告に書く。
- 要件は「その領域の監査の指摘が、残らず直っている」ことと、挙動を変える直しの決めごとにする。
- **修正は設計以降**で行う。

## Problem

VSCode でゴーストをデバッグする作者が困る。

- Lua 表示に切り替えると、`.pasta` のブレークポイントが黙って効かなくなる。
- コルーチンの最後でステップすると、ステップの状態が抜けなくなる。その後、無関係なトークで偽の停止が起きる。
- 大きい数の変数が誤って表示される。
- 一時停止・ウォッチ・デバッグコンソールが、応答なしで固まる。

なお、デバッグを無効にしているときの費用はゼロである（`enable` が早く `Ok(None)` を返す）。

## Current State

2026-10-19 の監査で確かめた。本番のコードは約 8k 行、テストは約 14k 行。`cargo test -p pasta_lua --lib debug::` の 333 本は通る。

1. **Lua 表示の間、`.pasta` のブレークポイントが死ぬ**（Must fix）
   - 場所：`wiring/inbound.rs` L186-201 の `source_map.pasta_active() && is_pasta_source(..)`。
   - 応答は `verified:true` なのに、当たらない。切り替えた後に VSCode が送り直す、既存のブレークポイントも壊れる。
   - `book/src/debug/source-level.md:118` の「切り替えても `.pasta` のブレークポイントは有効」という約束に反する。
2. **ステップが抜けない**（Must fix）
   - 場所：`session/stop_loop.rs` L118-121（Disconnect）と L206-207（Err）が `mode` を戻さない。`session/mod.rs` L155 の `ThreadId` は、生の `lua_State` の番地。`stop_loop.rs` L287-296、`stepping.rs` L382-387。
   - A：シーンのコルーチンの最後の行で F10 を押すと、`RunMode::Stepping` が残る。その間、全 thread の全行が `capture_stack` を走らせる。死んだコルーチンの番地を LuaJIT が使い回すと、無関係な後のトークで偽のステップの停止が起きる。
   - B：ステップで止まっている間に切断すると、Stepping のまま残る。以後の行は、死んだ通路に Stopped を送って Err を受け取る。
3. **大きい整数の表示が誤る**
   - 場所：`inspect.rs` L378-388。
   - `n as i64` が飽和し、`1e20` が `9223372036854775807` と表示される。
4. **知らない DAP 要求に応答しない**
   - 場所：`dap/decode.rs` L303 の `_ => Decoded::default()`。
   - pause・evaluate（ウォッチとデバッグコンソール）が、永遠に待つ。DAP の仕様は、全部の要求に応答を求めている。
   - マニュアル `book/src/internals/debug.md` は「その他: 無視する」と書いている。
5. **行ごとの割り当て**（デバッグが有効なとき）
   - 場所：`breakpoints.rs` L120-133、`session/stop_loop.rs` L24-34・L250-253、`source_map/mod.rs` の `resolve_lua_to_pasta`。
   - `should_pause` が、毎行、フックの chunk と全部のブレークポイントの chunk を正規化する。ブレークポイントが 0 件でもする。
6. **ステップ中、thread を比べる前に全スタックを計算する**
   - 場所：`session/stop_loop.rs` L296、`stepping.rs` L382-387。
7. **止まったクライアントで、片付けが固まる**
   - 場所：`transport/mod.rs` L436（フレームの途中では読み取りの時間切れを外す）、`transport/framing.rs` L72（`read_line` に上限が無い）。
8. **テストだけ、または誰も使わないコード**
   - `LineEvent`
   - `StopReason::Entry`・`Pause`
   - `DebugError::Protocol`
   - `SessionEvent::Error` と `report_error`
   - `HookHandle`・`PanicCause`・`record_panic_cause`（`hook.rs` L68-125・L163・L202+）
   - `stop_loop` の `SetBreakpoints` の分岐（L123-132）
   - `playscene.rs`・`scene_index.rs` の `#![allow(dead_code)]` と `insert_file`
9. **scopes の往復が何もしない**
   - decode の時点で答えている。session が作る `SessionEvent::Scopes` は、encoder が捨てる。
10. **表示の形の写しが 2 つ**
    - 場所：`session/mod.rs` L215-224・L278-313。
    - `source_mode` と `Option<SharedSourceMode>` の 2 つがある。`enable` は常に `Some` を渡す。
11. **transport の「無効」の道が、テストだけのもの**
    - 場所：`transport/mod.rs` L147-160 の `start(None)`、L236-261 の `shutdown`・`join`、`enable.rs` L246-256 の `unreachable!`。
12. **`DebugConfig::resolve` の引数が 8 つ**
    - 場所：`config.rs` L97-136。
    - `attach_source_mode` は、テスト以外では `None`。`file_source_mode`・`file_sidecar` は、`file` から導ける。

## Desired Outcome

- `.pasta` のブレークポイントが、表示の形によらず効く。
- ステップは、コルーチンを抜けると continue と同じになり、偽の停止が無くなる。切断すると Running に戻る。
- 数は正しく表示される。
- 知らない要求には、`success:false` と「unsupported」で答える（2026-10-19 開発者の決定）。
- デバッグが有効なときの、行ごとの費用が下がる。
- 死んだコードと写しが無くなる。

## Approach

監査の最小の直しを採る。

- ブレークポイントの訳は、表示の形でなく対応表の有無で決める。
- ステップは、`ThreadId` の代わりに `mlua::Thread` を持つ。参照を持てば、番地の使い回しが起きない。終わった thread なら Running に戻す。
  - 残る限界（`co_scene` の差し替えで、中断のまま捨てられたコルーチン）には、`ponytail:` の注釈を付ける。
- 数は、常に `format!("{n}")` で出す。
- 応答は、`error_response(seq, cmd, msg)` の 1 つにまとめる。`play_scene_at_error` もこれに寄せる。

## Scope

- **In**: 上の 1〜12 と、ギャップ分析で増えた同じ領域の指摘。マニュアル `book/src/internals/debug.md` の該当の行、`book/src/debug/source-level.md`。
- **Out**: VSCode 拡張の側（`ponytail-editor-tooling` が持つ。`host` の廃止もそちら）。

## Boundary Candidates

- ブレークポイントの訳（wiring）
- ステップの状態（session・stepping）
- 表示（inspect）
- DAP の応答（dap）
- 行ごとの費用（breakpoints・source_map）
- transport の片付け
- 死んだコードの撤去

## Out of Boundary

- `editors/vscode`（`ponytail-editor-tooling`）
- 単一の DLL・opt-in・127.0.0.1 固定の待ち受けの方針（確定済み）
- `runtime/exec.rs` の `debug_source_mode()`（領域の外。テストだけが使う。拾うかはギャップ分析で決める）

## Upstream / Downstream

- **Upstream**: `pasta_lua` のトランスパイラが作るソースの対応表
- **Downstream**: VSCode 拡張の DebugAdapterServer

## Existing Spec Touchpoints

- **Extends**: なし
- **Adjacent**:
  - `ponytail-editor-tooling`（`book/src/debug/*.md` の別の節を触る。後から入る側が rebase する）
  - `ponytail-lua-transpiler`（行の対応は変えない約束）

## Constraints

- 振る舞いを固めるテストを先に書き、1 つ直すごとに確かめて 1 コミットにする。
- 品質の関門：`cargo fmt --check`、clippy（警告 0）、`cargo test -p pasta_lua`。
- cargo の前に `NoDefaultCurrentDirectoryInExePath` を外す。`PASTA_DEBUG=1` がテストを壊さないこと（固定ポート 9276。`#[ctor]` で中和済み。新しいテストにも同じ守りを付ける）。
