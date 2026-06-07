# Gap Analysis: pasta-vscode-lua-debug

> 本書は要件（requirements.md）と既存コードベースの実装ギャップを分析し、設計フェーズの意思決定材料を提供する。**決定ではなく情報と選択肢**を示す。上流 `pasta-lua-debug-feasibility`（GO+）が採用方式と既知制約を実証済みのため、本分析は「PoC 知見の本番化に向けた既存資産との接合点」に焦点を置く。

## 分析サマリー

- **全体像**: 4 サブシステム（① Rust ホスト型デバッグバックエンド、② `.pasta` ソースマップ＝code_gen 拡張、③ VSCode 拡張 DAP 統合、④ 旧資産撤去＋PoC ハーネス除去）の協調。技術的実現性は上流 PoC で実証済み（GO+）。
- **最大の新規性**: ①のデバッグバックエンド（transport＋DAP＋hook＋状態機械）は pasta_lua に**新規モジュール**として追加するのが自然。PoC ハーネス（`lua_debug_poc_test/` の 7 サブモジュール）が本番設計の青写真になる。
- **既存資産で吸収できる部分**: ②ソースマップは `LuaCodeGenerator::writeln`（出力一元点・mod.rs:53-57）への行カウンタ注入で実現可能。③VSCode 統合は既存 `activate()` の register パターンに沿って低コスト。デバッグ有効化フラグは `RuntimeConfig`/`LuaConfig` の既存経路を流用可能。
- **最大のリスク**: コルーチン本体フレームの変数 inspect（PoC 制約 R3.4 の未解決部分）。pasta のシーンはコルーチンで走るため必須だが、PoC はメインステートのみ実証。設計フェーズで別 `lua_State` を辿る方式の確証が必要（**Research Needed**）。
- **構造的制約**: ブレーク中は SHIORI 応答が停止（Mutex 直列・同期 blocking）。根本解決はスコープ外、緩和策に留める前提で設計する。

---

## 1. 現状調査（Current State）

### 1.1 VSCode 拡張（editors/vscode/）
- `package.json`: `contributes` は languages/grammars/semanticTokens のみ。**`debuggers` 不在**。`engines.vscode: ^1.85.0`、`@types/vscode: ^1.85.0`（DAP API は 1.41+ 安定のため互換）。
- ビルド: `esbuild`（compile）→ `vsce package`（VSIX）。`prepackage` で `build:wasm`（**wasm-pack 必須**・scripts/build-wasm.ps1）→ compile。
- `src/extension.ts`: `activate()` は OutputChannel → DiagnosticsManager → WasmBridge 初期化 → `registerDocumentSemanticTokensProvider` → DocumentSync の順。`context.subscriptions.push()` で disposable 管理する確立パターンあり。
- テスト: `src/test/` に esbuild + node CLI テスト（tmGrammar/wasmBridge/integration）。

### 1.2 code_gen（`.pasta`→Lua 生成）
- `LuaCodeGenerator { writer, indent_level, line_ending }`（mod.rs:17-24）。**出力行カウンタなし**。
- 出力は `writeln`/`write_blank_line`/`write_raw` の 3 メソッドに集約（mod.rs:46-69）。**`writeln` が行↔span 対応の単一絞り込み点**になり得る。
- `generate_action`（element_gen.rs:236）等で Action の `span` フィールドは利用可能だが出力に渡されず破棄。span（start_line/start_col）は完全に可用（pasta_dsl ast/span）。
- `TranspilerConfig::comment_mode`（config.rs:46）は宣言済みだが code_gen 未使用。

### 1.3 Lua ランタイム / VM 初期化
- `Lua::unsafe_new_with(std_lib, LuaOptions::default())`（runtime/mod.rs:108）。**`set_hook`/`set_global_hook` 未使用**。
- `debug` ライブラリは既定 OFF（`StdLib::ALL_SAFE`）。`RuntimeConfig.libs` に `std_debug` を加える経路は**既存**（runtime_config.rs:161 が `StdLib::DEBUG` へマッピング、`validate_and_warn` で非本番警告）。
- pasta.toml → `LuaConfig { libs }`（config.rs:305-320）→ `From<LuaConfig> for RuntimeConfig`（runtime_config.rs:236）。`[lua]` セクションがフラグ追加の自然な置き場所。
- Rust 側 `std::env` 利用は現状なし（環境変数フラグは新規パターン）。Lua 側 `@env` はセキュリティ上無効。
- モジュール登録は `register` 系 public fn の確立パターン（runtime/mod.rs:114-135、`log.rs` がスタック検査・値→文字列変換の参照実装）。

