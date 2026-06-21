//! SSP スレッド側 FFI 反転ハーネス（Marshal・R2.1 / R2.2 / R2.4・task 3.2）。
//!
//! design.md「Marshal（SSP スレッド側 FFI 反転ハーネス）」を実装する。PoC ハーネスは
//! 出荷経路（`shiori::request` / `lua_request.rs`）を **改変せず**、request 文字列を
//! 既存 pest パーサ（`crate::util::parsers::req::{Parser, Rule}`）で **再パース** して
//! `method`（GET/NOTIFY）を得る（design.md Q3「再パース」決定＝出荷 `.rs` は diff ゼロ）。
//!
//! # 責務配置方針（Rust 寄せ・design.md「責務配置方針」）
//!
//! GET/NOTIFY の判定・marshaling 分岐・block-on-reply/204 は **決定論ロジック** であり
//! Rust 側で完結させる。method-dispatch の判断を Lua へ投げない。シーン実行・コルーチン
//! 継続の意味論（R4）は Lua のままだが、本タスクの殻（method 振り分けと往復）は Rust。
//!
//! # 設計プリミティブの再利用（review 指摘の是正）
//!
//! 本ハーネスは **独自のアクターループ／チャネル／block_on を持たない**。代わりに
//! pasta_lua の既にレビュー済みプリミティブを **そのまま組み合わせる**:
//!
//! - [`ActorThread`](pasta_lua::actor_poc::actor_thread::ActorThread) — `std::thread`＋
//!   `block_on` で `!Send` な実 `PastaLuaRuntime` をアクタースレッドに pin してホスト
//!   する（task 2.1）。`Marshal` は自前で thread/executor を建てず、これを **保持** する。
//! - [`Mailbox`](pasta_lua::actor_poc::mailbox)（[`ActorMsg`] / [`MailboxSender::enqueue`]）
//!   — SSP→アクターの単一直列キュー（task 1.4）。`Marshal` はここへ enqueue する。
//! - [`Responder`](pasta_lua::actor_poc::responder::Responder) — GET 応答 oneshot ＋
//!   Drop→204 ガード（task 3.1）。GET は `ActorMsg::Get` に `Responder` を載せて enqueue
//!   し、対応する `Receiver<PocResponse>` を block-on-reply する。応答未送信のまま drop
//!   されれば 204 が撃たれ、SSP 側は無限待機しない。
//! - [`PocResponse`](pasta_lua::actor_poc::responder::PocResponse) — `Value(u64)` /
//!   `NoContent204`。GET の戻り値型として再利用（新しい応答型を作らない）。
//! - [`VerdictRecorder`](pasta_lua::actor_poc::verdict::VerdictRecorder) — marshaling
//!   不成立（R2.4）のブロッカー記録。
//!
//! GET の応答値・NOTIFY 完了通知は **値だけ** がチャネルで越境する（R2.3 スレッド分離）。
//! VM 本体（`mlua::Lua` の `!Send`）はアクタースレッドを越えない。
//!
//! # 出荷コード非干渉（R7・diff ゼロ）
//!
//! 本モジュールは `crate::util::parsers::req`（`pub(crate)`）を **読み取り専用** で再利用
//! するだけで、`shiori.rs` / `lua_request.rs` を一切改変しない。`actor-poc` feature 無効時は
//! コンパイル単位に現れない（バイト不変）。

use pasta_lua::actor_poc::actor_thread::ActorThread;
use pasta_lua::actor_poc::mailbox::ActorMsg;
use pasta_lua::actor_poc::responder::Responder;

/// PoC 応答表現の再エクスポート（GET の戻り値型）。Marshal は新しい応答型を作らず、
/// task 3.1 でレビュー済みの [`pasta_lua::actor_poc::responder::PocResponse`] を再利用する
/// （`Value(u64)` / `NoContent204`）。
pub use pasta_lua::actor_poc::responder::PocResponse;
/// アクタースレッド上の VM 実行スレッド id を数値化するヘルパの再エクスポート。
/// `Marshal` の GET 応答値（`PocResponse::Value`）はこの値で、テストが「アクター
/// スレッド上で実行された」ことを裏取りするのに使う（R2.3 の証跡値）。
pub use pasta_lua::actor_poc::actor_thread::thread_id_digits;

use crate::util::parsers::req::{Parser, Rule};
use pest::Parser as _;
use std::thread::ThreadId;

/// pest が確定する SHIORI メソッド（marshaling 分岐点）。
///
/// `lua_request.rs:86-87` と同じく `Rule::get` / `Rule::notify` から決まる値を、
/// Lua テーブルを介さず Rust の判別共用体として取り出す（決定論ロジックの Rust 寄せ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShioriMethod {
    /// GET（SHIORI/3.0 同期契約・block-on-reply）。
    Get,
    /// NOTIFY（fire-and-forget・即 204）。
    Notify,
}

