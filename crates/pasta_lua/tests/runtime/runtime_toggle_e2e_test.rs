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

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::{Value, json};

use pasta_dsl::parser::parse_str;
use pasta_lua::debug::source_map::canonicalize_chunk_name;
use pasta_lua::loader::CacheManager;
use pasta_lua::{LuaTranspiler, PastaLoader, RuntimeConfig, SourceMode};

/// TEST-ONLY watchdog。停止コア自体は無期限。
const WATCHDOG: Duration = Duration::from_secs(15);

/// E2E フィクスチャ（単一トーク行 `.pasta`）。`fixtures/debug_toggle_e2e.pasta` と同一バイト列を
/// `dic/` 配下へ書き出し、loader でロードする。
const FIXTURE: &str = include_str!("../fixtures/debug_toggle_e2e.pasta");

/// BP を張る `.pasta` 行（フィクスチャの `＊あいさつ` 見出し行 = 4 行目）。
/// この `.pasta` 行は生成 `.lua` の `local SCENE = PASTA.create_scene(...)` 行へ一意に対応し、
/// その `.lua` 行はトップレベル `exec` 時に実行される（=フックが停止できる）。対応行番号は
/// `probe`（開発時）で確認済みだが、本テストはハードコードせずソースマップから動的に解決する。
const BP_PASTA_LINE: u32 = 4;

/// 実 TCP ソケット越しの最小 DAP クライアント（Content-Length フレーミング）。
/// 本体フレーミングは production の `read_frame`/`write_frame` を写したもの（crate 外からは private）。
struct DapClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl DapClient {
    fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
        stream
            .set_read_timeout(Some(WATCHDOG))
            .expect("TEST-ONLY read timeout");
        let writer = stream.try_clone().expect("clone socket for writing");
        Self {
            reader: BufReader::new(stream),
            writer,
        }
    }

    fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
        let req = json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        });
        write_frame(&mut self.writer, &req).expect("client write must succeed");
    }

    fn recv(&mut self) -> Value {
        read_frame(&mut self.reader)
            .expect("client read must succeed (TEST-ONLY timeout)")
            .expect("a frame must be present (peer did not close)")
    }

    fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
        loop {
            let msg = self.recv();
            if pred(&msg) {
                return msg;
            }
        }
    }
}

/// Write one DAP `Content-Length`-framed JSON message (TEST-LOCAL framing).
fn write_frame<W: Write>(out: &mut W, value: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(value)?;
    write!(out, "Content-Length: {}\r\n\r\n", body.len())?;
    out.write_all(&body)?;
    out.flush()
}

/// Read one DAP `Content-Length`-framed JSON message (TEST-LOCAL framing).
fn read_frame<R: BufRead>(reader: &mut R) -> std::io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, val)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("Content-Length") {
                content_length = val.trim().parse::<usize>().ok();
            }
        }
    }
    let len = content_length.expect("framed message must carry a Content-Length");
    let mut body = vec![0u8; len];
    std::io::Read::read_exact(reader, &mut body)?;
    let value = serde_json::from_slice(&body)?;
    Ok(Some(value))
}

fn is_event(msg: &Value, name: &str) -> bool {
    msg["type"] == "event" && msg["event"] == name
}

fn is_response(msg: &Value, command: &str) -> bool {
    msg["type"] == "response" && msg["command"] == command
}

/// フレームの `source.path` が `.pasta` 提示（当該 `.pasta` ファイル）であることを、
/// バックエンドと同一の正規化規則（`canonicalize_chunk_name`: 区切り統一・Windows 大小無視）で
/// 突合する。バックエンドは `.pasta` パスを正規化系で提示するため、生のパス文字列の完全一致では
/// なく canonical 一致で判定する（design "Source Identity"）。
fn assert_pasta_source(frame: &Value, expect_pasta_file: &str, ctx: &str) {
    let got = frame["source"]["path"].as_str().expect("`.pasta` 提示 source path");
    assert_eq!(
        canonicalize_chunk_name(got),
        canonicalize_chunk_name(expect_pasta_file),
        "{ctx}: トップフレーム source は `.pasta` ファイル (got {got})"
    );
}

/// テスト用 base_dir 配下にフィクスチャ `.pasta` と `[debug] enabled, port=0` の pasta.toml を
/// 配置し、ランタイム初期化に必要な pasta_scripts / scriptlibs をクレートルートからコピーする
/// （`source_map_handoff_test.rs` と同じ構成）。`.pasta` の絶対パスを返す。
fn make_base_dir(base: &Path) -> PathBuf {
    make_base_dir_with(base, None)
}

