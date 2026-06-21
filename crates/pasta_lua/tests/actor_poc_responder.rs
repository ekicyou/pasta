//! Responder drop→204 ガードの実証テスト（R3.1 / R3.2 / R3.3 / R3.4・task 3.1）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切
//! 汚染しない（R7.1/R7.2/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 3.1）:
//! 「未 reply drop と panic 注入の双方で SSP 側が 204 を受け取り無限待機しない
//!   単体テストが緑。」
//!
//! # panic semantics（重要・tasks.md Implementation Notes 写し）
//!
//! drop→204-on-panic は **unwinding** に依存する。root `Cargo.toml` は
//! `[profile.release] panic = "abort"` を設定するが、`cargo test` は dev/test
//! profile（既定 `panic = "unwind"`）で動くため、panic 時にも Drop ガードが走る。
//! 本テストは既定の `cargo test`（NOT `--release`）で走らせること。
//! `panic = "abort"`（release）では巻き戻らないため、この保証は test/unwind
//! profile でのみ検証される——これは `pasta-actor-runtime` へ申し送る feasibility
//! caveat である。
//!
//! # 「無限待機しない」を観測可能にする設計（正直さ）
//!
//! ガードが無ければ Responder の drop で内部 `Sender` も drop され、SSP 側の
//! `recv()` は `Err(RecvError)`（disconnected）を返す。テストは `recv()` の戻り値が
//! `Ok(PocResponse::NoContent204)` であることを **明示的に** 要求し、`Err`（=ガード
//! 喪失で何も送られず切断）を **失敗** として扱う。これにより「ガードを外すと
//! テストが赤になる」性質を担保し、ハングではなく即座の検出に変換する。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::responder::{PocResponse, Responder};
use std::sync::mpsc;
use std::time::Duration;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元:
/// `crates/pasta_lua/tests/common/mod.rs:26-32`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// R3（不変条件）: 正常 `reply(value)` で受信側は値を 1 回だけ受け取り、204 は
/// 二重送信されない（exactly-once）。
#[test]
fn reply_delivers_value_exactly_once_and_no_204() {
    let (responder, rx) = Responder::new();

    responder.reply(PocResponse::Value(42));

    // 値が届く。
    let first = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("reply value must be received, not a hang");
    assert_eq!(
        first,
        PocResponse::Value(42),
        "normal reply must deliver the exact value"
    );

    // それ以上の応答（204 二重送信）は無い。reply が self を消費して Drop ガードを
    // 撃たないことの証跡: 次の recv は切断（送信端が消えた）で終わる。
    match rx.recv_timeout(Duration::from_millis(200)) {
        Err(mpsc::RecvTimeoutError::Disconnected) => { /* 期待どおり: 二重送信なし */ }
        Ok(extra) => panic!(
            "reply must terminate exactly once; got an unexpected second response: {extra:?}"
        ),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("sender should have been dropped after reply (disconnected), but channel is still open")
        }
    }
}

