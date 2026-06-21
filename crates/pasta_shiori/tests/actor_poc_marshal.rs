//! Marshal GET/NOTIFY 分岐＋pest 再パースの統合テスト（R2.1 / R2.2 / R2.4・task 3.2）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切
//! 汚染しない（R7.1/R7.2/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 3.2）:
//! 「GET が応答値（または drop→204）、NOTIFY が即 204 を返し、VM 操作が drain 側に
//!   閉じる統合テストが緑。」
//!
//! # 設計プリミティブを通る経路を観測する（review 指摘の是正）
//!
//! 本テストは marshaling が **task 1.4 `Mailbox`／task 3.1 `Responder`／task 2.1
//! `ActorThread`** を通って流れることを観測する。`Marshal` は自前のアクターループを
//! 持たず、`ActorThread` を保持して `ActorMsg::Get`/`ActorMsg::Notify` を mailbox へ
//! enqueue し、GET は `Responder` の `Receiver` を block-on-reply する。
//!
//! # 何を「正直に」観測するか
//!
//! - GET（R2.1）: request 文字列を **既存 pest パーサで再パース** して `method=get` を得、
//!   `Marshal` が `ActorMsg::Get`（`Responder` 内包）を mailbox へ enqueue、実 `!Send` VM を
//!   持つアクタースレッドが drain して VM 実行→`Responder::reply` し、SSP 側が
//!   block-on-reply で応答値を受け取る。応答値には **VM を実行したスレッド id 由来の値**
//!   を載せ、それがテストスレッドではなくアクタースレッドであることを assert する
//!   （R2.3 分離）。
//! - GET drop→204（R2.4 故障注入）: アクターが（VM 失敗により）reply せず responder を
//!   drop すると、Responder ガードが 204 を撃ち SSP 側が無限待機しない。`record_blocker`
//!   でブロッカーを残す。
//! - NOTIFY（R2.2）: 即 204 を返し、アクターの処理完了を **待たない**。完了は後追いの
//!   観測用 GET（同一 mailbox の FIFO で NOTIFY 完了後に返る）で観測できる。

#![cfg(feature = "actor-poc")]

use pasta::actor_poc::ffi_marshal::{
    reparse_method, thread_id_digits, Marshal, PocResponse, ShioriMethod,
};
use pasta_lua::actor_poc::responder::PocResponse as LuaPocResponse;
use pasta_lua::actor_poc::verdict::{Blocker, ReqId, VerdictRecorder};
use std::thread;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元:
/// `crates/pasta_shiori/tests/common/mod.rs:23-29`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

const GET_REQ: &str = "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: version\r\nSender: SSP\r\n\r\n";
const NOTIFY_REQ: &str =
    "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnBoot\r\nSender: SSP\r\n\r\n";

/// `Marshal` を建て、テストブロックに渡してから必ず teardown する。
fn with_marshal<R>(f: impl FnOnce(&Marshal) -> R) -> R {
    let marshal = Marshal::spawn();
    let out = f(&marshal);
    marshal.shutdown();
    out
}

/// R2.1（pest 再パース＋GET block-on-reply）: request 文字列→`method=get`→
/// `ActorMsg::Get`（mailbox）→実 VM→`Responder::reply`→SSP が VM 由来の応答値を
/// 受け取る end-to-end。
///
/// 併せて R2.3（スレッド分離）: 応答値はアクタースレッド id 由来であり、テストスレッド
/// とは **異なる**（VM 操作が drain 側に閉じていた証跡）。
#[test]
fn get_reparses_then_block_on_reply_returns_vm_value_from_actor_thread() {
    with_marshal(|marshal| {
        // 出荷パーサで method を再解決（diff ゼロ・read-only reuse）。
        let method = reparse_method(GET_REQ).expect("GET request must re-parse");
        assert_eq!(method, ShioriMethod::Get, "pest must classify this as GET");

        // GET: VM が `return <int>` を計算し、アクターは実行スレッド id を載せて reply する。
        let resp = marshal.dispatch(method, "return 7").0;

        let test_thread_digits = thread_id_digits(thread::current().id());
        match resp {
            PocResponse::Value(actor_digits) => {
                assert_ne!(
                    actor_digits, test_thread_digits,
                    "VM op must run on the actor thread, NOT the SSP/test thread (R2.3)"
                );
                assert_eq!(
                    actor_digits,
                    thread_id_digits(marshal.actor_thread_id()),
                    "the GET reply value must originate from the marshal actor thread's VM op"
                );
            }
            PocResponse::NoContent204 => {
                panic!("GET with a replying actor must return the VM value, not 204")
            }
        }
    });
}

