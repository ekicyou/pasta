//! サンプルゴースト統合テスト（起動・検索パス・失敗継続）。
//!
//! pasta-scripts-self-deploy Task 5.3 / Req 1.3, 3.1, 3.2, 5.5, 6.2, 6.3, 6.5。
//!
//! 実際のサンプルゴースト（`ghosts/hello-pasta/ghost/master`）を一時ディレクトリへ
//! コピーし、本物の `PastaLoader::load` で駆動して以下を観測・検証する:
//! - フレッシュ初回起動で自己展開先（`profile/pasta/pasta_scripts/`）が生成され、
//!   フレームワークモジュールが `require` で解決すること（Req 1.3 / 5.5 / 6.3）。
//! - `scripts/` 上書きが自己展開先より優先されること（Req 6.2 / 6.5）。
//! - 自己展開先のスワップを書込不可（排他ロック）にしても、ERROR ログを出しつつ
//!   起動が継続すること（Req 3.1 / 3.2）。
//!
//! # 汚染ガード
//! フィクスチャはコミット済みのため絶対にその場でロードしない。常に tempdir へコピーし、
//! 自己展開の書き込みは temp 配下にのみ発生させる。コピー時に `profile/` をスキップして
//! 必ずフレッシュ状態から開始する。

use std::path::Path;

use pasta_lua::loader::PastaLoader;
use pasta_lua::mlua;
use tempfile::TempDir;

/// Lua の文字列値を Rust 文字列として取り出すヘルパー。
fn value_as_str(value: &mlua::Value) -> Option<String> {
    value
        .as_string()
        .and_then(|s| s.to_str().ok())
        .map(|s| s.to_string())
}

/// テスト対象の本物のサンプルゴースト master ディレクトリ。
fn fixture_master_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ghosts/hello-pasta/ghost/master")
}

/// `src` のディレクトリツリーを `dst` へ再帰コピーする。ただし `profile/` サブディレクトリ
/// （フィクスチャに自己展開済みの版が万一あっても）は持ち越さず、必ずフレッシュ初回起動から
/// 始められるようスキップする。
fn copy_tree_skip_profile(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create dst dir");
    for entry in std::fs::read_dir(src).expect("read src dir").flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let meta = entry.metadata().expect("metadata");
        if meta.is_dir() {
            // フレッシュ自己展開を保証するため profile/ は持ち越さない。
            if name == std::ffi::OsStr::new("profile") {
                continue;
            }
            copy_tree_skip_profile(&path, &dst.join(&name));
        } else {
            std::fs::copy(&path, dst.join(&name)).expect("copy file");
        }
    }
}

/// フィクスチャを tempdir へ（profile を除いて）コピーし、その tempdir を返す。
fn fresh_ghost_copy() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    copy_tree_skip_profile(&fixture_master_dir(), temp.path());
    temp
}

/// 自己展開先（`base_dir` 相対）。production の `SELF_DEPLOY_REL` と一致させる。
const SELF_DEPLOY_REL: &str = "profile/pasta/pasta_scripts";
const MARKER_NAME: &str = ".md5";

// ============================================================================
// 1. フレッシュ初回起動: 自己展開 + require 解決（Req 1.3 / 5.5 / 6.3）
// ============================================================================

