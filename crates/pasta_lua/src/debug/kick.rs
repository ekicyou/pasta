//! Generic kick sink seam (`KickSinkSeam`) — host-agnostic injection point.
//!
//! This module defines the *type-only* boundary by which an outer host
//! (`pasta_shiori`) can hand `pasta_lua` a closure that delivers a scene-kick
//! request, without `pasta_lua` knowing anything about the request's eventual
//! destination (`ActorMsg` / `MAILBOX`). The dependency direction
//! `pasta_shiori` → `pasta_lua` is preserved: this module never references
//! `pasta_shiori` (requirements 2.4, 2.6).
//!
//! # Wiring (later tasks)
//!
//! [`RuntimeConfig::with_kick_sink`](crate::RuntimeConfig::with_kick_sink) stores
//! an `Option<KickSink>`. When the debug backend is enabled, task 2.x forwards
//! the sink through `enable` into the socket-bridge so an inbound `playScene`
//! request invokes it. When the sink is not injected (`None`) — the default —
//! the kick path is never activated (requirement 2.6).

use std::sync::Arc;

/// Minimal kick request carried across the crate boundary.
///
/// Holds only the named scene. `pasta_lua` treats this opaquely: it does not
/// resolve the scene or know how the request is ultimately consumed.
#[derive(Debug, Clone)]
pub struct KickRequest {
    /// The named scene to kick.
    pub scene: String,
}

/// Type-erased, thread-crossing kick sink.
///
/// `pasta_shiori` injects a closure of this shape (typically a `MAILBOX`
/// `try_send`) via [`RuntimeConfig::with_kick_sink`](crate::RuntimeConfig::with_kick_sink).
/// `pasta_lua` holds it opaquely and (in later tasks) calls it from the
/// socket-bridge thread, hence the `Send + Sync` bound.
pub type KickSink = Arc<dyn Fn(KickRequest) + Send + Sync>;
