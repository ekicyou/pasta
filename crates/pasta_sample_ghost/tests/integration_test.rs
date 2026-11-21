//! pasta_sample_ghost 統合テスト

use std::path::PathBuf;

/// ghosts/hello-pasta ディレクトリのパスを取得
fn ghost_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ghosts/hello-pasta")
}

/// pasta.toml 内容検証テスト（ghosts/hello-pasta 直接読み込み）
///
/// `[package]` はエンジンプロファイル専用で、サンプルには置かない。
#[test]
fn test_pasta_toml_content() {
    let content = std::fs::read_to_string(ghost_dir().join("ghost/master/pasta.toml")).unwrap();

    assert!(
        !content.contains("[package]"),
        "[package] セクションが残っています（R4.4: サンプルから除去されているべき）"
    );
    assert!(
        !content.contains(r#"name = "hello-pasta""#),
        "package name が残っています（R4.4: 除去されているべき）"
    );
    assert!(
        !content.contains(r#"version = "1.0.0""#),
        "package version が残っています（R4.4: 除去されているべき）"
    );
    assert!(
        !content.contains(r#"edition = "2024""#),
        "package edition が残っています（R4.4: 除去されているべき）"
    );

    assert!(
        content.contains("[loader]"),
        "[loader] セクションがありません"
    );
    assert!(
        content.contains("[ghost]"),
        "[ghost] セクションがありません"
    );
    assert!(
        content.contains("[persistence]"),
        "[persistence] セクションがありません"
    );

    assert!(
        content.contains(r#"pasta_patterns = ["dic/*.pasta"]"#),
        "pasta_patterns がありません"
    );
    assert!(
        content.contains("lua_search_paths"),
        "lua_search_paths がありません"
    );
    assert!(
        content.contains(r#""profile/pasta/save/lua""#),
        "lua_search_paths の順序1がありません"
    );
    assert!(
        content.contains(r#""profile/pasta/pasta_scripts""#),
        "lua_search_paths に自己展開先 profile/pasta/pasta_scripts がありません"
    );
    assert!(
        !content.contains(r#""pasta_scripts","#) && !content.contains(r#""pasta_scripts"]"#),
        "lua_search_paths に旧 pasta_scripts エントリが残っています"
    );
    assert!(
        content.contains(r#""scripts""#),
        "lua_search_paths に scripts がありません"
    );
    assert!(
        content.contains(r#""profile/pasta/cache/lua""#),
        "lua_search_paths に cache がありません"
    );
    assert!(
        content.contains(r#""scriptlibs""#),
        "lua_search_paths に scriptlibs がありません"
    );
    assert!(
        content.contains(r#"transpiled_output_dir = "profile/pasta/cache/lua""#),
        "transpiled_output_dir がありません"
    );

    assert!(
        content.contains("talk_interval_min = 45"),
        "talk_interval_min = 45 がありません"
    );
    assert!(
        content.contains("talk_interval_max = 75"),
        "talk_interval_max = 75 がありません"
    );

    assert!(
        content.contains("教育的サンプル"),
        "教育的コメントがありません"
    );
    assert!(content.contains("省略可能"), "省略可能の説明がありません");
}

/// ukadoc 設定ファイル検証テスト（ghosts/hello-pasta 直接読み込み）
#[test]
fn test_ukadoc_files() {
    let ghost_dir = ghost_dir();

    let install = std::fs::read_to_string(ghost_dir.join("install.txt")).unwrap();
    // pasta_check release は install.txt の charset,UTF-8 宣言を必須とする
    assert_eq!(
        install.lines().next(),
        Some("charset,UTF-8"),
        "install.txt の 1 行目が charset,UTF-8 ではありません"
    );
    assert!(
        install.contains("type,ghost"),
        "install.txt に type,ghost がありません"
    );
    assert!(
        install.contains("name,hello-pasta"),
        "install.txt に name がありません"
    );
    assert!(
        install.contains("directory,hello-pasta"),
        "install.txt に directory がありません"
    );

    let ghost_desc = std::fs::read_to_string(ghost_dir.join("ghost/master/descript.txt")).unwrap();
    assert!(
        ghost_desc.contains("charset,UTF-8"),
        "ghost descript.txt に charset がありません"
    );
    assert!(
        ghost_desc.contains("type,ghost"),
        "ghost descript.txt に type,ghost がありません"
    );
    assert!(
        ghost_desc.contains("shiori,pasta.dll"),
        "ghost descript.txt に shiori がありません"
    );
    assert!(
        ghost_desc.contains("sakura.name,女の子"),
        "ghost descript.txt に sakura.name がありません"
    );
    assert!(
        ghost_desc.contains("kero.name,男の子"),
        "ghost descript.txt に kero.name がありません"
    );
    assert!(
        ghost_desc.contains("craftman,ekicyou"),
        "ghost descript.txt に craftman がありません"
    );
    assert!(
        ghost_desc.contains("craftmanw,えちょ"),
        "ghost descript.txt に craftmanw がありません"
    );
    assert!(
        ghost_desc.contains("homeurl,https://ekicyou.github.io/pasta/hello-pasta/"),
        "ghost descript.txt に homeurl がありません"
    );

    let shell_desc = std::fs::read_to_string(ghost_dir.join("shell/master/descript.txt")).unwrap();
    assert!(
        shell_desc.contains("charset,UTF-8"),
        "shell descript.txt に charset がありません"
    );
    assert!(
        shell_desc.contains("type,shell"),
        "shell descript.txt に type,shell がありません"
    );
    assert!(
        shell_desc.contains("name,master"),
        "shell descript.txt に name がありません"
    );
    assert!(
        shell_desc.contains("seriko.use_self_alpha,1"),
        "shell descript.txt に seriko.use_self_alpha がありません"
    );
    assert!(
        shell_desc.contains("sakura.balloon.offsetx,100"),
        "shell descript.txt の sakura.balloon.offsetx が100ではありません"
    );
    assert!(
        shell_desc.contains("sakura.balloon.offsety,-105"),
        "shell descript.txt の sakura.balloon.offsety が-105ではありません"
    );
    assert!(
        shell_desc.contains("kero.balloon.offsetx,50"),
        "shell descript.txt の kero.balloon.offsetx が50ではありません"
    );
    assert!(
        shell_desc.contains("kero.balloon.offsety,120"),
        "shell descript.txt の kero.balloon.offsety が120ではありません"
    );
}
