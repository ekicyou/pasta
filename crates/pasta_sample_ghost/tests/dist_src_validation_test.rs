//! ghosts/hello-pasta ディレクトリ構造検証テスト

/// ghosts/hello-pasta ディレクトリに8ファイルが存在することを確認（静的検証）
#[test]
fn test_ghost_directory_structure() {
    let ghost_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ghosts/hello-pasta");

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
        let path = ghost_dir.join(file);
        assert!(
            std::fs::metadata(&path).is_ok(),
            "ghosts/hello-pasta/{} が存在しません: {}",
            file,
            path.display()
        );
    }
}
