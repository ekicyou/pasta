//! validate サブコマンドの CLI 統合テスト
//!
//! ビルド済みのバイナリを起動し、終了コード・出力・読むだけであること（Req 1.6）・
//! 見本のゴースト hello-pasta が誤り 0 で通ること（Req 9.5）を確かめる。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::SystemTime;
use tempfile::TempDir;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pasta_check"))
        .args(args)
        .output()
        .expect("failed to spawn pasta_check binary")
}

fn validate(target: &Path) -> Output {
    run(&["validate", "--target", target.to_str().unwrap()])
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// target/ghost/master/dic を作り、(TempDir, target, master) を返す
fn ghost() -> (TempDir, PathBuf, PathBuf) {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("ghost_dev");
    let master = target.join("ghost").join("master");
    fs::create_dir_all(master.join("dic")).unwrap();
    fs::write(master.join("pasta.toml"), "").unwrap();
    (temp, target, master)
}

/// フォルダの下の全エントリの (パス, 更新時刻) をパス順で集める
fn snapshot(root: &Path) -> Vec<(PathBuf, SystemTime)> {
    let mut v = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(dir).unwrap() {
            let e = e.unwrap();
            let p = e.path();
            if e.file_type().unwrap().is_dir() {
                stack.push(p.clone());
            }
            // DirEntry::metadata は Windows では親の一覧の写し（NTFS が遅れて更新する）を
            // 返し、フォルダの時刻が書き込みなしでもずれるので、パスから引く
            let modified = fs::metadata(&p).unwrap().modified().unwrap();
            v.push((p, modified));
        }
    }
    v.sort();
    v
}

/// 誤りあり → 終了コード 1、問題の行のパスは ghost/master/ から始まる（Req 7.2・7.5）
#[test]
fn test_validate_errors_exit_one() {
    let (_t, target, master) = ghost();
    fs::write(master.join("dic/a.pasta"), "＊\n  ＠＠＠\n").unwrap();
    let output = validate(&target);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = stdout_of(&output);
    let problem = stdout.lines().next().unwrap();
    assert!(problem.starts_with("ghost/master/dic/a.pasta:"), "{stdout}");
    assert!(problem.contains(": 誤り: "), "{stdout}");
    assert!(stdout.contains("誤り 1 件"), "{stdout}");
}

/// 構文エラーも 1 件 1 行。標準出力のどの行も `ghost/master/…:行:列: 誤り|注意: ` か、案内・まとめの行（Req 7.1）
#[test]
fn test_validate_each_finding_is_one_line() {
    let (_t, target, master) = ghost();
    fs::write(master.join("dic/a.pasta"), "＊\n  ＠＠＠\n").unwrap();
    fs::write(master.join("dic/b.pasta"), "＊会話\n  ＄＄＄\n").unwrap();
    let output = validate(&target);
    let stdout = stdout_of(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 4, "{stdout}");
    for line in &lines[..2] {
        // `ghost/master/{path}:{行}:{列}: {誤り|注意}: {理由}` の形を前から順に確かめる
        let rest = line.strip_prefix("ghost/master/dic/").expect(line);
        let (_, rest) = rest.split_once(".pasta:").expect(line);
        let mut parts = rest.splitn(3, ':');
        let (l, c) = (parts.next().unwrap(), parts.next().unwrap());
        assert!(
            l.chars().all(|ch| ch.is_ascii_digit()) && !l.is_empty(),
            "{line}"
        );
        assert!(
            c.chars().all(|ch| ch.is_ascii_digit()) && !c.is_empty(),
            "{line}"
        );
        let rest = parts.next().unwrap();
        assert!(
            rest.starts_with(" 誤り: 構文エラー: ") && !rest.contains("-->"),
            "{line}"
        );
    }
    assert_eq!(
        lines[2],
        "構文エラーがあるため、飛び先の検査をしていません。"
    );
    assert!(
        lines[3].starts_with("検査した辞書ファイル 2 個: "),
        "{stdout}"
    );
}

/// 読めない辞書があると飛び先の検査を省き、理由を案内の行に出す（1.1・1.4・1.5）
#[test]
fn test_validate_unreadable_dictionary_skips_references() {
    let (_t, target, master) = ghost();
    fs::write(master.join("dic/a.pasta"), "＊挨拶\n  ＞どこにも無い\n").unwrap();
    fs::write(master.join("dic/b.pasta"), [0xffu8, 0xfe, 0x00]).unwrap();
    let output = validate(&target);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = stdout_of(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.contains(&"読めない辞書ファイルがあるため、飛び先の検査をしていません。"),
        "{stdout}"
    );
    assert!(!stdout.contains("飛び先「"), "{stdout}");
    assert!(!stdout.contains("飛び先を検査しなかった"), "{stdout}");
}

