//! Task 7.3 — **提示モード切替**（`.lua` モード回帰・requirements **6.2**/**9.5**）と
//! **多対多マッピングのエッジケース確定挙動**（requirements **8.1**/**8.2**/**8.3**）の
//! **E2E**。
//!
//! task 7.1 [`super::pasta_bp_e2e`] / 7.2 [`super::pasta_step_e2e`] と同型の
//! DAP-over-TCP ハーネス（実 `Arc<SourceMap>`・実 [`enable`](crate::debug::enable)
//! セッション）を用い、**手組みの集約 [`SourceMap`]**（`ChunkSourceMap::from_forward`・
//! task 5.4 unit test / 7.2 流儀）に **集約行**（複数 `.pasta` 行 → 単一 `.lua` 行・8.1）と
//! **展開行**（単一 `.pasta` 行 → 複数 `.lua` 行・8.2）の双方を仕込む。
//!
//! # 観測する「done」（task 7.3 完了状態）
//!
//! 1. **`.lua` 提示モード回帰（6.2/9.5）**: 実 DAP `attach sourcePresentation="lua"`
//!    （VSCode 等価クライアント経路）で提示モードを `.lua` へ切替えると、BP・停止位置・
//!    コールスタックが **`.lua` 座標**（`.pasta` ではない）で提示され、ステップも
//!    **`.lua` 行単位**になる（[`mode_switch_lua_presents_lua_coords_and_lua_step_granularity_over_tcp`]）。
//!    **歯**: 同一マップ・同一 `.lua` 行を `.pasta` モードで提示すると **`.pasta` 座標**が
//!    出ることを併せて表明し、`.lua` モードのアサートが本物（恒真でない）ことを裏づける。
//! 2. **8.1 集約 → 確定的単一 `.pasta`**: 複数 `.pasta` 行が集約された単一 `.lua` 行で
//!    停止すると、確定的に **単一の** `.pasta` 位置を提示する（last-write-wins・task 3.3）。
//!    セッション提示＋マップ直接（`pasta_for_lua` 反復一致）で確定性を表明
//!    （[`edge_8_1_aggregated_lua_line_presents_deterministic_single_pasta`]）。
//! 3. **8.2 展開 → 同一 `.pasta`**: 単一 `.pasta` 行が展開された複数 `.lua` 行の **いずれ**で
//!    停止しても **同一の** `.pasta` 行を提示する
//!    （[`edge_8_2_expanded_pasta_line_same_pasta_at_every_lua_line`]）。
//! 4. **8.3 提示順安定**: 同一 `.pasta` 位置に対するマッピング（`lua_lines_for_pasta` /
//!    `resolve_pasta_to_lua`）の提示順が反復・複数回構築をまたいで **安定（決定的）**
//!    （[`edge_8_3_presentation_order_is_stable_deterministic`]）。
//!
//! `mlua::Lua`（`!Send`）は VM ホストスレッドにのみ生存し、バウンド `SocketAddr`
//! （`Copy`）と go/done チャネルだけが越境する。全クライアント待機は TEST-ONLY
//! watchdog でバウンドし CI がハングしないようにする（停止コアは無期限）。

use std::collections::BTreeMap;
use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::{Value, json};

use crate::debug::source_map::{ChunkSourceMap, PastaPos, SourceMap};
use crate::debug::transport::{read_frame, write_frame};
use crate::debug::{DebugConfig, SourceMode, enable};

/// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
const WATCHDOG: Duration = Duration::from_secs(15);

/// 生成 `.lua` チャンク名（フック source = `set_name` 値）。map のキーと一致させる。
const EDGE_SOURCE: &str = "@pasta_mode_edge_e2e_scenario";

/// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致させる側）。
/// `.pasta` 拡張子で [`super::is_pasta_source`] の BP 翻訳経路を通す。
const EDGE_PASTA_FILE: &str = "scene_edge.pasta";