### 1.4 旧 luasocket デバッグ資産（未配線・残存）
- `crates/pasta_lua/pasta_scripts/` 配下に `vscode-debuggee.lua`、`socket/core.dll`（~102KB）、`mime/core.dll`（~75KB）、`dkjson.lua`。DLL 内蔵 zip に同梱されるが主プログラムから未参照。

### 1.5 request 同期性（構造的制約）
- `pasta_shiori`：`request()` が Arc<Mutex> 直列・`request_fn.call::<String>()` で blocking（shiori.rs:145-310）。ブレークで VM が止まると Lua 関数が復帰せず SHIORI 応答が返せない＝**ブレーク中の応答停止は設計上の必然**。

### 1.6 上流 PoC ハーネス（青写真・除去対象）
- `crates/pasta_lua/tests/runtime/lua_debug_poc_test/`：harness_types / hook_probe / pause_gate / frame_inspector / session / transport_loop / verdict の 7 サブモジュール（feature `lua-debug-poc` gate、default 無効・test 専用）。本番モジュール設計の直接の下敷きになる。

---

## 2. 要件→資産マップ（Requirement-to-Asset Map）

| 要件 | 既存資産 | ギャップ | タグ |
|---|---|---|---|
| **R1** ブレーク/ステップ（over/into/out）・コルーチン横断 | PoC `pause_gate`/`session`（実証済み青写真）、`set_global_hook`（PoC で実証） | 本番停止コア・状態機械（step 種別判定）が未実装。VM 側 hook 未配線 | Missing |
| **R2** コールスタック・変数 inspect | PoC `frame_inspector`（FFI で number/string/bool/table 取得・実証）、`log.rs`（スタック検査パターン） | 本番フレーム→変数 API 化。**コルーチン本体フレーム inspect は PoC 制約 R3.4 で未実証** | Missing / Unknown |
| **R3** DAP 最小サブセット・attach 接続 | PoC `transport_loop`（std::net 往復・最小行プロトコル実証）、`serde_json`（既存依存） | DAP メッセージ（initialize/setBreakpoints/…/stopped/terminated）の手書きマッピングが未実装 | Missing |
| **R3-6** VSCode 拡張デバッグ構成 | 既存 `activate()` register パターン、`@types/vscode 1.85` | `contributes.debuggers` ＋ `DebugAdapterDescriptorFactory`（**`DebugAdapterServer(port)` 返却の薄実装**）が未実装 | Missing |
| **R4** `.pasta` ソースマップ | `LuaCodeGenerator::writeln`（単一絞り込み点）、Action.span（可用） | 行カウンタ＋行↔span 記録＋Lua 行↔.pasta 行変換が未実装 | Missing |
| **R5** デバッグ有効化フラグ・本番ゼロコスト | `RuntimeConfig`/`LuaConfig.libs`、`std_debug` マッピング経路 | デバッグ用フラグ（`[lua] debug` 等）と hook 設置の条件分岐、無効時ネットワーク非起動の保証 | Missing（経路は既存） |
| **R6** ホスト非依存基盤 | pasta_lua レイヤ（SHIORI は上位） | デバッグ API を SHIORI 非依存 IF として公開する境界設計 | Constraint |
| **R7** ブレーク中応答停止・緩和 | `pasta_shiori` Mutex 直列（制約の出所） | 緩和策・運用ガイダンスの提供（根本解決はスコープ外） | Constraint |
| **R8** 旧 luasocket 撤去 | `pasta_scripts/` 内の該当ファイル群、build.rs（zip 化） | ファイル削除＋zip 再生成＋回帰なし確認 | Missing（撤去作業） |
| **R9** PoC ハーネス除去 | `lua-debug-poc` feature／テスト一式 | 前提充足後の削除（完了条件）。本番テストでの GO+ 担保移行 | Constraint（完了条件） |

---

## 3. 実装アプローチ選択肢

サブシステムごとに最適解が異なるため、**全体としてはハイブリッド（Option C）**になる。各サブシステムの選択肢を示す。

