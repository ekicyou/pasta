//! 標準 Lua ファイル searcher と `install_module_searcher` のバイト同一性契約テスト。
//!
//! 同一の検索パス設定・同一のファイル群に対し、`package.loaders[2]` が標準のままの
//! VM（STD）と Rust 実装へ置換した VM（NEW）を並べ、次をバイト単位で比較する。
//!
//! - チャンク識別子（`source` / `short_src`）とチャンクへ渡される引数
//!   （`?.lua` と `?/init.lua` の両パターン・モジュール名にドットを含む場合を含む）
//!   — 要件 3.4（「現行の区切り文字の混在を含む」）
//! - 未検出・構文エラー・実行時エラーの 3 種について、`pcall(require, …)` が受け取る
//!   値の型と文言 — 要件 1.5 / 4.9
//! - 検索パスの優先順位と `?.lua` → `?/init.lua` の試行順 — 要件 3.1 / 3.2 / 3.3
//! - `package.loaded` 登録済みモジュールが searcher を経由せず解決されること
//!
//! これが要件 3.4 の恒久ゲートであり、`#[ignore]` は付けない（要件 6.6: 常時実行）。
//! `PASTA_DEBUG` の中和は `tests/common` の `#[ctor]` ガードが担う（要件 6.4）。
//!
//! 標準 searcher は長パス・非 ANSI パスを扱えないため、比較の土台は短い ASCII の
//! 一時ディレクトリに限る。長パス・非 ANSI 側の検証は
//! `tests/loader/path_robustness_test.rs` が担当する。

use mlua::{Lua, LuaOptions, StdLib, Table};
use pasta_lua::runtime::install_module_searcher;
use std::path::{Path, PathBuf};

/// 解決されたチャンク自身が自分の識別子と引数を報告するモジュール本体。
///
/// 戻り値は `source \1 short_src \1 引数の個数 \1 引数を \2 で連結したもの`。
/// 単一の文字列に畳むことで、比較が正規化を挟まないバイト比較になる。
const PROBE_BODY: &str = r#"local n = select('#', ...)
local args = {}
for i = 1, n do args[i] = tostring((select(i, ...))) end
local info = debug.getinfo(1, 'S')
return info.source .. "\1" .. info.short_src .. "\1" .. n .. "\1" .. table.concat(args, "\2")
"#;

/// モジュール名の `.` が置換される OS のディレクトリ区切り（LuaJIT の `LUA_DIRSEP` と同じ）。
const DIR_SEP: char = std::path::MAIN_SEPARATOR;

/// `PROBE_BODY` の戻り値から `source` フィールドを取り出す。
fn source_field(probe_result: &str) -> &str {
    probe_result
        .split('\u{1}')
        .next()
        .expect("プローブの戻り値が空")
}

/// 比較の土台となる一時ツリー。2 つの検索ルートと、両 VM に与える `package.path` を持つ。
struct Tree {
    _dir: tempfile::TempDir,
    /// 優先順位の高い検索ルート（検索パス文字列と同じ `/` 区切り表記）。
    first: String,
    /// 優先順位の低い検索ルート（同上）。
    second: String,
    package_path: String,
}

impl Tree {
    /// 比較に使うモジュール群を配置した一時ツリーを作る。
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗した");
        let first_dir = dir.path().join("first");
        let second_dir = dir.path().join("second");

        // 標準 searcher は narrow API でファイルを開くため、土台は短い ASCII でなければ
        // ならない。前提が崩れたときに原因不明の失敗にならないよう明示的に確かめる。
        let root = dir.path().to_string_lossy().into_owned();
        assert!(
            root.is_ascii() && root.len() < 150,
            "比較の土台は短い ASCII パスである必要がある（標準 searcher の制約）: {root}"
        );

        // `?.lua` パターン・ドット無し。
        write(&first_dir, "plain.lua", PROBE_BODY);
        // `?.lua` パターン・ドット有り（前置部 `/`・展開部 OS 区切りの混在を作る）。
        write(&first_dir, "a/b.lua", PROBE_BODY);
        // `?/init.lua` パターン・ドット有り。
        write(&first_dir, "pkg/sub/init.lua", PROBE_BODY);
        // 同一モジュール名が両パターンに存在する（試行順の検証用・要件 3.2）。
        write(&first_dir, "both.lua", PROBE_BODY);
        write(&first_dir, "both/init.lua", PROBE_BODY);
        // 同一モジュール名が両検索ルートに存在する（優先順位の検証用・要件 3.3）。
        write(&first_dir, "dup.lua", PROBE_BODY);
        write(&second_dir, "dup.lua", PROBE_BODY);
        // ロード失敗の 2 種。
        write(&first_dir, "bad_syntax.lua", "return return\n");
        write(&first_dir, "bad_runtime.lua", "error(\"boom\")\n");
        // `package.loaded` 登録済みモジュールと同名のファイル。searcher を経由したら
        // こちらが返るため、バイパスの検証が空振りしない。
        write(&first_dir, "@pasta_config.lua", "return \"from file\"\n");

