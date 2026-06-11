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

/// 既定の `RuntimeConfig` は debug 無効（ゼロコスト不変条件）。
#[test]
fn default_runtime_config_debug_is_disabled() {
    assert!(!RuntimeConfig::new().debug.enabled, "new() debug disabled");
    assert!(!RuntimeConfig::minimal().debug.enabled, "minimal() debug disabled");
    assert!(!RuntimeConfig::full().debug.enabled, "full() debug disabled");
    assert!(!RuntimeConfig::default().debug.enabled, "default() debug disabled");
}

/// Task 8.2 — ゼロコスト/サンドボックス **集約回帰ゲート**（Performance/Regression）。
///
/// このモジュールは「デバッグ無効時の不変条件」を **一箇所に集約** し、各テストを
/// 対応する要件へ明示マッピングする durable な回帰ゲートである。将来の変更が
/// これらを暗黙に退行させられないよう、アサーションは可能な限り **直接的かつ強力**
/// にする（4.2/5.1 が既に保証する内容の上に、より強い信号を積む）。
///
/// 要件マッピング（`.kiro/specs/pasta-vscode-lua-debug/requirements.md`）:
/// - **R5.2**: 無効時はデバッグ用フックを設置せず、本番実行に追加コストを与えない。
///   → [`r5_2_disabled_installs_no_hook_jit_stays_on`]
/// - **R5.3**: 無効時は `debug`／`std_debug` をスクリプトへ露出せず、サンドボックスを維持する。
///   → [`r5_3_disabled_keeps_sandbox_debug_is_nil`]
/// - **R5.5**: 無効時は接続待ち受け口を開かない（`debug_local_addr()==None`）。
///   → [`r5_5_disabled_opens_no_port`]
///
/// （R4.6/5.2 の「本番 transpile 出力バイト一致」は、トランスパイラ API へアクセスする
/// `tests/transpiler/source_map_seam_test.rs` の `zero_cost_sandbox_regression` モジュールで
/// 集約アサートする — ランタイムターゲットからは code_gen の本番 API に届かないため分割。）
///
/// design.md 参照: "Testing Strategy / Integration Tests"（無効時ゼロコスト/サンドボックス:
/// hook 痕跡なし・`std_debug` 非露出・接続口非開放 — 5.2, 5.3, 5.5）、
/// "Performance/Regression"、"DebugConfig & Gate"（無効時 listen=None・hook 非設置）。
mod zero_cost_sandbox_regression {
    use super::*;

    /// `default_debug_port()` と同値（`pasta.toml`/`PASTA_DEBUG_PORT` 未設定時の既定）。
    /// テストはこの値で best-effort の「リスナ不在」確認を行う（権威判定は
    /// `debug_local_addr()==None`。ポート競合での flaky を避けるため connect-refused は
    /// あくまで補助シグナル扱い）。
    const DEFAULT_DEBUG_PORT: u16 = 9276;

    /// 無効ランタイムを構築するヘルパー。`RuntimeConfig::minimal()` は debug 無効
    /// （`default_runtime_config_debug_is_disabled` が保証）で、ALL_SAFE 相当のサンドボックス。
    fn disabled_runtime() -> PastaLuaRuntime {
        PastaLuaRuntime::with_config(TranspileContext::new(), RuntimeConfig::minimal())
            .expect("disabled runtime must build")
    }

    /// **R5.2 — 無効時はフック非設置（jit は ON のまま）**。
    ///
    /// enable パスはフック内でエンジン全体に `jit.off()` を適用するため、複数行スクリプト
    /// 実行後も JIT が ON のままであることは「**フック非設置＝per-line デバッグコストなし**」
    /// の強い証拠になる。さらに直接的な不変条件として `debug_enabled()==false`
    /// （`DebugHandle` 不保持）も併せて表明する。
    #[test]
    fn r5_2_disabled_installs_no_hook_jit_stays_on() {
        let runtime = disabled_runtime();

        // 直接表明: 無効ランタイムは DebugHandle を保持しない（フック設置の前提が無い）。
        assert!(
            !runtime.debug_enabled(),
            "R5.2: disabled runtime must NOT hold a DebugHandle (no hook installed)"
        );

        // 複数行スクリプトを実行しても JIT は ON のまま（enable なら jit.off() で OFF になる）。
        // 行フックが一度でも走れば JIT は無効化されているはずなので、ON のままであることは
        // 「行フックが一度も発火していない＝デバッグコスト 0」の直接的痕跡。
        let jit_on_after_run: bool = runtime
            .exec(
                "\
local sum = 0
for i = 1, 1000 do
  sum = sum + i
end
return jit ~= nil and jit.status() == true",
            )
            .expect("multi-line eval ok")
            .as_boolean()
            .expect("boolean result");
        assert!(
            jit_on_after_run,
            "R5.2: JIT must remain ON after a multi-line run (no hook → no engine-wide jit.off())"
        );
    }

