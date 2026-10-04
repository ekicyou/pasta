//! 起動のたびに `[logging]` の設定がトレースのフィルタへ反映されることの統合テスト。
//!
//! フィルタはプロセス全体で 1 つのため、他のテストと干渉しないよう専用のテストバイナリに
//! 1 本だけ置く。`mod common` の宣言により `PASTA_DEBUG` 中和ガードが適用される。

mod common;

use common::create_temp_with_pasta;
use pasta_lua::loader::PastaLoader;
use tracing::Level;

const BASE_TOML: &str = "[loader]\ndebug_mode = true\n";

#[test]
fn tracing_filter_follows_logging_config_on_every_load() {
    pasta_lua::init_tracing_with_reload(&pasta_lua::LoggingConfig::default());
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();
    let toml = base_dir.join("pasta.toml");

    // `[logging]` の level が反映される。
    std::fs::write(&toml, format!("{BASE_TOML}[logging]\nlevel = \"warn\"\n")).unwrap();
    PastaLoader::load(base_dir).unwrap();
    assert!(!tracing::enabled!(Level::INFO));

    // `[logging]` を消して読み直すと既定（info）へ戻る。
    std::fs::write(&toml, BASE_TOML).unwrap();
    PastaLoader::load(base_dir).unwrap();
    assert!(tracing::enabled!(Level::INFO));

    // ログファイルを作れない file_path でも level は反映される。
    std::fs::write(
        &toml,
        format!("{BASE_TOML}[logging]\nlevel = \"warn\"\nfile_path = \"../outside.log\"\n"),
    )
    .unwrap();
    PastaLoader::load(base_dir).unwrap();
    assert!(!tracing::enabled!(Level::INFO));
}