        let first = to_search_path_string(&first_dir);
        let second = to_search_path_string(&second_dir);
        // 実際の `LoaderContext::generate_package_path()` と同じ形（検索パスごとに
        // `?.lua` → `?/init.lua`・区切りは `/`・連結は `;`）。
        let package_path =
            format!("{first}/?.lua;{first}/?/init.lua;{second}/?.lua;{second}/?/init.lua");

        Self {
            _dir: dir,
            first,
            second,
            package_path,
        }
    }

    /// このツリーに対する VM を作る。
    ///
    /// 本番と同じ構築方法（`unsafe_new_with`）を使う。安全モードの `Lua::new` は C
    /// モジュールを落とし `package.loaders` が 3 要素になるため、標準側の未検出文言が
    /// 本番と食い違い、`install_module_searcher` もレイアウト検証で `Err` になる。
    /// `StdLib::ALL` はプローブが使う `debug` ライブラリを含む。両 VM は同一の構築
    /// 方法・同一のライブラリ構成であり、比較に非対称は生じない。
    fn vm(&self, searcher: Searcher) -> Lua {
        let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, LuaOptions::default()) };
        if searcher == Searcher::New {
            install_module_searcher(&lua).expect("本番同等 VM への searcher 設置に失敗した");
        }
        let package: Table = lua
            .globals()
            .get("package")
            .expect("package テーブルが無い");
        package
            .set("path", self.package_path.as_str())
            .expect("package.path の設定に失敗した");
        lua
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Searcher {
    /// `package.loaders[2]` が LuaJIT 標準のままの VM。
    Std,
    /// `install_module_searcher` を適用した VM。
    New,
}

/// 検索パス文字列と同じ規則でパスを表記する（`generate_package_path` と同じく区切りは `/`）。
fn to_search_path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write(root: &Path, relative: &str, content: &str) {
    let path: PathBuf = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("親ディレクトリが無い"))
        .expect("ディレクトリの作成に失敗した");
    std::fs::write(&path, content).expect("フィクスチャの書き込みに失敗した");
}

fn eval(lua: &Lua, chunk: &str) -> String {
    lua.load(chunk)
        .eval()
        .unwrap_or_else(|e| panic!("チャンクの評価に失敗した: {chunk}\n  エラー: {e}"))
}

/// 同じチャンクを STD / NEW の両 VM で評価し、結果がバイト単位で一致することを確かめる。
///
/// 正規化・部分一致は行わない。一致した値を返す。
fn assert_identical(tree: &Tree, chunk: &str) -> String {
    let standard = eval(&tree.vm(Searcher::Std), chunk);
    let replaced = eval(&tree.vm(Searcher::New), chunk);
    assert_eq!(
        standard, replaced,
        "標準 searcher と置換 searcher の結果がバイト一致しない\n  チャンク: {chunk}"
    );
    replaced
}

/// `require` の結果（プローブの戻り値）を取り出すチャンク。
fn require_probe(module: &str) -> String {
    format!("return require(\"{module}\")")
}

/// `pcall(require, …)` が受け取る値の型と文言を 1 つの文字列に畳むチャンク。
fn pcall_require(module: &str) -> String {
    format!(
        "local ok, e = pcall(require, \"{module}\")\n\
         return tostring(ok) .. \"\\1\" .. type(e) .. \"\\1\" .. tostring(e)"
    )
}

/// 要件 3.4 / 3.1 / 3.2: `?.lua` と `?/init.lua` の両パターンについて、チャンク識別子
/// （`source` / `short_src`）とチャンクへ渡される引数がバイト単位で一致する。
///
/// ドットを含むモジュール名を含めることで、前置部が `/`・展開部が OS 区切りという
/// 現行の混在形が実際に通る。
#[test]
fn chunk_identity_matches_standard_for_both_filename_patterns() {
    let tree = Tree::new();

    // `?.lua` パターン・ドット無し。
    let plain = assert_identical(&tree, &require_probe("plain"));
    assert_eq!(
        source_field(&plain),
        format!("@{}/plain.lua", tree.first),
        "チャンク識別子が候補パスそのものになっていない"
    );

    // `?.lua` パターン・ドット有り（区切り文字の混在）。
    let dotted = assert_identical(&tree, &require_probe("a.b"));
    assert_eq!(
        source_field(&dotted),
        format!("@{}/a{DIR_SEP}b.lua", tree.first),
        "ドットを含むモジュール名で区切り文字の混在形が再現されていない"
    );

    // `?/init.lua` パターン・ドット有り。
    let init = assert_identical(&tree, &require_probe("pkg.sub"));
    assert_eq!(
        source_field(&init),
        format!("@{}/pkg{DIR_SEP}sub/init.lua", tree.first),
        "`?/init.lua` パターンのチャンク識別子が候補パスそのものになっていない"
    );

    // 引数（モジュール名 1 個）まで比較対象に入っていることを明示する。
    for (module, probe) in [("plain", &plain), ("a.b", &dotted), ("pkg.sub", &init)] {
        let fields: Vec<&str> = probe.split('\u{1}').collect();
        assert_eq!(
            fields.len(),
            4,
            "プローブの戻り値の構造が想定と違う: {probe}"
        );
        assert_eq!(fields[2], "1", "チャンクへの引数の個数が 1 でない: {probe}");
        assert_eq!(fields[3], module, "チャンクへの引数がモジュール名でない");
    }
}

