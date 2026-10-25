//! 2 つのインスタンスの隔たりの試験（仕様 `shiori4-v3-portable`・4.3・4.4・5.1〜5.6・6.1・6.3・6.4）。
//!
//! `shiori_get_api(3)` のテーブルを `shiori4_abi` で呼び、2 つのゴースト（別の一時ディレクトリ）を
//! 並べて、変数・チェイントークの続き・ログの出し先と量・デバッグのポート・kick の送り先が
//! インスタンスごとに分かれていることを確かめる。
//!
//! この試験のバイナリの `#[test]` は、[`serial`] で 1 本ずつ走らせる。旧来の窓口の置き場は
//! プロセスに 1 つで、unload の後のスレッドの数え上げはほかの試験のアクタースレッドと混ざるため。

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Barrier, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use common::copy_fixture_to_temp;
use pasta::actor::lifecycle;
use pasta::shiori_get_api;
use shiori4_abi::{SHIORI_ABI_VERSION_3, ShioriApi, ShioriInstance};
use tempfile::TempDir;

static SERIAL: Mutex<()> = Mutex::new(());

/// この試験のバイナリの `#[test]` を 1 本ずつにする（落ちた試験の毒は引き継がない）。
fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// フィクスチャを一時ディレクトリへ写し、`pasta.toml` の末尾に `extra_toml` を書き足す。
fn ghost(extra_toml: &str) -> TempDir {
    let temp = copy_fixture_to_temp("multi_instance");
    let toml = temp.path().join("pasta.toml");
    let mut text = std::fs::read_to_string(&toml).expect("read pasta.toml");
    text.push_str(extra_toml);
    std::fs::write(&toml, text).expect("write pasta.toml");
    temp
}

fn api() -> &'static ShioriApi {
    // SAFETY: shiori_get_api はこの試験のバイナリに静的にリンクされ、テーブルは static。
    unsafe { ShioriApi::from_raw(shiori_get_api(SHIORI_ABI_VERSION_3), SHIORI_ABI_VERSION_3) }
        .expect("v3 table must pass validation")
}

fn load(dir: &Path) -> Option<ShioriInstance<'static>> {
    api().load_v3(dir.to_str().expect("utf-8 path"))
}

fn get(id: &str) -> String {
    format!("GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nID: {id}\r\n\r\n")
}

/// X-Pasta-Time を `12:05:{sec}` に固定した OnSecondChange（OnTalk の機会を決定論化する）。
fn on_second_change(sec: u32) -> String {
    format!(
        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nStatus: idle\r\nID: OnSecondChange\r\n\
         Reference0: 1\r\nX-Pasta-Time: 2025-07-15T12:05:{sec:02}Z\r\n\r\n"
    )
}

fn request(inst: &mut ShioriInstance<'_>, req: &str) -> (i32, String) {
    let res = inst.request(req).expect("no ABI error");
    let body = String::from_utf8(res.body.to_vec()).expect("utf-8 body");
    (res.status, body)
}

fn read_log(ghost: &Path) -> String {
    std::fs::read_to_string(ghost.join("profile/pasta/logs/pasta.log")).unwrap_or_default()
}

