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
///
/// # なぜ「絶対増分 ≤ 固定許容」では不十分か（flake の根本原因）
/// `GetProcessHandleCount`/`GetGuiResources` は **プロセス全体**のカウンタである。
/// `cargo test --all` の並列実行では、同一テストバイナリ内の他テストや同時実行中の
/// 活動が計測窓の途中でこのカウンタを上下させ、単発の before/after 差分（絶対増分）に
/// **有界だが非ゼロのノイズ**を乗せる。ノイズは本質的にサイクル数 N に依存しない
/// ほぼ一定のオフセットだが、固定許容値を一時的に超えて偽陽性（flake）を生む。
///
/// # 本テストの判定（signal-vs-noise / slope 法）
/// 真の per-cycle リークは増分が N に **線形比例**して増える（growth ≈ L·N）。一方
/// 並列ノイズは N に比例しない（growth ≈ noise、N 非依存）。そこで小さな N と 3×N の
/// 2 水準で計測し、**増分の傾き（per-cycle 増分）が大きい N で増えていない**ことを
/// assert する:
///   - L=0（リークなし）: growth(N)≈growth(3N)≈noise。差は N に依存せず小さい。
///   - L≥1（per-cycle リーク）: growth(3N)-growth(N) ≈ L·2N。N=6 なら ≥12 となり、
///     N 非依存の小さなノイズ許容を確実に超える → 検出される。
///
/// これにより「一定のノイズオフセット」には頑健でありながら、per-cycle リークには
/// 歯を残す（リーク検出力を保ったまま決定論化する）。
#[test]
fn repeated_reload_tears_down_and_does_not_leak() {
    let (load_dir, _temp) = build_async_callback_dir();

    // 小さい水準 N と大きい水準 3×N。N と 3N の傾き比較で per-cycle リークを増幅して
    // 検出する。per-cycle リーク 1 個でも slope ≈ L·2N = 12（N=6）となり、N 非依存の
    // 並列ノイズ差（差し引きで概ね相殺）と明確に分離できる。
    const N_SMALL: usize = 6;
    const N_LARGE: usize = N_SMALL * 3; // 18

    // 各 run_cycles は内部でウォームアップ→baseline→N サイクル→final を完結させるため、
    // 連続呼び出しは互いに独立・公平（それぞれ自前の baseline を採る）。
    let small = ReloadProbe::run_cycles(&load_dir, N_SMALL, Duration::from_secs(10));
    let large = ReloadProbe::run_cycles(&load_dir, N_LARGE, Duration::from_secs(10));

    assert_eq!(
        small.cycles_run, N_SMALL,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        large.cycles_run, N_LARGE,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        small.clean_teardowns, N_SMALL,
        "every reload cycle must tear down cleanly (Stop{{done}} ack received)"
    );
    assert_eq!(
        large.clean_teardowns, N_LARGE,
        "every reload cycle must tear down cleanly (Stop{{done}} ack received)"
    );

    #[cfg(windows)]
    {
        let small_leak = small
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");
        let large_leak = large
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");

        // slope（傾き）法の許容: per-cycle リークが 1 でもあれば growth(3N)-growth(N)
        // ≈ L·2N = 12（N=6）となるので、これを確実に下回る許容を置く。N 非依存の並列
        // ノイズは N と 3N の双方の窓に同程度乗るため差し引きで概ね相殺され、ここに
        // 残るのは小さなゆらぎのみ。8 を超えたら「N に比例して増えている＝per-cycle
        // リーク」と判定する（per-cycle=1 の 12 は確実に超え、ノイズ差には触れない位置）。
        const SLOPE_TOLERANCE: i64 = 8;

        let handle_slope = large_leak.kernel_handle_growth - small_leak.kernel_handle_growth;
        let user_slope = large_leak.user_object_growth - small_leak.user_object_growth;

        assert!(
            handle_slope <= SLOPE_TOLERANCE,
            "kernel handle growth scales with cycle count: growth({}) = {} but growth({}) = {} \
             (delta = {} > tolerance {}); a per-cycle handle leak of L would make this delta \
             ~L*2N = ~{}. small(baseline={}, final={}), large(baseline={}, final={})",
            N_SMALL,
            small_leak.kernel_handle_growth,
            N_LARGE,
            large_leak.kernel_handle_growth,
            handle_slope,
            SLOPE_TOLERANCE,
            2 * N_SMALL,
            small_leak.kernel_handles_baseline,
            small_leak.kernel_handles_final,
            large_leak.kernel_handles_baseline,
            large_leak.kernel_handles_final,
        );

        assert!(
            user_slope <= SLOPE_TOLERANCE,
            "USER object growth scales with cycle count: growth({}) = {} but growth({}) = {} \
             (delta = {} > tolerance {}); a leaked message-only window per cycle would make this \
             delta ~2N = ~{}. small(baseline={}, final={}), large(baseline={}, final={})",
            N_SMALL,
            small_leak.user_object_growth,
            N_LARGE,
            large_leak.user_object_growth,
            user_slope,
            SLOPE_TOLERANCE,
            2 * N_SMALL,
            small_leak.user_objects_baseline,
            small_leak.user_objects_final,
            large_leak.user_objects_baseline,
            large_leak.user_objects_final,
        );

        // 注意: 絶対増分 ≤ 固定許容 の assert は **意図的に置かない**。それこそが旧
        // テストの flake 源（プロセス全体カウンタへ並列ノイズが一定オフセットとして
        // 乗る）だったため。リーク検出力は上の slope（N 非依存ノイズに頑健・per-cycle
        // リークには L·2N で確実に反応）で担保する。
    }

    #[cfg(not(windows))]
    {
        assert!(
            small.leak_metric.is_none() && large.leak_metric.is_none(),
            "non-windows builds do not sample a handle metric"
        );
    }

    // ベースライン後の各サイクルで spawn した新規 VM が稼働できること（再 spawn 正常）
    // は cycles_run==N と clean_teardowns==N で担保される（壊れた spawn は load 失敗→
    // teardown 不成立で検出される）。
    let _ = thread::current().id();
}
