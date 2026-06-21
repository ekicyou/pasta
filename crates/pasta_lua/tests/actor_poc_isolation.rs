//! actor-poc テスト隔離土台の実証テスト（R7.4）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこの
//! テストバイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを
//! 一切汚染しない（R7.1/R7.2/R7.4）。
//!
//! 写経内容:
//! - `#[ctor]` による `PASTA_DEBUG`／`PASTA_DEBUG_PORT` 中和（写経元:
//!   `tests/common/mod.rs:26-32`）。固定ポート 9276 を握る DAP リスナの
//!   暗黙有効化を防ぎ、env 汚染でテストが壊れないようにする。
//! - socket2 `set_reuse_address` ＋ port 0 のエフェメラル待受
//!   （`actor_poc::test_isolation`、写経元: `debug/transport/mod.rs:170-176`）。
//!
//! 観測可能な完了条件: エフェメラル待受の反復取得が固定ポート枯渇・
//! `PASTA_DEBUG` 汚染なしで成立すること。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::test_isolation;

/// `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前（プロセスが単一スレッドの間）に
/// 中和する。`std::env::remove_var` は Rust 2024 edition で `unsafe` のため、
/// レース回避のために `#[ctor]`（before-main）で実行する。
/// 写経元: `crates/pasta_lua/tests/common/mod.rs:26-32`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// env 中和が in-process で効いていることを直接 assert する。
///
/// `#[ctor]` ガードが壊れていれば（または除去されれば）開発セッションが
/// export した `PASTA_DEBUG=1` がそのまま残り、この assert が失敗する。
#[test]
fn debug_env_is_neutralized_in_process() {
    assert!(
        std::env::var_os("PASTA_DEBUG").is_none(),
        "PASTA_DEBUG must be neutralized by the #[ctor] guard before any test runs"
    );
    assert!(
        std::env::var_os("PASTA_DEBUG_PORT").is_none(),
        "PASTA_DEBUG_PORT must be neutralized by the #[ctor] guard before any test runs"
    );
}

/// エフェメラル待受を**反復**取得しても固定ポート枯渇しないことを実証する。
///
/// scaffold が壊れていれば（例: port 0 でなく固定ポートを使う・SO_REUSEADDR を
/// 設定しない）反復 bind が `AddrInUse` で失敗する、または割当ポートが 0 になり
/// この assert が失敗する。あわせて env 中和が反復中も維持されることを確認する。
#[test]
fn repeated_ephemeral_bind_does_not_exhaust_fixed_port() {
    const ITERATIONS: usize = 50;

    let mut seen_ports = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        // env 汚染が反復中も無いこと（scaffold の前提）。
        assert!(
            std::env::var_os("PASTA_DEBUG").is_none(),
            "iteration {i}: PASTA_DEBUG leaked into the test process"
        );

        let listener = test_isolation::bind_ephemeral_listener()
            .unwrap_or_else(|e| panic!("iteration {i}: ephemeral bind failed: {e}"));
        let port = listener
            .local_addr()
            .unwrap_or_else(|e| panic!("iteration {i}: local_addr failed: {e}"))
            .port();

        // OS が実ポートを割り当てている（固定ポート/未割当でない）。
        assert_ne!(port, 0, "iteration {i}: bound port must be a real OS-assigned ephemeral port, not 0");
        seen_ports.push(port);
        // listener はスコープ末尾で drop され、SO_REUSEADDR により即座に再利用可能。
    }

    // 反復回数ぶんの取得が成立したこと（途中の枯渇/失敗で panic していない）。
    assert_eq!(
        seen_ports.len(),
        ITERATIONS,
        "all {ITERATIONS} ephemeral binds should have succeeded without fixed-port exhaustion"
    );

    // エフェメラルである以上、固定ポートに張り付いていないこと（ほぼ確実に複数の
    // 異なるポートが観測される）を緩く確認する。万一すべて同一なら固定ポート
    // 疑いの強いシグナルとして失敗させる。
    let distinct = seen_ports.iter().collect::<std::collections::HashSet<_>>().len();
    assert!(
        distinct > 1,
        "expected multiple distinct ephemeral ports across {ITERATIONS} binds, got {distinct} (looks like a fixed port)"
    );
}

/// `ephemeral_port` ヘルパが非 0 のポートを返すことを確認する。
#[test]
fn ephemeral_port_helper_returns_nonzero() {
    let port = test_isolation::ephemeral_port().expect("ephemeral_port should succeed");
    assert_ne!(port, 0, "ephemeral_port must return a real OS-assigned port");
}
