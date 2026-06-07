//! 共有型定義（LineEvent / Variable / FrameInfo / DebugCommand / DebugEvent /
//! Breakpoint / ItemOutcome / Tier）と、jit.off 済み VM 構築ヘルパ。
//!
//! ここで定義する型は兄弟サブモジュール（hook_probe / pause_gate /
//! frame_inspector / session / transport_loop / verdict）が後続タスクで利用する
//! ため `pub(crate)` で公開する。`HookStrategy` はここでは定義しない
//! （hook_probe.rs / タスク 2.1 の所有）。
//!
//! いくつかの型・関数は後続タスクでのみ消費されるため、ビルド警告を抑止する。

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// (A) 共有型定義（design.md "Components and Interfaces" のシグネチャに厳密準拠）
// ---------------------------------------------------------------------------

/// ラインフック発火 1 件分の記録（HookProbe）。
/// `thread_ptr` でコルーチンを識別し横断発火を判定する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LineEvent {
    pub source: String,
    pub line: u32,
    pub thread_ptr: usize,
}

/// inspect で取得した 1 変数（FrameInspector）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Variable {
    pub name: String,
    pub type_name: String,
    pub repr: String,
}

/// 停止フレームの情報（FrameInspector）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrameInfo {
    pub source: String,
    pub line: u32,
    pub func_name: Option<String>,
}

/// 外部（コントローラ / トランスポート）→ フックへのコマンド（PauseGate）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DebugCommand {
    Continue,
    Inspect,
}

/// フック → 外部へのイベント（PauseGate / TransportLoop）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DebugEvent {
    Stopped { source: String, line: u32 },
    Vars(Vec<Variable>),
}

/// ブレークポイント標的（source, line）。`HashSet<Breakpoint>` で利用する。
pub(crate) type Breakpoint = (String, u32);

/// チャレンジ項目（R1〜R4）1 件の結果（VerdictRecorder）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ItemOutcome {
    pub id: String,
    pub passed: bool,
    pub method: String,
    pub notes: String,
}

/// 段階的 GO 判定（VerdictRecorder）。R1→R2→R3→R4 の単調な積み上げ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tier {
    NoGo,
    ConditionalGo,
    Go,
    GoPlus,
}

// ---------------------------------------------------------------------------
// (B) jit-off VM 構築ヘルパ
// ---------------------------------------------------------------------------

/// ALL_SAFE（`jit` を含み `debug` を含まない）で VM を構築し、
/// 無引数 `jit.off()` を適用して JIT エンジン全体を停止した `Lua` を返す。
///
/// JIT 無効化はラインフックの取りこぼし（未発火行）を防ぐための前提
/// （Requirement 1.4 / design "HookProbe Responsibilities" + "jit.off
/// セマンティクス注"）。
///
/// **重要（PoC 知見）**: `jit.off(true, true)` は呼び出し元関数とその下位関数
/// のみを対象とする**関数単位**の制御で、`jit.status()` が報告する**グローバル
/// エンジン状態は変えない**。別チャンクで exec しても後続ロードのシーンチャンクや
/// 動的生成コルーチンには波及しないため、取りこぼし防止の目的を満たさない。
/// VM 全体へ確実に効かせるには**無引数 `jit.off()`**（エンジン自体の停止、
/// `jit.status()` を `false` 化）を用いる。
///
/// VM 構築には既存の生 VM ヘルパ
/// `common::e2e_helpers::create_runtime_with_finalize()` を流用する。
pub(crate) fn build_jit_off_vm() -> mlua::Result<mlua::Lua> {
    let lua = crate::common::e2e_helpers::create_runtime_with_finalize()?;
    // 無引数 jit.off() で JIT エンジン自体を停止（VM 全体に波及、ラインフック
    // 取りこぼし防止）。
    lua.load("jit.off()").exec()?;
    Ok(lua)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ヘルパ構築 VM で JIT エンジンがグローバルに無効化されていること
    /// （Requirement 1.4 / design "jit.off セマンティクス注"）。
    ///
    /// `jit.status()` の第 1 戻り値は **JIT エンジンがグローバルに有効か** を表す。
    /// 無引数 `jit.off()` を適用した本ヘルパでは `false` でなければならない
    /// （`jit.off(true, true)` のような関数単位フォームでは `true` のまま）。
    #[test]
    fn helper_vm_has_jit_disabled() {
        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        // 無引数 jit.off() で JIT エンジン自体が停止し、jit.status() 第 1 戻り値が
        // false になることを観測する（VM 全体への確実な適用の証拠）。
        let jit_enabled: bool = lua
            .load("return (jit.status())")
            .eval()
            .expect("jit.status() should be callable on the helper-built VM");
        assert_eq!(
            jit_enabled, false,
            "no-arg jit.off() must disable the global JIT engine (Req 1.4)"
        );
    }

    /// ヘルパ構築 VM で `debug`（std_debug）が露出していないこと（Requirement 5.3）。
    #[test]
    fn helper_vm_keeps_debug_sandboxed() {
        let lua = build_jit_off_vm().expect("build_jit_off_vm should succeed");
        let debug_is_nil: bool = lua
            .load("return debug == nil")
            .eval()
            .expect("eval should succeed");
        assert!(
            debug_is_nil,
            "std_debug must NOT be exposed (sandbox maintained)"
        );
    }

    /// 共有型が構築・等値比較できること（後続タスクの利用前提の最小確認）。
    #[test]
    fn shared_types_construct_and_compare() {
        let ev = LineEvent {
            source: "@chunk".to_string(),
            line: 1,
            thread_ptr: 0,
        };
        assert_eq!(ev.clone(), ev);

        let bp: Breakpoint = ("@chunk".to_string(), 1);
        let mut set = std::collections::HashSet::new();
        set.insert(bp.clone());
        assert!(set.contains(&bp));

        assert_eq!(DebugCommand::Continue, DebugCommand::Continue);
        assert_ne!(Tier::NoGo, Tier::GoPlus);
    }
}
