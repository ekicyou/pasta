# Brief: ponytail-lua-debug

> **ステータス**: 未着手（2026-10-19 の `/kiro-discovery` で起票。`/ponytail:ponytail-audit` の指摘より）。0.4.5（2026-10-24 開発者の決定「ポニーテールは 0.0.1 後ろからの着手に変更」。経緯: 2026-10-19 に 0.4.3、2026-10-22 に版に縛らない隙間、2026-10-23 に 0.4.3）。0.4.3 の mlua 0.12 の後の main で測り直す。同じ監査の 10 本と同じウェーブで並走できる。

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

## 申し送り（scene-play-event より、2026-10-22 完了時）

- 上の「新しく見つけた点」（ポートが取れなくても表を作るのに使われない）は、scene-play-event で**仕様にした**。ランタイムは、設定でデバッグが有効なら、待ち受けが立たなくてもソースの対応表と索引を持ち続ける（`runtime/mod.rs` の `if config.debug.enabled`）。位置でシーンを再生する `OnPastaSceneAt` がこれを使う。直す対象ではない。
- 内部用のモジュール `@pasta_debug`（`runtime/module_registry.rs` の `register_debug_module`）を足した。`enabled()` は**解決した設定の値**を返し、待ち受けの有無（`debug_enabled()`）は見ない。`scene_at(file, line)` は `debug/playscene.rs` の新しい `resolve_position` を呼ぶ。`playscene.rs` に呼び手が増えたので、`#![allow(dead_code)]` の外し方を見直すときはこれを数える。
- `debug_enabled()`（`runtime/exec.rs`）と `debug_handle` の doc は「待ち受けの有無」を表す（doc は完了時に直した）。設定の有効と待ち受けの有無の 2 つがある、という前提で整理すること。

## 申し送り（2026-10-24 の棚卸より。サイドカーの TOML 化）

- 0.4.3 の `toml-serialization` が、ソースマップのサイドカー（`.map`）を JSON から TOML に替える。サイドカーの読み書きはこの spec の持ち場（`crates/pasta_lua/src/debug/source_map/`）と重なりうるので、この spec は `toml-serialization` が入った後の main でギャップ分析をやり直す（roadmap の「0.4.5 の範囲」）。
- 0.4.3 の `mlua-0-12-upgrade` が `mlua::` を参照するファイルを広く書き換えるので、`debug/**` もその後の形で測り直す。

## 2026-11-02 棚卸の再測定（main 491cc23c）

- **前提の変化**: 約束どおり、`toml-serialization`・`mlua-0-12-upgrade`（と `shiori4-v3-portable`）が入った後の main で測り直した。12 の指摘はどれも残る。
  - `debug/` で変わったのは、サイドカー `source_map/sidecar.rs` の `serde_json`→`toml`（中身の形と、ランタイムがサイドカーを読まないことは同じ）、`mlua::Debug`→`mlua::debug::Debug` の `use` の行（`hook.rs`・`session/mod.rs`・`session/stop_loop.rs`）、`inspect.rs`・`kick.rs` の doc だけ。DAP の通信は JSON のまま（`dap/`・`transport/framing.rs`。TOML の方針の対象外）。
  - 行番号: `inspect.rs` の数の表示は 3 行下へずれて 381〜391 行（`n as i64` は 385 行）。ほか（`wiring/inbound.rs` 194 行の `pasta_active() && is_pasta_source`、`dap/decode.rs` 303 行、`enable.rs` 124 行の `unreachable!`、`debug/config.rs` 98〜136 行の `resolve`）は変わらない。
  - `ponytail-lua-loader` は入ったが、`runtime/exec.rs` 86 行の `debug_source_mode()`（テストだけが使う）は触らなかった。拾うかはこの spec のギャップ分析で決めてよい（境の相手はもう居ない）。デバッグの「有効」は 2 つ（設定の有効と待ち受けの有無）で、混ぜない（10-22 の申し送りのとおり）。
- **触るファイル**: `crates/pasta_lua/src/debug/**`（本番は約 8k 行。大きいのは `source_map/mod.rs` 675 行・`breakpoints.rs` 626 行・`types.rs` 581 行で、1,000 行に近いものは無い）、マニュアル `book/src/internals/debug.md`・`book/src/debug/source-level.md`。新しいファイルは無い見込み。`pasta_lua/src/lib.rs` 52 行の再輸出の型を消すなら、`ponytail-lua-transpiler` と同じファイルの別の行（後から入る側が合わせる）。
- **規模**: 14〜18 タスク（変わらず）。
- **先に要るもの**: 働きの上では無い。ほかの ponytail 7 本とはソースを共有しない（`ponytail-lua-transpiler` の `config.rs` は `pasta_lua/src/config.rs` で、こちらの `debug/config.rs` とは別）。`ponytail-editor-tooling` とは `book/src/debug/*.md` の別の節。0.5.0 の `scene-count`・`scene-condition-attributes` が触る `book/src/debug/validate.md` は別の頁。
- **種別**: バグ（Lua 表示で `.pasta` のブレークポイントが効かない・ステップが抜けない）。監査の指摘より。
- **要件定義のモデル**: Opus。
- **分割の案**: 今は不要。ギャップ分析で 20 を超えたら、正しさ（ブレークポイント・ステップ・数の表示・応答）と無駄（死んだコード・写し・行ごとの費用）の 2 つに分ける。
- **見つけた穴・古くなった記述**
  - Current State 3 番の `inspect.rs` L378-388 は、上のとおり 381〜391 行。12 番の `config.rs` は `debug/config.rs`（`pasta_lua/src/config.rs` と取り違えやすい）。
  - `playscene.rs` の頭の説明（1〜3 行）は `resolve_and_kick` だけを書き、`resolve_position` に触れない。`#![allow(dead_code)]`（`playscene.rs` 31 行・`source_map/scene_index.rs` 39 行）は結線の前の名残り（10-23 のまま）。
  - 4 番の知らない要求は、マニュアル `book/src/internals/debug.md` 207 行の表（「その他: なし（無視する）」）も変える（10-23 のまま）。

## 申し送り（2026-11-02、`ponytail-lua-transpiler` の要件より）

- `ponytail-lua-transpiler` は、テストだけが使う包み `normalize_output`（`pasta_lua/src/normalize.rs`）を消す。`LineShift` と `normalize_output_with_shift` の形は保つ（こちらの `source_map/mod.rs` と `loader/source_map_build.rs` が読むため）。
- 消すと、`debug/source_map/mod.rs` 25 行の説明「producer の `out_line` は `normalize_output` **適用前**のバッファ行を数える」の 1 語が古くなる（リンクではないので doc のビルドは壊れない）。この spec が、その 1 語を `normalize_output_with_shift` に直す（`ponytail-lua-transpiler` は `debug/` に触らない約束。`normalize_output_with_shift` はどちらが先に入っても在るので、入る順によらない）。
- `ponytail-lua-transpiler` は、アクター辞書の単語定義の行（`ACTOR:create_word(…)`）もソースマップに載せる。対応は増えるだけで、今ある対応は変わらない。ブレークポイントの解決の期待値が増える見込み。
