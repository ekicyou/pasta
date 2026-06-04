//! Build script for `pasta_lua`.
//!
//! pasta-scripts-self-deploy (Req 4.1–4.6): ソースの `pasta_scripts/` ツリー全体を
//! 決定論的（byte-deterministic）な zip アーカイブへ固め、その MD5 を基準ダイジェスト
//! として公開する。runtime は `include_bytes!(concat!(env!("OUT_DIR"), "/pasta_scripts.zip"))`
//! と `env!("PASTA_SCRIPTS_MD5")` で参照する。
//!
//! 決定論を担保するパッキングロジックは、ビルド決定論テスト（tests/build_determinism_test.rs）
//! と共有するため `build_zip.rs`（純粋・cargo ディレクティブ非依存）へ切り出してある。
//! build.rs はそれを呼び出し、OUT_DIR への書き出し・MD5 算出・rerun-if-changed 発行を担う。
//! 同一ソースからは常にバイト同一の zip を生成し（4.3/4.4）、内容変化時のみ MD5 が変わる（4.5）。
//! 埋め込み正本はビルドのたびにソースから再生成され、手動同期は不要（4.6）。

use std::collections::BTreeMap;
use std::path::PathBuf;

// 決定論的 zip パッカー（テストと共有する純粋ロジック）を include する。
// build.rs はこれを呼び出すだけで、アルゴリズムの実体は build_zip.rs に一元化する。
#[path = "build_zip.rs"]
mod build_zip;

fn main() {
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set by cargo"),
    );
    let scripts_root = manifest_dir.join("pasta_scripts");
    assert!(
        scripts_root.is_dir(),
        "pasta_scripts source tree not found at {}",
        scripts_root.display()
    );

    // rerun-if-changed 用にツリーを walk する（共有 collect を再利用）。
    // entry_name（forward-slash 相対パス） -> 絶対パス
    let mut files: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    build_zip::collect(&scripts_root, &scripts_root, &mut files, &mut dirs);

    // rerun-if-changed: 各サブディレクトリ・各ファイルごとに発行（4.4/4.5）。
    // ディレクトリ1個のみへの発行では nested 変更を検知できないため、全要素へ発行する。
    println!("cargo:rerun-if-changed={}", scripts_root.display());
    for dir in &dirs {
        println!("cargo:rerun-if-changed={}", dir.display());
    }
    for abs in files.values() {
        println!("cargo:rerun-if-changed={}", abs.display());
    }

    // 決定論的 zip を生成（共有パッカーへ委譲）。
    let zip_bytes = build_zip::build_deterministic_zip(&scripts_root);

    // OUT_DIR へ書き出し。
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set by cargo"));
    let zip_path = out_dir.join("pasta_scripts.zip");
    std::fs::write(&zip_path, &zip_bytes)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", zip_path.display()));

    // 最終 zip バイト列の MD5 を算出し、基準ダイジェストとして公開（4.2）。
    let digest = format!("{:x}", md5::compute(&zip_bytes));
    println!("cargo:rustc-env=PASTA_SCRIPTS_MD5={digest}");
}
