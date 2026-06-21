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

/// テスト隔離土台（socket2 エフェメラル待受の写経・R7.4）。
pub mod test_isolation;

/// 単一直列 mailbox（enqueue／drain・FIFO 順序・スレッド分離・R2.3）。
pub mod mailbox;

/// 段階判定レコーダ土台（項目別 outcome／blocker 累積・隔離前提・R8.2/R8.3/R7.3）。
pub mod verdict;
