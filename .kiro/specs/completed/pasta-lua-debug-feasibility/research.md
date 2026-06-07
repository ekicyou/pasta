# Gap Analysis: pasta-lua-debug-feasibility

> 本仕様は「検証（PoC）仕様」であり、ここでの「実装」とは **go/no-go を確定する検証ハーネス**を指す。
> 分析日: 2026-06-07 / 対象: mlua 0.11.6（Cargo.lock 確定）/ LuaJIT 2.1（vendored 静的リンク）

## 分析サマリー

- **大前提は成立**: `mlua::Lua::set_global_hook` は **0.11.6 に確実に存在**（CHANGELOG v0.11.0-beta.1 で追加、`luau` 非有効＝luajit52 で利用可）。フック設定・行トリガ・ブロッキング停止の土台は揃う。
- **最大の未確認（#1リスク）**: mlua の global hook は doc 上「mlua が生成する新規スレッドに適用」と記述。pasta のシーンコルーチンは **Lua 側 `coroutine.create`**（`scene.lua:212`）で生成されるため、これに global hook が効くかは **要実証**。LuaJIT は `lua_sethook` がスレッド引数を実質無視しメインステートにグローバル適用される挙動（LuaJIT #666）があり、これが有利に働く可能性があるが、机上では断定不可。→ Requirement 1.2/1.3 がこれを正面から検証する設計になっている。
- **変数inspectに API ギャップ（Missing）**: mlua 0.11.6 の `Debug`/`DebugStack` は**個数（`num_ups`）しか公開せず**、ローカル変数・upvalue の**名前付き取得APIが無い**。Requirement 3 の充足には (a) 生 FFI（`lua_getlocal`/`lua_getupvalue`）か (b) Lua 側 `debug.getlocal`（＝`std_debug` 露出が必要で Requirement 5.3 と衝突）のいずれかが要る。**設計判断事項**。
- **足場は完備**: 生 VM テストヘルパ（`create_runtime_with_finalize() -> Lua`）、`runtime.lua()` 生ハンドル、`pub use mlua`、`tests/runtime/` のサブモジュール規約、`serde_json`・`std::net` 利用可。`jit` テーブルは `ALL_SAFE` に含まれ `std_debug` 露出なしで `jit.off()` 呼び出し可。
- **feature gate は新設**: `crates/pasta_lua/Cargo.toml` に `[features]` セクションが**存在しない**。PoC が最初の feature 導入（`lua-debug-poc`）になる。

## 要件 → 資産マップ（ギャップタグ: Missing / Unknown / Constraint）

| 要件 | 利用できる既存資産 | ギャップ |
|---|---|---|
| **R1** フック発火（jit.off + set_global_hook・コルーチン横断） | `Lua::set_global_hook`（mlua 0.11.6 `state.rs:581`, `#[cfg(not(feature="luau"))]`）／`HookTriggers::EVERY_LINE`／`runtime.lua()` `mod.rs:188`／生VMヘルパ `e2e_helpers.rs:40`／`jit` は `ALL_SAFE` 内（`std_jit` 不要・`std_debug` 不要） | **Unknown**: Lua側 `coroutine.create`（`scene.lua:212`）由来コルーチンに global hook が伝播するか（LuaJIT #666 が補償しうるが要実証）。**Constraint**: フックは設定**前**にコンパイル済みコードへ効かない → 起動時に **グローバル `jit.off()`（無引数）** 必須（`jit.off(true,true)` は関数単位で不十分。後述「LuaJIT jit.off セマンティクス」参照） |
| **R2** フック内ブロッキング停止・再開 | フックコールバックは素の Rust `Fn`（ブロッキング自由）／`std::sync::mpsc`・`std::net` 標準利用可 | **Constraint**: `VmState::Yield` は Lua 5.3+/Luau 限定で **LuaJIT では `Continue` のみ** → フック内 yield 不可・ブロッキング待機のみ（要件通り）。デッドロック/`!Send` 規律に注意 |
| **R3** フック内変数 inspect | `Debug::source()/current_line()/names()`（`debug.rs`）／`lua.inspect_stack` 実績（`log.rs:54`）／`DebugStack.num_ups`／**`mlua::ffi`（無条件公開・mlua-sys 0.10.0）**＝`lua_getlocal`/`lua_getupvalue`/`lua_getstack`／`Lua::exec_raw`（公開 unsafe で生 `lua_State` 取得） | **Constraint（緩和済）**: mlua 高レベルに名前付き変数取得 API は無いが、**FFI 正攻法が成立確認済**（フック内で `exec_raw`→`lua_getstack(L,0,&ar)`→`lua_getlocal`）。`std_debug` 非露出のままサンドボックス維持可（R5.3 整合）→ R3↔R5.3 の衝突は解消。残課題は mlua 固有前例が無いため **PoC での実証**（unsafe 範囲・型判別の確認） |
| **R4** トランスポート最小往復（任意） | `std::net::TcpListener`・`std::thread`・`mpsc`（追加クレート不要） | **Constraint**: `mlua::Lua` は `!Send` → ソケットは別スレッドで I/O のみ、VM 操作はフック内（VMスレッド上）に閉じチャネル分離 |
| **R5** 隔離・再現性 | `tests/runtime/main.rs` + `#[path] common` 規約／`pub use mlua`（`lib.rs:65`）／`PastaLuaRuntime::with_config` + `RuntimeConfig` | **Missing**: `[features]` セクション未存在 → `lua-debug-poc` 新設（default 無効）。**Constraint**: LuaJIT ビルド前に環境変数 `NoDefaultCurrentDirectoryInExePath` を外さないと `cargo test` が exit 101 |
| **R6** go/no-go 判定成果物 | `research.md`／spec ドキュメント群 | 構造的ギャップなし（成果物＝文書化。R1〜R3・R5 の観測結果を結論へ集約） |