/// R3.1: 未 reply のまま Responder を drop すると、受信側（SSP 側）は無限待機に
/// 陥らず 204 フォールバックを受け取る。
#[test]
fn unreplied_drop_sends_204_not_a_hang() {
    let (responder, rx) = Responder::new();

    // reply せずに drop。Drop ガードが 204 を撃つはず。
    drop(responder);

    let resp = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("unreplied drop must yield a 204 (guard fired), not a disconnected hang");
    assert_eq!(
        resp,
        PocResponse::NoContent204,
        "an unreplied Responder drop must auto-send 204 (success/no-content)"
    );

    // 204 は 1 回だけ。以降は切断。
    assert!(
        matches!(
            rx.recv_timeout(Duration::from_millis(200)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ),
        "the drop→204 guard must fire exactly once"
    );
}

/// R3.2: GET 処理中に panic して応答経路が巻き戻り（drop）された場合でも、当該
/// GET が 204 として終結し SSP 側スレッドが解放される。
///
/// `catch_unwind` で panic を封じ込め、unwind の過程で Responder の Drop ガードが
/// 走ったこと（=受信側が 204 を得たこと）を assert する。
#[test]
fn panic_before_reply_unwinds_drop_guard_to_204() {
    let (responder, rx) = Responder::new();

    // panic を封じ込めつつ、Responder を unwind 経路に巻き込む。
    let result = std::panic::catch_unwind(move || {
        let _held = responder; // panic で unwind されると _held が drop → Drop ガード発火
        panic!("simulated executor-side panic before replying");
    });

    assert!(
        result.is_err(),
        "the injected panic must have been caught (and contained)"
    );

    // unwind 中に Drop ガードが 204 を撃ち、SSP 側が解放される。ハング（disconnected）
    // なら panic で responder が消えただけで 204 が来ていない＝ガード喪失で失敗。
    let resp = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("panic-unwound Responder must release the SSP side with a 204, not a hang");
    assert_eq!(
        resp,
        PocResponse::NoContent204,
        "panic before reply must terminate the GET as 204 (drop guard fired during unwind)"
    );
}

/// R3.2 / R3.3: panic が **別スレッド**（アクタースレッド相当）で起きても、SSP 側
/// （受信スレッド）は無限待機に陥らず 204 で解放される——デッドロック経路の
/// 原理的消滅をスレッド分離下で実証する。
#[test]
fn panic_on_worker_thread_releases_blocked_receiver_with_204() {
    let (responder, rx) = Responder::new();

    // アクタースレッド相当で responder を保持しつつ panic させる。panic は
    // join 時に観測できるよう、worker 内で catch せずスレッド境界へ伝播させる。
    let worker = std::thread::spawn(move || {
        let _held = responder;
        panic!("worker-thread panic before reply");
    });

    // SSP 側（このスレッド）は block-on-reply 相当でブロック受信する。ガードが
    // 機能すれば worker の unwind で 204 が届く。機能しなければ disconnected。
    let resp = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("a panicking worker thread must not leave the SSP receiver hanging; expected 204");
    assert_eq!(
        resp,
        PocResponse::NoContent204,
        "worker-thread panic before reply must release the receiver with 204"
    );

    // worker は panic した（join は Err）。これも観測して握り潰さない。
    assert!(
        worker.join().is_err(),
        "the worker thread must have actually panicked (proving the unwind path was exercised)"
    );
}

/// R3.3（取りこぼしなし）: Responder は「reply 1 回」または「drop→204」の
/// いずれかで必ず終結する。reply 後の drop が追加で 204 を撃たないこと
/// （exactly-once）を、受信総数で確認する。
#[test]
fn responder_terminates_exactly_once_never_both() {
    // ケース A: reply のみ → 値 1 件のみ。
    {
        let (responder, rx) = Responder::new();
        responder.reply(PocResponse::Value(7));
        let all: Vec<PocResponse> = collect_until_disconnect(&rx);
        assert_eq!(
            all,
            vec![PocResponse::Value(7)],
            "reply path must terminate with exactly one value response and nothing else"
        );
    }

    // ケース B: drop のみ → 204 を 1 件のみ。
    {
        let (responder, rx) = Responder::new();
        drop(responder);
        let all: Vec<PocResponse> = collect_until_disconnect(&rx);
        assert_eq!(
            all,
            vec![PocResponse::NoContent204],
            "drop path must terminate with exactly one 204 and nothing else"
        );
    }
}

/// 受信側がチャネル切断（送信端消失）まで全応答を集める。タイムアウト付きで
/// あり、ハングにはならない（regression 時は切断 or タイムアウトで早期終了する）。
fn collect_until_disconnect(rx: &mpsc::Receiver<PocResponse>) -> Vec<PocResponse> {
    let mut out = Vec::new();
    loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(r) => out.push(r),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                panic!("collect_until_disconnect timed out waiting for a terminal response")
            }
        }
    }
    out
}
