//! 版 3 の入口の読み込みの終わりの行（仕様 `debug-mcp-runtime`・要件 10.1〜10.4）。
//!
//! `[logging]` を絞ったゴーストでも、`pasta.log` に `SHIORI load entry completed` の行が
//! `entry`・`loaded` の欄を付けて 1 行出て、ほかの info は出ないことを確かめる。
//!
//! `mod common;` は `PASTA_DEBUG` / `PASTA_LOG` を main 前に中和する `#[ctor]` を取り込む（要件 9.3）。

mod common;

use common::{copy_fixture_to_temp, wait_log_contains};
use pasta::shiori_get_api;
use shiori4_abi::{SHIORI_ABI_VERSION_3, ShioriApi};

#[test]
fn v3_load_entry_line_survives_narrowed_logging() {
    // SAFETY: shiori_get_api はこの試験のバイナリに静的にリンクされ、テーブルは static。
    let api =
        unsafe { ShioriApi::from_raw(shiori_get_api(SHIORI_ABI_VERSION_3), SHIORI_ABI_VERSION_3) }
            .expect("v3 table must pass validation");

    let ghost = copy_fixture_to_temp("shiori_lifecycle");
    let toml = ghost.path().join("pasta.toml");
    let mut text = std::fs::read_to_string(&toml).expect("read pasta.toml");
    text.push_str("\n[logging]\nlevel = \"warn\"\n");
    std::fs::write(&toml, text).expect("write pasta.toml");

    let inst = api
        .load_v3(ghost.path().to_str().expect("utf-8 path"))
        .expect("v3 load");
    drop(inst);

    let log = wait_log_contains(ghost.path(), "SHIORI load entry completed");
    assert!(
        log.matches("SHIORI load entry completed").count() == 1
            && log.contains("entry=\"v3\" loaded=true"),
        "絞った設定でも版 3 の読み込みの終わりの行が出るべき（要件 10.1・10.2）\nlog:\n{log}"
    );
    assert!(
        // `[logging]` を読む前の起動の数行は既定の info で出る（今のまま）。読んだ後の info で見る
        !log.contains("Startup sequence completed"),
        "読み込みの終わりの行のほかの info は絞られたままであるべき（要件 10.4）\nlog:\n{log}"
    );
}