/// `make_base_dir` の `[debug] present_as` をパラメータ化した版（task 5.2、requirement 4.4 file/default 階層）。
/// `present_as = Some("lua")` のとき `pasta.toml` に `present_as = "lua"` を書き出し、loader の初期解決
/// （`DebugConfig::from_env`: env > file > 既定）を **file 階層 = `lua`** へ確定させる。`None` のときは
/// `present_as` キー自体を省略し、**既定 = `.pasta`** を確定させる（env はテストハーネスで変更しない）。
fn make_base_dir_with(base: &Path, present_as: Option<&str>) -> PathBuf {
    let pasta_file = base.join("dic/test/debug_toggle_e2e.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, FIXTURE).unwrap();
    let present_as_line = match present_as {
        Some(mode) => format!("present_as = \"{mode}\"\n"),
        None => String::new(),
    };
    std::fs::write(
        base.join("pasta.toml"),
        format!(
            "\
[loader]
debug_mode = true

[debug]
enabled = true
port = 0
{present_as_line}"
        ),
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

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest)?;
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// フィクスチャを本番ローダと同じ（sink 無し）トランスパイルし、生成 `.lua` テキストを返す。
/// loader の `build_source_map`（sink 有り）の出力はこれとバイト一致するため
/// （`sink_attachment_is_byte_invariant`）、`exec_named` の chunk 名と行番号がマップと整合する。
fn transpile_fixture(file: &Path) -> String {
    let parsed = parse_str(FIXTURE, &file.to_string_lossy()).expect("fixture must parse");
    let transpiler = LuaTranspiler::default();
    let mut out = Vec::new();
    transpiler
        .transpile(&parsed, &mut out)
        .expect("fixture must transpile");
    String::from_utf8(out).expect("generated lua is valid utf-8")
}

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

/// 1 セッション分の「BP `.pasta`→`.lua` 実行座標」解決結果（5.1 メインテストと同一手順）。
struct SessionCoords {
    base: PathBuf,
    pasta_file_key: String,
    chunk: String,
    bp_lua_line: u32,
    generated_lua: String,
}

/// 5.1 メインテストの事前計算ブロックを関数化したもの（ハーネスを fork せず再利用）。
/// `present_as` は `pasta.toml` `[debug] present_as`（file 階層の初期モード）を制御する。
fn resolve_session(base: &Path, present_as: Option<&str>) -> SessionCoords {
    let pasta_file = make_base_dir_with(base, present_as);
    let cache_manager = CacheManager::new(base.to_path_buf(), "profile/pasta/cache/lua");
    let chunk = cache_manager
        .source_to_cache_path(&pasta_file)
        .to_string_lossy()
        .to_string();
    let pasta_file_key = pasta_file.to_string_lossy().to_string();

    let expect_map =
        PastaLoader::build_source_map(std::slice::from_ref(&pasta_file), &cache_manager, false);
    let bp_lua_coords = expect_map.resolve_pasta_to_lua(&pasta_file_key, BP_PASTA_LINE);
    assert_eq!(
        bp_lua_coords.len(),
        1,
        "fixture invariant: BP `.pasta` 行 {BP_PASTA_LINE} は単一の `.lua` 実行座標へ一意対応する, got {bp_lua_coords:?}"
    );
    let (bp_chunk, bp_lua_line) = bp_lua_coords[0].clone();
    assert_eq!(
        canonicalize_chunk_name(&bp_chunk),
        canonicalize_chunk_name(&chunk),
        "BP の `.lua` 実行座標は当該チャンクを指す"
    );
    let generated_lua = transpile_fixture(&pasta_file);
    SessionCoords {
        base: base.to_path_buf(),
        pasta_file_key,
        chunk,
        bp_lua_line,
        generated_lua,
    }
}

/// attach（オプションの `sourcePresentation`）→ setBreakpoints → configurationDone →
/// exec #1 で BP 停止、までを駆動する単一停止セッションハーネス。停止状態の `client` と
/// `thread_id`、teardown 用 `go_tx`/`host`/`done_*` を返す。caller は停止状態で提示モードを
/// 検証・トグルし、最後に `finish_session` で continue → teardown する。
///
/// `expected_initial_mode` は attach 直後に push される `pasta/sourcePresentation` イベントの
/// 期待値（初期解決モード）。「最初の停止が既にこのモードの座標である」ことを caller が assert
/// できるよう、停止状態のまま返す。
struct StoppedSession {
    client: DapClient,
    thread_id: u64,
    go_tx: mpsc::Sender<()>,
    host: std::thread::JoinHandle<Result<(), String>>,
}

/// セッションを起動し、最初の BP 停止まで進めて停止状態で返す。
/// `attach_source_presentation`: attach 引数の `sourcePresentation`（`None` なら省略 = 4.4 経路）。
/// `expected_initial_mode`: attach push イベントで期待する初期解決モード（"pasta" | "lua"）。
fn start_stopped_session(
    coords: &SessionCoords,
    attach_source_presentation: Option<&str>,
    expected_initial_mode: &str,
) -> StoppedSession {
    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (go_tx, go_rx) = mpsc::channel::<()>();

    let base_for_host = coords.base.clone();
    let chunk_for_host = coords.chunk.clone();
    let generated_for_host = coords.generated_lua.clone();
    let expected_host_mode = expected_initial_mode.to_string();
    let attach_present_for_host = attach_source_presentation.is_some();

    let host = std::thread::spawn(move || -> Result<(), String> {
        let runtime = PastaLoader::load_with_config(&base_for_host, RuntimeConfig::new())
            .map_err(|e| format!("loader must build an enabled-debug runtime: {e}"))?;
        if !runtime.debug_enabled() {
            return Err("enabled [debug] must install the backend".to_string());
        }
        if runtime.debug_source_map().is_none() {
            return Err("enabled debug runtime must hold the aggregated source map".to_string());
        }
        // `debug_source_mode()` は BAKED 解決（env > file > 既定）を返す（attach 引数や実行時
        // トグルは SharedSourceMode を変えるが baked config は不変）。attach 引数が無い経路
        // （4.4）では、この baked 値が初期解決モードと一致しなければならない（file/default 階層）。
        if !attach_present_for_host {
            let baked = runtime.debug_source_mode();
            let expect = match expected_host_mode.as_str() {
                "lua" => SourceMode::Lua,
                _ => SourceMode::Pasta,
            };
            if baked != Some(expect) {
                return Err(format!(
                    "4.4: attach 引数なしの初期解決モード（env>file>既定）は {expect:?} のはず, got {baked:?}"
                ));
            }
        }

        let addr = runtime
            .debug_local_addr()
            .ok_or_else(|| "enabled runtime must expose a bound debug addr (port 0)".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "no go signal before exec #1".to_string())?;
        runtime
            .exec_named(&generated_for_host, &chunk_for_host)
            .map_err(|e| format!("exec #1 failed: {e}"))?;
        drop(runtime);
        Ok(())
    });

    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

    // initialize ハンドシェイク。
    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    // attach（オプションの `sourcePresentation`）。attach 完了時にバックエンドは解決済み初期
    // モードを `pasta/sourcePresentation` イベントとして push する（design "Event Contract" (a)）。
    let attach_args = match attach_source_presentation {
        Some(mode) => json!({ "sourcePresentation": mode }),
        None => json!({}),
    };
    client.send_request(2, "attach", attach_args);
    let _attach_ack = client.recv_until(|m| is_response(m, "attach"));

    // (4.1 / 4.4) attach 時 push イベントが初期解決モードを報告する。
    let attach_event = client.recv_until(|m| is_event(m, "pasta/sourcePresentation"));
    assert_eq!(
        attach_event["body"]["mode"], expected_initial_mode,
        "attach 完了時の push イベントは初期解決モード {expected_initial_mode} を報告する \
         (design \"Event Contract\" (a))"
    );

    // BP は **初期提示モードに合わせた座標** で張る（利用者が見ている提示でブレークを張る経路を
    // 模す）。`.pasta` 提示なら `.pasta` source の `.pasta` 行（バックエンドが `.lua` 実行座標へ
    // 翻訳して登録: task 5.3）、`.lua` 提示なら生成 `.lua` チャンクの `.lua` 実行行を直接張る
    // （`.lua` モードでは `.pasta` source 翻訳は行われない: wiring.rs `pasta_active()` ガード）。
    // どちらのモードでも停止位置 = 同一の `.lua` 実行座標 `bp_lua_line` で、提示だけが異なる。
    let (bp_source_path, bp_line) = if expected_initial_mode == "lua" {
        (coords.chunk.clone(), coords.bp_lua_line)
    } else {
        (coords.pasta_file_key.clone(), BP_PASTA_LINE)
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
        "初期 {expected_initial_mode} 提示座標で張った BP は検証済み"
    );

    client.send_request(4, "configurationDone", json!({}));
    let _ = client.recv_until(|m| is_response(m, "configurationDone"));

    // exec #1 開始 → BP 停止。
    go_tx.send(()).expect("go #1");
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped["body"]["reason"], "breakpoint",
        "exec #1 は `.pasta` 行 BP で停止する"
    );
    let thread_id = stopped["body"]["threadId"].as_u64().unwrap_or(1);

    StoppedSession {
        client,
        thread_id,
        go_tx,
        host,
    }
}

/// 停止セッションを continue で流し切り、host スレッドを watchdog 内で join する。
fn finish_session(mut session: StoppedSession) {
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
                .expect("exec #1 must run to completion");
        }
        Err(RecvTimeoutError::Timeout) => panic!("host thread did not finish (hang?)"),
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }
    // go_tx を保持して early-drop による host 側 recv エラーを避ける。
    drop(session.go_tx);
}

