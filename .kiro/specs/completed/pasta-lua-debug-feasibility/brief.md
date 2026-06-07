# Brief: pasta-lua-debug-feasibility

## Problem
VSCode から pasta（生成 Lua / 最終的に .pasta）をデバッグしたいが、組込 LuaJIT（mlua 0.11 vendored 静的リンク）へのデバッガ連携の可否が未確定。特に **`mlua::Lua::set_global_hook` ＋ `jit.off(true,true)` が、pasta のコルーチン多用モデル（シーン1つ＝コルーチン1つを動的量産）で実際にラインフックを撃てるか**が、デバッグ環境全体の GO/NO-GO 分岐になっている。机上調査では「GO（要 PoC）」までしか確定できない。

## Current State
- `debug` ライブラリは既定 OFF（`std_all`=`StdLib::ALL_SAFE`、`std_debug` は別途必要）。`crates/pasta_lua/src/runtime/runtime_config.rs`。
- Rust 側で `set_hook`/`set_global_hook` は未使用（grep でヒットなし）。debug 内省は `runtime/log.rs` の `lua.inspect_stack` のみ実績あり。
- 過去の Lua 側 luasocket デバッグ試行（`vscode-debuggee.lua` ＋ `socket/core.dll`/`mime/core.dll` ＋ `dkjson.lua`）が DLL 内 zip に同梱されているが**未配線**。本路線では不要化される。
- VSCode 拡張は `contributes.debuggers`/DAP なし（ハイライト＋診断のみ）。

## Desired Outcome
以下を最小 PoC（実機 or 単体テスト）で実証し、実装仕様への **GO/NO-GO を確定**する:
1. `jit.off(true,true)` 適用後、`set_global_hook` の line hook が **動的生成される複数のシーンコルーチンで発火**する。
2. フック（Rust コールバック）内で**ブロッキング待機（チャネル/ソケット read）→ 実行停止 → 再開**ができる（フック内 yield は LuaJIT で C-call boundary エラーになるため使わない）。
3. フック内から **mlua でスタック/ローカル変数を inspect** できる。

## Approach
依存最小・Rust 側トランスポートの「プレーン実装」前提の最小 PoC。検証用ブランチを切り、使い捨て/feature-gate で検証コードを書き `cargo test`（必要なら SSP ロード実機）で確認。luasocket 非依存。`std::net::TcpListener` だけで往復最小ループ（任意）まで叩ければ理想。

## Scope
- **In**:
  - `jit.off(true,true)` の適用方法確立とフック発火確認（フック設定前にコンパイル済みコードは飛ばない点も検証）
  - `set_global_hook` による複数（動的生成）コルーチンへのラインフック発火確認
  - フック内ブロッキングでの停止 → 再開の往復
  - フック内からのスタック/ローカル変数・upvalue の inspect（mlua）
  - （任意）別スレッド `TcpListener` ↔ VM スレッドフックのチャネル受け渡し最小往復
  - （可能なら）SSP ロード環境でのソケット待受・ブロッキング許容性の確認
- **Out**:
  - DAP プロトコル実装
  - .pasta ソースマップ
  - VSCode 拡張製品化
  - 本実装の恒久統合・設計確定（実装仕様で実施）

## Boundary Candidates
- フック機構（`set_global_hook` ＋ `jit.off`）
- フック内ブロッキング待機（チャネル/ソケット）
- フック内変数 inspect

## Out of Boundary
- 製品コードへの恒久統合（検証は使い捨て/分離・feature-gate）
- トランスポートやプロトコルの正式設計

## Upstream / Downstream
- **Upstream**: pasta_lua runtime（`PastaLuaRuntime` が `mlua::Lua` を保持）、mlua 0.11、LuaJIT 2.1
- **Downstream**: pasta-vscode-lua-debug（本実装。本検証の GO 判定に依存）

## Existing Spec Touchpoints
- **Extends**: なし（新規）
- **Adjacent**: ukagaka-desktop-mascot Req28（DAP/LSP デバッグ要件の出典・参照価値あり）

## Constraints
- LuaJIT 2.1（mlua 0.11 `features=["luajit52","vendored","serialize"]`、静的リンク）
- `mlua::Lua` は `!Send`・SSP から単一スレッド呼び出し前提（`OnceLock`→`PastaShiori`→`PastaLuaRuntime.lua` のグローバル単一常駐）
- **JIT 中はフック不発火** → `jit.off` 必須
- **フック内 yield 不可**（C-call boundary）→ ブロッキング待機のみ
- ソケット accept は別スレッド必須だが `!Send` のため VM 操作はフック内（VM スレッド上）に閉じ、I/O スレッドとはチャネル分離
- LuaJIT ビルド時は環境変数 `NoDefaultCurrentDirectoryInExePath` に注意（既知のビルド落とし穴）
