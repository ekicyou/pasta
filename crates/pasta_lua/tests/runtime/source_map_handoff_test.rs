//! Task 4.4 — ランタイムへの集約ソースマップ受け渡し（統合）。
//!
//! 仕様参照: requirements.md 3.5・3.6（有効なときだけ読み込みでシーンマップを作り、
//! ランタイムが持つ）。
//!
//! # 観測可能な「done」
//!
//! 1. `[debug] enabled = true` の base_dir を `PastaLoader::load_with_config` で読み込むと、
//!    ランタイムはトランスパイル後に作ったシーンマップ（`Arc<SourceMap>`）を持ち、有効である。
//! 2. 既定（`[debug]` 無し）では、ランタイムはマップを持たず（`None`）、無効である。

use std::path::Path;

use pasta_lua::RuntimeConfig;
use pasta_lua::loader::PastaLoader;

/// テスト用に base_dir を作り、`.pasta` と指定の pasta.toml を配置する。
/// pasta_scripts / scriptlibs はクレートルートからコピーしてランタイム初期化を成立させる。
fn make_base_dir(temp: &Path, pasta_toml: &str) {
    std::fs::create_dir_all(temp.join("dic/test")).unwrap();
    std::fs::write(
        temp.join("dic/test/hello.pasta"),
        "\
＊あいさつ
  さくら：「おはよう！」
  さくら：「げんき？」
",
    )
    .unwrap();
    std::fs::write(temp.join("pasta.toml"), pasta_toml).unwrap();

    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let src = crate_root.join(sub);
        let dst = temp.join(sub);
        if src.exists() {
            std::fs::create_dir_all(&dst).unwrap();
            copy_dir(&src, &dst).unwrap();
        }
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest)?;
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// 3.1: デバッグ有効時、トランスパイル後に構築した集約 `Arc<SourceMap>` がランタイムへ
/// 到達・保持される。
#[test]
fn enabled_debug_runtime_holds_aggregated_source_map_in_order() {
    let temp = tempfile::TempDir::new().unwrap();
    // 古い `port` のキーは読み飛ばされる（3.3）。
    make_base_dir(
        temp.path(),
        "\
[loader]
debug_mode = true

[debug]
enabled = true
port = 0
",
    );

    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("loader must build an enabled-debug runtime");

    // (1) デバッグが有効である（シーンマップを持つ）。
    assert!(
        runtime.debug_enabled(),
        "3.1: enabled [debug] must enable debug"
    );

    // (2) 集約ソースマップがランタイムに保持されている。
    let map = runtime
        .debug_source_map()
        .expect("3.1: enabled debug runtime must hold the aggregated Arc<SourceMap>");

    // (3) マップはその `.pasta` のシーンを引ける（空のマップではない）。
    let pasta_file = temp.path().join("dic/test/hello.pasta");
    assert!(
        map.scene_at(&pasta_file.to_string_lossy(), 2).is_some(),
        "3.5: held map must resolve the scene at {}:2",
        pasta_file.display()
    );
}

/// 3.1/7.1: デバッグ無効（[debug] 無し）ではマップを構築・保持しない（None）。
#[test]
fn disabled_debug_runtime_holds_no_source_map() {
    let temp = tempfile::TempDir::new().unwrap();
    make_base_dir(
        temp.path(),
        "\
[loader]
debug_mode = true
",
    );

    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("loader must build a disabled-debug runtime");

    assert!(
        !runtime.debug_enabled(),
        "7.1: no [debug] => debug disabled (zero-cost)"
    );
    assert!(
        runtime.debug_source_map().is_none(),
        "3.1/7.1: disabled debug must NOT build/hold a source map"
    );
}
