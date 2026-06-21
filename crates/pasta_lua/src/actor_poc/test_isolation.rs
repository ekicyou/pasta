//! テスト隔離土台（写経）。
//!
//! 本モジュールは `pasta_lua/src/debug/transport/mod.rs:170-176` の socket2
//! エフェメラル待受 idiom（`SO_REUSEADDR` を bind 前に設定 → port 0 で bind →
//! OS 割当ポートを取得）を**写経**したものである。debug 内部を import/改変せず、
//! 同じ作法を actor-poc 検証ハーネス向けに独立して持つ（design.md
//! 「Allowed Dependencies / 写経対象」・R7.4）。
//!
//! 目的: reload／bind を反復しても固定ポートを枯渇させないよう、毎回 OS が
//! 割り当てるエフェメラルポートで待受を成立させること（R1.3・R7.4）。
//! `ctor` による `PASTA_DEBUG`／`PASTA_DEBUG_PORT` 中和は dev-dependency の
//! 制約上テストバイナリ側（`tests/actor_poc_isolation.rs`）に写経する。

use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};

/// エフェメラルポートに束ねた `TcpListener` を返す。
///
/// `debug/transport/mod.rs` と同じ作法で socket2 を駆動する:
/// `SO_REUSEADDR` を bind 前に設定 → `127.0.0.1:0`（port 0）で bind →
/// listen → std `TcpListener` へ変換。port 0 を渡すことで OS が空きポートを
/// 割り当てるため、reload/bind を反復しても固定ポート枯渇が起きない。
///
/// 待受は loopback 固定（debug transport の loopback 方針を踏襲）。
pub fn bind_ephemeral_listener() -> std::io::Result<TcpListener> {
    // SO_REUSEADDR は bind より前に設定する必要がある（std `TcpListener::bind`
    // では bind 前設定ができないため raw socket を駆動する）。写経元:
    // transport/mod.rs:170-176。
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?; // SO_REUSEADDR（bind 前）
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)); // port 0 = エフェメラル
    socket.bind(&addr.into())?;
    socket.listen(1)?; // 単一クライアント設計に倣い小さい backlog
    Ok(TcpListener::from(socket))
}

/// エフェメラル待受を確立し、OS が割り当てた実ポート番号を返す。
///
/// 戻り値のポートは常に非 0（OS 割当済み）であることが保証される。呼び出し側は
/// listener を即座に drop するため、反復取得しても枯渇しない（`SO_REUSEADDR` ＋
/// port 0 ＋即時 drop の組合せ）。
pub fn ephemeral_port() -> std::io::Result<u16> {
    let listener = bind_ephemeral_listener()?;
    Ok(listener.local_addr()?.port())
}
