//! TransportLoop: 長命 socket スレッド + 最小行プロトコル。session 上に
//! 停止→取得→再開のトランスポート往復を成立させる（R4 検証・task 3.2）。
//!
//! 兄弟モジュール `harness_types`（`build_jit_off_vm` / `DebugCommand` /
//! `DebugEvent` / `Variable` / `Breakpoint`）、`pause_gate`（`should_pause`）、
//! `frame_inspector`（`inspect_locals`）を再利用する。
//!
//! ## 設計準拠（design.md "TransportLoop" / "System Flows / R4 トランスポート往復" /
//! "スレッドモデル" ②③ / requirements.md R4.1〜R4.4 / research D3）
//!
//! ### 3 スレッドトポロジ（D3）
//! - **VM ホストスレッド**: jit-off VM を**このスレッド上で**構築（`mlua::Lua` は
//!   `!Send` のため決して move しない・R4.2）。`set_global_hook`（`EVERY_LINE`・
//!   R1 実証済みのグローバルフック方式）を設置し、標的行（**トップレベルチャンク**
//!   の行）で停止ループに入る。停止ループは PauseGate の `block_until_command`
//!   （Continue のみ対応）とは異なり、**Inspect も処理する**（R4 新規）。Inspect 受信時は
//!   `inspect_locals(lua)`（フック内＝VM スレッド上で実行・R4.2）を呼び
//!   `DebugEvent::Vars(..)` を送り、なお停止を継続する。`mlua::Error` はスレッド境界で
//!   `String` へ変換する（`!Send` 境界・スレッドモデル③）。
//! - **listener スレッド（長命 1 本）**: `TcpListener` を accept（1 接続）した後、
//!   socket とチャネルを橋渡しする（**socket I/O のみ・VM/Lua 非アクセス**・R4.2）。
//!   `std::net` のみ使用（R4.3）。デバッグセッション中ずっと生存する（スレッドモデル②）。
//! - **client スレッド（= テストドライバ）**: `TcpStream::connect` し、最小行
//!   プロトコルで `stopped` 受信 → `vars` 送信 → `vars <payload>` 受信 → `continue`
//!   送信、の往復を駆動する。
//!
//! ### 最小行プロトコル（DAP は使わない・R4.3）
//! 全メッセージは `\n` 終端の 1 行。
//! - listener → client: `stopped <source> <line>`（停止通知）
//! - client → listener: `vars`（変数要求）
//! - listener → client: `vars <payload>`（変数応答。payload は `name=type:repr` を
//!   `;` 区切りで連結した文字列）
//! - client → listener: `continue`（再開指示）
//!
//! ### R3.4 既知制約への配慮（停止フレームは MAIN-THREAD に置く）
//! `inspect_locals` はメインステート FFI 経路でトップレベル Lua フレームのローカルを
//! 取得する（走行中コルーチン本体のフレームには到達不可・R3.4）。本 R4 検証では
//! **ブレークポイント行をトップレベルチャンクに置く**ことで、`inspect_locals` が実変数を
//! 返し、それが socket 越しに client へ運ばれることを実証する（空往復ではない）。
//!
//! ### timeout はテスト専用（スレッドモデル④）
//! 本番のブレークは無期限ブロックが正。停止コアには timeout を組み込まない。
//! client の `set_read_timeout` と join watchdog は**テスト専用**で、CI を吊らせない
//! ためだけに置く。

#![allow(dead_code)]

use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use mlua::{HookTriggers, VmState};

use super::frame_inspector::inspect_locals;
use super::harness_types::{Breakpoint, DebugCommand, DebugEvent, Variable};
use super::pause_gate::should_pause;

// ---------------------------------------------------------------------------
// (A) listener スレッド: socket ↔ チャネルの橋渡し（socket I/O のみ・R4.2/R4.3）
// ---------------------------------------------------------------------------

