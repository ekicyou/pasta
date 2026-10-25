//! 版 3 の入口の試験（仕様 `shiori4-v3-portable`・1.2〜1.4・2.1〜2.5・2.7・3.1〜3.7・3.9・3.10・4.1・6.2）。
//!
//! `shiori_get_api(3)` のテーブルを `shiori4_abi` の `ShioriApi::load_v3` と `ShioriInstance` で呼び、
//! ABI のテーブルを通して版 3 の口を確かめる。ゴーストは試験ごとに一時ディレクトリへ写す。
//!
//! 旧来の窓口（`lifecycle`）はプロセスに 1 つの置き場を使うので、この試験の中で旧来の窓口を
//! 呼ぶのは 1 つの `#[test]` だけにする（ほかの試験とは別のディレクトリを使い、置き場を取り合わない）。

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use common::copy_fixture_to_temp;
use pasta::actor::lifecycle;
use pasta::shiori_get_api;
use pasta_lua::GlobalLoggerRegistry;
use shiori4_abi::{SHIORI_ABI_VERSION_3, ShioriApi, ShioriInstance};

fn get(id: &str) -> String {
    format!("GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: {id}\r\n\r\n")
}

fn notify(id: &str) -> String {
    format!("NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nID: {id}\r\n\r\n")
}

/// 版 3 のテーブル（検証を通したもの）。
fn api() -> &'static ShioriApi {
    // SAFETY: shiori_get_api はこの試験のバイナリに静的にリンクされ、テーブルは static。
    unsafe { ShioriApi::from_raw(shiori_get_api(SHIORI_ABI_VERSION_3), SHIORI_ABI_VERSION_3) }
        .expect("v3 table must pass validation")
}

fn load(dir: &Path) -> Option<ShioriInstance<'static>> {
    api().load_v3(dir.to_str().expect("utf-8 path"))
}

/// 版 3 の `request` の戻り値と本文。
fn request(inst: &mut ShioriInstance<'_>, req: &str) -> (i32, String) {
    let res = inst.request(req).expect("no ABI error");
    let body = String::from_utf8(res.body.to_vec()).expect("utf-8 body");
    (res.status, body)
}

/// 登録にそのディレクトリの項目が無いこと（占有できれば無い。確かめた札はすぐ返す）。
fn assert_no_registry_entry(dir: &Path) {
    let registry = GlobalLoggerRegistry::instance();
    let token = registry
        .try_claim(dir.to_path_buf())
        .unwrap_or_else(|| panic!("registry must not keep an entry for {}", dir.display()));
    registry.release_claim(&token);
}