/// 再パースの失敗理由（出荷経路と同様に pest の失敗を切り分ける）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReparseError {
    /// pest が request 文字列を解析できなかった（不正な SHIORI request）。
    PestFailed(String),
    /// 解析できたが GET/NOTIFY のいずれの method トークンも現れなかった。
    NoMethod,
}

impl std::fmt::Display for ReparseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReparseError::PestFailed(e) => write!(f, "pest re-parse failed: {e}"),
            ReparseError::NoMethod => write!(f, "no GET/NOTIFY method token in request"),
        }
    }
}

/// request 文字列を **既存 pest パーサで再パース** し、`method`（GET/NOTIFY）を得る。
///
/// `lua_request.rs:54/86-87`（`Parser::parse(Rule::req, text)?.flatten()` →
/// `Rule::get` / `Rule::notify`）と同一の読み取りロジックを、Lua VM を介さず Rust で
/// 再現する。**出荷パーサ（`Parser` / `Rule`）を改変せず読み取り専用で再利用** する
/// ため、出荷 `.rs` は diff ゼロ（design.md Q3「再パース」決定）。
pub fn reparse_method(request: &str) -> Result<ShioriMethod, ReparseError> {
    let pairs = Parser::parse(Rule::req, request)
        .map_err(|e| ReparseError::PestFailed(e.to_string()))?
        .flatten();
    for pair in pairs {
        match pair.as_rule() {
            Rule::get => return Ok(ShioriMethod::Get),
            Rule::notify => return Ok(ShioriMethod::Notify),
            _ => {}
        }
    }
    Err(ReparseError::NoMethod)
}

/// アクター上で 1 件の NOTIFY を完了した証跡（fire-and-forget の後追い観測用）。
///
/// NOTIFY は応答経路を持たない（design.md）。だが「SSP は待たなかったが、アクターは
/// 後から処理を完了する」ことを **観測可能** にするため、`Marshal` は NOTIFY を
/// **単一直列 mailbox の FIFO 性** を使って後追い確認する: NOTIFY を enqueue した直後に
/// 観測用 GET を enqueue すると、mailbox の直列処理により観測用 GET の応答は NOTIFY の
/// VM 実行 **完了後** に返る。応答値はアクタースレッド id 由来であり、NOTIFY VM が
/// アクタースレッドに閉じて実行されたことの証跡になる（R2.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotifyCompletion {
    /// 観測用 GET の応答に載った VM 実行スレッド id 由来の数値。
    pub actor_thread_digits: u64,
}

/// SSP スレッド側 marshaling ハンドル。
///
/// pest 再パースで得た `method` に応じて GET=block-on-reply／NOTIFY=即 204 へ分岐し、
/// アクタースレッド（`!Send` VM 所有）と値のみを marshaling する。
///
/// 自前のアクターループ／executor を持たず、pasta_lua の [`ActorThread`] を **保持** し、
/// その [`Mailbox`](pasta_lua::actor_poc::mailbox) 経由で [`ActorMsg`] を enqueue する。
pub struct Marshal {
    /// `!Send` VM をアクタースレッドに pin してホストする pasta_lua プリミティブ。
    /// `Marshal` の teardown はこのハンドルの `shutdown`/`Drop`（写経済み idiom）に委譲。
    actor: ActorThread,
}

impl Marshal {
    /// アクタースレッドを建てて Marshal を起動する。
    ///
    /// 自前で thread/executor を建てず、レビュー済みの [`ActorThread::spawn`]
    /// （`std::thread`＋`block_on`＋実 `PastaLuaRuntime`）に委譲する。
    pub fn spawn() -> Self {
        Marshal {
            actor: ActorThread::spawn(),
        }
    }

    /// アクタースレッドの id（VM が pin されているスレッド）。
    pub fn actor_thread_id(&self) -> ThreadId {
        self.actor.actor_thread_id()
    }

    /// pest 再パースで得た `method` に応じて marshaling する（design.md `Marshal::dispatch`）。
    ///
    /// - `Get`   → `ActorMsg::Get{responder}` を mailbox へ enqueue し、応答受信まで
    ///   ブロックして応答値を返す（SHIORI/3.0 同期契約）。アクターが reply せず drop した
    ///   場合は Responder ガードが 204 を撃ち、SSP 側は無限待機しない。
    /// - `Notify`→ `ActorMsg::Notify` を enqueue し、executor 完了を待たず **即 204** を
    ///   返す（fire-and-forget）。完了の後追い観測手段（観測用 GET）を第 2 戻り値で
    ///   返す（テスト用・SSP の応答経路では使わない）。
    ///
    /// 戻り値: `(応答, NOTIFY 完了観測クロージャ or None)`。
    pub fn dispatch(
        &self,
        method: ShioriMethod,
        script: &str,
    ) -> (PocResponse, Option<NotifyCompletion>) {
        match method {
            ShioriMethod::Get => (self.get(script, false), None),
            ShioriMethod::Notify => {
                self.notify(script);
                // fire-and-forget の即 204。完了の後追い観測は呼び出し側が
                // `observe_notify_completion` を呼ぶ（FIFO により NOTIFY 完了後に返る）。
                (PocResponse::NoContent204, None)
            }
        }
    }