/// listener スレッドが握るチャネル端点（VM ホストスレッドへの seam）。
///
/// `cmd_tx` でフックへコマンド（`Inspect` / `Continue`）を送り、`event_rx` で
/// フックからのイベント（`Stopped` / `Vars`）を受ける。`mlua::Lua` は**一切
/// 含まれない**（VM 操作は VM ホストスレッドに閉じる・`!Send` 遵守・スレッドモデル③）。
pub(crate) struct Bridge {
    /// listener → フック: コマンド送出端（`Inspect` / `Continue`）。
    pub cmd_tx: Sender<DebugCommand>,
    /// フック → listener: イベント受信端（`Stopped` / `Vars`）。
    pub event_rx: Receiver<DebugEvent>,
}

/// 変数群を最小行プロトコルの payload 文字列へ整形する。
///
/// `name=type:repr` を `;` 区切りで連結する。`name` 内に区切り文字が現れる懸念は
/// PoC スコープ外（ローカル名は識別子）。空集合なら空文字列を返す。
fn encode_vars(vars: &[Variable]) -> String {
    vars.iter()
        .map(|v| format!("{}={}:{}", v.name, v.type_name, v.repr))
        .collect::<Vec<_>>()
        .join(";")
}

/// listener スレッド本体（長命 1 スレッド・スレッドモデル②）。
///
/// `listener` で 1 接続を accept し、socket とチャネルを橋渡しする。
/// **socket I/O のみを担当し、Lua/VM へは一切アクセスしない**（R4.2）。
/// `std::net` のみ使用（R4.3）。
///
/// 動作（design "System Flows / R4 トランスポート往復"）:
/// 1. `event_rx` から `DebugEvent::Stopped { source, line }` が来たら socket へ
///    `stopped <source> <line>\n` を書く。
/// 2. client から `vars\n` が来たら `cmd_tx` へ `DebugCommand::Inspect` を送り、
///    `event_rx` で `DebugEvent::Vars(..)` を待ち、socket へ `vars <payload>\n` を書く。
/// 3. client から `continue\n` が来たら `cmd_tx` へ `DebugCommand::Continue` を送る
///    （そこで橋渡しを終える）。
///
/// チャネル切断・socket EOF・I/O エラー時は安全に return する（VM をハングさせない）。
pub(crate) fn serve(listener: TcpListener, bridge: Bridge) {
    // (1) 1 接続を accept（ループバック・1 client）。
    let stream = match listener.accept() {
        Ok((s, _peer)) => s,
        Err(_) => return,
    };
    // 読み書きで同一 socket を使うため複製する（BufReader が read 半分を所有）。
    let write_half = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let mut reader = BufReader::new(stream);
    let mut writer = write_half;

    // (2) 停止通知: フックの `Stopped` を待って client へ流す。
    //     VM ホストスレッドがフックで標的行に達すると `Stopped` が来る。
    match bridge.event_rx.recv() {
        Ok(DebugEvent::Stopped { source, line }) => {
            if writeln!(writer, "stopped {source} {line}").is_err() {
                return;
            }
            if writer.flush().is_err() {
                return;
            }
        }
        // 想定外イベント or 切断 → 橋渡しを終える。
        _ => return,
    }

    // (3) client からの行コマンドを処理する（vars / continue）。
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return, // EOF（client 切断）。
            Ok(_) => {}
            Err(_) => return,
        }
        let cmd = line.trim_end();

        if cmd == "vars" {
            // フックへ Inspect 指示 → Vars 応答を待って client へ流す。
            if bridge.cmd_tx.send(DebugCommand::Inspect).is_err() {
                return;
            }
            // Inspect の応答（Vars）を待つ。Stopped が再送される実装ではないため
            // 次に来るイベントは Vars のはず。
            match bridge.event_rx.recv() {
                Ok(DebugEvent::Vars(vars)) => {
                    let payload = encode_vars(&vars);
                    if writeln!(writer, "vars {payload}").is_err() {
                        return;
                    }
                    if writer.flush().is_err() {
                        return;
                    }
                }
                _ => return,
            }
        } else if cmd == "continue" {
            // フックを再開させて橋渡し終了。
            let _ = bridge.cmd_tx.send(DebugCommand::Continue);
            return;
        } else {
            // 未知コマンドは無視して次行を待つ（プロトコルに無いため）。
            continue;
        }
    }
}

