//! 本番アクターランタイムモジュール。
//!
//! `!Send` な Lua VM をアクタースレッドへ pin し、SHIORI スレッドはチャネル経由でのみ VM へアクセスする。

pub mod mailbox;

/// モジュールの参照（足す・返す・返してスレッドを終える）。アクタースレッドが pasta.dll を外させない。
pub(crate) mod module_ref;

/// 本番アクタースレッド。`!Send` VM を pin し mailbox を同期 `recv()` のループで駆動する。
pub mod thread;

/// 本番 marshaling 層。GET=block-on-reply／NOTIFY=即 204／drop・timeout・異常→204。
pub mod marshaling;

/// 本番 teardown。`ActorMsg::Stop { done }` で drain→VM 破棄→done ack を成立させる。
pub mod teardown;

/// 旧来の入口の置き場（`static LEGACY`）と spawn/marshal/teardown 自由関数（FFI 入口）。
pub mod lifecycle;

/// 芯の型 `ActorInstance`。インスタンス 1 つ＝アクター 1 つ。
pub mod instance;
