//! Shared, DAP-independent debug data types (DTOs) for the debug backend.
//!
//! These plain data types are promoted from the validated PoC harness
//! (`tests/runtime/lua_debug_poc_test/harness_types.rs`) and expanded to the
//! design's richer command/event vocabulary. They are reused across the debug
//! submodules (`session`, `inspect`, `breakpoints`, `hook`, `transport`,
//! `dap`) and form the seam between the VM-thread hook loop and the transport
//! thread.
//!
//! # Channel-seam invariant
//!
//! [`SessionCommand`] and [`SessionEvent`] cross a `std::sync::mpsc` channel
//! between the controller/transport and the VM-thread session loop. Every
//! payload here is therefore `Send`: all fields are `String` / `u32` / `usize`
//! / `Vec` / plain enums, with no raw pointers and no `mlua` handles.
//!
//! Crucially, `mlua::Error` is `!Send`; the [`SessionEvent::Error`] variant
//! carries the *stringified* error so VM/FFI failures can cross the thread
//! boundary (design "Invariants": never pass `mlua::Lua`/`mlua::Error` across
//! the channel — commands and events only).
//!
//! These are pure DTOs. Behavioural state machines (`RunMode`, `StepKind`,
//! `StepController`) live in `session.rs`, not here.

/// A single inspected variable (locals / upvalues), captured by the
/// FrameInspector.
///
/// `type_name` is the Lua type label (`number` / `string` / `boolean` /
/// `table` / `<unsupported T>`); `repr` is a display string the IDE can show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    /// Variable name as reported by the Lua debug API.
    pub name: String,
    /// Lua type label, e.g. `number`, `string`, `boolean`, `table`.
    pub type_name: String,
    /// Human-readable value representation for the IDE.
    pub repr: String,
}

/// One stack frame at a stop point (FrameInspector).
///
/// `source` and `line` are the generated `.lua` execution position (R2.1);
/// `func_name` is the frame's function name when the debug API can resolve it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameInfo {
    /// Source identifier (generated `.lua` source / chunk name).
    pub source: String,
    /// Execution line within `source`.
    pub line: u32,
    /// Function name for this frame, when resolvable.
    pub func_name: Option<String>,
}

/// Why execution stopped, reported to the controller in
/// [`SessionEvent::Stopped`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// Hit a registered breakpoint (R1.2).
    Breakpoint,
    /// Completed a step (over / in / out) (R1.3–R1.5).
    Step,
    /// Stopped at program entry.
    Entry,
    /// Stopped by an explicit pause request.
    Pause,
}

/// A plain source reference (DAP-independent).
///
/// Used by [`SessionCommand::SetBreakpoints`] to name the source whose lines
/// are being (un)set. Kept deliberately simple (a single `path` string) so it
/// carries no DAP types; the future `.pasta` source-map work substitutes a
/// `.pasta` path here without changing the shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceRef {
    /// Source path or chunk name (generated `.lua` by default).
    pub path: String,
}

impl SourceRef {
    /// Construct a [`SourceRef`] from any string-like path/name.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

/// A breakpoint target: `(source, line)`.
///
/// Promoted from the PoC, where breakpoints were stored in a
/// `HashSet<Breakpoint>`; kept as a tuple alias so it remains `Hash`/`Eq` for
/// set membership in `breakpoints.rs`.
pub type Breakpoint = (String, u32);

/// A breakpoint after resolution against the loaded source.
///
/// `verified` is `false` when the requested line could not be bound to an
/// executable location; the controller reports this back to the IDE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBreakpoint {
    /// The source this breakpoint resolved against.
    pub source: SourceRef,
    /// The (possibly adjusted) line the breakpoint is bound to.
    pub line: u32,
    /// Whether the breakpoint was successfully bound to an executable line.
    pub verified: bool,
}

/// A variable scope descriptor (locals / upvalues) for a stopped frame.
///
/// `variables_reference` is the opaque handle the controller passes back in
/// [`SessionCommand::Variables`] to enumerate this scope's variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// Display name of the scope, e.g. `Locals` / `Upvalues`.
    pub name: String,
    /// Opaque reference used to request this scope's variables.
    pub variables_reference: u32,
}

/// A debuggable thread (Lua coroutine) descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadInfo {
    /// Stable numeric id presented to the controller.
    pub id: u32,
    /// Human-readable thread name.
    pub name: String,
}