/// 停止状態のトップフレームが生成 `.lua` 座標（path = chunk, line = bp_lua_line）であることを assert。
fn assert_lua_frame(session: &mut StoppedSession, coords: &SessionCoords, seq: u64, ctx: &str) {
    session
        .client
        .send_request(seq, "stackTrace", json!({ "threadId": session.thread_id }));
    let stack = session.client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"].as_array().expect("stackFrames");
    assert!(!frames.is_empty(), "{ctx}: 停止フレームが存在する");
    let path = frames[0]["source"]["path"].as_str().expect("`.lua` 提示 source path");
    assert_eq!(
        canonicalize_chunk_name(path),
        canonicalize_chunk_name(&coords.chunk),
        "{ctx}: トップフレーム source は生成 `.lua` チャンク (got {path})"
    );
    assert_eq!(
        frames[0]["line"].as_u64().expect("lua line"),
        coords.bp_lua_line as u64,
        "{ctx}: トップフレーム行は生成 `.lua` 実行行 {}",
        coords.bp_lua_line
    );
}

/// 停止状態のトップフレームが `.pasta` 座標（file/line）であることを assert（既存 `assert_pasta_source` 利用）。
fn assert_pasta_frame(session: &mut StoppedSession, coords: &SessionCoords, seq: u64, ctx: &str) {
    session
        .client
        .send_request(seq, "stackTrace", json!({ "threadId": session.thread_id }));
    let stack = session.client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"].as_array().expect("stackFrames");
    assert!(!frames.is_empty(), "{ctx}: 停止フレームが存在する");
    assert_pasta_source(&frames[0], &coords.pasta_file_key, ctx);
    assert_eq!(
        frames[0]["line"], BP_PASTA_LINE,
        "{ctx}: トップフレーム行は `.pasta` 行 {BP_PASTA_LINE}"
    );
}

