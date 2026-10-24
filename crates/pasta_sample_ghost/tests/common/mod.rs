//! ゴーストを読み込む試験の共通の手助け

use std::path::Path;

use tempfile::TempDir;

/// 環境の PASTA_DEBUG・PASTA_DEBUG_PORT を main の前に外す（固定ポートの AddrInUse を避ける）。
/// main の前（まだ 1 スレッド）なので Rust 2024 の unsafe な `remove_var` を競合なく呼べる。
#[ctor::ctor(unsafe)]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// hello-pasta の ghost/master を、profile/ と pasta.dll を除いて新しい一時フォルダへ写して返す。
/// コミット済みのゴーストをその場で読み込まないため。読み取りの誤りは expect で止める（握りつぶさない）
pub fn master_copy() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("ghosts/hello-pasta/ghost/master"),
        temp.path(),
    );
    temp
}

fn copy(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create dst dir");
    for entry in std::fs::read_dir(src).expect("read src dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        let name = entry.file_name();
        if entry.metadata().expect("metadata").is_dir() {
            if name != "profile" {
                copy(&path, &dst.join(&name));
            }
        } else if name != "pasta.dll" {
            std::fs::copy(&path, dst.join(&name)).expect("copy file");
        }
    }
}
