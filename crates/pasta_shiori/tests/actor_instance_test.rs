//! 芯の型 [`ActorInstance`] の後始末の決まりの試験（仕様 `shiori4-v3-portable`・4.2・4.4・7.6）。
//!
//! 芯の型を直に起こし、`TeardownReport` の `acked`・`joined` で後始末の決まりを確かめる:
//! - 「ack の後に join」（[`JoinPolicy::JoinAfterAck`]）は ack あり・join 済み（4.4）。
//! - 「手放す」（[`JoinPolicy::Detach`]）は ack あり・join なし。
//! - ack が上限の中に来なければ ack なし・join なしで返り、手放しの警告を出す（4.2）。
//!
//! - 同じディレクトリの 2 つ目のインスタンスは使用中の誤りになり、占有している側のログに理由を出す（6.2）。
//! - 旧来の窓口で、片付けが打ち切りに終わった直後に同じディレクトリを読み直すと占有を引き継ぎ、
//!   手放した古いスレッドがあとで片付けを終えても新しいインスタンスの登録と占有が残る（7.1・7.2）。
//!
//! ゴーストは `tests/fixtures/v3_probe` を試験ごとに一時ディレクトリへ写して使う。

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use common::copy_fixture_to_temp;
use pasta::actor::instance::{ActorInstance, JoinPolicy, SpawnError};
use pasta::actor::lifecycle;
use pasta_lua::GlobalLoggerRegistry;

fn request(id: &str) -> String {
    format!("GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: {id}\r\n\r\n")
}

fn notify(id: &str) -> String {
    format!("NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: {id}\r\n\r\n")
}

/// ログのファイルに `needle` が現れるのを待つ（書き込みは非同期の書き手を通るため）。
fn wait_log_contains(ghost: &Path, needle: &str, deadline: Duration) -> String {
    let path = ghost.join("profile/pasta/logs/pasta.log");
    let start = Instant::now();
    loop {
        let log = std::fs::read_to_string(&path).unwrap_or_default();
        if log.contains(needle) || start.elapsed() > deadline {
            return log;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// 試験用のゴーストが読めて、呼ばれた回数と壊れた開始行をそのまま返すこと。
/// その後の「ack の後に join」の後始末が、ack あり・join 済みで返ること（4.4）。
#[test]
fn shutdown_join_after_ack_is_acked_and_joined() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let instance = ActorInstance::spawn(0, ghost.path().to_path_buf()).expect("spawn");
    assert!(instance.loaded(), "v3_probe must load");

    instance.notify(&notify("OnSomething"));
    let count = instance.get(&request("OnProbeCount"));
    assert!(
        count.contains("Value: 2\r\n"),
        "count must include the NOTIFY and this GET: {count:?}"
    );
    let broken = instance.get(&request("OnProbeBadStatus"));
    assert!(
        broken.starts_with("SHIORI/3.0 2x0 OK\r\n"),
        "broken status line must come back as Lua returned it: {broken:?}"
    );

    let report = instance.shutdown(JoinPolicy::JoinAfterAck);
    assert!(
        report.acked && report.joined && !report.already_done,
        "{report:?}"
    );
}

/// 「手放す」の後始末は、ack あり・join なしで返る。
#[test]
fn shutdown_detach_is_acked_and_not_joined() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let instance = ActorInstance::spawn(0, ghost.path().to_path_buf()).expect("spawn");
    assert!(instance.loaded(), "v3_probe must load");

    let report = instance.shutdown(JoinPolicy::Detach);
    assert!(
        report.acked && !report.joined && !report.already_done,
        "{report:?}"
    );
}

/// 回り続ける ID を NOTIFY で積み、上限を 100 ミリ秒に注入した後始末は、
/// ack なし・join なしで返り、手放しの警告（`seam = "instance.abandon"`）を出す（4.2）。
#[test]
fn shutdown_times_out_on_spinning_actor_and_warns_abandon() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let instance = ActorInstance::spawn(0, ghost.path().to_path_buf()).expect("spawn");
    assert!(instance.loaded(), "v3_probe must load");

    instance.notify(&notify("OnProbeSpin"));
    let started = Instant::now();
    let report =
        instance.shutdown_with_timeout(JoinPolicy::JoinAfterAck, Duration::from_millis(100));
    let elapsed = started.elapsed();

    assert!(
        !report.acked && !report.joined && !report.already_done,
        "{report:?}"
    );
    assert!(
        elapsed < Duration::from_millis(900),
        "shutdown must give up near the injected limit, not wait for the spin: {elapsed:?}"
    );

    // ログの書き込みは非同期なので、警告の行が現れるまで読み直して待つ。
    let log = wait_log_contains(ghost.path(), "instance.abandon", Duration::from_secs(10));
    assert!(
        log.contains("instance.abandon"),
        "abandon warning must reach the ghost's log:\n{log}"
    );
}

