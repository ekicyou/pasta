//! Task 7.2 — `.pasta` 粒度ステップ **E2E**（E1–E8）。実デバッグセッションを
//! 実 DAP-over-TCP トランスポート越しに駆動し、`.pasta` 行単位のステップ実行
//! （step over/into/out）が design 640 の 8 シナリオすべてで **期待どおりの
//! `.pasta` 停止位置**へ到達することを end-to-end で検証する（requirements
//! **9.1**/**9.2**/**9.3**/**9.4**/**9.5**）。
//!
//! # ハーネス（task 7.1 [`super::pasta_bp_e2e`] / `.lua` ステップ E2E
//! [`super::tests::full_lua_debug_session_all_steps_all_var_types_coroutine_body`]
//! の踏襲）
//!
//! - 実マップ＋`SourceMode::Pasta` を [`enable`](crate::debug::enable) へ渡し
//!   （task 4.2／design 582: map+Pasta で `.pasta` resolver/BP 翻訳/stepper が装着）、
//!   生成 `.lua`（ここでは決定的に制御するため**素の Lua チャンク**）を VM 上で走らせる。
//! - クライアントは実 TCP ソケット越しに `next`/`stepIn`/`stepOut` を送り、各停止の
//!   **`.pasta` 行**を `stackTrace` の top フレーム `line` で読む（DAP は停止位置を
//!   `stopped` body ではなく `stackTrace` で報告する・task 7.1 と同型の [`top_pasta_line`]）。
//!   resolver（task 5.2）が装着済みなので、`stackTrace` の `line` は `.pasta` 座標
//!   （E1–E7）。
//!
//! # マップは「素の Lua チャンク」へ手組みする（task 5.4 unit test 流儀）
//!
//! `.pasta` トランスパイラの出力に依存せず E1–E8 の行構造（複数 `.lua`→同一
//! `.pasta`、サブ呼び出し、再帰、未対応行、コルーチン）を **決定的に**作るため、
//! [`super::super::session`] の task 5.4 unit test と同じく `ChunkSourceMap::
//! from_forward` で `lua_line → PastaPos` を手組みした集約 [`SourceMap`] を注入する。
//! これは task 5.4 が stop-decision ロジックで検証した**同じ振る舞い**を、本物の
//! セッション（transport→dap→session→hook）で合成して end-to-end に証明する 7.2 の
//! 役割そのもの。**期待停止 `.pasta` 行は map から導出**してハードコードを避ける
//! （[`derive`](Expected::derive)・回帰耐性）。
//!
//! # 「歯」（teeth）
//!
//! 中核シナリオ（E1/E3/E6）について、`SourceMode::Lua` へ切り替えると `.pasta`
//! 粒度が無効化されて `.lua` 行で停止する（`.pasta` 行ではない）ことを別テスト
//! [`teeth_lua_mode_stops_at_lua_line_not_pasta`] で示し、アサートが**本物**
//! （恒真でない）ことを裏づける。E8（9.5）はその `.lua` 粒度回帰そのもの。
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
const STEP_SOURCE: &str = "@pasta_step_e2e_scenario";

/// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致させる側）。
/// `.pasta` 拡張子を持たせて [`super::is_pasta_source`] の BP 翻訳経路を通す。
const STEP_PASTA_FILE: &str = "scene_step.pasta";

