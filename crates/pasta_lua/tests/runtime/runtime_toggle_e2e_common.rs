//! Task 5.1/5.2 共有ヘルパー（C2 クラスタ分割で外出し）。
//!
//! `runtime_toggle_e2e_*` クラスタ（basic / initial_mode / step）が共有する DAP-over-TCP
//! クライアント・フレーミング・フィクスチャ配置・セッション駆動ヘルパーを集約する。元の
//! `runtime_toggle_e2e_test.rs` から **バイト不変** で移設したもの（テスト本体の分割に伴う
//! 構造的移動のみ）。各クラスタは `use crate::runtime_toggle_e2e_common::*;` で参照する。

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
pub(crate) const WATCHDOG: Duration = Duration::from_secs(15);

/// E2E フィクスチャ（単一トーク行 `.pasta`）。`fixtures/debug_toggle_e2e.pasta` と同一バイト列を
/// `dic/` 配下へ書き出し、loader でロードする。
pub(crate) const FIXTURE: &str = include_str!("../fixtures/debug_toggle_e2e.pasta");

/// BP を張る `.pasta` 行（フィクスチャの `＊あいさつ` 見出し行 = 4 行目）。
/// この `.pasta` 行は生成 `.lua` の `local SCENE = PASTA.create_scene(...)` 行へ一意に対応し、
/// その `.lua` 行はトップレベル `exec` 時に実行される（=フックが停止できる）。対応行番号は
/// `probe`（開発時）で確認済みだが、本テストはハードコードせずソースマップから動的に解決する。
pub(crate) const BP_PASTA_LINE: u32 = 4;

/// 実 TCP ソケット越しの最小 DAP クライアント（Content-Length フレーミング）。
/// 本体フレーミングは production の `read_frame`/`write_frame` を写したもの（crate 外からは private）。
pub(crate) struct DapClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    /// Optional host-status receiver. When the peer (VM host thread) closes the
    /// socket cleanly before sending an expected frame, the host has exited — the
    /// generic "peer did not close" panic then hides *why* it exited (a failed
    /// `exec`, a missing scene, etc.). If wired, the host sends a descriptive
    /// final-status string just before exiting, and `recv` surfaces it instead of
    /// the bare panic. This makes host-side failures diagnosable from the client
    /// panic alone — essential when the failure only reproduces on CI.
    pub(crate) host_status: Option<mpsc::Receiver<String>>,
}

impl DapClient {
    pub(crate) fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
        stream
            .set_read_timeout(Some(WATCHDOG))
            .expect("TEST-ONLY read timeout");
        let writer = stream.try_clone().expect("clone socket for writing");
        Self {
            reader: BufReader::new(stream),
            writer,
            host_status: None,
        }
    }

    pub(crate) fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
        let req = json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        });
        write_frame(&mut self.writer, &req).expect("client write must succeed");
    }

    pub(crate) fn recv(&mut self) -> Value {
        match read_frame(&mut self.reader).expect("client read must succeed (TEST-ONLY timeout)") {
            Some(value) => value,
            None => {
                // Clean EOF: the host (VM) thread closed the socket, i.e. it has
                // exited. Surface its actual final status (a failed `exec`, a
                // breakpoint that never held, etc.) so the cause is visible from
                // this panic — instead of the bare "peer did not close".
                if let Some(rx) = &self.host_status
                    && let Ok(status) = rx.recv_timeout(Duration::from_secs(5))
                {
                    panic!("a frame must be present, but the peer closed first; host {status}");
                }
                panic!("a frame must be present (peer did not close)");
            }
        }
    }

    pub(crate) fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
        loop {
            let msg = self.recv();
            if pred(&msg) {
                return msg;
            }
        }
    }
}

/// Write one DAP `Content-Length`-framed JSON message (TEST-LOCAL framing).
pub(crate) fn write_frame<W: Write>(out: &mut W, value: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(value)?;
    write!(out, "Content-Length: {}\r\n\r\n", body.len())?;
    out.write_all(&body)?;
    out.flush()
}

