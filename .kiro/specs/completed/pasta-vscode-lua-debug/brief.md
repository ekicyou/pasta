# Brief: pasta-vscode-lua-debug

## Problem
ゴースト作者・pasta 開発者は VSCode 上で pasta（.pasta）をステップ実行・ブレークポイント・変数監視できない。現状は print デバッグと `@pasta_log`/tracing ログのみで、シーン実行（yield/resume コルーチン）の挙動追跡が困難。

## Current State
- VSCode 拡張は `languages`/`grammars`/`semanticTokens` のみ。`contributes.debuggers`・DAP コードは皆無（`editors/vscode/`）。
- Lua VM は Rust 側 `set_hook` 未使用、`debug` ライブラリ既定 OFF。
- 生成 Lua ↔ .pasta の **source map なし**（AST 全ノードに `Span` はあるが codegen が行対応を出力していない）。
- 旧 luasocket デバッグ資産（`vscode-debuggee.lua`・`socket/core.dll`・`mime/core.dll`・`dkjson.lua`）が DLL 内 zip に同梱・未配線で残存。
- request は完全同期ブロッキング（`pasta_shiori` の `Mutex` 直列）。

## Desired Outcome
VSCode から **.pasta ファイル上で**ブレークポイント設定・ステップ実行（over/into/out）・コールスタック表示・変数 inspect ができる。トランスポートは Rust 側提供で**依存最小**。デバッグ基盤は pasta_lua に内蔵し、**SHIORI 以外の pasta ホストでも再利用可能**。デバッグ無効時は本番ゼロコスト。

## Approach
Rust ホスト型 DAP バックエンド（satoren/LRDB と同型）。
- **トランスポート**: `std::net::TcpListener`（追加クレートなし、同期 blocking I/O）。
- **プロトコル**: `serde_json`（既存依存）で DAP 最小サブセットを手書き（`dap` クレートは alpha のため依存最小方針で不採用）。
- **フック**: `mlua::Lua::set_global_hook`（全コルーチン横断・PoC で実証済み `HookStrategy::GlobalHook`）＋ デバッグモード時 **グローバル `jit.off()`（無引数）**（PoC 知見: `jit.off(true,true)` は関数単位制御でグローバルエンジン状態を変えず不十分）。
- **スレッド分離**: ソケットスレッドは I/O のみ、VM 操作はフック内（VM 呼び出しスレッド上）でチャネル経由（`mlua::Lua` が `!Send` のため）。
- **ソースマップ**: `code_gen/element_gen.rs`・`scope_gen.rs` の `writeln` 群で出力行 → `.pasta` 行を記録（`generate_action` で捨てている span を回収）。Lua 行 ↔ .pasta 行を変換しブレーク/スタックを .pasta 座標で提示。
- **VSCode 拡張**: `contributes.debuggers` ＋ `DebugAdapterDescriptorFactory` で `DebugAdapterServer(port)` を返す薄実装。`launch.json` は attach。

## Scope
- **In**:
  - pasta_lua 内蔵デバッグサーバ（TCP・`std::net` のみ）
  - DAP 最小サブセット手書き（initialize / setBreakpoints / configurationDone / threads / stackTrace / scopes / variables / continue / next / stepIn / stepOut ＋ stopped/terminated イベント）
  - `set_global_hook` ＋ `jit.off`（デバッグモード時）、コルーチン横断発火
  - スレッド分離（socket=I/O、VM 操作=フック内、チャネル受け渡し）
  - .pasta ソースマップ（codegen 改修・Lua 行↔.pasta 行変換）
  - VSCode 拡張統合（`contributes.debuggers` ＋ `DebugAdapterServer` ＋ attach 用 launch.json）
  - デバッグ有効化フラグ（pasta.toml/環境変数）、無効時は本番ゼロコスト・`std_debug` を Lua へ非露出（Rust 側 set_hook のみ）
  - 旧 luasocket 資産（vscode-debuggee.lua・socket/mime core.dll・dkjson）の撤去（DLL 内 zip 肥大解消）
  - SHIORI 非依存設計（デバッグ基盤を pasta_lua に置きホスト非依存に）
  - **コルーチン本体フレームの変数 inspect 対応**（PoC 制約 R3.4 への本番対応。フックの `&Lua`＝メインステートのため走行中コルーチンの局所変数は別 `lua_State` を辿る等が必要。pasta のシーンはコルーチンで走るため必須）
  - **検証コード（PoC ハーネス）の最終除去**（採用方式の本番移行完了・検証後に上流 `pasta-lua-debug-feasibility` の feature `lua-debug-poc` 一式を撤去。完了条件。詳細は「完了条件: 検証コードの除去」節）