/// 同じディレクトリで 2 つ目のインスタンスを起こすと使用中の誤りになり、占有している側の
/// ログに理由の行（`seam = "instance.claim"`）が出る（6.2・2.5）。1 つ目は動き続ける。
#[test]
fn second_instance_on_same_dir_is_dir_in_use_and_logged() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let first = ActorInstance::spawn(0, ghost.path().to_path_buf()).expect("spawn");
    assert!(first.loaded(), "v3_probe must load");

    let second = ActorInstance::spawn(0, ghost.path().to_path_buf());
    assert!(
        matches!(second, Err(SpawnError::DirInUse)),
        "second instance on the same dir must be refused: {:?}",
        second.err()
    );

    let log = wait_log_contains(ghost.path(), "instance.claim", Duration::from_secs(10));
    assert!(
        log.contains("instance.claim"),
        "the refusal must reach the occupying side's log:\n{log}"
    );
    let count = first.get(&request("OnProbeCount"));
    assert!(
        count.starts_with("SHIORI/3.0 200 OK\r\n"),
        "the first instance must keep running: {count:?}"
    );

    let report = first.shutdown(JoinPolicy::JoinAfterAck);
    assert!(report.acked && report.joined, "{report:?}");
    // 片付けの後は占有が外れ、同じディレクトリをまた起こせる。
    let again = ActorInstance::spawn(0, ghost.path().to_path_buf()).expect("respawn after unload");
    assert!(again.shutdown(JoinPolicy::JoinAfterAck).joined);
}

/// 旧来の窓口で、回り続けるアクターの片付けが打ち切りに終わった直後に同じディレクトリを読むと
/// 成功し（占有を引き継ぐ）、手放された古いスレッドがあとで片付けを終えても、新しいインスタンスの
/// ログの登録と占有が残る（7.1・7.2）。
#[test]
fn legacy_reload_after_timed_out_teardown_takes_over_claim() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let dir = ghost.path().to_path_buf();
    let registry = GlobalLoggerRegistry::instance();

    assert!(lifecycle::spawn_actor(0, dir.clone()), "v3_probe must load");
    lifecycle::marshal_request(&notify("OnProbeSpin"));

    // 前の片付けは 100 ミリ秒で打ち切られ、手放した古いスレッドの VM がまだ占有を持つ。
    assert!(
        lifecycle::spawn_actor_with_teardown_timeout(0, dir.clone(), Duration::from_millis(100)),
        "reloading the same dir right after a timed-out teardown must succeed"
    );
    let log = wait_log_contains(ghost.path(), "claim taken over", Duration::from_secs(10));
    let taken_at = log
        .find("claim taken over")
        .unwrap_or_else(|| panic!("the takeover must be logged:\n{log}"));

    // 古いスレッドが回り終えて VM を片付けるのを待つ（古い札での解放は何もしない）。
    let start = Instant::now();
    loop {
        let log = std::fs::read_to_string(ghost.path().join("profile/pasta/logs/pasta.log"))
            .unwrap_or_default();
        if log[taken_at..].contains("Unregistering logger") {
            break;
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "the abandoned thread must finish its cleanup:\n{log}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }

    assert!(
        registry.get(&dir).is_some(),
        "the new instance's logger registration must remain"
    );
    assert!(
        registry.try_claim(dir.clone()).is_none(),
        "the new instance's claim must remain"
    );
    let count = lifecycle::marshal_request(&request("OnProbeCount"));
    assert!(
        count.starts_with("SHIORI/3.0 200 OK\r\n"),
        "the new instance must answer: {count:?}"
    );

    let report = lifecycle::teardown_actor();
    assert!(report.acked, "{report:?}");
    assert!(
        registry.get(&dir).is_none(),
        "unloading the new instance must release the registration"
    );
}