#[test]
fn fresh_first_boot_self_deploys_and_require_resolves() {
    let temp = fresh_ghost_copy();
    let base = temp.path();
    let target = base.join(SELF_DEPLOY_REL);

    // 事前条件: 自己展開先はロード前には存在しない（フレッシュ初回起動）。
    assert!(
        !target.exists(),
        "precondition: self-deploy target must NOT exist before load: {}",
        target.display()
    );

    // 本物のローダーで駆動（Phase 2.5 が自己展開を実施するはず）。
    let runtime = PastaLoader::load(base).expect("load fresh ghost");

    // 自己展開先が生成され、既知の内蔵ファイルとマーカーが存在すること（Req 1.3 / 5.5）。
    assert!(
        target.exists(),
        "self-deploy target must exist after load: {}",
        target.display()
    );
    assert!(
        target.join("main.lua").exists(),
        "known embedded file main.lua must be deployed under {}",
        target.display()
    );
    // フレームワークの実在サブモジュール（pasta/init.lua → module 'pasta'）も展開されること。
    assert!(
        target.join("pasta/init.lua").exists(),
        "known embedded submodule pasta/init.lua must be deployed under {}",
        target.display()
    );
    let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read .md5 marker");
    assert!(
        !marker.trim().is_empty(),
        ".md5 marker must be written with a digest"
    );

    // package.path に自己展開先が含まれること（Req 6.3）。
    let pkg_path = runtime
        .exec("return package.path")
        .expect("exec package.path");
    let pkg_path = value_as_str(&pkg_path).expect("package.path to string");
    assert!(
        pkg_path.contains("profile/pasta/pasta_scripts")
            || pkg_path.contains("profile\\pasta\\pasta_scripts"),
        "package.path must include the self-deploy dir, got: {pkg_path}"
    );

    // フレームワークのコアモジュール（pasta: pasta/init.lua）が
    // 自己展開先から require で解決できること（Req 6.3 / 5.5）。
    // 注: 旧 probe の `dkjson` は luasocket デバッグ資産撤去（pasta-vscode-lua-debug R8）で
    // 内蔵 zip から除去されたため、常設のフレームワークコア `pasta` を probe に用いる。
    let resolved = runtime
        .exec("return (require('pasta') ~= nil)")
        .expect("exec require pasta");
    assert_eq!(
        resolved.as_boolean(),
        Some(true),
        "framework module 'pasta' must resolve via require after self-deploy"
    );

    // searchpath が確かに自己展開先を指していること（解決位置の裏取り）。
    let where_resolved = runtime
        .exec("return package.searchpath('pasta', package.path)")
        .expect("exec searchpath pasta");
    let where_resolved = value_as_str(&where_resolved).expect("searchpath result to string");
    assert!(
        where_resolved.contains("profile/pasta/pasta_scripts")
            || where_resolved.contains("profile\\pasta\\pasta_scripts"),
        "pasta must resolve under the self-deploy dir, got: {where_resolved}"
    );
}

// ============================================================================
// 2. scripts/ 上書きが自己展開先より優先（Req 6.2 / 6.5）
// ============================================================================

#[test]
fn scripts_override_takes_priority() {
    let temp = fresh_ghost_copy();
    let base = temp.path();

    // フレームワークも提供する単純モジュール名 'dkjson' を scripts/ 側でセンチネル上書き。
    // pasta.toml の lua_search_paths では "scripts" が "profile/pasta/pasta_scripts" より
    // 先（高優先）に並ぶため、ユーザー層が勝つはず。
    let scripts = base.join("scripts");
    std::fs::create_dir_all(&scripts).expect("mkdir scripts");
    std::fs::write(
        scripts.join("dkjson.lua"),
        b"return { SOURCE = \"user_scripts_override\" }\n",
    )
    .expect("write user override");

    let runtime = PastaLoader::load(base).expect("load ghost with scripts override");

    // require が scripts/ のセンチネルを返すこと（= scripts/ が自己展開先より優先）。
    let source = runtime
        .exec("return require('dkjson').SOURCE")
        .expect("exec require dkjson.SOURCE");
    let source = value_as_str(&source).expect("SOURCE to string");
    assert_eq!(
        source, "user_scripts_override",
        "scripts/ override must win over the self-deployed framework copy"
    );

    // searchpath も scripts/ を先に解決すること（順序の裏取り）。
    let where_resolved = runtime
        .exec("return package.searchpath('dkjson', package.path)")
        .expect("exec searchpath dkjson");
    let where_resolved = value_as_str(&where_resolved).expect("searchpath to string");
    assert!(
        where_resolved.contains(&format!("scripts{}dkjson", std::path::MAIN_SEPARATOR))
            || where_resolved.replace('\\', "/").contains("scripts/dkjson"),
        "dkjson must resolve under scripts/ (higher priority), got: {where_resolved}"
    );

    // SHIORI 起動挙動が不変であること（Req 6.5）: ロード成功し runtime が使用可能。
    let ok = runtime.exec("return 1+1").expect("exec arithmetic");
    assert_eq!(
        ok.as_i64(),
        Some(2),
        "runtime must remain usable; override must not break startup"
    );
}

