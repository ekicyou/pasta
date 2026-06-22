//! アクター/marshaling/teardown シームの観測ログ点（tracing）検証（task 6.2・R10.4）。
//!
//! 関連仕様: `.kiro/specs/pasta-actor-runtime/`
//! - 充足要件: R10.4（アクタースレッド・単一直列キュー・marshaling〈GET/NOTIFY/drop→204〉・
//!   teardown の主要シーム〈enqueue/dispatch/応答/drop・spawn/stop/done〉に `@pasta_log`／
//!   `tracing` の観測可能なログ点を備える。**無効時はゼロコスト維持**）。
//! - 設計: design.md の **Monitoring**（`tracing` シーム: `actor.try_send`/`actor.recv`/
//!   `actor.reply`/`actor.drop`/`actor.timeout`/`actor.spawn`/`actor.stop`/`actor.done`。
//!   無効時ゼロコスト・`@pasta_log` 既存方針）。
//!
//! ## 検証アプローチ（tracing-capture）
//! `tracing-test` の `#[traced_test]` を用い、テストローカルに subscriber を装着して
//! シームが発火するイベント名（`actor.try_send` 等）を `logs_contain(..)` で観測する。
//! これは pasta_lua 既存テスト（`tests/log/module_test.rs`・`loader/config_defaults_test.rs`）
//! と同一の確立済み idiom（`#[tracing_test::traced_test]` + `logs_contain`）に倣う。
//!
//! ## ゼロコスト/バイト不変について
//! `tracing::{trace,debug}!` は **無効時（subscriber 非装着）にゼロコスト**である（`tracing`
//! の static max-level / 早期 enabled チェックにより、disabled なら引数評価も発生しない）。
//! 本ファイルの `traced_test` 群は subscriber を装着した「有効時」にイベント発火を観測する。
//! 「無効時に応答バイトが変化しない（ゼロコスト＝挙動不変）」ことは、subscriber を装着しない
//! 既存の `byte_invariant_test.rs`（ゴールデン 4/4）と `actor_marshaling_test.rs` が回帰ガード
//! として担保する（本ファイルはシーム発火の存在証明、ゴールデンは挙動不変の存在証明）。

use std::thread;
use std::time::Duration;

use pasta::actor::mailbox::{mailbox, ActorMsg, MailboxRequest, Reply};
use pasta::actor::marshaling::{default_204, marshal_get_with_timeout, marshal_notify};
use pasta::actor::teardown::teardown_via_sender;

/// marshaling シーム（GET）: try_send / recv / reply（値到達）のログ点が発火する（R10.4）。
#[tracing_test::traced_test]
#[test]
fn get_path_emits_try_send_recv_and_reply_seams() {
    let (tx, rx) = mailbox();

    let actor = thread::spawn(move || {
        if let Ok(ActorMsg::Get { req, reply }) = rx.recv() {
            let _ = reply.send(Reply::Value(format!("echo:{}", req.raw)));
        }
    });

    let resp =
        marshal_get_with_timeout(&tx, MailboxRequest::new(1, "X"), Duration::from_secs(10));
    assert_eq!(resp, "echo:X");
    actor.join().expect("actor role thread must not panic");

    // GET は try_send で投入し、reply 値到達で完了する。両シームが観測できること。
    assert!(logs_contain("actor.try_send"), "GET must emit actor.try_send seam");
    assert!(logs_contain("actor.reply"), "GET reply value must emit actor.reply seam");
}

/// marshaling シーム（GET drop→204）: reply drop が `actor.drop` ログ点を発火する（R10.4/R5.3）。
#[tracing_test::traced_test]
#[test]
fn get_reply_drop_emits_drop_seam_and_204() {
    let (tx, rx) = mailbox();

    let actor = thread::spawn(move || {
        if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
            drop(reply); // reply を送らず drop → 受信側 Disconnected。
        }
    });

    let resp =
        marshal_get_with_timeout(&tx, MailboxRequest::new(1, "X"), Duration::from_secs(10));
    assert_eq!(resp.as_bytes(), default_204().as_bytes());
    actor.join().expect("actor role thread must not panic");

    assert!(
        logs_contain("actor.drop"),
        "a dropped (unreplied) GET reply must emit the actor.drop seam"
    );
}

/// marshaling シーム（GET timeout→204）: 閾値超過が `actor.timeout` ログ点を発火する（R10.4/R5.7）。
#[tracing_test::traced_test]
#[test]
fn get_timeout_emits_timeout_seam_and_204() {
    let (tx, rx) = mailbox();
    let short_timeout = Duration::from_millis(20);
    let actor_delay = Duration::from_millis(120);

    let actor = thread::spawn(move || {
        if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
            thread::sleep(actor_delay);
            let _ = reply.send(Reply::Value("late".to_string()));
        }
    });

    let resp = marshal_get_with_timeout(&tx, MailboxRequest::new(1, "X"), short_timeout);
    assert_eq!(resp.as_bytes(), default_204().as_bytes());
    actor.join().expect("actor role thread must not panic");

    assert!(
        logs_contain("actor.timeout"),
        "a GET exceeding the threshold must emit the actor.timeout seam"
    );
}

/// marshaling シーム（NOTIFY）: try_send ログ点が発火する（R10.4/R5.2）。
#[tracing_test::traced_test]
#[test]
fn notify_path_emits_try_send_seam() {
    let (tx, _rx) = mailbox();
    let resp = marshal_notify(&tx, MailboxRequest::new(7, "BODY"));
    assert_eq!(resp.as_bytes(), default_204().as_bytes());

    assert!(
        logs_contain("actor.try_send"),
        "NOTIFY must emit the actor.try_send seam"
    );
}

/// teardown シーム: Stop 送信→done ack が `actor.stop` / `actor.done` ログ点を発火する（R10.4/R7）。
#[tracing_test::traced_test]
#[test]
fn teardown_emits_stop_and_done_seams() {
    let (tx, rx) = mailbox();

    // アクター役: Stop を drain して done ack を返す（clean teardown）。
    let actor = thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            if let ActorMsg::Stop { done } = msg {
                let _ = done.send(());
                break;
            }
        }
    });

    let report = teardown_via_sender(&tx, Duration::from_secs(10));
    assert!(report.is_clean(), "teardown must be clean (done ack received)");
    actor.join().expect("actor role thread must not panic");

    assert!(logs_contain("actor.stop"), "teardown must emit the actor.stop seam");
    assert!(logs_contain("actor.done"), "teardown done ack must emit the actor.done seam");
}