/// エッジケースを 1 本で覆う **素の Lua チャンク**。`.pasta` 行は手組みマップ
/// （[`edge_scenario_map`]）が与える。行番号（1-origin）と各行の役割:
///
/// ```text
///   1: local a = 1            -- .pasta 20（集約 `.lua` 行・8.1 / `.lua` モード回帰起点）
///   2: local b = a + 1        -- .pasta 30（展開 `.pasta` 30 の 1 本目・8.2）
///   3: local c = b + 1        -- .pasta 30（展開 `.pasta` 30 の 2 本目・8.2）
///   4: local d = c + 1        -- .pasta 30（展開 `.pasta` 30 の 3 本目・8.2）
///   5: local e = d + 1        -- .pasta 40（展開行の後・異なる `.pasta`）
///   6: return e
/// ```
///
/// - 行1（`.pasta` 20）は **集約行**: マップ上は単一の `PastaPos`（`.pasta` 20）だが、
///   トランスパイル時に複数の `.pasta` 行（例 19/20）が同一 `.lua` 行へ集約された結果を
///   模す。確定挙動（8.1）は「単一の `.pasta` を確定的に提示」であり、`from_forward` の
///   `BTreeMap<lua_line, PastaPos>` が 1 `.lua` 行 → 高々 1 `.pasta` 位置を構造的に担保する。
/// - 行2/3/4（同一 `.pasta` 30）は **展開行**: 単一 `.pasta` 行が複数 `.lua` 行へ展開された
///   ケース（8.2）。いずれの行で停止しても同一 `.pasta` 30 を提示する。
const EDGE_CHUNK: &str = "\
local a = 1
local b = a + 1
local c = b + 1
local d = c + 1
local e = d + 1
return e
";

/// `EDGE_CHUNK` の手組み `SourceMap`（task 5.4 / 7.2 流儀・`ChunkSourceMap::from_forward`）。
/// フック source 名でキーし、map が内部で正規化する（task 3.4）。
///
/// - 集約行（8.1）: `.lua` 行1 → `.pasta` 20（単一・確定的）。
/// - 展開行（8.2）: `.lua` 行2/3/4 → 同一 `.pasta` 30。
/// - 行5 → `.pasta` 40（展開とは異なる `.pasta`・ステップ観測の終端）。
fn edge_scenario_map() -> Arc<SourceMap> {
    let pp = |line: u32| PastaPos {
        file: EDGE_PASTA_FILE.to_string(),
        line,
    };
    let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
    forward.insert(1, pp(20)); // 集約 `.lua` 行（8.1）: 単一 `.pasta` 20。
    forward.insert(2, pp(30)); // 展開（8.2）: `.pasta` 30 の 1 本目。
    forward.insert(3, pp(30)); // 展開（8.2）: `.pasta` 30 の 2 本目。
    forward.insert(4, pp(30)); // 展開（8.2）: `.pasta` 30 の 3 本目。
    forward.insert(5, pp(40)); // 展開後の異なる `.pasta` 行。

    let mut sm = SourceMap::new();
    sm.insert_chunk(
        EDGE_SOURCE.to_string(),
        EDGE_PASTA_FILE.to_string(),
        ChunkSourceMap::from_forward(forward),
    );
    Arc::new(sm)
}

/// 実 TCP ソケット越しの最小 DAP クライアント（[`super::tests::DapClient`] と同型）。
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

fn is_event(msg: &Value, name: &str) -> bool {
    msg["type"] == "event" && msg["event"] == name
}

fn is_response(msg: &Value, command: &str) -> bool {
    msg["type"] == "response" && msg["command"] == command
}

/// `stackTrace` の top フレーム `(source.path, line)` を返す（task 7.1/7.2 同型）。
/// `.pasta` モードでは resolver（task 5.2）が装着済みなので `.pasta` 座標、`.lua`
/// モードでは `.lua` 座標（チャンク名 + `.lua` 行）を返す。
fn top_frame(client: &mut DapClient, thread_id: u64, seq: u64) -> (String, u32) {
    client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
    let stack = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"]
        .as_array()
        .expect("stackFrames array");
    assert!(!frames.is_empty(), "stack must have the stopped frame");
    let path = frames[0]["source"]["path"]
        .as_str()
        .expect("top frame source path")
        .to_string();
    let line = frames[0]["line"].as_u64().expect("top frame line") as u32;
    (path, line)
}

/// VM ホストスレッドの結果を watchdog 内で確認する（ハング無し）。
fn join_host(host: std::thread::JoinHandle<Result<(), String>>) {
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(host.join());
    });
    match done_rx.recv_timeout(WATCHDOG) {
        Ok(joined) => {
            joined
                .expect("host VM thread must not panic")
                .expect("scenario must run to completion");
        }
        Err(RecvTimeoutError::Timeout) => {
            panic!("host VM thread did not finish within the watchdog (hang?)");
        }
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }
}