### 3.1 デバッグバックエンド（transport＋DAP＋hook＋状態機械）
- **Option A（既存 runtime 内に展開）**: `runtime/` に直接実装。`log.rs` 並置（`runtime/debug.rs`）。✅近接・参照実装あり ❌runtime/ が肥大、ホスト非依存（R6）の境界が曖昧化。
- **Option B（新規 `debug/` モジュール）★推奨**: `crates/pasta_lua/src/debug/`（transport / dap / hook / state / source_map のサブモジュール）。PoC の 7 サブモジュール構成を本番化。✅R6 のホスト非依存境界が明確、✅単体テスト容易、✅runtime と疎結合 ❌ファイル増・runtime との IF 設計が必要。
- **Option C（ハイブリッド）**: hook 設置のみ runtime に薄く配線、停止コア・transport・DAP は新規 `debug/`。✅`!Send` な `mlua::Lua` 所有は runtime、操作は hook 内（VM スレッド）でチャネル経由という PoC のスレッドモデルに整合。
- **トレードオフ要約**: R6（ホスト非依存・再利用）を満たすには B/C が有利。PoC が証明したスレッド分離（socket=I/O、VM 操作=hook 内、チャネル seam）を素直に写せるのも B/C。

### 3.2 `.pasta` ソースマップ（code_gen 拡張）
- **Option A（既存拡張）★推奨**: `LuaCodeGenerator` に出力行カウンタを追加し、`writeln`（単一点）で「現在 Lua 行 → 現在 span」を記録。`generate_action` 等に span を引き回す。✅注入点が一箇所に集約、✅既存パターン内 ❌code_gen の全 `generate_*` に span 受け渡しの軽微改修が波及。
- **Option B（新規パス）**: 別途 AST 走査で行対応表を生成。❌出力行と二重管理になり乖離リスク（非推奨）。
- 生成物（行対応表）の保持先・シリアライズ形式は設計判断（**Research Needed**: マップを生成 Lua に焼くか、別データとして runtime に渡すか）。

### 3.3 VSCode 拡張 DAP 統合
- **Option A（既存拡張へ追加）★推奨**: `package.json` に `contributes.debuggers`（`type: pasta`）＋`activationEvents` に `onDebug:pasta`、`extension.ts` の `activate()` に `vscode.debug.registerDebugAdapterDescriptorFactory()` を追加。Factory は **`DebugAdapterServer(port)`** を返す薄実装（DAP 本体は Rust 側＝attach）。✅ビルド/パッケージング（esbuild→vsce）に変更なし、✅低リスク ❌DAP 用テスト新規。
- **注意**: brief の設計意図は「拡張は薄く、サーバ記述子を返すだけ」。バンドル JS アダプタ（`program: debugAdapter.js`）方式は不採用（Rust バックエンドが DAP を話す）。

### 3.4 旧資産撤去・PoC ハーネス除去
- いずれも**削除作業**（新規/拡張の判断対象外）。R8 は撤去後の回帰なし確認、R9 は完了条件（前提充足後に実施）。

---

## 4. 工数・リスク評価

| サブシステム | 工数 | リスク | 根拠 |
|---|---|---|---|
| デバッグバックエンド（hook/状態機械/停止コア） | **L** | **Medium** | PoC で核心は実証済みだが、step over/into/out の状態機械と DAP 変換は本番新規。スレッド/`!Send`/panic 境界の既知制約に沿う必要 |
| 変数 inspect（コルーチン本体フレーム） | **M** | **High** | PoC 制約 R3.4 が未実証。別 `lua_State` 走査の確証が必要。pasta シーン＝コルーチンのため必須 |
| `.pasta` ソースマップ（code_gen） | **M** | **Medium** | 注入点は単一だが全 `generate_*` への span 波及と Lua↔.pasta 双方向変換の正確性確保 |
| DAP プロトコル（手書き最小サブセット） | **M** | **Medium** | 外部標準仕様への準拠。serde_json で手書き、リクエスト/イベント網羅 |
| VSCode 拡張統合 | **S** | **Low** | 既存 register パターン＋薄い Factory。ビルド変更なし |
| デバッグ有効化フラグ・ゼロコスト | **S** | **Low** | `RuntimeConfig`/`LuaConfig` 既存経路を流用 |
| 旧 luasocket 撤去 | **S** | **Low** | ファイル削除＋zip 再生成＋起動回帰確認 |
| PoC ハーネス除去 | **S** | **Low** | 前提充足後の削除。完了条件ゲート |
| **総計** | **L〜XL** | **Medium（局所 High）** | 複数サブシステム協調。コルーチン inspect が最大の未知 |

