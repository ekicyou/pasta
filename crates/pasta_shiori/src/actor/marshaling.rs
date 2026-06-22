//! 本番 marshaling 層（task 3.4・design.md「MarshalingLayer」コンポーネント）。
//!
//! SHIORI スレッド（producer）とアクタースレッド（task 3.3・VM pin・consumer）の間の
//! **同期契約 marshaling** を本番化する。SHIORI/3.0 の「GET は応答を要する／NOTIFY は
//! 応答不要」という同期意味論を、スレッド分離後も byte-invariant に保つ:
//!
//! - **GET（R5.1）**: flume `bounded(1)` の reply tx を [`ActorMsg::Get`] に同梱して
//!   `try_send`、同期 [`Receiver::recv_timeout`]`(GET_TIMEOUT)` でブロック受信。
//!   `Ok(Value)`→応答文字列、`Timeout`/`Disconnected`→204。
//! - **NOTIFY（R5.2）**: 応答経路なしで `try_send`、アクター完了を待たず **即 204**。
//! - **drop→204（R5.3）**: アクターが reply を送らず drop（処理失敗）すると、受信側の
//!   `recv_timeout` が `Disconnected` を観測し 204。flume の move/drop 意味論で
//!   exactly-once（独自 `Responder` 不要）。
//! - **異常→204（R5.6）**: `try_send` 失敗（チャネル閉鎖＝アクター不在/異常）や未初期化
//!   は 204。SHIORI スレッドを無限待機させない（デッドロック経路を作らない）。
//! - **timeout→204＋コルーチン保存（R5.7）**: 閾値超過で 204 を返し SHIORI スレッドの
//!   待機を打ち切る。LuaJIT はプリエンプション不可ゆえ **アクタースレッドの Lua 実行は
//!   止めない**。`STORE.co_scene` は VM 内に persist し、次 `OnSecondChange`
//!   （`resume_until_valid` 駆動）が同コルーチンを resume して回復する。
//!
//! # 本タスクのスコープ / 申し送り
//! - 本モジュールは marshaling 契約を **隔離テスト**するための本番ロジックである。FFI
//!   入口（`windows.rs`/`shiori.rs`）の `static MAILBOX` への配線は **task 5.1**。それまで
//!   `static MAILBOX` の実体は導入せず、marshaling 自由関数は **呼び出し側が渡した
//!   `Sender`** に対して動作する（テストは task 3.3 の `spawn_actor_thread` が返す mailbox
//!   `Sender` を直接渡す）。これにより既存出荷 `PastaShiori::request` 同期経路を一切
//!   改変せず（byte-invariant ゴールデン 4/4 を緑のまま保つ）、5.1 の所有モデル再配線へ
//!   素直に接続できる。
//! - 本モジュール全体は task 3.3 と同じく `actor-poc` feature でガードされる（mailbox 型の
//!   再利用と wintf 依存のアクタースレッドを使うため）。
//!
//! # GET タイムアウト閾値（R5.8/R5.9・RN4 決定）
//! [`GET_TIMEOUT`] は **通常運転で決して発火しない安全網**である（R5.8: 通常経路バイト不変）。
//! task 5.1 で FFI 出荷経路がアクター経路へ切り替わり、ゴールデンスイートが実 GET をこの
//! タイムアウト越しに駆動するようになった。PoC 申し送りの 6.68ms は **実測レイテンシ**であって
//! 安全なハードタイムアウト値ではない——デバッグビルドのコールド VM や OS スケジューリング揺れで
//! 容易に超過し、通常 GET が 204 に化けてバイト不変が壊れる。したがって RN4 の決定として、
//! 閾値を実 GET が決して接近しない **秒オーダーの安全網**（[`GET_TIMEOUT`] = 5 秒）へ引き上げる。
//! これは「病的ブロッキング／デバッガ停止で SHIORI スレッドが無限ハングするのを防ぐ最後の砦」
//! であって、タイトな SLA ではない。タイムアウトは **デバッガ停止中も抑止しない**（R5.9）——
//! 停止中の 204 は次 `OnSecondChange` でコルーチンが resume され回復する。実機 SSP での更なる
//! チューニング（必要なら短縮）は後続の実測作業に委ねる（design.md OPEN QUESTION 3）。
//!
//! # panic-free 正常経路（R5.10）/ abort プロファイル割り切り（R5.11）
//! 本モジュールの非テストコードは正常経路で `unwrap`/`expect`/境界外索引を **一切使わない**
//! （fallible 操作は `Result`/`Option`/`let else` で扱う）。これは
//! `actor_marshaling_has_no_unwrap_on_normal_path` テストで静的に確認する。drop→204 ガードが
//! panic 経路の保険として効くのは unwind プロファイル（dev/test）に限られる前提であり、
//! リリース（`panic=abort`）では既存 `windows.rs` の `catch_unwind` 同様「到達不能の dev/test
//! 向け保険」として扱う（リリースのビルドプロファイルは変更しない・R5.11）。

