//! Task 5.3 — 提示モードとステップ粒度整合の検証（requirement 5）クラスタ。
//! 共有 DAP/フィクスチャ・ハーネスは `runtime_toggle_e2e_common` に外出し済み
//! （C2 クラスタ分割）。本ファイルはステップ粒度シナリオ固有のヘルパーとテストのみを保持する。

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};

use serde_json::json;

use pasta_dsl::parser::parse_str;
use pasta_lua::debug::source_map::canonicalize_chunk_name;
use pasta_lua::loader::CacheManager;
use pasta_lua::{LuaTranspiler, PastaLoader, RuntimeConfig};

use crate::runtime_toggle_e2e_common::*;

// ============================================================================
// Task 5.3 — 提示モードとステップ粒度整合の検証（requirement 5: 5.1 / 5.2 / 5.3 / 5.4）
//
// 5.1/5.2 のステップ往復・初期モード解決を踏まえ、本タスクは **ステップ粒度が提示モードに
// 追従する**こと（requirement 5）を検証する。粒度ロジック自体は実装済みで、純粋な停止判定核
// `pasta_step_should_stop` と毎行の `effective_mode()` 読取は `src/debug/session.rs` の
// ユニットテスト、および `src/debug/wiring.rs` の実 DAP-over-TCP E2E（E1–E8）が網羅済み。
// 本 E2E の付加価値は **停止中トグル（5.3）** —— 停止状態で `pasta/sourcePresentation` を
// 送って提示モードを反転させると、**次の** ステップ操作が新しい粒度になる —— を、実ローダ経由
// （`PastaLoader::load_with_config`）の実 DAP-over-TCP で証明する点にある。
//
// そのために、1つの `.pasta` トーク行が複数の `.lua` トーク呼び出しへ展開される
// フィクスチャ（`debug_toggle_step_e2e.pasta`）を用いる。展開された `.lua` 行を**実行**する
// ため、生成 `.lua` で scene を定義した後、scene エントリポイント `__start__` を最小 act で
// 駆動する（talk 本体行 = 複数 `.lua` 行が実行され、フックが各行を報告する）。
//
// シナリオ（観測可能な「done」）:
//   1. 単一 `.lua` 行へ対応する 1 本目のトーク行（origin `.pasta` 行）に BP を張って停止する。
//   2. BP を解除（同一 `.lua` 行への line-hook 再入で BP が再発火するのを避ける）し、`next` で
//      多対1トーク行（複数 `.lua` 行へ展開された `.pasta` 行）の先頭 `.lua` 行へ進める。ここまでは
//      両モードで同一の停止位置（origin の次の異なる `.pasta` 行 = 多対1行）になる。
//   3. **多対1行で停止した状態のまま提示モードを反転（5.3）**。
//   4. 次の `next` で粒度差を観測する:
//      - `.pasta` 粒度（5.1）: 同一 `.pasta` 行の残り `.lua` 行を消化し、次の異なる `.pasta` 行で停止。
//      - `.lua` 粒度（5.2）: 同一 `.pasta` 行内の次の `.lua` 行で停止（消化しない）。
//      停止中トグルが **次の** `next` の粒度を反転させる（`effective_mode()` 毎行読取が切替を拾う）。
//
// 5.4（コルーチン跨ぎでの粒度継続）は、ステップキー `(thread, base_depth)` が yield/resume を
// またいで生存することに依存し、`src/debug/session.rs` の
// `step_over_survives_coroutine_yield_and_skips_other_threads`（採択B 生存・thread 不一致
// スキップ）および `src/debug/wiring.rs` の E7（`.pasta` 粒度でのコルーチン step over が
// yield をまたいで resume 後の `.pasta` 行で停止）が実証済み。フル「コルーチン跨ぎ + 停止中
// トグル」E2E は本ハーネスの scene 駆動経路では実用的でないため複製せず、上記の既存カバレッジ
// に委譲する（下の CONCERNS と Status Report を参照）。
// ============================================================================

