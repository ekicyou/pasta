//! scene-count の結合テスト。一時ゴーストを `PastaLoader::load` で実物のランタイムとして起こし、
//! `@pasta_search` の `count_scene`・`count_scene_exact` の数え方を確かめる
//! （requirements 1.1〜1.6・2.1〜2.3・5.1〜5.3・8.1）。
//! `mod common` の宣言により `PASTA_DEBUG` 中和ガード（`#[ctor]`）が適用される。

mod common;

use common::{create_temp_with_pasta, value_as_str};
use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use std::collections::HashSet;
use tempfile::TempDir;
use tracing_test::traced_test;

/// Introduction の辞書: `＊雑談` 3 本（1 本は `＆` の属性つき）・`＊雑談_朝` 2 本（1 本は `・` で書く）、
/// ローカルシーン `・雑談` を持つシーン、`＞雑談` を呼ぶシーン。
const INTRO_PASTA: &str = "\
＊雑談
  さくら：雑談一

＊雑談
  さくら：雑談二

＊雑談＆季節：夏
  ＆時間：夜
  さくら：雑談三

＊雑談_朝
  さくら：朝一

＊雑談・朝
  さくら：朝二

＊親
  さくら：親です

  ・雑談
    さくら：ローカルの雑談です

＊呼ぶ
  ＞雑談
";

/// Lua から登録するグローバルシーン 2 本。
const INTRO_MAIN_LUA: &str = r#"local SCENE = require "pasta.scene"
SCENE.create_scene("Lua話題", "__start__", function(act) end)
SCENE.create_scene("Lua話題", "__start__", function(act) end)
return {}
"#;

/// 既定の別名（`会話` → `OnTalk`）を確かめる辞書。`＊会話` は `OnTalk_1` として登録される。
const ALIAS_PASTA: &str = "\
＊会話
  さくら：会話です

＊OnTalk朝
  さくら：OnTalk朝です

＊会話・朝
  さくら：朝の会話です
";

/// 一時ゴーストを作って実物のランタイムを読み込む。`TempDir` はランタイムより長く保持すること。
fn load_ghost(pasta: &str, main_lua: Option<&str>) -> (TempDir, PastaLuaRuntime) {
    let temp = create_temp_with_pasta(pasta);
    let toml = temp.path().join("pasta.toml");
    let base = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(&toml, format!("{base}[actor.\"さくら\"]\nspot = 0\n")).unwrap();
    if let Some(main_lua) = main_lua {
        let scripts = temp.path().join("scripts");
        std::fs::create_dir_all(&scripts).unwrap();
        std::fs::write(scripts.join("main.lua"), main_lua).unwrap();
    }
    let runtime = PastaLoader::load(temp.path()).expect("ゴーストの読み込みに失敗");
    (temp, runtime)
}

/// `SEARCH:<method>(name)` の戻り（数）。
fn count(runtime: &PastaLuaRuntime, method: &str, name: &str) -> i64 {
    runtime
        .lua()
        .load(format!(
            r#"local SEARCH = require "@pasta_search"
return SEARCH:{method}(...)"#
        ))
        .call::<i64>(name)
        .unwrap_or_else(|e| panic!("{method}({name:?}) に失敗: {e}"))
}

/// `SEARCH:<method>(name)`（範囲省略の `search_scene`・`search_scene_exact`）を `n` 回回して、
/// 出た登録名の種類を返す。
fn searched_ids(runtime: &PastaLuaRuntime, method: &str, name: &str, n: i64) -> HashSet<String> {
    (0..n)
        .map(|_| {
            runtime
                .lua()
                .load(format!(
                    r#"local SEARCH = require "@pasta_search"
return (SEARCH:{method}(...))"#
                ))
                .call::<Option<String>>(name)
                .unwrap_or_else(|e| panic!("{method}({name:?}) に失敗: {e}"))
                .unwrap_or_else(|| panic!("{method}({name:?}) が見つからない"))
        })
        .collect()
}

/// `EVENT.fire` でイベント（未登録ならイベント名と同じ名前のシーン）を起こし、応答文字列を返す。
fn fire(runtime: &PastaLuaRuntime, id: &str) -> String {
    let script = format!(
        r#"local EVENT = require "pasta.shiori.event"
return EVENT.fire({{ id = "{id}", method = "get", version = 30 }})"#
    );
    let value = runtime.exec(&script).expect("EVENT.fire に失敗");
    value_as_str(&value).expect("応答が文字列でない")
}

/// 1.1・2.1・1.3・1.5・5.3: 前方一致 5・完全一致 3・名前が続くシーン 2。ローカルシーン `・雑談` は数えず、
/// `＆` の属性を書いたシーンも数に入り、`雑談・朝` は照合用の名前 `雑談_朝` に揃って数えられる。
#[test]
fn counts_prefix_and_exact_global_candidates() {
    let (_temp, runtime) = load_ghost(INTRO_PASTA, None);
    assert_eq!(count(&runtime, "count_scene", "雑談"), 5);
    assert_eq!(count(&runtime, "count_scene_exact", "雑談"), 3);
    assert_eq!(count(&runtime, "count_scene", "雑談_朝"), 2);
    assert_eq!(count(&runtime, "count_scene_exact", "雑談_朝"), 2);
    assert_eq!(count(&runtime, "count_scene", "雑談・朝"), 2);
    assert_eq!(count(&runtime, "count_scene_exact", "雑談・朝"), 2);
}