## 実装アプローチ（検証ハーネスの構造）

### Option A: 既存テスト基盤を拡張（推奨）
`crates/pasta_lua/tests/runtime/` に `lua_debug_poc_test.rs` を新設し、`tests/runtime/main.rs` に `#[cfg(feature="lua-debug-poc")] mod lua_debug_poc_test;` を1行追加。VM は `common::e2e_helpers::create_runtime_with_finalize()`（または `with_config` + `runtime.lua()`）で構築。コルーチン横断発火は `scene.lua:212` の `coroutine.create` パターンを模した最小 Lua を複数生成・resume 駆動して Rust 側カウンタで assert。
- ✅ 既存規約・ヘルパに乗るため最小・`cargo test` で再現可・default 無効でリリース非汚染（R5 充足）
- ✅ 実シーンモデルの忠実再現が容易（参照が同クレート内）
- ❌ ソケット往復（R4）の手動確認はテスト内では扱いづらい

### Option B: 独立 example/bin として作成
`examples/` か小規模 bin に PoC を置き、SSP ロードや socket 往復を手動実行で確認。
- ✅ socket 往復・実機（SSP）確認（R4・R5.4）に向く・本体コードと完全分離
- ❌ assert ベースの自動判定が弱く、CI 再現性が落ちる・足場を一から用意

### Option C: ハイブリッド（推奨補完）
R1〜R3・R5 の自動判定は Option A の feature-gate テストで、R4（任意）の socket 往復だけ feature-gate した小 example/bin で手動確認。
- ✅ 自動判定の再現性（A）と socket/実機確認（B）の両取り
- ❌ 配置が2系統になり計画がやや増える

## 工数・リスク

| 観点 | 評価 | 根拠 |
|---|---|---|
| 工数 | **S〜M（3〜7日）** | API・足場は確定済み。新規 feature 1個＋テスト1〜2本＋（任意）小 example。変数inspect の FFI 検証が入るとMより。 |
| リスク | **Medium** | `set_global_hook` × Lua側 `coroutine.create` × LuaJIT の発火が**未実証（#1）**。変数inspect が mlua 高レベルに無く FFI/std_debug 判断が要る（#2）。いずれも回避路はあるが go/no-go の本体。 |

## 設計フェーズへの引き継ぎ

**推奨アプローチ**: Option A を主軸、R4 のみ Option C で補完。

