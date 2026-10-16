//! 見本のゴーストのつつき・撫で（touch-poke-shortcuts）。
//!
//! 12 段目の辞書（＝配布辞書 `dic/` の全部）を tempdir へコピーして実ローダーで読み込み、
//! `OnMouseDoubleClick`・`OnMouseMove` を送って、呼ばれたシーンを各シーンに固有の
//! 台詞の一部が応答に含まれるかで確かめる（Req 9.6）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;

/// `tutorial_stages_test.rs` と同じく、環境の PASTA_DEBUG で固定ポートがぶつからないようにする
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

// 各シーンに固有の台詞の一部（08-touch.pasta・09-choice.pasta）。読点の後ろにはウェイトが入るので、読点をまたがない
const POKE_GIRL_HEAD: &str = "頭をつつかれると";
const POKE_BOY_OTHER: &str = "つついた？";
const POKE_FACE: &str = "ほっぺたは";
const POKE_OTHER: &str = "なにか用事？";
const POKE_CHOICE: &str = "\\q[";
const TOUCH_GIRL_HEAD: &str = "なでなでされちゃった";
const TOUCH_OTHER: &str = "くすぐったいってば";

/// 撫での時刻の起点（任意の unix 秒）
const T0: f64 = 1_700_000_000.0;

/// master/ を（profile/ と pasta.dll を除いて）tempdir へ再帰コピーする
fn copy_master(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create dst dir");
    for entry in std::fs::read_dir(src).expect("read src dir").flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if entry.metadata().expect("metadata").is_dir() {
            if name != "profile" {
                copy_master(&path, &dst.join(&name));
            }
        } else if name != "pasta.dll" {
            std::fs::copy(&path, dst.join(&name)).expect("copy file");
        }
    }
}

/// 配布辞書のゴーストを読み込む（TempDir は runtime より長く生かす）
fn load_ghost() -> (TempDir, PastaLuaRuntime) {
    let master = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ghosts/hello-pasta/ghost/master");
    let temp = TempDir::new().expect("tempdir");
    copy_master(&master, temp.path());
    let runtime = PastaLoader::load(temp.path()).expect("load hello-pasta");
    (temp, runtime)
}

/// マウスのイベントを SHIORI.request へ送り、応答文字列を返す。
/// `time` があれば、パース済みのリクエストと同じ形の `date`（unix 秒＋ナノ秒）を付ける。
fn mouse(
    runtime: &PastaLuaRuntime,
    id: &str,
    scope: &str,
    part: &str,
    time: Option<f64>,
) -> String {
    let date = time
        .map(|t| {
            format!(
                "date = {{ unix = {}, ns = {} }}, ",
                t.trunc() as i64,
                (t.fract() * 1e9).round() as i64
            )
        })
        .unwrap_or_default();
    let script = format!(
        r#"local SHIORI = require("pasta.shiori.entry")
return SHIORI.request({{ id = "{id}", method = "get", version = 30, charset = "UTF-8",
    sender = "SSP", {date}reference = {{ [0] = "100", [1] = "100", [2] = "0", [3] = "{scope}", [4] = "{part}" }}, dic = {{}} }})"#
    );
    let v = runtime
        .exec(&script)
        .unwrap_or_else(|e| panic!("{id} {scope}/{part}: Lua error: {e}"));
    v.as_string()
        .and_then(|s| s.to_str().ok().map(|s| s.to_string()))
        .unwrap_or_else(|| panic!("{id}: SHIORI.request が文字列を返さない: {v:?}"))
}

fn poke(runtime: &PastaLuaRuntime, scope: &str, part: &str) -> String {
    mouse(runtime, "OnMouseDoubleClick", scope, part, None)
}

fn assert_has(response: &str, line: &str, what: &str) {
    assert!(
        response.starts_with("SHIORI/3.0 200 OK") && response.contains(line),
        "{what}: `{line}` を含む 200 を期待した:\n{response}"
    );
}

/// 応答がどちらの台詞か（どちらでもなければ panic）
fn other_or_choice(response: &str, what: &str) -> &'static str {
    match (
        response.contains(POKE_OTHER),
        response.contains(POKE_CHOICE),
    ) {
        (true, false) => "その他",
        (false, true) => "選択肢",
        _ => panic!("{what}: その他 か 選択肢 のどちらか一方を期待した:\n{response}"),
    }
}

#[test]
fn poke_calls_shortcut_scenes() {
    let (_temp, rt) = load_ghost();

    assert_has(&poke(&rt, "0", "Head"), POKE_GIRL_HEAD, "女の子の頭");
    assert_has(&poke(&rt, "1", "Head"), POKE_BOY_OTHER, "男の子の頭");
    assert_has(&poke(&rt, "0", "Face"), POKE_FACE, "女の子の顔");

    // 8 段目と 9 段目の ＊つっつき反応その他 は、2 回で 1 回ずつ選ばれる
    let picked: BTreeSet<_> = (0..2)
        .map(|i| other_or_choice(&poke(&rt, "0", "Body"), &format!("女の子の体 {i} 回目")))
        .collect();
    assert_eq!(picked, BTreeSet::from(["その他", "選択肢"]));

    // 部位なし（当たり判定の外）
    other_or_choice(&poke(&rt, "0", ""), "部位なし");
}

#[test]
fn touch_for_two_seconds_calls_stroke_scenes() {
    let (_temp, rt) = load_ghost();

    // 女の子の頭: 2 秒までは 204、2 秒で撫でのシーン
    for dt in [0.0, 0.5, 1.0, 1.5, 1.9] {
        let r = mouse(&rt, "OnMouseMove", "0", "Head", Some(T0 + dt));
        assert!(
            r.starts_with("SHIORI/3.0 204 No Content"),
            "女の子の頭 +{dt}s: 204 を期待した:\n{r}"
        );
    }
    let r = mouse(&rt, "OnMouseMove", "0", "Head", Some(T0 + 2.0));
    assert_has(&r, TOUCH_GIRL_HEAD, "女の子の頭 +2s");

    // 男の子の体: さわり反応その他（相手を替えたので数え直す）
    let t1 = T0 + 10.0;
    for dt in [0.0, 1.0] {
        let r = mouse(&rt, "OnMouseMove", "1", "Body", Some(t1 + dt));
        assert!(
            r.starts_with("SHIORI/3.0 204 No Content"),
            "男の子の体 +{dt}s: 204 を期待した:\n{r}"
        );
    }
    let r = mouse(&rt, "OnMouseMove", "1", "Body", Some(t1 + 2.0));
    assert_has(&r, TOUCH_OTHER, "男の子の体 +2s");
}
