//! Task 5.2 / 5.4 — 初期モード解決・実行時上書き（requirement 4）と無回帰（requirement 6）の
//! 検証クラスタ。共有 DAP/セッションハーネスは `runtime_toggle_e2e_common` に外出し済み
//! （C2 クラスタ分割）。本ファイルはテスト本体と本クラスタ固有ヘルパーのみを保持する。

use std::path::{Path, PathBuf};

use serde_json::json;

use pasta_lua::{PastaLoader, RuntimeConfig};

use crate::runtime_toggle_e2e_common::*;

// ============================================================================
// Task 5.2 — 初期モード解決と実行時上書きの検証（requirement 4: 4.1 / 4.2 / 4.3 / 4.4）
//
// 5.1 が「停止中トグルの基本往復」を実 DAP-over-TCP で証明済み。本タスクはそこへ
// **初期モード解決と実行時トグルによる上書きの整合**（requirement 4）を加える:
//   - 4.1: attach 引数 `sourcePresentation: "lua"` が初期提示モードとして適用される
//          ことを (a) attach 時 `pasta/sourcePresentation` push イベント = `lua`、
//          (b) **最初の停止**の stackTrace が既に `.lua` 座標、で検証する。
//   - 4.2/4.3: 明示初期モードから実行時トグルで他方へ切り替えると、上書き後モードが
//          採用され以後の提示に持続することを検証する。
//   - 4.4: attach 引数 `sourcePresentation` 未指定時、初期モードは既存解決
//          （env `PASTA_DEBUG_SOURCE_MODE` > `pasta.toml` `present_as` > 既定 `.pasta`）。
//          file 階層（`present_as = "lua"`）と default 階層（設定なし → `.pasta`）を
//          attach push イベント + 最初の停止 stackTrace で検証し、さらに解決済み初期
//          モードを実行時トグルで上書きできることを検証する。
//
// env 階層 / 優先順位（env>file>default, attach>env）は `DebugConfig::resolve` の既存
// ユニットテストが網羅済み（`src/debug/mod.rs` の `source_mode_file_overrides_default`,
// `source_mode_env_overrides_file`, `source_mode_attach_overrides_env`,
// `default_source_mode_is_pasta_and_sidecar_false`）。env のプロセスグローバル変更は
// 並行 cargo test でレースするため、本 E2E では env を変更せず上記ユニットテストを参照
// するに留める（CLAUDE 指示・タスク境界）。
// ============================================================================

#[test]
fn attach_initial_lua_is_applied_then_runtime_toggle_overrides_to_pasta() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    // attach 引数が最優先（attach > env > file > 既定）なので file 階層は既定（`.pasta`）のまま、
    // attach の "lua" が初期モードを決める。
    let coords = resolve_session(temp.path(), None);
    let mut session = start_stopped_session(&coords, Some("lua"), "lua");

    // (4.1) 最初の停止は既に `.lua` 座標（attach 初期モードが適用済み・トグル前）。
    assert_lua_frame(
        &mut session,
        &coords,
        10,
        "4.1: attach 初期 `.lua` での最初の停止",
    );

    // (4.2) 実行時トグルで初期モード `.lua` を `.pasta` へ上書き。
    toggle_mode(&mut session, 20, "pasta");
    // (4.3) 上書き後モードが以後の提示で採用される。
    assert_pasta_frame(
        &mut session,
        &coords,
        21,
        "4.2/4.3: トグルで初期 `.lua` を `.pasta` へ上書き",
    );
    // (4.3 持続) 同一停止での再読でも上書き後モードが持続する（再トグルしていない）。
    assert_pasta_frame(
        &mut session,
        &coords,
        22,
        "4.3: 上書き後モードが後続の読みでも持続",
    );

    finish_session(session);
}

