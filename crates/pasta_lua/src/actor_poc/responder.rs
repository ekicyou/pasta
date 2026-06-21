//! GET 応答用 oneshot ＋ Drop→204 ガード（R3.1 / R3.2 / R3.3 / R3.4・task 3.1）。
//!
//! design.md「Responder」コンポーネントを実装する。GET 処理側が応答を送信せず
//! responder を drop（応答忘れ／panic 巻き戻し）した場合でも、SSP 側スレッドが
//! 無限待機に陥らず **204（成功・返り値なし）** フォールバックを受け取る保護を
//! 与える。これにより「応答未送信のまま drop」のデッドロック経路が原理的に
//! 消滅する（R3.3）。
//!
//! # 不変条件（critical）
//!
//! Responder は **必ず厳密に 1 回** で終結する——「reply 1 回」**または**
//! 「drop→204」のいずれか一方であり、両方でも零でもない（取りこぼしなし・二重
//! 送信なし）。これを型で担保する:
//! - 内部 `Sender` を `Option` に持ち、[`Responder::reply`] が `take()` して値を
//!   送る。`reply` は `self` を消費するため Drop と同時には起こらない。
//! - [`Drop`] は `Option` がまだ `Some`（=未 reply）のときだけ 204 を送る。
//!   `reply` 済みなら `Option` は `None` なので Drop は何も送らない（二重送信なし）。
//!
//! # 応答表現（pasta_shiori に依存しない）
//!
//! `pasta_lua` は **`pasta_shiori` に依存してはならない**（依存方向が逆。
//! `ShioriResponse` は下流の `pasta_shiori` に在る）。そこで本 PoC は 204 を含む
//! 応答を **ローカルな最小表現** [`PocResponse`] でモデル化する: 通常の値応答
//! [`PocResponse::Value`] と 204 No Content [`PocResponse::NoContent204`]。後者は
//! SHIORI 204 のセマンティクス（成功・返り値なし）に忠実な番兵である。実 SHIORI
//! 応答型への接合は `pasta-actor-runtime`（下流）の責務として繰り延べる。
//!
//! # panic セマンティクス（feasibility caveat・tasks.md Implementation Notes）
//!
//! drop→204-on-panic は **unwinding** に依存する。`cargo test` は dev/test profile
//! （既定 `panic = "unwind"`）で走るため、panic 時にも本 Drop ガードが走り 204 を
//! 撃つ。一方 root `Cargo.toml` の `[profile.release] panic = "abort"`（release）
//! では巻き戻りが起こらず Drop は走らない。したがってこの保証は **test/unwind
//! profile でのみ** 検証され、release（abort）では成立しない——これは
//! `pasta-actor-runtime` へ申し送る feasibility caveat である（このコメントは
//! TODO/FIXME ではなく、確定済みの設計上の前提を正直に明記したもの）。

use std::sync::mpsc::{self, Receiver, Sender};

/// PoC ローカルの SHIORI 応答表現（`pasta_shiori` 非依存・最小モデル）。
///
/// 実 `ShioriResponse` は下流 `pasta_lua` から見えない（依存方向が逆）ため、
/// 検証に必要な 2 形だけをここで表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PocResponse {
    /// 通常の値応答（GET の戻り値相当）。PoC では順序／一致確認に用いる数値。
    Value(u64),
    /// 204 No Content（成功・返り値なし）。応答未送信のまま drop された GET を
    /// SHIORI 204 セマンティクスで終結させる番兵。
    NoContent204,
}

/// GET 応答用 oneshot を包む responder。
///
/// 内部の `std::sync::mpsc::Sender`（1 回送信＝oneshot 相当。design.md Q5 の仮定
/// どおり追加依存を避ける）を `Option` で保持し、終結が厳密に 1 回であることを
/// 構造的に担保する:
/// - [`reply`](Responder::reply): `Some` を `take()` して値を送り、`self` を消費。
/// - [`Drop`]: まだ `Some`（未 reply）なら 204 を送る。`reply` 済みなら何もしない。
///
/// これにより「reply 1 回」または「drop→204」のどちらか一方で必ず終結する。
pub struct Responder {
    /// 未 reply の間だけ `Some`。`reply` で `take()` され、Drop ガードはこれが
    /// `Some` のときだけ 204 を撃つ（二重送信・取りこぼしを共に防ぐ）。
    tx: Option<Sender<PocResponse>>,
}