---

## 5. 設計フェーズへの申し送り

### 推奨アプローチ（preferred）
- **全体ハイブリッド**: デバッグバックエンドは**新規 `crates/pasta_lua/src/debug/`**（PoC 7 サブモジュールを本番化、R6 ホスト非依存境界を明示）、ソースマップは **code_gen 既存拡張**（`writeln` 単一点＋行カウンタ）、VSCode は**既存拡張へ薄い Factory 追加**（`DebugAdapterServer(port)` attach）、有効化は **`RuntimeConfig`/`LuaConfig` 既存経路**。
- **スレッドモデル**: PoC 実証どおり「VM 所有=runtime/ホストスレッド、socket=I/O 専用スレッド、VM 操作=hook 内、チャネルが唯一の seam（`mlua::Lua` を move しない）」を設計の Boundary Commitment として固定。
- **デバッグモード時の JIT**: 無引数 `jit.off()`（エンジン全体）を有効化条件に結線（`jit.off(true,true)` は不十分＝PoC 知見）。

### Research Needed（設計で確証）
1. **コルーチン本体フレームの変数 inspect**（R3.4）: hook の `&Lua`＝メインステートから走行中コルーチンの `lua_State` を辿る具体手段（FFI 経路・mlua 0.11 で安全に到達できるか）。**最優先**。
2. **ソースマップの保持/受け渡し形式**: 生成 Lua への埋め込み（行コメント/チャンク名）か、別データを runtime/debug へ渡すか。Lua スタックの `currentline`↔.pasta 行の解決経路。
3. **DAP 最小サブセットの境界**: variablesReference の階層（table 展開）方針、scopes の粒度（local/upvalue）、source 参照の path 表現。
4. **panic/エラー境界**: hook 内 panic のサイドチャネル記録（MSVC/LuaJIT C-unwind でペイロード消失の PoC 知見）、`mlua::Error`（`!Send`）のスレッド越え String 変換。
5. **ブレーク中応答停止の緩和策の具体**: SSP タイムアウト回避の運用ガイダンス（デバッグ専用起動モード・タイムアウト延長の現実的手段）。
6. **PoC→本番のコード移行範囲**: `frame_inspector`/`transport_loop`/`session` のどこを本番 `debug/` へ昇格し、どこを作り直すか（R9 除去前提の「同等以上の自動テスト」設計を含む）。

### Boundary 継続性メモ（design の Boundary Commitments で確定すべき点）
- **In**: pasta_lua 内蔵デバッグ基盤、DAP 最小サブセット、コルーチン inspect、有効化フラグ・ゼロコスト、旧資産撤去、PoC 除去（完了条件）、**`.pasta` ソースマップの実現可能性確定（調査＋薄い実証スライス＋設計シーム）**。
- **Out**: **`.pasta` ソースマップ本番実装（→別仕様）**、条件付き BP/ウォッチ式/ホットリロード、非 SHIORI ホスト実配線、SSP タイムアウト根本解決、LSP。

---

## 追補: スコープ決定（`.pasta` ソースマップの分割）｜2026-06-07

ユーザー判断により、**`.pasta`↔`.lua` ソースマップと `.pasta` 座標でのブレークポイントの本番実装を、ダウンストリーム別仕様（仮称 `pasta-source-map`）へ分割**することを確定した。本仕様の出荷コアは **Lua レベルのデバッグ**（生成 `.lua` 上で BP/ステップ/変数 inspect・VSCode attach）に再定義する。本仕様には `.pasta` ソースマップの**実現可能性確定**（調査確定）責務として、**調査＋薄い実証スライス＋将来仕様向け設計シーム**を残す（要件 R4 に反映）。

### この決定が gap 分析に与える影響（差分）

