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
    pasta_lua::init_tracing();
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();
    let toml = base_dir.join("pasta.toml");

    // 反映は量の上限（max_level_hint）で見る。読み込みの終わりの行の target はいつも info を
    // 通すので上限は info より下がらない。そのため debug で見分ける。
    // `[logging]` の level が反映される。
    std::fs::write(&toml, format!("{BASE_TOML}[logging]\nlevel = \"debug\"\n")).unwrap();
    PastaLoader::load(base_dir).unwrap();
    assert!(tracing::enabled!(Level::DEBUG));

    // `[logging]` を消して読み直すと既定（info）へ戻る。
    std::fs::write(&toml, BASE_TOML).unwrap();
    PastaLoader::load(base_dir).unwrap();
    assert!(!tracing::enabled!(Level::DEBUG));

    // ログファイルを作れない file_path でも level は反映される。
    std::fs::write(
        &toml,
        format!("{BASE_TOML}[logging]\nlevel = \"debug\"\nfile_path = \"../outside.log\"\n"),
    )
    .unwrap();
    PastaLoader::load(base_dir).unwrap();
    assert!(tracing::enabled!(Level::DEBUG));
}
