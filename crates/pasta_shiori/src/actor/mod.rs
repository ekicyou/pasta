//! 本番アクターランタイムモジュール（design.md「File Structure Plan」`actor/`）。
//!
//! `actor_poc/`（feature-gated・default off の使い捨て足場）を出荷経路へ昇格する先。
//! 本仕様（pasta-actor-runtime）は `!Send` な Lua VM をアクタースレッドへ pin し、
//! SHIORI スレッドはチャネル marshaling 経由でのみ VM へアクセスする構造を確立する。
//!
//! ## モジュール構成
//! task 5.1 で FFI 出荷経路（`windows.rs`）が [`lifecycle`] の `static MAILBOX` 所有モデルへ
//! 配線され、本モジュール群はすべて **既定（no-feature）ビルドでコンパイル・リンク**される
//! （wintf は出荷依存へ昇格）。`unsafe impl Send/Sync` は撤去済み（R8）。
//! - [`mailbox`]: 単一直列 mailbox（flume unbounded・`ActorMsg{Get/Notify/Stop}`・
//!   単一 consumer・FIFO 順序保証・task 3.2）。
//! - [`thread`]: アクタースレッド起動・`!Send` VM pin・`recv_async().await` 単一ループで実
//!   `SHIORI.request` を駆動（task 3.3）。
//! - [`marshaling`]: GET=block-on-reply／NOTIFY=即 204／drop・timeout・異常→204（task 3.4）。
//! - [`teardown`]: `Stop{done}` ack→drain→cleanup→detach、reload リーク検査（task 4.1）。
//! - [`lifecycle`]: `static MAILBOX` 所有・spawn/marshal/teardown 自由関数（FFI 入口・task 5.1）。

pub mod mailbox;

/// 本番アクタースレッド（task 3.3）。`!Send` VM を pin し mailbox を `recv_async`
/// 駆動する。task 5.1 で FFI 入口が本スレッドへ切り替わり、wintf（`block_on`）が出荷
/// 依存へ昇格したため、既定（no-feature）ビルドでコンパイルされる。
pub mod thread;

/// 本番 marshaling 層（task 3.4）。GET=block-on-reply（`recv_timeout`）／NOTIFY=即 204／
/// drop・timeout・異常→204 の同期契約を、SHIORI スレッドとアクタースレッド（task 3.3）の
/// 間で本番化する。task 5.1 で FFI 入口（`static MAILBOX`）へ配線され、既定ビルドで
/// コンパイルされる。
pub mod marshaling;

/// 本番 teardown（task 4.1）。`ActorMsg::Stop { done }` 制御メッセージで drain→VM 破棄・
/// debug teardown・ウィンドウ破棄→done ack を成立させ、SHIORI 側は ack を有界に待って
/// スレッドを detach する（join 廃止・冪等・異常記録）。reload 反復のリーク検査
/// （`GetProcessHandleCount`/`GetGuiResources`）も提供する。task 5.1 で FFI 入口
/// （`windows.rs` の load/unload）へ配線され、既定ビルドでコンパイルされる。
pub mod teardown;

/// FFI 入口の所有モデル（`static MAILBOX` ＋ spawn/marshal/teardown 自由関数・task 5.1）。
/// `OnceLock<RawShiori>`＋`Arc<Mutex<Option<PastaShiori>>>`＋`unsafe impl Send/Sync` を
/// 置換する本番ライフサイクル層。`windows.rs` の FFI extern 関数から呼ばれる。
pub mod lifecycle;