- **§2 要件→資産マップ R4 の再解釈**: 「ソースマップ本番実装（Missing）」から、**「実現可能性確定＝調査＋薄い実証スライス＋設計シーム（Missing だが規模縮小）」**へ縮退。本番マップ出力（全 `generate_*` 網羅・双方向変換の本番品質）は別仕様の Missing として送る。
- **§3.2 ソースマップ（code_gen 拡張）の再解釈**: Option A（`writeln` 単一点＋行カウンタ）は**設計シームの素地**として最小実装に留める（出力行↔span を記録できる接合点の用意まで）。本番の全経路マッピングは別仕様。
- **§4 工数・リスクの差分**: 本仕様の「`.pasta` ソースマップ」分は **M/Medium → S〜M/Medium**（薄い実証スライス＋シームに縮小）。本番ソースマップの **M/Medium は別仕様へ移送**。本仕様総計は **L〜XL → L** に低下見込み（コルーチン inspect の局所 High は据え置き）。

### 薄い実証スライスの達成定義（design で具体化する受け入れの芽）

- 代表 1 経路（例: 単純な talk アクション 1 行）について、生成 `.lua` の停止位置 → 対応 `.pasta` 行へ変換し、`.pasta` 行に張った BP がヒットすることを **experimental／フィーチャーgate 下**で実証する。
- 無効時（本番）はスライスのコード経路が露出せずゼロコスト（R4.6 / R5 と結線）。
- 実証で判明した残課題（全 `generate_*` 網羅・`currentline` 端ケース・双方向変換の正確性・source パス表現）を**別仕様への申し送り**として `research.md` に追記する。

### 設計シーム（将来別仕様の差し込み口）として design で確定すべきもの

1. **code_gen 接合点**: `LuaCodeGenerator` の出力行↔`.pasta` span を記録できるフック点（`writeln` 単一点）。本番では全 `generate_*` がここを通る前提の IF を定義。
2. **マップ受け渡し IF**: 行対応データを runtime/debug へ渡す型・経路（生成 Lua 埋め込み or 別データ）。本仕様では最小定義＋スライス分のみ充填。
3. **DAP source 取り扱い口**: `source`/`stackTrace` が将来 `.pasta` パスを提示できる構造。本仕様の既定提示は生成 `.lua`。

### Research Needed の更新

- **#2 を分割**: 「ソースマップの保持/受け渡し形式」は **(a) 本仕様で確定する最小シーム＋スライス**と **(b) 別仕様で確定する本番形式**に二分する。本仕様 design では (a) のみ確定し、(b) は別仕様へ申し送り。
- 新規: **別仕様の分割境界の明文化**（どこまでが本仕様のシーム/スライスで、どこからが別仕様の本番実装か）を design の Boundary Commitments に明記する。

---

## 追補: 設計 synthesis（Light Discovery）｜2026-06-07

`design.md` 生成に先立ち、PoC 実コードの精読（lua_debug_poc_test 7 サブモジュール）と既存接合面（gap 分析）を統合し、3 レンズを適用した。

### Discovery: PoC からの本番昇格マップ

| PoC サブモジュール | 本番 `debug/` 先 | 方針 |
|---|---|---|
| harness_types（LineEvent/Variable/FrameInfo/DebugCommand/DebugEvent/Breakpoint） | session.rs / inspect.rs の共有型 | **そのまま昇格**（型シグネチャ流用） |
| hook_probe（set_global_hook + jit.off） | hook.rs | 昇格。**D1 per-coroutine フォールバックは削除**（GlobalHook で R1 成立済み） |
| pause_gate（should_pause/block_until_command） | session.rs | 昇格。**watchdog timeout 削除**（本番は無期限ブレークが正） |
| frame_inspector（FFI lua_getstack/getlocal、型判別） | inspect.rs | 安全 API＋型判別は昇格。**変数取得対象を `current_thread().state()` へ変更**（R2.4 解決） |
| session（VM ホストスレッド＋mpsc seam） | session.rs / hook.rs | スレッドモデル昇格。SSP リクエスト境界に適応 |
| transport_loop（TcpListener＋行プロトコル） | transport.rs / dap.rs | TCP I/O は昇格。**行プロトコル→DAP Content-Length フレーミングへ作り直し** |
| verdict（compute_tier） | （昇格せず） | 検証専用。R9 で除去 |

### R3.4（コルーチン本体フレーム inspect）の解決方針（最重要）

