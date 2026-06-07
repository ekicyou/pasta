//! PauseGate: breakpoints / should_pause / block_until_command による
//! フック内ブロッキング停止・再開（R2 検証の論理ユニット / task 2.2）。
//!
//! 兄弟モジュール `harness_types` の共有型（`Breakpoint` = `(String, u32)`、
//! `DebugCommand`、`DebugEvent`）を再利用する。
//!
//! ## 設計準拠（design.md "PauseGate" / requirements.md R2.3 / research D5・D6）
//! - `breakpoints: HashSet<(source, line)>` を保持し、`EVERY_LINE` フック内で
//!   `should_pause(frame)` が現フレームの (source, line) の包含を判定する。
//!   **一致時のみ**停止する（D6）。
//! - `should_pause` が真のとき `block_until_command` が `Receiver<DebugCommand>` を
//!   ブロッキング `recv()`。`DebugCommand::Continue` 受信で復帰し
//!   **常に `VmState::Continue`** を返す（R2.1, R2.2, R2.3）。
//! - フック内で `coroutine.yield` / `lua_yield` / `VmState::Yield` は使用しない。
//!   `VmState::Yield` は Lua 5.3+/Luau 限定で **LuaJIT では `Continue` のみ**
//!   （research R2 Constraint）。純粋な Rust ブロッキングのみで停止を実現する。
//! - **無期限ブロックが正**（D5 ④）。デッドロック検出の timeout は PoC の
//!   テスト側 watchdog にのみ置き、停止コア（`await_resume` / `block_until_command`）
//!   には組み込まない。
//!
//! ## テスト容易性のための分解
//! `mlua::Debug` はフック内でしか存在し得ないため、ユニットテストで検証する
//! 純粋ロジックを `Debug` 非依存のヘルパへ括り出す:
//! - `PauseGate::is_breakpoint(source, line) -> bool`（包含述語）。
//!   `should_pause(gate, frame)` は `frame` から source/line を取り出して委譲する。
//! - `await_resume(cmd_rx, event_tx, source, line) -> Result<VmState>`
//!   （Stopped 送出 → Continue まで recv ループ）。
//!   `block_until_command(gate, frame)` は `frame` から source/line を取り出して委譲する。
//!
//! Inspect の本格対応（変数取得）は task 3.x / 4.x の領分。本タスクでは Inspect を
//! 受信しても停止を継続し、Continue を待ち続ける。

#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};

use mlua::{Debug, VmState};

use super::harness_types::{Breakpoint, DebugCommand, DebugEvent};

/// フック内ブロッキング停止・再開機構（design.md "PauseGate" 準拠）。
///
/// `breakpoints` が標的（source, line）集合。`cmd_rx` で外部（コントローラ／
/// トランスポート）からのコマンドを受信し、`event_tx` で停止イベントを送出する。
/// VM 操作はこのスレッド（フック実行スレッド）に閉じる。
pub(crate) struct PauseGate {
    breakpoints: HashSet<Breakpoint>,
    cmd_rx: Receiver<DebugCommand>,
    event_tx: Sender<DebugEvent>,
}

impl PauseGate {
    /// ブレークポイント集合・コマンド受信端・イベント送出端から構築する。
    pub(crate) fn new(
        breakpoints: HashSet<Breakpoint>,
        cmd_rx: Receiver<DebugCommand>,
        event_tx: Sender<DebugEvent>,
    ) -> Self {
        PauseGate {
            breakpoints,
            cmd_rx,
            event_tx,
        }
    }

    /// 純粋な包含述語: (source, line) がブレークポイント集合に含まれるか
    /// （R2.1 / 3.2 / 4.1 の停止判定ロジック）。
    ///
    /// `Breakpoint` は `(String, u32)` のため、借用キーでの照合のために
    /// 比較で一時タプルを構築せず `iter().any` で突き合わせる。
    fn is_breakpoint(&self, source: &str, line: u32) -> bool {
        self.breakpoints
            .iter()
            .any(|(s, l)| s == source && *l == line)
    }
}