/// ステップ粒度フィクスチャ（1つの `.pasta` トーク行が複数 `.lua` 行へ展開される）。
const STEP_FIXTURE: &str = include_str!("../fixtures/debug_toggle_step_e2e.pasta");

/// base_dir 配下にステップ用フィクスチャ `.pasta` と `[debug] enabled, port=0` の pasta.toml を
/// 配置し、pasta_scripts / scriptlibs をコピーする（`make_base_dir` と同構成・別フィクスチャ）。
/// `.pasta` の絶対パスを返す。
fn make_step_base_dir(base: &Path) -> PathBuf {
    let pasta_file = base.join("dic/test/debug_toggle_step_e2e.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, STEP_FIXTURE).unwrap();
    std::fs::write(
        base.join("pasta.toml"),
        "\
[loader]
debug_mode = true

[debug]
enabled = true
port = 0
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

/// ステップ用フィクスチャを本番ローダと同じ（sink 無し）トランスパイルして生成 `.lua` を返す。
fn transpile_step_fixture(file: &Path) -> String {
    let parsed = parse_str(STEP_FIXTURE, &file.to_string_lossy()).expect("step fixture must parse");
    let transpiler = LuaTranspiler::default();
    let mut out = Vec::new();
    transpiler
        .transpile(&parsed, &mut out)
        .expect("step fixture must transpile");
    String::from_utf8(out).expect("generated lua is valid utf-8")
}

/// `SCENE.get_start("あいさつ1")` を最小 act で駆動し、talk 本体（複数 `.lua` 行）を実行させる
/// ドライバ。生成 `.lua` を `exec_named` で**定義**した後にこれを `exec` する（BP は定義実行では
/// 張られておらず、本ドライバ実行中の talk 本体行ではじめてヒットする）。`さくら` アクターは
/// `talk` がトークンへ格納するだけなので任意の非 nil 値でよい。
const STEP_DRIVER: &str = r#"
    local SCENE = require("pasta.scene")
    local ACT = require("pasta.act")
    local start = SCENE.get_start("あいさつ1")
    if not start then error("scene entrypoint must be registered") end
    local act = ACT.new({ ["さくら"] = { name = "さくら" } })
    start(act)
    return #act.token
"#;

/// ステップ粒度シナリオの導出済み座標（map から動的解決・ハードコード回避）。
struct StepCoords {
    base: PathBuf,
    chunk: String,
    pasta_file_key: String,
    generated_lua: String,
    /// origin（単一 `.lua` 本体行へ対応する `.pasta` トーク行）と、その本体 `.lua` 実行行。
    origin_pasta_line: u32,
    origin_lua_line: u32,
    /// 多対1の `.pasta` 行（≥2 本の本体 `.lua` 行へ展開・トグル後の粒度差を観測する行）。
    multi_pasta_line: u32,
    /// その本体 `.lua` 実行行（昇順・≥2 本）。`[0]` が `next` で最初に止まる先頭行、
    /// `[1]` が同一 `.pasta` 行の 2 本目（`.lua` 粒度 next の停止先）。
    multi_lua_lines: Vec<u32>,
    /// 多対1行の次の異なる `.pasta` 行（`.pasta` 粒度 next の停止先）。
    next_pasta_line: u32,
    /// その本体 `.lua` 実行行。
    next_lua_line: u32,
}

/// origin（単一本体 `.lua` 行）→ 多対1行（≥2 本の本体 `.lua` 行）→ 次の異なる `.pasta` 行、の
/// 3 段を map から導出する。本体 `.lua` 行（= `function SCENE.__start__` ヘッダより後の行）のみ
/// が driver 実行で停止対象になるため、ヘッダ行を除いた本体側で関係を判定する。フィクスチャ
/// 不変条件は build 時に表明する。
fn resolve_step_session(base: &Path) -> StepCoords {
    let pasta_file = make_step_base_dir(base);
    let cache_manager = CacheManager::new(base.to_path_buf(), "profile/pasta/cache/lua");
    let chunk = cache_manager
        .source_to_cache_path(&pasta_file)
        .to_string_lossy()
        .to_string();
    let pasta_file_key = pasta_file.to_string_lossy().to_string();

    let map =
        PastaLoader::build_source_map(std::slice::from_ref(&pasta_file), &cache_manager, false);
    let generated_lua = transpile_step_fixture(&pasta_file);
    let header_lua_line = generated_lua
        .lines()
        .position(|l| l.contains("function SCENE.__start__"))
        .map(|i| i as u32 + 1)
        .expect("生成 `.lua` に `__start__` ヘッダがある");

    // 各 `.pasta` 行の「本体側 `.lua` 行」（ヘッダより後）を昇順で集める。
    let body_lua_for = |pl: u32| -> Vec<u32> {
        let mut v: Vec<u32> = map
            .resolve_pasta_to_lua(&pasta_file_key, pl)
            .iter()
            .map(|(_, l)| *l)
            .filter(|l| *l > header_lua_line)
            .collect();
        v.sort_unstable();
        v
    };

    // multi: 本体側で ≥2 本へ展開される最初の `.pasta` 行。
    let mut multi: Option<(u32, Vec<u32>)> = None;
    for pl in 1..=60u32 {
        let body = body_lua_for(pl);
        if body.len() >= 2 {
            multi = Some((pl, body));
            break;
        }
    }
    let (multi_pasta_line, multi_lua_lines) =
        multi.expect("fixture invariant: ある `.pasta` トーク行が本体で ≥2 の `.lua` 行へ展開される");
    let first_multi_lua = multi_lua_lines[0];
    let last_multi_lua = *multi_lua_lines.last().unwrap();

    // origin: multi の直前で、本体側 `.lua` 行を**ちょうど 1 本**持ち、その行が multi の先頭
    // 本体行より手前にある `.pasta` 行（= step 起点。単一行なので line-hook 1 回で素直に進む）。
    let mut origin: Option<(u32, u32)> = None;
    for pl in (1..multi_pasta_line).rev() {
        let body = body_lua_for(pl);
        if body.len() == 1 && body[0] < first_multi_lua {
            origin = Some((pl, body[0]));
            break;
        }
    }
    let (origin_pasta_line, origin_lua_line) =
        origin.expect("fixture invariant: 多対1行の手前に単一 `.lua` 本体行の `.pasta` 行がある");

    // multi の各本体 `.lua` 行が当該チャンクで同一 `.pasta` 行へ等価解決すること（消化対象）。
    for &lua_line in &multi_lua_lines {
        let back = map
            .resolve_lua_to_pasta(&chunk, lua_line)
            .expect("本体 `.lua` 行は前方解決できる");
        assert_eq!(
            back.line, multi_pasta_line,
            "多対1: `.lua` 行 {lua_line} は `.pasta` 行 {multi_pasta_line} へ等価解決する"
        );
    }

    // next: multi の最終本体 `.lua` 行より後ろに本体行を持つ最初の異なる `.pasta` 行。
    let mut next: Option<(u32, u32)> = None;
    for pl in (multi_pasta_line + 1)..=60u32 {
        let body: Vec<u32> = body_lua_for(pl)
            .into_iter()
            .filter(|l| *l > last_multi_lua)
            .collect();
        if let Some(&lua_line) = body.iter().min() {
            next = Some((pl, lua_line));
            break;
        }
    }
    let (next_pasta_line, next_lua_line) =
        next.expect("fixture invariant: 多対1行の直後に異なる `.pasta` 行（本体実行）がある");

    assert!(
        origin_pasta_line < multi_pasta_line && multi_pasta_line < next_pasta_line,
        "順序: origin {origin_pasta_line} < multi {multi_pasta_line} < next {next_pasta_line}"
    );
    assert!(
        origin_lua_line < first_multi_lua && last_multi_lua < next_lua_line,
        "本体 `.lua` 行の順序: origin {origin_lua_line} < multi {multi_lua_lines:?} < next {next_lua_line}"
    );

    StepCoords {
        base: base.to_path_buf(),
        chunk,
        pasta_file_key,
        generated_lua,
        origin_pasta_line,
        origin_lua_line,
        multi_pasta_line,
        multi_lua_lines,
        next_pasta_line,
        next_lua_line,
    }
}

/// ステップ粒度シナリオの停止セッション。host は (define exec -> driver exec) の 2 段。
struct StepSession {
    client: DapClient,
    thread_id: u64,
    host: std::thread::JoinHandle<Result<(), String>>,
}

/// scene を定義してから `__start__` を駆動し、**origin（単一 `.lua` 本体行）の BP** で停止する
/// まで進める。`initial_mode`（"pasta" | "lua"）が attach の `sourcePresentation`。`.pasta` 提示
/// なら origin の `.pasta` 行 BP（翻訳経路）、`.lua` 提示なら origin の本体 `.lua` 行を直接張る。
/// どちらでも停止位置 = 同一の本体 `.lua` 行 `origin_lua_line`。
fn start_step_session(coords: &StepCoords, initial_mode: &str) -> StepSession {
    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (go_tx, go_rx) = mpsc::channel::<()>();
    // Host-status channel (see `DapClient::host_status`): the host reports its
    // final outcome here before exiting, so a host-side failure surfaces in the
    // client's panic even when it closes the socket first. The label distinguishes
    // the two step sessions (initial `.lua` vs `.pasta`) in CI logs.
    let (status_tx, status_rx) = mpsc::channel::<String>();
    let mode_label = initial_mode.to_string();

    let base_for_host = coords.base.clone();
    let chunk_for_host = coords.chunk.clone();
    let generated_for_host = coords.generated_lua.clone();

    let host = std::thread::spawn(move || -> Result<(), String> {
        // Run the host body in an inner closure so its final `Result` (whichever
        // `?` short-circuited) can be reported on `status_tx` before the thread
        // exits and drops the runtime (which closes the client socket). Without
        // this, a host-side failure is hidden behind the client's bare
        // "peer did not close" frame-recv panic.
        let body = || -> Result<(), String> {
            let runtime = PastaLoader::load_with_config(&base_for_host, RuntimeConfig::new())
                .map_err(|e| format!("loader must build an enabled-debug runtime: {e}"))?;
            if !runtime.debug_enabled() {
                return Err("enabled [debug] must install the backend".to_string());
            }
            if runtime.debug_source_map().is_none() {
                return Err("enabled debug runtime must hold the aggregated source map".to_string());
            }

            let addr = runtime.debug_local_addr().ok_or_else(|| {
                "enabled runtime must expose a bound debug addr (port 0)".to_string()
            })?;
            addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

            // (define) scene を**定義**する。BP は未設定なので talk 本体行では止まらない。
            runtime
                .exec_named(&generated_for_host, &chunk_for_host)
                .map_err(|e| format!("scene define exec failed: {e}"))?;

            // クライアントが BP+configurationDone を終えるまで待つ。
            go_rx
                .recv_timeout(WATCHDOG)
                .map_err(|_| "no go signal before driver exec".to_string())?;
            // (driver) `__start__` を駆動 -> talk 本体行が実行され BP ヒット。
            // BP がヒットすれば VM はここで停止しブロックする。BP が当たらなければ
            // driver はそのまま完走して戻る。
            runtime
                .exec(STEP_DRIVER)
                .map_err(|e| format!("scene driver exec failed: {e}"))?;

            drop(runtime);
            Ok(())
        };
        let result = body();
        // Report the final outcome regardless of success/failure so a host-side
        // failure (or a driver that completed without the breakpoint ever holding
        // the VM, = `Ok(())`) surfaces in the client's panic instead of a bare
        // "peer did not close".
        let _ = status_tx.send(format!("[{mode_label}] thread result = {result:?}"));
        result
    });

    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);
    client.host_status = Some(status_rx);

    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    client.send_request(2, "attach", json!({ "sourcePresentation": initial_mode }));
    let _ = client.recv_until(|m| is_response(m, "attach"));
    let attach_event = client.recv_until(|m| is_event(m, "pasta/sourcePresentation"));
    assert_eq!(
        attach_event["body"]["mode"], initial_mode,
        "attach 完了時の push イベントは初期解決モード {initial_mode} を報告する"
    );

    let (bp_source_path, bp_line) = if initial_mode == "lua" {
        (coords.chunk.clone(), coords.origin_lua_line)
    } else {
        (coords.pasta_file_key.clone(), coords.origin_pasta_line)
    };
    client.send_request(
        3,
        "setBreakpoints",
        json!({
            "source": { "path": bp_source_path },
            "breakpoints": [{ "line": bp_line }],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let bps = bp_resp["body"]["breakpoints"].as_array().expect("breakpoints array");
    assert_eq!(bps.len(), 1, "exactly one breakpoint resolved");
    assert_eq!(
        bps[0]["verified"], true,
        "初期 {initial_mode} 提示座標で張った origin BP は検証済み"
    );

    client.send_request(4, "configurationDone", json!({}));
    let _ = client.recv_until(|m| is_response(m, "configurationDone"));

    // driver exec 開始 -> origin の本体 `.lua` 行 BP で停止。
    go_tx.send(()).expect("go driver");
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped["body"]["reason"], "breakpoint",
        "driver は origin（単一 `.lua` 本体行）の BP で停止する"
    );
    let thread_id = stopped["body"]["threadId"].as_u64().unwrap_or(1);

    StepSession {
        client,
        thread_id,
        host,
    }
}

/// 停止セッションを continue で流し切り、host を watchdog 内で join する。
fn finish_step_session(mut session: StepSession) {
    session
        .client
        .send_request(900, "continue", json!({ "threadId": session.thread_id }));
    let _ = session.client.recv_until(|m| is_response(m, "continue"));

    let host = session.host;
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(host.join());
    });
    match done_rx.recv_timeout(WATCHDOG) {
        Ok(joined) => {
            joined
                .expect("host thread must not panic")
                .expect("define + driver execs must run to completion");
        }
        Err(RecvTimeoutError::Timeout) => panic!("host thread did not finish (hang?)"),
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }
}

/// origin BP 停止状態から、BP を**解除**して `next` を 1 回送り、多対1行の先頭で停止させる
/// （両モードで同一の `.lua` 実行位置。提示行は `initial_lua` のとき `.lua` 先頭行、そうでなければ
/// 多対1 `.pasta` 行）。BP 解除は同一 `.lua` 行への line-hook 再入による BP 再発火を避けるため
/// （`.lua` モードにはアンカー合体が無いので必須）。トグル前の足場をここで確定する。
fn clear_bp_and_step_into_multi(session: &mut StepSession, coords: &StepCoords, initial_lua: bool) {
    // origin BP を解除（`.lua` チャンク / `.pasta` ファイル の双方に空配列）。
    for (seq, path) in [(10, coords.chunk.clone()), (11, coords.pasta_file_key.clone())] {
        session.client.send_request(
            seq,
            "setBreakpoints",
            json!({ "source": { "path": path }, "breakpoints": [] }),
        );
        let _ = session.client.recv_until(|m| is_response(m, "setBreakpoints"));
    }

    session
        .client
        .send_request(12, "next", json!({ "threadId": session.thread_id }));
    let _ = session.client.recv_until(|m| is_response(m, "next"));
    let stopped = session.client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped["body"]["reason"], "step",
        "origin からの `next` は reason step で多対1行へ進む（BP 再発火なし）"
    );

    session
        .client
        .send_request(13, "stackTrace", json!({ "threadId": session.thread_id }));
    let stack = session.client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"].as_array().expect("stackFrames");
    let top_line = frames[0]["line"].as_u64().expect("line") as u32;
    let top_path = frames[0]["source"]["path"].as_str().expect("source path");
    if initial_lua {
        // `.lua` 提示: 多対1の先頭 `.lua` 実行行を提示。
        assert_eq!(
            canonicalize_chunk_name(top_path),
            canonicalize_chunk_name(&coords.chunk),
            "`.lua` 提示: トップフレーム source は生成 `.lua` チャンク (got {top_path})"
        );
        assert_eq!(
            top_line, coords.multi_lua_lines[0],
            "1 回目 `next`（`.lua` 提示）は多対1行の先頭 `.lua` 行 {} で停止する",
            coords.multi_lua_lines[0]
        );
    } else {
        // `.pasta` 提示: 多対1 `.pasta` 行を提示。
        assert_eq!(
            canonicalize_chunk_name(top_path),
            canonicalize_chunk_name(&coords.pasta_file_key),
            "`.pasta` 提示: トップフレーム source は `.pasta` ファイル (got {top_path})"
        );
        assert_eq!(
            top_line, coords.multi_pasta_line,
            "1 回目 `next`（`.pasta` 提示）は多対1 `.pasta` 行 {} で停止する",
            coords.multi_pasta_line
        );
    }
}

/// 実行時トグルを送り、(a) 受理レスポンスのエコー、(b) 同名イベント、(c) 再描画 `stopped` 再送、
/// を消化して検証する（5.1/5.2 と同じ契約）。
fn step_toggle_mode(session: &mut StepSession, seq: u64, mode: &str) {
    session
        .client
        .send_request(seq, "pasta/sourcePresentation", json!({ "mode": mode }));
    let resp = session
        .client
        .recv_until(|m| is_response(m, "pasta/sourcePresentation"));
    assert_eq!(
        resp["body"]["mode"], mode,
        "受理レスポンスは適用後モード {mode} をエコーする"
    );
    let _event = session
        .client
        .recv_until(|m| is_event(m, "pasta/sourcePresentation") && m["body"]["mode"] == mode);
    // 再描画のための `stopped` 再送（3.3）。再送は **現在の停止** をそのまま再送するため、
    // reason は現停止のもの（step で多対1行に止まっているので "step"）になる。RunMode は変えず、
    // 以後のステップ粒度だけが `effective_mode()` 経由で反転する。
    let restopped = session.client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        restopped["body"]["reason"], "step",
        "3.3: 切替後、現停止（step で多対1行）が再送され再描画が起動する"
    );
}