/// E1–E8 を 1 本で覆う**素の Lua チャンク**。`.pasta` 行は手組みマップ
/// （[`step_scenario_map`]）が与える。行番号（1-origin）と各行の役割:
///
/// ```text
///   1: local function helper(x)
///   2:     local hy = x + 1     -- callee: 未対応（step into で通過・E3/9.4）
///   3:     local hz = hy + 1    -- callee: .pasta 30（step into 停止先・E3 / step out 起点）
///   4:     return hz
///   5: end
///   6: local function recur(n)
///   7:     if n > 0 then        -- .pasta 40（再帰: 別フレームで同一 .pasta 行・E5）
///   8:         return recur(n-1)-- .pasta 41（再帰呼び出し）
///   9:     end
///  10:     return 0
///  11: end
///  12: local body = function()
///  13:     local p = 1          -- .pasta 50（コルーチン本体・E7 step 起点）
///  14:     coroutine.yield()    -- .pasta 51（E7: step over で yield を跨ぐ）
///  15:     local q = p + 1      -- .pasta 52（E7: resume 後の停止先）
///  16:     return q
///  17: end
///  18: local a = 1              -- .pasta 10（BP / step over 起点・単一 .lua 行）
///  19: local b = a + 1          -- .pasta 11（E1: 複数 .lua 行へ展開された .pasta 行の 1 本目）
///  20: local c = b + 1          -- .pasta 11（E1: 同一 .pasta 11 の 2 本目 → 消化・9.1）
///  21: local g = c + 1          -- 未対応（通過・E6/9.4）
///  22: local d = helper(c)      -- .pasta 12（E1 step over 停止先 / E2 サブ呼び出し / E3 step into 起点）
///  23: local e = recur(2)       -- .pasta 13（E5 再帰呼び出し / E2 step over 停止先）
///  24: local f = e + 1          -- .pasta 14（E4 step out 停止先 / E5 step over 停止先）
///  25: local co = coroutine.create(body)
///  26: while coroutine.status(co) ~= 'dead' do  -- 駆動ループ（別スレッド・E7 で skip）
///  27:     coroutine.resume(co)
///  28: end
///  29: return f
/// ```
///
/// E1 の「複数 `.lua` 行を生む `.pasta` 行」は**起点ではなく**行19/20（`.pasta` 11）に
/// 置く（起点 `.pasta` 10 は単一 `.lua` 行18）。これにより `.pasta` 行 BP が起点 1 本
/// だけを登録し、step over 中に同一 BP 行へ再入して `breakpoint` で再停止する事故を
/// 避けつつ、step over が `.pasta` 11 の 2 本（行19/20）を消化することを観測できる。
const STEP_CHUNK: &str = "\
local function helper(x)
local hy = x + 1
local hz = hy + 1
return hz
end
local function recur(n)
if n > 0 then
    return recur(n - 1)
end
return 0
end
local body = function()
local p = 1
coroutine.yield()
local q = p + 1
return q
end
local a = 1
local b = a + 1
local c = b + 1
local g = c + 1
local d = helper(c)
local e = recur(2)
local f = e + 1
local co = coroutine.create(body)
while coroutine.status(co) ~= 'dead' do
coroutine.resume(co)
end
return f
";

/// `STEP_CHUNK` の手組み `SourceMap`（task 5.4 unit test と同流儀・
/// `ChunkSourceMap::from_forward`）。フック source 名でキーし、map が内部で
/// 正規化する（task 3.4）。各 `(lua_line → .pasta line)` は上記コメントの対応表。
fn step_scenario_map() -> Arc<SourceMap> {
    let pp = |line: u32| PastaPos {
        file: STEP_PASTA_FILE.to_string(),
        line,
    };
    let mut forward: BTreeMap<u32, PastaPos> = BTreeMap::new();
    // helper 本体（callee）
    // 行2 は意図的に未対応（step into で通過・E3/9.4）
    forward.insert(3, pp(30)); // step into 停止先 / step out 起点
    forward.insert(4, pp(31));
    // recur 本体（再帰: 別フレームで同一 .pasta 行を踏む・E5）
    forward.insert(7, pp(40));
    forward.insert(8, pp(41));
    forward.insert(10, pp(42));
    // body（コルーチン本体・E7）
    forward.insert(13, pp(50));
    forward.insert(14, pp(51));
    forward.insert(15, pp(52));
    forward.insert(16, pp(53));
    // トップレベル（caller フレーム）
    forward.insert(18, pp(10)); // BP / step over 起点（単一 .lua 行）
    forward.insert(19, pp(11)); // E1: .pasta 11 の 1 本目（複数 .lua 行展開）
    forward.insert(20, pp(11)); // E1: .pasta 11 の 2 本目 → 消化（9.1）
    // 行21 は意図的に未対応（通過・E6/9.4）
    forward.insert(22, pp(12)); // E1 停止先 / E2 サブ呼び出し / E3 step into 起点
    forward.insert(23, pp(13)); // E5 再帰呼び出し / E2 step over 停止先
    forward.insert(24, pp(14)); // E4 step out 停止先 / E5 step over 停止先
    forward.insert(29, pp(15));

    let mut sm = SourceMap::new();
    sm.insert_chunk(
        STEP_SOURCE.to_string(),
        STEP_PASTA_FILE.to_string(),
        ChunkSourceMap::from_forward(forward),
    );
    Arc::new(sm)
}