- PoC は `Lua::exec_raw` 経由で **メインステート固定**の `lua_State` に対し `lua_getstack`/`lua_getlocal` を撃つため、走行中コルーチン本体フレームへ未到達（既知制約）。
- 本番は、フックの `&Lua`（メインステート）ではなく **`lua.current_thread().state()`（走行中コルーチンの生 `lua_State*`、PoC で `LineEvent.thread_ptr` 取得に既使用）** に対して `lua_getstack`/`lua_getlocal`/`lua_getupvalue` を走査する。これによりコルーチン局所変数へ到達。**本番唯一の新規 FFI リスク**として薄い実証で先行確認（design Open Risk R-2）。

### Lens 1: Generalization
- R1（BP/step）・R2（callstack/inspect）・R3（DAP）は「停止した VM を制御しクライアントと状態交換する 1 つのデバッグセッション」の変種 → **`DebugSession`（protocol 非依存コア）＋ `DapAdapter`（変換）＋ `Transport`（I/O）** に三分割。R6（ホスト非依存）と将来プロトコル拡張を **interface レベルで一般化**（実装は DAP 単一）。
- step over/into/out（R1.3–1.5）は「スタック深さ条件＋行変化で停止」の変種 → **単一 `StepController`（`StepKind` でパラメタ化）** に一般化。

### Lens 2: Build vs Adopt
- **Adopt**: DAP 標準（Microsoft 仕様）／`std::net::TcpListener`／`mlua set_global_hook`＋`jit.off()`／`mlua::ffi`。いずれも追加依存ゼロ・PoC 実証済み。
- **Build（最小）**: DAP メッセージは `serde_json` で手書き（`dap` クレートは alpha・依存最小方針で却下＝brief 決定の踏襲）。code_gen ソースマップ・シーム（既存解なし、本番は別仕様）。

### Lens 3: Simplification（投機的抽象の排除）
- D1 per-coroutine フォールバック戦略を**不採用**（GlobalHook が実証済み・"just in case" 排除）。
- watchdog timeout を本番コアから**除去**（テスト専用）。
- **単一クライアント・単一接続**（PoC の accept-one 踏襲、マルチクライアント非構築）。
- DAP table 展開（variablesReference 階層）は**最小**（深掘りは将来）。

### 設計確定により closed/移送された Research Needed
- #1 コルーチン inspect → **方針確定**（current_thread state 走査）。実証で先行確認（Open Risk R-2 として残置）。
- #2 ソースマップ形式 → (a) 本仕様シーム＋スライスを `SourceMapSink`/`LineMap` で確定、(b) 本番形式は `pasta-source-map` へ移送。
- #3 DAP 境界 → 最小サブセット表（design API Contract）で確定。
- #4 panic/エラー境界 → hook 内 panic サイドチャネル＋`mlua::Error` の String 越境を `DebugError`/`SessionEvent::Error` で確定。
- #5 SSP 緩和 → BreakStallGuidance（R7）で運用注意に確定。
- #6 PoC 移行範囲 → 上記昇格マップで確定（R9 ゲート）。

---

## R4 薄い実証スライス：実現可能性 RESULT と残課題（task 5.3）

> feature `pasta-source-map-slice`（default 無効）配下で、`.pasta`↔生成 `.lua` の行
> 対応を代表 1 経路について end-to-end に実証した結果。本番品質のソースマップ実装は
> ダウンストリーム別仕様 `pasta-source-map` の担当（R4.1 の残課題申し送り）。

### 実証 RESULT（代表経路で成立）

- **代表経路**: 単純 talk 1 行の `.pasta`（`＊あいさつ` ＋ `　さくら：「こんにちは！」`）。
  `.pasta` 行 2 のトークアクションが、生成 `.lua`（normalize 後）の **行 11**
  （`act.さくら:talk("「こんにちは！」")`）へ着地する。
- **producer→consumer 接合**: code_gen の `SourceMapSink`（task 5.1）が
  `record(out_line=11, span)` を通知し、consumer 側 `SliceSink`（`debug/source_map.rs`）
  が `span.start_byte` から `.pasta` 行 2 を算出して `LineMap`（`lua_line→PastaPos`）を
  構築。マップは **実トランスパイル**（codegen→normalize の実パイプライン）から導出
  であり、数合わせではない。
- **R4.4（.lua→.pasta 変換）**: `resolve_lua_to_pasta(&map, 11)` が
  `PastaPos { file: "slice.pasta", line: 2 }` を返すことを assert。さらに `LineMap` から
  組んだ `.pasta` `SourceResolver`（task 5.2 の DAP source 取り扱い口）を `DapAdapter`
  へ挿すと、停止フレームの `stackTrace` 提示が `.pasta` パス＋`.pasta` 行（2）になること
  を実証。