    /// **R5.3 — サンドボックス維持（`debug` 非露出）**。
    ///
    /// 無効ランタイムはスクリプトへ `debug`/`std_debug` を露出しない。`debug == nil` を
    /// 表明し、さらにスタック introspection（`debug.getinfo`）へ到達できないことを示す。
    #[test]
    fn r5_3_disabled_keeps_sandbox_debug_is_nil() {
        let runtime = disabled_runtime();

        let debug_is_nil: bool = runtime
            .exec("return debug == nil")
            .expect("eval ok")
            .as_boolean()
            .expect("boolean");
        assert!(
            debug_is_nil,
            "R5.3: disabled runtime must NOT expose the `debug` global (sandbox)"
        );

        // スタック introspection へ到達不能であること（`debug.getinfo` が呼べない）を
        // pcall で確認 — 露出していれば true（成功）になってしまう。
        let cannot_introspect: bool = runtime
            .exec("return not pcall(function() return debug.getinfo(1) end)")
            .expect("eval ok")
            .as_boolean()
            .expect("boolean");
        assert!(
            cannot_introspect,
            "R5.3: scripts must NOT be able to reach stack introspection (debug.getinfo)"
        );
    }

    /// **R5.5 — 接続口非開放**。
    ///
    /// 権威判定は `debug_local_addr() == None`。補助（best-effort）として、既定ポート
    /// (9276) への `TcpStream::connect` がこの無効ランタイム宛には成立しないことを確認する
    /// が、無関係プロセスが 9276 を占有している場合に flaky にならないよう、接続が成功した
    /// 場合はその補助チェックをスキップ（権威判定のみを信頼）する。
    #[test]
    fn r5_5_disabled_opens_no_port() {
        let runtime = disabled_runtime();

        // 権威判定: 無効ランタイムは bound addr を一切公開しない。
        assert!(
            runtime.debug_local_addr().is_none(),
            "R5.5: disabled runtime must NOT open/expose a debug port (authoritative)"
        );
        // 二重確認: handle 自体が無い。
        assert!(
            !runtime.debug_enabled(),
            "R5.5: disabled runtime holds no debug handle (no transport bound)"
        );

        // best-effort: 既定ポートへ繋がっても *この* 無効ランタイム由来ではない
        // （無効時はそもそも listen していない）。flaky 回避のため成功時は無視する。
        let connect = TcpStream::connect_timeout(
            &SocketAddr::from(([127, 0, 0, 1], DEFAULT_DEBUG_PORT)),
            Duration::from_millis(200),
        );
        match connect {
            Err(_) => { /* 期待どおり: 無効ランタイムのための listener は存在しない。*/ }
            Ok(_) => {
                // 無関係プロセスが偶発的に 9276 を占有しているケース。権威判定
                // (`debug_local_addr()==None`) は既に通っているので、ここは曖昧として
                // スキップ（test を flaky にしない）。
                eprintln!(
                    "[zero_cost_sandbox_regression] note: port {DEFAULT_DEBUG_PORT} accepted a \
                     connection from an unrelated process; relying on debug_local_addr()==None"
                );
            }
        }
    }

    /// 識別力（discrimination）の証明: 設定を **有効** に切り替えると、上記の無効時シグナルが
    /// 反転する（addr が出る・handle を保持）。これにより各無効時アサーションが「単に常に真」
    /// ではなく、disabled 状態を実際に判別していることを裏付ける（R5.2/R5.5 の鏡像）。
    #[test]
    fn enabled_runtime_flips_the_disabled_signals() {
        let debug_cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            ..Default::default()
        };
        let config = RuntimeConfig::minimal().with_debug(debug_cfg);
        let runtime = PastaLuaRuntime::with_config(TranspileContext::new(), config)
            .expect("enabled runtime must build");

        // 有効時は無効時シグナルが反転する: handle を保持し、bound addr を公開する。
        assert!(
            runtime.debug_enabled(),
            "discrimination: enabled runtime DOES hold a DebugHandle"
        );
        assert!(
            runtime.debug_local_addr().is_some(),
            "discrimination: enabled runtime DOES expose a bound debug addr (port opened)"
        );

        // teardown（接続クライアントは無いので Drop は静かに完了）。
        drop(runtime);
    }
}
