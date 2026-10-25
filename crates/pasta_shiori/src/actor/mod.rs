//! 本番アクターランタイムモジュール（design.md「File Structure Plan」`actor/`）。
//!
//! PoC 段階の足場（feature-gated の `actor_poc/`）は撤去済みで、本モジュール群が出荷経路である。
//! 本仕様（pasta-actor-runtime）は `!Send` な Lua VM をアクタースレッドへ pin し、
//! SHIORI スレッドはチャネル marshaling 経由でのみ VM へアクセスする構造を確立する。
//!
//! ## モジュール構成
//! task 5.1 で FFI 出荷経路（`windows.rs`）が [`lifecycle`] の置き場（芯の型 1 つ）へ
//! 配線され、本モジュール群はすべて **既定（no-feature）ビルドでコンパイル・リンク**される。
//! `unsafe impl Send/Sync` は撤去済み（R8）。
//! - [`mailbox`]: 単一直列 mailbox（flume unbounded・`ActorMsg{Get/Notify/Stop}`・
//!   単一 consumer・FIFO 順序保証・task 3.2）。
//! - [`thread`]: アクタースレッド起動・`!Send` VM pin・同期 `recv()` 単一ループで実
//!   `SHIORI.request` を駆動（task 3.3）。
//! - [`marshaling`]: GET=block-on-reply／NOTIFY=即 204／drop・timeout・異常→204（task 3.4）。
//! - [`teardown`]: `Stop{done}` ack→drain→cleanup→detach、reload リーク検査（task 4.1）。
//! - [`lifecycle`]: 旧来の入口の置き場（`static LEGACY`）と spawn/marshal/teardown 自由関数（FFI 入口）。

pub mod mailbox;

/// 本番アクタースレッド（task 3.3）。`!Send` VM を pin し mailbox を同期 `recv()`
/// のループで駆動する（OS のウィンドウやメッセージの仕組みを使わない）。task 5.1 で FFI 入口が
/// 本スレッドへ切り替わり、既定（no-feature）ビルドでコンパイルされる。
pub mod thread;

/// 本番 marshaling 層（task 3.4）。GET=block-on-reply（`recv_timeout`）／NOTIFY=即 204／
/// drop・timeout・異常→204 の同期契約を、SHIORI スレッドとアクタースレッド（task 3.3）の
/// 間で本番化する。task 5.1 で FFI 入口（`static LEGACY`）へ配線され、既定ビルドで
/// コンパイルされる。
pub mod marshaling;

/// 本番 teardown（task 4.1）。`ActorMsg::Stop { done }` 制御メッセージで drain→VM 破棄・
/// debug teardown→done ack を成立させ、SHIORI 側は ack を有界に待って
/// スレッドを detach する（join 廃止・冪等・異常記録）。reload 反復のリーク検査
/// （`GetProcessHandleCount`/`GetGuiResources`）も提供する。task 5.1 で FFI 入口
/// （`windows.rs` の load/unload）へ配線され、既定ビルドでコンパイルされる。
pub mod teardown;

/// 旧来の入口の置き場（`static LEGACY` に芯の型 1 つ ＋ spawn/marshal/teardown 自由関数）。
/// `OnceLock<RawShiori>`＋`Arc<Mutex<Option<PastaShiori>>>`＋`unsafe impl Send/Sync` を
/// 置換する本番ライフサイクル層。`windows.rs` の FFI extern 関数から呼ばれる。
pub mod lifecycle;

/// 芯の型 `ActorInstance`（仕様 `shiori4-v3-portable`）。インスタンス 1 つ＝アクター 1 つで、
/// mailbox の送信端・アクタースレッドのハンドル・ディレクトリ・要求の連番を持つ。
pub mod instance;
