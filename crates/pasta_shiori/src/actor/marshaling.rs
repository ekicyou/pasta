//! 本番 marshaling 層。
//!
//! SHIORI/3.0 の「GET は応答を要する／NOTIFY は応答不要」という同期意味論を、
//! SHIORI スレッドとアクタースレッドの分離後も byte-invariant に保つ:
//!
//! - **GET**: flume `bounded(1)` の reply tx を [`ActorMsg::Get`] に同梱して
//!   `try_send`、同期 [`Receiver::recv_timeout`]`(GET_TIMEOUT)` でブロック受信。
//!   `Ok(応答)`→応答文字列（VM 応答、またはリクエスト処理エラー由来の 500 応答）、
//!   `Timeout`/`Disconnected`→204。
//! - **NOTIFY**: 応答経路なしで `try_send`、アクター完了を待たず **即 204**。
//! - **drop→204**: アクターが reply を **送れない**（スレッド消滅・panic 等）まま
//!   drop すると、受信側の `recv_timeout` が `Disconnected` を観測し 204。
//!   リクエスト処理のエラーは drop ではなく 500 応答として返る（`thread.rs` の GET アーム）。
//!   204 は「応答を返せなかった」場合の安全網である。
//! - **異常→204**: `try_send` 失敗（チャネル閉鎖＝アクター不在/異常）や未初期化
//!   は 204。SHIORI スレッドを無限待機させない。
//! - **timeout→204**: 閾値超過で 204 を返し SHIORI スレッドの待機を打ち切る。
//!   LuaJIT はプリエンプション不可ゆえ **アクタースレッドの Lua 実行は止めない**。
//!   タイムアウトした GET の処理はアクタースレッドで最後まで続き、その応答は捨てられる。
//!   `STORE.co_scene` などの VM 内状態はその処理の結果どおりに更新される。
//!
//! # panic-free 正常経路
//! 本モジュールの非テストコードは正常経路で `unwrap`/`expect`/境界外索引を使わない。
//! `actor_marshaling_has_no_unwrap_on_normal_path` テストがソースを静的に確かめる。
//! drop→204 が panic の保険として効くのは unwind プロファイル（dev/test）だけである。

use std::time::Duration;

use flume::{Sender, bounded};

use crate::actor::mailbox::{ActorMsg, MailboxRequest};
use crate::util::parsers::req::{ParseError, Parser, Rule};
use pest::Parser as _;
use pest::Position;
use pest::error::ErrorVariant;

/// GET の block-on-reply タイムアウト閾値。
///
/// **通常運転では決して発火しない安全網**で、タイトな SLA ではない。ミリ秒オーダーに
/// すると、デバッグビルドのコールド VM やスケジューリングの揺れで通常の GET が 204 に化ける。
/// デバッガ停止中も抑止しない（停止中の GET は 204 になり、再開後の応答は捨てられる）。
pub const GET_TIMEOUT: Duration = Duration::from_secs(5);

/// SHIORI スレッド側で **VM 投入前**に決定する SHIORI メソッド。
///
/// `lua_request.rs` と同じ `Rule::get`/`Rule::notify` を、Lua テーブルを介さず pest パーサから取り出す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShioriMethod {
    /// GET（SHIORI/3.0 同期契約・block-on-reply）。
    Get,
    /// NOTIFY（fire-and-forget・即 204）。
    Notify,
}

/// 既定の 204 No Content 応答。Rust の側で作る 204 はこれ 1 つ。
///
/// `pasta_scripts/pasta/shiori/res.lua` の `RES.no_content` と同じバイト列に保つ
/// （`byte_invariant_test` の `GOLDEN_NOTIFY_204` / `GOLDEN_ON_SECOND_CHANGE_204` が固定する）。
pub fn default_204() -> String {
    "SHIORI/3.0 204 No Content\r\n\
     Charset: UTF-8\r\n\
     Sender: Pasta\r\n\
     SecurityLevel: local\r\n\
     \r\n"
        .to_string()
}

/// 要求の見分け。2.x を Lua へ届けるか 400 にするかは入口が決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestKind {
    /// `GET SHIORI/3.0`。
    Get,
    /// `NOTIFY SHIORI/3.0`。
    Notify,
    /// SHIORI/2.x のヘッダ（旧来は Lua へ、版 3 は 400）。
    Shiori2(ShioriMethod),
}

