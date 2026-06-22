//! 作者デバッグ保全 E2E（task 6.1・R10.1/R10.2/R10.3）。
//!
//! task 5.1 で VM は SHIORI スレッドから **アクタースレッド**（`actor::thread`）へ
//! pin された。`debug::enable` は VM 構築の一部として `PastaShiori::load` 内で呼ばれ、
//! その `load` は **アクタースレッド上**で走る（`spawn_actor_thread` の future）。
//! したがって `set_global_hook(EVERY_LINE)` はアクタースレッドで装着・発火する。
//! 本テストはそれを **end-to-end で証明**する:
//!
//! 1. デバッグ有効フィクスチャ（`[debug] enabled=true port=0`・エフェメラルポート）を
//!    アクタースレッドでロードし、束縛された DAP ポートを **アクタースレッドから**
//!    読み戻す（`ActorThread::debug_local_addr()`）。`Some(addr)` であること自体が、
//!    `enable`→`Transport::start` がアクタースレッドで成立した証跡（R10.2 attach 前提）。
//! 2. その DAP ポートへ実 TCP で **attach**（`initialize`→`initialized`）— VSCode attach
//!    の自動証跡（R10.2）。
//! 3. scene ハンドラの `.lua` 行へ行ブレークポイントを張り、mailbox 経由で GET を投入。
//!    GET 応答はブレークポイント停止中はブロックし、`continue` 後にのみ到着する
//!    （＝アクタースレッドがフックにより genuinely 停止した・R10.1）。`stopped` 観測 →
//!    `stackTrace`/`scopes`/`variables` でローカル `debug_local`（=4242）を inspect
//!    （R10.1 変数 inspect・R10.3 source-map された VM 内フローに停止が留まる）。
//! 4. `continue` で流し切り、GET 応答が期待値で到着することを確認。
//!
//! # フェイク不能性（テストの「歯」）
//! - `debug_local_addr()` が `None` ならフックがアクタースレッドで装着されていない →
//!   step1 で失敗。
//! - GET 応答がブレークポイント前に到着するならフックは VM を停止していない → step3 の
//!   順序アサートで失敗。
//! - `debug_local` が読めない／値が違うなら inspect 経路が壊れている → step3 で失敗。
//!
//! # ポート枯渇対策
//! `port = 0`（OS 割当エフェメラル）。固定ポート 9276 を一切使わないため、開発セッションの
//! ambient `PASTA_DEBUG` も `#[ctor]` で中和しつつ、CI で AddrInUse になり得ない。
//! 全クライアント待機は TEST-ONLY watchdog で有界化し CI をハングさせない。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use pasta::actor::mailbox::{mailbox, ActorMsg, MailboxRequest, Reply};
use pasta::actor::thread::spawn_actor_thread;
use serde_json::{json, Value};
use tempfile::TempDir;

/// TEST-ONLY watchdog（停止コアは無期限・本クライアント待機のみ有界化）。
const WATCHDOG: Duration = Duration::from_secs(20);

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和する。
/// 本フィクスチャは pasta.toml `[debug]` で明示的にデバッグを有効化し、ポートは 0
/// （エフェメラル）なので、ambient env が `enabled`/`port` を上書きしないようにする。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// デバッグ保全フィクスチャ（本番 pasta_scripts + `[debug]` 有効 entry.lua override）を
/// temp へ展開し、`load_dir` を返す。`TempDir` は返り値で寿命保持する。
/// 写経元: `actor_thread_vm_test.rs::build_async_callback_dir`。
fn build_debug_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let pasta_scripts_src = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_lua")
        .join("pasta_scripts");
    let pasta_scripts_dst = temp.path().join("pasta_scripts");
    std::fs::create_dir_all(&pasta_scripts_dst).expect("create pasta_scripts dir");
    copy_dir_recursive(&pasta_scripts_src, &pasta_scripts_dst).expect("copy pasta_scripts");

    let fixture_src = manifest_dir.join("tests/fixtures/debug_preservation");
    copy_dir_recursive(&fixture_src, temp.path()).expect("copy fixture");

    (temp.path().to_path_buf(), temp)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// SHIORI/3.0 リクエストを正規化（改行→CRLF・終端付与）する。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