/// client スレッド（= テストドライバ）の往復本体。
///
/// `addr` へ `TcpStream::connect` し、最小行プロトコルで往復を駆動する:
/// 1. `stopped <source> <line>` を読む。
/// 2. `vars` を送る。
/// 3. `vars <payload>` を読む。
/// 4. `continue` を送る。
///
/// 完全な往復（stopped 受信 ∧ vars payload 受信 ∧ continue 送信）が成立したときのみ
/// `Ok(true)` を返す。**テスト専用**の read timeout（`set_read_timeout`）を設定し、
/// 停止往復が詰まっても CI を吊らせない（スレッドモデル④: 停止コアには timeout を
/// 入れず、client 側にのみ置く）。
pub(crate) fn run_round_trip(addr: SocketAddr) -> std::io::Result<bool> {
    let stream = TcpStream::connect(addr)?;
    // テスト専用 read timeout（詰まり時に永久ハングさせない）。
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
    let write_half = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let mut writer = write_half;

    // (1) stopped 行を読む。
    let mut stopped_line = String::new();
    if reader.read_line(&mut stopped_line)? == 0 {
        return Ok(false); // EOF（停止通知が来なかった）。
    }
    let stopped_received = stopped_line.trim_end().starts_with("stopped ");
    if !stopped_received {
        return Ok(false);
    }

    // (2) vars 要求を送る。
    writeln!(writer, "vars")?;
    writer.flush()?;

    // (3) vars payload を読む。
    let mut vars_line = String::new();
    if reader.read_line(&mut vars_line)? == 0 {
        return Ok(false); // EOF（変数応答が来なかった）。
    }
    let vars_received = vars_line.trim_end().starts_with("vars");
    if !vars_received {
        return Ok(false);
    }

    // (4) continue を送る（再開指示）。
    writeln!(writer, "continue")?;
    writer.flush()?;

    // stopped 受信 ∧ vars 受信 ∧ continue 送信 が全て成立した。
    Ok(true)
}

/// vars payload 行（`vars <payload>`）から payload 部分を取り出す（テスト用）。
fn parse_vars_payload(line: &str) -> Option<String> {
    let trimmed = line.trim_end();
    trimmed
        .strip_prefix("vars")
        .map(|rest| rest.trim_start().to_string())
}

// ---------------------------------------------------------------------------
// (B) VM ホストスレッド: jit-off VM + フック内 R4 停止ループ（Inspect 対応）
// ---------------------------------------------------------------------------

/// R4 トランスポート往復検証用シナリオのチャンク名（標的ソース）。
pub(crate) const R4_SOURCE: &str = "@r4_scenario";

/// R4 標的（ブレークポイント）行番号。`R4_SOURCE` 内の 1-origin 行。
///
/// **トップレベルチャンク**の行を選ぶ（コルーチン本体ではない）。これにより
/// `inspect_locals`（メインステート FFI 経路）が実ローカルを返す（R3.4 制約遵守）。
pub(crate) const R4_BREAKPOINT_LINE: u32 = 5;

/// inspect で必ず観測できるべきローカル名（client 側の非空往復 assert に使う）。
pub(crate) const R4_EXPECTED_LOCAL: &str = "answer";