/// 要求のメソッドを **VM 投入前**に判定する（旧来と版 3 の入口で共通）。
///
/// `Rule::header2` なら `Shiori2(メソッド)` を返す。解けなければ pest の誤り（位置つき）を返す。
/// 解けたのにメソッドが無い（文法上起きない）ときも panic せず誤りを返す。
pub(crate) fn classify_request(request: &str) -> Result<RequestKind, Box<ParseError>> {
    let pairs = Parser::parse(Rule::req, request).map_err(Box::new)?;
    let mut method = None;
    let mut shiori2 = false;
    // header2 はメソッドの後ろに来るので、見出しの行の終わりまで見る。
    for pair in pairs.flatten() {
        match pair.as_rule() {
            Rule::get => method = Some(ShioriMethod::Get),
            Rule::notify => method = Some(ShioriMethod::Notify),
            Rule::header2 => shiori2 = true,
            Rule::key_values => break,
            _ => {}
        }
    }
    let Some(method) = method else {
        let variant = ErrorVariant::CustomError {
            message: "request has no method".to_string(),
        };
        return Err(Box::new(ParseError::new_from_pos(
            variant,
            Position::from_start(request),
        )));
    };
    Ok(match (shiori2, method) {
        (true, m) => RequestKind::Shiori2(m),
        (false, ShioriMethod::Get) => RequestKind::Get,
        (false, ShioriMethod::Notify) => RequestKind::Notify,
    })
}

/// GET marshaling。**必ず文字列を返し無限待機しない**。
///
/// アクターが値を送れば応答文字列（VM 応答、またはリクエスト処理エラー由来の 500 応答）、
/// reply を送れず drop されれば 204、[`GET_TIMEOUT`] 超過なら 204、
/// `try_send` 失敗（アクター不在/満杯）なら即 204。
pub fn marshal_get(tx: &Sender<ActorMsg>, req: MailboxRequest) -> String {
    marshal_get_with_timeout(tx, req, GET_TIMEOUT)
}

/// [`marshal_get`] の閾値注入版（テストから短い `timeout` を与える口）。
pub fn marshal_get_with_timeout(
    tx: &Sender<ActorMsg>,
    req: MailboxRequest,
    timeout: Duration,
) -> String {
    let (reply_tx, reply_rx) = bounded::<String>(1);

    let seq = req.seq;
    // try_send 失敗＝アクタースレッド不在/異常/満杯。reply_tx はここで drop されるので待たない。
    if tx
        .try_send(ActorMsg::Get {
            req,
            reply: reply_tx,
        })
        .is_err()
    {
        tracing::debug!(
            seam = "actor.try_send",
            method = "get",
            seq,
            ok = false,
            "GET try_send failed (actor absent/full) -> 204"
        );
        return default_204();
    }
    tracing::trace!(
        seam = "actor.try_send",
        method = "get",
        seq,
        ok = true,
        "GET enqueued; awaiting reply"
    );

    // 同期 recv_timeout ゆえ flume の cancel 欠陥に無関係。
    match reply_rx.recv_timeout(timeout) {
        Ok(s) => {
            tracing::trace!(
                seam = "actor.reply",
                method = "get",
                seq,
                "GET reply value received"
            );
            s
        }
        Err(flume::RecvTimeoutError::Disconnected) => {
            tracing::debug!(
                seam = "actor.drop",
                method = "get",
                seq,
                "GET reply dropped (Disconnected) -> 204"
            );
            default_204()
        }
        // SHIORI 待機を打ち切るが Lua（co_scene）は止めない。
        Err(flume::RecvTimeoutError::Timeout) => {
            tracing::debug!(
                seam = "actor.timeout",
                method = "get",
                seq,
                ?timeout,
                "GET reply timed out -> 204 (actor work not killed)"
            );
            default_204()
        }
    }
}