/// `Send`-safe identity of a running coroutine.
///
/// The underlying value is a `lua_State` pointer, but raw pointers are `!Send`
/// and cannot cross the channel seam, so the pointer is stored as a `usize`.
/// This is used ONLY for identity comparison across `resume` (design
/// "DebugSession 状態機械"); it is never dereferenced and must not be turned
/// back into a pointer for FFI use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(pub usize);

impl ThreadId {
    /// Build a [`ThreadId`] from a raw `lua_State` pointer by capturing its
    /// address as a `usize` (so the identity is `Send`).
    ///
    /// The pointer is only read as an address for identity; it is not retained
    /// or dereferenced.
    pub fn from_state(p: *mut mlua::ffi::lua_State) -> Self {
        Self(p as usize)
    }
}

/// One line-hook firing (promoted from the PoC).
///
/// Records the coroutine identity (`thread_ptr`, a `lua_State` address as
/// `usize`) so cross-coroutine firing can be distinguished. Consumed by the
/// hook in task 1.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineEvent {
    /// Source identifier of the firing line.
    pub source: String,
    /// Line number that fired.
    pub line: u32,
    /// Address (`usize`) of the firing coroutine's `lua_State`.
    pub thread_ptr: usize,
}

/// Commands from the controller / transport to the [`DebugSession`].
///
/// DAP-independent: the DAP adapter translates protocol requests into these.
/// All payloads are `Send` so they can cross the `std::sync::mpsc` seam.
///
/// [`DebugSession`]: crate::debug
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    /// Set the breakpoint lines for a source (replaces prior set for it).
    SetBreakpoints {
        /// Source the breakpoints apply to.
        source: SourceRef,
        /// Requested breakpoint lines.
        lines: Vec<u32>,
    },
    /// Resume until the next breakpoint or termination (R1.6).
    Continue,
    /// Step over the current line's calls (R1.3).
    Next,
    /// Step into the next call (R1.4).
    StepIn,
    /// Step out to the caller (R1.5).
    StepOut,
    /// Request the call stack of the stopped thread (R2.1).
    StackTrace,
    /// Request the scopes for a stopped frame.
    Scopes {
        /// Frame whose scopes are requested.
        frame_id: u32,
    },
    /// Request the variables for a scope/variable reference (R2.2).
    Variables {
        /// The `variables_reference` to enumerate.
        var_ref: u32,
    },
    /// Request the list of debuggable threads (coroutines).
    Threads,
    /// Disconnect and tear down the session.
    Disconnect,
}