**Research Needed（設計で詰める）**:
1. **【最優先】** `jit.off(true,true)` 適用後、`set_global_hook` が **Lua 側 `coroutine.create` 由来の複数コルーチン**で line フックを撃つか実証。撃たない場合の代替: (a) `coroutine.create` を Lua 側でラップして `Thread::set_hook` を都度張る、(b) LuaJIT のグローバル sethook 挙動に依拠、(c) mlua `create_thread` 経由に寄せる——のいずれが成立するか。
2. **変数 inspect 方式（FFI 経路は成立確認済・要 PoC 実証）**: mlua 0.11.6 は `mlua::ffi`（mlua-sys 0.10.0）を無条件公開し `lua_getlocal`/`lua_getupvalue`/`lua_getstack` を提供、`Lua::exec_raw` で生 `lua_State` を取得できる。フック内で `exec_raw`→`lua_getstack(L,0,&mut ar)`→`lua_getlocal(L,&ar,n)` の正攻法で `std_debug` 非露出のまま（R5.3 維持）ローカル/upvalue を名前・値で取得可能（mlua 固有前例なしのため PoC で実証）。`std_debug`（Lua 側 `debug.getlocal`）はデバッグモード限定の回避策として位置づけ。
3. **`jit.off` の適用範囲・タイミング**: 対象がコンパイルされる前に効かせる必要。`jit.off(true,true)` の再帰効果と、デバッグモード時のみ適用するゲート設計。
4. **R5.4（SSP 実機）の扱い**: `cargo test` を必須エビデンスとし、SSP 実機は Where（可能なら補足）。ブレーク中の SHIORI 応答ブロッキング＝SSP タイムアウトは本検証では「観測・記録」に留め、解決は実装仕様へ送る。
5. **ビルド前提**: テスト実行手順に `NoDefaultCurrentDirectoryInExePath` 解除を明記（LuaJIT ビルド落ち回避）。

**設計で確定すべき主要判断**: (1) コルーチン横断フックの成立方式、(2) 変数inspectの取得経路（FFI vs std_debug）、(3) PoC の配置（A 主軸＋C 補完）と feature 名。

---

## 設計フェーズ追記（2026-06-07）

### Synthesis 適用結果
- **一般化**: R1〜R4 は「pasta 風コルーチンシナリオをデバッグフック下で駆動し観測・制御する」の変奏。単一の PoC ハーネス（共有の最小スキャフォールド）に集約する。実装スコープは現要件に限定し、インタフェースのみ素直に保つ。
- **Build vs Adopt（全て Adopt）**: フック＝mlua `set_global_hook`、生状態＝`Lua::exec_raw`、変数取得＝`mlua::ffi`（`lua_getstack`/`lua_getlocal`/`lua_getupvalue`）、トランスポート＝`std::net`/`std::sync::mpsc`/`std::thread`、VM 構築＝既存 `tests/common/e2e_helpers::create_runtime_with_finalize()`。**DAP・luasocket・追加クレートは不採用**（PoC は最小行プロトコルで十分。DAP/VSCode は実装仕様送り）。
- **Simplification**: R4 の socket 往復はループバックで cargo test 内に閉じ込め、別 example は作らない（Option A 単独、Option C は不要化）。production コードは無改変（`Cargo.toml [features]` 追加とテスト側 gated `mod` 行のみ）。

