//! Task 4.2 — runtime VM 初期化への debug バックエンド統合と runtime スコープ永続。
//!
//! 検証する観測可能な「done」:
//! 1. **有効ランタイム**: フックを一度だけ設置し、ブレークポイント/セッション状態が
//!    複数スクリプト実行（＝複数 SHIORI リクエスト）を跨いで **永続** する
//!    （BP を設定 → script #1 でヒット → continue → script #2 で再びヒット）。
//! 2. **無効ランタイム（既定）**: フック非設置・ポート非開放・`std_debug` 非露出で、
//!    既存挙動と同一（ゼロコスト）。
//! 3. **terminated セマンティクス**: per-request の「実行終了」では terminated を出さず
//!    （デバッグ対象は長命ランタイム）、runtime teardown（`DebugHandle` の Drop）で
//!    接続中クライアントへ `terminated` を送出する。
//!
//! `mlua::Lua` は `!Send` なので VM はランタイム所有スレッド（このテストスレッド）に
//! 固定される。DAP クライアントは別スレッドで駆動し、チャネル/バウンド addr のみ越境する。
//! 全クライアント待機は TEST-ONLY watchdog でバウンドし CI がハングしないようにする。

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::{Value, json};

use pasta_lua::{DebugConfig, PastaLuaRuntime, RuntimeConfig, TranspileContext};

/// Write one DAP `Content-Length`-framed JSON message (TEST-LOCAL framing — the
/// production `read_frame`/`write_frame` are crate-private; this mirrors them).
fn write_frame<W: Write>(out: &mut W, value: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(value)?;
    write!(out, "Content-Length: {}\r\n\r\n", body.len())?;
    out.write_all(&body)?;
    out.flush()
}

/// Read one DAP `Content-Length`-framed JSON message (TEST-LOCAL framing).
/// Returns `Ok(None)` on a clean EOF before any header.
fn read_frame<R: BufRead>(reader: &mut R) -> std::io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None); // EOF before a complete header block.
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break; // blank line → end of headers.
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

/// TEST-ONLY watchdog。停止コア自体は無期限。
const WATCHDOG: Duration = Duration::from_secs(15);

/// 永続テストのブレークポイント対象 `.lua` source 名と行。
const PERSIST_SOURCE: &str = "@persist_scenario";
const PERSIST_BP_LINE: u32 = 2;

/// シナリオ chunk（1-origin 行）。BP は 2 行目（`local b = a + 1`）に置く。
///   1: local a = 1
///   2: local b = a + 1   <- BREAKPOINT
///   3: return b
const PERSIST_CHUNK: &str = "\
local a = 1
local b = a + 1
return b
";