/// E1–E8 の期待座標を **map から導出**した束（ハードコード回避）。`.lua` 行を起点に
/// `resolve_lua_to_pasta` で `.pasta` 行を引き、シナリオが要求する関係（同一/異なる/
/// 未対応）を build 時に表明する。各テストはこの導出値に対してアサートする。
struct Expected {
    /// BP/step over 起点（単一 `.lua` 行・`derive` 内アンカー行 18）の対応 `.pasta` 行。
    origin_pasta: u32,
    /// E1: 複数 `.lua` 行へ展開された `.pasta` 行（行19/20 = 同一 `.pasta`）と、その
    /// `.lua` 行（1 本目/2 本目）。1 回目 step over の停止先（= 起点の次の異なる
    /// `.pasta` 行）でもある。
    multi_pasta: u32,
    multi_lua_first: u32,
    multi_lua_second: u32,
    /// E6: step over がまたぐ未対応 `.lua` 行（停止しない）。
    unmapped_lua: u32,
    /// E1 の 2 回目 step over 停止先（`.pasta` 11 を消化＋未対応行通過の次の `.pasta` 行）
    /// = helper 呼び出し行（E2 サブ呼び出し / E3 step into 起点）と対応 `.pasta` 行。
    call_helper_lua: u32,
    call_helper_pasta: u32,
    /// E3: helper 内の未対応 `.lua` 行（step into で通過）。
    callee_unmapped_lua: u32,
    /// E3: helper 内の最初の対応 `.lua` 行（step into 停止先・`derive` 内アンカー
    /// 行 3）の `.pasta` 行。
    callee_first_pasta: u32,
    /// E2/E5: helper / recur 呼び出し行から step over した停止先（呼出元フレームの
    /// 次の `.pasta` 行 = 再帰呼び出し行）の `.pasta` 行。
    next_caller_pasta: u32,
    /// E4: step out が呼出元で停止する `.pasta` 行（呼出行の次の対応行）。
    step_out_pasta: u32,
    /// E5: 再帰呼び出し行（step over 起点・`derive` 内アンカー行 23）の `.pasta` 行。
    recur_call_pasta: u32,
    /// E5: 再帰呼び出し行から step over した停止先（呼出元フレームの次の `.pasta`
    /// 行 = 行24）の `.pasta` 行。recur 内の同一 `.pasta` 行（40/41）ではない。
    after_recur_pasta: u32,
    /// E7: コルーチン本体 step 起点（`.pasta` 行）。
    co_origin_pasta: u32,
    /// E7: yield 行の `.pasta` 行（first step over の停止先）。
    co_yield_pasta: u32,
    /// E7: resume 後の `.pasta` 行（yield をまたぐ step over の停止先）。
    co_post_yield_pasta: u32,
}

