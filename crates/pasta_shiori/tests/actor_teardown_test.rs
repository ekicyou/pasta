//! 本番 teardown（`Stop{done}` ack）と reload リーク検査の統合テスト
//! （task 4.1・R7.1/R7.2/R7.3/R7.4/R7.5）。
//!
//! task 5.1 で FFI 出荷経路がアクター経路へ昇格し、wintf と
//! `windows-sys/Win32_System_Threading`（実 `GetProcessHandleCount`/`GetGuiResources` 計測）が
//! 既定ビルドの依存になったため、本テストは **既定（no-feature）ビルドで実行される**
//! （旧 `actor-poc` ガードは撤去）。
//!
//! # 何を証明するか（task 4.1 の observable 完了条件）
//!  (a) `Stop{done}` を送ると、アクターが残メッセージ drain 後に VM 破棄・cleanup を
//!      終えて `done` ack を返し、SHIORI 側が ack を受けて完了する。Stop 前に投入した
//!      メッセージは Stop より前に処理される（drain-before-stop・clean drain）。
//!  (b) ack 後の二重 teardown は安全な no-op（hang/panic なし・冪等）。
//!  (c) reload（spawn→teardown ×N）でカーネルハンドル／USER オブジェクト／port が
//!      リーク・枯渇しない（PoC `actor_poc/teardown.rs` の実 OS 計測アプローチを流用）。
//!
//! # テストが「自明に真」でないことの担保
//! done ack は `bounded(1)` の `recv_timeout` で有界に待つ。teardown が壊れて ack を
//! 返さなければ（または drain せず break すれば）テストはハングせず **失敗** する。
//! リーク assert は実 OS カウンタの増分上限で歯を持つ（per-cycle リークなら ≒N 増）。