/// ログのファイルに `needle` が現れるのを待つ（書き込みは非同期の書き手を通るため）。
fn wait_log_contains(ghost: &Path, needle: &str) -> String {
    let path = ghost.join("profile/pasta/logs/pasta.log");
    let start = Instant::now();
    loop {
        let log = std::fs::read_to_string(&path).unwrap_or_default();
        if log.contains(needle) || start.elapsed() > Duration::from_secs(10) {
            return log;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// 版 3 だけにテーブルを返し、2 回呼んでも同じポインタ（1.2〜1.4）。
#[test]
fn get_api_returns_v3_table_only() {
    let a = shiori_get_api(3);
    assert!(!a.is_null());
    assert_eq!(a, shiori_get_api(3));
    assert_eq!(api().abi_version(), 3);
    for v in [4, 0, 2, 5] {
        assert!(shiori_get_api(v).is_null(), "version {v} must be NULL");
    }
}

/// 同じゴーストに旧来の窓口と版 3 で同じ GET を送ると、応答のバイト列が同じで、
/// 版 3 の戻り値は開始行のコード（3.1・3.9・3.10）。旧来を片付けてから版 3 を読む。
#[test]
fn v3_get_matches_legacy_bytes_and_returns_start_line_code() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let dir = ghost.path().to_path_buf();

    assert!(lifecycle::spawn_actor(0, dir.clone()), "v3_probe must load");
    let legacy = lifecycle::marshal_request(&get("OnProbeCount"));
    assert!(lifecycle::teardown_actor().acked);

    let mut inst = load(&dir).expect("v3 load after legacy unload");
    let (status, body) = request(&mut inst, &get("OnProbeCount"));
    assert_eq!(body.as_bytes(), legacy.as_bytes());
    assert!(body.starts_with("SHIORI/3.0 200 OK\r\n"), "{body:?}");
    assert_eq!(status, 200);
}

/// NOTIFY は 204。notify への GET・2.x・読めない要求は 400 で、Lua には届かない（3.2〜3.6）。
#[test]
fn notify_is_204_and_rejected_requests_are_400_and_not_counted() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let mut inst = load(ghost.path()).expect("v3 load");

    // Lua に届く: 1 と 2
    let (status, body) = request(&mut inst, &notify("OnSomething"));
    assert_eq!(status, 204);
    assert!(
        body.starts_with("SHIORI/3.0 204 No Content\r\n"),
        "{body:?}"
    );
    assert!(body.ends_with("\r\n\r\n"), "{body:?}");
    assert_eq!(inst.notify(&notify("OnSomething")), Ok(204));

    // 弾かれる（Lua に届かない）
    assert_eq!(inst.notify(&get("OnProbeCount")), Ok(400));
    assert_eq!(
        inst.notify("NOTIFY OwnerGhostName SHIORI/2.0\r\nCharset: UTF-8\r\n\r\n"),
        Ok(400)
    );
    assert_eq!(inst.notify("garbage"), Ok(400));
    for req in [
        "GET Sentence SHIORI/2.6\r\nCharset: UTF-8\r\n\r\n",
        "PUT SHIORI/3.0\r\nCharset: UTF-8\r\n\r\n",
        "",
    ] {
        let (status, body) = request(&mut inst, req);
        assert_eq!(status, 400, "request {req:?}");
        assert!(
            body.starts_with("SHIORI/3.0 400 Bad Request\r\n") && body.contains("X-ERROR-REASON: "),
            "request {req:?}: {body:?}"
        );
    }

    // この GET が 3 つ目
    let (status, body) = request(&mut inst, &get("OnProbeCount"));
    assert_eq!(status, 200);
    assert!(
        body.contains("\r\nValue: 3\r\n"),
        "rejected requests must not reach Lua: {body:?}"
    );
}

/// 読めない開始行は戻り値 500、本文は Lua が返したまま（3.7）。
#[test]
fn unreadable_start_line_is_500_with_body_unchanged() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let mut inst = load(ghost.path()).expect("v3 load");

    let (status, body) = request(&mut inst, &get("OnProbeBadStatus"));
    assert_eq!(status, 500);
    assert_eq!(
        body,
        "SHIORI/3.0 2x0 OK\r\nCharset: UTF-8\r\nValue: broken\r\n\r\n"
    );
}

/// 無いディレクトリ・相対パスは NULL で、登録に項目を残さない（2.2・2.4）。
#[test]
fn missing_or_relative_dir_is_null_without_registry_entry() {
    let root = tempfile::TempDir::new().expect("temp dir");
    let missing = root.path().join("no_such_ghost");
    assert!(load(&missing).is_none());
    assert_no_registry_entry(&missing);

    let relative = Path::new("relative/ghost");
    assert!(api().load_v3("relative/ghost").is_none());
    assert_no_registry_entry(relative);
}

/// 辞書に書き間違いのあるゴーストでもインスタンスは返り、GET は 500＋理由のヘッダ（2.1・2.3・2.7）。
#[test]
fn dictionary_typo_returns_instance_and_get_is_500_with_reason() {
    let ghost = copy_fixture_to_temp("shiori_lifecycle");
    std::fs::write(
        ghost.path().join("dic/test/typo.pasta"),
        "＊壊れたシーン\n  ゴースト：「閉じていない\n＄＝＝\n",
    )
    .expect("write typo dictionary");

    let mut inst = load(ghost.path()).expect("instance must be returned despite the typo");
    let (status, body) = request(&mut inst, &get("version"));
    assert_eq!(status, 500, "{body:?}");
    assert!(
        body.starts_with("SHIORI/3.0 500 Internal Server Error\r\n")
            && body.contains("\r\nX-ERROR-REASON: ")
            && body.contains("typo.pasta"),
        "{body:?}"
    );
}

/// 同じディレクトリの 2 つ目は NULL で、占有している側のログに理由が出る（6.2・2.5）。
/// unload の直後に登録から項目が消え、同じディレクトリをすぐ読み直せる（4.1）。
#[test]
fn second_load_is_null_and_unload_releases_dir() {
    let ghost = copy_fixture_to_temp("v3_probe");
    let dir = ghost.path();

    let mut first = load(dir).expect("first load");
    assert!(
        load(dir).is_none(),
        "second instance on the same dir must be NULL"
    );
    let log = wait_log_contains(dir, "instance.claim");
    assert!(
        log.contains("instance.claim"),
        "reason must be logged:\n{log}"
    );
    assert_eq!(request(&mut first, &get("OnProbeCount")).0, 200);

    drop(first);
    assert_no_registry_entry(dir);
    let mut again = load(dir).expect("reload right after unload");
    assert_eq!(request(&mut again, &get("OnProbeCount")).0, 200);
}