impl Expected {
    /// map から導出し、シナリオ前提（同一/異なる/未対応）を build 時に表明する。
    /// `.lua` 行番号は [`STEP_CHUNK`] のコメント対応表に由来する固定アンカーだが、
    /// `.pasta` 行は全て map から引き、行同士の関係（同一/異なる/未対応）を表明する
    /// ことで「ハードコードした `.pasta` 行で恒真になる」ことを防ぐ。
    fn derive(map: &SourceMap) -> Self {
        let lp = |lua: u32| map.resolve_lua_to_pasta(STEP_SOURCE, lua).map(|p| p.line);

        // 起点（行18）→ 単一 `.lua` 行の `.pasta` 行。
        let origin_lua = 18;
        let origin_pasta = lp(origin_lua).expect("起点 `.lua` 18 は対応 `.pasta` を持つ");

        // E1: 行19/20 が同一 `.pasta` 行（複数 `.lua` 行展開）。
        let multi_lua_first = 19;
        let multi_lua_second = 20;
        let multi_pasta = lp(multi_lua_first).expect("行19 は対応 `.pasta` を持つ");
        assert_eq!(
            lp(multi_lua_second),
            Some(multi_pasta),
            "E1: 行19/20 は同一 `.pasta` 行（複数 `.lua` 行展開・消化対象）"
        );
        assert_ne!(
            multi_pasta, origin_pasta,
            "E1: 複数 `.lua` の `.pasta` 行は起点の `.pasta` 行と異なる（1 回目 step over の停止先）"
        );

        // E6: 行21 は未対応（通過対象）。
        let unmapped_lua = 21;
        assert_eq!(lp(unmapped_lua), None, "E6: 行21 は未対応（通過する）");

        // E1 の 2 回目 step over 停止先 / E2 サブ呼び出し / E3 step into 起点:
        // 行22（helper 呼び出し）。`.pasta` は `multi_pasta` と異なる次の行。
        let call_helper_lua = 22;
        let call_helper_pasta = lp(call_helper_lua).expect("行22 は helper 呼び出しの対応 `.pasta`");
        assert_ne!(
            call_helper_pasta, multi_pasta,
            "E1: 2 回目 step over 停止先は `.pasta` 11 と異なる次の `.pasta` 行"
        );

        // E3 step into: helper（行2 未対応 → 行3 対応）。
        let callee_unmapped_lua = 2;
        assert_eq!(
            lp(callee_unmapped_lua),
            None,
            "E3: helper 内 行2 は未対応（step into で通過）"
        );
        let callee_first_lua = 3;
        let callee_first_pasta =
            lp(callee_first_lua).expect("helper 内 行3 は最初の対応 `.pasta`（step into 停止先）");

        // E5 再帰: 呼び出し行23（recur）。recur 内（行7/8）は別フレームで同一 `.pasta`
        // 行を踏むが、step over は depth で別扱い → 呼出元フレームの次行へ。
        let recur_call_lua = 23;
        let recur_call_pasta = lp(recur_call_lua).expect("行23 は再帰呼び出しの対応 `.pasta`");
        assert_ne!(
            recur_call_pasta, call_helper_pasta,
            "E2/E5: 再帰呼び出し行は helper 呼び出し行と異なる `.pasta` 行"
        );
        // E5 step over 停止先: 行24（recur 呼び出しの直後の対応行）。
        let after_recur_lua = 24;
        let after_recur_pasta = lp(after_recur_lua).expect("行24 は recur 呼び出し直後の対応 `.pasta`");
        assert_ne!(
            after_recur_pasta, recur_call_pasta,
            "E5: 再帰 step over 停止先は recur 呼び出し行と異なる次の `.pasta` 行"
        );
        // recur 内の同一 `.pasta` 行（40/41）が誤停止の罠であることを表明（前提）。
        assert!(
            lp(7).is_some() && lp(8).is_some(),
            "E5: recur 本体（行7/8）は対応 `.pasta` 行を持つ（深いフレームの誤停止候補）"
        );

        // E2: helper 呼び出し行（行22）から step over → 呼出元フレームの次の `.pasta`
        // 行 = 行23（recur 呼び出し・`.pasta` 13）。helper の `.pasta`（30/31）ではない。
        let next_caller_pasta = recur_call_pasta;

        // E4 step out: helper から戻り、呼出元の次の対応行 = 行23（recur 呼び出し）。
        // helper 呼び出し行（行22）の直後の対応行であり、呼出行の `.pasta` とは異なる。
        let step_out_pasta = recur_call_pasta;
        assert_ne!(
            step_out_pasta, call_helper_pasta,
            "E4: step out 停止先は呼出行の `.pasta` 行と異なる（次の対応行）"
        );

        // E7 コルーチン: 本体 行13 起点 → 行14 yield → 行15 resume 後。
        let co_origin_pasta = lp(13).expect("コルーチン本体 行13 の対応 `.pasta`");
        let co_yield_pasta = lp(14).expect("yield 行14 の対応 `.pasta`");
        let co_post_yield_pasta = lp(15).expect("resume 後 行15 の対応 `.pasta`");
        assert!(
            co_origin_pasta != co_yield_pasta && co_yield_pasta != co_post_yield_pasta,
            "E7: コルーチンの 3 停止位置は相異なる `.pasta` 行"
        );

        Expected {
            origin_pasta,
            multi_pasta,
            multi_lua_first,
            multi_lua_second,
            unmapped_lua,
            call_helper_lua,
            call_helper_pasta,
            callee_unmapped_lua,
            callee_first_pasta,
            next_caller_pasta,
            step_out_pasta,
            recur_call_pasta,
            after_recur_pasta,
            co_origin_pasta,
            co_yield_pasta,
            co_post_yield_pasta,
        }
    }
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

/// `stackTrace` の top フレーム `line` を返す（task 7.1 [`super::tests::top_frame_line`]
/// と同型）。`SourceMode::Pasta` では resolver（task 5.2）が装着済みなので、これは
/// **`.pasta` 行**を返す（E1–E7 はこれで `.pasta` 停止位置を観測する）。`Lua` モード
/// では `.lua` 行を返す（E8/teeth）。`seq` は stackTrace 要求の相関 seq。
fn top_frame_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
    client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
    let stack = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"]
        .as_array()
        .expect("stackFrames array");
    assert!(!frames.is_empty(), "stack must have the stopped frame");
    frames[0]["line"].as_u64().expect("top frame line") as u32
}