- **R4.5（.pasta 行 BP ヒット）**: `.pasta` 行 2 の BP を逆引き
  （`LineMap::lua_lines_for_pasta(2) == [11]`）で生成 `.lua` 行 11 へ翻訳し、**実
  `DebugSession`** で生成 `.lua` を走らせると（スタブ `pasta`/`act` で `__start__` 起動）、
  フックが行 11 で **実際に停止**（reason=Breakpoint・停止行=11）することを実コードで
  実証。モック停止ではない。
- **R4.6（無効時ゼロコスト）**: `debug/source_map.rs` 全体と 4 つのスライステストは
  `#[cfg(feature = "pasta-source-map-slice")]` 配下。default ビルド／`cargo test
  -p pasta_lua`（273 lib テスト）は本スライス追加前と同一挙動（feature ON では +4＝277）。
  OFF/ON いずれの構成でも新規警告ゼロ。

### normalize_output 行ズレの扱い（本スライスの前提）

- `out_line` は `normalize_output` **適用前**のバッファ行を数える。`normalize_output` が
  削除するのは「`end` 直前の空行」と「末尾空白」のみで、これらはトーク行 **より後ろ**に
  しか現れない。したがって代表経路ではトークの生成行は normalize 前後で **不変**
  （`out_line` == ランタイム実行行）。本スライスは「normalize 行ズレが無い代表経路を
  選ぶ」option を採り（design の選択肢 b）、生成 `.lua` を実フックで走らせ行 11 で発火する
  ことで前提を検証した。実証コードはこの不変性に依存することを doc 明記。

### `pasta-source-map`（ダウンストリーム別仕様）へ申し送る残課題

1. **全 `generate_*` 網羅**: 本シーム/スライスは代表 1 経路（`generate_action` の talk）
   のみ。var 代入・関数呼び出し・選択肢・単語定義・シーン/関数ヘッダ・コードブロック等、
   全構文ノードの span を各 `writeln` 記録点へ引き回す必要がある。
2. **`normalize_output` 行ズレ補正の一般化**: `end` 直前空行削除などで行がズレる経路を
   含む全網羅では、`out_line`→最終 `.lua` 行の補正（normalize の行削除を再現する差分
   計算、または normalize 後にマップを再キー化）が必須。本スライスはズレの無い経路に
   限定して回避している。
3. **`currentline` 端ケース**: 1 ソース行に複数バイトコードがあるとフックが同一行で複数
   回発火する（本スライスのトーク行も 11 を複数回発火）。BP 多重ヒットの扱い・行頭/行末
   命令境界・末尾呼び出し最適化（TCO）等の `current_line()` 端ケースの確定が必要。
4. **双方向マッピングの正確性**: 1 つの `.pasta` 行が複数 `.lua` 行へ展開（および逆）する
   一般ケースの双方向変換。本スライスは 1:1 の代表経路のみで `lua_lines_for_pasta` は
   `Vec` を返す構造にしてあるが、本番は多対多の選択規則（BP 解決の代表行・スタック提示の
   代表行）の確定が必要。
5. **本番マップ出力と `.pasta` 座標常時提示**: feature gate ではなくデフォルト経路での
   マップ生成・永続化、DAP の `setBreakpoints`/`stackTrace` を `.pasta` 座標で常時駆動
   する結線（本スライスの `SourceResolver` 接続はデモ範囲）。

### 分割境界 / 申し送り（→ `pasta-source-map`）｜task 8.3 統合 E2E で確定

> R4.1 の結論記録として、本仕様（`pasta-vscode-lua-debug`）が R4 で **確定して引き渡す
> もの**と、ダウンストリーム別仕様 `pasta-source-map` が **所有して本番化するもの**の
> 分界点を明文化する。本節は上の残課題 1〜5 の上位サマリ（重複ではなく統合）であり、
> roadmap.md Phase 5「pasta-source-map（仮称）」エントリと整合する。

#### 統合 E2E による実現可能性 CONFIRMED（本仕様の最終証跡）