use std::time::Duration;

use flume::{Sender, bounded};

use crate::actor::mailbox::{ActorMsg, MailboxRequest, Reply};
use crate::util::parsers::req::{Parser, Rule};
use pest::Parser as _;

/// GET の block-on-reply タイムアウト閾値（R5.8・RN4 決定値 = 5 秒）。
///
/// **通常運転では決して発火しない安全網**。実 GET はこの値の遥か手前で完了するため、出荷
/// 経路のゴールデンが 204 に化けることはない（R5.8 通常経路バイト不変）。PoC 申し送りの
/// 6.68ms は実測レイテンシであり安全なハードタイムアウトではないため、RN4 として実 GET が
/// 決して接近しない秒オーダーへ引き上げた（病的ブロッキング／デバッガ停止で SHIORI スレッドを
/// 無限ハングさせないための最後の砦であって、タイトな SLA ではない）。デバッガ停止中も抑止
/// しない（R5.9）。実機 SSP での更なるチューニングは後続作業（design.md OPEN QUESTION 3）。
pub const GET_TIMEOUT: Duration = Duration::from_secs(5);

/// SHIORI スレッド側で決定する SHIORI メソッド（marshaling 分岐点・R5.5）。
///
/// GET/NOTIFY のメソッド判定は **VM 投入前**に Rust 側で決定論的に確定する（Lua 意味論は
/// 不変）。`lua_request.rs` と同じ `Rule::get`/`Rule::notify` を、Lua テーブルを介さず
/// 出荷 pest パーサ（読み取り専用再利用）から取り出す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShioriMethod {
    /// GET（SHIORI/3.0 同期契約・block-on-reply）。
    Get,
    /// NOTIFY（fire-and-forget・即 204）。
    Notify,
}

/// 既定の 204 No Content 応答（marshaling の安全網フォールバック）。
///
/// バイト列は出荷経路 `PastaShiori::default_204_response` と同一（`SHIORI/3.0 204 No
/// Content` + `Charset`/`Sender` + 終端）。timeout/drop/異常/未初期化のいずれでも
/// SHIORI スレッドへ必ずこの応答を返し、無限待機を生じさせない。
pub fn default_204() -> String {
    "SHIORI/3.0 204 No Content\r\n\
     Charset: UTF-8\r\n\
     Sender: Pasta\r\n\
     \r\n"
        .to_string()
}

/// request 文字列から SHIORI メソッド（GET/NOTIFY）を **VM 投入前**に決定する（R5.5）。
///
/// 出荷 pest パーサ（`crate::util::parsers::req::{Parser, Rule}`）を **読み取り専用**で
/// 再利用し、`lua_request.rs:86-87`（`Rule::get`/`Rule::notify`）と同一の解釈を Lua VM を
/// 介さず Rust の決定論ロジックとして得る。出荷 `.rs` は diff ゼロ。
///
/// 正常経路は panic-free（`Result`/`Option` のみ・`unwrap`/`expect`/索引なし）。パース
/// 失敗やメソッド未検出は `None`（呼び出し側で 204 に倒す）。
pub fn determine_method(request: &str) -> Option<ShioriMethod> {
    let pairs = Parser::parse(Rule::req, request).ok()?.flatten();
    for pair in pairs {
        match pair.as_rule() {
            Rule::get => return Some(ShioriMethod::Get),
            Rule::notify => return Some(ShioriMethod::Notify),
            _ => {}
        }
    }
    None
}

/// GET marshaling（R5.1/R5.3/R5.6/R5.7）。
///
/// flume `bounded(1)` の reply tx を [`ActorMsg::Get`] に同梱して `try_send`、同期
/// [`Receiver::recv_timeout`]`(GET_TIMEOUT)` でブロックする。アクターが値を送れば応答
/// 文字列、reply を送らず drop すれば `Disconnected`→204、閾値超過なら `Timeout`→204、
/// `try_send` 失敗（アクター不在/満杯）なら即 204。**必ず文字列を返し無限待機しない**。
///
/// `GET_TIMEOUT` を用いる本番エントリ。タイムアウトを差し替えた検証は
/// [`marshal_get_with_timeout`] を使う。
pub fn marshal_get(tx: &Sender<ActorMsg>, req: MailboxRequest) -> String {
    marshal_get_with_timeout(tx, req, GET_TIMEOUT)
}