/// 誤りなし（注意あり）→ 終了コード 0（Req 7.6）
#[test]
fn test_validate_warnings_only_exit_zero() {
    let (_t, target, master) = ghost();
    fs::write(master.join("dic/a.pasta"), "＊挨拶\n  ＞どこにも無い\n").unwrap();
    let output = validate(&target);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = stdout_of(&output);
    assert!(stdout.contains("ghost/master/dic/a.pasta:"), "{stdout}");
    assert!(stdout.contains(": 注意: "), "{stdout}");
    assert!(stdout.contains("誤り 0 件、注意 1 件"), "{stdout}");
}

/// フォルダが無い・ghost/master を直接渡す → 終了コード 1、理由を stderr に出し検査しない（Req 1.3）
#[test]
fn test_validate_bad_folder_exit_one() {
    let (_t, target, master) = ghost();
    for bad in [target.join("nope"), master.clone()] {
        let output = validate(&bad);
        assert_eq!(output.status.code(), Some(1), "{bad:?}");
        assert!(stderr_of(&output).contains("Error:"), "{bad:?}");
        assert!(stdout_of(&output).is_empty(), "{bad:?}");
    }
}

/// 知らないオプション → 終了コード 1、理由と使い方（Req 1.4）
#[test]
fn test_validate_unknown_option_exit_one() {
    let (_t, target, _master) = ghost();
    let output = run(&["validate", "--target", target.to_str().unwrap(), "--bogus"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("Error:") && stderr.contains("--bogus"),
        "{stderr}"
    );
    assert!(stderr.contains("Usage: pasta_check"), "{stderr}");
}

/// 出力は 2 回とも同じ。実行の前後でフォルダの一覧と更新時刻が変わらない（Req 1.6・7.3）
#[test]
fn test_validate_stable_and_read_only() {
    let (t, target, master) = ghost();
    fs::create_dir_all(master.join("dic/sub")).unwrap();
    fs::write(master.join("dic/b.pasta"), "＊\n  ＠＠＠\n").unwrap();
    fs::write(master.join("dic/sub/a.pasta"), "＊挨拶\n  ＞どこにも無い\n").unwrap();
    fs::write(master.join("dic/c.pasta"), [0xff, 0xfe, 0x81]).unwrap();
    let before = snapshot(t.path());
    let first = validate(&target);
    let second = validate(&target);
    assert_eq!(snapshot(t.path()), before);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);
    assert!(
        stdout_of(&first).lines().count() >= 4,
        "{}",
        stdout_of(&first)
    );
}

/// 中身の無い `dic/x.pasta/` → 終了コード 1、フォルダの 1 行（行・列なし）を出す（2.1・4.1）
#[test]
fn test_validate_folder_matching_pattern_exit_one() {
    let (_t, target, master) = ghost();
    fs::write(master.join("dic/a.pasta"), "＊挨拶\n  さくら：こんにちは\n").unwrap();
    // Git は空のフォルダを持てないので、走らせるときに作る
    fs::create_dir(master.join("dic/x.pasta")).unwrap();
    let output = validate(&target);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = stdout_of(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines[0],
        "ghost/master/dic/x.pasta: 誤り: 辞書のパターンに一致するフォルダです。pasta.dll はこのフォルダを辞書として読めず、ゴーストを読み込めません",
        "{stdout}"
    );
    // フォルダは検査した辞書ファイルの数に入らない（2.5）
    assert!(
        stdout.contains("検査した辞書ファイル 1 個: 誤り 1 件、注意 0 件"),
        "{stdout}"
    );
}

/// 見本のゴースト hello-pasta は誤り 0 件・注意 0 件で終了コード 0、読むだけ（Req 9.5・1.6・5.3）。
/// 11 段目の前方一致の候補（`11-jump.pasta` の `＞寄り道` → `＊寄り道公園`・`＊寄り道本屋`・`＊寄り道駄菓子屋`）が注意にならないこと＝
/// 前方一致の重なりを報告しないこと（5.4）も、これで守る
#[test]
fn test_validate_hello_pasta_clean() {
    let target =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../pasta_sample_ghost/ghosts/hello-pasta");
    let before = snapshot(&target);
    let output = validate(&target);
    assert_eq!(snapshot(&target), before);
    let stdout = stdout_of(&output);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{stdout}{}",
        stderr_of(&output)
    );
    assert!(stdout.contains("誤り 0 件、注意 0 件"), "{stdout}");
    // 問題の行は 1 行も無い
    assert!(!stdout.contains("ghost/master/"), "{stdout}");
}
