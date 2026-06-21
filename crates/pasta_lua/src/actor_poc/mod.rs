//! pasta-actor-feasibility 使い捨て検証ハーネス（PoC）。
//!
//! `actor-poc` feature 有効時のみコンパイルされる隔離モジュール。
//! `wintf_winmsg_executor` 上に `!Send` な `mlua` VM をアクタースレッドとして
//! ホストし、reload teardown・block-on-reply marshaling・drop→204 ガード・
//! coroutine 生存・キック配信・GET レイテンシの go/no-go を実証する。
//!
//! 後続タスク（1.4 以降）が `actor_thread` / `mailbox` / `responder` 等の
//! サブモジュールをここに追加する。本モジュールは出荷コードを改変せず、
//! feature 無効時はコンパイル単位に現れない（リリースビルドはバイト不変）。