// ============================================================================
// 3. 自己展開先が書込不可でも ERROR ログを出して起動継続（Req 3.1 / 3.2）
// ============================================================================
//
// Req 3.2 の「既存スクリプトで起動継続」を忠実に再現するため、まず一度フレッシュに
// ロードして**実際に動く**自己展開先を生成する（= 既存の正常な版が存在する状態）。
// その後 `.md5` を stale に書き換えて再展開を強制し、live 内のファイルへ排他
// （no-share）ハンドルを開いたまま再ロードする。production の swap 手順
// `rename(live → backup)` が共有違反で失敗し、Phase 2.5 の自己展開は Err になるが、
// 既存の（ロック中の）正常な版がそのまま使われるため起動は継続しなければならない。
//
// 注: 真にフレッシュ（自己展開先が空）な状態で自己展開が失敗すると、フレームワーク
// スクリプトがどこにも存在せず `require('pasta')` が解決できないため、起動継続は
// 成立しない。これは「既存スクリプトで継続（version drift unresolved）」という Req 3.2
// の前提どおりの挙動であり、本テストはその前提（既存の正常な版あり）を満たす。
#[cfg(windows)]
#[test]
#[tracing_test::traced_test]
fn write_failure_logs_error_and_continues() {
    use std::os::windows::fs::OpenOptionsExt;

    let temp = fresh_ghost_copy();
    let base = temp.path();
    let target = base.join(SELF_DEPLOY_REL);

    // --- 準備: 一度フレッシュにロードして「実際に動く」既存の自己展開先を作る ---
    {
        let runtime =
            PastaLoader::load(base).expect("initial fresh load to establish live scripts");
        // 既存版が機能していること（フレームワークが解決する）を軽く確認。
        let ok = runtime
            .exec("return (require('pasta') ~= nil)")
            .expect("exec require");
        assert_eq!(
            ok.as_boolean(),
            Some(true),
            "framework must resolve from initial deploy"
        );
    }
    assert!(
        target.join("pasta/init.lua").exists(),
        "precondition: working framework scripts must exist in live dir after initial deploy"
    );

    // --- .md5 を stale に書き換えて再展開を強制し、live 内ファイルをロックして swap を失敗させる ---
    const STALE: &str = "stale";
    std::fs::write(target.join(MARKER_NAME), STALE).expect("overwrite marker with stale digest");
    // 既存の実ファイル main.lua をロック対象にする（live を rename できなくする）。
    let locked = target.join("main.lua");
    let locked_before = std::fs::read(&locked).expect("read locked file before");

    // live 内ファイルへ排他ハンドル（share_mode 0）を開いたまま保持する。
    // これにより rename(live → backup) が共有違反で失敗し、自己展開は Err になる。
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&locked)
        .expect("open exclusive handle inside live dir");

    // 自己展開が失敗しても、既存スクリプトで起動は継続（load は Ok）しなければならない（Req 3.2）。
    let runtime = PastaLoader::load(base).expect("load must continue despite self-deploy failure");

    // 起動継続の裏取り: runtime が使用可能で、既存フレームワークが依然解決すること。
    let ok = runtime.exec("return 1+1").expect("exec after continue");
    assert_eq!(
        ok.as_i64(),
        Some(2),
        "runtime must be usable after non-fatal self-deploy failure"
    );
    let resolved = runtime
        .exec("return (require('pasta') ~= nil)")
        .expect("exec require after continue");
    assert_eq!(
        resolved.as_boolean(),
        Some(true),
        "existing framework must still resolve after failed self-deploy (continue with existing)"
    );

    // 自己展開失敗の ERROR ログが出ていること（Req 3.1）。
    assert!(
        logs_contain("Self-deploy failed"),
        "an ERROR about self-deploy failure must be logged (Req 3.1)"
    );

    // 直前の live 状態が保全されていること（マーカーは stale のまま = 書き換わっていない）。
    let marker = std::fs::read_to_string(target.join(MARKER_NAME)).expect("read marker");
    assert_eq!(
        marker.trim(),
        STALE,
        "stale .md5 must remain unchanged on failed deploy (prior state preserved)"
    );
    assert!(
        locked.exists(),
        "locked file must still exist after failed deploy (prior live preserved)"
    );

    // ハンドルは load より長生きさせる（共有違反を成立させるため）。最後に解放。
    drop(lock);
    assert_eq!(
        std::fs::read(&locked).expect("read locked after drop"),
        locked_before,
        "locked file content must survive failed deploy unchanged"
    );
}