### Design Decisions
- **D1 コルーチン横断フック**: `set_global_hook` を第一手。Lua 側 `coroutine.create` 由来コルーチンに発火しない場合のフォールバックとして (a) `coroutine.create` を Lua 側でラップし生成毎に `Thread::set_hook`、(b) LuaJIT グローバル sethook 挙動依拠、を順に試行して成立方式を記録（R1.5）。
- **D2 変数 inspect**: FFI 正攻法（`exec_raw`→`lua_getstack(L,0,&ar)`→`lua_getlocal` ／ 対象関数を stack に積んで `lua_getupvalue`）で `std_debug` 非露出のまま実現を第一目標。失敗時のみ `std_debug`（デバッグ時限定）を比較記録（R3.3/3.4）。
- **D3 トランスポート配置（3スレッド）**: VM スレッド（Lua 生成・フック実行）／ listener スレッド（accept・socket I/O）／ client スレッド（= テストドライバ・ループバック接続）。VM 操作はフック内に閉じ `!Send` を遵守し、チャネルで VM スレッド↔listener スレッドを連結（R4.2）。ポートは `127.0.0.1:0` で OS 割当。
- **D4 段階的判定の所在**: ハーネスは項目別 `ItemOutcome` を出力し `compute_tier` で Tier を算出・assert。最終判定文は本 research.md 末尾の「PoC 検証結果」節へ実装完了時に追記（R6）。
- **D5 停止スレッドモデル（本番トポロジ写像・設計ディスカッションで確定）**: R2/R3/R4 は停止コア（`PauseGate`＋チャネル）を共有し、トランスポートのみ差し替える統一モデル。原則 ①VM スレッドはホスト所有（本番=SSP、PoC=テストが SSP 役で spawn。フックは自スレッドを所有しない）②トランスポートは長命 1 スレッド（pasta が spawn してよい唯一のスレッド）③チャネルが唯一の seam（VM は socket 不接触・トランスポート差し替え可・`!Send` 遵守）④無期限ブレークが正で timeout はテスト専用 watchdog（停止コアに組み込まない）。根拠: 本番では SSP がリクエストスレッドを所有し pasta は VM スレッドを spawn できず、SHIORI 同期ゆえブロックは回避不能・受容。将来の非 SHIORI ホスト（ノベルゲームエンジン等）では通常の無期限ブレークが正しい挙動。これにより PoC で証明したトポロジが本番・非 SHIORI へ無改変で運べる。
- **D6 ブレーク制御モデル（設計ディスカッションで確定）**: `PauseGate` が `breakpoints: HashSet<(source,line)>` を保持し、`EVERY_LINE` フック内 `should_pause(frame)` の包含判定で標的行のみ停止。VSCode `setBreakpoints`/`stopped` に直結。step モードは持たない（YAGNI）。

### 追加リスク
- jit.off のタイミング: フック設定・対象コンパイルより前に効かせる必要。ハーネスは VM 構築直後・シナリオ実行前に **グローバル `jit.off()`（無引数）** を exec（R1.4）。【実装フェーズで `jit.off(true,true)` は関数単位制御と判明し無引数 `jit.off()` へ訂正。下記「実装フェーズ知見」参照】
- フック内 FFI のスタック整合: `lua_getlocal`/`lua_getupvalue` 後の `lua_pop` 漏れで VM を壊さないよう、取得処理は `exec_raw` クロージャ内でスタックを必ず復元する。

---

## 設計検証メモ（2026-06-07・/kiro-validate-design 結果 = GO）

**全 3 件は設計ディスカッション（2026-06-07）で design.md へ反映済み**: 検証Issue1→D5（停止スレッドモデル統一）、検証Issue2→D6（ブレークポイント集合方式）、検証Issue3→A1（モジュールパス修正）。以下は経緯記録。

- **検証Issue 1（R2/R3 の停止検証スレッド構成・未定義）**: フック内ブロッキング停止は VM 同一スレッドだとテスト自身がデッドロックする。R4 のみ3スレッド構成が明記され、R2/R3 単体テストの「VM 別スレッド＋コントローラ（join timeout）」構成と、停止継続の観測手順（ブレーク後に Lua が立てる共有アトミックの非進行→Continue→進行確認）、`2.4` のデッドロック検出（watchdog/timeout）が未記載。→ design.md の System Flows/PauseGate/Error Handling へ追記候補。Traceability: 2.1, 2.2, 2.4, 3.2。
- **検証Issue 2（ブレークポイント標的機構・未定義）**: `EVERY_LINE` で全行発火するが「どの (source,line) で停止するか」の標的集合・`should_pause(frame)` 判定が未定義。→ HookProbe か PauseGate に `breakpoints: HashSet<(String,u32)>` と停止条件を明示する候補。Traceability: 2.1, 3.2, 4.1。
- **検証Issue 3（File Structure のモジュールパス不整合・確実にビルド不通）**: 非 mod.rs の `lua_debug_poc_test.rs` から `mod debug_poc;` は Rust 2018 規則で `tests/runtime/lua_debug_poc_test/debug_poc/` を探す。design.md の `tests/runtime/debug_poc/` では解決不可。→ `tests/runtime/lua_debug_poc_test/debug_poc/` へ移動 or `#[path]` 指定 or 単一ファイル集約。Traceability: 5.1。

---

## 実装フェーズ知見（2026-06-07・/kiro-impl）

### LuaJIT jit.off セマンティクス（タスク 1.2 で実証）

