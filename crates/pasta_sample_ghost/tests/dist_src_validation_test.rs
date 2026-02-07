//! dist-src ディレクトリ構造検証テスト

/// dist-src ディレクトリに8ファイルが存在することを確認（静的検証）
#[test]
fn test_dist_src_directory_structure() {
    let dist_src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist-src");

    let required_files = [
        "install.txt",
        "ghost/master/descript.txt",
        "ghost/master/pasta.toml",
        "ghost/master/dic/actors.pasta",
        "ghost/master/dic/boot.pasta",
        "ghost/master/dic/talk.pasta",
        "ghost/master/dic/click.pasta",
        "shell/master/descript.txt",
    ];

    for file in &required_files {
        let path = dist_src.join(file);
        assert!(
            std::fs::metadata(&path).is_ok(),
            "dist-src/{} が存在しません: {}",
            file,
            path.display()
        );
    }
}