/// 実 DAP セッションを起動する共通ヘルパ。`EDGE_CHUNK` を `EDGE_SOURCE` で走らせ、
/// `map`＋サーバ既定 `mode` で `enable` する。
///
/// `attach_mode` が `Some(m)` のとき、initialize 後に実 DAP `attach` リクエスト
/// （`sourcePresentation` 付き）を送って提示モードを **クライアント経路で切替える**
/// （task 5.5・requirement 6.3）。`None` のときは attach を送らず、サーバ既定 `mode` の
/// まま（resolved env > file > 既定）。
///
/// その後 setBreakpoints（`bp_source_path` の `bp_line`）→ configurationDone を済ませ、
/// 最初の停止（reason breakpoint）まで進めて `thread_id` を確定して返す。
fn start_session(
    map: Arc<SourceMap>,
    server_mode: SourceMode,
    attach_mode: Option<SourceMode>,
    bp_source_path: &str,
    bp_line: u32,
) -> (std::thread::JoinHandle<Result<(), String>>, DapClient, u64) {
    let map_for_host = Arc::clone(&map);

    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (go_tx, go_rx) = mpsc::channel::<()>();

    let host = std::thread::spawn(move || -> Result<(), String> {
        let lua = unsafe {
            mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
        };

        let cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            source_mode: server_mode,
            ..Default::default()
        };
        let handle = enable(&lua, &cfg, Some(map_for_host), None)
            .map_err(|e| format!("enable failed: {e}"))?
            .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

        let addr = handle
            .local_addr()
            .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
        addr_tx
            .send(addr)
            .map_err(|_| "addr send failed".to_string())?;

        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "did not receive go signal before running the VM".to_string())?;

        lua.load(EDGE_CHUNK)
            .set_name(EDGE_SOURCE)
            .exec()
            .map_err(|e| format!("scenario exec failed: {e}"))?;
        lua.remove_global_hook();
        drop(handle);
        Ok(())
    });

    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

    // initialize
    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    // attach（任意）: 実クライアント経路で `sourcePresentation` を渡し提示モードを
    // 切替える（task 5.5 / R6.3）。`attach` 応答が来たら適用済み。
    if let Some(m) = attach_mode {
        let presentation = match m {
            SourceMode::Pasta => "pasta",
            SourceMode::Lua => "lua",
        };
        client.send_request(4, "attach", json!({ "sourcePresentation": presentation }));
        let _ = client.recv_until(|m| is_response(m, "attach"));
    }

    // setBreakpoints: `.pasta` 行 BP は翻訳経路（map+Pasta）で `.lua` へ展開され
    // verified になる。`.lua` 源（Lua モード）は直接登録。
    client.send_request(
        2,
        "setBreakpoints",
        json!({
            "source": { "path": bp_source_path },
            "breakpoints": [{ "line": bp_line }],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let bps = bp_resp["body"]["breakpoints"]
        .as_array()
        .expect("breakpoints array");
    assert_eq!(bps.len(), 1);
    assert_eq!(bps[0]["verified"], true, "BP は verified で登録される");

    // configurationDone → VM 実行開始
    client.send_request(3, "configurationDone", json!({}));
    let _ = client.recv_until(|m| is_response(m, "configurationDone"));
    go_tx.send(()).expect("send go signal");

    // 最初の停止（reason breakpoint）
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped["body"]["reason"], "breakpoint",
        "最初の停止は BP（観測起点）"
    );
    let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

    (host, client, thread_id)
}

/// `continue` を投げて完走させ、host を join する。
///
/// 展開 `.pasta` 行 BP（8.2）は複数 `.lua` 行を登録するため、`continue` 後に同一
/// `.pasta` 行 BP の別 `.lua` 行で **再停止**し得る（task 7.1 と同じ多重ヒット挙動）。
/// よって `stopped`（再停止 → もう一度 `continue`）／host 完了（`terminated` 相当の
/// 決定的シグナル）のいずれかを観測するまでループする。`continue` を送ったら、その
/// 後に来る次の制御フレーム（`stopped`／`terminated`）を待つ。再入数で有限（CI 無限
/// ループ防止に上限）。
fn continue_to_end(
    host: std::thread::JoinHandle<Result<(), String>>,
    client: &mut DapClient,
    thread_id: u64,
    mut seq: u64,
) {
    for _ in 0..30u64 {
        client.send_request(seq, "continue", json!({ "threadId": thread_id }));
        seq += 1;
        let next = client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));
        if is_event(&next, "terminated") {
            break;
        }
        // それ以外は同一 BP 行（別 `.lua` 行）への再停止 → もう一度 continue。
    }
    join_host(host);
}

#[cfg(test)]
#[path = "wiring_pasta_mode_edge_e2e_scenarios.rs"]
mod scenarios;