/// 要件 3.2: 1 つの検索パスに対し `?.lua` を `?/init.lua` より先に試す。
#[test]
fn plain_file_pattern_is_tried_before_init_pattern() {
    let tree = Tree::new();

    let resolved = assert_identical(&tree, &require_probe("both"));

    assert_eq!(
        source_field(&resolved),
        format!("@{}/both.lua", tree.first),
        "`?.lua` より先に `?/init.lua` が解決されている"
    );
}

/// 要件 3.3: 同名モジュールが複数の検索パスにあるとき優先順位の高い側が勝つ。
#[test]
fn higher_priority_search_path_wins() {
    let tree = Tree::new();

    let resolved = assert_identical(&tree, &require_probe("dup"));

    assert_eq!(
        source_field(&resolved),
        format!("@{}/dup.lua", tree.first),
        "優先順位の低い検索パスのモジュールが解決されている"
    );
}

/// 要件 1.5 / 4.9 / 3.1 / 3.2: 未検出時に `pcall` が受け取る値の型と文言がバイト一致し、
/// 全 searcher の候補行を検索パス順・パターン順に含む。
#[test]
fn not_found_error_matches_standard_byte_for_byte() {
    let tree = Tree::new();

    let observed = assert_identical(&tree, &pcall_require("no.such"));

    assert!(
        observed.starts_with("false\u{1}string\u{1}module 'no.such' not found:"),
        "未検出エラーが標準の型（文字列）・書式になっていない: {observed}"
    );

    // 候補パスが「検索パス順 × `?.lua` → `?/init.lua`」の順に並ぶ（要件 3.1 / 3.2）。
    let first = &tree.first;
    let second = &tree.second;
    let expected_order = format!(
        "\n\tno file '{first}/no{DIR_SEP}such.lua'\
         \n\tno file '{first}/no{DIR_SEP}such/init.lua'\
         \n\tno file '{second}/no{DIR_SEP}such.lua'\
         \n\tno file '{second}/no{DIR_SEP}such/init.lua'"
    );
    assert!(
        observed.contains(&expected_order),
        "候補パスの探索順が現行と違う\n  期待する並び: {expected_order}\n  実際: {observed}"
    );
}

/// 要件 4.9: 構文エラー時に `pcall` が受け取る値の型と文言がバイト一致する。
#[test]
fn syntax_error_matches_standard_byte_for_byte() {
    let tree = Tree::new();

    let observed = assert_identical(&tree, &pcall_require("bad_syntax"));

    assert!(
        observed.starts_with(&format!(
            "false\u{1}string\u{1}error loading module 'bad_syntax' from file '{}/bad_syntax.lua':",
            tree.first
        )),
        "構文エラーが標準の型（文字列）・書式になっていない: {observed}"
    );
}

/// 要件 4.9: 実行時エラー時に `pcall` が受け取る値の型と文言がバイト一致する。
///
/// 文言には短縮ソース名が現れるため、チャンク識別子の同一性もここで効く。
#[test]
fn runtime_error_matches_standard_byte_for_byte() {
    let tree = Tree::new();

    let observed = assert_identical(&tree, &pcall_require("bad_runtime"));

    assert!(
        observed.starts_with("false\u{1}string\u{1}"),
        "実行時エラーが文字列として届いていない: {observed}"
    );
    assert!(
        observed.ends_with(":1: boom"),
        "実行時エラーの文言が標準の書式になっていない: {observed}"
    );
    assert!(
        observed.contains("bad_runtime.lua"),
        "実行時エラーの文言にチャンク識別子が現れていない: {observed}"
    );
}

/// `package.loaded` に登録済みのモジュール（本クレートが登録する `@pasta_config` 等）は
/// searcher を経由せず解決される。同名のファイルを検索パス上に置いてあるため、
/// バイパスが壊れれば戻り値が変わる。
#[test]
fn preregistered_module_bypasses_the_searcher() {
    let tree = Tree::new();

    let observed = assert_identical(
        &tree,
        "package.loaded[\"@pasta_config\"] = \"sentinel\"\n\
         local v = require(\"@pasta_config\")\n\
         return type(v) .. \"\\1\" .. tostring(v)",
    );

    assert_eq!(
        observed, "string\u{1}sentinel",
        "登録済みモジュールが searcher 経由で再解決されている"
    );
}
