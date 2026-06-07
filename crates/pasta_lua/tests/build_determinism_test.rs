//! ビルド決定論テスト（pasta-scripts-self-deploy Task 5.2 / Req 4.3, 4.4, 4.5）。
//!
//! build.rs と同一のパッキングロジック（`build_zip.rs`）を直接 include して再実行し、
//! 「再現実装」ではなく **実際に dll へ埋め込まれるコード** の決定論を検証する。
//!
//! 検証内容:
//! - 決定論（4.3/4.4）: 同一ソースから2回生成した zip がバイト同一・MD5 同一。
//! - 埋め込みとの整合: 生成 zip の MD5 が build.rs の `PASTA_SCRIPTS_MD5` と一致
//!   （= リファクタが出力バイトを1バイトも変えていない証拠）。
//! - 変化反映（4.5）: ソースツリーのコピーで MD5 が原本と一致し、1ファイル変更で MD5 が変化。

// build.rs と共有する純粋パッカーを include（cargo:/OUT_DIR 非依存）。
#[path = "../build_zip.rs"]
mod build_zip;

use std::path::{Path, PathBuf};

/// ソースの `pasta_scripts/` ツリーへの絶対パス。
fn scripts_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("pasta_scripts")
}

/// `src` ツリーを `dst` へ再帰コピーする（ファイル・ディレクトリのみ、シンボリックリンク非対応）。
fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create dst dir");
    for entry in std::fs::read_dir(src).expect("read src dir") {
        let entry = entry.expect("dir entry");
        let ty = entry.file_type().expect("file type");
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_tree(&from, &to);
        } else if ty.is_file() {
            std::fs::copy(&from, &to).expect("copy file");
        }
    }
}

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

/// 4.3/4.4: 同一ソースから2回生成した zip がバイト同一・MD5 同一であること。
#[test]
fn zip_is_byte_and_md5_deterministic() {
    let root = scripts_root();
    assert!(root.is_dir(), "pasta_scripts source not found: {}", root.display());

    let first = build_zip::build_deterministic_zip(&root);
    let second = build_zip::build_deterministic_zip(&root);

    assert_eq!(first, second, "two builds from same source produced different bytes");
    assert_eq!(
        md5_hex(&first),
        md5_hex(&second),
        "two builds from same source produced different MD5"
    );
    assert!(!first.is_empty(), "generated zip is unexpectedly empty");
}

/// リファクタ後も build.rs の埋め込み出力（PASTA_SCRIPTS_MD5）とバイト一致すること。
/// build_zip.rs を build.rs と共有するため、これが破れたら抽出がアルゴリズムを変えた証拠。
#[test]
fn zip_md5_matches_embedded_expected() {
    let zip = build_zip::build_deterministic_zip(&scripts_root());
    let actual = md5_hex(&zip);

    // build.rs が `cargo:rustc-env=PASTA_SCRIPTS_MD5` で公開する基準ダイジェスト。
    // テストターゲットでも build.rs は実行されるため env! で参照できる。
    let expected = env!("PASTA_SCRIPTS_MD5");

    assert_eq!(
        actual, expected,
        "generated zip MD5 diverged from embedded PASTA_SCRIPTS_MD5 \
         (refactor must be byte-preserving)"
    );
}

/// 4.5: ソースのコピーでは MD5 が原本と一致し、1ファイル変更で MD5 が変化すること。
/// 実ソースツリーは一切変更しない（tempdir 上のコピーのみ操作する）。
#[test]
fn md5_changes_when_a_single_file_changes() {
    let root = scripts_root();
    let original_md5 = md5_hex(&build_zip::build_deterministic_zip(&root));

    let tmp = tempfile::tempdir().expect("create tempdir");
    let copy_root = tmp.path().join("pasta_scripts");
    copy_tree(&root, &copy_root);

    // コピー直後は原本と完全一致するはず。
    let copy_md5 = md5_hex(&build_zip::build_deterministic_zip(&copy_root));
    assert_eq!(
        copy_md5, original_md5,
        "verbatim copy of source produced a different MD5"
    );

    // コピー内の1ファイルへ1バイト追記して変化を起こす。
    let mut target: Option<PathBuf> = None;
    find_first_file(&copy_root, &mut target);
    let target = target.expect("no file found in copied tree to mutate");

    let mut content = std::fs::read(&target).expect("read target");
    content.push(b'\n');
    std::fs::write(&target, &content).expect("write mutated target");

    let mutated_md5 = md5_hex(&build_zip::build_deterministic_zip(&copy_root));
    assert_ne!(
        mutated_md5, original_md5,
        "MD5 did not change after mutating a single file ({})",
        target.display()
    );
}

/// 生成 zip 内の全エントリ名（forward-slash 相対パス）を列挙する。
fn zip_entry_names(zip_bytes: &[u8]) -> Vec<String> {
    let reader = std::io::Cursor::new(zip_bytes.to_vec());
    let mut archive = zip::ZipArchive::new(reader).expect("open generated zip");
    let mut names = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let entry = archive.by_index(i).expect("read zip entry");
        names.push(entry.name().to_string());
    }
    names
}

/// R8.1: 旧 luasocket デバッグ資産（撤去対象の4ファイル）が埋め込み zip に同梱されないこと。
/// build.rs と同一のパッカーで生成した zip のエントリ一覧を直接検査する
/// （= 実際に dll へ埋め込まれるバイト列の内容を検証する）。
#[test]
fn legacy_luasocket_debug_assets_absent_from_zip() {
    let zip = build_zip::build_deterministic_zip(&scripts_root());
    let names = zip_entry_names(&zip);

    // requirements 8.1 / design "Removed Files (R8)" が名指しする厳密な4資産。
    let removed = [
        "vscode-debuggee.lua",
        "socket/core.dll",
        "mime/core.dll",
        "dkjson.lua",
    ];

    for asset in removed {
        assert!(
            !names.contains(&asset.to_string()),
            "removed legacy luasocket debug asset still present in embedded zip: {asset}\n\
             zip entries: {names:?}"
        );
    }
}

/// ツリー内で最初に見つかったファイルを（決定的な順序で）返す。
fn find_first_file(dir: &Path, out: &mut Option<PathBuf>) {
    if out.is_some() {
        return;
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("read dir")
        .map(|e| e.expect("entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if out.is_some() {
            return;
        }
        let ty = std::fs::symlink_metadata(&path).expect("stat").file_type();
        if ty.is_dir() {
            find_first_file(&path, out);
        } else if ty.is_file() {
            *out = Some(path);
            return;
        }
    }
}