/// [`marshal_get`] の閾値注入版。
///
/// 本番は [`marshal_get`]（`GET_TIMEOUT`）を使う。タイムアウト→204 を決定論的に検証する
/// ため（実 VM の wall-clock に依存しないため）、テストから短い `timeout` と遅延アクターを
/// 与えて 204 経路を炙り出すのに用いる（R5.7 の隔離テスト）。
pub fn marshal_get_with_timeout(
    tx: &Sender<ActorMsg>,
    req: MailboxRequest,
    timeout: Duration,
) -> String {
    // flume・同期 recv_timeout で受ける reply 経路（exactly-once は move/drop 意味論）。
    let (reply_tx, reply_rx) = bounded::<Reply>(1);

    // try_send 失敗＝アクタースレッド不在/異常/満杯（R5.6）。reply_tx はここで drop され、
    // 受信側を作らないため無限待機は生じない。安全側として 204 を返す。
    if tx
        .try_send(ActorMsg::Get {
            req,
            reply: reply_tx,
        })
        .is_err()
    {
        return default_204();
    }

    // wake は flume の native Waker が executor を起こす（手動 wake 不要・task 3.1 実証）。
    // 同期 recv_timeout ゆえ flume の cancel 欠陥に無関係。
    match reply_rx.recv_timeout(timeout) {
        // 値到達: アクター VM が構築した応答文字列を返す（R5.1・通常経路バイト不変 R5.8）。
        Ok(Reply::Value(s)) => s,
        // Disconnected（reply drop・R5.3）/ Timeout（R5.7）のいずれも 204 に倒す。
        // timeout は SHIORI 待機の打ち切りのみ。アクターの Lua（co_scene）は止めない。
        Err(_) => default_204(),
    }
}

/// NOTIFY marshaling（R5.2）。
///
/// 応答経路を持たない [`ActorMsg::Notify`] を `try_send` し、アクターの完了を **待たず**
/// 即 204 を返す（fire-and-forget）。`try_send` の成否に関わらず SHIORI スレッドは
/// ブロックしない（アクター不在でも即 204・無限待機なし）。
pub fn marshal_notify(tx: &Sender<ActorMsg>, req: MailboxRequest) -> String {
    // 送信失敗（アクター不在）でも契約上 SSP 側はブロックしない。即 204 で終結する。
    let _ = tx.try_send(ActorMsg::Notify { req });
    default_204()
}