/// Events from the [`DebugSession`] to the controller / transport.
///
/// DAP-independent: the DAP adapter translates these into protocol responses
/// and events. All payloads are `Send`.
///
/// The [`Error`](SessionEvent::Error) variant is MANDATORY: `mlua::Error` is
/// `!Send`, so VM/FFI failures are stringified to cross the thread boundary
/// (design "Invariants").
///
/// [`DebugSession`]: crate::debug
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    /// Execution stopped (R3.4); reports the reason and stopped thread.
    Stopped {
        /// Why execution stopped.
        reason: StopReason,
        /// Numeric id of the stopped thread.
        thread_id: u32,
    },
    /// Debug target execution terminated (R3.5).
    Terminated,
    /// Result of a [`SessionCommand::SetBreakpoints`].
    Breakpoints(Vec<ResolvedBreakpoint>),
    /// Result of a [`SessionCommand::StackTrace`].
    Stack(Vec<FrameInfo>),
    /// Result of a [`SessionCommand::Scopes`].
    Scopes(Vec<Scope>),
    /// Result of a [`SessionCommand::Variables`].
    Variables(Vec<Variable>),
    /// Result of a [`SessionCommand::Threads`].
    Threads(Vec<ThreadInfo>),
    /// A stringified error (e.g. an `mlua::Error`, which is `!Send`) crossing
    /// the thread/channel boundary.
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_round_trip() {
        let v = Variable {
            name: "x".to_string(),
            type_name: "number".to_string(),
            repr: "42".to_string(),
        };
        let cloned = v.clone();
        assert_eq!(v, cloned);
        assert_eq!(cloned.name, "x");
        assert_eq!(cloned.type_name, "number");
        assert_eq!(cloned.repr, "42");
    }

    #[test]
    fn frame_info_round_trip() {
        let f = FrameInfo {
            source: "@scene.lua".to_string(),
            line: 7,
            func_name: Some("talk".to_string()),
        };
        let cloned = f.clone();
        assert_eq!(f, cloned);
        assert_eq!(cloned.source, "@scene.lua");
        assert_eq!(cloned.line, 7);
        assert_eq!(cloned.func_name.as_deref(), Some("talk"));
    }

    #[test]
    fn session_event_error_carries_stringified_message() {
        // Proves the !Send-crossing path: mlua::Error must be stringified into
        // SessionEvent::Error to cross the channel boundary.
        let ev = SessionEvent::Error("lua boom".to_string());
        match ev {
            SessionEvent::Error(msg) => assert_eq!(msg, "lua boom"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn source_ref_round_trip() {
        let s = SourceRef::new("@scene.lua");
        assert_eq!(s, SourceRef { path: "@scene.lua".to_string() });
        assert_eq!(s.clone().path, "@scene.lua");
    }

    #[test]
    fn resolved_breakpoint_round_trip() {
        let bp = ResolvedBreakpoint {
            source: SourceRef::new("@scene.lua"),
            line: 12,
            verified: true,
        };
        let cloned = bp.clone();
        assert_eq!(bp, cloned);
        assert_eq!(cloned.line, 12);
        assert!(cloned.verified);
        assert_eq!(cloned.source.path, "@scene.lua");
    }

    #[test]
    fn breakpoint_tuple_is_hashable() {
        use std::collections::HashSet;
        let bp: Breakpoint = ("@scene.lua".to_string(), 3);
        let mut set: HashSet<Breakpoint> = HashSet::new();
        set.insert(bp.clone());
        assert!(set.contains(&bp));
    }

    #[test]
    fn scope_and_thread_info_round_trip() {
        let scope = Scope {
            name: "Locals".to_string(),
            variables_reference: 1001,
        };
        assert_eq!(scope.clone(), scope);
        assert_eq!(scope.variables_reference, 1001);

        let t = ThreadInfo {
            id: 1,
            name: "main".to_string(),
        };
        assert_eq!(t.clone(), t);
        assert_eq!(t.id, 1);
        assert_eq!(t.name, "main");
    }

    #[test]
    fn thread_id_identity_and_from_state() {
        // ThreadId is usize-backed for Send safety; identity compares by value.
        let a = ThreadId(0xDEAD);
        let b = ThreadId(0xDEAD);
        let c = ThreadId(0xBEEF);
        assert_eq!(a, b);
        assert_ne!(a, c);

        // Constructor captures a raw lua_State pointer's address as usize.
        let p: *mut mlua::ffi::lua_State = std::ptr::null_mut();
        assert_eq!(ThreadId::from_state(p), ThreadId(0));
    }

    #[test]
    fn line_event_round_trip() {
        let ev = LineEvent {
            source: "@scene.lua".to_string(),
            line: 5,
            thread_ptr: 0x1234,
        };
        assert_eq!(ev.clone(), ev);
        assert_eq!(ev.line, 5);
        assert_eq!(ev.thread_ptr, 0x1234);
    }

    #[test]
    fn session_command_variants_construct() {
        let set = SessionCommand::SetBreakpoints {
            source: SourceRef::new("@scene.lua"),
            lines: vec![1, 2, 3],
        };
        assert_eq!(set.clone(), set);
        assert_eq!(SessionCommand::Continue, SessionCommand::Continue);
        assert_ne!(SessionCommand::Next, SessionCommand::StepIn);
        let scopes = SessionCommand::Scopes { frame_id: 0 };
        let vars = SessionCommand::Variables { var_ref: 1001 };
        assert_ne!(scopes, vars);
    }

    #[test]
    fn session_event_stopped_carries_reason_and_thread() {
        let ev = SessionEvent::Stopped {
            reason: StopReason::Breakpoint,
            thread_id: 2,
        };
        match ev {
            SessionEvent::Stopped { reason, thread_id } => {
                assert_eq!(reason, StopReason::Breakpoint);
                assert_eq!(thread_id, 2);
            }
            other => panic!("expected Stopped, got {other:?}"),
        }
    }

    /// Compile-time proof that the channel-seam payloads are `Send`.
    fn _assert_send<T: Send>() {}

    #[test]
    fn command_and_event_are_send() {
        // If any field were !Send (e.g. a raw pointer or mlua handle), this
        // would fail to compile — guarding the std::sync::mpsc seam invariant.
        _assert_send::<SessionCommand>();
        _assert_send::<SessionEvent>();
        _assert_send::<ThreadId>();
        _assert_send::<LineEvent>();
    }
}
