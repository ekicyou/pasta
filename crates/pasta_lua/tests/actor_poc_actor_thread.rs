//! R1（本丸）実証テスト: executor 上での `!Send` Lua VM ホスティング（task 2.1）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切汚染
//! しない（R7.1/R7.2/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 2.1）:
//! 「VM がアクタースレッド内で Lua 実行を完了し、スレッド境界を越えないことを
//!   assert するテストが緑。」
//!
//! 正直さ（テストが本当に失敗しうること）の設計:
//! - `ActorThread::spawn` は `std::thread::spawn` 内で `wintf_winmsg_executor::block_on`
//!   を回し、その future の中で **実 `PastaLuaRuntime`（`!Send` mlua VM）** を生成・所有し
//!   Lua を実行する。VM はその future（＝アクタースレッド）を越えない。
//! - VM が Lua を実行したスレッドの `ThreadId` を、Lua の戻り値（数値）とともに
//!   reply チャネルで持ち帰る。値だけが越境し、VM 本体は越境しない。
//! - テストは (a) Lua の計算結果が正しいこと、(b) 実行スレッド id が main/test
//!   スレッドと**異なる**こと、(c) `ActorThread::spawn` が報告するアクタースレッド id
//!   と Lua 実行スレッド id が**一致**すること を assert する。
//!   → VM/Lua が誤って test スレッドで動いた場合や、そもそも動かなかった場合に
//!     この assert は確実に失敗する。
//! - shutdown → `JoinHandle::join` のクリーンな単一 teardown を assert する。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::actor_thread::ActorThread;
use std::thread;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。`#[ctor]` 写経元:
/// `crates/pasta_lua/tests/common/mod.rs:26-32`（PASTA_DEBUG=1 がテストを壊す既知問題）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// R1.1 / R2.3: 実 `PastaLuaRuntime`（`!Send` VM）がアクタースレッド上で
/// `block_on` を介して生成・所有され、そこで Lua 実行を完了し、スレッド境界を
/// 越えないことを実証する。
#[test]
fn vm_hosts_lua_on_actor_thread_not_main_thread() {
    let main_thread_id = thread::current().id();

    // アクタースレッドを起動。中で block_on(actor future) が回り、future が
    // 実 PastaLuaRuntime を生成・所有して Lua を実行する。
    let handle = ActorThread::spawn();

    // アクタースレッド上で Lua を実行し、(計算結果, 実行スレッド id) を持ち帰る。
    // VM 本体は越境せず、値だけが reply で戻る。
    let outcome = handle
        .run_lua_probe("return 6 * 7")
        .expect("actor thread must execute Lua and return a value across the channel");

    // (a) Lua の計算が実 VM 上で正しく行われた。
    assert_eq!(
        outcome.lua_result, 42,
        "the !Send VM must actually evaluate Lua on the actor thread (6*7=42)"
    );

    // (b) Lua を実行したスレッドは main/test スレッドではない（スレッド境界越え）。
    assert_ne!(
        outcome.exec_thread_id, main_thread_id,
        "Lua must NOT execute on the main/test thread; the VM is pinned to the actor thread"
    );

    // (c) ActorThread が報告するアクタースレッド id と Lua 実行スレッド id が一致。
    //     これにより「VM が block_on を回したまさにそのアクタースレッド上で
    //     pin され実行された」ことが裏取りできる。
    assert_eq!(
        outcome.exec_thread_id, handle.actor_thread_id(),
        "the Lua exec thread must be exactly the actor thread that runs block_on (VM pinned there)"
    );

    // clean な単一 teardown: shutdown フラグ → JoinHandle::join。
    let report = handle.shutdown();
    assert!(
        report.joined_cleanly,
        "the actor thread must join cleanly on shutdown (no panic, single teardown)"
    );
    assert_eq!(
        report.actor_thread_id, outcome.exec_thread_id,
        "the joined thread must be the same actor thread that hosted the VM"
    );
}

/// 多重の Lua 実行が同一アクタースレッド上で連続して成立すること
/// （VM は spawn〜shutdown の間 pin され続け、毎回越境しない）。
#[test]
fn multiple_lua_probes_run_on_the_same_pinned_actor_thread() {
    let main_thread_id = thread::current().id();
    let handle = ActorThread::spawn();

    let first = handle.run_lua_probe("return 1 + 1").expect("first probe");
    let second = handle.run_lua_probe("return 10 + 10").expect("second probe");

    assert_eq!(first.lua_result, 2);
    assert_eq!(second.lua_result, 20);

    // どちらも main ではないアクタースレッド上で、しかも同一スレッドで実行された。
    assert_ne!(first.exec_thread_id, main_thread_id);
    assert_eq!(
        first.exec_thread_id, second.exec_thread_id,
        "the VM stays pinned to one actor thread across multiple probes"
    );
    assert_eq!(first.exec_thread_id, handle.actor_thread_id());

    let report = handle.shutdown();
    assert!(report.joined_cleanly);
}