/// `next` を送り、停止後のトップフレーム行（`.lua`/`.pasta` 提示の数値行）を返す。`reason step`
/// と、提示モードに応じた `source.path`（`.lua` チャンク / `.pasta` ファイル）を表明する。
fn step_next_then_top_line(session: &mut StepSession, coords: &StepCoords, base_seq: u64, expect_lua: bool) -> u32 {
    session
        .client
        .send_request(base_seq, "next", json!({ "threadId": session.thread_id }));
    let _ = session.client.recv_until(|m| is_response(m, "next"));
    let stopped = session.client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "`next` は reason step で再停止する");

    session
        .client
        .send_request(base_seq + 1, "stackTrace", json!({ "threadId": session.thread_id }));
    let stack = session.client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"].as_array().expect("stackFrames");
    assert!(!frames.is_empty(), "停止フレームが存在する");
    let path = frames[0]["source"]["path"].as_str().expect("source path");
    let expect_path = if expect_lua { &coords.chunk } else { &coords.pasta_file_key };
    assert_eq!(
        canonicalize_chunk_name(path),
        canonicalize_chunk_name(expect_path),
        "{} 提示: トップフレーム source は期待する提示先 (got {path})",
        if expect_lua { "`.lua`" } else { "`.pasta`" }
    );
    frames[0]["line"].as_u64().expect("line") as u32
}