/// Read one DAP `Content-Length`-framed JSON message (TEST-LOCAL framing).
pub(crate) fn read_frame<R: BufRead>(reader: &mut R) -> std::io::Result<Option<Value>> {
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
        if let Some((name, val)) = trimmed.split_once(':')
            && name.trim().eq_ignore_ascii_case("Content-Length")
        {
            content_length = val.trim().parse::<usize>().ok();
        }
    }
    let len = content_length.expect("framed message must carry a Content-Length");
    let mut body = vec![0u8; len];
    std::io::Read::read_exact(reader, &mut body)?;
    let value = serde_json::from_slice(&body)?;
    Ok(Some(value))
}

pub(crate) fn is_event(msg: &Value, name: &str) -> bool {
    msg["type"] == "event" && msg["event"] == name
}

pub(crate) fn is_response(msg: &Value, command: &str) -> bool {
    msg["type"] == "response" && msg["command"] == command
}

/// フレームの `source.path` が `.pasta` 提示（当該 `.pasta` ファイル）であることを、
/// バックエンドと同一の正規化規則（`canonicalize_chunk_name`: 区切り統一・Windows 大小無視）で
/// 突合する。バックエンドは `.pasta` パスを正規化系で提示するため、生のパス文字列の完全一致では
/// なく canonical 一致で判定する（design "Source Identity"）。
pub(crate) fn assert_pasta_source(frame: &Value, expect_pasta_file: &str, ctx: &str) {
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
pub(crate) fn make_base_dir(base: &Path) -> PathBuf {
    make_base_dir_with(base, None)
}

/// `make_base_dir` の `[debug] present_as` をパラメータ化した版（task 5.2、requirement 4.4 file/default 階層）。
/// `present_as = Some("lua")` のとき `pasta.toml` に `present_as = "lua"` を書き出し、loader の初期解決
/// （`DebugConfig::from_env`: env > file > 既定）を **file 階層 = `lua`** へ確定させる。`None` のときは
/// `present_as` キー自体を省略し、**既定 = `.pasta`** を確定させる（env はテストハーネスで変更しない）。
pub(crate) fn make_base_dir_with(base: &Path, present_as: Option<&str>) -> PathBuf {
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

pub(crate) fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
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
pub(crate) fn transpile_fixture(file: &Path) -> String {
    let parsed = parse_str(FIXTURE, &file.to_string_lossy()).expect("fixture must parse");
    let transpiler = LuaTranspiler::default();
    let mut out = Vec::new();
    transpiler
        .transpile(&parsed, &mut out)
        .expect("fixture must transpile");
    String::from_utf8(out).expect("generated lua is valid utf-8")
}

/// 1 セッション分の「BP `.pasta`→`.lua` 実行座標」解決結果（5.1 メインテストと同一手順）。
pub(crate) struct SessionCoords {
    pub(crate) base: PathBuf,
    pub(crate) pasta_file_key: String,
    pub(crate) chunk: String,
    pub(crate) bp_lua_line: u32,
    pub(crate) generated_lua: String,
}

/// 5.1 メインテストの事前計算ブロックを関数化したもの（ハーネスを fork せず再利用）。
/// `present_as` は `pasta.toml` `[debug] present_as`（file 階層の初期モード）を制御する。
pub(crate) fn resolve_session(base: &Path, present_as: Option<&str>) -> SessionCoords {
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
pub(crate) struct StoppedSession {
    pub(crate) client: DapClient,
    pub(crate) thread_id: u64,
    pub(crate) go_tx: mpsc::Sender<()>,
    pub(crate) host: std::thread::JoinHandle<Result<(), String>>,
}

/// セッションを起動し、最初の BP 停止まで進めて停止状態で返す。
/// `attach_source_presentation`: attach 引数の `sourcePresentation`（`None` なら省略 = 4.4 経路）。
/// `expected_initial_mode`: attach push イベントで期待する初期解決モード（"pasta" | "lua"）。
pub(crate) fn start_stopped_session(
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
pub(crate) fn finish_session(mut session: StoppedSession) {
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
pub(crate) fn assert_lua_frame(session: &mut StoppedSession, coords: &SessionCoords, seq: u64, ctx: &str) {
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
pub(crate) fn assert_pasta_frame(session: &mut StoppedSession, coords: &SessionCoords, seq: u64, ctx: &str) {
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
pub(crate) fn toggle_mode(session: &mut StoppedSession, seq: u64, mode: &str) {
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