- **Out**:
  - 条件付きブレークポイント・ウォッチ式・ホットリロード（将来）
  - areka/非 SHIORI ホストへの実配線（基盤は再利用可能にするが実ホスト統合は将来）
  - SSP タイムアウトの根本解決（構造的制約として明示し緩和策に留める）
  - LSP 機能（既存 pasta_lsp の領分）

## Boundary Candidates
- デバッグバックエンド（Rust: transport ＋ DAP ＋ hook ＋ 状態機械）
- ソースマップ（transpiler/codegen 拡張）
- VSCode 拡張統合（DAP クライアント接続）

## Out of Boundary
- LSP 機能（pasta_lsp）
- 旧 Lua 側 luasocket デバッグ経路の維持

## Upstream / Downstream
- **Upstream**: pasta-lua-debug-feasibility（**GO+ 達成済み・2026-06-07**。採用方式＝グローバル `jit.off()`＋`set_global_hook`／FFI 変数 inspect／停止・再開／`std::net` 往復、および既知制約を本仕様へ引き継ぎ。検証結果は当該 `research.md`「PoC 検証結果」節）、pasta_lua runtime（`PastaLuaRuntime`/VM 初期化）、code_gen（ソースマップ素材）、editors/vscode
- **Downstream**: areka/IDE 統合（ukagaka-desktop-mascot Req28 AC11-14）、非 SHIORI ホストでのデバッグ

## Existing Spec Touchpoints
- **Extends**: VSCode 拡張（完了仕様 pasta-vscode-extension）、code_gen（生成 Lua 出力）
- **Adjacent**: pasta_lsp（言語サーバ）、ukagaka-desktop-mascot Req28（DAP/LSP 要件の出典）

## Constraints
- 依存最小（`std::net` ＋ `serde_json`、追加クレートは極小に）
- LuaJIT 2.1（mlua 0.11 vendored 静的リンク）、`mlua::Lua` は `!Send`・単一スレッド呼び出し
- `jit.off` 必須、フック内 yield 不可（ブロッキング待機のみ）
- ブレーク中は SHIORI 応答が停止 → SSP タイムアウトに留意（緩和策・運用注意で対応）
- デバッグ OFF 時は本番無コスト・サンドボックス（`std_debug` 非露出）維持
- ライセンスは MIT/Apache-2.0 互換のみ
- LuaJIT ビルド時 `NoDefaultCurrentDirectoryInExePath` 注意

## 完了条件: 検証コード（PoC ハーネス）の除去

上流 `pasta-lua-debug-feasibility` の検証ハーネス（feature `lua-debug-poc`）は **「使い捨て前提」**（feasibility Requirement 5.5）である。本仕様は当該 PoC が実証した方式を本番デバッグ基盤として実装するため、**本番実装が完了し PoC からのソースコード移行が完全に終了・検証された時点で、検証コードを除去する**ことを本仕様の**完了条件（要件）**とする。requirements.md 生成時に正式な EARS 要件として明文化すること。

**除去対象**（feasibility が追加した一式・production `src/` 変更はゼロ＝対象外）:
- `crates/pasta_lua/Cargo.toml` の `[features]` から `lua-debug-poc = []` を削除
- `crates/pasta_lua/tests/runtime/main.rs` の gated `#[cfg(feature = "lua-debug-poc")] mod lua_debug_poc_test;` 行を削除
- `crates/pasta_lua/tests/runtime/lua_debug_poc_test.rs` ＋ `crates/pasta_lua/tests/runtime/lua_debug_poc_test/`（7 サブモジュール: harness_types/hook_probe/pause_gate/frame_inspector/session/transport_loop/verdict）一式を削除

**除去の前提**（すべて満たすこと）:
1. 本番デバッグ基盤（transport / DAP / hook / 状態機械 / ソースマップ）が実装・検証済み。
2. PoC が実証した知見（グローバル hook によるコルーチン横断発火・FFI 変数 inspect・停止/再開・`std::net` トランスポート往復）が本番実装へ移行済み。
3. 本番実装側に同等以上の自動テストが存在し、PoC ハーネスへの依存が残っていない。
4. 検証結論（GO+）の再現性は、移行後は本番テスト ＋ feasibility の `research.md`「PoC 検証結果」節（文書）で担保する。

> 補足: PoC ハーネスは feature-gate（default 無効）の test 専用コードであり production ランタイムへの恒久統合ではない。本除去はあくまで「役目を終えた検証足場の片付け」であり、上記前提を満たすまでは再検証エビデンスとして残置してよい。
