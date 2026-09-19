//! テストパスヘルパ自体の自己検証（要件 6.4 / 6.6 / 6.7）。
//!
//! 長パス・非 ANSI パスの一時ゴーストを、テストから構築・読み取り・破棄できることを確かめる。
//! `#[ignore]` は付けない（要件 6.6: 常時実行）。

mod common;

use common::{NON_ANSI_DIR_NAME, copy_fixture_into, make_deep_dir};

/// 非 ANSI 名のルート配下に 300 文字超の深いディレクトリを掘り、
/// フィクスチャを設置して読み出せることを確認する。
#[test]
fn deep_non_ansi_fixture_is_buildable_and_readable() {
    let temp = tempfile::TempDir::new().expect("一時ディレクトリの作成に失敗");
    let root = temp.path().join(NON_ANSI_DIR_NAME);
    std::fs::create_dir_all(&root).expect("非 ANSI ルートの作成に失敗");

    let base_dir = make_deep_dir(&root, 300);
    let shown = base_dir.display().to_string();

    assert!(
        shown.chars().count() > 300,
        "絶対パスが 300 文字を超えていない: {} 文字",
        shown.chars().count()
    );
    assert!(
        !shown.contains(r"\?\"),
        "verbatim プレフィックスが漏れている: {shown}"
    );
    assert!(
        shown.contains(NON_ANSI_DIR_NAME),
        "非 ANSI セグメントが保たれていない: {shown}"
    );
    assert!(base_dir.starts_with(&root), "掘った先がルート配下にない");

    copy_fixture_into("shiori_lifecycle", &base_dir);

    let pasta_toml = base_dir.join("pasta.toml");
    assert!(pasta_toml.is_file(), "pasta.toml が設置されていない");
    assert!(
        !std::fs::read_to_string(&pasta_toml)
            .expect("pasta.toml を読めない")
            .is_empty(),
        "pasta.toml が空"
    );
    assert!(
        base_dir.join("scripts").is_dir(),
        "scripts が設置されていない"
    );
    assert!(
        base_dir.join("scriptlibs").is_dir(),
        "scriptlibs が設置されていない"
    );
}

/// 単一 ANSI コードページで表現できない複数文字体系の混在であること（要件 6.7）。
#[test]
fn non_ansi_dir_name_mixes_multiple_writing_systems() {
    let has = |range: std::ops::RangeInclusive<char>| {
        NON_ANSI_DIR_NAME.chars().any(|c| range.contains(&c))
    };
    assert!(
        has('\u{3040}'..='\u{30ff}') || has('\u{4e00}'..='\u{9fff}'),
        "日本語が無い"
    );
    assert!(has('\u{ac00}'..='\u{d7af}'), "ハングルが無い");
    assert!(has('\u{0400}'..='\u{04ff}'), "キリル文字が無い");
    assert!(has('\u{0370}'..='\u{03ff}'), "ギリシャ文字が無い");
}

/// 一時ディレクトリの drop で深いパスごと後始末されること（M5）。
#[test]
fn deep_dir_is_cleaned_up_on_drop() {
    let temp = tempfile::TempDir::new().expect("一時ディレクトリの作成に失敗");
    let root = temp.path().join(NON_ANSI_DIR_NAME);
    std::fs::create_dir_all(&root).expect("非 ANSI ルートの作成に失敗");
    let base_dir = make_deep_dir(&root, 300);
    copy_fixture_into("shiori_lifecycle", &base_dir);

    let temp_root = temp.path().to_path_buf();
    drop(temp);
    assert!(!temp_root.exists(), "一時ディレクトリが残っている");
}