/// stackTrace の top フレーム `source.path` が `.pasta` を提示することを表明し、その
/// `line`（= `.pasta` 行）を返す。`.pasta` 提示の「歯」を各停止で効かせる
/// （resolver/翻訳が無効なら `.lua` チャンク名が出てこのアサートが落ちる）。
fn top_pasta_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
    client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
    let stack = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"]
        .as_array()
        .expect("stackFrames array");
    assert!(!frames.is_empty(), "stack must have the stopped frame");
    let top_src = frames[0]["source"]["path"]
        .as_str()
        .expect("top frame source path");
    assert!(
        top_src.ends_with(".pasta"),
        "`.pasta` 提示中は top フレームが `.pasta` を提示すること（`.lua` ではない）: {top_src:?}"
    );
    frames[0]["line"].as_u64().expect("top frame line") as u32
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

/// 実 DAP セッションを起動する共通ヘルパ。`STEP_CHUNK` を `STEP_SOURCE` で走らせ、
/// `map`＋`mode` で `enable` する。`(host, client, thread_id)` を返す。`bp_pasta_line`
/// は最初に張る `.pasta` 行 BP（`pasta_path` が `.pasta` なら BP 翻訳経路を通る）。
/// `mode == Lua` のときは `.lua` 行 BP を直接張る（`lua_bp_line` を使う）。
///
/// クライアントは initialize→setBreakpoints→configurationDone を済ませ、最初の停止
/// （reason breakpoint）まで進めて `thread_id` を確定して返す。
#[allow(clippy::too_many_arguments)]
fn start_session(
    map: Arc<SourceMap>,
    mode: SourceMode,
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
            source_mode: mode,
            ..Default::default()
        };
        let handle = enable(&lua, &cfg, Some(map_for_host))
            .map_err(|e| format!("enable failed: {e}"))?
            .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

        let addr = handle
            .local_addr()
            .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "did not receive go signal before running the VM".to_string())?;

        lua.load(STEP_CHUNK)
            .set_name(STEP_SOURCE)
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
        "最初の停止は BP（step 起点）"
    );
    let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

    (host, client, thread_id)
}

/// `continue` を投げて完走させ、host を join する（停止が再発する行 BP の再入は無いので
/// 1 回で `terminated`／完走）。
fn continue_to_end(
    host: std::thread::JoinHandle<Result<(), String>>,
    client: &mut DapClient,
    thread_id: u64,
    seq: u64,
) {
    client.send_request(seq, "continue", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "continue"));
    join_host(host);
}


#[cfg(test)]
#[path = "wiring_pasta_step_e2e_scenarios.rs"]
mod scenarios;