task 8.3 で、代表 `.pasta` 経路を **1 本のゲート付きテスト**
（`debug::source_map::tests::slice_e2e_pasta_breakpoint_hits_and_reports_pasta_line`・
feature `pasta-source-map-slice`）に統合し、以下を端から端まで連結して実証した:

1. `SliceSink` 装着で代表 `.pasta` を実トランスパイル（codegen→normalize）→
   `record(out_line, span)` から `LineMap` を **実記録由来で導出**（ハードコードでない）。
2. `.pasta` 行 2 の BP を `lua_lines_for_pasta` で生成 `.lua` 行 11 へ翻訳。
3. 翻訳先 `.lua` 行に BP を張り、生成 `.lua` を **実 `DebugSession`** で走らせ、フックが
   行 11 で **実際に停止**（reason=Breakpoint・停止行=11）（R4.5）。
4. その停止フレームを LineMap-backed `.pasta` `SourceResolver`（task 5.2 の DAP source
   口）付き `DapAdapter` で encode し、応答が `.pasta` パス `slice.pasta` ＋ `.pasta` 行
   `2` を報告（R4.4）。報告行は「実停止 `.lua` 行 → 実記録 `LineMap`」の逆写像であり、
   ステージ 3 と同一の記録に根ざす（往復の整合）。

→ 代表経路で `.pasta`↔`.lua` の **双方向解決が可能**であること（R4.1）を、モックでない
実コードの 1 フローで確定した。残る一般化（下記 (B)）は別仕様の本番化作業であり、実現
可能性そのものはここで決着している。

#### (A) 本仕様が確定して引き渡すもの（DELIVERED）

- **実現可能性 CONFIRMED**: 上記統合 E2E で `.pasta`↔`.lua` が代表経路で解決可能と確定
  （R4.1・結論を本 research に記録）。
- **code_gen producer シーム**: `code_gen::source_map::{SourceMapSink, PastaPos}` ＋
  `LuaCodeGenerator` の `out_line` カウンタ／`set_source_map`。本番 transpile は sink
  `None` でバイト一致・ゼロコスト（R4.2/R4.6）。下流はこのシームに全 `generate_*` の
  span を流し込むだけで本番マップを構築できる（IF は確定済み）。
- **DAP presentation source シーム**: `dap::{SourceResolver, ResolvedSource}` ＋
  `DapAdapter::set_source_resolver`。既定は生成 `.lua` 提示（R4.3）。下流は `.pasta`
  resolver を挿すだけで `stackTrace`/`source` を `.pasta` 提示へ切替できる（口は確定済み）。
- **consumer 側薄スライス（feature `pasta-source-map-slice`・default OFF）**:
  `debug::source_map::{LineMap, SliceSink, resolve_lua_to_pasta}` と統合 E2E。
  `.pasta`↔`.lua` が resolvable であることの実証物。**本番経路ではない**（OFF で未コンパイル
  ・ゼロコスト R4.6）。

#### (B) ダウンストリーム `pasta-source-map` が所有するもの（OWNED BY DOWNSTREAM）

- **全 `generate_*` 網羅**（残課題 1）: 代表 talk 以外の全構文ノードの span を各記録点へ
  引き回す本番マッピング。
- **`normalize_output` 行ズレ補正の一般化**（残課題 2）: `out_line`→最終 `.lua` 行の差分
  補正（本スライスはズレの無い代表経路に限定して回避）。
- **`currentline`／1 行複数バイトコード端ケース**（残課題 3）: 同一行複数発火・命令境界・
  TCO 等の確定。
- **双方向マッピングの本番規則**（残課題 4）: 多対多展開時の BP 解決代表行・スタック提示
  代表行の選択規則。
- **本番マップ出力と `.pasta` 座標常時提示**（残課題 5）: feature gate ではなくデフォルト
  経路でのマップ生成・永続化、`setBreakpoints`/`stackTrace` の `.pasta` 座標常時駆動
  （本スライスの `SourceResolver` 接続はデモ範囲）。

→ (B) は本仕様が引き渡したシーム（(A) の producer/ presentation 二口）と薄スライスの
実証物を入力として消費する。分界点は「シーム＋代表 1 経路の実証（本仕様）」と「全網羅の
本番マップ・常時提示の本番化（別仕様）」の境界に置く。これは roadmap.md Phase 5 の
`pasta-source-map（仮称）` エントリ（依存: pasta-vscode-lua-debug）と一致する。