// ===========================================================================
// 最小 DAP-over-TCP クライアント（Content-Length フレーミング）。
// pasta_lua の `transport::{read_frame, write_frame}` は `pub(crate)` で外部から
// 使えないため、DAP の `Content-Length: <N>\r\n\r\n<json>` フレーミングを自前実装する。
// ===========================================================================
struct DapClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl DapClient {
    fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).expect("client must connect to the bound DAP port");
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
        let body = serde_json::to_vec(&req).expect("serialize DAP request");
        write!(self.writer, "Content-Length: {}\r\n\r\n", body.len())
            .expect("write DAP header");
        self.writer.write_all(&body).expect("write DAP body");
        self.writer.flush().expect("flush DAP frame");
    }

    /// 1 フレーム読み取り（`Content-Length` ヘッダ → ボディ）。
    fn recv(&mut self) -> Value {
        // ヘッダ読み取り（空行まで）。
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = self
                .reader
                .read_line(&mut line)
                .expect("read DAP header line (TEST-ONLY timeout)");
            assert_ne!(n, 0, "peer closed before a full DAP frame header");
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break; // ヘッダ終端。
            }
            if let Some(v) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(v.trim().parse().expect("Content-Length is a number"));
            }
        }
        let len = content_length.expect("DAP frame must carry Content-Length");
        let mut body = vec![0u8; len];
        self.reader
            .read_exact(&mut body)
            .expect("read DAP body of declared length");
        serde_json::from_slice(&body).expect("DAP body is valid JSON")
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

fn is_event(msg: &Value, name: &str) -> bool {
    msg["type"] == "event" && msg["event"] == name
}

fn is_response(msg: &Value, command: &str) -> bool {
    msg["type"] == "response" && msg["command"] == command
}

