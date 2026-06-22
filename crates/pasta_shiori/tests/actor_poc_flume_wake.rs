//! RN1 薄い実証: flume `recv_async().await` × wintf `block_on` の wake 統合テスト
//! （task 3.1）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切
//! 汚染しない（R7.1/R7.2）。出荷既定ビルドはバイト不変のまま。
//!
//! # 何を証明するか（design.md Open Question RN1 の薄い実証）
//!
//! 本番アクタースレッド（task 3.3）に先立ち、設計が前提とする次の核心が **成立する**
//! ことを最小実証する:
//!
//! > 専用 OS スレッド上で `wintf_winmsg_executor::block_on(future)` を回し、その future が
//! > flume `Receiver::recv_async().await` をループで待つ。別スレッドが flume `Sender::try_send`
//! > すると、**手動 wake / `PostMessage` なしで** その await が起床しメッセージを消費する。
//!
//! これは flume 自身の `Waker` が wintf のメッセージ専用ウィンドウへ wake を post し、
//! メッセージループが future を再ポーリングすることを意味する（設計の
//! 「wake は flume の Waker が executor を起こす（手動不要）」の裏取り）。
//!
//! # 既存 PoC（actor_poc/actor_thread.rs）との差分（本タスクが NEW である理由）
//!
//! 既存 PoC は **std `mpsc` + 手動捕捉 `Waker` の `wake_by_ref()`** でメッセージループを
//! 進めていた（producer が明示的に起こす）。本番設計はチャンネルを **flume 1 本** に
//! 統一し、`recv_async().await` の **native Waker 統合** に依存して手動 wake を撤廃する。
//! その native 統合が wintf executor 上で実際に効くかは未実証だったため、本タスクで
//! 薄く実証する。
//!
//! # テストが「自明に真」でないことの担保（RED の意味を保つ）
//!
//! 受信スレッドの `join` に **有界タイムアウト** を課す。もし wake 統合が効かなければ
//! `recv_async().await` は起床せず future は永久に Pending のまま `block_on` が戻らず、
//! 受信スレッドは join できずタイムアウト → テストは失敗する。すなわち「起床した」
//! ことを genuinely 観測している（消費数・順序・clean stop も併せて検証）。

#![cfg(feature = "actor-poc")]

use pasta::actor_poc::flume_wake::{run_flume_wake_proof, FlumeWakeProof, ProofMsg};
use std::time::Duration;

/// 別スレからの `try_send` が wintf `block_on` 上の `recv_async().await` を **手動 wake
/// なしで** 起床させ、メッセージが直列・FIFO で消費され、Stop で clean に join される。
#[test]
fn flume_try_send_wakes_recv_async_under_wintf_block_on() {
    // 3 件のデータ＋Stop を別スレッドから try_send する。
    let proof: FlumeWakeProof = run_flume_wake_proof(
        &[ProofMsg::Value(10), ProofMsg::Value(20), ProofMsg::Value(30)],
        // 有界タイムアウト: wake が効かなければ join できずここで失敗する。
        Duration::from_secs(10),
    );

    // (1) 受信スレッドが期限内に join できた = block_on が戻った = wake 統合が効いた。
    assert!(
        proof.joined_within_timeout,
        "actor thread did not join within timeout: recv_async().await never woke \
         (flume Waker did not drive the wintf message loop)"
    );

    // (2) 3 件すべてが消費された（消費漏れなし）。
    assert_eq!(
        proof.consumed,
        vec![10, 20, 30],
        "messages were not consumed in FIFO order / some were dropped"
    );

    // (3) 受信は専用アクタースレッド上で起きた（送信スレッドとは別スレッド・R4.5）。
    assert_ne!(
        proof.actor_thread_id, proof.sender_thread_id,
        "recv_async must run on a dedicated actor thread, not the sender thread"
    );

    // (4) wake はすべて flume native（手動 wake / PostMessage を一切呼んでいない）。
    assert!(
        proof.manual_wake_calls == 0,
        "proof must not perform any manual wake; flume's Waker alone must drive the loop"
    );
}