/// 実 TCP ソケット越しの最小 DAP クライアント（Content-Length フレーミング）。
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

    fn recv(&mut self) -> Option<Value> {
        read_frame(&mut self.reader)
            .expect("client read must succeed (TEST-ONLY timeout)")
    }

    fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
        loop {
            let msg = self.recv().expect("a frame must be present (peer did not close)");
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

/// (1) 有効ランタイム: フックは一度だけ設置され、BP/セッション状態が複数の
/// スクリプト実行（リクエスト）を跨いで永続する。
///
/// BP を設定 → exec #1 でヒット（stopped）→ continue → exec #2 で **同じ BP に再びヒット**。
/// これが永続（debug 状態が per-request スコープではなく runtime スコープにある）の証明。
#[test]
fn enabled_runtime_persists_breakpoint_across_requests() {
    // ランタイム所有スレッド: enabled な RuntimeConfig でランタイムを構築し
    // （フックが VM 初期化時に一度だけ設置される）、バウンド addr を発行、
    // クライアントの go 後に PERSIST_CHUNK を 2 回 exec する。
    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (go_tx, go_rx) = mpsc::channel::<()>();

    let host = std::thread::spawn(move || -> Result<(), String> {
        let debug_cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            ..Default::default()
        };
        let config = RuntimeConfig::minimal().with_debug(debug_cfg);
        let runtime = PastaLuaRuntime::with_config(TranspileContext::new(), config)
            .map_err(|e| format!("runtime build failed: {e}"))?;

        // 有効ランタイムは enable を一度だけ呼び DebugHandle を保持している。
        let addr = runtime
            .debug_local_addr()
            .ok_or_else(|| "enabled runtime must expose a bound debug addr".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        // クライアントが setBreakpoints/configurationDone を終えるまで待つ。
        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "no go signal before request #1".to_string())?;

        // リクエスト #1: BP ヒット → クライアントが continue するまでブロック。
        // set_name 付き chunk で実行（BP の source 一致が必要）。
        runtime
            .exec_named(PERSIST_CHUNK, PERSIST_SOURCE)
            .map_err(|e| format!("request #1 exec failed: {e}"))?;

        // リクエスト #1 完了をクライアントへ知らせ、#2 の go を待つ。
        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "no go signal before request #2".to_string())?;

        // リクエスト #2: BP は **まだ有効**（runtime スコープ永続）。再びヒットする。
        runtime
            .exec_named(PERSIST_CHUNK, PERSIST_SOURCE)
            .map_err(|e| format!("request #2 exec failed: {e}"))?;

        // teardown（runtime drop → DebugHandle drop → terminated）。
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

    // BP 設定（永続対象 source・行）。
    client.send_request(
        2,
        "setBreakpoints",
        json!({
            "source": { "path": PERSIST_SOURCE },
            "breakpoints": [{ "line": PERSIST_BP_LINE }],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    assert_eq!(bp_resp["body"]["breakpoints"][0]["verified"], true);

    client.send_request(3, "configurationDone", json!({}));
    let _ = client.recv_until(|m| is_response(m, "configurationDone"));

    // リクエスト #1 を開始。
    go_tx.send(()).expect("go #1");

    // リクエスト #1: BP ヒット。
    let stopped1 = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped1["body"]["reason"], "breakpoint",
        "request #1 must hit the persisted breakpoint"
    );
    let thread_id = stopped1["body"]["threadId"].as_u64().unwrap_or(1);
    // continue で #1 を流し切る。
    client.send_request(10, "continue", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "continue"));

    // リクエスト #2 を開始（BP を再設定しない — 永続を証明する）。
    go_tx.send(()).expect("go #2");

    // リクエスト #2: 同じ BP に **再び** ヒット（永続の証明）。
    let stopped2 = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped2["body"]["reason"], "breakpoint",
        "request #2 must hit the SAME breakpoint without re-setting it (runtime-scope persistence)"
    );
    client.send_request(20, "continue", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "continue"));

    // host スレッドが teardown まで到達したことを watchdog 内で確認。
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(host.join());
    });
    match done_rx.recv_timeout(WATCHDOG) {
        Ok(joined) => {
            joined
                .expect("host thread must not panic")
                .expect("both requests must run to completion with persisted BP");
        }
        Err(RecvTimeoutError::Timeout) => panic!("host thread did not finish (hang?)"),
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }
}

/// (3) terminated-on-teardown: クライアント接続中に runtime（＝DebugHandle）が
/// drop されると `terminated` が送出される。per-request の exec 復帰では terminated を
/// 出さない（デバッグ対象は長命ランタイム）。
#[test]
fn runtime_teardown_emits_terminated_to_connected_client() {
    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (drop_tx, drop_rx) = mpsc::channel::<()>();

    let host = std::thread::spawn(move || -> Result<(), String> {
        let debug_cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            ..Default::default()
        };
        let config = RuntimeConfig::minimal().with_debug(debug_cfg);
        let runtime = PastaLuaRuntime::with_config(TranspileContext::new(), config)
            .map_err(|e| format!("runtime build failed: {e}"))?;
        let addr = runtime
            .debug_local_addr()
            .ok_or_else(|| "enabled runtime must expose a bound debug addr".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        // クライアントが接続・ハンドシェイクするまで待ち、その後 runtime を drop。
        drop_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "no drop signal".to_string())?;

        // per-request 終了（exec 復帰）では terminated を出さないことを示すため、
        // 軽い exec を 1 回行ってから drop する（この exec 復帰では terminated は来ない）。
        let _ = runtime.exec("return 1");

        drop(runtime); // teardown → terminated 送出
        Ok(())
    });

    let addr = addr_rx.recv_timeout(WATCHDOG).expect("bound addr");
    let mut client = DapClient::connect(addr);

    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    // runtime を drop させる。
    drop_tx.send(()).expect("drop signal");

    // teardown により `terminated` イベントが到達する。
    let terminated = client.recv_until(|m| is_event(m, "terminated"));
    assert!(
        is_event(&terminated, "terminated"),
        "runtime teardown must emit a DAP terminated event to the connected client"
    );

    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(host.join());
    });
    match done_rx.recv_timeout(WATCHDOG) {
        Ok(joined) => {
            joined.expect("host thread must not panic").expect("teardown ok");
        }
        Err(_) => panic!("host thread did not finish (hang?)"),
    }
}