/// 5.3（`.lua`->`.pasta` 停止中トグル）+ 5.1: 初期 `.lua` 提示で origin BP 停止 -> BP 解除 ->
/// `next` で多対1行へ -> **停止中に `.pasta` へトグル** -> 次の `next` が **`.pasta` 粒度** に
/// なり、同一 `.pasta` 行の残り `.lua` 行を消化して次の異なる `.pasta` 行で停止する
/// （`.lua` 粒度のままなら同一 `.pasta` 行内の 2 本目 `.lua` 行で止まっていた）。
#[test]
fn paused_toggle_lua_to_pasta_switches_next_step_to_pasta_granularity() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let coords = resolve_step_session(temp.path());
    let mut session = start_step_session(&coords, "lua");

    // origin BP 停止 -> BP 解除 -> `next` で多対1行の先頭へ（`.lua` 提示）。
    clear_bp_and_step_into_multi(&mut session, &coords, true);

    // 停止中に `.pasta` へトグル（5.3）。以後のステップは `.pasta` 粒度になる。
    step_toggle_mode(&mut session, 20, "pasta");

    // 2 回目 `next` は `.pasta` 粒度: 同一 `.pasta` 行 {multi} の残り `.lua` 行を消化し、次の
    // 異なる `.pasta` 行 {next} で停止する。
    let stopped_pasta = step_next_then_top_line(&mut session, &coords, 21, false);
    assert_eq!(
        stopped_pasta, coords.next_pasta_line,
        "5.3/5.1: 停止中 `.lua`->`.pasta` トグル後の `next` は `.pasta` 粒度 —— 同一 `.pasta` 行 \
         {} の `.lua` 行群（{:?}）を消化し、次の異なる `.pasta` 行 {} で停止する",
        coords.multi_pasta_line, coords.multi_lua_lines, coords.next_pasta_line
    );

    finish_step_session(session);
}

