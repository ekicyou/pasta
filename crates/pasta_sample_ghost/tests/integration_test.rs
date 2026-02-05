//! pasta_sample_ghost 統合テスト

mod common;

use pasta_sample_ghost::{GhostConfig, generate_ghost};
use tempfile::TempDir;

/// ディレクトリ構造生成テスト
#[test]
fn test_directory_structure() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    // 必須ファイル存在確認
    assert!(
        ghost_root.join("install.txt").exists(),
        "install.txt が存在しません"
    );
    assert!(
        ghost_root.join("ghost/master/descript.txt").exists(),
        "ghost descript.txt が存在しません"
    );
    assert!(
        ghost_root.join("ghost/master/pasta.toml").exists(),
        "pasta.toml が存在しません"
    );
    assert!(
        ghost_root.join("ghost/master/dic/actors.pasta").exists(),
        "actors.pasta が存在しません"
    );
    assert!(
        ghost_root.join("ghost/master/dic/boot.pasta").exists(),
        "boot.pasta が存在しません"
    );
    assert!(
        ghost_root.join("ghost/master/dic/talk.pasta").exists(),
        "talk.pasta が存在しません"
    );
    assert!(
        ghost_root.join("ghost/master/dic/click.pasta").exists(),
        "click.pasta が存在しません"
    );
    assert!(
        ghost_root.join("shell/master/descript.txt").exists(),
        "shell descript.txt が存在しません"
    );
    assert!(
        ghost_root.join("shell/master/surfaces.txt").exists(),
        "surfaces.txt が存在しません"
    );
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
///
/// 仕様準拠: requirements.md Requirement 7.1-7.4
#[test]
fn test_pasta_toml_content() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    let content = std::fs::read_to_string(ghost_root.join("ghost/master/pasta.toml")).unwrap();

    // 必須セクション確認 (Req 7.1)
    assert!(
        content.contains("[package]"),
        "[package] セクションがありません"
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

    // [package] セクション内容確認 (Req 7.1)
    assert!(
        content.contains(r#"name = "hello-pasta""#),
        "package name がありません"
    );
    assert!(
        content.contains(r#"version = "1.0.0""#),
        "package version がありません"
    );
    assert!(
        content.contains(r#"edition = "2024""#),
        "package edition がありません"
    );

    // [loader] セクション内容確認 (Req 7.1, 7.2)
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
    // Task 7.5: user_scripts が検索パスに含まれることを確認
    assert!(
        content.contains(r#""user_scripts""#),
        "lua_search_paths に user_scripts がありません"
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

    // [ghost] セクション内容確認 (Req 7.1)
    assert!(
        content.contains("talk_interval_min = 180"),
        "talk_interval_min = 180 がありません"
    );
    assert!(
        content.contains("talk_interval_max = 300"),
        "talk_interval_max = 300 がありません"
    );

    // 教育的コメント確認 (Req 7.3)
    assert!(
        content.contains("教育的サンプル"),
        "教育的コメントがありません"
    );
    assert!(content.contains("省略可能"), "省略可能の説明がありません");
}

/// ukadoc 設定ファイル検証テスト
///
/// 仕様準拠: requirements.md Requirement 9.1-9.4
#[test]
fn test_ukadoc_files() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    // install.txt (Req 9.1)
    let install = std::fs::read_to_string(ghost_root.join("install.txt")).unwrap();
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

    // ghost descript.txt (Req 9.2)
    let ghost_desc = std::fs::read_to_string(ghost_root.join("ghost/master/descript.txt")).unwrap();
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
        ghost_desc.contains("craftmanw,どっとステーション駅長"),
        "ghost descript.txt に craftmanw がありません"
    );
    assert!(
        ghost_desc.contains("homeurl,https://github.com/ekicyou/pasta"),
        "ghost descript.txt に homeurl がありません"
    );

    // shell descript.txt (Req 9.3)
    let shell_desc = std::fs::read_to_string(ghost_root.join("shell/master/descript.txt")).unwrap();
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
        shell_desc.contains("sakura.balloon.offsetx,64"),
        "shell descript.txt の sakura.balloon.offsetx が64ではありません"
    );
    assert!(
        shell_desc.contains("sakura.balloon.offsety,0"),
        "shell descript.txt の sakura.balloon.offsety が0ではありません"
    );
    assert!(
        shell_desc.contains("kero.balloon.offsetx,64"),
        "shell descript.txt の kero.balloon.offsetx が64ではありません"
    );
    assert!(
        shell_desc.contains("kero.balloon.offsety,0"),
        "shell descript.txt の kero.balloon.offsety が0ではありません"
    );
}

/// pasta DSL スクリプト検証テスト
#[test]
fn test_pasta_scripts() {
    let temp = TempDir::new().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let config = GhostConfig::default();

    generate_ghost(&ghost_root, &config).unwrap();

    let dic_dir = ghost_root.join("ghost/master/dic");

    // actors.pasta - アクター辞書
    let actors = std::fs::read_to_string(dic_dir.join("actors.pasta")).unwrap();
    assert!(actors.contains("％女の子"), "女の子アクターがありません");
    assert!(actors.contains("％男の子"), "男の子アクターがありません");
    assert!(actors.contains("＠笑顔"), "笑顔表情がありません");
    assert!(actors.contains("＠怒り"), "怒り表情がありません");

    // boot.pasta
    let boot = std::fs::read_to_string(dic_dir.join("boot.pasta")).unwrap();
    assert!(boot.contains("＊OnBoot"), "OnBoot シーンがありません");
    assert!(
        boot.contains("＊OnFirstBoot"),
        "OnFirstBoot シーンがありません"
    );
    assert!(boot.contains("＊OnClose"), "OnClose シーンがありません");
    // アクター辞書が含まれていないことを確認
    assert!(
        !boot.contains("％女の子"),
        "boot.pasta にアクター辞書が含まれています"
    );

    // talk.pasta
    let talk = std::fs::read_to_string(dic_dir.join("talk.pasta")).unwrap();
    assert!(talk.contains("＊OnTalk"), "OnTalk シーンがありません");
    assert!(talk.contains("＊OnHour"), "OnHour シーンがありません");
    assert!(talk.contains("＄時"), "時刻変数参照がありません");
    // アクター辞書が含まれていないことを確認
    assert!(
        !talk.contains("％女の子"),
        "talk.pasta にアクター辞書が含まれています"
    );

    // click.pasta
    let click = std::fs::read_to_string(dic_dir.join("click.pasta")).unwrap();
    assert!(
        click.contains("＊OnMouseDoubleClick"),
        "OnMouseDoubleClick シーンがありません"
    );
    // アクター辞書が含まれていないことを確認
    assert!(
        !click.contains("％女の子"),
        "click.pasta にアクター辞書が含まれています"
    );

    // ダブルクリック反応は7種以上
    let click_count = click.matches("＊OnMouseDoubleClick").count();
    assert!(
        click_count >= 7,
        "ダブルクリック反応が7種未満: {}",
        click_count
    );
}

/// ランダムトークパターン数テスト
#[test]
fn test_random_talk_patterns() {
    let talk = pasta_sample_ghost::scripts::TALK_PASTA;

    // OnTalk パターン数（5〜10種）
    let talk_count = talk.matches("＊OnTalk").count();
    assert!(talk_count >= 5, "OnTalk パターンが5種未満: {}", talk_count);
    assert!(
        talk_count <= 10,
        "OnTalk パターンが10種超過: {}",
        talk_count
    );
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
    use pasta_sample_ghost::image_generator::{Character, Expression, generate_surface};

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
        assert!(
            msg.contains("pasta_shiori.dll"),
            "エラーメッセージにDLL名が含まれていません"
        );
        assert!(
            msg.contains("cargo build"),
            "ビルドコマンドの案内がありません"
        );
    }
    // DLL がある場合は成功
}
