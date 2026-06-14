//! Task 8.2 — ゼロコスト/サンドボックス集約回帰ゲート（`debug_integration_test.rs` から
//! C2 クラスタ分割で外出し）。元ファイルの内側 `mod zero_cost_sandbox_regression` を
//! **バイト不変**で移設したもの（テスト名 `zero_cost_sandbox_regression::*` は不変・モジュール
//! ラッパも保持）。共有 DAP ハーネスには依存せず、必要な型のみを直接 import する。

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use pasta_lua::{DebugConfig, PastaLuaRuntime, RuntimeConfig, TranspileContext};

/// Task 8.2 — ゼロコスト/サンドボックス **集約回帰ゲート**（Performance/Regression）。
///
/// このモジュールは「デバッグ無効時の不変条件」を **一箇所に集約** し、各テストを
/// 対応する要件へ明示マッピングする durable な回帰ゲートである。将来の変更が
/// これらを暗黙に退行させられないよう、アサーションは可能な限り **直接的かつ強力**
/// にする（4.2/5.1 が既に保証する内容の上に、より強い信号を積む）。
///
/// 要件マッピング（`.kiro/specs/pasta-vscode-lua-debug/requirements.md`）:
/// - **R5.2**: 無効時はデバッグ用フックを設置せず、本番実行に追加コストを与えない。
///   → [`r5_2_disabled_installs_no_hook_jit_stays_on`]
/// - **R5.3**: 無効時は `debug`／`std_debug` をスクリプトへ露出せず、サンドボックスを維持する。
///   → [`r5_3_disabled_keeps_sandbox_debug_is_nil`]
/// - **R5.5**: 無効時は接続待ち受け口を開かない（`debug_local_addr()==None`）。
///   → [`r5_5_disabled_opens_no_port`]
///
/// （R4.6/5.2 の「本番 transpile 出力バイト一致」は、トランスパイラ API へアクセスする
/// `tests/transpiler/source_map_seam_test.rs` の `zero_cost_sandbox_regression` モジュールで
/// 集約アサートする — ランタイムターゲットからは code_gen の本番 API に届かないため分割。）
///
/// design.md 参照: "Testing Strategy / Integration Tests"（無効時ゼロコスト/サンドボックス:
/// hook 痕跡なし・`std_debug` 非露出・接続口非開放 — 5.2, 5.3, 5.5）、
/// "Performance/Regression"、"DebugConfig & Gate"（無効時 listen=None・hook 非設置）。
mod zero_cost_sandbox_regression {
    use super::*;

    /// `default_debug_port()` と同値（`pasta.toml`/`PASTA_DEBUG_PORT` 未設定時の既定）。
    /// テストはこの値で best-effort の「リスナ不在」確認を行う（権威判定は
    /// `debug_local_addr()==None`。ポート競合での flaky を避けるため connect-refused は
    /// あくまで補助シグナル扱い）。
    const DEFAULT_DEBUG_PORT: u16 = 9276;

    /// 無効ランタイムを構築するヘルパー。`RuntimeConfig::minimal()` は debug 無効
    /// （`default_runtime_config_debug_is_disabled` が保証）で、ALL_SAFE 相当のサンドボックス。
    fn disabled_runtime() -> PastaLuaRuntime {
        PastaLuaRuntime::with_config(TranspileContext::new(), RuntimeConfig::minimal())
            .expect("disabled runtime must build")
    }

    /// **R5.2 — 無効時はフック非設置（jit は ON のまま）**。
    ///
    /// enable パスはフック内でエンジン全体に `jit.off()` を適用するため、複数行スクリプト
    /// 実行後も JIT が ON のままであることは「**フック非設置＝per-line デバッグコストなし**」
    /// の強い証拠になる。さらに直接的な不変条件として `debug_enabled()==false`
    /// （`DebugHandle` 不保持）も併せて表明する。
    #[test]
    fn r5_2_disabled_installs_no_hook_jit_stays_on() {
        let runtime = disabled_runtime();

        // 直接表明: 無効ランタイムは DebugHandle を保持しない（フック設置の前提が無い）。
        assert!(
            !runtime.debug_enabled(),
            "R5.2: disabled runtime must NOT hold a DebugHandle (no hook installed)"
        );

        // 複数行スクリプトを実行しても JIT は ON のまま（enable なら jit.off() で OFF になる）。
        // 行フックが一度でも走れば JIT は無効化されているはずなので、ON のままであることは
        // 「行フックが一度も発火していない＝デバッグコスト 0」の直接的痕跡。
        let jit_on_after_run: bool = runtime
            .exec(
                "\
local sum = 0
for i = 1, 1000 do
  sum = sum + i
end
return jit ~= nil and jit.status() == true",
            )
            .expect("multi-line eval ok")
            .as_boolean()
            .expect("boolean result");
        assert!(
            jit_on_after_run,
            "R5.2: JIT must remain ON after a multi-line run (no hook → no engine-wide jit.off())"
        );
    }