/// 5.3（`.pasta`->`.lua` 停止中トグル）+ 5.2: 初期 `.pasta` 提示で origin BP 停止 -> BP 解除 ->
/// `next` で多対1行へ -> **停止中に `.lua` へトグル** -> 次の `next` が **`.lua` 粒度** になり、
/// 同一 `.pasta` 行内の次の `.lua` 行（消化せず）で停止する（`.pasta` 粒度のままなら次の異なる
/// `.pasta` 行まで進んでいた）。逆方向の停止中トグルでも `effective_mode()` 毎行読取が粒度を反転
/// させる証拠（requirement 5.3 / 5.2）。
#[test]
fn paused_toggle_pasta_to_lua_switches_next_step_to_lua_granularity() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let coords = resolve_step_session(temp.path());
    let mut session = start_step_session(&coords, "pasta");

    // origin BP 停止 -> BP 解除 -> `next` で多対1行の先頭へ（`.pasta` 提示）。
    clear_bp_and_step_into_multi(&mut session, &coords, false);

    // 停止中に `.lua` へトグル（5.3）。以後のステップは `.lua` 粒度になる。
    step_toggle_mode(&mut session, 20, "lua");

    // 2 回目 `next` は `.lua` 粒度: 同一 `.pasta` 行内の次の `.lua` 行（消化しない）で停止する。
    let body_second_lua = coords.multi_lua_lines[1];
    let stopped_lua = step_next_then_top_line(&mut session, &coords, 21, true);
    assert_eq!(
        stopped_lua, body_second_lua,
        "5.3/5.2: 停止中 `.pasta`->`.lua` トグル後の `next` は `.lua` 粒度 —— 同一 `.pasta` 行 \
         {} 内の次の `.lua` 行 {} で停止する（次の異なる `.pasta` 行 {} まで進まない）",
        coords.multi_pasta_line, body_second_lua, coords.next_pasta_line
    );
    assert!(
        stopped_lua < coords.next_lua_line,
        "5.3/5.2: `.lua` 粒度の停止 `.lua` 行 {stopped_lua} は次の異なる `.pasta` 行の `.lua` 行 \
         {} より手前（同一 `.pasta` 行内に留まる）",
        coords.next_lua_line
    );

    finish_step_session(session);
}
