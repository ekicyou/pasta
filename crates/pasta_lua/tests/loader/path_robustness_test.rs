//! 長パス・非 ANSI 設置パスでのモジュール解決に関する統合テスト。
//!
//! 3 種の設置先（300 文字超の ASCII ／ 非 ANSI ／ 両方）それぞれで、本番経路である
//! `PastaLoader::load` が完走し、内蔵スクリプト・利用者スクリプト・トランスパイル済み
//! シーンモジュールの 3 層が解決されることを検証する
//! （要件 1.1 / 1.2 / 1.3 / 1.4 / 2.1 / 2.2 / 6.1 / 6.2 / 6.7）。
//!
//! あわせて、解決済みモジュールのチャンク識別子が拡張長プレフィックスを含まない
//! 検索パス由来の UTF-8 文字列であること（要件 3.7 / 3.8）と、未検出エラーの文言が
//! 候補パスを非 ASCII 文字ごと欠落なく含むこと（要件 1.5 / 2.3）を確認する。
//!
//! `#[ignore]` は付けない（要件 6.6: 常時実行）。`PASTA_DEBUG` の中和は
//! `tests/common` の `#[ctor]` ガードが担う（要件 6.4）。

use crate::common;

use common::{NON_ANSI_DIR_NAME, copy_fixture_into, make_deep_dir, value_as_str};
use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use std::path::Path;

/// 利用者スクリプト層の解決を確かめるために `scripts/` へ置く追加モジュール。
const PROBE_MODULE: &str = "pasta_path_probe";

/// 非 ASCII のファイル名を持つシーンソース（設計の例に従う）。
const NON_ASCII_SCENE_FILE: &str = "dic/会話.pasta";

/// `NON_ASCII_SCENE_FILE` から生成されるモジュール名（`dic/` を落として `pasta.scene.` を冠する）。
const NON_ASCII_SCENE_MODULE: &str = "pasta.scene.会話";

/// `minimal` フィクスチャから生成されるシーンモジュール名。
const ASCII_SCENE_MODULE: &str = "pasta.scene.test.hello";

/// 自己展開される内蔵スクリプトのうち、SHIORI 応答関数を定義するモジュール。
const BUILTIN_MODULE: &str = "pasta.shiori.entry";

/// 検索パス文字列と同じ規則でパスを表記する（`generate_package_path` と同じく区切りは `/`）。
fn to_search_path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// `base_dir` へ、3 層すべてを含む一時ゴーストを設置する。
fn install_ghost(base_dir: &Path) {
    copy_fixture_into("minimal", base_dir);

    // 利用者スクリプト層: 明示的な `require` で解決させる追加モジュール。
    write_probe_script(base_dir);

    // シーン層: 非 ASCII のファイル名を持つシーンソースを 1 本追加する。
    let scene_path = base_dir.join(NON_ASCII_SCENE_FILE);
    std::fs::create_dir_all(scene_path.parent().unwrap()).expect("dic ディレクトリの作成に失敗");
    std::fs::write(&scene_path, "＊会話テスト\n  ゴースト：「こんにちは！」\n")
        .expect("シーンソースの書き込みに失敗");
}

/// 利用者スクリプト層の追加モジュールを `scripts/` へ置く。
///
/// 本番ランタイムは `debug` ライブラリを Lua 側へ公開しないため、チャンク識別子は
/// Rust 側の `Function::info()` で取得する。そのためモジュールは関数を返す。
fn write_probe_script(base_dir: &Path) {
    let scripts_dir = base_dir.join("scripts");
    std::fs::create_dir_all(&scripts_dir).expect("scripts ディレクトリの作成に失敗");
    std::fs::write(
        scripts_dir.join(format!("{PROBE_MODULE}.lua")),
        "return function() end\n",
    )
    .expect("利用者スクリプトの書き込みに失敗");
}

/// 解決済みモジュールのチャンク識別子（`lua_Debug.source`）を取得する。
fn resolve_probe_chunk_source(runtime: &PastaLuaRuntime) -> String {
    let value = runtime
        .exec(&format!("return require(\"{PROBE_MODULE}\")"))
        .unwrap_or_else(|e| panic!("利用者スクリプト `{PROBE_MODULE}` の解決に失敗した: {e}"));

    match value {
        mlua::Value::Function(f) => f
            .info()
            .source
            .expect("解決済みチャンクには識別子があるべき"),
        other => panic!("利用者スクリプトは関数を返すべき: {other:?}"),
    }
}

/// 起動時に解決されたモジュールが `package.loaded` に載っていることを確認する。
fn assert_module_loaded(runtime: &PastaLuaRuntime, module: &str) {
    let loaded = runtime
        .exec(&format!(
            "return package.loaded[\"{module}\"] ~= nil and type(package.loaded[\"{module}\"])"
        ))
        .unwrap_or_else(|e| panic!("`package.loaded` の参照に失敗した ({module}): {e}"));

    match value_as_str(&loaded) {
        Some(_) => {}
        None => panic!("モジュール `{module}` が解決されていない（`package.loaded` に無い）"),
    }
}

