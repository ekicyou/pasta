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

/// アクタースレッド（block_on ＋ `!Send` VM pin・JoinHandle/shutdown idiom・R1.1/R2.3）。
pub mod actor_thread;

/// reload teardown と反復リーク検査（shutdown→再 spawn を N 回・実ハンドル計測・R1.2/R1.3）。
pub mod teardown;

/// R1 ブロッカー記録経路（VM ホスト/teardown 不成立条件を切り分け record_blocker・R1.4）。
pub mod r1_probe;

/// テスト隔離土台（socket2 エフェメラル待受の写経・R7.4）。
pub mod test_isolation;

/// 単一直列 mailbox（enqueue／drain・FIFO 順序・スレッド分離・R2.3）。
pub mod mailbox;

/// GET oneshot responder ＋ Drop で未送信時 204 ガード（R3.1/R3.2/R3.3/R3.4）。
pub mod responder;

/// 段階判定レコーダ土台（項目別 outcome／blocker 累積・隔離前提・R8.2/R8.3/R7.3）。
pub mod verdict;

/// CoroutineProbe（executor 駆動下で実 `*.lua` の coroutine resume／callback 生存・R4.1〜R4.4）。
pub mod coroutine_probe;

/// SimDriver 忠実シミュレータ（OnSecondChange 周期＋GET/NOTIFY≡Reference3 自前タグ付け＋Status: talking 遷移・R5.6）。
pub mod sim_driver;