/// R4 停止ループ（フック内・VM スレッド上で実行）。
///
/// PauseGate の `block_until_command` は `Continue` のみ対応するため、R4 で必要な
/// **Inspect 処理**を含むこのループを transport_loop 側に新設する（pause_gate.rs は
/// 改変しない）。標的判定には `should_pause` を再利用する。
///
/// 動作:
/// 1. `DebugEvent::Stopped { source, line }` を送出（listener が client へ転送）。
/// 2. `cmd_rx` を**無期限**ブロッキング `recv()`:
///    - `DebugCommand::Inspect`: `inspect_locals(lua)`（フック内＝VM スレッド上・R4.2）を
///      呼び `DebugEvent::Vars(vars)` を送る。停止を継続（ループ）。
///    - `DebugCommand::Continue`: `VmState::Continue` を返して再開（ループ終了）。
///    - 切断（`Err`）: VM をハングさせないため `Continue` で復帰する。
///
/// `mlua::Lua` の参照（`lua`）はこのスレッド上にのみ存在する（move しない・`!Send`
/// 遵守）。`inspect_locals` のエラーは `?` で関数の `mlua::Result` へ伝播し、スレッド
/// 境界で `String` 化される。
fn r4_stop_loop(
    lua: &mlua::Lua,
    cmd_rx: &Receiver<DebugCommand>,
    event_tx: &Sender<DebugEvent>,
    source: &str,
    line: u32,
) -> mlua::Result<VmState> {
    // (1) 停止を外部へ通知。受信側不在でも停止自体は継続するため送出失敗は無視。
    let _ = event_tx.send(DebugEvent::Stopped {
        source: source.to_string(),
        line,
    });

    // (2) Continue が来るまで無期限ブロック。Inspect は変数を返して停止継続。
    loop {
        match cmd_rx.recv() {
            Ok(DebugCommand::Inspect) => {
                // フック内（VM スレッド上）で FFI ローカル取得（R4.2: VM 操作は
                // フック内に閉じる）。
                let vars = inspect_locals(lua)?;
                // 変数を外部へ。送出失敗（listener 切断）は無視して停止継続。
                let _ = event_tx.send(DebugEvent::Vars(vars));
                continue;
            }
            Ok(DebugCommand::Continue) => return Ok(VmState::Continue),
            // 送信端が全て drop（切断）。これ以上コマンドは来ないため、VM を
            // ハングさせないよう Continue で復帰する（安全側フォールバック）。
            Err(_) => return Ok(VmState::Continue),
        }
    }
}

/// R4 用 VM ホストスレッドの本体（**この関数の中だけで `mlua::Lua` を構築・所有**）。
///
/// 1. jit-off VM をこのスレッド上で構築。
/// 2. `set_global_hook`（`EVERY_LINE`）を設置。コールバックは:
///    - 進行カウンタを +1（実行行の証拠）。
///    - `should_pause` が真の標的行でのみ `r4_stop_loop`（Inspect 対応停止ループ）へ。
///    - 標的外は即 `VmState::Continue`。
/// 3. **トップレベル**シナリオを実行（標的行のローカルに実変数 `answer` を持つ）。
///
/// `mlua::Result<()>` を返すため `?` を素直に使える。スレッド境界での `String` 変換は
/// 呼び出し元（spawn クロージャ）が `map_err` で行う（`!Send` 境界・スレッドモデル③）。
fn run_r4_host_thread(
    breakpoints: HashSet<Breakpoint>,
    cmd_rx: Receiver<DebugCommand>,
    event_tx: Sender<DebugEvent>,
    progress: Arc<AtomicUsize>,
) -> mlua::Result<()> {
    // (1) ホスト役スレッド上で VM を構築（このスレッドに閉じる・move しない）。
    let lua = super::harness_types::build_jit_off_vm()?;

    // PauseGate を新設せず、targeting だけを `should_pause` 用の一時 PauseGate で行う。
    // PauseGate はチャネル端点を所有してしまうため、ここでは targeting 専用に
    // 別チャネル（捨てる側）は作らず、`should_pause` 相当の包含判定をクロージャに
    // 直接持たせる。ただし設計上の seam を尊重し、停止ループは r4_stop_loop が
    // 所有する cmd_rx/event_tx を使う。
    //
    // `should_pause(&gate, frame)` を使うため、targeting 専用の PauseGate を構築する。
    // この gate のチャネル端点は停止ループでは使わない（r4_stop_loop が別途
    // cmd_rx/event_tx を直接受け取る）ため、ダミーのチャネル端点を持たせる。
    let (gate_cmd_tx_dummy, gate_cmd_rx) = std::sync::mpsc::channel::<DebugCommand>();
    let (gate_event_tx, _gate_event_rx_dummy) = std::sync::mpsc::channel::<DebugEvent>();
    // ダミー送信端を保持して gate_cmd_rx を生かす（drop すると recv が即 Err になるが、
    // gate の cmd_rx は targeting には使わないので影響はない。明示保持で意図を示す）。
    let _keep_alive = gate_cmd_tx_dummy;
    let gate = super::pause_gate::PauseGate::new(breakpoints, gate_cmd_rx, gate_event_tx);

    // 停止ループが使う本物のチャネル端点（listener との seam）。
    let loop_cmd_rx = cmd_rx;
    let loop_event_tx = event_tx;

    lua.set_global_hook(HookTriggers::EVERY_LINE, move |hook_lua, debug| {
        // 実行行の証拠としてカウンタを進める。
        progress.fetch_add(1, Ordering::SeqCst);

        // 標的判定は should_pause（pause_gate を再利用）。
        if should_pause(&gate, debug) {
            // R4 停止ループ（Inspect 対応）。source/line は frame から再抽出する。
            let src = debug.source();
            let source = src
                .source
                .as_ref()
                .map(|c| c.as_ref().to_string())
                .or_else(|| src.short_src.as_ref().map(|c| c.as_ref().to_string()))
                .unwrap_or_default();
            let line = debug.current_line().unwrap_or(0) as u32;
            return r4_stop_loop(hook_lua, &loop_cmd_rx, &loop_event_tx, &source, line);
        }
        Ok(VmState::Continue)
    })?;

    // (3) トップレベルシナリオ実行（標的行のローカルに実変数を持つ）。
    run_r4_scenario(&lua)?;

    lua.remove_global_hook();
    Ok(())
}

