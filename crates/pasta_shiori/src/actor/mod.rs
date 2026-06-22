//! 本番アクターランタイムモジュール（design.md「File Structure Plan」`actor/`）。
//!
//! `actor_poc/`（feature-gated・default off の使い捨て足場）を出荷経路へ昇格する先。
//! 本仕様（pasta-actor-runtime）は `!Send` な Lua VM をアクタースレッドへ pin し、
//! SHIORI スレッドはチャネル marshaling 経由でのみ VM へアクセスする構造を確立する。
//!
//! ## モジュール構成（段階導入）
//! - [`mailbox`]: 単一直列 mailbox（flume unbounded・`ActorMsg{Get/Notify/Stop}`・
//!   単一 consumer・FIFO 順序保証）。**本タスク（3.2）で本番化**。
//! - thread.rs（task 3.3）: アクタースレッド起動・VM pin・`recv_async().await` 単一ループ。
//! - teardown.rs（task 7.x）: `Stop{done}` ack→drain→cleanup→detach。
//!
//! mailbox は flume のみに依存し wintf を要しないため default ビルドに同居できる
//! （出荷応答バイトには影響しない＝R7.2 バイト不変。FFI 入口への配線は task 5.1）。

pub mod mailbox;