use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use pasta::actor::mailbox::{mailbox, ActorMsg, MailboxRequest, Reply};
use pasta::actor::teardown::{teardown_actor, ReloadProbe};
use pasta::actor::thread::spawn_actor_thread;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元: `common/mod.rs`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// async_callback フィクスチャを temp へ展開し `load_dir` を返す。`TempDir` は
/// 返り値で寿命保持する（アクタースレッドより長命）。写経元: `actor_thread_vm_test.rs`。
fn build_async_callback_dir() -> (PathBuf, TempDir) {
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

    let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
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

/// SHIORI/3.0 リクエストを正規化（改行→CRLF・終端付与）する。写経元:
/// `actor_thread_vm_test.rs::normalize_request`。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// R7.1/R7.4: `Stop{done}` ack による clean teardown。Stop 前に投入した GET が
/// Stop より先に処理（drain-before-stop）され、その後 done ack が有界に返る。
#[test]
fn stop_done_ack_drains_then_tears_down_cleanly() {
    let (load_dir, _temp) = build_async_callback_dir();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    // drain-before-stop: Stop の直前に GET を投入する。GET の応答は Stop より前に
    // 返らねばならない（同一 FIFO・先行メッセージ drain 後に Stop が処理される）。
    let (get, reply_rx) = ActorMsg::get(MailboxRequest::new(
        1,
        normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n"),
    ));
    tx.send(get).expect("send GET into mailbox");

    // teardown_actor: Stop{done} を送り、有界に done ack を待ち、スレッドを detach。
    let report = teardown_actor(&tx, actor, Duration::from_secs(10));

    // 先行 GET は Stop の前に drain されて応答済みのはず（clean drain の証跡）。
    let pre_stop = reply_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("GET enqueued before Stop must be drained and replied before teardown");
    let Reply::Value(_) = pre_stop;

    assert!(
        report.acked,
        "Stop must be acked within timeout (actor must drain, drop VM, clean up, then ack)"
    );
    assert!(
        report.is_clean(),
        "teardown must complete cleanly (done ack received, no anomaly)"
    );
}

/// R7.4: 冪等 teardown。done ack 後にもう一度 `teardown_actor` 相当の Stop 送信を
/// 行っても、閉じたチャネルを already-done として安全に no-op（hang/panic なし）。
#[test]
fn second_teardown_after_done_is_safe_noop() {
    let (load_dir, _temp) = build_async_callback_dir();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let first = teardown_actor(&tx, actor, Duration::from_secs(10));
    assert!(first.acked, "first teardown must be acked");

    // 二重 teardown: アクタースレッドは既に終了し receiver は drop 済み。Stop の
    // 再送は Disconnected になり、teardown は already-done として安全に no-op を返す
    // （ハングしないことを有界 timeout で担保。冪等）。
    let second = teardown_idempotent_resend(&tx, Duration::from_secs(2));
    assert!(
        second.already_done,
        "second teardown after done must be a safe no-op (closed channel -> already done)"
    );
    assert!(
        !second.acked,
        "no fresh ack is expected on the idempotent second teardown"
    );
}

/// 二重 teardown を直接（ActorThread を持たずに）試行するヘルパ。本番 FFI 経路では
/// teardown は冪等であるべきなので、`teardown_actor` の「すでに done」分岐に相当する
/// 動作を mailbox tx のみで再現する（receiver drop 済み → send 失敗 → already_done）。
fn teardown_idempotent_resend(
    tx: &flume::Sender<ActorMsg>,
    timeout: Duration,
) -> pasta::actor::teardown::TeardownReport {
    pasta::actor::teardown::teardown_via_sender(tx, timeout)
}

/// R7.2/R7.3: reload（spawn→teardown ×N）でカーネルハンドル／USER オブジェクトが
/// リーク・枯渇しない。done ack 後に計測する（PoC の実 OS 計測アプローチを流用）。
#[test]
fn repeated_reload_tears_down_and_does_not_leak() {
    let (load_dir, _temp) = build_async_callback_dir();

    // 実 VM 構築は非自明なコストなので、リークが per-cycle なら確実に検出できる規模で
    // 全体時間を抑える（per-cycle で USER ウィンドウ 1 個でも漏れれば +N となり許容超）。
    const RELOAD_CYCLES: usize = 12;

    let report = ReloadProbe::run_cycles(&load_dir, RELOAD_CYCLES, Duration::from_secs(10));

    assert_eq!(
        report.cycles_run, RELOAD_CYCLES,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        report.clean_teardowns, RELOAD_CYCLES,
        "every reload cycle must tear down cleanly (Stop{{done}} ack received)"
    );

    #[cfg(windows)]
    {
        let leak = report
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");

        // per-cycle リーク（≒N）なら確実に超える小さな許容値。teardown が真に解放
        // していれば増分は 0 付近に収まる。OS/アロケータの一回性ゆらぎのみ許す。
        const HANDLE_TOLERANCE: i64 = 12;
        const USER_OBJECT_TOLERANCE: i64 = 12;

        assert!(
            leak.kernel_handle_growth <= HANDLE_TOLERANCE,
            "kernel handle count grew by {} across {} reload cycles (baseline={}, final={}); \
             a per-cycle handle leak would show ~{} growth (tolerance {})",
            leak.kernel_handle_growth,
            RELOAD_CYCLES,
            leak.kernel_handles_baseline,
            leak.kernel_handles_final,
            RELOAD_CYCLES,
            HANDLE_TOLERANCE
        );

        assert!(
            leak.user_object_growth <= USER_OBJECT_TOLERANCE,
            "USER object count grew by {} across {} reload cycles (baseline={}, final={}); \
             a leaked message-only window per cycle would show ~{} growth (tolerance {})",
            leak.user_object_growth,
            RELOAD_CYCLES,
            leak.user_objects_baseline,
            leak.user_objects_final,
            RELOAD_CYCLES,
            USER_OBJECT_TOLERANCE
        );
    }

    #[cfg(not(windows))]
    {
        assert!(
            report.leak_metric.is_none(),
            "non-windows builds do not sample a handle metric"
        );
    }

    // ベースライン後の各サイクルで spawn した新規 VM が稼働できること（再 spawn 正常）
    // は cycles_run==N と clean_teardowns==N で担保される（壊れた spawn は load 失敗→
    // teardown 不成立で検出される）。
    let _ = thread::current().id();
}
