//! pasta_check バイナリの CLI 統合テスト
//!
//! `main()` のディスパッチ・終了コード・stdout/stderr 出力という
//! ユニットテスト（`parse_args_from`）では到達できない外部観測挙動を検証する。

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

/// ビルド済み pasta_check バイナリを引数付きで実行する
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pasta_check"))
        .args(args)
        .output()
        .expect("failed to spawn pasta_check binary")
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// `--version` はバージョン文字列を stdout へ出力し終了コード 0
#[test]
fn test_version_flag_prints_version_and_exits_zero() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    let stdout = stdout_of(&output);
    assert!(
        stdout.starts_with("pasta_check "),
        "unexpected stdout: {stdout:?}"
    );
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

/// `--help` は Usage を stderr へ出力し終了コード 0
#[test]
fn test_help_flag_prints_usage_and_exits_zero() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let stderr = stderr_of(&output);
    assert!(stderr.contains("Usage: pasta_check"));
    assert!(stderr.contains("release"));
}

/// 引数なしはエラー報告＋Usage を stderr へ出力し終了コード 1
#[test]
fn test_no_args_reports_error_and_exits_one() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_of(&output);
    assert!(stderr.contains("Error:"), "stderr: {stderr:?}");
    assert!(stderr.contains("Usage: pasta_check"));
}

/// 未知サブコマンドは終了コード 1
#[test]
fn test_unknown_subcommand_exits_one() {
    let output = run(&["frobnicate"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr_of(&output).contains("Error:"));
}

/// release 実行時の I/O エラー（存在しない target）は
/// "Error:" を stderr へ出力し終了コード 1（Usage は出さない）
#[test]
fn test_release_io_error_exits_one() {
    let temp = TempDir::new().unwrap();
    let missing_target = temp.path().join("no_such_ghost");
    let output = run(&[
        "release",
        "--target",
        missing_target.to_str().unwrap(),
        "--release",
        temp.path().join("release_out").to_str().unwrap(),
        "--nar",
        temp.path().join("out.nar").to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_of(&output);
    assert!(stderr.contains("Error:"), "stderr: {stderr:?}");
    assert!(
        !stderr.contains("Usage:"),
        "I/O error should not print usage: {stderr:?}"
    );
}

/// release E2E: バイナリ経由でリリース一式（release フォルダー・updates.txt・NAR）が
/// 生成され、進捗とサマリが stdout へ出力される
#[test]
fn test_release_end_to_end_via_binary() {
    let temp = TempDir::new().unwrap();

    let target = temp.path().join("target_ghost");
    fs::create_dir_all(target.join("ghost/master")).unwrap();
    fs::write(target.join("ghost/master/descript.txt"), "desc").unwrap();
    fs::write(
        target.join("install.txt"),
        "charset,UTF-8\r\ntype,ghost\r\n",
    )
    .unwrap();

    let overlay = temp.path().join("overlay");
    fs::create_dir_all(&overlay).unwrap();
    fs::write(overlay.join("readme.txt"), "extra").unwrap();

    let release = temp.path().join("release_out");
    let nar = temp.path().join("dist/out.nar");

    let output = run(&[
        "release",
        "--target",
        target.to_str().unwrap(),
        "--release",
        release.to_str().unwrap(),
        "--nar",
        nar.to_str().unwrap(),
        "--copy",
        overlay.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr_of(&output));
    let stdout = stdout_of(&output);
    assert!(stdout.contains("[1/5]"));
    assert!(stdout.contains("[5/5]"));
    assert!(stdout.contains("Release complete!"));

    // 成果物の実在検証
    assert!(release.join("ghost/master/descript.txt").is_file());
    assert!(release.join("readme.txt").is_file(), "overlay file applied");
    assert!(release.join("updates.txt").is_file());
    assert!(nar.is_file(), "NAR created (parent dir auto-created)");
    assert!(fs::metadata(&nar).unwrap().len() > 0);

    // 同梱バルーン無しでは警告を出さない（Req 8.3）
    assert!(
        !stderr_of(&output).contains("Warning:"),
        "stderr: {}",
        stderr_of(&output)
    );

    // target は無変更（updates.txt が混入しない）
    assert!(!Path::new(&target).join("updates.txt").exists());
}

/// release を target / release_out / out.nar の配置で実行する
fn run_release(temp: &TempDir, target: &Path) -> Output {
    run(&[
        "release",
        "--target",
        target.to_str().unwrap(),
        "--release",
        temp.path().join("release_out").to_str().unwrap(),
        "--nar",
        temp.path().join("out.nar").to_str().unwrap(),
    ])
}

/// homeurl の無い同梱バルーン: 警告を `Warning: ` 付きで stderr に出し、
/// 止めずにバルーン用 updates.txt と nar を作って終了コード 0（Req 4.9・7.3・7.4）
#[test]
fn test_release_bundled_balloon_without_homeurl_warns_and_succeeds() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("target_ghost");
    fs::create_dir_all(target.join("ghost/master")).unwrap();
    fs::create_dir_all(target.join("bal")).unwrap();
    fs::write(target.join("ghost/master/descript.txt"), "desc").unwrap();
    fs::write(
        target.join("install.txt"),
        "charset,UTF-8\r\ntype,ghost\r\nballoon.directory,bal\r\n",
    )
    .unwrap();
    fs::write(
        target.join("bal/descript.txt"),
        "charset,UTF-8\r\ntype,balloon\r\n",
    )
    .unwrap();
    fs::write(target.join("bal/balloons0.png"), "png").unwrap();

    let output = run_release(&temp, &target);

    assert!(output.status.success(), "stderr: {}", stderr_of(&output));
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("Generated bal/updates.txt (2 entries)"),
        "stdout: {stdout:?}"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("Warning: bundled balloon \"bal\""),
        "stderr: {stderr:?}"
    );
    assert!(temp.path().join("release_out/bal/updates.txt").is_file());
    assert!(temp.path().join("out.nar").is_file());
}

/// charset 宣言の無い install.txt: `Error:` と UTF-8 の旨を stderr に出し、
/// 終了コード 1 で nar を作らない（Req 1.10・8.6）
#[test]
fn test_release_install_txt_without_charset_fails_without_nar() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("target_ghost");
    fs::create_dir_all(target.join("ghost/master")).unwrap();
    fs::write(target.join("ghost/master/descript.txt"), "desc").unwrap();
    fs::write(target.join("install.txt"), "type,ghost\r\n").unwrap();

    let output = run_release(&temp, &target);

    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("Error: install.txt is not UTF-8: "),
        "stderr: {stderr:?}"
    );
    assert!(
        !temp.path().join("out.nar").exists(),
        "nar must not be created"
    );
}