/// NOTIFY marshaling。アクターの完了を待たず、`try_send` の成否に関わらず即 204 を返す。
pub fn marshal_notify(tx: &Sender<ActorMsg>, req: MailboxRequest) -> String {
    let seq = req.seq;
    let ok = tx.try_send(ActorMsg::Notify { req }).is_ok();
    tracing::trace!(
        seam = "actor.try_send",
        method = "notify",
        seq,
        ok,
        "NOTIFY enqueued (fire-and-forget) -> 204"
    );
    default_204()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::mailbox::mailbox;
    use std::thread;
    use std::time::Duration;

    /// 7 通りが期待どおりに分かれる。
    /// 知らないメソッド（PUT）は今の文法で読めないので誤り。誤りは位置（行と列）を持つ。
    #[test]
    fn classify_request_splits_seven_cases() {
        let cases = [
            (
                "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: version\r\n\r\n",
                Some(RequestKind::Get),
            ),
            (
                "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnBoot\r\n\r\n",
                Some(RequestKind::Notify),
            ),
            (
                "GET Sentence SHIORI/2.6\r\nCharset: UTF-8\r\n\r\n",
                Some(RequestKind::Shiori2(ShioriMethod::Get)),
            ),
            (
                "NOTIFY OwnerGhostName SHIORI/2.0\r\nCharset: UTF-8\r\n\r\n",
                Some(RequestKind::Shiori2(ShioriMethod::Notify)),
            ),
            ("PUT SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n", None),
            ("", None),
            ("GET SHIORI/3.0\r\nCharset UTF-8\r\n\r\n", None),
        ];
        for (req, expected) in cases {
            let got = classify_request(req);
            match expected {
                Some(kind) => assert_eq!(got.ok(), Some(kind), "request: {req:?}"),
                None => {
                    let Err(e) = got else {
                        panic!("must be unreadable: {req:?}");
                    };
                    let (line, col) = match e.line_col {
                        pest::error::LineColLocation::Pos(p) => p,
                        pest::error::LineColLocation::Span(p, _) => p,
                    };
                    assert!(line >= 1 && col >= 1, "request: {req:?}");
                }
            }
        }
    }

    /// `Key:`（値が空）と `Key:value`（空白なし）を含む要求も解ける。
    #[test]
    fn classify_request_accepts_empty_and_unspaced_values() {
        let req = "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnInitialize\r\n\
                   Reference0:\r\nReference1:abc\r\n\r\n";
        assert_eq!(classify_request(req).ok(), Some(RequestKind::Notify));
    }

    /// 安全網 204 のバイト列が VM 産 204 と同一（標準ヘッダ 3 種・`SecurityLevel: local` を含む）。
    #[test]
    fn default_204_matches_shipping_bytes() {
        let expected = "SHIORI/3.0 204 No Content\r\n\
                        Charset: UTF-8\r\n\
                        Sender: Pasta\r\n\
                        SecurityLevel: local\r\n\
                        \r\n";
        assert_eq!(default_204().as_bytes(), expected.as_bytes());
    }

    /// アクター役が reply を送れば、その値が marshal_get の戻り値になる。
    #[test]
    fn marshal_get_returns_actor_reply_value() {
        let (tx, rx) = mailbox();

        let actor = thread::spawn(move || {
            if let Ok(ActorMsg::Get { req, reply }) = rx.recv() {
                let _ = reply.send(format!("echo:{}", req.raw));
            }
        });

        let resp = marshal_get_with_timeout(
            &tx,
            MailboxRequest::new(1, "PAYLOAD"),
            Duration::from_secs(10),
        );
        assert_eq!(
            resp, "echo:PAYLOAD",
            "GET must return the actor's reply value"
        );
        actor.join().expect("actor role thread must not panic");
    }

    /// NOTIFY は即 204 を返し、アクターの完了を待たない。メッセージは後から drain できる。
    #[test]
    fn marshal_notify_returns_204_immediately_without_awaiting() {
        let (tx, rx) = mailbox();

        let resp = marshal_notify(&tx, MailboxRequest::new(7, "NOTIFY-BODY"));
        assert_eq!(
            resp.as_bytes(),
            default_204().as_bytes(),
            "NOTIFY must return 204 immediately (fire-and-forget)"
        );

        match rx.try_recv() {
            Ok(ActorMsg::Notify { req }) => {
                assert_eq!(req.raw, "NOTIFY-BODY");
                assert_eq!(req.seq, 7);
            }
            other => panic!("NOTIFY must be enqueued as ActorMsg::Notify, got {other:?}"),
        }
    }

    /// アクター役が reply を送らず drop すると 204 になる（無限待機なし）。
    #[test]
    fn marshal_get_reply_drop_yields_204() {
        let (tx, rx) = mailbox();

        let actor = thread::spawn(move || {
            if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
                drop(reply);
            }
        });

        let resp =
            marshal_get_with_timeout(&tx, MailboxRequest::new(1, "X"), Duration::from_secs(10));
        assert_eq!(
            resp.as_bytes(),
            default_204().as_bytes(),
            "an unreplied (dropped) GET reply must surface as 204 (Disconnected)"
        );
        actor.join().expect("actor role thread must not panic");
    }

    /// receiver がドロップ済み（アクター不在/異常）の mailbox への GET は即 204 を返す。
    #[test]
    fn marshal_get_closed_channel_yields_204() {
        let (tx, rx) = mailbox();
        drop(rx);

        let resp =
            marshal_get_with_timeout(&tx, MailboxRequest::new(1, "X"), Duration::from_secs(10));
        assert_eq!(
            resp.as_bytes(),
            default_204().as_bytes(),
            "a closed channel (absent/abnormal actor) GET must return 204 without blocking"
        );
    }

    /// NOTIFY もアクター不在でブロックせず即 204。
    #[test]
    fn marshal_notify_closed_channel_returns_204() {
        let (tx, rx) = mailbox();
        drop(rx);
        let resp = marshal_notify(&tx, MailboxRequest::new(1, "X"));
        assert_eq!(resp.as_bytes(), default_204().as_bytes());
    }

    /// 閾値超過時に 204 を返しつつ、アクター（Lua）の in-flight work は打ち切らず、
    /// 後続の GET は値を返す。
    #[test]
    fn marshal_get_timeout_yields_204_but_actor_work_survives_and_recovers() {
        let (tx, rx) = mailbox();
        let short_timeout = Duration::from_millis(20);
        // 遅延を sleep でなく門で作る（混んだ macOS ランナーでは sleep が閾値を追い越した）。
        let (release_tx, release_rx) = bounded::<()>(1);

        let actor = thread::spawn(move || {
            // SHIORI が打ち切った後に門が開くまで reply しない。
            if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
                let _ = release_rx.recv();
                let _ = reply.send("late-but-completed".to_string());
            }
            if let Ok(ActorMsg::Get { req, reply }) = rx.recv() {
                let _ = reply.send(format!("recovered:{}", req.raw));
            }
        });

        let round1 = marshal_get_with_timeout(&tx, MailboxRequest::new(1, "R1"), short_timeout);
        assert_eq!(
            round1.as_bytes(),
            default_204().as_bytes(),
            "GET exceeding the threshold must return 204 (SHIORI wait aborted)"
        );
        release_tx
            .send(())
            .expect("actor role must be waiting at the gate");

        // Round1 の work が打ち切られていれば FIFO が崩れ、この assert が落ちる。
        let round2 =
            marshal_get_with_timeout(&tx, MailboxRequest::new(2, "R2"), Duration::from_secs(10));
        assert_eq!(
            round2, "recovered:R2",
            "the follow-up GET must recover and return a value (FIFO preserved; actor work not killed)"
        );

        actor.join().expect("actor role thread must not panic");
    }

    /// 複数 GET を順に投入すると、各応答が対応する req の値を順に返す。
    #[test]
    fn marshal_get_preserves_fifo_order() {
        let (tx, rx) = mailbox();

        let actor = thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                match msg {
                    ActorMsg::Get { req, reply } => {
                        let _ = reply.send(req.raw);
                    }
                    ActorMsg::Stop { done } => {
                        let _ = done.send(());
                        break;
                    }
                    _ => {}
                }
            }
        });

        for i in 0..10u64 {
            let resp = marshal_get_with_timeout(
                &tx,
                MailboxRequest::new(i, format!("v{i}")),
                Duration::from_secs(10),
            );
            assert_eq!(
                resp,
                format!("v{i}"),
                "GET #{i} must return its own reply in order"
            );
        }

        let (stop, done_rx) = ActorMsg::stop();
        tx.send(stop).expect("send Stop");
        done_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("Stop must be acked");
        actor.join().expect("actor role thread must not panic");
    }

    /// 非テストコード（`mod tests` より前）に `unwrap(`/`expect(`/`panic!(`/`unreachable!(` が
    /// 現れない。コメントも走査されるので、上のコメントにもこれらを書かない。
    #[test]
    fn actor_marshaling_has_no_unwrap_on_normal_path() {
        let src = include_str!("marshaling.rs");
        let prod = match src.find("mod tests") {
            Some(idx) => &src[..idx],
            None => src,
        };
        for needle in ["unwrap(", "expect(", "panic!(", "unreachable!(", "[0]"] {
            assert!(
                !prod.contains(needle),
                "marshaling normal-path code must be panic-free (found `{needle}` in non-test code)"
            );
        }
    }
}