/// ログのファイルに `needle` が現れるのを待つ（書き込みは非同期の書き手を通るため）。
fn wait_log_contains(ghost: &Path, needle: &str) -> String {
    let start = Instant::now();
    loop {
        let log = read_log(ghost);
        if log.contains(needle) || start.elapsed() > Duration::from_secs(10) {
            return log;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// ログの各行の量（`<時刻> <量> ...` の 2 つ目の語）。
fn levels(log: &str) -> Vec<&str> {
    log.lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .collect()
}

/// このゴーストのデバッグの待受が実際に使っているポート（`debug backend listening` の行から読む）。
fn listening_port(ghost: &Path) -> u16 {
    let log = wait_log_contains(ghost, "debug backend listening");
    let line = log
        .lines()
        .find(|l| l.contains("debug backend listening"))
        .unwrap_or_else(|| panic!("debug backend must listen:\n{log}"));
    let addr = line.split("addr=").nth(1).expect("addr field");
    let addr = addr.split_whitespace().next().expect("addr value");
    addr.rsplit(':').next().unwrap().parse().expect("port")
}

/// DAP の `pasta/reloadShiori` を送り、応答を受けるまで待つ（応答の前に kick の送り先が呼ばれる）。
fn dap_reload_shiori(port: u16) {
    let stream = TcpStream::connect(("127.0.0.1", port)).expect("connect DAP");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout");
    let mut writer = stream.try_clone().expect("clone socket");
    let body = r#"{"seq":1,"type":"request","command":"pasta/reloadShiori","arguments":{}}"#;
    write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("write DAP frame");

    let mut reader = BufReader::new(stream);
    loop {
        let mut len = 0;
        loop {
            let mut line = String::new();
            assert_ne!(
                reader.read_line(&mut line).expect("read header"),
                0,
                "DAP closed"
            );
            match line.trim_end().strip_prefix("Content-Length:") {
                Some(v) => len = v.trim().parse().expect("length"),
                None if line.trim_end().is_empty() => break,
                None => {}
            }
        }
        let mut frame = vec![0; len];
        reader.read_exact(&mut frame).expect("read body");
        if String::from_utf8_lossy(&frame).contains(r#""command":"pasta/reloadShiori""#) {
            return;
        }
    }
}

/// 2 つの版 3 のインスタンスを別々のスレッドで読み、変数とチェイントークの続きが混ざらない
/// （5.1・5.2・6.1・6.3）。両方が書いてから読み、両方が前半を出してから後半を出す。
#[test]
fn variables_and_chain_talk_do_not_mix_across_threads() {
    let _serial = serial();
    let barrier = Arc::new(Barrier::new(2));
    let threads = ["甲", "乙"].map(|mark| {
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            let temp = ghost("");
            let inst = load(temp.path());
            // 片方が読み込みに失敗しても、もう片方を barrier で待たせ続けない。
            let Some(mut inst) = inst else {
                barrier.wait();
                barrier.wait();
                return None;
            };
            request(&mut inst, &get(&format!("印{mark}")));
            barrier.wait();
            let look = request(&mut inst, &get("印を見る")).1;
            request(&mut inst, &on_second_change(0));
            let first = request(&mut inst, &on_second_change(10)).1;
            barrier.wait();
            let second = request(&mut inst, &on_second_change(20)).1;
            Some((mark, look, first, second))
        })
    });

    for thread in threads {
        let (mark, look, first, second) = thread.join().unwrap().expect("v3 load");
        let other = if mark == "甲" { "乙" } else { "甲" };
        assert!(
            look.contains(&format!("印は{mark}")) && !look.contains(other),
            "{mark}: variable must stay in its own VM: {look:?}"
        );
        assert!(
            first.contains("続きの前半") && !first.contains("続きの後半"),
            "{mark}: first OnTalk must be this VM's first beat: {first:?}"
        );
        assert!(
            second.contains("続きの後半"),
            "{mark}: chain talk must continue in this VM: {second:?}"
        );
    }
}

/// `[logging]` の量は自分のログだけに効き（5.3・5.4）、同じデバッグのポートを指した後の方は
/// 警告を出してデバッグなしで立ち上がる（5.6）。
#[test]
fn log_levels_and_debug_port_stay_per_instance() {
    let _serial = serial();
    let a = ghost("\n[logging]\nlevel = \"debug\"\n\n[debug]\nenabled = true\nport = 0\n");
    let mut inst_a = load(a.path()).expect("first instance");
    let port = listening_port(a.path());

    let b = ghost(&format!(
        "\n[logging]\nlevel = \"warn\"\n\n[debug]\nenabled = true\nport = {port}\n"
    ));
    let mut inst_b = load(b.path()).expect("the later instance must still load");
    wait_log_contains(b.path(), "debug transport bind failed");
    assert_eq!(request(&mut inst_b, &get("印を見る")).0, 200);
    assert_eq!(request(&mut inst_a, &get("印を見る")).0, 200);

    // unload で各ゴーストのロガーが外れ、書き手が書き終えてから読む。
    drop(inst_a);
    drop(inst_b);
    let log_a = read_log(a.path());
    let log_b = read_log(b.path());

    // 出し先: それぞれのファイルに相手のディレクトリの行が無い（5.3）。
    let (dir_a, dir_b) = (
        a.path().display().to_string(),
        b.path().display().to_string(),
    );
    assert!(
        log_a.contains(&dir_a) && !log_a.contains(&dir_b),
        "A:\n{log_a}"
    );
    assert!(
        log_b.contains(&dir_b) && !log_b.contains(&dir_a),
        "B:\n{log_b}"
    );

    // 量（5.4）: A は debug を書き、B は debug を書かない。
    assert!(levels(&log_a).contains(&"DEBUG"), "A logs debug:\n{log_a}");
    assert!(!levels(&log_b).contains(&"DEBUG"), "B:\n{log_b}");
    // B の pasta.toml を読む前の 2 行（読み込みの開始）は既定の量で出る。読んだ後は warn 以上だけ。
    let after_config = log_b
        .split_once("Starting pasta loader")
        .and_then(|(_, rest)| rest.split_once('\n'))
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("B loader start line:\n{log_b}"));
    assert!(
        levels(after_config)
            .iter()
            .all(|l| matches!(*l, "WARN" | "ERROR")),
        "B logs warn and above only after reading pasta.toml:\n{log_b}"
    );

    // デバッグのポート（5.6）: 後の B だけが警告を出し、A は待ち受けたまま。
    assert!(
        !log_a.contains("debug transport bind failed"),
        "A:\n{log_a}"
    );
    assert!(
        log_b.contains("debug transport bind failed; starting without debugging"),
        "B must warn about the port:\n{log_b}"
    );
    assert!(!log_b.contains("debug backend listening"), "B:\n{log_b}");
}