/// 4.4（file 階層）+ 4.2/4.3: attach 引数 `sourcePresentation` 未指定で `pasta.toml`
/// `[debug] present_as = "lua"` のとき初期モードは file 階層解決 = `.lua`。最初の停止が
/// `.lua` 座標であること、続いて実行時トグルで解決済み初期モード（`.lua`）を `.pasta` へ
/// 上書きできることを検証する。env 階層・優先順位は `DebugConfig::resolve` の既存ユニット
/// テスト（`source_mode_env_overrides_file` 他）が網羅済みのため重複しない。
#[test]
fn no_attach_arg_file_present_as_lua_resolves_initial_lua_then_toggle_overrides() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let coords = resolve_session(temp.path(), Some("lua"));
    // attach 引数なし → 初期解決 = file 階層 `lua`。host 側 `debug_source_mode()` でも検証される。
    let mut session = start_stopped_session(&coords, None, "lua");

    // (4.4 file 階層) 最初の停止が `.lua` 座標（file `present_as="lua"` が初期モードを決めた）。
    assert_lua_frame(
        &mut session,
        &coords,
        10,
        "4.4 file: present_as=\"lua\" 初期 `.lua` の最初の停止",
    );

    // (4.2/4.3) 解決済み初期モードを実行時トグルで `.pasta` へ上書きでき、以後持続する。
    toggle_mode(&mut session, 20, "pasta");
    assert_pasta_frame(
        &mut session,
        &coords,
        21,
        "4.4/4.2/4.3: 解決済み `.lua` をトグルで `.pasta` へ上書き",
    );
    assert_pasta_frame(
        &mut session,
        &coords,
        22,
        "4.3: 上書き後モードが後続の読みでも持続",
    );

    finish_session(session);
}

/// 4.4（default 階層）+ 4.2/4.3: attach 引数なし・`present_as` 設定なしのとき初期モードは
/// 既定 = `.pasta`。最初の停止が `.pasta` 座標であること、続いて実行時トグルで `.lua` へ
/// 上書きできることを検証する。
#[test]
fn no_attach_arg_no_config_resolves_initial_pasta_then_toggle_overrides() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let coords = resolve_session(temp.path(), None);
    let mut session = start_stopped_session(&coords, None, "pasta");

    // (4.4 default 階層) 最初の停止が `.pasta` 座標（既定 `.pasta` が初期モード）。
    assert_pasta_frame(
        &mut session,
        &coords,
        10,
        "4.4 default: 設定なし初期 `.pasta` の最初の停止",
    );

    // (4.2/4.3) 解決済み既定 `.pasta` を実行時トグルで `.lua` へ上書きでき、以後持続する。
    toggle_mode(&mut session, 20, "lua");
    assert_lua_frame(
        &mut session,
        &coords,
        21,
        "4.4/4.2/4.3: 解決済み `.pasta` をトグルで `.lua` へ上書き",
    );
    assert_lua_frame(
        &mut session,
        &coords,
        22,
        "4.3: 上書き後モードが後続の読みでも持続",
    );

    finish_session(session);
}

// ============================================================================
// Task 5.4 — 無回帰の検証（requirement 6: 6.1 / 6.2 / 6.3 / 6.4）
//
// 本タスクは「実行時トグルの追加が既存挙動を壊さない」ことを **無回帰の観点で確定**する。
// 6.x の多くは既存テストで実証済みのため、本ファイルでの新規付加価値は **6.1（トグル未使用
// セッションは初期解決どおりに動作し続ける）** の集中 E2E と、**6.2（OFF 経路ではトグル機構が
// 一切実体化しない）** の集中アサーションに限定する。6.3 / 6.4 は既存の通過テストを参照し、
// 検証コマンドで再実行して退行が無いことを実証する（重複させない）。
//
// 参照する既存カバレッジ（重複させない）:
//   - 6.2（OFF ゼロコスト・バイト不変）: `src/debug/mod.rs` の
//     `enable_disabled_returns_none_and_no_trace`（`if !cfg.enabled { return Ok(None) }`
//     ゲート＝フック/ポート/`std_debug` 非露出）、`tests/runtime/debug_integration_test.rs`
//     `zero_cost_sandbox_regression::{r5_2_disabled_installs_no_hook_jit_stays_on,
//     r5_3_disabled_keeps_sandbox_debug_is_nil, r5_5_disabled_opens_no_port}`、
//     `tests/transpiler/zero_cost_regression_test.rs`
//     （`test_zero_cost_off_path_all_syntax_byte_invariant`）、
//     `tests/runtime/source_map_handoff_test.rs::disabled_debug_runtime_holds_no_source_map`。
//     本ファイルの 6.2 アサーションは **トグル機構固有**の角度 —— 提示モードセル
//     （`debug_source_mode()` が公開する `SharedSourceMode` の baked 値）が OFF では
//     そもそも存在しない（`None`）—— を 1 点だけ追加し、既存ゼロコストテストを複製しない。
//   - 6.3（`.pasta` BP は切替後も有効）: task 5.1 の
//     `pasta_breakpoint_toggle_lua_then_pasta_over_tcp`（切替の前後で同一 `.pasta` 行 BP に
//     再停止）が実証済み。本ファイルでは重複させない。
//   - 6.4（既存 attach 接続・診断・ハイライト等を損なわない）: backend 側は
//     `src/debug/dap.rs` の attach アーム・`tests/runtime/debug_integration_test.rs` の
//     attach/DAP 統合テスト、frontend 側は VSCode unit suite（WasmBridge / Integration /
//     DebugAdapterFactory / sourcePresentationToggle）が実証済み。検証コマンドで再実行する。
// ============================================================================

