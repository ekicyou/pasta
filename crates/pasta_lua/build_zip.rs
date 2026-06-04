//! 決定論的 zip パッカー（build.rs とビルド決定論テストで共有する純粋ロジック）。
//!
//! このファイルは `#[path = ...]` 経由で build.rs / tests から include される。
//! cargo ディレクティブ（`cargo:` 出力）や `OUT_DIR` への依存は **一切持たない**。
//! ここにあるのは、ソースツリーのパスからバイト決定論的な zip blob を生成する純関数のみ。
//!
//! 決定論の担保（pasta-scripts-self-deploy Req 4.3）:
//! - エントリ名（相対パス・常に `/` 区切り）でソート（BTreeMap）
//! - `last_modified_time` を固定値（DateTime::default）に
//! - Unix 権限を固定（0o644）に
//! - `CompressionMethod::Deflated` ＋ 固定圧縮レベル

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, DateTime};

/// 固定圧縮レベル（決定論のため定数化）。
const FIXED_COMPRESSION_LEVEL: i64 = 6;

/// `root` ツリー全体から決定論的（byte-deterministic）な zip blob を生成する。
///
/// 同一内容のソースからは常にバイト同一の blob を返す（Req 4.3/4.4）。
/// 内容が1ファイルでも変化すれば blob が変化し、MD5 も変化する（Req 4.5）。
pub fn build_deterministic_zip(root: &Path) -> Vec<u8> {
    let mut files: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    collect(root, root, &mut files, &mut dirs);
    pack(&files)
}

/// `root` 直下を再帰的に walk し、ファイルとサブディレクトリを収集する。
/// `files` のキーは `root` からの相対パス（forward-slash、OS 非依存で安定）。
pub fn collect(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, PathBuf>,
    dirs: &mut Vec<PathBuf>,
) {
    let read = std::fs::read_dir(current)
        .unwrap_or_else(|e| panic!("failed to read dir {}: {e}", current.display()));

    // 安定した走査順のためにエントリをソート（rerun/walk の決定性に寄与）。
    let mut entries: Vec<PathBuf> = read
        .map(|e| {
            e.unwrap_or_else(|err| panic!("failed to read dir entry in {}: {err}", current.display()))
                .path()
        })
        .collect();
    entries.sort();

    for path in entries {
        let file_type = std::fs::symlink_metadata(&path)
            .unwrap_or_else(|e| panic!("failed to stat {}: {e}", path.display()))
            .file_type();

        // シンボリックリンクは決定性・移植性のためスキップ。
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            dirs.push(path.clone());
            collect(root, &path, files, dirs);
        } else if file_type.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or_else(|e| panic!("strip_prefix failed for {}: {e}", path.display()))
                .to_string_lossy()
                // Windows の `\` を `/` に正規化し、OS 間でエントリ名を安定化（4.3）。
                .replace('\\', "/");
            files.insert(rel, path);
        }
    }
}

/// 収集済みファイル集合から決定論的な zip blob を生成する。
fn pack(files: &BTreeMap<String, PathBuf>) -> Vec<u8> {
    let buf = std::io::Cursor::new(Vec::<u8>::new());
    let mut zw = ZipWriter::new(buf);

    // 全エントリ共通の固定オプション（mtime・権限・圧縮方式/レベルを固定）。
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(FIXED_COMPRESSION_LEVEL))
        // ビルド時刻を埋め込まないよう固定値に（決定論の要）。
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644);

    // BTreeMap のイテレーションはキー（entry_name）昇順 = ソート済み（4.3）。
    for (entry_name, abs_path) in files {
        let bytes = std::fs::read(abs_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", abs_path.display()));
        zw.start_file(entry_name.as_str(), options)
            .unwrap_or_else(|e| panic!("failed to start zip entry {entry_name}: {e}"));
        zw.write_all(&bytes)
            .unwrap_or_else(|e| panic!("failed to write zip entry {entry_name}: {e}"));
    }

    let cursor = zw
        .finish()
        .unwrap_or_else(|e| panic!("failed to finish zip: {e}"));
    cursor.into_inner()
}