/// 実行時トグルを送り、(a) 受理レスポンスが適用後モードをエコー、(b) 同名イベントが新モードを push、
/// (c) 再描画のための `stopped` 再送、を消費して検証する（5.1 と同じ契約）。
fn toggle_mode(session: &mut StoppedSession, seq: u64, mode: &str) {
    session
        .client
        .send_request(seq, "pasta/sourcePresentation", json!({ "mode": mode }));
    let resp = session
        .client
        .recv_until(|m| is_response(m, "pasta/sourcePresentation"));
    assert_eq!(
        resp["body"]["mode"], mode,
        "受理レスポンスは適用後モード {mode} をエコーする (requirement 1.3)"
    );
    let _event = session
        .client
        .recv_until(|m| is_event(m, "pasta/sourcePresentation") && m["body"]["mode"] == mode);
    let restopped = session.client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        restopped["body"]["reason"], "breakpoint",
        "3.3: 切替後、現停止が再送され再描画が起動する"
    );
}

/// 4.1 / 4.2 / 4.3: attach 引数 `sourcePresentation: "lua"` が初期提示モードとして適用され、
/// **最初の停止**が既に `.lua` 座標であること（4.1）、続いて実行時トグルで `.pasta` へ上書き
/// すると以後の提示が `.pasta` へ切り替わり持続すること（4.2 / 4.3）を実 DAP-over-TCP で検証する。
#[test]
fn attach_initial_lua_is_applied_then_runtime_toggle_overrides_to_pasta() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    // attach 引数が最優先（attach > env > file > 既定）なので file 階層は既定（`.pasta`）のまま、
    // attach の "lua" が初期モードを決める。
    let coords = resolve_session(temp.path(), None);
    let mut session = start_stopped_session(&coords, Some("lua"), "lua");

    // (4.1) 最初の停止は既に `.lua` 座標（attach 初期モードが適用済み・トグル前）。
    assert_lua_frame(&mut session, &coords, 10, "4.1: attach 初期 `.lua` での最初の停止");

    // (4.2) 実行時トグルで初期モード `.lua` を `.pasta` へ上書き。
    toggle_mode(&mut session, 20, "pasta");
    // (4.3) 上書き後モードが以後の提示で採用される。
    assert_pasta_frame(&mut session, &coords, 21, "4.2/4.3: トグルで初期 `.lua` を `.pasta` へ上書き");
    // (4.3 持続) 同一停止での再読でも上書き後モードが持続する（再トグルしていない）。
    assert_pasta_frame(&mut session, &coords, 22, "4.3: 上書き後モードが後続の読みでも持続");

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
    assert_lua_frame(&mut session, &coords, 10, "4.4 file: present_as=\"lua\" 初期 `.lua` の最初の停止");

    // (4.2/4.3) 解決済み初期モードを実行時トグルで `.pasta` へ上書きでき、以後持続する。
    toggle_mode(&mut session, 20, "pasta");
    assert_pasta_frame(&mut session, &coords, 21, "4.4/4.2/4.3: 解決済み `.lua` をトグルで `.pasta` へ上書き");
    assert_pasta_frame(&mut session, &coords, 22, "4.3: 上書き後モードが後続の読みでも持続");

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
    assert_pasta_frame(&mut session, &coords, 10, "4.4 default: 設定なし初期 `.pasta` の最初の停止");

    // (4.2/4.3) 解決済み既定 `.pasta` を実行時トグルで `.lua` へ上書きでき、以後持続する。
    toggle_mode(&mut session, 20, "lua");
    assert_lua_frame(&mut session, &coords, 21, "4.4/4.2/4.3: 解決済み `.pasta` をトグルで `.lua` へ上書き");
    assert_lua_frame(&mut session, &coords, 22, "4.3: 上書き後モードが後続の読みでも持続");

    finish_session(session);
}

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

    let base_for_host = coords.base.clone();
    let chunk_for_host = coords.chunk.clone();
    let generated_for_host = coords.generated_lua.clone();

    let host = std::thread::spawn(move || -> Result<(), String> {
        let runtime = PastaLoader::load_with_config(&base_for_host, RuntimeConfig::new())
            .map_err(|e| format!("loader must build an enabled-debug runtime: {e}"))?;
        if !runtime.debug_enabled() {
            return Err("enabled [debug] must install the backend".to_string());
        }
        if runtime.debug_source_map().is_none() {
            return Err("enabled debug runtime must hold the aggregated source map".to_string());
        }

        let addr = runtime
            .debug_local_addr()
            .ok_or_else(|| "enabled runtime must expose a bound debug addr (port 0)".to_string())?;
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
        runtime
            .exec(STEP_DRIVER)
            .map_err(|e| format!("scene driver exec failed: {e}"))?;

        drop(runtime);
        Ok(())
    });

    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

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
    assert_pasta_frame(&mut session, &coords, 10, "6.1: トグル未使用・初期 `.pasta` の最初の停止");
    // (6.1-b) 同一停止での再読でも `.pasta` のまま（追加 stackTrace で揺れない）。
    assert_pasta_frame(&mut session, &coords, 11, "6.1: トグル未使用・再読でも初期 `.pasta` が持続");

    // (6.1-c) **トグルせず** BP を解除して `next` を 1 回。停止後も `.pasta` 提示のまま
    // （別の `.pasta` 行へ進むが、提示モードはドリフトしない）。BP 解除は同一行 line-hook 再入での
    // BP 再発火を避けるためで、提示モードには影響しない。
    session.client.send_request(
        12,
        "setBreakpoints",
        json!({ "source": { "path": coords.pasta_file_key.clone() }, "breakpoints": [] }),
    );
    let _ = session.client.recv_until(|m| is_response(m, "setBreakpoints"));

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
    let frames = stack["body"]["stackFrames"].as_array().expect("stackFrames");
    assert!(!frames.is_empty(), "6.1: ステップ後も停止フレームが存在する");
    assert_pasta_source(
        &frames[0],
        &coords.pasta_file_key,
        "6.1: トグル未使用・ステップ後もトップフレームは `.pasta` 提示（ドリフトなし）",
    );

    finish_session(session);
}