/// DAP の kick は、そのデバッガがつながっているインスタンスの VM だけに届く（5.5）。
#[test]
fn dap_kick_reaches_only_its_own_instance() {
    let _serial = serial();
    let debug = "\n[debug]\nenabled = true\nport = 0\n";
    let a = ghost(debug);
    let b = ghost(debug);
    let mut inst_a = load(a.path()).expect("A");
    let mut inst_b = load(b.path()).expect("B");
    // 初回の OnSecondChange は内部の時計を決めるだけ。
    request(&mut inst_a, &on_second_change(0));
    request(&mut inst_b, &on_second_change(0));

    dap_reload_shiori(listening_port(a.path()));

    let (_, resp_b) = request(&mut inst_b, &on_second_change(1));
    assert!(!resp_b.contains(r"\![reload,shiori]"), "B: {resp_b:?}");
    let (_, resp_a) = request(&mut inst_a, &on_second_change(1));
    assert!(resp_a.contains(r"\![reload,shiori]"), "A: {resp_a:?}");
}

/// 旧来の窓口のインスタンスと版 3 のインスタンスが並び、片方の unload がもう片方の応答を
/// 変えない（4.3・6.4）。
#[test]
fn legacy_and_v3_side_by_side_unload_one_keeps_the_other() {
    let _serial = serial();
    let legacy_ghost = ghost("");
    let v3_ghost = ghost("");

    assert!(lifecycle::spawn_actor(0, legacy_ghost.path().to_path_buf()));
    let mut v3 = load(v3_ghost.path()).expect("v3 beside legacy");
    lifecycle::marshal_request(&get("印甲"));
    request(&mut v3, &get("印乙"));

    let legacy_before = lifecycle::marshal_request(&get("印を見る"));
    assert!(legacy_before.contains("印は甲"), "{legacy_before:?}");
    drop(v3);
    assert_eq!(lifecycle::marshal_request(&get("印を見る")), legacy_before);

    let mut v3 = load(v3_ghost.path()).expect("v3 reload beside legacy");
    request(&mut v3, &get("印乙"));
    let (_, v3_before) = request(&mut v3, &get("印を見る"));
    assert!(v3_before.contains("印は乙"), "{v3_before:?}");
    assert!(lifecycle::teardown_actor().acked);
    assert_eq!(request(&mut v3, &get("印を見る")).1, v3_before);
}

/// 版 3 の unload が返った時点で、そのアクタースレッドは終わっている（4.4）。
/// `SHIORI.unload` を 0.3 秒回らせ、終わりを待たずに返ればスレッドが残るようにする。
#[cfg(any(windows, target_os = "linux"))]
#[test]
fn v3_unload_leaves_no_actor_thread() {
    let _serial = serial();
    let before = actor_thread_ids();
    let temp = copy_fixture_to_temp("v3_probe");
    let mut inst = load(temp.path()).expect("v3 load");
    assert_eq!(request(&mut inst, &get("OnProbeSlowUnload")).0, 204);
    let ours: Vec<u32> = actor_thread_ids()
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect();
    assert_eq!(ours.len(), 1, "one actor thread per instance: {ours:?}");

    drop(inst);
    assert!(
        !actor_thread_ids().contains(&ours[0]),
        "actor thread must be joined before unload returns"
    );
}

/// このプロセスの `pasta-actor` という名のスレッドの ID。
#[cfg(windows)]
fn actor_thread_ids() -> Vec<u32> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE, LocalFree};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcessId, GetThreadDescription, OpenThread, THREAD_QUERY_LIMITED_INFORMATION,
    };

    let mut ids = Vec::new();
    // SAFETY: 読み取りだけの Win32 の呼び出し。開いたハンドルと説明の文字列はここで手放す。
    unsafe {
        let pid = GetCurrentProcessId();
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        assert_ne!(snapshot, INVALID_HANDLE_VALUE, "thread snapshot");
        let mut entry: THREADENTRY32 = std::mem::zeroed();
        entry.dwSize = size_of::<THREADENTRY32>() as u32;
        let mut more = Thread32First(snapshot, &mut entry);
        while more != 0 {
            if entry.th32OwnerProcessID == pid {
                let thread = OpenThread(THREAD_QUERY_LIMITED_INFORMATION, 0, entry.th32ThreadID);
                if !thread.is_null() {
                    let mut desc = std::ptr::null_mut();
                    if GetThreadDescription(thread, &mut desc) >= 0 && !desc.is_null() {
                        let len = (0..).take_while(|&i| *desc.add(i) != 0).count();
                        if String::from_utf16_lossy(std::slice::from_raw_parts(desc, len))
                            == "pasta-actor"
                        {
                            ids.push(entry.th32ThreadID);
                        }
                        LocalFree(desc.cast());
                    }
                    CloseHandle(thread);
                }
            }
            more = Thread32Next(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
    }
    ids
}

/// このプロセスの `pasta-actor` という名のスレッドの ID。
#[cfg(target_os = "linux")]
fn actor_thread_ids() -> Vec<u32> {
    std::fs::read_dir("/proc/self/task")
        .expect("/proc/self/task")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let comm = std::fs::read_to_string(entry.path().join("comm")).ok()?;
            if comm.trim_end() != "pasta-actor" {
                return None;
            }
            entry.file_name().to_str()?.parse().ok()
        })
        .collect()
}