/// (2) 無効ランタイム（既定）: フック非設置・ポート非開放・`std_debug` 非露出。
/// 既存挙動と同一（ゼロコスト）。
#[test]
fn disabled_runtime_is_zero_cost() {
    // 既定の RuntimeConfig::new() / minimal() は debug 無効。
    let runtime =
        PastaLuaRuntime::with_config(TranspileContext::new(), RuntimeConfig::minimal()).unwrap();

    // ポート非開放: debug_local_addr は None。
    assert!(
        runtime.debug_local_addr().is_none(),
        "disabled runtime must NOT open a debug port"
    );

    // std_debug 非露出: minimal は ALL_SAFE 相当で debug グローバルが nil。
    let debug_is_nil: bool = runtime
        .exec("return debug == nil")
        .expect("eval ok")
        .as_boolean()
        .expect("boolean");
    assert!(debug_is_nil, "disabled runtime must NOT expose std_debug");

    // フック非設置: jit は ON のまま（enable はフック内で jit.off() するため、
    // 無効時は JIT が有効＝フック非設置の痕跡）。
    let jit_on: bool = runtime
        .exec("return jit ~= nil and jit.status() == true")
        .expect("eval ok")
        .as_boolean()
        .expect("boolean");
    assert!(
        jit_on,
        "disabled runtime must NOT install the hook (jit stays ON, no jit.off())"
    );
}

/// 接続クライアント無しの debug-enabled ランタイムを `addr` 指定で構築するヘルパー。
/// `listen` に明示アドレス（OS 割当 `:0` か捕捉済み固定ポート）を渡す。
///
/// 明示 `DebugConfig`（`from_env` ではない）なので、環境変数 `PASTA_DEBUG=1` の有無は
/// この経路に影響しない（既存テスト群と同じ理由で `#[ctor]` ガード不要）。
fn build_debug_runtime(listen: SocketAddr) -> PastaLuaRuntime {
    let debug_cfg = DebugConfig {
        enabled: true,
        listen: Some(listen),
        ..Default::default()
    };
    let config = RuntimeConfig::minimal().with_debug(debug_cfg);
    PastaLuaRuntime::with_config(TranspileContext::new(), config)
        .expect("debug-enabled runtime must build (bind must succeed)")
}

/// debug-enabled ランタイムを構築し、bound addr を読み出し、即 drop して
/// OS 割当ポート P を捕捉するヘルパー（`reload_*` テストで固定ポートとして再利用）。
///
/// 同期 teardown（`DebugHandle::drop` → bridge join → `Transport::drop` → serve join）
/// により drop 完了時点で P は解放済み。返した P を次の build で再 bind する。
fn capture_os_assigned_port() -> u16 {
    let runtime = build_debug_runtime("127.0.0.1:0".parse().unwrap());
    let port = runtime
        .debug_local_addr()
        .expect("enabled runtime must expose a bound debug addr")
        .port();
    assert_ne!(port, 0, "OS must assign a concrete port");
    drop(runtime); // 同期 teardown → P 解放
    port
}

/// runtime drop（同期 teardown）を TEST-ONLY watchdog でバウンドして実行する。
/// drop が（リスナースレッドの leak / hang で）戻らない回帰は、スイート全体が
/// 無期限にハングする代わりに fast-fail させる。
///
/// `mlua::Lua` は `!Send` なので runtime 自体を別スレッドへ move して join できない。
/// 代わりに番兵スレッドを spawn し、本スレッドが `drop` 後に done を送る。drop が
/// 戻れば done が届き番兵は静かに終わる。drop が hang すると done は決して届かず、
/// 番兵は watchdog 期間後に `process::abort()` でプロセスを落とす（CI が無期限に
/// 詰まる代わりに即座に失敗する）。`Drop` がブロックする本スレッドからは panic できない
/// ため、ハング検出はこの out-of-thread abort が担う。production teardman は watchdog 非依存。
fn drop_runtime_with_watchdog(runtime: PastaLuaRuntime) {
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let sentinel = std::thread::spawn(move || match done_rx.recv_timeout(WATCHDOG) {
        Ok(()) => {} // drop returned in time → wind down quietly.
        Err(RecvTimeoutError::Timeout) => {
            eprintln!("runtime drop did not return within the watchdog (teardown hang?)");
            std::process::abort();
        }
        Err(RecvTimeoutError::Disconnected) => {} // sender dropped without send → see below.
    });

    // 同期 drop を本スレッドで実行（`!Send` のため）。hang すればここで止まり、
    // 番兵が watchdog 後に abort する。戻れば done を送って番兵を解放する。
    drop(runtime);
    done_tx.send(()).expect("sentinel must still be alive to receive done");
    sentinel.join().expect("watchdog sentinel must not panic");
}