/// base_dir 配下にフィクスチャ `.pasta` と **`[debug] enabled = false`** の pasta.toml を配置し、
/// pasta_scripts / scriptlibs をコピーする（`make_base_dir` の OFF 版）。`.pasta` の絶対パスを返す。
/// env はテストハーネスで一切変更しない（並行 cargo test でのプロセスグローバルレースを避ける）。
fn make_disabled_base_dir(base: &Path) -> PathBuf {
    let pasta_file = base.join("dic/test/debug_toggle_e2e.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, FIXTURE).unwrap();
    std::fs::write(
        base.join("pasta.toml"),
        "\
[loader]
debug_mode = true

[debug]
enabled = false
",
    )
    .unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let src = crate_root.join(sub);
        let dst = base.join(sub);
        if src.exists() {
            std::fs::create_dir_all(&dst).unwrap();
            copy_dir(&src, &dst).unwrap();
        }
    }
    pasta_file
}

/// 6.2（OFF 経路バイト不変・ゼロコスト — **トグル機構固有の角度**）。
///
/// `[debug] enabled = false` のランタイムでは、実行時トグルが操作する提示モードセル
/// （`SharedSourceMode`）が **そもそも実体化しない**。`enable()` の OFF ゲート
/// （`src/debug/mod.rs`: `if !cfg.enabled { return Ok(None) }`）が `DebugHandle` を返さない
/// ため、トグル経路（カスタムリクエスト→`SharedSourceMode` 更新→レゾルバ差し替え→再描画）に
/// 到達する足場が一切存在しないことを、ランタイムの可観測シグナルで確定する:
///   - `debug_enabled() == false`（ハンドル不保持 ＝ ブリッジ/アダプタが起動しない）。
///   - `debug_source_mode() == None`（トグルが反転させる提示モードセルが存在しない）。
///   - `debug_local_addr() == None`（カスタムリクエストを受ける接続口が無い）。
///   - `debug_source_map() == None`（モード別提示の対象となるマップが構築されない）。
///
/// これは既存ゼロコストテスト（フック非設置・`std_debug` 非露出・生成 `.lua` バイト不変）を
/// 複製せず、**トグル状態が OFF では到達不能**であることだけを集中的に表明する
/// （design "No-Regression": 「OFF でカスタムリクエスト経路が一切走らないこと（6.2）」）。
#[test]
fn disabled_runtime_has_no_toggle_state_to_run() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.path().to_path_buf();
    let _pasta_file = make_disabled_base_dir(&base);

    let runtime = PastaLoader::load_with_config(&base, RuntimeConfig::new())
        .expect("disabled-debug runtime must build");

    assert!(
        !runtime.debug_enabled(),
        "6.2: OFF ランタイムは DebugHandle を保持しない（トグルブリッジ/アダプタ非起動）"
    );
    assert_eq!(
        runtime.debug_source_mode(),
        None,
        "6.2: OFF では実行時トグルが反転させる提示モードセル（SharedSourceMode）が実体化しない"
    );
    assert_eq!(
        runtime.debug_local_addr(),
        None,
        "6.2: OFF では `pasta/sourcePresentation` カスタムリクエストを受ける接続口を開かない"
    );
    assert!(
        runtime.debug_source_map().is_none(),
        "6.2: OFF ではモード別提示の対象マップを構築しない（ゼロコスト）"
    );
}