/// 1.6・2.3: 無い名前は前方一致・完全一致とも 0 で、警告が出ない（別名を当てて見つからないときも）。
#[traced_test]
#[test]
fn missing_name_counts_zero_without_warning() {
    // 読み込みの間の WARN（保存ファイルが無い等）は数えず、数えた後に増えないことを見る
    let warn_lines = || {
        let n = std::cell::Cell::new(0);
        logs_assert(|lines| {
            n.set(lines.iter().filter(|l| l.contains(" WARN ")).count());
            Ok(())
        });
        n.get()
    };
    let (_temp, runtime) = load_ghost(INTRO_PASTA, None);
    let before = warn_lines();
    assert_eq!(count(&runtime, "count_scene", "無い名前"), 0);
    assert_eq!(count(&runtime, "count_scene_exact", "無い名前"), 0);
    // `雑談一` は台詞で、シーン名ではない
    assert_eq!(count(&runtime, "count_scene", "雑談一"), 0);
    // 別名を当てて（会話 → OnTalk）見つからなくても警告しない（検索なら "Scene not found" が出る）
    assert_eq!(count(&runtime, "count_scene", "会話"), 0);
    assert_eq!(count(&runtime, "count_scene_exact", "会話"), 0);
    assert_eq!(warn_lines(), before);
    assert!(!logs_contain("Scene not found"));
}

/// 3.7 の前提・1.1: `main.lua` で Lua から登録したグローバルシーンも数える。
#[test]
fn counts_scenes_registered_from_lua() {
    let (_temp, runtime) = load_ghost(INTRO_PASTA, Some(INTRO_MAIN_LUA));
    assert_eq!(count(&runtime, "count_scene", "Lua話題"), 2);
    assert_eq!(count(&runtime, "count_scene_exact", "Lua話題"), 2);
    assert_eq!(count(&runtime, "count_scene", "雑談"), 5);
}

/// 1.4・2.3: 既定の別名 `会話` → `OnTalk`。前方一致は `＊OnTalk朝` を入れて `＊会話・朝` を入れず、
/// 完全一致は `＊OnTalk朝` を入れない（`OnTalk` と同じ数）。`会話・朝` は揃えた `会話_朝` で数える。
#[test]
fn alias_is_applied_before_counting() {
    let (_temp, runtime) = load_ghost(ALIAS_PASTA, None);
    assert_eq!(count(&runtime, "count_scene", "会話"), 2);
    assert_eq!(count(&runtime, "count_scene_exact", "会話"), 1);
    assert_eq!(count(&runtime, "count_scene", "OnTalk"), 2);
    assert_eq!(count(&runtime, "count_scene_exact", "OnTalk"), 1);
    assert_eq!(count(&runtime, "count_scene", "会話・朝"), 1);
    assert_eq!(count(&runtime, "count_scene_exact", "会話・朝"), 1);
}

/// 1.2・2.2: 検索を候補の数だけ回して出た登録名の種類の数が、数えた数と等しい（巡回は重ならない）。
#[test]
fn count_equals_distinct_search_candidates() {
    let cases = [
        (INTRO_PASTA, "雑談"),
        (INTRO_PASTA, "雑談・朝"),
        (ALIAS_PASTA, "会話"),
        (ALIAS_PASTA, "会話・朝"),
    ];
    for (pasta, name) in cases {
        let (_temp, runtime) = load_ghost(pasta, None);
        for (count_method, search_method) in [
            ("count_scene", "search_scene"),
            ("count_scene_exact", "search_scene_exact"),
        ] {
            let n = count(&runtime, count_method, name);
            assert!(n > 0, "{count_method}({name:?})");
            let ids = searched_ids(&runtime, search_method, name, n);
            assert_eq!(
                ids.len() as i64,
                n,
                "{search_method}({name:?}) を {n} 回: {ids:?}"
            );
        }
    }
}

/// 5.1・5.2: 同じ選び手の並びを与えた 2 つのランタイムで、片方だけ `＞雑談` の間に数を挟んでも、
/// 選ばれた並びが同じ（2 巡ぶん）。
#[test]
fn counting_does_not_change_next_selection() {
    let (_ta, plain) = load_ghost(INTRO_PASTA, None);
    let (_tb, counted) = load_ghost(INTRO_PASTA, None);
    for runtime in [&plain, &counted] {
        runtime
            .exec(r#"require("@pasta_search"):set_scene_selector(3, 1, 4, 0, 2)"#)
            .unwrap();
    }
    let mut a = Vec::new();
    let mut b = Vec::new();
    for _ in 0..10 {
        a.push(fire(&plain, "呼ぶ"));
        count(&counted, "count_scene", "雑談");
        count(&counted, "count_scene_exact", "雑談");
        b.push(fire(&counted, "呼ぶ"));
        count(&counted, "count_scene", "雑談_朝");
    }
    assert_eq!(a, b);
    // 1 巡で 5 本すべてが選ばれている（並びが本当に巡回している）
    let first_round: HashSet<_> = a[..5].iter().collect();
    assert_eq!(first_round.len(), 5, "{a:?}");
}