/// (4.2-1) `reload_rebinds_same_port_no_client`（R1.1 / R2.4）— 真因の no-client 経路。
///
/// debug-enabled ランタイムを `127.0.0.1:0` で構築し OS 割当ポート P を捕捉 → `drop` →
/// 同一固定ポート P で SECOND ランタイムを構築し、それが SUCCEED し P に bound すること
/// を assert する。
///
/// 修正前はここで 2 回目の `enable`→`Transport::start` が `DebugError::Bind`
/// （`WSAEADDRINUSE` / `os error 10048`）で失敗していた — teardown が serve リスナー
/// スレッドを停止・join せずデタッチしたため、leak したスレッドが固定ポート P を
/// プロセス寿命いっぱい握り続けたから。修正後は同期 join で P が解放され再 bind が成功する。
/// teardown はバウンド済み watchdog で囲み、hang を fast-fail させる。
#[test]
fn reload_rebinds_same_port_no_client() {
    // FIRST ランタイム: OS に :0 でポートを割り当てさせ、その P を捕捉する。
    let first = build_debug_runtime("127.0.0.1:0".parse().unwrap());
    let addr = first
        .debug_local_addr()
        .expect("first enabled runtime must expose a bound debug addr (R3.1)");
    let port = addr.port();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_ne!(port, 0, "OS must assign a concrete port");

    // unload: 同期 teardown が serve リスナーを join して P を解放する（修正の核心）。
    // 修正前はここでスレッドが leak し P を握り続けた。
    drop_runtime_with_watchdog(first);

    // reload: SAME 固定ポート P で SECOND ランタイムを構築。修正前は 10048 で失敗。
    let fixed: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let second = build_debug_runtime(fixed);
    let rebound = second
        .debug_local_addr()
        .expect("reloaded runtime must rebind the SAME fixed port (R1.1 — no 10048)");
    assert_eq!(
        rebound.port(),
        port,
        "reload must rebind the SAME OS-assigned port P (R2.4: re-start after teardown)"
    );

    drop_runtime_with_watchdog(second);
}

/// (4.2-2) `reload_multiple_cycles_each_succeed`（R1.3）— 連続複数回 reload。
///
/// throwaway `:0` ランタイムで固定ポート P を捕捉し、`{build P → bound 確認 → drop}` の
/// サイクルを 3 回以上連続実行する。各サイクルが成功することが、同一プロセス内の反復
/// reload が毎回 P を再 bind できる証明になる。真因は live-thread leak（TIME_WAIT ではない）
/// なので、実 15 秒待機は不要・タイミング非依存で再現できる。各 teardown は watchdog で囲む。
#[test]
fn reload_multiple_cycles_each_succeed() {
    let port = capture_os_assigned_port();
    let fixed: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    // >=3 サイクル: 各回 SAME 固定ポート P で build → bound 確認 → 同期 drop。
    // 前サイクルのリスナーが leak していれば次の build が 10048 で panic する。
    for cycle in 0..3 {
        let runtime = build_debug_runtime(fixed);
        let bound = runtime.debug_local_addr().unwrap_or_else(|| {
            panic!("cycle {cycle}: reloaded runtime must expose a bound addr on fixed port {port}")
        });
        assert_eq!(
            bound.port(),
            port,
            "cycle {cycle}: must rebind the SAME fixed port P (R1.3 repeated reload)"
        );
        // 同期 teardown が P を解放してから次サイクルへ。hang は watchdog で fast-fail。
        drop_runtime_with_watchdog(runtime);
    }
}