/// R2.4（故障注入→drop→204）: アクターが reply せず responder を drop すると、
/// SSP 側 block-on-reply は無限待機せず 204 を受け取る。marshaling 不成立として
/// `record_blocker` でブロッカーを残す（NO-GO 根拠）。応答経路は task 3.1 の
/// `Responder`（mailbox の `ActorMsg::Get` に内包）を通る。
#[test]
fn get_unreplied_drop_yields_204_and_records_blocker() {
    let mut rec = VerdictRecorder::new();

    let resp = with_marshal(|marshal| {
        // GET 経路であることを再パースで確定してから故障注入する。
        let method = reparse_method(GET_REQ).expect("GET request must re-parse");
        assert_eq!(method, ShioriMethod::Get, "fault injection targets the GET path");
        // inject_no_reply=true: アクターは（VM 失敗により）reply せず responder を drop する。
        marshal.get("return 1", true)
    });

    assert_eq!(
        resp,
        PocResponse::NoContent204,
        "an unreplied GET must terminate as 204 via the Responder drop guard (no deadlock)"
    );

    // marshaling の往復が成立しなかった経路を R2.4 のブロッカーとして記録する。
    rec.record_blocker(
        ReqId::sub(2, 4),
        Blocker::with_workaround(
            "GET block-on-reply で応答未送信のまま responder が drop された",
            "Responder Drop ガードが 204 を撃ち SSP 側を解放（デッドロックは原理的に消滅）",
        ),
    );
    assert_eq!(
        rec.blockers(ReqId::sub(2, 4)).len(),
        1,
        "the unreplied-GET path must be recorded as an R2.4 blocker"
    );
}

/// R2.2（NOTIFY fire-and-forget）: NOTIFY は **即 204** を返し、アクターの処理完了を
/// 待たない。完了は後から（同一 mailbox の FIFO による観測用 GET で）観測でき、それが
/// SSP の即時 204 とは独立であることを示す。NOTIFY も観測 GET も `ActorMsg` として
/// mailbox を通り、VM 操作はアクタースレッドに閉じる（R2.3）。
#[test]
fn notify_returns_204_immediately_without_waiting_for_actor() {
    with_marshal(|marshal| {
        let method = reparse_method(NOTIFY_REQ).expect("NOTIFY request must re-parse");
        assert_eq!(method, ShioriMethod::Notify, "pest must classify this as NOTIFY");

        // NOTIFY を発行。即 204 が返るはず（アクターの VM 実行完了を待たない）。
        let (resp, observer) = marshal.dispatch(method, "return 99");
        assert_eq!(
            resp,
            PocResponse::NoContent204,
            "NOTIFY must return 204 immediately (fire-and-forget)"
        );
        assert!(
            observer.is_none(),
            "NOTIFY dispatch returns 204 immediately with no inline completion wait"
        );

        // fire-and-forget の証跡: SSP は待たなかったが、アクターは後から NOTIFY の VM を
        // 完了する。同一 mailbox の FIFO により、後続の観測用 GET の応答は NOTIFY の VM
        // 完了後に返る。応答値はアクタースレッド id 由来（R2.3）。
        let completion = marshal
            .observe_notify_completion()
            .expect("the actor must eventually complete the NOTIFY work (observed after 204)");
        assert_eq!(
            completion.actor_thread_digits,
            thread_id_digits(marshal.actor_thread_id()),
            "NOTIFY/observe VM op must run on the actor thread (R2.3 separation)"
        );
        assert_ne!(
            completion.actor_thread_digits,
            thread_id_digits(thread::current().id()),
            "NOTIFY/observe VM op must NOT run on the SSP/test thread"
        );
    });
}

/// PocResponse の同一性確認（pasta_shiori 再エクスポートが pasta_lua の型そのもの）。
/// これにより責務配置（応答表現は pasta_lua プリミティブの再利用）が壊れていないことを示す。
#[test]
fn poc_response_is_the_pasta_lua_primitive() {
    let a: PocResponse = PocResponse::NoContent204;
    let b: LuaPocResponse = LuaPocResponse::NoContent204;
    assert_eq!(a, b, "Marshal must reuse the reviewed pasta_lua PocResponse type");
}
