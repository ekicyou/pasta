//! reload リーク検査の統合テスト（task 4.1・R7.2/R7.3）。
//!
//! reload（spawn→teardown ×N）でカーネルハンドル／USER オブジェクトがリーク・枯渇
//! しないことを、実 OS カウンタ（`GetProcessHandleCount`/`GetGuiResources`）で検査する。
//!
//! # なぜ専用のテストバイナリに置くか（flake の根本原因）
//! 計測するカウンタは **プロセス全体**の値である。libtest は同一バイナリ内のテストを
//! 並列スレッドで走らせるため、以前 `actor_teardown_test.rs` に同居していた頃は、
//! 兄弟テスト（各々アクタースレッド＋VM を spawn→teardown する）のハンドル解放が
//! 計測窓の途中に重なり、baseline だけが兄弟分（~14）高く採られて growth が負に振れ、
//! slope 判定が偽陽性になった（遅い CI ランナーでのみ顕在化）。
//! cargo はテストバイナリを直列に実行するので、本ファイルにこのテスト 1 本だけを置けば
//! 計測中に同一プロセスで他のテストが動くことはない。**このファイルにテストを追加しないこと。**

use std::path::{Path, PathBuf};
use std::time::Duration;

use pasta::actor::teardown::ReloadProbe;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元: `common/mod.rs`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// async_callback フィクスチャを temp へ展開し `load_dir` を返す。`TempDir` は
/// 返り値で寿命保持する（アクタースレッドより長命）。写経元: `actor_teardown_test.rs`。
fn build_async_callback_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let pasta_scripts_src = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_lua")
        .join("pasta_scripts");
    let pasta_scripts_dst = temp.path().join("pasta_scripts");
    std::fs::create_dir_all(&pasta_scripts_dst).expect("create pasta_scripts dir");
    copy_dir_recursive(&pasta_scripts_src, &pasta_scripts_dst).expect("copy pasta_scripts");

    let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
    copy_dir_recursive(&fixture_src, temp.path()).expect("copy fixture");

    (temp.path().to_path_buf(), temp)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// R7.2/R7.3: reload（spawn→teardown ×N）でカーネルハンドル／USER オブジェクトが
/// リーク・枯渇しない。done ack 後に計測する（PoC の実 OS 計測アプローチを流用）。
///
/// # 判定（slope 法）
/// 真の per-cycle リークは増分が N に **線形比例**して増える（growth ≈ L·N）。一方
/// プロセス内の残留ノイズ（ランタイムの一回性確保等）は N に比例しない。そこで小さな N と
/// 3×N の 2 水準で計測し、**増分の傾き（per-cycle 増分）が大きい N で増えていない**ことを
/// assert する:
///   - L=0（リークなし）: growth(N)≈growth(3N)。差は N に依存せず小さい。
///   - L≥1（per-cycle リーク）: growth(3N)-growth(N) ≈ L·2N。N=6 なら ≥12 となり、
///     小さなノイズ許容を確実に超える → 検出される。
#[test]
fn repeated_reload_tears_down_and_does_not_leak() {
    let (load_dir, _temp) = build_async_callback_dir();

    // 小さい水準 N と大きい水準 3×N。per-cycle リーク 1 個でも slope ≈ L·2N = 12（N=6）。
    const N_SMALL: usize = 6;
    const N_LARGE: usize = N_SMALL * 3; // 18

    // 各 run_cycles は内部でウォームアップ→baseline→N サイクル→final を完結させるため、
    // 連続呼び出しは互いに独立・公平（それぞれ自前の baseline を採る）。
    let small = ReloadProbe::run_cycles(&load_dir, N_SMALL, Duration::from_secs(10));
    let large = ReloadProbe::run_cycles(&load_dir, N_LARGE, Duration::from_secs(10));

    assert_eq!(
        small.cycles_run, N_SMALL,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        large.cycles_run, N_LARGE,
        "all reload cycles must complete (no hang/panic mid-loop)"
    );
    assert_eq!(
        small.clean_teardowns, N_SMALL,
        "every reload cycle must tear down cleanly (Stop{{done}} ack received)"
    );
    assert_eq!(
        large.clean_teardowns, N_LARGE,
        "every reload cycle must tear down cleanly (Stop{{done}} ack received)"
    );

    #[cfg(windows)]
    {
        let small_leak = small
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");
        let large_leak = large
            .leak_metric
            .expect("on Windows a real resource-leak metric must be sampled");

        // slope（傾き）法の許容: per-cycle リークが 1 でもあれば growth(3N)-growth(N)
        // ≈ L·2N = 12（N=6）となるので、これを確実に下回る許容を置く。
        const SLOPE_TOLERANCE: i64 = 8;

        let handle_slope = large_leak.kernel_handle_growth - small_leak.kernel_handle_growth;
        let user_slope = large_leak.user_object_growth - small_leak.user_object_growth;

        assert!(
            handle_slope <= SLOPE_TOLERANCE,
            "kernel handle growth scales with cycle count: growth({}) = {} but growth({}) = {} \
             (delta = {} > tolerance {}); a per-cycle handle leak of L would make this delta \
             ~L*2N = ~{}. small(baseline={}, final={}), large(baseline={}, final={})",
            N_SMALL,
            small_leak.kernel_handle_growth,
            N_LARGE,
            large_leak.kernel_handle_growth,
            handle_slope,
            SLOPE_TOLERANCE,
            2 * N_SMALL,
            small_leak.kernel_handles_baseline,
            small_leak.kernel_handles_final,
            large_leak.kernel_handles_baseline,
            large_leak.kernel_handles_final,
        );

        assert!(
            user_slope <= SLOPE_TOLERANCE,
            "USER object growth scales with cycle count: growth({}) = {} but growth({}) = {} \
             (delta = {} > tolerance {}); a leaked message-only window per cycle would make this \
             delta ~2N = ~{}. small(baseline={}, final={}), large(baseline={}, final={})",
            N_SMALL,
            small_leak.user_object_growth,
            N_LARGE,
            large_leak.user_object_growth,
            user_slope,
            SLOPE_TOLERANCE,
            2 * N_SMALL,
            small_leak.user_objects_baseline,
            small_leak.user_objects_final,
            large_leak.user_objects_baseline,
            large_leak.user_objects_final,
        );
    }

    #[cfg(not(windows))]
    {
        assert!(
            small.leak_metric.is_none() && large.leak_metric.is_none(),
            "non-windows builds do not sample a handle metric"
        );
    }
}