- **Context**: タスク 1.2 で jit 無効化 VM ヘルパを実装中、設計が mandate していた `jit.off(true,true)` を VM 構築時に別チャンクで exec しても、ヘルパが返す VM の `jit.status()` 第一返値が `true` のまま（＝ JIT エンジンが有効）であることを実機 cargo test で観測。
- **Sources Consulted**: LuaJIT 公式ドキュメント [Extensions / jit.* library](https://luajit.org/ext_jit.html)（`jit.on`/`jit.off`/`jit.flush`/`jit.status`）＋ 当該 vendored LuaJIT 2.1 ビルドでの実測。
- **Findings**:
  - 無引数 `jit.off()` … **JIT コンパイラエンジン全体**を停止。以後 `jit.status()` 第一返値は `false`。VM 全体・後続ロードコード・動的生成コルーチンすべてに波及。
  - `jit.off(func|true [, recursive])`（例 `jit.off(true,true)`）… **関数単位**の制御。「呼び出し元関数（`true`）＋（recursive=`true` なら）その下位関数」のコンパイルを無効化し既存トレースを flush するのみ。**グローバルエンジン状態（`jit.status()`）は変えない**。公式の想定用途は「デバッグしたいモジュールの main chunk 先頭に置く」イディオム。
  - したがって VM 構築時に別チャンクで `jit.off(true,true)` を exec しても、別 `load` で後からロードされるシーンチャンクや `coroutine.create` 由来コルーチンには効かず、ラインフック取りこぼし防止（R1.4）の目的を**満たさない**。
- **Decision / Implications**: PoC は VM 全体への確実な適用として **無引数 `jit.off()`** を採用（design.md「jit.off セマンティクス注」・requirements.md R1.1・tech スタック表を訂正）。ヘルパの観測条件は `jit.status()` 第一返値 == `false` で確認可能となり、タスク 1.2 の完了条件（「ヘルパが返す VM で jit が無効化されている」）を真正に充足。サンドボックス（R5.3）は不変（`debug` グローバルは nil のまま）。後続タスク 2.1（HookProbe / R1.4）は、この無引数 `jit.off()` 済み VM 上でコルーチン横断のフック取りこぼしゼロを検証する。

---

## PoC 検証結果（2026-06-07・/kiro-impl 完了）

> 本節は段階的 go/no-go 判定の最終成果物（Requirement 6.5）。`cargo test -p pasta_lua --features lua-debug-poc --test runtime`（必須エビデンス）で全項目を試行・実証した結果を集約する。各項目の裏付けテストは `crates/pasta_lua/tests/runtime/lua_debug_poc_test/` 配下に存在し、判定算出は `verdict.rs` の `compute_tier` ／ `report`（`poc_verdict_aggregation_reports_goplus` テストが GO+ を assert・出力）が担う。

### 到達段階 = **GO+（GoPlus）**

チャレンジ項目 R1〜R4 が**すべて成立**し、単調な積み上げ（R1→R2→R3→R4）の最上位段階 GO+（高信頼）に到達した。`compute_tier` 入力で R1〜R4 すべて `passed=true` → `Tier::GoPlus`。後続実装仕様 `pasta-vscode-lua-debug` は本判定を着手前提として実装に進める。

| 項目 | 成否 | 採用方式 | 主要制約 | 裏付けテスト |
|---|---|---|---|---|
| **R1** フック発火（コルーチン横断） | **GO** | グローバル `jit.off()` 済み VM ＋ `set_global_hook(EVERY_LINE)`＝`HookStrategy::GlobalHook` | フックコールバックの `&Lua` は常にメインステート（走行中コルーチンの `lua_State*` は非伝達）。横断識別は記録 source/line で実施 | `hook_probe`: `hook_fires_on_single_chunk` / `hook_fires_across_dynamic_coroutines` |
| **R2** フック内停止・再開 | **成立** | フック内ブロッキング `recv()`（yield 不使用）→ 常に `VmState::Continue` | LuaJIT は `Yield` 不可。停止コアは無期限 `recv()`、timeout はテスト専用 watchdog | `session`: `session_stops_at_breakpoint_and_resumes` / `progress_advances_when_no_breakpoint` |
| **R3** 変数 inspect | **成立** | `Lua::exec_raw` ＋ `mlua::ffi`（`lua_getstack`/`lua_getlocal`/`lua_getupvalue`/`lua_getinfo`） | **R3.4**: 走行中コルーチン本体フレームはメインステート FFI から不可達（メインフレームのみ実証）。`std_debug` 非露出 | `frame_inspector`: `inspect_locals_via_ffi_basic_types` / `inspect_upvalues_via_ffi` / `frame_info_via_safe_api` / `inspect_unsupported_type_recorded` |
| **R4** トランスポート往復 | **成立** | 別スレッドの `std::net` ループバック（`127.0.0.1:0`）＋最小行プロトコル（`stopped`/`vars`/`continue`） | `mlua::Lua` の `!Send` を 3 スレッド分離で遵守。追加クレートなし（`std` のみ） | `transport_loop`: `transport_round_trip_loopback` / `round_trip_carries_real_inspected_vars` |

### 採用フック方式

- **JIT 無効化**: 無引数 `jit.off()`（エンジン全体停止・`jit.status()` 第一返値 `false`）。`jit.off(true,true)` は関数単位制御でグローバルエンジン状態を変えず取りこぼし防止（R1.4）に不十分なため不採用（上記「LuaJIT jit.off セマンティクス」参照）。
- **フック設置**: `mlua::Lua::set_global_hook` ＋ `HookTriggers::EVERY_LINE`、戦略は `HookStrategy::GlobalHook`。Lua 側 `coroutine.create` 由来の**動的コルーチン群すべてに line フックが発火**することを実証（LuaJIT #666 のグローバル sethook 挙動）。D1 フォールバック（Rust 製コルーチン生成差し替え＋スレッド毎フック）は**不要**だった。
- **フック戻り値**: 常に `VmState::Continue`（LuaJIT は `Yield` 不可）。停止はフック内 Rust ブロッキングのみで実現（`coroutine.yield`/`lua_yield` 不使用）。
- **注意**: フックコールバック内の `&Lua` は常にメインステートで、走行中コルーチンの `lua_State*` は渡されない（`thread_ptr` はベストエフォート）。コルーチン横断の識別は記録した source/line 内容で行う。

### 変数 inspect 方式

- **経路**: `Lua::exec_raw` で生 `lua_State` を取得し、`mlua::ffi`（`lua_getstack` で現フレーム ar 再取得 → `lua_getlocal`／`lua_getupvalue`、`lua_getinfo` でフレーム情報）でローカル・upvalue を**名前＋値**で取得。
- **型判別**: number / string / boolean / table を `lua_type` で判別。inspect 非対応種別（function/nil/userdata/thread/cdata）は `lua_typename` で `<unsupported ...>` として記録（クラッシュせずスタック維持）。
- **サンドボックス維持**: `std_debug` を**非露出**のまま成立（`debug == nil` を維持・R5.3 整合）。Lua 側 `debug.getlocal` は不要。
- **スタック整合**: `exec_raw` クロージャ内で entry/exit の `lua_gettop` 一致を保証（必ず復元）。
- **exec_raw レベルオフセット知見**: `exec_raw` はクロージャを内部 `lua_pcall`（`do_call` C フレーム）下で実行するため `lua_getstack(L,0)` は C フレームを指す。`what` が "Lua"/"main" の最初のフレームを走査して選ぶ（`find_first_lua_frame`）必要がある。

### 既知制約（後続実装仕様への重要引き継ぎ）

1. **R3.4 コルーチン本体フレーム不可達**: フックの `&Lua` および `exec_raw` はメインステートで動くため、**走行中コルーチン本体のフレームは別 `lua_State` 上にあり、このメインステート FFI 経路からは到達不可**。本 PoC はメインスレッド（トップレベル）Lua フレームの変数取得を実証。pasta のシーンはコルーチンで走るため、コルーチン内ローカルの inspect は実装仕様でコルーチン自身の state を辿る等の追加対応が必要（回避策: デバッグ時限定の `std_debug` 露出）。
2. **`mlua::Error` は `!Send`**: `Arc<dyn Error>` を保持するため `JoinHandle<mlua::Result<()>>` がスレッド境界を越えられない。VM スレッド境界では mlua エラーを `String` 等 Send 安全な値へ変換して渡す必要がある（thread model ③ 準拠）。
3. **MSVC で hook panic ペイロード消失**: フック内パニックは `catch_unwind(AssertUnwindSafe)` で同一スレッド捕捉でき VM スレッドは異常終了しないが、**MSVC/LuaJIT の C-unwind 境界で panic ペイロードが失われる**（`&str`/`String` への downcast 不可・不透明 TypeId）。原因記録にはフック内で Send 安全なサイドチャネルへ事前格納が必要。
4. **`jit.off(true,true)` は関数単位で不十分**: VM 全体への確実な JIT 無効化には無引数 `jit.off()` が必須。
5. **inspect 非対応種別**: function/nil/userdata/thread/cdata は `<unsupported ...>` として記録（基本型の取得は継続・R3.4）。

### SSP 応答ブロッキングの取り扱い方針

- ブレーク中は SSP のリクエストスレッドを**無期限ブロック**する（無期限ブレークが正・スレッドモデル ④）。SHIORI は同期プロトコルゆえブロックは回避不能で受容する。
- これにより **SSP タイムアウトのリスク**が生じるが、本 PoC では**観測・記録に留める**（Non-Goals）。緩和策（応答ブロッキング回避）の設計は後続実装仕様 `pasta-vscode-lua-debug` の領分とする。
- 将来の非 SHIORI ホスト（ノベルゲームエンジン等）では通常の無期限ブレークが正しい挙動であり、本 PoC が証明した停止トポロジは本番・非 SHIORI へ無改変で運べる。

### 隔離前提（Requirement 5.2 / 6.3）

全検証は以下の隔離条件のもとで成立した（判定の妥当性前提）。

- **feature-gate**: `lua-debug-poc`（default 無効）。production コードは無改変（`Cargo.toml [features]` 追加と `tests/runtime/main.rs` の gated `mod` 行のみ）。
- **実行手段**: `cargo test --features lua-debug-poc`（必須エビデンス）。
- **サンドボックス維持**: `std_debug` 非露出（`debug == nil`）のまま全項目が成立。
- feature 無効の既定（リリース）ビルド・既存テストは不変（`cargo build -p pasta_lua` がハーネス非コンパイルで完了）。

### SSP 実機確認（Requirement 5.4・任意）

本 PoC の**必須エビデンスは `cargo test`** である。SSP ロード実機での発火・停止挙動の確認は**任意**であり、**未実施**（cargo test を必須エビデンスとする）。実装仕様での DAP 統合時に実機確認を行うことが望ましい。

### 後続実装仕様 `pasta-vscode-lua-debug` への引き継ぎ結論

- **判定**: **GO+**。実装着手可。
- **採用すべき基盤**（本 PoC で実証済み）:
  - グローバル `jit.off()` ＋ `set_global_hook(EVERY_LINE)` による全行フック（`HookStrategy::GlobalHook`）。
  - `breakpoints: HashSet<(source,line)>` ＋ `should_pause` の包含判定で標的行のみ停止。
  - `Lua::exec_raw` ＋ `mlua::ffi` による変数 inspect（`std_debug` 非露出）。
  - Rust ホスト型 `std::net` トランスポート（最小行プロトコル、追加クレート最小）。停止コアとトランスポートは分離（差し替え可）。
- **実装側で要対応（本 PoC の制約に対応）**:
  - **コルーチン本体フレームの変数 inspect 対応**（R3.4・最重要。コルーチン自身の state を辿る等）。
  - DAP プロトコル変換、`.pasta` ソースマップ。
  - SSP 応答ブロッキングの緩和策設計。
  - hook panic 原因のサイドチャネル記録（MSVC C-unwind 境界対策）。
  - `mlua::Error`（`!Send`）をスレッド境界で扱う Send 安全なエラー設計。
- **検証コードの除去（使い捨て前提・Req 5.5）**: 本 PoC ハーネス（feature `lua-debug-poc` 一式）は、`pasta-vscode-lua-debug` で採用方式の本番移行が完全に完了・検証された時点で**除去する**こと。これは下流仕様 `pasta-vscode-lua-debug` の**完了条件（要件）**として明記済み（同 `brief.md`「完了条件: 検証コードの除去」節）。移行後の再検証エビデンスは本「PoC 検証結果」節（文書）＋本番テストで担保する。
