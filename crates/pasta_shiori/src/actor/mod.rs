//! 本番アクターランタイムモジュール（design.md「File Structure Plan」`actor/`）。
//!
//! `actor_poc/`（feature-gated・default off の使い捨て足場）を出荷経路へ昇格する先。
//! 本仕様（pasta-actor-runtime）は `!Send` な Lua VM をアクタースレッドへ pin し、
//! SHIORI スレッドはチャネル marshaling 経由でのみ VM へアクセスする構造を確立する。
//!
//! ## モジュール構成（段階導入）
//! - [`mailbox`]: 単一直列 mailbox（flume unbounded・`ActorMsg{Get/Notify/Stop}`・
//!   単一 consumer・FIFO 順序保証）。**task 3.2 で本番化**。
//! - `thread`（task 3.3）: アクタースレッド起動・`!Send` VM pin・`recv_async().await`
//!   単一ループで実 `SHIORI.request` を駆動。**本タスク（3.3）で本番化**。wintf
//!   （`block_on`）を要するため `actor-poc` feature でガードする（FFI 配線＝task 5.1 で
//!   wintf が出荷依存へ昇格するまで、既定出荷ビルドに wintf を早期リンクしない）。
//! - teardown.rs（task 7.x）: `Stop{done}` ack→drain→cleanup→detach。
//!
//! mailbox は flume のみに依存し wintf を要しないため default ビルドに同居できる
//! （出荷応答バイトには影響しない＝R7.2 バイト不変。FFI 入口への配線は task 5.1）。

pub mod mailbox;

/// 本番アクタースレッド（task 3.3）。`!Send` VM を pin し mailbox を `recv_async`
/// 駆動する。wintf（`block_on`）依存のため `actor-poc` feature 有効時のみコンパイル。
#[cfg(feature = "actor-poc")]
pub mod thread;

/// 本番 marshaling 層（task 3.4）。GET=block-on-reply（`recv_timeout`）／NOTIFY=即 204／
/// drop・timeout・異常→204 の同期契約を、SHIORI スレッドとアクタースレッド（task 3.3）の
/// 間で本番化する。mailbox 型と wintf 依存のアクタースレッドを使うため、task 3.3 と同じく
/// `actor-poc` feature でガードする。FFI 入口（`static MAILBOX`）への配線は task 5.1。
#[cfg(feature = "actor-poc")]
pub mod marshaling;

/// 本番 teardown（task 4.1）。`ActorMsg::Stop { done }` 制御メッセージで drain→VM 破棄・
/// debug teardown・ウィンドウ破棄→done ack を成立させ、SHIORI 側は ack を有界に待って
/// スレッドを detach する（join 廃止・冪等・異常記録）。reload 反復のリーク検査
/// （`GetProcessHandleCount`/`GetGuiResources`）も提供する。アクタースレッド（wintf
/// 依存）と実 OS 計測（`windows-sys/Win32_System_Threading`）を使うため `actor-poc`
/// feature でガードする。FFI 入口（`windows.rs` の load/unload）への配線は task 5.1。
#[cfg(feature = "actor-poc")]
pub mod teardown;