/// デバッグ保全 E2E 本体。VM がアクタースレッドへ pin された後も DAP デバッグが
/// 完全機能すること（フックのアクタースレッド発火・行 BP 停止・変数 inspect・
/// VSCode attach）を 1 本の実 DAP-over-TCP セッションで end-to-end 検証する。
#[test]
fn debug_backend_fires_on_actor_thread_with_bp_inspect_and_attach() {
    let (load_dir, _temp) = build_debug_dir();

    // (1) アクタースレッドで VM をロード（debug 有効・エフェメラルポート）。
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir.clone(), rx);
    assert!(actor.loaded(), "actor thread must load the debug-enabled ghost VM");
    assert_ne!(
        actor.actor_thread_id(),
        thread::current().id(),
        "VM must run on a dedicated actor thread, not the SHIORI/test thread"
    );

    // 束縛された DAP ポートをアクタースレッドから読み戻す。`Some` であること自体が
    // `enable`→`set_global_hook`→`Transport::start` がアクタースレッドで成立した証跡
    // （R10.1/R10.2: フックは VM スレッド＝アクタースレッドで装着される）。
    let dap_addr = actor
        .debug_local_addr()
        .expect("debug backend must bind a DAP port ON THE ACTOR THREAD (R10.1/R10.2)");

    // (2) DAP attach（VSCode attach の自動証跡・R10.2）。
    let mut client = DapClient::connect(dap_addr);
    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let init_resp = client.recv_until(|m| is_response(m, "initialize"));
    assert_eq!(init_resp["success"], true, "DAP initialize must succeed (attach)");
    let _initialized = client.recv_until(|m| is_event(m, "initialized"));

    // (3) scene ハンドラの `.lua` 行へ BP を張る。chunk 名は `require` 経由でロードされた
    //     entry.lua の絶対パス（`canonicalize_chunk_name` で正規化突合される）。
    //     `debug_local` を使用する行（act:raw_script(...)）に張ると、停止時に
    //     `debug_local` がスコープ内（=4242）で inspect できる。
    let entry_path = load_dir
        .join("scripts/pasta/shiori/entry.lua")
        .to_string_lossy()
        .to_string();
    const BREAKPOINT_LINE: u64 = 52; // `local marker = debug_local + 1`
                                     // （`debug_local` は line 51 で代入済み → 停止時に named local として inspect 可能）
    client.send_request(
        2,
        "setBreakpoints",
        json!({
            "source": { "path": entry_path },
            "breakpoints": [{ "line": BREAKPOINT_LINE }],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let bps = bp_resp["body"]["breakpoints"].as_array().expect("breakpoints array");
    assert_eq!(bps.len(), 1, "one breakpoint response");
    assert_eq!(
        bps[0]["verified"], true,
        "the scene `.lua` line BP must verify against the actor-thread VM chunk"
    );

    client.send_request(3, "configurationDone", json!({}));
    let cfg_resp = client.recv_until(|m| is_response(m, "configurationDone"));
    assert_eq!(cfg_resp["success"], true);

    // GET を mailbox へ投入（応答経路は reply_rx）。ブレークポイント停止中は VM が
    // アクタースレッドで止まるため、reply はまだ到着しない。
    let (get_msg, reply_rx) =
        ActorMsg::get(MailboxRequest::new(1, normalize_request(
            "GET SHIORI/3.0\nCharset: UTF-8\nID: OnDebugScene\n",
        )));
    tx.send(get_msg).expect("send GET into mailbox");

    // (3a) フックが BP 行でアクタースレッドを停止 → `stopped(breakpoint)`。
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped["body"]["reason"], "breakpoint",
        "R10.1: the actor-thread line hook must pause the VM at the `.lua` breakpoint"
    );

    // テストの「歯」: 停止中は GET 応答がまだ到着していない（アクタースレッドが
    // genuinely 停止している）。短い有界待ちで「来ていない」ことを確認する。
    assert!(
        reply_rx.recv_timeout(Duration::from_millis(300)).is_err(),
        "R10.1: the GET reply must NOT arrive while the VM is paused at the breakpoint \
         (proves the actor thread is genuinely stopped by the hook)"
    );

    let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

    // (3b) stackTrace → 停止フレーム（scene コルーチン本体）。
    client.send_request(10, "stackTrace", json!({ "threadId": thread_id }));
    let stack = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"].as_array().expect("stackFrames array");
    assert!(!frames.is_empty(), "stopped frame must exist (R10.1)");
    assert_eq!(
        frames[0]["line"].as_u64().expect("top frame line"),
        BREAKPOINT_LINE,
        "top frame must be the breakpoint line in the scene handler"
    );

    // (3c) scopes/variables → ローカル `debug_local`（=4242）を inspect（R10.1）。
    let frame_id = frames[0]["id"].as_u64().expect("frame id");
    client.send_request(11, "scopes", json!({ "frameId": frame_id }));
    let scopes = client.recv_until(|m| is_response(m, "scopes"));
    assert_eq!(scopes["success"], true, "R10.1: scopes must succeed at the actor-thread stop");
    let scope_arr = scopes["body"]["scopes"].as_array().expect("scopes array");
    assert!(!scope_arr.is_empty(), "stopped frame must expose a scope");
    let var_ref = scope_arr[0]["variablesReference"].as_u64().expect("variablesReference");

    client.send_request(12, "variables", json!({ "variablesReference": var_ref }));
    let vars = client.recv_until(|m| is_response(m, "variables"));
    let var_arr = vars["body"]["variables"].as_array().expect("variables array");
    let debug_local = var_arr
        .iter()
        .find(|v| v["name"] == "debug_local")
        .unwrap_or_else(|| panic!("R10.1: coroutine local `debug_local` must be inspectable: {var_arr:?}"));
    assert_eq!(
        debug_local["value"], "4242",
        "R10.1: variable inspection through the actor-hosted VM must read the live value"
    );

    // (4) continue で流し切り、GET 応答が期待値で到着することを確認。BP 行は
    //     複数回到達し得る（行フックの再入）ため、stopped が来る限り continue を繰り返す。
    for continue_seq in 30u64..90u64 {
        client.send_request(continue_seq, "continue", json!({ "threadId": thread_id }));
        if let Ok(reply) = reply_rx.recv_timeout(Duration::from_millis(500)) {
            match reply {
                Reply::Value(v) => {
                    assert!(
                        v.contains("debug=4242"),
                        "GET reply (after continue) must carry the actor-VM scene output: {v:?}"
                    );
                }
            }
            // 応答到着＝scene 完走。clean stop へ進む。
            let (stop, done_rx) = ActorMsg::stop();
            tx.send(stop).expect("send Stop into mailbox");
            done_rx
                .recv_timeout(WATCHDOG)
                .expect("Stop must be acked after the debug session completes");
            actor.join().expect("actor thread must join cleanly after Stop");
            return;
        }
        // 応答未到着 → BP 行へ再停止しているはず。stopped を消化してもう一度 continue。
        let _ = client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));
    }
    panic!("GET reply did not arrive after draining continue iterations (hang or missed stop?)");
}