/// R4 停止対象の既知チャンクを実行する（**トップレベル**・コルーチン本体ではない）。
///
/// 行番号（1-origin）:
///   1: local answer = 42        <- 実変数（number）
///   2: local label = 'ok'       <- 実変数（string）
///   3: local flag = true        <- 実変数（boolean）
///   4: local total = answer + 1
///   5: local marker = total     <- R4_BREAKPOINT_LINE（ここで停止）。answer 等が可視。
///   6: return marker
///
/// 標的行（5）でフックが停止すると、`inspect_locals` がトップレベルフレームの
/// ローカル（`answer` / `label` / `flag` / `total` / `marker`）を実値で返す
/// （R3.4 制約: メインスレッドフレームのため到達可能）。
fn run_r4_scenario(lua: &mlua::Lua) -> mlua::Result<()> {
    let chunk = "\
local answer = 42
local label = 'ok'
local flag = true
local total = answer + 1
local marker = total
return marker
";
    lua.load(chunk).set_name(R4_SOURCE).exec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// テスト専用 watchdog（停止コアには無し・スレッドモデル④）。
    const WATCHDOG: Duration = Duration::from_secs(15);

    /// R4.1 / R4.2 / R4.3（トランスポート往復・3スレッド・std::net のみ）:
    ///
    /// 1. `TcpListener::bind("127.0.0.1:0")` で OS 割当ポートに待受、`addr` を得る。
    /// 2. 2 組のチャネル対（cmd: listener→フック / event: フック→listener）を作る。
    /// 3. **VM ホストスレッド**を spawn（cmd_rx + event_tx を所有・**スレッド上で**
    ///    jit-off VM を構築・トップレベルブレークポイントで停止）。
    /// 4. **listener スレッド**を spawn（`serve(listener, Bridge{cmd_tx, event_rx})`）。
    /// 5. **テストスレッド（= client）**で `run_round_trip(addr)` を呼び、`Ok(true)`
    ///    （stopped→vars→continue の往復完了）を assert。
    /// 6. 別経路でも vars payload に期待ローカル名（`answer`）が乗っていることを確認し、
    ///    「実 inspect が socket 越しに運ばれた（空往復ではない）」ことを実証する。
    /// 7. 両スレッドをテスト専用 watchdog 付きで join（ハング不可）。
    ///
    /// 追加クレートは使わない（std::net / std::sync::mpsc / std::thread のみ・R4.3）。
    #[test]
    fn transport_round_trip_loopback() {
        // (1) OS 割当ポートで待受。
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0 must succeed");
        let addr = listener.local_addr().expect("local_addr must be available");

        // (2) 2 組のチャネル対（スレッドモデル③: チャネルが唯一の seam）。
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<DebugCommand>();
        let (event_tx, event_rx) = std::sync::mpsc::channel::<DebugEvent>();

        // 標的ブレークポイント = トップレベルチャンクの行（R3.4 制約遵守）。
        let mut bps: HashSet<Breakpoint> = HashSet::new();
        bps.insert((R4_SOURCE.to_string(), R4_BREAKPOINT_LINE));

        let progress = Arc::new(AtomicUsize::new(0));

        // (3) VM ホストスレッド（cmd_rx + event_tx を所有・VM はこのスレッドに閉じる）。
        //     `mlua::Error`（`!Send`）はスレッド境界で `String` へ変換する。
        let host_progress = Arc::clone(&progress);
        let host = thread::spawn(move || -> Result<(), String> {
            run_r4_host_thread(bps, cmd_rx, event_tx, host_progress).map_err(|e| e.to_string())
        });

        // (4) listener スレッド（長命 1 本・socket I/O のみ・スレッドモデル②）。
        let bridge = Bridge { cmd_tx, event_rx };
        let listener_handle = thread::spawn(move || {
            serve(listener, bridge);
        });

        // (5) テストスレッド（= client）で往復を駆動。
        let round_trip = run_round_trip(addr);
        let completed = round_trip.expect("run_round_trip must not error");
        assert!(
            completed,
            "full round-trip (stopped -> vars -> continue) must complete over std::net loopback (R4.1)"
        );

        // (7) スレッドを watchdog 付きで join（ハング不可）。
        join_with_watchdog("listener", listener_handle, WATCHDOG);
        let host_result = join_host_with_watchdog(host, WATCHDOG);
        host_result
            .expect("VM host thread must finish after continue (no deadlock, R4.1)")
            .expect("VM host thread must not panic")
            .expect("R4 scenario must execute to completion (mlua error mapped to String)");

        // 進行観測の妥当性（実行行を跨いだ証拠）。
        assert!(
            progress.load(Ordering::SeqCst) >= R4_BREAKPOINT_LINE as usize,
            "VM host thread must have executed lines up to the breakpoint"
        );

        println!(
            "[R4] transport round-trip completed end-to-end over std::net loopback \
             (stopped -> vars -> continue); socket I/O isolated to the listener thread, \
             VM/FFI inspect confined to the hook (VM thread); std-only (no extra crates)."
        );
    }

    /// R4.1（補強・非空往復の実証）: 往復で運ばれる `vars` payload に、停止フレームの
    /// **実ローカル**（`answer` = number）が含まれていることを直接検証する。
    ///
    /// `transport_round_trip_loopback` は往復「完了」を assert するが、payload が空でも
    /// `vars\n` 1 行で完了し得る。本テストは client 側で payload を実際に読み、期待
    /// ローカル名が乗っていることを確認して「FFI inspect が socket 越しに運ばれた
    /// （空往復ではない）」ことを保証する（R4.1 ＋ R3 連携・R4.2）。
    #[test]
    fn round_trip_carries_real_inspected_vars() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind must succeed");
        let addr = listener.local_addr().expect("local_addr must be available");

        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<DebugCommand>();
        let (event_tx, event_rx) = std::sync::mpsc::channel::<DebugEvent>();

        let mut bps: HashSet<Breakpoint> = HashSet::new();
        bps.insert((R4_SOURCE.to_string(), R4_BREAKPOINT_LINE));

        let progress = Arc::new(AtomicUsize::new(0));
        let host_progress = Arc::clone(&progress);
        let host = thread::spawn(move || -> Result<(), String> {
            run_r4_host_thread(bps, cmd_rx, event_tx, host_progress).map_err(|e| e.to_string())
        });

        let bridge = Bridge { cmd_tx, event_rx };
        let listener_handle = thread::spawn(move || {
            serve(listener, bridge);
        });

        // client 往復を手動で駆動して payload を読む（run_round_trip と同手順だが
        // payload を検証するため inline で実行する）。
        let stream = TcpStream::connect(addr).expect("connect must succeed");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set_read_timeout (test-only) must succeed");
        let write_half = stream.try_clone().expect("try_clone must succeed");
        let mut reader = BufReader::new(stream);
        let mut writer = write_half;

        // stopped 行。
        let mut stopped_line = String::new();
        let n = reader
            .read_line(&mut stopped_line)
            .expect("reading stopped line must not error (test-only timeout)");
        assert!(n > 0, "must receive a stopped line (not EOF)");
        assert!(
            stopped_line.trim_end().starts_with("stopped "),
            "first line must be a 'stopped' notification, got: {stopped_line:?}"
        );
        // stopped 行は標的 source/line を運ぶ。
        assert!(
            stopped_line.contains(R4_SOURCE)
                && stopped_line.contains(&R4_BREAKPOINT_LINE.to_string()),
            "stopped line must carry the breakpoint source/line, got: {stopped_line:?}"
        );

        // vars 要求 → payload 読み。
        writeln!(writer, "vars").expect("send vars must succeed");
        writer.flush().expect("flush must succeed");
        let mut vars_line = String::new();
        let n = reader
            .read_line(&mut vars_line)
            .expect("reading vars line must not error (test-only timeout)");
        assert!(n > 0, "must receive a vars payload line (not EOF)");

        let payload =
            parse_vars_payload(&vars_line).expect("vars line must start with 'vars'");
        assert!(
            !payload.is_empty(),
            "vars payload must be NON-EMPTY (proves real inspect over the wire, not an empty round-trip), got: {vars_line:?}"
        );
        assert!(
            payload.contains(R4_EXPECTED_LOCAL),
            "vars payload must contain the expected top-level local '{R4_EXPECTED_LOCAL}' \
             (real FFI inspect carried over std::net), got payload: {payload:?}"
        );
        // 実値も乗っていること（answer = 42 / number）。
        assert!(
            payload.contains("answer=number:42"),
            "vars payload must carry the real inspected value answer=number:42, got: {payload:?}"
        );

        // continue 送出で再開。
        writeln!(writer, "continue").expect("send continue must succeed");
        writer.flush().expect("flush must succeed");

        join_with_watchdog("listener", listener_handle, WATCHDOG);
        let host_result = join_host_with_watchdog(host, WATCHDOG);
        host_result
            .expect("VM host thread must finish after continue (no deadlock)")
            .expect("VM host thread must not panic")
            .expect("R4 scenario must run to completion");

        println!(
            "[R4] round-trip carried real inspected vars over the wire: payload = {payload:?}"
        );
    }

    /// `encode_vars` の整形（payload フォーマットの単体確認・非空往復 assert の前提）。
    #[test]
    fn encode_vars_formats_name_type_repr() {
        let vars = vec![
            Variable {
                name: "answer".to_string(),
                type_name: "number".to_string(),
                repr: "42".to_string(),
            },
            Variable {
                name: "label".to_string(),
                type_name: "string".to_string(),
                repr: "ok".to_string(),
            },
        ];
        let encoded = encode_vars(&vars);
        assert_eq!(encoded, "answer=number:42;label=string:ok");

        // 空集合は空文字列。
        assert_eq!(encode_vars(&[]), "");

        // parse_vars_payload は `vars <payload>` から payload を取り出す。
        assert_eq!(
            parse_vars_payload("vars answer=number:42\n").as_deref(),
            Some("answer=number:42")
        );
        // payload 空でも prefix があれば Some("")。
        assert_eq!(parse_vars_payload("vars \n").as_deref(), Some(""));
        assert_eq!(parse_vars_payload("stopped @s 1\n"), None);
    }

    // --- テスト専用 watchdog ヘルパ（停止コアには組み込まない・スレッドモデル④）---

    /// 値を返さない join ハンドルを watchdog 付きで join する（listener 用）。
    /// timeout 内に終わらなければ panic（デッドロック疑いをテスト失敗として顕在化）。
    fn join_with_watchdog(name: &str, handle: thread::JoinHandle<()>, timeout: Duration) {
        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
        thread::spawn(move || {
            let _ = handle.join();
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(timeout)
            .unwrap_or_else(|_| panic!("{name} thread did not finish within watchdog (deadlock?)"));
    }

    /// VM ホストスレッドを watchdog 付きで join する。
    /// `Some(thread::Result<Result<(), String>>)` を返し、`None` はデッドロック疑い。
    fn join_host_with_watchdog(
        handle: thread::JoinHandle<Result<(), String>>,
        timeout: Duration,
    ) -> Option<thread::Result<Result<(), String>>> {
        let (done_tx, done_rx) =
            std::sync::mpsc::channel::<thread::Result<Result<(), String>>>();
        thread::spawn(move || {
            let _ = done_tx.send(handle.join());
        });
        done_rx.recv_timeout(timeout).ok()
    }
}