impl Responder {
    /// 新しい responder と、その応答を受け取る `Receiver` のペアを作る。
    ///
    /// 受信側（SSP スレッド相当）は `recv`（または `recv_timeout`）で
    /// [`PocResponse::Value`]（正常 reply）か [`PocResponse::NoContent204`]
    /// （未 reply drop→204 ガード）のいずれか **1 つ** を必ず受け取る。
    pub fn new() -> (Self, Receiver<PocResponse>) {
        let (tx, rx) = mpsc::channel();
        (Responder { tx: Some(tx) }, rx)
    }

    /// 正常応答を送る（`self` を消費）。
    ///
    /// `self` 消費により、reply 後に同じ responder で Drop ガードが追加発火する
    /// ことはない（reply と drop が同時に起こらない＝二重送信なし）。内部 `Sender`
    /// は `take()` され、これにより Drop は「未 reply」を `None` として判別できる。
    ///
    /// 受信側が既に drop されている場合は送信が失敗するが、PoC では握り潰す
    /// （ガードの責務は「無限待機を作らない」ことであり、受信側不在は待機者なし）。
    pub fn reply(mut self, value: PocResponse) {
        if let Some(tx) = self.tx.take() {
            // 受信側不在（drop 済み）の場合の Err は無視する: 待機者がいない以上
            // デッドロックは生じない。`self.tx` は None になったので Drop は撃たない。
            let _ = tx.send(value);
        }
        // ここで self が drop されるが、tx は既に None のため Drop ガードは沈黙する。
    }
}

impl Drop for Responder {
    /// 未 reply のまま drop（応答忘れ・panic 巻き戻し）されたら 204 を自動送信する。
    ///
    /// `reply` 済みなら `tx` は `None` で何も送らない（二重送信なし）。未 reply
    /// なら 204 を撃ち、ブロック中の受信側（SSP スレッド）を解放する——これが
    /// 「応答未送信のまま drop」デッドロック経路の原理的消滅（R3.1/R3.2/R3.3）。
    ///
    /// panic unwind 中もこの Drop は走る（dev/test profile・上記モジュール doc の
    /// caveat 参照）。受信側不在の Err は無視する（待機者がいなければ無害）。
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(PocResponse::NoContent204);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::TryRecvError;
    use std::time::Duration;

    /// 不変条件: 正常 reply は値を 1 回だけ届け、204 を二重送信しない。
    #[test]
    fn reply_sends_value_once_then_disconnects() {
        let (responder, rx) = Responder::new();
        responder.reply(PocResponse::Value(99));

        assert_eq!(
            rx.recv_timeout(Duration::from_secs(5)),
            Ok(PocResponse::Value(99)),
            "reply must deliver the exact value"
        );
        // reply 済みなので Drop は撃たない → 送信端消失で切断。
        assert_eq!(
            rx.try_recv(),
            Err(TryRecvError::Disconnected),
            "no second (204) response may follow a normal reply (exactly-once)"
        );
    }

    /// R3.1: 未 reply drop は 204 を 1 回だけ撃つ（無限待機ではない）。
    #[test]
    fn unreplied_drop_emits_single_204() {
        let (responder, rx) = Responder::new();
        drop(responder);

        assert_eq!(
            rx.recv_timeout(Duration::from_secs(5)),
            Ok(PocResponse::NoContent204),
            "unreplied drop must auto-emit 204, releasing the receiver"
        );
        assert_eq!(
            rx.try_recv(),
            Err(TryRecvError::Disconnected),
            "the drop→204 guard must fire exactly once"
        );
    }
}
