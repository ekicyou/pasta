//! ログの量の設定がゴーストのディレクトリごとに分かれることの統合テスト（shiori4-v3-portable 5.4）。
//!
//! 購読者とロガーの登録簿はプロセス全体で 1 つのため、他のテストと干渉しないよう専用の
//! テストバイナリに 1 本だけ置く。`mod common` の宣言により `PASTA_DEBUG`・`PASTA_LOG`
//! 中和ガードが適用される。
//!
//! 判定は書き出されたログで確かめる（層のフィルタが 1 つだけのとき、`tracing::enabled!` は
//! その層が断っても真を返すため。書き出しは層のフィルタに従う）。

mod common;

use common::create_temp_with_pasta;
use pasta_lua::loader::PastaLoader;
use pasta_lua::{GlobalLoggerRegistry, LoadDirGuard};
use std::path::Path;

const BASE_TOML: &str = "[loader]\ndebug_mode = true\n";

fn write_logging(dir: &Path, level: &str) {
    std::fs::write(
        dir.join("pasta.toml"),
        format!("{BASE_TOML}[logging]\nlevel = \"{level}\"\n"),
    )
    .unwrap();
}

/// そのディレクトリの文脈で、量ごとの印を 1 行ずつ出す。
fn emit_markers(dir: &Path, tag: &str) {
    let _guard = LoadDirGuard::new(dir.to_path_buf());
    tracing::warn!("marker-{tag}-warn");
    tracing::info!("marker-{tag}-info");
    tracing::debug!("marker-{tag}-debug");
    tracing::trace!("marker-{tag}-trace");
}

fn read_log(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("profile/pasta/logs/pasta.log")).unwrap()
}

#[test]
fn tracing_filter_follows_each_directory_logging_config() {
    pasta_lua::init_tracing();
    let quiet = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    let verbose = create_temp_with_pasta("＊テスト\n  ゴースト：「こんにちは」\n");
    write_logging(quiet.path(), "warn");
    write_logging(verbose.path(), "debug");

    // 2 つを読む（後から読んだ側の設定が先に読んだ側へ効かないこと）。
    let quiet_rt = PastaLoader::load(quiet.path()).unwrap();
    let verbose_rt = PastaLoader::load(verbose.path()).unwrap();

    // 文脈を行き来しても、それぞれの設定のまま。
    emit_markers(quiet.path(), "q1");
    emit_markers(verbose.path(), "v");
    emit_markers(quiet.path(), "q2");
    // 文脈が無く、ロガーを持つ項目が 2 つ在るときは選ばない（ほかのゴーストの設定へ逃げない）。
    tracing::error!("marker-none-error");

    // ランタイムと登録簿の両方が手放すとロガーが破棄され、ログがフラッシュされる。
    drop(quiet_rt);
    drop(verbose_rt);
    let registry = GlobalLoggerRegistry::instance();
    registry.unregister(quiet.path());
    registry.unregister(verbose.path());

    let quiet_log = read_log(quiet.path());
    let verbose_log = read_log(verbose.path());
    for tag in ["q1", "q2"] {
        assert!(
            quiet_log.contains(&format!("marker-{tag}-warn")),
            "{quiet_log}"
        );
        assert!(
            !quiet_log.contains(&format!("marker-{tag}-info")),
            "{quiet_log}"
        );
        assert!(
            !quiet_log.contains(&format!("marker-{tag}-debug")),
            "{quiet_log}"
        );
    }
    assert!(verbose_log.contains("marker-v-info"), "{verbose_log}");
    assert!(verbose_log.contains("marker-v-debug"), "{verbose_log}");
    assert!(!verbose_log.contains("marker-v-trace"), "{verbose_log}");
    // 印は自分のディレクトリのログにだけ出る。
    assert!(!quiet_log.contains("marker-v-"), "{quiet_log}");
    assert!(!verbose_log.contains("marker-q"), "{verbose_log}");
    assert!(!quiet_log.contains("marker-none"), "{quiet_log}");
    assert!(!verbose_log.contains("marker-none"), "{verbose_log}");
}