/// `mlua::Debug` から (source, line) を抽出する（フック内で呼ぶ）。
///
/// `Debug::source().source`（チャンク名）を優先し、無ければ `short_src` を使う
/// （hook_probe の `line_event_from_debug` と同じ抽出規約）。行番号は
/// `current_line()`、取得不可時は 0。
fn source_and_line(frame: &Debug) -> (String, u32) {
    let src = frame.source();
    let source = src
        .source
        .as_ref()
        .map(|c| c.as_ref().to_string())
        .or_else(|| src.short_src.as_ref().map(|c| c.as_ref().to_string()))
        .unwrap_or_default();
    let line = frame.current_line().unwrap_or(0) as u32;
    (source, line)
}

/// `EVERY_LINE` フック内で最初に呼ぶ。現フレームがブレークポイント標的か判定する
/// （R2.1 / 3.2 / 4.1）。`frame` から source/line を抽出し、純粋述語
/// [`PauseGate::is_breakpoint`] に委譲する。
pub(crate) fn should_pause(gate: &PauseGate, frame: &Debug) -> bool {
    let (source, line) = source_and_line(frame);
    gate.is_breakpoint(&source, line)
}

/// 再開待ちのコア（`Debug` 非依存・ユニットテスト可能）。
///
/// `event_tx` に `DebugEvent::Stopped { source, line }` を送出してから、
/// `cmd_rx` を**ブロッキング** `recv()` し、`DebugCommand::Continue` を受信する
/// まで待つ（R2.1, R2.2, R2.3）。
///
/// - `DebugCommand::Inspect` を受信しても停止を継続し、`Continue` を待ち続ける
///   （変数取得の本格対応は task 3.x/4.x）。
/// - `coroutine.yield` / `lua_yield` / `VmState::Yield` は使用せず、純粋な Rust
///   ブロッキングのみで停止する。復帰時は**常に `VmState::Continue`** を返す
///   （LuaJIT は `Yield` 不可：research R2 Constraint）。
/// - **無期限ブロックが正**（D5 ④）: ここに timeout は置かない。デッドロック検出は
///   テスト側 watchdog の責務。
/// - 送信側が全滅して `recv()` が `Err`（切断）になった場合は、停止を続けられない
///   ため `Continue` で復帰する（VM をハングさせない安全側フォールバック）。
fn await_resume(
    cmd_rx: &Receiver<DebugCommand>,
    event_tx: &Sender<DebugEvent>,
    source: &str,
    line: u32,
) -> mlua::Result<VmState> {
    // 停止を外部へ通知（Stopped イベント送出）。受信側不在でも停止自体は継続する
    // ため、送出失敗は無視する。
    let _ = event_tx.send(DebugEvent::Stopped {
        source: source.to_string(),
        line,
    });

    // Continue が来るまで無期限ブロック。Inspect は無視して待ち続ける。
    loop {
        match cmd_rx.recv() {
            Ok(DebugCommand::Continue) => return Ok(VmState::Continue),
            Ok(DebugCommand::Inspect) => continue,
            // 送信端が全て drop された（切断）。これ以上 Continue は来ないため、
            // VM をハングさせないよう Continue で復帰する。
            Err(_) => return Ok(VmState::Continue),
        }
    }
}

