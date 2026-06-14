//! Task 5.1 (spec: pasta-debug-lua-view-toggle) — 実 DAP-over-TCP の往復 E2E。
//!
//! 「`.pasta` 行に張ったブレークポイントで停止した状態から提示モードを実行時トグルで
//! `.pasta` ⇔ `.lua` へ切り替える」ワークフローを、実 TCP ソケット越しの DAP セッションで
//! End-to-End 検証する（requirements.md Requirement 7: 7.1 / 7.2 / 7.3、および 3.3 即時再描画 /
//! 6.3 ブレークポイント維持）。
//!
//! # 検証する観測可能な「done」（design.md "E2E Tests" / System Flows「停止中トグル」）
//!
//! 1. **7.1 / 3.3**: `.pasta` 行 BP で停止（`.pasta` 提示）した状態から
//!    `pasta/sourcePresentation{ mode: "lua" }` カスタムリクエストを送ると、
//!    (a) `lua` をエコーする受理レスポンス、(b) `{ mode: "lua" }` のカスタムイベント、
//!    (c) 再描画のための `stopped` 再送、が返り、続く `stackTrace` がトップフレームを
//!    生成 `.lua` 座標（path/line）で提示する（新レゾルバ下での即時再描画）。
//! 2. **7.2**: `{ mode: "pasta" }` で戻すと、続く `stackTrace` が `.pasta` 座標へ戻る。
//! 3. **7.3 / 6.3**: `.pasta` 行 BP が切替の前後で有効であり続ける —— continue 後、同じ
//!    `.pasta` 行 BP で再び停止することを確認する（BP は提示モード非依存で維持される）。
//!
//! # 構成（既存 `debug_integration_test.rs` の TCP/DAP ハーネスを踏襲）
//!
//! 実 `.pasta`↔`.lua` 双方向マップを伴う実セッションが必要なため、`PastaLoader::load_with_config`
//! で `[debug] enabled = true, port = 0` のランタイムを構築する（OS 割当ポートを `debug_local_addr()`
//! で取得）。これによりランタイムは loader が transpile 後に構築した集約 `Arc<SourceMap>` を保持し、
//! 既定提示モード `.pasta` でデバッグバックエンドを enable する（`source_map_handoff_test.rs` と同経路）。
//! 停止対象は同一フィクスチャを `LuaTranspiler` で transpile した生成 `.lua` を、ソースマップの
//! チャンクキー（`CacheManager::source_to_cache_path`）と一致する chunk 名で `exec_named` する。
//! これによりフック報告 chunk とマップ chunk が一致し、`.pasta`/`.lua` 提示が実際に異なる
//! （フックは生成 `.lua` 行を報告し、レゾルバが提示モードに応じて `.pasta`/`.lua` を提示する）。
//!
//! `mlua::Lua` は `!Send` なので VM はランタイム所有スレッドに固定する。DAP クライアントは別スレッドで
//! 駆動し、チャネル/バウンド addr のみ越境する。全クライアント待機は TEST-ONLY watchdog でバウンドする。

use std::net::SocketAddr;
use std::sync::mpsc::{self, RecvTimeoutError};

use serde_json::json;

use pasta_lua::debug::source_map::canonicalize_chunk_name;
use pasta_lua::loader::CacheManager;
use pasta_lua::{PastaLoader, RuntimeConfig, SourceMode};