/// 6.1（トグル未使用セッションは初期解決どおりに動作し続ける）。
///
/// `pasta/sourcePresentation` を **一度も送らない** デバッグセッションが、初期解決モード
/// （attach 引数なし・`present_as` 設定なし → 既定 `.pasta`）のまま、停止・`stackTrace`・
/// `next` の一連の操作を通じて **提示が一切ドリフトしない** ことを実 DAP-over-TCP で検証する。
/// 5.1/5.2/5.3 の既存 E2E はいずれも途中で必ずトグルするため、「トグル未使用で初期モードが
/// 安定持続する」経路は本テスト固有の付加価値である（design "No-Regression": 「トグル未使用
/// セッションが初期解決どおり動作（6.1）」）。
///
/// 観測可能な「done」:
///   1. attach（`sourcePresentation` 未指定）→ push イベントが初期解決モード `pasta` を報告。
///   2. `.pasta` 行 BP で停止し、停止直後の `stackTrace` トップフレームが `.pasta` 座標。
///   3. **トグルせず** BP を解除して `next` を 1 回送り、別の `.pasta` 行で停止 —— トップフレーム
///      は依然 `.pasta` 座標（提示はドリフトしない）。
///   4. 同一停止で `stackTrace` を再読しても提示は `.pasta` のまま（追加操作でモードが揺れない）。
#[test]
fn no_toggle_session_stays_in_initial_pasta_mode_throughout() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let coords = resolve_session(temp.path(), None);
    // attach 引数なし・`present_as` 設定なし → 初期解決 = 既定 `.pasta`。
    // start_stopped_session は host 側でも `debug_source_mode() == Some(Pasta)` を表明する。
    let mut session = start_stopped_session(&coords, None, "pasta");

    // (6.1-a) 初期解決どおりの最初の停止: トップフレームは `.pasta` 座標。
    assert_pasta_frame(
        &mut session,
        &coords,
        10,
        "6.1: トグル未使用・初期 `.pasta` の最初の停止",
    );
    // (6.1-b) 同一停止での再読でも `.pasta` のまま（追加 stackTrace で揺れない）。
    assert_pasta_frame(
        &mut session,
        &coords,
        11,
        "6.1: トグル未使用・再読でも初期 `.pasta` が持続",
    );

    // (6.1-c) **トグルせず** BP を解除して `next` を 1 回。停止後も `.pasta` 提示のまま
    // （別の `.pasta` 行へ進むが、提示モードはドリフトしない）。BP 解除は同一行 line-hook 再入での
    // BP 再発火を避けるためで、提示モードには影響しない。
    session.client.send_request(
        12,
        "setBreakpoints",
        json!({ "source": { "path": coords.pasta_file_key.clone() }, "breakpoints": [] }),
    );
    let _ = session
        .client
        .recv_until(|m| is_response(m, "setBreakpoints"));

    session
        .client
        .send_request(13, "next", json!({ "threadId": session.thread_id }));
    let _ = session.client.recv_until(|m| is_response(m, "next"));
    let stepped = session.client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stepped["body"]["reason"], "step",
        "6.1: トグル未使用の `next` は reason step で再停止する"
    );

    session
        .client
        .send_request(14, "stackTrace", json!({ "threadId": session.thread_id }));
    let stack = session.client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"]
        .as_array()
        .expect("stackFrames");
    assert!(
        !frames.is_empty(),
        "6.1: ステップ後も停止フレームが存在する"
    );
    assert_pasta_source(
        &frames[0],
        &coords.pasta_file_key,
        "6.1: トグル未使用・ステップ後もトップフレームは `.pasta` 提示（ドリフトなし）",
    );

    finish_session(session);
}