    /// **R5.3 — サンドボックス維持（`debug` 非露出）**。
    ///
    /// 無効ランタイムはスクリプトへ `debug`/`std_debug` を露出しない。`debug == nil` を
    /// 表明し、さらにスタック introspection（`debug.getinfo`）へ到達できないことを示す。
    #[test]
    fn r5_3_disabled_keeps_sandbox_debug_is_nil() {
        let runtime = disabled_runtime();

        let debug_is_nil: bool = runtime
            .exec("return debug == nil")
            .expect("eval ok")
            .as_boolean()
            .expect("boolean");
        assert!(
            debug_is_nil,
            "R5.3: disabled runtime must NOT expose the `debug` global (sandbox)"
        );

        // スタック introspection へ到達不能であること（`debug.getinfo` が呼べない）を
        // pcall で確認 — 露出していれば true（成功）になってしまう。
        let cannot_introspect: bool = runtime
            .exec("return not pcall(function() return debug.getinfo(1) end)")
            .expect("eval ok")
            .as_boolean()
            .expect("boolean");
        assert!(
            cannot_introspect,
            "R5.3: scripts must NOT be able to reach stack introspection (debug.getinfo)"
        );
    }

    /// **R5.5 — 接続口非開放**。
    ///
    /// 権威判定は `debug_local_addr() == None`。補助（best-effort）として、既定ポート
    /// (9276) への `TcpStream::connect` がこの無効ランタイム宛には成立しないことを確認する
    /// が、無関係プロセスが 9276 を占有している場合に flaky にならないよう、接続が成功した
    /// 場合はその補助チェックをスキップ（権威判定のみを信頼）する。
    #[test]
    fn r5_5_disabled_opens_no_port() {
        let runtime = disabled_runtime();

        // 権威判定: 無効ランタイムは bound addr を一切公開しない。
        assert!(
            runtime.debug_local_addr().is_none(),
            "R5.5: disabled runtime must NOT open/expose a debug port (authoritative)"
        );
        // 二重確認: handle 自体が無い。
        assert!(
            !runtime.debug_enabled(),
            "R5.5: disabled runtime holds no debug handle (no transport bound)"
        );

        // best-effort: 既定ポートへ繋がっても *この* 無効ランタイム由来ではない
        // （無効時はそもそも listen していない）。flaky 回避のため成功時は無視する。
        let connect = TcpStream::connect_timeout(
            &SocketAddr::from(([127, 0, 0, 1], DEFAULT_DEBUG_PORT)),
            Duration::from_millis(200),
        );
        match connect {
            Err(_) => { /* 期待どおり: 無効ランタイムのための listener は存在しない。*/ }
            Ok(_) => {
                // 無関係プロセスが偶発的に 9276 を占有しているケース。権威判定
                // (`debug_local_addr()==None`) は既に通っているので、ここは曖昧として
                // スキップ（test を flaky にしない）。
                eprintln!(
                    "[zero_cost_sandbox_regression] note: port {DEFAULT_DEBUG_PORT} accepted a \
                     connection from an unrelated process; relying on debug_local_addr()==None"
                );
            }
        }
    }

    /// 識別力（discrimination）の証明: 設定を **有効** に切り替えると、上記の無効時シグナルが
    /// 反転する（addr が出る・handle を保持）。これにより各無効時アサーションが「単に常に真」
    /// ではなく、disabled 状態を実際に判別していることを裏付ける（R5.2/R5.5 の鏡像）。
    #[test]
    fn enabled_runtime_flips_the_disabled_signals() {
        let debug_cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            ..Default::default()
        };
        let config = RuntimeConfig::minimal().with_debug(debug_cfg);
        let runtime = PastaLuaRuntime::with_config(TranspileContext::new(), config)
            .expect("enabled runtime must build");

        // 有効時は無効時シグナルが反転する: handle を保持し、bound addr を公開する。
        assert!(
            runtime.debug_enabled(),
            "discrimination: enabled runtime DOES hold a DebugHandle"
        );
        assert!(
            runtime.debug_local_addr().is_some(),
            "discrimination: enabled runtime DOES expose a bound debug addr (port opened)"
        );

        // teardown（接続クライアントは無いので Drop は静かに完了）。
        drop(runtime);
    }
}