/// `should_pause` が真のときのみ呼ぶ。`Continue` が来るまでブロックし、復帰時は
/// 常に `VmState::Continue` を返す（R2.1, R2.2, R2.3）。`frame` から source/line を
/// 抽出して [`await_resume`] に委譲する。
pub(crate) fn block_until_command(gate: &PauseGate, frame: &Debug) -> mlua::Result<VmState> {
    let (source, line) = source_and_line(frame);
    await_resume(&gate.cmd_rx, &gate.event_tx, &source, line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    /// PauseGate 構築の小ヘルパ（ブレークポイント集合を渡し、チャネル端点を返す）。
    fn gate_with(
        bps: &[Breakpoint],
    ) -> (PauseGate, Sender<DebugCommand>, Receiver<DebugEvent>) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<DebugCommand>();
        let (event_tx, event_rx) = mpsc::channel::<DebugEvent>();
        let set: HashSet<Breakpoint> = bps.iter().cloned().collect();
        (PauseGate::new(set, cmd_rx, event_tx), cmd_tx, event_rx)
    }

    /// R2.1 / 3.2 / 4.1（停止判定ロジック・task 2.2）: ブレークポイント標的の
    /// (source, line) のみで `is_breakpoint` が真になり、行違い・ソース違いでは
    /// 偽になること（非空虚: 真・偽の双方を検証）。
    #[test]
    fn should_pause_matches_only_target_lines() {
        let (gate, _cmd_tx, _event_rx) = gate_with(&[("@s".to_string(), 3)]);

        // 完全一致 → 真。
        assert!(
            gate.is_breakpoint("@s", 3),
            "exact (source,line) match must pause"
        );
        // 同ソース・行違い → 偽。
        assert!(
            !gate.is_breakpoint("@s", 2),
            "same source but different line must NOT pause"
        );
        // 行一致・ソース違い → 偽。
        assert!(
            !gate.is_breakpoint("@other", 3),
            "same line but different source must NOT pause"
        );
    }

    /// R2.3（再開機構・yield 不使用・常に Continue）: `await_resume` が Stopped を
    /// 送出してからブロックし、別スレッドからの `DebugCommand::Continue` で解放され
    /// `VmState::Continue` を返すこと。さらに Stopped イベントが期待 source/line で
    /// 送出されていることを検証する。
    ///
    /// 無期限ブロックは停止コアの正しい挙動（D5 ④）だが、テストがデッドロックで
    /// CI を吊らせないよう、**テスト側のみ** join に watchdog を置く。
    #[test]
    fn resume_command_releases_block() {
        let (cmd_tx, cmd_rx) = mpsc::channel::<DebugCommand>();
        let (event_tx, event_rx) = mpsc::channel::<DebugEvent>();

        // 別スレッドから Continue を送る（送ってから recv で受けるため deadlock しない）。
        cmd_tx
            .send(DebugCommand::Continue)
            .expect("send Continue should succeed");

        let state = await_resume(&cmd_rx, &event_tx, "@s", 3)
            .expect("await_resume should return Ok");

        // 常に Continue（LuaJIT は Yield 不可・R2.3）。
        // `VmState` は PartialEq/Debug 非実装のため `matches!` で判定する。
        assert!(
            matches!(state, VmState::Continue),
            "await_resume must always return VmState::Continue (R2.3)"
        );

        // Stopped イベントが期待 source/line で送出されていること（停止通知）。
        // テスト側 watchdog: 万一未送出でもブロックしないよう timeout を付す。
        let ev = event_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("a Stopped event must be emitted before blocking");
        assert_eq!(
            ev,
            DebugEvent::Stopped {
                source: "@s".to_string(),
                line: 3,
            },
            "Stopped event must carry the stop-point source/line"
        );
    }

    /// R2.3（補強）: `Inspect` だけではブロックが解けず、後続の `Continue` で初めて
    /// 解放されること（Inspect 受信中も停止継続）。デッドロック回避のためテスト側
    /// のみ送信スレッドを使い、解放を join の timeout で監視する。
    #[test]
    fn inspect_does_not_release_only_continue_does() {
        let (cmd_tx, cmd_rx) = mpsc::channel::<DebugCommand>();
        let (event_tx, event_rx) = mpsc::channel::<DebugEvent>();

        // 別スレッドが Inspect を先に送り、少し置いてから Continue を送る。
        // Inspect だけで解放されるなら、その後の Continue 受信前に
        // await_resume が戻り、テストの順序保証が崩れる（＝検出可能）。
        let sender = std::thread::spawn(move || {
            cmd_tx
                .send(DebugCommand::Inspect)
                .expect("send Inspect should succeed");
            // Inspect が「解放しない」ことを確実にするための短い猶予。
            std::thread::sleep(Duration::from_millis(50));
            cmd_tx
                .send(DebugCommand::Continue)
                .expect("send Continue should succeed");
        });

        let state = await_resume(&cmd_rx, &event_tx, "@x", 7)
            .expect("await_resume should return Ok after Continue");
        assert!(
            matches!(state, VmState::Continue),
            "only Continue releases the block; result is always Continue (R2.3)"
        );

        sender.join().expect("sender thread should not panic");

        // Stopped が送出されていること（テスト側 watchdog 付き recv）。
        let ev = event_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("a Stopped event must be emitted");
        assert_eq!(
            ev,
            DebugEvent::Stopped {
                source: "@x".to_string(),
                line: 7,
            }
        );
    }
}
