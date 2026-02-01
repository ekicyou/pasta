//! pasta_sample_ghost 統合テスト

mod common;

use pasta_sample_ghost::{generate_ghost, GhostConfig};
use tempfile::TempDir;

/// ディレクトリ構造生成テスト
#[test]
fn test_directory_structure() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    // 必須ファイル存在確認
    assert!(ghost_root.join("install.txt").exists(), "install.txt が存在しません");
    assert!(ghost_root.join("ghost/master/descript.txt").exists(), "ghost descript.txt が存在しません");
    assert!(ghost_root.join("ghost/master/pasta.toml").exists(), "pasta.toml が存在しません");
    assert!(ghost_root.join("ghost/master/dic/boot.pasta").exists(), "boot.pasta が存在しません");
    assert!(ghost_root.join("ghost/master/dic/talk.pasta").exists(), "talk.pasta が存在しません");
    assert!(ghost_root.join("ghost/master/dic/click.pasta").exists(), "click.pasta が存在しません");
    assert!(ghost_root.join("shell/master/descript.txt").exists(), "shell descript.txt が存在しません");
    assert!(ghost_root.join("shell/master/surfaces.txt").exists(), "surfaces.txt が存在しません");
}

/// シェル画像生成テスト
#[test]
fn test_shell_images() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    let shell_dir = ghost_root.join("shell/master");

    // sakura サーフェス (0-8)
    for i in 0..=8 {
        let path = shell_dir.join(format!("surface{}.png", i));
        assert!(path.exists(), "surface{}.png が存在しません", i);
    }

    // kero サーフェス (10-18)
    for i in 10..=18 {
        let path = shell_dir.join(format!("surface{}.png", i));
        assert!(path.exists(), "surface{}.png が存在しません", i);
    }
}

/// pasta.toml 内容検証テスト
#[test]
fn test_pasta_toml_content() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    let content = std::fs::read_to_string(ghost_root.join("ghost/master/pasta.toml")).unwrap();

    // 必須セクション確認
    assert!(content.contains("[package]"), "[package] セクションがありません");
    assert!(content.contains("[loader]"), "[loader] セクションがありません");
    assert!(content.contains("[ghost]"), "[ghost] セクションがありません");
    assert!(content.contains("[persistence]"), "[persistence] セクションがありません");

    // 教育的コメント確認
    assert!(content.contains("教育的サンプル"), "教育的コメントがありません");
    assert!(content.contains("省略可能"), "省略可能の説明がありません");
}

/// ukadoc 設定ファイル検証テスト
#[test]
fn test_ukadoc_files() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    // install.txt
    let install = std::fs::read_to_string(ghost_root.join("install.txt")).unwrap();
    assert!(install.contains("type,ghost"));
    assert!(install.contains("name,hello-pasta"));

    // ghost descript.txt
    let ghost_desc = std::fs::read_to_string(ghost_root.join("ghost/master/descript.txt")).unwrap();
    assert!(ghost_desc.contains("charset,UTF-8"));
    assert!(ghost_desc.contains("shiori,pasta.dll"));
    assert!(ghost_desc.contains("sakura.name,女の子"));
    assert!(ghost_desc.contains("kero.name,男の子"));

    // shell descript.txt
    let shell_desc = std::fs::read_to_string(ghost_root.join("shell/master/descript.txt")).unwrap();
    assert!(shell_desc.contains("type,shell"));
    assert!(shell_desc.contains("name,master"));
}

/// pasta DSL スクリプト検証テスト
#[test]
fn test_pasta_scripts() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    let dic_dir = ghost_root.join("ghost/master/dic");

    // boot.pasta
    let boot = std::fs::read_to_string(dic_dir.join("boot.pasta")).unwrap();
    assert!(boot.contains("＊OnBoot"), "OnBoot シーンがありません");
    assert!(boot.contains("＊OnFirstBoot"), "OnFirstBoot シーンがありません");
    assert!(boot.contains("＊OnClose"), "OnClose シーンがありません");

    // talk.pasta
    let talk = std::fs::read_to_string(dic_dir.join("talk.pasta")).unwrap();
    assert!(talk.contains("＊OnTalk"), "OnTalk シーンがありません");
    assert!(talk.contains("＊OnHour"), "OnHour シーンがありません");
    assert!(talk.contains("＄時"), "時刻変数参照がありません");

    // click.pasta
    let click = std::fs::read_to_string(dic_dir.join("click.pasta")).unwrap();
    assert!(click.contains("＊OnMouseDoubleClick"), "OnMouseDoubleClick シーンがありません");

    // ダブルクリック反応は5種以上
    let click_count = click.matches("＊OnMouseDoubleClick").count();
    assert!(click_count >= 5, "ダブルクリック反応が5種未満: {}", click_count);
}

/// ランダムトークパターン数テスト
#[test]
fn test_random_talk_patterns() {
    let talk = pasta_sample_ghost::scripts::TALK_PASTA;

    // OnTalk パターン数（5〜10種）
    let talk_count = talk.matches("＊OnTalk").count();
    assert!(talk_count >= 5, "OnTalk パターンが5種未満: {}", talk_count);
    assert!(talk_count <= 10, "OnTalk パターンが10種超過: {}", talk_count);
}

/// 時報パターンテスト
#[test]
fn test_hour_chime_patterns() {
    let talk = pasta_sample_ghost::scripts::TALK_PASTA;

    // OnHour パターン存在確認
    let hour_count = talk.matches("＊OnHour").count();
    assert!(hour_count >= 1, "OnHour パターンがありません");

    // 時刻変数参照確認
    assert!(talk.contains("＄時"), "＄時 変数参照がありません");
    assert!(talk.contains("＄時１２"), "＄時１２ 変数参照がありません");
}

/// 画像サイズ検証テスト
#[test]
fn test_image_dimensions() {
    use pasta_sample_ghost::image_generator::{generate_surface, Character, Expression};

    let img = generate_surface(Character::Sakura, Expression::Happy);
    assert_eq!(img.width(), 128, "画像幅が128pxではありません");
    assert_eq!(img.height(), 256, "画像高さが256pxではありません");
}

/// 表情バリエーションテスト
#[test]
fn test_expression_variations() {
    use pasta_sample_ghost::image_generator::Expression;

    let expressions = Expression::all();
    assert_eq!(expressions.len(), 9, "表情は9種類必要です");
}

/// DLL コピーヘルパーテスト（DLL存在時のみ成功）
#[test]
fn test_dll_copy_helper_message() {
    let temp = TempDir::new().unwrap();
    let result = common::copy_pasta_shiori_dll(temp.path());

    // DLL がない場合はエラーメッセージを確認
    if let Err(msg) = result {
        assert!(msg.contains("pasta_shiori.dll"), "エラーメッセージにDLL名が含まれていません");
        assert!(msg.contains("cargo build"), "ビルドコマンドの案内がありません");
    }
    // DLL がある場合は成功
}