/// 設置先 `base_dir` について、本仕様が保証すべきモジュール解決の性質をすべて検証する。
fn assert_module_resolution_is_path_shape_independent(base_dir: &Path) {
    install_ghost(base_dir);

    // 要件 1.1 / 1.2 / 2.1 / 2.2: 本番経路のロードが設置パスの形に依存せず完走する。
    let runtime = PastaLoader::load(base_dir).unwrap_or_else(|e| {
        panic!(
            "設置パスの形に依存せずロードは成功するべき\n  設置先: {}\n  エラー: {e}",
            base_dir.display()
        )
    });

    // 要件 1.4: 内蔵スクリプト・シーンモジュールの 2 層は起動シーケンス中に解決される。
    assert_module_loaded(&runtime, BUILTIN_MODULE);
    assert_module_loaded(&runtime, ASCII_SCENE_MODULE);

    // 要件 2.1 / 3.8: モジュール名に非 ASCII を含むシーンも解決される。
    assert_module_loaded(&runtime, NON_ASCII_SCENE_MODULE);

    // 要件 1.4: 利用者スクリプト層。未ロードの追加モジュールを検索パスから解決させる。
    let probe_source = resolve_probe_chunk_source(&runtime);

    // 要件 3.7: 長パス対応のための内部表現（拡張長プレフィックス）を露出させない。
    assert!(
        !probe_source.contains(r"\\?\") && !probe_source.contains("//?/"),
        "チャンク識別子に拡張長プレフィックスが露出している: {probe_source}"
    );

    // 要件 3.7 / 3.8: チャンク識別子は検索パス由来の UTF-8 文字列（`@` + 候補パス）である。
    let expected_source = format!(
        "@{}/scripts/{PROBE_MODULE}.lua",
        to_search_path_string(base_dir)
    );
    assert_eq!(
        probe_source, expected_source,
        "チャンク識別子が検索パス由来の UTF-8 文字列になっていない"
    );

    // 要件 1.5 / 2.3: 未検出エラーは候補パスを含み、非 ASCII 文字を欠落・置換しない。
    let not_found = runtime
        .exec("return require(\"pasta_absent_module_marker\")")
        .expect_err("存在しないモジュールの `require` は失敗するべき")
        .to_string();
    assert!(
        not_found.contains("pasta_absent_module_marker"),
        "未検出エラーにモジュール名が含まれていない: {not_found}"
    );
    let scripts_search_dir = format!("{}/scripts", to_search_path_string(base_dir));
    assert!(
        not_found.contains(&scripts_search_dir),
        "未検出エラーに探索した候補パスがそのまま含まれていない\n  期待: {scripts_search_dir}\n  実際: {not_found}"
    );
    if base_dir.to_string_lossy().contains(NON_ANSI_DIR_NAME) {
        assert!(
            not_found.contains(NON_ANSI_DIR_NAME),
            "未検出エラーの非 ASCII ディレクトリ名が欠落・置換されている: {not_found}"
        );
    }
}

/// 要件 1.1 / 1.3 / 6.1: 300 文字超の ASCII 設置パス。
#[test]
fn long_ascii_install_path_resolves_all_layers() {
    let temp = tempfile::TempDir::new().expect("一時ディレクトリの作成に失敗");
    let base_dir = make_deep_dir(temp.path(), 300);

    assert!(
        base_dir.display().to_string().chars().count() > 300,
        "設置先が 300 文字を超えていない: {}",
        base_dir.display()
    );

    assert_module_resolution_is_path_shape_independent(&base_dir);
}

/// 要件 2.1 / 6.2 / 6.7: ANSI コードページ外の文字を含む設置パス。
#[test]
fn non_ansi_install_path_resolves_all_layers() {
    let temp = tempfile::TempDir::new().expect("一時ディレクトリの作成に失敗");
    let base_dir = temp.path().join(NON_ANSI_DIR_NAME);
    std::fs::create_dir_all(&base_dir).expect("非 ANSI ディレクトリの作成に失敗");

    assert_module_resolution_is_path_shape_independent(&base_dir);
}

/// 要件 2.2 / 6.1 / 6.2: 300 文字超かつ非 ANSI を兼ねる設置パス。
#[test]
fn long_and_non_ansi_install_path_resolves_all_layers() {
    let temp = tempfile::TempDir::new().expect("一時ディレクトリの作成に失敗");
    let root = temp.path().join(NON_ANSI_DIR_NAME);
    std::fs::create_dir_all(&root).expect("非 ANSI ディレクトリの作成に失敗");
    let base_dir = make_deep_dir(&root, 300);

    assert!(
        base_dir.display().to_string().chars().count() > 300,
        "設置先が 300 文字を超えていない: {}",
        base_dir.display()
    );
    assert!(
        base_dir.to_string_lossy().contains(NON_ANSI_DIR_NAME),
        "設置先に非 ANSI セグメントが残っていない: {}",
        base_dir.display()
    );

    assert_module_resolution_is_path_shape_independent(&base_dir);
}