    /// GET marshaling: `ActorMsg::Get`（`Responder` 内包）を enqueue し block-on-reply（R2.1）。
    ///
    /// `inject_no_reply = true` のとき、アクターが reply できないよう **意図的に失敗する
    /// Lua** を送る。アクターは VM 失敗時に responder を reply せず drop し、Responder の
    /// Drop ガードが 204 を撃つ（R2.4 の故障注入＝「応答未送信のまま drop」経路）。
    /// 応答受信までブロックし、応答値（または 204）を返す。
    pub fn get(&self, script: &str, inject_no_reply: bool) -> PocResponse {
        let (responder, reply_rx) = Responder::new();
        // R2.4: 故障注入時は VM が確実に失敗するスニペットを送る。アクターループは
        // VM 失敗 → responder を reply せず drop → Drop→204 ガード、という経路を通る。
        let vm_script = if inject_no_reply {
            "error('R2.4 fault injection: actor drops responder without replying')".to_string()
        } else {
            script.to_string()
        };

        if self
            .actor
            .submit(ActorMsg::Get {
                payload: 0,
                script: vm_script,
                responder,
            })
            .is_err()
        {
            // アクタースレッドが既に落ちている。enqueue 失敗時は responder も Err に
            // 同梱されて返り、ここで drop される → Drop ガードが 204 を撃つ。安全側として
            // 204 を返す（待機者を作らない）。
            return PocResponse::NoContent204;
        }

        // block-on-reply: アクターが reply（値）または drop→204 のいずれかを必ず返す
        // （Responder の不変条件「reply 1 回 or drop→204」）。
        reply_rx.recv().unwrap_or(PocResponse::NoContent204)
    }

    /// NOTIFY marshaling: `ActorMsg::Notify` を enqueue し **即** 制御を返す（R2.2）。
    ///
    /// 応答経路（`Responder`）を持たないため、SSP 側は executor 完了を待たない
    /// （待たないことが fire-and-forget の本質）。送信失敗（アクター不在）でも
    /// 契約上 SSP 側はブロックしない。
    pub fn notify(&self, script: &str) {
        let _ = self.actor.submit(ActorMsg::Notify {
            payload: 0,
            script: script.to_string(),
        });
    }

    /// NOTIFY 完了を **後から** 観測する（fire-and-forget の証跡）。
    ///
    /// 単一直列 mailbox の FIFO 性を利用する: 先行 NOTIFY の後にこの観測用 GET を
    /// enqueue すると、mailbox が直列処理するため観測用 GET の応答は **先行 NOTIFY の
    /// VM 実行完了後** に返る。応答値はアクタースレッド id 由来であり、NOTIFY/観測 GET
    /// の VM 操作がアクタースレッドに閉じていたこと（R2.3）を裏取りできる。
    ///
    /// 戻り値: 観測 GET が応答値（アクタースレッド id 由来）を返せば `Some`、
    /// 何らかの理由で 204 になれば `None`。
    pub fn observe_notify_completion(&self) -> Option<NotifyCompletion> {
        match self.get("return 0", false) {
            PocResponse::Value(actor_thread_digits) => Some(NotifyCompletion {
                actor_thread_digits,
            }),
            PocResponse::NoContent204 => None,
        }
    }

    /// shutdown フラグを立て `JoinHandle` を join して teardown する（`self` を消費）。
    ///
    /// teardown は保持する [`ActorThread::shutdown`]（写経済み idiom）に委譲する。
    pub fn shutdown(self) {
        let _ = self.actor.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 再パース: GET request 文字列から `ShioriMethod::Get` が得られる（出荷 `lua_request.rs`
    /// と同じ method トークン解釈）。
    #[test]
    fn reparse_extracts_get() {
        let req = "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: version\r\nSender: SSP\r\n\r\n";
        assert_eq!(reparse_method(req), Ok(ShioriMethod::Get));
    }

    /// 再パース: NOTIFY request 文字列から `ShioriMethod::Notify` が得られる。
    #[test]
    fn reparse_extracts_notify() {
        let req = "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: OnBoot\r\nSender: SSP\r\n\r\n";
        assert_eq!(reparse_method(req), Ok(ShioriMethod::Notify));
    }

    /// 再パース: 不正な request は `PestFailed` で切り分ける（ハングしない）。
    #[test]
    fn reparse_rejects_garbage() {
        let err = reparse_method("this is not a SHIORI request").unwrap_err();
        assert!(matches!(err, ReparseError::PestFailed(_)));
    }
}