/// (4.2-3) `reload_with_connected_client_rebinds`（R2.5 / R3.2）— 接続クライアントを伴う reload。
///
/// debug-enabled ランタイムを `:0` で構築し P を捕捉、実 `DapClient` を接続して initialize
/// ハンドシェイクを行い（本物のクライアント接続を成立させる）、その状態で `drop(runtime)`
/// する（接続中クライアントを伴う teardown → 接続ソケット + reader の同期解放, R2.5）。
/// その後 SAME ポート P で SECOND ランタイムを構築し、再 bind 成功を assert する。
///
/// これは接続クライアント teardown（R2.5）と `SO_REUSEADDR` 防御層（R3.2）の両方を行使する:
/// 直前に閉じた accepted 接続が P 上に残存 TIME_WAIT を残しうるが、新リスナーはそれでも
/// 再 bind できねばならない。全クライアント/host 待機は watchdog でバウンドする。
#[test]
fn reload_with_connected_client_rebinds() {
    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (drop_tx, drop_rx) = mpsc::channel::<()>();
    let (port_tx, port_rx) = mpsc::channel::<u16>();

    // FIRST ランタイムはホストスレッドが所有（`mlua::Lua` は `!Send`）。接続クライアントの
    // initialize 完了後に drop し、解放した P を本スレッドへ返す。
    let host = std::thread::spawn(move || -> Result<(), String> {
        let first = build_debug_runtime("127.0.0.1:0".parse().unwrap());
        let addr = first
            .debug_local_addr()
            .ok_or_else(|| "first enabled runtime must expose a bound debug addr".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        // クライアントが接続・ハンドシェイクするまで待ち、その後 first を drop。
        drop_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "no drop signal before teardown".to_string())?;

        // 接続中クライアントを伴う teardown（R2.5）: 接続ソケット + reader を同期解放し、
        // serve リスナーを join して P を解放する。
        drop(first);

        // 解放した P を reload 側へ通知。
        port_tx
            .send(addr.port())
            .map_err(|_| "port send failed".to_string())?;
        Ok(())
    });

    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

    // 本物のクライアント接続を成立させる（initialize ハンドシェイク）。
    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    // 接続中クライアントを保持したまま host に first を drop させる（R2.5 teardown 経路）。
    drop_tx.send(()).expect("drop signal");

    // host が teardown を完了し P を返すまで待つ（watchdog バウンド）。
    let port = port_rx
        .recv_timeout(WATCHDOG)
        .expect("host must release the connected-client runtime and publish P (R2.5 teardown)");
    assert_eq!(port, addr.port(), "released port must be the captured P");

    // client は明示 drop（accepted 接続のクローズ → P 上に TIME_WAIT 残存しうる）。
    drop(client);

    // host スレッドが panic せず正常完了したことを watchdog 内で確認。
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(host.join());
    });
    match done_rx.recv_timeout(WATCHDOG) {
        Ok(joined) => {
            joined
                .expect("host thread must not panic")
                .expect("connected-client teardown must complete (R2.5)");
        }
        Err(RecvTimeoutError::Timeout) => panic!("host thread did not finish (teardown hang?)"),
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }

    // reload: SAME ポート P で SECOND ランタイムを構築。R3.2: 直前に閉じた accepted 接続が
    // 残す残存 TIME_WAIT があっても、`SO_REUSEADDR` 防御層により再 bind は成功せねばならない。
    let fixed: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let second = build_debug_runtime(fixed);
    let rebound = second
        .debug_local_addr()
        .expect("reloaded runtime must rebind P after a connected-client teardown (R2.5 + R3.2)");
    assert_eq!(
        rebound.port(),
        port,
        "reload after connected-client teardown must rebind the SAME port P (R2.5 + R3.2)"
    );

    drop_runtime_with_watchdog(second);
}

/// 既定の `RuntimeConfig` は debug 無効（ゼロコスト不変条件）。
#[test]
fn default_runtime_config_debug_is_disabled() {
    assert!(!RuntimeConfig::new().debug.enabled, "new() debug disabled");
    assert!(!RuntimeConfig::minimal().debug.enabled, "minimal() debug disabled");
    assert!(!RuntimeConfig::full().debug.enabled, "full() debug disabled");
    assert!(!RuntimeConfig::default().debug.enabled, "default() debug disabled");
}
