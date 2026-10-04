//! 組み込み経路（`PastaLoader` を直接使う）で、不正な `[logging] file_path` が
//! 既定のログファイルへフォールバックすることの統合テスト。
//!
//! ロガーの登録簿はプロセス全体で 1 つのため、専用のテストバイナリに 1 本だけ置く。
//! `mod common` の宣言により `PASTA_DEBUG`・`PASTA_LOG` 中和ガードが適用される。

mod common;

use common::create_temp_with_pasta;
use pasta_lua::GlobalLoggerRegistry;
use pasta_lua::loader::PastaLoader;

#[test]
fn invalid_file_path_falls_back_to_default_log_file() {
    pasta_lua::init_tracing_with_reload(&pasta_lua::LoggingConfig::default());
    let temp = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let base_dir = temp.path();
    std::fs::write(
        base_dir.join("pasta.toml"),
        "[loader]\ndebug_mode = true\n[logging]\nfile_path = \"profile.log\"\n",
    )
    .unwrap();

    let runtime = PastaLoader::load(base_dir).unwrap();

    // 組み込み経路はロガーの登録を外さないので、テストが外す。
    // ランタイムと登録簿の両方が手放すとロガーが破棄され、ログがフラッシュされる。
    drop(runtime);
    GlobalLoggerRegistry::instance().unregister(base_dir);

    let log = std::fs::read_to_string(base_dir.join("profile/pasta/logs/pasta.log")).unwrap();
    let warn_pos = log
        .lines()
        .position(|l| l.contains("WARN") && l.contains("profile.log"))
        .unwrap_or_else(|| panic!("fallback warn not found in default log:\n{log}"));
    let later_pos = log
        .lines()
        .position(|l| l.contains("Startup sequence completed"))
        .unwrap_or_else(|| panic!("loader log not found in default log:\n{log}"));
    assert!(
        warn_pos < later_pos,
        "loader log must follow the warn:\n{log}"
    );

    assert!(!base_dir.join("profile.log").exists());
}