/// メソッド判定込みの marshaling ディスパッチ（R5.5・SHIORI スレッド入口想定）。
///
/// 生 request 文字列から **VM 投入前**にメソッドを決定し、GET=block-on-reply／NOTIFY=即
/// 204 へ分岐する。メソッド未検出（不正リクエスト）は 204 に倒す（panic させない）。
/// `seq` は mailbox の順序識別子（FIFO 観測用）。
pub fn marshal_request(tx: &Sender<ActorMsg>, seq: u64, request: &str) -> String {
    match determine_method(request) {
        Some(ShioriMethod::Get) => {
            marshal_get(tx, MailboxRequest::new(seq, request.to_string()))
        }
        Some(ShioriMethod::Notify) => {
            marshal_notify(tx, MailboxRequest::new(seq, request.to_string()))
        }
        // 不正リクエスト（メソッド未検出）: 安全側として 204（無限待機/パニックなし）。
        None => default_204(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::mailbox::mailbox;
    use std::thread;
    use std::time::Duration;

    /// determine_method（R5.5）: GET/NOTIFY が VM を介さず Rust 側で確定する。
    #[test]
    fn determine_method_classifies_get_and_notify() {
        let get = "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: version\r\nSender: SSP\r\n\r\n";
        let notify = "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnBoot\r\nSender: SSP\r\n\r\n";
        assert_eq!(determine_method(get), Some(ShioriMethod::Get));
        assert_eq!(determine_method(notify), Some(ShioriMethod::Notify));
        // 不正リクエストは None（呼び出し側で 204）。パニックしない。
        assert_eq!(determine_method("not a shiori request"), None);
    }

    /// default_204（R5.6 等）: 安全網 204 のバイト列が出荷 204 と同一であること。
    #[test]
    fn default_204_matches_shipping_bytes() {
        let expected = "SHIORI/3.0 204 No Content\r\n\
                        Charset: UTF-8\r\n\
                        Sender: Pasta\r\n\
                        \r\n";
        assert_eq!(default_204().as_bytes(), expected.as_bytes());
    }

    /// GET 正常（R5.1）: アクター役が reply を送れば、その値が marshal_get の戻り値になる。
    /// 単一スレッドで決定論的に検証する（実 VM 非依存）: 別スレッドのアクター役が
    /// mailbox を drain して reply を返す。
    #[test]
    fn marshal_get_returns_actor_reply_value() {
        let (tx, rx) = mailbox();

        // アクター役: GET を drain し、req.raw をそのまま値として reply する。
        let actor = thread::spawn(move || {
            if let Ok(ActorMsg::Get { req, reply }) = rx.recv() {
                let _ = reply.send(Reply::Value(format!("echo:{}", req.raw)));
            }
        });

        let resp = marshal_get_with_timeout(
            &tx,
            MailboxRequest::new(1, "PAYLOAD"),
            Duration::from_secs(10),
        );
        assert_eq!(resp, "echo:PAYLOAD", "GET must return the actor's reply value");
        actor.join().expect("actor role thread must not panic");
    }

    /// NOTIFY（R5.2）: 即 204 を返し、アクターの完了を待たない。投入されたメッセージは
    /// 後から drain でき（fire-and-forget の証跡）、Notify variant であること。
    #[test]
    fn marshal_notify_returns_204_immediately_without_awaiting() {
        let (tx, rx) = mailbox();

        let resp = marshal_notify(&tx, MailboxRequest::new(7, "NOTIFY-BODY"));
        assert_eq!(
            resp.as_bytes(),
            default_204().as_bytes(),
            "NOTIFY must return 204 immediately (fire-and-forget)"
        );

        // 待たずに 204 が返ったが、メッセージは mailbox に積まれている（後追い処理可能）。
        match rx.try_recv() {
            Ok(ActorMsg::Notify { req }) => {
                assert_eq!(req.raw, "NOTIFY-BODY");
                assert_eq!(req.seq, 7);
            }
            other => panic!("NOTIFY must be enqueued as ActorMsg::Notify, got {other:?}"),
        }
    }

    /// drop→204（R5.3）: アクター役が reply を送らず drop すると、受信側の recv_timeout が
    /// Disconnected を観測し 204 になる（無限待機なし）。
    #[test]
    fn marshal_get_reply_drop_yields_204() {
        let (tx, rx) = mailbox();

        // アクター役: GET を drain するが reply を送らず drop（処理失敗の模擬）。
        let actor = thread::spawn(move || {
            if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
                // reply を送らずスコープ離脱 → drop → 受信側 Disconnected。
                drop(reply);
            }
        });

        let resp = marshal_get_with_timeout(
            &tx,
            MailboxRequest::new(1, "X"),
            Duration::from_secs(10),
        );
        assert_eq!(
            resp.as_bytes(),
            default_204().as_bytes(),
            "an unreplied (dropped) GET reply must surface as 204 (Disconnected)"
        );
        actor.join().expect("actor role thread must not panic");
    }

    /// try_send 失敗→204（R5.6）: receiver がドロップ済み（アクター不在/異常）の mailbox へ
    /// GET を送ると try_send が失敗し、即 204 を返す（デッドロックなし）。
    #[test]
    fn marshal_get_closed_channel_yields_204() {
        let (tx, rx) = mailbox();
        // receiver を落としてチャネルを閉じる（アクタースレッド消滅の模擬）。
        drop(rx);

        let resp = marshal_get_with_timeout(
            &tx,
            MailboxRequest::new(1, "X"),
            Duration::from_secs(10),
        );
        assert_eq!(
            resp.as_bytes(),
            default_204().as_bytes(),
            "a closed channel (absent/abnormal actor) GET must return 204 without blocking"
        );
    }

    /// NOTIFY も try_send 失敗で即 204（R5.6・R5.2）: アクター不在でもブロックしない。
    #[test]
    fn marshal_notify_closed_channel_returns_204() {
        let (tx, rx) = mailbox();
        drop(rx);
        let resp = marshal_notify(&tx, MailboxRequest::new(1, "X"));
        assert_eq!(resp.as_bytes(), default_204().as_bytes());
    }

    /// timeout→204＋coroutine 保存（R5.7）: 閾値超過時に 204 を返しつつ、アクター（Lua）の
    /// in-flight work は **打ち切らない**ことを決定論的に示す。実 VM の wall-clock に
    /// 依存しないよう、閾値より明確に遅いアクター役を注入する。
    ///
    /// - Round1（slow GET）: アクター役が閾値より長く遅延してから reply するため、
    ///   marshal_get は timeout で 204 を返す（SHIORI 待機の打ち切り）。
    /// - 重要: timeout は SHIORI 側の待機を切るだけで、アクター役（Lua 実行相当）は
    ///   その後も処理を続行し reply を **完了** する（co_scene 保存の構造的根拠）。
    /// - Round2（recovery）: 後続 GET が同一 mailbox の FIFO で処理され、アクター役が
    ///   通常速度で reply するため値が返る（次 tick による回復に相当）。
    #[test]
    fn marshal_get_timeout_yields_204_but_actor_work_survives_and_recovers() {
        let (tx, rx) = mailbox();
        let short_timeout = Duration::from_millis(20);
        let actor_delay = Duration::from_millis(120); // 閾値 << 遅延（決定論的に timeout）。

        // アクター役: 2 件の GET を順に drain する単一直列ループ（FIFO・単一 consumer）。
        let actor = thread::spawn(move || {
            // Round1: 閾値より遅延してから reply（SHIORI は既に打ち切っている）。
            //         work は失われず最後まで完走する（co_scene 保存の構造的模擬）。
            if let Ok(ActorMsg::Get { req: _, reply }) = rx.recv() {
                thread::sleep(actor_delay);
                let _ = reply.send(Reply::Value("late-but-completed".to_string()));
            }
            // Round2: 通常速度で reply（次 tick での回復に相当）。
            if let Ok(ActorMsg::Get { req, reply }) = rx.recv() {
                let _ = reply.send(Reply::Value(format!("recovered:{}", req.raw)));
            }
        });

        // Round1: 短い閾値で待つ → アクター役が遅延中ゆえ timeout → 204。
        let round1 =
            marshal_get_with_timeout(&tx, MailboxRequest::new(1, "R1"), short_timeout);
        assert_eq!(
            round1.as_bytes(),
            default_204().as_bytes(),
            "GET exceeding the threshold must return 204 (SHIORI wait aborted)"
        );

        // Round2: 通常速度で待つ → 値が返る（後続 tick による回復）。アクター役の Round1
        // work が打ち切られていれば Round2 の FIFO が崩れ、この assert が落ちる。
        let round2 = marshal_get_with_timeout(
            &tx,
            MailboxRequest::new(2, "R2"),
            Duration::from_secs(10),
        );
        assert_eq!(
            round2, "recovered:R2",
            "the follow-up GET must recover and return a value (FIFO preserved; actor work not killed)"
        );

        actor.join().expect("actor role thread must not panic");
    }

    /// FIFO/順序保存（R6 横断・marshaling 経由でも順序不変）: 複数 GET を順に投入すると、
    /// 単一 consumer が投入順に処理し、各応答が対応する req の値を返す。
    #[test]
    fn marshal_get_preserves_fifo_order() {
        let (tx, rx) = mailbox();

        // アクター役: GET を drain し続け、req.raw を echo する単一直列ループ。
        let actor = thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                match msg {
                    ActorMsg::Get { req, reply } => {
                        let _ = reply.send(Reply::Value(req.raw));
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
            assert_eq!(resp, format!("v{i}"), "GET #{i} must return its own reply in order");
        }

        let (stop, done_rx) = ActorMsg::stop();
        tx.send(stop).expect("send Stop");
        done_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("Stop must be acked");
        actor.join().expect("actor role thread must not panic");
    }

    /// panic-free 正常経路の静的確認（R5.10）: marshaling モジュールの **非テストコード**に
    /// `unwrap(`/`expect(`/`panic!(`/`unreachable!(` が現れないことを、ソースを読み取って
    /// 静的に保証する。`#[cfg(test)]` 以下（本テストモジュール）は対象外とするため、
    /// `mod tests` 出現位置以降を除外して走査する。
    #[test]
    fn actor_marshaling_has_no_unwrap_on_normal_path() {
        let src = include_str!("marshaling.rs");
        // テストモジュール開始位置以降を切り落とし、本番（非テスト）コードのみを検査する。
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