use crate::runtime_toggle_e2e_common::*;
/// 実 DAP-over-TCP の往復 E2E（7.1 / 7.2 / 7.3 / 3.3 / 6.3）を 1 セッションで検証する。
#[test]
fn pasta_breakpoint_toggle_lua_then_pasta_over_tcp() {
    // --- 事前計算（メインスレッド側で温度確認できる純データ）。 ---
    // ランタイム構築前に「同一フィクスチャの BP `.pasta` 行 → 生成 `.lua` 実行座標」を解決し、
    // 期待する `.lua` 提示座標（chunk/line）と `.pasta` 提示座標（file/line）を確定する。
    // これらの座標は exec する chunk 名・行番号と一致しなければならない（ハードコードしない）。
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.path().to_path_buf();
    let pasta_file = make_base_dir(&base);

    let cache_manager = CacheManager::new(base.clone(), "profile/pasta/cache/lua");
    let chunk = cache_manager
        .source_to_cache_path(&pasta_file)
        .to_string_lossy()
        .to_string();
    let pasta_file_key = pasta_file.to_string_lossy().to_string();

    // ローダと同一経路で構築した集約マップから BP `.pasta` 行の `.lua` 実行座標を解決する。
    let expect_map = PastaLoader::build_source_map(std::slice::from_ref(&pasta_file), &cache_manager, false);
    let bp_lua_coords = expect_map.resolve_pasta_to_lua(&pasta_file_key, BP_PASTA_LINE);
    assert_eq!(
        bp_lua_coords.len(),
        1,
        "fixture invariant: BP `.pasta` 行 {BP_PASTA_LINE} は単一の `.lua` 実行座標へ一意対応する \
         (top-level に実行される行), got {bp_lua_coords:?}"
    );
    let (bp_chunk, bp_lua_line) = bp_lua_coords[0].clone();
    assert_eq!(
        canonicalize_chunk_name(&bp_chunk),
        canonicalize_chunk_name(&chunk),
        "BP の `.lua` 実行座標は当該チャンクを指す"
    );

    // 生成 `.lua`。`exec_named(generated, chunk)` でフックがこの chunk を報告し、
    // 停止行 = `bp_lua_line` になる。
    let generated_lua = transpile_fixture(&pasta_file);

    // --- スレッド間チャネル: host → main は bound addr、main → host は go/再go 信号。 ---
    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (go_tx, go_rx) = mpsc::channel::<()>();

    let chunk_for_host = chunk.clone();
    let base_for_host = base.clone();
    let generated_for_host = generated_lua.clone();

    // VM HOST スレッド: ランタイムを構築（map 保持 + debug enable, port 0）、bound addr を発行、
    // クライアントの go を待ってから BP 対象 chunk を 2 回 exec する（7.3 の再ヒット検証）。
    let host = std::thread::spawn(move || -> Result<(), String> {
        let runtime = PastaLoader::load_with_config(&base_for_host, RuntimeConfig::new())
            .map_err(|e| format!("loader must build an enabled-debug runtime: {e}"))?;

        // ランタイムは集約マップを保持し、既定提示モード `.pasta` で enable されている。
        if !runtime.debug_enabled() {
            return Err("enabled [debug] must install the backend".to_string());
        }
        if runtime.debug_source_map().is_none() {
            return Err("enabled debug runtime must hold the aggregated source map".to_string());
        }
        match runtime.debug_source_mode() {
            Some(SourceMode::Pasta) => {}
            other => {
                return Err(format!(
                    "initial resolved mode must default to `.pasta` (env override not set in CI): {other:?}"
                ));
            }
        }

        let addr = runtime
            .debug_local_addr()
            .ok_or_else(|| "enabled runtime must expose a bound debug addr (port 0)".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        // (#1) クライアントが setBreakpoints/configurationDone を終えるまで待つ。
        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "no go signal before exec #1".to_string())?;
        // exec #1: BP ヒット → クライアントがトグル検証後に continue するまでブロック。
        runtime
            .exec_named(&generated_for_host, &chunk_for_host)
            .map_err(|e| format!("exec #1 failed: {e}"))?;

        // (#2) 再 exec の go を待つ（7.3: BP は切替の前後で維持され、再び停止する）。
        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "no go signal before exec #2".to_string())?;
        runtime
            .exec_named(&generated_for_host, &chunk_for_host)
            .map_err(|e| format!("exec #2 failed: {e}"))?;

        drop(runtime); // teardown
        Ok(())
    });

    // --- CLIENT（このスレッド）: DAP ハンドシェイク + トグル検証を駆動する。 ---
    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

    // initialize ハンドシェイク。
    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    // `.pasta` 行に BP を設定（提示モード `.pasta`）。source.path は `.pasta` ファイル、
    // line は `.pasta` 行。バックエンドが `.lua` 実行座標へ翻訳して登録する（6.3 / 7.3 の前提）。
    client.send_request(
        2,
        "setBreakpoints",
        json!({
            "source": { "path": pasta_file_key },
            "breakpoints": [{ "line": BP_PASTA_LINE }],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let bps = bp_resp["body"]["breakpoints"].as_array().expect("breakpoints array");
    assert_eq!(bps.len(), 1, "exactly one breakpoint resolved");
    assert_eq!(
        bps[0]["verified"], true,
        "7.3/6.3: `.pasta` 行 BP は検証済み（`.lua` 実行座標へ翻訳・登録された）"
    );
    assert_eq!(
        bps[0]["line"], BP_PASTA_LINE,
        "`.pasta` 行 BP は元の `.pasta` 行で報告される"
    );

    client.send_request(3, "configurationDone", json!({}));
    let _ = client.recv_until(|m| is_response(m, "configurationDone"));

    // exec #1 開始。
    go_tx.send(()).expect("go #1");

    // ===== 7.1 / 3.3: `.pasta` BP で停止 → `.lua` 提示へ切替 → `.lua` 座標で再描画 =====

    // (1) BP ヒット（`.pasta` 提示で停止）。
    let stopped1 = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped1["body"]["reason"], "breakpoint",
        "exec #1 は `.pasta` 行 BP で停止する"
    );
    let thread_id = stopped1["body"]["threadId"].as_u64().unwrap_or(1);

    // 停止直後（`.pasta` 提示）の stackTrace: トップフレームは `.pasta` 座標。
    client.send_request(10, "stackTrace", json!({ "threadId": thread_id }));
    let stack_pasta0 = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames0 = stack_pasta0["body"]["stackFrames"].as_array().expect("stackFrames");
    assert!(!frames0.is_empty(), "停止フレームが存在する");
    assert_pasta_source(&frames0[0], &pasta_file_key, "初期 `.pasta` 提示");
    assert_eq!(
        frames0[0]["line"], BP_PASTA_LINE,
        "初期 `.pasta` 提示: トップフレーム行は `.pasta` 行 {BP_PASTA_LINE}"
    );

    // (2) `pasta/sourcePresentation { mode: "lua" }` 送出。
    client.send_request(20, "pasta/sourcePresentation", json!({ "mode": "lua" }));

    // (a) 受理レスポンス: `lua` をエコー（requirement 1.3 / 7.1）。
    let toggle_resp_lua = client.recv_until(|m| is_response(m, "pasta/sourcePresentation"));
    assert_eq!(toggle_resp_lua["request_seq"], 20, "受理レスポンスは要求 seq に対応");
    assert_eq!(
        toggle_resp_lua["body"]["mode"], "lua",
        "7.1: 受理レスポンスは適用後モード `lua` をエコーする"
    );

    // (b) `pasta/sourcePresentation` カスタムイベント `{ mode: "lua" }`（requirement 2.6）。
    let toggle_event_lua =
        client.recv_until(|m| is_event(m, "pasta/sourcePresentation") && m["body"]["mode"] == "lua");
    assert_eq!(
        toggle_event_lua["body"]["mode"], "lua",
        "7.1: 切替後モードのカスタムイベントが送出される"
    );

    // (c) 再描画のための `stopped` 再送（requirement 3.3）。
    let restopped_lua = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        restopped_lua["body"]["reason"], "breakpoint",
        "3.3: 切替後、現停止が再送され再描画が起動する"
    );

    // (d) 切替後の stackTrace: トップフレームが生成 `.lua` 座標（path = chunk, line = bp_lua_line）。
    client.send_request(21, "stackTrace", json!({ "threadId": thread_id }));
    let stack_lua = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames_lua = stack_lua["body"]["stackFrames"].as_array().expect("stackFrames");
    assert!(!frames_lua.is_empty(), "切替後も停止フレームが存在する");
    let top_lua_path = frames_lua[0]["source"]["path"]
        .as_str()
        .expect("`.lua` 提示 source path");
    assert_eq!(
        canonicalize_chunk_name(top_lua_path),
        canonicalize_chunk_name(&chunk),
        "7.1/3.4: `.lua` 提示: トップフレーム source は生成 `.lua` チャンク (got {top_lua_path})"
    );
    assert_eq!(
        frames_lua[0]["line"].as_u64().expect("lua line"),
        bp_lua_line as u64,
        "7.1/3.4: `.lua` 提示: トップフレーム行は生成 `.lua` 実行行 {bp_lua_line}"
    );

    // ===== 7.2: `.lua` → `.pasta` へ戻すと提示が `.pasta` 座標へ戻る =====
    client.send_request(30, "pasta/sourcePresentation", json!({ "mode": "pasta" }));
    let toggle_resp_pasta = client.recv_until(|m| is_response(m, "pasta/sourcePresentation"));
    assert_eq!(
        toggle_resp_pasta["body"]["mode"], "pasta",
        "7.2: 受理レスポンスは適用後モード `pasta` をエコーする"
    );
    let _toggle_event_pasta = client
        .recv_until(|m| is_event(m, "pasta/sourcePresentation") && m["body"]["mode"] == "pasta");
    let _restopped_pasta = client.recv_until(|m| is_event(m, "stopped"));

    client.send_request(31, "stackTrace", json!({ "threadId": thread_id }));
    let stack_pasta = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames_pasta = stack_pasta["body"]["stackFrames"].as_array().expect("stackFrames");
    assert!(!frames_pasta.is_empty(), "戻し後も停止フレームが存在する");
    assert_pasta_source(&frames_pasta[0], &pasta_file_key, "7.2/3.5: `.pasta` 提示へ復帰");
    assert_eq!(
        frames_pasta[0]["line"], BP_PASTA_LINE,
        "7.2/3.5: `.pasta` 提示へ復帰: トップフレーム行は `.pasta` 行 {BP_PASTA_LINE}"
    );

    // ===== 7.3 / 6.3: `.pasta` 行 BP は切替の前後で有効であり続ける =====
    // exec #1 を continue で流し切り、BP を再設定せずに exec #2 で **同じ `.pasta` 行 BP** に
    // 再び停止することを確認する（トグルは BP ストアに影響しない）。
    client.send_request(40, "continue", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "continue"));

    // exec #2 開始（BP を再設定しない）。
    go_tx.send(()).expect("go #2");

    let stopped2 = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped2["body"]["reason"], "breakpoint",
        "7.3/6.3: トグルの前後で `.pasta` 行 BP は有効であり続け、再 exec で同じ BP に再停止する"
    );

    // 再停止時の提示は（直前に `.pasta` へ戻したため）`.pasta` 座標である。
    client.send_request(41, "stackTrace", json!({ "threadId": thread_id }));
    let stack_after = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames_after = stack_after["body"]["stackFrames"].as_array().expect("stackFrames");
    assert_pasta_source(&frames_after[0], &pasta_file_key, "7.3: 再停止フレームも `.pasta` 提示");
    assert_eq!(
        frames_after[0]["line"], BP_PASTA_LINE,
        "7.3: 再停止は同じ `.pasta` 行 {BP_PASTA_LINE}"
    );

    // continue で exec #2 を流し切り、host を teardown まで到達させる。
    client.send_request(50, "continue", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "continue"));

    // host スレッドが watchdog 内で完了することを確認（ハングしない）。
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(host.join());
    });
    match done_rx.recv_timeout(WATCHDOG) {
        Ok(joined) => {
            joined
                .expect("host thread must not panic")
                .expect("both execs must run to completion with the persisted `.pasta` BP");
        }
        Err(RecvTimeoutError::Timeout) => panic!("host thread did not finish (hang?)"),
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }
}
